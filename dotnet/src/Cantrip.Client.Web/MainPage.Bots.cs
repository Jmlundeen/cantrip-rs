using Cantrip.Client;
using Cantrip.Client.Tactics;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// The seats this page plays for its player's opponents (<c>?bots=2</c>): each one a client of its own over a socket of
/// its own, answered by <see cref="TacticalPlay"/> in this browser.
/// <para>
/// A bot is a client like any other, so nothing has to be anywhere in particular to play one: the server sees a seat
/// joined and answered, and cannot tell that the answers were worked out in the tab of the person across the table.
/// Which is the point. An opponent worth playing costs some arithmetic every time it is asked something, and done here
/// it costs whoever wanted the opponent, on the machine they are already playing on, rather than whoever hosts the table.
/// </para>
/// <para>
/// Each seat is kept across a reload as this page's own seat is (<see cref="SeatMemory"/>), so a tab refreshed mid-game
/// takes its opponents' seats back with its own instead of leaving the table waiting on bots that are gone. A seat is
/// played at the one table it was seated at: one that is moved to another table, as an event's are between rounds, is
/// let go rather than followed.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    /// <summary>How often a lost or refused connection for a bot's seat is tried again before it is given up on.</summary>
    private const int BotConnectTries = 5;

    /// <summary>One seat played from this page.</summary>
    private sealed class HostedBot
    {
        /// <summary>The seat to ask for, or zero for one claimed by <see cref="Ticket"/>.</summary>
        public int Seat { get; init; }

        public string? Ticket { get; init; }

        /// <summary>What its seat is remembered under: the seat number, or the ticket's place in the list.</summary>
        public string Memory { get; init; } = string.Empty;

        public string Name { get; init; } = "Bot";

        public AutoSeat? Play { get; set; }

        public MembershipMessage? Remembered { get; set; }

        public bool Connecting { get; set; }

        public int Tries { get; set; }

        /// <summary>Not before this: a connection that failed is tried again after a pause, as the page's own is.</summary>
        public DateTime NextTry { get; set; }

        /// <summary>When the seat was last granted, so one lost again at once is not chased.</summary>
        public DateTime? Granted { get; set; }

        /// <summary>Given up on: the table let the seat go, or it keeps being taken away.</summary>
        public bool Stopped { get; set; }
    }

    private List<HostedBot>? _bots;

    /// <summary>The seats this page was asked to play, made once.</summary>
    private List<HostedBot> HostedBots() => _bots ??=
    [
        .. ClientConfig.BotSeats.Select(seat => new HostedBot
        {
            Seat = seat,
            Memory = $":bot{seat}",
            Name = ClientConfig.BotSeats.Count == 1 ? "Bot" : $"Bot {seat}",
        }),
        .. ClientConfig.BotTickets.Select((ticket, index) => new HostedBot
        {
            Ticket = ticket,
            Memory = $":bot-ticket{index}",
            Name = ClientConfig.BotTickets.Count == 1 ? "Bot" : $"Bot {index + 1}",
        }),
    ];

    /// <summary>
    /// Seats the bots that are not seated, and lets each answer one question. Once a tick and one answer each, so the
    /// table moves at a pace a person can follow and no frame waits on a bot.
    /// </summary>
    private void PlayBots(GameClient mine)
    {
        // Not before this page's own seat is settled: the bots sit down around their player, not instead of them.
        if (mine.Membership is null && mine.View is null)
        {
            return;
        }

        foreach (HostedBot bot in HostedBots())
        {
            try
            {
                PlayBot(mine, bot);
            }
            catch (Exception)
            {
                // Whatever went wrong with one bot's seat - a question it faulted on, storage that would not take its
                // seat - is not allowed to stop the page's own tick, or the other bots'.
                if (bot.Play?.Client is { Prompt: { } asked } client)
                {
                    client.Answer(asked.Prompt.Default());
                }
            }
        }
    }

    private void PlayBot(GameClient mine, HostedBot bot)
    {
        if (bot.Stopped)
        {
            return;
        }

        if (bot.Play is not { } play)
        {
            if (!bot.Connecting && bot.Tries < BotConnectTries && DateTime.UtcNow >= bot.NextTry && !mine.IsOver)
            {
                _ = SeatBotAsync(bot);
            }

            return;
        }

        GameClient client = play.Client;

        if (play.Step())
        {
            _needsRedraw = true;
        }

        if (client.Membership is { } granted && !ReferenceEquals(granted, bot.Remembered))
        {
            bot.Remembered = granted;
            bot.Granted = DateTime.UtcNow;
            SeatMemory.Remember(granted, bot.Memory);
        }

        // Seated and still seated a while later: the tries start over, for the next time the connection drops.
        if (bot.Granted is { } since && DateTime.UtcNow - since >= SettledAfter)
        {
            bot.Tries = 0;
        }

        if (client.Released is not null || client.Moved is not null)
        {
            // Let go by the table, or sent to another one: not this page's to play any more.
            SeatMemory.Forget(bot.Memory);
            client.Disconnect();
            bot.Play = null;
            bot.Stopped = true;
        }
        else if (client.Errors.LastOrDefault() is { Code: ErrorCode.MembershipRefused or ErrorCode.MembershipRequired or ErrorCode.TableClosed })
        {
            // The seat it remembered is gone with the table it was at: asked for afresh next time round.
            SeatMemory.Forget(bot.Memory);
            bot.Remembered = null;
            client.Disconnect();
            bot.Play = null;
            bot.NextTry = DateTime.UtcNow + ReconnectPause;
        }
        else if (!client.IsConnected && !client.IsOver)
        {
            bot.Play = null;
            bot.NextTry = DateTime.UtcNow + ReconnectPause;

            // Lost as soon as it was got back: most likely this tab was duplicated, seats and all, and the other tab
            // has taken it. Each taking it from the other would go on for ever, so this one stops.
            if (bot.Granted is { } granted_ && DateTime.UtcNow - granted_ < SettledAfter && bot.Tries > 1)
            {
                bot.Stopped = true;
            }
        }
    }

    private async Task SeatBotAsync(HostedBot bot)
    {
        bot.Connecting = true;
        bot.Tries++;

        try
        {
            var client = new GameClient(await WebSocketTransport.ConnectAsync(ClientConfig.Server));

            if ((bot.Remembered ?? SeatMemory.Recall(bot.Memory)) is { } seat)
            {
                client.Resume(seat);
            }
            else
            {
                client.Join(bot.Name, bot.Seat, bot.Ticket);
            }

            bot.Play = new AutoSeat(client, ClientConfig.NewBot(name: bot.Name));
        }
        catch (Exception)
        {
            // Tried again after a pause, a few times; a table that cannot be reached shows as much on this page's own seat.
            bot.NextTry = DateTime.UtcNow + ReconnectPause;
        }
        finally
        {
            bot.Connecting = false;
        }
    }
}
