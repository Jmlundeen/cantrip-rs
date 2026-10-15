using System.Diagnostics;
using System.Globalization;
using Cantrip.Client;
using Cantrip.Client.Embedded;
using Cantrip.Client.Llm;
using Cantrip.Protocol;
using Cantrip.Server;

namespace Cantrip.LlmGames;

/// <summary>
/// Plays whole games with a language model in one seat, and prints every decision it made.
/// <para>
/// The measure this is built for is a transcript, not a win rate. A game costs money and minutes, so a hundred of
/// them is not the first thing to reach for; one game read end to end says immediately whether the model is playing
/// Magic or flailing, and which of the two it is decides what to change next. The tally is printed anyway, because
/// beating <see cref="RandomPlay"/> is the bar: a seat that cannot do that reliably is not thinking, whatever its
/// reasons read like.
/// </para>
/// <para>
/// The same session and the same protocol as <c>cantrip-playtest</c>, one game at a time rather than one per core,
/// because a seat that waits on a model holds the loop while it waits.
/// </para>
/// </summary>
public static class Program
{
    public static int Main(string[] args)
    {
        ArgumentNullException.ThrowIfNull(args);

        if (args.Contains("--help") || args.Contains("-h"))
        {
            Help();
            return 0;
        }

        string root = FindRoot();
        int games = Number(args, "--games", 1);
        int players = Math.Clamp(Number(args, "--players", 2), 2, 4);
        ulong seed = (ulong)Number(args, "--seed", Random.Shared.Next(1, 1_000_000_000));
        int botSeed = Number(args, "--bot-seed", Random.Shared.Next(1, 1_000_000_000));
        bool both = args.Contains("--both");
        bool quiet = args.Contains("--quiet");
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

        OpenRouterModel model;
        try
        {
            model = new OpenRouterModel(
                Text(args, "--model", OpenRouterModel.DefaultModel),
                Text(args, "--key", null),
                Number(args, "--temperature-percent", 20) / 100.0);
        }
        catch (InvalidOperationException error)
        {
            Console.Error.WriteLine(error.Message);
            return 2;
        }

        using (model)
        {
            Console.WriteLine($"Model: {model.Model}");
            Console.WriteLine($"Decks: {string.Join(" vs ", decks.Select(deck => deck.Name))}");
            Console.WriteLine(both
                ? "Both seats are played by the model."
                : "Seat 1 is played by the model; the rest draw their choices at random.");
            Console.WriteLine($"Seeds: shuffle from {seed}, draws from {botSeed}.");
            Console.WriteLine();

            var run = new Table(Path.Combine(root, "content"), decks, model)
            {
                Both = both,
                Quiet = quiet,
                Verbose = verbose,
            };

            List<Result> results = [];
            var clock = Stopwatch.StartNew();

            for (int game = 0; game < games; game++)
            {
                if (games > 1)
                {
                    Console.WriteLine($"=== Game {game + 1} of {games} ===");
                }

                results.Add(run.Play(seed + (ulong)game, botSeed + (game * 16), players));
                Console.WriteLine();
            }

            clock.Stop();
            return Report(results, model, clock.Elapsed);
        }
    }

    private static int Report(List<Result> results, OpenRouterModel model, TimeSpan elapsed)
    {
        List<Result> finished = [.. results.Where(result => result.Failure.Length == 0)];

        Console.WriteLine($"{finished.Count} of {results.Count} game(s) finished in {elapsed.TotalMinutes:N1} minutes.");

        if (finished.Count > 0)
        {
            int won = finished.Count(result => result.Winner == Table.ModelSeat);
            int drawn = finished.Count(result => result.Winner == 0);

            Console.WriteLine(
                $"The model won {won} of {finished.Count}"
                + (drawn > 0 ? $", drew {drawn}" : string.Empty)
                + $", over {finished.Sum(result => result.Turns)} turn(s) in all.");

            Console.WriteLine(
                $"It was asked {finished.Sum(result => result.Questions)} question(s) across "
                + $"{finished.Sum(result => result.Answers)} prompt(s); "
                + $"{finished.Sum(result => result.Fallbacks)} fell back to the game's own answer.");
        }

        Console.WriteLine($"Usage: {model.Usage}.");

        if (model.Prices() is { } prices)
        {
            Console.WriteLine(string.Create(
                CultureInfo.InvariantCulture,
                $"Cost: about ${model.Usage.Cost(prices.In, prices.Out):N4} "
                + $"(${prices.In:N3} in / ${prices.Out:N3} out per million tokens)."));
        }

        foreach (Result failed in results.Where(result => result.Failure.Length > 0))
        {
            Console.WriteLine($"Game from seed {failed.Seed} did not finish: {failed.Failure}");
        }

        return results.TrueForAll(result => result.Failure.Length == 0) ? 0 : 1;
    }

    /// <summary>
    /// The decks the seats play: <c>--deck</c> once per seat, or the demo decks when none is named. Fewer decks than
    /// seats is dealt round again, which is how a deck is played against itself.
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

    private static string? Text(IReadOnlyList<string> args, string name, string? fallback)
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

    private static void Help() => Console.WriteLine(
        """
        cantrip-llm — plays games with a language model in one seat and prints every decision it made.

        The model is reached through OpenRouter. Set OPENROUTER_API_KEY in the environment, or pass --key.

          --games N        how many games to play (default 1)
          --players N      seats per game, 2 to 4 (default 2)
          --deck NAME      a deck, once per seat (default: the demo decks). A path, or a name under decks\
          --model ID       the OpenRouter model (default deepseek/deepseek-v4-pro-0813). Cheaper ones, which
                           play legally but weigh combat poorly: google/gemini-2.5-flash-lite,
                           qwen/qwen3.7-flash, deepseek/deepseek-v4-flash
          --key KEY        the OpenRouter key, if it is not in the environment
          --both           the model plays every seat, rather than only seat 1
          --seed N         the shuffle (default: a new one each run)
          --bot-seed N     the random seat's draws (default: a new one each run)
          --temperature-percent N   how much the model wanders, 0 to 100 (default 20)
          --quiet          print only what was chosen, not the choices it was offered
          --verbose        print the whole position sent to the model before each decision
          --help           this

        Exits 1 if any game was left stuck, 0 otherwise.

        Only one game runs at a time, because a seat waiting on a model holds the loop. For volume against the
        random seat instead, tools\bots-random-games.cmd plays thousands a minute and costs nothing.
        """);
}
