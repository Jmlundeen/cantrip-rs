using Cantrip.Cards.Model;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// A league: runs entered, their drafts said over, players paired as they look for a match, each match played at a table
/// of its own, and runs ended by a trophy, too many losses or their player. Driven without tables, on a clock the test
/// moves; the drafts are the host's, so a run's pool and deck are given here as a host gives them.
/// </summary>
public sealed class LeagueTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-league-" + Guid.NewGuid().ToString("N"));
    private readonly Clock _clock = new();
    private int _drafts;

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private static Cantrip.Datasets.ContentLibrary Content => ContentCache.Get(RepoRoot.ContentRoot);

    private sealed class Clock : TimeProvider
    {
        private DateTimeOffset _now = new(2026, 10, 19, 18, 0, 0, TimeSpan.Zero);
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan by) => _now += by;
    }

    /// <summary>Twenty-three spells of the loaded cards, as a quick draft's pool, built with seventeen Mountains.</summary>
    private static readonly Lazy<List<string>> Pool = new(() => [.. Content.Cards.InOrder()
        .Where(card => !card.IsLand).Take(23).Select(card => card.Id.Value)]);

    private static EventDeck Built => new() { Main = [.. Enumerable.Range(0, 23)], Basics = new() { ["Mountain"] = 17 }, Submitted = true };

    private League Make(Action<LeagueSettings>? rules = null)
    {
        var settings = new LeagueSettings { Cube = "LSVCube" };
        rules?.Invoke(settings);
        return League.Ensure(new EventStore(_data), Content, "test-league", "Test League", settings, _clock);
    }

    /// <summary>A player enters and drafts: the run plays from now on.</summary>
    private void Enter(League league, string player)
    {
        string draft = $"draft-{++_drafts}";
        league.Enter(player, char.ToUpperInvariant(player[0]) + player[1..], () => draft);
        league.DraftOver(draft, Pool.Value, Built);
    }

    /// <summary>Two players look for a match and are paired; its first game's table is opened.</summary>
    private MatchGame Pair(League league, string first, string second)
    {
        Assert.Null(league.Search(first));
        _clock.Advance(TimeSpan.FromSeconds(1));
        Assert.Null(league.Search(second));
        league.Tick(_clock.GetUtcNow());
        MatchGame game = Assert.Single(league.TablesToOpen());
        game.Opened(new Uri($"ws://localhost:{9000 + _drafts}/game"));
        return game;
    }

    /// <summary>
    /// A game's deal, as its table records it when it deals (<see cref="MatchGame.Deal"/>) - without the rules engine,
    /// which nothing here plays - so that how it ends is the match's.
    /// </summary>
    private void Dealt(MatchGame game) =>
        game.Match.Games.Add(new GameRecord { Number = game.Number, Seed = 1, Started = _clock.GetUtcNow() });

    /// <summary>A match between two players, won by the first.</summary>
    private void Play(League league, string winner, string loser)
    {
        MatchGame game = Pair(league, winner, loser);
        int seat = game.PlayerAt(1) == winner ? 1 : 2;
        Dealt(game);
        game.GameEnded(abandoned: false, [seat], "won");
        _clock.Advance(TimeSpan.FromSeconds(10));
        game.Tick(_clock.GetUtcNow());
        Assert.Contains(game, league.TablesDone());
    }

    private static LeagueRunView Current(League league, string player) => league.View(player).Mine!.Current!;

    [Fact]
    public void Players_looking_for_a_match_are_paired_and_play_it_at_a_table_of_its_own()
    {
        League league = Make();
        Enter(league, "ann");
        Enter(league, "ben");
        Assert.Equal("Between matches", Current(league, "ann").Status);

        Assert.Null(league.Search("ann"));
        Assert.Equal("Looking for a match", Current(league, "ann").Status);
        league.Tick(_clock.GetUtcNow());
        Assert.Empty(league.TablesToOpen());

        Assert.Null(league.Search("ben"));
        league.Tick(_clock.GetUtcNow());
        MatchGame game = Assert.Single(league.TablesToOpen());
        Assert.Equal("Test League: Ann vs Ben", game.Name);
        Assert.False(league.IsSearching("ann"));
        Assert.Equal("Ben", league.View("ann").Mine!.Opponent!.Name);
        Assert.False(league.View("ann").Mine!.MatchReady);
        Assert.Equal("Your match's table is opening. Try again in a moment.", league.Seat("ann").Refusal);

        game.Opened(new Uri("ws://localhost:9100/game"));
        (Uri? address, string? ticket, string? refusal) = league.Seat("ben");
        Assert.Null(refusal);
        Assert.Equal(new Uri("ws://localhost:9100/game"), address);
        Assert.Equal(2, game.SeatFor(ticket!));
        Assert.True(league.View("ben").Mine!.MatchReady);
        Assert.True(game.ReadyToDeal);
        Assert.Equal("You are already in a match.", league.Search("ann"));
        Assert.Equal("Your match is being played: finish it first.", league.EndRun("ann").Refusal);

        Dealt(game);
        game.GameEnded(abandoned: false, [2], "Ben won.");
        Assert.Equal((0, 1), (Current(league, "ann").Wins, Current(league, "ann").Losses));
        Assert.Equal((1, 0), (Current(league, "ben").Wins, Current(league, "ben").Losses));
        Assert.Equal([new LeagueRunMatch(new PlayerFace("Ann"), "won", "Ben won 1-0")], Current(league, "ben").Matches);
        Assert.Equal("Ben", Assert.Single(league.View(null).Recent).Winner);

        // Nobody is sent anywhere: the result stays up, and leads back to the league's page.
        _clock.Advance(TimeSpan.FromSeconds(10));
        game.Tick(_clock.GetUtcNow());
        Assert.Empty(game.TakeVacated());
        Assert.Contains(game, league.TablesDone());
        Assert.Null(league.Search("ann"));
    }

    [Fact]
    public void Seven_wins_earn_a_trophy_three_losses_end_a_run_and_a_player_enters_again_as_often_as_they_like()
    {
        League league = Make();
        Enter(league, "ann");
        for (int win = 1; win <= 7; win++)
        {
            if (league.View("ben").Mine!.Current is null) Enter(league, "ben");
            Play(league, "ann", "ben");
        }

        LeagueMine ann = league.View("ann").Mine!;
        Assert.Null(ann.Current);
        LeagueRunView trophy = Assert.Single(ann.Previous);
        Assert.Equal((7, 0, LeagueEndings.Trophy, "Trophy"), (trophy.Wins, trophy.Losses, trophy.Ending, trophy.Status));
        Assert.Equal(1, ann.Trophies);
        Assert.Equal([new LeagueTrophies(new PlayerFace("Ann"), 1, true)], league.View("ann").Trophies);

        // Ben's first two runs went 0-3; the third is 0-1 and still going.
        LeagueMine ben = league.View("ben").Mine!;
        Assert.Equal([(0, 3, "Out"), (0, 3, "Out")], ben.Previous.Select(run => (run.Wins, run.Losses, run.Status)));
        Assert.Equal((3, 0, 1), (ben.Current!.Entry, ben.Current.Wins, ben.Current.Losses));
        Assert.Null(ben.EntriesLeft);

        // Ann may start a new run, and her standings show only the one being played.
        Assert.Null(league.CanEnter("ann"));
        Assert.Equal(["Ben"], league.View(null).Players.Select(run => run.Name));
    }

    [Fact]
    public void A_run_ended_by_its_player_counts_its_losses_as_maxed_and_uses_up_its_entry()
    {
        League league = Make(rules => rules.MaxEntries = 2);
        Enter(league, "ann");
        Enter(league, "ben");
        Play(league, "ann", "ben");

        Assert.Null(league.EndRun("ann").Refusal);
        LeagueRunView ended = Assert.Single(league.View("ann").Mine!.Previous);
        Assert.Equal((1, 3, LeagueEndings.Retired, "Ended early"), (ended.Wins, ended.Losses, ended.Ending, ended.Status));
        Assert.Equal("You have no run going in this league.", league.EndRun("ann").Refusal);
        Assert.Equal(1, league.View("ann").Mine!.EntriesLeft);

        Enter(league, "ann");
        Assert.Null(league.EndRun("ann").Refusal);
        Assert.Equal("You have had all 2 of your entries into this league.", league.CanEnter("ann"));
        Assert.Throws<InvalidOperationException>(() => league.Enter("ann", "Ann", () => "draft-x"));

        // One still drafting is ended, and the host is told which draft to call off.
        Enter(league, "cat");
        Assert.Null(league.EndRun("cat").Refusal);
        league.Enter("cat", "Cat", () => "cats-draft");
        Assert.Equal("cats-draft", league.DraftOf("cat"));
        Assert.Equal(("cats-draft", (string?)null), league.EndRun("cat"));
        Assert.Equal(0, league.View("cat").Mine!.Previous[0].Wins);
    }

    [Fact]
    public void A_draft_called_off_is_no_entry()
    {
        League league = Make(rules => rules.MaxEntries = 1);
        league.Enter("ann", "Ann", () => "anns-draft");
        Assert.Equal("You already have a run going in this league.", league.CanEnter("ann"));
        Assert.Equal("Your draft isn't over yet.", league.Search("ann"));
        Assert.Equal(["anns-draft"], league.Drafting);

        league.DraftCalledOff("anns-draft");
        Assert.Empty(league.Drafting);
        Assert.Empty(league.View("ann").Mine!.Previous);
        Assert.Null(league.CanEnter("ann"));
    }

    [Fact]
    public void A_player_not_seen_for_a_while_stops_looking_for_a_match()
    {
        League league = Make();
        Enter(league, "ann");
        Assert.Null(league.Search("ann"));

        _clock.Advance(TimeSpan.FromSeconds(30));
        league.Seen("ann");
        _clock.Advance(TimeSpan.FromSeconds(30));
        league.Tick(_clock.GetUtcNow());
        Assert.True(league.IsSearching("ann"));

        _clock.Advance(TimeSpan.FromSeconds(45));
        league.Tick(_clock.GetUtcNow());
        Assert.False(league.IsSearching("ann"));
        Assert.Equal("You are not looking for a match.", league.StopSearching("ann"));
    }

    [Fact]
    public void Free_pairing_pairs_whoever_has_looked_longest_never_a_player_with_themselves_and_repeats_only_when_allowed()
    {
        DateTimeOffset now = _clock.GetUtcNow();
        LeagueSearcher ann = new("ann", "run-1", 0, 0, now);
        LeagueSearcher ben = new("ben", "run-2", 3, 1, now.AddSeconds(1));
        LeagueSearcher cat = new("cat", "run-3", 0, 2, now.AddSeconds(2));
        var free = new LeagueSettings { Cube = "LSVCube" };

        Assert.Equal([(ann, ben)], LeaguePairing.Pair([cat, ben, ann], free, (_, _) => false, now));
        Assert.Empty(LeaguePairing.Pair([ann, ann with { RunId = "run-9" }], free, (_, _) => false, now));

        // Ann's run has played Ben: with no repeats she is paired with Cat instead, and Ben waits.
        bool AnnMetBen(string run, string player) => run == "run-1" && player == "ben";
        Assert.Equal([(ann, ben)], LeaguePairing.Pair([ann, ben, cat], free, AnnMetBen, now));
        var fresh = new LeagueSettings { Cube = "LSVCube", AllowRepeatPairings = false };
        Assert.Equal([(ann, cat)], LeaguePairing.Pair([ann, ben, cat], fresh, AnnMetBen, now));
    }

    [Fact]
    public void A_league_that_pairs_by_record_is_not_played_yet()
    {
        Assert.Contains("not played yet", new LeagueSettings { Cube = "LSVCube", Pairing = LeaguePairingMode.RecordLoose }.Problem());
        Assert.Null(new LeagueSettings { Cube = "LSVCube" }.Problem());
        Assert.Throws<ArgumentException>(() => Make(rules => rules.Pairing = LeaguePairingMode.RecordStrict));
    }

    [Fact]
    public void A_deck_is_changed_at_a_table_of_its_own_between_matches_and_kept_as_each_change_is_made()
    {
        League league = Make();
        Enter(league, "ann");
        Enter(league, "ben");
        Play(league, "ann", "ben");

        (LeagueDeckTable? table, string? refusal) = league.EditDeck("ann");
        Assert.Null(refusal);
        Assert.Equal("Changing their deck", Current(league, "ann").Status);
        Assert.Equal("You are changing your deck: save it first.", league.Search("ann"));
        table!.Opened(new Uri("ws://localhost:9200/game"));
        (_, string? ticket, _) = league.Seat("ann");
        Assert.Equal(1, table.SeatFor(ticket!));
        table.Seated(1);

        var deck = (DeckBuildingMessage)table.ViewFor(1)!;
        Assert.Equal((23, 40, "Your deck for Test League (1-0)"), (deck.Pool.Count, deck.Minimum, deck.Title));
        Assert.Equal(TablePhase.DeckBuilding, table.Phase);

        // A change is kept as it is made; one too small to play is not saved as the deck.
        Assert.Null(table.Receive(1, new BuildDeckMessage { Change = 1, Main = [.. Enumerable.Range(0, 22)], Basics = [new BasicLandCount { Name = "Mountain", Count = 18 }] }));
        Assert.Equal(18, new EventStore(_data).LoadEvent("test-league")!.League!.Runs[0].Deck.Basics["Mountain"]);
        Assert.NotNull(table.Receive(1, new BuildDeckMessage { Change = 2, Main = [0], Basics = [], Submit = true }));

        Assert.Null(table.Receive(1, new BuildDeckMessage { Change = 3, Main = [.. Enumerable.Range(0, 22)], Basics = [new BasicLandCount { Name = "Mountain", Count = 18 }], Submit = true }));
        Assert.Equal(TablePhase.Finished, table.Phase);
        Assert.Contains("saved", table.Detail);
        Assert.Contains(table, league.DeckTablesDone());
        Assert.Null(league.Search("ann"));

        LeagueDeckView shown = league.DeckOf("ann", "run-1")!;
        Assert.Equal(40, shown.Count);
        Assert.Equal(18, shown.Main.Single(card => card.Name == "Mountain").Count);
        Assert.Single(shown.Sideboard);
        Assert.Null(league.DeckOf("ben", "run-1"));
    }

    [Fact]
    public void A_league_that_keeps_its_decks_once_played_refuses_a_change_after_the_first_match()
    {
        League league = Make(rules => rules.DeckEditing = LeagueDeckEditing.UntilFirstMatch);
        Enter(league, "ann");
        Enter(league, "ben");
        LeagueDeckTable table = league.EditDeck("ann").Table!;
        Assert.Null(table.Receive(1, new BuildDeckMessage { Change = 1, Main = [.. Enumerable.Range(0, 23)], Basics = [new BasicLandCount { Name = "Mountain", Count = 17 }], Submit = true }));
        Assert.Contains(table, league.DeckTablesDone());

        Play(league, "ann", "ben");
        Assert.Equal("In this league a deck is kept as it is once its first match is played.", league.EditDeck("ann").Refusal);
        Assert.False(league.DeckOf("ann", "run-1")!.Editable);
    }

    [Fact]
    public void A_deck_table_its_player_never_comes_to_closes_with_the_deck_as_it_was()
    {
        League league = Make();
        Enter(league, "ann");
        LeagueDeckTable table = league.EditDeck("ann").Table!;
        _clock.Advance(TimeSpan.FromMinutes(2));
        league.Tick(_clock.GetUtcNow());
        Assert.Equal(TablePhase.Finished, table.Phase);
        Assert.Contains(table, league.DeckTablesDone());
        Assert.Null(league.Search("ann"));
    }

    [Fact]
    public void A_best_of_three_match_plays_its_next_game_at_a_table_of_its_own_after_sideboarding()
    {
        League league = Make(rules => rules.GamesToWin = 2);
        Enter(league, "ann");
        Enter(league, "ben");
        MatchGame first = Pair(league, "ann", "ben");
        Assert.Equal("Test League: Ann vs Ben, game 1", first.Name);
        Dealt(first);
        first.GameEnded(abandoned: false, [1], "won");
        Assert.Equal((0, 0), (Current(league, "ann").Wins, Current(league, "ann").Losses));

        MatchGame second = Assert.Single(league.TablesToOpen());
        second.Opened(new Uri("ws://localhost:9300/game"));
        Assert.Equal(TablePhase.DeckBuilding, second.Phase);

        // The first game's players are sent on to the second's.
        _clock.Advance(TimeSpan.FromSeconds(10));
        first.Tick(_clock.GetUtcNow());
        Assert.Equal(2, first.TakeVacated().Count(vacated => vacated.Farewell is TableMoveMessage));
    }

    /// <summary>
    /// Sideboarding for the next game of a match (CR 100.4) submits the deck again, and the server checks it again before that
    /// game is dealt: too few cards is refused, and so is a companion that has no companion (CR 702.139a); the deck as the
    /// rules allow is taken.
    /// </summary>
    [Fact]
    public void Sideboarding_for_the_next_game_checks_the_deck_again_its_companion_with_it()
    {
        League league = Make(rules => rules.GamesToWin = 2);
        Enter(league, "ann");
        Enter(league, "ben");
        MatchGame first = Pair(league, "ann", "ben");
        Dealt(first);
        first.GameEnded(abandoned: false, [1], "won");
        MatchGame second = Assert.Single(league.TablesToOpen());
        second.Opened(new Uri("ws://localhost:9302/game"));

        BuildDeckMessage Deck(int change, int spells, int companion) => new()
        {
            Change = change, Main = [.. Enumerable.Range(0, spells)], Basics = [new BasicLandCount { Name = "Mountain", Count = 18 }], Submit = true, Companion = companion,
        };
        Assert.Equal("A deck needs at least 40 cards; this one has 39.", second.Receive(1, Deck(1, 21, -1)));
        Assert.Contains("has no companion ability, so it can't be the deck's companion", second.Receive(1, Deck(2, 22, 22)), StringComparison.Ordinal);
        Assert.Null(second.Receive(1, Deck(3, 22, -1)));
    }

    /// <summary>
    /// A player whose match clock runs out loses the whole match, whatever the score (docs/events.md, "Clocks"): Ann, a game
    /// up in a best of three, runs out of time in game two, and Ben wins the match - its game two too - with no third game,
    /// and their runs count it as his win and her loss.
    /// </summary>
    [Fact]
    public void A_player_up_a_game_who_runs_out_of_time_loses_the_match()
    {
        League league = Make(rules => rules.GamesToWin = 2);
        Enter(league, "ann");
        Enter(league, "ben");
        MatchGame first = Pair(league, "ann", "ben");
        int ann = first.PlayerAt(1) == "ann" ? 1 : 2;
        Dealt(first);
        first.GameEnded(abandoned: false, [ann], "won");
        MatchGame second = Assert.Single(league.TablesToOpen());
        second.Opened(new Uri("ws://localhost:9301/game"));
        Dealt(second);

        second.ClockRanOut(ann);
        second.GameEnded(abandoned: false, [3 - ann], "Ann ran out of time.");

        MatchRecord match = second.Match;
        Assert.True(match.Complete);
        Assert.Equal(["ben"], match.Winners);
        Assert.Equal("ann", match.TimedOut);
        Assert.Equal(["ben"], match.Games.Single(game => game.Number == 2).Winners);
        Assert.False(second.ReadyToDeal);
        Assert.Empty(league.TablesToOpen());
        Assert.Equal((0, 1), (Current(league, "ann").Wins, Current(league, "ann").Losses));
        Assert.Equal((1, 0), (Current(league, "ben").Wins, Current(league, "ben").Losses));
    }

    /// <summary>
    /// Both players' clocks run out at the same look: neither is preferred, so the game and the match are drawn, and no further
    /// game is dealt.
    /// </summary>
    [Fact]
    public void Both_players_out_of_time_together_draw_the_match()
    {
        League league = Make(rules => rules.GamesToWin = 2);
        Enter(league, "ann");
        Enter(league, "ben");
        MatchGame game = Pair(league, "ann", "ben");
        Dealt(game);

        game.ClocksRanOut([1, 2]);
        game.GameEnded(abandoned: false, [], "Ann and Ben ran out of time.");

        MatchRecord match = game.Match;
        Assert.True(match.Complete);
        Assert.Empty(match.Winners);
        Assert.Null(match.TimedOut);
        Assert.Equal("draw", Assert.Single(match.Games).Ending);
        Assert.False(game.ReadyToDeal);
        Assert.Empty(league.TablesToOpen());
    }

    [Fact]
    public void A_match_cut_off_by_a_restart_is_played_again_and_the_runs_are_as_they_were()
    {
        League league = Make();
        Enter(league, "ann");
        Enter(league, "ben");
        Play(league, "ann", "ben");
        Pair(league, "ben", "ann");

        League again = League.Open(new EventStore(_data), "test-league", Content, _clock);
        MatchGame game = Assert.Single(again.TablesToOpen());
        Assert.Equal("Test League: Ben vs Ann", game.Name);
        Assert.Equal((1, 0), (Current(again, "ann").Wins, Current(again, "ann").Losses));
        Assert.Equal("Playing a match", Current(again, "ben").Status);
    }

    [Fact]
    public void A_league_ended_ends_the_runs_between_matches_and_takes_no_more_entries()
    {
        League league = Make(rules => rules.Ends = _clock.GetUtcNow().AddDays(1));
        Enter(league, "ann");
        Assert.True(league.View(null).Open);

        _clock.Advance(TimeSpan.FromDays(1));
        league.Tick(_clock.GetUtcNow());
        LeagueView over = league.View("ann");
        Assert.False(over.Open);
        Assert.Equal("League over", Assert.Single(over.Mine!.Previous).Status);
        Assert.Equal("The league is over.", league.CanEnter("ann"));
        Assert.Equal("The league is over.", league.Search("ann"));
    }
}
