using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

public sealed class MembershipStateTests
{
    [Fact]
    public void An_in_flight_prompt_cannot_reopen_an_abandoned_table()
    {
        var client = new GameClient(new FixtureTransport([
            new PromptMessage { RequestId = 1 },
            new TableMessage { Phase = TablePhase.Abandoned },
            new PromptMessage { RequestId = 2 },
        ]));
        client.Pump();
        Assert.Equal(TablePhase.Abandoned, client.Table!.Phase);
        Assert.Null(client.Prompt);
        Assert.False(client.IsWaitingOnMe);
        Assert.Null(client.Outcome);
    }
}
