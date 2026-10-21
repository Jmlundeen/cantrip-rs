using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// Keeps an automated seat from going round a loop for ever.
/// <para>
/// The rules engine never cuts off a loop its players keep going by choice (a combination may need hundreds of rounds to
/// win), so a seat nobody is driving must stop by itself. This one lets the seat it wraps answer the first
/// <see cref="AnswersPerStep"/> questions of a step as it likes; past that it answers each one passively itself: it passes,
/// declines, and takes each question's default, but for a question that picks one thing among several, where it picks
/// each in turn - a different answer each time round, so that a choice that would end the loop is tried. A loop that goes on
/// whatever is picked is then one of passes and declines, which the engine makes a draw (CR 732.4-5).
/// </para>
/// <para>
/// And past <see cref="ConcedeAfter"/> questions in one step - a loop the passive answers could not end, which the engine
/// does not judge a draw - the seat concedes (CR 104.3a), so that no table of automated seats ever goes on for ever.
/// </para>
/// </summary>
public sealed class LoopGuard : IAutoPlayer
{
    /// <summary>More questions than a step outside a loop asks a seat.</summary>
    public const int AnswersPerStep = 4096;

    /// <summary>Far past what the passive answers need to end any loop that can be ended: a million questions in one step.</summary>
    public const int ConcedeAfter = 1_000_000;

    private readonly IAutoPlayer _inner;
    private readonly int _answersPerStep;
    private readonly int _concedeAfter;
    private (int Turn, TurnStepView Step) _step;
    private int _answers;
    private int _turn;

    public LoopGuard(IAutoPlayer inner)
        : this(inner, AnswersPerStep, ConcedeAfter)
    {
    }

    /// <summary>A guard with its own limits, for a test.</summary>
    public LoopGuard(IAutoPlayer inner, int answersPerStep, int concedeAfter)
    {
        ArgumentNullException.ThrowIfNull(inner);
        _inner = inner is LoopGuard guard ? guard._inner : inner;
        _answersPerStep = answersPerStep;
        _concedeAfter = Math.Max(concedeAfter, answersPerStep);
    }

    /// <summary>The seat it guards.</summary>
    public IAutoPlayer Inner => _inner;

    public bool Retrying => _inner.Retrying;

    public bool Thinking => _inner.Thinking;

    public bool Act(GameClient client)
    {
        ArgumentNullException.ThrowIfNull(client);

        if (client.IsOver || !client.IsWaitingOnMe || client.View is not { } view)
        {
            return _inner.Act(client);
        }

        if ((view.Turn, view.Step) != _step)
        {
            _step = (view.Turn, view.Step);
            _answers = 0;
        }

        // Counted no further than past the last limit: the count stops there, and never wraps back below it.
        if (_answers <= _concedeAfter)
        {
            _answers++;
        }

        if (_answers <= _answersPerStep || _inner.Thinking)
        {
            return _inner.Act(client);
        }

        if (_answers > _concedeAfter)
        {
            client.Concede();
            return true;
        }

        client.Answer(Passive(client.Prompt!.Prompt, _turn));
        // Which candidate comes next: only ever read modulo the number of candidates, so it goes back to 0 rather than overflow.
        _turn = _turn == int.MaxValue ? 0 : _turn + 1;
        return true;
    }

    /// <summary>The passive answer to <paramref name="prompt"/>: its default, or the <paramref name="turn"/>th of its picks in turn.</summary>
    public static AnswerPayload Passive(PromptPayload prompt, int turn)
    {
        ArgumentNullException.ThrowIfNull(prompt);
        static int Pick(int turn, int count) => (turn % count + count) % count;

        return prompt switch
        {
            ChooseTargetPromptMessage { Optional: false, Count: 1, Candidates.Count: > 1 } target
                when target.MaximumTotalManaValue is null
                => new TargetsChosenAnswer { Targets = [target.Candidates[Pick(turn, target.Candidates.Count)]] },
            ChooseCardsPromptMessage { Count: 1, Eligible.Count: > 1, UpTo: false } cards
                when cards.MaximumTotalManaValue is null && cards.MinimumPower == 0
                => new CardsChosenAnswer { Cards = [cards.Eligible[Pick(turn, cards.Eligible.Count)]] },
            ChoosePlayerPromptMessage { Candidates.Count: > 1 } player
                => new PlayerChosenAnswer { Player = player.Candidates[Pick(turn, player.Candidates.Count)] },
            ChooseModesPromptMessage { Count: 1, Eligible.Count: > 1 } modes
                => new ModesChosenAnswer { Modes = [modes.Eligible[Pick(turn, modes.Eligible.Count)]] },
            _ => prompt.Default(),
        };
    }
}
