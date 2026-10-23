using Cantrip.Cards.Model;
using Cantrip.Decks;
using Cantrip.Protocol;
using Cantrip.Server.Decks;
using Cantrip.Server.Events;
using Cantrip.Server.Rules;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// The server's check of a deck before it deals a game with it (<see cref="DeckValidation"/>), the one that counts: each
/// reason a deck is refused, in Constructed (the content's cards and their legality, <c>content/mtg/legality.json</c>) and in
/// Limited (a deck built from a pool, <see cref="PoolDecks.Refusal"/>). The companion's condition is the rules engine's to
/// judge; here a stand-in judges it, as the engine would have.
/// </summary>
public sealed class DeckValidationTests
{
    private static IReadOnlyDictionary<string, CatalogCard> Catalog => DeckCatalog.ByName(RepoRoot.ContentRoot);

    private static readonly DeckFormat Modern = DeckFormat.Find(DeckFormat.ModernKey)!;
    private static readonly DeckFormat Vintage = DeckFormat.Find(DeckFormat.VintageKey)!;

    /// <summary>A rules engine that deals nothing and judges every companion's condition as it is told to, counting the asks.</summary>
    private sealed class Judge(bool fulfilled) : IRulesHostFactory
    {
        public int Asked { get; private set; }

        public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options) => throw new NotSupportedException();

        public bool CompanionFulfilled(IReadOnlyList<string> deck, string companion, int minimumDeckSize)
        {
            Asked++;
            return fulfilled;
        }
    }

    /// <summary>A rules engine that does not answer: its sidecar gone.</summary>
    private sealed class Gone : IRulesHostFactory
    {
        public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options) => throw new NotSupportedException();

        public bool CompanionFulfilled(IReadOnlyList<string> deck, string companion, int minimumDeckSize) =>
            throw new InvalidOperationException("The host's reply to Companions was empty.");
    }

    /// <summary>
    /// The rules engine not answering for a companion is no crash of the table: the deck is refused, its companion unjudged,
    /// and the refusal says it was not checked - to be tried again, not remembered.
    /// </summary>
    [Fact]
    public void A_companion_the_engine_does_not_answer_for_refuses_the_deck_without_failing()
    {
        (IReadOnlyList<string> problems, bool unanswered) = DeckValidation.Check(Deck("vintage", Swamps(60), companion: "Lurrus of the Dream-Den"), Vintage, Catalog, new Gone());
        Assert.True(unanswered);
        Assert.StartsWith("The deck's companion could not be checked: the rules engine did not answer", Assert.Single(problems), StringComparison.Ordinal);
        Assert.Equal((0, false), (DeckValidation.Check(Deck("vintage", Swamps(60)), Vintage, Catalog, new Gone()) is var plain ? (plain.Problems.Count, plain.Unanswered) : default));
    }

    private static SavedDeck Deck(string format, List<DeckEntry> main, List<DeckEntry>? side = null, string companion = "") =>
        new() { Name = "Test", Format = format, Main = main, Side = side ?? [], Companion = companion };

    private static List<DeckEntry> Swamps(int count) => [new("Swamp", count)];

    [Fact]
    public void A_constructed_deck_is_refused_for_its_size_a_banned_card_its_copies_and_its_sideboard()
    {
        var judge = new Judge(fulfilled: true);
        Assert.Empty(DeckValidation.Problems(Deck("modern", Swamps(60)), Modern, Catalog, judge));
        Assert.Equal(["A Modern deck is at least 60 cards: this one is 59."], DeckValidation.Problems(Deck("modern", Swamps(59)), Modern, Catalog, judge));
        // Lurrus of the Dream-Den is banned in Modern, legal in Vintage.
        Assert.Equal(["Lurrus of the Dream-Den is banned in Modern."],
            DeckValidation.Problems(Deck("modern", [.. Swamps(59), new("Lurrus of the Dream-Den", 1)]), Modern, Catalog, judge));
        Assert.Equal(["Lightning Bolt: 5 copies, more than the 4 a Modern deck may have."],
            DeckValidation.Problems(Deck("modern", [.. Swamps(55), new("Lightning Bolt", 5)]), Modern, Catalog, judge));
        Assert.Equal(["A sideboard is at most 15 cards, its companion included: this one is 16."],
            DeckValidation.Problems(Deck("modern", Swamps(60), [new("Lightning Bolt", 4), new("Swamp", 12)]), Modern, Catalog, judge));
        Assert.Equal(0, judge.Asked);
    }

    /// <summary>
    /// Freeform, the default where no format is given, holds a deck to nothing: five cards, ten copies of one, a card banned
    /// everywhere, a sideboard of thirty - all dealt. Only its companion is still held to its own condition (CR 702.139a), as
    /// the engine judges it with the rules' minimum deck size, sixty (CR 100.2a).
    /// </summary>
    [Fact]
    public void A_freeform_deck_is_held_to_nothing_but_its_companion()
    {
        DeckFormat freeform = DeckFormat.Freeform;
        var judge = new Judge(fulfilled: true);
        Assert.Empty(DeckValidation.Problems(Deck("freeform", Swamps(5)), freeform, Catalog, judge));
        Assert.Empty(DeckValidation.Problems(Deck("freeform", [new("Lightning Bolt", 10), new("Black Lotus", 1)], [new("Swamp", 30)]), freeform, Catalog, judge));
        Assert.Empty(DeckValidation.Problems(Deck("freeform", Swamps(5), companion: "Lurrus of the Dream-Den"), freeform, Catalog, judge));
        Assert.Equal(["Lurrus of the Dream-Den can't be the deck's companion: the deck doesn't fulfil its companion condition."],
            DeckValidation.Problems(Deck("freeform", Swamps(5), companion: "Lurrus of the Dream-Den"), freeform, Catalog, new Judge(fulfilled: false)));
        Assert.Equal((0, 60), (freeform.MainSize, freeform.MinimumDeckSize));
        Assert.Same(freeform, DeckFormat.Find("freeform"));
    }

    /// <summary>
    /// A companion (CR 702.139a) must have companion, and the deck must fulfil its condition as the engine judges it, with the
    /// format's minimum deck size: the server refuses a deck that doesn't, so the case of an unrevealed declared companion never
    /// reaches a game. Lurrus in Vintage; a Lightning Bolt is no companion.
    /// </summary>
    [Fact]
    public void A_constructed_deck_is_refused_for_a_companion_whose_condition_the_engine_says_it_does_not_fulfil()
    {
        SavedDeck deck = Deck("vintage", Swamps(60), companion: "Lurrus of the Dream-Den");
        var fulfilled = new Judge(fulfilled: true);
        Assert.Empty(DeckValidation.Problems(deck, Vintage, Catalog, fulfilled));
        Assert.Equal(1, fulfilled.Asked);
        Assert.Equal(["Lurrus of the Dream-Den can't be the deck's companion: the deck doesn't fulfil its companion condition."],
            DeckValidation.Problems(deck, Vintage, Catalog, new Judge(fulfilled: false)));
        var unasked = new Judge(fulfilled: true);
        Assert.Equal(["Lightning Bolt has no companion ability, so it can't be the deck's companion."],
            DeckValidation.Problems(Deck("vintage", Swamps(60), companion: "Lightning Bolt"), Vintage, Catalog, unasked));
        Assert.Equal(0, unasked.Asked);
    }

    /// <summary>A decklist file's <c>Companion</c> line names the deck's companion, which the server checks with the rest.</summary>
    [Fact]
    public void A_decklist_files_companion_is_checked_with_the_deck()
    {
        DeckList list = DeckList.Parse("lurrus", "60 Swamp\nSideboard\n2 Lightning Bolt\nCompanion\n1 Lurrus of the Dream-Den\n", "test");
        Assert.Equal("Lurrus of the Dream-Den", list.Companion);
        SavedDeck deck = DeckValidation.Of(list, Vintage);
        Assert.Equal(("Lurrus of the Dream-Den", 2), (deck.Companion, deck.Side.Sum(entry => entry.Count)));
        Assert.Contains("can't be the deck's companion", Assert.Single(DeckValidation.Problems(deck, Vintage, Catalog, new Judge(fulfilled: false))), StringComparison.Ordinal);
    }

    /// <summary>
    /// Limited (CR 100.2b): a deck built from a pool is held to the event's minimum and to its companion, a card of the pool
    /// outside the main deck - no format's legality, no limit to copies.
    /// </summary>
    [Fact]
    public void A_limited_deck_is_refused_for_its_size_and_its_companion()
    {
        Cantrip.Datasets.ContentLibrary content = ContentCache.Get(RepoRoot.ContentRoot);
        var cards = new EventCards(content);
        string Id(string name) => content.Cards.InOrder().First(card => card.Name == name).Id.Value;
        List<string> pool = [.. Enumerable.Repeat(Id("Ornithopter"), 23), Id("Lurrus of the Dream-Den"), Id("Lightning Bolt")];
        EventDeck Built(int companion) => new() { Main = [.. Enumerable.Range(0, 23)], Basics = new() { ["Swamp"] = 17 }, Companion = companion };

        Assert.Null(cards.Refusal(pool, Built(-1), 40, new Judge(fulfilled: false)));
        Assert.Equal("A Limited deck is at least 41 cards: this one is 40.", cards.Refusal(pool, Built(-1), 41, new Judge(fulfilled: true)));
        Assert.Null(cards.Refusal(pool, Built(23), 40, new Judge(fulfilled: true)));
        Assert.Equal("Lurrus of the Dream-Den can't be the deck's companion: the deck doesn't fulfil its companion condition.",
            cards.Refusal(pool, Built(23), 40, new Judge(fulfilled: false)));
        Assert.Equal("Lightning Bolt has no companion ability, so it can't be the deck's companion.", cards.Refusal(pool, Built(24), 40, new Judge(fulfilled: true)));

        // Submitted, the deck is refused for what the check finds; a companion in the main deck is no sideboard card.
        BuildDeckMessage Submit(int companion, List<int>? main = null) => new()
        {
            Main = main ?? [.. Enumerable.Range(0, 23)], Basics = [new BasicLandCount { Name = "Swamp", Count = 17 }], Submit = true, Companion = companion,
        };
        Assert.Equal("The companion is a card of your sideboard.",
            PoolDecks.Built(new EventDeck(), pool.Count, Submit(0), cards.Basics, 40, deck => cards.Refusal(pool, deck, 40, new Judge(true))).Refusal);
        Assert.Contains("can't be the deck's companion",
            PoolDecks.Built(new EventDeck(), pool.Count, Submit(23), cards.Basics, 40, deck => cards.Refusal(pool, deck, 40, new Judge(false))).Refusal, StringComparison.Ordinal);
        (EventDeck? taken, string? refusal) = PoolDecks.Built(new EventDeck(), pool.Count, Submit(23), cards.Basics, 40, deck => cards.Refusal(pool, deck, 40, new Judge(true)));
        Assert.Null(refusal);
        Assert.Equal(23, taken!.Companion);
        Assert.DoesNotContain(PoolDecks.Sideboard(pool, taken), card => card.Value == Id("Lurrus of the Dream-Den"));
    }

    /// <summary>
    /// A deck the server submits for a player whose time ran out (<see cref="PoolDecks.Submitted"/>) was not checked by them:
    /// its companion is dealt all the same - every card of the pool outside the main deck is still the seat's, the companion
    /// one of them - and the engine leaves one whose condition the deck doesn't fulfil a sideboard card, unrevealed (CR
    /// 103.2b). Here the main deck holds a Lurrus of its own, a permanent card with mana value 3.
    /// </summary>
    [Fact]
    public void A_timed_out_decks_companion_is_dealt_with_the_rest_of_the_pool_outside_the_deck()
    {
        Cantrip.Datasets.ContentLibrary content = ContentCache.Get(RepoRoot.ContentRoot);
        var cards = new EventCards(content);
        string Id(string name) => content.Cards.InOrder().First(card => card.Name == name).Id.Value;
        List<string> pool = [.. Enumerable.Repeat(Id("Ornithopter"), 20), Id("Lurrus of the Dream-Den"), Id("Lurrus of the Dream-Den"), Id("Lightning Bolt"), Id("Lightning Bolt")];
        EventDeck building = new() { Main = [.. Enumerable.Range(0, 21)], Companion = 21 };
        EventDeck submitted = PoolDecks.Submitted(pool, building, 40, _ => []);
        Assert.Equal(21, submitted.Companion);
        Assert.NotNull(cards.Refusal(pool, submitted, 40, new Judge(fulfilled: false)));

        var rules = new CapturingRulesFactory();
        cards.NewGame([("Ann", pool, submitted), ("Ben", pool, new EventDeck { Main = [.. Enumerable.Range(0, 20)], Basics = new() { ["Island"] = 20 } })], 1, 40, rules);
        RulesSeat ann = rules.Seats[0];
        Assert.Equal("Lurrus of the Dream-Den", ann.Companion);
        Assert.Equal(["Lightning Bolt", "Lightning Bolt"], ann.Sideboard);
        Assert.Equal(pool.Count - submitted.Main.Count, ann.Sideboard.Count + 1);
        Assert.Null(rules.Seats[1].Companion);
    }
}
