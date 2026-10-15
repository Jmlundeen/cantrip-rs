using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text.RegularExpressions;
using Cantrip.Client;
using Cantrip.Server;

namespace Cantrip.BotLab;

/// <summary>
/// Plays thousands of games between bots and says only which did not finish.
/// <para>
/// This is fuzzing, and is meant as that: the shuffles are random on purpose, nobody is trying to reach a position,
/// and nothing here is a test of anything in particular. Two seats that play every card they are dealt, over every
/// card there is, walk the engine through more interactions in an hour than anyone will write scenarios for, and a
/// game that throws, stalls or never ends is a bug in the engine, the session or a seat's own loop guards. Each one
/// is printed with the command that plays it again exactly, which is where a scenario for it starts.
/// </para>
/// <para>
/// It says nothing of who won. That is <c>duel</c>.
/// </para>
/// </summary>
public static partial class Soak
{
    [GeneratedRegex(@"\d+")]
    private static partial Regex Digits();

    public static int Run(string root, string[] args, int parallelism)
    {
        ArgumentNullException.ThrowIfNull(args);

        string a = Program.Text(args, "--a", "tactical");
        string b = Program.Text(args, "--b", "tactical");
        int games = Program.Number(args, "--games", 2000);
        ulong seed = (ulong)Program.Number(args, "--seed", Random.Shared.Next(1, 1_000_000_000));

        // The every-card deck unless decks are named, and a pool beside it when one is given.
        List<string> named = [];
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (args[i] == "--deck")
            {
                named.Add(args[i + 1]);
            }
        }

        if (named.Count == 0)
        {
            named.Add("demo/card-test");
        }

        string pool = Program.Text(args, "--decks", string.Empty);
        string match = Program.Text(args, "--match", string.Empty);

        List<(string How, List<DeckList> Decks)> tables =
        [
            (string.Join(" ", named.Take(2).Select(deck => $"--deck \"{deck}\"")),
                Program.Decks(root, [.. named.Take(2).SelectMany(deck => new[] { "--deck", deck })])),
        ];

        if (pool.Length > 0)
        {
            tables.Add(($"--decks \"{pool}\"" + (match.Length > 0 ? $" --match \"{match}\"" : string.Empty),
                Program.Decks(root, ["--decks", pool, "--match", match])));
        }

        Func<int, int, IAutoPlayer> seatA = Program.Maker(root, a);
        Func<int, int, IAutoPlayer> seatB = Program.Maker(root, b);
        string content = Path.Combine(root, "content");
        int each = Math.Max(games / tables.Count, 1);

        var clock = Stopwatch.StartNew();
        ConcurrentBag<(string Why, string Replay)> unfinished = [];
        int played = 0;

        foreach ((string how, List<DeckList> decks) in tables)
        {
            Parallel.For(0, each, new ParallelOptions { MaxDegreeOfParallelism = parallelism }, game =>
            {
                (List<DeckList> seated, List<IAutoPlayer> players) = Program.Seating(decks, game, seatA, seatB);
                MatchResult result = Match.Play(content, seated, seed + (ulong)(game / 2), players, sample: false);
                Interlocked.Increment(ref played);

                if (!result.Finished)
                {
                    unfinished.Add((
                        result.Failure,
                        $"tools\\bot-duel.cmd watch --game {game} --seed {seed} --a {a} --b {b} {how}" +
                        (decks.Count > 2 ? $"   ({seated[0].Name})" : string.Empty)));
                }
            });
        }

        if (unfinished.IsEmpty)
        {
            Console.WriteLine($"{played} games, {a} against {b}, shuffles from {seed}: every one finished ({clock.Elapsed.TotalSeconds:N0}s).");
            return 0;
        }

        // The same failure on another shuffle, another turn or another card's handle is the same failure.
        foreach (IGrouping<string, (string Why, string Replay)> group in unfinished
            .GroupBy(failure => Digits().Replace(failure.Why, "#"))
            .OrderByDescending(group => group.Count()))
        {
            Console.WriteLine($"{group.Count()}x {group.First().Why}");
            foreach ((string _, string replay) in group.OrderBy(failure => failure.Replay, StringComparer.Ordinal).Take(6))
            {
                Console.WriteLine($"    {replay}");
            }

            if (group.Count() > 6)
            {
                Console.WriteLine($"    and {group.Count() - 6} more");
            }

            Console.WriteLine();
        }

        Console.WriteLine($"{played} games, {a} against {b}, shuffles from {seed}: {unfinished.Count} did not finish ({clock.Elapsed.TotalSeconds:N0}s).");
        return 1;
    }
}
