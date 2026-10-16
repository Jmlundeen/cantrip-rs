using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// Everything a seat is asked besides what to do with priority and how to fight: what to aim a trigger at, which
/// cards to keep, whether to take a mulligan, whether to do what it "may".
/// </summary>
public sealed partial class TacticalPlay
{
    // --- Aiming a trigger, or a mode -----------------------------------------------------------------------------------

    private AnswerPayload Target(GameClient client, PlayerView view, ChooseTargetPromptMessage prompt)
    {
        if (prompt.Candidates.Count == 0)
        {
            return new TargetsChosenAnswer();
        }

        EffectRead effect = Reading(client, prompt.Text, prompt.Source, prompt.Ability);
        (List<TargetMessage> targets, _) = Pick(client, view, effect, prompt.Candidates, prompt.Count, prompt.Optional);

        // A slot that takes several wants that many (CR 601.2c), or its minimum where it is a printed range ("one or two
        // target creatures"), so it is filled with the best of what is left.
        int fewest = Math.Min(prompt.Minimum > 0 ? prompt.Minimum : prompt.Count, prompt.Candidates.Count);
        if (!prompt.Optional && targets.Count < fewest)
        {
            targets.AddRange(prompt.Candidates.Where(candidate => !targets.Contains(candidate)).Take(fewest - targets.Count));
        }

        return new TargetsChosenAnswer { Targets = targets };
    }

    /// <summary>
    /// What is being aimed: the text the prompt came with when it has one, and otherwise the line of the source that
    /// talks about a target.
    /// </summary>
    private static EffectRead Reading(GameClient client, string text, ViewHandle source, ViewHandle ability)
    {
        if (CardReading.Effect(text) is { Kind: not EffectKind.Unknown } stated)
        {
            return stated;
        }

        foreach (ViewHandle handle in new[] { ability, source })
        {
            if (client.Object(handle) is not { } card)
            {
                continue;
            }

            foreach (string line in CardReading.Text(card).Split('\n'))
            {
                if (line.Contains("target", StringComparison.OrdinalIgnoreCase)
                    && CardReading.Effect(line) is { Kind: not EffectKind.Unknown } read)
                {
                    return read;
                }
            }
        }

        // Unread, and so taken to be something done to an opponent — unless the prompt's own words say otherwise.
        return CardReading.Plain(text).Contains("you control", StringComparison.Ordinal) ? new EffectRead(EffectKind.Help) : default;
    }

    /// <summary>
    /// Divides damage among targets (CR 601.2d): lethal to each creature worth killing, the ones worth most per point
    /// first, and whatever is left at the opponent, or failing that at the last creature chosen.
    /// </summary>
    private AnswerPayload Divide(GameClient client, PlayerView view, ChooseDividedTargetsPromptMessage prompt)
    {
        if (prompt.Total <= 0 || prompt.Candidates.Count == 0)
        {
            return prompt.Default();
        }

        int most = prompt.MaxTargets > 0 ? prompt.MaxTargets : prompt.Candidates.Count;
        int left = prompt.Total;
        List<TargetShareMessage> shares = [];

        // One target is kept back for the opponent's face only where the face is on offer.
        int rivalSeat = Rival(client, view);
        bool faceOffered = prompt.Candidates.Exists(candidate => candidate.IsPlayer && candidate.Player == rivalSeat);
        int creatures = faceOffered ? most - 1 : most;

        foreach ((TargetMessage target, ObjectView creature, int needed) in prompt.Candidates
            .Where(candidate => !candidate.IsPlayer)
            .Select(candidate => (candidate, client.Object(candidate.Object)))
            .Where(pair => pair.Item2 is { } card && !Mine(client, card) && IsCreature(card))
            .Select(pair => (pair.candidate, pair.Item2!, Math.Max((pair.Item2!.Toughness ?? 0) - pair.Item2.DamageMarked, 1)))
            .OrderByDescending(triple => _model.Worth(triple.Item2) / triple.Item3))
        {
            if (shares.Count >= creatures || needed > left)
            {
                continue;
            }

            shares.Add(new TargetShareMessage { Target = target, Amount = needed });
            left -= needed;
        }

        if (left > 0)
        {
            int rival = Rival(client, view);
            TargetMessage? face = prompt.Candidates.Find(candidate => candidate.IsPlayer && candidate.Player == rival);

            if (face is not null && shares.Count < most)
            {
                shares.Add(new TargetShareMessage { Target = face, Amount = left });
            }
            else if (shares.Count > 0)
            {
                shares[^1].Amount += left;
            }
            else
            {
                (List<TargetMessage> targets, _) = Pick(client, view, new EffectRead(EffectKind.Damage, prompt.Total), prompt.Candidates, 1, optional: false);
                if (targets.Count == 0)
                {
                    return prompt.Default();
                }

                shares.Add(new TargetShareMessage { Target = targets[0], Amount = left });
            }
        }

        return new TargetsDividedAnswer { Shares = shares };
    }

    /// <summary>
    /// The offered creature type that most of this seat's creatures and creature cards in hand have, the earlier offered on
    /// a tie (the first eligible when none has any).
    /// </summary>
    /// <summary>What a miracle option costs: the symbols of its cost, which the server words already reduced ("Miracle {W},
    /// from Aminatou, Veil Piercer"); what is said in brackets after them (X's reduction) is not mana.</summary>
    private static int MiracleCost(string option)
    {
        string text = option.Split(", ")[0];
        int aside = text.IndexOf(" (", StringComparison.Ordinal);
        return CardReading.ManaValue(aside < 0 ? text : text[..aside]);
    }

    private static int MostCommonCreatureType(GameClient client, ChooseModesPromptMessage prompt)
    {
        List<ObjectView> mine = [.. Creatures(client, client.Seat), .. client.MyHand().Where(IsCreature)];
        int best = prompt.Eligible[0];
        int most = 0;
        foreach (int mode in prompt.Eligible)
        {
            string type = prompt.Modes[mode];
            int count = mine.Count(card => card.Subtypes.Exists(subtype => string.Equals(subtype, type, StringComparison.OrdinalIgnoreCase)));
            if (count > most)
            {
                (best, most) = (mode, count);
            }
        }

        return best;
    }

    /// <summary>The modes worth most on this board, as many as are asked for.</summary>
    private AnswerPayload Modes(GameClient client, PlayerView view, ChooseModesPromptMessage prompt)
    {
        // Which Phyrexian symbols to pay with life (CR 107.4f, 601.2b).
        if (prompt.AdditionalCost && prompt.Count == 0 && prompt.Modes.Count > 0 && prompt.Modes.TrueForAll(mode => LifeIn(mode) > 0))
        {
            return LifeForPhyrexian(client, view, prompt);
        }

        // An optional additional cost that gives something up for something: collect evidence (CR 701.59a) pays cards of the
        // graveyard for what the spell does "if evidence was collected" (Deadly Cover-Up).
        if (prompt.AdditionalCost && prompt.Count == 1 && prompt.Modes.Count == 2 && prompt.Eligible.Contains(1)
            && prompt.Modes[1].StartsWith("Collect evidence", StringComparison.Ordinal))
        {
            return new ModesChosenAnswer { Modes = [CollectsEvidence(client, view) ? 1 : 0] };
        }

        // The faces a card may be cast as (CR 709.3): each worth what it does or, for a permanent, what it is, the dearer
        // of two equal ones first.
        if (prompt.Eligible.Count > 0 && prompt.Texts.Count == prompt.Modes.Count && prompt.Modes.Count > 1)
        {
            int face = prompt.Eligible
                .OrderByDescending(mode => FaceWorth(client, view, prompt.Modes[mode], prompt.Texts[mode]))
                .ThenByDescending(mode => CardReading.ManaValue(prompt.Modes[mode]))
                .First();
            return new ModesChosenAnswer { Modes = [face] };
        }

        // An additional cost dressed as a choice ("discard a card or pay 3 life"), or a room of a dungeon: the
        // prompt's own order is as good as this seat can do.
        // "You may behold a Dragon" (CR 701.4a): offered only when there is one to choose or reveal, and it costs nothing
        // but showing a card, so it is taken for what it adds. "You may blight 1" stays declined: it shrinks a creature of
        // its own for a bonus, which this seat does not weigh yet (a known limitation: Requiting Hex's and Pyrrhic Strike's optional blight is never paid).
        if (prompt.AdditionalCost && prompt.CostKind == "Behold" && prompt.Count == 1 && prompt.Modes.Count == 2 && prompt.Eligible.Contains(1))
        {
            return new ModesChosenAnswer { Modes = [1] };
        }

        // Discarding one card instead of two (Steamcore Scholar): the cheaper discard, whenever it is offered.
        if (!prompt.AdditionalCost && prompt.Count == 1 && prompt.Modes.Count == 2 && prompt.Eligible.Contains(0)
            && prompt.Modes[0].StartsWith("Discard an instant or sorcery", StringComparison.Ordinal))
        {
            return new ModesChosenAnswer { Modes = [0] };
        }

        // "As this enters, choose a creature type" (Roaming Throne, Chronicle of Victory, Door of Destinies): every such card
        // rewards the type this seat has most of, so the type most common among its creatures on the battlefield and the
        // creature cards in its hand; with none of any offered type, the prompt's own first.
        if (prompt.Heading == "Choose a creature type" && prompt.Eligible.Count > 0)
        {
            return new ModesChosenAnswer { Modes = [MostCommonCreatureType(client, prompt)] };
        }

        // Which miracle ability to reveal a card with (CR 702.94a): each is the same cast at a different cost, so the cheapest,
        // and never one that can't be paid. The cost shown is already reduced; an X in it is left as the bot would announce it.
        if (prompt.Heading == "Choose which miracle to reveal it with" && prompt.Eligible.Count > 0)
        {
            int cheapest = prompt.Eligible
                .OrderBy(mode => prompt.Modes[mode].Contains("unpayable", StringComparison.Ordinal))
                .ThenBy(mode => MiracleCost(prompt.Modes[mode]))
                .First();
            return new ModesChosenAnswer { Modes = [cheapest] };
        }

        if (prompt.Eligible.Count == 0 || prompt.AdditionalCost || prompt.Heading.Length > 0)
        {
            return prompt.Default();
        }

        // The modes it could pay for before the ones it could not (spree), and within each the one worth most: a mode
        // chosen and then given up on while paying is a spell that is never cast.
        List<(int Mode, double Worth)> worths = [.. prompt.Eligible
            .Select(mode => (Mode: mode, Worth: mode < prompt.Modes.Count ? Mode(client, view, prompt.Modes[mode]) : 0))
            .OrderBy(pair => prompt.Affordable.Count > 0 && !prompt.Affordable.Contains(pair.Mode))
            .ThenByDescending(pair => pair.Worth)];
        List<int> ranked = [.. worths.Select(pair => pair.Mode)];

        if (prompt.MayRepeat)
        {
            return new ModesChosenAnswer { Modes = [.. Enumerable.Repeat(ranked[0], prompt.Count)] };
        }

        int take = Math.Max(prompt.Count, 1);

        // "Choose one or both" with nothing to pay for the second: every further mode that is worth doing is free.
        if (!prompt.ExtraModesCost && prompt.Most > take)
        {
            take += worths.Skip(take).Take(prompt.Most - take).Count(pair => pair.Worth > 0);
        }

        return new ModesChosenAnswer { Modes = [.. ranked.Take(take)] };
    }

    /// <summary>
    /// What one face of a card is worth cast: an effect for what it does to the best target there is, and anything else (a
    /// creature, an artifact) for what a card of that cost is worth on the table.
    /// </summary>
    private double FaceWorth(GameClient client, PlayerView view, string name, string text)
    {
        if (CardReading.Effect(text).Kind != EffectKind.Unknown)
        {
            return Mode(client, view, text);
        }

        return 0.3 + _model.Card + (0.1 * Math.Min(CardReading.ManaValue(name), 4));
    }

    /// <summary>
    /// Whether to pay collect evidence for its bonus: only with something in an opponent's graveyard for the bonus to take
    /// (it starts there), and with no card of this seat's graveyard it would rather keep - a creature to bring back, or
    /// anything with flashback, escape or the like - for the cards that pay for it are the best of them.
    /// </summary>
    private static bool CollectsEvidence(GameClient client, PlayerView view)
    {
        if (!view.Objects.Exists(card => card.Zone == ZoneKindView.Graveyard && card.ZoneOwner != client.Seat))
        {
            return false;
        }

        return !view.Objects.Exists(card => card.Zone == ZoneKindView.Graveyard && card.ZoneOwner == client.Seat
            && (IsCreature(card) || CardReading.Text(card).Contains("from your graveyard", StringComparison.OrdinalIgnoreCase)
                || CardReading.Text(card).Contains("flashback", StringComparison.OrdinalIgnoreCase)
                || CardReading.Text(card).Contains("escape", StringComparison.OrdinalIgnoreCase)
                || CardReading.Text(card).Contains("unearth", StringComparison.OrdinalIgnoreCase)));
    }

    // --- Cards ---------------------------------------------------------------------------------------------------------

    /// <summary>
    /// How much this seat wants a card it could have: a land while it is short of them, a spell it could soon cast,
    /// and little for either past that.
    /// </summary>
    private double Wanted(GameClient client, ObjectView card, int landsHeld)
    {
        int lands = Lands(client, client.Seat);

        if (IsLand(card))
        {
            return lands + landsHeld < 4 ? 1.0 : lands + landsHeld < 6 ? 0.3 : 0.05;
        }

        double worth = IsPermanentCard(card) ? _model.WorthOnceCast(card) : _model.Card + (0.1 * Math.Min(card.ManaValue, 4));
        return 0.3 + worth - (0.15 * Math.Max(card.ManaValue - lands - landsHeld - 1, 0));
    }

    /// <summary>These cards from the one this seat wants most to the one it wants least.</summary>
    private List<ViewHandle> Ranked(GameClient client, IEnumerable<ViewHandle> cards)
    {
        List<ObjectView> known = [.. cards.Select(client.Object).OfType<ObjectView>()];
        List<ViewHandle> ranked = [];
        int landsHeld = 0;

        // One at a time, because a second land is wanted less than the first.
        while (known.Count > 0)
        {
            ObjectView next = known.MaxBy(card => Wanted(client, card, landsHeld))!;
            known.Remove(next);
            ranked.Add(next.Handle);
            landsHeld += IsLand(next) ? 1 : 0;
        }

        return ranked;
    }

    /// <summary>The cards among these this seat can best do without.</summary>
    private List<ViewHandle> Least(GameClient client, PlayerView view, IReadOnlyList<ViewHandle> cards, int count)
    {
        _ = view;
        List<ViewHandle> ranked = Ranked(client, cards);
        ranked.Reverse();

        // Anything it cannot see is as good as anything else it cannot see.
        ranked.AddRange(cards.Where(card => !ranked.Contains(card)));
        return [.. ranked.Take(Math.Clamp(count, 0, cards.Count))];
    }

    /// <summary>
    /// The creature harmonize taps (CR 702.180a), up to one: none while the mana at hand pays the printed harmonize cost, since the
    /// creature is then worth more untapped (to attack, to block). Short of the mana, a creature whose power (the prompt's, which
    /// is what comes off the cost) leaves the rest payable once its own mana is no longer there (CR 702.180b: tapped for the
    /// cost, it makes none), the one worth least; else the one that comes nearest. Cost increases (Thalia) are not in the
    /// prompt, so they are not counted: the seat then answers as if there were none, and pays what the game asks.
    /// </summary>
    private AnswerPayload TapToReduce(GameClient client, ChooseCardsPromptMessage prompt)
    {
        string text = client.Object(prompt.Source) is { } spell ? CardReading.Text(spell) : string.Empty;
        System.Text.RegularExpressions.Match harmonize = System.Text.RegularExpressions.Regex.Match(text, @"Harmonize ((\{[^}]+\})+)");
        int cost = harmonize.Success ? CardReading.ManaValue(harmonize.Groups[1].Value) : 0;
        int mana = Mana(client);

        if (cost <= mana)
        {
            return new CardsChosenAnswer { Cards = [] };
        }

        List<(ViewHandle Card, int Margin)> options = [.. prompt.Eligible.Select((card, index) =>
        {
            ObjectView? creature = client.Object(card);
            int own = creature is { ProducesMana: true } && !creature.SummoningSick ? 1 : 0;
            return (card, mana - own - (cost - (index < prompt.Powers.Count ? prompt.Powers[index] : 0)));
        })];
        List<(ViewHandle Card, int Margin)> enough = [.. options.Where(option => option.Margin >= 0)];
        ViewHandle chosen = enough.Count > 0
            ? enough.MinBy(option => client.Object(option.Card) is { } card ? Worth(client, card) : 0).Card
            : options.MaxBy(option => option.Margin).Card;

        return new CardsChosenAnswer { Cards = [chosen] };
    }

    private AnswerPayload Cards(GameClient client, PlayerView view, ChooseCardsPromptMessage prompt)
    {
        // A choice that has to add up to a power (saddle, crew): the creatures that reach it and give up least, where
        // the prompt's own answer is every creature there is.
        if (prompt.MinimumPower > 0 && Policy("crew") > 0
            && Crewing(client, prompt.Eligible, prompt.Powers, prompt.MinimumPower) is { Count: > 0 } crew)
        {
            return new CardsChosenAnswer { Cards = crew };
        }

        // Collect evidence (CR 701.59a): the most valuable cards first, so the fewest cards leave the graveyard.
        if (prompt.MinimumManaValue > 0)
        {
            List<ViewHandle> taken = [];
            int total = 0;

            foreach (int index in Enumerable.Range(0, prompt.Eligible.Count)
                .OrderByDescending(index => index < prompt.ManaValues.Count ? prompt.ManaValues[index] : 0))
            {
                if (total >= prompt.MinimumManaValue)
                {
                    break;
                }

                taken.Add(prompt.Eligible[index]);
                total += index < prompt.ManaValues.Count ? prompt.ManaValues[index] : 0;
            }

            return total >= prompt.MinimumManaValue ? new CardsChosenAnswer { Cards = taken } : prompt.Default();
        }

        // Otherwise that, an order or a vote: the prompt's own answer.
        if (prompt.MinimumPower > 0 || prompt.Eligible.Count == 0
            || prompt.Purpose is CardChoicePurposeView.OrderSimultaneous or CardChoicePurposeView.Vote or CardChoicePurposeView.Splice)
        {
            return prompt.Default();
        }

        // Harmonize (CR 702.180a): the creature tapped to take its power off the generic mana of the cost. A tapped creature can
        // neither attack nor block until it untaps, so none is tapped unless the mana for the cost is not there without it.
        if (prompt.Purpose == CardChoicePurposeView.TapToReduce)
        {
            return TapToReduce(client, prompt);
        }

        int most = Math.Min(prompt.Count, prompt.Eligible.Count);

        // What to behold (CR 701.4a): a permanent on the battlefield before a card from the hand, since revealing a card tells the
        // opponent what is in it and choosing a permanent tells them nothing.
        if (prompt.Purpose == CardChoicePurposeView.Behold)
        {
            List<ViewHandle> beheld = [.. prompt.Eligible.OrderBy(card => client.Object(card)?.Zone == ZoneKindView.Battlefield ? 0 : 1)];
            return new CardsChosenAnswer { Cards = [.. beheld.Take(most)] };
        }

        // Craft's materials (CR 702.167a) are exiled for good: the fewest the ability takes (its minimum for "one or more"),
        // cards from the graveyard before permanents still at work, and among each the ones worth least.
        if (prompt.Purpose == CardChoicePurposeView.Craft)
        {
            int need = prompt.UpTo ? Math.Max(prompt.Minimum, 1) : prompt.Count;
            List<ViewHandle> materials = [.. prompt.Eligible
                .OrderBy(card => client.Object(card)?.Zone == ZoneKindView.Battlefield ? 1 : 0)
                .ThenBy(card => client.Object(card) is { } known ? Worth(client, known) : 0)];
            return new CardsChosenAnswer { Cards = [.. materials.Take(need)] };
        }

        // Giving something up, or taking something: which end of the ranking is wanted.
        bool losing = prompt.Purpose is CardChoicePurposeView.Discard or CardChoicePurposeView.Sacrifice or CardChoicePurposeView.Exile
            or CardChoicePurposeView.PutOnBottom or CardChoicePurposeView.PutIntoGraveyard or CardChoicePurposeView.PutOnTop
            or CardChoicePurposeView.ExileToPay or CardChoicePurposeView.TapToPay or CardChoicePurposeView.Destroy
            // Blight (CR 701.68a) shrinks its own creature: the one worth least takes the counters. Behold loses nothing, so the
            // ordinary best-first ranking stands (every candidate already is the quality the cost names).
            or CardChoicePurposeView.Blight
            // Counters put on its own permanent (empower Jace, amass) help it, so the best one takes them, as the ordinary ranking
            // does; harmful ones (-1/-1, stun) go on the one worth least.
            || prompt.Purpose == CardChoicePurposeView.PutCounters && prompt.Counter is "-1/-1" or "stun";

        List<ObjectView> known = [.. prompt.Eligible.Select(client.Object).OfType<ObjectView>()];
        bool theirs = known.Count > 0 && known.TrueForAll(card => card.Owner != client.Seat && card.Controller != client.Seat);
        bool board = known.Count > 0 && known.TrueForAll(card => card.Zone == ZoneKindView.Battlefield);

        // Returning its own permanent to hand is something given up — a bounce land's cost — where returning a card
        // from a graveyard is something gained.
        losing |= prompt.Purpose == CardChoicePurposeView.ReturnToHand && board && !theirs;

        // Onto the battlefield for nothing (Sneak Attack, Exhume): the one worth most there, whatever it would cost.
        // From the hand, by the reckoning the cheat was used for: Flash's has to be one it can pay for, and none of
        // them one it could as well have cast. Lands are not chosen this way (Uro's land wants a colour).
        bool free = Cheating && prompt.Purpose == CardChoicePurposeView.PutOntoBattlefield && !board && known.Count > 0
            && known.TrueForAll(card => IsPermanentCard(card) && !IsLand(card));

        if (free && client.Object(prompt.Source) is { } cheat && FromHandText(CardReading.Plain(CardReading.Text(cheat)))
            && known.TrueForAll(card => card.Zone == ZoneKindView.Hand))
        {
            List<ViewHandle> worth = [.. Cheated(client, view, CardReading.Text(cheat), ViewHandle.None, Mana(client))
                .Where(one => prompt.Eligible.Contains(one.Creature.Handle))
                .OrderByDescending(one => one.Net)
                .Select(one => one.Creature.Handle)];

            if (worth.Count > 0 || prompt.UpTo)
            {
                return new CardsChosenAnswer { Cards = [.. worth.Take(Math.Min(prompt.Count, prompt.Eligible.Count))] };
            }
        }

        List<ViewHandle> ranked = board
            ? [.. known.OrderByDescending(card => Worth(client, card)).Select(card => card.Handle)]
            : free ? [.. known.OrderByDescending(_model.WorthOnceCast).Select(card => card.Handle)]
            : Ranked(client, prompt.Eligible);

        ranked.AddRange(prompt.Eligible.Where(card => !ranked.Contains(card)));

        // The worst of its own when it is losing them, the best of an opponent's when they are: Duress takes the
        // card worth most, an edict gives up the creature worth least.
        if (losing != theirs && !free)
        {
            ranked.Reverse();
        }

        int take = most;
        if (prompt.UpTo)
        {
            if (losing && !theirs)
            {
                // "You may discard a card": only for something, and only a card it can spare — a land it has no use for.
                bool pays = client.Object(prompt.Source) is { } source
                    && CardReading.Plain(CardReading.Text(source)) is var text
                    && (text.Contains("when you do", StringComparison.Ordinal) || text.Contains("if you do", StringComparison.Ordinal));
                bool spare = client.Object(ranked[0]) is { } worst && IsLand(worst) && Lands(client, client.Seat) >= 4;
                take = pays && spare && prompt.Purpose == CardChoicePurposeView.Discard ? 1 : 0;

                // Chrome Mox was cast to be imprinted: the card wanted least that has a colour for it to make. (The
                // prompt's Source is the trigger, whose text is the imprint line. Mox Diamond's discard is not "up to":
                // the ordinary path above discards the land wanted least.)
                if (Policy("imprint") > 0 && Policy("rocks") > 0 && client.Object(prompt.Source) is { } mox
                    && CardReading.Plain(CardReading.Text(mox)).Contains("exile a nonartifact, nonland card from your hand", StringComparison.Ordinal))
                {
                    ranked = [.. ranked.Where(card => client.Object(card) is { Colors.Count: > 0 }), .. ranked.Where(card => client.Object(card) is not { Colors.Count: > 0 })];
                    take = Math.Min(1, ranked.Count);
                }
            }
        }

        // "You may cast it": a card chosen once this step and offered again was not cast - its cost could not be paid,
        // its mode had no target - and choosing it again only asks the same question again.
        if (prompt.Purpose is CardChoicePurposeView.Cast or CardChoicePurposeView.CastFree or CardChoicePurposeView.CastCopyFree or CardChoicePurposeView.Play)
        {
            ranked.RemoveAll(card => _offered.Contains(card.Value));
            take = Math.Min(take, ranked.Count);

            if (take == 0 && !prompt.UpTo)
            {
                return prompt.Default();
            }

            foreach (ViewHandle chosen in ranked.Take(take))
            {
                _offered.Add(chosen.Value);
            }
        }

        _ = view;
        return new CardsChosenAnswer { Cards = [.. ranked.Take(take)] };
    }

    /// <summary>
    /// From a library: a land that makes a color this seat is short of when it is lands on offer, and otherwise the
    /// card it wants most.
    /// </summary>
    private AnswerPayload Search(GameClient client, PlayerView view, SearchLibraryPromptMessage prompt)
    {
        _ = view;
        List<ObjectView> eligible = [.. prompt.Eligible.Select(client.Object).OfType<ObjectView>()];
        int most = Math.Min(prompt.Count, prompt.Eligible.Count);

        if (eligible.Count == 0 || most <= 0)
        {
            return new CardsChosenAnswer { Cards = [.. prompt.Eligible.Take(most)] };
        }

        if (eligible.TrueForAll(IsLand))
        {
            HashSet<ColorView> have = [.. client.BattlefieldOf(client.Seat).Where(IsLand).SelectMany(Makes)];
            HashSet<ColorView> want = [.. client.MyHand().Where(card => !IsLand(card)).SelectMany(card => card.Colors)];

            return new CardsChosenAnswer
            {
                Cards = [.. eligible
                    .OrderByDescending(land => Makes(land).Count(color => want.Contains(color) && !have.Contains(color)))
                    .ThenByDescending(land => Makes(land).Count)
                    .ThenBy(land => CardReading.Plain(CardReading.Text(land)).Contains("enters tapped", StringComparison.Ordinal))
                    .Take(most)
                    .Select(land => land.Handle)],
            };
        }

        // Onto the battlefield, nothing is paid for it: the permanent worth most there, whatever it would cost to cast
        // (Tinker's Colossus, Natural Order's biggest creature), not the one this seat could soonest afford.
        if (prompt.Destination == SearchDestinationView.Battlefield && Policy("fetch") > 0 && eligible.Exists(card => IsPermanentCard(card) && !IsLand(card)))
        {
            return new CardsChosenAnswer
            {
                Cards = [.. eligible.Where(card => IsPermanentCard(card) && !IsLand(card))
                    .OrderByDescending(_model.WorthOnceCast).Take(most).Select(card => card.Handle)],
            };
        }

        // Into the graveyard is where a card goes to be brought back: the biggest creature there is.
        if (prompt.Destination == SearchDestinationView.Graveyard && eligible.Exists(IsCreature))
        {
            return new CardsChosenAnswer
            {
                Cards = [.. eligible.Where(IsCreature).OrderByDescending(_model.WorthOnceCast).Take(most).Select(card => card.Handle)],
            };
        }

        return new CardsChosenAnswer { Cards = [.. Ranked(client, prompt.Eligible).Take(most)] };
    }

    /// <summary>
    /// A scry or a surveil: a card it wants stays on top, one it does not goes wherever else the prompt offers.
    /// </summary>
    private AnswerPayload Distribute(GameClient client, PlayerView view, DistributeCardsPromptMessage prompt)
    {
        _ = view;
        int keep = prompt.Destinations.FindIndex(destination => destination.Kind is CardDestinationKindView.LibraryTop or CardDestinationKindView.Hand);
        int away = prompt.Destinations.FindLastIndex(destination =>
            destination.Kind is CardDestinationKindView.LibraryBottom or CardDestinationKindView.Graveyard);

        if (keep < 0 || away < 0 || keep == away)
        {
            return AutoPlay.Distribute(client, prompt);
        }

        int held = client.MyHand().Count(IsLand);
        var answer = new CardsDistributedAnswer { Placements = [.. prompt.Destinations.Select(_ => new List<ViewHandle>())] };

        foreach (ViewHandle handle in prompt.Cards)
        {
            bool wanted = client.Object(handle) is not { } card || Wanted(client, card, held) >= 0.3;
            answer.Placements[wanted ? keep : away].Add(handle);
        }

        return prompt.Problem(answer) is null ? answer : AutoPlay.Distribute(client, prompt);
    }

    /// <summary>
    /// Keeps a hand it can play: two to five lands of seven, and anything at all once it is down to five cards,
    /// where another mulligan costs more than a clumsy hand does.
    /// </summary>
    private static AnswerPayload Mulligan(GameClient client, MulliganPromptMessage prompt)
    {
        IReadOnlyList<ObjectView> hand = client.MyHand();
        int lands = hand.Count(IsLand);

        bool keep = prompt.Keeps <= 5 || hand.Count == 0
            || (prompt.Keeps >= 7 ? lands is >= 2 and <= 5 : lands is >= 2 and <= 4);

        return new MayChosenAnswer { Accept = !keep };
    }

    // --- Yes or no, who, and what color --------------------------------------------------------------------------------

    private AnswerPayload May(GameClient client, PlayerView view, MayPromptMessage prompt)
    {
        int life = LifeOf(view, client.Seat);

        bool accept = prompt.Verb switch
        {
            // A land untapped is worth two life while there is life to spend and something to cast with it.
            "PayLife" => life - prompt.Amount >= 6
                && client.MyHand().Any(card => !IsLand(card) && card.ManaValue <= Lands(client, client.Seat) + 1),
            "PayLifeUnlessCountered" => life - prompt.Amount >= 4,

            // Master of Death: one life for a creature back in hand, while life is not short.
            "PayLifeToReturn" => life - prompt.Amount >= 5,
            "DiscardUnlessCountered" => client.MyHand().Count > prompt.Amount,

            // Ad Nauseam: the next card's price is unknown, so keep going only while one more costly card cannot
            // kill — life loss is not a payment and nothing stops a player revealing past zero.
            "RevealAgain" => life >= 10,

            // Pia's Revolution: three damage to keep an artifact out of its owner's hand, while it cannot get close.
            "TakeDamageInsteadOfReturn" => life - prompt.Amount >= 10,

            // Gift (CR 702.174a): what the opponent is given is a price for what the spell does more. A Food or a tapped 1/1
            // Fish is small; a card, a Treasure, an 8/8 Octopus or an extra turn is worth more to them than any spell's
            // better mode is to this seat, so those are never promised.
            "PromiseGiftFood" or "PromiseGiftTappedFish" => true,
            "PromiseGiftCard" or "PromiseGiftTreasure" or "PromiseGiftOctopus" or "PromiseGiftExtraTurn" => false,

            // Each of these is a way to lose to itself, or to hand an opponent something.
            "PayLifeToLookAgain" or "Sacrifice" or "HaveControllerDraw" or "PutIntoGraveyard" or "PutOnBottom"
                or "ChooseOddOrEven" or "ProtectFromColorless" => false,

            // Chain Lightning's copy, when there is something for it to hit — and only a few times a turn, and only
            // of a spell that removes something. Two seats that always copy Chain of Smog pass it back and forth for
            // ever.
            "CopySpell" => _copies < CopiesPerTurn
                && client.Object(prompt.Source) is { } copied
                && CardReading.Effect(CardReading.Text(copied).Split('.')[0]) is { Kind: EffectKind.Damage or EffectKind.Remove } copy
                && Pick(client, view, copy, Guess(client, view, CardReading.Text(copied).Split('.')[0]), 1, optional: true).Worth > Policy("margin")
                && ++_copies > 0,

            // Obeka (CR 724.1): asked of the player whose turn it is, so ending it gives up the rest of this seat's own turn.
            // Worth it only to stop an opponent's spell or ability waiting beneath the resolving one, when none of this
            // seat's own would be exiled with it.
            "EndTurn" => client.Zone(ZoneKindView.Stack, 0) is { Count: > 1 } stack
                && stack.Take(stack.Count - 1).All(item => !Mine(client, item)),

            // Everything else a card says its controller "may" do is something a card's controller wants done: a
            // search, a draw, a free spell, an untap.
            _ => true,
        };

        return new MayChosenAnswer { Accept = accept };
    }

    private static AnswerPayload Player(GameClient client, ChoosePlayerPromptMessage prompt)
    {
        if (prompt.Candidates.Count == 0)
        {
            return prompt.Default();
        }

        bool good = prompt.Verb is "Draw" or "GainLife" or "Scry" or "Surveil" or "CreateTreasure" or "Untap";
        int seat = good && prompt.Candidates.Contains(client.Seat)
            ? client.Seat
            : prompt.Candidates.FirstOrDefault(candidate => candidate != client.Seat, prompt.Candidates[0]);

        return new PlayerChosenAnswer { Player = seat };
    }

    /// <summary>
    /// A color: for protection, that of whatever is being protected from; for mana, one its hand asks for; and
    /// otherwise the one its opponents have most of.
    /// </summary>
    private AnswerPayload Colors(GameClient client, PlayerView view, ChooseColorsPromptMessage prompt)
    {
        if (prompt.Choices.Count == 0)
        {
            return prompt.Default();
        }

        IEnumerable<ColorView> theirs = view.Objects
            .Where(card => card.Zone is ZoneKindView.Battlefield or ZoneKindView.Stack && !Mine(client, card))
            .SelectMany(card => card.Colors);

        IEnumerable<ColorView> source = prompt.Verb == "AddMana"
            ? client.MyHand().Where(card => !IsLand(card)).SelectMany(card => card.Colors)
            : _threat.Count > 0 ? _threat : theirs;

        List<ColorView> ranked = [.. source
            .Where(prompt.Choices.Contains)
            .GroupBy(color => color)
            .OrderByDescending(group => group.Count())
            .Select(group => group.Key)];

        ranked.AddRange(prompt.Choices.Where(color => !ranked.Contains(color)));
        _threat = [];

        return new ColorsChosenAnswer
        {
            Colors = prompt.MayRepeat ? [.. Enumerable.Repeat(ranked[0], prompt.Count)] : [.. ranked.Take(prompt.Count)],
        };
    }
}
