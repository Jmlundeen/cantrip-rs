using Cantrip.Protocol;

namespace Cantrip.Server;

/// <summary>
/// A game's chess clocks (docs/events.md, "Clocks"), as Magic Online keeps them: each seat's runs only while the game waits on
/// that seat, and a seat whose clock runs out loses. Kept by whatever plays the game - a table (<c>GameServer</c>), a game in
/// one process (<c>HostAndPlay</c>) - which charges them as time passes (<see cref="Tick"/>) and does what they say. Not
/// thread-safe: its owner's one thread.
/// </summary>
public sealed class MatchClocks
{
    private readonly Dictionary<int, TimeSpan> _left;
    private readonly HashSet<int> _out = [];
    private HashSet<int> _waiting = [];
    private HashSet<int> _running = [];
    private DateTimeOffset _at;

    /// <summary>The wait each seat was last given the courtesy for, so one wait is given it once (<see cref="Tick"/>).</summary>
    private readonly Dictionary<int, int> _courteousWait = [];

    /// <summary>Until when each seat given the courtesy runs no clock.</summary>
    private readonly Dictionary<int, DateTimeOffset> _graceUntil = [];

    /// <param name="left">Each seat's time on its clock as the game is dealt.</param>
    /// <param name="now">The deal.</param>
    public MatchClocks(IReadOnlyDictionary<int, TimeSpan> left, DateTimeOffset now)
    {
        ArgumentNullException.ThrowIfNull(left);
        _left = new Dictionary<int, TimeSpan>(left);
        _at = now;
    }

    /// <summary>Each player's time unless a table says otherwise: an hour, for the whole match.</summary>
    public static readonly TimeSpan DefaultPlayerClock = TimeSpan.FromHours(1);

    /// <summary>
    /// How long each priority of a player whose draw offer stands runs no clock (docs/rules-exceptions.md, RX-4): a courtesy,
    /// so an offer can't be used to run an opponent's clock down.
    /// </summary>
    public static readonly TimeSpan DrawOfferCourtesy = TimeSpan.FromSeconds(3);

    /// <summary>Each seat's time left, as of the last <see cref="Tick"/>.</summary>
    public IReadOnlyDictionary<int, TimeSpan> Left => _left;

    /// <summary>
    /// The seats whose clocks are running: the ones the game waits on, but for one still in the first moments of a courteous
    /// wait (<see cref="Tick"/>).
    /// </summary>
    public IReadOnlySet<int> Running => _running;

    /// <summary>
    /// Charges the time since the last look to every running clock, then runs the clocks of the seats the game waits on now.
    /// Says which seats' clocks ran out - each said once, its clock stopped at zero; a seat dealt a game with no time left
    /// runs out at the first look - and whether whose clocks run changed.
    /// <para>
    /// A wait in <paramref name="courteous"/> - a seat's own, by an id that tells one wait from the next (a draw offer's
    /// offerer, at each of its later priorities: <c>GameSession.Courtesies</c>) - runs no clock for its first
    /// <see cref="DrawOfferCourtesy"/>, from the first look that sees it: the seat's clock starts when that is up, charged from
    /// then and not from the next look. No other seat's clock runs for it.
    /// </para>
    /// </summary>
    public ClockTick Tick(DateTimeOffset now, IEnumerable<int> waitingOn, IReadOnlyDictionary<int, int>? courteous = null)
    {
        ArgumentNullException.ThrowIfNull(waitingOn);
        Charge(now);
        List<int> ranOut = [.. _left.Where(clock => clock.Value <= TimeSpan.Zero && !_out.Contains(clock.Key)).Select(clock => clock.Key).Order()];
        foreach (int seat in ranOut)
        {
            _out.Add(seat);
            _left[seat] = TimeSpan.Zero;
        }

        _waiting = [.. waitingOn.Where(seat => _left.ContainsKey(seat) && !_out.Contains(seat))];
        courteous ??= new Dictionary<int, int>();

        // A wait no longer courteous - the offer withdrawn, or the seat asked something other than priority - runs at once.
        foreach (int seat in _graceUntil.Keys.Where(seat => !courteous.ContainsKey(seat)).ToList())
        {
            _graceUntil.Remove(seat);
            _courteousWait.Remove(seat);
        }

        foreach ((int seat, int wait) in courteous)
        {
            if (_waiting.Contains(seat) && (!_courteousWait.TryGetValue(seat, out int given) || given != wait))
            {
                _courteousWait[seat] = wait;
                _graceUntil[seat] = now + DrawOfferCourtesy;
            }
        }

        HashSet<int> running = [.. _waiting.Where(seat => !InGrace(seat, now))];
        bool changed = !running.SetEquals(_running) || ranOut.Count > 0;
        _running = running;
        return new ClockTick(ranOut, changed);
    }

    /// <summary>Whether a seat is still in the first moments of a courteous wait, its clock not yet running.</summary>
    private bool InGrace(int seat, DateTimeOffset now) => _graceUntil.TryGetValue(seat, out DateTimeOffset until) && now < until;

    /// <summary>Charges the time since the last look, then stops every clock: the game is over.</summary>
    public void Stop(DateTimeOffset now)
    {
        Charge(now);
        _waiting = [];
        _running = [];
    }

    /// <summary>The clocks as the seats are told them.</summary>
    public ClockMessage Message() => new()
    {
        Clocks = [.. _left.OrderBy(clock => clock.Key).Select(clock => new SeatClockView
        {
            Seat = clock.Key, LeftMs = Math.Max(0, (long)clock.Value.TotalMilliseconds), Running = _running.Contains(clock.Key),
        })],
    };

    /// <summary>
    /// Charges every seat the game waits on for the time since the last look, but for what of it fell in a courteous wait's
    /// first moments.
    /// </summary>
    private void Charge(DateTimeOffset now)
    {
        DateTimeOffset from = _at;
        _at = now > _at ? now : _at;
        foreach (int seat in _waiting)
        {
            DateTimeOffset start = _graceUntil.TryGetValue(seat, out DateTimeOffset until) && until > from ? until : from;
            if (now > start)
            {
                _left[seat] -= now - start;
            }
        }
    }
}

/// <summary>What charging a game's clocks came to (<see cref="MatchClocks.Tick"/>).</summary>
/// <param name="RanOut">The seats whose clocks ran out just now, in seat order.</param>
/// <param name="Changed">Whose clocks run, or which ran out, changed: the seats are to be told.</param>
public readonly record struct ClockTick(IReadOnlyList<int> RanOut, bool Changed);
