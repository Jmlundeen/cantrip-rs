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

    public event Action? Changed;

    /// <summary>Whether anyone receives priority in the step, and so whether there is anything to stop for.</summary>
    public static bool Stoppable(TurnStepView step) => step is not (TurnStepView.Untap or TurnStepView.Cleanup);

    public bool StopsAt(TurnStepView step, bool myTurn) => (myTurn ? _mine : _theirs).Contains(step);

    public void Toggle(TurnStepView step, bool myTurn)
    {
        HashSet<TurnStepView> set = myTurn ? _mine : _theirs;
        if (!set.Remove(step))
        {
            set.Add(step);
        }

        Store(myTurn ? MineKey : TheirsKey, set);
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
