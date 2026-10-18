using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// A seat in an event played with no window, across every table the event takes it to: it drafts and builds at the
/// event's table, follows each <see cref="TableMoveMessage"/> to the table of its next game and back, and plays every
/// game it is dealt (<see cref="AutoSeat"/>). What a bot fills a pod's empty seats with, and what a test seats a pod
/// with.
/// </summary>
public sealed class EventSeat : IDisposable
{
    private readonly string _name;
    private readonly Func<IAutoPlayer?> _player;
    private AutoSeat _seat;
    private readonly Func<Uri, Uri> _reach;

    private EventSeat(string name, Func<IAutoPlayer?> player, GameClient client, Uri address, Func<Uri, Uri> reach)
    {
        _reach = reach;
        _name = name;
        _player = player;
        _seat = new AutoSeat(client, player());
        Address = address;
    }

    /// <summary>Sits down at an event's table with a ticket for it.</summary>
    /// <param name="player">Makes how the seat decides, afresh for every table; null plays the one deterministic way.</param>
    /// <param name="reach">Where to connect for an address the server gives, at this table and each it moves the seat to; null connects to the address as given.</param>
    public static async Task<EventSeat> JoinAsync(Uri address, string ticket, string name, Func<IAutoPlayer?>? player = null, CancellationToken cancellation = default, Func<Uri, Uri>? reach = null)
    {
        ArgumentNullException.ThrowIfNull(address);
        reach ??= given => given;
        var client = new GameClient(await WebSocketTransport.ConnectAsync(reach(address), cancellation).ConfigureAwait(false));
        client.Join(name, 0, ticket);
        return new EventSeat(name, player ?? (() => null), client, address, reach);
    }

    /// <summary>The client at the table the seat is at now: a new one at every table.</summary>
    public GameClient Client => _seat.Client;

    /// <summary>The table the seat is at now.</summary>
    public Uri Address { get; private set; }

    /// <summary>How many times the seat has been moved to another table.</summary>
    public int Moves { get; private set; }

    /// <summary>How many games this seat has seen to the end.</summary>
    public int GamesPlayed { get; private set; }

    /// <summary>The question this seat answered most recently at its table, for a report about a game that stopped moving.</summary>
    public string LastAnswered => _seat.LastAnswered;

    /// <summary>Whether the event is over: its table is showing the final standings.</summary>
    public bool Finished => Client.Pairings is { Final: true };

    /// <summary>
    /// Does whatever there is to do: goes to the next table when told to, and otherwise drafts, builds or plays.
    /// </summary>
    /// <returns>Whether anything was done, so a caller pauses only when nothing was.</returns>
    public async Task<bool> StepAsync(CancellationToken cancellation = default)
    {
        bool wasOver = Client.IsOver;
        if (Client.Moved is { } moved)
        {
            Client.Disconnect();
            Uri next = new(moved.Server);
            var client = new GameClient(await WebSocketTransport.ConnectAsync(_reach(next), cancellation).ConfigureAwait(false));
            client.Join(_name, 0, moved.Ticket);
            _seat = new AutoSeat(client, _player());
            Address = next;
            Moves++;
            return true;
        }

        bool acted = _seat.Step();
        if (!wasOver && Client.IsOver)
        {
            GamesPlayed++;
        }

        return acted;
    }

    public void Dispose() => Client.Disconnect();
}
