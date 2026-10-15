using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>The client's side of the log: the server's lines set out under turn and step headings, and nothing else.</summary>
public sealed class GameJournalTests
{
    [Fact]
    public void The_servers_lines_are_set_out_under_a_heading_for_each_turn_and_step_something_happened_in()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        inbox.Enqueue(new LogMessage
        {
            Entries =
            [
                Entry(1, turn: 0, active: 0, step: null, "You mulligan."),
                Entry(2, turn: 1, active: 1, TurnStepView.PrecombatMain, "You play [Swamp]."),
                Entry(3, turn: 1, active: 1, TurnStepView.PrecombatMain, "You cast [Duress]."),
                Entry(4, turn: 2, active: 2, TurnStepView.Draw, "Bot 2 draws a card."),
            ],
        });

        var journal = new GameJournal();
        client.Pump();
        journal.Follow(client);

        Assert.Equal(
            [
                "You mulligan.",
                "You Turn 1", "Precombat Main:", "You play [Swamp].", "You cast [Duress].",
                "Bot 2 Turn 2", "Draw:", "Bot 2 draws a card.",
            ],
            journal.Lines.Select(line => line.Text));
    }

    [Fact]
    public void The_whole_log_sent_again_replaces_the_lines_and_says_they_changed()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        var journal = new GameJournal();

        inbox.Enqueue(new LogMessage { Entries = [Entry(1, 1, 1, TurnStepView.Upkeep, "It becomes night.")] });
        client.Pump();
        journal.Follow(client);
        int before = journal.Revision;

        // A reload or a reconnect: the server's own copy, which has what this client missed as well.
        inbox.Enqueue(new LogMessage
        {
            Complete = true,
            Entries = [Entry(1, 1, 1, TurnStepView.Upkeep, "It becomes night."), Entry(2, 1, 1, TurnStepView.Draw, "You draw [Island].")],
        });
        client.Pump();
        journal.Follow(client);

        Assert.True(journal.Revision > before);
        Assert.Equal(["You Turn 1", "Upkeep:", "It becomes night.", "Draw:", "You draw [Island]."], journal.Lines.Select(line => line.Text));
    }

    [Fact]
    public void A_line_taken_back_leaves_no_empty_heading_behind()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        var journal = new GameJournal();

        inbox.Enqueue(new LogMessage
        {
            Entries = [Entry(1, 1, 1, TurnStepView.Upkeep, "It becomes night."), Entry(2, 1, 1, TurnStepView.PrecombatMain, "You tap [Mox Jet].")],
        });
        client.Pump();
        journal.Follow(client);

        inbox.Enqueue(new LogMessage { Removed = [2] });
        client.Pump();
        journal.Follow(client);

        Assert.Equal(["You Turn 1", "Upkeep:", "It becomes night."], journal.Lines.Select(line => line.Text));
    }

    [Fact]
    public void The_same_line_said_again_and_again_is_one_line_counted()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        inbox.Enqueue(new LogMessage
        {
            Entries =
            [
                Entry(1, turn: 1, active: 1, TurnStepView.PrecombatMain, "You create a Human Knight token."),
                Entry(2, turn: 1, active: 1, TurnStepView.PrecombatMain, "You create a Human Knight token."),
                Entry(3, turn: 1, active: 1, TurnStepView.PrecombatMain, "You create a Human Knight token."),
                Entry(4, turn: 1, active: 1, TurnStepView.PrecombatMain, "[Forth Eorlingas!] resolves."),
                Entry(5, turn: 1, active: 1, TurnStepView.PrecombatMain, "You create a Human Knight token."),
            ],
        });

        var journal = new GameJournal();
        client.Pump();
        journal.Follow(client);

        Assert.Equal(
            [
                ("You Turn 1", 1), ("Precombat Main:", 1),
                ("You create a Human Knight token.", 3), ("[Forth Eorlingas!] resolves.", 1), ("You create a Human Knight token.", 1),
            ],
            journal.Lines.Select(line => (line.Text, line.Count)));
        Assert.Equal(1, journal.Lines[2].Sequence);
    }

    [Fact]
    public void The_same_line_under_another_heading_is_a_line_of_its_own()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        inbox.Enqueue(new LogMessage
        {
            Entries =
            [
                Entry(1, turn: 1, active: 1, TurnStepView.PrecombatMain, "You tap [Mox Pearl]."),
                Entry(2, turn: 1, active: 1, TurnStepView.PostcombatMain, "You tap [Mox Pearl]."),
                Entry(3, turn: 3, active: 1, TurnStepView.PostcombatMain, "You tap [Mox Pearl]."),
            ],
        });

        var journal = new GameJournal();
        client.Pump();
        journal.Follow(client);

        Assert.Equal(3, journal.Lines.Count(line => line.Kind == LogLineKind.Entry));
        Assert.All(journal.Lines, line => Assert.Equal(1, line.Count));
    }

    [Fact]
    public void A_line_said_again_in_a_later_batch_replaces_the_last_line_with_its_count_raised()
    {
        (GameClient client, Queue<ServerMessage> inbox) = Client();
        inbox.Enqueue(new LogMessage { Entries = [Entry(1, turn: 1, active: 1, TurnStepView.PrecombatMain, "You tap [Basalt Monolith].")] });

        var journal = new GameJournal();
        client.Pump();
        journal.Follow(client);
        LogLine first = journal.Lines[^1];
        int count = journal.Lines.Count;

        inbox.Enqueue(new LogMessage { Entries = [Entry(2, turn: 1, active: 1, TurnStepView.PrecombatMain, "You tap [Basalt Monolith].")] });
        client.Pump();
        journal.Follow(client);

        Assert.Equal(count, journal.Lines.Count);
        Assert.NotSame(first, journal.Lines[^1]);
        Assert.Equal(2, journal.Lines[^1].Count);
        Assert.Equal(0, journal.Revision);
    }

    private static LogEntryView Entry(long number, int turn, int active, TurnStepView? step, string text) => new()
    {
        Number = number,
        Sequence = number,
        Turn = turn,
        ActivePlayer = active,
        Step = step,
        Text = text,
    };

    private static (GameClient Client, Queue<ServerMessage> Inbox) Client()
    {
        var transport = new QueueTransport();
        transport.Inbox.Enqueue(new WelcomeMessage { Seat = 1, PlayerNames = ["You", "Bot 2"] });
        transport.Inbox.Enqueue(new StateMessage
        {
            View = new PlayerView
            {
                Viewer = 1,
                Players = [new PlayerSummaryView { Seat = 1, Name = "You" }, new PlayerSummaryView { Seat = 2, Name = "Bot 2" }],
            },
        });

        var client = new GameClient(transport);
        client.Pump();
        return (client, transport.Inbox);
    }

    /// <summary>Hands the client whatever has been queued for it by the time it pumps.</summary>
    private sealed class QueueTransport : IGameTransport
    {
        public Queue<ServerMessage> Inbox { get; } = new();

        public bool IsConnected => true;

        public void Send(ClientMessage message)
        {
        }

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> taken = [.. Inbox];
            Inbox.Clear();
            return taken;
        }

        public void Dispose()
        {
        }
    }
}
