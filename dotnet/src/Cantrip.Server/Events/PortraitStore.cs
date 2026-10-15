namespace Cantrip.Server.Events;

/// <summary>
/// Players' pictures, kept in the data directory beside the players themselves:
/// <code>
/// portraits/{player}.png
/// </code>
/// One picture to a player. A new one is written over the old, through a temporary file, so a reader never sees half of
/// one. Where a picture comes from is the host's business - the Discord bot takes each player's avatar as they join a
/// draft - and a player without one simply has none.
/// <para>
/// A picture is shown by its <see cref="PathFor"/>, which carries when it was written, so a browser holding the old one
/// asks again once it changes. Ids are checked as <see cref="EventStore"/> checks them, so none can reach outside the
/// directory.
/// </para>
/// </summary>
public sealed class PortraitStore
{
    /// <summary>Where portraits are served, beside the web client: <c>portraits/{player}.png</c>.</summary>
    public const string Route = "portraits";

    public PortraitStore(string root)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(root);
        Directory = Path.Combine(Path.GetFullPath(root), Route);
        System.IO.Directory.CreateDirectory(Directory);
    }

    public string Directory { get; }

    /// <summary>Keeps <paramref name="png"/> as the player's picture, in place of any they had.</summary>
    public async Task SaveAsync(string playerId, byte[] png, CancellationToken cancellation = default)
    {
        string path = FileOf(playerId) ?? throw new ArgumentException($"'{playerId}' is not a player id.", nameof(playerId));
        string temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            await File.WriteAllBytesAsync(temporary, png, cancellation).ConfigureAwait(false);
            File.Move(temporary, path, overwrite: true);
        }
        finally
        {
            File.Delete(temporary);
        }
    }

    /// <summary>
    /// The player's picture as the client is told it - <c>portraits/{player}.png?v=...</c>, relative to the page - or
    /// null when they have none.
    /// </summary>
    public string? PathFor(string playerId)
    {
        if (FileOf(playerId) is not { } path || !File.Exists(path)) return null;
        long written = File.GetLastWriteTimeUtc(path).Ticks;
        return $"{Route}/{playerId}.png?v={written:x}";
    }

    /// <summary>The file a served name - <c>{player}.png</c> - is, or null when there is no such picture.</summary>
    public string? Served(string name) =>
        name.EndsWith(".png", StringComparison.Ordinal) && FileOf(name[..^4]) is { } path && File.Exists(path) ? path : null;

    private string? FileOf(string playerId) => EventStore.ValidId(playerId) ? Path.Combine(Directory, playerId + ".png") : null;
}
