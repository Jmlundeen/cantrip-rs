using System.Runtime.InteropServices.JavaScript;

namespace Cantrip.Client.Web;

/// <summary>Names the browser tab for what the page is showing (Title.js), only when that changes.</summary>
public static partial class BrowserTitle
{
    private static string? _shown;

    [JSImport("globalThis.cantripSetTitle")]
    private static partial void SetTitle(string title);

    public static void Show(string title)
    {
        if (title == _shown)
        {
            return;
        }

        _shown = title;
        try
        {
            SetTitle(title);
        }
        catch (JSException)
        {
            // A page without the script - a test harness of its own - keeps whatever title it has.
        }
    }
}
