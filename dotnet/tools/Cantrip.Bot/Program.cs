using System.Globalization;
using Cantrip.Client;
using Cantrip.Client.Llm;
using Cantrip.Client.Tactics;
using Cantrip.Protocol;

// Plays one seat of a game on a server, with no window: joins over the WebSocket as a person's client would, answers
// every question it is asked, and exits when the game is over. What the local test games put in the seats nobody is
// sitting in (tools\play-game.cmd, tools\play-release-build.cmd, tools\play-vs-llm.cmd).
//
// Usage: cantrip-bot --connect ws://host:port/game [--seat N] [--name NAME] [--ticket T]
//                    [--random [--bot-seed N]] [--tactical [--narrate PATH|none]]
//                    [--llm [--model ID] [--deck NAME] [--transcript PATH]]
//
// Four ways to play, and the default is the plain one on purpose: the one deterministic way (AutoPlayer), so a smoke
// game is the same game every time. --random draws among what it is offered, from --bot-seed when one is given so a
// game can be played again. --tactical weighs what each choice would do to the position (TacticalPlay): the one meant
// to be played against, and it writes each decision it made and what it weighed to --narrate (a file in the temp
// folder unless told otherwise; none for none), so a game lost to a person can be read afterwards - and in an event,
// each pick it drafted and the deck it built, first, in the same file. --llm asks a language model over OpenRouter (OPENROUTER_API_KEY), thinking in the background,
// and writes every question and answer to --transcript.
//
// Prints the outcome and exits 0 when the game ends; exits 1 when the connection is lost or refused first.
//
// --event plays a seat of an event across every table it is sent to - a booster draft's pod, then the table of each of
// its games - and exits 0 once the event's final standings are shown.

if (args.Contains("--help") || StringArg("--connect") is not { } address || !Uri.TryCreate(address, UriKind.Absolute, out Uri? server))
{
    Console.Error.WriteLine(
        "Usage: cantrip-bot --connect ws://host:port/game [--seat N] [--name NAME] [--ticket T]\n" +
        "                   [--random [--bot-seed N]] [--tactical [--narrate PATH|none]]\n" +
        "                   [--llm [--model ID] [--deck NAME] [--transcript PATH]]");
    return 2;
}

int seat = IntArg("--seat") ?? 0;
string name = StringArg("--name") ?? (seat > 0 ? $"Bot {seat}" : "Bot");

// Where the tactical seat tells its decisions: one file for the whole run, every table of an event in turn. Several
// bots started at once each get a file of their own (the process id is in the default name), and a file that cannot
// be opened costs the record, never the seat: it says so and plays on in silence.
StreamWriter? narration = null;
bool narrationTried = false;
StreamWriter? Narration()
{
    if (narrationTried)
    {
        return narration;
    }

    narrationTried = true;
    string? named = StringArg("--narrate") is { } value && !value.StartsWith("--", StringComparison.Ordinal) ? value : null;
    if (named is "none")
    {
        return null;
    }

    string safe = string.Concat(name.Select(c => char.IsLetterOrDigit(c) ? c : '-'));
    string path = named
        ?? Path.Combine(Path.GetTempPath(), $"cantrip-tactical-{safe}-seat{seat}-{Environment.ProcessId}-{DateTime.Now:yyyyMMdd-HHmmss}.log");
    try
    {
        string full = Path.GetFullPath(path);
        if (Path.GetDirectoryName(full) is { Length: > 0 } folder)
        {
            Directory.CreateDirectory(folder);
        }

        narration = new StreamWriter(new FileStream(full, FileMode.Append, FileAccess.Write, FileShare.ReadWrite)) { AutoFlush = true };
        Console.WriteLine($"{name} tells each decision it makes, and why, in {full}");
    }
    catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException or NotSupportedException)
    {
        Console.Error.WriteLine($"{name} cannot write its decisions to {path} ({error.Message}); it plays without.");
    }

    return narration;
}

IAutoPlayer? player;
Transcript? transcript = null;
if (args.Contains("--llm"))
{
    transcript = new Transcript(StringArg("--transcript")
        ?? Path.Combine(Path.GetTempPath(), $"cantrip-llm-seat{seat}-{DateTime.Now:yyyyMMdd-HHmmss}.log"));
    Console.WriteLine($"{name} is played by a language model. Its questions and answers: {transcript.Where}");
    player = new LlmPlay(new OpenRouterModel(StringArg("--model")), StringArg("--deck") ?? string.Empty, background: true)
    {
        Watching = transcript.Decided,
    };
}
else
{
    player = Unattended();
}

// A seat with nobody and no model at it, made afresh for every table an event takes it to.
IAutoPlayer? Unattended() =>
    args.Contains("--tactical") ? new TacticalPlay { Narrator = Narration() }
    : args.Contains("--random") ? new RandomPlay(IntArg("--bot-seed"))
    : null;

if (args.Contains("--event"))
{
    if (StringArg("--ticket") is not { } eventTicket)
    {
        Console.Error.WriteLine("--event needs the seat's --ticket.");
        return 2;
    }

    EventSeat seat_;
    try
    {
        seat_ = await EventSeat.JoinAsync(server, eventTicket, name, Unattended);
    }
    catch (Exception error) when (error is System.Net.WebSockets.WebSocketException or HttpRequestException or IOException)
    {
        Console.Error.WriteLine($"{name} could not reach {server}: {error.Message}");
        return 1;
    }

    using (seat_)
    {
        while (!seat_.Finished)
        {
            try
            {
                if (!await seat_.StepAsync())
                {
                    await Task.Delay(15);
                }
            }
            catch (Exception error) when (error is System.Net.WebSockets.WebSocketException or HttpRequestException or IOException)
            {
                Console.Error.WriteLine($"{name} could not reach the next table: {error.Message}");
                return 1;
            }

            if (seat_.Client.Errors.LastOrDefault() is { Code: ErrorCode.MembershipRefused or ErrorCode.ProtocolMismatch } refused)
            {
                Console.Error.WriteLine($"{name} was refused: {refused.Detail}");
                return 1;
            }

            if (!seat_.Client.IsConnected)
            {
                // A table lets a seat go with a move to the next one, or the final standings, as its last message;
                // closed with neither, the server is gone, and a bot left waiting on it would never end.
                seat_.Client.Pump();
                if (seat_.Client.Moved is null && !seat_.Finished)
                {
                    Console.Error.WriteLine($"{name} lost the connection to {seat_.Address}.");
                    return 1;
                }
            }
        }

        Console.WriteLine($"{name}: the event is over, after {seat_.GamesPlayed} game(s).");
        return 0;
    }
}

using (transcript)
{
    GameClient client;
    try
    {
        client = new GameClient(await WebSocketTransport.ConnectAsync(server));
    }
    catch (Exception error) when (error is System.Net.WebSockets.WebSocketException or HttpRequestException or IOException)
    {
        Console.Error.WriteLine($"{name} could not reach {server}: {error.Message}");
        return 1;
    }

    client.Join(name, seat, StringArg("--ticket"));
    // A person across a Winston or grid draft sees each pick being made: a moment's look first, --draft-think MS.
    var bot = new AutoSeat(client, player) { DraftThink = TimeSpan.FromMilliseconds(IntArg("--draft-think") ?? 1200) };

    while (!client.IsOver)
    {
        // Answered as often as there is something to answer; a pause only when there was nothing, so a bot seat never
        // holds up a game and never spins while it waits for the other.
        if (!bot.Step())
        {
            await Task.Delay(15);
        }

        if (client.Errors.LastOrDefault() is { Code: ErrorCode.MembershipRefused or ErrorCode.TableClosed or ErrorCode.ProtocolMismatch } refusal)
        {
            Console.Error.WriteLine($"{name} was refused: {refusal.Detail}");
            return 1;
        }

        if (!client.IsConnected && !client.IsOver)
        {
            // Whatever arrived before the socket closed is taken in: a game can end in its last message.
            client.Pump();
            if (!client.IsOver)
            {
                Console.Error.WriteLine($"{name} lost the connection to {server}.");
                return 1;
            }
        }
    }

    GameOverMessage outcome = client.Outcome!;
    Console.WriteLine($"{name} (seat {client.Seat}): game over, {outcome.Outcome}, winning team {outcome.WinningTeam}.");
    client.Disconnect();
    return 0;
}

string? StringArg(string flag)
{
    int at = Array.IndexOf(args, flag);
    return at >= 0 && at + 1 < args.Length ? args[at + 1] : null;
}

int? IntArg(string flag) =>
    StringArg(flag) is { } value && int.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out int parsed) ? parsed : null;
