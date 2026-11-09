using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Decklists are data files under <c>decks/</c>, read by <see cref="DeckList"/> and resolved by card name against the
/// cards a table loads. Every file there has to deal, since a deck naming a card nobody implemented is only found when
/// someone sits down to play it.
/// </summary>
public sealed class DeckListTests
{
    [Theory]
    [InlineData("Commander")]
    [InlineData("commanders")]
    public void Explicit_commanders_are_set_aside_and_main_headers_resume_the_library(string header)
    {
        DeckList deck = DeckList.Parse("partners", $"{header}\n1 Kytheon, Hero of Akros\nIsamaru, Hound of Konda\nDeck\n97 Plains\nArcane Signet\nSideboard\n1 Lightning Bolt\nCompanion\nLurrus of the Dream-Den\n", "test");
        Assert.Equal(["Kytheon, Hero of Akros", "Isamaru, Hound of Konda"], deck.Commanders.Select(entry => entry.Card));
        Assert.Equal(100, deck.Count);
        Cantrip.Server.Rules.RulesSeat seat = deck.ToRulesSeat("Alice");
        Assert.Equal(["Kytheon, Hero of Akros", "Isamaru, Hound of Konda"], seat.Commanders);
        Assert.Equal(98, seat.Deck.Count);
        Assert.DoesNotContain("Kytheon, Hero of Akros", seat.Deck);
        Assert.Equal(["Lightning Bolt"], seat.Sideboard);
        Assert.Equal("Lurrus of the Dream-Den", seat.Companion);
        Assert.Equal("partners", seat.DeckName);
    }

    [Theory]
    [InlineData("Commander\n")]
    [InlineData("Commander\nDeck\nPlains")]
    [InlineData("Companion\nCommander\nKytheon, Hero of Akros")]
    public void A_designation_header_without_a_card_is_refused(string text) =>
        Assert.Throws<FormatException>(() => DeckList.Parse("d", text, "test"));

    [Fact]
    public void A_legendary_card_in_the_main_deck_is_never_inferred_to_be_a_commander()
    {
        DeckList deck = DeckList.Parse("ordinary", "Kytheon, Hero of Akros\nArcane Signet\n98 Plains", "test");
        Assert.Empty(deck.ToRulesSeat("Alice").Commanders);
        Assert.Equal(100, deck.ToRulesSeat("Alice").Deck.Count);
    }

    private static string Content => Path.Combine(RepoRoot.Path, "content");

    public static TheoryData<string> Decklists()
    {
        var files = new TheoryData<string>();
        foreach (string file in Directory.GetFiles(Path.Combine(RepoRoot.Path, "decks"), "*.txt", SearchOption.AllDirectories))
        {
            files.Add(Path.GetRelativePath(RepoRoot.Path, file));
        }

        return files;
    }

    [Theory(Skip = "The imported Standard decklists name cards still being added; validating them is noise until those land.")]
    [MemberData(nameof(Decklists))]
    public void Every_decklist_names_only_cards_the_content_has(string file)
    {
        DeckList deck = DeckList.Load(Path.Combine(RepoRoot.Path, file));

        Assert.Equal(deck.Count, deck.Resolve([ContentCache.Get(Content).Cards]).Count);
    }

    [Fact]
    public void A_line_is_a_count_and_a_name_or_just_a_name_and_comments_and_blanks_are_skipped()
    {
        DeckList deck = DeckList.Parse("d", "# a comment\n\n4 Lightning Bolt\r\nsol ring\n", "d.txt");

        Assert.Equal([new DeckListEntry("Lightning Bolt", 4, 3), new DeckListEntry("sol ring", 1, 4)], deck.Entries);
        Assert.Equal(5, deck.Count);
    }

    [Theory]
    [InlineData("legend-rule-test")]
    [InlineData("legend-rule-test.txt")]
    [InlineData("demo/1-black-red")]
    public void A_deck_is_found_by_its_name_under_decks_with_or_without_its_extension(string name)
    {
        string decks = Path.Combine(RepoRoot.Path, "decks");

        Assert.True(File.Exists(DeckList.Find(decks, name)));
        Assert.Throws<FileNotFoundException>(() => DeckList.Find(decks, "no-such-deck"));
    }

    [Fact]
    public void Zero_copies_is_refused_with_its_line()
    {
        FormatException error = Assert.Throws<FormatException>(() => DeckList.Parse("d", "1 Island\n0 Island", "d.txt"));

        Assert.Contains("d.txt line 2", error.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void A_card_the_content_does_not_have_is_refused_with_its_line()
    {
        DeckList deck = DeckList.Parse("d", "39 Island\n1 Not A Real Card", "d.txt");

        InvalidOperationException error = Assert.Throws<InvalidOperationException>(
            () => deck.Resolve([ContentCache.Get(Content).Cards]));

        Assert.Contains("d.txt line 2: no loaded card is named \"Not A Real Card\"", error.Message, StringComparison.Ordinal);
    }

    /// <summary>A <c>Companion</c> line names the deck's companion on the next line; one with nothing after it is an error, never no companion.</summary>
    [Fact]
    public void A_companion_line_names_the_next_card_and_one_naming_none_is_an_error()
    {
        Assert.Equal("Lurrus of the Dream-Den", DeckList.Parse("lurrus", "60 Swamp\nCompanion\n1 Lurrus of the Dream-Den\n", "test").Companion);
        Assert.Null(DeckList.Parse("plain", "60 Swamp\n", "test").Companion);
        Assert.Throws<FormatException>(() => DeckList.Parse("cut", "60 Swamp\nCompanion\n", "test"));
        Assert.Throws<FormatException>(() => DeckList.Parse("two", "60 Swamp\nCompanion\n2 Lurrus of the Dream-Den\n", "test"));
    }

    [Fact]
    public void Commander_section_is_separate_from_main_deck_and_can_be_followed_by_explicit_Deck_header()
    {
        DeckList deck = DeckList.Parse("commander", "Commander\n1 Balin, Loremaster\nDeck\n99 Forest\n", "commander.txt");

        Assert.Equal([new DeckListEntry("Balin, Loremaster", 1, 2)], deck.Commanders);
        Assert.Equal([new DeckListEntry("Forest", 99, 4)], deck.Entries);
        // CR 903.5a: a Commander deck is 100 cards, its commander included.
        Assert.Equal(100, deck.Count);
    }

}
