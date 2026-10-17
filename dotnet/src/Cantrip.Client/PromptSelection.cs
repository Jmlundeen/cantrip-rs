using Cantrip.Protocol;

namespace Cantrip.Client;

public readonly record struct AttackChoice(int Player, ViewHandle Permanent);

/// <summary>Unsubmitted card choices, independent of rendering and of the transport carrying the prompt.</summary>
public sealed class PromptSelection
{
    public List<int> Cards { get; } = [];
    public List<int> Modes { get; } = [];
    public List<int> Colors { get; } = [];
    public List<TargetMessage> Targets { get; } = [];
    public List<int> TriggerOrder { get; } = [];
    public Dictionary<int, AttackChoice> Attacking { get; } = [];
    public Dictionary<int, int> Blocking { get; } = [];

    /// <summary>
    /// Creatures picked up to attack or to block but not yet sent anywhere: clicking what to attack, or whom to block,
    /// declares every one of them at once.
    /// </summary>
    public List<int> Pending { get; } = [];

    /// <summary>
    /// The attacker the last click sent more blockers at than it may be blocked by (CR 509.1b), whose extra blockers are
    /// still picked up; -1 for none.
    /// </summary>
    public int BlockLimit { get; set; } = -1;

    public Dictionary<(int Source, int Creature), int> DamageShares { get; } = [];
    /// <summary>A divided target's own share (CR 601.2d), keyed by candidate handle.</summary>
    public Dictionary<int, int> DividedShares { get; } = [];
    public ActionOptionMessage? Targeting { get; set; }

    /// <summary>
    /// The options of one ability whose cost taps or sacrifices a permanent of the player's choosing (Urza, Lord High
    /// Artificer; Squandered Resources), held while that permanent is picked on the board. Nothing is activated yet.
    /// </summary>
    public List<ActionOptionMessage>? CostChoice { get; set; }
    public string NameTyped { get; set; } = string.Empty;
    public List<List<int>>? Placements { get; set; }
    public bool CanPick(int limit) => Cards.Count < limit || limit == 1;

    /// <summary>
    /// The simultaneous triggers not yet put on the stack, in the order they triggered, which is the order the dock
    /// lays them out for the player to click.
    /// </summary>
    public IEnumerable<int> TriggersLeft(int count) =>
        Enumerable.Range(0, count).Where(index => !TriggerOrder.Contains(index));

    /// <summary>
    /// The order shown, taken as the answer: whatever has been placed already, then everything left in the order it
    /// is laid out. First in the list goes on the stack first and so resolves last (CR 603.3b), which is exactly what
    /// clicking them left to right would have sent.
    /// </summary>
    public List<int> TriggerOrderAsShown(int count) => [.. TriggerOrder, .. TriggersLeft(count)];

    public void Pick(int handle, int limit)
    {
        if (Cards.Remove(handle)) return;
        if (limit == 1) Cards.Clear();
        if (Cards.Count < limit) Cards.Add(handle);
    }
    public void Clear()
    {
        Cards.Clear();
        Modes.Clear();
        Colors.Clear();
        Targets.Clear();
        TriggerOrder.Clear();
        Attacking.Clear();
        Blocking.Clear();
        Pending.Clear();
        BlockLimit = -1;
        DamageShares.Clear();
        DividedShares.Clear();
        Targeting = null;
        CostChoice = null;
        NameTyped = string.Empty;
        Placements = null;
    }
}
