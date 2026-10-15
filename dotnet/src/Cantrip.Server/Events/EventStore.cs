using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.RegularExpressions;

namespace Cantrip.Server.Events;

/// <summary>
/// Events and players, kept as JSON files in a data directory the host names (<c>--data</c>, or <c>CANTRIP_DATA</c>):
/// <code>
/// players/{player}.json             who they are and the events they are in
/// events/{event}/event.json         the event (EventRecord)
/// events/{event}/draft.json         a draft's own state while it runs (WinstonDraft)
/// </code>
/// One directory per event, so a league of hundreds of drafts is hundreds of small directories rather than one file
/// that grows, and an event can be copied, archived or looked at on its own. Every write goes to a temporary file
/// first and is moved over the old one, so a crash mid-write leaves the last good copy rather than half of one. The
/// directory is the server's to back up; nothing here is a cache.
/// <para>
/// Ids are checked against a narrow pattern before they touch a path, so no id can reach outside the directory. Safe
/// to use from any thread: writes and the player index are serialized.
/// </para>
/// </summary>
public sealed partial class EventStore
{
    private static readonly JsonSerializerOptions Json = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.CamelCase) },
    };

    private readonly object _gate = new();

    public EventStore(string root)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(root);
        Root = Path.GetFullPath(root);
        Directory.CreateDirectory(Path.Combine(Root, "events"));
        Directory.CreateDirectory(Path.Combine(Root, "players"));
    }

    public string Root { get; }

    /// <summary>
    /// The data directory a host was given: <c>--data PATH</c> on its command line, else the <c>CANTRIP_DATA</c>
    /// environment variable; null when neither is set.
    /// </summary>
    public static string? Configured(string[] args)
    {
        ArgumentNullException.ThrowIfNull(args);
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (string.Equals(args[i], "--data", StringComparison.Ordinal))
            {
                return args[i + 1];
            }
        }

        return Environment.GetEnvironmentVariable("CANTRIP_DATA") is { Length: > 0 } fromEnvironment ? fromEnvironment : null;
    }

    [GeneratedRegex("^[a-z0-9][a-z0-9._-]{0,95}$")]
    private static partial Regex IdPattern();

    /// <summary>
    /// Whether an id may name an event or a player: lower-case letters, digits, '.', '_' and '-', not ending in '.' (which
    /// Windows would drop, making two ids one directory) and not a name Windows keeps for a device.
    /// </summary>
    public static bool ValidId(string? id) => id is not null && IdPattern().IsMatch(id) && !id.Contains("..", StringComparison.Ordinal)
        && !id.EndsWith('.') && !Reserved.Contains(id.Split('.')[0]);

    private static readonly HashSet<string> Reserved = new(StringComparer.Ordinal)
    {
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
        "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    };

    /// <summary>An id made from anything - a name, a Discord id - by keeping what an id may hold.</summary>
    public static string ToId(string text)
    {
        ArgumentNullException.ThrowIfNull(text);
        string id = new([.. text.Trim().ToLowerInvariant().Select(c => c is (>= 'a' and <= 'z') or (>= '0' and <= '9') or '_' or '-' ? c : '-')]);
        id = id.Trim('-');
        id = id.Length == 0 ? "player" : id[..Math.Min(id.Length, 96)];
        return ValidId(id) ? id : "player-" + id;
    }

    /// <summary>A new event id: its kind, the day, and enough randomness that two never meet.</summary>
    public static string NewEventId(string kind, DateTimeOffset now) =>
        $"{ToId(kind)}-{now:yyyyMMdd}-{Convert.ToHexString(RandomNumberGenerator.GetBytes(4)).ToLowerInvariant()}";

    public EventRecord? LoadEvent(string id) => Read<EventRecord>(EventFile(id, "event.json"));

    public void SaveEvent(EventRecord record)
    {
        ArgumentNullException.ThrowIfNull(record);
        Write(EventFile(record.Id, "event.json"), record);
    }

    /// <summary>A draft's own state, beside its event; null when the event has none yet.</summary>
    public T? LoadDraft<T>(string eventId) where T : class => Read<T>(EventFile(eventId, "draft.json"));

    public void SaveDraft<T>(string eventId, T draft) where T : class => Write(EventFile(eventId, "draft.json"), draft);

    /// <summary>
    /// When an event's files - its record and its draft - were last written, for a reader that keeps what it read and
    /// reads again only what has changed. Looks only at the files' times, so it takes no lock.
    /// </summary>
    public DateTime EventWrittenAt(string id)
    {
        DateTime record = File.GetLastWriteTimeUtc(EventFile(id, "event.json"));
        DateTime draft = File.GetLastWriteTimeUtc(EventFile(id, "draft.json"));
        return record > draft ? record : draft;
    }

    /// <summary>Every event saved, by id. Reads only the directory names.</summary>
    public IEnumerable<string> EventIds() =>
        Directory.EnumerateDirectories(Path.Combine(Root, "events"))
            .Select(Path.GetFileName)
            .OfType<string>()
            .Where(ValidId)
            .Order(StringComparer.Ordinal);

    /// <summary>Every event still to be played out.</summary>
    public IEnumerable<EventRecord> LiveEvents() =>
        EventIds().Select(LoadEvent).OfType<EventRecord>().Where(record => record.IsLive);

    public PlayerRecord? LoadPlayer(string id) => Read<PlayerRecord>(PlayerFile(id));

    /// <summary>
    /// Anything else a host keeps, one file per id in a folder of its own beside events and players - a Discord bot's
    /// queues, say, as <c>queues/{channel}.json</c>.
    /// </summary>
    public T? Load<T>(string folder, string id) where T : class => Read<T>(HostFile(folder, id));

    public void Save<T>(string folder, string id, T value) where T : class => Write(HostFile(folder, id), value);

    /// <summary>Removes what a host kept under an id; nothing when there is none.</summary>
    public void Delete(string folder, string id)
    {
        lock (_gate)
        {
            File.Delete(HostFile(folder, id));
        }
    }

    /// <summary>The ids a host has kept something under in a folder.</summary>
    public IEnumerable<string> Ids(string folder)
    {
        string directory = Path.Combine(Root, ValidId(folder) && folder is not ("events" or "players") ? folder : throw new ArgumentException($"'{folder}' is not a folder a host may keep things in.", nameof(folder)));
        return Directory.Exists(directory)
            ? Directory.EnumerateFiles(directory, "*.json").Select(Path.GetFileNameWithoutExtension).OfType<string>().Where(ValidId).Order(StringComparer.Ordinal)
            : [];
    }

    private string HostFile(string folder, string id) =>
        ValidId(folder) && folder is not ("events" or "players") && ValidId(id)
            ? Path.Combine(Root, folder, id + ".json")
            : throw new ArgumentException($"'{folder}/{id}' is not somewhere a host may keep things.", nameof(id));

    /// <summary>
    /// Notes in a player's record that they are in an event, making the record if this is their first, and keeps the
    /// name they go by up to date.
    /// </summary>
    public PlayerRecord Enlist(string playerId, string name, EventRecord record, DateTimeOffset now)
    {
        ArgumentNullException.ThrowIfNull(record);
        lock (_gate)
        {
            PlayerRecord player = LoadPlayer(playerId) ?? new PlayerRecord { Id = playerId };
            player.Name = name;
            if (!player.Events.Any(entry => entry.EventId == record.Id))
            {
                player.Events.Add(new PlayerEventEntry { EventId = record.Id, Kind = record.Kind, Joined = now });
            }

            Write(PlayerFile(playerId), player);
            return player;
        }
    }

    /// <summary>The SHA-256 of a ticket, in hex, as <see cref="EventParticipant.TicketHashes"/> keeps it.</summary>
    public static string HashTicket(string ticket) =>
        Convert.ToHexString(SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(ticket ?? string.Empty)));

    private string EventFile(string id, string file) =>
        ValidId(id) ? Path.Combine(Root, "events", id, file) : throw new ArgumentException($"'{id}' is not an event id.", nameof(id));

    private string PlayerFile(string id) =>
        ValidId(id) ? Path.Combine(Root, "players", id + ".json") : throw new ArgumentException($"'{id}' is not a player id.", nameof(id));

    // Under the lock, as writes are: a file held open for reading cannot be moved over on Windows.
    private T? Read<T>(string path) where T : class
    {
        lock (_gate)
        {
            if (!File.Exists(path))
            {
                return null;
            }

            using FileStream stream = File.OpenRead(path);
            return JsonSerializer.Deserialize<T>(stream, Json);
        }
    }

    private void Write<T>(string path, T value)
    {
        lock (_gate)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            string temporary = path + ".tmp";
            using (FileStream stream = File.Create(temporary))
            {
                JsonSerializer.Serialize(stream, value, Json);
                stream.Flush(flushToDisk: true);
            }

            File.Move(temporary, path, overwrite: true);
        }
    }
}
