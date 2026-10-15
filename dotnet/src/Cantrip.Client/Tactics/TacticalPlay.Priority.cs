using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// What this seat does with priority: which land, which spells, at what, and when to keep its mana.
/// <para>
/// Each thing it could do is given a gain — what the position would be worth afterwards less what it is worth now,
/// in the model's units, the card spent counted as a card. A creature is worth what it would be on the battlefield,
/// a removal spell what it removes, burn at a player what the life is worth at that life total, and anything whose
/// text this seat cannot read is worth very little. Then it takes the set of them that gains most for the mana it
/// has, one action at a time, looking again after each.
/// </para>
/// </summary>
public sealed partial class TacticalPlay
{
    /// <summary>One thing that could be done with priority, and what doing it is thought to be worth.</summary>
    private sealed record Candidate(
        ActionOptionMessage Option, double Gain, int Cost, List<TargetMessage> Targets, bool Kick, bool Permanent, bool Hostile);

    private AnswerPayload Priority(GameClient client, PlayerView view, PriorityPromptMessage prompt)
    {
        IReadOnlyList<ObjectView> stack = client.Zone(ZoneKindView.Stack, 0);

        // Whatever was last tried has had its chance once the stack is empty again. A board that reads the same as
        // before it — a spell given up while paying, an ability that resolved and changed nothing — says it was not
        // worth doing, and it is not done again this turn: which is what stops a seat with no rules engine from
        // activating Figure of Destiny's first ability for ever.
        if (_pending is { } pending && stack.Count == 0)
        {
            if (pending.Board == Board(view))
            {
                _spent.Add(pending.Key);
            }

            _pending = null;
        }

        if (++_actions > ActionsPerStep)
        {
            return new PassAnswer();
        }

        if (stack.Count > 0)
        {
            // Its own spell is let resolve (CR 117.3c gives the caster priority first; nothing is gained by using it).
            ObjectView top = stack[^1];
            return top.Controller == client.Seat ? new PassAnswer() : Respond(client, view, prompt, top);
        }

        bool mine = view.ActivePlayer == client.Seat;

        if (mine && view.Step is TurnStepView.PrecombatMain or TurnStepView.PostcombatMain)
        {
            if (Land(client, view, prompt) is { } land)
            {
                return land;
            }

            // A Vehicle is crewed last of all, by whatever the turn's spells left with nothing better to do: a
            // creature cast this turn cannot attack, and it can crew.
            AnswerPayload best = Best(client, view, prompt, hostileOnly: false);
            return best is PassAnswer && view.Step == TurnStepView.PrecombatMain && Crew(client, view, prompt) is { } crewed ? crewed : best;
        }

        // On an opponent's turn: removal once attackers are declared, when what is coming is known and before it is
        // blocked, and at their end step whatever the mana left untapped all turn will still buy.
        if (!mine && view.Step == TurnStepView.DeclareAttackers)
        {
            return Best(client, view, prompt, hostileOnly: true);
        }

        if (!mine && view.Step == TurnStepView.End)
        {
            return Best(client, view, prompt, hostileOnly: false);
        }

        return new PassAnswer();
    }

    // --- Vehicles ----------------------------------------------------------------------------------------------------

    /// <summary>
    /// Crews a Vehicle that is then worth attacking with (CR 702.122): one that could attack this turn, would come out
    /// of the fight ahead against the blocks that are best against it, and hits harder than the creatures tapped to
    /// crew it would have. Nothing is crewed to block, and nothing by a creature that would rather attack itself.
    /// </summary>
    private TakeActionAnswer? Crew(GameClient client, PlayerView view, PriorityPromptMessage prompt)
    {
        if (Policy("crew") <= 0)
        {
            return null;
        }

        int rival = Rival(client, view);

        foreach (ActionOptionMessage option in prompt.Options)
        {
            int needed = CardReading.Crew(option.Description);

            if (option.Kind != ActionKindView.ActivateAbility || needed <= 0
                || _spent.Contains((option.Source.Value, option.Kind, option.AbilityIndex))
                || client.Object(option.Source) is not { Zone: ZoneKindView.Battlefield } vehicle
                || IsCreature(vehicle) || vehicle.Tapped || vehicle.Power is not > 0)
            {
                continue;
            }

            Keywords keys = CardReading.KeywordsOf(vehicle);
            // Arrived this turn: crewed, it would be a creature that cannot attack (CR 302.6). The server does not say
            // so of a permanent that is not yet a creature, so it is told by what was there when the turn began.
            if ((keys & Keywords.Haste) == 0 && (vehicle.SummoningSick || (_settled is { } settled && !settled.Contains(vehicle.Handle))))
            {
                continue;
            }

            List<ObjectView> able = [.. Creatures(client, client.Seat).Where(creature => !creature.Tapped && creature.Handle != vehicle.Handle)];
            List<ViewHandle> crew = Crewing(client, [.. able.Select(creature => creature.Handle)], [.. able.Select(creature => creature.Power ?? 0)], needed);
            if (crew.Count == 0)
            {
                continue;
            }

            // What the crew would have attacked with themselves, against what the Vehicle brings.
            int given = crew.Select(client.Object).OfType<ObjectView>().Where(CouldAttack).Sum(creature => creature.Power ?? 0);
            int brings = (vehicle.Power ?? 0)
                + ((keys & (Keywords.Flying | Keywords.Unblockable | Keywords.Menace | Keywords.Trample)) != 0 ? 1 : 0)
                + (CardReading.AttackTrigger(vehicle.OracleText) ? 1 : 0);

            if (brings <= given)
            {
                continue;
            }

            List<ObjectView> theirs = [.. Creatures(client, rival)];
            List<Unit> blockers = [.. theirs.Where(creature => !creature.Tapped).Select(creature => Unit.From(creature, _model))];
            var combat = new Combat(_model, [Unit.From(vehicle, _model)], blockers, LifeOf(view, client.Seat), LifeOf(view, rival), connect: Policy("connect"));

            // The crew stay tapped through the opponent's turn (CR 502.3 untaps them only in this seat's own), so the
            // attack back meets a board without them: what more of it gets through is part of the price.
            int life = LifeOf(view, client.Seat);
            List<Unit> coming = [.. theirs.Where(creature => (creature.Power ?? 0) > 0).Select(creature => Unit.From(creature, _model))
                .Where(creature => !creature.Has(Keywords.Defender))];
            List<Unit> home = [.. able.Select(creature => Unit.From(creature, _model))];
            List<Unit> left = [.. home.Where(creature => !crew.Contains(creature.Handle))];
            int openWithout = Exposure(coming, home);
            int openWith = Exposure(coming, left);

            if (openWith >= life && openWithout < life)
            {
                continue;
            }

            double exposed = Policy("crackback") * ((_model.Life(life - openWithout) - _model.Life(life - openWith)));
            if (combat.BestBlocks(1).Value - exposed <= Policy("margin"))
            {
                continue;
            }

            return Take(view, new Candidate(option, 0, 0, [], Kick: false, Permanent: false, Hostile: false));
        }

        return null;
    }

    private static bool CouldAttack(ObjectView creature) =>
        !creature.Tapped && (!creature.SummoningSick || (CardReading.KeywordsOf(creature) & Keywords.Haste) != 0)
        && (CardReading.KeywordsOf(creature) & Keywords.Defender) == 0;

    /// <summary>
    /// The creatures to tap for a crew cost: the set that adds up to the power asked for and gives up least — the
    /// ones that could not have attacked this turn before the ones that could, and no more of either than it takes.
    /// Empty when no set reaches it.
    /// </summary>
    private List<ViewHandle> Crewing(GameClient client, IReadOnlyList<ViewHandle> eligible, IReadOnlyList<int> powers, int needed)
    {
        int count = Math.Min(eligible.Count, 12);
        double[] given = new double[count];

        for (int i = 0; i < count; i++)
        {
            // What tapping it gives up: its own attack if it had one, and a little for being tapped on the way back.
            ObjectView? creature = client.Object(eligible[i]);
            int power = i < powers.Count ? powers[i] : creature?.Power ?? 0;
            given[i] = 0.2 + (0.05 * Math.Max(power, 0)) + (creature is not null && CouldAttack(creature) ? Math.Max(power, 0) + _model.Worth(creature) : 0);
        }

        int chosen = 0;
        double least = double.MaxValue;

        for (int set = 1; set < 1 << count; set++)
        {
            int total = 0;
            double cost = 0;

            for (int i = 0; i < count; i++)
            {
                if ((set & (1 << i)) != 0)
                {
                    total += i < powers.Count ? powers[i] : client.Object(eligible[i])?.Power ?? 0;
                    cost += given[i];
                }
            }

            if (total >= needed && cost < least)
            {
                least = cost;
                chosen = set;
            }
        }

        return [.. eligible.Where((_, i) => i < count && (chosen & (1 << i)) != 0)];
    }

    // --- Lands -------------------------------------------------------------------------------------------------------

    /// <summary>The land to play, when one may be: there is never a reason not to play one.</summary>
    private TakeActionAnswer? Land(GameClient client, PlayerView view, PriorityPromptMessage prompt)
    {
        ActionOptionMessage? best = null;
        double bestScore = double.MinValue;

        List<ObjectView> hand = [.. client.MyHand()];
        int lands = Lands(client, client.Seat);
        HashSet<ColorView> have = [.. client.BattlefieldOf(client.Seat).Where(IsLand).SelectMany(Makes)];
        HashSet<ColorView> want = [.. hand.Where(card => !IsLand(card)).SelectMany(card => card.Colors)];

        // Whether some spell in hand is castable this turn only if this land comes in untapped.
        bool onCurve = hand.Exists(card => !IsLand(card) && card.ManaValue == lands + 1);

        foreach (ActionOptionMessage option in prompt.Options)
        {
            if (option.Kind != ActionKindView.PlayLand || client.Object(option.Source) is not { } land)
            {
                continue;
            }

            string text = CardReading.Plain(land.OracleText);
            List<ColorView> makes = Makes(land);

            double score = makes.Count(color => want.Contains(color) && !have.Contains(color))
                + (0.1 * makes.Count(want.Contains))
                + (makes.Count == 0 ? -0.2 : 0);

            if (text.Contains("enters tapped", StringComparison.Ordinal) && !text.Contains("unless", StringComparison.Ordinal))
            {
                // A tapped land costs a turn of its mana, so it is played on the turn that mana had no use.
                score += onCurve ? -0.5 : 0.3;
            }

            if (score > bestScore)
            {
                bestScore = score;
                best = option;
            }
        }

        return best is null ? null : AutoPlay.Answer(best);
    }

    /// <summary>The colors of mana a land makes, read off its basic land types and its text.</summary>
    private static List<ColorView> Makes(ObjectView land)
    {
        List<ColorView> colors = [];
        void Add(ColorView color)
        {
            if (!colors.Contains(color))
            {
                colors.Add(color);
            }
        }

        foreach (string subtype in land.Subtypes)
        {
            switch (subtype)
            {
                case "Plains": Add(ColorView.White); break;
                case "Island": Add(ColorView.Blue); break;
                case "Swamp": Add(ColorView.Black); break;
                case "Mountain": Add(ColorView.Red); break;
                case "Forest": Add(ColorView.Green); break;
            }
        }

        string text = land.OracleText;

        // A fetch land makes whatever the land it finds does.
        int search = text.IndexOf("Search your library for", StringComparison.Ordinal);
        if (search >= 0)
        {
            string finds = text[search..];
            if (finds.Contains("basic land", StringComparison.Ordinal))
            {
                return [ColorView.White, ColorView.Blue, ColorView.Black, ColorView.Red, ColorView.Green];
            }

            if (finds.Contains("Plains", StringComparison.Ordinal)) Add(ColorView.White);
            if (finds.Contains("Island", StringComparison.Ordinal)) Add(ColorView.Blue);
            if (finds.Contains("Swamp", StringComparison.Ordinal)) Add(ColorView.Black);
            if (finds.Contains("Mountain", StringComparison.Ordinal)) Add(ColorView.Red);
            if (finds.Contains("Forest", StringComparison.Ordinal)) Add(ColorView.Green);
        }

        int add = text.IndexOf("Add ", StringComparison.Ordinal);
        if (add >= 0)
        {
            string adds = text[add..];
            if (adds.Contains("any color", StringComparison.Ordinal))
            {
                return [ColorView.White, ColorView.Blue, ColorView.Black, ColorView.Red, ColorView.Green];
            }

            if (adds.Contains("{W}", StringComparison.Ordinal)) Add(ColorView.White);
            if (adds.Contains("{U}", StringComparison.Ordinal)) Add(ColorView.Blue);
            if (adds.Contains("{B}", StringComparison.Ordinal)) Add(ColorView.Black);
            if (adds.Contains("{R}", StringComparison.Ordinal)) Add(ColorView.Red);
            if (adds.Contains("{G}", StringComparison.Ordinal)) Add(ColorView.Green);
        }

        return colors;
    }

    // --- Choosing what to do -----------------------------------------------------------------------------------------

    /// <summary>The mana this seat could make right now: what is in its pool and every untapped source it controls.</summary>
    private int Mana(GameClient client)
    {
        int pool = client.Me is { } me ? me.Mana.Sum() + me.RestrictedMana.Sum(mana => mana.Amount) : 0;
        bool counted = Policy("rocks") > 0;

        // A source makes one unless its text says more: Sol Ring's two are two a plan can spend.
        return pool + client.BattlefieldOf(client.Seat)
            .Where(source => source.ProducesMana && !source.Tapped && (!IsCreature(source) || !source.SummoningSick))
            .Where(source => !counted || !Idle(client, source))
            .Sum(source => counted && !IsCreature(source) ? Math.Max(CardReading.ManaMade(source.OracleText).Amount, 1) : 1);
    }

    /// <summary>
    /// Whether a source makes mana only of the colours of its controller's legendary creatures and planeswalkers (Mox
    /// Amber) and this seat has none: it makes nothing, and is neither mana to plan with nor a land to have cast.
    /// </summary>
    private static bool Idle(GameClient client, ObjectView source) =>
        CardReading.Plain(source.OracleText).Contains("among legendary creatures and planeswalkers you control", StringComparison.Ordinal)
        && !client.BattlefieldOf(client.Seat).Any(permanent => permanent.Supertypes.Contains(SupertypeView.Legendary)
            && (IsCreature(permanent) || permanent.CardTypes.Contains(CardTypeView.Planeswalker)));

    /// <summary>
    /// The best thing to do now: the first action of the set of candidates that gains most within the mana there is,
    /// or a pass when nothing gains anything.
    /// </summary>
    private AnswerPayload Best(GameClient client, PlayerView view, PriorityPromptMessage prompt, bool hostileOnly)
    {
        int budget = Mana(client);
        List<Candidate> candidates = Candidates(client, view, prompt, budget);
        HashSet<int> weighed = Narrating ? [.. candidates.Select(candidate => candidate.Option.Source.Value)] : [];

        if (hostileOnly)
        {
            candidates.RemoveAll(candidate => !candidate.Hostile);
        }

        double margin = Policy("margin");
        if (Narrating && candidates.Count > 0)
        {
            Note($"weighed (mana {budget}, worth doing above {Num(margin)}): "
                + string.Join("; ", candidates.OrderByDescending(candidate => candidate.Gain).Take(8).Select(candidate => Told(client, candidate))));
        }

        // A spell it could have cast and did not even weigh is the first thing a reader asks about.
        if (Narrating)
        {
            List<string> passedOver = [.. prompt.Options
                .Where(option => option.Kind == ActionKindView.CastSpell && option.Affordable && !weighed.Contains(option.Source.Value))
                .Select(option => option.SpellName.Length > 0 ? option.SpellName : NameOf(client, option.Source))
                .Distinct()];
            if (passedOver.Count > 0)
            {
                Note($"castable, not weighed (unread, or nothing for it to do): {string.Join(", ", passedOver)}");
            }
        }

        candidates.RemoveAll(candidate => candidate.Gain <= margin);

        // Draw-go: with mana to keep up for a counterspell, an instant or a flash card is not cast into that mana on
        // its own turn but at the end of the opponent's (see Priority), once the counterspell has not been needed.
        if (Policy("drawgo") > 0 && view.ActivePlayer == client.Seat && Reserve(client, view, budget) > 0)
        {
            List<string> waiting = [.. candidates
                .Where(candidate => candidate.Option.Kind == ActionKindView.CastSpell && Instantly(client, candidate.Option))
                .Select(candidate => NameOf(client, candidate.Option.Source))];
            if (waiting.Count > 0)
            {
                candidates.RemoveAll(candidate => candidate.Option.Kind == ActionKindView.CastSpell && Instantly(client, candidate.Option));
                if (Narrating)
                {
                    Note($"waiting for the end of their turn, with a counterspell up: {string.Join(", ", waiting)}");
                }
            }
        }

        if (candidates.Count == 0)
        {
            return new PassAnswer();
        }

        candidates = [.. candidates.OrderByDescending(candidate => candidate.Gain).Take(12)];

        // Every set of them that the mana covers, one action per card. Mana used is worth a little in itself, so of
        // two plans that gain the same the one that spends the turn's mana is taken.
        double tempo = Policy("tempo");

        (int Set, double Total) Plan(int mana)
        {
            double most = double.MinValue;
            int chosen = 0;

            for (int set = 1; set < 1 << candidates.Count; set++)
            {
                int cost = 0;
                double total = 0;
                bool clash = false;
                HashSet<int> sources = [];

                for (int i = 0; i < candidates.Count && !clash; i++)
                {
                    if ((set & (1 << i)) == 0)
                    {
                        continue;
                    }

                    cost += candidates[i].Cost;
                    total += candidates[i].Gain + (tempo * candidates[i].Cost);
                    clash = !sources.Add(candidates[i].Option.Source.Value);
                }

                if (!clash && cost <= mana && total > most)
                {
                    most = total;
                    chosen = set;
                }
            }

            return (chosen, most);
        }

        // On its own turn, with a counterspell in hand and an opponent with cards to cast: the plan is made with that
        // much mana kept back, and the whole of it is spent only on plays worth leaving the counterspell stranded for.
        int reserve = view.ActivePlayer == client.Seat ? Reserve(client, view, budget) : 0;
        (int bestSet, double bestTotal) = Plan(budget - reserve);

        if (reserve > 0)
        {
            (int allIn, double allInTotal) = Plan(budget);
            bool tapOut = allInTotal - Math.Max(bestTotal, 0) >= Policy("tapout");
            if (Narrating)
            {
                Note(tapOut
                    ? $"not keeping {reserve} mana up for a counterspell: tapping out gains {Num(allInTotal - Math.Max(bestTotal, 0))}, at least {Num(Policy("tapout"))}"
                    : $"keeping {reserve} mana up for a counterspell");
            }

            if (tapOut)
            {
                (bestSet, reserve) = (allIn, 0);
            }
            else if (bestSet == 0)
            {
                return new PassAnswer();
            }
        }

        // What it costs was an estimate, and the server said each of these could be paid for on its own: when no set
        // fits the estimate, the single best one is tried and the payment finds out.
        List<Candidate> plan = bestSet == 0
            ? [.. candidates.Where(candidate => candidate.Option.Kind != ActionKindView.CastSpell || candidate.Option.AbilityIndex == 0).Take(1)]
            : [.. candidates.Where((_, index) => (bestSet & (1 << index)) != 0)];

        if (plan.Count == 0)
        {
            return new PassAnswer();
        }

        if (Narrating && plan.Count > 1)
        {
            Note("plan this turn: " + string.Join("; ", plan.Select(candidate => Told(client, candidate))));
        }

        // Permanents first: whatever triggers on a noncreature spell is then already there to see the others cast.
        Candidate first = plan.OrderByDescending(candidate => candidate.Permanent).ThenByDescending(candidate => candidate.Gain).First();
        return Take(view, first);
    }

    private TakeActionAnswer Take(PlayerView view, Candidate candidate)
    {
        ActionOptionMessage option = candidate.Option;

        _kick = candidate.Kick;
        _pending = ((option.Source.Value, option.Kind, option.AbilityIndex), Board(view));

        TakeActionAnswer answer = AutoPlay.Answer(option);
        answer.Targets = candidate.Targets;
        return answer;
    }

    private List<Candidate> Candidates(GameClient client, PlayerView view, PriorityPromptMessage prompt, int budget)
    {
        List<Candidate> candidates = [];
        int rival = Rival(client, view);

        foreach (ActionOptionMessage option in prompt.Options)
        {
            if (_spent.Contains((option.Source.Value, option.Kind, option.AbilityIndex))
                || client.Object(option.Source) is not { } card
                || (!option.Affordable && !ByLife(client, view, option, card, budget)))
            {
                continue;
            }

            if (option.Kind == ActionKindView.CastSpell)
            {
                // The printed cost when it can be paid, rather than a dash this seat cannot weigh; and never a cost
                // paid in something other than mana — an evoke, a pitch — which is a trade it cannot price.
                if (option.AbilityIndex > 0
                    && (CardReading.ManaValue(option.Description) == 0
                        || prompt.Options.Exists(other =>
                            other.Kind == ActionKindView.CastSpell && other.Source == option.Source && other.AbilityIndex == 0 && other.Affordable)))
                {
                    continue;
                }

                candidates.AddRange(Casts(client, view, option, card, budget));
            }
            else if (option.Kind == ActionKindView.ActivateAbility
                && option.Produces.Count == 0 && option.ProducesOfAnyOneColor == 0 && option.ProducesOfAnyColorEach == 0)
            {
                if (Activation(client, view, option, card) is { } activation)
                {
                    candidates.Add(activation);
                }
            }
        }

        // Burn that would end the game between it is not weighed against anything.
        List<Candidate> reach = [.. candidates
            .Where(candidate => candidate.Option.Kind == ActionKindView.CastSpell && Burn(client, candidate, rival) > 0)
            .GroupBy(candidate => candidate.Option.Source)
            .Select(group => group.OrderByDescending(candidate => Burn(client, candidate, rival) / (double)Math.Max(candidate.Cost, 1)).First())
            .OrderBy(candidate => candidate.Cost)];

        int left = budget;
        int damage = 0;
        List<Candidate> thrown = [];
        foreach (Candidate candidate in reach)
        {
            if (candidate.Cost <= left)
            {
                left -= candidate.Cost;
                damage += Burn(client, candidate, rival);
                thrown.Add(candidate);
            }
        }

        if (damage >= LifeOf(view, rival) && thrown.Count > 0)
        {
            return [.. thrown.Select(candidate => candidate with
            {
                Gain = Combat.Win,
                Targets = [new TargetMessage { Player = rival }],
            })];
        }

        return candidates;
    }

    /// <summary>The damage a spell would deal to this opponent if aimed at them, or zero if it could not be.</summary>
    private int Burn(GameClient client, Candidate candidate, int rival)
    {
        if (!candidate.Option.TargetCandidates.Exists(target => target.IsPlayer && target.Player == rival)
            || candidate.Option.SpellName.Length > 0
            || client.Object(candidate.Option.Source) is not { } card
            || CardReading.Effect(card.OracleText) is not { Kind: EffectKind.Damage } effect)
        {
            return 0;
        }

        return candidate.Kick && effect.Kicked > 0 ? effect.Kicked
            : effect.Amount < 0 ? Math.Max(candidate.Option.MaxX - (client.View is { } view ? LethalMana(client, view) : 0), 0)
            : effect.Amount;
    }

    /// <summary>
    /// How many of a card's Phyrexian mana symbols (CR 107.4f) this seat would pay for in life: as many as it has two
    /// life for above the floor <c>policy.phyrexian</c> sets, and none when that is zero.
    /// </summary>
    private int LifeSymbols(GameClient client, PlayerView view, ObjectView card)
    {
        double floor = Policy("phyrexian");
        int symbols = floor > 0 ? CardReading.Phyrexian(card.ManaCost) : 0;
        if (symbols == 0)
        {
            return 0;
        }

        return Math.Clamp((int)((LifeOf(view, client.Seat) - floor) / 2), 0, symbols);
    }

    /// <summary>
    /// Whether a spell the server called unaffordable is one this seat could still cast by paying life for its
    /// Phyrexian symbols. The server's word is about mana alone, so Dismember with one land out is "unaffordable",
    /// and a seat that took that word for it would never cast it for one mana and four life.
    /// </summary>
    private bool ByLife(GameClient client, PlayerView view, ActionOptionMessage option, ObjectView card, int budget) =>
        option.Kind == ActionKindView.CastSpell && option.AbilityIndex == 0 && !option.HasX
        && LifeSymbols(client, view, card) is > 0 and int symbols
        && card.ManaValue - symbols <= budget;

    /// <summary>The ways one cast could go — kicked and not, where that changes what it does — each with its gain.</summary>
    private IEnumerable<Candidate> Casts(GameClient client, PlayerView view, ActionOptionMessage option, ObjectView card, int budget)
    {
        // An adventure, or anything else that casts a spell other than the card: its text is not the card's, so it
        // is weighed as something unread.
        bool other = option.SpellName.Length > 0 && !string.Equals(option.SpellName, card.Name, StringComparison.Ordinal);
        string text = other ? string.Empty : card.OracleText;
        bool permanent = !other && IsPermanentCard(card);

        int cost = option.AbilityIndex == 0 ? card.ManaValue : CardReading.ManaValue(option.Description) is > 0 and int alternative ? alternative : card.ManaValue;

        // A Phyrexian symbol can be paid for in life, so a spell with one that the mana does not reach is counted as
        // costing that much less, for as many symbols as there is life to spare for.
        if (option.AbilityIndex == 0 && cost > budget)
        {
            cost -= Math.Min(cost - budget, LifeSymbols(client, view, card));
        }

        // The X it will be announced at: not counting mana that would cost the last of its life (see Paying).
        int maxX = option.HasX ? option.MaxX - LethalMana(client, view) : option.MaxX;

        if (option.HasX)
        {
            if (maxX < 1)
            {
                yield break;
            }

            // A spell with {X} takes all the mana there is: X is made as large as it will go.
            cost = Math.Max(budget, cost);
        }

        if (permanent && card.Supertypes.Contains(SupertypeView.Legendary)
            && client.BattlefieldOf(client.Seat).Any(mine => string.Equals(mine.Name, card.Name, StringComparison.Ordinal)))
        {
            // A second copy of a legend only replaces the first (CR 704.5j).
            yield break;
        }

        double played = permanent
            ? Rock(client, card, text) is > 0 and double rock
                ? Math.Max(option.AbilityIndex == 0 && CardReading.LeavesAtOnce(text) ? 0 : _model.WorthOnceCast(card), rock)
                    + Entering(client, view, text) + Stored(card, text)
                : (option.AbilityIndex == 0 && CardReading.LeavesAtOnce(text) ? 0 : _model.WorthOnceCast(card))
                    + Entering(client, view, text) + Stored(card, text)
            : 0;

        if (permanent && card.CardTypes.Contains(CardTypeView.Planeswalker))
        {
            played += Arriving(client, view, card);
        }

        if (option.TargetsRequired > 0 && option.TargetCandidates.Count > 0)
        {
            EffectRead effect = CardReading.Effect(text);
            if (effect.Amount < 0)
            {
                effect = effect with { Amount = maxX };
            }

            // Animate Dead is cast at the creature card it brings back, and is worth that creature, not an Aura.
            if (AnimatesDead(text))
            {
                effect = new EffectRead(EffectKind.Raise, 1);
                played = 0;
            }

            // Something it cannot read is not aimed at anything: there is no telling whose things it is good for. The
            // one thing it can still be cast for is a card it draws on the way - Gitaxian Probe - and then it is the
            // opponent's hand that is looked at.
            if (effect.Kind == EffectKind.Unknown)
            {
                if (CardReading.Draws(text) is > 0 and int cantrip && !CardReading.Sweeping(text))
                {
                    int rival = Rival(client, view);
                    TargetMessage aim = option.TargetCandidates.Find(target => target.IsPlayer && target.Player == rival) ?? option.TargetCandidates[0];
                    yield return new Candidate(
                        option, (cantrip * _model.Card) + Policy("select") - _model.Card, cost, [aim], Kick: false, Permanent: false, Hostile: false);
                }

                yield break;
            }

            (List<TargetMessage> targets, double worth) = Pick(client, view, effect, option.TargetCandidates, option.TargetsRequired, option.TargetsOptional);

            // "Target player draws three cards", aimed at itself, is worth the cards; an extra turn, a turn.
            bool atItself = targets.Exists(target => target.IsPlayer && target.Player == client.Seat);
            if (CardReading.TargetDraws(text) is > 0 and int drawn && atItself)
            {
                worth = drawn * _model.Card;
            }
            else if (CardReading.ExtraTurn(text) && atItself)
            {
                worth = _model.Card + Policy("extraturn");
            }

            if (worth > 0)
            {
                yield return new Candidate(option, played + worth - _model.Card, cost, targets, Kick: false, permanent, effect.Hostile);
            }

            int kicker = CardReading.KickerCost(text);
            if (kicker > 0 && effect.Kicked > 0 && cost + kicker <= budget)
            {
                (targets, worth) = Pick(client, view, effect with { Amount = effect.Kicked }, option.TargetCandidates, option.TargetsRequired, option.TargetsOptional);
                if (worth > 0)
                {
                    yield return new Candidate(option, played + worth - _model.Card, cost + kicker, targets, Kick: true, permanent, effect.Hostile);
                }
            }

            yield break;
        }

        if (option.TargetsRequired > 0 && !option.TargetsOptional)
        {
            yield break;
        }

        if (permanent)
        {
            // Necromancy cast when a sorcery could not be is sacrificed at that cleanup, with what it brought back.
            if (Cheating && CardReading.Plain(text).Contains("any time a sorcery couldn't have been cast", StringComparison.Ordinal)
                && (view.ActivePlayer != client.Seat || view.Step is not (TurnStepView.PrecombatMain or TurnStepView.PostcombatMain)))
            {
                yield break;
            }

            yield return new Candidate(option, played + Reanimating(client, view, text) - _model.Card, cost, [], Kick: false, Permanent: true, Hostile: false);
            yield break;
        }

        // A modal spell is worth its best mode; the mode and its target are asked for once it is cast.
        IReadOnlyList<string> modes = CardReading.Modes(text);
        // And so is a spell whose targets are only asked for once it is cast: damage divided as its caster chooses.
        EffectRead aimedLater = CardReading.Effect(text);
        bool divided = modes.Count == 0 && !other && aimedLater.Hostile && aimedLater.Kind != EffectKind.Counter
            && CardReading.Plain(text).Contains("target", StringComparison.Ordinal);

        double gain = modes.Count > 0
            ? modes.Max(mode => Mode(client, view, mode))
            : other ? _model.Card + Policy("unread")
            : divided ? Mode(client, view, text)
            : Cheat(client, view, text, card.Handle, budget - cost) ?? Untargeted(client, view, text);

        bool hostile = divided || (modes.Count > 0 && modes.Any(mode => CardReading.Effect(mode).Hostile));
        yield return new Candidate(option, gain - _model.Card, cost, [], Kick: false, Permanent: false, hostile);
    }

    /// <summary>What one activated ability is worth using now, or null for one this seat leaves alone.</summary>
    private Candidate? Activation(GameClient client, PlayerView view, ActionOptionMessage option, ObjectView source)
    {
        // The ability as it reads, cost and all, is the only thing that says which ability this is.
        string described = option.Description;
        int colon = described.IndexOf(':', StringComparison.Ordinal);
        string costText = CardReading.Plain(colon < 0 ? string.Empty : described[..colon]);
        string effectText = (colon < 0 ? described : described[(colon + 1)..]).Trim();
        string plain = CardReading.Plain(effectText);
        bool equip = CardReading.Plain(described).TrimStart().StartsWith("equip", StringComparison.Ordinal);

        if (equip)
        {
            costText = CardReading.Plain(described);
            plain = "attach to target creature you control";
        }

        int cost = CardReading.ManaValue(costText);
        string name = source.Name.ToLowerInvariant();
        bool sacrificesItself = costText.Contains("sacrifice this", StringComparison.Ordinal)
            || (name.Length > 0 && costText.Contains("sacrifice " + name, StringComparison.Ordinal));

        // A cost paid in other cards is a trade this seat cannot price.
        if (option.SacrificeCandidates.Count > 0
            || (costText.Contains("sacrifice", StringComparison.Ordinal) && !sacrificesItself)
            || costText.Contains("discard", StringComparison.Ordinal)
            || option.LifeCost >= Math.Max(LifeOf(view, client.Seat) - 3, 1))
        {
            return null;
        }

        double price = !sacrificesItself || source.Zone != ZoneKindView.Battlefield ? 0
            : IsLand(source) ? (Lands(client, client.Seat) > 5 ? _model.Weight("land.5+") : 4 * _model.Weight("land.1-4"))
            : GivenUp(client, source, effectText);

        // A permanent that costs its controller life for every counter on it, and whose ability adds one (The One
        // Ring): another is only taken while there are that many turns of the life it will cost to spare.
        if (Policy("burden") > 0 && plain.Contains(" counter on ", StringComparison.Ordinal)
            && CardReading.Plain(source.OracleText).Contains("you lose 1 life for each", StringComparison.Ordinal)
            && LifeOf(view, client.Seat) <= (Policy("burden") * (source.Counters.Sum(counter => counter.Count) + 1)) + 2)
        {
            return null;
        }

        if (Policy("loyalty") > 0 && source.CardTypes.Contains(CardTypeView.Planeswalker)
            && CardReading.Loyalty(described) is { } change)
        {
            return Loyalty(client, view, option, source, change, effectText);
        }

        if (option.TargetsRequired > 0 && option.TargetCandidates.Count > 0)
        {
            EffectRead effect = equip ? new EffectRead(EffectKind.Help) : CardReading.Effect(effectText);

            // Something that is given up for a card and has to be aimed on the way, as a Bauble is: at an opponent,
            // whose hand or library it is then that gets looked at.
            if (effect.Kind == EffectKind.Unknown && Policy("stored") > 0 && sacrificesItself
                && CardReading.Draws(effectText) is > 0 and int cashed)
            {
                int rival = Rival(client, view);
                TargetMessage aim = option.TargetCandidates.Find(target => target.IsPlayer && target.Player == rival) ?? option.TargetCandidates[0];
                return new Candidate(option, (cashed * _model.Card) - price, cost, [aim], Kick: false, Permanent: false, Hostile: false);
            }

            // Protection is kept for when something needs protecting (see Respond), and a pump that lasts a turn for
            // a combat this seat does not plan around.
            if (effect.Kind is EffectKind.Protect or EffectKind.Unknown
                || (effect.Kind == EffectKind.Help && plain.Contains("until end of turn", StringComparison.Ordinal)))
            {
                return null;
            }

            (List<TargetMessage> targets, double worth) = Pick(client, view, effect, option.TargetCandidates, option.TargetsRequired, option.TargetsOptional);
            if (targets.Count == 0)
            {
                return null;
            }

            // Equipment stays where it is unless somewhere clearly better has turned up.
            if (equip && source.AttachedTo is { IsNone: false } wearing)
            {
                if (targets[0].Object == wearing
                    || (client.Object(wearing) is { } worn && client.Object(targets[0].Object) is { } wanted
                        && _model.Worth(wanted) < _model.Worth(worn) + 0.3))
                {
                    return null;
                }
            }

            return new Candidate(option, worth - price, cost, targets, Kick: false, Permanent: false, effect.Hostile);
        }

        if (option.TargetsRequired > 0 && !option.TargetsOptional)
        {
            return null;
        }

        // A fetch land: a land for a land, and the colors this seat is short of. Once it has the lands it needs, the
        // life a fetch costs is only worth a thinner library while there is life to spare.
        if (plain.Contains("search your library", StringComparison.Ordinal) && plain.Contains("onto the battlefield", StringComparison.Ordinal))
        {
            bool wanted = Lands(client, client.Seat) < 6 || option.LifeCost == 0 || LifeOf(view, client.Seat) >= 12;
            return view.ActivePlayer == client.Seat && wanted
                ? new Candidate(option, 0.3, cost, [], Kick: false, Permanent: true, Hostile: false)
                : null;
        }

        // Sneak Attack: worth the creature it puts in, and nothing when there is none worth it.
        if (Cheating && FromHand(client, view, effectText, ViewHandle.None, Mana(client) - cost) is { } sneak)
        {
            return new Candidate(option, sneak - price, cost, [], Kick: false, Permanent: false, Hostile: false);
        }

        double gain;
        if (source.CardTypes.Contains(CardTypeView.Planeswalker) && described.TrimStart().StartsWith('+'))
        {
            gain = 0.2;
        }
        else if (plain.Contains("until end of turn", StringComparison.Ordinal) || plain.Contains("add {", StringComparison.Ordinal))
        {
            return null;
        }
        else if (CardReading.Draws(effectText) is > 0 and int draws)
        {
            // Sensei's Divining Top's draw puts the Top back where the card came from: no card at all, and two seats
            // that each cast it, draw it and cast it again every turn never finish the game.
            if (plain.Contains("on top of its owner's library", StringComparison.Ordinal))
            {
                return null;
            }

            gain = draws * _model.Card;
        }
        else if (CardReading.Tokens(effectText) is { Count: > 0 } tokens)
        {
            gain = tokens.Count * Token(tokens.Power, tokens.Toughness);
        }
        else if (plain.Contains("becomes", StringComparison.Ordinal) || plain.Contains("+1/+1 counter on this", StringComparison.Ordinal)
            || (name.Length > 0 && plain.Contains("+1/+1 counter on " + name, StringComparison.Ordinal)))
        {
            // "If this creature is a Spirit, it becomes…" does nothing for one that is not.
            if (CardReading.RequiredType(effectText) is { } type
                && !source.Subtypes.Exists(subtype => string.Equals(subtype, type, StringComparison.OrdinalIgnoreCase)))
            {
                return null;
            }

            // Growing for good. To a stated size it is worth the difference, and nothing to a creature already that
            // big; anything else is tried, and given up on for the turn if the board does not change (Priority).
            if (CardReading.BaseSize(effectText) is { } size)
            {
                int power = size.Power - (source.Power ?? 0);
                int toughness = size.Toughness - (source.Toughness ?? 0);
                if (power <= 0 && toughness <= 0)
                {
                    return null;
                }

                gain = (_model.Weight("creature.power") * power) + (_model.Weight("creature.toughness") * toughness);
            }
            else
            {
                gain = Policy("grow");
            }
        }
        else
        {
            return null;
        }

        return new Candidate(option, gain - price, cost, [], Kick: false, Permanent: false, Hostile: false);
    }

    /// <summary>
    /// What one loyalty ability is worth using now. A planeswalker gives one of these a turn for nothing but its own
    /// loyalty, so each is weighed as what it does plus what the loyalty it adds or spends is worth — and the
    /// planeswalker itself, for an ability that would take its last counter. One that adds loyalty is worth using
    /// whatever it says; one that spends it has to be read to be used.
    /// </summary>
    private Candidate? Loyalty(GameClient client, PlayerView view, ActionOptionMessage option, ObjectView source, int change, string effectText)
    {
        string plain = CardReading.Plain(effectText);
        int now = source.Counters.Find(counter => counter.Name == "loyalty")?.Count ?? 0;

        // A planeswalker that would die of it is lost whole, which is the loyalty it had and the rest of it: that once.
        double step = now + change <= 0 ? -_model.Worth(source) : Policy("loyalty") * change * _model.Weight("permanent.loyalty");

        // What it says it does for its controller whatever it is aimed at: cards, creatures, life.
        double worth = 0;
        bool read = false;

        if (CardReading.Draws(effectText) is > 0 and int draws)
        {
            // Brainstorm's "then put two back" keeps one card of the three, and the choice of which is half another.
            worth += plain.Contains("put two cards from your hand on top", StringComparison.Ordinal)
                ? (Math.Max(draws - 2, 0) + 0.5) * _model.Card
                : draws * _model.Card;
            read = true;
        }
        else if (plain.Contains("into your hand", StringComparison.Ordinal))
        {
            worth += _model.Card;
            read = true;
        }

        if (CardReading.Tokens(effectText) is { Count: > 0 } tokens)
        {
            worth += tokens.Count * Token(tokens.Power, tokens.Toughness);
            read = true;
        }

        if (CardReading.LifeGain(effectText) is > 0 and int life)
        {
            int mine = LifeOf(view, client.Seat);
            worth += _model.Life(mine + life) - _model.Life(mine);
            read = true;
        }

        List<TargetMessage> targets = [];
        bool hostile = false;

        if (option.TargetsRequired > 0 && option.TargetCandidates.Count > 0)
        {
            EffectRead effect = CardReading.Effect(effectText);
            if (effect.Kind == EffectKind.Protect || (effect.Kind == EffectKind.Unknown && change < 0 && !read))
            {
                return null;
            }

            (targets, double aimed) = Pick(client, view, effect, option.TargetCandidates, option.TargetsRequired, option.TargetsOptional);
            if (targets.Count == 0 && !option.TargetsOptional)
            {
                return null;
            }

            worth += aimed;
            read |= effect.Kind != EffectKind.Unknown;
            hostile = effect.Hostile;
        }
        else if (option.TargetsRequired > 0 && !option.TargetsOptional)
        {
            return null;
        }

        if (!read)
        {
            // Unread. Spending loyalty on it is only worth it for the emblem a last ability leaves behind; adding
            // loyalty, or keeping it, is worth it whatever it does — mana apart, which this seat has no plan for.
            if (change < 0)
            {
                if (!plain.Contains("you get an emblem", StringComparison.Ordinal))
                {
                    return null;
                }

                worth += Policy("emblem");
            }
            else
            {
                worth += plain.StartsWith("add {", StringComparison.Ordinal) ? Policy("margin") : 0.2;
            }
        }

        return new Candidate(option, worth + step, 0, targets, Kick: false, Permanent: false, hostile);
    }

    /// <summary>
    /// What a planeswalker in hand is worth beyond what the model sees of it there: the loyalty it will enter with,
    /// which a card in hand does not show (taken as its mana value, which is what most start near), and the best of
    /// its abilities used at once, which is what a planeswalker is cast for. Without it Jace, the Mind Sculptor was
    /// worth less than a three-mana 2/2. Under <c>policy.walkercast</c>.
    /// </summary>
    private double Arriving(GameClient client, PlayerView view, ObjectView card)
    {
        double weight = Policy("walkercast");
        if (weight <= 0)
        {
            return 0;
        }

        int loyalty = Math.Max(card.ManaValue, 1);
        ObjectView entered = new()
        {
            Handle = card.Handle, Zone = ZoneKindView.Battlefield, Kind = card.Kind, Owner = card.Owner, Controller = client.Seat,
            Name = card.Name, CardTypes = card.CardTypes, Subtypes = card.Subtypes, Supertypes = card.Supertypes,
            OracleText = card.OracleText, ManaValue = card.ManaValue, Colors = card.Colors,
            Counters = [new CounterView { Name = "loyalty", Count = loyalty }],
        };

        double first = 0;
        foreach (string line in card.OracleText.Split('\n'))
        {
            // Only those that aim at nothing are weighed here: an aimed one is weighed once there is something to aim at.
            if (CardReading.Loyalty(line) is not { } change || CardReading.Plain(line).Contains("target", StringComparison.Ordinal))
            {
                continue;
            }

            int colon = line.IndexOf(':', StringComparison.Ordinal);
            var use = new ActionOptionMessage { Kind = ActionKindView.ActivateAbility, Source = card.Handle, Description = line };
            if (Loyalty(client, view, use, entered, change, colon < 0 ? line : line[(colon + 1)..].Trim()) is { } candidate)
            {
                first = Math.Max(first, candidate.Gain);
            }
        }

        return weight * ((_model.Weight("permanent.loyalty") * loyalty) + first);
    }

    /// <summary>What a creature token of this size is worth.</summary>
    private double Token(int power, int toughness) =>
        _model.Weight("creature") + (_model.Weight("creature.power") * Math.Max(power, 0))
        + (_model.Weight("creature.toughness") * Math.Max(toughness, 0)) + _model.Weight("creature.token");

    /// <summary>What a spell that aims at nothing is worth, as far as its text can be read.</summary>
    private double Untargeted(GameClient client, PlayerView view, string text)
    {
        string plain = CardReading.Plain(text);
        double worth = 0;
        bool read = false;

        // Mana for nothing is a card for nothing.
        if (CardReading.Ritual(text))
        {
            return 0;
        }

        if (CardReading.ExtraTurn(text))
        {
            worth += _model.Card + Policy("extraturn");
            read = true;
        }

        if (plain.Contains("each player", StringComparison.Ordinal) && plain.Contains("draws seven", StringComparison.Ordinal))
        {
            // A wheel: everyone ends on seven, so it is worth the cards this seat is behind by, and the card it costs.
            int held = Math.Max(client.MyHand().Count - 1, 0);
            int theirs = Summary(view, Rival(client, view))?.HandCount ?? 0;
            return _model.Card * (1 + (0.5 * (theirs - held)));
        }

        if (CardReading.Draws(text) is > 0 and int draws)
        {
            worth += draws * _model.Card;
            read = true;
        }

        if (CardReading.Selects(text))
        {
            // With a draw this is a cantrip, which is what makes one worth the mana: the card comes back and the
            // next ones are chosen.
            worth += Policy("select");
        }

        if (CardReading.Tokens(text) is { Count: > 0 } tokens)
        {
            worth += tokens.Count * Token(tokens.Power, tokens.Toughness);
            read = true;
        }

        if (CardReading.LifeGain(text) is > 0 and int life)
        {
            int mine = LifeOf(view, client.Seat);
            worth += _model.Life(mine + life) - _model.Life(mine);
            read = true;
        }

        if (plain.Contains("destroy all creatures", StringComparison.Ordinal) || plain.Contains("destroy each creature", StringComparison.Ordinal)
            || plain.Contains("exile all creatures", StringComparison.Ordinal) || plain.Contains("all creatures get -", StringComparison.Ordinal)
            || plain.Contains("destroy all artifacts and creatures", StringComparison.Ordinal))
        {
            // A sweeper is worth what it sweeps of theirs less what it sweeps of its own, and is otherwise held.
            bool artifacts = plain.Contains("artifacts and creatures", StringComparison.Ordinal);
            worth += view.Objects
                .Where(card => card.Zone == ZoneKindView.Battlefield
                    && (IsCreature(card) || (artifacts && card.CardTypes.Contains(CardTypeView.Artifact))))
                .Sum(card => (Mine(client, card) ? -1 : 1) * _model.Worth(card));
            read = true;
        }
        else if (CardReading.Effect(text) is { Kind: EffectKind.Damage, Amount: > 0 } damage && plain.Contains("each opponent", StringComparison.Ordinal))
        {
            int rival = Rival(client, view);
            int theirs = LifeOf(view, rival);
            worth += theirs <= damage.Amount ? Combat.Win : _model.Life(theirs) - _model.Life(theirs - damage.Amount);
            read = true;
        }

        // Something unread is cast when there is nothing better to do with the mana: a card in hand does nothing. Not
        // something that does the same to everyone, though: an Upheaval cast for want of anything better loses games.
        return read ? worth : CardReading.Sweeping(text) ? 0 : _model.Card + Policy("unread");
    }

    /// <summary>An instant or a card with flash: what can as well be cast at the end of the opponent's turn.</summary>
    private static bool Instantly(GameClient client, ActionOptionMessage option) =>
        client.Object(option.Source) is { } card
        && (card.CardTypes.Contains(CardTypeView.Instant) || (CardReading.KeywordsOf(card) & Keywords.Flash) != 0);

    /// <summary>
    /// The mana to keep back on its own turn for a counterspell it holds, or zero: only once it has the lands to do
    /// something else as well, and only while the opponent has something to cast.
    /// </summary>
    private int Reserve(GameClient client, PlayerView view, int budget)
    {
        if (Policy("holdcounter") <= 0 || (Summary(view, Rival(client, view))?.HandCount ?? 0) == 0)
        {
            return 0;
        }

        int cheapest = client.MyHand()
            .Where(card => card.CardTypes.Contains(CardTypeView.Instant) && CardReading.Effect(card.OracleText).Kind == EffectKind.Counter)
            .Select(card => Math.Max(card.ManaValue, 1))
            .DefaultIfEmpty(0)
            .Min();

        return cheapest > 0 && budget >= cheapest + 2 ? cheapest : 0;
    }

    /// <summary>
    /// What an artifact that makes mana is worth to have cast: what a land would be, for each mana it makes, which the
    /// model has no way to say of a permanent it weighs by its type and its cost. A Mox is a second land drop, and one
    /// left in hand because an artifact that costs nothing is "worth less than a card" is the turn it would have
    /// bought. One that works only once — Black Lotus, Mana Vault — is a card kept where it can be used, so it is worth
    /// a card and a little.
    /// </summary>
    private double Rock(GameClient client, ObjectView card, string text)
    {
        double rocks = Policy("rocks");
        if (rocks > 0 && !IsCreature(card) && !IsLand(card) && Traded(client, card, text) is { } traded)
        {
            return traded;
        }

        if (rocks <= 0 || IsCreature(card) || IsLand(card) || CardReading.ManaMade(text) is not { Amount: > 0 } made)
        {
            return 0;
        }

        double kept = _model.Card + (2 * Policy("margin"));
        if (made.Once)
        {
            return kept;
        }

        // Mox Amber with no legend to name a colour: it costs nothing, so it is still no worse on the table, but it is
        // no land until there is one.
        if (Idle(client, card))
        {
            return card.ManaValue == 0 ? kept : 0;
        }

        int sources = client.BattlefieldOf(client.Seat).Count(source => source.ProducesMana && !IsCreature(source));
        double worth = rocks * made.Amount * (sources <= 4 ? _model.Weight("land.1-4") : _model.Weight("land.5+"));

        // One that costs nothing is never worse on the table than in hand.
        return card.ManaValue == 0 ? Math.Max(worth, kept) : worth;
    }

    /// <summary>
    /// A Mox paid for with a card (Chrome Mox's imprint, Mox Diamond's land): worth the card it costs and a mana for
    /// good while the seat is still building its mana (four sources or fewer) and has the card to spare - a second
    /// spell beside the one imprinted, a second land beside the one to play. Not cast otherwise: two cards for one
    /// mana late in a game is a card thrown away. Null for text that is no such Mox; under <c>policy.imprint</c>.
    /// </summary>
    private double? Traded(GameClient client, ObjectView card, string text)
    {
        string plain = CardReading.Plain(text);
        bool imprint = plain.Contains("exile a nonartifact, nonland card from your hand", StringComparison.Ordinal);
        bool diamond = plain.Contains("you may discard a land card instead", StringComparison.Ordinal);
        if (Policy("imprint") <= 0 || !(imprint || diamond))
        {
            return null;
        }

        int sources = client.BattlefieldOf(client.Seat).Count(source => source.ProducesMana && (!IsCreature(source) || !source.SummoningSick));
        List<ObjectView> hand = [.. client.MyHand().Where(other => other.Handle != card.Handle)];
        bool spare = imprint
            ? hand.Count(other => !IsLand(other) && !other.CardTypes.Contains(CardTypeView.Artifact)) >= 2
            : hand.Count(IsLand) >= 2;

        return sources <= 4 && spare ? Policy("imprint") * ((2 * _model.Card) + (2 * Policy("margin"))) : 0;
    }

    /// <summary>
    /// What a permanent that is sacrificed for its own ability was worth where it was. A creature, what the model
    /// says. An artifact that makes mana, what that mana is still worth to a seat with the lands it has: Mind Stone is
    /// a land early and a card late. Anything else that offers to be sacrificed — a Spellbomb, a Lantern — was on the
    /// battlefield for nothing else, and by the model's guess at an artifact (its type and its cost) would have been
    /// worth more kept than the card it gives, so it never was given.
    /// </summary>
    private double GivenUp(GameClient client, ObjectView source, string effectText = "")
    {
        if (Policy("stored") <= 0 || IsCreature(source) || source.CardTypes.Contains(CardTypeView.Planeswalker))
        {
            return _model.Worth(source);
        }

        // Graveyard hate is not cashed in for a card while an opponent's graveyard holds a creature worth bringing back.
        if (CardReading.Draws(effectText) > 0 && Hate(source.OracleText) && FattestDead(client) >= Reanimated)
        {
            return Policy("graveyard") * 2 * _model.Card;
        }

        if (source.ProducesMana && CardReading.ManaMade(source.OracleText) is { Amount: > 0, Once: false } made)
        {
            // Counted as Rock counts them: the others, as they were before this one was cast.
            int sources = client.BattlefieldOf(client.Seat).Count(other => other.ProducesMana && !IsCreature(other) && other.Handle != source.Handle);
            return made.Amount * (sources <= 4 ? _model.Weight("land.1-4") : _model.Weight("land.5+"));
        }

        return Math.Min(_model.Worth(source), 2 * Policy("margin"));
    }

    [System.Text.RegularExpressions.GeneratedRegex(@"enters with (a|an|one|two|three|four|five|\d+) charge counters?")]
    private static partial System.Text.RegularExpressions.Regex ChargeClause();

    /// <summary>
    /// What a permanent holds in store: the cards it says it can be cashed in for, as Mind Stone, a Spellbomb and a
    /// Bauble each can. The model weighs an artifact by its cost, and by that an artifact that costs nothing and
    /// replaces itself is worth less than the card it is — so it was never cast.
    /// </summary>
    private double Stored(ObjectView card, string text)
    {
        double stored = Policy("stored");
        if (stored <= 0 || IsCreature(card))
        {
            return 0;
        }

        double most = 0;
        string plain = CardReading.Plain(text);
        int charges = ChargeClause().Match(plain) is { Success: true } enters ? CardReading.Count(enters.Groups[1].Value) : 0;
        foreach (string line in plain.Split('\n'))
        {
            int colon = line.IndexOf(':', StringComparison.Ordinal);
            if (colon > 0 && line[..colon].Contains("sacrifice", StringComparison.Ordinal))
            {
                most = Math.Max(most, CardReading.Draws(line[(colon + 1)..]));
            }

            // Reckoner Bankbuster: a card for each charge counter it enters with, one a turn, at two mana each, so
            // half of each.
            if (colon > 0 && charges > 0 && line[..colon].Contains("remove a charge counter", StringComparison.Ordinal))
            {
                most = Math.Max(most, charges * CardReading.Draws(line[(colon + 1)..]) * 0.5);
            }
        }

        return stored * most * _model.Card;
    }

    /// <summary>What a creature does as it enters is worth, when that is aimed at something this seat can find.</summary>
    private double Entering(GameClient client, PlayerView view, string text)
    {
        double worth = 0;

        foreach (string line in text.Split('\n'))
        {
            string plain = CardReading.Plain(line);
            if (!plain.Contains(" enters", StringComparison.Ordinal) || plain.Contains("another", StringComparison.Ordinal)
                || !(plain.StartsWith("when ", StringComparison.Ordinal) || plain.StartsWith("whenever ", StringComparison.Ordinal)))
            {
                continue;
            }

            if (CardReading.Effect(line) is { Hostile: true })
            {
                worth += Math.Max(Mode(client, view, line), 0);
            }

            if (CardReading.LifeGain(line) is > 0 and int life)
            {
                int mine = LifeOf(view, client.Seat);
                worth += _model.Life(mine + life) - _model.Life(mine);
            }

            if (CardReading.Draws(line) is > 0 and int draws && !plain.Contains("discard", StringComparison.Ordinal))
            {
                worth += draws * _model.Card;
            }
        }

        return worth;
    }

    /// <summary>
    /// What one piece of rules text is worth with the best target the board has for it: one mode of a modal spell, or
    /// a trigger not yet on the stack. The candidates are this seat's guess from the words — the server offers the
    /// real ones when the time comes.
    /// </summary>
    private double Mode(GameClient client, PlayerView view, string text)
    {
        EffectRead effect = CardReading.Effect(text);
        if (effect.Kind == EffectKind.Unknown)
        {
            return Untargeted(client, view, text) - Policy("unread");
        }

        (_, double worth) = Pick(client, view, effect, Guess(client, view, text), 1, optional: true);
        return worth;
    }

    /// <summary>What a piece of text could be aimed at, guessed from its words.</summary>
    private static List<TargetMessage> Guess(GameClient client, PlayerView view, string text)
    {
        string plain = CardReading.Plain(text);
        bool any = plain.Contains("any target", StringComparison.Ordinal);
        bool permanents = plain.Contains("target permanent", StringComparison.Ordinal) || plain.Contains("target nonland permanent", StringComparison.Ordinal);
        bool own = plain.Contains("you control", StringComparison.Ordinal);
        bool theirs = plain.Contains("you don't control", StringComparison.Ordinal) || plain.Contains("an opponent controls", StringComparison.Ordinal);

        List<TargetMessage> guess = [];

        if (any || plain.Contains("target player", StringComparison.Ordinal) || plain.Contains("target opponent", StringComparison.Ordinal)
            || plain.Contains("each opponent", StringComparison.Ordinal))
        {
            guess.AddRange(view.Players.Where(player => player.Status == PlayerStatusView.Active)
                .Select(player => new TargetMessage { Player = player.Seat }));
        }

        foreach (ObjectView card in view.Objects)
        {
            if (card.Zone == ZoneKindView.Stack && plain.Contains("target spell", StringComparison.Ordinal))
            {
                guess.Add(new TargetMessage { Object = card.Handle });
                continue;
            }

            if (card.Zone != ZoneKindView.Battlefield || (own && !Mine(client, card)) || (theirs && Mine(client, card)))
            {
                continue;
            }

            bool fits = (IsCreature(card) && (any || plain.Contains("target creature", StringComparison.Ordinal)
                    || plain.Contains("target attacking", StringComparison.Ordinal) || plain.Contains("target blocking", StringComparison.Ordinal)))
                || (card.CardTypes.Contains(CardTypeView.Planeswalker) && (any || plain.Contains("planeswalker", StringComparison.Ordinal)))
                || (card.CardTypes.Contains(CardTypeView.Artifact) && plain.Contains("target artifact", StringComparison.Ordinal))
                || (card.CardTypes.Contains(CardTypeView.Enchantment) && plain.Contains("enchantment", StringComparison.Ordinal) && plain.Contains("target", StringComparison.Ordinal))
                || (IsLand(card) && plain.Contains("target land", StringComparison.Ordinal))
                || (permanents && (!IsLand(card) || !plain.Contains("nonland", StringComparison.Ordinal)));

            if (fits)
            {
                guess.Add(new TargetMessage { Object = card.Handle });
            }
        }

        return guess;
    }

    // --- Aiming ------------------------------------------------------------------------------------------------------

    /// <summary>
    /// The best things to aim an effect at among what is offered, and what aiming it there is worth. Where aiming at
    /// nothing is allowed, nothing is aimed at rather than at something it would be bad to hit.
    /// </summary>
    private (List<TargetMessage> Targets, double Worth) Pick(
        GameClient client, PlayerView view, EffectRead effect, IReadOnlyList<TargetMessage> candidates, int count, bool optional)
    {
        List<(TargetMessage Target, double Worth)> scored =
            [.. candidates.Select(candidate => (candidate, Aimed(client, view, effect, candidate))).OrderByDescending(pair => pair.Item2)];

        List<TargetMessage> targets = [];
        double worth = 0;

        foreach ((TargetMessage target, double value) in scored)
        {
            if (targets.Count >= Math.Max(count, 1) || (value <= 0 && optional))
            {
                break;
            }

            targets.Add(target);
            worth += value;
        }

        return (targets, worth);
    }

    /// <summary>What it is worth to this seat to do this to that.</summary>
    private double Aimed(GameClient client, PlayerView view, EffectRead effect, TargetMessage target)
    {
        // An X nobody has said the size of is taken as nothing, rather than as a negative amount.
        if (effect.Amount < 0)
        {
            effect = effect with { Amount = 0 };
        }

        if (target.IsPlayer)
        {
            bool me = target.Player == client.Seat || Summary(view, target.Player)?.Team == Summary(view, client.Seat)?.Team;
            int life = LifeOf(view, target.Player);

            double worth = effect.Kind switch
            {
                EffectKind.Damage => effect.Amount > 0 && life <= effect.Amount
                    ? Combat.Win
                    : _model.Life(life) - _model.Life(life - effect.Amount),
                EffectKind.Help or EffectKind.Protect => -Policy("help"),
                EffectKind.Unknown => Unread,

                // Cards discarded are cards: as many as it says, of as many as that player holds.
                EffectKind.Harm when effect.Amount > 0 && Policy("discard") > 0 =>
                    Policy("discard") * _model.Card * Math.Min(effect.Amount, Summary(view, target.Player)?.HandCount ?? 0),
                _ => Policy("harm"),
            };

            return me ? -worth : worth;
        }

        if (client.Object(target.Object) is not { } card)
        {
            return 0;
        }

        if (card.Zone == ZoneKindView.Stack)
        {
            double spell = IsPermanentCard(card) ? _model.WorthOnceCast(card) : 2 * _model.Card;
            return effect.Kind == EffectKind.Counter ? (Mine(client, card) ? -spell : spell) : 0;
        }

        if (card.Zone == ZoneKindView.Graveyard && effect.Kind == EffectKind.Raise)
        {
            // Whoever's graveyard it is in, it comes back to this seat: onto the battlefield for what it is worth
            // there, or to hand for a card.
            return effect.Amount > 0 && IsPermanentCard(card) ? _model.WorthOnceCast(card) + _model.Card : 2 * _model.Card;
        }

        // A card exiled from a graveyard (Soul-Guide Lantern as it enters, Ghost Vacuum): from an opponent's, a
        // creature that could be brought back before anything else; from this seat's own, as little as anything.
        if (card.Zone == ZoneKindView.Graveyard && effect.Kind == EffectKind.Remove && Policy("graveyard") > 0)
        {
            double gone = Policy("graveyard") * (0.01 + (IsCreature(card) ? 0.02 * card.ManaValue : 0));
            return card.Owner == client.Seat ? -gone : gone;
        }

        if (card.Zone != ZoneKindView.Battlefield)
        {
            return 0;
        }

        double side = Mine(client, card) ? -1 : 1;
        double value = Worth(client, card);
        Keywords keys = CardReading.KeywordsOf(card);
        int left = (card.Toughness ?? 0) - card.DamageMarked;
        bool creature = IsCreature(card);
        bool sturdy = (keys & Keywords.Indestructible) != 0;

        switch (effect.Kind)
        {
            case EffectKind.Damage:
                if (creature)
                {
                    // Damage that does not kill is gone by the end of the turn.
                    return effect.Amount >= left && !sturdy ? side * value : side * 0.01 * effect.Amount;
                }

                if (card.CardTypes.Contains(CardTypeView.Planeswalker))
                {
                    int loyalty = Math.Max(card.Counters.Find(counter => counter.Name == "loyalty")?.Count ?? 1, 1);
                    return side * value * Math.Min(1.0, effect.Amount / (double)loyalty);
                }

                return 0;

            case EffectKind.Shrink:
                return creature && effect.Amount >= left ? side * value : side * 0.05;

            case EffectKind.Remove:
                double removed = side * value;
                if (effect.OwnerGainsPower && creature)
                {
                    int life = LifeOf(view, card.Controller);
                    removed -= side * (_model.Life(life + (card.Power ?? 0)) - _model.Life(life));
                }

                return removed;

            case EffectKind.Steal:
                return side * 2 * value;

            case EffectKind.Bounce:
                return side * value * (card.Kind == ObjectKindView.Token ? 1 : Policy("bounce"));

            case EffectKind.Help or EffectKind.Protect:
                // Its own best creature, and one that can still use the help this turn before one that cannot.
                return -side * (Policy("help") + (0.1 * value) + (card.Tapped || card.SummoningSick ? 0 : 0.02));

            case EffectKind.Counter or EffectKind.Raise:
                return 0;

            case EffectKind.Reshape when Policy("loyalty") > 0:
                // What it would be afterwards, that size and nothing else, against what it is now: an opponent's
                // dragon made an Elk is most of a removal spell, this seat's own Food made one is a creature for
                // nothing, and an opponent's 2/1 made one is a gift. Its own mana is not given up for a body.
                double reshaped = _model.Weight("creature") + (_model.Weight("creature.power") * effect.Amount)
                    + (_model.Weight("creature.toughness") * effect.Toughness);
                // Of its own, only a token or a thing that does nothing, and never what makes its mana (a dork, a Mox):
                // what a card says is more than the model's guess at it, and a 3/3 is not worth a mana a turn.
                if (Mine(client, card) && (card.ProducesMana || (card.Kind != ObjectKindView.Token && CardReading.Plain(card.OracleText).Trim().Length > 0)))
                {
                    return -Math.Max(value, 0) - 1;
                }

                return side * (value - reshaped);

            case EffectKind.Unknown:
                return side * Unread;

            default:
                return side * value * Policy("harm");
        }
    }

    /// <summary>
    /// What aiming something unread at an opponent's thing is taken to be worth: next to nothing, and only so that
    /// when it has to be aimed somewhere it is not aimed at this seat's own.
    /// </summary>
    private const double Unread = 0.001;

    // --- Answering what an opponent is doing -------------------------------------------------------------------------

    /// <summary>
    /// With an opponent's spell or ability on top of the stack: counter it if it is worth a counterspell, protect what
    /// it is aimed at if there is a way to, and otherwise let it happen.
    /// </summary>
    private AnswerPayload Respond(GameClient client, PlayerView view, PriorityPromptMessage prompt, ObjectView top)
    {
        ObjectView? threatened = top.Targets
            .Where(target => target.StillLegal && !target.IsPlayer)
            .Select(target => client.Object(target.Object))
            .FirstOrDefault(card => card is not null && Mine(client, card) && card.Zone == ZoneKindView.Battlefield && Threatens(top, card));

        if (Unraise(client, view, prompt, top) is { } unraised)
        {
            return unraised;
        }

        foreach (ActionOptionMessage option in prompt.Options)
        {
            // Why a counterspell in hand was not cast is as much worth reading afterwards as why one was.
            if (Narrating && !option.Affordable && option.Kind == ActionKindView.CastSpell && top.Kind != ObjectKindView.AbilityOnStack
                && client.Object(option.Source) is { } unpaid && CardReading.Effect(unpaid.OracleText).Kind == EffectKind.Counter)
            {
                Note($"{unpaid.Name} in hand against {top.Name}: not enough mana for it");
            }

            if (!option.Affordable
                || option.Kind is not (ActionKindView.CastSpell or ActionKindView.ActivateAbility)
                || option.Produces.Count > 0 || option.ProducesOfAnyOneColor > 0 || option.ProducesOfAnyColorEach > 0
                || option.SacrificeCandidates.Count > 0
                || _spent.Contains((option.Source.Value, option.Kind, option.AbilityIndex))
                || client.Object(option.Source) is not { } card)
            {
                continue;
            }

            // The same answer to the same thing is given once: one that was given up while paying, or that left its
            // target as threatened as before, would otherwise be given until the mana ran out.
            (int, int, int) answer = (top.Handle.Value, option.Source.Value, option.AbilityIndex);
            if (_answered.Contains(answer))
            {
                continue;
            }

            string text = option.Kind == ActionKindView.CastSpell ? card.OracleText : option.Description;
            EffectRead effect = CardReading.Effect(text);

            // A counterspell paid for with another card (Force of Will) is two cards for one, and is kept for a spell
            // worth that.
            bool pitched = option.Kind == ActionKindView.CastSpell && option.AbilityIndex > 0 && CardReading.ManaValue(option.Description) == 0;

            if (effect.Kind == EffectKind.Counter)
            {
                if (option.TargetCandidates.Find(target => target.Object == top.Handle) is not { } spell)
                {
                    if (Narrating)
                    {
                        Note($"{card.Name} cannot counter {top.Name}");
                    }
                }
                else
                {
                    double worth = Aimed(client, view, effect, spell);
                    double least = pitched ? 2.5 * _model.Card : _model.Card + Policy("margin");
                    double held = CounterBar(view, top);
                    string Bar() => $"{Num(least + held)}: {(pitched ? "two and a half cards, pitching one" : "a card and the margin")}"
                        + (held > 0 ? $", {Num(held)} more for the {Summary(view, top.Controller)?.HandCount ?? 0} cards they hold" : string.Empty);

                    if (worth >= least + held)
                    {
                        if (Narrating)
                        {
                            Note($"{card.Name} on {top.Name}: worth {Num(worth)}, at least the bar {Bar()}");
                        }

                        _answered.Add(answer);
                        return Take(view, new Candidate(option, 1, 0, [spell], Kick: false, Permanent: false, Hostile: true));
                    }

                    if (Narrating)
                    {
                        Note($"{card.Name} kept: {top.Name} is worth {Num(worth)} to stop, under the bar {Bar()}");
                    }
                }
            }

            if (effect.Kind == EffectKind.Protect && threatened is not null
                && option.TargetCandidates.Find(target => target.Object == threatened.Handle) is { } saved
                && (option.Kind == ActionKindView.ActivateAbility || _model.Worth(threatened) > _model.Card))
            {
                _threat = [.. top.Colors];
                _answered.Add(answer);
                return Take(view, new Candidate(option, 1, 0, [saved], Kick: false, Permanent: false, Hostile: false));
            }
        }

        return new PassAnswer();
    }

    /// <summary>The mana value from which a creature card in a graveyard is taken for a reanimation target.</summary>
    private const int Reanimated = 6;

    /// <summary>
    /// Whether a permanent's text has an ability that exiles from graveyards: Soul-Guide Lantern, Ghost Vacuum,
    /// Keen-Eyed Curator, Lion Sash. Under <c>policy.graveyard</c>.
    /// </summary>
    private bool Hate(string text) => Policy("graveyard") > 0 && CardReading.Plain(text).Split('\n').Any(HateLine);

    private static bool HateLine(string line)
    {
        int colon = line.IndexOf(':', StringComparison.Ordinal);
        string effect = colon < 0 ? line : line[(colon + 1)..];
        return effect.Contains("exile each opponent's graveyard", StringComparison.Ordinal)
            || effect.Contains("exile all graveyards", StringComparison.Ordinal)
            || effect.Contains("from target player's graveyard", StringComparison.Ordinal)
            || effect.Contains("exile target card from a graveyard", StringComparison.Ordinal);
    }

    /// <summary>The highest mana value of a creature card in an opponent's graveyard, or 0.</summary>
    private static int FattestDead(GameClient client) => client.View is not { } view ? 0
        : view.Players.Where(player => player.Team != view.Players.Find(me => me.Seat == client.Seat)?.Team)
            .SelectMany(player => client.Zone(ZoneKindView.Graveyard, player.Seat))
            .Where(IsCreature).Select(card => card.ManaValue).DefaultIfEmpty(0).Max();

    /// <summary>
    /// An opponent bringing a creature back from a graveyard - Reanimate, Animate Dead, Necromancy aimed at it, or
    /// Exhume for the best of theirs - answered by exiling it first with graveyard hate this seat has out, when what
    /// comes back is worth more than the hate is. Null when there is nothing to answer or nothing to answer it with.
    /// </summary>
    private AnswerPayload? Unraise(GameClient client, PlayerView view, PriorityPromptMessage prompt, ObjectView top)
    {
        if (Policy("graveyard") <= 0)
        {
            return null;
        }

        ObjectView? raised = top.Targets
            .Where(target => target.StillLegal && !target.IsPlayer)
            .Select(target => client.Object(target.Object))
            .FirstOrDefault(card => card is not null && card.Zone == ZoneKindView.Graveyard && IsCreature(card)
                && CardReading.Plain(top.OracleText) is var raising
                && (raising.Contains("enchant creature card in a graveyard", StringComparison.Ordinal)
                    || CardReading.Effect(top.OracleText) is { Kind: EffectKind.Raise, Amount: > 0 }));

        // Exhume aims at nothing: each player brings back one, and theirs is taken to be the best they have.
        bool exhume = raised is null && CardReading.Plain(top.OracleText)
            .Contains("each player puts a creature card from their graveyard onto the battlefield", StringComparison.Ordinal);
        if (exhume)
        {
            raised = client.Zone(ZoneKindView.Graveyard, top.Controller).Where(IsCreature).MaxBy(_model.WorthOnceCast);
        }

        if (raised is null || (raised.Owner == client.Seat && exhume))
        {
            return null;
        }

        double worth = _model.WorthOnceCast(raised);

        foreach (ActionOptionMessage option in prompt.Options)
        {
            if (!option.Affordable || option.Kind != ActionKindView.ActivateAbility
                || option.SacrificeCandidates.Count > 0 || client.Object(option.Source) is not { } source
                || source.Zone != ZoneKindView.Battlefield)
            {
                continue;
            }

            string plain = CardReading.Plain(option.Description);
            if (!HateLine(plain))
            {
                continue;
            }

            // Whose graveyard it empties: the card itself, the player it belongs to, or every opponent's.
            List<TargetMessage> aim = [];
            if (option.TargetsRequired > 0)
            {
                TargetMessage? at = option.TargetCandidates.Find(target => !target.IsPlayer && target.Object == raised.Handle)
                    ?? option.TargetCandidates.Find(target => target.IsPlayer && target.Player == raised.Owner);
                if (at is null)
                {
                    continue;
                }

                aim.Add(at);
            }
            else if (plain.Contains("each opponent's graveyard", StringComparison.Ordinal) && raised.Owner == client.Seat)
            {
                continue;
            }

            int colon = plain.IndexOf(':', StringComparison.Ordinal);
            bool sacrificed = colon > 0 && plain[..colon].Contains("sacrifice", StringComparison.Ordinal);
            double cost = sacrificed ? Math.Min(_model.Worth(source), 2 * Policy("margin")) : 0;

            if (Narrating)
            {
                Note($"{raised.Name} coming back with {top.Name}: worth {Num(worth)}; {source.Name} exiles it for {Num(cost)}");
            }

            // Once for one spell, as any answer: an exile that changed nothing is not given again.
            if (worth - cost > _model.Card && _answered.Add((top.Handle.Value, option.Source.Value, option.AbilityIndex)))
            {
                return Take(view, new Candidate(option, worth - cost, 0, aim, Kick: false, Permanent: false, Hostile: true));
            }
        }

        return null;
    }

    /// <summary>
    /// How much more than a card a spell has to be worth to be countered, while the opponent still holds cards that
    /// could be worse: <c>policy.counterbar</c> cards with three or more in their hand, less with fewer, nothing with
    /// none. A counterspell spent on the first two-drop is not there for the bomb.
    /// </summary>
    private double CounterBar(PlayerView view, ObjectView top)
    {
        double bar = Policy("counterbar");
        if (bar <= 0)
        {
            return 0;
        }

        int held = Summary(view, top.Controller)?.HandCount ?? 0;
        return bar * _model.Card * Math.Min(held, 3) / 3.0;
    }

    /// <summary>Whether something on the stack, aimed at this creature, would cost this seat the creature.</summary>
    private static bool Threatens(ObjectView spell, ObjectView creature)
    {
        EffectRead effect = CardReading.Effect(spell.OracleText);

        return effect.Kind switch
        {
            EffectKind.Damage or EffectKind.Shrink => effect.Amount < 0 || effect.Amount >= (creature.Toughness ?? 0) - creature.DamageMarked,
            EffectKind.Remove or EffectKind.Bounce or EffectKind.Steal => true,
            _ => false,
        };
    }

    // --- Paying ------------------------------------------------------------------------------------------------------

    /// <summary>
    /// Pays for what was announced. Which land covers what is <see cref="AutoPlay"/>'s to work out — there is nothing
    /// to choose between two that both do — but whether to kick was decided with the target, when the spell was
    /// chosen, and that is what is announced.
    /// <para>
    /// Never with the last of its life: a land that costs a life to tap, or a Talisman that deals a damage, is not
    /// tapped by a seat that would be dead of it, and the spell is given up instead (CR 733.1).
    /// </para>
    /// </summary>
    private TakeActionAnswer Pay(GameClient client, PayCostsPromptMessage prompt)
    {
        TakeActionAnswer answer = Paying(client, prompt);

        if (answer.Kind == ActionKindView.ActivateAbility
            && prompt.Options.Find(option => option.Kind == ActionKindView.ActivateAbility
                && option.Source == answer.Source && option.AbilityIndex == answer.AbilityIndex) is { } source
            && Lethal(client, source))
        {
            // With nothing owed, that source was only raising X: the spell is cast as it stands. Owing, X is lowered
            // to what the safe mana pays for, and only failing both is the spell given up.
            ActionOptionMessage? instead = prompt.Options.Find(option => option.Kind == ActionKindView.ConfirmCasting)
                ?? (prompt.HasX ? prompt.Options.Find(option => option.Kind == ActionKindView.DecreaseX) : null)
                ?? prompt.Options.Find(option => option.Kind == ActionKindView.CancelCasting);

            if (instead is not null)
            {
                return AutoPlay.Answer(instead);
            }
        }

        return answer;
    }

    /// <summary>Whether using this mana ability would cost this seat the last of its life (CR 119.3, 704.5a).</summary>
    private bool Lethal(GameClient client, ActionOptionMessage source) =>
        Policy("lastlife") > 0 && client.View is { } view && source.LifeCost + source.DamageToController >= LifeOf(view, client.Seat);

    /// <summary>
    /// How much of the mana this seat could make would cost it the last of its life to make: untapped sources every
    /// one of whose mana abilities costs life or deals damage to it (Mana Confluence, City of Brass), at a life total
    /// that one use would end. What an X is announced at leaves these out, since the payment will not use them.
    /// </summary>
    private int LethalMana(GameClient client, PlayerView view)
    {
        if (Policy("lastlife") <= 0)
        {
            return 0;
        }

        int life = LifeOf(view, client.Seat);
        int lethal = 0;
        List<(int Pain, int Made)> painful = [];

        foreach (ObjectView source in client.BattlefieldOf(client.Seat))
        {
            if (!source.ProducesMana || source.Tapped || (IsCreature(source) && source.SummoningSick))
            {
                continue;
            }

            string plain = CardReading.Plain(source.OracleText);
            List<string> lines = [.. plain.Split('\n').Where(line => line.Contains(": add ", StringComparison.Ordinal))];
            bool tappedHurts = plain.Contains("becomes tapped, it deals", StringComparison.Ordinal);
            int Pain(string line)
            {
                System.Text.RegularExpressions.Match hurt = System.Text.RegularExpressions.Regex.Match(
                    line, @"pay (\d+) life|deals (\d+) damage to you");
                return hurt.Success ? int.Parse(hurt.Groups[1].Success ? hurt.Groups[1].Value : hurt.Groups[2].Value, System.Globalization.CultureInfo.InvariantCulture)
                    : tappedHurts ? 1 : 0;
            }

            if (lines.Count > 0 && lines.TrueForAll(line => Pain(line) > 0))
            {
                painful.Add((lines.Min(Pain), Math.Max(CardReading.ManaMade(source.OracleText).Amount, 1)));
            }
        }

        // The painful sources the seat can live through using, least painful first; the rest would be the last of it.
        int spent = 0;
        foreach ((int pain, int made) in painful.OrderBy(source => source.Pain))
        {
            if (spent + pain < life)
            {
                spent += pain;
            }
            else
            {
                lethal += made;
            }
        }

        return lethal;
    }

    private TakeActionAnswer Paying(GameClient client, PayCostsPromptMessage prompt)
    {
        if (prompt.AnnouncesXOrKicks && prompt.Kicker.Length > 0 && !prompt.VariableKicker)
        {
            return AutoPlay.Announce(prompt, _kick ? Math.Clamp(prompt.MostAffordable, 0, 1) : 0);
        }

        // X is announced before anything is paid (CR 601.2b), at what the mana reaches: not counting the mana that
        // would cost the last of its life, which the payment will not take.
        if (prompt.AnnouncesXOrKicks && prompt.Kicker.Length == 0 && !prompt.XPaidInLife && client.View is { } announcing
            && LethalMana(client, announcing) is > 0 and int lethal)
        {
            // Nothing left to announce for a spell that was worth casting for its X: it is given up, the card kept.
            int safe = AutoPlay.AnnouncedValue(prompt) - lethal;
            if (safe < 1 && prompt.HasX && prompt.Options.Find(option => option.Kind == ActionKindView.CancelCasting) is { } stop)
            {
                return AutoPlay.Answer(stop);
            }

            return AutoPlay.Announce(prompt, Math.Max(safe, 0));
        }

        // Generic mana, with nothing else owed and nothing to choose but which source: the one that leaves the most
        // behind. A Plains pays {1} as well as a Plateau does, and the Plateau is what the next red spell needs.
        bool onlyGeneric = !prompt.HasX && prompt.RemainingGeneric > 0 && prompt.RemainingColored.Count == 0
            && prompt.RemainingHybrid.Count == 0 && prompt.RemainingColorless == 0
            && prompt.Options.TrueForAll(option =>
                option.Kind is ActionKindView.ActivateAbility or ActionKindView.CancelCasting or ActionKindView.UndoManaAbility);

        if (onlyGeneric)
        {
            List<ColorView> wanted = [.. client.MyHand()
                .Where(card => card.Handle != prompt.Spell && !IsLand(card))
                .SelectMany(card => card.Colors)];

            List<ColorView> Sourced(ViewHandle source) => [.. prompt.Options
                .Where(option => option.Kind == ActionKindView.ActivateAbility && option.Source == source)
                .SelectMany(option => option.Produces
                    .Where(type => type != ManaTypeView.Colorless)
                    .Select(type => (ColorView)(int)type)
                    .Concat(option.ColorChoices))
                .Distinct()];

            ActionOptionMessage? cheapest = prompt.Options
                .Where(option => option.Kind == ActionKindView.ActivateAbility
                    && option.ProducesOfAnyOneColor == 0 && option.ProducesOfAnyColorEach == 0
                    && option.Produces.Count == 1 && option.SacrificeCandidates.Count == 0)
                .OrderBy(option => option.LifeCost + option.DamageToController)
                .ThenBy(option => Sourced(option.Source).Count)
                .ThenBy(option => Sourced(option.Source).Sum(color => wanted.Count(want => want == color)))
                .FirstOrDefault();

            if (cheapest is not null)
            {
                return AutoPlay.Answer(cheapest);
            }
        }

        return AutoPlay.Pay(prompt);
    }

    /// <summary>
    /// Which Phyrexian symbols (CR 107.4f) to announce as paid with life (CR 601.2b): the fewest the server advises for
    /// the mana to cover the rest, paid while the life left stays at or above the floor <c>policy.phyrexian</c> sets,
    /// and none when that floor is off or would be crossed — the mana then pays, or the spell is given up.
    /// </summary>
    private ModesChosenAnswer LifeForPhyrexian(GameClient client, PlayerView view, ChooseModesPromptMessage prompt)
    {
        double floor = Policy("phyrexian");
        int life = LifeOf(view, client.Seat);
        int paid = prompt.Affordable.Sum(mode => LifeIn(prompt.Modes[mode]));
        return floor > 0 && prompt.Affordable.Count <= Math.Max(prompt.AtMost, prompt.Count) && life - paid >= floor
            ? new ModesChosenAnswer { Modes = [.. prompt.Affordable] }
            : new ModesChosenAnswer { Modes = [] };
    }

    /// <summary>The life a "Pay 2 life for {B/P}" choice costs, or zero for one that reads otherwise.</summary>
    private static int LifeIn(string mode)
    {
        System.Text.RegularExpressions.Match life = System.Text.RegularExpressions.Regex.Match(mode, @"^Pay (\d+) life for \{[^}]*/P\}$");
        return life.Success ? int.Parse(life.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture) : 0;
    }

    /// <summary>
    /// Whether to pay an optional cost (CR 118.12): to keep a spell from being countered when it can, and for a copy
    /// of Chain Lightning when there is something for the copy to hit.
    /// </summary>
    private TakeActionAnswer Unless(GameClient client, UnlessPaysPromptMessage prompt)
    {
        // Paying has to get somewhere. A source that itself costs mana to use - Chromatic Star with every land tapped -
        // is activated, cannot be paid for, and leaves the same question with the same amount owed: asked that way
        // twice, the cost is one this seat cannot pay, and it says no.
        if (prompt.Source == _unless && prompt.Remaining == _unlessOwed)
        {
            if (++_unlessStalls >= 2)
            {
                _paying = false;
            }
        }
        else
        {
            _unlessStalls = 0;
        }

        _unlessOwed = prompt.Remaining;

        if (prompt.Source != _unless)
        {
            _unless = prompt.Source;

            // Mana Vault's {4} in the upkeep, when nothing can be cast, buys back three mana: a mana lost every turn
            // to save a point of life. Not with rocks valued as the mana they make.
            _paying = (prompt.Verb != "UntapSource" || Policy("rocks") <= 0)
                && (prompt.Verb.Length == 0 || client.View is not { } view || prompt.Verb != "CopySpell"
                || (_copies < CopiesPerTurn
                    && client.Object(prompt.Subject) is { } copied
                    && Mode(client, view, copied.OracleText.Split('.')[0]) > Policy("margin")
                    && ++_copies > 0));
        }

        if (!_paying)
        {
            return AutoPlay.Answer(prompt.Options.First(option => option.Kind == ActionKindView.DeclineCost));
        }

        // Not with the last of its life, as for a spell of its own: the spell kept is no use to a seat that is dead.
        TakeActionAnswer paid = AutoPlay.PayUnless(prompt);
        return paid.Kind == ActionKindView.ActivateAbility
            && prompt.Options.Find(option => option.Kind == ActionKindView.ActivateAbility
                && option.Source == paid.Source && option.AbilityIndex == paid.AbilityIndex) is { } source
            && Lethal(client, source)
            && prompt.Options.Find(option => option.Kind == ActionKindView.DeclineCost) is { } decline
                ? AutoPlay.Answer(decline)
                : paid;
    }
}
