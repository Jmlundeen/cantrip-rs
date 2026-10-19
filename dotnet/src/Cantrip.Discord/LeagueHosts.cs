using System.Text.Json;
using System.Text.Json.Serialization;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;

namespace Cantrip.Discord;

/// <summary>
/// A league as whoever runs the bot defines it: one file under <c>leagues/</c> at the top of the checkout, read as the
/// bot starts. The league's event in the data directory is made from it the first time, and takes its name and rules
/// from it every time after - so a league's end can be moved by changing its file.
/// </summary>
public sealed class LeagueDefinition
{
    /// <summary>The league's event id: its address on the site, <c>/leagues/{id}</c>.</summary>
    public string Id { get; set; } = string.Empty;

    public string Name { get; set; } = string.Empty;

    /// <summary>A line on what the league is, for its page.</summary>
    public string Description { get; set; } = string.Empty;

    /// <summary>What the cube is called on the page, where its file name is not what players call it.</summary>
    public string CubeName { get; set; } = string.Empty;

    public LeagueSettings Settings { get; set; } = new();

    private static readonly JsonSerializerOptions Json = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.CamelCase) },
    };

    /// <summary>Every league defined in a directory (<c>*.json</c>), by file name; and what is wrong with any that is not one.</summary>
    public static (List<LeagueDefinition> Leagues, List<string> Problems) LoadAll(string directory)
    {
        List<LeagueDefinition> leagues = [];
        List<string> problems = [];
        if (!Directory.Exists(directory)) return (leagues, problems);
        foreach (string path in Directory.GetFiles(directory, "*.json").Order(StringComparer.Ordinal))
        {
            try
            {
                LeagueDefinition league = JsonSerializer.Deserialize<LeagueDefinition>(File.ReadAllText(path), Json)
                    ?? throw new JsonException("The file is empty.");
                if (!EventStore.ValidId(league.Id)) throw new JsonException($"'{league.Id}' is not a league id: letters, digits and dashes.");
                if (string.IsNullOrWhiteSpace(league.Name)) throw new JsonException("A league has a name.");
                if (league.Settings.Problem() is { } problem) throw new JsonException(problem);
                if (leagues.Any(each => each.Id == league.Id)) throw new JsonException($"Another file defines the league '{league.Id}'.");
                leagues.Add(league);
            }
            catch (Exception error) when (error is JsonException or IOException)
            {
                problems.Add($"League {Path.GetFileName(path)} is not run: {error.Message}");
            }
        }

        return (leagues, problems);
    }
}

public sealed class LeagueHostsOptions
{
    public required string ContentRoot { get; init; }

    public required EventStore Store { get; init; }

    /// <summary>Where a league's quick drafts are hosted, and whose draft ports its match and deck tables are opened on.</summary>
    public required PodHosts Pods { get; init; }

    public required IReadOnlyList<LeagueDefinition> Leagues { get; init; }

    /// <summary>Where the cube lists are, which a quick draft's cube is named from.</summary>
    public IReadOnlyList<string> CubeDirectories { get; init; } = [];

    /// <summary>Each player's picture, by player id, as the league's pages show it beside their name; null for none.</summary>
    public Func<string, string?>? Portraits { get; init; }

    /// <summary>The site's page for a league, which its tables lead back to, by the league's id; null for none.</summary>
    public Func<string, ReturnPage?>? Return { get; init; }

    /// <summary>
    /// Why a player may not start a draft or look for a match now for being somewhere else - in another draft, at a table
    /// filling - or null when they may.
    /// </summary>
    public Func<ulong, string?>? Busy { get; init; }

    public LeagueTiming Timing { get; init; } = new();

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;

    /// <summary>How often the leagues are looked after: players paired, tables opened and closed, drafts seen over.</summary>
    public TimeSpan Every { get; init; } = TimeSpan.FromMilliseconds(250);
}

/// <summary>A league the bot runs, with what its page says about it beside its rules.</summary>
public sealed record HostedLeague(LeagueDefinition Definition, League League)
{
    public string Id => Definition.Id;
}

/// <summary>What a player gets for a seat in a league: the table's address and a fresh ticket to it, or why not.</summary>
public sealed record LeagueSeating(Uri? Address, string? Ticket, string? Refusal)
{
    public static LeagueSeating Refused(string why) => new(null, null, why);
}

/// <summary>
/// The leagues the bot runs, knowing nothing of Discord itself: each league's quick drafts, hosted as pods with bots in
/// every seat but the player's (<see cref="PodHosts"/>); the players looking for a match, paired as time passes; and the
/// tables of every match and every deck being changed, on the draft ports. Everything a league is is in the data
/// directory, and a restarted bot runs every league again from it, matches cut off by the restart played again.
/// </summary>
public sealed class LeagueHosts : IAsyncDisposable
{
    /// <summary>How long a game's table stays up once its players have been sent to the next game's.</summary>
    private static readonly TimeSpan MoveLinger = TimeSpan.FromSeconds(5);

    /// <summary>How long a table stays up once its match is over or its deck saved, for the result and the way back to the league.</summary>
    private static readonly TimeSpan OverLinger = TimeSpan.FromMinutes(2);

    private readonly LeagueHostsOptions _options;
    private readonly List<HostedLeague> _leagues = [];
    private readonly SemaphoreSlim _gate = new(1, 1);
    private readonly Dictionary<object, GameServer> _tables = [];
    private readonly List<(DateTimeOffset At, GameServer Server)> _closing = [];
    private readonly CancellationTokenSource _stop = new();
    private Task _loop = Task.CompletedTask;
    private DateTimeOffset _draftsLooked;

    public LeagueHosts(LeagueHostsOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        _options = options;
    }

    /// <summary>The leagues being run, as the definitions list them.</summary>
    public IReadOnlyList<HostedLeague> Leagues => _leagues;

    public HostedLeague? Find(string id) => _leagues.FirstOrDefault(each => each.Id == id);

    /// <summary>
    /// Runs every league defined: each made in the data directory if it is new, taken up as it was if not. Returns what
    /// stopped any from running.
    /// </summary>
    public IReadOnlyList<string> Start()
    {
        List<string> problems = [];
        foreach (LeagueDefinition definition in _options.Leagues)
        {
            try
            {
                CubeList.Find(definition.Settings.Cube, [.. _options.CubeDirectories]);
                League league = League.Ensure(_options.Store, ContentCache.Get(_options.ContentRoot), definition.Id, definition.Name, definition.Settings,
                    _options.TimeProvider, _options.Timing);
                league.Portraits = _options.Portraits;
                _leagues.Add(new HostedLeague(definition, league));
            }
            catch (Exception error) when (error is IOException or ArgumentException or InvalidOperationException)
            {
                problems.Add($"League {definition.Id} is not run: {error.Message}");
            }
        }

        _loop = Task.Run(RunAsync, CancellationToken.None);
        return problems;
    }

    /// <summary>
    /// A player enters a league: a quick draft of their own is made and hosted, with bots in its other seats, and they
    /// are given their seat at it.
    /// </summary>
    public async Task<LeagueSeating> EnterAsync(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        if (Find(leagueId) is not { } hosted) return LeagueSeating.Refused("There is no such league.");
        if (_options.Busy?.Invoke(player.Id) is { } busy) return LeagueSeating.Refused(busy);
        string id = DraftHost.PlayerId(player);
        BoosterPod pod;
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            if (hosted.League.CanEnter(id) is { } refusal) return LeagueSeating.Refused(refusal);
            LeagueSettings rules = hosted.League.Settings;
            string cube = CubeList.Find(rules.Cube, [.. _options.CubeDirectories]);
            string name = string.IsNullOrWhiteSpace(player.Name) ? "Player" : player.Name.Trim();
            CreatedEvent? created = null;
            hosted.League.Enter(id, name, () =>
            {
                created = BoosterPod.Create(_options.Store, ContentCache.Get(_options.ContentRoot), new BoosterDraftSetup
                {
                    Name = $"{hosted.Definition.Name}: {name}'s draft",
                    Cube = string.IsNullOrWhiteSpace(hosted.Definition.CubeName) ? Path.GetFileNameWithoutExtension(cube) : hosted.Definition.CubeName,
                    CubeCards = CubeList.Load(cube),
                    PickOrder = CubeList.PickOrder(cube),
                    Packs = rules.Packs,
                    PackSize = rules.PackSize,
                    Rounds = 0,
                    MinimumDeck = rules.MinimumDeck,
                    ParentId = hosted.Id,
                    Players = [(id, name), .. Enumerable.Range(2, rules.PodSize - 1).Select(seat => (BotSeats.Id(seat), $"Bot {seat - 1}"))],
                }, _options.TimeProvider);
                return created.Record.Id;
            });
            pod = BoosterPod.Open(_options.Store, created!.Record.Id, ContentCache.Get(_options.ContentRoot), _options.TimeProvider, _options.Pods.Timing);
        }
        catch (Exception error) when (error is ArgumentException or InvalidOperationException or IOException)
        {
            return LeagueSeating.Refused(error.Message);
        }
        finally
        {
            _gate.Release();
        }

        try
        {
            await _options.Pods.HostLeagueDraftAsync(pod, hosted.Id, _options.Return?.Invoke(hosted.Id)).ConfigureAwait(false);
        }
        catch (IOException error)
        {
            // No port to draft at: the draft is called off, and the entry with it.
            pod.Cancel("The draft could not be hosted.");
            hosted.League.DraftCalledOff(pod.EventId);
            return LeagueSeating.Refused(error.Message);
        }

        return await PlayAsync(leagueId, player).ConfigureAwait(false);
    }

    /// <summary>
    /// A fresh ticket to wherever the player's run in a league is being played now: its quick draft, its match's table,
    /// or the table its deck is being changed at.
    /// </summary>
    public async Task<LeagueSeating> PlayAsync(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        if (Find(leagueId) is not { } hosted) return LeagueSeating.Refused("There is no such league.");
        string id = DraftHost.PlayerId(player);
        if (hosted.League.DraftOf(id) is not null)
        {
            PodSeating pod = await _options.Pods.ReturnAsync(player).ConfigureAwait(false);
            return new LeagueSeating(pod.Address, pod.Ticket, pod.Refusal);
        }

        (Uri? address, string? ticket, string? refusal) = hosted.League.Seat(id);
        return new LeagueSeating(address, ticket, refusal);
    }

    /// <summary>A player looks for a match for their run in a league. Null once they are looking; otherwise why not.</summary>
    public string? Search(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        if (Find(leagueId) is not { } hosted) return "There is no such league.";
        if (_options.Busy?.Invoke(player.Id) is { } busy) return busy;
        string id = DraftHost.PlayerId(player);
        if (_leagues.FirstOrDefault(other => other != hosted && (other.League.InMatch(id) || other.League.IsSearching(id))) is { } elsewhere)
            return $"You are already looking for, or playing, a match in {elsewhere.Definition.Name}.";
        return hosted.League.Search(id);
    }

    public string? StopSearching(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        return Find(leagueId) is { } hosted ? hosted.League.StopSearching(DraftHost.PlayerId(player)) : "There is no such league.";
    }

    /// <summary>A page of the site the player has open asked after them: in every league they look for a match in, they go on looking.</summary>
    public void Seen(ulong player)
    {
        string id = DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));
        foreach (HostedLeague hosted in _leagues) hosted.League.Seen(id);
    }

    /// <summary>Opens the table the player's run's deck is changed at, and gives them their seat at it.</summary>
    public async Task<LeagueSeating> EditAsync(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        if (Find(leagueId) is not { } hosted) return LeagueSeating.Refused("There is no such league.");
        string id = DraftHost.PlayerId(player);
        await _gate.WaitAsync().ConfigureAwait(false);
        try
        {
            (LeagueDeckTable? table, string? refusal) = hosted.League.EditDeck(id);
            if (table is null) return LeagueSeating.Refused(refusal ?? "Your deck cannot be changed now.");
            if (table.Address is null)
            {
                (GameServer server, Uri address) = await _options.Pods.Open(table, table.Seats, _options.Return?.Invoke(hosted.Id)).ConfigureAwait(false);
                lock (_tables) _tables[table] = server;
                table.Opened(address);
            }
        }
        catch (IOException error)
        {
            return LeagueSeating.Refused(error.Message);
        }
        finally
        {
            _gate.Release();
        }

        (Uri? at, string? ticket, string? why) = hosted.League.Seat(id);
        return new LeagueSeating(at, ticket, why);
    }

    /// <summary>
    /// A player ends their run in a league before it is over; a run still drafting has its draft called off. Null once
    /// it is ended; otherwise why not.
    /// </summary>
    public async Task<string?> EndRunAsync(string leagueId, DiscordPlayer player)
    {
        ArgumentNullException.ThrowIfNull(player);
        if (Find(leagueId) is not { } hosted) return "There is no such league.";
        (string? draft, string? refusal) = hosted.League.EndRun(DraftHost.PlayerId(player));
        if (refusal is not null) return refusal;
        if (draft is not null) await _options.Pods.CancelAsync(draft, $"{player.Name} ended their run.").ConfigureAwait(false);
        return null;
    }

    /// <summary>
    /// The leagues a player has entered, each as they see it: what the site lists for them, on their page and in the
    /// strip under the menu bar.
    /// </summary>
    public IReadOnlyList<(HostedLeague League, LeagueView View)> Of(ulong player)
    {
        string id = DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));
        return [.. _leagues.Select(hosted => (hosted, hosted.League.View(id)))
            .Where(each => each.Item2.Mine is { } mine && (mine.Current is not null || mine.Previous.Count > 0))];
    }

    /// <summary>Whether a player is playing or looking for a league match, in any league.</summary>
    public bool Busy(ulong player)
    {
        string id = DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));
        return _leagues.Any(hosted => hosted.League.InMatch(id) || hosted.League.IsSearching(id));
    }

    // --- Looking after them. ---------------------------------------------------------------------------------------

    private async Task RunAsync()
    {
        while (!_stop.IsCancellationRequested)
        {
            try
            {
                await LookAfterAsync().ConfigureAwait(false);
            }
            catch (Exception error) when (error is not OperationCanceledException)
            {
                // A table that would not open is tried again on the next pass; one league's fault is not every league's.
                Console.Error.WriteLine($"Looking after the leagues failed: {error.Message}");
            }

            try
            {
                await Task.Delay(_options.Every, _options.TimeProvider, _stop.Token).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }
    }

    /// <summary>
    /// One pass over every league: time passed (pairing whoever is looking), the tables wanted opened and the tables
    /// done with closed after a while, and every run's draft looked at to see whether it is over.
    /// </summary>
    internal async Task LookAfterAsync()
    {
        DateTimeOffset now = _options.TimeProvider.GetUtcNow();
        bool drafts = now - _draftsLooked >= TimeSpan.FromSeconds(2);
        if (drafts) _draftsLooked = now;
        foreach (HostedLeague hosted in _leagues)
        {
            League league = hosted.League;
            league.Tick(now);
            if (drafts) LookAtDrafts(league);

            foreach (MatchGame game in league.TablesToOpen())
            {
                (GameServer server, Uri address) = await _options.Pods.Open(game, game.Seats, _options.Return?.Invoke(hosted.Id)).ConfigureAwait(false);
                lock (_tables) _tables[game] = server;
                game.Opened(address);
            }

            foreach (MatchGame game in league.TablesDone()) Closing(game, now + (game.MatchDecided ? OverLinger : MoveLinger));
            foreach (LeagueDeckTable table in league.DeckTablesDone()) Closing(table, now + OverLinger);
        }

        List<GameServer> closing;
        lock (_tables)
        {
            closing = [.. _closing.Where(each => each.At <= now).Select(each => each.Server)];
            _closing.RemoveAll(each => each.At <= now);
        }

        foreach (GameServer server in closing) await server.DisposeAsync().ConfigureAwait(false);
    }

    private void Closing(object table, DateTimeOffset at)
    {
        lock (_tables)
        {
            if (_tables.Remove(table, out GameServer? server)) _closing.Add((at, server));
        }
    }

    /// <summary>A run's quick draft over with its deck in, or called off, is said so to its league.</summary>
    private void LookAtDrafts(League league)
    {
        foreach (string draft in league.Drafting)
        {
            if (_options.Store.LoadEvent(draft) is not { } record)
            {
                league.DraftCalledOff(draft);
                continue;
            }

            if (record.Stage == EventStage.Cancelled)
            {
                league.DraftCalledOff(draft);
            }
            else if (record.Stage == EventStage.Complete && record.Participants.FirstOrDefault(each => !BotSeats.IsBot(each.PlayerId)) is { } player)
            {
                league.DraftOver(draft, player.Pool, player.Deck);
            }
        }
    }

    public async ValueTask DisposeAsync()
    {
        await _stop.CancelAsync().ConfigureAwait(false);
        await _loop.ConfigureAwait(false);
        List<GameServer> servers;
        lock (_tables)
        {
            servers = [.. _tables.Values, .. _closing.Select(each => each.Server)];
            _tables.Clear();
            _closing.Clear();
        }

        foreach (GameServer server in servers) await server.DisposeAsync().ConfigureAwait(false);
        _stop.Dispose();
        _gate.Dispose();
    }
}
