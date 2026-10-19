using Cantrip.Client.Decks;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// Whether a constructed deck is legal for its format, and what is wrong with one that is not: sizes, copies and each
/// card's place in the format, and for Commander its commanders - who may be one, who may be paired, and the color
/// identity they hold the deck to.
/// </summary>
public sealed class DeckRulesTests
{
    private static readonly string[] Formats = [.. DeckFormat.All.Select(format => format.Key)];

    private static CatalogCard Card(
        string name,
        CardTypeView type = CardTypeView.Creature,
        string text = "",
        bool legendary = false,
        bool basic = false,
        string[]? subtypes = null,
        ColorView[]? identity = null,
        Dictionary<string, string>? legalities = null)
    {
        var view = new ObjectView { Name = name, CardId = name, CardTypes = [type], OracleText = text, Subtypes = [.. subtypes ?? []] };
        if (legendary) view.Supertypes.Add(SupertypeView.Legendary);
        if (basic) view.Supertypes.Add(SupertypeView.Basic);
        return new CatalogCard
        {
            Card = view,
            Identity = [.. identity ?? []],
            Legalities = legalities ?? Formats.ToDictionary(format => format, _ => Legality.Legal),
        };
    }

    private static Dictionary<string, CatalogCard> Catalog(params CatalogCard[] cards) => cards.ToDictionary(card => card.Name, StringComparer.Ordinal);

    private static SavedDeck Deck(string format, IEnumerable<DeckEntry> main, IEnumerable<DeckEntry>? side = null, IEnumerable<DeckEntry>? commanders = null) => new()
    {
        Name = "Test",
        Format = format,
        Main = [.. main],
        Side = [.. side ?? []],
        Commanders = [.. commanders ?? []],
    };

    private static readonly CatalogCard Bear = Card("Grizzly Bears", identity: [ColorView.Green]);
    private static readonly CatalogCard Forest = Card("Forest", CardTypeView.Land, basic: true, identity: [ColorView.Green]);
    private static readonly CatalogCard Bolt = Card("Lightning Bolt", CardTypeView.Instant, identity: [ColorView.Red]);

    /// <summary>Sixty cards: four Grizzly Bears and fifty-six Forests.</summary>
    private static List<DeckEntry> Sixty() => [new("Grizzly Bears", 4), new("Forest", 56)];

    [Fact]
    public void A_constructed_deck_is_sixty_cards_or_more_with_a_sideboard_of_fifteen_at_most()
    {
        Dictionary<string, CatalogCard> cards = Catalog(Bear, Forest, Bolt);
        Assert.Empty(DeckRules.Problems(Deck("modern", Sixty()), cards));
        Assert.Empty(DeckRules.Problems(Deck("modern", [new("Forest", 200)], [new("Lightning Bolt", 4), new("Forest", 11)]), cards));

        Assert.Contains("A Modern deck is at least 60 cards: this one is 59.", DeckRules.Problems(Deck("modern", [new("Grizzly Bears", 4), new("Forest", 55)]), cards));
        Assert.Contains("A sideboard is at most 15 cards: this one is 16.", DeckRules.Problems(Deck("modern", Sixty(), [new("Forest", 16)]), cards));
    }

    [Fact]
    public void Four_copies_count_across_the_deck_and_its_sideboard_and_basic_lands_are_any_number()
    {
        Dictionary<string, CatalogCard> cards = Catalog(Bear, Forest);
        Assert.Contains(
            "Grizzly Bears: 5 copies, more than the 4 a Standard deck may have.",
            DeckRules.Problems(Deck("standard", Sixty(), [new("Grizzly Bears", 1)]), cards));
        Assert.Empty(DeckRules.Problems(Deck("standard", [new("Forest", 60)]), cards));
    }

    [Fact]
    public void A_card_that_says_how_many_a_deck_can_have_is_held_to_that_and_not_to_four()
    {
        CatalogCard rats = Card("Relentless Rats", text: "A deck can have any number of cards named Relentless Rats.");
        CatalogCard dwarves = Card("Seven Dwarves", text: "Seven Dwarves gets +1/+1 for each other creature named Seven Dwarves you control.\nA deck can have up to seven cards named Seven Dwarves.");
        Dictionary<string, CatalogCard> cards = Catalog(rats, dwarves, Forest);

        Assert.Empty(DeckRules.Problems(Deck("legacy", [new("Relentless Rats", 30), new("Seven Dwarves", 7), new("Forest", 23)]), cards));
        Assert.Contains("Seven Dwarves: 8 copies, more than the 7 a Legacy deck may have.",
            DeckRules.Problems(Deck("legacy", [new("Seven Dwarves", 8), new("Forest", 52)]), cards));
        Assert.Equal(7, DeckRules.MostCopies(dwarves, DeckFormat.Find("commander")!));
    }

    [Fact]
    public void A_banned_card_or_one_not_legal_in_the_format_is_said_and_a_restricted_one_is_one_copy()
    {
        Dictionary<string, string> legalities = Formats.ToDictionary(format => format, _ => Legality.Legal);
        legalities["vintage"] = Legality.Restricted;
        legalities["legacy"] = Legality.Banned;
        legalities["modern"] = Legality.NotLegal;
        CatalogCard recall = Card("Ancestral Recall", CardTypeView.Instant, legalities: legalities);
        Dictionary<string, CatalogCard> cards = Catalog(recall, Forest);

        Assert.Empty(DeckRules.Problems(Deck("vintage", [new("Ancestral Recall", 1), new("Forest", 59)]), cards));
        Assert.Contains("Ancestral Recall is restricted in Vintage: one copy at most, not 2.",
            DeckRules.Problems(Deck("vintage", [new("Ancestral Recall", 1), new("Forest", 59)], [new("Ancestral Recall", 1)]), cards));
        Assert.Contains("Ancestral Recall is banned in Legacy.", DeckRules.Problems(Deck("legacy", [new("Ancestral Recall", 1), new("Forest", 59)]), cards));
        Assert.Contains("Ancestral Recall is not legal in Modern.", DeckRules.Problems(Deck("modern", [new("Ancestral Recall", 1), new("Forest", 59)]), cards));
    }

    [Fact]
    public void A_card_the_catalog_has_not_is_one_the_engine_does_not_play_and_an_unknown_format_is_said()
    {
        Assert.Contains("Black Lotus is not a card the engine plays.", DeckRules.Problems(Deck("vintage", [new("Black Lotus", 1), new("Forest", 59)]), Catalog(Forest)));
        Assert.Equal(["\"pauper\" is not a format a deck is built for."], DeckRules.Problems(Deck("pauper", Sixty()), Catalog(Bear, Forest)));
    }

    private static readonly CatalogCard Omnath = Card("Omnath, Locus of Mana", legendary: true, identity: [ColorView.Green]);

    [Fact]
    public void A_Commander_deck_is_a_hundred_cards_with_its_commander_one_of_each_but_basic_lands_and_no_sideboard()
    {
        CatalogCard elf = Card("Llanowar Elves", identity: [ColorView.Green]);
        Dictionary<string, CatalogCard> cards = Catalog(Omnath, Bear, elf, Forest);
        List<DeckEntry> commander = [new("Omnath, Locus of Mana", 1)];

        Assert.Empty(DeckRules.Problems(Deck("commander", [new("Grizzly Bears", 1), new("Llanowar Elves", 1), new("Forest", 97)], commanders: commander), cards));
        Assert.Contains("A Commander deck is exactly 100 cards with its commander: this one is 101.",
            DeckRules.Problems(Deck("commander", [new("Grizzly Bears", 1), new("Llanowar Elves", 1), new("Forest", 98)], commanders: commander), cards));
        Assert.Contains("Grizzly Bears: 2 copies, more than the 1 a Commander deck may have.",
            DeckRules.Problems(Deck("commander", [new("Grizzly Bears", 2), new("Forest", 97)], commanders: commander), cards));
        Assert.Contains("A Commander deck has no sideboard.",
            DeckRules.Problems(Deck("commander", [new("Grizzly Bears", 1), new("Forest", 98)], [new("Llanowar Elves", 1)], commander), cards));
    }

    [Fact]
    public void Every_card_of_a_Commander_deck_is_within_its_commanders_color_identity()
    {
        Dictionary<string, CatalogCard> cards = Catalog(Omnath, Bolt, Forest);
        Assert.Contains("Lightning Bolt is outside the commander's color identity.",
            DeckRules.Problems(Deck("commander", [new("Lightning Bolt", 1), new("Forest", 98)], commanders: [new("Omnath, Locus of Mana", 1)]), cards));

        HashSet<ColorView> identity = DeckRules.IdentityOf([Omnath]);
        Assert.True(DeckRules.Fits(Forest, DeckFormat.Find("commander")!, identity));
        Assert.False(DeckRules.Fits(Bolt, DeckFormat.Find("commander")!, identity));
        Assert.True(DeckRules.Fits(Bolt, DeckFormat.Find("commander")!, null));
    }

    [Fact]
    public void A_Commander_deck_has_one_commander_that_is_a_legendary_creature_or_says_it_can_be()
    {
        CatalogCard planeswalker = Card("Teferi, Temporal Archmage", CardTypeView.Planeswalker, "Teferi, Temporal Archmage can be your commander.", legendary: true);
        Dictionary<string, CatalogCard> cards = Catalog(Omnath, Bear, planeswalker, Forest);

        Assert.Contains("A Commander deck needs a commander.", DeckRules.Problems(Deck("commander", [new("Forest", 100)]), cards));
        Assert.Contains("Grizzly Bears can't be a commander: it is not a legendary creature.",
            DeckRules.Problems(Deck("commander", [new("Forest", 99)], commanders: [new("Grizzly Bears", 1)]), cards));
        Assert.Contains("A deck has one commander, or two that may be paired: this one has 3.",
            DeckRules.Problems(Deck("commander", [new("Forest", 97)], commanders: [new("Omnath, Locus of Mana", 1), new("Grizzly Bears", 1), new("Teferi, Temporal Archmage", 1)]), cards));
        Assert.True(DeckRules.CanBeCommander(planeswalker));
        Assert.True(DeckRules.CanJoinCommanders(planeswalker, []));
        Assert.False(DeckRules.CanJoinCommanders(Bear, []));
    }

    private static readonly CatalogCard Wilson = Card("Wilson, Refined Grizzly", text: "Choose a Background (You can have a Background as a second commander.)\nTrample, ward {2}", legendary: true, identity: [ColorView.Green]);
    private static readonly CatalogCard Background = Card("Raised by Giants", CardTypeView.Enchantment, "Commander creatures you own have base power and toughness 10/10 and are Giants in addition to their other types.", legendary: true, subtypes: ["Background"], identity: [ColorView.Green]);

    [Fact]
    public void A_Background_is_a_commander_only_beside_one_that_chooses_a_Background()
    {
        Dictionary<string, CatalogCard> cards = Catalog(Wilson, Background, Omnath, Forest);
        Assert.Contains("Raised by Giants is a Background: it is a commander only beside one that says \"Choose a Background\".",
            DeckRules.Problems(Deck("commander", [new("Forest", 99)], commanders: [new("Raised by Giants", 1)]), cards));
        Assert.Empty(DeckRules.Problems(Deck("commander", [new("Forest", 98)], commanders: [new("Wilson, Refined Grizzly", 1), new("Raised by Giants", 1)]), cards));
        Assert.True(DeckRules.CanPair(Background, Wilson));
        Assert.False(DeckRules.CanPair(Omnath, Background));
        Assert.True(DeckRules.CanJoinCommanders(Background, [Wilson]));
        Assert.False(DeckRules.CanJoinCommanders(Background, [Omnath]));
    }

    [Fact]
    public void Two_commanders_with_partner_may_be_paired_and_partner_with_pairs_only_the_two_it_names()
    {
        CatalogCard thrasios = Card("Thrasios, Triton Hero", text: "{4}: Scry 1. Then reveal the top card of your library.\nPartner (You can have two commanders if both have partner.)", legendary: true, identity: [ColorView.Green, ColorView.Blue]);
        CatalogCard tymna = Card("Tymna the Weaver", text: "Lifelink\nPartner (You can have two commanders if both have partner.)", legendary: true, identity: [ColorView.White, ColorView.Black]);
        CatalogCard pir = Card("Pir, Imaginative Rascal", text: "Partner with Toothy, Imaginary Friend (When this creature enters, target player may put Toothy into their hand from their library, then shuffle.)\nIf one or more counters would be put on a permanent your team controls, that many plus one of each of those kinds of counters are put on that permanent instead.", legendary: true, identity: [ColorView.Green]);
        CatalogCard toothy = Card("Toothy, Imaginary Friend", text: "Partner with Pir, Imaginative Rascal (When this creature enters, target player may put Pir into their hand from their library, then shuffle.)\nWhenever you draw a card, put a +1/+1 counter on Toothy.", legendary: true, identity: [ColorView.Blue]);

        Dictionary<string, CatalogCard> cards = Catalog(thrasios, tymna, pir, toothy, Omnath, Forest);
        Assert.True(DeckRules.CanPair(thrasios, tymna));
        Assert.False(DeckRules.CanPair(thrasios, Omnath));
        Assert.True(DeckRules.CanPair(pir, toothy));
        Assert.False(DeckRules.CanPair(pir, thrasios));
        Assert.False(DeckRules.CanPair(thrasios, thrasios));
        Assert.Contains("Thrasios, Triton Hero and Omnath, Locus of Mana can't be commanders together: neither pairs with the other (partner, partner with, friends forever, choose a Background).",
            DeckRules.Problems(Deck("commander", [new("Forest", 98)], commanders: [new("Thrasios, Triton Hero", 1), new("Omnath, Locus of Mana", 1)]), cards));

        // Two commanders' identities together: a blue card in a Thrasios and Tymna deck.
        Assert.Empty(DeckRules.Problems(Deck("commander", [new("Forest", 98)], commanders: [new("Thrasios, Triton Hero", 1), new("Tymna the Weaver", 1)]), cards));
    }

    [Fact]
    public void Friends_forever_pair_together_a_partner_kind_only_with_its_own_kind_and_a_companion_with_the_Doctor()
    {
        CatalogCard will = Card("Will the Wise", text: "Friends forever (You can have two commanders if both have friends forever.)", legendary: true);
        CatalogCard mike = Card("Mike, the Dungeon Master", text: "Friends forever (You can have two commanders if both have friends forever.)", legendary: true);
        CatalogCard survivorA = Card("Survivor A", text: "Partner—Survivors (You can have two commanders if both have this ability.)", legendary: true);
        CatalogCard survivorB = Card("Survivor B", text: "Partner—Survivors (You can have two commanders if both have this ability.)", legendary: true);
        CatalogCard father = Card("Father", text: "Partner—Father & son (You can have two commanders if both have this ability.)", legendary: true);
        CatalogCard companion = Card("Rose Tyler", text: "Doctor's companion (You can have two commanders if the other is the Doctor.)", legendary: true);
        CatalogCard doctor = Card("The Tenth Doctor", legendary: true, subtypes: ["Time Lord", "Doctor"]);
        CatalogCard human = Card("The Human Doctor", legendary: true, subtypes: ["Human", "Time Lord", "Doctor"]);

        Assert.True(DeckRules.CanPair(will, mike));
        Assert.False(DeckRules.CanPair(will, survivorA));
        Assert.True(DeckRules.CanPair(survivorA, survivorB));
        Assert.False(DeckRules.CanPair(survivorA, father));
        Assert.True(DeckRules.CanPair(companion, doctor));
        Assert.True(DeckRules.CanPair(doctor, companion));
        Assert.False(DeckRules.CanPair(companion, human));
        Assert.False(DeckRules.CanPair(companion, mike));
    }

    [Fact]
    public void Only_a_Commander_deck_has_a_commander()
    {
        Assert.Contains("Only a Commander deck has a commander.",
            DeckRules.Problems(Deck("modern", Sixty(), commanders: [new("Omnath, Locus of Mana", 1)]), Catalog(Bear, Forest, Omnath)));
    }
}
