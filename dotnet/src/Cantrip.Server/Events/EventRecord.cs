namespace Cantrip.Server.Events;

/// <summary>
/// Where an event has got to. A draft goes Seating, Drafting, DeckBuilding, then Playing through its rounds
/// (<see cref="EventRecord.Round"/>) to Complete. Other kinds use the stages that apply to them: a league is Playing
/// while its sub-events run, and a sealed event would go from Seating straight to DeckBuilding.
/// </summary>
public enum EventStage { NotActive, Seating, Drafting, DeckBuilding, Playing, Complete, Cancelled }

/// <summary>
/// One organized-play event, as saved in the data directory: who is in it, where it has got to, what each player
/// drafted and built, and the matches played. The authority on an event, read again after a restart.
/// <para>
/// Deliberately general. An event has a <see cref="Kind"/> - a Winston draft today; booster drafts, sealed, leagues
/// tomorrow - and may sit under a parent (<see cref="ParentId"/>) and hold sub-events (<see cref="Children"/>), so a
/// league is an event whose children are its drafts. Participants are a list rather than two fixed seats, and matches
/// a list rather than one game, so neither a pod of eight nor a league of hundreds needs a different shape. What only
/// one kind needs lives in its own section (<see cref="Draft"/>) or its own file beside this one
/// (<see cref="EventStore.SaveDraft"/>).
/// </para>
/// </summary>
public sealed class EventRecord
{
    /// <summary>The shape of this file, for reading an older one after the shape changes.</summary>
    public int Version { get; set; } = 1;

    public string Id { get; set; } = string.Empty;

    /// <summary>What sort of event this is: <see cref="EventKinds.WinstonDraft"/>, and more to come.</summary>
    public string Kind { get; set; } = string.Empty;

    public string Name { get; set; } = string.Empty;

    /// <summary>The event this one is part of - a league, for one of its drafts - or null.</summary>
    public string? ParentId { get; set; }

    /// <summary>The events that are part of this one, by id.</summary>
    public List<string> Children { get; set; } = [];

    public EventStage Stage { get; set; } = EventStage.NotActive;

    /// <summary>The round being played while <see cref="Stage"/> is Playing, from 1; zero before any round.</summary>
    public int Round { get; set; }

    public DateTimeOffset Created { get; set; }

    public DateTimeOffset Updated { get; set; }

    /// <summary>
    /// Where every random choice the event makes comes from - the shuffle of the cards, who picks first, each game's
    /// deal - so an event can be followed, and a game dealt again, exactly.
    /// </summary>
    public ulong Seed { get; set; }

    public List<EventParticipant> Participants { get; set; } = [];

    /// <summary>How a draft event drafts; null for an event that does not draft.</summary>
    public DraftSettings? Draft { get; set; }

    /// <summary>A league's rules and every run its players have made in it; null for any other event.</summary>
    public LeagueRecord? League { get; set; }

    /// <summary>The fewest cards a main deck may have. Forty, for limited.</summary>
    public int MinimumDeck { get; set; } = 40;

    public List<MatchRecord> Matches { get; set; } = [];

    /// <summary>
    /// When the clock on where the event is runs out - deck building, or the wait before a round begins - or null when
    /// nothing is on a clock.
    /// </summary>
    public DateTimeOffset? Deadline { get; set; }

    /// <summary>A line on how the event stands or why it ended, for whoever looks at it.</summary>
    public string Detail { get; set; } = string.Empty;

    /// <summary>
    /// What the host that runs the event keeps with it, by name - where the Discord message about it is, say - so that a
    /// host coming back after a restart can pick up its own loose ends. Nothing here is the event's own business.
    /// </summary>
    public Dictionary<string, string> HostData { get; set; } = new(StringComparer.Ordinal);

    public EventParticipant? Participant(string playerId) =>
        Participants.FirstOrDefault(participant => string.Equals(participant.PlayerId, playerId, StringComparison.Ordinal));

    public EventParticipant? AtSeat(int seat) => Participants.FirstOrDefault(participant => participant.Seat == seat);

    /// <summary>Whether the event is still to be played out: not complete and not cancelled.</summary>
    public bool IsLive => Stage is not (EventStage.Complete or EventStage.Cancelled);
}

/// <summary>The kinds of event there are, as <see cref="EventRecord.Kind"/> names them.</summary>
public static class EventKinds
{
    public const string WinstonDraft = "winston-draft";

    /// <summary>A booster draft of a pod - eight, classically - and the Swiss rounds played with what was drafted.</summary>
    public const string BoosterDraft = "booster-draft";

    /// <summary>A two-player grid draft: nine cards a pack in a grid, a row or a column each.</summary>
    public const string GridDraft = "grid-draft";

    /// <summary>
    /// A league: players enter it, each entry a run with a deck of its own (a quick draft's, its children), and play
    /// matches against other players' runs until the run wins its trophy or loses too often.
    /// </summary>
    public const string League = "league";
}

/// <summary>A player in an event, and everything of theirs the event keeps.</summary>
public sealed class EventParticipant
{
    public string PlayerId { get; set; } = string.Empty;

    /// <summary>What the player is called at the table.</summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>Where they sit in the event's pod, from 1.</summary>
    public int Seat { get; set; }

    /// <summary>Whether they have taken their seat at least once; the draft starts when everyone has.</summary>
    public bool Seated { get; set; }

    /// <summary>
    /// The tickets that put this player in their seat, as SHA-256 hashes in hex: the event file never holds a ticket
    /// itself, so a copy of the data directory seats nobody. The newest few are kept, so a link opened earlier on
    /// another computer still works after a new one is issued.
    /// </summary>
    public List<string> TicketHashes { get; set; } = [];

    /// <summary>The cards this player has drafted or been given, by card id, in the order they came.</summary>
    public List<string> Pool { get; set; } = [];

    /// <summary>The deck they are building or have built from <see cref="Pool"/>.</summary>
    public EventDeck Deck { get; set; } = new();

    /// <summary>Whether they are out of the event: not paired again, though their results stand.</summary>
    public bool Dropped { get; set; }
}

/// <summary>
/// A deck built from a pool: the pool's cards in the main deck, by their place in the pool (so two copies of a card are
/// told apart), and how many of each basic land. The rest of the pool is the sideboard.
/// </summary>
public sealed class EventDeck
{
    public List<int> Main { get; set; } = [];

    /// <summary>Basic lands added, by name, which nobody drafts and anyone may add as many of as they like.</summary>
    public Dictionary<string, int> Basics { get; set; } = [];

    /// <summary>Whether the player has said this is their deck. The round is dealt once everyone has.</summary>
    public bool Submitted { get; set; }

    /// <summary>Bumped with every change, so a change sent from a window showing an older deck can be told apart.</summary>
    public int Revision { get; set; }
}

/// <summary>How an event drafts: the format, the cube, and its numbers, all set by whoever creates the event.</summary>
public sealed class DraftSettings
{
    /// <summary>The draft format: <see cref="DraftFormats"/>.</summary>
    public string Format { get; set; } = DraftFormats.Winston;

    /// <summary>The cube's name, as the host knows it.</summary>
    public string Cube { get; set; } = string.Empty;

    /// <summary>How many piles a Winston draft lays out.</summary>
    public int Piles { get; set; } = 3;

    /// <summary>How many cards are drafted in all, drawn at random from the cube.</summary>
    public int Cards { get; set; }

    /// <summary>How many packs each player opens, in a booster draft; how many grids are laid out, in a grid draft.</summary>
    public int Packs { get; set; }

    /// <summary>How many cards a pack has, in a booster draft.</summary>
    public int PackSize { get; set; }

    /// <summary>How many Swiss rounds are played after the draft; zero for one game at the draft's own table.</summary>
    public int Rounds { get; set; }
}

public static class DraftFormats
{
    public const string Winston = "winston";

    /// <summary>Packs opened and passed round the table.</summary>
    public const string Booster = "booster";

    /// <summary>Two players, each pack a three-by-three grid of which each takes a row or a column.</summary>
    public const string Grid = "grid";
}

/// <summary>One match in an event: the players in it, the round it belongs to, and its games.</summary>
public sealed class MatchRecord
{
    public string Id { get; set; } = string.Empty;
    public int Round { get; set; }
    public List<string> Players { get; set; } = [];

    /// <summary>In a league, each player's run the match is played by, as <see cref="Players"/> lists them; empty otherwise.</summary>
    public List<string> Entries { get; set; } = [];

    public List<GameRecord> Games { get; set; } = [];

    /// <summary>The players who won the match, by player id: empty while it is being played, or for a draw.</summary>
    public List<string> Winners { get; set; } = [];

    public bool Complete { get; set; }

    /// <summary>The table the match is played at in its round, from 1, as the pairings number them.</summary>
    public int Table { get; set; }

    /// <summary>A round's bye: one player, given the match won, two games to none, with nobody to play.</summary>
    public bool Bye { get; set; }

    /// <summary>
    /// Decided by someone not being there - not coming to the table, or gone from it for too long - rather than by its
    /// games: whoever was there won it two games to none, or nobody did when neither was.
    /// </summary>
    public bool Forfeit { get; set; }

    /// <summary>
    /// How many games a player has to win to win the match: one for a single game, two for the best of three.
    /// </summary>
    public int GamesToWin { get; set; } = 1;

    /// <summary>
    /// Each player's deck for the match's next game, by player id: the deck they submitted to the event, changed from
    /// their sideboard between games. Empty until the first game is dealt.
    /// </summary>
    public Dictionary<string, EventDeck> Decks { get; set; } = new(StringComparer.Ordinal);

    /// <summary>A line on how the match stands, for whoever looks at the pairings.</summary>
    public string Detail { get; set; } = string.Empty;

    /// <summary>
    /// Each player's match clock, by player id, in milliseconds left: a chess clock across every game of the match, which
    /// runs while the game waits on its player. Missing for a player whose clock has not started.
    /// </summary>
    public Dictionary<string, long> ClockMs { get; set; } = new(StringComparer.Ordinal);

    /// <summary>The player whose match clock ran out, losing them the match; null otherwise.</summary>
    public string? TimedOut { get; set; }
}

/// <summary>One game of a match.</summary>
public sealed class GameRecord
{
    public int Number { get; set; }

    /// <summary>The deal, so the game can be dealt again exactly.</summary>
    public ulong Seed { get; set; }

    public DateTimeOffset Started { get; set; }
    public DateTimeOffset? Ended { get; set; }

    /// <summary>How it ended: empty while it is being played, else "won", "draw" or "abandoned".</summary>
    public string Ending { get; set; } = string.Empty;

    public List<string> Winners { get; set; } = [];
    public string Detail { get; set; } = string.Empty;
}

/// <summary>
/// A player, as the data directory knows them: their id, the name last seen for them, and every event they are in. The
/// list is what lets a player come back to whatever they were doing, from any computer.
/// </summary>
public sealed class PlayerRecord
{
    public int Version { get; set; } = 1;
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public List<PlayerEventEntry> Events { get; set; } = [];
}

public sealed class PlayerEventEntry
{
    public string EventId { get; set; } = string.Empty;
    public string Kind { get; set; } = string.Empty;
    public DateTimeOffset Joined { get; set; }
}
