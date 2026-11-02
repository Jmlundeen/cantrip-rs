using System.Security.Cryptography;
using System.Text;
using Cantrip.Datasets;
using Cantrip.Decks;
using Cantrip.Protocol;
using Cantrip.Server.Decks;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Server.InPerson;

/// <summary>
/// What starting an in-person table takes, wherever it is hosted - the standalone server's setup page
/// (<see cref="InPersonHost"/>) or the bot's site (<see cref="InPersonTables"/>): the form's limits, the grid draft it
/// makes, the relay and credentials for its devices, and the devices' links.
/// </summary>
public static class InPersonSetup
{
    /// <summary>The most grids a draft lays out; the cube's size may allow fewer (<see cref="DraftPod.CreateGrid"/>).</summary>
    public const int MostPacks = 60;

    /// <summary>The longest name a seat is given.</summary>
    public const int LongestName = 32;

    /// <summary>A layout's key in the event's host data, so hosting it again lays out the same devices.</summary>
    internal const string LayoutKey = "inperson.layout";

    internal const string ClocksKey = "inperson.clocks";

    /// <summary>The form's word for a grid draft of a cube, and for a constructed game of two offered decks.</summary>
    public const string GridMode = "grid";
    public const string ConstructedMode = "constructed";

    /// <summary>The format a constructed game is played in, whose decks are offered (<see cref="FormatDecks"/>).</summary>
    public static DeckFormat ConstructedFormat => DeckFormat.Find(DeckFormat.StandardKey)!;

    /// <summary>The decks a constructed game offers, by name and as each list calls itself, in the order a page lists them.</summary>
    public static List<(string Name, string Title)> DeckChoices(IEnumerable<DeckList> decks) =>
        [.. decks.Select(deck => (deck.Name, FormatDecks.Title(deck))).OrderBy(deck => deck.Item2, StringComparer.OrdinalIgnoreCase)];

    /// <summary>The cube lists in <paramref name="directories"/>, by name: the only cubes the form may name.</summary>
    public static List<string> Cubes(IEnumerable<string> directories) =>
        [.. directories.Where(Directory.Exists)
            .SelectMany(directory => Directory.EnumerateFiles(directory, "*.txt"))
            .Select(Path.GetFileNameWithoutExtension).OfType<string>()
            .Distinct(StringComparer.Ordinal).Order(StringComparer.OrdinalIgnoreCase)];

    /// <summary>
    /// Throws <see cref="ArgumentException"/> for a form outside its limits: two names, three or four devices, and either a
    /// cube of <paramref name="cubes"/> and a number of grids, or two decks of <paramref name="decks"/>.
    /// </summary>
    public static void Validate(SetupRequest request, IReadOnlyCollection<string> cubes, IReadOnlyCollection<string>? decks = null)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(cubes);
        if (request.Names is not { Count: 2 } || request.Names.Any(name => !ValidName(name)))
            throw new ArgumentException($"Each seat needs a name of 1-{LongestName} printable characters.");
        if (request.Layout is not (3 or 4)) throw new ArgumentException("The devices are three or four.");
        if (request.Mode is not (null or GridMode or ConstructedMode)) throw new ArgumentException("A table plays a grid draft or a constructed game.");
        if (request.Constructed)
        {
            if (request.Decks is not { Count: 2 } chosen || chosen.Any(deck => deck is null || !(decks ?? []).Contains(deck, StringComparer.Ordinal)))
                throw new ArgumentException("Each seat needs one of the decks offered.");
            return;
        }

        if (request.Packs is < 1 or > MostPacks) throw new ArgumentException($"A grid draft lays out 1-{MostPacks} grids.");
        if (request.Cube is null || !cubes.Contains(request.Cube, StringComparer.Ordinal)) throw new ArgumentException($"There is no cube '{request.Cube}'.");
    }

    /// <summary>
    /// Makes the grid draft a validated form asks for, its players given <paramref name="playerIds"/> (seat 1's, then seat
    /// 2's), and returns the event's id.
    /// </summary>
    internal static string CreateEvent(EventStore store, ContentLibrary library, IReadOnlyList<string> cubeDirectories, SetupRequest request,
        (string First, string Second) playerIds, TimeProvider time)
    {
        string cube = CubeList.Find(request.Cube, [.. cubeDirectories]);
        CreatedEvent created = DraftPod.CreateGrid(store, library, new GridDraftSetup
        {
            Name = $"In person: {request.Cube}",
            Cube = request.Cube,
            CubeCards = CubeList.Load(cube),
            PickOrder = CubeList.PickOrder(cube),
            Packs = request.Packs,
            Players = [(playerIds.First, request.Names[0].Trim()), (playerIds.Second, request.Names[1].Trim())],
        }, time);
        return created.Record.Id;
    }

    /// <summary>
    /// The relay of a two-player table and its devices' credentials: a ticket per seat, judged by the event, and a token per
    /// tablet - one shared tablet for three devices, one a seat for four.
    /// </summary>
    internal static Seating Seat(DraftPod pod, int devices, TimeProvider time, Action<string> log)
    {
        List<(string PlayerId, string Name, int Seat)> players = [.. pod.Players.OrderBy(player => player.Seat)];
        Dictionary<int, string> tickets = players.ToDictionary(player => player.Seat, player => pod.IssueTicket(player.PlayerId));
        (InPersonRelay relay, List<(int Side, string Token)> tablets) = Relay(players, pod.SeatFor,
            seat => pod.IssueTicket(players.Single(player => player.Seat == seat).PlayerId), devices, time, log);
        return new Seating(players, tickets, relay, tablets);
    }

    /// <summary>
    /// The relay of a constructed game and its tablets' tokens. Its seats' tickets are the table's own, made as each seat
    /// is reserved once the table is open (<see cref="ConstructedGame.Reserve"/>), so the seating is finished then
    /// (<see cref="ConstructedGame.Seating"/>).
    /// </summary>
    internal static (InPersonRelay Relay, List<(int Side, string Token)> Tablets) SeatConstructed(ConstructedGame game, int devices, TimeProvider time, Action<string> log) =>
        Relay([.. game.Players], game.SeatFor, game.TicketFor, devices, time, log);

    private static (InPersonRelay Relay, List<(int Side, string Token)> Tablets) Relay(IReadOnlyList<(string PlayerId, string Name, int Seat)> players,
        Func<string, int> seatFor, Func<int, string> issue, int devices, TimeProvider time, Action<string> log)
    {
        var relay = new InPersonRelay(new InPersonRelayOptions
        {
            PlayerNames = [.. players.Select(player => player.Name)],
            SeatForTicket = seatFor,
            IssueTicket = issue,
            TimeProvider = time,
            Log = log,
        });
        List<int> sides = devices == 4 ? [1, 2] : [0];
        return (relay, [.. sides.Select(side => (side, relay.MintTablet(side)))]);
    }

    /// <summary>
    /// What a constructed game's table is opened with: the format's decks offered by name, each seat held for its player and
    /// deck by a ticket (<see cref="GameServerOptions.RequireTickets"/>), and its game dealt through the relay, so the tablets
    /// see each step.
    /// </summary>
    internal static GameServerOptions ConstructedOptions(ConstructedGame game, InPersonRelay relay, Cantrip.Server.Rules.IRulesHostFactory rules, int port,
        string contentRoot, string listen, Uri art, bool clocks, TimeProvider time) => new()
    {
        Format = ConstructedFormat,
        Decks = [.. game.Decks],
        Port = port,
        Players = 2,
        ContentRoot = contentRoot,
        ListenAddress = listen,
        RequireTickets = true,
        Rules = new ObservedRulesFactory(rules, relay),
        Art = art,
        PlayerClock = clocks ? Cantrip.Server.MatchClocks.DefaultPlayerClock : InPersonHost.NoClock,
        TimeProvider = time,
    };

    /// <summary>
    /// Each device's name and link, phones then tablets, in the relay's order (<see cref="InPersonRelay.Devices"/>). A phone's
    /// link names the table (<c>?inperson=phone&amp;server=...#ticket=T</c>); a tablet's, which is sent no table message,
    /// where card art is (<c>?inperson=tablet&amp;side=N&amp;art=...#device=D</c>). The credential is after the <c>#</c>
    /// either way, so it is in no request and no log.
    /// </summary>
    internal static List<DeviceLink> Links(Seating seating, Uri site, Uri table, Uri art)
    {
        string game = Uri.EscapeDataString(table.AbsoluteUri);
        string tabletArt = Uri.EscapeDataString(art.AbsoluteUri);
        string page = site.GetLeftPart(UriPartial.Path);
        return
        [
            .. seating.Players.Select(player => new DeviceLink($"{player.Name}'s phone",
                $"{page}?inperson=phone&server={game}#ticket={seating.Tickets[player.Seat]}")),
            .. seating.Tablets.Select(tablet => new DeviceLink(tablet.Side == 0 ? "The shared tablet" : $"{seating.Players[tablet.Side - 1].Name}'s tablet",
                $"{page}?inperson=tablet&side={tablet.Side}&art={tabletArt}#device={tablet.Token}")),
        ];
    }

    private static bool ValidName(string? name) =>
        name is not null && name.Trim().Length is >= 1 and <= LongestName && !name.Any(char.IsControl);

    internal sealed record Seating(
        IReadOnlyList<(string PlayerId, string Name, int Seat)> Players,
        IReadOnlyDictionary<int, string> Tickets,
        InPersonRelay Relay,
        IReadOnlyList<(int Side, string Token)> Tablets);
}

/// <summary>
/// A constructed game at an in-person table: two players, each with a deck the setup chose from the format's, playing one
/// game at an ordinary table - no event, no draft. Each seat is held for its player and deck by a ticket of the table's
/// (<see cref="GameServer.Reserve"/>), which seats its player as soon as they sit; the game is dealt once both have. A
/// phone's re-entry code (the home-screen app's first launch) is traded for the same ticket, which takes the seat back
/// from the browser tab it was opened in once that tab's connection is gone.
/// </summary>
internal sealed class ConstructedGame
{
    private readonly object _gate = new();
    private readonly Dictionary<int, string> _tickets = [];

    public ConstructedGame(IReadOnlyList<string> names, IReadOnlyList<DeckList> decks)
    {
        Players = [.. names.Select((name, index) => (string.Empty, name.Trim(), index + 1))];
        Decks = decks;
    }

    public IReadOnlyList<(string PlayerId, string Name, int Seat)> Players { get; }

    /// <summary>Each seat's deck, seat 1's first.</summary>
    public IReadOnlyList<DeckList> Decks { get; }

    /// <summary>What the setup page calls it: never which decks, which the players see as they are played.</summary>
    public string Name => $"In person: {InPersonSetup.ConstructedFormat.Title}";

    /// <summary>Holds each seat at the open table for its player and deck.</summary>
    public void Reserve(GameServer server)
    {
        lock (_gate)
        {
            foreach ((_, string name, int seat) in Players)
            {
                _tickets[seat] = server.Reserve(seat, name, Decks[seat - 1].Name);
            }
        }
    }

    /// <summary>The seating once the seats are reserved: the players, their tickets, the relay and the tablets.</summary>
    public InPersonSetup.Seating Seating(InPersonRelay relay, IReadOnlyList<(int Side, string Token)> tablets)
    {
        lock (_gate) return new InPersonSetup.Seating(Players, new Dictionary<int, string>(_tickets), relay, tablets);
    }

    /// <summary>The seat a ticket holds, compared in constant time, or zero for none.</summary>
    public int SeatFor(string ticket)
    {
        byte[] presented = SHA256.HashData(Encoding.UTF8.GetBytes(ticket ?? string.Empty));
        int found = 0;
        lock (_gate)
        {
            foreach ((int seat, string held) in _tickets)
            {
                if (CryptographicOperations.FixedTimeEquals(presented, SHA256.HashData(Encoding.UTF8.GetBytes(held)))) found = seat;
            }
        }

        return found;
    }

    /// <summary>A seat's ticket again, for its phone's home-screen app: the same one, the table's only for that seat.</summary>
    public string TicketFor(int seat)
    {
        lock (_gate) return _tickets.TryGetValue(seat, out string? ticket) ? ticket : throw new InvalidOperationException($"Seat {seat} is not held.");
    }

    /// <summary>Where the game stands, for the tablets and the setup page: its table phase, a line, and a stage's name.</summary>
    public (TablePhase Phase, string Line, string Stage) Status(GameServer server)
    {
        if (server.Result is { } result)
        {
            string line = result.Phase == TablePhase.Abandoned ? "The game was abandoned."
                : result.WinningSeats.Count == 0 ? "The game was a draw."
                : $"{string.Join(" and ", result.WinningSeats.Select(seat => Players[seat - 1].Name))} won.";
            return (TablePhase.Finished, line, nameof(EventStage.Complete));
        }

        return server.Started.IsCompleted
            ? (TablePhase.Playing, "The game is being played.", nameof(EventStage.Playing))
            : (TablePhase.Setup, "Waiting for both phones to take their seats.", nameof(EventStage.Seating));
    }
}

/// <summary>A device's name and its link, which holds its credential; its QR code drawn once.</summary>
internal sealed record DeviceLink(string Name, string Url)
{
    public string Qr { get; } = InPersonHost.QrSvg(Url);
}
