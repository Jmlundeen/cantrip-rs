using System.Text.Json;
using Cantrip.Core.Random;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// The server's status page (<see cref="EventStatusBoard"/>): every event being played and every one over, each with
/// its draft, pairings and standings - and never anything a player keeps from the others.
/// </summary>
public sealed class EventStatusTests : IDisposable
{
    private static readonly DateTimeOffset Now = new(2026, 10, 5, 12, 0, 0, TimeSpan.Zero);

    private readonly string _root = Path.Combine(Path.GetTempPath(), "cantrip-status-" + Guid.NewGuid().ToString("N"));
    private readonly EventStore _store;

    public EventStatusTests() => _store = new EventStore(_root);

    public void Dispose()
    {
        if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true);
    }

    private static List<string> Cards(int count) => [.. Enumerable.Range(1, count).Select(i => $"secret-card-{i}")];

    /// <summary>An event with everything a player keeps to themselves filled in: their pool, deck, tickets, the seed, the host's notes.</summary>
    private EventRecord Event(string id, string kind, string format, string cube, EventStage stage, DateTimeOffset updated, params string[] players)
    {
        var record = new EventRecord
        {
            Id = id,
            Kind = kind,
            Name = $"{cube} {format}",
            Stage = stage,
            Created = updated.AddHours(-1),
            Updated = updated,
            Seed = 9876543210123,
            Draft = new DraftSettings { Format = format, Cube = cube, Packs = 3, PackSize = 15, Rounds = format == DraftFormats.Booster ? 3 : 0 },
            HostData = { ["discord.message"] = "secret-host-note" },
        };
        for (int seat = 1; seat <= players.Length; seat++)
        {
            record.Participants.Add(new EventParticipant
            {
                PlayerId = $"discord-secret-{seat}",
                Name = players[seat - 1],
                Seat = seat,
                Seated = true,
                TicketHashes = [$"secret-ticket-hash-{seat}"],
                Pool = [$"secret-pool-{seat}-a", $"secret-pool-{seat}-b"],
                Deck = new EventDeck { Main = [0, 1], Basics = { ["Island"] = 7 }, Submitted = seat == 1 },
            });
        }

        return record;
    }

    private void Save(EventRecord record) => _store.SaveEvent(record);

    private static string Json(object value) => JsonSerializer.Serialize(value, new JsonSerializerOptions(JsonSerializerDefaults.Web));

    [Fact]
    public void Events_being_played_are_active_and_events_over_are_history()
    {
        Save(Event("grid-live", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.Drafting, Now, "Alice", "Bob"));
        Save(Event("winston-done", EventKinds.WinstonDraft, DraftFormats.Winston, "LSVCube", EventStage.Complete, Now.AddHours(-2), "Carol", "Dan"));
        Save(Event("booster-off", EventKinds.BoosterDraft, DraftFormats.Booster, "Vintage", EventStage.Cancelled, Now.AddHours(-3), "Eve", "Finn"));

        var board = new EventStatusBoard(_store);

        StatusEvent live = Assert.Single(board.Active().Events);
        Assert.Equal("grid-live", live.Id);
        Assert.Equal(["Alice", "Bob"], live.Players.Select(player => player.Name));
        Assert.Equal("LSVCube", live.Cube);

        StatusHistory history = board.History(new StatusQuery());
        Assert.Equal(["winston-done", "booster-off"], history.Events.Select(summary => summary.Id));
        Assert.Equal(["LSVCube", "Vintage"], history.Cubes);
        Assert.Equal(["Carol", "Dan", "Eve", "Finn"], history.Players);
    }

    [Fact]
    public void An_event_the_host_is_not_running_is_not_being_played()
    {
        // One the host runs, and one left live in the directory by a draft played locally.
        Save(Event("hosted", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.Drafting, Now, "Alice", "Bob"));
        Save(Event("left", EventKinds.BoosterDraft, DraftFormats.Booster, "LSVCube", EventStage.Playing, Now.AddDays(-2), "Carol", "Dan"));

        var board = new EventStatusBoard(_store, hosting: () => ["hosted"]);

        Assert.Equal(["hosted"], board.Active().Events.Select(summary => summary.Id));
        StatusEvent left = Assert.Single(board.History(new StatusQuery()).Events);
        Assert.Equal("left", left.Id);
        Assert.False(left.Live);
        Assert.Equal("Playing", left.Stage);
        Assert.False(board.Detail("left")!.Event.Live);
        Assert.Equal(["left"], board.History(new StatusQuery(Outcome: StatusOutcomes.Unfinished)).Events.Select(summary => summary.Id));
        Assert.Empty(board.History(new StatusQuery(Outcome: StatusOutcomes.Complete)).Events);
    }

    [Fact]
    public void The_history_is_filtered_by_when_kind_cube_player_and_outcome()
    {
        Save(Event("a", EventKinds.WinstonDraft, DraftFormats.Winston, "LSVCube", EventStage.Complete, Now.AddHours(-2), "Carol", "Dan"));
        Save(Event("b", EventKinds.BoosterDraft, DraftFormats.Booster, "Vintage", EventStage.Cancelled, Now.AddDays(-3), "Eve", "Carol"));
        Save(Event("c", EventKinds.GridDraft, DraftFormats.Grid, "lsvcube", EventStage.Complete, Now.AddDays(-10), "Finn", "Gus"));
        var board = new EventStatusBoard(_store);

        string[] Ids(StatusQuery query) => [.. board.History(query).Events.Select(summary => summary.Id)];

        Assert.Equal(["a"], Ids(new StatusQuery(From: Now.AddDays(-1))));
        Assert.Equal(["b"], Ids(new StatusQuery(From: Now.AddDays(-7), To: Now.AddDays(-1))));
        Assert.Equal(["c"], Ids(new StatusQuery(Kind: EventKinds.GridDraft)));
        Assert.Equal(["a", "c"], Ids(new StatusQuery(Cube: "LSVCUBE")));
        Assert.Equal(["a", "b"], Ids(new StatusQuery(Player: "car")));
        Assert.Equal(["b"], Ids(new StatusQuery(Outcome: "Cancelled")));
        Assert.Equal(["a"], Ids(new StatusQuery(Limit: 1)));
        Assert.Equal(3, board.History(new StatusQuery(Limit: 1)).Total);
    }

    [Fact]
    public void Nothing_a_player_keeps_from_the_others_is_shown()
    {
        // A booster draft under way, with packs in front of the seats; a grid draft with its grid laid out; a Winston
        // draft with cards in its piles; and a game being played, with decks submitted.
        EventRecord booster = Event("booster", EventKinds.BoosterDraft, DraftFormats.Booster, "LSVCube", EventStage.Drafting, Now, "Alice", "Bob");
        Save(booster);
        BoosterDraft packs = BoosterDraft.Start(Cards(90), 2, 3, 15, new Pcg32Random(3), 4242, Now);
        packs.Take(1, 0, 0, Now);
        _store.SaveDraft(booster.Id, packs);

        EventRecord grid = Event("grid", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.Drafting, Now, "Carol", "Dan");
        Save(grid);
        _store.SaveDraft(grid.Id, GridDraft.Start(Cards(27), 3, 1, new Pcg32Random(5)));

        EventRecord winston = Event("winston", EventKinds.WinstonDraft, DraftFormats.Winston, "LSVCube", EventStage.Drafting, Now, "Eve", "Finn");
        Save(winston);
        _store.SaveDraft(winston.Id, WinstonDraft.Start(Cards(45), 3, 2, 1, new Pcg32Random(7)));

        EventRecord playing = Event("playing", EventKinds.BoosterDraft, DraftFormats.Booster, "LSVCube", EventStage.Playing, Now, "Gus", "Hal");
        playing.Round = 1;
        playing.Matches.Add(new MatchRecord
        {
            Id = "r1t1", Round = 1, Table = 1, Players = ["discord-secret-1", "discord-secret-2"], GamesToWin = 2,
            Decks = { ["discord-secret-1"] = new EventDeck { Main = [0] } },
            Games =
            {
                new GameRecord { Number = 1, Seed = 1234567890987, Started = Now },
                new GameRecord { Number = 2, Started = Now, Ended = Now, Ending = "abandoned", Detail = $"{DraftPod.DealFailedLine}: secret-pool-2-b is not a loaded card." },
            },
            Detail = $"{DraftPod.DealFailedLine}: secret-pool-2-b is not a loaded card.",
        });
        Save(playing);

        // A game that could not be dealt says why, naming a card from a pool.
        EventRecord failed = Event("failed", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.DeckBuilding, Now, "Ivy", "Jo");
        failed.Detail = $"{DraftPod.DealFailedLine}: secret-pool-1-a is not a loaded card.";
        Save(failed);

        var board = new EventStatusBoard(_store);
        string shown = Json(board.Active()) + Json(board.History(new StatusQuery()))
            + string.Concat(new[] { "booster", "grid", "winston", "playing", "failed" }.Select(id => Json(board.Detail(id)!)));

        Assert.DoesNotContain("secret", shown, StringComparison.Ordinal);
        Assert.DoesNotContain("9876543210123", shown, StringComparison.Ordinal);
        Assert.DoesNotContain("1234567890987", shown, StringComparison.Ordinal);
        Assert.DoesNotContain("4242", shown, StringComparison.Ordinal);
        Assert.DoesNotContain("Island", shown, StringComparison.Ordinal);
    }

    [Fact]
    public void A_booster_draft_shows_the_pack_and_each_seat_s_progress()
    {
        EventRecord record = Event("booster", EventKinds.BoosterDraft, DraftFormats.Booster, "LSVCube", EventStage.Drafting, Now, "Alice", "Bob");
        Save(record);
        BoosterDraft draft = BoosterDraft.Start(Cards(90), 2, 3, 15, new Pcg32Random(3), 4242, Now);
        draft.Take(1, 0, 0, Now);
        _store.SaveDraft(record.Id, draft);

        StatusEventDetail detail = new EventStatusBoard(_store).Detail("booster")!;

        StatusDraft shown = detail.Draft!;
        Assert.Equal(1, shown.Pack);
        Assert.Equal(3, shown.Packs);
        Assert.Equal(1, shown.Direction);
        Assert.Equal(90, shown.Cards);
        Assert.Equal([1, 0], shown.Seats.Select(seat => seat.Drafted));
        Assert.Equal([0, 2], shown.Seats.Select(seat => seat.Queued));
        Assert.Equal("Drafting pack 1 of 3", detail.Event.Progress);
    }

    [Fact]
    public void Decks_being_built_show_who_has_submitted_and_nothing_else()
    {
        Save(Event("building", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.DeckBuilding, Now, "Alice", "Bob"));

        StatusEvent shown = new EventStatusBoard(_store).Detail("building")!.Event;

        Assert.Equal([true, false], shown.Players.Select(player => player.Submitted));
        Assert.Equal("Building decks: 1 of 2 submitted", shown.Progress);
    }

    [Fact]
    public void Swiss_rounds_show_their_pairings_results_and_standings()
    {
        EventRecord record = Event("pod", EventKinds.BoosterDraft, DraftFormats.Booster, "LSVCube", EventStage.Complete, Now, "Alice", "Bob", "Carol", "Dan");
        string Id(int seat) => $"discord-secret-{seat}";
        MatchRecord Match(int round, int table, int first, int second, int winner, int lost = 0) => new()
        {
            Id = $"r{round}t{table}", Round = round, Table = table, Players = [Id(first), Id(second)], Complete = true, GamesToWin = 2,
            Winners = [Id(winner)],
            Games =
            [
                .. Enumerable.Range(1, 2).Select(number => new GameRecord { Number = number, Started = Now, Ended = Now, Ending = "won", Winners = [Id(winner)] }),
                .. Enumerable.Range(3, lost).Select(number => new GameRecord { Number = number, Started = Now, Ended = Now, Ending = "won", Winners = [Id(winner == first ? second : first)] }),
            ],
        };
        record.Matches.Add(Match(1, 1, 1, 3, 1));
        record.Matches.Add(Match(1, 2, 2, 4, 4, lost: 1));
        record.Matches.Add(Match(2, 1, 1, 4, 1));
        record.Matches.Add(Match(2, 2, 2, 3, 3));
        Save(record);

        StatusEventDetail detail = new EventStatusBoard(_store).Detail("pod")!;

        Assert.Equal("Alice", detail.Event.Winner);
        Assert.Equal("Alice", detail.Standings[0].Name);
        Assert.Equal(["Carol", "Dan"], detail.Standings.Skip(1).Take(2).Select(standing => standing.Name).Order());
        Assert.Equal("Bob", detail.Standings[3].Name);
        Assert.Equal(6, detail.Standings[0].Points);
        Assert.Equal([1, 2], detail.Rounds.Select(round => round.Round));
        StatusMatch second = detail.Rounds[0].Matches[1];
        Assert.Equal(["Bob", "Dan"], second.Players.Select(player => player.Name));
        Assert.Equal([1, 2], second.Players.Select(player => player.Wins));
        Assert.Equal("Dan", second.Winner);
    }

    [Fact]
    public void An_event_changed_is_read_again()
    {
        EventRecord record = Event("grid", EventKinds.GridDraft, DraftFormats.Grid, "LSVCube", EventStage.Drafting, Now, "Alice", "Bob");
        Save(record);
        var board = new EventStatusBoard(_store);
        Assert.Equal("Drafting", board.Detail("grid")!.Event.Stage);

        record.Stage = EventStage.Complete;
        Save(record);
        File.SetLastWriteTimeUtc(Path.Combine(_root, "events", "grid", "event.json"), DateTime.UtcNow.AddMinutes(1));

        Assert.Equal("Complete", board.Detail("grid")!.Event.Stage);
        Assert.Empty(board.Active().Events);
    }

    [Fact]
    public void A_host_s_other_tables_are_shown_with_its_events()
    {
        StatusTable forming = new(StatusTableKinds.Forming, DraftFormats.Grid, "LSVCube", "Forming", ["Alice"], 2, 0, Now, "1 of 2 seated");
        var board = new EventStatusBoard(null, () => [forming]);

        StatusActive active = board.Active();

        Assert.Empty(active.Events);
        Assert.Equal([forming], active.Tables);
        Assert.Empty(board.History(new StatusQuery()).Events);
        Assert.Null(board.Detail("anything"));
    }

    [Fact]
    public void An_id_that_could_reach_outside_the_data_directory_names_no_event()
    {
        Assert.Null(new EventStatusBoard(_store).Detail("../players"));
    }

    [Fact]
    public void The_status_page_is_built_into_the_host()
    {
        Assert.Contains("<title>Cantrip - Server Status</title>", StatusSite.Html, StringComparison.Ordinal);
    }
}
