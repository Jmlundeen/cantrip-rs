using Cantrip.Server.Events;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Host;

/// <summary>Opens a table for an event - the pod's own, or one of its games - and says where it is.</summary>
public delegate Task<(GameServer Server, Uri Address)> TableOpener(ITableEvent table, int seats);

/// <summary>
/// Hosts a <see cref="BoosterPod"/>: its own table, where its players draft, build and see the pairings, and a table
/// for every game of every match, opened when the pod wants one and closed once its players have gone on. Where the
/// tables are is the opener's business - a port of its own on this machine, or a port from a host's pool behind a
/// public address.
/// </summary>
public sealed class PodHost : IAsyncDisposable
{
    /// <summary>How long a game's table stays up after its players are sent on, so the message saying where reaches them.</summary>
    private static readonly TimeSpan Linger = TimeSpan.FromSeconds(5);

    private readonly BoosterPod _pod;
    private readonly TableOpener _open;
    private readonly Dictionary<string, GameServer> _tables = new(StringComparer.Ordinal);
    private readonly List<(DateTime At, GameServer Server)> _closing = [];
    private readonly CancellationTokenSource _stop = new();
    private Task _loop = Task.CompletedTask;

    private PodHost(BoosterPod pod, TableOpener open, GameServer table, Uri address)
    {
        _pod = pod;
        _open = open;
        Table = table;
        Address = address;
    }

    public BoosterPod Pod => _pod;

    /// <summary>The pod's own table.</summary>
    public GameServer Table { get; }

    /// <summary>Where the pod's own table is: what a player's ticket is for.</summary>
    public Uri Address { get; }

    /// <summary>How many games' tables are open now.</summary>
    public int GamesOpen
    {
        get { lock (_tables) return _tables.Count; }
    }

    /// <summary>A fault while opening or closing tables, if one stopped the host; null while it runs.</summary>
    public Exception? Fault { get; private set; }

    public static async Task<PodHost> StartAsync(BoosterPod pod, TableOpener open)
    {
        ArgumentNullException.ThrowIfNull(pod);
        ArgumentNullException.ThrowIfNull(open);
        (GameServer table, Uri address) = await open(pod, pod.Seats).ConfigureAwait(false);
        pod.Hosted(address);
        var host = new PodHost(pod, open, table, address);
        host._loop = Task.Run(host.RunAsync, CancellationToken.None);
        return host;
    }

    /// <summary>
    /// Hosts a pod whose own table is already open - a table that formed from a queue and became the pod's when everyone
    /// was ready - opening only its games' tables.
    /// </summary>
    public static PodHost Adopt(BoosterPod pod, GameServer table, Uri address, TableOpener open)
    {
        ArgumentNullException.ThrowIfNull(pod);
        ArgumentNullException.ThrowIfNull(table);
        ArgumentNullException.ThrowIfNull(address);
        ArgumentNullException.ThrowIfNull(open);
        pod.Hosted(address);
        var host = new PodHost(pod, open, table, address);
        host._loop = Task.Run(host.RunAsync, CancellationToken.None);
        return host;
    }

    /// <summary>
    /// Opens each table on this machine at a port of its own, on loopback: for a pod hosted locally, and for tests.
    /// <paramref name="portraits"/> are the players' pictures (<see cref="GameServerOptions.Portraits"/>), and
    /// <paramref name="art"/> where card art is (<see cref="GameServerOptions.Art"/>), the same for every table, and
    /// <paramref name="rules"/> what every game is dealt through (<see cref="GameServerOptions.Rules"/>).
    /// </summary>
    public static TableOpener Loopback(string contentRoot, TimeProvider? time = null, string listen = "127.0.0.1", Func<string, string?>? portraits = null, Uri? art = null, IRulesHostFactory? rules = null) => async (table, seats) =>
    {
        GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0, Players = seats, ContentRoot = contentRoot, Event = table, TimeProvider = time ?? TimeProvider.System, ListenAddress = listen,
            Portraits = portraits, Art = art, Rules = rules ?? RulesHostFactory.Native,
        }).ConfigureAwait(false);
        return (server, new Uri($"ws://{(listen == "0.0.0.0" ? "localhost" : listen)}:{server.Port}/game"));
    };

    private async Task RunAsync()
    {
        while (!_stop.IsCancellationRequested)
        {
            try
            {
                foreach (MatchGame game in _pod.TablesToOpen())
                {
                    (GameServer server, Uri address) = await _open(game, game.Seats).ConfigureAwait(false);
                    lock (_tables) _tables[game.Key] = server;
                    game.Opened(address);
                }

                foreach (MatchGame game in _pod.TablesDone())
                {
                    GameServer? server;
                    lock (_tables)
                    {
                        _tables.Remove(game.Key, out server);
                    }

                    if (server is not null) _closing.Add((DateTime.UtcNow + Linger, server));
                }

                foreach ((DateTime at, GameServer server) in _closing.Where(each => each.At <= DateTime.UtcNow).ToList())
                {
                    _closing.Remove((at, server));
                    await server.DisposeAsync().ConfigureAwait(false);
                }
            }
            catch (Exception error) when (error is not OperationCanceledException)
            {
                // A table that would not open is tried again; anything else is kept for whoever looks.
                Fault = error;
            }

            try
            {
                await Task.Delay(100, _stop.Token).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }
    }

    public async ValueTask DisposeAsync()
    {
        await _stop.CancelAsync().ConfigureAwait(false);
        await _loop.ConfigureAwait(false);
        List<GameServer> servers;
        lock (_tables)
        {
            servers = [.. _tables.Values, .. _closing.Select(each => each.Server)];
            _tables.Clear();
        }

        _closing.Clear();
        foreach (GameServer server in servers)
        {
            await server.DisposeAsync().ConfigureAwait(false);
        }

        await Table.DisposeAsync().ConfigureAwait(false);
        _stop.Dispose();
    }
}
