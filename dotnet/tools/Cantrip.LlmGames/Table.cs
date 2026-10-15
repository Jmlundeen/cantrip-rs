using System.Globalization;
using Cantrip.Client;
using Cantrip.Client.Embedded;
using Cantrip.Client.Llm;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Rules;

namespace Cantrip.LlmGames;

/// <summary>What one game did.</summary>
public sealed record Result
{
    public required ulong Seed { get; init; }

    public int Turns { get; init; }

    /// <summary>The seat that won, zero for a draw, or -1 for a game that did not finish.</summary>
    public int Winner { get; init; } = -1;

    /// <summary>How many questions the model was put, how many prompts it answered, and how many it could not.</summary>
    public int Questions { get; init; }

    public int Answers { get; init; }

    public int Fallbacks { get; init; }

    public string Failure { get; init; } = string.Empty;
}

/// <summary>
/// One table: a real <see cref="GameSession"/> over a loopback transport, with a model in one seat and, unless
/// <see cref="Both"/>, <see cref="RandomPlay"/> in the others.
/// <para>
/// The random seat is the control. It is a known quantity that makes no decisions at all, so a model that does not
/// beat it is not making decisions either, whatever the reasons it gives read like.
/// </para>
/// </summary>
public sealed class Table
{
    /// <summary>The seat the model always takes, so a transcript is always about the same player.</summary>
    public const int ModelSeat = 1;

    private readonly string _contentRoot;
    private readonly IReadOnlyList<DeckList> _decks;
    private readonly ILanguageModel _model;

    public Table(string contentRoot, IReadOnlyList<DeckList> decks, ILanguageModel model)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(contentRoot);
        ArgumentNullException.ThrowIfNull(decks);
        ArgumentNullException.ThrowIfNull(model);

        _contentRoot = contentRoot;
        _decks = decks;
        _model = model;
    }

    /// <summary>Whether the model plays every seat, rather than only <see cref="ModelSeat"/>.</summary>
    public bool Both { get; init; }

    /// <summary>Print only what was chosen, not the choices offered.</summary>
    public bool Quiet { get; init; }

    /// <summary>Print the whole position as the model was shown it.</summary>
    public bool Verbose { get; init; }

    public Result Play(ulong seed, int botSeed, int players)
    {
        List<string> names = [.. Enumerable.Range(1, players).Select(seat => $"Bot {seat}")];

        using HostedGame hosted = HostAndPlay.StartTable(RulesHostFactory.Native, _decks, seed, names);

        List<(GameClient Client, IAutoPlayer Bot)> seats = [];
        List<LlmPlay> thinking = [];

        foreach (int seat in hosted.Seats.Order())
        {
            var client = new GameClient(hosted.TransportFor(seat));
            client.Join(names[seat - 1]);

            IAutoPlayer bot;

            if (Both || seat == ModelSeat)
            {
                var llm = new LlmPlay(_model, Deck(_decks[(seat - 1) % _decks.Count]))
                {
                    Watching = (question, answer) => Decided(seat, question, answer),
                    Sending = Verbose ? text => Console.WriteLine($"\n--- shown to seat {seat} ---\n{text}\n") : null,
                };

                thinking.Add(llm);
                bot = llm;
            }
            else
            {
                bot = new RandomPlay(botSeed + seat);
            }

            seats.Add((client, bot));
        }

        hosted.Start();

        int answers = 0;
        int refusals = 0;
        int idle = 0;
        string step = string.Empty;

        while (true)
        {
            foreach ((GameClient client, IAutoPlayer _) in seats)
            {
                client.Pump();
            }

            if (seats.TrueForAll(seat => seat.Client.IsOver))
            {
                break;
            }

            // Whose turn it is and where in it, printed off the model's own view so the transcript reads as a game
            // rather than as a list of questions.
            if (seats[0].Client.View is { } view
                && string.Create(CultureInfo.InvariantCulture, $"Turn {view.Turn}, {GameText.Step(view.Step)}, {Name(view, view.ActivePlayer)}") is var now
                && now != step)
            {
                step = now;
                Console.WriteLine($"\n[{now}]");
            }

            bool moved = false;

            foreach ((GameClient client, IAutoPlayer bot) in seats)
            {
                if (!bot.Act(client))
                {
                    continue;
                }

                moved = true;

                if (++answers > AnswerLimit)
                {
                    return Stuck(seed, seats, thinking, $"no end after {AnswerLimit} answers");
                }

                if (client.Errors.Count > refusals)
                {
                    refusals = client.Errors.Count;

                    if (!bot.Retrying)
                    {
                        ErrorMessage refused = client.Errors[^1];
                        return Stuck(
                            seed,
                            seats,
                            thinking,
                            $"seat {client.Seat} could not answer {client.Prompt?.Prompt?.GetType().Name}: "
                            + $"{refused.Code}: {refused.Detail.ReplaceLineEndings(" ")}");
                    }
                }
            }

            idle = moved ? 0 : idle + 1;

            if (idle > IdlePasses)
            {
                return Stuck(seed, seats, thinking, "every seat idle, game unfinished");
            }
        }

        GameClient first = seats[0].Client;
        GameOverMessage outcome = first.Outcome!;

        int won = outcome.Outcome == GameOutcomeView.Draw
            ? 0
            : first.View?.Players.FirstOrDefault(player => player.Team == outcome.WinningTeam)?.Seat ?? 0;

        Console.WriteLine(won == 0
            ? "\nThe game was a draw."
            : $"\n{Name(first.View, won)} won on turn {first.View?.Turn}"
                + (won == ModelSeat ? " — the model." : " — the random seat."));

        return new Result
        {
            Seed = seed,
            Turns = first.View?.Turn ?? 0,
            Winner = won,
            Questions = thinking.Sum(seat => seat.Questions),
            Answers = thinking.Sum(seat => seat.Answers),
            Fallbacks = thinking.Sum(seat => seat.Fallbacks),
        };
    }

    private void Decided(int seat, string question, string answer)
    {
        if (!Quiet)
        {
            Console.WriteLine();
            Console.WriteLine(Indent(question));
        }

        Console.WriteLine($"  seat {seat} -> {answer}");
    }

    private static string Indent(string text) =>
        string.Join("\n", text.ReplaceLineEndings("\n").Split('\n').Select(line => "  " + line));

    private static string Name(PlayerView? view, int seat) =>
        view?.Players.FirstOrDefault(player => player.Seat == seat)?.Name ?? $"seat {seat}";

    /// <summary>
    /// A decklist as names and counts, which is what the model is shown once at the start.
    /// <para>
    /// Names and no rules text: a general model knows what a Lightning Bolt does, and every card it can actually see
    /// arrives with its oracle text on it. What the list adds is the one thing no projection of the position can —
    /// what is still left in the library.
    /// </para>
    /// </summary>
    private static string Deck(DeckList deck) => string.Join(
        "\n",
        deck.Entries.Select(entry => string.Create(CultureInfo.InvariantCulture, $"{entry.Copies} {entry.Card}")));

    private Result Stuck(
        ulong seed,
        List<(GameClient Client, IAutoPlayer Bot)> seats,
        List<LlmPlay> thinking,
        string why)
    {
        string asked = string.Join(", ", seats.Select(seat =>
            $"seat {seat.Client.Seat}: {seat.Client.Prompt?.Prompt?.GetType().Name ?? "idle"}"));

        return new Result
        {
            Seed = seed,
            Turns = seats[0].Client.View?.Turn ?? 0,
            Questions = thinking.Sum(seat => seat.Questions),
            Answers = thinking.Sum(seat => seat.Answers),
            Fallbacks = thinking.Sum(seat => seat.Fallbacks),
            Failure = $"{why} (turn {seats[0].Client.View?.Turn}, {asked})",
        };
    }

    /// <summary>Answers one game may take before it is called stuck. Far more than an honest game needs.</summary>
    private const int AnswerLimit = 20_000;

    /// <summary>Passes with nobody asked anything before a game is called stuck rather than merely quiet.</summary>
    private const int IdlePasses = 4;
}
