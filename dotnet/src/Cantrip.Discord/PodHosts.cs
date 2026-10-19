using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Discord;

public sealed class PodHostsOptions
{
    public required string ContentRoot { get; init; }

    public required EventStore Store { get; init; }

    /// <summary>Each player's picture, by player id, as their tables show it (<see cref="GameServerOptions.Portraits"/>); null for none.</summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>
    /// The ports the pods' tables listen on: the pod's own and one for every game being played. Shared with the other
    /// draft tables - a port in use is passed over - so a pod of eight wants five of them free while its round is played.
    /// </summary>
    public required IReadOnlyList<int> Ports { get; init; }

    public string ListenAddress { get; init; } = "127.0.0.1";

    public string PublicHost { get; init; } = "localhost";

    /// <summary>The address players are given for the table on a port, when not <c>ws://{PublicHost}:{port}/game</c>.</summary>
    public Func<int, Uri>? TableAddress { get; init; }

    /// <summary>Where card art is, said by every table to the clients at it (<see cref="GameServerOptions.Art"/>); null for not said.</summary>
    public Uri? Art { get; init; }

    /// <summary>
    /// The site's page a pod's own table names for its players to go back to (<see cref="GameServerOptions.Return"/>):
    /// from the final standings, or once they have dropped. Its games' tables name none - the pod sends their players on.
    /// </summary>
    public ReturnPage? Return { get; init; }

    /// <summary>How long a finished pod's table stays up for its players to read the final standings.</summary>
    public TimeSpan Linger { get; init; } = TimeSpan.FromMinutes(30);

    public BoosterTiming Timing { get; init; } = new();

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}

/// <summary>What a player gets back for their pod: its table's address and a fresh ticket to their seat, or why not.</summary>
public sealed record PodSeating(Uri? Address, string? Ticket, int Seat, string? Refusal)
{
    public static PodSeating Refused(string why) => new(null, null, 0, why);
}

/// <summary>
/// The booster drafts the bot runs, knowing nothing of Discord itself: each pod's own table and the tables of its games,
/// on the draft ports (<see cref="PodHost"/>). A pod comes from a queue's table once everyone at it is ready, and is
/// hosted again after a restart from the data directory, wherever it had got to. A finished pod's table stays up for a
/// while for its players to read the final standings.
/// </summary>
public sealed class PodHosts : IAsyncDisposable
{
    /// <summary>Which queue a pod came from, kept with the event so the queue's message can count it after a restart.</summary>
    public const string QueueKey = "discord.queue";

    private readonly PodHostsOptions _options;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly Dictionary<string, Hosted> _pods = new(StringComparer.Ordinal);

    private sealed class Hosted(PodHost host, ulong queue)
    {
        public PodHost Host { get; } = host;
        public ulong Queue { get; } = queue;
        public DateTimeOffset? EndedAt { get; set; }

        /// <summary>Calls off the bots playing the seats the queue kept for them (<see cref="BotSeats"/>).</summary>
        public CancellationTokenSource Bots { get; } = new();
    }

    public PodHosts(PodHostsOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        _options = options;
    }

    /// <summary>A pod's stage changed. Raised outside the lock, with the queue it came from (zero for none).</summary>
    public event Func<ulong, Task>? Changed;

    /// <summary>Takes over a queue's table whose pod has just begun: from then on its games are given tables of their own.</summary>
    public async Task AdoptAsync(BoosterPod pod, GameServer table, int port, Uri address, ulong queue)
    {
        ArgumentNullException.ThrowIfNull(pod);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            pod.Keep(QueueKey, queue.ToString(System.Globalization.CultureInfo.InvariantCulture));
            _pods[pod.EventId] = PlayBots(new Hosted(PodHost.Adopt(pod, table, address, Open), queue));
        }
        finally
        {
            _gate.Release();
        }

        await Raise(queue).ConfigureAwait(false);
    }

    /// <summary>
    /// Hosts every live booster draft of the bot's in the data directory again, as it stood: what a restarted bot does.
    /// Only a pod that came from a queue is the bot's; one hosted some other way in the same directory - a pod played
    /// locally - is its own host's, and is left to it.
    /// </summary>
    public async Task<int> ResumeAsync()
    {
        int resumed = 0;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            foreach (EventRecord record in _options.Store.LiveEvents().Where(record => record.Kind == EventKinds.BoosterDraft))
            {
                if (_pods.ContainsKey(record.Id) || !ulong.TryParse(record.HostData.GetValueOrDefault(QueueKey), out ulong queue)) continue;
                try
                {
                    BoosterPod pod = BoosterPod.Open(_options.Store, record.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider, _options.Timing);
                    _pods[record.Id] = PlayBots(new Hosted(await PodHost.StartAsync(pod, Open).ConfigureAwait(false), queue));
                    resumed++;
                }
                catch (Exception error) when (error is IOException or InvalidOperationException or ArgumentException)
                {
                    Console.Error.WriteLine($"Booster draft {record.Id} could not be hosted again yet: {error.Message}");
                }
            }
        }
        finally
        {
            _gate.Release();
        }

        return resumed;
    }

    /// <summary>The pods being hosted and not yet over, by event id.</summary>
    public IReadOnlyList<string> Hosting
    {
        get
        {
            _gate.Wait();
            try { return [.. _pods.Where(pair => pair.Value.EndedAt is null).Select(pair => pair.Key)]; }
            finally { _gate.Release(); }
        }
    }

    /// <summary>How many of a queue's pods are under way.</summary>
    public int Underway(ulong queue)
    {
        _gate.Wait();
        try { return _pods.Values.Count(hosted => hosted.Queue == queue && hosted.EndedAt is null); }
        finally { _gate.Release(); }
    }

    /// <summary>Whether a player is in a pod still being played, and has not dropped from it.</summary>
    public bool InDraft(ulong player)
    {
        _gate.Wait();
        try { return Live(player) is not null; }
        finally { _gate.Release(); }
    }

    /// <summary>A fresh ticket to the pod a player is in, which takes them to wherever in it they should be.</summary>
    public async Task<PodSeating> ReturnAsync(DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (Live(player.Id) is not { } hosted) return PodSeating.Refused("You are not in a booster draft.");
            string id = DraftHost.PlayerId(player);
            int seat = hosted.Host.Pod.Players.First(each => each.PlayerId == id).Seat;
            return new PodSeating(hosted.Host.Address, hosted.Host.Pod.IssueTicket(id), seat, null);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>Whether a pod being played is this host's, by its event id.</summary>
    public bool Hosts(string eventId)
    {
        _gate.Wait();
        try { return _pods.TryGetValue(eventId, out Hosted? hosted) && hosted.EndedAt is null; }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// A player drops out of their pod (<see cref="BoosterPod.Drop(string)"/>): from then on they are in no draft, and free
    /// to join another. Null once they have; otherwise why not.
    /// </summary>
    public async Task<string?> DropAsync(string eventId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        ulong queue;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_pods.TryGetValue(eventId, out Hosted? hosted) || hosted.EndedAt is not null) return "That draft is over.";
            if (hosted.Host.Pod.Drop(DraftHost.PlayerId(player)) is { } refusal) return refusal;
            queue = hosted.Queue;
        }
        finally
        {
            _gate.Release();
        }

        await Raise(queue).ConfigureAwait(false);
        return null;
    }

    /// <summary>
    /// Notes pods that have finished, and closes the tables of those finished long enough ago for their players to have
    /// read the final standings.
    /// </summary>
    public async Task SweepAsync()
    {
        List<(Hosted Hosted, bool Raise)> changed = [];
        List<PodHost> closing = [];
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            DateTimeOffset now = _options.TimeProvider.GetUtcNow();
            foreach ((string id, Hosted hosted) in _pods.ToList())
            {
                if (hosted.EndedAt is null && hosted.Host.Pod.Stage is EventStage.Complete or EventStage.Cancelled)
                {
                    hosted.EndedAt = now;
                    changed.Add((hosted, true));
                }

                if (hosted.EndedAt is { } ended && now - ended >= _options.Linger)
                {
                    _pods.Remove(id);
                    hosted.Bots.Cancel();
                    closing.Add(hosted.Host);
                }
            }
        }
        finally
        {
            _gate.Release();
        }

        foreach (PodHost host in closing)
        {
            await host.DisposeAsync().ConfigureAwait(false);
        }

        foreach ((Hosted hosted, _) in changed)
        {
            await Raise(hosted.Queue).ConfigureAwait(false);
        }
    }

    /// <summary>
    /// Sits a bot down in each of the pod's seats its queue kept for one, at the pod's table; from there each follows the
    /// pod to the tables of its games. Returns the pod.
    /// </summary>
    private Hosted PlayBots(Hosted hosted)
    {
        BoosterPod pod = hosted.Host.Pod;
        CancellationToken cancellation = hosted.Bots.Token;
        foreach ((string id, string name, int _) in pod.Players.Where(player => BotSeats.IsBot(player.PlayerId)))
        {
            string ticket = pod.IssueTicket(id);
            _ = Task.Run(() => BotSeats.PlayPodAsync(hosted.Host.Address, _options.ListenAddress, ticket, name, cancellation), CancellationToken.None);
        }

        return hosted;
    }

    private Hosted? Live(ulong player)
    {
        string id = DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));
        // A player who has dropped is in the pod no more: free to join another draft.
        return _pods.Values.FirstOrDefault(hosted => hosted.EndedAt is null && hosted.Host.Pod.Players.Any(each => each.PlayerId == id) && !hosted.Host.Pod.HasDropped(id));
    }

    /// <summary>
    /// Opens a table on the first draft port that will have it: a port another table holds - this host's, a queue's or
    /// a Winston draft's - refuses, and the next is tried.
    /// </summary>
    private async Task<(GameServer Server, Uri Address)> Open(ITableEvent table, int seats)
    {
        List<string> failures = [];
        foreach (int port in _options.Ports)
        {
            try
            {
                GameServer server = await GameServer.StartAsync(new GameServerOptions
                {
                    Port = port,
                    Players = seats,
                    ContentRoot = _options.ContentRoot,
                    ListenAddress = _options.ListenAddress,
                    TimeProvider = _options.TimeProvider,
                    Event = table,
                    Portraits = _options.Portraits,
                    Art = _options.Art,
                    Return = table is BoosterPod ? _options.Return : null,
                }).ConfigureAwait(false);
                return (server, _options.TableAddress?.Invoke(server.Port) ?? new Uri($"ws://{_options.PublicHost}:{server.Port}/game"));
            }
            catch (IOException error)
            {
                failures.Add($"{port}: {error.Message}");
            }
        }

        throw new IOException("Every draft port is in use: a pod's games want a port each. Give the bot more --draft-ports. " + string.Join("; ", failures));
    }

    private async Task Raise(ulong queue)
    {
        if (Changed is not { } changed) return;
        foreach (Func<ulong, Task> handler in changed.GetInvocationList().Cast<Func<ulong, Task>>())
            await handler(queue).ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        List<PodHost> hosts;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            hosts = [.. _pods.Values.Select(hosted => hosted.Host)];
            foreach (Hosted hosted in _pods.Values) hosted.Bots.Cancel();
            _pods.Clear();
        }
        finally
        {
            _gate.Release();
        }

        foreach (PodHost host in hosts)
        {
            await host.DisposeAsync().ConfigureAwait(false);
        }
    }
}
