using Cantrip.Client.Embedded;
using System.Collections.Concurrent;
using System.Diagnostics;
using Cantrip.Server;
using Cantrip.Server.Rules;

namespace Cantrip.Playtest;

/// <summary>
/// Plays many random games at the session layer and reports what happened, with the seed of anything that went
/// wrong.
/// <para>
/// The fast layer: no client, no socket, no process per game. Its point is volume — enough games that a rare rules
/// interaction turns up at all — and a seed for every failure so the one game that found something can be played
/// again on its own, or in the real client with the same seeds.
/// </para>
/// </summary>
public static class Program
{
    /// <summary>The format the decks are played in (<c>--format</c>, <see cref="HostAndPlay.FormatFrom"/>): Freeform unless named.</summary>
    public static Cantrip.Decks.DeckFormat Format { get; private set; } = Cantrip.Decks.DeckFormat.Freeform;

    /// <summary>The format the command line names, or null - for none, or for one there is not, said.</summary>
    private static Cantrip.Decks.DeckFormat? FormatOf(string[] args)
    {
        try
        {
            return HostAndPlay.FormatFrom(args);
        }
        catch (ArgumentException error)
        {
            Console.Error.WriteLine(error.Message);
            return null;
        }
    }

    public static int Main(string[] args)
    {
        ArgumentNullException.ThrowIfNull(args);
        if (args.Contains("--help") || args.Contains("-h"))
        {
            Help();
            return 0;
        }

        if (FormatOf(args) is not { } format)
        {
            return 2;
        }

        Format = format;

        string root = FindRoot();
        int games = Number(args, "--games", 100);
        int parallel = Number(args, "--parallel", Math.Clamp(Environment.ProcessorCount, 1, 32));
        int players = Math.Clamp(Number(args, "--players", 2), 2, 4);
        // A new pair each run unless one is given. Replaying the same hundred games every time says nothing the first
        // hundred did not, and volume is the whole reason this layer exists. Both are printed below, so a run that
        // turns something up is repeated by handing them back.
        ulong seed = (ulong)Number(args, "--seed", Random.Shared.Next(1, 1_000_000_000));
        int botSeed = Number(args, "--bot-seed", Random.Shared.Next(1, 1_000_000_000));
        bool verbose = args.Contains("--verbose");

        List<DeckList> decks;
        try
        {
            decks = Decks(root, args, players);
        }
        catch (Exception error) when (error is FileNotFoundException or DirectoryNotFoundException)
        {
            Console.Error.WriteLine(error.Message);
            return 2;
        }

        // A mirror match has nothing to swap: both seats hold the same list, so the seats are left alone and the
        // report leads with the splits that can still differ.
        bool sameDeck = decks.Select(deck => deck.Name).Distinct(StringComparer.Ordinal).Count() == 1;
        bool mirror = !sameDeck && !args.Contains("--no-mirror");

        Console.WriteLine(
            $"Playing {games} game(s) of {players}, {parallel} at a time, at the session layer (no client, no socket).");
        Console.WriteLine($"Decks: {string.Join(" vs ", decks.Select(deck => deck.Name))}");
        Console.WriteLine($"Rules: native (the cantrip-native-host sidecar at {NativeHostProcess.Locate()}, one for the whole run).");
        // Said plainly rather than as a command line, because the two wrappers spell the flags differently and this
        // one line is printed by both.
        Console.WriteLine(
            $"Seeds: shuffle from {seed}, draws from {botSeed}. Give both back to play this run again.");
        Console.WriteLine(
            sameDeck ? "A mirror match: both seats hold the same list, so the decks cannot be told apart."
            : mirror ? "Seats swapped every other game, so each deck sits in each seat for half of them."
            : "Seats fixed, so a tally counts the seat along with the deck.");
        Console.WriteLine();

        var run = new PlaytestRun(decks);
        var reports = new ConcurrentBag<GameReport>();
        var clock = Stopwatch.StartNew();
        int done = 0;

        Parallel.For(0, games, new ParallelOptions { MaxDegreeOfParallelism = parallel }, index =>
        {
            // One seed per game off the two given, so the whole run is repeatable and any one game of it is too.
            GameReport report = run.Play(seed + (ulong)index, botSeed + (index * 16), players, mirror && index % 2 == 1);
            reports.Add(report);

            if (verbose)
            {
                Console.WriteLine(Describe(report));
            }
            else if (Interlocked.Increment(ref done) % 100 == 0 && !Console.IsOutputRedirected)
            {
                // Only to a terminal: a redirected run is a log, and a log of one line rewritten fifty times is one
                // unreadable line.
                Console.Write($"\r{done} of {games}...    ");
            }
        });

        clock.Stop();

        if (!verbose && !Console.IsOutputRedirected)
        {
            Console.Write($"\r{new string(' ', 32)}\r");
        }

        return Report(reports, clock.Elapsed, games, sameDeck);
    }

    private static int Report(ConcurrentBag<GameReport> reports, TimeSpan elapsed, int games, bool sameDeck)
    {
        List<GameReport> finished = [.. reports.Where(report => report.Finished)];
        List<GameReport> failed = [.. reports.Where(report => !report.Finished)];

        Console.WriteLine(
            $"{finished.Count} of {games} finished in {elapsed.TotalSeconds:N1}s " +
            $"({games / Math.Max(elapsed.TotalSeconds, 0.001):N0} games/s, " +
            $"{elapsed.TotalMilliseconds / Math.Max(games, 1):N1} ms/game).");

        if (finished.Count > 0)
        {
            List<int> turns = [.. finished.Select(report => report.Turn).Order()];
            List<int> answers = [.. finished.Select(report => report.Answers).Order()];

            Console.WriteLine(
                $"Turns {turns[0]}-{turns[^1]} (median {turns[turns.Count / 2]}); " +
                $"answers {answers[0]}-{answers[^1]} (median {answers[answers.Count / 2]}).");

            Tally(finished, sameDeck);
        }

        if (failed.Count == 0)
        {
            Console.WriteLine("No game was left stuck and no answer was refused.");
            return 0;
        }

        Console.WriteLine();
        Console.WriteLine($"{failed.Count} game(s) did not finish:");

        // Grouped, because one fault usually shows up in many games and a page of the same line hides the second one.
        foreach (IGrouping<string, GameReport> group in failed.GroupBy(report => report.Failure).OrderByDescending(group => group.Count()))
        {
            Console.WriteLine($"  {group.Count()}x {group.Key}");
            Console.WriteLine($"       replay: cantrip-playtest {group.First().Replay}");
        }

        return 1;
    }

    /// <summary>
    /// Who won: overall per deck, and the same split by whether that deck took the first turn.
    /// <para>
    /// The play and draw columns are the point of running hundreds of games rather than ten. Who begins is drawn per
    /// game (CR 103.2) rather than fixed to a seat, so a deck's overall number already averages the two together and
    /// hides the thing a matchup usually turns on — a deck that only wins when it goes first is a different deck from
    /// one that wins either way, and the overall column reads the same for both.
    /// </para>
    /// <para>
    /// A mirror match has no decks to tell apart, so there it is only the one split, which is then a straight reading
    /// of what the first turn is worth.
    /// </para>
    /// </summary>
    private static void Tally(List<GameReport> finished, bool sameDeck)
    {
        List<GameReport> decided = [.. finished.Where(report => report.Winner > 0 && report.Starter > 0)];
        int drawn = finished.Count(report => report.Winner == 0);

        Console.WriteLine();

        if (sameDeck)
        {
            Console.WriteLine("Wins by who took the first turn (a mirror match, so there is nothing else to split by):");
            Row("on the play", decided.Count(report => report.WonOnThePlay), decided.Count);
            Row("on the draw", decided.Count(report => !report.WonOnThePlay), decided.Count);
        }
        else
        {
            // Widths in step with Cell, so the three columns sit under their headings.
            Console.WriteLine($"{"Wins by deck:",-32}{"overall",-25}{"on the play",-25}on the draw");

            foreach (string deck in finished.SelectMany(report => report.Decks).Distinct().Order(StringComparer.Ordinal))
            {
                // On the play means this deck began; on the draw means it was in the game and did not. Counted off the
                // games themselves rather than halved, because who begins is drawn and lands unevenly.
                List<GameReport> began = [.. decided.Where(report => report.StarterDeck == deck)];
                List<GameReport> followed = [.. decided.Where(report => report.StarterDeck != deck && report.Decks.Contains(deck))];

                Console.WriteLine(
                    $"  {deck,-30}"
                    + Cell(finished.Count(report => report.WinningDeck == deck), finished.Count(report => report.Decks.Contains(deck)))
                    + Cell(began.Count(report => report.WinningDeck == deck), began.Count)
                    + Cell(followed.Count(report => report.WinningDeck == deck), followed.Count));
            }
        }

        if (drawn > 0)
        {
            Console.WriteLine($"  {"(drawn)",-30} {drawn,6}");
        }
    }

    private static string Cell(int won, int of) =>
        $"{won,5} of {of,-5} ({(of == 0 ? 0 : 100.0 * won / of),5:N1}%)  ";

    private static void Row(string label, int won, int of) => Console.WriteLine(
        $"  {label,-30} {won,6} of {of,-6} ({(of == 0 ? 0 : 100.0 * won / of),5:N1}%)");

    private static string Describe(GameReport report) => report.Finished
        ? $"seed {report.Seed}/{report.BotSeed}: turn {report.Turn}, {report.Answers} answers, " +
          (report.Winner == 0 ? "a draw" : $"{report.WinningDeck} won from seat {report.Winner}")
        : $"seed {report.Seed}/{report.BotSeed}: FAILED — {report.Failure}";

    /// <summary>
    /// The decks the seats play: <c>--deck</c> once per seat, or when none is named the demo decks, which are made of
    /// cards the native engine has. Fewer decks than seats is dealt round again, which is how
    /// a deck is played against itself.
    /// </summary>
    private static List<DeckList> Decks(string root, IReadOnlyList<string> args, int players)
    {
        string decksRoot = Path.Combine(root, "decks");
        List<string> named = [];

        for (int i = 0; i < args.Count - 1; i++)
        {
            if (string.Equals(args[i], "--deck", StringComparison.Ordinal))
            {
                named.Add(args[i + 1]);
            }
        }

        if (named.Count == 0)
        {
            return [.. Directory.GetFiles(Path.Combine(decksRoot, "demo"), "*.txt")
                .Order(StringComparer.Ordinal)
                .Take(players)
                .Select(path => Load(decksRoot, path))];
        }

        return [.. named.Select(deck => Load(decksRoot, DeckList.Find(decksRoot, deck)))];
    }

    /// <summary>
    /// A decklist named the way <c>--deck</c> takes it back: relative to <c>decks\</c>, forward slashes, no extension,
    /// so <c>demo/3-green-red</c> rather than the file stem <c>3-green-red</c>. Which is what makes the replay line a
    /// failure prints something that can be pasted and run — a bare stem names no file, since a deck is looked up by
    /// path and not searched for.
    /// </summary>
    private static DeckList Load(string decksRoot, string path)
    {
        string full = Path.GetFullPath(path);
        string root = Path.GetFullPath(decksRoot) + Path.DirectorySeparatorChar;
        string name = full.StartsWith(root, StringComparison.OrdinalIgnoreCase)
            ? full[root.Length..].Replace(Path.DirectorySeparatorChar, '/')
            : full;

        if (name.EndsWith(".txt", StringComparison.OrdinalIgnoreCase))
        {
            name = name[..^4];
        }

        return DeckList.Parse(name, File.ReadAllText(full), full);
    }

    /// <summary>
    /// The repository root, found by walking up from the executable for the content and decks a table needs.
    /// Discovered rather than passed, because the same build is run from the tools directory, from the solution root
    /// and from a bin directory several levels down.
    /// </summary>
    private static string FindRoot()
    {
        DirectoryInfo? here = new(AppContext.BaseDirectory);

        while (here is not null)
        {
            if (Directory.Exists(Path.Combine(here.FullName, "content", "mtg"))
                && Directory.Exists(Path.Combine(here.FullName, "decks")))
            {
                return here.FullName;
            }

            here = here.Parent;
        }

        throw new DirectoryNotFoundException("No content\\mtg above this executable; run it from inside the repository.");
    }

    private static string? String(IReadOnlyList<string> args, string name)
    {
        for (int i = 0; i < args.Count - 1; i++)
        {
            if (string.Equals(args[i], name, StringComparison.Ordinal))
            {
                return args[i + 1];
            }
        }

        return null;
    }

    private static int Number(IReadOnlyList<string> args, string name, int fallback)
    {
        for (int i = 0; i < args.Count - 1; i++)
        {
            if (string.Equals(args[i], name, StringComparison.Ordinal) && int.TryParse(args[i + 1], out int value))
            {
                return value;
            }
        }

        return fallback;
    }

    private static void Help() => Console.WriteLine(
        """
        cantrip-playtest — plays random games at the session layer and reports what happened.

        The fast layer: the real session and the real protocol, in this process, no client and no socket. For volume —
        enough games that a rare interaction turns up — and every failure names the seeds that produced it.

          --games N        how many games to play (default 100)
          --parallel N     how many at once (default: one per core, at most 32)
          --players N      seats per game, 2 to 4 (default 2)
          --seed N         the first shuffle; game n is dealt from seed + n (default: a new one each run)
          --bot-seed N     the first seat's draws; each game and seat is offset from it (default: a new one each run)
          --deck NAME      a deck, once per seat (default: the demo decks, made of cards the native engine has).
          --format KEY     the format the decks are played in: a constructed format's key, or limited
                           with --min-deck N (default: freeform, no deck-construction restriction)
                           A path, or a name under decks\\
          --verbose        one line per game rather than a summary
          --help           this

        Exits 1 if any game was left stuck or had an answer refused, 0 otherwise.

        The other layers, when this one is not the question:
          tools\\play-game.cmd    a real server, a socket and the web client — the wire and the screen
          tools\\test.cmd         every test project, plus one smoke game in the web client
        """);
}
