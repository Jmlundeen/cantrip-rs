using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Llm.Tests.Board;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// The seat itself: what it sends, what it does when it is refused, and what it remembers.
/// <para>
/// A model that answers badly has to cost the game nothing but a bad move. These prove the two ways that could fail
/// — a refused answer sent back forever, and a model that will not answer at all — end in the prompt's own answer
/// and a game that carries on.
/// </para>
/// </summary>
public sealed class LlmPlayTests
{
    [Fact]
    public void Falls_back_to_the_games_own_answer_once_the_same_question_has_been_refused()
    {
        // A refusal is the identical prompt sent back, which is the only notice this seat gets. Not every rule an
        // answer is judged by is in the prompt, so trying again is right up to a point — and then it has to stop, or
        // the server and the seat trade the same question until somebody runs out of credits.
        var model = new ScriptedModel("A", "A", "A", "A", "A", "A");
        var transport = new Held();
        var client = new GameClient(transport);
        var bot = new LlmPlay(model);

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View(Card(2, "Lightning Bolt", "{R}")) });

        List<AnswerPayload> answers = [];

        for (int i = 0; i < 4; i++)
        {
            transport.Inbound.Enqueue(new PromptMessage { RequestId = 1, Prompt = Cast() });
            client.Pump();

            Assert.True(bot.Act(client), "stopped answering a question that is still outstanding");
            answers.Add(((AnswerMessage)transport.Sent[^1]).Answer);
        }

        // It tried twice, then took the prompt's own answer — passing — and stopped claiming it had another to try,
        // so a harness can report the refusal rather than spin on it.
        Assert.False(bot.Retrying);
        Assert.All(answers.Skip(2), answer => Assert.IsType<PassAnswer>(answer));
        Assert.True(bot.Fallbacks > 0);
    }

    [Fact]
    public void Treats_a_new_question_as_a_new_question()
    {
        var transport = new Held();
        var client = new GameClient(transport);
        var bot = new LlmPlay(new ScriptedModel([.. Enumerable.Repeat("B", 8)]));

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View(Card(2, "Lightning Bolt", "{R}")) });

        for (int id = 1; id <= 5; id++)
        {
            transport.Inbound.Enqueue(new PromptMessage { RequestId = id, Prompt = Cast() });
            client.Pump();

            Assert.True(bot.Act(client));
            Assert.True(bot.Retrying, "a fresh question was treated as a refused one");
        }
    }

    [Fact]
    public void Tells_the_model_what_happened_while_it_was_not_being_asked()
    {
        // Most of a game happens on somebody else's turn. A handle only names a card while this seat can still see
        // it, so the line has to be written as the event arrives, not when the next question is put.
        var model = new ScriptedModel("B");
        var transport = new Held();
        var client = new GameClient(transport);
        var bot = new LlmPlay(model);

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage
        {
            View = View(Card(2, "Lightning Bolt", "{R}"), Creature(6, "Serra Angel", 4, 4, Them)),
        });
        transport.Inbound.Enqueue(new EventsMessage
        {
            Sequence = 4,
            Sequences = [3, 4],
            Events =
            [
                new SpellCastEvent { Spell = Handle(6), Controller = Them },
                new LifeChangedEvent { Player = Me, Delta = -4, NewTotal = 16 },
            ],
        });

        client.Pump();
        bot.Act(client);

        // Nothing was asked, so nothing was sent yet — but the lines are held for the next question.
        transport.Inbound.Enqueue(new PromptMessage { RequestId = 1, Prompt = Cast() });
        client.Pump();
        bot.Act(client);

        string shown = model.Heard[0];
        Assert.Contains("Since you last acted", shown, StringComparison.Ordinal);
        Assert.Contains("Opponent cast Serra Angel", shown, StringComparison.Ordinal);
        Assert.Contains("you lost 4 life", shown, StringComparison.Ordinal);
    }

    [Fact]
    public void Puts_its_deck_in_front_of_the_model_once()
    {
        // What is left in the library is most of what a decision turns on, and it is the one thing no projection of
        // the position can say. It goes in the standing instructions, so it is sent once and not per decision.
        var model = new ScriptedModel("B", "B");
        var transport = new Held();
        var client = new GameClient(transport);
        var bot = new LlmPlay(model, "4 Lightning Bolt\n20 Mountain");

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View(Card(2, "Lightning Bolt", "{R}")) });
        transport.Inbound.Enqueue(new PromptMessage { RequestId = 1, Prompt = Cast() });
        client.Pump();
        bot.Act(client);

        Assert.DoesNotContain("20 Mountain", model.Heard[0], StringComparison.Ordinal);
    }

    [Fact]
    public void Answers_nothing_when_it_is_not_being_asked()
    {
        var transport = new Held();
        var client = new GameClient(transport);
        var bot = new LlmPlay(new ScriptedModel());

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View() });
        client.Pump();

        Assert.False(bot.Act(client));
        Assert.Empty(transport.Sent.OfType<AnswerMessage>());
    }

    private static PriorityPromptMessage Cast() => new()
    {
        Player = Me,
        Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(2), Affordable = true }],
    };
}
