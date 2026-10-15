using System.Globalization;
using System.Runtime.InteropServices.JavaScript;

namespace Cantrip.Client.Web;

/// <summary>
/// Small per-browser preferences in localStorage: a size the player dragged something to, a switch in the settings. Anything that fails to read or write - a private window, blocked site
/// data - falls back to the default, so the page works the same without them.
/// </summary>
public static class LocalSettings
{
    public static float Number(string key, float fallback) =>
        float.TryParse(Text(key), NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : fallback;

    public static void Store(string key, float value) => Store(key, value.ToString(CultureInfo.InvariantCulture));

    public static bool Flag(string key, bool fallback) => Text(key) switch
    {
        "true" => true,
        "false" => false,
        _ => fallback,
    };

    public static void Store(string key, bool value) => Store(key, value ? "true" : "false");

    public static string? Text(string key)
    {
        try
        {
            return JSHost.GlobalThis.GetPropertyAsJSObject("localStorage")?.GetPropertyAsString(key);
        }
        catch (JSException)
        {
            return null;
        }
    }

    public static void Store(string key, string value)
    {
        try
        {
            JSHost.GlobalThis.GetPropertyAsJSObject("localStorage")?.SetProperty(key, value);
        }
        catch (JSException)
        {
            // Not kept: the value still holds for this session.
        }
    }
}
