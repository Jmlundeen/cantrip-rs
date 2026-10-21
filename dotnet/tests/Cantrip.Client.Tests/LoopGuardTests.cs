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
}
