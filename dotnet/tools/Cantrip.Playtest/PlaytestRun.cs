using Cantrip.Client;
using Cantrip.Client.Embedded;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Rules;

namespace Cantrip.Playtest;

/// <summary>What one game did, or what went wrong with it.</summary>
public sealed record GameReport
{
    /// <summary>The shuffle this game was dealt from, so a game worth looking at again can be.</summary>
    public required ulong Seed { get; init; }

    /// <summary>The seed the seats drew their choices from.</summary>
    public required int BotSeed { get; init; }

    public int Turn { get; init; }

    public int Answers { get; init; }

    /// <summary>The seat that won, zero for a draw, or -1 when the game did not finish.</summary>
    public int Winner { get; init; } = -1;

    /// <summary>What each seat played, seat 1 first. Named rather than numbered, because the seats swap between games.</summary>
    public IReadOnlyList<string> Decks { get; init; } = [];

    /// <summary>The deck that won, empty for a draw or a game that did not finish.</summary>
    public string WinningDeck { get; init; } = string.Empty;

    /// <summary>
    /// The seat that took the first turn, or zero if the game never got that far.
    /// <para>
    /// Who begins is drawn per game rather than fixed to a seat (CR 103.2), so it is read off the table rather than
    /// assumed: without it there is no way to tell a deck that wins from a deck that wins because it began.
    /// </para>
    /// </summary>
    public int Starter { get; init; }

    /// <summary>Whether the winner was the one who took the first turn. False for a draw or an unfinished game.</summary>
    public bool WonOnThePlay => Winner > 0 && Winner == Starter;

    /// <summary>The deck that took the first turn, or empty where the game never got that far.</summary>
    public string StarterDeck => Starter > 0 && Starter <= Decks.Count ? Decks[Starter - 1] : string.Empty;

    /// <summary>What went wrong, or empty for a game that finished.</summary>
    public string Failure { get; init; } = string.Empty;

    public bool Finished => Failure.Length == 0;

    /// <summary>
    /// How to run this one game again, printed for a failure.
    /// <para>
    /// The decks are named seat by seat, in this game's seating rather than the run's, because the seats swap every
    /// other game: a line that named them the way the run did would deal a different game than the one that failed.
    /// A single game is never swapped, so seat order on the command line is seat order in the replay.
    /// </para>
    /// </summary>
    public string Replay =>
        $"--games 1 --seed {Seed} --bot-seed {BotSeed}"
        + string.Concat(Decks.Select(deck => $" --deck {Argument(deck)}"))
        + (Decks.Count is 0 or 2 ? string.Empty : $" --players {Decks.Count}");

    /// <summary>A deck name as one command-line argument, quoted where it holds a space ("multiblock test").</summary>
    private static string Argument(string deck) => deck.Contains(' ', StringComparison.Ordinal) ? $"\"{deck}\"" : deck;
}

/// <summary>
/// Plays whole games at the session layer: the real <see cref="GameSession"/>, the real protocol, a real
/// <see cref="GameClient"/> per seat, and <see cref="RandomPlay"/> answering — all inside this process over a
/// loopback transport.
/// <para>
/// The layer is the point. There is no client here and no socket: what a game costs is what the engine and the
/// session cost, so thousands of games are minutes rather than hours, and a run that finds something can say which
/// seed found it. The same bot over the same protocol also plays a seat of a real server (dotnet\tools\Cantrip.Bot), so a position this turns up is
/// reachable there by its seed.
/// </para>
/// <para>
/// What it therefore does not cover: the wire (a real server process and a WebSocket — that is
/// <c>tools\scripts\play-game.ps1</c>) and the web client (drawing, docks, input — that is Cantrip.Client.Web.Tests and
/// <c>test-game</c>). A clean bulk run here says the rules and the session hold up, not that anything is on screen.
/// </para>
/// </summary>
public sealed class PlaytestRun
{
    /// <summary>
    /// Answers one game may take before it is called stuck.
    /// <para>
    /// Generous: a long random game is a few thousand, and the number is here to turn a hang into a report naming a
    /// seed rather than to bound honest play.
    /// </para>
    /// </summary>
    private const int AnswerLimit = 200_000;

    /// <summary>Passes with nobody asked anything before a game is called stuck rather than merely quiet.</summary>
    private const int IdlePasses = 4;

    /// <summary>Frames of a throw a report carries.</summary>
    private const int StackFrames = 4;

    private readonly IReadOnlyList<DeckList> _decks;

    public PlaytestRun(IReadOnlyList<DeckList> decks)
    {
        ArgumentNullException.ThrowIfNull(decks);

        if (decks.Count == 0)
        {
            throw new ArgumentException("A table needs at least one deck.", nameof(decks));
        }

        _decks = decks;
    }

    /// <summary>
    /// Plays one game and says what happened. Never throws for a game that went wrong: a bulk run reports the bad
    /// game alongside the good ones and keeps going, because the interesting output is which seeds failed.
    /// </summary>
    /// <param name="swapped">
    /// Whether the decks take each other's seats. Half a run each way is what makes a tally a comparison of the decks
    /// rather than of the seats: whoever plays first draws fewer cards, and any advantage in that would otherwise be
    /// counted as one deck beating the other.
    /// </param>
    public GameReport Play(ulong seed, int botSeed, int players, bool swapped = false)
    {
        List<DeckList> order = swapped ? [.. _decks.Reverse()] : [.. _decks];
        List<string> seatDecks = [.. Enumerable.Range(0, players).Select(seat => order[seat % order.Count].Name)];

        try
        {
            return Game(seed, botSeed, players, order, seatDecks);
        }
        catch (Exception error)
        {
            // The seating is on the failure too, not only on a game that finished: it is what the replay line needs to
            // deal this game again, and a game that threw is the one anybody wants to deal again.
            return new GameReport
            {
                Seed = seed,
                BotSeed = botSeed,
                Decks = seatDecks,
                Failure = $"{error.GetType().Name}: {error.Message.ReplaceLineEndings(" ")} at {Where(error)}",
            };
        }
    }

    private GameReport Game(ulong seed, int botSeed, int players, List<DeckList> order, List<string> seatDecks)
    {
        List<string> names = [.. Enumerable.Range(1, players).Select(seat => $"Bot {seat}")];

        using HostedGame hosted = HostAndPlay.StartTable(RulesHostFactory.Native, order, seed, names);

        List<(GameClient Client, LoopGuard Bot)> seats = [];
        foreach (int seat in hosted.Seats)
        {
            // A seed per seat, so two bots offered the same shape of choice do not make the same choice as each other.
            var client = new GameClient(hosted.TransportFor(seat));
            client.Join($"Bot {seat}");
            seats.Add((client, new LoopGuard(new RandomPlay(botSeed + seat))));
        }

        hosted.Start();

        int answers = 0;
        int refusals = 0;
        int idle = 0;
        int starter = 0;

        while (true)
        {
            // Every seat drains its mailbox before anyone is asked whether it is over: an answer runs the session
            // inside the send, so the message that ends the game lands in one seat's mailbox while another has not
            // looked yet, and a seat that has not looked has neither a prompt nor an outcome.
            foreach ((GameClient client, LoopGuard _) in seats)
            {
                client.Pump();

                // Whoever is active on the first turn is whoever began it. Read while the turn is still 1, because
                // from the second turn on this is simply whose turn it is.
                if (starter == 0 && client.View is { Turn: 1 } opening)
                {
                    starter = opening.ActivePlayer;
                }
            }

            if (seats.TrueForAll(seat => seat.Client.IsOver))
            {
                break;
            }

            bool moved = false;

            foreach ((GameClient client, LoopGuard bot) in seats)
            {
                // The client lets go of its prompt as it answers, so what was asked is read before the answer.
                string? asked = client.Prompt?.Prompt?.GetType().Name;

                if (!bot.Act(client))
                {
                    continue;
                }

                moved = true;

                if (++answers > AnswerLimit)
                {
                    return Stuck(seed, botSeed, seats, answers, $"no end after {AnswerLimit} answers", seatDecks);
                }

                // A refused answer is not fatal on its own: not every rule a declaration is judged by is in the
                // prompt, so the bot draws again. One it has given up on is a seat with nothing legal left to say.
                if (client.Errors.Count > refusals)
                {
                    refusals = client.Errors.Count;

                    if (!bot.Retrying)
                    {
                        ErrorMessage refused = client.Errors[^1];
                        return Stuck(
                            seed,
                            botSeed,
                            seats,
                            answers,
                            $"seat {client.Seat} could not answer {asked ?? "(no prompt)"}: " +
                            $"{refused.Code}: {refused.Detail.ReplaceLineEndings(" ")}",
                            seatDecks);
                    }
                }
            }

            // Nobody asked anything of anybody and the game is not over. One pass of that is ordinary — a seat can
            // be waiting on a message another seat's answer will produce — so it takes a few in a row to mean the
            // game is waiting on something that will never arrive.
            idle = moved ? 0 : idle + 1;

            if (idle > IdlePasses)
            {
                return Stuck(seed, botSeed, seats, answers, $"every seat idle for {IdlePasses} passes, game unfinished", seatDecks);
            }
        }

        GameClient first = seats[0].Client;
        GameOverMessage outcome = first.Outcome!;
        bool drawn = outcome.Outcome == GameOutcomeView.Draw;

        // A team rather than a seat is what wins (CR 104.2a), so the seat is looked up rather than assumed equal to it:
        // a two-player table happens to number them the same and a four-player one in two teams does not.
        int won = drawn
            ? 0
            : first.View?.Players.FirstOrDefault(player => player.Team == outcome.WinningTeam)?.Seat ?? 0;

        return new GameReport
        {
            Seed = seed,
            BotSeed = botSeed,
            Turn = first.View?.Turn ?? 0,
            Answers = answers,
            Winner = won,
            Decks = seatDecks,
            WinningDeck = won > 0 && won <= seatDecks.Count ? seatDecks[won - 1] : string.Empty,
            Starter = starter,
        };
    }

    /// <summary>
    /// The first few frames of a throw. Enough to act on — a bare message names no code — but not the whole stack,
    /// which for a fault that hits many games at once would bury the second fault under the first.
    /// </summary>
    private static string Where(Exception error) => error.StackTrace is { } stack
        ? string.Join(
            " <- ",
            stack.Split('\n')
                .Take(StackFrames)
                .Select(frame => frame.Trim().Replace("at ", string.Empty, StringComparison.Ordinal)))
        : "an unknown frame";

    private GameReport Stuck(
        ulong seed,
        int botSeed,
        List<(GameClient Client, LoopGuard Bot)> seats,
        int answers,
        string why,
        IReadOnlyList<string> decks)
    {
        string asked = string.Join(", ", seats.Select(seat =>
            $"seat {seat.Client.Seat}: {seat.Client.Prompt?.Prompt?.GetType().Name ?? "idle"}"));

        return new GameReport
        {
            Seed = seed,
            BotSeed = botSeed,
            Turn = seats[0].Client.View?.Turn ?? 0,
            Answers = answers,
            Decks = decks,
            Failure = $"{why} (turn {seats[0].Client.View?.Turn}, {asked})",
        };
    }
}
