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
    private HashSet<int> _running = [];
    private DateTimeOffset _at;

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

    /// <summary>Each seat's time left, as of the last <see cref="Tick"/>.</summary>
    public IReadOnlyDictionary<int, TimeSpan> Left => _left;

    /// <summary>The seats whose clocks are running: the ones the game waits on.</summary>
    public IReadOnlySet<int> Running => _running;

    /// <summary>
    /// Charges the time since the last look to every running clock, then runs the clocks of the seats the game waits on now.
    /// Says which seats' clocks ran out - each said once, its clock stopped at zero; a seat dealt a game with no time left
    /// runs out at the first look - and whether whose clocks run changed.
    /// </summary>
    public ClockTick Tick(DateTimeOffset now, IEnumerable<int> waitingOn)
    {
        ArgumentNullException.ThrowIfNull(waitingOn);
        Charge(now);
        List<int> ranOut = [.. _left.Where(clock => clock.Value <= TimeSpan.Zero && !_out.Contains(clock.Key)).Select(clock => clock.Key).Order()];
        foreach (int seat in ranOut)
        {
            _out.Add(seat);
            _left[seat] = TimeSpan.Zero;
        }

        HashSet<int> running = [.. waitingOn.Where(seat => _left.ContainsKey(seat) && !_out.Contains(seat))];
        bool changed = !running.SetEquals(_running) || ranOut.Count > 0;
        _running = running;
        return new ClockTick(ranOut, changed);
    }

    /// <summary>Charges the time since the last look, then stops every clock: the game is over.</summary>
    public void Stop(DateTimeOffset now)
    {
        Charge(now);
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

    private void Charge(DateTimeOffset now)
    {
        TimeSpan elapsed = now > _at ? now - _at : TimeSpan.Zero;
        _at = now > _at ? now : _at;
        foreach (int seat in _running)
        {
            _left[seat] -= elapsed;
        }
    }
}

/// <summary>What charging a game's clocks came to (<see cref="MatchClocks.Tick"/>).</summary>
/// <param name="RanOut">The seats whose clocks ran out just now, in seat order.</param>
/// <param name="Changed">Whose clocks run, or which ran out, changed: the seats are to be told.</param>
public readonly record struct ClockTick(IReadOnlyList<int> RanOut, bool Changed);
