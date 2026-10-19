namespace Cantrip.Server.Events;

/// <summary>The kinds of league there are, as <see cref="LeagueSettings.Format"/> names them.</summary>
public static class LeagueFormats
{
    /// <summary>
    /// Each entry is a booster draft of a pod of eight - the player and seven bots that draft but play nothing - and the
    /// deck built from it is played against other players' quick-draft decks.
    /// </summary>
    public const string QuickDraft = "quick-draft";
}

/// <summary>When a run's deck may be changed from its pool.</summary>
public enum LeagueDeckEditing
{
    /// <summary>Built after the draft, and changed until the run's first match; from then on it is as it was.</summary>
    UntilFirstMatch,

    /// <summary>Changed whenever the run is not in a match.</summary>
    BetweenMatches,
}

/// <summary>Who a player looking for a match is paired with.</summary>
public enum LeaguePairingMode
{
    /// <summary>Whoever else is looking, longest waiting first, whatever their record.</summary>
    Free,

    /// <summary>Only a run with the same record. Not played yet.</summary>
    RecordStrict,

    /// <summary>
    /// A run with the same record at first, then one a game further away for every <see cref="LeagueSettings.LoosenAfterSeconds"/>
    /// waited. Not played yet.
    /// </summary>
    RecordLoose,
}

/// <summary>
/// A league's rules, as whoever runs it sets them (a definition under <c>leagues/</c>): what an entry is, how its
/// matches are played, when a run ends, and who is paired with whom.
/// </summary>
public sealed class LeagueSettings
{
    /// <summary>What an entry is: <see cref="LeagueFormats"/>.</summary>
    public string Format { get; set; } = LeagueFormats.QuickDraft;

    /// <summary>The cube a quick draft drafts, by its name in the cube lists.</summary>
    public string Cube { get; set; } = string.Empty;

    /// <summary>How many seats a quick draft's pod has: the player's, and a bot in each of the rest.</summary>
    public int PodSize { get; set; } = 8;

    public int Packs { get; set; } = 3;

    public int PackSize { get; set; } = 15;

    /// <summary>The fewest cards a main deck may have.</summary>
    public int MinimumDeck { get; set; } = 40;

    /// <summary>Games a player has to win to win a match: one for best of one, two for best of three.</summary>
    public int GamesToWin { get; set; } = 1;

    /// <summary>Match wins that end a run with a trophy; zero for no end by winning.</summary>
    public int WinsToTrophy { get; set; } = 7;

    /// <summary>Match losses that end a run.</summary>
    public int LossesToEnd { get; set; } = 3;

    public LeagueDeckEditing DeckEditing { get; set; } = LeagueDeckEditing.BetweenMatches;

    /// <summary>How many runs a player may make in the league; zero for as many as they like, one for no re-entry.</summary>
    public int MaxEntries { get; set; }

    public LeaguePairingMode Pairing { get; set; } = LeaguePairingMode.Free;

    /// <summary>For <see cref="LeaguePairingMode.RecordLoose"/>: how long a player waits before the record asked of an opponent widens by a game.</summary>
    public int LoosenAfterSeconds { get; set; } = 60;

    /// <summary>Whether a run may be paired again with a player it has already played.</summary>
    public bool AllowRepeatPairings { get; set; } = true;

    /// <summary>When entries open; null for at once.</summary>
    public DateTimeOffset? Starts { get; set; }

    /// <summary>When the league ends - no more entries or matches - or null to run until it is taken down.</summary>
    public DateTimeOffset? Ends { get; set; }

    /// <summary>Why these rules cannot be played, or null when they can.</summary>
    public string? Problem()
    {
        if (Format != LeagueFormats.QuickDraft) return $"'{Format}' is not a league format: only {LeagueFormats.QuickDraft} is played.";
        if (string.IsNullOrWhiteSpace(Cube)) return "A quick draft league names its cube.";
        if (PodSize is < 2 or > 8) return "A quick draft's pod has two to eight seats.";
        if (Packs is < 1 or > 6 || PackSize is < 2 or > 30) return "A quick draft opens one to six packs of two to thirty cards.";
        if (MinimumDeck is < 1 or > 200) return "The smallest deck is between 1 and 200 cards.";
        if (GamesToWin is < 1 or > 3) return "A match is best of one, three or five (one to three games to win).";
        if (WinsToTrophy is < 0 or > 100 || LossesToEnd is < 1 or > 100) return "A run ends after 1 to 100 losses, and 0 to 100 wins.";
        if (MaxEntries is < 0) return "Entries are zero (as many as a player likes) or more.";
        if (Pairing != LeaguePairingMode.Free) return $"Pairing by record ({Pairing}) is not played yet: only Free pairing is.";
        if (Starts is { } starts && Ends is { } ends && ends <= starts) return "A league ends after it starts.";
        return null;
    }
}

/// <summary>A league's own part of its event record: its rules, and every run made in it.</summary>
public sealed class LeagueRecord
{
    public LeagueSettings Settings { get; set; } = new();

    public List<LeagueRun> Runs { get; set; } = [];
}

/// <summary>Where a run has got to.</summary>
public enum LeagueRunStage
{
    /// <summary>Its quick draft is being drafted, or its deck built at the draft's table.</summary>
    Drafting,

    /// <summary>Its deck is built, and it plays matches.</summary>
    Playing,

    /// <summary>Ended: <see cref="LeagueRun.Ending"/> says how.</summary>
    Over,
}

/// <summary>How a run ended, as <see cref="LeagueRun.Ending"/> names it.</summary>
public static class LeagueEndings
{
    public const string Trophy = "trophy";

    /// <summary>It lost as many matches as a run may.</summary>
    public const string Eliminated = "eliminated";

    /// <summary>Its player ended it: it counts as having lost as many matches as a run may.</summary>
    public const string Retired = "retired";

    /// <summary>The league ended while it was being played.</summary>
    public const string LeagueOver = "league-over";

    /// <summary>Its draft never finished: it is no entry at all.</summary>
    public const string CalledOff = "called-off";
}

/// <summary>
/// One entry of a player's into a league: the pool its quick draft gave them, the deck built from it, and how it stands.
/// Its matches are the league's (<see cref="EventRecord.Matches"/>, by <see cref="MatchRecord.Entries"/>), and its record
/// is counted from them.
/// </summary>
public sealed class LeagueRun
{
    /// <summary>The run's id in its league: "run-12".</summary>
    public string Id { get; set; } = string.Empty;

    public string PlayerId { get; set; } = string.Empty;

    public string Name { get; set; } = string.Empty;

    /// <summary>Which of the player's entries this is, from 1.</summary>
    public int Entry { get; set; }

    /// <summary>The quick draft the run's pool comes from, by event id.</summary>
    public string DraftId { get; set; } = string.Empty;

    public LeagueRunStage Stage { get; set; } = LeagueRunStage.Drafting;

    /// <summary>The cards the player drafted, by card id, once the draft is over.</summary>
    public List<string> Pool { get; set; } = [];

    /// <summary>The deck built from <see cref="Pool"/>, as the run's next match is played with.</summary>
    public EventDeck Deck { get; set; } = new();

    public DateTimeOffset Started { get; set; }

    public DateTimeOffset? Ended { get; set; }

    /// <summary>How it ended - <see cref="LeagueEndings"/> - or empty while it is being played.</summary>
    public string Ending { get; set; } = string.Empty;
}
