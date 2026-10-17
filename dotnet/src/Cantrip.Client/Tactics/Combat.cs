using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>A creature as combat arithmetic sees it: its size, what is marked on it, its keywords and what it is worth.</summary>
public sealed class Unit
{
    public ViewHandle Handle { get; init; }

    public int Power { get; init; }

    public int Toughness { get; init; }

    /// <summary>Damage already marked on it this turn (CR 120.3).</summary>
    public int Damage { get; init; }

    public Keywords Keys { get; init; }

    /// <summary>What it is worth to its controller (<see cref="TacticalModel.Worth(ObjectView)"/>).</summary>
    public double Worth { get; init; }

    public bool Tapped { get; init; }

    /// <summary>Whether it does something when it deals combat damage to a player.</summary>
    public bool Connects { get; init; }

    public bool Has(Keywords keyword) => (Keys & keyword) != 0;

    /// <summary>Whether this could block that, as far as keywords say (CR 702.9b): the server's own list is the authority when there is one.</summary>
    public bool CouldBlock(Unit attacker)
    {
        ArgumentNullException.ThrowIfNull(attacker);

        return !Tapped
            && !Has(Keywords.CantBlock)
            && !attacker.Has(Keywords.Unblockable)
            && (!attacker.Has(Keywords.Flying) || Has(Keywords.Flying | Keywords.Reach))
            && attacker.Has(Keywords.Shadow) == Has(Keywords.Shadow);
    }

    public static Unit From(ObjectView creature, TacticalModel model)
    {
        ArgumentNullException.ThrowIfNull(creature);
        ArgumentNullException.ThrowIfNull(model);

        Keywords keys = CardReading.KeywordsOf(creature);
        return new Unit
        {
            Handle = creature.Handle,
            Power = Math.Max(creature.Power ?? 0, 0),
            Toughness = creature.Toughness ?? 0,
            Damage = creature.DamageMarked,
            Keys = keys,
            Worth = model.Worth(creature, keys),
            Tapped = creature.Tapped,
            Connects = CardReading.ConnectTrigger(CardReading.Text(creature)),
        };
    }
}

/// <summary>
/// One combat, worked out ahead of time: some creatures attacking one player, some creatures able to block them.
/// <para>
/// Combat is the one part of the game a client can predict from what it is shown — power, toughness and a handful of
/// keywords decide it (CR 510) — so it is the part this seat searches rather than guesses at. An attacker and
/// whatever blocks it fight on their own, since a creature blocks one attacker, which makes a whole combat a sum of
/// small fights and one life total: cheap enough to try every block against every attack on an ordinary board, and
/// to climb towards a good one on a crowded one.
/// </para>
/// <para>
/// Everything is scored from the attacking side, in the model's units: what the defender loses in creatures and
/// life, less what the attacker does. The defender is assumed to block as well as this seat would, which is the
/// worst case for an attack and so the one worth planning for.
/// </para>
/// </summary>
public sealed class Combat
{
    /// <summary>What winning on the spot is scored as: more than any board is worth.</summary>
    public const double Win = 1000;

    private readonly TacticalModel _model;
    private readonly IReadOnlyList<Unit> _attackers;
    private readonly IReadOnlyList<Unit> _blockers;
    private readonly bool[,] _allowed;
    private readonly int[] _minimum;
    private readonly int[] _maximum;
    private readonly int _attackerLife;
    private readonly int _defenderLife;
    private readonly double _connect;
    private readonly IReadOnlyList<(int Loyalty, double Worth)> _walkers;
    private readonly int[]? _aims;
    private readonly Dictionary<long, Fight> _fights = [];

    /// <param name="attackers">Every creature that might attack. Which of them do is said per question.</param>
    /// <param name="blockers">Every creature that might block; at most thirty are considered.</param>
    /// <param name="allowed">Which blocker may block which attacker, or null to go by keywords.</param>
    /// <param name="minimum">The fewest blockers each attacker needs, or null to go by menace.</param>
    /// <param name="connect">What a creature's "deals combat damage to a player" trigger is worth each time it happens.</param>
    /// <param name="walkers">The defender's planeswalkers that are being attacked: the loyalty each has and what it is worth.</param>
    /// <param name="aims">
    /// For each attacker, which of <paramref name="walkers"/> it attacks (CR 508.1b), or -1 for the player. Null when
    /// every attacker attacks the player.
    /// </param>
    /// <param name="maximum">
    /// The most blockers each attacker may have (CR 509.1b), or null to go by "can't be blocked by more than one creature".
    /// </param>
    public Combat(
        TacticalModel model,
        IReadOnlyList<Unit> attackers,
        IReadOnlyList<Unit> blockers,
        int attackerLife,
        int defenderLife,
        bool[,]? allowed = null,
        int[]? minimum = null,
        double connect = 0,
        IReadOnlyList<(int Loyalty, double Worth)>? walkers = null,
        int[]? aims = null,
        int[]? maximum = null)
    {
        ArgumentNullException.ThrowIfNull(model);
        ArgumentNullException.ThrowIfNull(attackers);
        ArgumentNullException.ThrowIfNull(blockers);

        _model = model;
        _attackers = attackers;
        _blockers = blockers.Count > 30 ? [.. blockers.Take(30)] : blockers;
        _attackerLife = attackerLife;
        _defenderLife = defenderLife;
        _connect = connect;
        _walkers = walkers ?? [];
        _aims = _walkers.Count > 0 && aims is { Length: > 0 } && aims.Length == attackers.Count ? aims : null;

        _allowed = allowed ?? new bool[_blockers.Count, attackers.Count];
        _minimum = minimum ?? new int[attackers.Count];
        _maximum = maximum ?? new int[attackers.Count];

        for (int a = 0; a < attackers.Count; a++)
        {
            if (minimum is null)
            {
                _minimum[a] = attackers[a].Has(Keywords.Menace) ? 2 : 1;
            }

            if (maximum is null)
            {
                _maximum[a] = attackers[a].Has(Keywords.OneBlocker) ? 1 : int.MaxValue;
            }

            if (allowed is null)
            {
                for (int b = 0; b < _blockers.Count; b++)
                {
                    _allowed[b, a] = _blockers[b].CouldBlock(attackers[a]);
                }
            }
        }
    }

    /// <summary>How one attacker's fight came out.</summary>
    private readonly record struct Fight(int ToPlayer, bool AttackerDies, int BlockersDead, int AttackerGains, int DefenderGains, double Swing);

    /// <summary>A combat as it would come out.</summary>
    /// <param name="Value">What it is worth to the attacking side.</param>
    /// <param name="Blocks">For each blocker, the index of the attacker it blocks, or -1.</param>
    /// <param name="Damage">Combat damage the defending player takes.</param>
    /// <param name="AttackersDead">A bit per attacker that dies.</param>
    /// <param name="BlockersDead">A bit per blocker that dies.</param>
    /// <param name="AttackerGains">Life the attacking player gains.</param>
    /// <param name="DefenderGains">Life the defending player gains.</param>
    public readonly record struct Result(
        double Value, int[] Blocks, int Damage, int AttackersDead, int BlockersDead, int AttackerGains, int DefenderGains)
    {
        public bool Lethal => Value >= Win / 2;
    }

    /// <summary>
    /// How many combats have been worked out so far. What a search is costing, counted in the one unit that is the
    /// same on every machine: a caller that has spent enough stops looking and takes the best it has found, so a
    /// board of thirty tokens costs a browser a bounded amount of work rather than however long it takes.
    /// </summary>
    public int Work { get; private set; }

    /// <summary>
    /// The blocks that are best for the defender against these attackers, and how the combat then comes out.
    /// </summary>
    /// <param name="attacking">A bit per attacker that is attacking.</param>
    /// <param name="budget">How many whole block declarations may be tried one by one before climbing instead.</param>
    /// <param name="pairs">
    /// Whether climbing also tries two blockers at once on every attacker, which finds the double block neither would
    /// make alone and costs the square of the blockers. Always tried on an attacker that needs two.
    /// </param>
    public Result BestBlocks(int attacking, int budget = 4000, bool pairs = true)
    {
        int count = _blockers.Count;
        int[] blocks = new int[count];
        Array.Fill(blocks, -1);

        List<int> attackers = [];
        for (int a = 0; a < _attackers.Count; a++)
        {
            if ((attacking & (1 << a)) != 0)
            {
                attackers.Add(a);
            }
        }

        if (attackers.Count == 0)
        {
            return Evaluate(attacking, blocks);
        }

        // Blockers with nothing they may block are left out of the search altogether.
        List<int> free = [];
        double combinations = 1;
        for (int b = 0; b < count; b++)
        {
            int choices = attackers.Count(a => _allowed[b, a]);
            if (choices > 0)
            {
                free.Add(b);
                combinations *= choices + 1;
            }
        }

        if (free.Count == 0)
        {
            return Evaluate(attacking, blocks);
        }

        Result best = Evaluate(attacking, blocks);

        if (combinations <= budget)
        {
            int[] trying = (int[])blocks.Clone();
            Every(attacking, attackers, free, 0, trying, ref best);
            return best;
        }

        return Climb(attacking, attackers, free, best, pairs);
    }

    /// <summary>How the combat comes out with these attackers and exactly these blocks.</summary>
    public Result Evaluate(int attacking, int[] blocks)
    {
        ArgumentNullException.ThrowIfNull(blocks);

        Work++;

        double swing = 0;
        int damage = 0;
        int attackersDead = 0;
        int blockersDead = 0;
        int attackerGains = 0;
        int defenderGains = 0;
        int[]? onWalkers = null;

        for (int a = 0; a < _attackers.Count; a++)
        {
            if ((attacking & (1 << a)) == 0)
            {
                continue;
            }

            int mask = 0;
            for (int b = 0; b < blocks.Length; b++)
            {
                if (blocks[b] == a)
                {
                    mask |= 1 << b;
                }
            }

            Fight fight = FightOf(a, mask);
            swing += fight.Swing;
            attackerGains += fight.AttackerGains;

            // What gets past the blockers lands on whatever the attacker was sent at (CR 510.1b): a planeswalker loses
            // that much loyalty, and the player it belongs to loses no life.
            if (_aims is not null && _aims[a] >= 0 && _aims[a] < _walkers.Count)
            {
                onWalkers ??= new int[_walkers.Count];
                onWalkers[_aims[a]] += fight.ToPlayer;

                // Damage to a planeswalker is not damage to a player: no "deals combat damage to a player" trigger.
                if (fight.ToPlayer > 0 && _attackers[a].Connects)
                {
                    swing -= _connect;
                }
            }
            else
            {
                damage += fight.ToPlayer;
            }

            defenderGains += fight.DefenderGains;
            blockersDead |= fight.BlockersDead;
            if (fight.AttackerDies)
            {
                attackersDead |= 1 << a;
            }
        }

        // A planeswalker is worth its share of what it was for each point of loyalty taken, and all of it once dead.
        double walkers = 0;
        for (int w = 0; onWalkers is not null && w < onWalkers.Length; w++)
        {
            walkers += _walkers[w].Worth * Math.Min(1.0, onWalkers[w] / (double)Math.Max(_walkers[w].Loyalty, 1));
        }

        int defenderLeft = _defenderLife + defenderGains - damage;
        double value = damage > 0 && defenderLeft <= 0
            ? Win
            : swing + walkers
              + (_model.Life(_defenderLife) - _model.Life(defenderLeft))
              + (_model.Life(_attackerLife + attackerGains) - _model.Life(_attackerLife));

        return new Result(value, (int[])blocks.Clone(), damage, attackersDead, blockersDead, attackerGains, defenderGains);
    }

    /// <summary>
    /// Whether a declaration gives every blocked attacker as many blockers as it needs, and no more than it may have
    /// (CR 509.1b).
    /// </summary>
    private bool Legal(List<int> attackers, int[] blocks)
    {
        foreach (int a in attackers)
        {
            if (_minimum[a] <= 1 && _maximum[a] == int.MaxValue)
            {
                continue;
            }

            int blocking = 0;
            for (int b = 0; b < blocks.Length; b++)
            {
                if (blocks[b] == a)
                {
                    blocking++;
                }
            }

            if ((blocking > 0 && blocking < _minimum[a]) || blocking > _maximum[a])
            {
                return false;
            }
        }

        return true;
    }

    private void Every(int attacking, List<int> attackers, List<int> free, int at, int[] blocks, ref Result best)
    {
        if (at == free.Count)
        {
            if (Legal(attackers, blocks))
            {
                Result result = Evaluate(attacking, blocks);
                if (result.Value < best.Value)
                {
                    best = result;
                }
            }

            return;
        }

        int blocker = free[at];
        blocks[blocker] = -1;
        Every(attacking, attackers, free, at + 1, blocks, ref best);

        foreach (int a in attackers)
        {
            if (_allowed[blocker, a])
            {
                blocks[blocker] = a;
                Every(attacking, attackers, free, at + 1, blocks, ref best);
            }
        }

        blocks[blocker] = -1;
    }

    /// <summary>
    /// A crowded board: start from no blocks and keep making whichever one change helps the defender most — one
    /// blocker moved, or two put on the same attacker at once, which is the only way onto an attacker with menace
    /// and the only way a double block that neither creature would make alone is ever found.
    /// </summary>
    private Result Climb(int attacking, List<int> attackers, List<int> free, Result best, bool pairs)
    {
        int[] blocks = (int[])best.Blocks.Clone();

        for (int round = 0; round < 24; round++)
        {
            Result improved = best;

            foreach (int b in free)
            {
                int was = blocks[b];
                foreach (int a in attackers.Append(-1))
                {
                    if (a == was || (a >= 0 && !_allowed[b, a]))
                    {
                        continue;
                    }

                    blocks[b] = a;
                    Consider(attacking, attackers, blocks, ref improved);
                }

                blocks[b] = was;
            }

            foreach (int a in attackers)
            {
                if (!pairs && _minimum[a] < 2)
                {
                    continue;
                }

                for (int i = 0; i < free.Count; i++)
                {
                    int first = free[i];
                    if (blocks[first] == a || !_allowed[first, a])
                    {
                        continue;
                    }

                    for (int j = i + 1; j < free.Count; j++)
                    {
                        int second = free[j];
                        if (blocks[second] == a || !_allowed[second, a])
                        {
                            continue;
                        }

                        (int wasFirst, int wasSecond) = (blocks[first], blocks[second]);
                        (blocks[first], blocks[second]) = (a, a);
                        Consider(attacking, attackers, blocks, ref improved);
                        (blocks[first], blocks[second]) = (wasFirst, wasSecond);
                    }
                }
            }

            if (improved.Value >= best.Value - 1e-9)
            {
                break;
            }

            best = improved;
            blocks = (int[])best.Blocks.Clone();
        }

        return best;
    }

    private void Consider(int attacking, List<int> attackers, int[] blocks, ref Result best)
    {
        if (!Legal(attackers, blocks))
        {
            return;
        }

        Result result = Evaluate(attacking, blocks);
        if (result.Value < best.Value)
        {
            best = result;
        }
    }

    private Fight FightOf(int attacker, int blockerMask)
    {
        long key = ((long)attacker << 32) | (uint)blockerMask;
        if (!_fights.TryGetValue(key, out Fight fight))
        {
            fight = Resolve(attacker, blockerMask);
            _fights[key] = fight;
        }

        return fight;
    }

    /// <summary>
    /// One attacker against whatever blocks it, through the first-strike and regular damage steps (CR 510.4).
    /// </summary>
    private Fight Resolve(int attackerIndex, int blockerMask)
    {
        Unit attacker = _attackers[attackerIndex];

        List<int> blocking = [];
        for (int b = 0; b < _blockers.Count; b++)
        {
            if ((blockerMask & (1 << b)) != 0)
            {
                blocking.Add(b);
            }
        }

        int toPlayer = 0;
        int attackerGains = 0;
        int defenderGains = 0;
        int onAttacker = attacker.Damage;
        bool attackerTouched = false;
        bool attackerDead = false;

        int[] onBlocker = new int[blocking.Count];
        bool[] blockerTouched = new bool[blocking.Count];
        bool[] blockerDead = new bool[blocking.Count];
        for (int i = 0; i < blocking.Count; i++)
        {
            onBlocker[i] = _blockers[blocking[i]].Damage;
        }

        bool anyFirst = attacker.Has(Keywords.FirstStrike | Keywords.DoubleStrike)
            || blocking.Exists(b => _blockers[b].Has(Keywords.FirstStrike | Keywords.DoubleStrike));

        for (int step = anyFirst ? 0 : 1; step < 2; step++)
        {
            int[] dealt = new int[blocking.Count];
            bool[] touch = new bool[blocking.Count];
            int dealtToAttacker = 0;
            bool touchAttacker = false;

            // An attacker that died in the first-strike step is gone before the regular one: nothing hits it then.
            bool attackerGone = attackerDead;

            if (!attackerDead && Strikes(attacker, step) && attacker.Power > 0)
            {
                int left = attacker.Power;

                if (blocking.Count == 0)
                {
                    toPlayer += left;
                    attackerGains += attacker.Has(Keywords.Lifelink) ? left : 0;
                    left = 0;
                }
                else
                {
                    List<int> alive = [.. Enumerable.Range(0, blocking.Count).Where(i => !blockerDead[i])];
                    int lethalToAll = alive.Sum(i => Needed(attacker, _blockers[blocking[i]], onBlocker[i]));

                    if (alive.Count == 0)
                    {
                        // Blocked, and every blocker already dead: trample sends it all through and nothing else
                        // sends any (CR 702.19e, 510.1c).
                        if (attacker.Has(Keywords.Trample))
                        {
                            toPlayer += left;
                            attackerGains += attacker.Has(Keywords.Lifelink) ? left : 0;
                        }

                        left = 0;
                    }
                    else if (attacker.Has(Keywords.Trample) && left >= lethalToAll)
                    {
                        // Trample: lethal to every blocker first, an indestructible one included, and only what is
                        // left over goes to the player (CR 702.19b).
                        foreach (int i in alive)
                        {
                            dealt[i] = Needed(attacker, _blockers[blocking[i]], onBlocker[i]);
                            touch[i] = attacker.Has(Keywords.Deathtouch);
                        }

                        toPlayer += left - lethalToAll;
                        attackerGains += attacker.Has(Keywords.Lifelink) ? left : 0;
                        left = 0;
                    }
                    else
                    {
                        // Lethal to each blocker it can kill, the ones worth most per point first. What is left is
                        // marked on one it could not: it stays there for a second strike to finish.
                        attackerGains += attacker.Has(Keywords.Lifelink) ? left : 0;

                        foreach (int i in alive.OrderByDescending(i =>
                            _blockers[blocking[i]].Worth / Math.Max(Needed(attacker, _blockers[blocking[i]], onBlocker[i]), 1)))
                        {
                            Unit blocker = _blockers[blocking[i]];
                            int needed = Needed(attacker, blocker, onBlocker[i]);
                            if (blocker.Has(Keywords.Indestructible) || needed > left)
                            {
                                continue;
                            }

                            dealt[i] = needed;
                            touch[i] = attacker.Has(Keywords.Deathtouch);
                            left -= needed;
                        }

                        if (left > 0)
                        {
                            int spare = alive.Find(i => dealt[i] == 0);
                            int onto = alive.Exists(i => dealt[i] == 0) ? spare : alive[0];
                            dealt[onto] += left;
                            touch[onto] = attacker.Has(Keywords.Deathtouch);
                            left = 0;
                        }
                    }
                }
            }

            for (int i = 0; i < blocking.Count; i++)
            {
                Unit blocker = _blockers[blocking[i]];
                if (attackerGone || blockerDead[i] || !Strikes(blocker, step) || blocker.Power <= 0)
                {
                    continue;
                }

                dealtToAttacker += blocker.Power;
                touchAttacker |= blocker.Has(Keywords.Deathtouch);
                defenderGains += blocker.Has(Keywords.Lifelink) ? blocker.Power : 0;
            }

            // Damage in a step is dealt at once (CR 510.2), so nothing that dies in it fails to hit back.
            onAttacker += dealtToAttacker;
            attackerTouched |= touchAttacker && dealtToAttacker > 0;
            attackerDead |= !attacker.Has(Keywords.Indestructible)
                && ((onAttacker >= attacker.Toughness && dealtToAttacker > 0) || attackerTouched);

            for (int i = 0; i < blocking.Count; i++)
            {
                Unit blocker = _blockers[blocking[i]];
                onBlocker[i] += dealt[i];
                blockerTouched[i] |= touch[i] && dealt[i] > 0;
                blockerDead[i] |= !blocker.Has(Keywords.Indestructible)
                    && ((onBlocker[i] >= blocker.Toughness && dealt[i] > 0) || blockerTouched[i]);
            }
        }

        int dead = 0;
        double swing = attackerDead ? -attacker.Worth : 0;
        for (int i = 0; i < blocking.Count; i++)
        {
            if (blockerDead[i])
            {
                dead |= 1 << blocking[i];
                swing += _blockers[blocking[i]].Worth;
            }
        }

        if (toPlayer > 0 && attacker.Connects)
        {
            swing += _connect;
        }

        return new Fight(toPlayer, attackerDead, dead, attackerGains, defenderGains, swing);
    }

    /// <summary>Whether a creature deals damage in the first-strike step (0) or the regular one (1) (CR 510.4).</summary>
    private static bool Strikes(Unit unit, int step) => step == 0
        ? unit.Has(Keywords.FirstStrike | Keywords.DoubleStrike)
        : !unit.Has(Keywords.FirstStrike) || unit.Has(Keywords.DoubleStrike);

    /// <summary>Lethal damage from this source to that creature (CR 702.2c, 702.19b).</summary>
    private static int Needed(Unit source, Unit creature, int marked) =>
        source.Has(Keywords.Deathtouch) ? 1 : Math.Max(creature.Toughness - marked, 1);
}
