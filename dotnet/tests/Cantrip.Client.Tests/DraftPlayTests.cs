using Cantrip.Client.Drafting;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the seat nobody is sitting in does in an event: a booster pack and a deck in earnest (<c>CubeDrafterTests</c>
/// and <c>DeckBuilderTests</c> say how), a grid's line and a Winston pile by the same sum, and the plain ways those
/// replaced - the cube's pick order when the server sends one, its own colours-and-creatures notion without.
/// </summary>
public sealed class DraftPlayTests
{
    private static int _next = 1;

    private static ObjectView Card(string name, int manaValue, CardTypeView type, params ColorView[] colors) => new()
    {
        Handle = new ViewHandle(_next++),
        Name = name,
        ManaValue = manaValue,
        CardTypes = [type],
        Colors = [.. colors],
    };

    private static readonly ObjectView Bear = Card("Grizzly Bears", 2, CardTypeView.Creature, ColorView.Green);
    private static readonly ObjectView Lotus = Card("Black Lotus", 0, CardTypeView.Artifact);
    private static readonly ObjectView Recall = Card("Ancestral Recall", 1, CardTypeView.Instant, ColorView.Blue);

    private static readonly ObjectView[] RedPool =
        [Card("Goblin Guide", 1, CardTypeView.Creature, ColorView.Red), Card("Lightning Bolt", 1, CardTypeView.Instant, ColorView.Red),
         Card("Monastery Swiftspear", 1, CardTypeView.Creature, ColorView.Red), Card("Fireblast", 6, CardTypeView.Instant, ColorView.Red)];

    [Fact]
    public void A_pack_with_a_pick_order_has_its_best_card_taken_whatever_the_pool_is()
    {
        Assert.Equal(2, DraftPlay.Choose([Bear, Lotus, Recall], RedPool, [40, 1, 0]));
        Assert.Equal(1, DraftPlay.Choose([Bear, Lotus, Recall], [], [40, 1, 2]));
    }

    [Fact]
    public void Cards_the_order_ranks_alike_fall_back_to_the_pools_colours_and_creatures()
    {
        ObjectView goblin = Card("Raging Goblin", 1, CardTypeView.Creature, ColorView.Red);
        Assert.Equal(1, DraftPlay.Choose([Recall, goblin, Bear], RedPool, [5, 5, 5]));
    }

    [Fact]
    public void Without_a_pick_order_a_creature_comes_first()
    {
        Assert.Equal(0, DraftPlay.Choose([Bear, Lotus, Recall], []));
        Assert.Equal(0, DraftPlay.Choose([Bear, Lotus, Recall], [], []));
    }

    private static ObjectView Gone => new() { Known = false };

    [Fact]
    public void A_grid_has_the_line_worth_most_by_the_pick_order_taken()
    {
        ObjectView[] grid = [Lotus, Recall, Gone, Bear, Bear, Bear, Gone, Gone, Gone];

        // Row 1 is the order's two best cards, against three of its worst in row 2.
        Assert.Equal(0, DraftPlay.ChooseLine(grid, [], [0, 1, 9, 9, 9, 9, 9, 9, 9]));

        // Without one, three creatures beat the two; a taken line is never chosen.
        Assert.Equal(1, DraftPlay.ChooseLine(grid, []));
        Assert.Equal(1, DraftPlay.ChooseLine([Gone, Gone, Gone, Bear, Gone, Gone, Gone, Gone, Gone], []));
    }

    [Fact]
    public void A_bot_given_time_to_think_looks_at_a_grid_that_long_before_it_picks()
    {
        var transport = new Recorded([
            new WelcomeMessage { Seat = 2 },
            new DraftMessage { Format = "grid", Active = 2, Decision = 4, Pile = [Bear, Bear, Bear, Lotus, Lotus, Lotus, Recall, Recall, Recall] },
        ]);
        var client = new GameClient(transport);
        client.Pump();
        var clock = new Clock();
        var bot = new DraftPlay { Think = TimeSpan.FromSeconds(1), Time = clock };

        Assert.False(bot.Act(client));
        clock.Ticks += TimeSpan.FromMilliseconds(900).Ticks;
        Assert.False(bot.Act(client));
        clock.Ticks += TimeSpan.FromMilliseconds(100).Ticks;
        Assert.True(bot.Act(client));
        Assert.Equal(4, Assert.IsType<DraftPickMessage>(Assert.Single(transport.Sent)).Decision);
        Assert.False(bot.Act(client));
    }

    [Fact]
    public void A_booster_pack_has_a_card_taken_from_it_once()
    {
        var transport = new Recorded([
            new WelcomeMessage { Seat = 2 },
            new DraftMessage
            {
                Format = "booster", Active = 2, Decision = 0, Pile = [Bear, Recall, Lotus], PileRanks = [40, 1, 0],
                Total = 360, PackNumber = 1, Packs = 3, PackSize = 15, PickNumber = 1, Direction = 1,
            },
        ]);
        var client = new GameClient(transport);
        client.Pump();
        var bot = new DraftPlay();

        Assert.True(bot.Act(client));
        DraftPickMessage pick = Assert.IsType<DraftPickMessage>(Assert.Single(transport.Sent));
        Assert.True(pick.Take);
        Assert.Equal(2, pick.Card);
        Assert.Empty(pick.Also);
        Assert.False(bot.Act(client));
    }

    [Fact]
    public void A_seat_with_a_librarian_to_spend_takes_a_second_card_from_a_deep_pack()
    {
        var transport = new Recorded([
            new WelcomeMessage { Seat = 2 },
            new DraftMessage
            {
                Format = "booster", Active = 2, Decision = 3, Pile = [Bear, Recall, Lotus], PileRanks = [40, 1, 0], AdditionalPicks = [0],
                Total = 360, PackNumber = 1, Packs = 3, PackSize = 4, PickNumber = 2, Direction = 1,
            },
        ]);
        var client = new GameClient(transport);
        client.Pump();

        Assert.True(new DraftPlay().Act(client));
        DraftPickMessage pick = Assert.IsType<DraftPickMessage>(Assert.Single(transport.Sent));
        Assert.Equal(2, pick.Card);
        Assert.Equal([1], pick.Also);
    }

    private static DeckBuildingMessage Building(string title = "") => new()
    {
        Pool = [.. Enumerable.Range(0, 14).Select(number => Card($"Red {number}", 1 + (number % 5), CardTypeView.Creature, ColorView.Red)),
            .. Enumerable.Range(0, 14).Select(number => Card($"Green {number}", 1 + (number % 5), CardTypeView.Creature, ColorView.Green)),
            .. Enumerable.Range(0, 6).Select(number => Card($"Blue {number}", 3, CardTypeView.Creature, ColorView.Blue))],
        Main = [.. Enumerable.Range(0, 34)],
        Basics = [.. new[] { "Plains", "Island", "Swamp", "Mountain", "Forest" }.Select(name => new BasicLandView { Name = name })],
        Minimum = 40,
        Title = title,
    };

    [Fact]
    public void A_deck_is_built_at_the_minimum_size_and_submitted_once()
    {
        DeckBuildingMessage building = Building();
        foreach (ObjectView card in building.Pool)
        {
            card.ManaCost = card.Colors[0] switch { ColorView.Red => "{R}", ColorView.Green => "{G}", _ => "{U}" };
        }

        var transport = new Recorded([new WelcomeMessage { Seat = 1 }, building]);
        var client = new GameClient(transport);
        client.Pump();
        var bot = new DraftPlay();

        Assert.True(bot.Act(client));
        BuildDeckMessage built = Assert.IsType<BuildDeckMessage>(Assert.Single(transport.Sent));
        Assert.True(built.Submit);
        Assert.Equal(40, built.Main.Count + built.Basics.Sum(basic => basic.Count));
        Assert.All(built.Main, place => Assert.True(place < 28, "a blue card in a red-green deck"));
        Assert.DoesNotContain(built.Basics, basic => basic.Name is "Plains" or "Island" or "Swamp");

        // The server has not answered yet: nothing is sent twice.
        Assert.False(bot.Act(client));
    }

    [Fact]
    public void Between_the_games_of_a_match_the_deck_already_built_is_played_again()
    {
        DeckBuildingMessage building = Building("Sideboarding for game 2: you lead 1-0");
        building.Main = [.. Enumerable.Range(0, 23)];
        building.Basics[3].Count = 9;
        building.Basics[4].Count = 8;

        var transport = new Recorded([new WelcomeMessage { Seat = 1 }, building]);
        var client = new GameClient(transport);
        client.Pump();

        Assert.True(new DraftPlay().Act(client));
        BuildDeckMessage built = Assert.IsType<BuildDeckMessage>(Assert.Single(transport.Sent));
        Assert.True(built.Submit);
        Assert.Equal(Enumerable.Range(0, 23), built.Main);
        Assert.Equal(9, built.Basics.Single(basic => basic.Name == "Mountain").Count);
        Assert.Equal(8, built.Basics.Single(basic => basic.Name == "Forest").Count);
    }

    [Fact]
    public void A_whole_pool_submitted_for_a_seat_that_was_not_there_is_built_into_a_deck_before_the_next_game()
    {
        // The deck-building clock ran out on this seat: its deck is its whole pool, with no lands at all.
        DeckBuildingMessage building = Building("Sideboarding for game 2: you trail 0-1");
        foreach (ObjectView card in building.Pool)
        {
            card.ManaCost = card.Colors[0] switch { ColorView.Red => "{R}", ColorView.Green => "{G}", _ => "{U}" };
        }

        foreach (int number in Enumerable.Range(34, 11))
        {
            building.Pool.Add(Card($"Red more {number}", 2, CardTypeView.Creature, ColorView.Red));
        }

        building.Main = [.. Enumerable.Range(0, building.Pool.Count)];

        var transport = new Recorded([new WelcomeMessage { Seat = 1 }, building]);
        var client = new GameClient(transport);
        client.Pump();

        Assert.True(new DraftPlay().Act(client));
        BuildDeckMessage built = Assert.IsType<BuildDeckMessage>(Assert.Single(transport.Sent));
        Assert.True(built.Submit);
        Assert.Equal(40, built.Main.Count + built.Basics.Sum(basic => basic.Count));
        Assert.True(built.Basics.Sum(basic => basic.Count) >= 15, "a deck with no lands");
    }

    // --- Narration: what a seat says it was thinking, and that saying it changes nothing --------------------------------

    /// <summary>
    /// Thirty cards for a hand-built draft of two seats, three packs of five each: colours, curve, a gold card, mana, a
    /// cheat and something to cheat, a counterspell and two lands, each at its own place in a pick order.
    /// </summary>
    private static List<ObjectView> NarrationCube() =>
    [
        DraftingCards.Filler('R', 1, 1), DraftingCards.Filler('R', 2, 2), DraftingCards.Filler('R', 3, 3), DraftingCards.Filler('G', 2, 4), DraftingCards.Filler('G', 4, 5),
        DraftingCards.Filler('U', 2, 6), DraftingCards.Filler('U', 3, 7), DraftingCards.Filler('W', 1, 8), DraftingCards.Filler('W', 3, 9), DraftingCards.Filler('B', 2, 10),
        DraftingCards.Spell("Gold Charm", "{R}{G}", CardTypeView.Instant, "Gold Charm deals 3 damage to any target."),
        DraftingCards.Spell("Red Bolt", "{R}", CardTypeView.Instant, "Red Bolt deals 3 damage to any target."),
        DraftingCards.Spell("Plain Rock", "{2}", CardTypeView.Artifact, "{T}: Add {C}."),
        DraftingCards.Land("Root Land", "{T}: Add {R} or {G}."),
        DraftingCards.Filler('R', 5, 11),
        DraftingCards.Filler('G', 6, 12),
        DraftingCards.Spell("Two Cards", "{2}{U}", CardTypeView.Sorcery, "Draw two cards."),
        DraftingCards.Creature("Big Green", "{5}{G}{G}", 7, 7),
        DraftingCards.Filler('B', 4, 13), DraftingCards.Filler('W', 2, 14),
        DraftingCards.Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield."),
        DraftingCards.Filler('R', 2, 15), DraftingCards.Filler('G', 3, 16), DraftingCards.Filler('U', 1, 17), DraftingCards.Filler('B', 5, 18),
        DraftingCards.Spell("No", "{U}{U}", CardTypeView.Instant, "Counter target spell."),
        DraftingCards.Filler('W', 4, 19), DraftingCards.Filler('R', 4, 20), DraftingCards.Filler('G', 1, 21),
        DraftingCards.Land("Waste Land", "{T}: Add {C}."),
    ];

    /// <summary>A transport the test hands the server's messages to one at a time, keeping what the client sends.</summary>
    private sealed class Fed : IGameTransport
    {
        private readonly List<ServerMessage> _waiting = [];

        public List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Give(ServerMessage message) => _waiting.Add(message);

        public void Send(ClientMessage message) => Sent.Add(message);

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> waiting = [.. _waiting];
            _waiting.Clear();
            return waiting;
        }

        public void Dispose()
        {
        }
    }

    /// <summary>A seat played by a <see cref="DraftPlay"/>, shown one message at a time: a new client for each table, as an event seat has.</summary>
    private sealed class Shown(DraftPlay bot)
    {
        private Fed _transport = new();
        private GameClient _client = null!;

        public DraftPlay Bot { get; } = bot;

        /// <summary>Sits the seat at a new table, as seat 1.</summary>
        public void Sit()
        {
            _transport = new Fed();
            _client = new GameClient(_transport);
            _transport.Give(new WelcomeMessage { Seat = 1 });
            _client.Pump();
        }

        /// <summary>What the seat sent in answer to a message, as text to compare: nothing, if it sent nothing.</summary>
        public string Answer(ServerMessage message)
        {
            int before = _transport.Sent.Count;
            _transport.Give(message);
            _client.Pump();
            Bot.Act(_client);
            return string.Join(" | ", _transport.Sent.Skip(before).Select(sent => System.Text.Json.JsonSerializer.Serialize(sent, sent.GetType())));
        }

        public DraftPickMessage LastPick => (DraftPickMessage)_transport.Sent[^1];
    }

    /// <summary>
    /// Drafts the hand-built cube at a table of two - this seat's own packs and the other seat's, the other seat always
    /// taking a pack's first card - and has a narrated seat and a silent one answer every pack, grid, pile and deck alike.
    /// </summary>
    /// <returns>What the narrated seat wrote.</returns>
    private static string DraftNarratedBesideSilent(Action<string, string> same)
    {
        List<ObjectView> cube = NarrationCube();
        int Rank(ObjectView card) => cube.IndexOf(card) * 11 % cube.Count;
        var writer = new StringWriter();
        var silent = new Shown(new DraftPlay());
        var narrated = new Shown(new DraftPlay { Narrator = writer });
        silent.Sit();
        narrated.Sit();

        // A booster draft: three rounds of two packs of five, one opened by each seat.
        List<ObjectView> pool = [];
        int decision = 0;
        for (int round = 1; round <= 3; round++)
        {
            List<ObjectView> mine = [.. cube.Skip((round - 1) * 10).Take(5)];
            List<ObjectView> theirs = [.. cube.Skip(((round - 1) * 10) + 5).Take(5).Skip(1)];
            for (int look = 0; mine.Count + theirs.Count > 0; look++)
            {
                List<ObjectView> pack = look % 2 == 0 ? mine : theirs;
                if (pack.Count == 0)
                {
                    continue;
                }

                var shown = new DraftMessage
                {
                    Format = "booster", Active = 1, Decision = decision++, Pile = [.. pack], PileRanks = [.. pack.Select(Rank)], Pool = [.. pool],
                    Total = cube.Count, PackNumber = round, Packs = 3, PackSize = 5, PickNumber = 5 - pack.Count + 1, Direction = round == 2 ? -1 : 1,
                    AdditionalPicks = round == 2 && look == 0 ? [0] : [],
                    Seats = [new DraftSeatView { Seat = 1 }, new DraftSeatView { Seat = 2 }],
                };

                same($"round {round} look {look}", silent.Answer(shown) + " / " + narrated.Answer(shown));
                DraftPickMessage pick = silent.LastPick;
                List<ObjectView> taken = [.. new[] { pick.Card }.Concat(pick.Also).Select(place => pack[place])];
                pool.AddRange(taken);
                pack.RemoveAll(taken.Contains);
                if (pack.Count > 0)
                {
                    pack.RemoveAt(0);
                }
            }
        }

        // The deck, at the table it is built at.
        silent.Sit();
        narrated.Sit();
        var building = new DeckBuildingMessage
        {
            Pool = [.. pool, .. Enumerable.Range(30, 14).Select(number => DraftingCards.Filler("RG"[number % 2], 1 + (number % 4), number))],
            Basics = [.. DeckBuilder.BasicLands.Select(name => new BasicLandView { Name = name })],
            Minimum = 40,
        };
        building.Main = [.. Enumerable.Range(0, building.Pool.Count)];
        same("deck", silent.Answer(building) + " / " + narrated.Answer(building));

        // A grid draft of four grids, this seat picking first from every other one.
        silent = new Shown(new DraftPlay());
        narrated = new Shown(new DraftPlay { Narrator = writer });
        silent.Sit();
        narrated.Sit();
        pool = [];
        for (int number = 1; number <= 3; number++)
        {
            List<ObjectView?> grid = [.. cube.Skip((number - 1) * 9).Take(9)];
            bool first = number % 2 == 1;
            if (!first)
            {
                // The other seat took the first row.
                grid[0] = grid[1] = grid[2] = null;
            }

            var shown = new DraftMessage
            {
                Format = "grid", Active = 1, Decision = decision++, Pile = [.. grid.Select(card => card ?? new ObjectView { Known = false })],
                PileRanks = [.. grid.Select(card => card is null ? cube.Count : Rank(card))], Pool = [.. pool], Total = cube.Count,
                PackNumber = number, Packs = 3, PickNumber = first ? 1 : 2, PackSize = 9,
            };

            same($"grid {number}", silent.Answer(shown) + " / " + narrated.Answer(shown));
            int line = silent.LastPick.Card;
            IEnumerable<int> places = line < 3 ? Enumerable.Range(line * 3, 3) : Enumerable.Range(0, 3).Select(row => (row * 3) + line - 3);
            pool.AddRange(places.Select(place => grid[place]).OfType<ObjectView>());
        }

        // Winston piles, one look at a time: small and large, of the pool's colours and not, and the last that cannot be passed.
        silent = new Shown(new DraftPlay());
        narrated = new Shown(new DraftPlay { Narrator = writer });
        silent.Sit();
        narrated.Sit();
        List<ObjectView>[] piles = [[cube[0]], [cube[5], cube[6]], [cube[11], cube[12], cube[13]], [cube[24]], [cube[17], cube[25]]];
        for (int at = 0; at < piles.Length; at++)
        {
            var shown = new DraftMessage
            {
                Format = "winston", Active = 1, Decision = decision++, Looking = at % 3, Piles = [1, 2, 3], Stack = 20 - at,
                Pile = [.. piles[at]], PileRanks = [.. piles[at].Select(Rank)], CanPass = at != 3, Pool = [.. cube.Skip(20).Take(at)],
                Total = cube.Count, Seats = [new DraftSeatView { Seat = 1 }, new DraftSeatView { Seat = 2 }],
            };

            same($"pile {at}", silent.Answer(shown) + " / " + narrated.Answer(shown));
        }

        return writer.ToString();
    }

    [Fact]
    public void A_narrated_seat_drafts_and_builds_exactly_as_a_silent_one()
    {
        string told = DraftNarratedBesideSilent((what, both) =>
        {
            string[] halves = both.Split(" / ");
            Assert.True(halves[0].Length > 0, $"{what}: nothing was sent");
            Assert.True(halves[0] == halves[1], $"{what}: the silent seat sent {halves[0]}, the narrated one {halves[1]}");
        });

        Assert.NotEmpty(told);
    }

    [Fact]
    public void A_narrated_seat_says_what_it_took_what_it_passed_and_why_and_what_deck_it_built()
    {
        string told = DraftNarratedBesideSilent((_, _) => { });
        string[] lines = told.Split('\n', StringSplitOptions.TrimEntries);

        // Once, before the first pick: the draft, and how to read what follows.
        Assert.Equal(1, lines.Count(line => line.StartsWith("Booster draft: 3 packs of 5", StringComparison.Ordinal)));

        // Every pick, with the pick named and the cards that came closest scored, the pick marked.
        Assert.Equal(1, lines.Count(line => line.StartsWith("Pack 1 pick 1: 5 cards, pool 0. Took ", StringComparison.Ordinal)));
        Assert.Contains(lines, line => line.StartsWith("Pack 3 pick 5:", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("-> ", StringComparison.Ordinal) && line.Contains("worth", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("+> ", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("in ", StringComparison.Ordinal) && line.Contains("plays as", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("deck so far:", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("wheel: pick 1's pack came back without", StringComparison.Ordinal));

        // The deck: colours, lands and why that many, the curve, and what only just did not make it.
        Assert.Contains(lines, line => line.StartsWith("Deck: ", StringComparison.Ordinal) && line.Contains("lands", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("lands: mana value", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("last in:", StringComparison.Ordinal));

        // A grid's lines, each scored, and a Winston pile against passing it.
        Assert.Equal(1, lines.Count(line => line.StartsWith("Grid draft:", StringComparison.Ordinal)));
        Assert.Contains(lines, line => line.StartsWith("Grid 2 of 3, second pick", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("-> row", StringComparison.Ordinal) || line.StartsWith("-> column", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("Winston pile ", StringComparison.Ordinal) && line.Contains("took it", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.StartsWith("passing ", StringComparison.Ordinal));
        Assert.Contains(lines, line => line.Contains("the last pile cannot be passed", StringComparison.Ordinal));
    }

    [Fact]
    public void A_seat_whose_narration_cannot_be_written_drafts_on_in_silence()
    {
        var bot = new DraftPlay { Narrator = new Broken() };
        var seat = new Shown(bot);
        seat.Sit();
        string sent = seat.Answer(new DraftMessage
        {
            Format = "booster", Active = 1, Pile = [Bear, Recall, Lotus], PileRanks = [40, 1, 0],
            Total = 360, PackNumber = 1, Packs = 3, PackSize = 15, PickNumber = 1, Direction = 1,
        });

        Assert.Contains("\"Card\":2", sent, StringComparison.Ordinal);
        Assert.Null(bot.Narrator);
    }

    [Fact]
    public void A_seat_whose_games_are_narrated_tells_its_draft_in_the_same_place()
    {
        var writer = new StringWriter();
        var transport = new Recorded([
            new WelcomeMessage { Seat = 2 },
            new DraftMessage
            {
                Format = "booster", Active = 2, Decision = 0, Pile = [Bear, Recall, Lotus], PileRanks = [40, 1, 0],
                Total = 360, PackNumber = 1, Packs = 3, PackSize = 15, PickNumber = 1, Direction = 1,
            },
        ]);
        var seat = new AutoSeat(new GameClient(transport), new Tactics.TacticalPlay { Narrator = writer }) { DraftThink = TimeSpan.FromSeconds(1) };

        Assert.True(seat.Step());
        Assert.Contains("Pack 1 pick 1: 3 cards, pool 0. Took Black Lotus.", writer.ToString(), StringComparison.Ordinal);

        // A seat that is not narrated takes the same card.
        var shown = (DraftMessage)transport.Shown[1];
        var quietTransport = new Recorded([new WelcomeMessage { Seat = 2 }, shown]);
        var quiet = new AutoSeat(new GameClient(quietTransport), new Tactics.TacticalPlay());
        Assert.True(quiet.Step());
        Assert.Equal(2, Assert.IsType<DraftPickMessage>(Assert.Single(quietTransport.Sent)).Card);
        Assert.Equal(2, Assert.IsType<DraftPickMessage>(Assert.Single(transport.Sent)).Card);
    }

    /// <summary>A writer whose file has gone.</summary>
    private sealed class Broken : TextWriter
    {
        public override System.Text.Encoding Encoding => System.Text.Encoding.UTF8;

        public override void Write(char value) => throw new IOException("the disk is full");
    }

    /// <summary>A transport with the server's messages already in it, keeping what the client sends.</summary>
    private sealed class Recorded(IReadOnlyList<ServerMessage> messages) : IGameTransport
    {
        private IReadOnlyList<ServerMessage>? _pending = messages;

        /// <summary>The messages it was made with.</summary>
        public IReadOnlyList<ServerMessage> Shown { get; } = messages;

        public List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Send(ClientMessage message) => Sent.Add(message);

        public IReadOnlyList<ServerMessage> Receive()
        {
            IReadOnlyList<ServerMessage> waiting = _pending ?? [];
            _pending = null;
            return waiting;
        }

        public void Dispose()
        {
        }
    }

    /// <summary>A clock that moves only when told.</summary>
    private sealed class Clock : TimeProvider
    {
        public long Ticks { get; set; }

        public override long TimestampFrequency => TimeSpan.TicksPerSecond;

        public override long GetTimestamp() => Ticks;
    }
}
