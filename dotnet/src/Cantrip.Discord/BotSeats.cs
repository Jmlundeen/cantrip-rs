using Cantrip.Client;
using Cantrip.Client.Tactics;
using Cantrip.Protocol;

namespace Cantrip.Discord;

/// <summary>
/// The seats a queue keeps for bots (<see cref="QueueSettings.Bots"/>), played inside the bot's own process: each one a
/// client over the table's socket with a ticket of its own, as a person's browser would be, so a table cannot tell a bot
/// from a player. Whoever hosts the draft starts them - when it begins, and again when it is hosted after a restart.
/// <para>
/// They draft by the cube's pick order and play their games as <see cref="TacticalPlay"/> does: an opponent that weighs
/// the board, for whoever sat down at a table with seats to spare. The same seat runs in a player's own browser
/// (the web client's <c>?bot-tickets=</c>), which is where a bot's arithmetic belongs once a table hands its bots'
/// tickets to a player; until it does they are played here.
/// </para>
/// </summary>
internal static class BotSeats
{
    /// <summary>What a bot's player id starts with in an event: "bot-2" for a bot in seat 2.</summary>
    private const string Prefix = "bot-";

    public static string Id(int seat) => Prefix + seat.ToString(System.Globalization.CultureInfo.InvariantCulture);

    public static bool IsBot(string playerId) => playerId.StartsWith(Prefix, StringComparison.Ordinal);

    /// <summary>A bot as the draft host keeps it beside its players: no Discord account, so id zero.</summary>
    public static DiscordPlayer Player(string name) => new(0, name);

    /// <summary>
    /// Where each bot seat writes its own account of the draft and the games it plays (<see cref="TacticalPlay.Narrator"/>,
    /// which its <see cref="AutoSeat"/> drafts into too): one file a seat, named for when it sat down and who it is. Null,
    /// the default, for none. Set from --bot-narration.
    /// </summary>
    public static string? NarrationFolder { get; set; }

    /// <summary>A file for one seat's narration, or null when nobody asked for one.</summary>
    private static StreamWriter? Narration(string name)
    {
        if (NarrationFolder is not { Length: > 0 } folder)
        {
            return null;
        }

        try
        {
            Directory.CreateDirectory(folder);
            string safe = string.Concat(name.Select(c => char.IsLetterOrDigit(c) ? c : '-'));
            string path = Path.Combine(folder, $"{DateTime.Now:yyyyMMdd-HHmmss}-{safe}-{Guid.NewGuid().ToString("N")[..6]}.log");
            return new StreamWriter(path, append: false) { AutoFlush = true };
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            Console.Error.WriteLine($"{name} cannot write its narration to {folder}: {error.Message}");
            return null;
        }
    }

    /// <summary>How long a bot looks at a Winston pile or a grid before it picks, so the person across sees each pick made.</summary>
    private static readonly TimeSpan DraftThink = TimeSpan.FromMilliseconds(1200);

    /// <summary>
    /// Plays a seat of a Winston or grid draft at its one table: the draft, the deck and the game, until the game is over,
    /// the table goes or <paramref name="cancellation"/> calls it off.
    /// </summary>
    public static async Task PlayDraftAsync(Uri address, string ticket, string name, CancellationToken cancellation)
    {
        try
        {
            var client = new GameClient(await WebSocketTransport.ConnectAsync(address, cancellation).ConfigureAwait(false));
            client.Join(name, 0, ticket);
            using StreamWriter? narration = Narration(name);
            var seat = new AutoSeat(client, new TacticalPlay { Narrator = narration }) { DraftThink = DraftThink };
            try
            {
                while (!cancellation.IsCancellationRequested && !client.IsOver)
                {
                    if (!seat.Step()) await Task.Delay(15, cancellation).ConfigureAwait(false);
                    if (Refused(client, name)) return;
                    if (!client.IsConnected)
                    {
                        client.Pump();
                        if (!client.IsOver) Console.Error.WriteLine($"{name} lost the connection to {address}.");
                        return;
                    }
                }
            }
            finally
            {
                client.Disconnect();
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (Exception error) when (error is System.Net.WebSockets.WebSocketException or HttpRequestException or IOException)
        {
            Console.Error.WriteLine($"{name} could not play at {address}: {error.Message}");
        }
    }

    /// <summary>
    /// Plays a seat of a booster pod across every table it is sent to - the pod's, then each of its games' - until the
    /// final standings, the table goes or <paramref name="cancellation"/> calls it off.
    /// </summary>
    public static async Task PlayPodAsync(Uri address, string ticket, string name, CancellationToken cancellation)
    {
        try
        {
            // One file for the seat across every table it is sent to, each game's decisions in turn.
            using StreamWriter? narration = Narration(name);
            using EventSeat seat = await EventSeat.JoinAsync(address, ticket, name, () => new TacticalPlay { Narrator = narration }, cancellation).ConfigureAwait(false);
            while (!cancellation.IsCancellationRequested && !seat.Finished)
            {
                if (!await seat.StepAsync(cancellation).ConfigureAwait(false)) await Task.Delay(15, cancellation).ConfigureAwait(false);
                if (Refused(seat.Client, name)) return;
                if (!seat.Client.IsConnected)
                {
                    // Let go with a move to the next table, or the final standings; closed with neither, the table is gone.
                    seat.Client.Pump();
                    if (seat.Client.Moved is null && !seat.Finished)
                    {
                        Console.Error.WriteLine($"{name} lost the connection to {seat.Address}.");
                        return;
                    }
                }
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (Exception error) when (error is System.Net.WebSockets.WebSocketException or HttpRequestException or IOException)
        {
            Console.Error.WriteLine($"{name} could not play at {address}: {error.Message}");
        }
    }

    private static bool Refused(GameClient client, string name)
    {
        if (client.Errors.LastOrDefault() is not { Code: ErrorCode.MembershipRefused or ErrorCode.TableClosed or ErrorCode.ProtocolMismatch } refusal) return false;
        Console.Error.WriteLine($"{name} was refused: {refusal.Detail}");
        return true;
    }
}
