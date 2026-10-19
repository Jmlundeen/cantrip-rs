using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// An event played at one table: what a table hosts when it is part of an event rather than a game set up in its lobby.
/// The table keeps doing what it does - sockets, membership, resuming, dealing and running the game - and asks the
/// event who may sit where, what each seat is shown before the cards are dealt, what to do with what they send, and
/// what to deal.
/// <para>
/// Called only from the table's pump, apart from <see cref="SeatFor"/>, which is also safe from anywhere.
/// </para>
/// </summary>
public interface ITableEvent
{
    string EventId { get; }

    string Name { get; }

    /// <summary>How many seats the table has.</summary>
    int Seats { get; }

    /// <summary>
    /// Where the event is, as the table puts it: Setup while players take their seats, Drafting, DeckBuilding, and
    /// Finished once the event is over. Playing is the table's own to say, from when it deals.
    /// </summary>
    TablePhase Phase { get; }

    /// <summary>The seat a ticket is for, or zero when it is for none of them.</summary>
    int SeatFor(string ticket);

    string NameAt(int seat);

    /// <summary>The id of the player in a seat, for what is kept about them - their portrait - or empty for none.</summary>
    string PlayerAt(int seat) => string.Empty;

    /// <summary>A player has taken their seat, by ticket.</summary>
    void Seated(int seat);

    /// <summary>What a seat is shown of the event now - the draft, or the deck being built - or null for nothing.</summary>
    ServerMessage? ViewFor(int seat);

    /// <summary>Applies what a seat sent, or says why not.</summary>
    /// <returns>Null when it was applied; otherwise why it was refused.</returns>
    string? Receive(int seat, ClientMessage message);

    /// <summary>Whether a game is ready to be dealt: everyone has submitted a deck, or a round was cut off by a restart.</summary>
    bool ReadyToDeal { get; }

    /// <summary>
    /// Deals the next game, with each seat's deck, on the rules engine asked for, and notes that it has begun. The game
    /// comes back as the rules a session plays it by, not as a position: which engine dealt it is the table's choice,
    /// and an event names its cards the same way for either.
    /// </summary>
    IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null);

    /// <summary>The game could not be dealt: the event goes back to where the players can change what stopped it.</summary>
    void DealFailed(string why);

    /// <summary>The game dealt has ended: won by the seats named (none for a draw), or abandoned.</summary>
    void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail);

    // --- A match clock. An event with none needs none of this. ---

    /// <summary>
    /// How long a seat has left on its match clock as a game is dealt, or null for a game with no clock. Asked of every
    /// seat once the game is dealt; a clock runs only when every seat has one.
    /// </summary>
    TimeSpan? ClockAtDeal(int seat) => null;

    /// <summary>What is left on a seat's clock now, to be kept for the match's next game - and a restart.</summary>
    void ClockUsed(int seat, TimeSpan left)
    {
    }

    /// <summary>
    /// A seat's clock has run out. Said before the game is ended for it (<see cref="GameEnded"/>), so the event can take
    /// the whole match from the seat, not just the game.
    /// </summary>
    void ClockRanOut(int seat)
    {
    }

    /// <summary>
    /// The seats that have forfeited the game being played since this was last asked - gone from the table too long -
    /// each said once. The table ends the game with every other seat winning, and says so (<see cref="GameEnded"/>).
    /// </summary>
    IReadOnlyList<int> TakeForfeits() => [];

    /// <summary>
    /// Why a seat forfeited the game when it was not for being away - its player dropped from the event - as the table
    /// is to say it; null for being away.
    /// </summary>
    string? ForfeitReason(int seat) => null;

    // --- A table still being formed from a queue. An event made up front is never forming, and needs none of this. ---

    /// <summary>
    /// Whether seats are still being taken and given up before the event begins (<see cref="QueueTable"/>). While
    /// forming, a player may leave, and once every seat is taken each must say they are ready.
    /// </summary>
    bool Forming => false;

    /// <summary>A seat's player has the table open, or no longer does.</summary>
    void Connected(int seat, bool connected)
    {
    }

    /// <summary>Whether a seat is someone's: always, once the table is formed.</summary>
    bool Holds(int seat) => true;

    /// <summary>Whether the player in a seat has said they are ready, while forming.</summary>
    bool IsReady(int seat) => false;

    /// <summary>How long is left to say ready, while a full forming table waits on it; null otherwise.</summary>
    TimeSpan? ReadyLeft(DateTimeOffset now) => null;

    /// <summary>
    /// Counts up whenever something the players are shown changes other than by what they sent - a seat taken from
    /// outside, by the host - so the table knows to show it again.
    /// </summary>
    int Version => 0;

    /// <summary>Lets time pass: a ready check that runs out gives up the seats of everyone who did not say ready.</summary>
    void Tick(DateTimeOffset now)
    {
    }

    /// <summary>
    /// The seats given up since this was last asked - a player who left, timed out or was moved to another table - each
    /// with what to tell its player before the connection is closed.
    /// </summary>
    IReadOnlyList<(int Seat, ServerMessage Farewell)> TakeVacated() => [];
}
