using Cantrip.Cards.Model;
using Cantrip.Datasets;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Cards.Tests;

/// <summary>Card files read for what they print: characteristics, and the faces a card carries.</summary>
public sealed class CardFileReaderTests
{
    [Fact]
    public void A_creature_is_read_with_its_cost_type_line_and_power()
    {
        CardDefinition card = CardFileReader.Load("""
            {
              // Comments and keys the reader does not know are left alone.
              "name": "Test Bear", "manaCost": "{1}{G}", "type": "Creature — Bear", "power": 2, "toughness": 2,
              "oracleText": "Fancy.", "staticAbilities": ["Trample"]
            }
            """);

        Assert.Equal("Test Bear", card.Name);
        Assert.Equal(new CardId("Test Bear"), card.Id);
        Assert.Equal(2, card.ManaCost.ManaValue);
        Assert.True(card.IsCreature);
        Assert.Contains("Bear", card.TypeLine.Subtypes);
        Assert.Equal((2, 2), (card.Power, card.Toughness));
        Assert.Equal("Fancy.", card.OracleText);
    }

    [Fact]
    public void A_split_card_has_its_halves_combined_cost_and_type_line()
    {
        CardDefinition card = CardFileReader.Load("""
            {
              "name": "Fire // Ice",
              "halves": [
                { "name": "Fire", "manaCost": "{1}{R}", "type": "Instant" },
                { "name": "Ice", "manaCost": "{1}{U}", "type": "Sorcery" }
              ]
            }
            """);

        Assert.Equal(2, card.Halves.Count);
        Assert.Equal(4, card.ManaCost.ManaValue);
        Assert.True(card.TypeLine.Is(CardType.Instant));
        Assert.True(card.TypeLine.Is(CardType.Sorcery));
    }

    [Fact]
    public void A_double_faced_card_has_its_back_face_under_its_own_id()
    {
        CardDefinition card = CardFileReader.Load("""
            {
              "name": "Front", "manaCost": "{W}", "type": "Creature — Human", "power": 1, "toughness": 1,
              "backFace": { "name": "Back", "type": "Creature — Angel", "power": 3, "toughness": 3, "colorIndicator": ["White"] }
            }
            """);

        Assert.NotNull(card.BackFace);
        Assert.Equal(new CardId("Front"), card.BackFace!.Id);
        Assert.Equal([ManaColor.White], card.BackFace.ColorIndicator);
    }

    [Theory]
    [InlineData("""{ "manaCost": "{G}", "type": "Instant" }""")]
    [InlineData("""{ "name": "X", "manaCost": "{G", "type": "Instant" }""")]
    [InlineData("""{ "name": "X", "type": "Gizmo" }""")]
    [InlineData("""[ { "name": "X" } ]""")]
    [InlineData("not json")]
    public void A_file_that_does_not_read_is_refused(string json)
    {
        Assert.Throws<CardFileException>(() => CardFileReader.Load(json));
    }

    [Fact]
    public void Every_card_file_of_the_repository_reads_and_is_named_for_its_file()
    {
        string[] files = [.. Directory.GetFiles(RepoRoot.ContentRoot, "*.json", SearchOption.AllDirectories)
            .Where(path => Path.GetFileName(Path.GetDirectoryName(path)) == "cards" && !path.Contains("images", StringComparison.Ordinal))];

        Assert.NotEmpty(files);

        foreach (string file in files)
        {
            CardDefinition card = CardFileReader.LoadFile(file);
            Assert.Equal(CardFileName.For(card.Name), Path.GetFileName(file));
        }
    }
}
