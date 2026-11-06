using System.Runtime.CompilerServices;
using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// An X announced for more than could be paid (CR 601.2b, 601.2h), for a seat a bot answers: remembering a payment for X
/// given up, so the X it announces next is less.
/// <para>
/// X is announced before the payment and is locked in once it is: the payment offers no lower X, and a payment given up
/// takes the whole cast back (CR 733.1) to the same priority, where the same spell is still offered at its least X. A bot
/// that announced the same X again would give up again for ever. The server's
/// <see cref="PayCostsPromptMessage.MostAffordable"/> is advice, and a seat must not depend on it to finish its turn: once a
/// payment for an X it announced is given up, every X the seat announces for the rest of that turn is less than the least
/// one it gave up on, and once that leaves no X above 0 it casts and activates nothing with X that turn. Each payment given
/// up lowers that ceiling, so the seat tries at most as many times as the X it first announced.
/// </para>
/// </summary>
public static class XCosts
{
    private static readonly ConditionalWeakTable<GameClient, Seat> Seats = new();

    /// <summary>
    /// Notes a seat's answer to an announcement or a payment: the X it confirmed, and a payment for it given up.
    /// </summary>
    public static void Answered(GameClient client, PayCostsPromptMessage prompt, TakeActionAnswer answer)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);
        ArgumentNullException.ThrowIfNull(answer);

        Seat seat = Seats.GetValue(client, _ => new Seat());
        int turn = client.View?.Turn ?? 0;
        if (seat.Turn != turn)
        {
            seat.Turn = turn;
            seat.Below = null;
        }

        if (prompt.AnnouncesXOrKicks)
        {
            // An X paid in mana, confirmed: what the payment that follows is for.
            bool paidInMana = prompt.XGoverns.Length == 0 && !prompt.XRemovesLoyalty && !prompt.XPaidInLife && prompt.SacrificeReduces == 0;
            seat.Announced = paidInMana && answer.Kind == ActionKindView.ConfirmCasting && (prompt.HasX || prompt.VariableKicker) ? prompt.X : null;
            return;
        }

        if (prompt.Attackers.Count == 0 && answer.Kind == ActionKindView.CancelCasting && seat.Announced is { } announced)
        {
            seat.Below = Math.Min(seat.Below ?? int.MaxValue, announced);
            seat.Announced = null;
        }
    }

    /// <summary>
    /// The most X this seat announces this turn: one less than the least X whose payment it gave up this turn, or null
    /// when it has given none up.
    /// </summary>
    public static int? Most(GameClient? client)
    {
        if (client is null || !Seats.TryGetValue(client, out Seat? seat) || seat.Below is not { } below || seat.Turn != (client.View?.Turn ?? 0))
        {
            return null;
        }

        return below - 1;
    }

    /// <summary>What one seat has announced and given up this turn.</summary>
    private sealed class Seat
    {
        public int Turn { get; set; }

        /// <summary>The X confirmed for the payment under way, or null.</summary>
        public int? Announced { get; set; }

        /// <summary>The least X whose payment was given up this turn, or null.</summary>
        public int? Below { get; set; }
    }
}
