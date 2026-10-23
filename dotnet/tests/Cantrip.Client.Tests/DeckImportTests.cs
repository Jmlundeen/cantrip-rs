using Cantrip.Decks;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// Decklists as Magic Online and Arena export them, read into a deck: sections headed and unheaded, sets, collector
/// numbers and foil marks ignored, split and double-faced cards by either way of writing them, and the cards the engine
/// does not play listed rather than imported.
/// </summary>
public sealed class DeckImportTests
{
    private static readonly Func<string, string?> Find = DeckImport.Matcher(
        ["Lightning Bolt", "Mountain", "Smash to Smithereens", "Bedeck // Bedazzle", "Delver of Secrets", "Balin, Loremaster", "Lurrus of the Dream-Den"]);

    private static string[] Listed(List<DeckEntry> entries) => [.. entries.Select(entry => $"{entry.Count} {entry.Card}")];

    [Fact]
    public void An_Arena_export_reads_its_sections_and_name_and_ignores_the_printings()
    {
        DeckImport.Result read = DeckImport.Read("""
            About
            Name Red Burn

            Commander
            1 Balin, Loremaster (LTC) 12

            Companion
            1 Lurrus of the Dream-Den (IKO) 226

            Deck
            4 Lightning Bolt (M10) 146
            20 Mountain (ANA) 59 *F*
            1 Bedeck // Bedazzle (RNA) 221

            Sideboard
            2 Smash to Smithereens (ALA) 117
            """, Find);

        Assert.Empty(read.Unsupported);
        Assert.Equal("Red Burn", read.Deck.Name);
        Assert.Equal(["1 Balin, Loremaster"], Listed(read.Deck.Commanders));
        Assert.Equal(["4 Lightning Bolt", "20 Mountain", "1 Bedeck // Bedazzle"], Listed(read.Deck.Main));
        Assert.Equal(["2 Smash to Smithereens"], Listed(read.Deck.Side));
        Assert.Equal("Lurrus of the Dream-Den", read.Deck.Companion);
    }

    /// <summary>Arena lists the companion under Companion and again under Sideboard: it is the deck's companion, one card (CR 702.139a).</summary>
    [Fact]
    public void An_Arena_companion_listed_in_the_sideboard_too_is_one_card()
    {
        DeckImport.Result read = DeckImport.Read("""
            Companion
            1 Lurrus of the Dream-Den (IKO) 226

            Deck
            20 Mountain (ANA) 59

            Sideboard
            2 Smash to Smithereens (ALA) 117
            1 Lurrus of the Dream-Den (IKO) 226
            """, Find);

        Assert.Equal(["2 Smash to Smithereens"], Listed(read.Deck.Side));
        Assert.Equal("Lurrus of the Dream-Den", read.Deck.Companion);
        Assert.Equal(["20 Mountain"], Listed(read.Deck.Main));
    }

    [Fact]
    public void A_Magic_Online_export_takes_the_sideboard_after_the_first_blank_line()
    {
        DeckImport.Result read = DeckImport.Read("4 Lightning Bolt\r\n20 Mountain\r\n1 Bedeck/Bedazzle\r\n\r\n2 Smash to Smithereens\r\n", Find);
        Assert.Equal(Listed(read.Deck.Side), Listed(DeckImport.Read("4 Lightning Bolt\r20 Mountain\r1 Bedeck/Bedazzle\r\r2 Smash to Smithereens\r", Find).Deck.Side));

        Assert.Equal(["4 Lightning Bolt", "20 Mountain", "1 Bedeck // Bedazzle"], Listed(read.Deck.Main));
        Assert.Equal(["2 Smash to Smithereens"], Listed(read.Deck.Side));
        Assert.Equal(string.Empty, read.Deck.Name);
    }

    [Fact]
    public void Counts_written_with_x_set_codes_in_brackets_SB_prefixes_and_repeated_cards_all_read()
    {
        DeckImport.Result read = DeckImport.Read("""
            // Burn
            4x Lightning Bolt [M10]
            Lands (22)
            20 mountain
            2 Mountain
            SB: 2 Smash to Smithereens
            1 Delver of Secrets // Insectile Aberration
            Maybeboard
            1 Lightning Bolt
            """, Find);

        Assert.Equal(["4 Lightning Bolt", "22 Mountain", "1 Delver of Secrets"], Listed(read.Deck.Main));
        Assert.Equal(["2 Smash to Smithereens"], Listed(read.Deck.Side));
        Assert.Empty(read.Unsupported);
    }

    [Fact]
    public void Cards_the_engine_does_not_play_are_listed_once_each_and_left_out()
    {
        DeckImport.Result read = DeckImport.Read("""
            Deck
            4 Lightning Bolt (M10) 146
            4 Goblin Guide (ZEN) 126
            2 Not A Real Card

            Sideboard
            2 Goblin Guide
            """, Find);

        Assert.Equal(["Goblin Guide", "Not A Real Card"], read.Unsupported);
        Assert.Equal(["4 Lightning Bolt"], Listed(read.Deck.Main));
        Assert.Empty(read.Deck.Side);
        Assert.False(read.Empty);
    }

    [Fact]
    public void A_list_with_no_cards_is_empty()
    {
        Assert.True(DeckImport.Read("\n  \nDeck\n\nSideboard\n", Find).Empty);
    }
}
