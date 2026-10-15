using System.Net;
using System.Text;
using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// Card illustrations fetched from the art service on demand, at the smallest of its fixed sizes that covers what is
/// being drawn.
/// <para>
/// Three layers keep this from asking twice: decoded pictures held here for the session; one request at a time per card,
/// however many cards on screen share it; and the browser's own HTTP cache, which the art service's Cache-Control and
/// ETag headers fill, so a reload - or a CDN in front of the service - is answered without it. A card with no art is
/// remembered as such for the session, and a failed request is not retried for a while.
/// </para>
/// </summary>
public static class CardArt
{
    private static readonly int[] Sizes = [128, 256, 512, 1024];
    private static readonly TimeSpan RetryAfter = TimeSpan.FromSeconds(30);

    private static readonly HttpClient Http = new();
    private static readonly Dictionary<string, (SKBitmap Bitmap, int Size)> Have = new(StringComparer.Ordinal);

    /// <summary>Each picture as an image to draw, made once and sharing its pixels, let go with the picture.</summary>
    private static readonly Dictionary<SKBitmap, SKImage> Images = new(ReferenceEqualityComparer.Instance);

    /// <summary>A picture from <see cref="Get"/> as an image to draw, for as long as the picture is kept.</summary>
    public static SKImage ImageOf(SKBitmap bitmap)
    {
        if (!Images.TryGetValue(bitmap, out SKImage? image))
        {
            image = SKImage.FromBitmap(bitmap);
            Images[bitmap] = image;
        }

        return image;
    }
    private static readonly Dictionary<string, int> Fetching = new(StringComparer.Ordinal);

    /// <summary>Whether any art is on its way, so a picture of the page can wait for it.</summary>
    public static bool Arriving => Fetching.Count > 0;
    private static readonly Dictionary<string, DateTime> FailedAt = new(StringComparer.Ordinal);
    private static readonly HashSet<string> Missing = new(StringComparer.Ordinal);

    /// <summary>A card's art arrived, or turned out not to exist: anything drawn with that slug is out of date.</summary>
    public static event Action<string>? Changed;

    /// <summary>
    /// The best picture on hand for <paramref name="cardName"/>, asking for a sharper one when what is here is smaller than
    /// <paramref name="pixels"/> on its longest side. Null while nothing has arrived, or when the card has no art.
    /// </summary>
    public static SKBitmap? Get(string cardName, int pixels)
    {
        _ = Fetch(cardName, pixels);
        return Have.TryGetValue(Slug(cardName), out var held) ? held.Bitmap : null;
    }

    /// <summary>
    /// Asks for <paramref name="cardName"/>'s art at the size covering <paramref name="pixels"/>, unless it is here, on its
    /// way, missing, or resting after a failure. Finishes when that answer has come, or at once when none was asked for.
    /// </summary>
    public static Task Fetch(string cardName, int pixels)
    {
        string slug = Slug(cardName);
        if (slug.Length == 0 || Missing.Contains(slug))
        {
            return Task.CompletedTask;
        }

        int size = SizeFor(pixels);
        bool haveEnough = Have.TryGetValue(slug, out var held) && held.Size >= size;
        bool asked = Fetching.TryGetValue(slug, out int inFlight) && inFlight >= size;
        bool resting = FailedAt.TryGetValue(slug, out DateTime failed) && DateTime.UtcNow - failed < RetryAfter;

        return haveEnough || asked || resting ? Task.CompletedTask : FetchAsync(slug, size);
    }

    /// <summary>The art size asked for to draw <paramref name="pixels"/>: the smallest that covers it, never past the Card art quality setting.</summary>
    public static int SizeFor(int pixels) =>
        Math.Min(Sizes.FirstOrDefault(s => s >= pixels, Sizes[^1]), PlayerSettings.Pixels(PlayerSettings.ArtQuality));

    /// <summary>The largest art size there is to ask for, by the Card art quality setting.</summary>
    public static int Best => SizeFor(int.MaxValue);

    /// <summary>
    /// Whether <paramref name="cardName"/>'s art here now is all a drawing at <paramref name="pixels"/> will get: it is at
    /// that size, or the card has none. Art that failed is not: it will be asked for again, and a drawing without it
    /// wants drawing again when it comes.
    /// </summary>
    public static bool Complete(string cardName, int pixels)
    {
        string slug = Slug(cardName);
        return slug.Length == 0 || Missing.Contains(slug) || (Have.TryGetValue(slug, out var held) && held.Size >= SizeFor(pixels));
    }

    /// <summary>
    /// Whether a face drawn now for <paramref name="cardName"/> at <paramref name="pixels"/> would be drawn with all the art
    /// it will get: the art is here at that size, or the card has none, or asking has failed for now.
    /// </summary>
    public static bool Settled(string cardName, int pixels)
    {
        string slug = Slug(cardName);
        if (slug.Length == 0 || Missing.Contains(slug))
        {
            return true;
        }

        return (Have.TryGetValue(slug, out var held) && held.Size >= SizeFor(pixels))
            || (FailedAt.TryGetValue(slug, out DateTime failed) && DateTime.UtcNow - failed < RetryAfter);
    }

    /// <summary>
    /// Rests a card's art after a failed answer, then says it changed, so whatever was drawn without it - which wanted it
    /// - is let go and drawn again, asking again. Without that, a face drawn during an outage kept no art for the session.
    /// </summary>
    private static async void Failed(string slug)
    {
        FailedAt[slug] = DateTime.UtcNow;
        await Task.Delay(RetryAfter);
        FailedAt.Remove(slug);
        Changed?.Invoke(slug);
    }

    /// <summary>One picture is decoded at a time, since it is decoded on the page's thread.</summary>
    private static readonly SemaphoreSlim Decoding = new(1, 1);

    /// <summary>
    /// How much decoding may run back to back before the page is left a frame to paint. Answers that arrive together - a
    /// deck's worth at the start of a game - would otherwise all be decoded before it could paint once.
    /// </summary>
    private static readonly TimeSpan DecodeRun = TimeSpan.FromMilliseconds(8);

    private static readonly TimeSpan Frame = TimeSpan.FromMilliseconds(16);

    private static TimeSpan _decodedInRun;

    /// <summary>Decodes a picture in its turn, leaving the page a frame to paint once a run of them has taken long enough.</summary>
    private static async Task<SKBitmap?> Decode(byte[] bytes)
    {
        await Decoding.WaitAsync();
        try
        {
            DateTime started = DateTime.UtcNow;
            SKBitmap? bitmap = SKBitmap.Decode(bytes);
            _decodedInRun += DateTime.UtcNow - started;
            return bitmap;
        }
        finally
        {
            if (_decodedInRun >= DecodeRun)
            {
                _decodedInRun = TimeSpan.Zero;
                await Task.Delay(Frame);
            }

            Decoding.Release();
        }
    }

    private static async Task FetchAsync(string slug, int size)
    {
        Fetching[slug] = size;
        string askedAt = ClientConfig.ArtTemplate;

        try
        {
            using HttpResponseMessage response = await Http.GetAsync(ClientConfig.ArtUrl(slug, size));

            // Asked of where art was before the table said where it is: the answer says nothing about the card.
            if (askedAt != ClientConfig.ArtTemplate)
            {
                Changed?.Invoke(slug);
                return;
            }

            if (response.StatusCode == HttpStatusCode.NotFound)
            {
                Missing.Add(slug);
                Changed?.Invoke(slug);
                return;
            }

            response.EnsureSuccessStatusCode();
            byte[] bytes = await response.Content.ReadAsByteArrayAsync();

            SKBitmap? bitmap = await Decode(bytes);
            if (bitmap is null)
            {
                Failed(slug);
                return;
            }

            // Never changed once decoded, so an image of it can share its pixels rather than copy them.
            bitmap.SetImmutable();

            // A smaller answer landing after a larger one is kept out: sharper art never gives way to blurrier.
            if (Have.TryGetValue(slug, out var held) && held.Size >= size)
            {
                bitmap.Dispose();
                return;
            }

            if (held.Bitmap is { } replaced)
            {
                if (Images.Remove(replaced, out SKImage? image))
                {
                    image.Dispose();
                }

                replaced.Dispose();
            }

            Have[slug] = (bitmap, size);
            FailedAt.Remove(slug);
            Changed?.Invoke(slug);
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException)
        {
            if (askedAt == ClientConfig.ArtTemplate)
            {
                Failed(slug);
            }
            else
            {
                Changed?.Invoke(slug);
            }
        }
        finally
        {
            if (Fetching.TryGetValue(slug, out int asked) && asked == size)
            {
                Fetching.Remove(slug);
            }
        }
    }

    /// <summary>Who illustrated each picture, by slug, from where art is now; empty until it has come.</summary>
    private static Dictionary<string, string> _artists = new(StringComparer.Ordinal);

    /// <summary>The art address the credits were last asked of, so a move asks again.</summary>
    private static string? _creditsAskedAt;

    /// <summary>
    /// Who illustrated <paramref name="cardName"/>'s art, or null while the credits are on their way or name no one. The
    /// first asking asks the art service for every credit at once; when they come, everything drawn with credited art is
    /// out of date (<see cref="Changed"/>).
    /// </summary>
    public static string? Artist(string cardName)
    {
        if (_creditsAskedAt != ClientConfig.ArtTemplate)
        {
            _creditsAskedAt = ClientConfig.ArtTemplate;
            _ = FetchCreditsAsync(_creditsAskedAt);
        }

        return _artists.GetValueOrDefault(Slug(cardName));
    }

    private static async Task FetchCreditsAsync(string askedAt)
    {
        if (ClientConfig.CreditsUrl is not { } address)
        {
            return;
        }

        Dictionary<string, string>? artists = null;
        try
        {
            using HttpResponseMessage response = await Http.GetAsync(address);
            if (response.IsSuccessStatusCode)
            {
                artists = System.Text.Json.JsonSerializer.Deserialize(await response.Content.ReadAsStreamAsync(), CreditsJson.Default.DictionaryStringString);
            }
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or System.Text.Json.JsonException)
        {
            // Faces are drawn without credits, as they are for art that has none.
        }

        if (askedAt != ClientConfig.ArtTemplate)
        {
            return;
        }

        if (artists is null)
        {
            // Asked again on the next face drawn after a rest, as a failed picture is.
            await Task.Delay(RetryAfter);
            if (_creditsAskedAt == askedAt)
            {
                _creditsAskedAt = null;
            }

            return;
        }

        Dictionary<string, string> before = _artists;
        _artists = new Dictionary<string, string>(artists, StringComparer.Ordinal);

        // Only what has been drawn is out of date: a face drawn before now asked for its art, so its slug is known here.
        foreach (string slug in Have.Keys.Concat(Missing).Concat(FailedAt.Keys).ToList())
        {
            if (before.GetValueOrDefault(slug) != _artists.GetValueOrDefault(slug))
            {
                Changed?.Invoke(slug);
            }
        }
    }

    /// <summary>Where art is changed: every face drawn is let go, since one drawn as having no art may have some now.</summary>
    public static event Action? Relocated;

    /// <summary>
    /// Where art is changed (<see cref="ClientConfig.TableArt"/>): what was missing or failing at the old address is
    /// asked for again at the new one, and what was drawn without it drawn again.
    /// </summary>
    public static void Moved()
    {
        List<string> again = [.. Missing, .. FailedAt.Keys];
        Missing.Clear();
        FailedAt.Clear();
        Relocated?.Invoke();
        foreach (string slug in again)
        {
            Changed?.Invoke(slug);
        }
    }

    /// <summary>
    /// Whether a face drawn with <paramref name="slugs"/> - one slug, or a split card's halves' with spaces between
    /// (<see cref="CardFace.ArtSlug"/>) - is drawn with <paramref name="slug"/>'s art.
    /// </summary>
    public static bool Covers(string slugs, string slug) =>
        slugs == slug || (slugs.Contains(' ', StringComparison.Ordinal) && slugs.Split(' ').Contains(slug, StringComparer.Ordinal));

    /// <summary>
    /// The data-file convention art is named by: accents folded away, lowercase, apostrophes dropped, other separators
    /// hyphenated.
    /// </summary>
    public static string Slug(string cardName)
    {
        var slug = new StringBuilder(cardName.Length);
        bool separator = false;

        foreach (char raw in cardName.Normalize(NormalizationForm.FormD))
        {
            if (char.GetUnicodeCategory(raw) == System.Globalization.UnicodeCategory.NonSpacingMark)
            {
                continue;
            }

            char c = char.ToLowerInvariant(raw);
            if (c is '\'' or '’')
            {
                continue;
            }

            if (char.IsAsciiLetterOrDigit(c))
            {
                if (separator && slug.Length > 0)
                {
                    slug.Append('-');
                }

                slug.Append(c);
                separator = false;
            }
            else
            {
                separator = true;
            }
        }

        return slug.ToString();
    }
}

/// <summary>Generated rather than reflected, so trimming a release build cannot strip what the credits are read into.</summary>
[System.Text.Json.Serialization.JsonSerializable(typeof(Dictionary<string, string>))]
internal sealed partial class CreditsJson : System.Text.Json.Serialization.JsonSerializerContext;
