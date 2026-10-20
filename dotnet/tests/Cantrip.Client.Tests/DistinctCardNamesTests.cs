using Cantrip.Client;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

public sealed class DistinctCardNamesTests
{
    [Fact]
    public void KeepsOnlyCardsWhoseAuthoritativeNameTokensDoNotIntersect()
    {
        (int Id, string[] Names)[] cards =
        [
            (1, ["Fire", "Ice"]),
            (2, ["Ice"]),
            (3, ["Fire // Ice"]),
            (4, ["Soulfire"]),
            (5, []),
        ];

        int[] kept = [.. DistinctCardNames.Keep(cards, card => card.Names).Select(card => card.Id)];

        Assert.Equal([1, 3, 4], kept);
    }

    [Fact]
    public void AutoPlayUsesPromptNameTokensInsteadOfDisplayNames()
    {
        var transport = new PromptTransport(new PromptMessage
        {
            RequestId = 1,
            Prompt = new SearchLibraryPromptMessage
            {
                Player = 1,
                Count = 2,
                DistinctNames = true,
                Eligible = [new ViewHandle(10), new ViewHandle(11)],
                EligibleNames = [["Fire", "Ice"], ["Ice"]],
            },
        });
        var client = new GameClient(transport);
        client.Pump();

        Assert.True(AutoPlay.Act(client));
        AnswerMessage sent = Assert.IsType<AnswerMessage>(Assert.Single(transport.Sent));
        CardsChosenAnswer answer = Assert.IsType<CardsChosenAnswer>(sent.Answer);
        Assert.Equal([new ViewHandle(10)], answer.Cards);
    }

    private sealed class PromptTransport(ServerMessage message) : IGameTransport
    {
        private ServerMessage? _message = message;
        public List<ClientMessage> Sent { get; } = [];
        public bool IsConnected => true;
        public void Send(ClientMessage message) => Sent.Add(message);
        public IReadOnlyList<ServerMessage> Receive()
        {
            if (_message is not { } message) return [];
            _message = null;
            return [message];
        }
        public void Dispose() { }
    }
}
