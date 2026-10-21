using Cantrip.Protocol;

namespace Cantrip.Server.Events;

/// <summary>
/// How long a match's games give their players, whichever event the match is part of: a booster pod's rounds
/// (<see cref="BoosterTiming"/>) or a league's matches.
/// </summary>
public class MatchTiming
{
    /// <summary>How long a player has to change their deck from their sideboard between the games of a match.</summary>
    public TimeSpan Sideboarding { get; init; } = TimeSpan.FromMinutes(3);

    /// <summary>How long a player has to come to their match's table before it is forfeited to their opponent.</summary>
    public TimeSpan NoShow { get; init; } = TimeSpan.FromMinutes(3);

    /// <summary>How long a player may be gone from their match's table before it is forfeited to their opponent.</summary>
    public TimeSpan Away { get; init; } = TimeSpan.FromMinutes(5);

    /// <summary>
    /// Each player's match clock: a chess clock across every game of a match, running while the game waits on them -
    /// priority, the mulligans, every choice. A player whose clock runs out loses the match, whatever the score. An hour, as
    /// every table's (docs/events.md, "Clocks"); zero leaves the players the table's own.
    /// </summary>
    public TimeSpan MatchClock { get; init; } = TimeSpan.FromHours(1);

    /// <summary>How long the end of a game is shown before its players are taken on to the next game, or back.</summary>
    public TimeSpan AfterGame { get; init; } = TimeSpan.FromSeconds(6);
}

/// <summary>
/// The event a match is part of - a booster pod's round, a league's match - as the tables of its games
/// (<see cref="MatchGame"/>) see it. The event keeps everything: the tables only play the games, under its lock, and
/// tell it how each went.
/// </summary>
internal interface IMatchHost
{
    /// <summary>Everything the event and its games' tables do is done under this.</summary>
    object Gate { get; }

    DateTimeOffset Now { get; }

    MatchTiming Timing { get; }

    string EventId { get; }

    /// <summary>The fewest cards a main deck may have.</summary>
    int MinimumDeck { get; }

    EventCards Cards { get; }

    /// <summary>A player of a match, as the table names them.</summary>
    string NameOf(string playerId);

    /// <summary>The pool a player builds their deck for the match from.</summary>
    IReadOnlyList<string> PoolOf(MatchRecord match, string playerId);

    /// <summary>What the table of a game of a match is called: "Round 2, game 1: Alice vs Bob".</summary>
    string TableName(MatchRecord match, int game);

    void Save();

    /// <summary>Something the seats are shown has changed: every table shows it again.</summary>
    void Changed();

    /// <summary>A game's table has opened, and its players can be sent to it.</summary>
    void GameOpened(MatchGame game);

    /// <summary>The table wanted for the game after this one of its match, if there is one yet.</summary>
    MatchGame? Next(MatchGame game);

    /// <summary>Where a player goes once their match is decided - back to the pod's table - or null to stay with the result.</summary>
    TableMoveMessage? BackFrom(MatchGame game, string playerId);

    /// <summary>The deal of a game of a match, from the event's seed.</summary>
    ulong SeedFor(MatchRecord match, int game);

    /// <summary>A game of a match has ended: the match goes on to its next game, or is decided.</summary>
    void GameFinished(MatchGame game, IReadOnlyList<string> winners, bool abandoned, string detail);

    /// <summary>Players were not at their match: it is forfeited to whoever was there.</summary>
    void Forfeited(MatchGame game, IReadOnlyList<string> absent);

    /// <summary>A player's match clock ran out: they lose the match.</summary>
    void TimedOut(MatchGame game, string loser);

    /// <summary>
    /// Both players' match clocks ran out at the same moment: neither is preferred, so the game being played and the match are
    /// drawn, and nothing more is dealt.
    /// </summary>
    void TimedOutTogether(MatchGame game);
}
