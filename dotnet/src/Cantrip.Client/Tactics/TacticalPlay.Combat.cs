using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// Attacking and blocking: the two questions this seat answers by working the combat out (<see cref="Combat"/>).
/// </summary>
public sealed partial class TacticalPlay
{
    /// <summary>How many attackers are tried in every combination before the search climbs instead.</summary>
    private const int EveryAttack = 7;

    /// <summary>
    /// How many combats one attack decision may work out before it takes the best attack found so far
    /// (<see cref="Combat.Work"/>). Never reached on an ordinary board; on a board of dozens of tokens it is what keeps
    /// the decision to a fraction of a second in a browser.
    /// </summary>
    private const int AttackWork = 60_000;

    /// <summary>The most attackers, and the most blockers, an attack is planned around: the biggest of each.</summary>
    private const int PlannedAttackers = 14;
    private const int PlannedBlockers = 12;

    /// <summary>
    /// Attacks with whichever creatures leave it best off once the defender has blocked as well as they can and has
    /// had their own attack back.
    /// <para>
    /// The second half is what makes this an attack decision rather than a damage count. A creature that attacks is
    /// tapped on the opponent's turn (CR 508.1f) unless it has vigilance, so every attack is weighed against the
    /// damage it leaves this seat open to — and an attack that would leave it dead on the board is only made when
    /// staying home would not save it either.
    /// </para>
    /// </summary>
    private AnswerPayload Attack(GameClient client, PlayerView view, DeclareAttackersPromptMessage prompt)
    {
        if (prompt.Eligible.Count == 0 || prompt.AttackablePlayers.Count == 0)
        {
            return prompt.Default();
        }

        int defender = prompt.AttackablePlayers.OrderBy(seat => LifeOf(view, seat)).First();
        int myLife = LifeOf(view, client.Seat);
        int theirLife = LifeOf(view, defender);

        // The strongest few: a board past that is decided by its biggest creatures anyway, and the rest stay home.
        List<ObjectView> eligible = [.. prompt.Eligible
            .Select(client.Object)
            .OfType<ObjectView>()
            .OrderByDescending(creature => creature.Power ?? 0)
            .Take(PlannedAttackers)];

        List<Unit> attackers = [.. eligible.Select(creature => Unit.From(creature, _model))];
        List<ObjectView> theirs = [.. Creatures(client, defender)];
        List<Unit> defenders = [.. theirs.Select(creature => Unit.From(creature, _model))];
        List<Unit> blockers = [.. defenders.Where(creature => !creature.Tapped)
            .OrderByDescending(creature => creature.Power + creature.Toughness)
            .Take(PlannedBlockers)];

        // Creatures that must attack (CR 508.1d) are in every attack considered.
        int forced = 0;
        for (int a = 0; a < attackers.Count; a++)
        {
            if (prompt.Requirements.Exists(requirement => requirement.Attacker == attackers[a].Handle))
            {
                forced |= 1 << a;
            }
        }

        var combat = new Combat(_model, attackers, blockers, myLife, theirLife, connect: Policy("connect"));

        // What stays home to block, whatever attacks: everything untapped that is not being considered for the attack.
        HashSet<ViewHandle> considered = [.. attackers.Select(attacker => attacker.Handle)];

        // A crewed Vehicle is a creature only until the turn ends (CR 702.122a): it blocks nothing on the way back.
        HashSet<ViewHandle> passing = Policy("crew") > 0
            ? [.. Creatures(client, client.Seat).Where(creature => creature.Subtypes.Contains("Vehicle")).Select(creature => creature.Handle)]
            : [];

        List<Unit> home = [.. Creatures(client, client.Seat)
            .Where(creature => !creature.Tapped && !considered.Contains(creature.Handle) && !passing.Contains(creature.Handle))
            .Select(creature => Unit.From(creature, _model))];

        // Its own planeswalkers, which an attack back may be sent at instead of at it.
        List<(int Loyalty, double Worth)> guarded = Policy("walkers") > 0
            ? [.. client.BattlefieldOf(client.Seat)
                .Where(permanent => permanent.CardTypes.Contains(CardTypeView.Planeswalker))
                .Select(walker => (LoyaltyOf(walker), _model.Worth(walker)))]
            : [];

        double perAttacker = Policy("attacktrigger");
        double[] triggers = [.. eligible.Select(creature => CardReading.AttackTrigger(CardReading.Text(creature))
            && !CardReading.Plain(CardReading.Text(creature)).Contains("whenever you attack", StringComparison.Ordinal) ? perAttacker : 0)];
        double wheneverYouAttack = perAttacker * client.BattlefieldOf(client.Seat)
            .Count(permanent => CardReading.Plain(CardReading.Text(permanent)).Contains("whenever you attack", StringComparison.Ordinal));

        // Costs to attack (CR 508.1h): what each creature costs to send at the defender or, more cheaply where that is
        // so, at one of the defender's planeswalkers (Ghostly Prison taxes the player, not the planeswalker), and the mana
        // there is to pay with, less what a creature that taps for mana stops making by attacking (CR 508.1f). An attack
        // that cannot be paid for is never made: the payment would be given up and the attack declared again.
        List<ViewHandle> theirWalkers = [.. prompt.AttackablePlaneswalkers.Where(walker => client.Object(walker)?.Controller == defender)];
        int AttackCost(ViewHandle attacker, ViewHandle walker) => prompt.CostOf(walker.IsNone
            ? new AttackMessage { Attacker = attacker, TargetPlayer = defender }
            : new AttackMessage { Attacker = attacker, TargetPermanent = walker })?.ManaValue ?? 0;
        int[] costs = [.. attackers.Select(attacker =>
            theirWalkers.Aggregate(AttackCost(attacker.Handle, ViewHandle.None), (least, walker) => Math.Min(least, AttackCost(attacker.Handle, walker))))];
        int mana = AttackCosts.Mana(client, prompt);
        bool[] makesMana = [.. eligible.Select(creature => CardReading.Plain(CardReading.Text(creature)).Contains("{t}: add", StringComparison.Ordinal))];
        bool MakesMana(ViewHandle creature) =>
            attackers.FindIndex(attacker => attacker.Handle == creature) is >= 0 and var a && makesMana[a] && !attackers[a].Has(Keywords.Vigilance);

        int CostOf(int attacking)
        {
            int total = 0;
            for (int a = 0; a < attackers.Count; a++)
            {
                if ((attacking & (1 << a)) != 0)
                {
                    total += costs[a];
                }
            }

            return total;
        }

        bool Affordable(int attacking)
        {
            if (prompt.Costs.Count == 0)
            {
                return true;
            }

            int have = mana;
            for (int a = 0; a < attackers.Count && have != int.MaxValue; a++)
            {
                if ((attacking & (1 << a)) != 0 && makesMana[a] && !attackers[a].Has(Keywords.Vigilance))
                {
                    have--;
                }
            }

            return CostOf(attacking) <= have;
        }

        double Score(int attacking)
        {
            if (!Affordable(attacking & ~forced))
            {
                return double.NegativeInfinity;
            }

            Combat.Result result = combat.BestBlocks(attacking, budget: 300, pairs: blockers.Count <= 6);
            if (result.Lethal)
            {
                return Combat.Win;
            }

            double score = result.Value;
            List<Unit> back = [.. home];

            for (int a = 0; a < attackers.Count; a++)
            {
                bool attacks = (attacking & (1 << a)) != 0;
                bool dies = (result.AttackersDead & (1 << a)) != 0;

                if (attacks)
                {
                    // A little for every point sent, so that of two attacks that come out the same the bigger is made.
                    score += triggers[a] + (0.001 * attackers[a].Power);
                }

                if (!dies && (!attacks || attackers[a].Has(Keywords.Vigilance)) && !passing.Contains(attackers[a].Handle))
                {
                    back.Add(attackers[a]);
                }
            }

            if (attacking != 0)
            {
                score += wheneverYouAttack;
            }

            List<Unit> coming = [];
            for (int d = 0; d < defenders.Count; d++)
            {
                int blockerIndex = blockers.IndexOf(defenders[d]);
                bool dead = blockerIndex >= 0 && (result.BlockersDead & (1 << blockerIndex)) != 0;
                if (!dead && defenders[d].Power > 0 && !defenders[d].Has(Keywords.Defender))
                {
                    coming.Add(defenders[d]);
                }
            }

            int life = myLife + result.AttackerGains;
            int open = Exposure(coming, back);

            if (open >= life)
            {
                return score - Policy("exposed");
            }

            // What gets through on the way back is sent wherever it costs most: at this seat's life, or at a
            // planeswalker of its own that an attack has left with nothing in front of it.
            double lost = _model.Life(life) - _model.Life(life - open);
            foreach ((int loyalty, double worth) in guarded)
            {
                lost = Math.Max(lost, worth * Math.Min(1.0, open / (double)loyalty));
            }

            return score - (Policy("crackback") * lost);
        }

        int best = forced;
        double bestScore = Score(forced);

        void Try(int attacking)
        {
            attacking |= forced;
            double score = Score(attacking);
            if (score > bestScore)
            {
                bestScore = score;
                best = attacking;
            }
        }

        int everything = (1 << attackers.Count) - 1;

        if (attackers.Count <= EveryAttack)
        {
            // Everything first: if the work runs out, the attack most likely to end the game has been looked at.
            Try(everything);

            for (int attacking = 1; attacking < everything && combat.Work < AttackWork; attacking++)
            {
                Try(attacking);
            }
        }
        else
        {
            // A crowded board: from nobody, from everybody and from the evasive ones, change one creature at a time
            // for as long as that helps.
            int evasive = 0;
            for (int a = 0; a < attackers.Count; a++)
            {
                if (attackers[a].Has(Keywords.Flying | Keywords.Unblockable | Keywords.Menace))
                {
                    evasive |= 1 << a;
                }
            }

            foreach (int start in new[] { forced, everything, evasive | forced })
            {
                int at = start;
                double atScore = Score(at);

                for (int round = 0; round < attackers.Count && combat.Work < AttackWork; round++)
                {
                    int next = at;
                    double nextScore = atScore;

                    for (int a = 0; a < attackers.Count && combat.Work < AttackWork; a++)
                    {
                        int flipped = (at ^ (1 << a)) | forced;
                        if (flipped == at)
                        {
                            continue;
                        }

                        double score = Score(flipped);
                        if (score > nextScore)
                        {
                            nextScore = score;
                            next = flipped;
                        }
                    }

                    if (next == at)
                    {
                        break;
                    }

                    (at, atScore) = (next, nextScore);
                }

                if (atScore > bestScore)
                {
                    bestScore = atScore;
                    best = at;
                }
            }
        }

        // Mana paid to attack is mana the turn's second main phase does not have: the creatures that cost something to
        // send are sent only if the attack gains a play's worth (the margin) over the same attack without them.
        int paid = 0;
        for (int a = 0; a < attackers.Count; a++)
        {
            if (costs[a] > 0 && (forced & (1 << a)) == 0)
            {
                paid |= 1 << a;
            }
        }

        if ((best & paid) != 0 && bestScore < Combat.Win && bestScore - Score(best & ~paid) < Policy("margin"))
        {
            best &= ~paid;
            bestScore = Score(best);
        }

        Dictionary<ViewHandle, ViewHandle> aside = Aside(
            client, prompt, defender, attackers, blockers, best, myLife, theirLife, aims => AimedCost(aims) <= mana);

        // What sending the attackers at these planeswalkers (none: at the player) costs, to keep to the mana there is.
        int AimedCost(ViewHandle[] aims)
        {
            int total = 0;
            for (int a = 0; a < attackers.Count; a++)
            {
                if ((best & (1 << a)) != 0 && (forced & (1 << a)) == 0)
                {
                    total += AttackCost(attackers[a].Handle, aims[a]);
                }
            }

            return total;
        }

        if (Narrating)
        {
            double stay = best == forced ? bestScore : Score(forced);
            Note($"attack scored {Num(bestScore)} against {Num(stay)} for {(forced == 0 ? "staying home" : "only what must attack")}"
                + "; their untapped blockers: "
                + (blockers.Count == 0 ? "none" : string.Join(", ", blockers.Select(blocker => $"{NameOf(client, blocker.Handle)} {blocker.Power}/{blocker.Toughness}"))));
        }

        var answer = new AttackersDeclaredAnswer();
        for (int a = 0; a < attackers.Count; a++)
        {
            if ((best & (1 << a)) == 0)
            {
                continue;
            }

            // A goaded creature at a seat its requirement allows (CR 701.15b).
            AttackRequirementMessage? must = prompt.Requirements.Find(requirement => requirement.Attacker == attackers[a].Handle);
            answer.Attacks.Add(new AttackMessage
            {
                Attacker = attackers[a].Handle,
                TargetPlayer = must is { Players.Count: > 0 } && !must.Players.Contains(defender) ? must.Players[0] : defender,
                TargetPermanent = aside.GetValueOrDefault(attackers[a].Handle, ViewHandle.None),
            });
        }

        // Past the ones considered, a creature that must attack still has to.
        foreach (AttackRequirementMessage requirement in prompt.Requirements)
        {
            if (!answer.Attacks.Exists(attack => attack.Attacker == requirement.Attacker))
            {
                answer.Attacks.Add(new AttackMessage
                {
                    Attacker = requirement.Attacker,
                    TargetPlayer = requirement.Players.Count > 0 && !requirement.Players.Contains(defender) ? requirement.Players[0] : defender,
                });
            }
        }

        // A creature something says can't attack what it was aimed at (CR 508.1c) goes at the first thing it may.
        for (int i = 0; i < answer.Attacks.Count; i++)
        {
            if (prompt.IsForbidden(answer.Attacks[i]))
            {
                answer.Attacks[i] = prompt.AttacksFor(answer.Attacks[i].Attacker)[0];
            }
        }

        answer.Attacks = AttackCosts.Within(prompt, answer.Attacks, mana, MakesMana);
        return answer;
    }

    private static int LoyaltyOf(ObjectView walker) =>
        Math.Max(walker.Counters.Find(counter => counter.Name == "loyalty")?.Count ?? 1, 1);

    /// <summary>
    /// Which of the creatures in an attack go at a planeswalker rather than at the player it belongs to (CR 508.1b),
    /// by attacker: none, unless that leaves this seat better off once the defender has blocked as well as it can.
    /// <para>
    /// The attack was chosen as one on the player. Two other ways of sending the same creatures are then tried — the
    /// ones nobody would block at the planeswalkers in turn, most valuable first, and everything at the most valuable
    /// — and each is worked out against the blocks that are best against it. A planeswalker left alone adds loyalty
    /// every turn, so the creatures that would have got through to a player at twenty life are nearly always better
    /// sent at it.
    /// </para>
    /// </summary>
    private Dictionary<ViewHandle, ViewHandle> Aside(
        GameClient client, DeclareAttackersPromptMessage prompt, int defender, List<Unit> attackers, List<Unit> blockers, int attacking,
        int myLife, int theirLife, Func<ViewHandle[], bool> affordable)
    {
        Dictionary<ViewHandle, ViewHandle> aside = [];
        if (Policy("walkers") <= 0 || attacking == 0 || prompt.AttackablePlaneswalkers.Count == 0)
        {
            return aside;
        }

        List<ObjectView> walkers = [.. prompt.AttackablePlaneswalkers
            .Select(client.Object)
            .OfType<ObjectView>()
            .Where(walker => walker.Controller == defender)
            .OrderByDescending(_model.Worth)
            .Take(4)];

        if (walkers.Count == 0)
        {
            return aside;
        }

        List<(int Loyalty, double Worth)> worths = [.. walkers.Select(walker => (LoyaltyOf(walker), _model.Worth(walker)))];

        Combat.Result Blocked(int[]? aims) => new Combat(
            _model, attackers, blockers, myLife, theirLife, connect: Policy("connect"), walkers: worths, aims: aims)
            .BestBlocks(attacking, budget: 2000, pairs: blockers.Count <= 6);

        Combat.Result atPlayer = Blocked(null);
        if (atPlayer.Lethal)
        {
            return aside;
        }

        // A creature that must attack a player (CR 701.15b) stays on the player.
        bool Bound(int a) => prompt.Requirements.Exists(requirement => requirement.Attacker == attackers[a].Handle && requirement.Players.Count > 0);

        int[] none = [.. attackers.Select(_ => -1)];

        // The ones the defender would leave unblocked, biggest first, at each planeswalker until it is dead.
        int[] unblocked = (int[])none.Clone();
        List<int> free = [.. Enumerable.Range(0, attackers.Count)
            .Where(a => (attacking & (1 << a)) != 0 && !Bound(a) && !atPlayer.Blocks.Contains(a))
            .OrderByDescending(a => attackers[a].Power)];

        for (int w = 0; w < walkers.Count && free.Count > 0; w++)
        {
            int needed = worths[w].Loyalty;
            while (needed > 0 && free.Count > 0)
            {
                unblocked[free[0]] = w;
                needed -= attackers[free[0]].Has(Keywords.DoubleStrike) ? 2 * attackers[free[0]].Power : attackers[free[0]].Power;
                free.RemoveAt(0);
            }
        }

        int[] everything = [.. attackers.Select((_, a) => (attacking & (1 << a)) != 0 && !Bound(a) ? 0 : -1)];

        int[]? chosen = null;
        double most = atPlayer.Value + Policy("margin");

        foreach (int[] aims in new[] { unblocked, everything })
        {
            if (!aims.Any(aim => aim >= 0) || !affordable([.. aims.Select(aim => aim >= 0 ? walkers[aim].Handle : ViewHandle.None)]))
            {
                continue;
            }

            Combat.Result result = Blocked(aims);
            if (result.Value > most)
            {
                most = result.Value;
                chosen = aims;
            }
        }

        for (int a = 0; chosen is not null && a < attackers.Count; a++)
        {
            if (chosen[a] >= 0)
            {
                aside[attackers[a].Handle] = walkers[chosen[a]].Handle;
            }
        }

        return aside;
    }

    /// <summary>
    /// The damage these creatures would deal attacking into those, if every blocker were thrown in front of the
    /// biggest thing it could block: what an attack back cannot be stopped from doing.
    /// </summary>
    private static int Exposure(List<Unit> attackers, List<Unit> blockers)
    {
        List<Unit> free = [.. blockers.OrderBy(blocker => blocker.Worth)];
        int through = 0;

        foreach (Unit attacker in attackers.OrderByDescending(attacker => attacker.Power))
        {
            int needed = attacker.Has(Keywords.Menace) ? 2 : 1;
            List<Unit> able = [.. free.Where(blocker => !blocker.Has(Keywords.CantBlock)
                && !attacker.Has(Keywords.Unblockable)
                && (!attacker.Has(Keywords.Flying) || blocker.Has(Keywords.Flying | Keywords.Reach))
                && attacker.Has(Keywords.Shadow) == blocker.Has(Keywords.Shadow)).Take(needed)];

            if (able.Count < needed)
            {
                through += attacker.Has(Keywords.DoubleStrike) ? 2 * attacker.Power : attacker.Power;
                continue;
            }

            foreach (Unit blocker in able)
            {
                free.Remove(blocker);
            }

            if (attacker.Has(Keywords.Trample))
            {
                through += Math.Max(attacker.Power - able.Sum(blocker => blocker.Toughness), 0);
            }
        }

        return through;
    }

    /// <summary>
    /// Blocks however leaves it best off: every declaration the prompt allows is worked out, on a board small enough
    /// to, and the one that costs the attacker most — in creatures traded and life kept — is made. A chump block
    /// falls out of the same sum: it is made when the life is worth more than the creature, which at twenty life it
    /// is not and at three it is.
    /// </summary>
    private AnswerPayload Block(GameClient client, PlayerView view, DeclareBlockersPromptMessage prompt)
    {
        if (prompt.Eligible.Count == 0 || prompt.Attackers.Count == 0)
        {
            return prompt.Default();
        }

        List<ObjectView> attacking = [.. prompt.Attackers.Select(client.Object).OfType<ObjectView>().Take(30)];
        List<ObjectView> blocking = [.. prompt.Eligible.Select(client.Object).OfType<ObjectView>()
            .OrderByDescending(creature => creature.Power ?? 0).Take(30)];

        if (attacking.Count == 0 || blocking.Count == 0)
        {
            return prompt.Default();
        }

        List<Unit> attackers = [.. attacking.Select(creature => Unit.From(creature, _model))];
        List<Unit> blockers = [.. blocking.Select(creature => Unit.From(creature, _model))];

        // Which pairs are allowed and how many blockers each attacker needs are the prompt's to say, not a keyword's.
        bool[,] allowed = new bool[blockers.Count, attackers.Count];
        foreach (BlockMessage pair in prompt.Allowed)
        {
            int b = blockers.FindIndex(blocker => blocker.Handle == pair.Blocker);
            int a = attackers.FindIndex(attacker => attacker.Handle == pair.Attacker);
            if (a >= 0 && b >= 0)
            {
                allowed[b, a] = true;
            }
        }

        int[] minimum = [.. attackers.Select(attacker => prompt.MinimumFor(prompt.Attackers.IndexOf(attacker.Handle)))];
        int[] maximum = [.. attackers.Select(attacker => prompt.MaximumFor(prompt.Attackers.IndexOf(attacker.Handle)))];

        // An attacker sent at one of this seat's planeswalkers costs the planeswalker loyalty and the seat no life.
        List<ObjectView> walkers = [];
        int[] aims = [.. attackers.Select(_ => -1)];

        for (int a = 0; Policy("walkers") > 0 && a < attackers.Count; a++)
        {
            if (view.Combat.Attackers.Find(attack => attack.Attacker == attackers[a].Handle) is { TargetKind: AttackTargetKindView.Planeswalker } attack
                && client.Object(attack.TargetPermanent) is { } walker)
            {
                if (!walkers.Contains(walker))
                {
                    walkers.Add(walker);
                }

                aims[a] = walkers.IndexOf(walker);
            }
        }

        int attackerSeat = attacking[0].Controller;
        var combat = new Combat(
            _model, attackers, blockers, LifeOf(view, attackerSeat), LifeOf(view, client.Seat), allowed, minimum, Policy("connect"),
            [.. walkers.Select(walker => (LoyaltyOf(walker), _model.Worth(walker)))], aims, maximum);

        Combat.Result result = combat.BestBlocks((1 << attackers.Count) - 1, budget: 20000);

        var answer = new BlockersDeclaredAnswer();
        for (int b = 0; b < result.Blocks.Length; b++)
        {
            if (result.Blocks[b] >= 0)
            {
                answer.Blocks.Add(new BlockMessage { Blocker = blockers[b].Handle, Attacker = attackers[result.Blocks[b]].Handle });
            }
        }

        if (Narrating)
        {
            Note($"the attack, so blocked, is worth {Num(result.Value)} to the attacker and deals me {result.Damage}");
        }

        // Blocks an effect requires (CR 509.1c: Lure) that the best blocks leave out are not mine to decline: the engine's
        // own declaration obeys them.
        if (!prompt.ObeysRequirements(answer.Blocks))
        {
            if (Narrating)
            {
                Note("an effect requires blocks these leave out, so I block as it requires");
            }

            return (BlockersDeclaredAnswer)prompt.Default();
        }

        return answer;
    }
}
