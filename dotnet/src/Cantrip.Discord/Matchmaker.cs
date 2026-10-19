using System.Security.Cryptography;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Discord;

/// <summary>A Discord account by its stable ID. The name is presentation, taken when the player sat down.</summary>
public sealed record DiscordPlayer(ulong Id, string Name);

public sealed class MatchmakerOptions
{
    public required string ContentRoot { get; init; }

    /// <summary>The deck every seat of a game hosted from Discord plays, so there is nothing to choose.</summary>
    public required DeckList Deck { get; init; }

    /// <summary>
    /// The constructed formats games may be hosted in from the play page, by name ("standard"), each with the decks its
    /// players choose from as they sit down. Empty for none.
    /// </summary>
    public IReadOnlyDictionary<string, IReadOnlyList<DeckList>> Formats { get; init; } = new Dictionary<string, IReadOnlyList<DeckList>>();

    /// <summary>The ports tables may listen on, one table to a port. How many games can run at once.</summary>
    public required IReadOnlyList<int> Ports { get; init; }

    /// <summary>Where tables listen. Loopback unless players on other machines are meant to reach them.</summary>
    public string ListenAddress { get; init; } = "127.0.0.1";

    /// <summary>The host name players connect to, which is what goes into the address the bot hands out.</summary>
    public string PublicHost { get; init; } = "localhost";

    /// <summary>
    /// The address players are given for the table on a port, when not <c>ws://{PublicHost}:{port}/game</c>: behind the
    /// bot's one public URL, <c>wss://host/table/{port}/game</c> (<see cref="TableProxy"/>).
    /// </summary>
    public Func<int, Uri>? TableAddress { get; init; }

    /// <summary>Where card art is, said by every table to the clients at it (<see cref="GameServerOptions.Art"/>); null for not said.</summary>
    public Uri? Art { get; init; }

    /// <summary>Each player's picture, by player id, as the pages show it beside their name; null for none.</summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>
    /// The site's page its players go back to once a table's game is over (<see cref="GameServerOptions.Return"/>); null
    /// when the bot serves no pages.
    /// </summary>
    public ReturnPage? Return { get; init; }

    /// <summary>How long a table may wait for its players to sit down and the cards to be dealt.</summary>
    public TimeSpan SetupTimeout { get; init; } = TimeSpan.FromMinutes(30);

    /// <summary>How long an ended table stays up, so both clients can see the result before it goes.</summary>
    public TimeSpan Linger { get; init; } = TimeSpan.FromMinutes(2);

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}

public enum GameStage
{
    /// <summary>Hosted; the second seat is free.</summary>
    Open,

    /// <summary>Both seats are taken; waiting for both players to connect.</summary>
    Seated,

    /// <summary>The cards have been dealt.</summary>
    Playing,

    Ended,
}

public enum GameEnding { None, Won, Draw, Abandoned, Cancelled, Expired, Failed }

/// <summary>One hosted game: who is in it, where it is, and how it went. Changed only under the matchmaker's lock.</summary>
public sealed class HostedGame
{
    private readonly Dictionary<ulong, string> _tickets = [];

    internal HostedGame(string id, DiscordPlayer host, int port, Uri address, GameServer server, DateTimeOffset createdAt, string? format = null)
    {
        Id = id;
        Format = format;
        Host = host;
        Port = port;
        Address = address;
        Server = server;
        CreatedAt = createdAt;
    }

    /// <summary>Short and opaque, for button IDs. Not a secret: knowing it seats nobody.</summary>
    public string Id { get; }

    /// <summary>The constructed format it is played in, its players each choosing a deck; null for a game hosted from Discord.</summary>
    public string? Format { get; }

    public DiscordPlayer Host { get; }
    public DiscordPlayer? Guest { get; internal set; }
    public int Port { get; }

    /// <summary>The WebSocket address players connect to.</summary>
    public Uri Address { get; }

    public DateTimeOffset CreatedAt { get; }
    public GameStage Stage { get; internal set; }
    public GameEnding Ending { get; internal set; }

    /// <summary>Who won, when someone did.</summary>
    public IReadOnlyList<DiscordPlayer> Winners { get; internal set; } = [];

    /// <summary>Who cancelled it, for a cancelled game.</summary>
    public DiscordPlayer? CancelledBy { get; internal set; }

    /// <summary>What the table said about how it ended, if anything.</summary>
    public string Detail { get; internal set; } = string.Empty;

    internal DateTimeOffset? EndedAt { get; set; }
    internal GameServer Server { get; }

    /// <summary>Whether the server has been disposed and its port given back.</summary>
    internal bool Closed { get; set; }

    public IEnumerable<DiscordPlayer> Players => Guest is null ? [Host] : [Host, Guest];
    public bool Seats(ulong player) => _tickets.ContainsKey(player);
    internal string Ticket(ulong player) => _tickets[player];
    internal void Seat(ulong player, string ticket) => _tickets[player] = ticket;
    internal DiscordPlayer? AtSeat(int seat) => seat switch { 1 => Host, 2 => Guest, _ => null };
}

/// <summary>What a player gets for hosting or joining: their game and seat ticket, or why not.</summary>
public sealed record Seating(HostedGame? Game, string? Ticket, int Seat, string? Refusal, bool Again = false)
{
    public static Seating Refused(string why, HostedGame? game = null) => new(game, null, 0, why);
}

/// <summary>
/// Hosts two-player games for Discord players, knowing nothing of Discord itself. Each game is its own
/// <see cref="GameServer"/> on a port from the pool, seating only the two players it issued tickets to.
/// <para>
/// The matchmaker is authoritative for who may sit where; the Discord adapter only asks it. A player is in at most one
/// unfinished game at a time, which is also what keeps one person from taking every port. Time-based work — setup
/// that never finishes, ended tables whose result has been seen — happens in <see cref="SweepAsync"/>, which the host
/// calls on a timer and a test calls with its clock moved on.
/// </para>
/// </summary>
public sealed class Matchmaker : IAsyncDisposable
{
    private readonly MatchmakerOptions _options;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly Dictionary<string, HostedGame> _games = new(StringComparer.Ordinal);

    public Matchmaker(MatchmakerOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        if (options.Ports.Count == 0) throw new ArgumentException("Give the matchmaker at least one port.", nameof(options));
        _options = options;
    }

    /// <summary>
    /// A game moved on: seated, dealt or ended. Raised outside the lock, on whichever thread moved it; the adapter
    /// renders the game's message from it.
    /// </summary>
    public event Func<HostedGame, Task>? Changed;

    public HostedGame? Find(string id)
    {
        _gate.Wait();
        try { return _games.GetValueOrDefault(id); }
        finally { _gate.Release(); }
    }

    /// <summary>A player in a game, as the pages list them: their name, picture and Discord account.</summary>
    public PlayerFace FaceOf(DiscordPlayer player) => PlayerFace.Of(DraftHost.PlayerId(player), player.Name, _options.Portraits);

    /// <summary>Every game not yet over, as the server's status page shows it: who is playing, and where the game stands.</summary>
    public IReadOnlyList<StatusTable> StatusTables()
    {
        _gate.Wait();
        try
        {
            return [.. _games.Values.Where(game => game.Stage != GameStage.Ended).OrderBy(game => game.CreatedAt).Select(game => new StatusTable(
                StatusTableKinds.Game, string.Empty, string.Empty, game.Stage.ToString(), [.. game.Players.Select(FaceOf)], 2, 0, game.CreatedAt,
                game.Stage switch { GameStage.Open => "Waiting for a second player.", GameStage.Seated => "Waiting for both players to connect.", _ => string.Empty }))];
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>The constructed formats games may be hosted in, and each one's decks.</summary>
    public IReadOnlyDictionary<string, IReadOnlyList<DeckList>> Formats => _options.Formats;

    /// <summary>Every game of a constructed format not yet over, oldest first, as the play page lists them.</summary>
    public IReadOnlyList<HostedGame> Games(string format)
    {
        _gate.Wait();
        try { return [.. _games.Values.Where(game => game.Format == format && game.Stage != GameStage.Ended).OrderBy(game => game.CreatedAt)]; }
        finally { _gate.Release(); }
    }

    /// <summary>The game a player has a seat in that is not yet over, if any.</summary>
    public HostedGame? GameOf(ulong player)
    {
        _gate.Wait();
        try { return Active(player); }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// Hosts a game with its first seat the host's: from Discord, both seats playing <see cref="MatchmakerOptions.Deck"/>;
    /// in a constructed <paramref name="format"/>, the host playing <paramref name="deck"/>, one of the format's, and the
    /// player who joins choosing their own.
    /// </summary>
    public async Task<Seating> HostAsync(DiscordPlayer host, string? format = null, string? deck = null)
    {
        ArgumentNullException.ThrowIfNull(host);
        IReadOnlyList<DeckList> decks = [_options.Deck];
        if (format is not null)
        {
            if (!_options.Formats.TryGetValue(format, out IReadOnlyList<DeckList>? formatDecks)) return Seating.Refused($"There are no {format} games here.");
            if (formatDecks.FirstOrDefault(each => each.Name == deck) is null) return Seating.Refused("Choose a deck from the list.");
            decks = formatDecks;
        }

        await _gate.WaitAsync().ConfigureAwait(false);
        HostedGame game;
        string ticket;
        try
        {
            if (Active(host.Id) is { } current)
                return Seating.Refused("You are already in a game. Finish or cancel it before hosting another.", current);

            HashSet<int> taken = [.. _games.Values.Where(game => !game.Closed).Select(game => game.Port)];
            GameServer? server = null;
            int port = 0;
            List<string> failures = [];
            foreach (int candidate in _options.Ports.Where(candidate => !taken.Contains(candidate)))
            {
                try
                {
                    server = await GameServer.StartAsync(new GameServerOptions
                    {
                        Port = candidate,
                        Players = 2,
                        Seed = BitConverter.ToUInt64(RandomNumberGenerator.GetBytes(8)),
                        ContentRoot = _options.ContentRoot,
                        Decks = decks,
                        RequireTickets = true,
                        ListenAddress = _options.ListenAddress,
                        TimeProvider = _options.TimeProvider,
                        Art = _options.Art,
                    Return = _options.Return,
                    }).ConfigureAwait(false);
                    port = candidate;
                    break;
                }
                catch (IOException error)
                {
                    // Something else on the machine holds this port. Try the next rather than refusing outright.
                    failures.Add($"{candidate}: {error.Message}");
                }
            }

            if (server is null)
            {
                return Seating.Refused(failures.Count == 0
                    ? "Every table is in use. Try again when a game finishes."
                    : "No table could be opened: " + string.Join("; ", failures));
            }

            string id;
            do id = Convert.ToHexString(RandomNumberGenerator.GetBytes(4)).ToLowerInvariant();
            while (_games.ContainsKey(id));
            var address = _options.TableAddress?.Invoke(server.Port) ?? new Uri($"ws://{_options.PublicHost}:{server.Port}/game");
            game = new HostedGame(id, host with { Name = SeatName(host.Name) }, port, address, server, _options.TimeProvider.GetUtcNow(), format);
            ticket = server.Reserve(1, game.Host.Name, format is null ? _options.Deck.Name : deck!);
            game.Seat(host.Id, ticket);
            _games.Add(id, game);
            Watch(game);
        }
        finally
        {
            _gate.Release();
        }

        return new Seating(game, ticket, 1, null);
    }

    /// <summary>
    /// Seats a player in the free seat. A player already seated at this game gets their own ticket again, which is how
    /// someone who lost it — or whose client died — gets back in. A constructed game's player sits down with
    /// <paramref name="deck"/>, one of its format's.
    /// </summary>
    public async Task<Seating> JoinAsync(string gameId, DiscordPlayer player, string? deck = null)
    {
        ArgumentNullException.ThrowIfNull(player);
        Seating seating;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_games.TryGetValue(gameId, out HostedGame? game) || game.Stage == GameStage.Ended)
                return Seating.Refused("That game has ended.");
            if (game.Seats(player.Id))
                return new Seating(game, game.Ticket(player.Id), game.Host.Id == player.Id ? 1 : 2, null, Again: true);
            if (game.Guest is not null)
                return Seating.Refused("That game already has both its players.", game);
            if (Active(player.Id) is { } current)
                return Seating.Refused("You are already in another game. Finish or cancel it first.", current);
            if (game.Format is { } format && !_options.Formats[format].Any(each => each.Name == deck))
                return Seating.Refused("Choose a deck from the list.", game);

            game.Guest = player with { Name = SeatName(player.Name) };
            string ticket = game.Server.Reserve(2, game.Guest.Name, game.Format is null ? _options.Deck.Name : deck!);
            game.Seat(player.Id, ticket);
            game.Stage = GameStage.Seated;
            seating = new Seating(game, ticket, 2, null);
        }
        finally
        {
            _gate.Release();
        }

        await Raise(seating.Game!).ConfigureAwait(false);
        return seating;
    }

    /// <summary>Ends a game early. Its host may; so may whoever <paramref name="moderator"/> says can. Returns a
    /// refusal, or null once the game has ended.</summary>
    public async Task<string?> CancelAsync(string gameId, DiscordPlayer by, bool moderator)
    {
        ArgumentNullException.ThrowIfNull(by);
        HostedGame? game;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_games.TryGetValue(gameId, out game) || game.Stage == GameStage.Ended) return "That game has already ended.";
            if (game.Host.Id != by.Id && !moderator) return "Only the game's host or a server manager can cancel it.";
            game.CancelledBy = by;
            End(game, GameEnding.Cancelled);
            await Close(game).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }

        await Raise(game).ConfigureAwait(false);
        return null;
    }

    /// <summary>
    /// Expires tables that never got going and closes ended ones once their result has had time to be seen, giving
    /// their ports back. Ended games are then forgotten.
    /// </summary>
    public async Task SweepAsync()
    {
        List<HostedGame> expired = [];
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            DateTimeOffset now = _options.TimeProvider.GetUtcNow();
            foreach (HostedGame game in _games.Values.ToArray())
            {
                if (game.Stage is GameStage.Open or GameStage.Seated && now - game.CreatedAt >= _options.SetupTimeout)
                {
                    End(game, GameEnding.Expired);
                    await Close(game).ConfigureAwait(false);
                    expired.Add(game);
                }
                else if (game.Stage == GameStage.Ended && !game.Closed && now - game.EndedAt >= _options.Linger)
                {
                    await Close(game).ConfigureAwait(false);
                }

                if (game.Closed) _games.Remove(game.Id);
            }
        }
        finally
        {
            _gate.Release();
        }

        foreach (HostedGame game in expired) await Raise(game).ConfigureAwait(false);
    }

    /// <summary>A player's unfinished game, if they have one.</summary>
    private HostedGame? Active(ulong player) =>
        _games.Values.FirstOrDefault(game => game.Stage != GameStage.Ended && game.Seats(player));

    private void Watch(HostedGame game)
    {
        _ = game.Server.Started.ContinueWith(_ => Update(game, () =>
        {
            if (game.Stage == GameStage.Ended) return false;
            game.Stage = GameStage.Playing;
            return true;
        }), TaskScheduler.Default).Unwrap();

        _ = game.Server.Finished.ContinueWith(finished => Update(game, () =>
        {
            if (game.Stage == GameStage.Ended) return false;
            if (finished.IsFaulted)
            {
                game.Detail = game.Server.FaultDetail;
                End(game, GameEnding.Failed);
                return true;
            }

            TableResult result = game.Server.Result!;
            game.Detail = result.Detail;
            game.Winners = [.. result.WinningSeats.Select(game.AtSeat).OfType<DiscordPlayer>()];
            End(game, result.Phase == TablePhase.Abandoned ? GameEnding.Abandoned
                : game.Winners.Count > 0 ? GameEnding.Won : GameEnding.Draw);
            return true;
        }), TaskScheduler.Default).Unwrap();
    }

    /// <summary>Applies a change the table itself reported, under the lock, and tells the adapter if it took.</summary>
    private async Task Update(HostedGame game, Func<bool> change)
    {
        bool changed;
        await _gate.WaitAsync().ConfigureAwait(false);
        try { changed = change(); }
        finally { _gate.Release(); }

        if (changed) await Raise(game).ConfigureAwait(false);
    }

    private void End(HostedGame game, GameEnding ending)
    {
        game.Stage = GameStage.Ended;
        game.Ending = ending;
        game.EndedAt = _options.TimeProvider.GetUtcNow();
    }

    private static async Task Close(HostedGame game)
    {
        if (game.Closed) return;
        game.Closed = true;
        await game.Server.DisposeAsync().ConfigureAwait(false);
    }

    private async Task Raise(HostedGame game)
    {
        if (Changed is not { } changed) return;
        foreach (Func<HostedGame, Task> handler in changed.GetInvocationList().Cast<Func<HostedGame, Task>>())
            await handler(game).ConfigureAwait(false);
    }

    /// <summary>
    /// A Discord name as a table seat name: printable, trimmed, at most 32 characters, which is what a table accepts.
    /// </summary>
    internal static string SeatName(string name)
    {
        string printable = new string([.. name.Where(character => !char.IsControl(character))]).Trim();
        // Cut before a surrogate pair rather than through it.
        if (printable.Length > 32) printable = printable[..(char.IsHighSurrogate(printable[31]) ? 31 : 32)].TrimEnd();
        return printable.Length == 0 ? "Player" : printable;
    }

    public async ValueTask DisposeAsync()
    {
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            foreach (HostedGame game in _games.Values) await Close(game).ConfigureAwait(false);
            _games.Clear();
        }
        finally
        {
            _gate.Release();
        }
    }
}
