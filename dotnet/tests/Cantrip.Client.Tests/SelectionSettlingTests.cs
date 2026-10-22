using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// An answer to a choice of cards is settled with the host before it is sent (<see cref="GameClient.Answer"/>): checked
/// (<see cref="CheckSelectionMessage"/>), sent once the host calls it an answer, and mended a card at a time from what the
/// host says may be added when it does not - so a player that picks without knowing the question's rule still answers
/// what the rules take.
/// </summary>
public sealed class SelectionSettlingTests
{
    private static ViewHandle H(int value) => new(value);

    /// <summary>A transport whose host judges every check with <paramref name="rule"/>, as the native host does with the question's own rule.</summary>
    private sealed class Judging(Func<IReadOnlyList<ViewHandle>, SelectionVerdictView> rule) : IGameTransport
    {
        public Queue<ServerMessage> Inbound { get; } = [];

        public List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Send(ClientMessage message)
        {
            Sent.Add(message);
            if (message is CheckSelectionMessage check)
            {
                Inbound.Enqueue(new SelectionVerdictMessage { RequestId = check.RequestId, Selected = check.Selected, Verdict = rule(check.Selected) });
            }
        }

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> messages = [.. Inbound];
            Inbound.Clear();
            return messages;
        }

        public void Dispose()
        {
        }
    }

    /// <summary>
    /// "Exactly two cards with different names" over cards 1 to 4, where 1 is a split card sharing a name with both 2 and 3,
    /// and 2 and 4 share theirs (CR 201.2a-b).
    /// </summary>
    private static SelectionVerdictView TwoWithDifferentNames(IReadOnlyList<ViewHandle> picked)
    {
        HashSet<(int, int)> same = [(1, 2), (2, 1), (1, 3), (3, 1), (2, 4), (4, 2)];
        bool apart(ViewHandle a, ViewHandle b) => !same.Contains((a.Value, b.Value)) && a != b;
        bool valid = picked.Select((card, i) => picked.Take(i).All(other => apart(card, other))).All(ok => ok);
        List<ViewHandle> addable = picked.Count >= 2 ? [] : [.. new[] { H(1), H(2), H(3), H(4) }.Where(card => !picked.Contains(card) && picked.All(other => apart(card, other)))];
        return new SelectionVerdictView { Complete = valid && picked.Count == 2, Addable = addable };
    }

    private static (GameClient Client, Judging Transport) Asked(Func<IReadOnlyList<ViewHandle>, SelectionVerdictView> rule, bool noneIsAnswer = false)
    {
        var transport = new Judging(rule);
        var client = new GameClient(transport);
        transport.Inbound.Enqueue(new PromptMessage
        {
            RequestId = 7,
            Prompt = new SearchLibraryPromptMessage
            {
                Player = 1,
                Count = 2,
                CountKind = noneIsAnswer ? CountKindView.UpTo : CountKindView.Exactly,
                Library = [H(1), H(2), H(3), H(4)],
                Words = ["with different names"],
                Verdict = rule([]) is var first ? new SelectionVerdictView { Complete = noneIsAnswer || first.Complete, Addable = first.Addable } : new(),
            },
        });
        client.Pump();
        return (client, transport);
    }

    private static List<ViewHandle>? Answered(Judging transport) =>
        transport.Sent.OfType<AnswerMessage>().LastOrDefault()?.Answer is CardsChosenAnswer chosen ? chosen.Cards : null;

    [Fact]
    public void An_answer_the_host_takes_is_sent_once_checked()
    {
        (GameClient client, Judging transport) = Asked(TwoWithDifferentNames);
        client.Answer(new CardsChosenAnswer { Cards = [H(2), H(3)] });

        Assert.False(client.IsWaitingOnMe);
        Assert.True(client.IsSettling);
        Assert.Null(Answered(transport));
        Assert.Single(transport.Sent.OfType<CheckSelectionMessage>());

        client.Pump();
        Assert.False(client.IsSettling);
        Assert.Equal([H(2), H(3)], Answered(transport));
    }

    [Fact]
    public void An_answer_the_host_refuses_is_mended_from_what_may_be_added_keeping_the_cards_meant_first()
    {
        // 2 and 4 share a name: 2 is kept, 4 is not addable beside it, and 3 - the first card of the question's own order
        // that is - takes its place.
        (GameClient client, Judging transport) = Asked(TwoWithDifferentNames);
        client.Answer(new CardsChosenAnswer { Cards = [H(2), H(4)] });
        for (int i = 0; i < 10 && client.IsSettling; i++) client.Pump();

        Assert.Equal([H(2), H(3)], Answered(transport));
    }

    [Fact]
    public void Mending_keeps_a_card_meant_and_takes_the_first_that_may_join_it()
    {
        // The split card 1 shares a name with 2: 1 is kept, and 4, the first card that may join it, takes 2's place.
        (GameClient client, Judging transport) = Asked(TwoWithDifferentNames);
        client.Answer(new CardsChosenAnswer { Cards = [H(1), H(2)] });
        for (int i = 0; i < 20 && client.IsSettling; i++) client.Pump();

        Assert.Equal([H(1), H(4)], Answered(transport));
    }

    [Fact]
    public void Mending_goes_back_a_card_where_nothing_more_may_be_added()
    {
        // Here the split card 1 shares a name with every other card: kept, it reaches no answer, so mending gives it up and
        // takes the next card meant (2) and the first that may join it (3).
        static SelectionVerdictView Rule(IReadOnlyList<ViewHandle> picked) =>
            picked.Contains(H(1)) && picked.Count > 1 ? new SelectionVerdictView()
            : picked.Contains(H(1)) ? new SelectionVerdictView { Addable = [] }
            : TwoWithDifferentNames(picked);
        (GameClient client, Judging transport) = Asked(Rule);
        client.Answer(new CardsChosenAnswer { Cards = [H(1), H(2)] });
        for (int i = 0; i < 20 && client.IsSettling; i++) client.Pump();

        Assert.Equal([H(2), H(3)], Answered(transport));
    }

    [Fact]
    public void None_where_none_is_an_answer_goes_at_once_without_a_check()
    {
        (GameClient client, Judging transport) = Asked(TwoWithDifferentNames, noneIsAnswer: true);
        client.Answer(new CardsChosenAnswer());

        Assert.Empty(transport.Sent.OfType<CheckSelectionMessage>());
        Assert.Equal([], Answered(transport));
    }

    [Fact]
    public void A_selection_judged_while_picking_is_answered_without_asking_again()
    {
        (GameClient client, Judging transport) = Asked(TwoWithDifferentNames);
        client.Check([H(3), H(4)]);
        client.Check([H(4), H(3)]);
        client.Pump();
        Assert.Single(transport.Sent.OfType<CheckSelectionMessage>());
        Assert.True(client.Verdict([H(4), H(3)])!.Complete);

        client.Answer(new CardsChosenAnswer { Cards = [H(3), H(4)] });
        Assert.Equal([H(3), H(4)], Answered(transport));
    }
}
