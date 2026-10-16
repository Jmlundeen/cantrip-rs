using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Projection.Tests;

/// <summary>The log's wording, which the server writes for each seat from what that seat was sent.</summary>
public sealed class LogWriterTests
{
    private const int Me = 1;
    private const int Rival = 2;
    private static readonly ViewHandle Swamp = new(7);
    private static readonly ViewHandle Hidden = new(8);

    [Fact]
    public void Revealing_the_top_card_and_putting_it_into_hand_is_not_called_a_search()
    {
        Assert.Equal(
            ["You reveal [Swamp].", "You put [Swamp] into your hand."],
            Lines(
                new CardsRevealedEvent { Player = Me, Cards = [Swamp] },
                Move(Swamp, ZoneChangeReasonView.PutIntoHand, ZoneKindView.Hand)));
    }

    [Fact]
    public void Search_scry_and_surveil_keep_their_distinct_wording()
    {
        Assert.Equal(
            [
                "You search your library and take [Swamp].",
                "You put 1 card on the bottom.",
                "You surveil and put [Swamp] into your graveyard.",
            ],
            Lines(
                Move(Swamp, ZoneChangeReasonView.Searched, ZoneKindView.Hand),
                new LibraryArrangedEvent { Player = Me, OnBottom = 1 },
                Move(Swamp, ZoneChangeReasonView.Surveilled, ZoneKindView.Graveyard)));
    }

    [Fact]
    public void Day_and_night_is_read_out_as_the_rules_word_it()
    {
        // The designation belongs to the game (CR 731.1), so the line names nobody.
        Assert.Equal(
            ["It becomes night.", "It becomes day."],
            Lines(
                new DayNightBecameEvent { Designation = DayNightView.Night },
                new DayNightBecameEvent { Designation = DayNightView.Day }));
    }

    [Fact]
    public void A_coin_flip_and_a_die_roll_are_read_out_with_their_outcome()
    {
        Assert.Equal(
            ["You flip a coin: won the flip.", "Bot 2 flips a coin: lost the flip.", "You roll 4 on a d6."],
            Lines(
                new CoinFlippedEvent { Player = Me, Won = true },
                new CoinFlippedEvent { Player = Rival, Won = false },
                new DieRolledEvent { Player = Me, Sides = 6, Result = 4 }));
    }

    [Fact]
    public void A_turn_ended_by_an_effect_is_read_out_as_whose_turn_it_was()
    {
        // CR 724.1: Time Stop, Ultima and the like end the turn of the player whose turn it is, whoever cast them.
        Assert.Equal(
            ["Your turn ends.", "Bot 2's turn ends."],
            Lines(new TurnEndedEvent { Player = Me }, new TurnEndedEvent { Player = Rival }));
    }

    [Fact]
    public void Becoming_the_monarch_is_read_out_for_the_player_it_happened_to()
    {
        // The designation belongs to a player (CR 725.1), so unlike day and night the line names one, and names the
        // reader as "you".
        Assert.Equal(
            ["You become the monarch.", "Bot 2 becomes the monarch."],
            Lines(new MonarchBecameEvent { Player = Me }, new MonarchBecameEvent { Player = Rival }));
    }

    [Fact]
    public void Random_player_selection_names_its_source_and_is_readable_from_either_seat()
    {
        Assert.Equal(
            ["You choose Bot 2 at random for [Swamp].", "Bot 2 chooses you at random for [Swamp].", "You choose Bot 2 at random."],
            Lines(
                new RandomPlayerChosenEvent { Controller = Me, SelectedPlayer = Rival, Source = Swamp },
                new RandomPlayerChosenEvent { Controller = Rival, SelectedPlayer = Me, Source = Swamp },
                new RandomPlayerChosenEvent { Controller = Me, SelectedPlayer = Rival }));
    }

    [Fact]
    public void The_same_event_reads_as_you_for_its_player_and_by_name_for_the_other()
    {
        var drawn = Move(Swamp, ZoneChangeReasonView.Draw, ZoneKindView.Hand);

        Assert.Equal("You draw [Swamp].", LogWriter.Write(Me, View(Me), drawn)!.Text);
        Assert.Equal("Alice draws [Swamp].", LogWriter.Write(Rival, View(Rival), drawn)!.Text);
    }

    [Fact]
    public void A_card_the_seat_may_not_see_is_a_card_and_is_named_by_no_reference()
    {
        LogWriter.Line line = LogWriter.Write(Rival, View(Rival), new ObjectMovedEvent
        {
            Object = Hidden,
            From = ZoneKindView.Library,
            FromOwner = Me,
            To = ZoneKindView.Hand,
            ToOwner = Me,
            Reason = ZoneChangeReasonView.Draw,
        })!;

        Assert.Equal("Alice draws a card.", line.Text);
        Assert.Empty(line.References);
    }

    [Fact]
    public void Each_named_object_is_referenced_by_its_handle_in_the_order_the_line_names_it()
    {
        LogWriter.Line line = LogWriter.Write(Me, View(Me), new SpellCastEvent
        {
            Controller = Me,
            Spell = Swamp,
            Targets = [new TargetMessage { Player = Rival }, new TargetMessage { Object = Swamp }],
        })!;

        Assert.Equal("You cast [Swamp] targeting Bot 2 and [Swamp].", line.Text);
        Assert.Equal([("Swamp", Swamp), ("Swamp", Swamp)], line.References.Select(named => (named.Name, named.Object)));
    }

    [Fact]
    public void A_land_played_from_exile_is_played_by_the_active_player()
    {
        // Exile is nobody's zone and neither is the battlefield, so the move names no player; only the active player
        // plays lands (CR 305.1).
        PlayerView view = View(Me);
        view.ActivePlayer = Me;

        LogWriter.Line line = LogWriter.Write(Me, view, new ObjectMovedEvent
        {
            Object = Swamp,
            From = ZoneKindView.Exile,
            To = ZoneKindView.Battlefield,
            Reason = ZoneChangeReasonView.Play,
        })!;

        Assert.Equal("You play [Swamp].", line.Text);
    }

    [Fact]
    public void A_loss_is_marked_as_one_for_the_game_over_panel()
    {
        LogWriter.Line line = LogWriter.Write(Me, View(Me), new PlayerLostEvent { Player = Rival, Reason = LossReasonView.Life })!;

        Assert.Equal("Bot 2 loses the game: their life total is 0.", line.Text);
        Assert.Equal(LogEntryKind.PlayerLost, line.Kind);
    }

    [Fact]
    public void A_modal_triggered_ability_names_its_mode_as_a_modal_spell_does()
    {
        // CR 603.3c, 700.2: the mode is announced as the ability goes on the stack, a mode with no target included.
        Assert.Equal(
            [
                "[Swamp] triggers choosing \"Create a Treasure token\".",
                "[Swamp] triggers choosing \"Put a +1/+1 counter on target Wolf you control\", targeting [Swamp].",
            ],
            Lines(
                new AbilityActivatedEvent
                {
                    Controller = Rival, Source = Swamp, Triggered = true, Text = "Whenever ...", Modes = ["Create a Treasure token."],
                },
                new AbilityActivatedEvent
                {
                    Controller = Rival,
                    Source = Swamp,
                    Triggered = true,
                    Text = "Whenever ...",
                    Modes = ["Put a +1/+1 counter on target Wolf you control."],
                    Targets = [new TargetMessage { Object = Swamp }],
                }));
    }

    private static List<string> Lines(params GameEventView[] events) =>
        [.. events.Select(entry => LogWriter.Write(Me, View(Me), entry)?.Text).OfType<string>()];

    /// <summary>
    /// Two seats, Alice and Bot 2, and a Swamp both can see. Handle 8 is a card in Alice's hand: she sees it, Bot 2
    /// does not, so in Bot 2's view it is only a back.
    /// </summary>
    private static PlayerView View(int seat) => new()
    {
        Viewer = seat,
        Players =
        [
            new PlayerSummaryView { Seat = Me, Name = "Alice", Life = 20, Team = 1 },
            new PlayerSummaryView { Seat = Rival, Name = "Bot 2", Life = 0, Team = 2 },
        ],
        Objects =
        [
            new ObjectView { Handle = Swamp, Name = "Swamp", Known = true },
            seat == Me
                ? new ObjectView { Handle = Hidden, Name = "Island", Known = true }
                : new ObjectView { Handle = Hidden, Known = false },
        ],
    };

    private static ObjectMovedEvent Move(ViewHandle card, ZoneChangeReasonView reason, ZoneKindView to) => new()
    {
        Object = card,
        From = ZoneKindView.Library,
        FromOwner = Me,
        To = to,
        ToOwner = Me,
        Reason = reason,
    };
}
