using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// Players' pictures, as the table names them (<see cref="Cantrip.Protocol.TablePlayerView.Portrait"/>): fetched once each
/// for the session, relative to the page. A new picture of a player comes under a new address, so nothing here is ever
/// out of date; one that cannot be had is null, and the portrait keeps its stand-in.
/// </summary>
public static class Portraits
{
    private static readonly HttpClient Http = new();
    private static readonly Dictionary<string, Task<SKImage?>> Asked = new(StringComparer.Ordinal);

    /// <summary>The picture at <paramref name="source"/>, once it has come; null when there is none to be had.</summary>
    public static Task<SKImage?> Get(string source)
    {
        if (!Asked.TryGetValue(source, out Task<SKImage?>? asked))
        {
            asked = FetchAsync(source);
            Asked[source] = asked;
        }

        return asked;
    }

    /// <summary>The picture at <paramref name="source"/> if it has come, asking for it if it has not been; null meanwhile.</summary>
    public static SKImage? Peek(string source) =>
        source.Length > 0 && Get(source) is { IsCompletedSuccessfully: true } done ? done.Result : null;

    private static async Task<SKImage?> FetchAsync(string source)
    {
        try
        {
            byte[] bytes = await Http.GetByteArrayAsync(new Uri(ClientConfig.Page, source));
            return SKImage.FromEncodedData(bytes);
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or UriFormatException)
        {
            return null;
        }
    }
}
