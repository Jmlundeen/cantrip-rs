using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// What a seat nobody drives answers once a step has gone on too long (<see cref="LoopGuard"/>): the engine cuts no loop off,
/// so the seat stops going round by itself, passing and declining, and picking each answer of a pick in turn.
/// </summary>
public sealed class LoopGuardTests
{
    [Fact]
    public void Picks_each_candidate_in_turn()
    {
        var prompt = new ChooseTargetPromptMessage
        {
            Player = 1,
            Candidates = [new TargetMessage { Object = new ViewHandle(3) }, new TargetMessage { Object = new ViewHandle(4) }, new TargetMessage { Player = 2 }],
            Count = 1,
        };

        List<TargetMessage> picked = [.. Enumerable.Range(0, 4).Select(turn => Assert.IsType<TargetsChosenAnswer>(LoopGuard.Passive(prompt, turn)).Targets.Single())];

        Assert.Equal([prompt.Candidates[0], prompt.Candidates[1], prompt.Candidates[2], prompt.Candidates[0]], picked);
    }

    [Fact]
    public void Passes_and_declines()
    {
        Assert.IsType<PassAnswer>(LoopGuard.Passive(new PriorityPromptMessage { Player = 1 }, 5));
        Assert.False(Assert.IsType<MayChosenAnswer>(LoopGuard.Passive(new MayPromptMessage { Player = 1 }, 5)).Accept);
    }

    /// <summary>
    /// A seat that keeps a loop going, asked the same priority over and over in one step: the guard lets it answer the first
    /// questions and answers passively after that, however long the step goes on. It never concedes: a run the passive
    /// answers cannot end is a draw once the game reaches the most events the engine records (RX-1).
    /// </summary>
    [Fact]
    public void Answers_passively_past_its_limit_and_never_concedes()
    {
        var table = new TacticsBoard.Table([]);
        var looping = new Looping();
        var guard = new LoopGuard(looping, answersPerStep: 3);

        for (int request = 1; request <= 20; request++)
        {
            table.Ask(request, new PriorityPromptMessage { Player = TacticsBoard.Me });
            Assert.True(guard.Act(table.Client));
        }

        Assert.Equal(3, looping.Asked);
        Assert.Equal(20, table.Sent.OfType<AnswerMessage>().Count());
        Assert.Equal(3, table.Sent.OfType<AnswerMessage>().Count(a => a.Answer is TakeActionAnswer));
        Assert.Equal(17, table.Sent.OfType<AnswerMessage>().Count(a => a.Answer is PassAnswer));
        Assert.Empty(table.Sent.OfType<ConcedeMessage>());
    }

    /// <summary>A seat that acts every time it is asked: the loop goes on as long as it is the one answering.</summary>
    private sealed class Looping : IAutoPlayer
    {
        public int Asked { get; private set; }

        public bool Retrying => false;

        public bool Act(GameClient client)
        {
            Asked++;
            client.Answer(new TakeActionAnswer { Kind = ActionKindView.ActivateAbility, Source = new ViewHandle(9) });
            return true;
        }
    }
}
