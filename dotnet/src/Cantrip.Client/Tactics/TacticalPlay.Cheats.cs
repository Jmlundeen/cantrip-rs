using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// Cheating a creature into play: from the hand (Sneak Attack, Through the Breach, Flash), from a graveyard (Exhume,
/// Animate Dead, Necromancy), and putting one into a graveyard to be brought back (Entomb, Buried Alive). Each is
/// worth what it puts onto the battlefield, which the mana it costs says nothing of. All behind <c>policy.cheat</c>:
/// at zero these are read as they were, as spells the seat cannot read.
/// </summary>
public sealed partial class TacticalPlay
{
    private bool Cheating => Policy("cheat") > 0;

    private static bool FromHandText(string plain) =>
        plain.Contains("put a creature card from your hand onto the battlefield", StringComparison.Ordinal);

    /// <summary>
    /// What putting the best creature in hand onto the battlefield this way is worth, less the creature card it costs,
    /// or null when the text is no such cheat or there is nothing worth cheating. One that is sacrificed at the end of
    /// the turn is worth one hasty attack and whatever it does as it enters, and only before combat on this seat's own
    /// turn; Flash's, one kept only if its cost less two can still be paid.
    /// </summary>
    /// <param name="exclude">The card being cast to do it, which is not one of the creatures it could put in.</param>
    /// <param name="manaAfter">The mana left once the cheat itself is paid for.</param>
    private double? FromHand(GameClient client, PlayerView view, string text, ViewHandle exclude, int manaAfter) =>
        Cheated(client, view, text, exclude, manaAfter) is { Count: > 0 } each ? each.Max(one => one.Net) : null;

    /// <summary>
    /// Each creature in hand such a cheat would be worth using on, with what it would be worth (see
    /// <see cref="FromHand"/>); empty when none, or when the text is no such cheat. The same reckoning chooses the
    /// creature when the cheat resolves, so what is put in is what the cheat was cast for.
    /// </summary>
    private List<(ObjectView Creature, double Net)> Cheated(GameClient client, PlayerView view, string text, ViewHandle exclude, int manaAfter)
    {
        List<(ObjectView Creature, double Net)> each = [];
        string plain = CardReading.Plain(text);
        if (!Cheating || !FromHandText(plain))
        {
            return each;
        }

        bool temporary = plain.Contains("sacrifice the creature at the beginning of the next end step", StringComparison.Ordinal)
            || plain.Contains("sacrifice that creature at the beginning of the next end step", StringComparison.Ordinal);
        bool flash = plain.Contains("sacrifice it unless you pay its mana cost reduced by", StringComparison.Ordinal);

        if (temporary && (view.ActivePlayer != client.Seat || view.Step != TurnStepView.PrecombatMain))
        {
            return each;
        }

        int rival = Rival(client, view);
        List<Unit> blockers = [.. Creatures(client, rival).Where(creature => !creature.Tapped).Select(creature => Unit.From(creature, _model))];

        foreach (ObjectView creature in client.MyHand().Where(card => IsCreature(card) && card.Handle != exclude))
        {
            double entering = Entering(client, view, CardReading.Text(creature)) + Used(client, view, creature);
            double worth;
            if (temporary)
            {
                var combat = new Combat(_model, [Unit.From(creature, _model)], blockers, LifeOf(view, client.Seat), LifeOf(view, rival), connect: Policy("connect"));
                worth = Math.Max(combat.BestBlocks(1).Value, 0) + entering;
            }
            else if (flash)
            {
                worth = Math.Max(creature.ManaValue - 2, 0) <= manaAfter ? _model.WorthOnceCast(creature) + entering : entering;
            }
            else
            {
                worth = _model.WorthOnceCast(creature) + entering;
            }

            // A creature it could as well cast is no reason to cheat (and one only borrowed for the turn is a card
            // thrown away); one it cannot is the reason.
            if (creature.ManaValue <= manaAfter + 1)
            {
                continue;
            }

            each.Add((creature, (Policy("cheat") * worth) - _model.Card));
        }

        return each;
    }

    [System.Text.RegularExpressions.GeneratedRegex(@"^\s*pay (\d+) life: draw (a|one|two|three|four|five|six|seven|\d+) cards?")]
    private static partial System.Text.RegularExpressions.Regex LifeForCards();

    /// <summary>
    /// What a creature put in this way does for its controller at once with an ability that costs no mana:
    /// Griselbrand's seven cards for seven life, while there is life to spare (the floor <c>policy.phyrexian</c> sets).
    /// The model weighs a creature by its size, which is why a cheated Griselbrand was not worth the cheat.
    /// </summary>
    private double Used(GameClient client, PlayerView view, ObjectView creature)
    {
        int life = LifeOf(view, client.Seat);
        double worth = 0;
        foreach (string line in CardReading.Plain(CardReading.Text(creature)).Split('\n'))
        {
            if (LifeForCards().Match(line) is { Success: true } draw
                && int.TryParse(draw.Groups[1].Value, System.Globalization.NumberStyles.None, System.Globalization.CultureInfo.InvariantCulture, out int pay)
                && life - pay >= Math.Max(Policy("phyrexian"), 1))
            {
                int cards = draw.Groups[2].Value == "six" ? 6 : draw.Groups[2].Value == "seven" ? 7 : CardReading.Count(draw.Groups[2].Value);
                worth = Math.Max(worth, (cards * _model.Card) - (_model.Life(life) - _model.Life(life - pay)));
            }
        }

        return worth;
    }

    /// <summary>The creature card in this graveyard worth most on the battlefield, or 0 for none.</summary>
    private double BestDead(GameClient client, int seat) =>
        client.Zone(ZoneKindView.Graveyard, seat).Where(IsCreature).Select(_model.WorthOnceCast).DefaultIfEmpty(0).Max();

    /// <summary>
    /// What an untargeted cheat spell is worth, in the terms of <see cref="Untargeted"/> (a card's worth is taken off
    /// it afterwards), or null for text that is none.
    /// </summary>
    private double? Cheat(GameClient client, PlayerView view, string text, ViewHandle exclude, int manaAfter)
    {
        if (!Cheating)
        {
            return null;
        }

        string plain = CardReading.Plain(text);

        if (FromHand(client, view, text, exclude, manaAfter) is { } fromHand)
        {
            return fromHand;
        }

        if (FromHandText(plain))
        {
            return 0;
        }

        // Exhume: each player's best, and the opponent's comes too.
        if (plain.Contains("each player puts a creature card from their graveyard onto the battlefield", StringComparison.Ordinal))
        {
            double mine = BestDead(client, client.Seat);
            double theirs = BestDead(client, Rival(client, view));
            return mine <= 0 ? 0 : _model.Card + (Policy("cheat") * (mine - theirs));
        }

        // Entomb, Buried Alive: a card for the graveyard is a card when something in hand brings it back, and there
        // is not already something worth bringing back there.
        if (plain.Contains("search your library", StringComparison.Ordinal)
            && (plain.Contains("into your graveyard", StringComparison.Ordinal) || plain.Contains("put that card into your graveyard", StringComparison.Ordinal)))
        {
            return Reanimates(client) && BestDead(client, client.Seat) < 4 * _model.Card ? _model.Card * (1 + (0.5 * Policy("cheat"))) : null;
        }

        return null;
    }

    /// <summary>Whether this seat holds something that brings a creature back from a graveyard onto the battlefield.</summary>
    private static bool Reanimates(GameClient client) => client.MyHand().Any(card =>
    {
        string plain = CardReading.Plain(CardReading.Text(card));
        return plain.Contains("enchant creature card in a graveyard", StringComparison.Ordinal)
            || (plain.Contains("graveyard", StringComparison.Ordinal) && plain.Contains("onto the battlefield", StringComparison.Ordinal)
                && plain.Contains("creature card", StringComparison.Ordinal));
    });

    /// <summary>
    /// What a permanent whose arrival brings a creature back is worth beyond itself (Necromancy, whose target is asked
    /// for when it enters), or 0.
    /// </summary>
    private double Reanimating(GameClient client, PlayerView view, string text)
    {
        string plain = CardReading.Plain(text);

        // Only on its own main phase: Necromancy cast with flash is sacrificed, creature and all, at that cleanup.
        if (!Cheating || !plain.Contains("put target creature card from a graveyard onto the battlefield", StringComparison.Ordinal)
            || view.ActivePlayer != client.Seat || view.Step is not (TurnStepView.PrecombatMain or TurnStepView.PostcombatMain))
        {
            return 0;
        }

        return Policy("cheat") * Math.Max(BestDead(client, client.Seat), BestDead(client, Rival(client, view)));
    }

    /// <summary>An Aura that enchants a creature card in a graveyard (Animate Dead) is read as bringing it back.</summary>
    private bool AnimatesDead(string text) =>
        Cheating && CardReading.Plain(text).TrimStart().StartsWith("enchant creature card in a graveyard", StringComparison.Ordinal);
}
