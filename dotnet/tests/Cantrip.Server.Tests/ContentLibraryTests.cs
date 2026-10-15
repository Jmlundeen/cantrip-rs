using Cantrip.Cards.Model;
using Cantrip.Datasets;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// That the content a server reads holds the cards the repository defines, as they are printed: the pool drafting and
/// deck building read. The rules of those cards are the native engine's.
/// </summary>
public sealed class ContentLibraryTests
{
    private static ContentLibrary Library => ContentCache.Get(Path.Combine(RepoRoot.Path, "content"));

    [Fact]
    public void Every_card_file_is_read_and_named_as_its_file_says()
    {
        Assert.NotEmpty(Library.Datasets);

        int files = Directory.GetFiles(Path.Combine(RepoRoot.Path, "content"), "*.json", SearchOption.AllDirectories)
            .Count(path => path.Contains($"{Path.DirectorySeparatorChar}cards{Path.DirectorySeparatorChar}", StringComparison.Ordinal));

        Assert.Equal(files, Library.Cards.Count);
    }

    [Fact]
    public void A_card_is_read_as_it_is_printed()
    {
        Assert.True(Library.Cards.TryGet(new CardId("Lightning Bolt"), out CardDefinition? bolt));
        Assert.Equal(1, bolt!.ManaCost.ManaValue);
        Assert.True(bolt.TypeLine.Is(CardType.Instant));
        Assert.Contains("3 damage", bolt.OracleText, StringComparison.Ordinal);

        Assert.True(Library.Cards.TryGet(new CardId("Llanowar Elves"), out CardDefinition? elves));
        Assert.Equal(1, elves!.Power);
        Assert.Equal(1, elves.Toughness);
    }

    [Fact]
    public void A_split_card_is_its_halves_combined_and_a_double_faced_card_has_its_other_face()
    {
        CardDefinition[] split = [.. Library.Cards.InOrder().Where(card => card.Halves.Count > 0)];
        Assert.NotEmpty(split);
        Assert.All(split, card => Assert.Equal(card.Halves.Sum(half => half.ManaCost.ManaValue), card.ManaCost.ManaValue));

        Assert.True(Library.Cards.TryGet(new CardId("Aang, Swift Savior"), out CardDefinition? aang));
        Assert.NotNull(aang!.BackFace);
    }

    [Fact]
    public void Every_set_lists_only_cards_that_exist()
    {
        foreach (SetList set in Library.Sets)
        {
            Assert.All(set.Cards, card => Assert.True(Library.Cards.Cards.ContainsKey(card), $"{set.Code} lists {card}."));
        }
    }
}
