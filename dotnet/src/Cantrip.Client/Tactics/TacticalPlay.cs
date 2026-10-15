using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// Plays a seat by weighing what each choice would do to the position.
/// <para>
/// The third way a seat is answered without a person, and the first meant to be played against.
/// <see cref="AutoPlay"/> takes the one same line every game and <see cref="RandomPlay"/> draws among what it is
/// offered; neither looks at the board. This one does. It reads its cards off their text
/// (<see cref="CardReading"/>), works out combat before it happens (<see cref="Combat"/>), and scores what it sees
/// with a model fitted to games it played against itself (<see cref="TacticalModel"/>): attack where the defender's
/// best blocks still leave it ahead, block where the trade or the life is worth the creature, spend removal on what
/// is worth a card, and send burn at a player once their life total makes that the better use of it.
/// </para>
/// <para>
/// It is a client like any other. Everything it knows is in the view and the prompt a player's browser would be
/// sent, every answer is built from what the prompt offers, and the server remains the only authority on what is
/// legal — so it runs wherever a client runs, the web client included, on the machine of whoever wants an opponent,
/// with nothing to download and nobody else's hardware to ask. None of it waits on anything: a decision is some
/// arithmetic over a board, made inside <see cref="Act"/>.
/// </para>
/// <para>
/// It plays the same way every time unless <see cref="Explore"/> says otherwise, which is for the games it learns
/// from and nowhere else.
/// </para>
/// </summary>
public sealed partial class TacticalPlay : IAutoPlayer
{
    /// <summary>How many times one question is answered before this seat takes the prompt's own answer instead.</summary>
    private const int Attempts = 2;

    /// <summary>Actions one seat takes in one step before it passes regardless: the backstop against a loop.</summary>
    private const int ActionsPerStep = 40;

    /// <summary>
    /// Answers of any kind one seat gives in one step before it gives every further question the prompt's own answer:
    /// pass, decline, give up the cast. No honest step takes a fraction of this, and a seat with no rules engine cannot
    /// rule out every way two questions might lead back to each other, so the backstop is a count.
    /// </summary>
    private const int AnswersPerStep = 400;

    private readonly TacticalModel _model;
    private readonly RandomPlay _wander;
    private readonly Random _random;

    private int _askedId = -1;
    private int _attempts;

    /// <summary>The turn and step the per-step state below belongs to.</summary>
    private (int Turn, TurnStepView Step) _step = (-1, TurnStepView.Untap);
    private int _turn = -1;
    private int _actions;
    private int _answers;

    /// <summary>
    /// The cards this seat has already chosen to cast this step when an effect offered them, so that one whose cast
    /// could not be finished - a spree mode it cannot pay for, off Etali - is not chosen again each time it is offered.
    /// </summary>
    private readonly HashSet<int> _offered = [];

    /// <summary>What was tried this turn and changed nothing, so it is not tried again until the next.</summary>
    private readonly HashSet<(int Source, ActionKindView Kind, int Ability)> _spent = [];

    /// <summary>The action last taken at priority, and the board as it stood, to tell whether it did anything.</summary>
    private ((int Source, ActionKindView Kind, int Ability) Key, long Board)? _pending;

    /// <summary>Whether the spell being paid for is to be kicked.</summary>
    private bool _kick;

    /// <summary>The optional cost last decided, so one decision covers the whole payment.</summary>
    private ViewHandle _unless;
    private bool _paying;

    /// <summary>What was still owed of it when last asked, and how many times running that has not gone down.</summary>
    private string _unlessOwed = string.Empty;
    private int _unlessStalls;

    /// <summary>The colors of whatever this seat last protected something from, for the color it is then asked to name.</summary>
    private List<ColorView> _threat = [];

    /// <summary>What has been done in answer to each thing on the stack this turn, so the same answer is not given twice.</summary>
    private readonly HashSet<(int Onto, int Source, int Ability)> _answered = [];

    /// <summary>Spells this seat has agreed to copy this turn. Two seats that always copy pass a spell between them for ever.</summary>
    private int _copies;

    /// <summary>The most copies of spells this seat makes in one turn.</summary>
    private const int CopiesPerTurn = 3;

    /// <summary>
    /// The permanents this seat controlled when the turn began, so far as it saw: those it last saw before the turn
    /// changed. A Vehicle is not sent summoning sickness (it is not a creature until crewed), and one cast this turn
    /// cannot attack (CR 302.6), so only one of these is worth crewing. Null until the seat has seen a turn change,
    /// when everything is taken to have been there.
    /// </summary>
    private HashSet<ViewHandle>? _settled;

    /// <summary>This seat's permanents when it was last asked anything.</summary>
    private HashSet<ViewHandle> _seen = [];

    /// <param name="model">What a position is worth; the shipped weights when null.</param>
    /// <param name="seed">Where <see cref="Explore"/> draws from, or null for a different draw every run.</param>
    public TacticalPlay(TacticalModel? model = null, int? seed = null)
    {
        _model = model ?? TacticalModel.Default;
        _random = seed is { } fixed_ ? new Random(fixed_) : new Random();
        _wander = new RandomPlay(seed);
    }

    /// <summary>
    /// How often, from 0 to 1, a decision is drawn at random instead of weighed. Zero for play. The games the model
    /// is fitted to set it a little above, so that they reach positions sound play never would and the fit has
    /// something to compare sound play against.
    /// </summary>
    public double Explore { get; init; }

    public bool Retrying => _attempts < Attempts;

    public bool Act(GameClient client)
    {
        ArgumentNullException.ThrowIfNull(client);

        if (client.IsOver || !client.IsWaitingOnMe)
        {
            if (client.IsOver && Narrating)
            {
                NarrateEnd(client);
            }

            return false;
        }

        PromptMessage asked = client.Prompt!;

        // The identical question, id and all, is what the server sends back when it refuses an answer.
        if (asked.RequestId == _askedId)
        {
            _attempts++;
        }
        else
        {
            _askedId = asked.RequestId;
            _attempts = 0;
        }

        // An answer the server would not take is not argued with: the prompt's own answer is always legal, and
        // whatever was being attempted is not attempted again this turn.
        if (_attempts >= Attempts)
        {
            if (_pending is { } refused)
            {
                _spent.Add(refused.Key);
                _pending = null;
            }

            client.Answer(asked.Prompt.Default());
            return true;
        }

        AnswerPayload answer;
        try
        {
            answer = Decide(client, asked.Prompt);
        }
        catch (Exception error) when (error is InvalidOperationException or ArgumentException or IndexOutOfRangeException
            or NullReferenceException or KeyNotFoundException or FormatException or ArithmeticException)
        {
            // A position this seat cannot make sense of costs it a poor answer, not the table a seat that has stopped:
            // the prompt's own answer is always legal.
            Faults++;
            answer = asked.Prompt.Default();
        }

        client.Answer(answer);
        return true;
    }

    /// <summary>How many questions this seat gave the prompt's own answer to because working one out failed.</summary>
    public int Faults { get; private set; }

    /// <summary>
    /// This seat's answer to one question about the position the client holds. Public so a test can ask what it
    /// would do with a board without a game behind it.
    /// </summary>
    public AnswerPayload Decide(GameClient client, PromptPayload prompt)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);

        if (client.View is not { } view)
        {
            return prompt.Default();
        }

        _why.Clear();
        AnswerPayload answer = Deciding(client, view, prompt);
        if (Narrating)
        {
            // Telling it never touches the answer: a record that fails, whatever the reason, is given up.
            try
            {
                Narrate(client, view, prompt, answer);
            }
            catch (Exception)
            {
                Narrator = null;
                _why.Clear();
            }
        }

        return answer;
    }

    private AnswerPayload Deciding(GameClient client, PlayerView view, PromptPayload prompt)
    {
        if (view.Turn != _turn)
        {
            _settled = _turn < 0 ? null : _seen;
            _turn = view.Turn;
            _spent.Clear();
            _answered.Clear();
            _copies = 0;
        }

        _seen = [.. client.BattlefieldOf(client.Seat).Select(permanent => permanent.Handle)];

        if (_step != (view.Turn, view.Step))
        {
            _step = (view.Turn, view.Step);
            _actions = 0;
            _answers = 0;
            _threat = [];
            _offered.Clear();
        }

        if (++_answers > AnswersPerStep)
        {
            // Where a question can be answered with nothing - no card, no target, no - that is the answer, since the
            // prompt's own answer to "which of these will you cast" is the first of them, which is how it got here.
            _pending = null;
            return prompt switch
            {
                ChooseCardsPromptMessage { UpTo: true } => new CardsChosenAnswer(),
                SearchLibraryPromptMessage { Optional: true } => new CardsChosenAnswer(),
                ChooseTargetPromptMessage { Optional: true } => new TargetsChosenAnswer(),
                MayPromptMessage => new MayChosenAnswer { Accept = false },
                _ => prompt.Default(),
            };
        }

        if (Explore > 0 && _random.NextDouble() < Explore
            && prompt is PriorityPromptMessage or DeclareAttackersPromptMessage or DeclareBlockersPromptMessage or ChooseTargetPromptMessage)
        {
            _pending = null;
            _kick = false;
            return _wander.Decide(prompt);
        }

        return prompt switch
        {
            PriorityPromptMessage priority => Priority(client, view, priority),
            PayCostsPromptMessage pay => Pay(client, pay),
            UnlessPaysPromptMessage unless => Unless(client, unless),
            DeclareAttackersPromptMessage attackers => Attack(client, view, attackers),
            DeclareBlockersPromptMessage blockers => Block(client, view, blockers),
            ChooseTargetPromptMessage target => Target(client, view, target),
            ChooseDividedTargetsPromptMessage divided => Divide(client, view, divided),
            ChooseModesPromptMessage modes => Modes(client, view, modes),
            ChooseCardsPromptMessage cards => Cards(client, view, cards),
            SearchLibraryPromptMessage search => Search(client, view, search),
            DiscardPromptMessage discard => new CardsChosenAnswer { Cards = Least(client, view, discard.Hand, discard.Count) },
            DistributeCardsPromptMessage distribute => Distribute(client, view, distribute),
            MulliganPromptMessage mulligan => Mulligan(client, mulligan),
            MayPromptMessage may => May(client, view, may),
            ChoosePlayerPromptMessage player => Player(client, player),
            ChooseStartingPlayerPromptMessage starting => new PlayerChosenAnswer
            {
                // Playing first: a turn ahead is worth more than a card to any deck that means to attack.
                Player = starting.Candidates.Contains(client.Seat) ? client.Seat : starting.Candidates[0],
            },
            ChooseColorsPromptMessage colors => Colors(client, view, colors),

            // The engine's own division of combat damage, the first replacement and the order things triggered in
            // are each legal and each as good as this seat can do without knowing what the triggers are.
            _ => prompt.Default(),
        };
    }

    // --- The position, as this seat reads it -------------------------------------------------------------------------

    private double Policy(string name) => _model.Weight("policy." + name);

    private static PlayerSummaryView? Summary(PlayerView view, int seat) => view.Players.Find(player => player.Seat == seat);

    private static int LifeOf(PlayerView view, int seat) => Summary(view, seat)?.Life ?? 0;

    /// <summary>The opponent this seat is racing: with several, the one nearest to losing.</summary>
    private static int Rival(GameClient client, PlayerView view) => view.Players
        .Where(player => player.Seat != client.Seat && player.Status == PlayerStatusView.Active
            && player.Team != (Summary(view, client.Seat)?.Team ?? -1))
        .OrderBy(player => player.Life)
        .Select(player => player.Seat)
        .FirstOrDefault(client.Seat == 1 ? 2 : 1);

    private static bool Mine(GameClient client, ObjectView card) => card.Controller == client.Seat;

    private static bool IsCreature(ObjectView card) => card.CardTypes.Contains(CardTypeView.Creature);

    private static bool IsLand(ObjectView card) => card.CardTypes.Contains(CardTypeView.Land);

    private static bool IsPermanentCard(ObjectView card) => card.CardTypes.Exists(type =>
        type is CardTypeView.Creature or CardTypeView.Artifact or CardTypeView.Enchantment or CardTypeView.Planeswalker
            or CardTypeView.Land or CardTypeView.Battle);

    private static IEnumerable<ObjectView> Creatures(GameClient client, int seat) =>
        client.BattlefieldOf(seat).Where(IsCreature);

    private static int Lands(GameClient client, int seat) => client.BattlefieldOf(seat).Count(IsLand);

    /// <summary>
    /// What a permanent is worth to its controller, lands included: the model weighs lands by how many there are, so
    /// one of them is worth what the next land would be.
    /// </summary>
    private double Worth(GameClient client, ObjectView permanent)
    {
        if (IsLand(permanent) && !IsCreature(permanent))
        {
            return Lands(client, permanent.Controller) <= 4 ? _model.Weight("land.1-4") : _model.Weight("land.5+");
        }

        return _model.Worth(permanent);
    }

    /// <summary>
    /// The board, leaving out whatever paying for something changes — what is tapped, what is in a pool — so that an
    /// action which cost mana and did nothing else reads as having done nothing.
    /// </summary>
    private static long Board(PlayerView view)
    {
        var hash = new HashCode();

        foreach (PlayerSummaryView player in view.Players)
        {
            // Not its own life: a land that costs a life to tap is a payment like any other.
            hash.Add(player.Seat == view.Viewer ? 0 : player.Life);
            hash.Add(player.HandCount);
            hash.Add(player.LibraryCount);
            hash.Add(player.Energy);
        }

        foreach (ZoneView zone in view.Zones)
        {
            hash.Add(zone.Count);
        }

        foreach (ObjectView card in view.Objects)
        {
            if (card.Zone != ZoneKindView.Battlefield)
            {
                continue;
            }

            hash.Add(card.Handle.Value);
            hash.Add(card.Controller);
            hash.Add(card.Power);
            hash.Add(card.Toughness);
            hash.Add(card.AttachedTo.Value);
            hash.Add(card.Subtypes.Count);
            hash.Add(card.GrantedAbilities.Count);

            foreach (CounterView counter in card.Counters)
            {
                hash.Add(counter.Count);
            }
        }

        return hash.ToHashCode();
    }
}
