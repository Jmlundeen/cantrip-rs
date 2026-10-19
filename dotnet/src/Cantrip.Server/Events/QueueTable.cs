using System.Security.Cryptography;
using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// A table formed from a queue, before its event exists. Its host gives each player who joins a seat and a ticket
/// (<see cref="Reserve"/>); players see who else is at the table and the seats still empty, and may leave. When every
/// seat is taken and every player has the table open, a ready check starts: each player has a while (thirty seconds) to
/// say they are ready, and whoever has not by then loses their seat, which goes to the next player to join. A player who
/// joined but has not opened the table has a while of their own (thirty seconds too) to open it - and to come back to it,
/// having closed it - before their seat goes the same way. A table may keep some of its seats for bots
/// (<see cref="QueueTableSetup.Bots"/>): those are taken from the start and always ready, and the table waits only for
/// its players. When everyone is ready the event is made
/// (<see cref="QueueTableSetup.Start"/>) and this table becomes its table: the draft begins in the same page.
/// <para>
/// The forming part is only in memory: a table that never filled is not worth keeping, and its players join the queue
/// again. Once the event is made it is the event that is kept, as any event is.
/// </para>
/// <para>
/// Safe to call from any thread; the table calls it from its pump and the host from wherever players join.
/// <see cref="Changed"/> is raised outside the lock, on whichever thread made the change.
/// </para>
/// </summary>
public sealed class QueueTable : ITableEvent
{
    private readonly object _gate = new();
    private readonly QueueTableSetup _setup;
    private readonly Seat?[] _seats;
    private readonly List<(int Seat, ServerMessage Farewell)> _vacated = [];
    private DateTimeOffset? _readyBy;
    private ITableEvent? _pod;
    private int _version;

    private sealed class Seat(string playerId, string name, DateTimeOffset joined, bool bot = false)
    {
        public string PlayerId { get; } = playerId;

        /// <summary>A seat the table keeps for a bot: never given up, never waited for, and played once the event begins.</summary>
        public bool Bot { get; } = bot;

        public string Name { get; } = name;
        public DateTimeOffset Joined { get; } = joined;
        public List<byte[]> Tickets { get; } = [];
        public bool Ready { get; set; }

        /// <summary>Whether the player has the table open now: the only players worth moving to another table.</summary>
        public bool Connected { get; set; }

        /// <summary>When the seat goes if the player has not opened the table by then; null while they have it open.</summary>
        public DateTimeOffset? OpenBy { get; set; }
    }

    public QueueTable(QueueTableSetup setup)
    {
        ArgumentNullException.ThrowIfNull(setup);
        if (setup.Seats is < 2 or > 8) throw new ArgumentException("A table seats two to eight.", nameof(setup));
        _setup = setup;
        if (setup.Bots.Count >= setup.Seats) throw new ArgumentException("A table needs at least one seat for a player.", nameof(setup));
        _seats = new Seat?[setup.Seats];
        Id = "table-" + Convert.ToHexString(RandomNumberGenerator.GetBytes(4)).ToLowerInvariant();
        Created = setup.TimeProvider.GetUtcNow();

        // The bots in the last seats, there and ready from the start.
        for (int bot = 0; bot < setup.Bots.Count; bot++)
        {
            (string id, string name) = setup.Bots[bot];
            _seats[setup.Seats - setup.Bots.Count + bot] = new Seat(id, name, Created, bot: true) { Connected = true, Ready = true };
        }
    }

    /// <summary>The table's own id, until it has an event: for the host's bookkeeping, and the page's.</summary>
    public string Id { get; }

    public DateTimeOffset Created { get; }

    /// <summary>Something players see changed: a seat taken or given up, a ready said, the event begun.</summary>
    public event Action<QueueTable>? Changed;

    /// <summary>The event, once everyone was ready; null while the table is forming.</summary>
    public ITableEvent? Pod
    {
        get { lock (_gate) return _pod; }
    }

    /// <summary>How many seats players have taken; a bot's seat is not counted.</summary>
    public int Taken
    {
        get { lock (_gate) return _seats.Count(seat => seat is { Bot: false }); }
    }

    /// <summary>Whether every seat is taken and the table is waiting for its players to say they are ready.</summary>
    public bool Checking
    {
        get { lock (_gate) return _pod is null && _readyBy is not null; }
    }

    /// <summary>The players at the table, by seat, with when each sat down and whether they have the table open; bots are not among them.</summary>
    public IReadOnlyList<(int Seat, string PlayerId, string Name, DateTimeOffset Joined, bool Connected)> Players
    {
        get
        {
            lock (_gate)
            {
                return [.. _seats.Select((seat, index) => (seat, index)).Where(pair => pair.seat is { Bot: false })
                    .Select(pair => (pair.index + 1, pair.seat!.PlayerId, pair.seat.Name, pair.seat.Joined, pair.seat.Connected))];
            }
        }
    }

    /// <summary>
    /// Gives every seat up with a word for its player - the queue was closed - before the table is taken down. Nothing,
    /// once the event has begun.
    /// </summary>
    public void Close(string why)
    {
        lock (_gate)
        {
            if (_pod is not null) return;
            for (int index = 0; index < _seats.Length; index++)
            {
                if (_seats[index] is { Bot: false }) Vacate(index, new SeatReleasedMessage { Detail = why });
            }
        }

        Changed?.Invoke(this);
    }

    public bool Seats(string playerId)
    {
        lock (_gate) return _seats.Any(seat => seat?.PlayerId == playerId);
    }

    /// <summary>
    /// Gives a player the first empty seat and returns the ticket that takes them to it; or, for a player already at the
    /// table, a fresh ticket to their own seat. Null when the table is full, or its event has begun.
    /// </summary>
    public string? Reserve(string playerId, string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(playerId);
        string ticket;
        lock (_gate)
        {
            if (_pod is not null) return null;
            Seat? seat = _seats.FirstOrDefault(each => each?.PlayerId == playerId);
            if (seat is null)
            {
                int empty = Array.IndexOf(_seats, null);
                if (empty < 0) return null;
                seat = _seats[empty] = new Seat(playerId, name.Trim(), _setup.TimeProvider.GetUtcNow());
            }

            // A while to open the table from the link: from joining, or from asking for the link again.
            if (!seat.Connected) seat.OpenBy = _setup.TimeProvider.GetUtcNow() + _setup.OpenWindow;

            ticket = Convert.ToHexString(RandomNumberGenerator.GetBytes(32));
            seat.Tickets.Add(SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(ticket)));
            _version++;
        }

        Changed?.Invoke(this);
        return ticket;
    }

    /// <summary>Gives a player's seat up with a word for them - another table to go to - when the host moves them.</summary>
    public bool Release(string playerId, ServerMessage farewell)
    {
        lock (_gate)
        {
            if (_pod is not null) return false;
            int index = Array.FindIndex(_seats, seat => seat?.PlayerId == playerId);
            if (index < 0) return false;
            Vacate(index, farewell);
        }

        Changed?.Invoke(this);
        return true;
    }

    // --- The table. ---

    public string EventId
    {
        get { lock (_gate) return _pod?.EventId ?? Id; }
    }

    public string Name => _setup.Name;

    int ITableEvent.Seats => _setup.Seats;

    public TablePhase Phase
    {
        get { lock (_gate) return _pod?.Phase ?? TablePhase.Setup; }
    }

    public bool Forming
    {
        get { lock (_gate) return _pod is null; }
    }

    public bool Holds(int seat)
    {
        lock (_gate) return _pod is not null || (seat >= 1 && seat <= _seats.Length && _seats[seat - 1] is not null);
    }

    public bool IsReady(int seat)
    {
        lock (_gate) return seat >= 1 && seat <= _seats.Length && _seats[seat - 1] is { Ready: true };
    }

    public TimeSpan? ReadyLeft(DateTimeOffset now)
    {
        lock (_gate) return _pod is null && _readyBy is { } by ? (by > now ? by - now : TimeSpan.Zero) : null;
    }

    /// <summary>The table's own changes, and once formed its event's too.</summary>
    public int Version
    {
        get
        {
            ITableEvent? pod;
            int version;
            lock (_gate)
            {
                pod = _pod;
                version = _version;
            }

            return pod is null ? version : version + pod.Version;
        }
    }

    public int SeatFor(string ticket)
    {
        if (string.IsNullOrEmpty(ticket)) return 0;
        byte[] presented = SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(ticket));
        ITableEvent? pod;
        lock (_gate)
        {
            int found = 0;
            for (int index = 0; index < _seats.Length; index++)
            {
                foreach (byte[] hash in _seats[index]?.Tickets ?? [])
                {
                    if (CryptographicOperations.FixedTimeEquals(hash, presented)) found = index + 1;
                }
            }

            if (found != 0) return found;
            pod = _pod;
        }

        return pod?.SeatFor(ticket) ?? 0;
    }

    public string NameAt(int seat)
    {
        lock (_gate)
        {
            if (_pod is not null) return _pod.NameAt(seat);
            return seat >= 1 && seat <= _seats.Length ? _seats[seat - 1]?.Name ?? string.Empty : string.Empty;
        }
    }

    public string PlayerAt(int seat)
    {
        lock (_gate)
        {
            if (_pod is not null) return _pod.PlayerAt(seat);
            return seat >= 1 && seat <= _seats.Length ? _seats[seat - 1]?.PlayerId ?? string.Empty : string.Empty;
        }
    }

    public void Seated(int seat) => Pod?.Seated(seat);

    public void Connected(int seat, bool connected)
    {
        if (Pod is { } formed)
        {
            formed.Connected(seat, connected);
            return;
        }

        lock (_gate)
        {
            if (_pod is not null || seat < 1 || seat > _seats.Length || _seats[seat - 1] is not { } taken || taken.Connected == connected) return;
            taken.Connected = connected;

            // Here: nothing to wait for from them, and perhaps the last one the ready check was waiting for. Gone: a while
            // to come back before the seat goes - unless the ready check is on, which settles it either way.
            taken.OpenBy = connected || _readyBy is not null ? null : _setup.TimeProvider.GetUtcNow() + _setup.OpenWindow;
            if (connected) MaybeStartCheck();
            _version++;
        }

        Changed?.Invoke(this);
    }

    public ServerMessage? ViewFor(int seat) => Pod?.ViewFor(seat);

    public string? Receive(int seat, ClientMessage message)
    {
        ITableEvent? pod;
        string? refusal = null;
        bool changed = false;
        lock (_gate)
        {
            pod = _pod;
            if (pod is null)
            {
                refusal = message switch
                {
                    LeaveMessage => Leave(seat, ref changed),
                    ReadyMessage ready => Ready(seat, ready.Ready, ref changed),
                    _ => "The table is still being formed.",
                };
            }
        }

        if (pod is not null)
        {
            return pod.Receive(seat, message);
        }

        if (changed) Changed?.Invoke(this);
        return refusal;
    }

    private string? Leave(int seat, ref bool changed)
    {
        if (seat < 1 || seat > _seats.Length || _seats[seat - 1] is not { Bot: false }) return "That seat is not yours.";
        Vacate(seat - 1, new SeatReleasedMessage { Detail = "You left the table." });
        changed = true;
        return null;
    }

    private string? Ready(int seat, bool ready, ref bool changed)
    {
        if (_readyBy is null) return "Not every seat is taken yet.";
        if (seat < 1 || seat > _seats.Length || _seats[seat - 1] is not { Bot: false } taken) return "That seat is not yours.";
        taken.Ready = ready;
        _version++;
        changed = true;
        if (_seats.All(each => each is { Ready: true }))
        {
            try
            {
                Begin();
            }
            catch (Exception error) when (error is not OutOfMemoryException)
            {
                // Nothing begun: everyone is still seated, with a fresh while to say ready again and try once more - not
                // a ready check that runs out on players who did say ready.
                _pod = null;
                _readyBy = null;
                MaybeStartCheck();
                return "The draft could not be started: " + error.Message;
            }
        }

        return null;
    }

    public void Tick(DateTimeOffset now)
    {
        if (Pod is { } formed)
        {
            formed.Tick(now);
            return;
        }

        bool changed = false;
        lock (_gate)
        {
            if (_pod is not null) return;

            // Joined but never opened the table, or closed it and did not come back, in time: the seat goes back.
            for (int index = 0; index < _seats.Length; index++)
            {
                if (_seats[index] is { OpenBy: { } openBy } && now >= openBy)
                {
                    Vacate(index, new SeatReleasedMessage
                    {
                        Detail = $"Your table wasn't open within {_setup.OpenWindow.TotalSeconds:0} seconds of joining, so your seat went back to the queue. Join again from Discord when you're ready to play.",
                    });
                    changed = true;
                }
            }

            // The ready check run out: everyone who did not say ready loses their seat, and the rest wait for the seats
            // to fill again.
            if (_readyBy is { } by && now >= by)
            {
                _readyBy = null;
                changed = true;

                // Who is out is decided before anyone goes: letting a seat go makes nobody ready any more.
                List<int> unready = [.. Enumerable.Range(0, _seats.Length).Where(index => _seats[index] is { Ready: false })];
                foreach (int index in unready)
                {
                    Vacate(index, new SeatReleasedMessage
                    {
                        Detail = $"You didn't click Ready within {_setup.ReadyWindow.TotalSeconds:0} seconds, so your seat went back to the queue. Join again from Discord when you're ready to play.",
                    });
                }
            }
        }

        if (changed) Changed?.Invoke(this);
    }

    public IReadOnlyList<(int Seat, ServerMessage Farewell)> TakeVacated()
    {
        ITableEvent? pod;
        List<(int, ServerMessage)> taken;
        lock (_gate)
        {
            taken = [.. _vacated];
            _vacated.Clear();
            pod = _pod;
        }

        // Anyone the forming table let go first, then whoever the event has sent on since.
        if (pod is not null) taken.AddRange(pod.TakeVacated());
        return taken;
    }

    /// <summary>
    /// Starts the ready check once every seat is taken and every player has the table open: the thirty seconds are for
    /// pressing Ready!, not for getting to the page.
    /// </summary>
    private void MaybeStartCheck()
    {
        if (_readyBy is not null || _seats.Any(seat => seat is not { Connected: true })) return;
        _readyBy = _setup.TimeProvider.GetUtcNow() + _setup.ReadyWindow;
        foreach (Seat? seat in _seats)
        {
            if (seat!.Bot) continue;
            seat.Ready = false;
            seat.OpenBy = null;
        }
    }

    /// <summary>
    /// Empties a seat; a table that is no longer full stops its ready check, and nobody is ready any more. Anyone left
    /// who does not have the table open gets a fresh while to open it.
    /// </summary>
    private void Vacate(int index, ServerMessage farewell)
    {
        _seats[index] = null;
        _vacated.Add((index + 1, farewell));
        _readyBy = null;
        foreach (Seat? seat in _seats)
        {
            if (seat is null or { Bot: true }) continue;
            seat.Ready = false;
            if (!seat.Connected) seat.OpenBy ??= _setup.TimeProvider.GetUtcNow() + _setup.OpenWindow;
        }

        _version++;
    }

    /// <summary>Everyone is ready: the event is made with the players in their seats, and the table becomes its table.</summary>
    private void Begin()
    {
        List<(string Id, string Name)> players = [.. _seats.Select(seat => (seat!.PlayerId, seat.Name))];
        _pod = _setup.Start(players);
        _readyBy = null;
        for (int seat = 1; seat <= _seats.Length; seat++)
        {
            _pod.Seated(seat);

            // Who has the table open now, which the event goes on being told from here. A bot is not there yet: it sits
            // down once the event has begun, as anyone coming back to it would.
            if (_seats[seat - 1] is { Bot: false, Connected: true }) _pod.Connected(seat, true);
        }

        _version++;
    }

    // --- Once formed, the event's. ---

    public bool ReadyToDeal => Pod?.ReadyToDeal ?? false;

    public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null) =>
        (Pod ?? throw new InvalidOperationException("The table has not formed.")).Deal(names, rules);

    public void DealFailed(string why) => Pod?.DealFailed(why);

    public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail) => Pod?.GameEnded(abandoned, winningSeats, detail);

    public TimeSpan? ClockAtDeal(int seat) => Pod?.ClockAtDeal(seat);

    public void ClockUsed(int seat, TimeSpan left) => Pod?.ClockUsed(seat, left);

    public void ClockRanOut(int seat) => Pod?.ClockRanOut(seat);

    public IReadOnlyList<int> TakeForfeits() => Pod?.TakeForfeits() ?? [];

    public string? ForfeitReason(int seat) => Pod?.ForfeitReason(seat);
}

/// <summary>How a queue's table forms, and what it becomes.</summary>
public sealed class QueueTableSetup
{
    /// <summary>What the table is called: the queue's draft, as "Winston draft: LSVCube".</summary>
    public required string Name { get; init; }

    public int Seats { get; init; } = 2;

    /// <summary>
    /// The bots the table seats, as (player id, name): in its last seats from the start, always ready. Whoever hosts the
    /// event plays them once it begins.
    /// </summary>
    public IReadOnlyList<(string Id, string Name)> Bots { get; init; } = [];

    /// <summary>How long the players of a full table have to say they are ready.</summary>
    public TimeSpan ReadyWindow { get; init; } = TimeSpan.FromSeconds(30);

    /// <summary>How long a player who joined has to open the table from their link, or to come back to it having closed it.</summary>
    public TimeSpan OpenWindow { get; init; } = TimeSpan.FromSeconds(30);

    /// <summary>Makes the event for the players, in seat order, once they are all ready, and opens it.</summary>
    public required Func<IReadOnlyList<(string Id, string Name)>, ITableEvent> Start { get; init; }

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}
