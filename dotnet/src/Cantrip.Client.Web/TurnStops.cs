using System.Runtime.InteropServices.JavaScript;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// The steps this player wants to be stopped in, on their own turn and on an opponent's. By default both main phases
/// of one's own turn, and nothing on theirs. Kept in this browser's localStorage; a browser that will not store them
/// simply starts from the defaults each time.
/// </summary>
public sealed class TurnStops
{
    public static readonly IReadOnlyList<TurnStepView> DefaultMine = [TurnStepView.PrecombatMain, TurnStepView.PostcombatMain];

    public static readonly IReadOnlyList<TurnStepView> DefaultTheirs = [];

    private const string MineKey = "cantrip-stops-mine";
    private const string TheirsKey = "cantrip-stops-theirs";

    private readonly HashSet<TurnStepView> _mine;
    private readonly HashSet<TurnStepView> _theirs;

    public TurnStops()
    {
        _mine = [.. (Recall(MineKey) ?? DefaultMine).Where(Stoppable)];
        _theirs = [.. (Recall(TheirsKey) ?? DefaultTheirs).Where(Stoppable)];
    }

    /// <summary>Raised when where this player is stopped changes: their own stops, or the rest of the table's.</summary>
    public event Action? Changed;

    /// <summary>Raised when this player's own stops change, and only then: what an in-person phone tells the table.</summary>
    public event Action? OwnChanged;

    /// <summary>Whether anyone receives priority in the step, and so whether there is anything to stop for.</summary>
    public static bool Stoppable(TurnStepView step) => step is not (TurnStepView.Untap or TurnStepView.Cleanup);

    /// <summary>
    /// Whether this player is stopped in a step: by a stop of their own, or - at an in-person table, where a stop is the
    /// table's (docs/in-person.md, "Stops") - by one another player there set (<see cref="ShareWith"/>).
    /// </summary>
    public bool StopsAt(TurnStepView step, bool myTurn) => OwnStopAt(step, myTurn) || (_table?.Invoke(step, myTurn) ?? false);

    /// <summary>Whether this player set a stop of their own in a step: what the stop's toggle shows and changes.</summary>
    public bool OwnStopAt(TurnStepView step, bool myTurn) => (myTurn ? _mine : _theirs).Contains(step);

    /// <summary>The steps of this player's own turn they stop in, in turn order.</summary>
    public IReadOnlyList<TurnStepView> Mine => [.. _mine.Order()];

    /// <summary>The steps of an opponent's turn this player stops in, in turn order.</summary>
    public IReadOnlyList<TurnStepView> Theirs => [.. _theirs.Order()];

    private Func<TurnStepView, bool, bool>? _table;

    /// <summary>
    /// Stops this player at every stop the rest of the table set as well as their own: given, for a step and whether it is
    /// this player's turn, whether someone else stops there. An in-person phone's, from the relay; null for none.
    /// </summary>
    public void ShareWith(Func<TurnStepView, bool, bool>? others)
    {
        _table = others;
        Changed?.Invoke();
    }

    public void Toggle(TurnStepView step, bool myTurn)
    {
        HashSet<TurnStepView> set = myTurn ? _mine : _theirs;
        if (!set.Remove(step))
        {
            set.Add(step);
        }

        Store(myTurn ? MineKey : TheirsKey, set);
        OwnChanged?.Invoke();
        Changed?.Invoke();
    }

    private static IEnumerable<TurnStepView>? Recall(string key)
    {
        try
        {
            string? stored = Storage()?.GetPropertyAsString(key);
            return stored is null
                ? null
                : stored.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(Enum.Parse<TurnStepView>).ToList();
        }
        catch (Exception error) when (error is JSException or ArgumentException)
        {
            return null;
        }
    }

    private static void Store(string key, IEnumerable<TurnStepView> steps)
    {
        try
        {
            Storage()?.SetProperty(key, string.Join(',', steps.Order()));
        }
        catch (JSException)
        {
            // Storage refused (a private window, blocked site data): the stops still hold for this session.
        }
    }

    private static JSObject? Storage() => JSHost.GlobalThis.GetPropertyAsJSObject("localStorage");
}
