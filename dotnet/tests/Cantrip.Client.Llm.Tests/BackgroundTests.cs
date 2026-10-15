using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Llm.Tests.Board;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// The seat as a person plays against it: thinking off the calling thread, so the window keeps drawing.
/// <para>
/// A model takes seconds. A client that waited for one inside its frame would stop rendering for those seconds and
/// look, to the person across the table, exactly like a client that had crashed. So the decision is handed off and
/// picked up a frame or two later, and the two things that has to get right are proved here: that an answer
/// eventually arrives, and that the client is left strictly alone in the meantime.
/// </para>
/// </summary>
public sealed class BackgroundTests
{
    [Fact]
    public void Answers_nothing_while_it_is_thinking_and_then_answers()
    {
        var model = new Slow();
        var bot = new LlmPlay(model, deck: string.Empty, background: true);
        (GameClient client, Held transport) = Asked();

        // The first call starts the thinking and answers nothing. The frame loop carries on drawing.
        Assert.False(bot.Act(client));
        Assert.True(bot.Thinking);
        Assert.Empty(transport.Sent.OfType<AnswerMessage>());

        model.Finish("A");

        // Polled each frame until it lands.
        Assert.True(Eventually(() => bot.Act(client)), "the decision never came back");
        Assert.False(bot.Thinking);
        Assert.IsType<TakeActionAnswer>(Assert.Single(transport.Sent.OfType<AnswerMessage>()).Answer);
    }

    [Fact]
    public void Falls_back_to_the_games_own_answer_when_the_model_cannot_be_reached()
    {
        // The failure a networked opponent has that a local one does not. It must cost the game a bad move and
        // nothing else: a person mid-game does not lose the game because a provider fell over.
        var bot = new LlmPlay(new Broken(), deck: string.Empty, background: true);
        (GameClient client, Held transport) = Asked();

        Assert.False(bot.Act(client));
        Assert.True(Eventually(() => bot.Act(client)), "a failed decision never came back");

        Assert.IsType<PassAnswer>(Assert.Single(transport.Sent.OfType<AnswerMessage>()).Answer);
        Assert.Equal(1, bot.Fallbacks);
        Assert.NotEmpty(bot.Failure);
    }

    [Fact]
    public void Says_it_is_thinking_so_a_seat_driving_it_leaves_its_client_alone()
    {
        // AutoSeat stops pumping while this is true, which is the whole reason the background thread may read the
        // client at all: the position it is reading cannot change underneath it.
        var model = new Slow();
        var seat = new AutoSeat(Prompted(), new LlmPlay(model, deck: string.Empty, background: true));

        Assert.False(seat.Step());
        Assert.True(seat.Thinking);

        model.Finish("A");
        Assert.True(Eventually(seat.Step), "the seat never answered");
        Assert.False(seat.Thinking);
    }

    [Fact]
    public void Blocks_instead_when_it_was_not_asked_to_think_elsewhere()
    {
        // The harness's way round, which is simpler to read and to reason about when there is no window to keep alive.
        var bot = new LlmPlay(new ScriptedModel("A"), deck: string.Empty);
        (GameClient client, Held transport) = Asked();

        Assert.True(bot.Act(client));
        Assert.False(bot.Thinking);
        Assert.Single(transport.Sent.OfType<AnswerMessage>());
    }

    /// <summary>A client with one question outstanding, and the transport it would answer down.</summary>
    private static (GameClient Client, Held Transport) Asked()
    {
        var transport = Prompted();
        var client = new GameClient(transport);
        client.Pump();
        return (client, transport);
    }

    private static Held Prompted()
    {
        var transport = new Held();
        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View(Card(2, "Lightning Bolt", "{R}")) });
        transport.Inbound.Enqueue(new PromptMessage
        {
            RequestId = 1,
            Prompt = new PriorityPromptMessage
            {
                Player = Me,
                Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(2), Affordable = true }],
            },
        });

        return transport;
    }

    /// <summary>Polls as a frame loop would, rather than sleeping a guessed interval.</summary>
    private static bool Eventually(Func<bool> act)
    {
        for (int frame = 0; frame < 500; frame++)
        {
            if (act())
            {
                return true;
            }

            Thread.Sleep(10);
        }

        return false;
    }

    /// <summary>A model that does not answer until a test says so, standing in for a slow network.</summary>
    private sealed class Slow : ILanguageModel
    {
        private readonly ManualResetEventSlim _held = new(initialState: false);
        private string _reply = string.Empty;

        public ModelUsage Usage { get; } = new();

        public void Finish(string reply)
        {
            _reply = reply;
            _held.Set();
        }

        public string Continue(IReadOnlyList<ChatTurn> conversation)
        {
            _held.Wait(TimeSpan.FromSeconds(10));
            Usage.Record(1, 1);
            return _reply;
        }
    }

    /// <summary>A model that cannot be reached at all.</summary>
    private sealed class Broken : ILanguageModel
    {
        public ModelUsage Usage { get; } = new();

        public string Continue(IReadOnlyList<ChatTurn> conversation) =>
            throw new HttpRequestException("the provider is down");
    }
}
