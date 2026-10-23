using MessagePack;

namespace Cantrip.Protocol;

/// <summary>
/// A draft as one seat sees it, sent whenever it changes and again on joining or resuming. Every seat is told the piles'
/// sizes, how many cards are left in the stack, who is looking at which pile and what everyone has done; only the seat
/// looking at a pile is told what is in it, and only a seat's own pool is shown to it.
/// </summary>
[MessagePackObject]
public sealed class DraftMessage : ServerMessage
{
    [Key(0)] public string EventId { get; set; } = string.Empty;

    /// <summary>The draft format: "winston", "booster" or "grid".</summary>
    [Key(1)] public string Format { get; set; } = string.Empty;

    /// <summary>The seat whose turn it is; zero once the draft is over.</summary>
    [Key(2)] public int Active { get; set; }

    /// <summary>The pile the active seat is looking at, from zero; -1 once the draft is over.</summary>
    [Key(3)] public int Looking { get; set; } = -1;

    /// <summary>The decision being made, which <see cref="DraftPickMessage.Decision"/> names.</summary>
    [Key(4)] public int Decision { get; set; }

    /// <summary>How many cards are in each pile, in the order they are looked at.</summary>
    [Key(5)] public List<int> Piles { get; set; } = [];

    /// <summary>How many cards are still face down in the stack.</summary>
    [Key(6)] public int Stack { get; set; }

    /// <summary>What is in the pile this seat is looking at, as printed; empty unless it is this seat's turn.</summary>
    [Key(7)] public List<ObjectView> Pile { get; set; } = [];

    /// <summary>Whether this seat may pass the pile it is looking at; the last cards of a draft have to be taken.</summary>
    [Key(8)] public bool CanPass { get; set; }

    /// <summary>This seat's pool, in the order it was drafted.</summary>
    [Key(9)] public List<ObjectView> Pool { get; set; } = [];

    [Key(10)] public List<DraftSeatView> Seats { get; set; } = [];

    /// <summary>
    /// What has happened, newest last, in words for this seat: "You took pile 2 (3 cards)." Each card a line names is
    /// written as its name in square brackets, as the game log writes them, and is in <see cref="RecentCards"/>.
    /// </summary>
    [Key(11)] public List<string> Recent { get; set; } = [];

    /// <summary>How many cards the draft began with.</summary>
    [Key(12)] public int Total { get; set; }

    // --- A booster draft. The pack in front of this seat is Pile, and Decision is how many picks it has made. ---

    /// <summary>The pack being drafted, from 1; zero once the draft is over.</summary>
    [Key(13)] public int PackNumber { get; set; }

    /// <summary>How many packs each player opens.</summary>
    [Key(14)] public int Packs { get; set; }

    /// <summary>Which pick of the pack in front of this seat this is, from 1; zero with no pack in front of it.</summary>
    [Key(15)] public int PickNumber { get; set; }

    /// <summary>How many cards a pack starts with.</summary>
    [Key(16)] public int PackSize { get; set; }

    /// <summary>
    /// How long the pick has left, in milliseconds; zero with no clock running. A booster draft's is this seat's, on the
    /// pack in front of it; a Winston or grid draft's is the active player's, which every seat is shown.
    /// </summary>
    [Key(17)] public int PickLeftMs { get; set; }

    /// <summary>Which way packs are going: 1 to the next seat up, -1 to the next seat down.</summary>
    [Key(18)] public int Direction { get; set; }

    /// <summary>
    /// The clock on the pack in front of this seat has run out, and the server is asking for the card chosen: a pick of
    /// it, or - with none chosen - a pick that takes nothing (<see cref="DraftPickMessage.Take"/> false), which has one
    /// taken for it by the pick order. Asked once a pick, and waited on only a moment before that card is taken anyway.
    /// </summary>
    [Key(19)] public bool PickDue { get; set; }

    /// <summary>
    /// Where each card of <see cref="Pile"/> stands in the cube's pick order - a consensus of card quality, best from 0 -
    /// in the pile's order; a card the order does not name ranks past every one it does. Empty for a cube with no pick
    /// order. What a seat drafting with no ideas of its own (a bot) follows. Every format fills it: a booster draft's
    /// for the pack in front of this seat, a grid draft's for the grid, and a Winston draft's for the pile this seat is
    /// looking at - so empty, as <see cref="Pile"/> is, unless it is this seat's turn; nothing is ranked that this seat
    /// may not see.
    /// </summary>
    [Key(20)] public List<int> PileRanks { get; set; } = [];

    /// <summary>
    /// The places in <see cref="Pool"/> of the cards this seat may put into the pack in front of it for an additional
    /// card from that pack (Cogwork Librarian), the one that would go first first: one more card may be taken with
    /// the pick (<see cref="DraftPickMessage.Also"/>) for each. Empty with none, or with no pack of two or more cards.
    /// </summary>
    [Key(21)] public List<int> AdditionalPicks { get; set; } = [];

    /// <summary>
    /// In a grid draft, the line the last pick took - a row 0 to 2, a column 3 to 5 - from the grid as it was then
    /// (the one shown before this, a new grid having been laid out after a second pick); -1 before any.
    /// </summary>
    [Key(22)] public int LastLine { get; set; } = -1;

    /// <summary>
    /// The cards each line of <see cref="Recent"/> names, line by line, in the order its brackets name them: the card
    /// as printed, to be read by from the line.
    /// </summary>
    [Key(23)] public List<List<LogReferenceView>> RecentCards { get; set; } = [];

    // --- A grid draft. Pile is the grid's nine places row by row from the top left, each seat seeing all of it, a place
    // whose card has been taken an unknown card (Known false); PileRanks are theirs. PackNumber is the grid, Packs how
    // many there are, PickNumber 1 or 2 for the first or second pick from it, and Active whose pick it is. ---

    public override string Describe() => $"draft decision {Decision}, seat {Active} at pile {Looking + 1}";
}

[MessagePackObject]
public sealed class DraftSeatView
{
    [Key(0)] public int Seat { get; set; }
    [Key(1)] public string Name { get; set; } = string.Empty;

    /// <summary>How many cards this seat has drafted. Public: everyone sees how big a pile was when it was taken.</summary>
    [Key(2)] public int Drafted { get; set; }

    /// <summary>How many packs are waiting for this seat to pick from, in a booster draft: public, as at a real table.</summary>
    [Key(3)] public int Queued { get; set; }
}

/// <summary>
/// Building a deck from a pool, as one seat sees it: its pool, which of those cards are in the main deck, the basic
/// lands it has added, and whether each seat has submitted. Sent on entering deck building, on joining or resuming,
/// and whenever any seat's deck changes.
/// </summary>
[MessagePackObject]
public sealed class DeckBuildingMessage : ServerMessage
{
    [Key(0)] public string EventId { get; set; } = string.Empty;

    /// <summary>This seat's pool, as printed. A card is named by its place here.</summary>
    [Key(1)] public List<ObjectView> Pool { get; set; } = [];

    /// <summary>The places in <see cref="Pool"/> of the cards in the main deck; the rest are the sideboard.</summary>
    [Key(2)] public List<int> Main { get; set; } = [];

    /// <summary>Each basic land that may be added, with how many are.</summary>
    [Key(3)] public List<BasicLandView> Basics { get; set; } = [];

    /// <summary>The fewest cards a main deck may have.</summary>
    [Key(4)] public int Minimum { get; set; }

    [Key(5)] public bool Submitted { get; set; }

    /// <summary>
    /// The last of this seat's changes (<see cref="BuildDeckMessage.Change"/>) the server has dealt with, applied or
    /// refused. A client that has sent later ones keeps showing its own deck until they come back too.
    /// </summary>
    [Key(6)] public int Change { get; set; }

    [Key(7)] public List<DeckBuilderSeatView> Seats { get; set; } = [];

    /// <summary>How long is left before the deck as it stands is submitted, in milliseconds; zero with no clock.</summary>
    [Key(8)] public int LeftMs { get; set; }

    /// <summary>What the deck is being built for, when it is not the event's deck: "Sideboarding for game 2 - you lead 1-0".</summary>
    [Key(9)] public string Title { get; set; } = string.Empty;

    /// <summary>
    /// The place in <see cref="Pool"/> of the card chosen as the deck's companion (CR 702.139a), a sideboard card in a column
    /// of its own; -1 for none. The server refuses to take a deck as submitted whose companion's condition it doesn't fulfil.
    /// </summary>
    [Key(10)] public int Companion { get; set; } = -1;

    public override string Describe() => $"deck building, change {Change}{(Submitted ? ", submitted" : string.Empty)}";
}

[MessagePackObject]
public sealed class BasicLandView
{
    [Key(0)] public string Name { get; set; } = string.Empty;
    [Key(1)] public int Count { get; set; }
    [Key(2)] public ObjectView Card { get; set; } = new();
}

[MessagePackObject]
public sealed class DeckBuilderSeatView
{
    [Key(0)] public int Seat { get; set; }
    [Key(1)] public string Name { get; set; } = string.Empty;
    [Key(2)] public bool Submitted { get; set; }
}

/// <summary>
/// Takes or passes the pile this seat is looking at, for the decision named; in a booster draft, takes the card at
/// <see cref="Card"/> in the pack in front of this seat - or, without <see cref="Take"/>, answers a pick that is due
/// (<see cref="DraftMessage.PickDue"/>) with no card chosen; in a grid draft, takes the line <see cref="Card"/> of the
/// grid: a row, 0 to 2 from the top, or a column, 3 to 5 from the left.
/// </summary>
[MessagePackObject]
public sealed class DraftPickMessage : ClientMessage
{
    [Key(0)] public int Decision { get; set; }
    [Key(1)] public bool Take { get; set; }

    /// <summary>The card's place in the pack, from zero, in a booster draft; the row or column taken, in a grid draft.</summary>
    [Key(2)] public int Card { get; set; }

    /// <summary>
    /// The places in the pack of additional cards taken with <see cref="Card"/>, each paid for by a card of this seat's
    /// pool going into the pack (<see cref="DraftMessage.AdditionalPicks"/>). Empty for a pick of one card.
    /// </summary>
    [Key(3)] public List<int> Also { get; set; } = [];

    public override string Describe() => Take
        ? $"take {(Card > 0 || Also.Count > 0 ? $"card {string.Join(" and ", Also.Prepend(Card))} of " : string.Empty)}the pile, decision {Decision}"
        : $"pass the pile, decision {Decision}";
}

/// <summary>
/// This seat's deck as it now stands: the places in its pool of the main deck's cards, and the basic lands added. With
/// <see cref="Submit"/> it is the deck to play; without it, a deck still being built, or taken back to change. Built
/// with a number the client counts up, which the deck view carries back once the server has dealt with it.
/// </summary>
[MessagePackObject]
public sealed class BuildDeckMessage : ClientMessage
{
    [Key(0)] public int Change { get; set; }
    [Key(1)] public List<int> Main { get; set; } = [];
    [Key(2)] public List<BasicLandCount> Basics { get; set; } = [];
    [Key(3)] public bool Submit { get; set; }

    /// <summary>The place in the pool of the card chosen as the deck's companion (CR 702.139a), not in the main deck; -1 for none.</summary>
    [Key(4)] public int Companion { get; set; } = -1;

    public override string Describe() => Submit ? $"submit the deck, change {Change}" : $"build the deck, change {Change}";
}

[MessagePackObject]
public sealed class BasicLandCount
{
    [Key(0)] public string Name { get; set; } = string.Empty;
    [Key(1)] public int Count { get; set; }
}

/// <summary>
/// This seat has been given up and the connection is about to close: the player left the table, or did not say they
/// were ready in time. The client forgets the seat rather than trying to get it back, and says why.
/// </summary>
[MessagePackObject]
public sealed class SeatReleasedMessage : ServerMessage
{
    [Key(0)] public string Detail { get; set; } = string.Empty;
    public override string Describe() => "seat released";
}

/// <summary>
/// This player's seat has moved to another table - two tables short of players made into one - and the connection is
/// about to close: the client goes to the table named and sits down with the ticket.
/// </summary>
[MessagePackObject]
public sealed class TableMoveMessage : ServerMessage
{
    [Key(0)] public string Server { get; set; } = string.Empty;
    [Key(1)] public string Ticket { get; set; } = string.Empty;
    public override string Describe() => "table moved";
}

/// <summary>
/// An event's rounds as one seat sees them at its table: the round's pairings and how each match is going, how long
/// before the next round begins, and the standings - the final ones once the event is over.
/// </summary>
[MessagePackObject]
public sealed class PairingsMessage : ServerMessage
{
    [Key(0)] public string EventId { get; set; } = string.Empty;

    /// <summary>The round paired, from 1.</summary>
    [Key(1)] public int Round { get; set; }

    [Key(2)] public int Rounds { get; set; }

    /// <summary>How long before the round's matches begin, in milliseconds; zero once they have.</summary>
    [Key(3)] public int StartsInMs { get; set; }

    /// <summary>Whether the event is over, and the standings are the final ones.</summary>
    [Key(4)] public bool Final { get; set; }

    [Key(5)] public List<PairingView> Matches { get; set; } = [];

    [Key(6)] public List<StandingView> Standings { get; set; } = [];

    /// <summary>A line on where the event stands.</summary>
    [Key(7)] public string Detail { get; set; } = string.Empty;

    public override string Describe() => Final ? "final standings" : $"pairings, round {Round} of {Rounds}";
}

/// <summary>One match of a round, and how it is going.</summary>
[MessagePackObject]
public sealed class PairingView
{
    [Key(0)] public int Table { get; set; }
    [Key(1)] public int FirstSeat { get; set; }
    [Key(2)] public string First { get; set; } = string.Empty;

    /// <summary>Zero for a bye.</summary>
    [Key(3)] public int SecondSeat { get; set; }

    [Key(4)] public string Second { get; set; } = string.Empty;
    [Key(5)] public int FirstWins { get; set; }
    [Key(6)] public int SecondWins { get; set; }
    [Key(7)] public int Draws { get; set; }

    /// <summary>The game being played, from 1; zero before the first and once the match is over.</summary>
    [Key(8)] public int Game { get; set; }

    [Key(9)] public bool Complete { get; set; }

    /// <summary>How it stands, in words: "Game 2 is being played", "Ann won 2-1", "Bye".</summary>
    [Key(10)] public string Status { get; set; } = string.Empty;
}

/// <summary>One player's place in the standings.</summary>
[MessagePackObject]
public sealed class StandingView
{
    [Key(0)] public int Rank { get; set; }
    [Key(1)] public int Seat { get; set; }
    [Key(2)] public string Name { get; set; } = string.Empty;
    [Key(3)] public int Points { get; set; }
    [Key(4)] public int Wins { get; set; }
    [Key(5)] public int Losses { get; set; }
    [Key(6)] public int Draws { get; set; }

    /// <summary>Opponents' match-win, game-win and opponents' game-win percentages, from 0 to 1.</summary>
    [Key(7)] public double OpponentMatchWin { get; set; }
    [Key(8)] public double GameWin { get; set; }
    [Key(9)] public double OpponentGameWin { get; set; }

    [Key(10)] public bool Dropped { get; set; }
}
