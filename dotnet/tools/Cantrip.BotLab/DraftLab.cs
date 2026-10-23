using System.Collections.Concurrent;
using System.Diagnostics;
using System.Globalization;
using System.Text;
using Cantrip.Cards.Model;
using Cantrip.Client;
using Cantrip.Client.Drafting;
using Cantrip.Client.Tactics;
using Cantrip.Core.Random;
using Cantrip.Datasets;
using Cantrip.Projection;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;

namespace Cantrip.BotLab;

/// <summary>
/// Drafts a cube with nobody at the table, to say whether one way of drafting and building is better than another.
/// <para>
/// A pod is the server's own <see cref="BoosterDraft"/> - the same packs, passing and pick numbers a real table has -
/// with each seat shown what a client is shown (<see cref="DraftMessage"/>) and nothing else, so what is measured here
/// is what sits down in an event. No table, socket or clock: a pod of eight drafts and builds in a few milliseconds.
/// </para>
/// <para>
/// Two things are measured. What the decks are - colours, lands, curve, how much of their cards' worth their lands can
/// cast - which is quick and says what a drafter did. And who wins: every deck one way of drafting made plays every
/// deck the other made, the tactical seat playing both sides, which is slow, noisy, and the only thing that counts.
/// </para>
/// </summary>
internal static class DraftLab
{
    private const int Minimum = 40;

    // --- The cube ------------------------------------------------------------------------------------------------------

    /// <summary>A cube as a draft deals it: its loaded cards, how each reads, and where each stands in the pick order.</summary>
    internal sealed class Cube
    {
        public required string Name { get; init; }

        /// <summary>The ids of the cube's cards that are loaded, a card repeated for each copy.</summary>
        public required List<string> Cards { get; init; }

        /// <summary>The pick order, as the ids a draft is dealt.</summary>
        public required List<string> PickOrder { get; init; }

        public required Dictionary<string, ObjectView> Views { get; init; }

        /// <summary>The cube's cards a seat may spend for an additional pick (Cogwork Librarian), as a draft is told them.</summary>
        public required List<string> AdditionalPicks { get; init; }

        /// <summary>The basic lands by name, as the server checks a deck against them.</summary>
        public required Dictionary<string, CardId> Basics { get; init; }

        /// <summary>Each card read, and worth what the whole pick order says: what a deck is judged by afterwards.</summary>
        public required Dictionary<string, DraftCard> Known { get; init; }

        public ObjectView View(string id) => Views[id];

        public static Cube Load(string root, string name)
        {
            string path = CubeList.Find(name, Path.Combine(root, "docs", "cube-lists"));
            ContentLibrary content = ContentCache.Get(Path.Combine(root, "content"));
            var projector = new CardOracle(content.Catalog());

            Dictionary<string, string> idByName = new(StringComparer.OrdinalIgnoreCase);
            foreach (CardDefinition card in content.Cards.InOrder())
            {
                idByName[card.Name] = card.Id.Value;
            }

            List<string> cards = [.. CubeList.Load(path).Where(idByName.ContainsKey).Select(card => idByName[card])];
            List<string> order = [.. CubeList.PickOrder(path).Where(idByName.ContainsKey).Select(card => idByName[card])];
            Dictionary<string, ObjectView> views = cards.Concat(PoolDecks.BasicLands.Select(basic => idByName[basic])).Distinct(StringComparer.Ordinal)
                .ToDictionary(id => id, id => projector.Oracle(new CardId(id))!, StringComparer.Ordinal);

            Dictionary<string, DraftCard> known = new(StringComparer.Ordinal);
            foreach (string id in cards.Distinct(StringComparer.Ordinal))
            {
                DraftCard read = DraftCard.Read(views[id]);
                int rank = order.IndexOf(id);
                known[id] = rank >= 0 ? read.WithWorth(CardWorth.FromRank(rank, order.Count)) : read;
            }

            List<string> additional = [.. cards.Distinct(StringComparer.Ordinal).Where(id =>
                content.Cards.TryGet(new CardId(id), out CardDefinition? card) && card!.DraftAbilities.Contains(DraftAbility.AdditionalPickPutThisIntoPack))];

            return new Cube
            {
                Name = Path.GetFileNameWithoutExtension(path), Cards = cards, PickOrder = order, Views = views, Known = known,
                AdditionalPicks = additional, Basics = PoolDecks.Basics(content),
            };
        }
    }

    // --- The seats -----------------------------------------------------------------------------------------------------

    /// <summary>One way of drafting and building, at one seat of one pod.</summary>
    internal interface ISeat
    {
        IReadOnlyList<int> Pick(DraftMessage draft);

        (IReadOnlyList<int> Main, IReadOnlyDictionary<string, int> Basics) Build(DeckBuildingMessage building);
    }

    /// <summary>Whether a seat spends a Cogwork Librarian on this pack, as <see cref="DraftPlay"/> does: while the pack is still half full.</summary>
    private static bool Spends(DraftMessage draft) => draft.AdditionalPicks.Count > 0 && draft.Pile.Count * 2 >= draft.PackSize;

    /// <summary>
    /// The drafter under test. It can draft towards one shape of deck and build another, which a seat at a table
    /// cannot: that is how what a number does to the deck built from a pool is told from what it does to the picks,
    /// and so to the pools of the seats beside it.
    /// </summary>
    /// <param name="draftsBy">The deck the picks are made towards.</param>
    /// <param name="buildsBy">The deck that is built; the same, unless given.</param>
    internal sealed class NewSeat(DraftTuning tuning, DeckTuning? draftsBy = null, DeckTuning? buildsBy = null) : ISeat
    {
        private readonly DeckTuning _buildsBy = buildsBy ?? draftsBy ?? new DeckTuning();

        public CubeDrafter Drafter { get; } = new() { Tuning = tuning, Deck = draftsBy ?? new DeckTuning() };

        public DraftChoice? Last { get; private set; }

        /// <summary>Where the seat tells what it was thinking, as a bot seat at a table does (<see cref="DraftPlay.Narrator"/>); null for none.</summary>
        public TextWriter? Narrator { get; init; }

        private bool _opened;

        /// <summary>How long this seat's picks took, telling included, and how many: what <c>draft watch</c> says it costs.</summary>
        public (long Ticks, int Picks) Spent { get; private set; }

        public IReadOnlyList<int> Pick(DraftMessage draft)
        {
            long began = Stopwatch.GetTimestamp();
            List<string>? why = Narrator is null ? null : [];
            DraftChoice? choice = null;
            IReadOnlyList<int> picked = draft.Format switch
            {
                DraftFormats.Grid => [Drafter.ChooseLine(draft, why)],
                DraftFormats.Winston => [Drafter.TakePile(draft, why) ? 1 : 0],
                _ => (Last = choice = Drafter.Choose(draft, Spends(draft) ? 2 : 1, explain: why is not null)).Places,
            };

            if (Narrator is { } narrator)
            {
                if (!_opened)
                {
                    _opened = true;
                    foreach (string line in CubeDrafter.Opening(draft))
                    {
                        narrator.WriteLine(line);
                    }
                }

                foreach (string line in choice?.Why ?? why!)
                {
                    narrator.WriteLine(line);
                }
            }

            Spent = (Spent.Ticks + Stopwatch.GetTimestamp() - began, Spent.Picks + 1);
            return picked;
        }

        public (IReadOnlyList<int> Main, IReadOnlyDictionary<string, int> Basics) Build(DeckBuildingMessage building)
        {
            // What CubeDrafter.Build does, by this seat's own numbers for the deck; told as the drafter tells it when
            // those are the numbers it drafted by.
            if (Narrator is { } narrator && _buildsBy == Drafter.Deck)
            {
                List<string> why = [];
                BuiltDeck built = Drafter.Build(building, why);
                foreach (string line in why)
                {
                    narrator.WriteLine(line);
                }

                return (built.Main, built.Basics);
            }

            BuiltDeck deck = DeckBuilder.Build([.. building.Pool.Select(Drafter.Card)], building.Minimum, _buildsBy);
            return (deck.Main, deck.Basics);
        }
    }

    /// <summary>
    /// What it replaced, kept to be measured against: the best card by the cube's pick order whatever the pool, and a
    /// deck of the two colours the pool has most cards of, the cheapest twenty-three and seventeen basic lands.
    /// </summary>
    internal sealed class PlainSeat : ISeat
    {
        public IReadOnlyList<int> Pick(DraftMessage draft)
        {
            // A grid's line and a Winston pile as DraftPlay drafts them the plain way.
            if (draft.Format == DraftFormats.Grid)
            {
                return [DraftPlay.ChooseLine(draft.Pile, draft.Pool, draft.PileRanks)];
            }

            if (draft.Format == DraftFormats.Winston)
            {
                return [draft.Pile.Count >= 2 || !draft.CanPass ? 1 : 0];
            }

            int place = DraftPlay.Choose(draft.Pile, draft.Pool, draft.PileRanks);
            if (!Spends(draft))
            {
                return [place];
            }

            List<int> rest = [.. Enumerable.Range(0, draft.Pile.Count).Where(other => other != place)];
            List<int>? ranks = draft.PileRanks.Count == draft.Pile.Count ? [.. rest.Select(other => draft.PileRanks[other])] : null;
            return [place, rest[DraftPlay.Choose([.. rest.Select(other => draft.Pile[other])], draft.Pool, ranks)]];
        }

        public (IReadOnlyList<int> Main, IReadOnlyDictionary<string, int> Basics) Build(DeckBuildingMessage building)
        {
            List<ColorView> colors = [.. building.Pool
                .SelectMany(card => card.Colors)
                .GroupBy(color => color)
                .OrderByDescending(group => group.Count())
                .ThenBy(group => group.Key)
                .Take(2)
                .Select(group => group.Key)];

            List<int> main = [.. building.Pool
                .Select((card, place) => (card, place))
                .Where(pair => !pair.card.CardTypes.Contains(CardTypeView.Land) && pair.card.Colors.All(colors.Contains))
                .OrderBy(pair => pair.card.ManaValue)
                .ThenBy(pair => pair.place)
                .Take(23)
                .Select(pair => pair.place)];

            int lands = Math.Max(17, building.Minimum - main.Count);
            Dictionary<string, int> weights = Enumerable.Range(0, 5)
                .Where(color => colors.Contains((ColorView)color))
                .ToDictionary(color => DeckBuilder.BasicLands[color], color => main.Count(place => building.Pool[place].Colors.Contains((ColorView)color)), StringComparer.Ordinal);
            if (weights.Count == 0)
            {
                weights["Plains"] = 1;
            }

            int total = Math.Max(1, weights.Values.Sum());
            Dictionary<string, int> basics = weights.ToDictionary(pair => pair.Key, pair => lands * pair.Value / total, StringComparer.Ordinal);
            string most = weights.OrderByDescending(pair => pair.Value).ThenBy(pair => pair.Key, StringComparer.Ordinal).First().Key;
            basics[most] += lands - basics.Values.Sum();
            return (main, basics);
        }
    }

    /// <summary>The new drafter's picks with the plain deck, or the plain picks with the new deck: which half of the work a gain comes from.</summary>
    internal sealed class MixedSeat(ISeat picks, ISeat builds) : ISeat
    {
        public IReadOnlyList<int> Pick(DraftMessage draft) => picks.Pick(draft);

        public (IReadOnlyList<int> Main, IReadOnlyDictionary<string, int> Basics) Build(DeckBuildingMessage building) => builds.Build(building);
    }

    /// <summary>
    /// A way of drafting by name: <c>new</c>, <c>plain</c>, <c>new-picks</c> (the plain deck), <c>new-deck</c> (the
    /// plain picks), or <c>new:Name=value,...</c>. A seat is made from a number of its own - its pod's packs and its
    /// place at the table - which is what a seat that draws its numbers (<c>Varied</c>) draws them from.
    /// </summary>
    internal static Func<ulong, ISeat> Seat(string name)
    {
        if (name.StartsWith("new", StringComparison.Ordinal) && (name.Length == 3 || name[3] == ':'))
        {
            Tuning tuned = Tuned(name.Length > 4 ? name[4..] : string.Empty);
            return seat => new NewSeat(tuned.Draft, tuned.DraftsBy, tuned.Varied ? Varied(tuned.BuildsBy, seat) : tuned.BuildsBy);
        }

        return name switch
        {
            "plain" => _ => new PlainSeat(),
            "new-picks" => _ => new MixedSeat(new NewSeat(new DraftTuning()), new PlainSeat()),
            "new-deck" => _ => new PlainThenBuilt(),
            _ => throw new ArgumentException($"No drafter '{name}': new, plain, new-picks, new-deck, or new:Name=value,..."),
        };
    }

    /// <summary>The number a seat is made from: the same for the same packs and place, whatever else is at the table.</summary>
    private static ulong SeatNumber(ulong packs, int seat) => unchecked((packs * 1_000_003UL) + (ulong)seat);

    /// <summary>The plain picks, and the new builder with nothing learnt from the draft but the cards: it reads their ranks as they pass.</summary>
    private sealed class PlainThenBuilt : ISeat
    {
        private readonly NewSeat _watching = new(new DraftTuning());
        private readonly PlainSeat _plain = new();

        public IReadOnlyList<int> Pick(DraftMessage draft)
        {
            _watching.Drafter.Learn(draft.Pile, draft.PileRanks, draft.Total);
            return _plain.Pick(draft);
        }

        public (IReadOnlyList<int> Main, IReadOnlyDictionary<string, int> Basics) Build(DeckBuildingMessage building) => _watching.Build(building);
    }

    /// <summary>The numbers a seat was asked for: how it picks, the deck it picks towards, and the deck it builds.</summary>
    /// <param name="Varied">Whether each seat draws its own numbers for the deck it builds (<see cref="DraftLab.Varied"/>).</param>
    internal sealed record Tuning(DraftTuning Draft, DeckTuning DraftsBy, DeckTuning BuildsBy, bool Varied);

    /// <summary>
    /// Reads <c>Name=value,...</c>. A number of the deck's (<see cref="DeckTuning"/>) is both drafted towards and built
    /// by, unless it is given as <c>Build.Name</c> - the same picks, another deck of the same pool - or as
    /// <c>Draft.Name</c> - other picks, the deck built as ever. <c>Varied=1</c> draws every seat's deck numbers afresh.
    /// </summary>
    internal static Tuning Tuned(string settings)
    {
        var tuning = new DraftTuning();
        var draftsBy = new DeckTuning();
        var buildsBy = new DeckTuning();
        bool varied = false;
        bool builds = false;
        foreach (string setting in settings.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
        {
            string[] parts = setting.Split('=');
            if (parts.Length != 2 || !double.TryParse(parts[1], NumberStyles.Float, CultureInfo.InvariantCulture, out double value))
            {
                throw new ArgumentException($"'{setting}' is not Name=number.");
            }

            string name = parts[0];
            bool onlyBuild = name.StartsWith("Build.", StringComparison.Ordinal);
            bool onlyDraft = name.StartsWith("Draft.", StringComparison.Ordinal);
            name = onlyBuild || onlyDraft ? name[6..] : name;
            DraftTuning? picks = name switch
            {
                "Sureness" => tuning with { Sureness = value },
                "Signals" => tuning with { Signals = value },
                "Carry" => tuning with { Carry = value },
                "Cut" => tuning with { Cut = value },
                "Crowd" => tuning with { Crowd = value },
                "Fixing" => tuning with { Fixing = value },
                "Raw" => tuning with { Raw = value },
                "Slots" => tuning with { Slots = (int)value },
                "PayloadHope" => tuning with { PayloadHope = value },
                "WinstonMargin" => tuning with { WinstonMargin = value },
                "WinstonNext" => tuning with { WinstonNext = value },
                "Deny" => tuning with { Deny = value },
                _ => null,
            };

            if (picks is not null || name == "Varied")
            {
                // A number of the picks' own, or of the lab's: there is nothing for Build. or Draft. to tell apart.
                if (onlyBuild || onlyDraft)
                {
                    throw new ArgumentException($"'{parts[0]}': {name} is not a number of the deck's, so it is given as {name}.");
                }

                tuning = picks ?? tuning;
                varied |= name == "Varied" && value != 0;
                continue;
            }

            // The picks are made towards a deck's shape and what its cheats have to cheat, and nothing else of it
            // (CubeDrafter.Project), so another number cannot be drafted towards alone.
            if (onlyDraft && name is not ("Creatures" or "CreatureCost" or "Cheap" or "CheapCost" or "Expensive" or "ExpensiveCost" or "Unsupported" or "Payloads" or "Outlets" or "Ramp" or "LateFewer"))
            {
                throw new ArgumentException($"'{parts[0]}': the picks do not go by {name}, only the deck built does.");
            }

            draftsBy = onlyBuild ? draftsBy : Set(draftsBy, name, value);
            buildsBy = onlyDraft ? buildsBy : Set(buildsBy, name, value);
            builds |= !onlyDraft;
        }

        if (varied && builds)
        {
            throw new ArgumentException("Varied draws the numbers the deck is built by, so none can be given beside it but as Draft.Name.");
        }

        return new Tuning(tuning, draftsBy, buildsBy, varied);
    }

    private static DeckTuning Set(DeckTuning deck, string name, double value) => name switch
    {
        "SplashCost" => deck with { SplashCost = value },
        "Lands" => deck with { Lands = (int)value },
        "SplashCards" => deck with { SplashCards = (int)value },
        "Creatures" => deck with { Creatures = value },
        "CreatureCost" => deck with { CreatureCost = value },
        "SecondColorCost" => deck with { SecondColorCost = value },
        "Cheap" => deck with { Cheap = value },
        "CheapCost" => deck with { CheapCost = value },
        "Expensive" => deck with { Expensive = value },
        "ExpensiveCost" => deck with { ExpensiveCost = value },
        "Unsupported" => deck with { Unsupported = value },
        "Payloads" => deck with { Payloads = value },
        "Outlets" => deck with { Outlets = value },
        "Ramp" => deck with { Ramp = value },
        "LateFewer" => deck with { LateFewer = value },
        _ => throw new ArgumentException($"No tuning '{name}'."),
    };

    /// <summary>
    /// A deck's numbers drawn for one seat, each over the range worth asking about: for a set of decks whose shapes
    /// differ for no reason of their pools', so that what a shape does to winning can be read off them. Decks built
    /// by the shipped numbers differ in shape only as their pools do, and a pool with good creatures in it has more
    /// creatures and wins more, whichever of the two it is that wins.
    /// </summary>
    private static DeckTuning Varied(DeckTuning given, ulong seat)
    {
        // The seat's number well mixed first (SplitMix64): Random's draws follow its seed too closely for seats whose
        // numbers are one apart to be given numbers that have nothing to do with each other.
        ulong mixed = unchecked(seat + 0x9E37_79B9_7F4A_7C15UL);
        mixed = unchecked((mixed ^ (mixed >> 30)) * 0xBF58_476D_1CE4_E5B9UL);
        mixed = unchecked((mixed ^ (mixed >> 27)) * 0x94D0_49BB_1331_11EBUL);
        var random = new Random(unchecked((int)(mixed ^ (mixed >> 31))));
        T One<T>(params T[] of) => of[random.Next(of.Length)];
        return given with
        {
            Creatures = random.Next(8, 19),
            CreatureCost = One(0.45, 0.9, 1.5),
            Lands = One(-1, 0, 0, 1),
            Cheap = One(3.0, 6, 9),
            CheapCost = One(0.3, 0.8),
            Expensive = One(3.0, 5, 7),
            SplashCost = One(0.0, 0, 1.5, 100),
        };
    }

    // --- A pod ---------------------------------------------------------------------------------------------------------

    /// <summary>A deck a seat built: the pool's cards in it, by id, and its basic lands.</summary>
    internal sealed class SeatDeck
    {
        public required int Seat { get; init; }

        public required string Kind { get; init; }

        public required List<string> Pool { get; init; }

        public required List<string> Main { get; init; }

        public required Dictionary<string, int> Basics { get; init; }

        public int Size => Main.Count + Basics.Values.Sum();
    }

    /// <summary>What the seats have cost so far: every pick and every deck, timed.</summary>
    private static long _pickTicks;
    private static long _picks;
    private static long _buildTicks;
    private static long _builds;

    /// <summary>What a seat is shown as a pack reaches it, as the pod's table would send it.</summary>
    private static DraftMessage Shown(Cube cube, BoosterDraft draft, int seat, int total)
    {
        BoosterPack pack = draft.Current(seat)!;
        return new DraftMessage
        {
            Format = DraftFormats.Booster,
            Active = seat,
            Decision = draft.Picks[seat - 1],
            Pile = [.. pack.Cards.Select(cube.View)],
            PileRanks = draft.PickOrder.Count == 0 ? [] : [.. pack.Cards.Select(draft.Rank)],
            AdditionalPicks = [.. draft.Spendable(seat)],
            Pool = [.. draft.PoolOf(seat).Select(cube.View)],
            Seats = [.. Enumerable.Range(1, draft.Seats).Select(other => new DraftSeatView { Seat = other, Name = $"Seat {other}", Drafted = draft.PoolOf(other).Count, Queued = draft.QueueOf(other).Count })],
            Total = total,
            PackNumber = draft.Pack,
            Packs = draft.Packs,
            PackSize = draft.PackSize,
            PickNumber = draft.PackSize - pack.Cards.Count + 1,
            Direction = draft.Direction,
        };
    }

    /// <summary>Drafts one pod and builds its decks.</summary>
    /// <param name="watch">Told each pick as it is made: the seat, what it was shown, and the place it took.</param>
    internal static List<SeatDeck> Pod(Cube cube, ulong seed, IReadOnlyList<(string Kind, ISeat Seat)> seats, bool ranked = true, Action<int, DraftMessage, int>? watch = null)
    {
        const int Packs = 3;
        const int PackSize = 15;
        DateTimeOffset now = DateTimeOffset.UnixEpoch;
        BoosterDraft draft = BoosterDraft.Start(
            cube.Cards, seats.Count, Packs, PackSize, new Pcg32Random(seed), seed, now, pickOrder: ranked ? cube.PickOrder : [], additionalPicks: cube.AdditionalPicks);
        int total = seats.Count * Packs * PackSize;

        while (!draft.IsOver)
        {
            for (int seat = 1; seat <= seats.Count && !draft.IsOver; seat++)
            {
                if (draft.Current(seat) is null)
                {
                    continue;
                }

                DraftMessage shown = Shown(cube, draft, seat, total);
                long began = Stopwatch.GetTimestamp();
                IReadOnlyList<int> places = seats[seat - 1].Seat.Pick(shown);
                Interlocked.Add(ref _pickTicks, Stopwatch.GetTimestamp() - began);
                Interlocked.Increment(ref _picks);
                watch?.Invoke(seat, shown, places[0]);
                draft.Take(seat, draft.Picks[seat - 1], places, now);
            }
        }

        return Decks(cube, seats, draft.PoolOf);
    }

    /// <summary>
    /// Drafts one table of two by a grid draft (<c>grid</c>: eighteen grids of nine) or a Winston draft
    /// (<c>winston</c>: ninety cards, three piles), as the server deals them and shows them, and builds its decks.
    /// The seat that picks first is the first seat in even pods and the second in odd ones, so each way of drafting
    /// goes first as often. A Winston seat is shown where the pile it looks at stands in the pick order, as the server shows it.
    /// </summary>
    internal static List<SeatDeck> Table(Cube cube, ulong seed, string format, IReadOnlyList<(string Kind, ISeat Seat)> seats, bool ranked = true)
    {
        // The cube less what the format bans, as the server deals it (Banlist: Cogwork Librarian).
        var random = new Pcg32Random(seed);
        List<string> cards = [.. cube.Cards.Where(card => !Banlist.IsBanned(format, cube.View(card).Name))];
        for (int i = cards.Count - 1; i > 0; i--)
        {
            int j = random.NextInt(i + 1);
            (cards[i], cards[j]) = (cards[j], cards[i]);
        }

        int first = 1 + (int)(seed % 2);
        List<ObjectView> Pool(IReadOnlyList<string> pool) => [.. pool.Select(cube.View)];
        List<DraftSeatView> Seats(Func<int, int> drafted) => [.. Enumerable.Range(1, 2).Select(seat => new DraftSeatView { Seat = seat, Name = $"Seat {seat}", Drafted = drafted(seat) })];
        if (format == "grid")
        {
            const int Packs = 18;
            GridDraft grid = GridDraft.Start(cards[..(Packs * GridDraft.PackSize)], Packs, first, random, ranked ? cube.PickOrder : []);
            while (!grid.IsOver)
            {
                int seat = grid.Active;
                var shown = new DraftMessage
                {
                    Format = DraftFormats.Grid,
                    Active = seat,
                    Decision = grid.Decision,
                    Pile = [.. grid.Grid.Select(card => card is null ? new ObjectView { Known = false } : cube.View(card))],
                    PileRanks = grid.PickOrder.Count == 0 ? [] : [.. grid.Grid.Select(card => card is null ? grid.PickOrder.Count : grid.Rank(card))],
                    PackNumber = grid.Pack,
                    Packs = grid.Packs,
                    PickNumber = grid.PickOfPack,
                    PackSize = GridDraft.PackSize,
                    Pool = Pool(grid.PoolOf(seat)),
                    Seats = Seats(other => grid.PoolOf(other).Count),
                    Total = Packs * GridDraft.PackSize,
                };

                long began = Stopwatch.GetTimestamp();
                int line = seats[seat - 1].Seat.Pick(shown)[0];
                Interlocked.Add(ref _pickTicks, Stopwatch.GetTimestamp() - began);
                Interlocked.Increment(ref _picks);
                grid.Take(seat, grid.Decision, line);
            }

            return Decks(cube, seats, grid.PoolOf);
        }

        if (format != "winston")
        {
            throw new ArgumentException($"No draft format '{format}': booster, grid, winston.");
        }

        const int Dealt = 90;
        WinstonDraft winston = WinstonDraft.Start(cards[..Dealt], 3, 2, first, random, ranked ? cube.PickOrder : []);
        while (!winston.IsOver)
        {
            int seat = winston.Active;
            var shown = new DraftMessage
            {
                Format = DraftFormats.Winston,
                Active = seat,
                Looking = winston.Looking,
                Decision = winston.Decision,
                Piles = [.. winston.Piles.Select(pile => pile.Count)],
                Stack = winston.Stack.Count,
                Pile = winston.Looking >= 0 ? [.. winston.Piles[winston.Looking].Select(cube.View)] : [],
                PileRanks = winston.Looking < 0 || winston.PickOrder.Count == 0 ? [] : [.. winston.Piles[winston.Looking].Select(winston.Rank)],
                CanPass = winston.CanPass,
                Pool = Pool(winston.PoolOf(seat)),
                Seats = Seats(other => winston.PoolOf(other).Count),
                Total = Dealt,
            };

            long began = Stopwatch.GetTimestamp();
            bool take = seats[seat - 1].Seat.Pick(shown)[0] == 1;
            Interlocked.Add(ref _pickTicks, Stopwatch.GetTimestamp() - began);
            Interlocked.Increment(ref _picks);
            if (take)
            {
                winston.Take(seat, winston.Decision);
            }
            else
            {
                winston.Pass(seat, winston.Decision);
            }
        }

        return Decks(cube, seats, winston.PoolOf);
    }

    /// <summary>Each seat's deck, built from its pool by its own builder and put to the server's own check of a submitted deck.</summary>
    private static List<SeatDeck> Decks(Cube cube, IReadOnlyList<(string Kind, ISeat Seat)> seats, Func<int, IReadOnlyList<string>> poolOf)
    {
        List<SeatDeck> decks = [];
        for (int seat = 1; seat <= seats.Count; seat++)
        {
            List<string> pool = [.. poolOf(seat)];
            var building = new DeckBuildingMessage
            {
                Pool = [.. pool.Select(cube.View)],
                Main = [.. Enumerable.Range(0, pool.Count)],
                Basics = [.. PoolDecks.BasicLands.Select(name => new BasicLandView { Name = name })],
                Minimum = Minimum,
            };

            long began = Stopwatch.GetTimestamp();
            (IReadOnlyList<int> main, IReadOnlyDictionary<string, int> basics) = seats[seat - 1].Seat.Build(building);
            Interlocked.Add(ref _buildTicks, Stopwatch.GetTimestamp() - began);
            Interlocked.Increment(ref _builds);

            // The server's own word on the deck, as it would give it to a seat that submitted this.
            var submitted = new BuildDeckMessage
            {
                Main = [.. main], Basics = [.. basics.Select(basic => new BasicLandCount { Name = basic.Key, Count = basic.Value })], Submit = true,
            };

            // The draft bot names no companion, so the server's check of the format asks nothing more than the size.
            if (PoolDecks.Built(new EventDeck(), pool.Count, submitted, cube.Basics, Minimum, _ => null).Refusal is { } refusal)
            {
                throw new InvalidOperationException($"Seat {seat} ({seats[seat - 1].Kind}) built a deck the server would refuse: {refusal}");
            }
            decks.Add(new SeatDeck
            {
                Seat = seat,
                Kind = seats[seat - 1].Kind,
                Pool = pool,
                Main = [.. main.Select(place => pool[place])],
                Basics = basics.Where(basic => basic.Value > 0).ToDictionary(basic => basic.Key, basic => basic.Value, StringComparer.Ordinal),
            });
        }

        return decks;
    }

    // --- What a deck is ------------------------------------------------------------------------------------------------

    /// <summary>A deck's measurements, by the same yardstick whoever built it: the cube's whole pick order and Karsten's sources.</summary>
    internal sealed record Measured(
        int Size, int Lands, int Colors, double Sources, int Creatures, double ManaValue, double Worth, double Cast, int Uncastable, int Unplayed)
    {
        /// <summary>The colours the deck's spells ask for, and its sources of each: "WG+R, sources W 8 G 8 R 3".</summary>
        public string Mana { get; init; } = string.Empty;

        /// <summary>
        /// What the deck is, a number at a time, by name: a column each of <c>decks.csv</c>, and what <c>--keep</c>
        /// chooses decks by. Counts are of the main deck's spells unless they say otherwise.
        /// </summary>
        public IReadOnlyList<(string Name, double Value)> Columns { get; init; } = [];

        public double this[string name] => Columns.FirstOrDefault(column => string.Equals(column.Name, name, StringComparison.OrdinalIgnoreCase)) is { Name: not null } found
            ? found.Value
            : throw new ArgumentException($"A deck has no '{name}': {string.Join(", ", Columns.Select(column => column.Name))}.");
    }

    internal static Measured Measure(Cube cube, SeatDeck deck)
    {
        List<DraftCard> main = [.. deck.Main.Select(id => cube.Known[id])];
        List<DraftCard> spells = [.. main.Where(card => !card.IsLand)];
        double[] sources = new double[5];
        for (int color = 0; color < 5; color++)
        {
            sources[color] = deck.Basics.GetValueOrDefault(DeckBuilder.BasicLands[color])
                + main.Count(card => card.IsLand && ((card.Produces >> color) & 1) != 0)
                + (0.5 * spells.Count(card => card.MakesMana && ((card.Produces >> color) & 1) != 0));
        }

        // The deck's colours are the ones its spells ask for of the colours it has lands for, whatever else a land of
        // any colour or a Mox happens to make.
        int able = Enumerable.Range(0, 5).Where(color => sources[color] >= 1).Aggregate(0, (mask, color) => mask | (1 << color));
        int landed = Enumerable.Range(0, 5).Where(color => sources[color] >= 2.5).Aggregate(0, (mask, color) => mask | (1 << color));
        int colors = spells.Aggregate(0, (mask, card) => mask | card.Needs(card.OffColor(landed) == 0 ? landed : card.OffColor(able) == 0 ? able : DraftCard.AllColors));
        double cast = 0;
        int uncastable = 0;
        foreach (DraftCard card in spells)
        {
            double castable = ManaBase.Castable(card, sources, able);
            cast += card.Power * castable;
            uncastable += castable < 0.5 ? 1 : 0;
        }

        // The good cards left in the sideboard: what the draft took and the deck could not use.
        HashSet<string> played = [.. deck.Main];
        int unplayed = deck.Pool.Count(id => !played.Contains(id) && !cube.Known[id].IsLand && cube.Known[id].Worth >= CardWorth.Middling + 0.5);

        // What the builder would say the deck is worth, whoever built it: its sum, by the shipped numbers, on this yardstick.
        double lean = DeckBuilder.Lean(spells);
        int creatures = spells.Count(card => card.IsCreature);
        Payloads payloads = Payloads.Of(spells);
        double score = spells.Sum(card => (DeckBuilder.PowerIn(card, colors, payloads) * ManaBase.Castable(card, sources, able)) + (DeckBuilder.Synergy * card.Speed * lean))
            - DeckBuilder.ShapeCost(spells.Count, creatures, spells.Count(card => card.ManaValue <= 2), spells.Count(card => card.ManaValue >= 5), lean, makers: spells.Count(card => card.MakesMana));

        int Count(Func<DraftCard, bool> which) => spells.Count(which);
        bool Is(DraftCard card, CardTypeView type) => card.View.CardTypes.Contains(type);
        int landCount = deck.Size - spells.Count;
        double sourcesEach = spells.Count == 0 ? 0 : Enumerable.Range(0, 5).Where(color => ((colors >> color) & 1) != 0).Select(color => sources[color]).DefaultIfEmpty(0).Average();
        double manaValue = spells.Count == 0 ? 0 : spells.Average(card => (double)card.ManaValue);
        double worth = spells.Sum(card => card.Power);

        // Which kind of deck it is, as one of five and no other (Archetype): for pools of one kind, and for saying
        // where a drafter is strong and where it is weak rather than how it does on average.
        int enablers = Count(Enabler);
        int controlling = Count(card => !card.IsCreature && (card.Counterspell || card.Sweeper || card.Planeswalker || card.CardAdvantage));
        int makers = Count(card => card.MakesMana);
        int dear = Count(card => card.ManaValue >= 5);
        string archetype = enablers >= 4 ? "combo"
            : creatures <= 10 && controlling >= 7 ? "control"
            : makers >= 4 && dear >= 4 ? "ramp"
            : lean >= 0.25 && creatures >= 12 && manaValue <= 2.6 ? "aggro"
            : "midrange";

        return new Measured(deck.Size, landCount, int.PopCount(colors), sourcesEach, creatures, manaValue, worth, cast, uncastable, unplayed)
        {
            Mana = $"{Colors(colors)}, sources " + string.Join(" ", Enumerable.Range(0, 5).Where(color => sources[color] > 0).Select(color => $"{"WUBRG"[color]} {sources[color]:0.#}")),
            Columns =
            [
                ("lands", landCount),
                ("colours", int.PopCount(colors)),
                ("sources", sourcesEach),
                ("creatures", creatures),
                ("manaValue", manaValue),
                ("worth", worth),
                ("cast", cast),
                ("hardToCast", uncastable),
                ("leftOut", unplayed),
                ("score", score),
                ("lean", lean),
                ("best", spells.Count == 0 ? 0 : spells.Max(card => card.Power)),
                ("removal", Count(card => card.Removal)),
                ("sweepers", Count(card => card.Sweeper)),
                ("counterspells", Count(card => card.Counterspell)),
                ("cardAdvantage", Count(card => card.CardAdvantage)),
                ("planeswalkers", Count(card => card.Planeswalker)),
                ("evasive", Count(card => card.Evasive)),
                ("manaMakers", Count(card => card.MakesMana)),
                ("mv1", Count(card => card.ManaValue <= 1)),
                ("mv2", Count(card => card.ManaValue == 2)),
                ("mv3", Count(card => card.ManaValue == 3)),
                ("mv4", Count(card => card.ManaValue == 4)),
                ("mv5", Count(card => card.ManaValue == 5)),
                ("mv6", Count(card => card.ManaValue >= 6)),
                ("cheapCreatures", Count(card => card.IsCreature && card.ManaValue <= 2)),
                ("creaturePower", spells.Where(card => card.IsCreature).Sum(card => card.View.Power ?? 0)),
                ("instants", Count(card => Is(card, CardTypeView.Instant))),
                ("sorceries", Count(card => Is(card, CardTypeView.Sorcery))),
                ("artifacts", Count(card => !card.IsCreature && Is(card, CardTypeView.Artifact))),
                ("enchantments", Count(card => !card.IsCreature && Is(card, CardTypeView.Enchantment))),
                ("gold", Count(card => int.PopCount(card.Colors) >= 2)),
                ("colourless", Count(card => card.Colors == 0)),
                ("duals", main.Count(card => card.IsLand && int.PopCount(card.Produces & colors) >= 2)),
                ("otherLands", main.Count(card => card.IsLand && int.PopCount(card.Produces & colors) < 2)),
                ("unread", Count(card => Unread(card) == 1)),
                ("unreadAimed", Count(card => Unread(card) == 2)),

                // Every spell that is not a creature as one thing and no other, for weighing each kind against the rest.
                ("spellsUnread", Count(card => Kind(card) == "unread")),
                ("spellsRemoval", Count(card => Kind(card) == "removal")),
                ("spellsCounter", Count(card => Kind(card) == "counter")),
                ("spellsMana", Count(card => Kind(card) == "mana")),
                ("spellsDraw", Count(card => Kind(card) == "draw")),
                ("spellsOther", Count(card => Kind(card) == "spell")),
                ("permanentsOther", Count(card => Kind(card) == "permanent")),

                // The kind of deck, one of the five at 1 and the rest at 0, and the two counts it is told by.
                ("enablers", enablers),
                ("controlCards", controlling),
                .. Archetypes.Select(name => (name, name == archetype ? 1.0 : 0.0)),

                // Cards that cheat something into play, what the deck has worth cheating, and the cheats with nothing
                // of their kind to cheat (DraftCard.Cheats, Payload): last, so older columns keep their places.
                ("cheats", Count(card => card.Cheats != 0)),
                ("payloads", Count(card => card.Payload != 0)),
                ("unsupported", Count(card => card.Cheats != 0 && !spells.Any(other => (other.Payload & card.Cheats) != 0))),
                ("outlets", Count(card => card.Outlet)),
            ],
        };
    }

    /// <summary>
    /// The kinds of deck a deck is told into, the first it is: <c>combo</c>, four cards or more that cheat a card
    /// into play, bring one back from a graveyard, find one, or make a burst of mana (<see cref="Enabler"/>);
    /// <c>control</c>, ten creatures or fewer and seven or more counterspells, sweepers, planeswalkers and cards that
    /// draw; <c>ramp</c>, four or more cards that make mana and four or more at five mana or more; <c>aggro</c>,
    /// leaning early (0.25 or more) with twelve creatures or more and an average mana value of 2.6 or less; and
    /// <c>midrange</c>, the rest. Rough, and by what is in the deck, not by what the drafter meant: it drafts colours,
    /// not archetypes, so a combo deck here is a deck with combo cards in it, not one sure to have what they need.
    /// </summary>
    internal static readonly IReadOnlyList<string> Archetypes = ["combo", "control", "ramp", "aggro", "midrange"];

    /// <summary>Whether a card is one a combination deck is built round, going by its text: it cheats, reanimates, tutors, storms, or is a ritual.</summary>
    private static bool Enabler(DraftCard card)
    {
        if (card.IsLand)
        {
            return false;
        }

        string text = (card.View.OracleText ?? string.Empty).ToLowerInvariant();
        return EnablerText.IsMatch(text) || (!card.IsCreature && CardReading.Ritual(text));
    }

    private static readonly System.Text.RegularExpressions.Regex EnablerText = new(
        @"(^|\n|, )storm\b|from (your|a|any) graveyard (on)?to the battlefield|card from your hand onto the battlefield|search your library for (a|an|up to \w+) [^.]*card[^.]*put (it|that card|them) (onto the battlefield|into your graveyard)|search your library for a card\b|search your library for an (instant|artifact|enchantment)[^.]*card",
        System.Text.RegularExpressions.RegexOptions.CultureInvariant | System.Text.RegularExpressions.RegexOptions.Compiled);

    /// <summary>
    /// What a spell is for, as one word, the first of these it is: a creature, a planeswalker, an instant or sorcery
    /// its pilot does not read (<see cref="Unread"/>), removal, a counterspell, mana, cards, some other instant or
    /// sorcery, some other permanent.
    /// </summary>
    private static string Kind(DraftCard card) =>
        card.IsCreature ? "creature"
        : card.Planeswalker ? "planeswalker"
        : Unread(card) > 0 ? "unread"
        : card.Removal || card.Sweeper ? "removal"
        : card.Counterspell ? "counter"
        : card.MakesMana ? "mana"
        : card.CardAdvantage ? "draw"
        : card.View.CardTypes.Contains(CardTypeView.Instant) || card.View.CardTypes.Contains(CardTypeView.Sorcery) ? "spell"
        : "permanent";

    /// <summary>
    /// Whether the seat that plays the decks reads nothing off an instant or a sorcery, by its own reading of cards
    /// (<see cref="CardReading"/>, as <c>cantrip-botlab read</c> lists them): 0 for a card it reads, 1 for one it does
    /// not and so casts only when it has nothing better to do with the mana, 2 for one it does not that aims at
    /// something, which it does not cast at all. How many of these a deck has is how much of it its pilot cannot use.
    /// </summary>
    private static int Unread(DraftCard card)
    {
        if (!card.View.CardTypes.Contains(CardTypeView.Instant) && !card.View.CardTypes.Contains(CardTypeView.Sorcery))
        {
            return 0;
        }

        static bool Read(string text) =>
            CardReading.Effect(text).Kind != EffectKind.Unknown || CardReading.Draws(text) > 0 || CardReading.Tokens(text).Count > 0 || CardReading.Selects(text);

        string text = card.View.OracleText ?? string.Empty;
        IReadOnlyList<string> modes = CardReading.Modes(text);
        return (modes.Count > 0 ? modes.Any(Read) : Read(text)) ? 0
            : modes.Count == 0 && CardReading.Plain(text).Contains("target", StringComparison.Ordinal) ? 2
            : 1;
    }

    /// <summary>
    /// Which decks to keep, from <c>name:least..most,...</c> with either end left out: <c>counterspells:2..</c>,
    /// <c>creatures:..9,lean:..0</c>. The names are a deck's columns (<see cref="Measured.Columns"/>), and with games
    /// <c>share</c>, the part of its games it won. Written without <c>&lt;</c> or <c>&gt;</c>, which a shell would take.
    /// </summary>
    internal static Func<Measured, double?, bool> Keep(string conditions)
    {
        List<(string Name, double Least, double Most)> all = [];
        foreach (string condition in conditions.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
        {
            string[] parts = condition.Split(':');
            string[] ends = parts.Length == 2 ? parts[1].Split("..") : [];
            if (ends.Length != 2)
            {
                throw new ArgumentException($"'{condition}' is not name:least..most.");
            }

            double End(string text, double open) => text.Length == 0 ? open
                : double.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out double value) ? value
                : throw new ArgumentException($"'{text}' in '{condition}' is not a number.");
            all.Add((parts[0], End(ends[0], double.NegativeInfinity), End(ends[1], double.PositiveInfinity)));
        }

        return (deck, share) => all.TrueForAll(condition =>
        {
            double? value = string.Equals(condition.Name, "share", StringComparison.OrdinalIgnoreCase) ? share : deck[condition.Name];
            return value is { } has && has >= condition.Least && has <= condition.Most;
        });
    }

    private static string Colors(int mask) => mask == 0 ? "-" : string.Concat(Enumerable.Range(0, 5).Where(color => ((mask >> color) & 1) != 0).Select(color => "WUBRG"[color]));

    /// <summary>A deck as a decklist file, the sideboard after it, with what it is in comments at the top.</summary>
    internal static string DeckText(Cube cube, SeatDeck deck, string heading)
    {
        var text = new StringBuilder();
        Measured measured = Measure(cube, deck);
        text.AppendLine(CultureInfo.InvariantCulture, $"# {heading}");
        text.AppendLine(CultureInfo.InvariantCulture, $"# {measured.Size} cards, {measured.Lands} lands, {measured.Creatures} creatures, average mana value {measured.ManaValue:N2}.");
        text.AppendLine(CultureInfo.InvariantCulture, $"# Colours {measured.Mana}.");

        List<string> played = [.. deck.Main];
        foreach (IGrouping<bool, string> group in deck.Main.GroupBy(id => cube.Known[id].IsLand).OrderBy(group => group.Key))
        {
            foreach (string id in group.OrderBy(id => cube.Known[id].ManaValue).ThenBy(id => cube.View(id).Name, StringComparer.Ordinal))
            {
                text.AppendLine(cube.View(id).Name);
            }
        }

        foreach ((string basic, int count) in deck.Basics)
        {
            text.AppendLine(CultureInfo.InvariantCulture, $"{count} {basic}");
        }

        List<string> side = [.. deck.Pool];
        foreach (string id in played)
        {
            side.Remove(id);
        }

        if (side.Count > 0)
        {
            text.AppendLine("Sideboard");
            foreach (string id in side.OrderByDescending(id => cube.Known[id].Worth))
            {
                text.AppendLine(cube.View(id).Name);
            }
        }

        return text.ToString();
    }

    /// <summary>
    /// The deck as a game is dealt it: read from the same text its decklist file holds, card for card in the same order,
    /// so that a game played here is the game <c>bot-duel.cmd watch</c> plays from the files with the same shuffle.
    /// </summary>
    private static DeckList AsDeckList(Cube cube, SeatDeck deck, string name) => DeckList.Parse(name, DeckText(cube, deck, name), name);

    // --- draft ---------------------------------------------------------------------------------------------------------

    /// <summary>Pods of two ways of drafting in alternate seats, what their decks are, and - with games - which wins.</summary>
    public static int Draft(string root, string[] args, int parallelism)
    {
        string a = Text(args, "--a", "new");
        string b = Text(args, "--b", "plain");
        int pods = Number(args, "--pods", 50);
        int games = Number(args, "--games", 0);
        int seatCount = Number(args, "--seats", 8);
        ulong seed = (ulong)Number(args, "--seed", 20261003);
        string? output = args.Contains("--out") ? Text(args, "--out", string.Empty) : null;
        bool ranked = !args.Contains("--no-order");
        bool apart = args.Contains("--apart");
        string format = Text(args, "--format", DraftFormats.Booster);
        if (format != DraftFormats.Booster && apart)
        {
            throw new ArgumentException("--apart is for a booster draft: a grid or Winston table is two seats, one of each.");
        }
        Func<Measured, double?, bool>? keep = args.Contains("--keep") ? Keep(Text(args, "--keep", string.Empty)) : null;
        int limit = Number(args, "--limit", int.MaxValue);
        string pilotName = Text(args, "--pilot", "referee");
        Cube cube = Cube.Load(root, Text(args, "--cube", "LSVCube"));
        Func<ulong, ISeat> makeA = Seat(a);
        Func<ulong, ISeat> makeB = Seat(b);

        bool booster = format == DraftFormats.Booster;
        Console.WriteLine($"{a} against {b}: {pods} {(booster ? $"pods of {seatCount}" : $"{format} drafts of two")} drafting {cube.Name} ({cube.Cards.Count} cards loaded, {cube.PickOrder.Count} in its pick order{(ranked ? string.Empty : ", not sent")}), {(booster ? "packs" : "cards")} from {seed}.");
        Console.WriteLine(!booster
            ? "One seat of each, the first to pick in turn."
            : apart
            ? "Each has the table to itself: the same packs drafted twice, by a table of one and by a table of the other."
            : "Alternate seats, the other way round in every second pod.");

        var clock = Stopwatch.StartNew();
        List<SeatDeck>[] drafted = new List<SeatDeck>[pods];
        Parallel.For(0, pods, new ParallelOptions { MaxDegreeOfParallelism = parallelism }, pod =>
        {
            ulong packs = seed + (ulong)pod;
            List<(string, ISeat)> Table(Func<int, bool> first) => [.. Enumerable.Range(0, seatCount).Select(seat =>
                first(seat) ? ("a", makeA(SeatNumber(packs, seat))) : ("b", makeB(SeatNumber(packs, seat))))];

            drafted[pod] = format != DraftFormats.Booster
                ? DraftLab.Table(cube, packs, format, [("a", makeA(SeatNumber(packs, 0))), ("b", makeB(SeatNumber(packs, 1)))], ranked)
                : apart
                ? [.. Pod(cube, packs, Table(_ => true), ranked), .. Pod(cube, packs, Table(_ => false), ranked)]
                : Pod(cube, packs, Table(seat => (seat + pod) % 2 == 0), ranked);
        });

        Console.WriteLine(
            $"Drafted and built in {clock.Elapsed.TotalSeconds:N1}s: a pick took {Stopwatch.GetElapsedTime(0, _pickTicks).TotalMilliseconds / Math.Max(1, _picks):N2} ms on average, " +
            $"a deck {Stopwatch.GetElapsedTime(0, _buildTicks).TotalMilliseconds / Math.Max(1, _builds):N1} ms (both ways of drafting together).");
        Console.WriteLine();
        Report(cube, a, [.. drafted.SelectMany(decks => decks).Where(deck => deck.Kind == "a")]);
        Report(cube, b, [.. drafted.SelectMany(decks => decks).Where(deck => deck.Kind == "b")]);

        // A deck's file: its pod and seat, and which table it sat at where each way of drafting had its own.
        string Named(int pod, SeatDeck deck) => $"pod{pod + 1:D3}-{(apart ? deck.Kind : "seat")}{deck.Seat}-{Safe(deck.Kind == "a" ? a : b)}";

        void Write(IReadOnlyDictionary<SeatDeck, (int Wins, int Played)>? results)
        {
            Directory.CreateDirectory(output!);
            int written = 0;
            for (int pod = 0; pod < pods && written < limit; pod++)
            {
                foreach (SeatDeck deck in drafted[pod])
                {
                    double? share = results is not null && results.TryGetValue(deck, out (int Wins, int Played) did) && did.Played > 0 ? did.Wins / (double)did.Played : null;
                    if (written >= limit || (keep is not null && !keep(Measure(cube, deck), share)))
                    {
                        continue;
                    }

                    string kind = deck.Kind == "a" ? a : b;
                    File.WriteAllText(Path.Combine(output!, Named(pod, deck) + ".txt"), DeckText(cube, deck, $"{kind}, pod {pod + 1} (packs {seed + (ulong)pod}) seat {deck.Seat}, {cube.Name}"));
                    written++;
                }
            }

            Console.WriteLine($"{written} decklists written to {output}{(keep is null ? string.Empty : $", of {drafted.Sum(decks => decks.Count)} drafted")}.");
            Console.WriteLine();
        }

        // What the decks of this run are, in a line: a folder that holds the games of one run is not given the
        // decks of another, or draft shape would read each game beside decks it was not played with.
        string made = $"{a} against {b}, {pods} pods of {(format == DraftFormats.Booster ? seatCount : 2)} from {seed}, {cube.Name}{(ranked ? string.Empty : ", no order")}{(apart ? ", apart" : string.Empty)}{(format == DraftFormats.Booster ? string.Empty : ", " + format)}";
        if (output is not null)
        {
            string run = Path.Combine(output, "run.txt");
            if (games == 0 && File.Exists(Path.Combine(output, "games.csv")) && File.Exists(run) && File.ReadAllText(run).Trim() != made)
            {
                throw new ArgumentException($"{output} holds the games of another run ({File.ReadAllText(run).Trim()}): write these decks somewhere else.");
            }

            Directory.CreateDirectory(output);
            File.WriteAllText(run, made + "\n");
        }

        // Decks chosen by how they did are written once they have played; any others before, so that a game that
        // stops can be played again from its files.
        bool afterwards = keep is not null && games > 0;
        keep?.Invoke(Measure(cube, drafted[0][0]), null);
        if (output is not null && !afterwards)
        {
            Write(null);
        }

        IReadOnlyDictionary<SeatDeck, (int Wins, int Played)> results = games > 0
            ? Games(root, cube, a, b, drafted, games, seed, parallelism, Number(args, "--cards", 0), output, apart, Pilot(root, pilotName), PilotLabel(root, pilotName), Named)
            : new Dictionary<SeatDeck, (int Wins, int Played)>();
        if (output is null)
        {
            return 0;
        }

        if (afterwards)
        {
            Console.WriteLine();
            Write(results);
        }

        // Every deck with what it is and how it did, for asking afterwards what about a deck wins games (draft shape).
        // A deck is its pod, its side and its seat, which is how games.csv names the two decks of a game.
        var table = new StringBuilder();
        for (int pod = 0; pod < pods; pod++)
        {
            foreach (SeatDeck deck in drafted[pod])
            {
                Measured m = Measure(cube, deck);
                if (table.Length == 0)
                {
                    table.Append("pod,seat,side,kind,wins,played,").AppendJoin(',', m.Columns.Select(column => column.Name)).Append('\n');
                }

                (int won, int played) = results.GetValueOrDefault(deck);
                table.Append(CultureInfo.InvariantCulture, $"{pod + 1},{deck.Seat},{deck.Kind},{(deck.Kind == "a" ? Safe(a) : Safe(b))},{won},{played},")
                    .AppendJoin(',', m.Columns.Select(column => column.Value.ToString("0.###", CultureInfo.InvariantCulture))).Append('\n');
            }
        }

        File.WriteAllText(Path.Combine(output, "decks.csv"), table.ToString());
        return 0;
    }

    /// <summary>Who plays the decks, both of them: the tactical seat as shipped, on its hand-written weights (<c>prior</c>), or on a model file's.</summary>
    /// <summary>A pilot as a result should name it: a frozen one by the build it is, "referee main-1c9a5242".</summary>
    private static string PilotLabel(string root, string name) =>
        Champion.Named(root, name) is { } champion && !name.EndsWith(champion.Name, StringComparison.Ordinal) ? $"{name} {champion.Name}" : name;

    private static Func<IAutoPlayer> Pilot(string root, string name)
    {
        if (name is "tactical" or "shipped")
        {
            return () => new TacticalPlay();
        }

        // A frozen pilot (referee, champion, champion:NAME): the measure that stays put while the shipped one moves.
        if (Champion.Named(root, name) is { } champion)
        {
            return champion.Seat;
        }

        TacticalModel model = name == "prior" ? TacticalModel.Prior() : TacticalModel.Parse(File.ReadAllText(File.Exists(name) ? name : Path.Combine(root, name)));
        return () => new TacticalPlay(model);
    }

    private static string Safe(string name) => string.Concat(name.Select(letter => char.IsLetterOrDigit(letter) || letter is '-' or '.' ? letter : '_'));

    private static void Report(Cube cube, string name, List<SeatDeck> decks)
    {
        List<Measured> all = [.. decks.Select(deck => Measure(cube, deck))];
        if (all.Count == 0)
        {
            return;
        }

        Console.WriteLine($"{name}: {all.Count} decks");
        Console.WriteLine(
            $"  size {all.Average(deck => deck.Size):N1} ({all.Count(deck => deck.Size != Minimum)} not {Minimum}), lands {all.Average(deck => deck.Lands):N1}, " +
            $"colours {all.Average(deck => deck.Colors):N2} ({string.Join(", ", all.GroupBy(deck => deck.Colors).OrderBy(group => group.Key).Select(group => $"{group.Key}: {group.Count()}"))})");
        Console.WriteLine(
            $"  creatures {all.Average(deck => deck.Creatures):N1}, average mana value {all.Average(deck => deck.ManaValue):N2}, sources a colour {all.Average(deck => deck.Sources):N1}");
        Console.WriteLine(
            $"  removal {all.Average(deck => deck["removal"]):N1}, counterspells {all.Average(deck => deck["counterspells"]):N1}, card advantage {all.Average(deck => deck["cardAdvantage"]):N1}, " +
            $"two mana or less {all.Average(deck => deck["mv1"] + deck["mv2"]):N1}, five or more {all.Average(deck => deck["mv5"] + deck["mv6"]):N1}");
        Console.WriteLine("  " + string.Join(", ", Archetypes.Select(name => $"{name} {all.Average(deck => deck[name]):P0}")));
        Console.WriteLine(
            $"  spells' worth {all.Average(deck => deck.Worth):N1}, as far as the lands cast them {all.Average(deck => deck.Cast):N1}; " +
            $"hard to cast {all.Average(deck => deck.Uncastable):N2} a deck; good cards left out {all.Average(deck => deck.Unplayed):N1}");
        Console.WriteLine();
    }

    /// <summary>
    /// Every deck one way of drafting made against every deck the other made in its pod - or, where each had the table
    /// to itself, against the other table's decks at the seats that alternate with its own, which is as many games.
    /// </summary>
    private static IReadOnlyDictionary<SeatDeck, (int Wins, int Played)> Games(
        string root, Cube cube, string a, string b, List<SeatDeck>[] drafted, int games, ulong seed, int parallelism, int cards, string? output, bool apart,
        Func<IAutoPlayer> pilot, string pilotName, Func<int, SeatDeck, string> named)
    {
        string content = Path.Combine(root, "content");
        List<(int Pod, SeatDeck A, SeatDeck B)> pairings = [];
        for (int pod = 0; pod < drafted.Length; pod++)
        {
            foreach (SeatDeck first in drafted[pod].Where(deck => deck.Kind == "a"))
            {
                foreach (SeatDeck second in drafted[pod].Where(deck => deck.Kind == "b"))
                {
                    if (!apart || (first.Seat + second.Seat) % 2 == 1)
                    {
                        pairings.Add((pod, first, second));
                    }
                }
            }
        }

        Console.WriteLine($"{pairings.Count} pairings, {games} game(s) each, the tactical seat ({pilotName}) playing both decks.");
        var clock = Stopwatch.StartNew();
        int winsA = 0;
        int winsB = 0;
        int draws = 0;
        long turns = 0;
        var failures = new ConcurrentBag<string>();
        var played = new ConcurrentBag<(int Index, string Row)>();
        var perCard = new ConcurrentDictionary<string, (int Wins, int Played)>(StringComparer.Ordinal);
        var perCardOfA = new ConcurrentDictionary<string, (int Wins, int Played)>(StringComparer.Ordinal);
        var perDeck = new ConcurrentDictionary<SeatDeck, (int Wins, int Played)>();
        int[] podWinsA = new int[drafted.Length];
        int[] podDecided = new int[drafted.Length];

        Parallel.For(0, pairings.Count * games, new ParallelOptions { MaxDegreeOfParallelism = parallelism }, index =>
        {
            (int pod, SeatDeck first, SeatDeck second) = pairings[index / games];
            int game = index % games;
            bool swapped = game % 2 == 1;
            DeckList deckA = AsDeckList(cube, first, named(pod, first));
            DeckList deckB = AsDeckList(cube, second, named(pod, second));
            // Drafted decks are Limited's, with the lab's minimum (CR 100.2b).
            MatchResult result = Match.Play(
                content, swapped ? [deckB, deckA] : [deckA, deckB], Cantrip.Decks.DeckFormat.Limited(Minimum), seed + (ulong)(index * 7919), [pilot(), pilot()], sample: false);

            if (!result.Finished)
            {
                // Said with what plays it again: the two decks in their seats, beside the shuffle the failure names.
                failures.Add($"{result.Failure}  [seat 1 {(swapped ? deckB : deckA).Name}, seat 2 {(swapped ? deckA : deckB).Name}]");
                return;
            }

            // A game as a line of games.csv: the two decks by their seats in the pod, which was on the play, which won.
            string began = result.Starter == 0 ? string.Empty : (result.Starter == 1) != swapped ? "a" : "b";
            string winner = result.Winner == 0 ? "draw" : (result.Winner == 1) != swapped ? "a" : "b";
            played.Add((index, string.Create(CultureInfo.InvariantCulture, $"{pod + 1},{first.Seat},{second.Seat},{game + 1},{began},{winner},{result.Turns}")));

            Interlocked.Add(ref turns, result.Turns);
            if (result.Winner == 0)
            {
                Interlocked.Increment(ref draws);
                return;
            }

            bool aWon = result.Winner == (swapped ? 2 : 1);
            Interlocked.Increment(ref aWon ? ref winsA : ref winsB);
            Interlocked.Increment(ref podDecided[pod]);
            if (aWon)
            {
                Interlocked.Increment(ref podWinsA[pod]);
            }

            foreach ((SeatDeck deck, bool won) in (IEnumerable<(SeatDeck, bool)>)[(first, aWon), (second, !aWon)])
            {
                perDeck.AddOrUpdate(deck, (won ? 1 : 0, 1), (_, was) => (was.Wins + (won ? 1 : 0), was.Played + 1));
                foreach (string id in deck.Main.Distinct(StringComparer.Ordinal))
                {
                    perCard.AddOrUpdate(id, (won ? 1 : 0, 1), (_, was) => (was.Wins + (won ? 1 : 0), was.Played + 1));
                    if (ReferenceEquals(deck, first))
                    {
                        perCardOfA.AddOrUpdate(id, (won ? 1 : 0, 1), (_, was) => (was.Wins + (won ? 1 : 0), was.Played + 1));
                    }
                }
            }
        });

        int decided = winsA + winsB;
        double share = decided == 0 ? 0 : winsA / (double)decided;

        // Games in a pod are not independent - the same eight decks play all of them, and a pod where one side drafted
        // the better decks counts thirty-two times - so the margin is taken over pods, each pod's share one sample.
        List<double> shares = [.. Enumerable.Range(0, drafted.Length).Where(pod => podDecided[pod] > 0).Select(pod => podWinsA[pod] / (double)podDecided[pod])];
        double mean = shares.Count == 0 ? 0 : shares.Average();
        double spread = shares.Count < 2 ? 0 : Math.Sqrt(shares.Sum(one => (one - mean) * (one - mean)) / (shares.Count - 1));
        double margin = shares.Count < 2 ? 0 : 1.96 * spread / Math.Sqrt(shares.Count);
        Console.WriteLine($"{a} beat {b} {winsA} to {winsB}{(draws > 0 ? $", {draws} drawn" : string.Empty)} ({share:P1} +/- {margin:P1}).");
        Console.WriteLine($"{decided + draws} games in {clock.Elapsed.TotalSeconds:N0}s, {turns / (double)Math.Max(1, decided + draws):N1} turns a game.");

        if (cards > 0)
        {
            // How the decks playing a card did, set against what the card is thought to be worth: a strong card whose
            // decks lose is one the seat playing it cannot use, or one the pick order flatters.
            int enough = Math.Max(20, (decided / 60));
            List<(string Id, double Share, int Played)> known = [.. perCard
                .Where(card => card.Value.Played >= enough && !cube.Known[card.Key].IsLand)
                .Select(card => (card.Key, card.Value.Wins / (double)card.Value.Played, card.Value.Played))
                .OrderBy(card => card.Item2)];
            Console.WriteLine();
            Console.WriteLine($"Cards by how the decks playing them did ({enough} games or more; worth by the pick order beside it):");
            Console.WriteLine("  worst: " + string.Join(", ", known.Take(cards).Select(card => $"{cube.View(card.Id).Name} {card.Share:P0} of {card.Played} ({cube.Known[card.Id].Worth:N1})")));
            Console.WriteLine("  best:  " + string.Join(", ", known.TakeLast(cards).Reverse().Select(card => $"{cube.View(card.Id).Name} {card.Share:P0} of {card.Played} ({cube.Known[card.Id].Worth:N1})")));

            // What the first way of drafting plays and the second mostly does not, by how its own decks did with it:
            // a card it likes that keeps losing is one it overvalues, or one the seat playing it cannot use.
            Dictionary<string, int> decksOfA = [];
            Dictionary<string, int> decksOfB = [];
            foreach (SeatDeck deck in drafted.SelectMany(pod => pod))
            {
                Dictionary<string, int> into = deck.Kind == "a" ? decksOfA : decksOfB;
                foreach (string id in deck.Main.Distinct(StringComparer.Ordinal))
                {
                    into[id] = into.GetValueOrDefault(id) + 1;
                }
            }

            List<(string Id, double Share, int Played)> favoured = [.. perCardOfA
                .Where(card => card.Value.Played >= enough && !cube.Known[card.Key].IsLand
                    && decksOfA.GetValueOrDefault(card.Key) >= 3 * Math.Max(1, decksOfB.GetValueOrDefault(card.Key)))
                .Select(card => (card.Key, card.Value.Wins / (double)card.Value.Played, card.Value.Played))
                .OrderBy(card => card.Item2)];
            double shareOfA = decided == 0 ? 0 : winsA / (double)decided;
            Console.WriteLine();
            Console.WriteLine($"Cards {a} plays three times as often as {b}, by how {a}'s decks did with them ({a} won {shareOfA:P0} overall; decks of {a}, of {b}, worth):");
            Console.WriteLine("  worst: " + string.Join(", ", favoured.Take(cards).Select(card =>
                $"{cube.View(card.Id).Name} {card.Share:P0} of {card.Played} ({decksOfA.GetValueOrDefault(card.Id)}, {decksOfB.GetValueOrDefault(card.Id)}, {cube.Known[card.Id].Worth:N1})")));
        }

        if (output is not null)
        {
            // Every game: the seats of its two decks, as decks.csv has them, which was on the play, and which won.
            Directory.CreateDirectory(output);
            File.WriteAllText(Path.Combine(output, "games.csv"), "pod,a,b,game,first,winner,turns\n" + string.Concat(played.OrderBy(game => game.Index).Select(game => game.Row + "\n")));
            File.WriteAllText(Path.Combine(output, "pilot.txt"), pilotName + "\n");
        }

        if (!failures.IsEmpty)
        {
            Console.WriteLine();
            Console.WriteLine($"{failures.Count} game(s) did not finish:");
            foreach (string failure in failures.Order(StringComparer.Ordinal).Take(12))
            {
                Console.WriteLine($"  {failure}");
            }
        }

        return perDeck;
    }

    // --- draft watch ----------------------------------------------------------------------------------------------------

    /// <summary>
    /// One draft told as one seat thought it, pick by pick, in the words a bot seat narrates in at a table
    /// (<see cref="DraftPlay.Narrator"/>): a booster pod of bots, a grid or Winston table of two (<c>--format</c>), or -
    /// with <c>--event ID</c> - a booster draft kept in a data directory, replayed for one of its seats from its log.
    /// </summary>
    public static int Watch(string root, string[] args)
    {
        if (Text(args, "--event", string.Empty) is { Length: > 0 } eventId)
        {
            return WatchEvent(root, args, eventId);
        }

        ulong seed = (ulong)Number(args, "--seed", 20261003);
        int watched = Number(args, "--seat", 1);
        string format = Text(args, "--format", DraftFormats.Booster);
        string others = Text(args, "--others", "new");
        bool ranked = !args.Contains("--no-order");
        Cube cube = Cube.Load(root, Text(args, "--cube", "LSVCube"));
        int count = format == DraftFormats.Booster ? 8 : 2;
        if (watched < 1 || watched > count)
        {
            throw new ArgumentException($"--seat is 1 to {count} in a {format} draft.");
        }

        Tuning tuned = Tuned(Text(args, "--tuning", string.Empty));
        var mine = new NewSeat(tuned.Draft, tuned.DraftsBy, tuned.BuildsBy) { Narrator = Console.Out };
        Func<ulong, ISeat> rest = Seat(others);
        List<(string, ISeat)> seats = [.. Enumerable.Range(1, count).Select(seat => seat == watched ? ("a", (ISeat)mine) : ("b", rest(SeatNumber(seed, seat - 1))))];

        List<SeatDeck> decks = format == DraftFormats.Booster ? Pod(cube, seed, seats, ranked) : Table(cube, seed, format, seats, ranked);
        SeatDeck deck = decks[watched - 1];
        Console.WriteLine();
        Console.WriteLine(DeckText(cube, deck, $"Seat {watched}'s deck"));
        Console.WriteLine(string.Join(", ", Measure(cube, deck).Columns.Select(column => $"{column.Name} {column.Value:0.##}")));

        // What telling costs: this seat's picks, written out, against the others' in silence.
        double told = Stopwatch.GetElapsedTime(0, mine.Spent.Ticks).TotalMilliseconds / Math.Max(1, mine.Spent.Picks);
        double quiet = Stopwatch.GetElapsedTime(0, _pickTicks - mine.Spent.Ticks).TotalMilliseconds / Math.Max(1, _picks - mine.Spent.Picks);
        Console.Error.WriteLine($"A pick told took {told:N2} ms, written out; one in silence {quiet:N2} ms.");
        return 0;
    }

    /// <summary>
    /// A booster draft kept in a data directory (<c>--data</c>, <c>CANTRIP_DATA</c>, or the repository's
    /// <c>.cantrip-data</c>), told as the drafter would have thought it at one seat: each pack the seat was shown, from
    /// the draft's log, put to the drafter as it ships, and then the deck it would build. A bot seat drafts by nothing
    /// else, so for one of the event's own bots this is what it thought at the time; for a person's seat it is what the
    /// bot would have taken, set beside what was.
    /// </summary>
    private static int WatchEvent(string root, string[] args, string eventId)
    {
        string data = Text(args, "--data", Environment.GetEnvironmentVariable("CANTRIP_DATA") is { Length: > 0 } configured ? configured : Path.Combine(root, ".cantrip-data"));
        // Only read: a store opened on a directory without both of its folders would make them.
        if (!Directory.Exists(Path.Combine(data, "events")) || !Directory.Exists(Path.Combine(data, "players")))
        {
            throw new ArgumentException($"{data} is not a data directory: --data names one (or CANTRIP_DATA).");
        }

        var store = new EventStore(data);
        EventRecord record = store.LoadEvent(eventId) ?? throw new ArgumentException($"No event '{eventId}' in {data}.");
        if (record.Kind != EventKinds.BoosterDraft)
        {
            throw new ArgumentException($"'{eventId}' is a {record.Kind}: only a booster draft keeps what each seat was shown. A grid or Winston draft would have to be dealt again from its seed.");
        }

        BoosterDraft draft = store.LoadDraft<BoosterDraft>(eventId) ?? throw new ArgumentException($"Event '{eventId}' has no draft saved.");
        int watched = Number(args, "--seat", 1);
        List<BoosterPick> log = [.. draft.Log.Where(pick => pick.Seat == watched)];
        if (log.Count == 0)
        {
            throw new ArgumentException($"Seat {watched} made no picks in '{eventId}'.");
        }

        // Every card as a client is shown it, whether or not the cube still holds it - and, as the server shows it, by
        // its id alone if the content no longer has it.
        ContentLibrary content = ContentCache.Get(Path.Combine(root, "content"));
        var projector = new CardOracle(content.Catalog());
        Dictionary<string, ObjectView> views = new(StringComparer.Ordinal);
        ObjectView View(string id)
        {
            if (!views.TryGetValue(id, out ObjectView? view))
            {
                views[id] = view = projector.Oracle(new CardId(id)) ?? new ObjectView { Known = true, CardId = id, Name = id };
            }

            return view;
        }

        // What may be spent for an additional pick, read from the content as the server reads it when it loads a pod.
        bool Spendable(string id) => draft.AdditionalPicks.Contains(id)
            || (content.Cards.TryGet(new CardId(id), out CardDefinition? card) && card!.DraftAbilities.Contains(DraftAbility.AdditionalPickPutThisIntoPack));

        EventParticipant? who = record.AtSeat(watched);
        Console.WriteLine($"{record.Name} ({eventId}), seat {watched}{(who is null ? string.Empty : $", {who.Name}")}: each pack it was shown, as the drafter of this build weighs it.");
        Console.WriteLine();

        var seat = new NewSeat(new DraftTuning()) { Narrator = Console.Out };
        int total = draft.Seats * draft.Packs * draft.PackSize;
        List<string> pool = [];
        for (int at = 0; at < log.Count;)
        {
            // A pick and the cards taken with it for a Cogwork Librarian spent: one decision.
            BoosterPick pick = log[at];
            List<BoosterPick> taken = [.. log.Skip(at).TakeWhile((other, index) => index == 0 || (other.Spent.Length > 0 && other.Pack == pick.Pack && other.Pick == pick.Pick))];
            at += taken.Count;

            var shown = new DraftMessage
            {
                Format = DraftFormats.Booster,
                Active = watched,
                Decision = at,
                Pile = [.. pick.Offered.Select(View)],
                PileRanks = draft.PickOrder.Count == 0 ? [] : [.. pick.Offered.Select(draft.Rank)],
                AdditionalPicks = pick.Offered.Count <= 1 ? [] : [.. Enumerable.Range(0, pool.Count).Reverse().Where(place => Spendable(pool[place]))],
                Pool = [.. pool.Select(View)],
                Seats = [.. Enumerable.Range(1, draft.Seats).Select(other => new DraftSeatView { Seat = other, Name = $"Seat {other}" })],
                Total = total,
                PackNumber = pick.Pack,
                Packs = draft.Packs,
                PackSize = draft.PackSize,
                PickNumber = pick.Pick,
                Direction = pick.Pack % 2 == 0 ? -1 : 1,
            };

            List<string> chosen = [.. seat.Pick(shown).Select(place => pick.Offered[place])];
            List<string> actual = [.. taken.Select(card => card.Card)];
            if (!chosen.SequenceEqual(actual, StringComparer.Ordinal))
            {
                string how = pick.Auto ? " (the clock's pick)" : string.Empty;
                Console.WriteLine($"  !! the seat took {string.Join(" and ", actual.Select(id => View(id).Name))}{how}");
            }

            // The last drafted copy goes into the pack first, as the server spends it.
            foreach (BoosterPick card in taken.Where(card => card.Spent.Length > 0))
            {
                pool.RemoveAt(pool.LastIndexOf(card.Spent));
            }

            pool.AddRange(actual);
        }

        Console.WriteLine();
        var building = new DeckBuildingMessage
        {
            Pool = [.. pool.Select(View)],
            Main = [.. Enumerable.Range(0, pool.Count)],
            Basics = [.. PoolDecks.BasicLands.Select(name => new BasicLandView { Name = name })],
            Minimum = record.MinimumDeck,
        };

        (IReadOnlyList<int> main, IReadOnlyDictionary<string, int> basics) = seat.Build(building);
        if (who?.Deck is { Main.Count: > 0 } submitted && who.Pool.SequenceEqual(pool, StringComparer.Ordinal))
        {
            bool same = submitted.Main.Order().SequenceEqual(main.Order())
                && submitted.Basics.Where(pair => pair.Value > 0).OrderBy(pair => pair.Key, StringComparer.Ordinal)
                    .SequenceEqual(basics.Where(pair => pair.Value > 0).OrderBy(pair => pair.Key, StringComparer.Ordinal));
            Console.WriteLine(same ? "  the deck submitted is this one" : "  !! the deck submitted was another: the seat built its own, or the drafter has changed since");
        }

        return 0;
    }

    // --- draft calibrate -----------------------------------------------------------------------------------------------

    /// <summary>
    /// What a pack usually still holds: the best card of one colour, and the best a deck of one colour or of two could
    /// play, by how many cards are left in the pack. Printed as the tables the drafter carries.
    /// </summary>
    public static int Calibrate(string root, string[] args, int parallelism)
    {
        int pods = Number(args, "--pods", 200);
        ulong seed = (ulong)Number(args, "--seed", 77001);
        Cube cube = Cube.Load(root, Text(args, "--cube", "LSVCube"));
        Func<ulong, ISeat> make = Seat(Text(args, "--a", "new"));

        const int PackSize = 15;
        double[] ofColor = new double[PackSize + 1];
        double[] mono = new double[PackSize + 1];
        double[] pair = new double[PackSize + 1];
        long[] seen = new long[PackSize + 1];
        object gate = new();

        // What the signals are worth as forecasts: for each seat and colour, what was read earlier beside what then came.
        ConcurrentBag<(Dictionary<string, double> Read, double Came)>[] sameRound = [[], [], []];
        var nextRound = new ConcurrentBag<(Dictionary<string, double> Read, double Came)>();
        var roundAfter = new ConcurrentBag<(Dictionary<string, double> Read, double Came)>();

        Parallel.For(0, pods, new ParallelOptions { MaxDegreeOfParallelism = parallelism }, pod =>
        {
            double[] localColor = new double[PackSize + 1];
            double[] localMono = new double[PackSize + 1];
            double[] localPair = new double[PackSize + 1];
            long[] localSeen = new long[PackSize + 1];
            List<(string Kind, ISeat Seat)> seats = [.. Enumerable.Range(0, 8).Select(seat => ("a", make(SeatNumber(seed + (ulong)pod, seat))))];
            Pod(cube, seed + (ulong)pod, seats, watch: (_, shown, _) =>
            {
                List<DraftCard> pack = [.. shown.Pile.Select(view => cube.Known[view.CardId])];
                int left = pack.Count;
                localSeen[left]++;
                localColor[left] += DraftMemory.BestOfEachColor(pack).Average();
                foreach (int colors in DeckBuilder.Combinations)
                {
                    double best = pack.Where(card => !card.IsLand && card.OffColor(colors) == 0).Select(card => card.Power).DefaultIfEmpty(1.0).Max();
                    (int.PopCount(colors) == 1 ? localMono : localPair)[left] += best / (int.PopCount(colors) == 1 ? 5 : 10);
                }
            });

            foreach (NewSeat seat in seats.Select(seat => seat.Seat).OfType<NewSeat>())
            {
                DraftMemory memory = seat.Drafter.Memory;

                // What came: how far above the usual the best card of each colour stood, on average, over some of the packs seen.
                double[] Came(Func<DraftMemory.Sight, bool> which)
                {
                    double[] sum = new double[5];
                    int count = 0;
                    foreach (DraftMemory.Sight sight in memory.Sights.Where(which))
                    {
                        double[] best = DraftMemory.BestOfEachColor(sight.Cards);
                        double usual = DraftMemory.Expected(sight.Cards.Count, PackSize);
                        for (int color = 0; color < 5; color++)
                        {
                            sum[color] += best[color] - usual;
                        }

                        count++;
                    }

                    return [.. sum.Select(total => total / Math.Max(1, count))];
                }

                // What had been read by the sixth pick of each pack, as the drafter keeps it, beside the rest of that pack.
                for (int round = 1; round <= 3; round++)
                {
                    double[] early = new double[5];
                    int looks = 0;
                    foreach (DraftMemory.Sight sight in memory.Sights.Where(sight => sight.Round == round && sight.Pick is >= 2 and <= 6))
                    {
                        double[] reading = DraftMemory.Reading(sight.Cards);
                        for (int color = 0; color < 5; color++)
                        {
                            early[color] += reading[color];
                        }

                        looks++;
                    }

                    double[] late = Came(sight => sight.Round == round && sight.Pick is >= 7 and <= 12);
                    for (int color = 0; color < 5; color++)
                    {
                        sameRound[round - 1].Add((new() { ["bias"] = 1, ["Signals"] = early[color] / (looks + DraftMemory.Doubt) }, late[color]));
                    }
                }

                double[] two = Came(sight => sight.Round == 2 && sight.Pick is >= 2 and <= 12);
                double[] three = Came(sight => sight.Round == 3 && sight.Pick is >= 2 and <= 12);
                for (int color = 0; color < 5; color++)
                {
                    nextRound.Add((new() { ["bias"] = 1, ["Cut"] = -memory.Passed(color, 1), ["Crowd"] = -memory.Contested(color) }, two[color]));
                    roundAfter.Add((new() { ["bias"] = 1, ["Carry"] = memory.Flow(color, 1), ["Cut"] = -memory.Passed(color, 2), ["Crowd"] = -memory.Contested(color) }, three[color]));
                }
            }

            lock (gate)
            {
                for (int left = 0; left <= PackSize; left++)
                {
                    ofColor[left] += localColor[left];
                    mono[left] += localMono[left];
                    pair[left] += localPair[left];
                    seen[left] += localSeen[left];
                }
            }
        });

        string Row(double[] sums, double empty) => string.Join(", ", Enumerable.Range(0, PackSize + 1).Select(left => (left == 0 ? empty : sums[left] / Math.Max(1, seen[left])).ToString("N2", CultureInfo.InvariantCulture)));
        Console.WriteLine($"{pods} pods of {cube.Name}. By cards left in the pack, 0 to {PackSize}:");
        Console.WriteLine($"DraftMemory.UsualBestOfColor: [{Row(ofColor, 1.5)}]");
        Console.WriteLine($"CubeDrafter.UsualBest, one colour: [{Row(mono, 1.0)}]");
        Console.WriteLine($"CubeDrafter.UsualBest, two:        [{Row(pair, 1.0)}]");

        // What came, fitted to what was read: the weights the drafter gives each reading.
        static string Fitted(IEnumerable<(Dictionary<string, double> Read, double Came)> pairs)
        {
            List<(Dictionary<string, double> Features, double Worth)> all = [.. pairs];
            List<string> names = [.. all[0].Features.Keys];
            double[] weights = Solve(all, names, 1e-9);
            double mean = all.Average(pair => pair.Worth);
            double spread = all.Sum(pair => (pair.Worth - mean) * (pair.Worth - mean));
            double missed = all.Sum(pair => Math.Pow(pair.Worth - names.Select((name, at) => weights[at] * pair.Features[name]).Sum(), 2));
            return string.Join(", ", names.Select((name, at) => (name, at)).Where(pair => pair.name != "bias").Select(pair => $"{pair.name} {weights[pair.at]:0.00}"))
                + $"  (accounts for {1 - (missed / spread):P1} of what came, over {all.Count})";
        }

        if (roundAfter.IsEmpty)
        {
            // Only the cube drafter reads signals; a pod of anything else has none to weigh.
            return 0;
        }

        Console.WriteLine();
        Console.WriteLine("What the readings forecast, a seat and a colour at a time: the best card of the colour to come, above the usual,");
        Console.WriteLine("fitted to what had been read. These are DraftTuning's weights.");
        for (int round = 1; round <= 3; round++)
        {
            Console.WriteLine($"  the rest of pack {round}, from its first six picks:    {Fitted(sameRound[round - 1])}");
        }

        Console.WriteLine($"  pack two, from pack one:                         {Fitted(nextRound)}");
        Console.WriteLine($"  pack three, from packs one and two:              {Fitted(roundAfter)}");
        return 0;
    }

    // --- draft fit -----------------------------------------------------------------------------------------------------

    /// <summary>
    /// Fits what a card is worth going by the card alone (<see cref="CardWorth.Weights"/>) to the pick orders there are:
    /// a ridge regression of every ranked card's worth on what it costs, is and says. Prints the table.
    /// </summary>
    public static int Fit(string root, string[] args)
    {
        double ridge = Real(args, "--ridge", 25.0);
        List<(Dictionary<string, double> Features, double Worth)> samples = [];
        HashSet<string> counted = new(StringComparer.Ordinal);
        foreach (string name in Directory.GetFiles(Path.Combine(root, "docs", "cube-lists"), "*.picks.json").Select(path => Path.GetFileName(path).Replace(".picks.json", string.Empty, StringComparison.Ordinal)))
        {
            Cube cube = Cube.Load(root, name);
            foreach (string id in cube.PickOrder.Where(counted.Add))
            {
                DraftCard card = cube.Known[id];
                samples.Add((CardWorth.Features(card, (card.View.OracleText ?? string.Empty).ToLowerInvariant()), card.Worth));
            }
        }

        List<string> names = [.. samples.SelectMany(sample => sample.Features.Keys).GroupBy(feature => feature).Where(group => group.Count() >= 4 || group.Key == "bias").Select(group => group.Key)
            .Union(CardWorth.Prior.Keys).Order(StringComparer.Ordinal)];
        Console.WriteLine($"{samples.Count} ranked cards, {names.Count} features seen on four cards or more, ridge {ridge}.");

        // Held-out fit first, in five folds: how well a card nobody ranked would be guessed.
        double squared = 0;
        double spread = 0;
        double mean = samples.Average(sample => sample.Worth);
        for (int fold = 0; fold < 5; fold++)
        {
            double[] weights = Solve([.. samples.Where((_, at) => at % 5 != fold)], names, ridge);
            foreach ((Dictionary<string, double> features, double worth) in samples.Where((_, at) => at % 5 == fold))
            {
                double guess = Math.Clamp(names.Select((feature, at) => weights[at] * features.GetValueOrDefault(feature)).Sum(), 0.5, 6.0);
                squared += (guess - worth) * (guess - worth);
                spread += (mean - worth) * (mean - worth);
            }
        }

        Console.WriteLine($"Held out: explains {1 - (squared / spread):P0} of the spread in worth; off by {Math.Sqrt(squared / samples.Count):N2} a card, against {Math.Sqrt(spread / samples.Count):N2} for guessing the average.");
        Console.WriteLine();

        double[] fitted = Solve(samples, names, ridge);
        for (int at = 0; at < names.Count; at++)
        {
            Console.WriteLine($"        [\"{names[at]}\"] = {fitted[at].ToString("N3", CultureInfo.InvariantCulture)},");
        }

        return 0;
    }

    /// <summary>
    /// Ridge regression by the normal equations - few enough features that elimination will do - of what the prior
    /// leaves unexplained, so that a weight the cards say nothing about stays where the prior put it.
    /// </summary>
    private static double[] Solve(List<(Dictionary<string, double> Features, double Worth)> samples, List<string> names, double ridge)
    {
        int n = names.Count;
        double[] prior = [.. names.Select(feature => CardWorth.Prior.GetValueOrDefault(feature))];
        double[,] a = new double[n, n + 1];
        foreach ((Dictionary<string, double> features, double worth) in samples)
        {
            double[] x = [.. names.Select(feature => features.GetValueOrDefault(feature))];
            double left = worth - x.Zip(prior, (value, weight) => value * weight).Sum();
            for (int row = 0; row < n; row++)
            {
                if (x[row] == 0)
                {
                    continue;
                }

                for (int column = 0; column < n; column++)
                {
                    a[row, column] += x[row] * x[column];
                }

                a[row, n] += x[row] * left;
            }
        }

        for (int at = 0; at < n; at++)
        {
            // What a creature's size is worth is not the cubes' to say: every big body in one is there for its text,
            // so they would have it that size counts against a card.
            a[at, at] += names[at] == "bias" ? 1e-6 : names[at] == "creature.body" ? 1e9 : ridge;
        }

        for (int pivot = 0; pivot < n; pivot++)
        {
            int best = pivot;
            for (int row = pivot + 1; row < n; row++)
            {
                if (Math.Abs(a[row, pivot]) > Math.Abs(a[best, pivot]))
                {
                    best = row;
                }
            }

            for (int column = 0; column <= n; column++)
            {
                (a[pivot, column], a[best, column]) = (a[best, column], a[pivot, column]);
            }

            for (int row = 0; row < n; row++)
            {
                if (row == pivot || a[row, pivot] == 0)
                {
                    continue;
                }

                double factor = a[row, pivot] / a[pivot, pivot];
                for (int column = pivot; column <= n; column++)
                {
                    a[row, column] -= factor * a[pivot, column];
                }
            }
        }

        return [.. Enumerable.Range(0, n).Select(at => prior[at] + (a[at, n] / a[at, at]))];
    }

    // --- draft shape ---------------------------------------------------------------------------------------------------

    /// <summary>What the fit of <see cref="Shape"/> weighs unless told otherwise: what the builder's own sum is made of, and what it leaves out.</summary>
    private const string ShapeFeatures = "cast,creatures,removal,counterspells,cardAdvantage,cheap,expensive,lands,colours";

    /// <summary>
    /// What about a deck wins games, read from the decks and games of earlier runs (<c>--out</c> with <c>--games</c>).
    /// <para>
    /// A Bradley-Terry fit: the chance that one deck beats another is the logistic of the difference between what
    /// each is - its columns of <c>decks.csv</c> - weighted, with one more weight for being on the play. A game is
    /// between two decks of one pod, so a pod that was dealt better cards than another says nothing either way, and
    /// the margins come from drawing the pods again, since a pod's games share its decks.
    /// </para>
    /// <para>
    /// It is printed in the builder's own unit as well - how much worth, as far as the lands cast it, a creature or a
    /// land or a counterspell is worth - which is what <see cref="DeckTuning"/>'s costs are, and beside what the
    /// builder's own sum (<c>score</c>) makes of the same games. What it says is what wins with whoever played the
    /// games: a fit to another pilot's games is another fit.
    /// </para>
    /// </summary>
    public static int Shape(string root, string[] args)
    {
        List<string> folders = [.. Enumerable.Range(0, args.Length - 1).Where(at => args[at] == "--from").Select(at => args[at + 1])];
        if (folders.Count == 0)
        {
            throw new ArgumentException("draft shape --from DIR [--from DIR ...]: folders a draft was written to with --out and --games.");
        }

        List<string> features = [.. Text(args, "--features", ShapeFeatures).Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)];
        int draws = Math.Max(2, Number(args, "--draws", 300));

        List<List<(double[] Difference, bool Won)>> pods = [.. folders.SelectMany(folder => Played(folder, features))];
        List<(double[] Difference, bool Won)> all = [.. pods.SelectMany(pod => pod)];
        if (all.Count == 0)
        {
            throw new ArgumentException("No games in those folders.");
        }

        double[] weights = Logistic(all);

        // The margins: the same fit to as many pods drawn again from these, with replacement.
        var random = new Random(7);
        double[][] drawn = new double[draws][];
        for (int draw = 0; draw < draws; draw++)
        {
            drawn[draw] = Logistic([.. Enumerable.Range(0, pods.Count).SelectMany(_ => pods[random.Next(pods.Count)])]);
        }

        double Margin(Func<double[], double> of)
        {
            double mean = drawn.Average(of);
            return 1.96 * Math.Sqrt(drawn.Sum(one => (of(one) - mean) * (of(one) - mean)) / Math.Max(1, draws - 1));
        }

        // The builder's unit is worth as cast, and only a unit if these games say worth as cast wins any.
        int play = features.Count;
        int cast = features.IndexOf("cast");
        bool inWorth = cast >= 0 && weights[cast] > 2 * Margin(one => one[cast]);
        Console.WriteLine($"{all.Count} games of {pods.Count} pods, from {string.Join(", ", folders)}.");
        Console.WriteLine("What a deck having one more of each is worth: the weight, and in points of the games it wins against a deck its equal.");
        Console.WriteLine(inWorth
            ? "And in worth as cast, the builder's unit. Margins from drawing the pods again."
            : "Not in worth as cast, the builder's unit: these games do not say that it wins any. Margins from drawing the pods again.");
        Console.WriteLine();
        Console.WriteLine($"  {"",-16}{"weight",8}{"+/-",8}{"points",9}{"+/-",7}{(inWorth ? $"{"as worth",10}{"+/-",7}" : string.Empty)}");
        for (int at = 0; at <= play; at++)
        {
            int feature = at;
            string worth = !inWorth || at == cast || at == play
                ? string.Empty
                : $"{weights[at] / weights[cast],10:N2}{Margin(one => one[feature] / one[cast]),7:N2}";
            Console.WriteLine(
                $"  {(at == play ? "on the play" : features[at]),-16}{weights[at],8:N3}{Margin(one => one[feature]),8:N3}{weights[at] * 25,8:N1}%{Margin(one => one[feature]) * 25,6:N1}%{worth}");
        }

        // Beside it, the builder's own sum alone: if it knew what wins, nothing above would add to it. And how each
        // calls games it was not fitted to (--test), which is what says the weights are of decks and not of these pods.
        static double Called(List<(double[] Difference, bool Won)> games, double[] by) =>
            1000 * (Likelihood(games, by) - Likelihood(games, new double[by.Length])) / games.Count;

        List<(double[] Difference, bool Won)> scored = [.. folders.SelectMany(folder => Played(folder, ["score"])).SelectMany(pod => pod)];
        double[] own = Logistic(scored);
        Console.WriteLine();
        Console.WriteLine($"  The builder's own sum alone: {own[0]:N3} a point of it ({own[0] * 25:N1}% of games).");
        Console.WriteLine(
            $"  How much better than a coin each calls these games, in log-likelihood a thousand games: these weights {Called(all, weights):N1}, " +
            $"the builder's sum {Called(scored, own):N1}, being on the play alone {Called(all, [.. new double[play], weights[play]]):N1}.");

        foreach (string folder in Enumerable.Range(0, args.Length - 1).Where(at => args[at] == "--test").Select(at => args[at + 1]))
        {
            List<(double[] Difference, bool Won)> others = [.. Played(folder, features).SelectMany(pod => pod)];
            List<(double[] Difference, bool Won)> othersScored = [.. Played(folder, ["score"]).SelectMany(pod => pod)];
            if (others.Count == 0)
            {
                Console.WriteLine($"  No games in {folder}.");
                continue;
            }

            Console.WriteLine(
                $"  The {others.Count} games of {folder}, which neither was fitted to: these weights {Called(others, weights):N1}, the builder's sum {Called(othersScored, own):N1}; " +
                $"weights fitted to those games themselves, {Called(others, Logistic(others)):N1}.");
        }

        if (Number(args, "--cards", 0) is > 0 and int cards)
        {
            Cards(root, args, folders, cards);
        }

        return 0;
    }

    /// <summary>The games of one run, a pod at a time: each the difference between its two decks in each feature, then which was on the play, and whether the first won.</summary>
    private static List<List<(double[] Difference, bool Won)>> Played(string folder, IReadOnlyList<string> features)
    {
        Dictionary<(int Pod, string Side, int Seat), Dictionary<string, double>> decks = [];
        foreach (Dictionary<string, string> row in Rows(Path.Combine(folder, "decks.csv")))
        {
            if (!row.ContainsKey("side"))
            {
                throw new ArgumentException($"{folder}'s decks.csv does not say which side a deck was on: it was written before draft shape, and cannot be read by it.");
            }

            decks[(int.Parse(row["pod"], CultureInfo.InvariantCulture), row["side"], int.Parse(row["seat"], CultureInfo.InvariantCulture))] = row
                .Where(cell => double.TryParse(cell.Value, NumberStyles.Float, CultureInfo.InvariantCulture, out _))
                .ToDictionary(cell => cell.Key, cell => double.Parse(cell.Value, CultureInfo.InvariantCulture), StringComparer.Ordinal);
        }

        SortedDictionary<int, List<(double[], bool)>> byPod = [];
        foreach (Dictionary<string, string> row in Rows(Path.Combine(folder, "games.csv")).Where(row => row["winner"] != "draw"))
        {
            int pod = int.Parse(row["pod"], CultureInfo.InvariantCulture);
            Dictionary<string, double> a = decks[(pod, "a", int.Parse(row["a"], CultureInfo.InvariantCulture))];
            Dictionary<string, double> b = decks[(pod, "b", int.Parse(row["b"], CultureInfo.InvariantCulture))];
            double[] difference = [.. features.Select(feature => feature == "side" ? 1 : Column(a, feature) - Column(b, feature)), row["first"] == "a" ? 1 : row["first"] == "b" ? -1 : 0];
            if (!byPod.TryGetValue(pod, out List<(double[], bool)>? games))
            {
                byPod[pod] = games = [];
            }

            games.Add((difference, row["winner"] == "a"));
        }

        return [.. byPod.Values];
    }

    /// <summary>
    /// What each card is worth to whoever played the games: the same fit with a weight a card. The chance one deck
    /// beats another is the logistic of the sum of its cards' weights less the other's, every card of a main deck but
    /// a basic land a weight of its own, held towards nothing so that a card seen little says little. A weight is
    /// against the run of cards, roughly: a basic land has none, the rest are held towards none, and a deck is always
    /// forty cards, so the cards at nothing are the ordinary ones. Read the order and the gaps, not the zero.
    /// <para>
    /// This is the table to read for a card the pilot cannot use. A card's share of the games its decks won says as
    /// much about the decks it is found in - a Mox is in good decks - and this takes the rest of the deck out. Printed
    /// beside the card's worth by the pick order: where the order rates a card and these games do not, the pilot is
    /// not playing it as people do, or the order flatters it.
    /// </para>
    /// </summary>
    private static void Cards(string root, string[] args, List<string> folders, int count)
    {
        Cube cube = Cube.Load(root, Text(args, "--cube", "LSVCube"));
        Dictionary<string, double> worth = new(StringComparer.Ordinal);
        foreach (DraftCard card in cube.Known.Values)
        {
            worth[card.Name] = card.Worth;
        }

        // Each game as the two decks' cards: the first deck's at one, the second's at minus one.
        Dictionary<string, int> index = new(StringComparer.Ordinal);
        List<(int[] First, int[] Second, double Play, bool Won)> games = [];
        foreach (string folder in folders)
        {
            Dictionary<(int Pod, int Seat), string> sideOf = Rows(Path.Combine(folder, "decks.csv"))
                .GroupBy(row => (int.Parse(row["pod"], CultureInfo.InvariantCulture), int.Parse(row["seat"], CultureInfo.InvariantCulture)))
                .ToDictionary(seat => seat.Key, seat => seat.First()["side"]);
            Dictionary<(int Pod, string Side, int Seat), int[]> decks = [];
            foreach (string path in Directory.GetFiles(folder, "pod*.txt"))
            {
                // pod012-seat3-new.txt, or pod012-a3-new.txt where each way of drafting had its own table.
                string[] parts = Path.GetFileName(path).Split('-');
                int pod = int.Parse(parts[0][3..], CultureInfo.InvariantCulture);
                bool own = parts[1][0] is 'a' or 'b' && char.IsDigit(parts[1][1]);
                int seat = int.Parse(parts[1][(own ? 1 : 4)..], CultureInfo.InvariantCulture);
                List<int> main = [];
                foreach (string line in File.ReadLines(path).TakeWhile(line => line != "Sideboard").Where(line => line.Length > 0 && line[0] != '#' && !char.IsDigit(line[0])))
                {
                    if (!index.TryGetValue(line, out int at))
                    {
                        index[line] = at = index.Count;
                    }

                    main.Add(at);
                }

                decks[(pod, own ? parts[1][..1] : sideOf[(pod, seat)], seat)] = [.. main];
            }

            foreach (Dictionary<string, string> row in Rows(Path.Combine(folder, "games.csv")).Where(row => row["winner"] != "draw"))
            {
                int pod = int.Parse(row["pod"], CultureInfo.InvariantCulture);
                if (decks.TryGetValue((pod, "a", int.Parse(row["a"], CultureInfo.InvariantCulture)), out int[]? first)
                    && decks.TryGetValue((pod, "b", int.Parse(row["b"], CultureInfo.InvariantCulture)), out int[]? second))
                {
                    games.Add((first, second, row["first"] == "a" ? 1 : row["first"] == "b" ? -1 : 0, row["winner"] == "a"));
                }
            }
        }

        if (games.Count == 0)
        {
            Console.WriteLine("  No game there has both its decklists beside it: the cards need a run written whole, without --keep.");
            return;
        }

        double ridge = Real(args, "--ridge", 4.0);
        int n = index.Count + 1;
        double[] weights = new double[n];
        double[,] system = new double[n, 2 * n];
        int[] seen = new int[n];
        foreach ((int[] first, int[] second, double _, bool _) in games)
        {
            foreach (int card in first.Concat(second))
            {
                seen[card]++;
            }
        }

        int[] columns = new int[games.Max(game => game.First.Length + game.Second.Length) + 1];
        double[] signs = new double[columns.Length];
        for (int step = 0; step < 25; step++)
        {
            double[] slope = new double[n];
            Array.Clear(system);
            foreach ((int[] first, int[] second, double play, bool won) in games)
            {
                double at = weights[n - 1] * play;
                foreach (int card in first)
                {
                    at += weights[card];
                }

                foreach (int card in second)
                {
                    at -= weights[card];
                }

                double chance = 1 / (1 + Math.Exp(-at));
                double curve = chance * (1 - chance);
                double miss = (won ? 1 : 0) - chance;

                // The game's own columns, signed: the cards of the two decks and the play.
                int filled = 0;
                foreach (int card in first)
                {
                    (columns[filled], signs[filled++]) = (card, 1);
                }

                foreach (int card in second)
                {
                    (columns[filled], signs[filled++]) = (card, -1);
                }

                (columns[filled], signs[filled++]) = (n - 1, play);
                for (int row = 0; row < filled; row++)
                {
                    slope[columns[row]] += signs[row] * miss;
                    for (int column = 0; column < filled; column++)
                    {
                        system[columns[row], columns[column]] += curve * signs[row] * signs[column];
                    }
                }
            }

            for (int at = 0; at < n; at++)
            {
                double held = at == n - 1 ? 1e-6 : ridge;
                system[at, at] += held;
                slope[at] -= held * weights[at];
                system[at, n + at] = 1;
            }

            // The curve's inverse, beside it: it gives the step now and each weight's margin at the end.
            Invert(system, n);
            double most = 0;
            for (int row = 0; row < n; row++)
            {
                double move = 0;
                for (int column = 0; column < n; column++)
                {
                    move += system[row, n + column] * slope[column];
                }

                weights[row] += move;
                most = Math.Max(most, Math.Abs(move));
            }

            if (most < 1e-7)
            {
                break;
            }
        }

        int enough = Number(args, "--enough", Math.Max(100, games.Count / 40));
        List<(string Name, double Points, double Margin, int Games, double Worth)> cards = [.. index
            .Where(card => seen[card.Value] >= enough)
            .Select(card => (card.Key, weights[card.Value] * 25, 1.96 * Math.Sqrt(system[card.Value, n + card.Value]) * 25, seen[card.Value], worth.GetValueOrDefault(card.Key, double.NaN)))
            .OrderBy(card => card.Item2)];
        List<(string Name, double Points, double Margin, int Games, double Worth)> rated = [.. cards.Where(card => !double.IsNaN(card.Worth))];
        if (rated.Count < 2)
        {
            Console.WriteLine();
            Console.WriteLine($"  No card is in {enough} games or more of these {games.Count}: too few games to say anything card by card (--enough N to ask for fewer).");
            return;
        }

        double meanPoints = rated.Average(card => card.Points);
        double meanWorth = rated.Average(card => card.Worth);
        double together = rated.Sum(card => (card.Points - meanPoints) * (card.Worth - meanWorth))
            / Math.Sqrt(rated.Sum(card => Math.Pow(card.Points - meanPoints, 2)) * rated.Sum(card => Math.Pow(card.Worth - meanWorth, 2)));

        static string Line((string Name, double Points, double Margin, int Games, double Worth) card) =>
            $"    {card.Points,6:+0.0;-0.0} +/- {card.Margin,4:N1}  {card.Games,5} games  worth {card.Worth:N1}  {card.Name}";

        Console.WriteLine();
        Console.WriteLine($"  Card by card, over {games.Count} games: what a deck gains by a card over an ordinary one, in points of the games it wins.");
        Console.WriteLine($"  Cards in {enough} games or more ({cards.Count}); margins as if each game stood alone, so somewhat narrow; worth by the pick order beside it.");
        Console.WriteLine($"  A card's weight and its worth by the pick order go together {together:+0.00;-0.00} (1 would be an order that says what wins here).");
        Console.WriteLine();
        Console.WriteLine("  The least, of the cards the pick order rates 3.5 or more - its best third:");
        foreach ((string Name, double Points, double Margin, int Games, double Worth) card in cards.Where(card => card.Worth >= 3.5).Take(count))
        {
            Console.WriteLine(Line(card));
        }

        Console.WriteLine();
        Console.WriteLine("  The least of all:");
        foreach ((string Name, double Points, double Margin, int Games, double Worth) card in cards.Take(count))
        {
            Console.WriteLine(Line(card));
        }

        Console.WriteLine();
        Console.WriteLine("  The most:");
        foreach ((string Name, double Points, double Margin, int Games, double Worth) card in cards.AsEnumerable().Reverse().Take(count))
        {
            Console.WriteLine(Line(card));
        }
    }

    /// <summary>Turns the left half of <paramref name="a"/>, <paramref name="n"/> square, into the identity, and so the right half - the identity, coming in - into its inverse.</summary>
    private static void Invert(double[,] a, int n)
    {
        for (int pivot = 0; pivot < n; pivot++)
        {
            int best = pivot;
            for (int row = pivot + 1; row < n; row++)
            {
                if (Math.Abs(a[row, pivot]) > Math.Abs(a[best, pivot]))
                {
                    best = row;
                }
            }

            if (best != pivot)
            {
                for (int column = 0; column < 2 * n; column++)
                {
                    (a[pivot, column], a[best, column]) = (a[best, column], a[pivot, column]);
                }
            }

            double scale = a[pivot, pivot];
            for (int column = 0; column < 2 * n; column++)
            {
                a[pivot, column] /= scale;
            }

            for (int row = 0; row < n; row++)
            {
                double factor = a[row, pivot];
                if (row == pivot || factor == 0)
                {
                    continue;
                }

                for (int column = 0; column < 2 * n; column++)
                {
                    a[row, column] -= factor * a[pivot, column];
                }
            }
        }
    }

    /// <summary>
    /// A column of <c>decks.csv</c> by name, and two that are sums of others: <c>cheap</c>, two mana or less, and
    /// <c>expensive</c>, five or more. (<c>side</c> is not a deck's: it is one for every game, the weight of being the
    /// first way of drafting rather than the second, for runs where the two differ in more than the features say.)
    /// </summary>
    private static double Column(Dictionary<string, double> deck, string name) =>
        deck.TryGetValue(name, out double value) ? value
        : name == "cheap" ? deck["mv1"] + deck["mv2"]
        : name == "expensive" ? deck["mv5"] + deck["mv6"]
        : throw new ArgumentException($"decks.csv has no '{name}': {string.Join(", ", deck.Keys)}, cheap, expensive.");

    // --- draft paired ---------------------------------------------------------------------------------------------------

    /// <summary>
    /// Sets two runs of the same pods side by side, pod by pod: a's share of each pod's games in the variant less in the
    /// base. The same packs and the same shuffles take out most of what a pod's draw adds to either share, so the
    /// margin of the difference is much less than that of either (+/-1.3% against +/-2.7% at 60 pods).
    /// </summary>
    public static int Paired(string[] args)
    {
        string baseDir = Text(args, "--base", string.Empty);
        string variantDir = Text(args, "--variant", string.Empty);
        if (baseDir.Length == 0 || variantDir.Length == 0)
        {
            throw new ArgumentException("draft paired needs --base DIR and --variant DIR, each a run's --out with games.");
        }

        Dictionary<int, double> Shares(string folder)
        {
            Dictionary<int, (int Won, int Played)> pods = [];
            foreach (Dictionary<string, string> row in Rows(Path.Combine(folder, "games.csv")).Where(row => row["winner"] != "draw"))
            {
                int pod = int.Parse(row["pod"], CultureInfo.InvariantCulture);
                (int won, int played) = pods.GetValueOrDefault(pod);
                pods[pod] = (won + (row["winner"] == "a" ? 1 : 0), played + 1);
            }

            return pods.ToDictionary(pod => pod.Key, pod => pod.Value.Won / (double)pod.Value.Played);
        }

        // Runs of other packs, or games of other pilots, pair nothing: say so rather than print a number.
        string Said(string folder, string file) => File.Exists(Path.Combine(folder, file)) ? File.ReadAllText(Path.Combine(folder, file)).Trim() : "?";
        string Packs(string run) => run.Contains(" pods of ", StringComparison.Ordinal) ? run[run.IndexOf(" pods of ", StringComparison.Ordinal)..] : run;
        if (Packs(Said(baseDir, "run.txt")) != Packs(Said(variantDir, "run.txt")))
        {
            Console.WriteLine($"Warning: not the same packs - base '{Said(baseDir, "run.txt")}', variant '{Said(variantDir, "run.txt")}'.");
        }

        if (Said(baseDir, "pilot.txt") != Said(variantDir, "pilot.txt"))
        {
            Console.WriteLine($"Warning: not the same pilot - base '{Said(baseDir, "pilot.txt")}', variant '{Said(variantDir, "pilot.txt")}'.");
        }

        Dictionary<int, double> before = Shares(baseDir);
        Dictionary<int, double> after = Shares(variantDir);
        List<int> pods = [.. before.Keys.Intersect(after.Keys).Order()];
        if (pods.Count < 2)
        {
            throw new ArgumentException($"{baseDir} and {variantDir} have {pods.Count} pod(s) in common: nothing to pair.");
        }

        (double Mean, double Margin) MeanMargin(IReadOnlyList<double> values)
        {
            double mean = values.Average();
            double spread = Math.Sqrt(values.Sum(value => (value - mean) * (value - mean)) / (values.Count - 1));
            return (mean, 1.96 * spread / Math.Sqrt(values.Count));
        }

        (double b, double bm) = MeanMargin([.. pods.Select(pod => before[pod])]);
        (double v, double vm) = MeanMargin([.. pods.Select(pod => after[pod])]);
        (double d, double dm) = MeanMargin([.. pods.Select(pod => after[pod] - before[pod])]);
        Console.WriteLine($"{pods.Count} pods. base {b:P1} +/- {bm:P1}; variant {v:P1} +/- {vm:P1}; variant - base {(d >= 0 ? "+" : string.Empty)}{d:P1} +/- {dm:P1} (paired by pod).");
        return 0;
    }

    private static IEnumerable<Dictionary<string, string>> Rows(string path)
    {
        string[] lines = File.ReadAllLines(path);
        string[] names = lines[0].Split(',');
        foreach (string line in lines.Skip(1).Where(line => line.Length > 0))
        {
            string[] cells = line.Split(',');
            yield return Enumerable.Range(0, Math.Min(names.Length, cells.Length)).ToDictionary(at => names[at], at => cells[at], StringComparer.Ordinal);
        }
    }

    private static double Likelihood(List<(double[] Difference, bool Won)> games, double[] weights)
    {
        double sum = 0;
        foreach ((double[] difference, bool won) in games)
        {
            double at = 0;
            for (int feature = 0; feature < weights.Length; feature++)
            {
                at += weights[feature] * difference[feature];
            }

            sum += (won ? at : 0) - Math.Log(1 + Math.Exp(at));
        }

        return sum;
    }

    /// <summary>The weights that make the games likeliest, by Newton's method, held to nothing very lightly so that a column that never differs has an answer.</summary>
    private static double[] Logistic(List<(double[] Difference, bool Won)> games)
    {
        const double Ridge = 1e-3;
        int n = games[0].Difference.Length;
        double[] weights = new double[n];
        for (int step = 0; step < 30; step++)
        {
            double[,] system = new double[n, n + 1];
            foreach ((double[] x, bool won) in games)
            {
                double at = 0;
                for (int feature = 0; feature < n; feature++)
                {
                    at += weights[feature] * x[feature];
                }

                double chance = 1 / (1 + Math.Exp(-at));
                double curve = chance * (1 - chance);
                for (int row = 0; row < n; row++)
                {
                    if (x[row] == 0)
                    {
                        continue;
                    }

                    system[row, n] += x[row] * ((won ? 1 : 0) - chance);
                    for (int column = 0; column < n; column++)
                    {
                        system[row, column] += curve * x[row] * x[column];
                    }
                }
            }

            for (int at = 0; at < n; at++)
            {
                system[at, at] += Ridge;
                system[at, n] -= Ridge * weights[at];
            }

            double[] move = Eliminate(system, n);
            double most = 0;
            for (int at = 0; at < n; at++)
            {
                weights[at] += move[at];
                most = Math.Max(most, Math.Abs(move[at]));
            }

            if (most < 1e-8)
            {
                break;
            }
        }

        return weights;
    }

    /// <summary>Solves <paramref name="n"/> equations in as many unknowns, the right-hand side in the last column, by elimination.</summary>
    private static double[] Eliminate(double[,] a, int n)
    {
        for (int pivot = 0; pivot < n; pivot++)
        {
            int best = pivot;
            for (int row = pivot + 1; row < n; row++)
            {
                if (Math.Abs(a[row, pivot]) > Math.Abs(a[best, pivot]))
                {
                    best = row;
                }
            }

            for (int column = 0; column <= n; column++)
            {
                (a[pivot, column], a[best, column]) = (a[best, column], a[pivot, column]);
            }

            for (int row = 0; row < n; row++)
            {
                if (row == pivot || a[row, pivot] == 0)
                {
                    continue;
                }

                double factor = a[row, pivot] / a[pivot, pivot];
                for (int column = pivot; column <= n; column++)
                {
                    a[row, column] -= factor * a[pivot, column];
                }
            }
        }

        return [.. Enumerable.Range(0, n).Select(at => a[at, n] / a[at, at])];
    }

    // --- Arguments -----------------------------------------------------------------------------------------------------

    private static string Text(IReadOnlyList<string> args, string name, string fallback)
    {
        for (int i = 0; i < args.Count - 1; i++)
        {
            if (string.Equals(args[i], name, StringComparison.Ordinal))
            {
                return args[i + 1];
            }
        }

        return fallback;
    }

    private static int Number(IReadOnlyList<string> args, string name, int fallback) =>
        int.TryParse(Text(args, name, string.Empty), NumberStyles.Integer, CultureInfo.InvariantCulture, out int value) ? value : fallback;

    private static double Real(IReadOnlyList<string> args, string name, double fallback) =>
        double.TryParse(Text(args, name, string.Empty), NumberStyles.Float, CultureInfo.InvariantCulture, out double value) ? value : fallback;
}
