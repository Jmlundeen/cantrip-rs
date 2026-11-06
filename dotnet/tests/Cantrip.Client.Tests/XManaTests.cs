using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// "Spend only black mana on X" (CR 107.3, 601.2h: Drain Life, Crypt Rats; Emblazoned Golem's "Spend only colored mana on X.
/// No more than one mana of each color may be spent this way"), as a client reads it from the prompt
/// (<see cref="PayCostsPromptMessage.XMana"/>, <see cref="PayCostsPromptMessage.RemainingX"/>): said in words, the X the
/// planned mana reaches counting only the mana X may spend, and X's part owed as that mana, so a bot or the page taps a
/// source whose mana pays it.
/// </summary>
public sealed class XManaTests
{
    private static PayCostsPromptMessage Drain(int x = 0) =>
        new() { Total = "{X}{1}{B}", Remaining = "{X}{1}{B}", HasX = true, X = x, AnnouncesXOrKicks = true, MostAnnounced = 1000, XMana = [ManaTypeView.Black] };

    [Fact]
    public void The_restriction_is_said_in_words()
    {
        Assert.Equal("X: black mana only", Announcement.XManaNote(Drain()));
        Assert.Equal("X: black or red mana only", Announcement.XManaNote(new PayCostsPromptMessage { XMana = [ManaTypeView.Black, ManaTypeView.Red] }));
        Assert.Equal(
            "X: colored mana only, one of each color at most",
            Announcement.XManaNote(new PayCostsPromptMessage
            {
                XMana = [ManaTypeView.White, ManaTypeView.Blue, ManaTypeView.Black, ManaTypeView.Red, ManaTypeView.Green],
                XManaOneEach = true,
            }));
        Assert.Equal(string.Empty, Announcement.XManaNote(new PayCostsPromptMessage { Total = "{X}{R}" }));
    }

    [Theory]
    // Five mana, three of them black: the {B} takes one, so X is 2, not five mana's 3.
    [InlineData(5, 3, 2)]
    // No black but the {B}'s: X 0.
    [InlineData(5, 1, 0)]
    // Black enough: the mana itself is the limit.
    [InlineData(4, 4, 2)]
    public void The_x_planned_mana_reaches_counts_only_the_mana_x_may_spend(int mana, int black, int expected) =>
        Assert.Equal(expected, Announcement.XPaidBy(Drain(), mana, black));

    [Fact]
    public void One_of_each_color_reaches_as_far_as_the_colors_there_are()
    {
        var golem = new PayCostsPromptMessage
        {
            Total = "{X}{2}",
            HasX = true,
            AnnouncesXOrKicks = true,
            MostAnnounced = 1000,
            XMana = [ManaTypeView.White, ManaTypeView.Blue, ManaTypeView.Black, ManaTypeView.Red, ManaTypeView.Green],
            XManaOneEach = true,
        };
        Assert.Equal(2, Announcement.XPaidBy(golem, mana: 6, forX: 2));
    }

    [Fact]
    public void X_part_is_owed_as_the_mana_that_may_pay_it()
    {
        var paying = new PayCostsPromptMessage { HasX = true, RemainingGeneric = 3, RemainingX = 2, RemainingColored = [ColorView.Black], XMana = [ManaTypeView.Black] };
        OwedMana owed = OwedMana.Of(paying);
        Assert.Equal([ColorView.Black, ColorView.Black, ColorView.Black], owed.Colored);
        Assert.Equal(1, owed.Generic);

        // What is tapped or exiled instead of mana pays X's part first, as the server has it.
        OwedMana less = OwedMana.Of(paying, instead: 1);
        Assert.Equal([ColorView.Black, ColorView.Black], less.Colored);
        Assert.Equal(1, less.Generic);

        // Two types: each of X's part a symbol either pays.
        var soulBurn = new PayCostsPromptMessage { RemainingGeneric = 2, RemainingX = 1, XMana = [ManaTypeView.Black, ManaTypeView.Red] };
        OwedMana either = OwedMana.Of(soulBurn);
        Assert.True(either.Hybrid.Single().Accepts(ColorView.Red));
        Assert.Equal(1, either.Generic);
    }

    [Fact]
    public void A_bot_taps_a_swamp_for_black_only_x_where_a_mountain_would_pay_nothing_of_it()
    {
        var mountain = new ViewHandle(1);
        var swamp = new ViewHandle(2);
        var prompt = new PayCostsPromptMessage
        {
            Spell = new ViewHandle(3),
            Total = "{X}{3}{B}",
            Remaining = "{2}",
            HasX = true,
            X = 2,
            RemainingGeneric = 2,
            RemainingX = 2,
            XMana = [ManaTypeView.Black],
            Options =
            [
                new ActionOptionMessage { Kind = ActionKindView.ActivateAbility, Source = mountain, Produces = [ManaTypeView.Red] },
                new ActionOptionMessage { Kind = ActionKindView.ActivateAbility, Source = swamp, Produces = [ManaTypeView.Black] },
                new ActionOptionMessage { Kind = ActionKindView.CancelCasting },
            ],
        };

        TakeActionAnswer answer = AutoPlay.Pay(prompt);
        Assert.Equal(ActionKindView.ActivateAbility, answer.Kind);
        Assert.Equal(swamp, answer.Source);
    }

    /// <summary>
    /// An X spell under a tax ("{1} more", CR 601.2f) whose announcement says X 2 could be paid where only 1 can: the seat
    /// announces 2, can't pay it - the payment offers no lower X - and gives the cast up (CR 733.1), back at the same priority
    /// with the same spell offered. Asked again, it announces 1, not 2 for ever; and had 1 been given up as well, it passes
    /// rather than cast it a third time.
    /// </summary>
    [Fact]
    public void A_bot_that_cannot_pay_its_x_announces_less_and_then_passes_rather_than_the_same_x_again()
    {
        var table = new TacticsBoard.Table([]);
        var drain = new ViewHandle(7);
        int asked = 100;

        AnswerPayload Ask(PromptPayload prompt)
        {
            table.Ask(++asked, prompt);
            Assert.True(AutoPlay.Act(table.Client));
            return table.Sent.OfType<AnswerMessage>().Last().Answer;
        }

        var priority = new PriorityPromptMessage
        {
            Player = TacticsBoard.Me,
            Options = [new ActionOptionMessage { Kind = ActionKindView.CastSpell, Source = drain, Affordable = true, HasX = true, MaxX = 2 }],
        };

        // The announcement, its advice two where the tax leaves one.
        PayCostsPromptMessage Announcement(int x) => new()
        {
            Player = TacticsBoard.Me,
            Spell = drain,
            Total = "{X}{1}{B}",
            HasX = true,
            X = x,
            AnnouncesXOrKicks = true,
            MostAffordable = 2,
            MostAnnounced = 1000,
            XMana = [ManaTypeView.Black],
            Options =
            [
                new ActionOptionMessage { Kind = ActionKindView.IncreaseX },
                .. x > 0 ? [new ActionOptionMessage { Kind = ActionKindView.DecreaseX }] : Array.Empty<ActionOptionMessage>(),
                new ActionOptionMessage { Kind = ActionKindView.ConfirmCasting },
                new ActionOptionMessage { Kind = ActionKindView.CancelCasting },
            ],
        };

        // The X it announces: raised one step at a time from 0, then confirmed.
        int Announced()
        {
            for (int x = 0; ; x++)
            {
                var answer = Assert.IsType<TakeActionAnswer>(Ask(Announcement(x)));
                if (answer.Kind == ActionKindView.ConfirmCasting)
                {
                    return x;
                }

                Assert.Equal(ActionKindView.IncreaseX, answer.Kind);
                Assert.True(x < 10, "it never stops raising X");
            }
        }

        // The payment it can't finish: the {B} and the tax are owed and nothing is left to make them, nor a lower X.
        var payment = new PayCostsPromptMessage
        {
            Player = TacticsBoard.Me,
            Spell = drain,
            Total = "{X}{1}{B}",
            Remaining = "{1}{B}",
            HasX = true,
            RemainingGeneric = 1,
            RemainingColored = [ColorView.Black],
            XMana = [ManaTypeView.Black],
            Options = [new ActionOptionMessage { Kind = ActionKindView.CancelCasting }],
        };

        Assert.Equal(ActionKindView.CastSpell, Assert.IsType<TakeActionAnswer>(Ask(priority)).Kind);
        Assert.Equal(2, Announced());
        Assert.Equal(ActionKindView.CancelCasting, Assert.IsType<TakeActionAnswer>(Ask(payment)).Kind);

        // Back at priority with the same spell: cast again, for less.
        Assert.Equal(ActionKindView.CastSpell, Assert.IsType<TakeActionAnswer>(Ask(priority)).Kind);
        Assert.Equal(1, Announced());

        // Had X 1 not been paid either, nothing with X is cast again this turn.
        Assert.Equal(ActionKindView.CancelCasting, Assert.IsType<TakeActionAnswer>(Ask(payment)).Kind);
        Assert.IsType<PassAnswer>(Ask(priority));
    }
}
