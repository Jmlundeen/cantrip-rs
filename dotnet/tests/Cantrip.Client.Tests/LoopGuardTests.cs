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
    /// questions, answers passively after that, and past its last limit concedes (CR 104.3a), so the table cannot go on for ever.
    /// </summary>
    [Fact]
    public void Concedes_past_its_last_limit_in_one_step()
    {
        var table = new TacticsBoard.Table([]);
        var looping = new Looping();
        var guard = new LoopGuard(looping, answersPerStep: 3, concedeAfter: 6);

        for (int request = 1; request <= 7; request++)
        {
            table.Ask(request, new PriorityPromptMessage { Player = TacticsBoard.Me });
            Assert.True(guard.Act(table.Client));
        }

        Assert.Equal(3, looping.Asked);
        Assert.Equal(6, table.Sent.OfType<AnswerMessage>().Count());
        Assert.Equal(3, table.Sent.OfType<AnswerMessage>().Count(a => a.Answer is TakeActionAnswer));
        Assert.Single(table.Sent.OfType<ConcedeMessage>());
        Assert.IsType<ConcedeMessage>(table.Sent[^1]);
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
