using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Security.Cryptography;
using System.Threading.Channels;
using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Rules;
using MessagePack;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;

namespace Cantrip.Server.Host;

public sealed class GameServerOptions
{
    public int Port { get; init; }
    public int Players { get; init; } = 2;
    public ulong Seed { get; init; } = 1;
    public required string ContentRoot { get; init; }
    /// <summary>Allowlisted deck catalog, selected by name rather than a client-supplied path.</summary>
    public IReadOnlyList<DeckList> Decks { get; init; } = [];
    /// <summary>Explicit test/tool bootstrap only. Normal hosting requires selection and readiness.</summary>
    public bool AutoStart { get; init; }

    /// <summary>
    /// A <c>.cantrip</c> scenario to play instead of the decks (docs/scenarios-in-the-browser.md): the table starts once
    /// its driven seats are seated and ready, the script answers every other seat, and each step's scenario part is
    /// raised as <see cref="GameServer.ScenarioProgressed"/>. <see cref="Players"/> must be the scenario's seat count.
    /// </summary>
    public ScenarioSpec? Scenario { get; init; }
    /// <summary>
    /// The rules engine the table deals through: the native engine's sidecar unless a test stands another in its place.
    /// </summary>
    public IRulesHostFactory Rules { get; init; } = RulesHostFactory.Native;
    public TimeSpan ReconnectGrace { get; init; } = TimeSpan.FromMinutes(5);
    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;

    /// <summary>
    /// The address to listen on. Loopback unless the host deliberately exposes the table to other machines, as a
    /// Discord-hosted table must; the transport is still plaintext, so anything beyond a trusted network wants TLS in
    /// front of it.
    /// </summary>
    public string ListenAddress { get; init; } = "127.0.0.1";

    /// <summary>
    /// Seats only players holding a ticket from <see cref="GameServer.Reserve"/>. A join without one is refused, so
    /// knowing the address is not enough to sit down.
    /// </summary>
    public bool RequireTickets { get; init; }

    /// <summary>
    /// The event this table is part of, or null for a table set up in its own lobby. An event's table seats only the
    /// event's players, by its tickets; it drafts and builds decks before it deals, and deals the decks they built.
    /// A player who drops out of an event is never released from it: their seat waits for them, however long, and their
    /// ticket brings them back to it from any computer.
    /// </summary>
    public ITableEvent? Event { get; init; }

    /// <summary>
    /// The picture of an event's player, by their id, as the client is to fetch it (<see cref="PortraitStore.PathFor"/>);
    /// null, or null for a player, for none. Only an event's players are known by id, so only they have pictures.
    /// </summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>Where card art is, told to every client at the table (<see cref="TableMessage.Art"/>); null for not said.</summary>
    public Uri? Art { get; init; }
}

/// <summary>How a table ended: the phase it ended in, the seats that won (none for a draw or an abandonment), and
/// why, when the table says.</summary>
public sealed record TableResult(TablePhase Phase, IReadOnlyList<int> WinningSeats, string Detail);

/// <summary>One local table. Membership, setup and game mutation belong to the single pump.
/// Credentials exist only in memory: socket reconnect is supported, process-crash recovery is not.</summary>
public sealed class GameServer : IAsyncDisposable
{
    private readonly GameServerOptions _options;
    private readonly WebApplication _app;
    private readonly ConcurrentQueue<(Connection Connection, ClientMessage? Message)> _inbound = new();
    private readonly ConcurrentDictionary<Connection, byte> _connections = new();
    private readonly SemaphoreSlim _connectionSlots = new(16, 16);
    private readonly Dictionary<int, Member> _members = [];
    private readonly Dictionary<string, DeckList> _decks;
    private readonly CancellationTokenSource _shutdown = new();
    private readonly TaskCompletionSource _finished = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly TaskCompletionSource _started = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly ConcurrentDictionary<int, Reservation> _reservations = new();
    private readonly SemaphoreSlim _work = new(0);
    private Task _pump = Task.CompletedTask;
    private GameSession? _session;
    private TablePhase _phase;
    private List<string> _sets = [];
    private string _detail = string.Empty;

    /// <summary>The event's <see cref="ITableEvent.Version"/> last shown, so a change made from outside is shown again.</summary>
    private int _eventVersion;

    /// <summary>
    /// Each seat's match clock while a game with one is played: how long it has left, as of <see cref="_clockAt"/>. Empty
    /// for a game with no clock.
    /// </summary>
    private readonly Dictionary<int, TimeSpan> _clock = [];

    /// <summary>The seats whose clocks are running: the ones the game is waiting on.</summary>
    private HashSet<int> _running = [];

    private DateTimeOffset _clockAt;

    /// <summary>
    /// What the pump is doing right now, and the last few things that happened to the table. Both exist for one
    /// reason: the pump catches everything, so a fault there arrives as a task that is simply faulted, with no
    /// indication of which of its four steps threw or what the table had just been asked to do. An intermittent
    /// fault that only shows up on a loaded machine cannot be read off a reproduction, so it has to be read off the
    /// failure itself.
    /// </summary>
    private string _step = "starting";
    private readonly Queue<string> _recent = new();

    private GameServer(GameServerOptions options, WebApplication app)
    {
        _options = options;
        _app = app;
        _decks = options.Decks.ToDictionary(deck => deck.Name, StringComparer.Ordinal);
    }

    public int Port { get; private set; }
    /// <summary>Completes on a rules result or abandonment. TableMessage.Phase distinguishes them.</summary>
    public Task Finished => _finished.Task;

    /// <summary>Completes when the cards are dealt. Never completes for a table abandoned or disposed in setup.</summary>
    public Task Started => _started.Task;

    /// <summary>How the table ended; null until <see cref="Finished"/> completes without a fault.</summary>
    public TableResult? Result { get; private set; }

    /// <summary>
    /// Holds a seat for one player and returns the ticket that claims it, which the host hands to that player alone.
    /// The name and deck are fixed here and cannot be changed from the seat; the player is ready as soon as they sit.
    /// <para>
    /// The ticket claims only an empty seat, or its own seat while that seat's connection is lost — which is how a
    /// player whose client died gets back in, having no reconnect proof. It never displaces a connected player.
    /// Safe to call from any thread.
    /// </para>
    /// </summary>
    public string Reserve(int seat, string name, string deck)
    {
        if (seat < 1 || seat > _options.Players) throw new ArgumentOutOfRangeException(nameof(seat), "No such seat at this table.");
        if (!ValidName(name)) throw new ArgumentException("A name is 1–32 printable characters.", nameof(name));
        if (!_decks.ContainsKey(deck)) throw new ArgumentException($"'{deck}' is not in this table's catalog.", nameof(deck));
        byte[] ticket = RandomNumberGenerator.GetBytes(32);
        if (!_reservations.TryAdd(seat, new Reservation(ticket, name.Trim(), deck)))
            throw new InvalidOperationException($"Seat {seat} is already reserved.");
        return Convert.ToHexString(ticket);
    }

    /// <summary>
    /// What the table was doing when the pump faulted, and what had happened to it just before: empty until it
    /// does. Written once, from the pump's own thread, before <see cref="Finished"/> is completed, so anything that
    /// sees a faulted task can read it. Whoever reports such a fault should say this as well as the exception —
    /// the exception says what broke and this says what was being done to the table at the time.
    /// </summary>
    public string FaultDetail { get; private set; } = string.Empty;

    public static async Task<GameServer> StartAsync(GameServerOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        // A game seats two to four; an event's table seats its pod, which drafts there and plays at tables of its own.
        int most = options.Event is null ? 4 : 8;
        if (options.Players < 2 || options.Players > most || options.ReconnectGrace <= TimeSpan.Zero || options.ReconnectGrace > TimeSpan.FromHours(1))
            throw new ArgumentException($"A table needs 2–{most} players and a reconnect grace between zero and one hour.", nameof(options));
        if (options.Decks.Count > 128 || options.Decks.Any(deck => string.IsNullOrWhiteSpace(deck.Name)
            || deck.Name.Length > 96 || deck.Count is < 1 or > 2000)
            || options.Decks.Select(deck => deck.Name).Distinct(StringComparer.Ordinal).Count() != options.Decks.Count
            || (options.Event is null && options.Scenario is null && options.Decks.Count == 0))
            throw new ArgumentException("Provide 1–128 uniquely named decks, each with 1–2000 cards.", nameof(options));
        if (options.Event is { } hosted && hosted.Seats != options.Players)
            throw new ArgumentException($"The event has {hosted.Seats} players; the table has {options.Players} seats.", nameof(options));
        if (options.Event is { Phase: TablePhase.Finished })
            throw new ArgumentException("That event is over; there is nothing to host.", nameof(options));

        WebApplicationBuilder builder = WebApplication.CreateBuilder();
        builder.Logging.SetMinimumLevel(LogLevel.Information);
        builder.WebHost.UseUrls($"http://{options.ListenAddress}:{options.Port}");
        WebApplication app = builder.Build();
        var server = new GameServer(options, app);
        // The sidecar's stderr is its log; the first table to want it gives it the server's.
        NativeHostProcess.SharedLog ??= line => app.Logger.LogInformation("cantrip-native-host: {Line}", line);
        app.UseWebSockets();
        app.Map("/game", server.HandleAsync);
        try
        {
            await app.StartAsync().ConfigureAwait(false);
        }
        catch
        {
            // A port someone else holds is an ordinary failure for a host trying a pool; the host it built goes too.
            await app.DisposeAsync().ConfigureAwait(false);
            throw;
        }
        server.Port = app.Urls.Select(address => new Uri(address).Port).First(port => port != 0);
        server.FollowEvent();
        server._pump = Task.Run(server.PumpAsync, CancellationToken.None);
        app.Logger.LogInformation("Cantrip table listening on {Address} port {Port} for {Players} players.", options.ListenAddress, server.Port, options.Players);
        app.Logger.LogInformation("Rules engine: native (the cantrip-native-host sidecar at {Path}).", NativeHostProcess.Locate());
        return server;
    }

    private async Task HandleAsync(HttpContext context)
    {
        if (!context.WebSockets.IsWebSocketRequest)
        {
            context.Response.StatusCode = StatusCodes.Status400BadRequest;
            return;
        }
        // A URL seat is not identity. Versioned first frames acquire or resume membership.
        if (context.Request.Query.ContainsKey("seat") || !_connectionSlots.Wait(0))
        {
            context.Response.StatusCode = StatusCodes.Status409Conflict;
            return;
        }
        try
        {
            using WebSocket socket = await context.WebSockets.AcceptWebSocketAsync().ConfigureAwait(false);
            await ServeAsync(socket).ConfigureAwait(false);
        }
        finally
        {
            _connectionSlots.Release();
        }
    }

    /// <summary>
    /// Serves one accepted socket for as long as its request lasts, and detaches the connection from it before
    /// returning — after which the caller disposes the socket and the request is torn down.
    /// <para>
    /// Separate from <see cref="HandleAsync"/> because everything here is about a socket and nothing about the
    /// request that produced one. That also lets a test serve a socket of its own, which is the only way to put a
    /// connection into a state the host cannot otherwise be asked for: one whose socket no longer has a request
    /// behind it while the table still holds it.
    /// </para>
    /// </summary>
    internal async Task ServeAsync(WebSocket socket)
    {
        var connection = new Connection(socket, _options.TimeProvider.GetUtcNow());
        _connections.TryAdd(connection, 0);
        Task writer = connection.WriteLoopAsync(_shutdown.Token);
        try
        {
            await connection.ReadLoopAsync(message => Queue(connection, message), _shutdown.Token).ConfigureAwait(false);
        }
        finally
        {
            connection.Complete();
            await writer.ConfigureAwait(false);
            _connections.TryRemove(connection, out _);
            connection.Detach();
            Queue(connection, null);
        }
    }

    private void Queue(Connection connection, ClientMessage? message)
    {
        _inbound.Enqueue((connection, message));
        _work.Release();
    }

    private async Task PumpAsync()
    {
        try
        {
            while (!_shutdown.IsCancellationRequested)
            {
                while (_inbound.TryDequeue(out var item))
                {
                    if (item.Message is null)
                    {
                        _step = "handling a disconnect";
                        Disconnected(item.Connection);
                    }
                    else
                    {
                        _step = $"receiving {item.Message.GetType().Name}";
                        Receive(item.Connection, item.Message);
                    }
                }

                _step = "expiring memberships";
                ExpireMemberships();

                _step = "starting the table";
                TryStart();

                _step = "flushing to the seats";
                Flush();

                // After the flush, so a seat is sent the question before word that its clock runs for it.
                _step = "running the clocks";
                TickClocks();

                _step = "checking whether the game is over";
                if (_session is { IsOver: true } && _phase == TablePhase.Playing)
                {
                    _phase = TablePhase.Finished;
                    BroadcastTable();
                    Finish();
                }

                // Finished games remain available for resync until the host disposes the server.
                _step = "waiting for work";
                await _work.WaitAsync(TimeSpan.FromMilliseconds(100), _shutdown.Token).ConfigureAwait(false);
            }
        }
        catch (OperationCanceledException) when (_shutdown.IsCancellationRequested) { }
        catch (Exception error)
        {
            // Written before the task is faulted, so that anything waking on Finished already sees it. This is the
            // only account of the fault there will be: the pump is one loop behind a queue, so nothing above it
            // knows which step threw, and an intermittent fault gives no second chance to go and look.
            FaultDetail = $"Faulted while {_step}. {Describe()}";
            _app.Logger.LogError(error, "The table loop faulted while {Step}. {Table}", _step, Describe());
            _finished.TrySetException(error);
        }
    }

    /// <summary>
    /// The table in one line, for a fault log: the phase, what is connected, who is seated and whether they are
    /// still on a socket, and where the game has got to. Everything here is read from the pump's own thread.
    /// </summary>
    private string Describe()
    {
        string seats = _members.Count == 0
            ? "no members"
            : string.Join(", ", _members.Values.OrderBy(member => member.Seat).Select(member =>
                $"seat {member.Seat} {(member.Connection is null ? "dropped" + (member.DisconnectedAt is { } at ? $" at {at:HH:mm:ss.fff}" : string.Empty) : "connected")}"));

        string game = _session is null
            ? "no session"
            : $"turn {_session.Turn} {_session.Step}, {(_session.IsOver ? "over" : "in progress")}";

        return $"Phase {_phase}, {_connections.Count} connection(s), {seats}; {game}; "
            + $"{_inbound.Count} queued; shutdown {_shutdown.IsCancellationRequested}. "
            + $"Recently: {(_recent.Count == 0 ? "nothing" : string.Join(" -> ", _recent))}.";
    }

    /// <summary>
    /// Notes something the table did, keeping only the last handful. Read by <see cref="Describe"/> alone, so a
    /// fault says what led up to it without anyone having to turn debug logging on first — which, for a fault that
    /// shows up once in a dozen runs on a loaded machine, is a switch nobody gets to throw in time.
    /// </summary>
    private void Note(string what)
    {
        _recent.Enqueue(what);
        while (_recent.Count > 12)
        {
            _recent.Dequeue();
        }
    }

    private void Receive(Connection connection, ClientMessage message)
    {
        if (connection.Member is null)
        {
            switch (message)
            {
                case JoinMessage join: Join(connection, join); break;
                case ResumeMessage resume: Resume(connection, resume); break;
                default: Refuse(connection, ErrorCode.MembershipRequired, "Join or resume before sending commands."); break;
            }
            return;
        }
        Member member = connection.Member;
        // Superseded connections may still have queued frames. None retain authority.
        if (member.Connection != connection || !_members.ContainsKey(member.Seat))
        {
            Refuse(connection, ErrorCode.MembershipRefused, "This connection no longer owns the membership.");
            return;
        }
        switch (message)
        {
            case LeaveMessage or ReadyMessage when _options.Event is { Forming: true } forming:
                if (forming.Receive(member.Seat, message) is { } notNow) Refuse(connection, ErrorCode.InvalidSetup, notNow);
                Note($"seat {member.Seat}: {message.Describe()}");
                FollowVacated(forming);
                FollowEvent();
                break;
            case LeaveMessage when _options.Event is not null:
                Refuse(connection, ErrorCode.InvalidSetup, "A seat in an event is kept for its player; close the page and come back with your link.");
                break;
            case LeaveMessage:
                Depart(member, "A player left the table.");
                break;
            case DraftPickMessage or BuildDeckMessage when _options.Event is { } hosted && _phase is TablePhase.Drafting or TablePhase.DeckBuilding:
                if (hosted.Receive(member.Seat, message) is { } refusal)
                {
                    Refuse(connection, ErrorCode.InvalidSetup, refusal);
                    // Shown again as it stands, so a window that was behind catches up.
                    if (hosted.ViewFor(member.Seat) is { } view) connection.Send(view);
                    break;
                }
                Note($"seat {member.Seat}: {message.Describe()}");
                FollowEvent();
                break;
            case ReadyMessage or ConfigureTableMessage when _options.Event is not null:
                Refuse(connection, ErrorCode.InvalidSetup, "An event's table is set up by the event.");
                break;
            case ConfigurePlayerMessage when member.Reserved:
                Refuse(connection, ErrorCode.InvalidSetup, "This seat's name and deck were fixed when it was reserved.");
                break;
            case ConfigurePlayerMessage configure:
                if (!CanConfigure(connection)) break;
                if (!ValidName(configure.Name) || !_decks.ContainsKey(configure.Deck ?? string.Empty))
                {
                    Refuse(connection, ErrorCode.InvalidSetup, "Choose a name of 1–32 printable characters and a catalog deck.");
                    break;
                }
                member.Name = configure.Name.Trim();
                member.Deck = configure.Deck!; // The catalog check above excludes null and empty IDs.
                BroadcastTable();
                break;
            case ConfigureTableMessage configure:
                ConfigureSets(connection, member, configure);
                break;
            case ReadyMessage ready:
                if (_phase != TablePhase.Setup)
                {
                    Refuse(connection, ErrorCode.InvalidSetup, "The table is no longer being configured.");
                    break;
                }
                if (ready.Ready && !_options.AutoStart && !_decks.ContainsKey(member.Deck))
                {
                    Refuse(connection, ErrorCode.InvalidSetup, "Choose a deck before becoming ready.");
                    break;
                }
                member.Ready = ready.Ready;
                BroadcastTable();
                break;
            case ConcedeMessage when _phase == TablePhase.Playing:
                Concede(connection, member);
                break;
            case DropMessage when _options.Event is { } hosted && _phase == TablePhase.Pairings:
                if (hosted.Receive(member.Seat, message) is { } cannot)
                {
                    Refuse(connection, ErrorCode.InvalidSetup, cannot);
                    break;
                }
                Note($"seat {member.Seat}: {message.Describe()}");
                FollowEvent();
                break;
            case AnswerMessage answer when _phase == TablePhase.Playing:
                if (answer.Answer is null) Refuse(connection, ErrorCode.IllegalAnswer, "An answer payload is required.");
                else _session!.Submit(new PlayerId(member.Seat), answer);
                break;
            default:
                Refuse(connection, _phase is TablePhase.Finished or TablePhase.Abandoned ? ErrorCode.TableClosed : ErrorCode.IllegalAnswer,
                    "This message is not valid in the current table state.");
                break;
        }
    }

    /// <summary>
    /// A seat concedes the game (CR 104.3a). The game itself ends, so the pump finishes the table the way it does any
    /// game decided by its rules, and an event's match goes on to its next game as after any other loss.
    /// </summary>
    private void Concede(Connection connection, Member member)
    {
        var seat = new PlayerId(member.Seat);
        if (!_session!.CanConcede(seat))
        {
            Refuse(connection, ErrorCode.InvalidSetup, "Conceding is taken only when it ends the game, and this one would go on.");
            return;
        }

        Note($"seat {member.Seat} conceded");
        _detail = $"{member.Name} conceded.";
        _session.Concede(seat);
    }

    private void ConfigureSets(Connection connection, Member member, ConfigureTableMessage configure)
    {
        if (!CanConfigure(connection)) return;
        if (member.Seat != _members.Keys.Min())
        {
            Refuse(connection, ErrorCode.InvalidSetup, "Only the table host can select sets.");
            return;
        }
        try
        {
            if (configure.Sets is null || configure.Sets.Count > 16 || configure.Sets.Any(code => string.IsNullOrWhiteSpace(code) || code.Length > 64))
                throw new ArgumentException("Choose at most sixteen available set codes.");
            List<string> sets = [.. configure.Sets.Select(code => ContentCache.Get(_options.ContentRoot).Set(code).Code).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal)];
            _ = ContentCache.Get(_options.ContentRoot).Catalog([.. sets]);
            _sets = sets;
            BroadcastTable();
        }
        catch (Exception error) when (error is ArgumentException or KeyNotFoundException or InvalidOperationException)
        {
            Refuse(connection, ErrorCode.InvalidSetup, error.Message);
        }
    }

    private bool CanConfigure(Connection connection)
    {
        if (_phase != TablePhase.Setup || _members.Values.Any(member => member.Ready))
        {
            Refuse(connection, ErrorCode.InvalidSetup, "All players must be unready before changing setup.");
            return false;
        }
        return true;
    }

    private void Join(Connection connection, JoinMessage message)
    {
        if (!ProtocolMatches(connection, message.ProtocolVersion)) return;
        if (!string.IsNullOrEmpty(message.Ticket))
        {
            Claim(connection, message.Ticket);
            return;
        }
        if (_options.RequireTickets || _options.Event is not null)
        {
            Refuse(connection, ErrorCode.MembershipRefused, "This table seats only players holding a ticket from its host.");
            return;
        }
        // A reserved seat is held for its ticket, so an open join never lands in one.
        bool Open(int candidate) => !_members.ContainsKey(candidate) && !_reservations.ContainsKey(candidate);
        if (_phase != TablePhase.Setup || !Enumerable.Range(1, _options.Players).Any(Open))
        {
            Refuse(connection, ErrorCode.MembershipRefused, "No new membership is available.");
            return;
        }
        int seat = message.PreferredSeat == 0
            ? Enumerable.Range(1, _options.Players).First(Open)
            : message.PreferredSeat;
        if (seat < 1 || seat > _options.Players || !Open(seat) || !ValidName(message.PlayerName))
        {
            Refuse(connection, ErrorCode.MembershipRefused, "Choose an empty seat and a name of 1–32 printable characters.");
            return;
        }
        var member = new Member(seat, message.PlayerName.Trim())
        {
            Connection = connection, Ready = _options.AutoStart,
            Deck = _options.AutoStart && _options.Decks.Count > 0 ? _options.Decks[(seat - 1) % _options.Decks.Count].Name : string.Empty,
        };
        _members.Add(seat, member);
        connection.Member = member;
        SendMembership(connection, member);
        BroadcastTable();
    }

    /// <summary>
    /// Seats a ticket holder: in its empty seat during setup, or back into its own seat when that seat's connection
    /// has been lost. Tickets are compared in constant time, like reconnect proofs.
    /// </summary>
    private void Claim(Connection connection, string ticket)
    {
        if (_options.Event is { } hosted)
        {
            ClaimEventSeat(connection, hosted, ticket);
            return;
        }

        byte[]? presented = null;
        try { if (ticket.Length == 64) presented = Convert.FromHexString(ticket); }
        catch (FormatException) { }
        int seat = 0;
        Reservation? reservation = null;
        foreach ((int reserved, Reservation candidate) in _reservations)
        {
            if (presented is not null && CryptographicOperations.FixedTimeEquals(candidate.Ticket, presented))
                (seat, reservation) = (reserved, candidate);
        }
        if (reservation is null)
        {
            Refuse(connection, ErrorCode.MembershipRefused, "That ticket is not for this table.");
            return;
        }
        if (_members.TryGetValue(seat, out Member? held))
        {
            // Reserve runs off the pump, so an open join can have sat down in the seat before its reservation landed. That
            // seat is not the ticket's to take.
            if (held.Connection is not null || !held.Reserved)
            {
                Refuse(connection, ErrorCode.MembershipRefused, "That seat is already taken by another player.");
                return;
            }
            Note($"seat {seat} reclaimed by its ticket");
            held.Connection = connection;
            held.DisconnectedAt = null;
            if (_phase == TablePhase.Setup) held.Ready = true;
            connection.Member = held;
            SendMembership(connection, held);
            _session?.Resynchronize(new PlayerId(seat));
            BroadcastTable();
            return;
        }
        if (_phase != TablePhase.Setup)
        {
            Refuse(connection, ErrorCode.MembershipRefused, "The table is no longer taking players.");
            return;
        }
        var member = new Member(seat, reservation.Name) { Connection = connection, Deck = reservation.Deck, Ready = true, Reserved = true };
        _members.Add(seat, member);
        connection.Member = member;
        Note($"seat {seat} claimed by its ticket");
        SendMembership(connection, member);
        BroadcastTable();
    }

    /// <summary>
    /// Seats an event's player by their ticket, at any point in the event. Unlike a lobby's ticket, an event's takes the
    /// seat over from a connection that still holds it: the ticket is the player's own, and the other connection is most
    /// likely the same player on the computer they have just left.
    /// </summary>
    private void ClaimEventSeat(Connection connection, ITableEvent hosted, string ticket)
    {
        // Seats the host has given up since the pump last looked are let go first: one may be the seat this ticket is
        // for, given to a new player, and its old player must not be taken for them.
        FollowVacated(hosted);
        int seat = hosted.SeatFor(ticket);
        if (seat == 0)
        {
            Refuse(connection, ErrorCode.MembershipRefused, "That ticket is not for this event.");
            return;
        }

        if (_members.TryGetValue(seat, out Member? held))
        {
            Note($"seat {seat} taken up again by its ticket");
            Connection? previous = held.Connection;
            held.Connection = connection;
            held.DisconnectedAt = null;
            // A new proof, so the window the seat was taken from cannot resume its way back into it.
            held.Proof = RandomNumberGenerator.GetBytes(32);
            connection.Member = held;
            if (previous is not null && previous != connection)
            {
                // Told why, then closed once that has gone out, as a departure is: an abort could cut the reason off.
                previous.Send(new ErrorMessage { Code = ErrorCode.MembershipRefused, Detail = "This seat has been opened somewhere else." });
                previous.Complete();
            }
        }
        else
        {
            held = new Member(seat, hosted.NameAt(seat)) { Connection = connection, Ready = true, Reserved = true };
            _members.Add(seat, held);
            connection.Member = held;
            Note($"seat {seat} claimed by its ticket");
        }

        SendMembership(connection, held);
        hosted.Seated(seat);
        hosted.Connected(seat, true);
        FollowEvent();
        _session?.Resynchronize(new PlayerId(seat));
        if (_clock.Count > 0) connection.Send(Clocks());
    }

    /// <summary>
    /// Lets go of the seats a forming table gave up: each player is told why - or where their new table is - and their
    /// membership ends, so the page forgets the seat rather than trying to take it back.
    /// </summary>
    private void FollowVacated(ITableEvent hosted)
    {
        foreach ((int seat, ServerMessage farewell) in hosted.TakeVacated())
        {
            if (!_members.Remove(seat, out Member? member)) continue;
            Note($"seat {seat} given up");
            member.Connection?.Send(farewell);
            member.Connection?.Complete();
            member.Connection = null;
        }
    }

    /// <summary>
    /// Takes up where the event has got to: its phase while the cards are not dealt, and what each seat is shown of it.
    /// Called whenever the event may have moved on.
    /// </summary>
    private void FollowEvent()
    {
        if (_options.Event is not { } hosted)
        {
            return;
        }

        _eventVersion = hosted.Version;

        if (_phase is not (TablePhase.Playing or TablePhase.Finished or TablePhase.Abandoned))
        {
            _phase = hosted.Phase;
        }

        BroadcastTable();
        foreach (Member member in _members.Values)
        {
            if (member.Connection is { } connection && hosted.ViewFor(member.Seat) is { } view)
            {
                connection.Send(view);
            }
        }
    }

    private void Resume(Connection connection, ResumeMessage message)
    {
        if (!ProtocolMatches(connection, message.ProtocolVersion)) return;
        Member? member = _members.Values.FirstOrDefault(candidate => candidate.Id == message.MembershipId);
        byte[]? proof = null;
        try { if (message.ReconnectProof?.Length == 44) proof = Convert.FromBase64String(message.ReconnectProof); }
        catch (FormatException) { }
        if (_phase == TablePhase.Abandoned || member is null || proof is null
            || !CryptographicOperations.FixedTimeEquals(member.Proof, proof)
            || (_options.Event is null && member.DisconnectedAt is { } lost && _options.TimeProvider.GetUtcNow() - lost >= _options.ReconnectGrace))
        {
            Refuse(connection, ErrorCode.MembershipRefused, "Membership proof was refused or has expired.");
            return;
        }
        Connection? previous = member.Connection;
        member.Connection = connection;
        member.DisconnectedAt = null;
        connection.Member = member;
        previous?.Abort();
        SendMembership(connection, member);
        if (_session is not null) _session.Resynchronize(new PlayerId(member.Seat));
        _options.Event?.Connected(member.Seat, true);
        BroadcastTable();
        if (_options.Event?.ViewFor(member.Seat) is { } view) connection.Send(view);
        if (_clock.Count > 0) connection.Send(Clocks());
    }

    private static bool ValidName(string? name) => name is not null && name.Trim().Length is >= 1 and <= 32 && !name.Any(char.IsControl);
    private static bool ProtocolMatches(Connection connection, int version)
    {
        if (version == PlayerView.ProtocolVersion) return true;
        Refuse(connection, ErrorCode.ProtocolMismatch, $"Expected protocol {PlayerView.ProtocolVersion}.");
        return false;
    }
    private static void SendMembership(Connection connection, Member member) => connection.Send(new MembershipMessage
    {
        MembershipId = member.Id, ReconnectProof = Convert.ToBase64String(member.Proof), Seat = member.Seat,
    });

    private void Disconnected(Connection connection)
    {
        if (connection.Member is not { } member || member.Connection != connection)
        {
            Note("an unseated connection closed");
            return;
        }

        Note($"seat {member.Seat} dropped");
        member.Connection = null;
        member.DisconnectedAt = _options.TimeProvider.GetUtcNow();
        _options.Event?.Connected(member.Seat, false);
        if (_phase == TablePhase.Setup) member.Ready = false;
        BroadcastTable();
    }

    private void ExpireMemberships()
    {
        DateTimeOffset now = _options.TimeProvider.GetUtcNow();
        foreach (Connection connection in _connections.Keys)
            if (connection.Member is null && now - connection.CreatedAt >= TimeSpan.FromSeconds(15))
            {
                Note("aborted a connection that never joined");
                connection.Abort();
            }
        // An event's seat is never given up for being away: its player comes back by their ticket. A table still forming
        // gives up the seats of players who did not say ready in time, and the host may have changed who is seated.
        if (_options.Event is { } hosted)
        {
            hosted.Tick(now);
            if (_phase == TablePhase.Playing && _session is not null && hosted.TakeForfeits() is { Count: > 0 } forfeits)
            {
                Note($"seat(s) {string.Join(", ", forfeits)} forfeited");
                string absent = string.Join(" and ", forfeits.Select(hosted.NameAt));
                EndAgainst(forfeits, $"{absent} {(forfeits.Count == 1 ? "was" : "were")} not at the table.", ranOut: false);
            }

            FollowVacated(hosted);
            if (hosted.Version != _eventVersion) FollowEvent();
            return;
        }

        foreach (Member member in _members.Values.ToArray())
            if (member.DisconnectedAt is { } lost && now - lost >= _options.ReconnectGrace)
                Depart(member, "A player's reconnect grace expired.");
    }

    private void Depart(Member member, string detail)
    {
        Note($"seat {member.Seat} departed ({detail})");
        _members.Remove(member.Seat);
        member.Connection?.Send(new ErrorMessage { Code = ErrorCode.TableClosed, Detail = "Membership has been revoked." });
        member.Connection?.Complete();
        member.Connection = null;
        if (_phase == TablePhase.Playing)
        {
            _phase = TablePhase.Abandoned;
            _detail = detail;
            Finish();
        }
        // Unready so the others can review a changed table; a reserved seat has nothing to review, and its client never
        // shows a Ready to press again.
        foreach (Member remaining in _members.Values) remaining.Ready = remaining.Reserved;
        BroadcastTable();
    }

    private void TryStart()
    {
        if (_options.Event is { } hosted)
        {
            StartEventGame(hosted);
            return;
        }

        if (_options.Scenario is { } scenario)
        {
            StartScenario(scenario);
            return;
        }

        if (_phase != TablePhase.Setup || _members.Count != _options.Players || _members.Values.Any(member => !member.Ready || member.Connection is null)) return;
        List<Member> members = [.. _members.Values.OrderBy(member => member.Seat)];
        List<string> names = [.. members.Select(member => member.Name)];
        try
        {
            var session = new GameSession(BuildTable(members, names));
            session.Start();
            _session = session;
        }
        catch (Exception error) when (error is not OutOfMemoryException)
        {
            // Content/setup failures must not strand all members behind a dead pump. No partial session is published.
            foreach (Member member in members)
            {
                member.Ready = false;
                if (member.Connection is { } connection) Refuse(connection, ErrorCode.InvalidSetup, "The selected setup could not start: " + error.Message);
            }
            _app.Logger.LogWarning(error, "The selected table setup could not start.");
            Note("a table setup was refused");
            BroadcastTable();
            return;
        }
        Note($"the table started with {members.Count} seat(s)");
        _phase = TablePhase.Playing;
        BroadcastTable();
        _started.TrySetResult();
    }

    /// <summary>
    /// Deals a scenario once its driven seats are seated and ready; the seats the script plays need no one, and are named
    /// for the script's players (<c>Bot2</c>).
    /// </summary>
    private void StartScenario(ScenarioSpec scenario)
    {
        if (_phase != TablePhase.Setup
            || scenario.Driven.Any(seat => !_members.TryGetValue(seat, out Member? member) || !member.Ready || member.Connection is null))
        {
            return;
        }

        List<RulesSeat> seats = [.. Enumerable.Range(1, _options.Players)
            .Select(seat => new RulesSeat(_members.TryGetValue(seat, out Member? member) ? member.Name : $"Bot{seat}", []))];
        try
        {
            var session = new GameSession(_options.Rules.Create(seats, _options.Seed, new RulesGameOptions { Scenario = scenario }));
            session.ScenarioProgressed += progress => ScenarioProgressed?.Invoke(progress);
            session.Start();
            _session = session;
        }
        catch (Exception error) when (error is not OutOfMemoryException)
        {
            _app.Logger.LogWarning(error, "The scenario {Name} could not be dealt.", scenario.Name);
            Note("a scenario was refused");
            foreach (Member member in _members.Values)
                if (member.Connection is { } connection) Refuse(connection, ErrorCode.InvalidSetup, "The scenario could not start: " + error.Message);
            BroadcastTable();
            return;
        }

        Note($"the scenario {scenario.Name} started");
        _phase = TablePhase.Playing;
        BroadcastTable();
        _started.TrySetResult();
    }

    /// <summary>
    /// A scenario's part of each step of the game (<see cref="GameServerOptions.Scenario"/>), raised on the table's own
    /// thread as the step is published: what each driven seat is meant to answer, the client checks, and the verdict.
    /// </summary>
    public event Action<ScenarioProgress>? ScenarioProgressed;

    /// <summary>
    /// Deals the event's game once it is ready: every deck submitted, or a round that was being played when the table
    /// last stopped. Whoever is not connected yet is brought up to date when they take their seat.
    /// </summary>
    private void StartEventGame(ITableEvent hosted)
    {
        if (_phase is TablePhase.Playing or TablePhase.Finished or TablePhase.Abandoned || !hosted.ReadyToDeal) return;
        List<string> names = [.. Enumerable.Range(1, _options.Players).Select(hosted.NameAt)];
        try
        {
            var session = new GameSession(hosted.Deal(names, _options.Rules));
            session.Start();
            _session = session;
        }
        catch (Exception error) when (error is not OutOfMemoryException)
        {
            // Nothing is dealt and the event goes back to deck building, every deck taken back, so the players can
            // change what stopped it rather than be dealt the same failure again and again.
            _app.Logger.LogError(error, "The event's game could not be dealt.");
            Note("an event's deal was refused");
            hosted.DealFailed(error.Message);
            foreach (Member member in _members.Values)
                if (member.Connection is { } connection) Refuse(connection, ErrorCode.InvalidSetup, "The game could not be dealt: " + error.Message);
            FollowEvent();
            return;
        }

        Note($"the event's game was dealt to {names.Count} seat(s)");
        _phase = TablePhase.Playing;
        StartClocks(hosted);
        BroadcastTable();
        _started.TrySetResult();
    }

    /// <summary>
    /// Starts the match clocks for a game just dealt, when the event gives every seat one: running for the seats the game
    /// is waiting on from the first question - the mulligans - on.
    /// </summary>
    private void StartClocks(ITableEvent hosted)
    {
        _clock.Clear();
        List<(int Seat, TimeSpan? Left)> clocks = [.. Enumerable.Range(1, _options.Players).Select(seat => (seat, hosted.ClockAtDeal(seat)))];
        if (clocks.Any(clock => clock.Left is null)) return;
        foreach ((int seat, TimeSpan? left) in clocks) _clock[seat] = left!.Value;
        _clockAt = _options.TimeProvider.GetUtcNow();
        _running = [];
    }

    /// <summary>
    /// Charges the time since the last look to every running clock, as a chess clock does, and runs the clocks of whoever
    /// the game is waiting on now. A clock that reaches zero ends the game: its seat loses (<see cref="TimeOut"/>).
    /// </summary>
    private void TickClocks()
    {
        if (_clock.Count == 0 || _session is null || _phase != TablePhase.Playing) return;
        DateTimeOffset now = _options.TimeProvider.GetUtcNow();
        TimeSpan elapsed = now - _clockAt;
        _clockAt = now;
        foreach (int seat in _running) _clock[seat] -= elapsed;

        List<int> out_ = [.. _clock.Where(clock => clock.Value <= TimeSpan.Zero).Select(clock => clock.Key).Order()];
        if (out_.Count > 0)
        {
            TimeOut(out_);
            return;
        }

        HashSet<int> waiting = [.. _session.WaitingOn];
        if (waiting.SetEquals(_running)) return;
        _running = waiting;
        foreach ((int seat, TimeSpan left) in _clock) _options.Event?.ClockUsed(seat, left);
        ClockMessage clocks = Clocks();
        foreach (Member member in _members.Values) member.Connection?.Send(clocks);
    }

    private ClockMessage Clocks() => new()
    {
        Clocks = [.. _clock.OrderBy(clock => clock.Key).Select(clock => new SeatClockView
        {
            Seat = clock.Key, LeftMs = Math.Max(0, (long)clock.Value.TotalMilliseconds), Running = _running.Contains(clock.Key),
        })],
    };

    /// <summary>
    /// Seats whose clocks ran out lose: the event is told first, so it can take the match from them as well as the game,
    /// then the game ends with everyone else winning - or nobody, when every clock ran out at once.
    /// </summary>
    private void TimeOut(IReadOnlyList<int> losers)
    {
        Note($"the clock ran out for seat(s) {string.Join(", ", losers)}");
        string names = string.Join(" and ", losers.Select(seat => _members.TryGetValue(seat, out Member? member) ? member.Name : _options.Event?.NameAt(seat) ?? $"Seat {seat}"));
        EndAgainst(losers, $"{names} ran out of time.", ranOut: true);
    }

    /// <summary>
    /// Ends the game being played against some seats - their clocks ran out, or they forfeited it - with everyone else
    /// winning, or nobody when it is everyone.
    /// </summary>
    private void EndAgainst(IReadOnlyList<int> losers, string detail, bool ranOut)
    {
        if (ranOut)
        {
            foreach (int seat in losers)
            {
                _clock[seat] = TimeSpan.Zero;
                _options.Event?.ClockUsed(seat, TimeSpan.Zero);
                _options.Event?.ClockRanOut(seat);
            }
        }

        List<int> winners = [.. Enumerable.Range(1, _options.Players).Where(seat => !losers.Contains(seat))];
        _running = [];
        _session!.Forfeit(winners);
        _detail = detail;
        _phase = TablePhase.Finished;
        if (_clock.Count > 0)
        {
            ClockMessage clocks = Clocks();
            foreach (Member member in _members.Values) member.Connection?.Send(clocks);
        }

        Flush();
        BroadcastTable();
        Finish(winners);
    }

    /// <summary>
    /// Records how the table ended, then completes <see cref="Finished"/>. Called once, from the pump. The winners are the
    /// game's own unless it was ended from outside its rules, as a clock running out ends it.
    /// </summary>
    private void Finish(IReadOnlyList<int>? decided = null)
    {
        // What is left on each clock is kept for the match's next game, charged up to the end of this one.
        if (_clock.Count > 0)
        {
            DateTimeOffset now = _options.TimeProvider.GetUtcNow();
            foreach (int seat in _running) _clock[seat] -= now - _clockAt;
            _clockAt = now;
            _running = [];
            foreach ((int seat, TimeSpan left) in _clock) _options.Event?.ClockUsed(seat, left < TimeSpan.Zero ? TimeSpan.Zero : left);
        }

        List<int> winners = decided is null ? [] : [.. decided];
        if (decided is null && _phase == TablePhase.Finished) winners = [.. _session!.WinningSeats];
        Result = new TableResult(_phase, winners, _detail);
        _options.Event?.GameEnded(_phase == TablePhase.Abandoned, winners, _detail);
        _finished.TrySetResult();
    }

    /// <summary>
    /// Deals the lobby's game: the decks' card names as written, which are the native engine's to know.
    /// </summary>
    private IRulesHost BuildTable(List<Member> members, List<string> names)
    {
        static List<string> Named(IReadOnlyList<DeckListEntry> entries) =>
            [.. entries.SelectMany(entry => Enumerable.Repeat(entry.Card, entry.Copies))];

        List<RulesSeat> named = [.. members.Select((member, index) =>
            new RulesSeat(names[index], Named(_decks[member.Deck].Entries), Named(_decks[member.Deck].Sideboard)))];
        return _options.Rules.Create(named, _options.Seed, new RulesGameOptions());
    }

    private void BroadcastTable()
    {
        var message = new TableMessage
        {
            Phase = _phase, Capacity = _options.Players, HostSeat = _members.Count > 0 ? _members.Keys.Min() : 0,
            Decks = [.. _decks.Keys.Order(StringComparer.Ordinal)],
            AvailableSets = [.. ContentCache.Get(_options.ContentRoot).Sets.Select(set => set.Code)],
            SelectedSets = [.. _sets], Detail = _detail,
            Players = [.. _members.Values.OrderBy(member => member.Seat).Select(member => new TablePlayerView
            {
                Seat = member.Seat, Name = member.Name, Deck = member.Deck, Ready = member.Ready, Connected = member.Connection is not null, Fixed = member.Reserved,
            })],
            EventId = _options.Event?.EventId ?? string.Empty,
            EventName = _options.Event?.Name ?? string.Empty,
            Art = _options.Art?.OriginalString ?? string.Empty,
        };

        // An event's players are known before they sit down, so a seat still waiting for its player is shown as theirs; a
        // forming table's empty seat has no name, and its players say whether they are ready.
        if (_options.Event is { } hosted)
        {
            foreach (int seat in Enumerable.Range(1, _options.Players).Where(seat => !_members.ContainsKey(seat)))
            {
                // A forming table's seat that is ready with nobody connected is a bot's (QueueTableSetup.Bots): there
                // all along, not a player still to open the table.
                message.Players.Add(new TablePlayerView
                {
                    Seat = seat, Name = hosted.Holds(seat) ? hosted.NameAt(seat) : string.Empty, Connected = hosted.Forming && hosted.IsReady(seat),
                });
            }

            message.Players.Sort((a, b) => a.Seat.CompareTo(b.Seat));
            message.Decks = [];
            message.Forming = hosted.Forming;
            foreach (TablePlayerView player in message.Players)
            {
                player.Ready = hosted.Forming && hosted.IsReady(player.Seat);
                if (_options.Portraits is { } portraits && hosted.PlayerAt(player.Seat) is { Length: > 0 } id) player.Portrait = portraits(id) ?? string.Empty;
            }

            message.ReadyLeftMs = hosted.ReadyLeft(_options.TimeProvider.GetUtcNow()) is { } left ? Math.Max(1, (int)left.TotalMilliseconds) : 0;
        }

        foreach (Member member in _members.Values) member.Connection?.Send(message);
    }

    private void Flush()
    {
        if (_session is null) return;
        foreach (PlayerId seat in _session.Seats)
            foreach (ServerMessage message in _session.Drain(seat))
                if (_phase != TablePhase.Abandoned && _members.TryGetValue(seat.Value, out Member? member)) member.Connection?.Send(message);
    }
    private static void Refuse(Connection connection, ErrorCode code, string detail) => connection.Send(new ErrorMessage { Code = code, Detail = detail });

    public async ValueTask DisposeAsync()
    {
        await _shutdown.CancelAsync().ConfigureAwait(false);
        foreach (Connection connection in _connections.Keys) connection.Abort();
        await _pump.ConfigureAwait(false);
        await _app.StopAsync().ConfigureAwait(false);
        await _app.DisposeAsync().ConfigureAwait(false);
        _session?.Dispose();
        _shutdown.Dispose();
        _work.Dispose();
        _connectionSlots.Dispose();
    }

    internal sealed class Member(int seat, string name)
    {
        public int Seat { get; } = seat;
        public string Id { get; } = Convert.ToHexString(RandomNumberGenerator.GetBytes(16));
        public byte[] Proof { get; set; } = RandomNumberGenerator.GetBytes(32);
        public string Name { get; set; } = name;
        public string Deck { get; set; } = string.Empty;
        public bool Ready { get; set; }
        /// <summary>Seated by a ticket, so its name and deck are the reservation's and cannot be changed.</summary>
        public bool Reserved { get; init; }
        public Connection? Connection { get; set; }
        public DateTimeOffset? DisconnectedAt { get; set; }
    }

    private sealed record Reservation(byte[] Ticket, string Name, string Deck);

    internal sealed class Connection(WebSocket socket, DateTimeOffset createdAt)
    {
        private readonly Channel<byte[]> _outbound = Channel.CreateBounded<byte[]>(256);
        private volatile bool _completed;
        private volatile bool _detached;
        public Member? Member { get; set; }
        public DateTimeOffset CreatedAt { get; } = createdAt;

        /// <summary>
        /// Hands a frame to the write loop, and drops the connection if it cannot keep up.
        /// <para>
        /// A closed outbound channel is not a client falling behind, it is a connection already leaving: its read
        /// loop has ended, or its membership has been revoked. Aborting one would reach back through a request
        /// that may be over, and would buy nothing — the socket is going. The frame is dropped instead. Nothing
        /// downstream depends on it arriving, because a seat that comes back resumes, and a resume
        /// resynchronizes the whole view rather than replaying what it missed.
        /// </para>
        /// </summary>
        public void Send(ServerMessage message)
        {
            if (_completed) return;
            if (!_outbound.Writer.TryWrite(MessagePackSerializer.Serialize(message))) Abort();
        }

        public void Complete() { _completed = true; _outbound.Writer.TryComplete(); }

        /// <summary>
        /// The request that accepted the socket has finished with it and is about to dispose it. After this the
        /// socket must not be touched from any other thread.
        /// </summary>
        public void Detach() { _detached = true; Complete(); }

        /// <summary>
        /// Drops the connection. Called from the pump and from shutdown — never from inside the request that owns
        /// the socket.
        /// <para>
        /// The socket forwards <c>Abort()</c> to the <see cref="HttpContext"/> that accepted it, so an abort that
        /// lands after that request has ended reaches a torn-down feature collection. Once the connection is
        /// detached there is nothing left to abort — and, more to the point, nothing of ours to abort: the request
        /// is over and its context belongs to whatever ASP.NET Core does with it next.
        /// </para>
        /// <para>
        /// The catch is for the sliver between reading that flag and the call landing, in which the request can
        /// still end. A socket whose request has ended is already closed, so there is nothing to lose by it.
        /// </para>
        /// </summary>
        public void Abort()
        {
            Complete();
            if (_detached) return;
            try { socket.Abort(); }
            catch (ObjectDisposedException) { }
        }
        public async Task ReadLoopAsync(Action<ClientMessage> receive, CancellationToken cancellation)
        {
            var buffer = new byte[16 * 1024];
            try
            {
                while (socket.State == WebSocketState.Open && !cancellation.IsCancellationRequested)
                {
                    using var frame = new MemoryStream();
                    WebSocketReceiveResult result;
                    do
                    {
                        result = await socket.ReceiveAsync(new ArraySegment<byte>(buffer), cancellation).ConfigureAwait(false);
                        if (result.MessageType != WebSocketMessageType.Binary || frame.Length + result.Count > 64 * 1024) return;
                        frame.Write(buffer, 0, result.Count);
                    } while (!result.EndOfMessage);
                    if (frame.Length > 0) receive(MessagePackSerializer.Deserialize<ClientMessage>(frame.ToArray(), MessagePackSerializerOptions.Standard.WithSecurity(MessagePackSecurity.UntrustedData)));
                }
            }
            catch (OperationCanceledException) { }
            catch (WebSocketException) { }
            catch (MessagePackSerializationException) { }
        }
        public async Task WriteLoopAsync(CancellationToken cancellation)
        {
            try
            {
                await foreach (byte[] payload in _outbound.Reader.ReadAllAsync(cancellation).ConfigureAwait(false))
                {
                    if (socket.State != WebSocketState.Open) return;
                    await socket.SendAsync(payload, WebSocketMessageType.Binary, true, cancellation).ConfigureAwait(false);
                }
                if (socket.State == WebSocketState.Open)
                {
                    await socket.CloseOutputAsync(WebSocketCloseStatus.NormalClosure, "Connection ended.", cancellation).ConfigureAwait(false);

                    // The abort below resets the connection, and a reset lets the other end throw away whatever it has
                    // not read yet - the last frames, such as why it is being closed. So the other end is given a
                    // moment to read them and answer the close, which ends the read loop and closes the socket.
                    DateTime linger = DateTime.UtcNow + TimeSpan.FromSeconds(5);
                    while (socket.State == WebSocketState.CloseSent && DateTime.UtcNow < linger && !cancellation.IsCancellationRequested)
                        await Task.Delay(20, cancellation).ConfigureAwait(false);
                }
            }
            catch (OperationCanceledException) { }
            catch (WebSocketException) { }
            finally { socket.Abort(); }
        }
    }
}
