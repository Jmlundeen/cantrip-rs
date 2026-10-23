using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Discord;

public sealed class DraftHostOptions
{
    public required string ContentRoot { get; init; }

    /// <summary>Where events and players are kept: the server's data directory, which is backed up.</summary>
    public required EventStore Store { get; init; }

    /// <summary>Each player's picture, by player id, as their tables show it (<see cref="GameServerOptions.Portraits"/>); null for none.</summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>The cube's name, for the messages and the record.</summary>
    public required string Cube { get; init; }

    /// <summary>The cube's cards by name.</summary>
    public required IReadOnlyList<string> CubeCards { get; init; }

    /// <summary>The cube's cards best first (<see cref="CubeList.PickOrder"/>), for a bot to draft by; empty for none.</summary>
    public IReadOnlyList<string> PickOrder { get; init; } = [];

    /// <summary>How many piles a Winston draft lays out.</summary>
    public int Piles { get; init; } = 3;

    /// <summary>How many cards are drafted, drawn at random from the cube; zero for all of it.</summary>
    public int Cards { get; init; }

    public int MinimumDeck { get; init; } = 40;

    /// <summary>
    /// The ports draft tables may listen on, one table to a port. A draft holds its table from the moment both players
    /// are in until its game is over, however long that takes, so this is how many drafts can be under way at once.
    /// </summary>
    public required IReadOnlyList<int> Ports { get; init; }

    public string ListenAddress { get; init; } = "127.0.0.1";

    public string PublicHost { get; init; } = "localhost";

    /// <summary>
    /// The address players are given for the table on a port, when not <c>ws://{PublicHost}:{port}/game</c>: behind the
    /// bot's one public URL, <c>wss://host/table/{port}/game</c> (<see cref="TableProxy"/>).
    /// </summary>
    public Func<int, Uri>? TableAddress { get; init; }

    /// <summary>Where card art is, said by every table to the clients at it (<see cref="GameServerOptions.Art"/>); null for not said.</summary>
    public Uri? Art { get; init; }

    /// <summary>
    /// The site's page its players go back to once a table's game is over (<see cref="GameServerOptions.Return"/>); null
    /// when the bot serves no pages.
    /// </summary>
    public ReturnPage? Return { get; init; }

    /// <summary>How long a draft may wait for its second player.</summary>
    public TimeSpan OpenTimeout { get; init; } = TimeSpan.FromMinutes(30);

    /// <summary>How long a finished draft's table stays up, so both players can see the result.</summary>
    public TimeSpan Linger { get; init; } = TimeSpan.FromMinutes(2);

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}

/// <summary>
/// One Winston draft a Discord player has opened: waiting for a second player, then an event in the data directory
/// hosted at a table of its own. Changed only under the host's lock.
/// </summary>
public sealed class HostedDraft
{
    internal HostedDraft(string id, DiscordPlayer host, DateTimeOffset createdAt)
    {
        Id = id;
        Host = host;
        CreatedAt = createdAt;
    }

    /// <summary>The event's id, chosen when the draft is opened so its message's buttons name it throughout.</summary>
    public string Id { get; }

    public DiscordPlayer Host { get; }
    public DiscordPlayer? Guest { get; internal set; }
    public DateTimeOffset CreatedAt { get; }

    /// <summary>Where the event stands; NotActive until both players are in.</summary>
    public EventStage Stage { get; internal set; } = EventStage.NotActive;

    /// <summary>A line on how it stands or how it ended.</summary>
    public string Detail { get; internal set; } = string.Empty;

    /// <summary>The WebSocket address of its table, once it has one.</summary>
    public Uri? Address { get; internal set; }

    /// <summary>The queue channel it was formed in, for a draft that came from a queue; zero for one opened by hand.</summary>
    public ulong Queue { get; internal set; }

    /// <summary>
    /// Where the adapter posted the draft's public message, in its own terms ("channel:message" for Discord), kept with
    /// the event so the message can be found and brought up to date after a restart.
    /// </summary>
    public string? Message { get; internal set; }

    public IEnumerable<DiscordPlayer> Players => Guest is null ? [Host] : [Host, Guest];

    public bool Seats(ulong player) => Players.Any(seated => seated.Id == player);

    internal DraftPod? Pod { get; set; }
    internal GameServer? Server { get; set; }
    internal int Port { get; set; }
    internal DateTimeOffset? EndedAt { get; set; }
    internal bool Closed { get; set; }

    /// <summary>Calls off the bots playing its seats (<see cref="BotSeats"/>); null while none are.</summary>
    internal CancellationTokenSource? Bots { get; set; }

    /// <summary>Whether it is still to be played out.</summary>
    public bool IsLive => Stage is not (EventStage.Complete or EventStage.Cancelled);
}

/// <summary>What a player gets for opening, joining or asking again: the draft and a ticket to its table, or why not.</summary>
public sealed record DraftSeating(HostedDraft? Draft, string? Ticket, int Seat, string? Refusal)
{
    public static DraftSeating Refused(string why, HostedDraft? draft = null) => new(draft, null, 0, why);
}

/// <summary>
/// Hosts Winston drafts for Discord players, knowing nothing of Discord itself, as <see cref="Matchmaker"/> does games.
/// A draft opened by one player waits for a second; then it becomes an event in the data directory
/// (<see cref="DraftPod"/>) and gets a table of its own, which drafts, builds decks and plays the game.
/// <para>
/// Everything that matters is in the data directory, so nothing is lost to a restart: <see cref="ResumeAsync"/> hosts
/// every live draft again, wherever it had got to. A player's ticket is made fresh whenever they ask for it, and their
/// older ones keep working, so a player can come back from any computer at any time. What is only in memory is a draft
/// still waiting for its second player, which is cheap to open again.
/// </para>
/// <para>
/// A player may be in one live draft at a time.
/// </para>
/// </summary>
public sealed class DraftHost : IAsyncDisposable
{
    /// <summary>What <see cref="EventRecord.HostData"/> holds for a Discord draft.</summary>
    public const string HostKey = "discord.host";
    public const string GuestKey = "discord.guest";
    public const string MessageKey = "discord.message";
    public const string QueueKey = "discord.queue";

    private readonly DraftHostOptions _options;
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly Dictionary<string, HostedDraft> _drafts = new(StringComparer.Ordinal);

    public DraftHost(DraftHostOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        if (options.Ports.Count == 0) throw new ArgumentException("Give the draft host at least one port.", nameof(options));
        _options = options;
    }

    /// <summary>A draft moved on. Raised outside the lock; the adapter renders the draft's message from it.</summary>
    public event Func<HostedDraft, Task>? Changed;

    public HostedDraft? Find(string id)
    {
        _gate.Wait();
        try { return _drafts.GetValueOrDefault(id); }
        finally { _gate.Release(); }
    }

    /// <summary>Every draft the host knows of, live or lingering.</summary>
    public IReadOnlyList<HostedDraft> Drafts
    {
        get
        {
            _gate.Wait();
            try { return [.. _drafts.Values]; }
            finally { _gate.Release(); }
        }
    }

    /// <summary>
    /// Hosts every live Discord draft in the data directory again, as it stood: what a restarted bot does first.
    /// Returns the drafts it took up.
    /// </summary>
    public async Task<IReadOnlyList<HostedDraft>> ResumeAsync()
    {
        List<HostedDraft> resumed = [];
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            foreach (EventRecord record in _options.Store.LiveEvents().Where(record => record.Kind is EventKinds.WinstonDraft or EventKinds.GridDraft))
            {
                if (_drafts.ContainsKey(record.Id) || Player(record, HostKey) is not { } host || Player(record, GuestKey) is not { } guest)
                {
                    continue;
                }

                var draft = new HostedDraft(record.Id, host, record.Created)
                {
                    Guest = guest, Stage = record.Stage, Detail = record.Detail, Message = record.HostData.GetValueOrDefault(MessageKey),
                    Queue = ulong.TryParse(record.HostData.GetValueOrDefault(QueueKey), out ulong queue) ? queue : 0,
                };

                // Known either way; one with no table yet gets one when a player asks for their ticket.
                _drafts.Add(draft.Id, draft);
                if (await OpenTable(draft).ConfigureAwait(false) is { } refusal)
                {
                    Console.Error.WriteLine($"Draft {record.Id} could not be hosted again yet: {refusal}");
                    continue;
                }

                resumed.Add(draft);
            }
        }
        finally
        {
            _gate.Release();
        }

        return resumed;
    }

    /// <summary>Opens a draft for its first player, who waits for a second.</summary>
    public async Task<DraftSeating> OpenAsync(DiscordPlayer host)
    {
        ArgumentNullException.ThrowIfNull(host);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (Active(host.Id) is { } current)
                return DraftSeating.Refused("You are already in a draft. Finish or cancel it before opening another.", current);

            string id = EventStore.NewEventId(EventKinds.WinstonDraft, _options.TimeProvider.GetUtcNow());
            var draft = new HostedDraft(id, host with { Name = Matchmaker.SeatName(host.Name) }, _options.TimeProvider.GetUtcNow());
            _drafts.Add(id, draft);
            return new DraftSeating(draft, null, 1, null);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>
    /// Takes the second seat of an open draft, which makes the event and opens its table; or, for a player already in
    /// the draft, gives them a fresh ticket to their seat - from any computer, at any point in the event.
    /// </summary>
    public async Task<DraftSeating> JoinAsync(string draftId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        DraftSeating seating;
        bool changed = false;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_drafts.TryGetValue(draftId, out HostedDraft? draft) || !draft.IsLive)
                return DraftSeating.Refused("That draft is over.");

            if (draft.Seats(player.Id))
            {
                // Both players in but no table free when the second joined: try again now.
                if (draft.Pod is null && draft.Guest is not null && await OpenTable(draft).ConfigureAwait(false) is { } busy)
                    return DraftSeating.Refused(busy, draft);
                if (draft.Pod is null)
                    return DraftSeating.Refused("Your draft starts when a second player joins. Press **Join** again then for your seat.", draft);
                int seat = draft.Host.Id == player.Id ? 1 : 2;
                return new DraftSeating(draft, draft.Pod.IssueTicket(PlayerId(player)), seat, null);
            }

            if (draft.Guest is not null)
                return DraftSeating.Refused("That draft already has both its players.", draft);
            if (Active(player.Id) is { } current)
                return DraftSeating.Refused("You are already in another draft. Finish or cancel it first.", current);

            draft.Guest = player with { Name = Matchmaker.SeatName(player.Name) };
            CreatedEvent created;
            try
            {
                created = DraftPod.CreateWinston(_options.Store, ContentCache.Get(_options.ContentRoot), new WinstonDraftSetup
                {
                    Id = draft.Id,
                    Name = $"Winston draft: {_options.Cube}",
                    Cube = _options.Cube,
                    CubeCards = _options.CubeCards,
                    PickOrder = _options.PickOrder,
                    Piles = _options.Piles,
                    Cards = _options.Cards,
                    MinimumDeck = _options.MinimumDeck,
                    Players = [(PlayerId(draft.Host), draft.Host.Name), (PlayerId(draft.Guest), draft.Guest.Name)],
                }, _options.TimeProvider);
            }
            catch (Exception error) when (error is ArgumentException or InvalidOperationException or IOException)
            {
                draft.Guest = null;
                return DraftSeating.Refused("The draft could not be made: " + error.Message, draft);
            }

            // Who the players are on Discord, kept with the event before anything else can go wrong, so a restart finds them.
            DraftPod made = DraftPod.Open(_options.Store, draft.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider);
            made.Keep(HostKey, $"{draft.Host.Id}:{draft.Host.Name}");
            made.Keep(GuestKey, $"{draft.Guest.Id}:{draft.Guest.Name}");
            if (draft.Message is { } message) made.Keep(MessageKey, message);

            if (await OpenTable(draft).ConfigureAwait(false) is { } refusal)
            {
                // The event is made and kept; it is hosted when a table is free, at the latest on the next restart.
                draft.Detail = refusal;
                return DraftSeating.Refused(refusal, draft);
            }

            changed = true;
            seating = new DraftSeating(draft, created.Tickets[PlayerId(draft.Guest)], 2, null);
        }
        finally
        {
            _gate.Release();
        }

        if (changed) await Raise(seating.Draft!).ConfigureAwait(false);
        return seating;
    }

    /// <summary>Calls a draft off. Its host may; so may a moderator. Returns a refusal, or null once it is cancelled.</summary>
    public async Task<string?> CancelAsync(string draftId, DiscordPlayer by, bool moderator)
    {
        ArgumentNullException.ThrowIfNull(by);
        HostedDraft? draft;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_drafts.TryGetValue(draftId, out draft) || !draft.IsLive) return "That draft is already over.";
            if (draft.Host.Id != by.Id && !moderator) return "Only the draft's host or a server manager can cancel it.";
            string detail = $"Cancelled by {by.Name}.";
            draft.Pod?.Cancel(detail);
            End(draft, EventStage.Cancelled, detail);
            await Close(draft).ConfigureAwait(false);
        }
        finally
        {
            _gate.Release();
        }

        await Raise(draft).ConfigureAwait(false);
        return null;
    }

    /// <summary>
    /// A player drops out of their draft (<see cref="DraftPod.Drop"/>): a draft of two has nothing left to play without
    /// them, so it is called off - or, with its game being played, the game is forfeited to the other player. One still
    /// waiting for its second player is simply called off. The table stays up a while for the other player to see why,
    /// as a finished draft's does. Null once they have dropped; otherwise why not.
    /// </summary>
    public async Task<string?> DropAsync(string draftId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        HostedDraft? draft;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_drafts.TryGetValue(draftId, out draft) || !draft.IsLive) return "That draft is over.";
            if (!draft.Seats(player.Id)) return "You are not in that draft.";
            string id = PlayerId(player);
            if (draft.Pod is { } pod)
            {
                if (pod.Drop(id) is { } refusal) return refusal;
                if (pod.Stage is EventStage.Complete or EventStage.Cancelled) End(draft, pod.Stage, pod.Detail);
                draft.Detail = pod.Detail;
            }
            else if (draft.Guest is null)
            {
                End(draft, EventStage.Cancelled, $"{draft.Host.Name} dropped before anyone joined.");
            }
            else
            {
                // Made, but with no table free yet to host it: dropped in the data directory, where it would be hosted from.
                DraftPod waiting;
                try
                {
                    waiting = DraftPod.Open(_options.Store, draft.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider);
                }
                catch (Exception error) when (error is InvalidOperationException or IOException or System.Text.Json.JsonException)
                {
                    return error.Message;
                }

                if (waiting.Drop(id) is { } refusal) return refusal;
                End(draft, waiting.Stage, waiting.Detail);
            }
        }
        finally
        {
            _gate.Release();
        }

        await Raise(draft).ConfigureAwait(false);
        return null;
    }

    /// <summary>
    /// Drops drafts that never got a second player and closes finished ones once their result has been seen. A draft
    /// under way is never expired here: its own clocks (<see cref="DraftTiming"/>) see that it ends.
    /// </summary>
    public async Task SweepAsync()
    {
        List<HostedDraft> expired = [];
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            DateTimeOffset now = _options.TimeProvider.GetUtcNow();
            foreach (HostedDraft draft in _drafts.Values.ToArray())
            {
                if (draft.Guest is null && draft.IsLive && now - draft.CreatedAt >= _options.OpenTimeout)
                {
                    End(draft, EventStage.Cancelled, "Nobody joined in time.");
                    expired.Add(draft);
                }
                else if (!draft.IsLive && !draft.Closed && now - draft.EndedAt >= _options.Linger)
                {
                    await Close(draft).ConfigureAwait(false);
                }

                if (!draft.IsLive && (draft.Closed || draft.Server is null)) _drafts.Remove(draft.Id);
            }
        }
        finally
        {
            _gate.Release();
        }

        foreach (HostedDraft draft in expired) await Raise(draft).ConfigureAwait(false);
    }

    /// <summary>Notes where a draft's public message is, and keeps it with the event once there is one.</summary>
    public async Task NoteMessageAsync(HostedDraft draft, string message)
    {
        ArgumentNullException.ThrowIfNull(draft);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            draft.Message = message;
            draft.Pod?.Keep(MessageKey, message);
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>Whether a player is in a live draft, here or by their record in the data directory.</summary>
    public bool InDraft(ulong player)
    {
        _gate.Wait();
        try { return Active(player) is not null; }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// Takes over a table formed from a queue once its draft has begun: from then on it is a draft like any other, with
    /// its tickets, its end and its place in a restart.
    /// </summary>
    public async Task AdoptAsync(DraftPod pod, GameServer server, int port, Uri address, DiscordPlayer host, DiscordPlayer guest, ulong queue)
    {
        ArgumentNullException.ThrowIfNull(pod);
        ArgumentNullException.ThrowIfNull(server);
        HostedDraft draft;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            draft = new HostedDraft(pod.EventId, host, _options.TimeProvider.GetUtcNow())
            {
                Guest = guest, Pod = pod, Server = server, Port = port, Address = address, Queue = queue,
                Stage = pod.Stage, Detail = pod.Detail,
            };
            _drafts[draft.Id] = draft;
            Watch(draft);
            PlayBots(draft);
        }
        finally
        {
            _gate.Release();
        }

        await Raise(draft).ConfigureAwait(false);
    }

    /// <summary>A Discord account as a player id in the data directory.</summary>
    public static string PlayerId(DiscordPlayer player) => $"discord-{player.Id}";

    /// <summary>Opens a table for a draft's event, on a free port. Returns why not, or null. Called under the lock.</summary>
    private async Task<string?> OpenTable(HostedDraft draft)
    {
        DraftPod pod;
        try
        {
            pod = DraftPod.Open(_options.Store, draft.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider);
        }
        catch (Exception error) when (error is InvalidOperationException or IOException or System.Text.Json.JsonException)
        {
            return error.Message;
        }

        HashSet<int> taken = [.. _drafts.Values.Where(other => other.Server is not null && !other.Closed).Select(other => other.Port)];
        List<string> failures = [];
        foreach (int candidate in _options.Ports.Where(candidate => !taken.Contains(candidate)))
        {
            try
            {
                draft.Server = await GameServer.StartAsync(new GameServerOptions
                {
                    Format = Cantrip.Decks.DeckFormat.Limited(),
                    Port = candidate,
                    Players = pod.Seats,
                    ContentRoot = _options.ContentRoot,
                    ListenAddress = _options.ListenAddress,
                    TimeProvider = _options.TimeProvider,
                    Event = pod,
                    Portraits = _options.Portraits,
                    Art = _options.Art,
                    Return = _options.Return,
                }).ConfigureAwait(false);
                draft.Port = candidate;
                draft.Pod = pod;
                draft.Address = _options.TableAddress?.Invoke(draft.Server.Port) ?? new Uri($"ws://{_options.PublicHost}:{draft.Server.Port}/game");
                draft.Stage = pod.Stage;
                draft.Detail = pod.Detail;
                Watch(draft);
                PlayBots(draft);
                return null;
            }
            catch (IOException error)
            {
                failures.Add($"{candidate}: {error.Message}");
            }
        }

        return failures.Count == 0
            ? "Every draft table is in use. Try again when a draft finishes."
            : "No draft table could be opened: " + string.Join("; ", failures);
    }

    /// <summary>
    /// Sits a bot down in each of the draft's seats a queue kept for one, at the table it has just been given. Called
    /// under the lock.
    /// </summary>
    private void PlayBots(HostedDraft draft)
    {
        if (draft.Pod is not { } pod || draft.Address is not { } address || draft.Bots is not null || !draft.IsLive) return;
        List<(string PlayerId, string Name, int Seat)> bots = [.. pod.Players.Where(player => BotSeats.IsBot(player.PlayerId))];
        if (bots.Count == 0) return;
        draft.Bots = new CancellationTokenSource();
        CancellationToken cancellation = draft.Bots.Token;
        foreach ((string id, string name, int _) in bots)
        {
            string ticket = pod.IssueTicket(id);
            _ = Task.Run(() => BotSeats.PlayDraftAsync(address, _options.ListenAddress, ticket, name, cancellation), CancellationToken.None);
        }
    }

    /// <summary>Follows a draft's table: the event's stage as it moves on, and its end.</summary>
    private void Watch(HostedDraft draft)
    {
        GameServer server = draft.Server!;
        _ = Task.Run(async () =>
        {
            // The stage moves on inside the table (seated, drafting, building, playing), so it is looked at now and then
            // rather than raised: a message that says "drafting" a few seconds late is fine.
            while (!server.Finished.IsCompleted && !draft.Closed)
            {
                await Task.WhenAny(server.Finished, Task.Delay(TimeSpan.FromSeconds(2))).ConfigureAwait(false);
                await Update(draft, () =>
                {
                    if (!draft.IsLive || draft.Pod is null || draft.Server != server) return false;
                    EventStage now = draft.Pod.Stage;
                    if (now == draft.Stage) return false;

                    // Over at the table - its players never sat down, say - while its game was not played to the end:
                    // shown for a while, then closed, as a finished draft is.
                    if (now is EventStage.Complete or EventStage.Cancelled) End(draft, now, draft.Pod.Detail);
                    else draft.Stage = now;
                    draft.Detail = draft.Pod.Detail;
                    return true;
                }).ConfigureAwait(false);
            }

            await Update(draft, () =>
            {
                if (!draft.IsLive || draft.Server != server || draft.Closed) return false;

                // A table that failed leaves its event as it was in the data directory, to be hosted again on a restart.
                if (server.Finished.IsFaulted) End(draft, EventStage.Cancelled, "The table failed. The host's log has the details.");
                else End(draft, draft.Pod!.Stage, draft.Pod.Detail);
                return true;
            }).ConfigureAwait(false);
        });
    }

    private async Task Update(HostedDraft draft, Func<bool> change)
    {
        bool changed;
        await _gate.WaitAsync().ConfigureAwait(false);
        try { changed = change(); }
        finally { _gate.Release(); }

        if (changed) await Raise(draft).ConfigureAwait(false);
    }

    private void End(HostedDraft draft, EventStage stage, string detail)
    {
        draft.Stage = stage is EventStage.Complete or EventStage.Cancelled ? stage : EventStage.Complete;
        draft.Detail = detail;
        draft.EndedAt = _options.TimeProvider.GetUtcNow();
    }

    private static async Task Close(HostedDraft draft)
    {
        if (draft.Closed || draft.Server is null) return;
        draft.Closed = true;
        if (draft.Bots is { } bots) await bots.CancelAsync().ConfigureAwait(false);
        await draft.Server.DisposeAsync().ConfigureAwait(false);
    }

    /// <summary>
    /// A player's live draft, if they have one: one this host knows, or one their record in the data directory says they
    /// are in that is still live there.
    /// </summary>
    private HostedDraft? Active(ulong player)
    {
        // A player who has dropped from a draft is in it no more, though it plays on a moment to its end.
        string id = PlayerId(new DiscordPlayer(player, string.Empty));
        return _drafts.Values.FirstOrDefault(draft => draft.IsLive && draft.Seats(player) && draft.Pod?.HasDropped(id) != true)
            ?? _options.Store.LoadPlayer(id)?.Events
                .Select(entry => _drafts.GetValueOrDefault(entry.EventId))
                .FirstOrDefault(draft => draft is { IsLive: true } && draft.Pod?.HasDropped(id) != true);
    }

    /// <summary>Whether a draft being played is this host's, by its event id.</summary>
    public bool Hosts(string eventId)
    {
        _gate.Wait();
        try { return _drafts.TryGetValue(eventId, out HostedDraft? draft) && draft.IsLive; }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// The drafts opened from Discord that a player has started and nobody has joined yet: no event is made until someone
    /// does, so the data directory has nothing of them.
    /// </summary>
    public IReadOnlyList<HostedDraft> Waiting(ulong player)
    {
        _gate.Wait();
        try { return [.. _drafts.Values.Where(draft => draft.IsLive && draft.Guest is null && draft.Host.Id == player)]; }
        finally { _gate.Release(); }
    }

    /// <summary>
    /// A fresh ticket to the live draft a player is in, found from their record in the data directory: how a player
    /// gets back to their draft from any computer without finding its message.
    /// </summary>
    public async Task<DraftSeating> ReturnAsync(DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        string? id;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            id = Active(player.Id)?.Id;
        }
        finally
        {
            _gate.Release();
        }

        return id is null ? DraftSeating.Refused("You are not in a draft.") : await JoinAsync(id, player).ConfigureAwait(false);
    }

    /// <summary>A Discord player as the event kept them, "id:name".</summary>
    private static DiscordPlayer? Player(EventRecord record, string key) =>
        record.HostData.GetValueOrDefault(key) is { } kept && kept.Split(':', 2) is [{ } id, { } name] && ulong.TryParse(id, out ulong number)
            ? new DiscordPlayer(number, name)
            : null;

    private async Task Raise(HostedDraft draft)
    {
        if (Changed is not { } changed) return;
        foreach (Func<HostedDraft, Task> handler in changed.GetInvocationList().Cast<Func<HostedDraft, Task>>())
            await handler(draft).ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            foreach (HostedDraft draft in _drafts.Values) await Close(draft).ConfigureAwait(false);
            _drafts.Clear();
        }
        finally
        {
            _gate.Release();
        }
    }
}
