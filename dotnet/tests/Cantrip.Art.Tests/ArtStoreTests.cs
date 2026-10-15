using System.Net;
using Cantrip.Art;
using Microsoft.AspNetCore.Builder;
using SkiaSharp;
using Xunit;

namespace Cantrip.Art.Tests;

public sealed class ArtStoreTests : IDisposable
{
    private readonly string _root = Path.Combine(Path.GetTempPath(), $"cantrip-art-{Guid.NewGuid():N}");
    private readonly string _images;
    private readonly ArtStore _store;

    public ArtStoreTests()
    {
        _images = Path.Combine(_root, "content", "mtg", "images");
        Directory.CreateDirectory(_images);
        WritePicture("lightning-bolt.png", 2000, 1400);
        WritePicture("tiny-thing.jpg", 100, 70);

        _store = new ArtStore(new ArtStoreOptions
        {
            ContentRoot = Path.Combine(_root, "content"),
            CacheDirectory = Path.Combine(_root, "cache"),
        });
    }

    public void Dispose() => Directory.Delete(_root, recursive: true);

    [Fact]
    public async Task A_rendition_is_brought_down_to_its_size_on_the_longest_side()
    {
        ArtRendition? rendition = await _store.GetAsync("lightning-bolt", 256, CancellationToken.None);

        Assert.NotNull(rendition);
        using SKBitmap bitmap = SKBitmap.Decode(rendition.Path);
        Assert.Equal(256, bitmap.Width);
        Assert.Equal(179, bitmap.Height);
        Assert.Equal(Path.Combine(_root, "cache", "256", "lightning-bolt.webp"), rendition.Path);
    }

    [Fact]
    public async Task A_picture_smaller_than_the_size_is_never_enlarged()
    {
        ArtRendition? rendition = await _store.GetAsync("tiny-thing", 512, CancellationToken.None);

        Assert.NotNull(rendition);
        using SKBitmap bitmap = SKBitmap.Decode(rendition.Path);
        Assert.Equal((100, 70), (bitmap.Width, bitmap.Height));
    }

    [Theory]
    [InlineData("lightning-bolt", 300)]
    [InlineData("../lightning-bolt", 256)]
    [InlineData("Lightning-Bolt", 256)]
    [InlineData("lightning--bolt", 256)]
    [InlineData("no-such-card", 256)]
    public async Task Only_fixed_sizes_of_known_slugs_are_answered(string slug, int size)
    {
        Assert.Null(await _store.GetAsync(slug, size, CancellationToken.None));
    }

    [Fact]
    public async Task A_rendition_is_built_once_and_rebuilt_when_its_source_changes()
    {
        ArtRendition first = (await _store.GetAsync("lightning-bolt", 128, CancellationToken.None))!;
        DateTime built = File.GetLastWriteTimeUtc(first.Path);

        ArtRendition again = (await _store.GetAsync("lightning-bolt", 128, CancellationToken.None))!;
        Assert.Equal(first.ETag, again.ETag);
        Assert.Equal(built, File.GetLastWriteTimeUtc(again.Path));

        WritePicture("lightning-bolt.png", 600, 900);
        File.SetLastWriteTimeUtc(Path.Combine(_images, "lightning-bolt.png"), DateTime.UtcNow.AddMinutes(1));

        ArtRendition changed = (await _store.GetAsync("lightning-bolt", 128, CancellationToken.None))!;
        Assert.NotEqual(first.ETag, changed.ETag);
        using SKBitmap bitmap = SKBitmap.Decode(changed.Path);
        Assert.Equal((85, 128), (bitmap.Width, bitmap.Height));
    }

    [Fact]
    public async Task Over_http_art_is_cacheable_revalidatable_and_readable_from_any_origin()
    {
        await using WebApplication app = await ArtServer.StartAsync(_store, "127.0.0.1", 0);
        using var http = new HttpClient { BaseAddress = new Uri(app.Urls.First()) };

        using HttpResponseMessage found = await http.GetAsync(new Uri("/art/256/lightning-bolt.webp", UriKind.Relative));
        Assert.Equal(HttpStatusCode.OK, found.StatusCode);
        Assert.Equal("image/webp", found.Content.Headers.ContentType?.MediaType);
        Assert.Equal("*", found.Headers.GetValues("Access-Control-Allow-Origin").Single());
        Assert.Contains("max-age=604800", found.Headers.CacheControl?.ToString(), StringComparison.Ordinal);
        Assert.NotNull(found.Headers.ETag);

        using var conditional = new HttpRequestMessage(HttpMethod.Get, "/art/256/lightning-bolt.webp");
        conditional.Headers.IfNoneMatch.Add(found.Headers.ETag);
        using HttpResponseMessage unchanged = await http.SendAsync(conditional);
        Assert.Equal(HttpStatusCode.NotModified, unchanged.StatusCode);

        using HttpResponseMessage missing = await http.GetAsync(new Uri("/art/256/no-such-card.webp", UriKind.Relative));
        Assert.Equal(HttpStatusCode.NotFound, missing.StatusCode);
        Assert.Contains("max-age=600", missing.Headers.CacheControl?.ToString(), StringComparison.Ordinal);

        using HttpResponseMessage odd = await http.GetAsync(new Uri("/art/300/lightning-bolt.webp", UriKind.Relative));
        Assert.Equal(HttpStatusCode.NotFound, odd.StatusCode);
    }

    [Fact]
    public async Task Credits_are_served_from_the_cache_folder_beside_the_sizes()
    {
        await using (WebApplication bare = await ArtServer.StartAsync(_store, "127.0.0.1", 0))
        {
            using var none = new HttpClient { BaseAddress = new Uri(bare.Urls.First()) };
            using HttpResponseMessage absent = await none.GetAsync(new Uri("/art/artists.json", UriKind.Relative));
            Assert.Equal(HttpStatusCode.NotFound, absent.StatusCode);
        }

        string content = Path.Combine(_root, "content");
        File.WriteAllText(Path.Combine(content, "mtg", ArtStore.CreditsFile), """
            // Who illustrated the pictures in images/, by slug.
            { "lightning-bolt": "Christopher Rush", "Not A Slug": "Nobody" }
            """);
        var store = new ArtStore(new ArtStoreOptions { ContentRoot = content, CacheDirectory = Path.Combine(_root, "cache") });

        await using WebApplication app = await ArtServer.StartAsync(store, "127.0.0.1", 0);
        using var http = new HttpClient { BaseAddress = new Uri(app.Urls.First()) };
        using HttpResponseMessage found = await http.GetAsync(new Uri("/art/artists.json", UriKind.Relative));

        Assert.Equal(HttpStatusCode.OK, found.StatusCode);
        Assert.Equal("*", found.Headers.GetValues("Access-Control-Allow-Origin").Single());
        Assert.Equal("""{"lightning-bolt":"Christopher Rush"}""", await found.Content.ReadAsStringAsync());
        Assert.True(File.Exists(Path.Combine(_root, "cache", ArtStore.CreditsFile)));
    }

    private void WritePicture(string name, int width, int height)
    {
        using var bitmap = new SKBitmap(width, height);
        using (var canvas = new SKCanvas(bitmap))
        {
            canvas.Clear(SKColors.SteelBlue);
        }

        SKEncodedImageFormat format = name.EndsWith(".png", StringComparison.Ordinal) ? SKEncodedImageFormat.Png : SKEncodedImageFormat.Jpeg;
        using SKData data = SKImage.FromBitmap(bitmap).Encode(format, 90);
        File.WriteAllBytes(Path.Combine(_images, name), data.ToArray());
    }
}
