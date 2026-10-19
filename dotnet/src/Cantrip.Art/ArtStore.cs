using System.Collections.Concurrent;
using System.Globalization;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.RegularExpressions;
using SkiaSharp;

namespace Cantrip.Art;

public sealed class ArtStoreOptions
{
    /// <summary>The folder holding the datasets; each dataset's illustrations are in its own <c>images</c> folder.</summary>
    public required string ContentRoot { get; init; }

    /// <summary>Where renditions are kept between runs, laid out as <c>{size}/{slug}.webp</c> so it can be mirrored as is.</summary>
    public required string CacheDirectory { get; init; }

    public int WebpQuality { get; init; } = 82;
}

/// <summary>One illustration at one size, ready to send.</summary>
public sealed record ArtRendition(string Path, string ETag, DateTimeOffset LastModified);

/// <summary>Who illustrated each picture, by slug, as the file it is sent from.</summary>
public sealed record ArtCredits(string Path, string ETag, DateTimeOffset LastModified, IReadOnlyDictionary<string, string> Artists);

/// <summary>
/// Card illustrations, looked up by the card's slug, brought down to one of a few fixed sizes and kept on disk.
/// <para>
/// The sizes are fixed rather than asked for, so every client asking for a size asks for the same file and a cache in
/// front of this one — the browser's, or a CDN's — holds a handful of variants per card rather than one per window
/// width. It also means a request cannot make this build anything it would not have built anyway.
/// </para>
/// <para>
/// Keyed by slug, the data-file convention the web client's CardArt asks for art by: the source file for "Lim-Dûl's
/// Vault" is <c>images/lim-duls-vault.jpg</c>.
/// </para>
/// </summary>
public sealed partial class ArtStore
{
    /// <summary>
    /// The longest side each rendition is kept at: the web client's three art qualities, and twice the largest of
    /// them for a card held up close on a dense screen.
    /// </summary>
    public static readonly IReadOnlyList<int> Sizes = [128, 256, 512, 1024];

    private static readonly TimeSpan RescanAfter = TimeSpan.FromSeconds(30);
    private static readonly string[] Extensions = [".png", ".jpg", ".jpeg", ".webp"];

    private readonly ArtStoreOptions _options;
    private readonly ConcurrentDictionary<string, SemaphoreSlim> _building = new(StringComparer.Ordinal);
    private readonly SemaphoreSlim _encoders = new(Math.Max(1, Environment.ProcessorCount / 2));
    private readonly object _scanning = new();
    private Dictionary<string, string> _sources = [];
    private ArtCredits? _credits;
    private DateTime _scannedAt;

    public ArtStore(ArtStoreOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        _options = options;
        Directory.CreateDirectory(options.CacheDirectory);
        Scan();
    }

    public int Count => _sources.Count;

    /// <summary>
    /// The file in each dataset naming who illustrated its pictures, by slug - <c>{"llanowar-elves": "Anson Maddocks"}</c> -
    /// and the name it is served under from the cache folder, beside the sizes, so a mirror of the folder carries it.
    /// </summary>
    public const string CreditsFile = "artists.json";

    /// <summary>
    /// Every dataset's illustration credits in one file in the cache folder, written when they change. Null when no dataset
    /// credits anything.
    /// </summary>
    public ArtCredits? Credits => _credits;

    public static bool IsSlug(string slug) => SlugPattern().IsMatch(slug);

    /// <summary>
    /// The illustration for <paramref name="slug"/> at <paramref name="size"/>, built on first asking and read from disk
    /// after that. Null when there is no such card art, or the request is not one this store answers.
    /// </summary>
    public async Task<ArtRendition?> GetAsync(string slug, int size, CancellationToken cancellation)
    {
        if (!IsSlug(slug) || !Sizes.Contains(size))
        {
            return null;
        }

        if (Source(slug) is not { } source || new FileInfo(source) is not { Exists: true } info)
        {
            return Kept(slug, size);
        }

        string stamp = string.Create(
            CultureInfo.InvariantCulture,
            $"{info.Length:x}-{info.LastWriteTimeUtc.Ticks:x}-{_options.WebpQuality}");

        string folder = Path.Combine(_options.CacheDirectory, size.ToString(CultureInfo.InvariantCulture));
        string file = Path.Combine(folder, $"{slug}.webp");
        string stampFile = Path.Combine(folder, $"{slug}.stamp");
        var rendition = new ArtRendition(file, $"\"{size}-{stamp}\"", new DateTimeOffset(info.LastWriteTimeUtc));

        if (Fresh(file, stampFile, stamp))
        {
            return rendition;
        }

        SemaphoreSlim gate = _building.GetOrAdd(file, _ => new SemaphoreSlim(1, 1));
        await gate.WaitAsync(cancellation).ConfigureAwait(false);

        try
        {
            // Somebody else may have built it while this request waited for the gate.
            if (Fresh(file, stampFile, stamp))
            {
                return rendition;
            }

            await _encoders.WaitAsync(cancellation).ConfigureAwait(false);

            try
            {
                if (!Render(source, size, file))
                {
                    return null;
                }
            }
            finally
            {
                _encoders.Release();
            }

            // Written whole and moved into place, since Fresh reads it without the gate. Not cancelled with the request: the
            // picture is built, and a missing stamp would only have it built again for the next one.
            string written = stampFile + ".tmp";
            await File.WriteAllTextAsync(written, stamp, CancellationToken.None).ConfigureAwait(false);
            File.Move(written, stampFile, overwrite: true);
            return rendition;
        }
        finally
        {
            gate.Release();
        }
    }

    /// <summary>
    /// The rendition already in the cache folder for a slug with no source here, so a host that was given the folder alone
    /// (copied from another, as the folder is laid out to be) answers for what it holds. Null when it holds none.
    /// </summary>
    private ArtRendition? Kept(string slug, int size)
    {
        string folder = Path.Combine(_options.CacheDirectory, size.ToString(CultureInfo.InvariantCulture));
        string file = Path.Combine(folder, $"{slug}.webp");
        string stampFile = Path.Combine(folder, $"{slug}.stamp");
        try
        {
            if (!File.Exists(file))
            {
                return null;
            }

            // The stamp names the source the picture was built from; without one, the file's own size and time stand in.
            var info = new FileInfo(file);
            string stamp = File.Exists(stampFile)
                ? File.ReadAllText(stampFile)
                : string.Create(CultureInfo.InvariantCulture, $"{info.Length:x}-{info.LastWriteTimeUtc.Ticks:x}");
            return new ArtRendition(file, $"\"{size}-{stamp}\"", new DateTimeOffset(info.LastWriteTimeUtc));
        }
        catch (IOException)
        {
            return null;
        }
    }

    /// <summary>
    /// Whether the picture on disk was built from the source as it is now. A stamp that cannot be read - another process
    /// sharing the cache folder is replacing it - is not fresh, so the request waits its turn to build rather than failing.
    /// </summary>
    private static bool Fresh(string file, string stampFile, string stamp)
    {
        try
        {
            return File.Exists(file) && File.Exists(stampFile) && File.ReadAllText(stampFile) == stamp;
        }
        catch (IOException)
        {
            return false;
        }
    }

    /// <summary>Decodes, brings the longest side down to <paramref name="size"/> (never up), and writes WebP atomically.</summary>
    private bool Render(string source, int size, string file)
    {
        using SKBitmap? original = SKBitmap.Decode(source);
        if (original is null)
        {
            return false;
        }

        float scale = MathF.Min(1f, size / (float)Math.Max(original.Width, original.Height));
        var target = new SKImageInfo(
            Math.Max(1, (int)MathF.Round(original.Width * scale)),
            Math.Max(1, (int)MathF.Round(original.Height * scale)),
            SKColorType.Rgba8888,
            SKAlphaType.Premul);

        // Cubic: this is the only resampling the picture gets at this size before the GPU draws it.
        using SKBitmap resized = original.Resize(target, new SKSamplingOptions(SKCubicResampler.Mitchell)) ?? original.Copy();
        using SKImage image = SKImage.FromBitmap(resized);
        using SKData? encoded = image.Encode(SKEncodedImageFormat.Webp, _options.WebpQuality);
        if (encoded is null)
        {
            return false;
        }

        Directory.CreateDirectory(Path.GetDirectoryName(file)!);
        string partial = $"{file}.{Guid.NewGuid():N}.partial";

        using (FileStream stream = File.Create(partial))
        {
            encoded.SaveTo(stream);
        }

        File.Move(partial, file, overwrite: true);
        return true;
    }

    /// <summary>The source file for a slug. A miss rescans, at most so often, so art added while running is found.</summary>
    private string? Source(string slug)
    {
        if (_sources.TryGetValue(slug, out string? path))
        {
            return path;
        }

        if (DateTime.UtcNow - _scannedAt > RescanAfter)
        {
            Scan();
            return _sources.GetValueOrDefault(slug);
        }

        return null;
    }

    private void Scan()
    {
        lock (_scanning)
        {
            Dictionary<string, string> found = new(StringComparer.Ordinal);

            if (Directory.Exists(_options.ContentRoot))
            {
                foreach (string dataset in Directory.EnumerateDirectories(_options.ContentRoot).Order(StringComparer.Ordinal))
                {
                    string images = Path.Combine(dataset, "images");
                    if (!Directory.Exists(images))
                    {
                        continue;
                    }

                    foreach (string path in Directory.EnumerateFiles(images).Order(StringComparer.Ordinal))
                    {
                        string slug = Path.GetFileNameWithoutExtension(path);
                        if (IsSlug(slug) && Extensions.Contains(Path.GetExtension(path), StringComparer.OrdinalIgnoreCase))
                        {
                            found.TryAdd(slug, path);
                        }
                    }
                }
            }

            _sources = found;
            _credits = WriteCredits(ReadCredits());
            _scannedAt = DateTime.UtcNow;
        }
    }

    /// <summary>Each dataset's credits, the first dataset to credit a slug winning, as the first to hold its picture does.</summary>
    private SortedDictionary<string, string> ReadCredits()
    {
        SortedDictionary<string, string> credits = new(StringComparer.Ordinal);
        if (!Directory.Exists(_options.ContentRoot))
        {
            return credits;
        }

        foreach (string dataset in Directory.EnumerateDirectories(_options.ContentRoot).Order(StringComparer.Ordinal))
        {
            string file = Path.Combine(dataset, CreditsFile);
            if (!File.Exists(file))
            {
                continue;
            }

            Dictionary<string, string>? read = JsonSerializer.Deserialize<Dictionary<string, string>>(
                File.ReadAllText(file),
                new JsonSerializerOptions { ReadCommentHandling = JsonCommentHandling.Skip, AllowTrailingCommas = true });

            foreach ((string slug, string artist) in read ?? [])
            {
                if (IsSlug(slug) && artist.Trim() is { Length: > 0 } name)
                {
                    credits.TryAdd(slug, name);
                }
            }
        }

        return credits;
    }

    /// <summary>The credits written to the cache folder, if they are not there as they are already.</summary>
    private ArtCredits? WriteCredits(SortedDictionary<string, string> credits)
    {
        if (credits.Count == 0)
        {
            return null;
        }

        byte[] bytes = JsonSerializer.SerializeToUtf8Bytes(credits);
        string stamp = Convert.ToHexString(SHA256.HashData(bytes))[..16].ToLowerInvariant();
        if (_credits?.ETag == $"\"{stamp}\"")
        {
            return _credits;
        }

        string path = Path.Combine(_options.CacheDirectory, CreditsFile);
        bool current;
        try
        {
            current = File.Exists(path) && File.ReadAllBytes(path).AsSpan().SequenceEqual(bytes);
        }
        catch (IOException)
        {
            current = false;
        }

        if (!current)
        {
            string partial = $"{path}.{Guid.NewGuid():N}.partial";
            File.WriteAllBytes(partial, bytes);
            File.Move(partial, path, overwrite: true);
        }

        return new ArtCredits(path, $"\"{stamp}\"", new DateTimeOffset(File.GetLastWriteTimeUtc(path)), credits);
    }

    [GeneratedRegex("^[a-z0-9]+(-[a-z0-9]+)*$", RegexOptions.CultureInvariant)]
    private static partial Regex SlugPattern();
}
