using Cantrip.Datasets;
using Cantrip.Protocol;

namespace Cantrip.Server.Events;

/// <summary>How long a league gives its players, on top of what every match does (<see cref="MatchTiming"/>).</summary>
public sealed class LeagueTiming : MatchTiming
{
    /// <summary>
    /// How long a player stays looking for a match with nothing to say they are still there - a page of the site open,
    /// asking after them (<see cref="League.Seen"/>) - so nobody is paired who has gone and would forfeit.
    /// </summary>
    public TimeSpan Searching { get; init; } = TimeSpan.FromSeconds(45);

    /// <summary>How long a deck table waits for its player to come, or to come back, before it closes.</summary>
    public TimeSpan DeckAway { get; init; } = TimeSpan.FromMinutes(2);

    /// <summary>How long a deck table stays open at most: the changes are kept as they are made, so nothing is lost.</summary>
    public TimeSpan DeckMost { get; init; } = TimeSpan.FromMinutes(30);
}

/// <summary>
/// A league being run: players enter it, each entry a run with a deck of its own, and look for matches against other
/// players' runs until the run wins its trophy (<see cref="LeagueSettings.WinsToTrophy"/>) or loses as often as a run
/// may (<see cref="LeagueSettings.LossesToEnd"/>). A quick draft league's run starts with a booster draft against bots
/// (<see cref="BoosterPod"/> with no rounds, a child of the league's event); its host makes the draft and says when it
/// is over (<see cref="DraftOver"/>), and from then on the run plays.
/// <para>
/// A player asks to be paired (<see cref="Search"/>) and the league pairs whoever is looking by its rules
/// (<see cref="LeaguePairing"/>) as time passes (<see cref="Tick"/>). Each game of a match is played at a table of its
/// own (<see cref="MatchGame"/>), which the host opens when the league wants one (<see cref="TablesToOpen"/>) and closes
/// when it is done with (<see cref="TablesDone"/>), as a pod's are. A run's deck is changed at a table of its own too
/// (<see cref="EditDeck"/>), the same deck builder a draft ends with.
/// </para>
/// <para>
/// The record is saved after everything that changes it; who is looking for a match is not, and a restart has them ask
/// again. A match cut off by a restart is played again from the game it had got to.
/// </para>
/// </summary>
public sealed class League : IMatchHost
{
    private readonly EventStore _store;
    private readonly EventRecord _record;
    private readonly TimeProvider _time;
    private readonly EventCards _cards;
    private readonly Dictionary<string, MatchGame> _games = new(StringComparer.Ordinal);
    private readonly Dictionary<string, Looking> _searching = new(StringComparer.Ordinal);
    private readonly Dictionary<string, LeagueDeckTable> _decks = new(StringComparer.Ordinal);
    private readonly List<LeagueDeckTable> _decksDone = [];

    private sealed class Looking(string run, DateTimeOffset since)
    {
        public string Run { get; } = run;
        public DateTimeOffset Since { get; } = since;
        public DateTimeOffset Seen { get; set; } = since;
    }

    private League(EventStore store, EventRecord record, ContentLibrary content, TimeProvider time, LeagueTiming timing)
    {
        _store = store;
        _record = record;
        _time = time;
        Timing = timing;
        _cards = new EventCards(content);
        TakeUp();
    }

    /// <summary>
    /// Opens a league to be run, making it first if the data directory does not have it: by its id, with the name and
    /// rules given, which replace the ones it was saved with - so a definition changed (a later end, say) applies to a
    /// league already running, its runs and matches kept.
    /// </summary>
    public static League Ensure(EventStore store, ContentLibrary content, string id, string name, LeagueSettings settings, TimeProvider? time = null, LeagueTiming? timing = null)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(settings);
        if (!EventStore.ValidId(id)) throw new ArgumentException($"'{id}' is not an event id.", nameof(id));
        if (string.IsNullOrWhiteSpace(name)) throw new ArgumentException("A league has a name.", nameof(name));
        if (settings.Problem() is { } problem) throw new ArgumentException(problem, nameof(settings));
        time ??= TimeProvider.System;
        DateTimeOffset now = time.GetUtcNow();

        EventRecord record = store.LoadEvent(id) ?? new EventRecord
        {
            Id = id,
            Kind = EventKinds.League,
            Stage = EventStage.Playing,
            Created = now,
            Seed = BitConverter.ToUInt64(System.Security.Cryptography.RandomNumberGenerator.GetBytes(8)),
            League = new LeagueRecord(),
        };
        if (record.Kind != EventKinds.League) throw new InvalidOperationException($"Event '{id}' is a {record.Kind}, not a league.");
        record.League ??= new LeagueRecord();
        record.Name = name.Trim();
        record.League.Settings = settings;
        record.MinimumDeck = settings.MinimumDeck;
        // A league taken down and put up again with a later end is running again.
        if (record.Stage == EventStage.Complete && (settings.Ends is null || settings.Ends > now)) record.Stage = EventStage.Playing;
        record.Updated = now;
        store.SaveEvent(record);
        return new League(store, record, content, time, timing ?? new LeagueTiming());
    }

    /// <summary>Opens a saved league to be run, as it was.</summary>
    public static League Open(EventStore store, string id, ContentLibrary content, TimeProvider? time = null, LeagueTiming? timing = null)
    {
        ArgumentNullException.ThrowIfNull(store);
        EventRecord record = store.LoadEvent(id) ?? throw new InvalidOperationException($"No event '{id}' in {store.Root}.");
        if (record.Kind != EventKinds.League || record.League is null) throw new InvalidOperationException($"Event '{id}' is a {record.Kind}, not a league.");
        return new League(store, record, content, time ?? TimeProvider.System, timing ?? new LeagueTiming());
    }

    public object Gate { get; } = new();

    internal LeagueTiming Timing { get; }

    MatchTiming IMatchHost.Timing => Timing;

    public DateTimeOffset Now => _time.GetUtcNow();

    public string EventId => _record.Id;

    public string Name
    {
        get { lock (Gate) return _record.Name; }
    }

    /// <summary>The league's rules. A copy's worth: nothing changes them while it runs.</summary>
    public LeagueSettings Settings
    {
        get { lock (Gate) return _record.League!.Settings; }
    }

    private LeagueSettings Rules => _record.League!.Settings;

    private List<LeagueRun> Runs => _record.League!.Runs;

    private bool IsOpen(DateTimeOffset now) =>
        _record.Stage == EventStage.Playing && (Rules.Starts is not { } starts || now >= starts) && (Rules.Ends is not { } ends || now < ends);

    // --- Entering, and the draft. ----------------------------------------------------------------------------------

    /// <summary>Why a player may not make a new run now, or null when they may.</summary>
    public string? CanEnter(string playerId)
    {
        lock (Gate) return EntryRefusal(playerId, Now);
    }

    private string? EntryRefusal(string playerId, DateTimeOffset now)
    {
        if (_record.Stage != EventStage.Playing || Rules.Ends is { } ends && now >= ends) return "The league is over.";
        if (Rules.Starts is { } starts && now < starts) return "The league has not begun yet.";
        if (Current(playerId) is not null) return "You already have a run going in this league.";
        if (Rules.MaxEntries > 0 && EntriesOf(playerId) >= Rules.MaxEntries)
            return Rules.MaxEntries == 1 ? "You have had your entry into this league." : $"You have had all {Rules.MaxEntries} of your entries into this league.";
        return null;
    }

    /// <summary>
    /// A player makes a new run, its quick draft made by <paramref name="draft"/> - which returns the draft's event id -
    /// under the league's lock, so the draft's saving itself as a child of the league never meets the league's own save.
    /// The run plays once the host says the draft is over (<see cref="DraftOver"/>). Throws when they may not enter
    /// (<see cref="CanEnter"/>).
    /// </summary>
    public LeagueRun Enter(string playerId, string name, Func<string> draft)
    {
        ArgumentNullException.ThrowIfNull(draft);
        lock (Gate)
        {
            DateTimeOffset now = Now;
            if (EntryRefusal(playerId, now) is { } refusal) throw new InvalidOperationException(refusal);
            string draftId = draft();
            var run = new LeagueRun
            {
                Id = $"run-{Runs.Count + 1}",
                PlayerId = playerId,
                Name = name.Trim(),
                Entry = Runs.Count(each => each.PlayerId == playerId) + 1,
                DraftId = draftId,
                Started = now,
            };
            Runs.Add(run);
            if (_record.Participant(playerId) is { } known)
            {
                known.Name = run.Name;
            }
            else
            {
                _record.Participants.Add(new EventParticipant { PlayerId = playerId, Name = run.Name, Seat = _record.Participants.Count + 1, Seated = true });
            }

            if (!_record.Children.Contains(draftId)) _record.Children.Add(draftId);
            Save();
            _store.Enlist(playerId, run.Name, _record, now);
            return run;
        }
    }

    /// <summary>The quick drafts of runs still drafting, by event id: what the host watches to say when each is over.</summary>
    public IReadOnlyList<string> Drafting
    {
        get { lock (Gate) return [.. Runs.Where(run => run.Stage == LeagueRunStage.Drafting).Select(run => run.DraftId)]; }
    }

    /// <summary>A run's quick draft is over and its deck is in: the run plays from now on, with that pool and deck.</summary>
    public void DraftOver(string draftId, IReadOnlyList<string> pool, EventDeck deck)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        lock (Gate)
        {
            if (RunOfDraft(draftId) is not { Stage: LeagueRunStage.Drafting } run) return;
            run.Pool = [.. pool];
            run.Deck = EventCards.Copy(deck);
            run.Stage = LeagueRunStage.Playing;
            if (!IsOpen(Now)) End(run, LeagueEndings.LeagueOver);
            Save();
        }
    }

    /// <summary>A run's quick draft ended without its deck: the run is no entry, and the player may enter again.</summary>
    public void DraftCalledOff(string draftId)
    {
        lock (Gate)
        {
            if (RunOfDraft(draftId) is not { Stage: LeagueRunStage.Drafting } run) return;
            End(run, LeagueEndings.CalledOff);
            Save();
        }
    }

    private LeagueRun? RunOfDraft(string draftId) => Runs.FirstOrDefault(run => run.DraftId == draftId);

    /// <summary>The player's run still being played, if they have one.</summary>
    private LeagueRun? Current(string playerId) => Runs.LastOrDefault(run => run.PlayerId == playerId && run.Stage != LeagueRunStage.Over);

    private int EntriesOf(string playerId) => Runs.Count(run => run.PlayerId == playerId && run.Ending != LeagueEndings.CalledOff);

    /// <summary>The run a player is drafting for, by its quick draft's event id; null when they are not drafting.</summary>
    public string? DraftOf(string playerId)
    {
        lock (Gate) return Current(playerId) is { Stage: LeagueRunStage.Drafting } run ? run.DraftId : null;
    }

    // --- Ending a run. ---------------------------------------------------------------------------------------------

    /// <summary>
    /// A player ends their run before it is over: it counts as having lost as many matches as a run may. Refused while
    /// a match of theirs is being played. Returns the run's quick draft when it was still drafting, for the host to call
    /// off; and why not, when it was refused.
    /// </summary>
    public (string? Draft, string? Refusal) EndRun(string playerId)
    {
        lock (Gate)
        {
            if (Current(playerId) is not { } run) return (null, "You have no run going in this league.");
            if (MatchOf(run) is not null) return (null, "Your match is being played: finish it first.");
            string? draft = run.Stage == LeagueRunStage.Drafting ? run.DraftId : null;
            _searching.Remove(playerId);
            if (_decks.Remove(playerId, out LeagueDeckTable? table)) Close(table, "You ended your run.");
            End(run, LeagueEndings.Retired);
            Save();
            return (draft, null);
        }
    }

    private void End(LeagueRun run, string ending)
    {
        if (run.Stage == LeagueRunStage.Over) return;
        run.Stage = LeagueRunStage.Over;
        run.Ending = ending;
        run.Ended = Now;
        _searching.Remove(run.PlayerId);
        if (_decks.Remove(run.PlayerId, out LeagueDeckTable? table)) Close(table, "Your run is over.");
    }

    /// <summary>A run that has just played a match ends if it has won or lost enough, or the league has ended.</summary>
    private void Settle(LeagueRun run)
    {
        if (run.Stage != LeagueRunStage.Playing) return;
        (int wins, int losses, _) = RecordOf(run);
        if (Rules.WinsToTrophy > 0 && wins >= Rules.WinsToTrophy) End(run, LeagueEndings.Trophy);
        else if (losses >= Rules.LossesToEnd) End(run, LeagueEndings.Eliminated);
        else if (!IsOpen(Now)) End(run, LeagueEndings.LeagueOver);
    }

    /// <summary>
    /// A run's matches won, lost and drawn. A match forfeited by both players is a loss for each; a run its player ended
    /// has lost as many as a run may.
    /// </summary>
    private (int Wins, int Losses, int Draws) RecordOf(LeagueRun run)
    {
        int wins = 0, losses = 0, draws = 0;
        foreach (MatchRecord match in _record.Matches.Where(match => match.Complete && match.Entries.Contains(run.Id)))
        {
            if (match.Winners.Contains(run.PlayerId)) wins++;
            else if (match.Winners.Count > 0 || match.Forfeit) losses++;
            else draws++;
        }

        if (run.Ending == LeagueEndings.Retired) losses = Math.Max(losses, Rules.LossesToEnd);
        return (wins, losses, draws);
    }

    // --- Looking for a match. --------------------------------------------------------------------------------------

    /// <summary>A player looks for a match for their run. Null once they are looking; otherwise why not.</summary>
    public string? Search(string playerId)
    {
        lock (Gate)
        {
            DateTimeOffset now = Now;
            if (!IsOpen(now)) return "The league is over.";
            if (Current(playerId) is not { } run) return "You have no run going in this league: enter it first.";
            if (run.Stage == LeagueRunStage.Drafting) return "Your draft isn't over yet.";
            if (MatchOf(run) is not null) return "You are already in a match.";
            if (_decks.ContainsKey(playerId)) return "You are changing your deck: save it first.";
            if (_searching.TryGetValue(playerId, out Looking? looking))
            {
                looking.Seen = now;
                return null;
            }

            _searching[playerId] = new Looking(run.Id, now);
            return null;
        }
    }

    /// <summary>A player stops looking for a match. Null once they have; otherwise why not.</summary>
    public string? StopSearching(string playerId)
    {
        lock (Gate) return _searching.Remove(playerId) ? null : "You are not looking for a match.";
    }

    /// <summary>
    /// A page of the site that a player has open asked after them: they are still there, so they go on looking for a
    /// match. Someone looking who is not seen for a while (<see cref="LeagueTiming.Searching"/>) stops.
    /// </summary>
    public void Seen(string playerId)
    {
        lock (Gate)
        {
            if (_searching.TryGetValue(playerId, out Looking? looking)) looking.Seen = Now;
        }
    }

    /// <summary>Whether a player is looking for a match.</summary>
    public bool IsSearching(string playerId)
    {
        lock (Gate) return _searching.ContainsKey(playerId);
    }

    /// <summary>
    /// Lets time pass: whoever has not been seen for a while stops looking, whoever is still looking is paired, and the
    /// league ends when its time is up.
    /// </summary>
    public void Tick(DateTimeOffset now)
    {
        lock (Gate)
        {
            if (_record.Stage == EventStage.Playing && Rules.Ends is { } ends && now >= ends)
            {
                _record.Stage = EventStage.Complete;
                _record.Detail = "The league is over.";
                _searching.Clear();
                foreach (LeagueRun run in Runs.Where(run => run.Stage == LeagueRunStage.Playing && MatchOf(run) is null).ToList())
                {
                    End(run, LeagueEndings.LeagueOver);
                }

                Save();
            }

            foreach ((string player, Looking looking) in _searching.ToList())
            {
                if (now - looking.Seen >= Timing.Searching || Current(player) is not { Stage: LeagueRunStage.Playing } run || run.Id != looking.Run)
                {
                    _searching.Remove(player);
                }
            }

            foreach (LeagueDeckTable table in _decks.Values.ToList())
            {
                if (table.Expired(now))
                {
                    _decks.Remove(table.PlayerId);
                    Close(table, "Your deck is kept as it was when you left it.");
                }
            }

            if (_searching.Count < 2) return;
            List<LeagueSearcher> searchers = [.. _searching.Select(pair =>
            {
                LeagueRun run = Runs.First(each => each.Id == pair.Value.Run);
                (int wins, int losses, _) = RecordOf(run);
                return new LeagueSearcher(pair.Key, run.Id, wins, losses, pair.Value.Since);
            })];
            IReadOnlyList<(LeagueSearcher First, LeagueSearcher Second)> pairs = LeaguePairing.Pair(searchers, Rules, Met, now);
            foreach ((LeagueSearcher first, LeagueSearcher second) in pairs)
            {
                _searching.Remove(first.PlayerId);
                _searching.Remove(second.PlayerId);
                MakeMatch(Runs.First(run => run.Id == first.RunId), Runs.First(run => run.Id == second.RunId));
            }

            if (pairs.Count > 0) Save();
        }
    }

    /// <summary>Whether a run has played a player before.</summary>
    private bool Met(string runId, string playerId) =>
        _record.Matches.Any(match => match.Entries.Contains(runId) && match.Players.Contains(playerId));

    /// <summary>A match between two runs, its first game wanted at a table at once.</summary>
    private void MakeMatch(LeagueRun first, LeagueRun second)
    {
        int number = _record.Matches.Count + 1;
        var match = new MatchRecord
        {
            Id = $"m{number}",
            Round = number,
            Players = [first.PlayerId, second.PlayerId],
            Entries = [first.Id, second.Id],
            GamesToWin = Rules.GamesToWin,
            Detail = "Game 1 is about to begin.",
        };
        foreach (LeagueRun run in new[] { first, second })
        {
            // A deck left part-built at its table is made up to the smallest deck, as a draft's clock does.
            EventDeck deck = run.Deck.Submitted ? run.Deck : PoolDecks.Submitted(run.Pool, run.Deck, Rules.MinimumDeck, _cards.ColorsOf);
            match.Decks[run.PlayerId] = EventCards.Copy(deck);
            match.Decks[run.PlayerId].Submitted = true;
        }

        _record.Matches.Add(match);
        Want(match, 1);
    }

    /// <summary>The match a run is in now, if it is in one.</summary>
    private MatchRecord? MatchOf(LeagueRun run) => _record.Matches.FirstOrDefault(match => !match.Complete && match.Entries.Contains(run.Id));

    /// <summary>Whether a player is in a league match now.</summary>
    public bool InMatch(string playerId)
    {
        lock (Gate) return Current(playerId) is { } run && MatchOf(run) is not null;
    }

    // --- The tables. -----------------------------------------------------------------------------------------------

    /// <summary>The games whose tables are wanted and not open yet: the host opens each and says where (<see cref="MatchGame.Opened"/>).</summary>
    public IReadOnlyList<MatchGame> TablesToOpen()
    {
        lock (Gate) return [.. _games.Values.Where(game => game.Address is null && !game.Done)];
    }

    /// <summary>The games whose tables are done with, for the host to close, each said once.</summary>
    public IReadOnlyList<MatchGame> TablesDone()
    {
        lock (Gate)
        {
            List<MatchGame> done = [.. _games.Values.Where(game => game.Done)];
            foreach (MatchGame game in done) _games.Remove(game.Key);
            return done;
        }
    }

    /// <summary>
    /// A fresh ticket to the table a player should be at now - their match's game, or the table their deck is being
    /// changed at - and where it is; or why there is none.
    /// </summary>
    public (Uri? Address, string? Ticket, string? Refusal) Seat(string playerId)
    {
        lock (Gate)
        {
            if (Current(playerId) is not { } run) return (null, null, "You have no run going in this league.");
            if (MatchOf(run) is { } match)
            {
                MatchGame? game = _games.Values.Where(each => each.Match == match && !each.Done).OrderByDescending(each => each.Number).FirstOrDefault();
                return game?.Address is { } address
                    ? (address, game.IssueTicket(playerId), null)
                    : (null, null, "Your match's table is opening. Try again in a moment.");
            }

            if (_decks.TryGetValue(playerId, out LeagueDeckTable? table) && table.Address is { } at) return (at, table.IssueTicket(), null);
            return (null, null, "You are not in a match.");
        }
    }

    /// <summary>
    /// The table a player's run's deck is changed at, made if it is not there yet, for the host to open
    /// (<see cref="LeagueDeckTable.Opened"/>) and seat them at (<see cref="Seat"/>); or why the deck may not be changed now.
    /// </summary>
    public (LeagueDeckTable? Table, string? Refusal) EditDeck(string playerId)
    {
        lock (Gate)
        {
            if (Current(playerId) is not { } run) return (null, "You have no run going in this league.");
            if (EditRefusal(run) is { } refusal) return (null, refusal);
            if (!_decks.TryGetValue(playerId, out LeagueDeckTable? table))
            {
                table = new LeagueDeckTable(this, run);
                _decks[playerId] = table;
                _searching.Remove(playerId);
            }

            return (table, null);
        }
    }

    /// <summary>Why a run's deck may not be changed now, or null when it may.</summary>
    private string? EditRefusal(LeagueRun run)
    {
        if (run.Stage == LeagueRunStage.Drafting) return "Build your deck at your draft's table.";
        if (run.Stage == LeagueRunStage.Over) return "That run is over.";
        if (MatchOf(run) is not null) return "Your match is being played.";
        if (Rules.DeckEditing == LeagueDeckEditing.UntilFirstMatch && _record.Matches.Any(match => match.Entries.Contains(run.Id)))
            return "In this league a deck is kept as it is once its first match is played.";
        return null;
    }

    /// <summary>The deck tables done with - saved, left, or closed - for the host to close, each said once.</summary>
    public IReadOnlyList<LeagueDeckTable> DeckTablesDone()
    {
        lock (Gate)
        {
            List<LeagueDeckTable> done = [.. _decksDone, .. _decks.Values.Where(table => table.Done)];
            _decksDone.Clear();
            foreach (LeagueDeckTable table in done) _decks.Remove(table.PlayerId);
            return done;
        }
    }

    private void Close(LeagueDeckTable table, string why)
    {
        table.Close(why);
        _decksDone.Add(table);
    }

    /// <summary>A run's deck was changed at its table.</summary>
    internal void DeckChanged(LeagueRun run, EventDeck deck)
    {
        run.Deck = deck;
        Save();
    }

    /// <summary>A run's record as its deck table titles it: "3-1".</summary>
    internal string RecordLine(LeagueRun run)
    {
        (int wins, int losses, int draws) = RecordOf(run);
        return $"{wins}-{losses}" + (draws > 0 ? $"-{draws}" : string.Empty);
    }

    private MatchGame Want(MatchRecord match, int number)
    {
        var game = new MatchGame(this, match, number);
        _games[game.Key] = game;
        return game;
    }

    // --- For its games' tables (IMatchHost). -----------------------------------------------------------------------

    EventCards IMatchHost.Cards => _cards;

    internal EventCards Cards => _cards;

    public int MinimumDeck => _record.MinimumDeck;

    public string NameOf(string playerId) => _record.Participant(playerId)?.Name ?? playerId;

    public IReadOnlyList<string> PoolOf(MatchRecord match, string playerId)
    {
        int index = match.Players.IndexOf(playerId);
        string? run = index >= 0 && index < match.Entries.Count ? match.Entries[index] : null;
        return Runs.FirstOrDefault(each => each.Id == run)?.Pool ?? [];
    }

    public string TableName(MatchRecord match, int game) =>
        $"{_record.Name}: {NameOf(match.Players[0])} vs {NameOf(match.Players[1])}" + (match.GamesToWin > 1 ? $", game {game}" : string.Empty);

    public void Changed()
    {
        foreach (MatchGame game in _games.Values) game.Changed();
    }

    public void GameOpened(MatchGame game)
    {
    }

    public MatchGame? Next(MatchGame game) => _games.GetValueOrDefault($"{game.Match.Id}g{game.Number + 1}");

    /// <summary>A league has no table of its own to go back to: its players stay with the result, which leads back to the league's page.</summary>
    public TableMoveMessage? BackFrom(MatchGame game, string playerId) => null;

    public ulong SeedFor(MatchRecord match, int game) =>
        unchecked(_record.Seed + ((ulong)match.Round * 0x9E3779B97F4A7C15UL) + ((ulong)game * 0x165667B19E3779F9UL));

    public void GameFinished(MatchGame game, IReadOnlyList<string> winners, bool abandoned, string detail)
    {
        MatchRecord match = game.Match;
        if (match.Complete) return;
        if (match.Games.FirstOrDefault(each => each.Number == game.Number) is { } record)
        {
            record.Ended = Now;
            record.Ending = abandoned ? "abandoned" : winners.Count == 0 ? "draw" : "won";
            record.Winners = [.. winners];
            record.Detail = detail;
        }

        (int first, int second, int draws) = Score(match);
        int played = first + second + draws;
        if (first >= match.GamesToWin || second >= match.GamesToWin || played >= 2 * match.GamesToWin - 1 || abandoned)
        {
            match.Winners = first > second ? [match.Players[0]] : second > first ? [match.Players[1]] : [];
            match.Detail = match.Winners.Count == 0
                ? $"Drawn {first}-{second}" + (draws > 0 ? $"-{draws}" : string.Empty)
                : $"{NameOf(match.Winners[0])} won {Math.Max(first, second)}-{Math.Min(first, second)}" + (draws > 0 ? $"-{draws}" : string.Empty);
            Decided(match);
        }
        else
        {
            // The next game, at a table of its own, after sideboarding.
            foreach (string player in match.Players) match.Decks[player].Submitted = false;
            match.Detail = $"Game {game.Number + 1} is about to begin.";
            Want(match, game.Number + 1);
        }

        Changed();
        Save();
    }

    public void Forfeited(MatchGame game, IReadOnlyList<string> absent)
    {
        MatchRecord match = game.Match;
        if (match.Complete) return;
        if (match.Games.FirstOrDefault(each => each.Number == game.Number && each.Ending.Length == 0) is { } record)
        {
            record.Ended = Now;
            record.Ending = "abandoned";
            record.Detail = "Not finished: a player was not there.";
        }

        match.Forfeit = true;
        match.Winners = [.. match.Players.Where(player => !absent.Contains(player))];
        string names = string.Join(" and ", absent.Select(NameOf));
        match.Detail = match.Winners.Count == 0
            ? "Neither player came: a loss for both."
            : $"{NameOf(match.Winners[0])} won: {names} {(absent.Count == 1 ? "was" : "were")} not there.";
        Decided(match);
        Changed();
        Save();
    }

    public void TimedOut(MatchGame game, string loser)
    {
        MatchRecord match = game.Match;
        if (match.Complete) return;
        string winner = match.Players.First(player => player != loser);
        if (match.Games.FirstOrDefault(each => each.Number == game.Number && each.Ending.Length == 0) is { } record)
        {
            record.Ended = Now;
            record.Ending = "won";
            record.Winners = [winner];
            record.Detail = $"{NameOf(loser)} ran out of time.";
        }

        match.TimedOut = loser;
        match.Winners = [winner];
        match.Detail = $"{NameOf(winner)} won: {NameOf(loser)} ran out of time.";
        Decided(match);
        Changed();
        Save();
    }

    /// <summary>A match is decided: each of its runs ends if that was its last.</summary>
    private void Decided(MatchRecord match)
    {
        match.Complete = true;
        foreach (string id in match.Entries)
        {
            if (Runs.FirstOrDefault(run => run.Id == id) is { } run) Settle(run);
        }
    }

    private static (int First, int Second, int Draws) Score(MatchRecord match)
    {
        (int first, _, int draws) = Swiss.GamesOf(match, match.Players[0]);
        (int second, _, _) = Swiss.GamesOf(match, match.Players[1]);
        return (first, second, draws);
    }

    public void Save()
    {
        _record.Updated = Now;
        _store.SaveEvent(_record);
    }

    /// <summary>Takes the league up where it was saved: a table wanted again for every match that was being played.</summary>
    private void TakeUp()
    {
        foreach (MatchRecord match in _record.Matches.Where(match => !match.Complete))
        {
            int ended = match.Games.Count(game => game.Ending.Length > 0);
            bool cutOff = match.Games.Any(game => game.Ending.Length == 0);
            Want(match, cutOff ? match.Games.First(game => game.Ending.Length == 0).Number : ended + 1);
        }
    }

    // --- What the site shows. --------------------------------------------------------------------------------------

    /// <summary>The league as a player sees it - or anyone, for null: names and records only, never a deck but their own.</summary>
    public LeagueView View(string? playerId)
    {
        lock (Gate)
        {
            DateTimeOffset now = Now;
            List<LeagueRunView> active = [.. Runs.Where(run => run.Stage != LeagueRunStage.Over).Select(RunView)
                .OrderByDescending(run => run.Wins).ThenBy(run => run.Losses).ThenBy(run => run.Started)];
            List<LeagueTrophies> trophies = [.. Runs.Where(run => run.Ending == LeagueEndings.Trophy).GroupBy(run => run.PlayerId)
                .Select(group => new LeagueTrophies(NameOf(group.Key), group.Count(), group.Key == playerId))
                .OrderByDescending(each => each.Trophies).ThenBy(each => each.Name, StringComparer.OrdinalIgnoreCase)];
            List<LeagueMatchView> recent = [.. _record.Matches.Where(match => match.Complete).OrderByDescending(match => match.Round).Take(8).Select(match =>
                new LeagueMatchView(NameOf(match.Players[0]), NameOf(match.Players[1]), match.Winners.Count == 1 ? NameOf(match.Winners[0]) : null, match.Detail,
                    match.Games.Select(game => game.Ended).Max(), playerId is not null && match.Players.Contains(playerId)))];

            LeagueMine? mine = null;
            if (playerId is not null)
            {
                LeagueRun? current = Current(playerId);
                MatchRecord? match = current is null ? null : MatchOf(current);
                string? opponent = match?.Players.FirstOrDefault(player => player != playerId) is { } other ? NameOf(other) : null;
                bool ready = match is not null && _games.Values.Any(game => game.Match == match && !game.Done && game.Address is not null);
                int entries = EntriesOf(playerId);
                mine = new LeagueMine(
                    current is null ? null : RunView(current),
                    [.. Runs.Where(run => run.PlayerId == playerId && run.Stage == LeagueRunStage.Over && run.Ending != LeagueEndings.CalledOff)
                        .OrderByDescending(run => run.Ended).Select(RunView)],
                    entries,
                    Rules.MaxEntries > 0 ? Math.Max(0, Rules.MaxEntries - entries) : null,
                    EntryRefusal(playerId, now),
                    current is null ? "You have no run going." : EditRefusal(current),
                    _searching.TryGetValue(playerId, out Looking? looking) ? looking.Since : null,
                    opponent,
                    ready,
                    _decks.ContainsKey(playerId),
                    Runs.Count(run => run.PlayerId == playerId && run.Ending == LeagueEndings.Trophy));
            }

            return new LeagueView(_record.Id, _record.Name, Rules, IsOpen(now), _record.Stage == EventStage.Playing && Rules.Starts is { } starts && now < starts,
                active, trophies, recent, _record.Participants.Count, Runs.Count(run => run.Ending != LeagueEndings.CalledOff), _searching.Count, mine);
        }
    }

    private LeagueRunView RunView(LeagueRun run)
    {
        (int wins, int losses, int draws) = RecordOf(run);
        MatchRecord? match = MatchOf(run);
        string status = run.Stage switch
        {
            LeagueRunStage.Drafting => "Drafting",
            LeagueRunStage.Playing when match is not null => "Playing a match",
            LeagueRunStage.Playing when _searching.ContainsKey(run.PlayerId) => "Looking for a match",
            LeagueRunStage.Playing when _decks.ContainsKey(run.PlayerId) => "Changing their deck",
            LeagueRunStage.Playing => "Between matches",
            _ => run.Ending switch
            {
                LeagueEndings.Trophy => "Trophy",
                LeagueEndings.Eliminated => "Out",
                LeagueEndings.Retired => "Ended early",
                LeagueEndings.LeagueOver => "League over",
                _ => "Called off",
            },
        };
        List<LeagueRunMatch> matches = [.. _record.Matches.Where(each => each.Entries.Contains(run.Id)).OrderBy(each => each.Round).Select(each =>
        {
            string opponent = each.Players.FirstOrDefault(player => player != run.PlayerId) is { } other ? NameOf(other) : string.Empty;
            string result = !each.Complete ? "playing" : each.Winners.Contains(run.PlayerId) ? "won" : each.Winners.Count > 0 || each.Forfeit ? "lost" : "drawn";
            return new LeagueRunMatch(opponent, result, each.Detail);
        })];
        return new LeagueRunView(run.Id, run.PlayerId, run.Name, run.Entry, run.Stage, wins, losses, draws, run.Ending, status, run.Started, run.Ended,
            run.Stage == LeagueRunStage.Drafting ? 0 : PoolDecks.Cards(run.Pool, run.Deck, _cards.Basics).Count, matches);
    }

    /// <summary>A player's own run's deck, card by card, and its sideboard; null for a run that is not theirs or has no deck yet.</summary>
    public LeagueDeckView? DeckOf(string playerId, string runId)
    {
        lock (Gate)
        {
            if (Runs.FirstOrDefault(run => run.Id == runId && run.PlayerId == playerId) is not { } run || run.Stage == LeagueRunStage.Drafting) return null;
            List<LeagueDeckCard> Counted(IEnumerable<string> ids) => [.. ids.GroupBy(id => id, StringComparer.Ordinal).Select(group =>
            {
                ObjectView card = _cards.Card(group.Key);
                string kind = card.CardTypes.Contains(CardTypeView.Land) ? "land" : card.CardTypes.Contains(CardTypeView.Creature) ? "creature" : "spell";
                return new LeagueDeckCard(card.Name, group.Count(), card.ManaCost, card.ManaValue, kind, string.Join(' ', card.CardTypes));
            }).OrderBy(card => card.ManaValue).ThenBy(card => card.Name, StringComparer.OrdinalIgnoreCase)];
            List<string> main = [.. PoolDecks.Cards(run.Pool, run.Deck, _cards.Basics).Select(card => card.Value)];
            List<string> side = [.. PoolDecks.Sideboard(run.Pool, run.Deck).Select(card => card.Value)];
            return new LeagueDeckView(run.Id, run.Entry, Counted(main), Counted(side), main.Count, EditRefusal(run) is null && run.Stage == LeagueRunStage.Playing);
        }
    }
}

/// <summary>A league as its page shows it.</summary>
/// <param name="Open">Being played: begun, and not over.</param>
/// <param name="Upcoming">Not begun yet.</param>
/// <param name="Players">Every run being played, best record first.</param>
/// <param name="Trophies">Each player who has won a trophy, and how many.</param>
/// <param name="Recent">The matches played most recently.</param>
/// <param name="Entrants">Everyone who has entered.</param>
/// <param name="Searching">How many are looking for a match now.</param>
/// <param name="Mine">What is the player's own, for a player; null for anyone.</param>
public sealed record LeagueView(
    string Id,
    string Name,
    LeagueSettings Settings,
    bool Open,
    bool Upcoming,
    IReadOnlyList<LeagueRunView> Players,
    IReadOnlyList<LeagueTrophies> Trophies,
    IReadOnlyList<LeagueMatchView> Recent,
    int Entrants,
    int RunsPlayed,
    int Searching,
    LeagueMine? Mine);

/// <param name="Deck">How many cards its main deck has, once built.</param>
public sealed record LeagueRunView(
    string Id,
    string PlayerId,
    string Name,
    int Entry,
    LeagueRunStage Stage,
    int Wins,
    int Losses,
    int Draws,
    string Ending,
    string Status,
    DateTimeOffset Started,
    DateTimeOffset? Ended,
    int Deck,
    IReadOnlyList<LeagueRunMatch> Matches);

/// <param name="Result">"won", "lost", "drawn", or "playing".</param>
public sealed record LeagueRunMatch(string Opponent, string Result, string Detail);

public sealed record LeagueTrophies(string Name, int Trophies, bool Me);

/// <param name="Winner">The winner's name; null for a draw, or a match nobody won.</param>
public sealed record LeagueMatchView(string First, string Second, string? Winner, string Detail, DateTimeOffset? Ended, bool Mine);

/// <param name="Current">The run being played, if there is one.</param>
/// <param name="Previous">Every run over, newest first.</param>
/// <param name="EntriesLeft">How many more runs may be made; null for no limit.</param>
/// <param name="EntryRefusal">Why a new run may not be made now; null when it may.</param>
/// <param name="EditRefusal">Why the current run's deck may not be changed now; null when it may.</param>
/// <param name="SearchingSince">Since when they have looked for a match; null when they are not looking.</param>
/// <param name="Opponent">Who they are playing now, by name; null when they are in no match.</param>
/// <param name="MatchReady">Whether their match's table is open to go to.</param>
/// <param name="Editing">Whether their deck is being changed at its table.</param>
public sealed record LeagueMine(
    LeagueRunView? Current,
    IReadOnlyList<LeagueRunView> Previous,
    int EntriesUsed,
    int? EntriesLeft,
    string? EntryRefusal,
    string? EditRefusal,
    DateTimeOffset? SearchingSince,
    string? Opponent,
    bool MatchReady,
    bool Editing,
    int Trophies);

/// <summary>A run's deck as its own player is shown it on the site.</summary>
/// <param name="Editable">Whether it may be changed now.</param>
public sealed record LeagueDeckView(string Run, int Entry, IReadOnlyList<LeagueDeckCard> Main, IReadOnlyList<LeagueDeckCard> Sideboard, int Count, bool Editable);

/// <param name="Kind">"creature", "spell" or "land", as the list groups them.</param>
public sealed record LeagueDeckCard(string Name, int Count, string ManaCost, int ManaValue, string Kind, string Types);
