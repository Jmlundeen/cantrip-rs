using System.Globalization;
using System.Runtime.InteropServices.JavaScript;
using Windows.System;

namespace Cantrip.Client.Web;

/// <summary>A key going down or up, as the page heard it.</summary>
public readonly record struct KeyPress(bool Down, VirtualKey Key, bool Repeat, bool Ctrl, bool Shift, bool Alt, bool Typing);

/// <summary>
/// The keys the browser window heard since the last read, from Keys.js. The page's own keyboard events reach only a
/// focused control, and the table focuses nothing, so the hotkeys listen here instead.
/// </summary>
public static partial class KeyboardBridge
{
    [JSImport("globalThis.cantripTakeKeys")]
    private static partial string TakeKeys();

    /// <summary>Every key heard since the last call, oldest first; a <c>null</c> for the window losing focus.</summary>
    public static IEnumerable<KeyPress?> Take()
    {
        string taken;
        try
        {
            taken = TakeKeys();
        }
        catch (JSException)
        {
            yield break;
        }

        foreach (string line in taken.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            if (line == "b")
            {
                yield return null;
                continue;
            }

            string[] parts = line.Split(',');
            if (parts.Length != 7 || !int.TryParse(parts[1], NumberStyles.Integer, CultureInfo.InvariantCulture, out int code))
            {
                continue;
            }

            yield return new KeyPress(parts[0] == "d", (VirtualKey)code, parts[2] == "1", parts[3] == "1", parts[4] == "1", parts[5] == "1", parts[6] == "1");
        }
    }
}
