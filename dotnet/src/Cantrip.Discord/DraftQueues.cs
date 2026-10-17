using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Discord;

/// <summary>A queue as its channel's owner set it: what is drafted there, from which cube, and where its message is.</summary>
public sealed class QueueSettings
{
    /// <summary>The channel the queue is in; one queue a channel.</summary>
    public ulong Channel { get; set; }

    /// <summary>The draft format: "winston", "booster" for a pod of eight opening packs, or "grid".</summary>
    public string Format { get; set; } = DraftFormats.Winston;

    /// <summary>The cube, by name in the cube lists or by path.</summary>
    public string Cube { get; set; } = string.Empty;

    /// <summary>How many cards a draft draws from the cube; zero for all of it.</summary>
    public int Cards { get; set; } = 90;

    public int Piles { get; set; } = 3;

    /// <summary>
    /// How many packs each player opens, in a booster draft; how many grids are laid out, in a grid draft. Zero for the
    /// format's own number: three packs, or eighteen grids.
    /// </summary>
    public int Packs { get; set; }

    /// <summary>The packs a draft of this queue takes: <see cref="Packs"/>, or the format's own number.</summary>
    public int PacksDrafted => Packs > 0 ? Packs : Format == DraftFormats.Grid ? 18 : 3;

    /// <summary>How many cards a pack has, in a booster draft.</summary>
    public int PackSize { get; set; } = 15;

    /// <summary>How many Swiss rounds a booster draft plays.</summary>
    public int Rounds { get; set; } = 3;

    public int MinimumDeck { get; set; } = 40;

    /// <summary>How many seats a table of this queue has: two for Winston and grid, eight for a booster draft's pod.</summary>
    public int Seats => Format == DraftFormats.Booster ? 8 : 2;

    /// <summary>
    /// How many of a table's seats are played by bots, the last ones: a queue a single player can draft from, to try a
    /// format out. Zero for none.
    /// </summary>
    public int Bots { get; set; }

    /// <summary>The seats players take: the table's, less its bots'.</summary>
    public int PlayerSeats => Seats - Bots;

    /// <summary>The bot's message in the channel that shows the queue and its Join button; zero before it is posted.</summary>
    public ulong Message { get; set; }

    public DateTimeOffset Updated { get; set; }
}

/// <summary>What a queue's message shows: how full the table being filled is, and what else is going on.</summary>
public sealed record QueueStatus(QueueSettings Settings, int Waiting, int Seats, int Checking, int Underway);

/// <summary>
/// Every queue and every table still forming, as the draft queue page shows them (<see cref="QueueSite"/>): what each
/// queue drafts, and who has sat down where.
/// </summary>
public sealed record QueueBoard(IReadOnlyList<QueueBoardQueue> Queues, IReadOnlyList<QueueBoardTable> Tables);

/// <summary>A channel's queue: what a table made for it drafts, and the cube's name as it is shown.</summary>
public sealed record QueueBoardQueue(QueueSettings Settings, string Cube);

/// <summary>
/// A table still forming. <see cref="Channel"/> is its queue's, or zero for a table a player set up on the queue page with
/// settings of their own, which only that page seats players at. <see cref="Seats"/> are the players' seats, less the bots'.
/// </summary>
public sealed record QueueBoardTable(string Id, ulong Channel, QueueSettings Settings, string Cube, IReadOnlyList<string> Players, int Seats, bool Checking, DateTimeOffset Created);

/// <summary>What a player gets for joining a queue: the table's address and a ticket to their seat, or why not.</summary>
public sealed record QueueSeating(Uri? Address, string? Ticket, int Seat, string? Refusal)
{
    public static QueueSeating Refused(string why) => new(null, null, 0, why);
}

public sealed class DraftQueuesOptions
{
    public required string ContentRoot { get; init; }

    /// <summary>Where queues are kept (<c>queues/{channel}.json</c>), and the events their drafts become.</summary>
    public required EventStore Store { get; init; }

    /// <summary>Each player's picture, by player id, as their tables show it (<see cref="GameServerOptions.Portraits"/>); null for none.</summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>Takes over each table once its draft begins.</summary>
    public required DraftHost Drafts { get; init; }

    /// <summary>Takes over each booster draft's table once its pod begins; null hosts no booster queues.</summary>
    public PodHosts? Pods { get; init; }

    /// <summary>The ports tables may listen on: the draft ports, shared with the draft host - a port in use is passed over.</summary>
    public required IReadOnlyList<int> Ports { get; init; }

    /// <summary>Where to look for a cube named rather than given by path.</summary>
    public IReadOnlyList<string> CubeDirectories { get; init; } = [];

    public string ListenAddress { get; init; } = "127.0.0.1";

    public string PublicHost { get; init; } = "localhost";

    /// <summary>
    /// The address players are given for the table on a port, when not <c>ws://{PublicHost}:{port}/game</c>: behind the
    /// bot's one public URL, <c>wss://host/table/{port}/game</c> (<see cref="TableProxy"/>).
    /// </summary>
    public Func<int, Uri>? TableAddress { get; init; }

    /// <summary>Where card art is, said by every table to the clients at it (<see cref="GameServerOptions.Art"/>); null for not said.</summary>
    public Uri? Art { get; init; }

    /// <summary>How long a full table's players have to say they are ready.</summary>
    public TimeSpan ReadyWindow { get; init; } = TimeSpan.FromSeconds(30);

    /// <summary>The clocks of a booster draft made from a queue.</summary>
    public BoosterTiming PodTiming { get; init; } = new();

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}

/// <summary>
/// The draft queues, one to a channel, knowing nothing of Discord itself. A player who joins a queue gets a seat at a
/// table and a ticket to it at once: the oldest of the queue's tables with a seat free, or a new one. The table forms in
/// the player's browser (<see cref="QueueTable"/>) - who else is there, Leave Table, and a ready check once it is full -
/// and once everyone is ready its draft begins there and the <see cref="DraftHost"/> takes it over.
/// <para>
/// A table that loses players to a ready check, or to leaving, is filled again from the next players to join, oldest
/// table first. When an older table has a free seat and a newer one has players still waiting for theirs to fill, those
/// players are moved to the older table (the page goes there by itself), so a queue never has two tables each waiting for
/// one player.
/// </para>
/// <para>
/// A player may also set a table up with settings of their own, from the queue page (<see cref="CreateTableAsync"/>): it
/// belongs to no channel, a channel's Join never seats anyone at it, and it is filled only by players who pick it there.
/// </para>
/// <para>
/// The queues' settings are kept in the data directory. A table still forming is only in memory: a restart empties it,
/// and its players join again.
/// </para>
/// </summary>
public sealed class DraftQueues : IAsyncDisposable
{
    private const string Folder = "queues";

    private readonly DraftQueuesOptions _options;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly Dictionary<ulong, Queue> _queues = [];

    /// <summary>The tables players set up with settings of their own, in no channel's queue.</summary>
    private readonly List<Forming> _own = [];

    /// <summary>
    /// Players whose table has begun its draft but not yet been handed to the draft host: not to be seated anywhere else
    /// in the moment between.
    /// </summary>
    private readonly HashSet<string> _starting = new(StringComparer.Ordinal);

    private sealed class Queue(QueueSettings settings, IReadOnlyList<string> cards, IReadOnlyList<string> pickOrder, string cube)
    {
        public QueueSettings Settings { get; set; } = settings;
        public IReadOnlyList<string> CubeCards { get; set; } = cards;
        public IReadOnlyList<string> PickOrder { get; set; } = pickOrder;
        public string CubeName { get; set; } = cube;
        public List<Forming> Tables { get; } = [];
    }

    private sealed class Forming(QueueTable table, GameServer server, int port, Uri address, QueueSettings settings, string cube)
    {
        /// <summary>How the queue page names the table: never a ticket, and no use to anyone without a seat to take.</summary>
        public string Id { get; } = Guid.NewGuid().ToString("N")[..12];

        /// <summary>What the table drafts, as it was made: its queue's settings then, or a player's own.</summary>
        public QueueSettings Settings { get; } = settings;

        public string CubeName { get; } = cube;

        /// <summary>The players' seats: the table's, less its bots'.</summary>
        public int Seats => Settings.PlayerSeats;

        public QueueTable Table { get; } = table;
        public GameServer Server { get; } = server;
        public int Port { get; } = port;
        public Uri Address { get; } = address;
        public Dictionary<string, DiscordPlayer> Players { get; } = new(StringComparer.Ordinal);
        public bool Gone { get; set; }
    }

    public DraftQueues(DraftQueuesOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        if (options.Ports.Count == 0) throw new ArgumentException("Give the queues at least one port.", nameof(options));
        _options = options;
    }

    /// <summary>A queue's status changed. Raised outside the lock; the adapter brings the queue's message up to date.</summary>
    public event Func<ulong, Task>? Changed;

    /// <summary>Reads the queues kept in the data directory: what a starting bot does first. Returns the ones that could not be read.</summary>
    public async Task<IReadOnlyList<string>> LoadAsync()
    {
        List<string> problems = [];
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            foreach (string id in _options.Store.Ids(Folder))
            {
                if (_options.Store.Load<QueueSettings>(Folder, id) is not { } settings) continue;
                try
                {
                    (string cube, IReadOnlyList<string> cards, IReadOnlyList<string> pickOrder) = Cube(settings);
                    _queues[settings.Channel] = new Queue(settings, cards, pickOrder, cube);
                }
                catch (Exception error) when (error is IOException or ArgumentException or FormatException)
                {
                    problems.Add($"Queue {id}: {error.Message}");
                }
            }
        }
        finally
        {
            _gate.Release();
        }

        return problems;
    }

    /// <summary>Every queue's settings.</summary>
    public IReadOnlyList<QueueSettings> Queues
    {
        get
        {
            _gate.Wait();
            try { return [.. _queues.Values.Select(queue => queue.Settings)]; }
            finally { _gate.Release(); }
        }
    }

    /// <summary>
    /// Sets a channel's queue up, or changes what it drafts. Returns why not, or null. A change applies to the tables
    /// made from then on; the ones forming keep what they were made with.
    /// </summary>
    public async Task<string?> ConfigureAsync(QueueSettings settings)
    {
        ArgumentNullException.ThrowIfNull(settings);
        if (Check(settings, out var found) is { } refusal) return refusal;

        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_queues.TryGetValue(settings.Channel, out Queue? queue))
            {
                settings.Message = settings.Message == 0 ? queue.Settings.Message : settings.Message;
                queue.Settings = settings;
                queue.CubeCards = found.Cards;
                queue.PickOrder = found.PickOrder;
                queue.CubeName = found.Cube;
            }
            else
            {
                _queues[settings.Channel] = new Queue(settings, found.Cards, found.PickOrder, found.Cube);
            }

            settings.Updated = _options.TimeProvider.GetUtcNow();
            _options.Store.Save(Folder, settings.Channel.ToString(System.Globalization.CultureInfo.InvariantCulture), settings);
        }
        finally
        {
            _gate.Release();
        }

        return null;
    }

    /// <summary>Why a draft cannot be had with these settings, or null with the cube it draws from.</summary>
    private string? Check(QueueSettings settings, out (string Cube, IReadOnlyList<string> Cards, IReadOnlyList<string> PickOrder) found)
    {
        found = default;
        if (settings.Format is not (DraftFormats.Winston or DraftFormats.Booster or DraftFormats.Grid)) return $"There is no '{settings.Format}' draft. Winston, booster and grid are the ones there are.";
        if (settings.Format == DraftFormats.Booster && _options.Pods is null) return "This bot is not set up to run booster drafts.";
        if (settings.Piles is < 1 or > 8) return "A Winston draft has one to eight piles.";
        if (settings.Format == DraftFormats.Booster && (settings.PacksDrafted is < 1 or > 6 || settings.PackSize is < 5 or > 20)) return "A booster draft opens one to six packs of five to twenty cards.";
        if (settings.Format == DraftFormats.Grid && settings.PacksDrafted is < 1 or > 40) return "A grid draft lays out one to forty grids.";
        if (settings.Rounds is < 1 or > 5) return "A booster draft plays one to five rounds.";
        if (settings.MinimumDeck is < 1 or > 200) return "The smallest deck must be between 1 and 200 cards.";
        if (settings.Bots < 0 || settings.Bots >= settings.Seats) return $"A table of {settings.Seats} has up to {settings.Seats - 1} bot{(settings.Seats == 2 ? string.Empty : "s")}: at least one seat is a player's.";

        (string Cube, IReadOnlyList<string> Cards, IReadOnlyList<string> PickOrder) cube;
        try
        {
            cube = Cube(settings);
        }
        catch (Exception error) when (error is IOException or ArgumentException or FormatException)
        {
            return error.Message;
        }

        int playable = DraftPod.Playable(ContentCache.Get(_options.ContentRoot), cube.Cards, settings.Format);
        int wanted = settings.Format switch
        {
            DraftFormats.Booster => settings.Seats * settings.PacksDrafted * settings.PackSize,
            DraftFormats.Grid => settings.PacksDrafted * GridDraft.PackSize,
            _ => settings.Cards,
        };
        if (wanted < 0 || wanted > playable)
            return $"The cube {cube.Cube} has {playable} cards that can be played; a draft cannot take {wanted}.";

        found = cube;
        return null;
    }

    /// <summary>Notes where the queue's message is, and keeps it.</summary>
    public async Task NoteMessageAsync(ulong channel, ulong message)
    {
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_queues.TryGetValue(channel, out Queue? queue)) return;
            queue.Settings.Message = message;
            _options.Store.Save(Folder, channel.ToString(System.Globalization.CultureInfo.InvariantCulture), queue.Settings);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>Closes a channel's queue: its forming tables are closed, and it is no longer kept. Returns its settings, or null.</summary>
    public async Task<QueueSettings?> RemoveAsync(ulong channel)
    {
        Queue? queue;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_queues.Remove(channel, out queue)) return null;
            _options.Store.Delete(Folder, channel.ToString(System.Globalization.CultureInfo.InvariantCulture));

            // Each player told the queue is closed; the tables go once that has been said.
            foreach (Forming forming in queue.Tables.Where(forming => !forming.Gone))
            {
                forming.Table.Close("This draft queue was closed, so the table was taken down.");
            }
        }
        finally
        {
            _gate.Release();
        }

        return queue.Settings;
    }

    /// <summary>How a queue stands, for its message; null for a channel with no queue.</summary>
    public QueueStatus? Status(ulong channel)
    {
        _gate.Wait();
        try
        {
            if (!_queues.TryGetValue(channel, out Queue? queue)) return null;
            Forming? filling = queue.Tables.Where(forming => !forming.Gone && forming.Table.Forming && !forming.Table.Checking)
                .OrderBy(forming => forming.Table.Created).FirstOrDefault();
            int checking = queue.Tables.Count(forming => !forming.Gone && forming.Table.Checking);
            int underway = _options.Drafts.Drafts.Count(draft => draft.Queue == channel && draft.IsLive) + (_options.Pods?.Underway(channel) ?? 0);
            return new QueueStatus(queue.Settings, filling?.Table.Taken ?? 0, queue.Settings.PlayerSeats, checking, underway);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>Every queue's tables still forming, as the server's status page shows them: who has sat down, and the seats still open.</summary>
    public IReadOnlyList<StatusTable> StatusTables()
    {
        _gate.Wait();
        try
        {
            return [.. Tables().Where(forming => forming.Table.Forming).Select(forming => new StatusTable(
                StatusTableKinds.Forming,
                forming.Settings.Format,
                forming.CubeName,
                forming.Table.Checking ? "Checking" : "Forming",
                [.. forming.Table.Players.Select(player => player.Name)],
                forming.Seats,
                forming.Settings.Bots,
                forming.Table.Created,
                forming.Table.Checking ? "Ready check" : $"{forming.Table.Taken} of {forming.Seats} seated"))
                .OrderBy(table => table.Created)];
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>The cube's name as the queue shows it.</summary>
    public string CubeName(ulong channel)
    {
        _gate.Wait();
        try { return _queues.TryGetValue(channel, out Queue? queue) ? queue.CubeName : string.Empty; }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// Every queue, and every table still forming - a queue's, or one a player set up with settings of their own - oldest
    /// first, as the queue page shows them.
    /// </summary>
    public QueueBoard Board()
    {
        _gate.Wait();
        try
        {
            return new QueueBoard(
                [.. _queues.Values.OrderBy(queue => queue.Settings.Channel).Select(queue => new QueueBoardQueue(queue.Settings, queue.CubeName))],
                [.. Tables().Where(forming => forming.Table.Forming).OrderBy(forming => forming.Table.Created).Select(forming => new QueueBoardTable(
                    forming.Id,
                    forming.Settings.Channel,
                    forming.Settings,
                    forming.CubeName,
                    [.. forming.Table.Players.Select(player => player.Name)],
                    forming.Seats,
                    forming.Table.Checking,
                    forming.Table.Created))]);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>The table still forming that a player has a seat at, by its <see cref="QueueBoardTable.Id"/>; null for none.</summary>
    public string? TableOf(ulong player)
    {
        string id = DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));
        _gate.Wait();
        try { return Tables().FirstOrDefault(forming => forming.Table.Seats(id))?.Id; }
        finally { _gate.Release(); }
    }

    /// <summary>The cubes a queue or a table may draw from, by name: every list in the cube directories.</summary>
    public IReadOnlyList<string> Cubes() => [.. _options.CubeDirectories
        .Where(Directory.Exists)
        .SelectMany(directory => Directory.EnumerateFiles(directory, "*.txt"))
        .Select(Path.GetFileNameWithoutExtension)
        .OfType<string>()
        .Distinct(StringComparer.OrdinalIgnoreCase)
        .Order(StringComparer.OrdinalIgnoreCase)];

    /// <summary>
    /// Seats a player from a queue: a fresh ticket to the table they are already at, or the first free seat at the
    /// queue's oldest table with one, or a new table. <paramref name="web"/> is a player on the queue page rather than in
    /// Discord, which a refusal speaks to.
    /// </summary>
    public async Task<QueueSeating> JoinAsync(ulong channel, DiscordPlayer player, bool web = false)
    {
        ArgumentNullException.ThrowIfNull(player);
        string id = DraftHost.PlayerId(player);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_queues.TryGetValue(channel, out Queue? queue)) return QueueSeating.Refused(web ? "That draft queue was closed." : "This channel has no draft queue.");

            // Already at a table: a fresh ticket to it, in this queue; anywhere else, where it is.
            if (Tables().FirstOrDefault(forming => forming.Table.Seats(id)) is { } at)
                return at.Settings.Channel == channel && at.Table.Reserve(id, player.Name) is { } again ? Seating(at, id, again) : Elsewhere(at, web);
            if (Busy(id, player, web) is { } busy) return busy;

            Forming? forming = queue.Tables
                .Where(each => !each.Gone && each.Table.Forming && each.Table.Taken < each.Seats)
                .OrderBy(each => each.Table.Created)
                .FirstOrDefault();
            if (forming is null)
            {
                (forming, string? refusal) = await OpenTable(queue.Settings, (queue.CubeName, queue.CubeCards, queue.PickOrder)).ConfigureAwait(false);
                if (forming is null) return QueueSeating.Refused(refusal!);
                queue.Tables.Add(forming);
            }

            return Sit(forming, player);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>
    /// Seats a player at the table they picked on the queue page, by its <see cref="QueueBoardTable.Id"/>: a queue's or one
    /// of a player's own, so long as it is still forming and has a seat free.
    /// </summary>
    public async Task<QueueSeating> JoinTableAsync(string table, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        string id = DraftHost.PlayerId(player);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (Tables().FirstOrDefault(forming => forming.Table.Seats(id)) is { } at)
                return at.Id == table && at.Table.Reserve(id, player.Name) is { } again ? Seating(at, id, again) : Elsewhere(at, web: true);
            if (Busy(id, player, web: true) is { } busy) return busy;

            if (Tables().FirstOrDefault(forming => forming.Id == table && forming.Table.Forming) is not { } forming)
                return QueueSeating.Refused("That table has gone: its draft began, or everyone left it.");
            if (forming.Table.Checking || forming.Table.Taken >= forming.Seats)
                return QueueSeating.Refused("That table is full.");
            return Sit(forming, player);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>
    /// Sets up a table with settings of the player's own, from the queue page, and seats them at it. It is in no channel's
    /// queue: only players who pick it on the page sit down there, and it closes, as any forming table does, once
    /// everyone has left it.
    /// </summary>
    public async Task<QueueSeating> CreateTableAsync(QueueSettings settings, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(settings);
        ArgumentNullException.ThrowIfNull(player);
        settings.Channel = 0;
        settings.Message = 0;
        if (Check(settings, out var found) is { } refusal) return QueueSeating.Refused(refusal);

        string id = DraftHost.PlayerId(player);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (Tables().FirstOrDefault(forming => forming.Table.Seats(id)) is { } at) return Elsewhere(at, web: true);
            if (Busy(id, player, web: true) is { } busy) return busy;

            settings.Updated = _options.TimeProvider.GetUtcNow();
            (Forming? forming, string? failed) = await OpenTable(settings, found).ConfigureAwait(false);
            if (forming is null) return QueueSeating.Refused(failed!);
            _own.Add(forming);
            return Sit(forming, player);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>Every table not yet gone: the queues', and the players' own. Called under the lock.</summary>
    private IEnumerable<Forming> Tables() => _queues.Values.SelectMany(queue => queue.Tables).Concat(_own).Where(forming => !forming.Gone);

    /// <summary>Why a player at no table cannot take a seat: they are in a draft. Called under the lock.</summary>
    private QueueSeating? Busy(string id, DiscordPlayer player, bool web) =>
        _starting.Contains(id) || _options.Drafts.InDraft(player.Id) || _options.Pods?.InDraft(player.Id) == true
            ? QueueSeating.Refused(web ? "You're already in a draft. Press Back to my draft for your seat." : "You're already in a draft. Use **/cantrip mydraft** for a link back to it.")
            : null;

    /// <summary>A player with a seat at another table than the one they asked for, told where it is.</summary>
    private static QueueSeating Elsewhere(Forming at, bool web) => QueueSeating.Refused(
        web ? "You already have a seat at another table. Press Open my table for its link, or leave that table first."
        : at.Settings.Channel == 0 ? "You already have a seat at a table on the draft queue page. Open it from there, or leave that table first."
        : $"You already have a seat at a table in <#{at.Settings.Channel}>. Press **Join** there for its link, or leave that table first.");

    /// <summary>Seats a player at a table with a seat free. Called under the lock.</summary>
    private static QueueSeating Sit(Forming forming, DiscordPlayer player)
    {
        string id = DraftHost.PlayerId(player);
        string name = Matchmaker.SeatName(player.Name);
        forming.Players[id] = player with { Name = name };
        string ticket = forming.Table.Reserve(id, name) ?? throw new InvalidOperationException("A free seat was not free.");
        return Seating(forming, id, ticket);
    }

    private static QueueSeating Seating(Forming forming, string id, string ticket) =>
        new(forming.Address, ticket, forming.Table.Players.First(seat => seat.PlayerId == id).Seat, null);

    /// <summary>
    /// Opens a new table on a free port, drafting <paramref name="settings"/> from <paramref name="found"/>; the caller puts
    /// it in its queue, or with the players' own. Called under the lock.
    /// </summary>
    private async Task<(Forming? Forming, string? Refusal)> OpenTable(QueueSettings settings, (string Cube, IReadOnlyList<string> Cards, IReadOnlyList<string> PickOrder) found)
    {
        (string cube, IReadOnlyList<string> cubeCards, IReadOnlyList<string> pickOrder) = found;
        Forming? made = null;
        bool booster = settings.Format == DraftFormats.Booster;
        bool grid = settings.Format == DraftFormats.Grid;
        string title = booster ? $"Booster draft: {cube}" : grid ? $"Grid draft: {cube}" : $"Winston draft: {cube}";
        int firstBot = settings.PlayerSeats + 1;
        var table = new QueueTable(new QueueTableSetup
        {
            Name = title,
            Seats = settings.Seats,
            Bots = [.. Enumerable.Range(firstBot, settings.Bots).Select(seat => (BotSeats.Id(seat), settings.Bots == 1 ? "Bot" : $"Bot {seat - firstBot + 1}"))],
            ReadyWindow = _options.ReadyWindow,
            TimeProvider = _options.TimeProvider,
            Start = players =>
            {
                if (booster)
                {
                    CreatedEvent eight = BoosterPod.Create(_options.Store, ContentCache.Get(_options.ContentRoot), new BoosterDraftSetup
                    {
                        Name = title,
                        Cube = cube,
                        CubeCards = cubeCards,
                        PickOrder = pickOrder,
                        Packs = settings.PacksDrafted,
                        PackSize = settings.PackSize,
                        Rounds = settings.Rounds,
                        MinimumDeck = settings.MinimumDeck,
                        Players = players,
                    }, _options.TimeProvider);
                    return BoosterPod.Open(_options.Store, eight.Record.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider, _options.PodTiming);
                }

                CreatedEvent created = grid
                    ? DraftPod.CreateGrid(_options.Store, ContentCache.Get(_options.ContentRoot), new GridDraftSetup
                    {
                        Name = title,
                        Cube = cube,
                        CubeCards = cubeCards,
                        PickOrder = pickOrder,
                        Packs = settings.PacksDrafted,
                        MinimumDeck = settings.MinimumDeck,
                        Players = players,
                    }, _options.TimeProvider)
                    : DraftPod.CreateWinston(_options.Store, ContentCache.Get(_options.ContentRoot), new WinstonDraftSetup
                    {
                        Name = title,
                        Cube = cube,
                        CubeCards = cubeCards,
                        PickOrder = pickOrder,
                        Piles = settings.Piles,
                        Cards = settings.Cards,
                        MinimumDeck = settings.MinimumDeck,
                        Players = players,
                    }, _options.TimeProvider);
                DraftPod pod = DraftPod.Open(_options.Store, created.Record.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider);
                DiscordPlayer host = Seated(made!, players[0].Id, players[0].Name);
                DiscordPlayer guest = Seated(made!, players[1].Id, players[1].Name);
                pod.Keep(DraftHost.HostKey, $"{host.Id}:{host.Name}");
                pod.Keep(DraftHost.GuestKey, $"{guest.Id}:{guest.Name}");
                pod.Keep(DraftHost.QueueKey, settings.Channel.ToString(System.Globalization.CultureInfo.InvariantCulture));
                return pod;
            },
        });

        HashSet<int> taken = [.. Tables().Select(forming => forming.Port)];
        List<string> failures = [];
        foreach (int port in _options.Ports.Where(port => !taken.Contains(port)))
        {
            try
            {
                GameServer server = await GameServer.StartAsync(new GameServerOptions
                {
                    Port = port,
                    Players = settings.Seats,
                    ContentRoot = _options.ContentRoot,
                    ListenAddress = _options.ListenAddress,
                    TimeProvider = _options.TimeProvider,
                    Event = table,
                    Portraits = _options.Portraits,
                    Art = _options.Art,
                }).ConfigureAwait(false);
                made = new Forming(table, server, port, _options.TableAddress?.Invoke(server.Port) ?? new Uri($"ws://{_options.PublicHost}:{server.Port}/game"), settings, cube);
                Forming watched = made;
                table.Changed += changedTable => _ = Task.Run(() => TableChangedAsync(settings.Channel, watched));

                // A table whose server stops while it is still forming - a fault - is taken out of the queue, so nobody is
                // sent to it again.
                _ = server.Finished.ContinueWith(_ => Task.Run(() => DroppedAsync(settings.Channel, watched)), TaskScheduler.Default);
                return (made, null);
            }
            catch (IOException error)
            {
                failures.Add($"{port}: {error.Message}");
            }
        }

        return (null, failures.Count == 0
            ? "Every draft table is in use. Try again when a draft finishes."
            : "No draft table could be opened: " + string.Join("; ", failures));
    }

    /// <summary>
    /// Follows a table that changed: one whose draft has begun goes to the draft host, an empty one is closed, and the
    /// queue's tables are filled from one another, oldest first.
    /// </summary>
    private async Task TableChangedAsync(ulong channel, Forming changed)
    {
        (DraftPod Pod, Forming Forming, DiscordPlayer Host, DiscordPlayer Guest)? begun = null;
        (BoosterPod Pod, Forming Forming)? podBegun = null;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (changed.Gone) return;
            _queues.TryGetValue(channel, out Queue? queue);

            // Begun: handed to the draft host, even from a queue closed in the meantime - its draft goes on.
            if (changed.Table.Pod is { } formed)
            {
                changed.Gone = true;
                queue?.Tables.Remove(changed);
                _own.Remove(changed);
                if (formed is BoosterPod booster) podBegun = (booster, changed);
                else if (formed is DraftPod draft && draft.Players.OrderBy(player => player.Seat).ToList() is [var first, var second])
                    begun = (draft, changed, Seated(changed, first.PlayerId, first.Name), Seated(changed, second.PlayerId, second.Name));
                foreach (var player in changed.Table.Players) _starting.Add(player.PlayerId);
            }
            else if (changed.Table.Taken == 0)
            {
                // Not closed at once: its last player is still to be told why they left it, or where they were moved to.
                _ = Task.Run(() => CloseIfEmptyAsync(channel, changed));
            }

            if (queue is not null) Fill(queue);
        }
        finally
        {
            _gate.Release();
        }

        if (podBegun is { } pod)
        {
            try
            {
                await _options.Pods!.AdoptAsync(pod.Pod, pod.Forming.Server, pod.Forming.Port, pod.Forming.Address, channel).ConfigureAwait(false);
            }
            finally
            {
                await _gate.WaitAsync().ConfigureAwait(false);
                try { foreach (string id in pod.Forming.Players.Keys) _starting.Remove(id); }
                finally { _gate.Release(); }
            }
        }

        if (begun is { } handed)
        {
            try
            {
                await _options.Drafts.AdoptAsync(handed.Pod, handed.Forming.Server, handed.Forming.Port, handed.Forming.Address, handed.Host, handed.Guest, channel).ConfigureAwait(false);
            }
            finally
            {
                await _gate.WaitAsync().ConfigureAwait(false);
                try { foreach (string id in handed.Forming.Players.Keys) _starting.Remove(id); }
                finally { _gate.Release(); }
            }
        }

        await Raise(channel).ConfigureAwait(false);
    }

    /// <summary>Who is in a seat of a table that has begun: the Discord player who joined for it, or a bot.</summary>
    private static DiscordPlayer Seated(Forming forming, string playerId, string name) =>
        forming.Players.GetValueOrDefault(playerId) ?? BotSeats.Player(name);

    /// <summary>
    /// Moves players still waiting at newer tables into the free seats of older ones, so a queue's players are never
    /// spread across tables that each wait for one more. A table in its ready check is full and left alone. Called under
    /// the lock.
    /// </summary>
    private static void Fill(Queue queue)
    {
        List<Forming> forming = [.. queue.Tables.Where(each => !each.Gone && each.Table.Forming).OrderBy(each => each.Table.Created)];
        for (int older = 0; older < forming.Count; older++)
        {
            Forming into = forming[older];

            // Only into a table someone is waiting at - an emptied one is about to close - and only players who have their
            // page open, who are the ones that can follow the move.
            while (into.Table.Forming && into.Table.Taken > 0 && into.Table.Taken < into.Seats)
            {
                Forming? from = forming.Skip(older + 1).LastOrDefault(each =>
                    each.Table.Forming && !each.Table.Checking && each.Table.Players.Any(player => player.Connected));
                if (from is null) return;

                (int _, string playerId, string name, DateTimeOffset _, bool _) = from.Table.Players.Where(player => player.Connected).OrderBy(player => player.Joined).First();
                DiscordPlayer player = from.Players[playerId];
                string? ticket = into.Table.Reserve(playerId, name);
                if (ticket is null) break;
                into.Players[playerId] = player;
                from.Players.Remove(playerId);
                from.Table.Release(playerId, new TableMoveMessage { Server = into.Address.AbsoluteUri, Ticket = ticket });
            }
        }
    }

    /// <summary>A table whose server stopped before its draft began: out of the queue, and its players told where they stand.</summary>
    private async Task DroppedAsync(ulong channel, Forming forming)
    {
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (forming.Gone || !forming.Table.Forming) return;
            if (_queues.TryGetValue(channel, out Queue? queue)) queue.Tables.Remove(forming);
            _own.Remove(forming);
            Console.Error.WriteLine($"A forming draft table on port {forming.Port} stopped: {forming.Server.FaultDetail}");
            await Close(forming).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }

        await Raise(channel).ConfigureAwait(false);
    }

    /// <summary>How long an empty table stays up, so the last of its players is sent the word about where they went.</summary>
    private static readonly TimeSpan EmptyLinger = TimeSpan.FromSeconds(3);

    /// <summary>Closes a table that is still empty a moment after it emptied; one that somebody has joined since stays.</summary>
    private async Task CloseIfEmptyAsync(ulong channel, Forming forming)
    {
        await Task.Delay(EmptyLinger).ConfigureAwait(false);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (forming.Gone || forming.Table.Taken > 0 || !forming.Table.Forming) return;
            if (_queues.TryGetValue(channel, out Queue? queue)) queue.Tables.Remove(forming);
            _own.Remove(forming);
            await Close(forming).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }
    }

    private static async Task Close(Forming forming)
    {
        if (forming.Gone) return;
        forming.Gone = true;
        await forming.Server.DisposeAsync().ConfigureAwait(false);
    }

    /// <summary>A queue's cube, by name in the cube lists only: a server manager names a cube, not a file on this machine.</summary>
    private (string Cube, IReadOnlyList<string> Cards, IReadOnlyList<string> PickOrder) Cube(QueueSettings settings)
    {
        if (string.IsNullOrWhiteSpace(settings.Cube) || settings.Cube.IndexOfAny(['/', '\\', ':']) >= 0 || settings.Cube.Contains("..", StringComparison.Ordinal))
            throw new ArgumentException("Name a cube from the cube lists, such as LSVCube.");

        string? path = _options.CubeDirectories
            .SelectMany(directory => new[] { Path.Combine(directory, settings.Cube), Path.Combine(directory, settings.Cube + ".txt") })
            .FirstOrDefault(File.Exists);
        if (path is null)
            throw new ArgumentException($"There is no cube called {settings.Cube}.");

        return (Path.GetFileNameWithoutExtension(path), CubeList.Load(path), CubeList.PickOrder(path));
    }

    private async Task Raise(ulong channel)
    {
        if (channel == 0 || Changed is not { } changed) return;
        foreach (Func<ulong, Task> handler in changed.GetInvocationList().Cast<Func<ulong, Task>>())
            await handler(channel).ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            // Players at tables still forming are told before the tables go, rather than seeing the connection drop.
            List<Forming> forming = [.. Tables()];
            foreach (Forming each in forming) each.Table.Close("The draft queues are restarting. Press Join again in a moment.");
            if (forming.Count > 0) await Task.Delay(TimeSpan.FromMilliseconds(500)).ConfigureAwait(false);
            foreach (Forming each in forming) await Close(each).ConfigureAwait(false);
            _queues.Clear();
            _own.Clear();
        }
        finally
        {
            _gate.Release();
        }
    }
}
