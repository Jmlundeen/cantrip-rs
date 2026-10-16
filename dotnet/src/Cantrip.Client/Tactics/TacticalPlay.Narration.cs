using System.Globalization;
using System.Text;
using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// The seat's own account of a game: what it was asked, what it weighed, and what it answered, a decision at a time.
/// <para>
/// A game lost to a strong player is only worth something afterwards if the losing side's reasons can be read: the
/// spell it did not cast and what it thought the one it did cast was worth, the attack it made and what it scored
/// against staying home. Nothing here changes an answer; it is written only when <see cref="TacticalPlay.Narrator"/>
/// is set, and costs nothing when it is not.
/// </para>
/// </summary>
public sealed partial class TacticalPlay
{
    /// <summary>
    /// Where this seat tells each decision it makes and why, or null (the default) for silence. A priority pass with
    /// nothing considered is left out; every other question is told.
    /// </summary>
    public TextWriter? Narrator { get; set; }

    /// <summary>What the decision being made weighed, for <see cref="Narrator"/>.</summary>
    private readonly List<string> _why = [];

    private bool Narrating => Narrator is not null;

    /// <summary>The lines of the game's log already told, by number.</summary>
    private readonly HashSet<long> _toldLines = [];

    /// <summary>The turn whose log lines are being told, and the turn whose board was last told.</summary>
    private int _toldLogTurn = int.MinValue;
    private int _toldBoard = int.MinValue;

    /// <summary>Whether the heading has been written, and the end.</summary>
    private bool _toldStart;
    private bool _toldEnd;

    /// <summary>The last pass told and what it weighed, so that the same pass weighed the same way is told once a step.</summary>
    private string _toldPass = string.Empty;

    private void Note(string line)
    {
        if (Narrator is not null)
        {
            _why.Add(line);
        }
    }

    private static string Num(double value) => value >= Combat.Win / 2 ? "WIN" : value.ToString("+0.00;-0.00;0.00", CultureInfo.InvariantCulture);

    private static string NameOf(GameClient client, ViewHandle handle) => client.Object(handle)?.Name is { Length: > 0 } name ? name : handle.ToString();

    private static string TargetText(GameClient client, TargetMessage target) =>
        target.IsPlayer ? (target.Player == client.Seat ? "me" : $"player {target.Player}") : NameOf(client, target.Object);

    private static string Targets(GameClient client, IEnumerable<TargetMessage> targets) =>
        string.Join(", ", targets.Select(target => TargetText(client, target)));

    /// <summary>One candidate at priority, as the narration lists it.</summary>
    private static string Told(GameClient client, Candidate candidate)
    {
        ActionOptionMessage option = candidate.Option;
        string what = option.Kind == ActionKindView.CastSpell
            ? $"cast {(option.SpellName.Length > 0 ? option.SpellName : NameOf(client, option.Source))}"
            : $"{NameOf(client, option.Source)} [{Short(option.Description)}]";
        string at = candidate.Targets.Count > 0 ? $" -> {Targets(client, candidate.Targets)}" : string.Empty;
        return $"{what}{(candidate.Kick ? " kicked" : string.Empty)}{at} {Num(candidate.Gain)} ({candidate.Cost} mana)";
    }

    private void Narrate(GameClient client, PlayerView view, PromptPayload prompt, AnswerPayload answer)
    {
        if (prompt is MulliganPromptMessage)
        {
            Note($"hand: {string.Join(", ", client.MyHand().Select(card => card.Name))}");
        }

        // Which land paid for what, and each step of X on the way up, is the payment's own business: told is what
        // was given up, paid in life, or cast with X as it stood.
        bool routine = (prompt is PriorityPromptMessage && answer is PassAnswer && _why.Count == 0)
            || (prompt is PayCostsPromptMessage && answer is TakeActionAnswer
            {
                Kind: ActionKindView.ActivateAbility or ActionKindView.IncreaseX or ActionKindView.DecreaseX or ActionKindView.AssignMana,
            });

        if (Narrator is not { } narrator || routine)
        {
            _why.Clear();
            return;
        }

        // A pass weighed exactly as the last one this turn is the same decision again: told once.
        if (prompt is PriorityPromptMessage && answer is PassAnswer)
        {
            string pass = $"{view.Turn} {string.Join("\n", _why)}";
            if (pass == _toldPass)
            {
                _why.Clear();
                return;
            }

            _toldPass = pass;
        }
        else
        {
            _toldPass = string.Empty;
        }

        int rival = Rival(client, view);
        var line = new StringBuilder();
        line.Append(CultureInfo.InvariantCulture,
            $"T{view.Turn} {view.Step} {(view.ActivePlayer == client.Seat ? "mine" : "theirs")} | life {LifeOf(view, client.Seat)}-{LifeOf(view, rival)}");
        line.Append(CultureInfo.InvariantCulture, $" hand {client.MyHand().Count} | {Asked(client, prompt)}: {(prompt is MulliganPromptMessage && answer is MayChosenAnswer chosen ? (chosen.Accept ? "mulligan" : "keep") : Answered(client, answer))}");
        try
        {
            Story(narrator, client, view);
            narrator.WriteLine(line.ToString());

            foreach (string why in _why)
            {
                narrator.WriteLine("    " + why);
            }

            narrator.Flush();
        }
        catch (Exception error) when (error is IOException or ObjectDisposedException or UnauthorizedAccessException)
        {
            // A record that cannot be written is given up; the game is not.
            Narrator = null;
        }

        _why.Clear();
    }

    /// <summary>
    /// What happened since the last decision told, from the game's own log, and once a turn the board as it stands
    /// and this seat's hand: a decision is read against the position it was made in, and the opponent's plays are in
    /// no decision of this seat's.
    /// </summary>
    private void Story(TextWriter narrator, GameClient client, PlayerView view)
    {
        // A turn earlier than one already told is another game for the same seat: told from its start.
        if (view.Turn < _toldLogTurn || view.Turn < _toldBoard)
        {
            _toldLines.Clear();
            _toldLogTurn = int.MinValue;
            _toldBoard = int.MinValue;
            _toldStart = false;
            _toldEnd = false;
            narrator.WriteLine();
        }

        if (!_toldStart)
        {
            _toldStart = true;
            narrator.WriteLine($"# Seat {client.Seat}, the tactical bot, telling its own game (TacticalPlay.Narrator).");
            narrator.WriteLine("# '|' lines are the game's log as this seat saw it ('You' is this seat). A decision is");
            narrator.WriteLine("# 'T<turn> <step> mine|theirs | life <mine>-<theirs> hand <n> | <asked>: <answered>', and under it what was weighed.");
            narrator.WriteLine("# Numbers are log-odds of winning, gained (+) or given up (-); WIN is a win found outright.");
        }

        TellLog(narrator, client);

        // The turn's heading comes before its first decision, though the log lines that decision makes come after it.
        if (view.Turn > _toldLogTurn && view.Turn > 0)
        {
            _toldLogTurn = view.Turn;
            Heading(narrator, client, view.Turn, view.ActivePlayer);
        }

        if (view.Turn != _toldBoard && view.Turn > 0)
        {
            _toldBoard = view.Turn;
            foreach (string side in Board(client, view))
            {
                narrator.WriteLine(side);
            }
        }
    }

    private void TellLog(TextWriter narrator, GameClient client)
    {
        foreach (LogEntryView entry in client.Log)
        {
            // Each line has its own number, and is told once.
            if (!_toldLines.Add(entry.Number))
            {
                continue;
            }

            if (entry.Turn > _toldLogTurn && entry.Turn > 0)
            {
                _toldLogTurn = entry.Turn;
                Heading(narrator, client, entry.Turn, entry.ActivePlayer);
            }

            narrator.WriteLine($"  | {entry.Text}");
        }
    }

    private static void Heading(TextWriter narrator, GameClient client, int turn, int active)
    {
        narrator.WriteLine();
        narrator.WriteLine($"--- Turn {turn}, {(active == client.Seat ? "mine" : $"seat {active}'s")} ---");
    }

    /// <summary>Each side's life, hand, library and permanents, and this seat's hand by name.</summary>
    private static IEnumerable<string> Board(GameClient client, PlayerView view)
    {
        static string Permanent(ObjectView card)
        {
            var text = new StringBuilder(card.Name.Length > 0 ? card.Name : "face-down");
            if (card.Power is { } power && card.CardTypes.Contains(CardTypeView.Creature))
            {
                text.Append(CultureInfo.InvariantCulture, $" {power}/{card.Toughness}");
            }

            foreach (CounterView counter in card.Counters.Where(counter => counter.Count > 0))
            {
                text.Append(CultureInfo.InvariantCulture, $" [{counter.Count} {counter.Name}]");
            }

            if (card.Tapped)
            {
                text.Append(" (T)");
            }

            return text.ToString();
        }

        foreach (PlayerSummaryView player in view.Players.OrderBy(player => player.Seat != client.Seat))
        {
            List<ObjectView> theirs = [.. client.BattlefieldOf(player.Seat)];
            List<ObjectView> lands = [.. theirs.Where(card => card.CardTypes.Contains(CardTypeView.Land))];
            string others = string.Join(", ", theirs.Where(card => !card.CardTypes.Contains(CardTypeView.Land)).Select(Permanent));
            yield return $"  board {(player.Seat == client.Seat ? "me  " : "them")}: {player.Life} life, {player.HandCount} in hand, "
                + $"{player.LibraryCount} in library, {lands.Count} lands ({lands.Count(land => !land.Tapped)} untapped)"
                + (others.Length > 0 ? $" | {others}" : string.Empty);
        }

        yield return $"  my hand: {string.Join(", ", client.MyHand().Select(card => card.Name))}";
    }

    /// <summary>
    /// The end of the game, told once it is over: the log's last lines, which no decision of this seat's followed, and
    /// who won. A host whose loop stops at the game's end calls it then; <see cref="Act"/> calls it if it is asked
    /// after the end.
    /// </summary>
    public void NarrateEnd(GameClient client)
    {
        ArgumentNullException.ThrowIfNull(client);
        if (_toldEnd || Narrator is not { } narrator || client.Outcome is not { } outcome)
        {
            return;
        }

        _toldEnd = true;
        try
        {
            TellLog(narrator, client);
            int team = client.View?.Players.Find(player => player.Seat == client.Seat)?.Team ?? -1;
            narrator.WriteLine();
            narrator.WriteLine($"=== Game over: {outcome.Outcome}{(outcome.WinningTeam > 0 ? (outcome.WinningTeam == team ? ", this seat won" : ", this seat lost") : string.Empty)} ===");
            narrator.Flush();
        }
        catch (Exception error) when (error is IOException or ObjectDisposedException or UnauthorizedAccessException)
        {
            Narrator = null;
        }
    }

    /// <summary>An ability's text cut to what fits on a line of the narration.</summary>
    private static string Short(string text)
    {
        string line = text.ReplaceLineEndings(" ").Trim();
        return line.Length <= 60 ? line : string.Concat(line.AsSpan(0, 57), "...");
    }

    private static string Asked(GameClient client, PromptPayload prompt) => prompt switch
    {
        PriorityPromptMessage => "priority",
        PayCostsPromptMessage pay => $"pay {pay.Remaining} for {NameOf(client, pay.Spell)}{(pay.HasX ? $" with X={pay.X}" : string.Empty)}",
        UnlessPaysPromptMessage unless => $"pay {unless.Remaining} or not ({NameOf(client, unless.Source)})",
        DeclareAttackersPromptMessage => "attack",
        DeclareBlockersPromptMessage => "block",
        ChooseTargetPromptMessage => "target",
        MulliganPromptMessage mulligan => $"keep {mulligan.Keeps} or mulligan",
        ChooseCardsPromptMessage cards => $"choose cards ({cards.Purpose})",
        SearchLibraryPromptMessage search => $"search ({search.Destination})",
        DiscardPromptMessage => "discard",
        MayPromptMessage may => $"may {may.Verb} ({NameOf(client, may.Source)})",
        _ => prompt.GetType().Name.Replace("PromptMessage", string.Empty, StringComparison.Ordinal),
    };

    private static string Answered(GameClient client, AnswerPayload answer) => answer switch
    {
        PassAnswer => "pass",
        TakeActionAnswer action => action.Kind switch
        {
            ActionKindView.CastSpell => $"cast {NameOf(client, action.Source)}",
            ActionKindView.ActivateAbility => $"activate {NameOf(client, action.Source)}#{action.AbilityIndex}",
            ActionKindView.PlayLand => $"play {NameOf(client, action.Source)}",
            ActionKindView.UnlockDoor => $"unlock a door of {NameOf(client, action.Source)}",
            _ => action.Source.IsNone ? action.Kind.ToString() : $"{action.Kind} {NameOf(client, action.Source)}",
        } + (action.Targets.Count > 0 ? $" -> {Targets(client, action.Targets)}" : string.Empty),
        AttackersDeclaredAnswer attack => attack.Attacks.Count == 0 ? "no attack"
            : string.Join(", ", attack.Attacks.Select(one => NameOf(client, one.Attacker)
                + (one.TargetPermanent.IsNone ? string.Empty : $" at {NameOf(client, one.TargetPermanent)}"))),
        BlockersDeclaredAnswer block => block.Blocks.Count == 0 ? "no block"
            : string.Join(", ", block.Blocks.Select(one => $"{NameOf(client, one.Blocker)} blocks {NameOf(client, one.Attacker)}")),
        CardsChosenAnswer cards => cards.Cards.Count == 0 ? "none" : string.Join(", ", cards.Cards.Select(card => NameOf(client, card))),
        TargetsChosenAnswer targets => targets.Targets.Count == 0 ? "no target" : Targets(client, targets.Targets),
        MayChosenAnswer may => may.Accept ? "yes" : "no",
        _ => answer.GetType().Name.Replace("Answer", string.Empty, StringComparison.Ordinal),
    };
}
