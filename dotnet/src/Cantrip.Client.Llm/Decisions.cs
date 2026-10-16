using System.Globalization;
using System.Text;
using Cantrip.Protocol;

namespace Cantrip.Client.Llm;

/// <summary>
/// Every question the server can ask a seat, turned into keyed multiple choice and back again.
/// <para>
/// One rule runs through all of it: a choice is only ever built out of something the prompt itself offered — an
/// eligible attacker, an allowed blocker pair, a candidate target, an option in the list — and the answer is the
/// thing that was chosen, unchanged. So the server stays the only authority on what is legal, and this never has to
/// agree with it about a rule. It is the same invariant <see cref="RandomPlay"/> relies on, which is why a bot that
/// picks at random and a bot that thinks about it can share a harness.
/// </para>
/// <para>
/// Some questions are not one choice. Declaring attackers is a choice per creature, a divided spell is targets and
/// then a split, a scry is a choice per card. Those are decomposed into a run of ordinary questions rather than
/// enumerated as one, because the enumeration of "which subset of six creatures attacks" is sixty-four choices and
/// the decomposition is six. A chooser answers them in order, seeing what it has already said.
/// </para>
/// <para>
/// A question with one possible answer is never asked. Most of a game is priority with nothing worth doing, and
/// every question skipped here is a call not made.
/// </para>
/// </summary>
public sealed class Decisions
{
    /// <summary>X not yet chosen for the announcement being paid for, as opposed to chosen and zero.</summary>
    private const int Unchosen = -1;

    /// <summary>
    /// The most choices one question offers. Past this a list stops being a question and starts being a haystack, so
    /// what cannot be offered is answered by the prompt's own default instead of badly.
    /// </summary>
    private const int MostChoices = 60;

    /// <summary>
    /// Questions one step may take before this seat stops asking and passes.
    /// <para>
    /// A backstop against a loop, and against what a loop costs when every lap is a paid request. A step in which a
    /// player really has forty decisions to make does not happen; a step in which the same offer keeps coming back
    /// does, and without a ceiling it goes on until the credits do.
    /// </para>
    /// </summary>
    private const int MostQuestions = 40;

    private readonly IChooser _chooser;

    /// <summary>
    /// Spells announced this step and then given up on, which are not offered again until the step moves on.
    /// <para>
    /// Giving up on a spell (CR 733.1) puts the game back exactly where it was, with the same option on the same
    /// menu — so a chooser that liked it a moment ago likes it again, and a deterministic one likes it for ever.
    /// <see cref="RandomPlay"/> never notices this, because a fresh draw eventually passes; a seat that thinks about
    /// its answer has to be told.
    /// </para>
    /// </summary>
    private readonly HashSet<ViewHandle> _abandoned = [];

    /// <summary>
    /// The point this seat said to stop passing at, or none while it is answering every window.
    /// <para>
    /// Holding off is not only a saving, it closes a leak. A seat is asked at priority exactly when it has something
    /// it could cast, so a seat that thinks for a second at every phase of a quiet turn has told the other player it
    /// is holding an instant — information the board does not show and the rules do not give away. One pause and
    /// then silence says nothing.
    /// </para>
    /// <para>
    /// A point rather than a flag, because "I am done" is not the only useful thing to say. Keeping mana up for the
    /// opponent's end step and keeping it up for their upkeep are different plans, and a seat that could only hold
    /// to the end of the turn could not express either. <see cref="Stir"/> cuts any of them short.
    /// </para>
    /// </summary>
    private (int Turn, TurnStepView Step)? _until;

    /// <summary>The turn and step the counters below belong to.</summary>
    private (int Turn, TurnStepView Step) _when = (-1, default);

    /// <summary>How many questions have been put in that step.</summary>
    private int _asked;

    /// <summary>The X this seat is paying for, chosen as the spell was announced. <see cref="Unchosen"/> until then.</summary>
    private int _targetX = Unchosen;

    /// <summary>The spell whose optional cost was last decided, so one decision covers the whole payment.</summary>
    private ViewHandle _unless;

    /// <summary>Whether that decision was to pay.</summary>
    private bool _paying;

    public Decisions(IChooser chooser)
    {
        ArgumentNullException.ThrowIfNull(chooser);
        _chooser = chooser;
    }

    /// <summary>
    /// Something has happened that a seat holding off for the turn ought to look at again: a spell cast, a permanent
    /// gone, damage, a life total moved. See <see cref="_holding"/>.
    /// <para>
    /// Told rather than worked out here, because what counts as something happening is what the seat was told about
    /// — the events it narrates — and that is <see cref="LlmPlay"/>'s to know.
    /// </para>
    /// </summary>
    public void Stir() => _until = null;

    /// <summary>
    /// One answer to one question from the server.
    /// <para>
    /// Public and driven only by a prompt and a view, so what this seat would do with any board can be asked in a
    /// test with a scripted chooser and no game behind it.
    /// </para>
    /// </summary>
    /// <exception cref="NoChoiceException">The chooser could not answer, so the caller falls back to the prompt's own.</exception>
    public AnswerPayload Answer(GameClient client, PromptPayload prompt)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);

        return prompt switch
        {
            PriorityPromptMessage priority => Priority(client, priority),
            PayCostsPromptMessage pay => Pay(client, pay),
            UnlessPaysPromptMessage unless => Unless(client, unless),
            DeclareAttackersPromptMessage attackers => Attack(client, attackers),
            DeclareBlockersPromptMessage blockers => Block(client, blockers),
            AssignCombatDamagePromptMessage damage => Damage(client, damage),
            ChooseTargetPromptMessage target => Target(client, target),
            ChooseDividedTargetsPromptMessage divided => Divided(client, divided),
            ChooseModesPromptMessage modes => Modes(client, modes),
            ChooseCardsPromptMessage cards => Cards(client, cards),
            SearchLibraryPromptMessage search => Search(client, search),
            DiscardPromptMessage discard => Discard(client, discard),
            ChoosePlayerPromptMessage player => Player(client, player),
            ChooseColorsPromptMessage colors => Colors(client, colors),
            NameCardPromptMessage name => Name(client, name),
            ChooseReplacementPromptMessage replacement => Replacement(client, replacement),
            OrderTriggersPromptMessage order => Order(client, order),
            DistributeCardsPromptMessage distribute => Distribute(client, distribute),
            MayPromptMessage may => May(client, may),

            // Playing first is the stronger choice in nearly every game, and not worth paying a model to say so.
            ChooseStartingPlayerPromptMessage starting => new PlayerChosenAnswer { Player = starting.Player },
            MulliganPromptMessage mulligan => Mulligan(mulligan),

            // A prompt from a newer server than this bot. The prompt's own default is always legal, which is what
            // keeps a game moving past a question this seat has never heard of.
            _ => prompt.Default(),
        };
    }

    // --- Priority ---------------------------------------------------------------------------------------------------

    /// <summary>
    /// Everything this seat may do right now, with passing among the choices.
    /// <para>
    /// Passing is one of the choices rather than what happens when nothing is picked (CR 117.3d): holding a
    /// counterspell, leaving a blocker untapped and letting an end step arrive with mana up are all decisions, and a
    /// seat that acted whenever it could would never make any of them.
    /// </para>
    /// <para>
    /// With nothing to take, nobody is asked. That is the single biggest saving here: most priority in a game of
    /// Magic is an empty hand or an uncastable one, and none of it is worth paying a model to say "pass" about.
    /// </para>
    /// </summary>
    private AnswerPayload Priority(GameClient client, PriorityPromptMessage prompt)
    {
        _targetX = Unchosen;
        Step(client);

        if (_asked >= MostQuestions)
        {
            // This step has asked more than any real step has to. Something is being offered over and over, and
            // passing is both the way out and always legal.
            return new PassAnswer();
        }

        if (Waiting(client))
        {
            return new PassAnswer();
        }

        List<ActionOptionMessage> offered =
            [.. prompt.Options.Where(option => Takeable(option) && !_abandoned.Contains(option.Source))];

        if (offered.Count == 0)
        {
            return new PassAnswer();
        }

        // No stops offered while a land is still unplayed. Holding off is a decision about spells; the land for the
        // turn is a free once-a-turn resource, and a seat that gave one up by saying "nothing more this turn" would
        // be throwing away a land drop for a reason that had nothing to do with land.
        List<Stop> stops = offered.Exists(option => option.Kind == ActionKindView.PlayLand) ? [] : Stops(client);

        List<string> choices = [.. offered.Select(option => Action(client, option))];
        choices.Add("Pass for now, keeping your cards and your untapped lands. You will be asked again the next time "
            + "you could act this turn.");
        choices.AddRange(stops.Select(stop => stop.Text));

        int pick = One(new Question
        {
            Heading = "You have priority. What do you do?",
            Note = stops.Count > 0
                ? "Any of the waiting choices ends early if something happens — a spell cast, something entering or "
                    + "leaving the battlefield, damage, a life total moving — so waiting never costs you the chance "
                    + "to respond. Prefer one of them whenever you have nothing you want to do yet: being asked over "
                    + "and over in a quiet turn is itself a signal to your opponent that you are holding something."
                : string.Empty,
            Choices = choices,
        });

        if (pick > offered.Count)
        {
            _until = stops[pick - offered.Count - 1].At;
            return new PassAnswer();
        }

        if (pick == offered.Count)
        {
            return new PassAnswer();
        }

        ActionOptionMessage option = offered[pick];

        // The X this spell will be cast for, chosen here where the server has said how large it could be, and paid up
        // to over the prompts that follow.
        _targetX = Unchosen;

        return new TakeActionAnswer
        {
            Kind = option.Kind,
            Source = option.Source,
            AbilityIndex = option.AbilityIndex,
            Tap = option.Tap,
            Targets = Aim(
                client,
                option.TargetCandidates,
                option.TargetsRequired,
                least: option.TargetsOptional ? 0 : option.TargetsRequired,
                what: $"You are casting {Spell(client, option)}."),
        };
    }

    /// <summary>
    /// Whether an offered action is one to put in front of the chooser.
    /// <para>
    /// Two are left out, and neither is taste. A mana ability is never activated at priority: mana made there empties
    /// when the step ends (CR 500.4) and the land stays tapped until the untap step, so a seat that tapped for mana it
    /// had not been asked for would spend the turn on nothing — mana is made while paying a cost, where every prompt
    /// that needs it offers the same abilities. And an action the server says cannot be paid for is not offered,
    /// because announcing it, failing to pay and giving it up (CR 733.1) puts the seat back exactly where it was with
    /// the same action on the menu — a loop a model would pay for once per lap.
    /// </para>
    /// </summary>
    private static bool Takeable(ActionOptionMessage option) => option.Kind switch
    {
        // A land costs nothing, so nothing is said about affording it.
        ActionKindView.PlayLand => true,

        ActionKindView.ActivateAbility => option.Affordable && !MakesMana(option),
        ActionKindView.CastSpell => option.Affordable && (!option.HasX || option.MaxX >= 0),
        ActionKindView.TurnFaceUp or ActionKindView.TakeCompanion or ActionKindView.Plot or ActionKindView.Suspend or ActionKindView.Foretell or ActionKindView.UnlockDoor => option.Affordable,
        _ => false,
    };

    /// <summary>Whether an ability adds mana, either of stated types or of colors its answer would name (CR 106.1b).</summary>
    private static bool MakesMana(ActionOptionMessage option) =>
        option.Produces.Count > 0 || option.ProducesOfAnyOneColor > 0 || option.ProducesOfAnyColorEach > 0;

    /// <summary>One offered action, written so it can be chosen without reading anything else.</summary>
    private static string Action(GameClient client, ActionOptionMessage option)
    {
        ObjectView? card = client.Object(option.Source);
        string name = Spell(client, option);

        return option.Kind switch
        {
            ActionKindView.PlayLand =>
                $"Play {name} as your land for this turn."
                + (card?.OracleText is { Length: > 0 } land ? $" It reads: \"{land.ReplaceLineEndings(" / ")}\"" : string.Empty),

            ActionKindView.CastSpell =>
                $"Cast {name}"
                + (card?.ManaCost is { Length: > 0 } cost ? $" for {cost}" : string.Empty)
                + (option.AbilityIndex > 0 ? " by an alternative cost" : string.Empty)
                + (option.HasX ? $" (you can pay for X up to {option.MaxX})" : string.Empty)
                + (card?.OracleText is { Length: > 0 } spell ? $". It reads: \"{spell.ReplaceLineEndings(" / ")}\"" : ".")
                + Costs(option),

            ActionKindView.ActivateAbility =>
                $"Activate {name}: \"{option.Description}\"" + Costs(option),

            ActionKindView.TurnFaceUp => $"Turn {name} face up for its mana cost.",
            ActionKindView.TakeCompanion => $"Pay {{3}} to put your companion {name} into your hand.",
            ActionKindView.Plot => $"Plot {name}: exile it now and cast it for free on a later turn.",
            ActionKindView.Suspend =>
                $"Suspend {name}: exile it {option.Description}; one comes off at each of your upkeeps, and when the last does you may cast it for free.",
            ActionKindView.Foretell =>
                $"Foretell {name}: exile it face down {option.Description}, then cast it from exile for its foretell cost on a later turn.",
            ActionKindView.UnlockDoor =>
                $"Unlock a door of {name}, a Room: {option.Description}. That door's text then counts, and its \"when you unlock this door\" ability triggers.",
            _ => $"{option.Kind}: {name}",
        };
    }

    /// <summary>What an action costs besides mana, where the server says it costs something worth weighing.</summary>
    private static string Costs(ActionOptionMessage option)
    {
        List<string> parts = [];

        if (option.LifeCost > 0)
        {
            parts.Add(string.Create(CultureInfo.InvariantCulture, $"costs you {option.LifeCost} life"));
        }

        if (option.DamageToController > 0)
        {
            parts.Add(string.Create(CultureInfo.InvariantCulture, $"deals you {option.DamageToController} damage"));
        }

        if (option.LifeGained > 0)
        {
            parts.Add(string.Create(CultureInfo.InvariantCulture, $"gains you {option.LifeGained} life"));
        }

        return parts.Count == 0 ? string.Empty : $" (this {string.Join(" and ", parts)})";
    }

    /// <summary>What an action would put on the stack, which is not always the card's own name (CR 715.3).</summary>
    private static string Spell(GameClient client, ActionOptionMessage option) =>
        option.SpellName.Length > 0
            ? $"{option.SpellName} (from {GameText.Named(client, option.Source)})"
            : GameText.Named(client, option.Source);

    /// <summary>How large X is, asked once as the spell is announced and paid up to from there.</summary>
    private int X(GameClient client, PayCostsPromptMessage prompt)
    {
        int reach = Math.Max(0, prompt.MostAffordable);
        int most = Math.Min(reach, MostChoices - 1);
        string spell = GameText.Named(client, prompt.Spell);

        return One(new Question
        {
            Heading = $"You are casting {spell}. How large is X?",
            Note = prompt.XPaidInLife
                ? "X is paid in life. You can pay up to X = " + reach.ToString(CultureInfo.InvariantCulture)
                    + " and still be alive; every point of X is a point of life gone."
                : "Your mana will stretch to X = " + reach.ToString(CultureInfo.InvariantCulture)
                    + ". Everything you spend on X is mana you will not have for anything else this turn.",
            Choices = [.. Enumerable.Range(0, most + 1).Select(x => $"X = {x}")],
        });
    }

    // --- Paying ------------------------------------------------------------------------------------------------------

    /// <summary>
    /// Pays for what has been announced.
    /// <para>
    /// Which land to tap is arithmetic and not judgement, so it is left to <see cref="AutoPlay"/>: the server says
    /// what each source produces and what is still owed, and there is nothing to choose between two lands that both
    /// help. What is a decision — how large X is, whether to kick — is asked once and then paid up to.
    /// </para>
    /// </summary>
    private AnswerPayload Pay(GameClient client, PayCostsPromptMessage prompt)
    {
        AnswerPayload answer = Paying(client, prompt);

        // A spell given up on goes on the list, so the same offer is not taken again this step. See _abandoned.
        if (answer is TakeActionAnswer { Kind: ActionKindView.CancelCasting })
        {
            _abandoned.Add(prompt.Spell);
        }

        return answer;
    }

    private AnswerPayload Paying(GameClient client, PayCostsPromptMessage prompt)
    {
        // CR 601.2b: X, or how many times to kick, is announced before the targets and the payment.
        if (prompt.AnnouncesXOrKicks)
        {
            if (_targetX is Unchosen)
            {
                // Decided here, where the server says how far the mana — or, for an X paid in life, the life that
                // leaves this seat alive — reaches (CR 601.2b), and decided once.
                _targetX = prompt.Kicker.Length > 0 ? Kicker(client, prompt) : X(client, prompt);
            }

            return AutoPlay.Announce(prompt, _targetX);
        }

        if (!prompt.HasX)
        {
            return AutoPlay.Pay(prompt);
        }

        bool owing = AutoPlay.Owing(prompt);

        if (_targetX is Unchosen)
        {
            // Nothing offered a ceiling when this was announced: a kicker or a buyback (CR 702.33, 702.27) rather than
            // {X} in a mana cost. Decided here, and decided once — a value asked for again after failing to pay for it
            // is how a payment becomes a loop.
            _targetX = prompt.X + (prompt.Kicker.Length > 0 ? Kicker(client, prompt) : 0);
        }

        if (prompt.X < _targetX && Offered(prompt, ActionKindView.IncreaseX) is { } raise)
        {
            return AutoPlay.Answer(raise);
        }

        if (owing)
        {
            if (AutoPlay.Produce(
                    prompt.Options, prompt.RemainingColored, prompt.RemainingGeneric, prompt.RemainingHybrid, prompt.RemainingColorless)
                is { } produce)
            {
                return produce;
            }

            // Pool mana that would pay something still owed. Assigning it can raise X on its own, which is why the
            // chosen value is an aim rather than a cap.
            ActionOptionMessage? assign = prompt.Options.FirstOrDefault(option =>
                option.Kind == ActionKindView.AssignMana && AutoPlay.Helps(prompt, option));

            if (assign is not null)
            {
                return AutoPlay.Assign(assign);
            }

            // Out of reach at this X, so down one — and the value aimed at comes down with it. Leaving the aim where
            // it was would raise X again next time, fail again, and lower it again for ever.
            if (Take(prompt, ActionKindView.DecreaseX) is { } lower)
            {
                _targetX = prompt.X - 1;
                return lower;
            }

            return Take(prompt, ActionKindView.CancelCasting)!;
        }

        return Take(prompt, ActionKindView.ConfirmCasting) ?? Take(prompt, ActionKindView.CancelCasting)!;
    }

    /// <summary>Whether to pay an optional additional cost, and for multikicker how many times (CR 702.33).</summary>
    private int Kicker(GameClient client, PayCostsPromptMessage prompt)
    {
        string what = prompt.Buyback ? "buy back" : "kick";
        string spell = GameText.Named(client, prompt.Spell);

        if (!prompt.Multikicker)
        {
            return Yes($"Pay {prompt.Kicker} as well, to {what} {spell}?") ? 1 : 0;
        }

        return One(new Question
        {
            Heading = $"How many times do you want to kick {spell}? Each kick costs another {prompt.Kicker}.",
            Choices = [.. Enumerable.Range(0, 4).Select(times => $"{times} time(s)")],
        });
    }

    /// <summary>
    /// Whether to pay an optional cost that stops something happening (CR 118.12), such as Mana Leak's {3}.
    /// <para>
    /// Decided once per spell asking, not once per prompt: paying takes several answers, and a question asked again
    /// at each of them would tap lands for a cost it then declined.
    /// </para>
    /// </summary>
    private AnswerPayload Unless(GameClient client, UnlessPaysPromptMessage prompt)
    {
        if (prompt.Source != _unless)
        {
            _unless = prompt.Source;

            string source = GameText.Named(client, prompt.Source);
            string subject = GameText.Named(client, prompt.Subject);

            _paying = Yes(prompt.Verb.Length > 0
                ? $"{source} lets you pay {prompt.Total} to {prompt.Verb} {subject}. Pay it?"
                : $"{source} will affect {subject} unless you pay {prompt.Total}. Pay it?");
        }

        return _paying
            ? AutoPlay.PayUnless(prompt)
            : AutoPlay.Answer(prompt.Options.First(option => option.Kind == ActionKindView.DeclineCost));
    }

    // --- Combat ------------------------------------------------------------------------------------------------------

    /// <summary>
    /// Declares attackers (CR 508.1).
    /// <para>
    /// With one player to attack and no planeswalkers on the other side, this is one question: which of your
    /// creatures attack. Once there is more than one thing to attack, each creature gets its own question, because
    /// the choice is no longer whether it attacks but what it attacks.
    /// </para>
    /// <para>
    /// A creature that must attack (CR 508.1d) is declared without being asked about, and one whose requirement names
    /// which seats it may attack — a goaded creature (CR 701.15b) — is only offered those.
    /// </para>
    /// </summary>
    private AnswerPayload Attack(GameClient client, DeclareAttackersPromptMessage prompt)
    {
        var answer = new AttackersDeclaredAnswer();

        if (prompt.Eligible.Count == 0
            || (prompt.AttackablePlayers.Count == 0 && prompt.AttackablePlaneswalkers.Count == 0 && prompt.AttackableBattles.Count == 0))
        {
            return answer;
        }

        List<ViewHandle> forced = [.. prompt.Requirements.Select(requirement => requirement.Attacker)];

        if (prompt.AttackablePlayers.Count == 1 && prompt.AttackablePlaneswalkers.Count == 0 && prompt.AttackableBattles.Count == 0
            && prompt.Forbidden.Count == 0)
        {
            int defender = prompt.AttackablePlayers[0];

            foreach (ViewHandle attacker in prompt.Eligible.Where(forced.Contains))
            {
                answer.Attacks.Add(new AttackMessage { Attacker = attacker, TargetPlayer = defender });
            }

            List<ViewHandle> free = [.. prompt.Eligible.Where(attacker => !forced.Contains(attacker))];

            if (free.Count > 0)
            {
                IReadOnlyList<int> picks = Many(new Question
                {
                    Heading = $"Declare attackers. Which of your creatures attack {GameText.Player(client, defender)}?",
                    Note = Forced(client, prompt, forced) + Defenders(client, defender),
                    Choices = [.. free.Select(attacker => GameText.Creature(client, attacker))],
                    Picks = free.Count,
                    UpTo = true,
                });

                foreach (int pick in picks)
                {
                    answer.Attacks.Add(new AttackMessage { Attacker = free[pick], TargetPlayer = defender });
                }
            }

            return answer;
        }

        foreach (ViewHandle attacker in prompt.Eligible)
        {
            AttackRequirementMessage? must = prompt.Requirements.Find(requirement => requirement.Attacker == attacker);
            List<AttackMessage?> attacks = [];
            List<string> choices = [];

            IEnumerable<int> players = must is { Players.Count: > 0 } ? must.Players : prompt.AttackablePlayers;

            foreach (int seat in players.Where(seat => !prompt.IsForbidden(new AttackMessage { Attacker = attacker, TargetPlayer = seat })))
            {
                attacks.Add(new AttackMessage { Attacker = attacker, TargetPlayer = seat });
                choices.Add($"Attack {GameText.Player(client, seat)}.");
            }

            if (must is not { Players.Count: > 0 })
            {
                foreach (ViewHandle walker in prompt.AttackablePlaneswalkers
                    .Where(walker => !prompt.IsForbidden(new AttackMessage { Attacker = attacker, TargetPermanent = walker })))
                {
                    attacks.Add(new AttackMessage { Attacker = attacker, TargetPermanent = walker });
                    choices.Add($"Attack {GameText.Named(client, walker)}.");
                }

                foreach (ViewHandle battle in prompt.AttackableBattles
                    .Where(battle => !prompt.IsForbidden(new AttackMessage { Attacker = attacker, TargetPermanent = battle })))
                {
                    attacks.Add(new AttackMessage { Attacker = attacker, TargetPermanent = battle });
                    choices.Add($"Attack the battle {GameText.Named(client, battle)}.");
                }
            }

            if (must is null)
            {
                attacks.Add(null);
                choices.Add("Do not attack with it. Keep it back to block.");
            }

            if (choices.Count == 0)
            {
                continue;
            }

            int pick = One(new Question
            {
                Heading = $"Declare attackers. What does {GameText.Creature(client, attacker)} do?",
                Note = Declared(client, answer.Attacks) + (must is not null ? "\nThis creature must attack." : string.Empty),
                Choices = choices,
            });

            if (attacks[pick] is { } declared)
            {
                answer.Attacks.Add(declared);
            }
        }

        return answer;
    }

    /// <summary>
    /// Declares blockers (CR 509.1).
    /// <para>
    /// One attacker means one question — which of your creatures block it — and more than one means a question per
    /// creature, because then the choice is which attacker it stands in front of. One attacker per blocker either
    /// way: blocking two takes an ability, and the prompt offers pairs rather than a count.
    /// </para>
    /// <para>
    /// Blocks short of a stated minimum are taken back rather than sent. Menace and "except by three or more" are
    /// per attacker (CR 509.1b), and a declaration one blocker short of one is refused whole, which would throw away
    /// every other block with it. The minimums are in the question too, so the chooser can meet them on purpose.
    /// </para>
    /// </summary>
    private AnswerPayload Block(GameClient client, DeclareBlockersPromptMessage prompt)
    {
        var answer = new BlockersDeclaredAnswer();

        if (prompt.Eligible.Count == 0 || prompt.Attackers.Count == 0)
        {
            return answer;
        }

        if (prompt.Attackers.Count == 1)
        {
            ViewHandle attacker = prompt.Attackers[0];
            List<ViewHandle> able = [.. prompt.Allowed.Where(pair => pair.Attacker == attacker).Select(pair => pair.Blocker)];

            if (able.Count > 0)
            {
                IReadOnlyList<int> picks = Many(new Question
                {
                    Heading = $"You are being attacked by {GameText.Creature(client, attacker)}. "
                        + "Which of your creatures block it?",
                    Note = Minimums(client, prompt),
                    Choices = [.. able.Select(blocker => GameText.Creature(client, blocker))],
                    Picks = able.Count,
                    UpTo = true,
                });

                foreach (int pick in picks)
                {
                    answer.Blocks.Add(new BlockMessage { Blocker = able[pick], Attacker = attacker });
                }
            }

            return Legal(prompt, answer);
        }

        foreach (ViewHandle blocker in prompt.Eligible)
        {
            List<ViewHandle> attackers =
                [.. prompt.Allowed.Where(pair => pair.Blocker == blocker).Select(pair => pair.Attacker)];

            if (attackers.Count == 0)
            {
                continue;
            }

            List<string> choices = [.. attackers.Select(attacker => $"Block {GameText.Creature(client, attacker)}.")];
            choices.Add("Do not block with it.");

            int pick = One(new Question
            {
                Heading = $"Declare blockers. What does {GameText.Creature(client, blocker)} do?",
                Note = Minimums(client, prompt) + Blocking(client, answer.Blocks),
                Choices = choices,
            });

            if (pick < attackers.Count)
            {
                answer.Blocks.Add(new BlockMessage { Blocker = blocker, Attacker = attackers[pick] });
            }
        }

        return Legal(prompt, answer);
    }

    /// <summary>Drops any block that gives an attacker fewer creatures than it may be blocked by (CR 509.1b).</summary>
    private static BlockersDeclaredAnswer Legal(DeclareBlockersPromptMessage prompt, BlockersDeclaredAnswer answer)
    {
        for (int i = 0; i < prompt.Attackers.Count; i++)
        {
            ViewHandle attacker = prompt.Attackers[i];
            int minimum = i < prompt.MinimumBlockers.Count ? prompt.MinimumBlockers[i] : 1;
            int blocking = answer.Blocks.Count(block => block.Attacker == attacker);

            if (blocking > 0 && blocking < minimum)
            {
                answer.Blocks.RemoveAll(block => block.Attacker == attacker);
            }
        }

        return answer;
    }

    /// <summary>
    /// Divides each creature's combat damage among the things it may hit (CR 510.1c-d, 702.19b).
    /// <para>
    /// The legal divisions are a short list, not a free split: damage goes in the attacker's chosen order, and
    /// nothing may reach a later blocker — or, with trample, the player — until every earlier one has lethal. So the
    /// answers are enumerated and offered as they are, and the real decision shows up plainly: give the first blocker
    /// exactly lethal and trample the rest through, or bury it.
    /// </para>
    /// </summary>
    private AnswerPayload Damage(GameClient client, AssignCombatDamagePromptMessage prompt)
    {
        var answer = new CombatDamageAssignedAnswer();

        foreach (DamageDivisionMessage division in prompt.Divisions)
        {
            List<(string Text, List<DamageShareMessage> Shares)> ways = Divisions(client, division);

            if (ways.Count == 0)
            {
                // Nothing enumerated: fall back to the engine's own division of this creature's damage, which is
                // always legal.
                foreach (DamageShareMessage share in ((CombatDamageAssignedAnswer)prompt.Default()).Shares
                    .Where(share => share.Source == division.Source))
                {
                    answer.Shares.Add(share);
                }

                continue;
            }

            int pick = One(new Question
            {
                Heading = $"{GameText.Creature(client, division.Source)} assigns its {division.Damage} combat damage. "
                    + "How do you divide it?",
                Note = "It has to give each creature lethal damage before any goes to the next one"
                    + (division.MayAssignToDefender ? ", and only then may the rest trample through." : "."),
                Choices = [.. ways.Select(way => way.Text)],
            });

            answer.Shares.AddRange(ways[pick].Shares);
        }

        return answer;
    }

    /// <summary>
    /// Every way one creature may divide its damage: lethal to the first so many in order, and the rest onto the next
    /// one along — or, once everything has lethal, onto the player or planeswalker where trample allows it.
    /// </summary>
    private static List<(string Text, List<DamageShareMessage> Shares)> Divisions(
        GameClient client,
        DamageDivisionMessage division)
    {
        List<(string, List<DamageShareMessage>)> ways = [];
        int creatures = division.Creatures.Count;

        if (creatures == 0 || division.Damage <= 0)
        {
            return ways;
        }

        for (int taken = 0; taken <= creatures; taken++)
        {
            List<DamageShareMessage> shares = [];
            List<string> words = [];
            int spent = 0;

            for (int i = 0; i < taken; i++)
            {
                int lethal = i < division.Lethal.Count ? division.Lethal[i] : 1;
                spent += lethal;
                shares.Add(new DamageShareMessage { Source = division.Source, Creature = division.Creatures[i], Amount = lethal });
                words.Add($"{lethal} to {GameText.Named(client, division.Creatures[i])} (lethal)");
            }

            int left = division.Damage - spent;

            if (left < 0)
            {
                break;
            }

            if (taken < creatures)
            {
                if (left == 0)
                {
                    continue;
                }

                shares.Add(new DamageShareMessage
                {
                    Source = division.Source,
                    Creature = division.Creatures[taken],
                    Amount = left,
                });

                words.Add($"{left} to {GameText.Named(client, division.Creatures[taken])}");
            }
            else if (left > 0)
            {
                if (division.MayAssignToDefender)
                {
                    shares.Add(new DamageShareMessage { Source = division.Source, Creature = ViewHandle.None, Amount = left });
                    words.Add($"{left} trampling through to "
                        + (division.Defender > 0
                            ? GameText.Player(client, division.Defender)
                            : GameText.Named(client, division.DefenderPermanent)));
                }
                else
                {
                    // No trample, so the excess has to land on a creature. The last one along is as good as any: every
                    // earlier one already has lethal.
                    DamageShareMessage last = shares[^1];
                    last.Amount += left;
                    words[^1] = $"{last.Amount} to {GameText.Named(client, last.Creature)}";
                }
            }

            if (shares.Count > 0)
            {
                ways.Add((string.Join(", ", words), shares));
            }
        }

        return ways;
    }

    // --- Choosing ----------------------------------------------------------------------------------------------------

    private AnswerPayload Target(GameClient client, ChooseTargetPromptMessage prompt)
    {
        if (prompt.Candidates.Count == 0)
        {
            return new TargetsChosenAnswer();
        }

        string what = prompt.Targeted ? "targets" : "chooses";

        return new TargetsChosenAnswer
        {
            Targets = Aim(
                client,
                prompt.Candidates,
                prompt.Count,
                least: prompt.Optional ? 0 : prompt.Minimum > 0 ? prompt.Minimum : prompt.Count,
                what: $"{GameText.Named(client, prompt.Ability)} {what}: \"{prompt.Text}\"."),
        };
    }

    /// <summary>
    /// Chooses the targets of a divided slot and how much each gets (CR 601.2d).
    /// <para>
    /// Two questions rather than one free-form answer: which things it hits, and then which of the ways of splitting
    /// the total among them. Every chosen target has to get at least one (CR 601.2d), so the splits are the
    /// compositions of the total into that many positive parts, which for the totals a real card divides is a short
    /// list.
    /// </para>
    /// </summary>
    private AnswerPayload Divided(GameClient client, ChooseDividedTargetsPromptMessage prompt)
    {
        if (prompt.Total <= 0 || prompt.Candidates.Count == 0)
        {
            return prompt.Default();
        }

        int most = Math.Min(
            prompt.Total,
            prompt.MaxTargets > 0 ? Math.Min(prompt.MaxTargets, prompt.Candidates.Count) : prompt.Candidates.Count);

        List<TargetMessage> chosen = Aim(
            client,
            prompt.Candidates,
            most,
            least: 1,
            what: $"{GameText.Named(client, prompt.Ability)} divides {prompt.Total} among its targets: \"{prompt.Text}\".");

        if (chosen.Count == 0)
        {
            return prompt.Default();
        }

        List<int[]> splits = Splits(prompt.Total, chosen.Count);

        int[] amounts = splits.Count switch
        {
            0 => Even(prompt.Total, chosen.Count),
            1 => splits[0],
            _ => splits[One(new Question
            {
                Heading = $"How do you divide the {prompt.Total} among "
                    + string.Join(", ", chosen.Select(target => GameText.Aimed(client, target))) + "?",
                Choices = [.. splits.Select(split => string.Join(
                    ", ",
                    split.Select((amount, i) => $"{amount} to {GameText.Aimed(client, chosen[i])}")))],
            })],
        };

        return new TargetsDividedAnswer
        {
            Shares = [.. chosen.Select((target, i) => new TargetShareMessage { Target = target, Amount = amounts[i] })],
        };
    }

    private AnswerPayload Modes(GameClient client, ChooseModesPromptMessage prompt)
    {
        if (prompt.Eligible.Count == 0)
        {
            return prompt.Default();
        }

        string what = prompt.Heading.Length > 0
            ? prompt.Heading
            : prompt.AdditionalCost
                ? $"Casting {GameText.Named(client, prompt.Source)} has an additional cost. Which do you pay?"
                : $"Casting {GameText.Named(client, prompt.Source)}. Which mode do you choose?";

        List<int> left = [.. prompt.Eligible];
        List<int> chosen = [];

        for (int slot = 0; slot < prompt.Most && left.Count > 0; slot++)
        {
            List<string> choices = [.. left.Select(mode => prompt.Modes[mode])];
            bool mayStop = slot >= prompt.Count;

            if (mayStop)
            {
                choices.Add("Choose no more than that.");
            }

            int pick = One(new Question
            {
                Heading = prompt.Most > 1 ? $"{what} (choice {slot + 1})" : what,
                Note = chosen.Count > 0
                    ? "Already chosen: " + string.Join("; ", chosen.Select(mode => prompt.Modes[mode]))
                    : string.Empty,
                Choices = choices,
            });

            if (mayStop && pick == left.Count)
            {
                break;
            }

            chosen.Add(left[pick]);

            if (!prompt.MayRepeat)
            {
                left.RemoveAt(pick);
            }
        }

        return chosen.Count >= prompt.Count ? new ModesChosenAnswer { Modes = chosen } : prompt.Default();
    }

    private AnswerPayload Cards(GameClient client, ChooseCardsPromptMessage prompt)
    {
        if (prompt.Eligible.Count == 0 || prompt.Count == 0)
        {
            return prompt.Default();
        }

        string what = Purpose(client, prompt);

        if (prompt.MinimumPower > 0)
        {
            return new CardsChosenAnswer { Cards = ToPower(client, prompt, what) };
        }

        if (prompt.MinimumManaValue > 0)
        {
            return new CardsChosenAnswer { Cards = ToManaValue(client, prompt, what) };
        }

        int count = Math.Min(prompt.Count, prompt.Eligible.Count);

        IReadOnlyList<int> picks = Many(new Question
        {
            Heading = what,
            Choices = [.. prompt.Eligible.Select(card => GameText.Named(client, card))],
            Picks = count,
            UpTo = prompt.UpTo,
        });

        return new CardsChosenAnswer { Cards = [.. picks.Select(pick => prompt.Eligible[pick])] };
    }

    /// <summary>A choice that has to add up to a total power, as saddle's does (CR 702.171a): one creature at a time.</summary>
    private List<ViewHandle> ToPower(GameClient client, ChooseCardsPromptMessage prompt, string what)
    {
        List<int> left = [.. Enumerable.Range(0, prompt.Eligible.Count)];
        List<ViewHandle> taken = [];
        int power = 0;

        while (power < prompt.MinimumPower && left.Count > 0)
        {
            int pick = One(new Question
            {
                Heading = what,
                Note = string.Create(
                    CultureInfo.InvariantCulture,
                    $"You need {prompt.MinimumPower} total power and have {power} so far."),
                Choices = [.. left.Select(index => GameText.Creature(client, prompt.Eligible[index]))],
            });

            int chosen = left[pick];
            taken.Add(prompt.Eligible[chosen]);
            power += chosen < prompt.Powers.Count ? prompt.Powers[chosen] : 0;
            left.RemoveAt(pick);
        }

        return power >= prompt.MinimumPower ? taken : [.. ((CardsChosenAnswer)prompt.Default()).Cards];
    }

    /// <summary>A choice that has to add up to a total mana value, as collect evidence's does (CR 701.59a): one card at a time.</summary>
    private List<ViewHandle> ToManaValue(GameClient client, ChooseCardsPromptMessage prompt, string what)
    {
        List<int> left = [.. Enumerable.Range(0, prompt.Eligible.Count)];
        List<ViewHandle> taken = [];
        int total = 0;

        while (total < prompt.MinimumManaValue && left.Count > 0)
        {
            int pick = One(new Question
            {
                Heading = what,
                Note = string.Create(
                    CultureInfo.InvariantCulture,
                    $"You need total mana value {prompt.MinimumManaValue} and have {total} so far."),
                Choices = [.. left.Select(index => GameText.Named(client, prompt.Eligible[index]))],
            });

            int chosen = left[pick];
            taken.Add(prompt.Eligible[chosen]);
            total += chosen < prompt.ManaValues.Count ? prompt.ManaValues[chosen] : 0;
            left.RemoveAt(pick);
        }

        return total >= prompt.MinimumManaValue ? taken : [.. ((CardsChosenAnswer)prompt.Default()).Cards];
    }

    /// <summary>What a choice of cards is for, in the words the card would use.</summary>
    private static string Purpose(GameClient client, ChooseCardsPromptMessage prompt)
    {
        string source = GameText.Named(client, prompt.Source);
        string whose = GameText.Whose(client, prompt.Owner);
        string kind = prompt.Kind.Length > 0 ? $" {prompt.Kind}" : string.Empty;
        string many = prompt.UpTo ? $"up to {prompt.Count}" : prompt.Count.ToString(CultureInfo.InvariantCulture);

        return prompt.Purpose switch
        {
            CardChoicePurposeView.Discard => $"{source}: discard {many}{kind} card(s) from {whose} hand.",
            CardChoicePurposeView.Sacrifice => $"{source}: sacrifice {many}{kind} permanent(s).",
            CardChoicePurposeView.Destroy => $"{source}: choose {many}{kind} permanent(s) to destroy.",
            CardChoicePurposeView.Exile => $"{source}: exile {many}{kind} card(s) from {whose} hand.",
            CardChoicePurposeView.PutOnTop => $"{source}: put {many} card(s) on top of the library, last chosen on top.",
            CardChoicePurposeView.PutOntoBattlefield => $"{source}: put {many}{kind} card(s) onto the battlefield.",
            CardChoicePurposeView.PutIntoHand => $"{source}: put {many}{kind} card(s) into your hand.",
            CardChoicePurposeView.PutIntoGraveyard => $"{source}: put {many}{kind} card(s) into the graveyard.",
            CardChoicePurposeView.ReturnToHand => $"{source}: return {many}{kind} permanent(s) to their owner's hand.",
            CardChoicePurposeView.Untap => $"{source}: untap {many}{kind} permanent(s).",
            CardChoicePurposeView.Attach => $"{source}: attach {many}{kind} permanent(s) you control to the creature.",
            CardChoicePurposeView.Encode => $"{source}: choose up to one creature you control to exile it encoded on (cipher).",
            CardChoicePurposeView.Behold => $"{source}: choose {many}{(kind.Length > 0 ? kind : " matching")} permanent(s) you control or card(s) in your hand to behold as a cost. Nothing is lost: a permanent is only chosen and a card is only revealed (to your opponent), so prefer a permanent.",
            CardChoicePurposeView.Blight => $"{source}: choose a creature you control to put the -1/-1 counter(s) of the cost on. It shrinks, and dies if its toughness reaches 0, so choose the creature you can best spare, not your best.",
            CardChoicePurposeView.KeepLegend => "You control more than one legend with the same name. Choose the one to keep; the rest go to the graveyard.",
            CardChoicePurposeView.Keep => $"{source}: choose the{kind} one to keep. The rest are sacrificed.",
            CardChoicePurposeView.Cast => $"{source}: choose {many}{kind} card(s) to cast from the graveyard.",
            CardChoicePurposeView.Play => $"{source}: choose {many}{kind} card(s) you may play this turn without paying their cost.",
            CardChoicePurposeView.Vote => $"{source}: cast your vote.",
            CardChoicePurposeView.TapToPay => $"{source}: tap creatures to pay its cost.",
            CardChoicePurposeView.ExileToPay => $"{source}: exile {many}{kind} card(s) from your hand to pay for it.",
            CardChoicePurposeView.OrderSimultaneous => $"{source}: choose which happens first.",
            CardChoicePurposeView.PutOnBottom => $"You kept a hand after mulliganing: choose {many} card(s) to put on the bottom of your library.",
            _ => $"{source}: choose {many}{kind} card(s).",
        };
    }

    private AnswerPayload Search(GameClient client, SearchLibraryPromptMessage prompt)
    {
        if (prompt.Eligible.Count == 0)
        {
            return new CardsChosenAnswer();
        }

        int count = Math.Min(prompt.Count, prompt.Eligible.Count);

        if (prompt.Eligible.Count > MostChoices)
        {
            // A library too long to offer as a list. Taking what the prompt itself would is better than offering the
            // first sixty cards and calling it a search.
            return prompt.Default();
        }

        IReadOnlyList<int> picks = Many(new Question
        {
            Heading = $"You are searching your library for {(count == 1 ? "a card" : $"{count} cards")}, "
                + $"to go to {Destination(prompt.Destination)}. What do you take?",
            Note = prompt.Optional ? "You may find nothing, which shuffles the library and takes nothing." : string.Empty,
            Choices = [.. prompt.Eligible.Select(card => GameText.Named(client, card))],
            Picks = count,
            UpTo = prompt.Optional,
        });

        return new CardsChosenAnswer { Cards = [.. picks.Select(pick => prompt.Eligible[pick])] };
    }

    private static string Destination(SearchDestinationView destination) => destination switch
    {
        SearchDestinationView.Hand => "your hand",
        SearchDestinationView.TopOfLibrary => "the top of your library",
        SearchDestinationView.Graveyard => "your graveyard",
        SearchDestinationView.Exile => "exile",
        SearchDestinationView.ExileFaceDown => "exile, face down",
        SearchDestinationView.Cast => "the stack, cast by you as the effect resolves",
        _ => "the battlefield",
    };

    private AnswerPayload Discard(GameClient client, DiscardPromptMessage prompt)
    {
        if (prompt.Hand.Count == 0 || prompt.Count <= 0)
        {
            return prompt.Default();
        }

        IReadOnlyList<int> picks = Many(new Question
        {
            Heading = string.Create(
                CultureInfo.InvariantCulture,
                $"Your hand is over the limit of {prompt.MaximumHandSize}. Discard {prompt.Count} card(s)."),
            Choices = [.. prompt.Hand.Select(card => GameText.Named(client, card))],
            Picks = Math.Min(prompt.Count, prompt.Hand.Count),
        });

        return new CardsChosenAnswer { Cards = [.. picks.Select(pick => prompt.Hand[pick])] };
    }

    private AnswerPayload Player(GameClient client, ChoosePlayerPromptMessage prompt)
    {
        if (prompt.Candidates.Count == 0)
        {
            return prompt.Default();
        }

        int pick = One(new Question
        {
            Heading = $"{GameText.Named(client, prompt.Source)}: choose a player"
                + (prompt.Verb.Length > 0
                    ? $" to {prompt.Verb.ToLowerInvariant()}"
                        + (prompt.Amount > 0 ? " " + prompt.Amount.ToString(CultureInfo.InvariantCulture) : string.Empty)
                    : string.Empty)
                + ".",
            Choices = [.. prompt.Candidates.Select(seat => GameText.Player(client, seat))],
        });

        return new PlayerChosenAnswer { Player = prompt.Candidates[pick] };
    }

    private AnswerPayload Colors(GameClient client, ChooseColorsPromptMessage prompt)
    {
        if (prompt.Choices.Count == 0)
        {
            return prompt.Default();
        }

        List<ColorView> left = [.. prompt.Choices];
        List<ColorView> chosen = [];

        for (int slot = 0; slot < prompt.Count && left.Count > 0; slot++)
        {
            int pick = One(new Question
            {
                Heading = $"{GameText.Named(client, prompt.Source)}: choose a color"
                    + (prompt.Count > 1 ? $" ({slot + 1} of {prompt.Count})" : string.Empty) + ".",
                Note = chosen.Count > 0 ? "Already chosen: " + string.Join(", ", chosen) : string.Empty,
                Choices = [.. left.Select(color => color.ToString())],
            });

            chosen.Add(left[pick]);

            if (!prompt.MayRepeat)
            {
                left.RemoveAt(pick);
            }
        }

        return new ColorsChosenAnswer { Colors = chosen };
    }

    /// <summary>
    /// Names a card (CR 201.4).
    /// <para>
    /// Every card the game knows is a legal answer, which is not a multiple choice — so what is offered is the hand
    /// being looked at, which is the only place the answer is worth thinking about. Without one, the prompt's own
    /// suggestion is taken and nobody is asked.
    /// </para>
    /// </summary>
    private AnswerPayload Name(GameClient client, NameCardPromptMessage prompt)
    {
        List<string> names = [.. prompt.Hand
            .Select(card => client.Object(card)?.Name ?? string.Empty)
            .Where(name => name.Length > 0 && prompt.Names.Contains(name))
            .Distinct(StringComparer.Ordinal)];

        if (names.Count == 0)
        {
            return prompt.Default();
        }

        int pick = One(new Question
        {
            Heading = $"{GameText.Named(client, prompt.Source)}: name a card. "
                + $"You are looking at {GameText.Whose(client, prompt.Owner)} hand.",
            Choices = names,
        });

        return new CardNamedAnswer { Name = names[pick] };
    }

    private AnswerPayload Replacement(GameClient client, ChooseReplacementPromptMessage prompt)
    {
        if (prompt.Sources.Count == 0)
        {
            return prompt.Default();
        }

        int pick = One(new Question
        {
            Heading = $"More than one effect wants to change the same event: {prompt.Event}"
                + (prompt.Subject == ViewHandle.None ? "." : $" ({GameText.Named(client, prompt.Subject)}).")
                + " Which applies first?",
            Choices = [.. prompt.Sources.Select((source, i) =>
                $"{GameText.Named(client, source)}: {(i < prompt.Hints.Count ? prompt.Hints[i] : "applies")}")],
        });

        return new ReplacementChosenAnswer { Index = pick };
    }

    /// <summary>
    /// Orders abilities that triggered at once (CR 603.3b).
    /// <para>
    /// Asked as "which resolves first", which is how a player thinks about it, and turned round on the way out: the
    /// answer is stack order, and the ability named last goes on the stack last and so resolves first.
    /// </para>
    /// </summary>
    private AnswerPayload Order(GameClient client, OrderTriggersPromptMessage prompt)
    {
        if (prompt.Sources.Count <= 1)
        {
            return prompt.Default();
        }

        List<int> left = [.. Enumerable.Range(0, prompt.Sources.Count)];
        List<int> resolving = [];

        while (left.Count > 1)
        {
            int pick = One(new Question
            {
                Heading = "Several of your abilities triggered at once. Which one resolves first?",
                Note = resolving.Count > 0
                    ? "Resolving before it: " + string.Join("; ", resolving.Select(index => Trigger(client, prompt, index)))
                    : string.Empty,
                Choices = [.. left.Select(index => Trigger(client, prompt, index))],
            });

            resolving.Add(left[pick]);
            left.RemoveAt(pick);
        }

        resolving.AddRange(left);
        resolving.Reverse();

        var answer = new TriggersOrderedAnswer();
        answer.Order.AddRange(resolving);
        return answer;
    }

    private static string Trigger(GameClient client, OrderTriggersPromptMessage prompt, int index) =>
        $"{GameText.Named(client, prompt.Sources[index])}: \"{(index < prompt.Texts.Count ? prompt.Texts[index] : "an ability")}\"";

    /// <summary>
    /// Places the cards of a scry, a surveil or a Stock Up: one question per card, in the order they were looked at.
    /// <para>
    /// The placement is checked against the prompt's own rule before it is sent — some of these need an exact number
    /// in a pile — and a placement that does not hold falls back to the prompt's default rather than being refused.
    /// </para>
    /// </summary>
    private AnswerPayload Distribute(GameClient client, DistributeCardsPromptMessage prompt)
    {
        if (prompt.Destinations.Count == 0 || prompt.Cards.Count == 0)
        {
            return prompt.Default();
        }

        var answer = new CardsDistributedAnswer
        {
            Placements = [.. prompt.Destinations.Select(_ => new List<ViewHandle>())],
        };

        foreach (ViewHandle card in prompt.Cards)
        {
            List<int> places = [.. Enumerable.Range(0, prompt.Destinations.Count)
                .Where(index => prompt.Destinations[index].Eligible.Contains(card)
                    && answer.Placements[index].Count < prompt.Destinations[index].Max)];

            if (places.Count == 0)
            {
                return prompt.Default();
            }

            int pick = One(new Question
            {
                Heading = $"{GameText.Named(client, prompt.Source)}: you are looking at the top of your library. "
                    + $"Where does {GameText.Named(client, card)} go?",
                Note = Looking(client, prompt),
                Choices = [.. places.Select(index => Where(prompt.Destinations[index]))],
            });

            answer.Placements[places[pick]].Add(card);
        }

        return prompt.Problem(answer) is null ? answer : prompt.Default();
    }

    private static string Where(CardDestinationMessage destination) => destination.Kind switch
    {
        CardDestinationKindView.LibraryTop => "Back on top of your library.",
        CardDestinationKindView.LibraryBottom => "On the bottom of your library.",
        CardDestinationKindView.Graveyard => "Into your graveyard.",
        CardDestinationKindView.Hand => "Into your hand.",
        CardDestinationKindView.Exile => "Into exile.",
        _ => "Onto the battlefield face down.",
    };

    private static string Looking(GameClient client, DistributeCardsPromptMessage prompt) =>
        "All of them, in the order you are looking at them: "
        + string.Join(", ", prompt.Cards.Select(card => GameText.Named(client, card)))
        + ".";

    private AnswerPayload May(GameClient client, MayPromptMessage prompt)
    {
        string source = GameText.Named(client, prompt.Source);
        string amount = prompt.Amount > 0 ? " " + prompt.Amount.ToString(CultureInfo.InvariantCulture) : string.Empty;

        return new MayChosenAnswer
        {
            Accept = Yes($"{source} says you may {Verb(prompt.Verb)}{amount}. Do you?"),
        };
    }

    /// <summary>Keep the opening hand, or shuffle it away for a new one with a card fewer (CR 103.5).</summary>
    private AnswerPayload Mulligan(MulliganPromptMessage prompt) => new MayChosenAnswer
    {
        Accept = Yes(
            $"This is your opening hand{(prompt.Mulligans > 0 ? $" after {prompt.Mulligans} mulligan(s)" : string.Empty)}. "
            + $"Keeping it keeps {prompt.Keeps} card(s). A mulligan shuffles it away and deals a new hand, and each "
            + "mulligan taken costs a card put on the bottom once you keep. Do you take a mulligan?"),
    };

    /// <summary>A card-vocabulary verb as a sentence reads it: <c>SearchLibrary</c> becomes "search library".</summary>
    private static string Verb(string verb)
    {
        var words = new StringBuilder();

        foreach (char letter in verb)
        {
            if (char.IsUpper(letter) && words.Length > 0)
            {
                words.Append(' ');
            }

            words.Append(char.ToLowerInvariant(letter));
        }

        return words.Length > 0 ? words.ToString() : "do it";
    }

    // --- Asking ------------------------------------------------------------------------------------------------------

    /// <summary>
    /// Aims something at however many of the offered candidates it needs.
    /// <para>
    /// One question per target rather than one over every combination, and declining is offered only once as many as
    /// are required have been chosen — so "up to one target" (CR 601.2c) can be aimed at nothing and "two target
    /// creatures" cannot.
    /// </para>
    /// </summary>
    private List<TargetMessage> Aim(
        GameClient client,
        IReadOnlyList<TargetMessage> candidates,
        int count,
        int least,
        string what)
    {
        List<TargetMessage> chosen = [];

        if (count <= 0 || candidates.Count == 0)
        {
            return chosen;
        }

        List<TargetMessage> left = [.. candidates];

        for (int slot = 0; slot < count && left.Count > 0; slot++)
        {
            List<string> choices = [.. left.Select(candidate => GameText.Aimed(client, candidate))];
            bool mayStop = chosen.Count >= least;

            if (mayStop)
            {
                choices.Add(chosen.Count == 0 ? "Aim it at nothing." : "That is enough targets.");
            }

            int pick = One(new Question
            {
                Heading = count == 1
                    ? $"{what} What do you aim it at?"
                    : $"{what} Choose target {slot + 1} of up to {count}.",
                Note = chosen.Count > 0
                    ? "Already aimed at: " + string.Join(", ", chosen.Select(target => GameText.Aimed(client, target)))
                    : string.Empty,
                Choices = choices,
            });

            if (mayStop && pick == left.Count)
            {
                break;
            }

            chosen.Add(left[pick]);
            left.RemoveAt(pick);
        }

        return chosen;
    }

    /// <summary>
    /// Notices that the game has moved on, which is what clears the counters a step's worth of loop-guarding keeps.
    /// <para>
    /// The step rather than the turn, because a spell out of reach in an upkeep can be well within it after an untap
    /// and a land — and because a step is short enough that nothing is held against a seat for long.
    /// </para>
    /// </summary>
    private void Step(GameClient client)
    {
        (int Turn, TurnStepView Step) now = (Turn(client), client.View?.Step ?? default);

        if (now == _when)
        {
            return;
        }

        _when = now;
        _asked = 0;
        _abandoned.Clear();
    }

    private static int Turn(GameClient client) => client.View?.Turn ?? 0;

    /// <summary>A point in the game to start answering again at, and how the choice to wait for it reads.</summary>
    private readonly record struct Stop((int Turn, TurnStepView Step) At, string Text);

    /// <summary>
    /// Whether this seat is still waiting for the point it named, and so passes without being asked.
    /// <para>
    /// The point is cleared once it arrives, not merely passed over, so a stop that is somehow stepped straight past
    /// — an effect that skips a step, a turn that ends early — does not leave the seat silent for the rest of the
    /// game.
    /// </para>
    /// </summary>
    private bool Waiting(GameClient client)
    {
        if (_until is not { } wait)
        {
            return false;
        }

        if (Before(Turn(client), client.View?.Step ?? default, wait))
        {
            return true;
        }

        _until = null;
        return false;
    }

    private static bool Before(int turn, TurnStepView step, (int Turn, TurnStepView Step) mark) =>
        turn < mark.Turn || (turn == mark.Turn && step < mark.Step);

    /// <summary>
    /// The points worth waiting for from here: the ones a player names out loud at a table.
    /// <para>
    /// Written from whose turn it is, because that is what makes them mean anything. "Their end step" is this turn's
    /// end step while they are taking it and next turn's while you are taking yours, and a seat offered the wrong
    /// one would sit out its own turn to wait for a step it was already in.
    /// </para>
    /// </summary>
    private static List<Stop> Stops(GameClient client)
    {
        int turn = Turn(client);
        TurnStepView step = client.View?.Step ?? default;
        bool mine = client.View?.ActivePlayer == client.Seat;

        List<Stop> stops = [];

        if (step < TurnStepView.End)
        {
            stops.Add(new Stop(
                (turn, TurnStepView.End),
                mine
                    ? "Wait until your own end step, then decide again. Nothing you hold is spent, and your untapped "
                        + "lands stay untapped."
                    : "Wait until their end step, then decide again. This is the usual place to spend mana you have "
                        + "been keeping up, because it is your last chance before your own turn."));
        }

        stops.Add(new Stop(
            (turn + 1, TurnStepView.Upkeep),
            mine
                ? "Wait through the rest of your turn and stop at their upkeep."
                : "Wait through the rest of their turn and stop at your own upkeep, with your mana untapped again."));

        // Only from your own turn. From theirs, the next turn's end step is your own, and waiting for it would mean
        // sitting out a whole turn of your own.
        if (mine)
        {
            stops.Add(new Stop(
                (turn + 1, TurnStepView.End),
                "Wait through the rest of your turn and most of theirs, stopping at their end step. This keeps your "
                    + "mana up for their whole turn."));
        }

        return stops;
    }

    /// <summary>One choice, asked only where there is more than one to make.</summary>
    private int One(Question question)
    {
        if (question.Choices.Count == 0)
        {
            throw new NoChoiceException("A question was built with nothing to choose from.");
        }

        if (question.Trivial)
        {
            return 0;
        }

        _asked++;
        IReadOnlyList<int> picks = _chooser.Choose(question);

        return picks.Count > 0
            ? picks[0]
            : throw new NoChoiceException("The chooser named nothing.");
    }

    /// <summary>Several choices at once, asked only where the answer is not already forced.</summary>
    private IReadOnlyList<int> Many(Question question)
    {
        if (question.Choices.Count == 0)
        {
            return [];
        }

        // Exactly as many wanted as offered: there is nothing to choose, only to hand over.
        if (!question.UpTo && question.Picks >= question.Choices.Count)
        {
            return [.. Enumerable.Range(0, question.Choices.Count)];
        }

        _asked++;
        return _chooser.Choose(question);
    }

    private bool Yes(string heading) => One(new Question
    {
        Heading = heading,
        Choices = ["Yes.", "No."],
    }) == 0;

    // --- Wording -----------------------------------------------------------------------------------------------------

    private static string Forced(GameClient client, DeclareAttackersPromptMessage prompt, List<ViewHandle> forced) =>
        forced.Count == 0
            ? string.Empty
            : "Already attacking, because they must: "
                + string.Join(", ", prompt.Eligible.Where(forced.Contains).Select(a => GameText.Named(client, a))) + ".\n";

    /// <summary>
    /// What attacking costs and what the defender may do about it.
    /// <para>
    /// Two rules rather than any advice: an attacking creature taps (CR 508.1f) and so is not there to block on the
    /// way back, and blockers are declared after attackers (CR 509.1) by the defending player, who therefore sees
    /// the whole attack before choosing. Both change whether an attack is good and neither can be read off the
    /// board, so a seat not told them is being asked to decide without them.
    /// </para>
    /// </summary>
    private static string Defenders(GameClient client, int defender)
    {
        List<ObjectView> able =
            [.. client.BattlefieldOf(defender).Where(card => card.Power is not null && !card.Tapped)];

        var text = new StringBuilder(
            "A creature that attacks becomes tapped, so it cannot block on the following turn unless it has "
            + "vigilance. Blockers are declared after attackers, so they choose their blocks knowing your whole "
            + "attack, and a creature dealt damage equal to its toughness dies.\n");

        text.Append(able.Count > 0
            ? "They have these untapped and able to block: "
                + string.Join(", ", able.Select(card => GameText.Creature(client, card.Handle))) + "."
            : "They have nothing untapped that could block.");

        return text.ToString();
    }

    private static string Declared(GameClient client, List<AttackMessage> attacks) =>
        attacks.Count == 0
            ? string.Empty
            : "Already attacking: " + string.Join(
                ", ",
                attacks.Select(attack => GameText.Named(client, attack.Attacker)
                    + " at " + (attack.TargetPermanent.IsNone
                        ? GameText.Player(client, attack.TargetPlayer)
                        : GameText.Named(client, attack.TargetPermanent)))) + ".";

    private static string Blocking(GameClient client, List<BlockMessage> blocks) =>
        blocks.Count == 0
            ? string.Empty
            : "\nAlready blocking: " + string.Join(
                ", ",
                blocks.Select(block => $"{GameText.Named(client, block.Blocker)} blocks "
                    + GameText.Named(client, block.Attacker))) + ".";

    /// <summary>Which attackers may not be blocked by just one creature, so a block short of it is not wasted.</summary>
    private static string Minimums(GameClient client, DeclareBlockersPromptMessage prompt)
    {
        List<string> parts = [];

        for (int i = 0; i < prompt.Attackers.Count; i++)
        {
            int minimum = i < prompt.MinimumBlockers.Count ? prompt.MinimumBlockers[i] : 1;

            if (minimum > 1)
            {
                parts.Add(string.Create(
                    CultureInfo.InvariantCulture,
                    $"{GameText.Named(client, prompt.Attackers[i])} cannot be blocked by fewer than {minimum} creatures"));
            }
        }

        return parts.Count == 0 ? string.Empty : string.Join("; ", parts) + ".";
    }

    // --- Arithmetic --------------------------------------------------------------------------------------------------

    /// <summary>
    /// Every way of splitting a total among that many recipients, each getting at least one (CR 601.2d). Empty where
    /// there are too many ways to put in front of anybody, which is then split evenly instead.
    /// </summary>
    private static List<int[]> Splits(int total, int parts)
    {
        List<int[]> splits = [];

        if (parts <= 0 || total < parts)
        {
            return splits;
        }

        Build(new int[parts], 0, total);
        return splits.Count > MostChoices ? [] : splits;

        void Build(int[] amounts, int index, int left)
        {
            if (splits.Count > MostChoices)
            {
                return;
            }

            if (index == parts - 1)
            {
                amounts[index] = left;
                splits.Add([.. amounts]);
                return;
            }

            for (int amount = 1; amount <= left - (parts - index - 1); amount++)
            {
                amounts[index] = amount;
                Build(amounts, index + 1, left - amount);
            }
        }
    }

    /// <summary>The total spread as evenly as it goes, remainder to the front.</summary>
    private static int[] Even(int total, int parts)
    {
        int[] amounts = new int[parts];

        for (int i = 0; i < parts; i++)
        {
            amounts[i] = (total / parts) + (i < total % parts ? 1 : 0);
        }

        return amounts;
    }

    private static ActionOptionMessage? Offered(PayCostsPromptMessage prompt, ActionKindView kind) =>
        prompt.Options.FirstOrDefault(option => option.Kind == kind);

    private static TakeActionAnswer? Take(PayCostsPromptMessage prompt, ActionKindView kind) =>
        Offered(prompt, kind) is { } option ? AutoPlay.Answer(option) : null;
}
