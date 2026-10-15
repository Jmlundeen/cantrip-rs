using Cantrip.Cards;
using Cantrip.Cards.Model;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Projection.Tests;

/// <summary>A card as printed, for a client to read.</summary>
public sealed class CardOracleTests
{
    private static CardOracle Oracle(params string[] cards)
    {
        var catalog = new CardCatalog();
        foreach (string json in cards)
        {
            CardDefinition card = CardFileReader.Load(json);
            catalog.Cards.Add(card.Id, card);
        }

        return new CardOracle(catalog);
    }

    [Fact]
    public void A_card_reads_as_it_is_printed()
    {
        ObjectView view = Oracle("""
            { "name": "Bolt", "manaCost": "{R}", "type": "Instant", "oracleText": "Bolt deals 3 damage to any target." }
            """).Oracle(new CardId("Bolt"))!;

        Assert.Equal("Bolt", view.Name);
        Assert.Equal("{R}", view.ManaCost);
        Assert.Equal(1, view.ManaValue);
        Assert.Equal([CardTypeView.Instant], view.CardTypes);
        Assert.Equal([ColorView.Red], view.Colors);
        Assert.Equal("Bolt deals 3 damage to any target.", view.OracleText);
        Assert.False(view.ProducesMana);
        Assert.Equal(ViewHandle.None, view.Handle);
    }

    [Fact]
    public void A_card_the_content_does_not_have_is_null()
    {
        Assert.Null(Oracle().Oracle(new CardId("Nothing")));
    }

    [Fact]
    public void A_basic_land_and_a_land_that_adds_mana_produce_mana()
    {
        CardOracle oracle = Oracle(
            """{ "name": "Swamp", "type": "Basic Land — Swamp" }""",
            """{ "name": "Tower", "type": "Land", "oracleText": "{T}: Add {C}." }""",
            """{ "name": "Gate", "type": "Land", "oracleText": "Gate enters tapped." }""");

        Assert.True(oracle.Oracle(new CardId("Swamp"))!.ProducesMana);
        Assert.True(oracle.Oracle(new CardId("Tower"))!.ProducesMana);
        Assert.False(oracle.Oracle(new CardId("Gate"))!.ProducesMana);
    }

    [Fact]
    public void The_other_face_the_halves_and_the_loyalty_go_with_the_card()
    {
        CardOracle oracle = Oracle(
            """
            { "name": "Front", "manaCost": "{W}", "type": "Planeswalker — Gideon", "loyalty": 3,
              "backFace": { "name": "Back", "type": "Creature — Angel", "power": 3, "toughness": 3, "colorIndicator": ["White"] } }
            """,
            """
            { "name": "Fire // Ice", "halves": [
                { "name": "Fire", "manaCost": "{1}{R}", "type": "Instant" },
                { "name": "Ice", "manaCost": "{1}{U}", "type": "Instant" } ] }
            """);

        ObjectView front = oracle.Oracle(new CardId("Front"))!;
        Assert.Equal(3, front.StartingCounter);
        Assert.Equal("Back", front.OtherFace!.Name);
        Assert.Equal([ColorView.White], front.OtherFace.Colors);

        ObjectView split = oracle.Oracle(new CardId("Fire // Ice"))!;
        Assert.Equal(["Fire", "Ice"], split.Halves.Select(half => half.Name));
        Assert.Equal(4, split.ManaValue);
        Assert.Equal([ColorView.Blue, ColorView.Red], split.Colors.Order());
    }
}
