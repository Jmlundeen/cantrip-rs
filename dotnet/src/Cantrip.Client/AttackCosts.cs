using System.Runtime.CompilerServices;
using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// Costs to attack (CR 508.1d, 508.1h-j: "creatures can't attack you unless their controller pays {2} for each ..."), for
/// a seat a bot answers: keeping a declaration within the mana there is to pay for it, and remembering a payment given up.
/// <para>
/// A declaration whose cost is not paid is taken back and asked again (CR 733), so a bot that declared an attack it could
/// not pay for, and declared it again, would never get past the step. Each bot therefore trims its attack to the mana the
/// prompt says it could pay, and once a seat has given up paying for an attack it declares only what costs nothing for the
/// rest of that turn.
/// </para>
/// </summary>
public static class AttackCosts
{
    private static readonly ConditionalWeakTable<GameClient, StrongBox<int>> GaveUp = new();

    /// <summary>
    /// Notes the answer a seat gives to a payment: giving up a cost to attack means that turn's declarations are to cost
    /// nothing.
    /// </summary>
    public static void Answered(GameClient client, PayCostsPromptMessage prompt, TakeActionAnswer answer)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);
        ArgumentNullException.ThrowIfNull(answer);

        if (prompt.Attackers.Count > 0 && answer.Kind == ActionKindView.CancelCasting)
        {
            GaveUp.AddOrUpdate(client, new StrongBox<int>(client.View?.Turn ?? 0));
        }
    }

    /// <summary>The mana this seat has to pay its attack with: none once it gave a payment for one up this turn.</summary>
    public static int Mana(GameClient client, DeclareAttackersPromptMessage prompt)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);

        if (GaveUp.TryGetValue(client, out StrongBox<int>? turn) && turn.Value == (client.View?.Turn ?? 0))
        {
            return 0;
        }

        return prompt.CanPay < 0 ? int.MaxValue : prompt.CanPay;
    }

    /// <summary>
    /// <paramref name="attacks"/> kept within <paramref name="mana"/>: a creature that must attack goes where attacking
    /// costs nothing - a cost is never a requirement (CR 508.1d) - and while the rest still cost more than that, the
    /// costliest attack of a creature that need not attack is dropped. A creature attacking that taps for mana and has no
    /// vigilance (<paramref name="makesMana"/>) is mana that will not be there (CR 508.1f).
    /// </summary>
    public static List<AttackMessage> Within(
        DeclareAttackersPromptMessage prompt, List<AttackMessage> attacks, int mana, Func<ViewHandle, bool>? makesMana = null)
    {
        ArgumentNullException.ThrowIfNull(prompt);
        ArgumentNullException.ThrowIfNull(attacks);

        if (prompt.Costs.Count == 0)
        {
            return attacks;
        }

        List<AttackMessage> kept = [.. attacks];
        for (int i = 0; i < kept.Count; i++)
        {
            AttackMessage attack = kept[i];
            // A creature the engine's own declaration has attack for nothing goes there rather than pay (CR 508.1d).
            if (prompt.CostOf(attack) is not null && prompt.SuggestedFor(attack.Attacker) is { } free && prompt.CostOf(free) is null)
            {
                kept[i] = free;
            }
        }

        int Have() => mana == int.MaxValue
            ? int.MaxValue
            : mana - kept.Count(attack => makesMana?.Invoke(attack.Attacker) ?? false);

        while (prompt.TotalCost(kept) > Have())
        {
            AttackMessage? costliest = kept
                .Where(attack => prompt.CostOf(attack) is not null
                    && !prompt.IsRequired(attack.Attacker))
                .OrderByDescending(attack => prompt.CostOf(attack)!.ManaValue)
                .FirstOrDefault();
            if (costliest is null)
            {
                break;
            }

            kept.Remove(costliest);
        }

        return kept;
    }
}
