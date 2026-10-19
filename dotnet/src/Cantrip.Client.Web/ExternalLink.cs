using System.Runtime.InteropServices.JavaScript;

namespace Cantrip.Client.Web;

/// <summary>Opens a page somewhere else - a player's Discord account - in a new browser tab, leaving the table open (Links.js).</summary>
public static partial class ExternalLink
{
    [JSImport("globalThis.cantripOpenTab")]
    private static partial void OpenTab(string address);

    public static void Open(string address)
    {
        try
        {
            OpenTab(address);
        }
        catch (JSException)
        {
            // A page without the script - a test harness of its own - opens nothing.
        }
    }
}
