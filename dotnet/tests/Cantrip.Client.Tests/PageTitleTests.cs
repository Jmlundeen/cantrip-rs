using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>What the browser tab is named for each part of a table and an event.</summary>
public sealed class PageTitleTests
{
    [Fact]
    public void Before_the_table_is_heard_from_the_tab_is_the_app()
    {
        Assert.Equal("Cantrip", PageTitle.For(null, 1, null, null, null));
    }

    [Theory]
    [InlineData("winston", "Cantrip - Winston Draft (evt-7)")]
    [InlineData("grid", "Cantrip - Grid Draft (evt-7)")]
    [InlineData("booster", "Cantrip - Booster Draft (evt-7)")]
    public void A_draft_is_named_for_its_format_and_event(string format, string expected)
    {
        TableMessage table = Table(TablePhase.Drafting, "evt-7");

        Assert.Equal(expected, PageTitle.For(table, 1, new DraftMessage { EventId = "evt-7", Format = format }, null, null));
    }

    [Fact]
    public void Deck_building_and_sideboarding_are_told_apart()
    {
        TableMessage table = Table(TablePhase.DeckBuilding, "evt-7");

        Assert.Equal("Cantrip - Deck Building (evt-7)", PageTitle.For(table, 1, null, new DeckBuildingMessage(), null));
        Assert.Equal("Cantrip - Sideboarding (evt-7)",
            PageTitle.For(table, 1, null, new DeckBuildingMessage { Title = "Sideboarding for game 2 - you lead 1-0" }, null));
    }

    [Fact]
    public void Pairings_name_the_round_and_then_the_final_standings()
    {
        TableMessage table = Table(TablePhase.Pairings, "evt-7");

        Assert.Equal("Cantrip - Round 2 of 3 (evt-7)", PageTitle.For(table, 1, null, null, new PairingsMessage { Round = 2, Rounds = 3 }));
        Assert.Equal("Cantrip - Final Standings (evt-7)", PageTitle.For(table, 1, null, null, new PairingsMessage { Round = 3, Rounds = 3, Final = true }));
    }

    [Fact]
    public void A_game_names_the_opponents_and_a_table_of_its_own_has_no_event()
    {
        TableMessage table = Table(TablePhase.Playing, string.Empty);
        table.Players = [new TablePlayerView { Seat = 1, Name = "Ann" }, new TablePlayerView { Seat = 2, Name = "Bob" }];

        Assert.Equal("Cantrip - Game vs Bob", PageTitle.For(table, 1, null, null, null));
        Assert.Equal("Cantrip - Game vs Ann", PageTitle.For(table, 2, null, null, null));
    }

    [Fact]
    public void A_table_being_set_up_is_a_lobby_or_a_forming_table()
    {
        Assert.Equal("Cantrip - Lobby", PageTitle.For(Table(TablePhase.Setup, string.Empty), 1, null, null, null));

        TableMessage forming = Table(TablePhase.Setup, "evt-7");
        forming.Forming = true;
        Assert.Equal("Cantrip - Forming Table (evt-7)", PageTitle.For(forming, 1, null, null, null));
    }

    private static TableMessage Table(TablePhase phase, string eventId) => new() { Phase = phase, EventId = eventId };
}
