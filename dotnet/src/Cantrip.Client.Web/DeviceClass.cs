using System.Runtime.InteropServices.JavaScript;

namespace Cantrip.Client.Web;

/// <summary>
/// Whether this device is short of memory: what the page's caches are allowed, and what the art and animation settings
/// are before the player has chosen them. A tablet of a few years ago at an in-person table, browsing a cube's worth of
/// grids, holds every card it has drawn; one that runs out of memory stops drawing art, then stops altogether.
/// <para>
/// Read from <c>navigator.deviceMemory</c>, the device's memory in gigabytes as the browser rounds it (Chrome, Edge and
/// the Android browsers say it; Safari and Firefox do not). A device that says 4 GB or less is constrained; one that
/// says nothing is not, since every iPad and desktop browser that keeps quiet about it has plenty. A setting the player
/// has saved is theirs, whatever the device.
/// </para>
/// </summary>
public static class DeviceClass
{
    /// <summary>The most memory, in gigabytes, a device may report and still be held constrained.</summary>
    public const double ConstrainedAtOrBelow = 4;

    private static bool? _constrained;

    /// <summary>Whether this device is short of memory. Asked of the browser once.</summary>
    public static bool Constrained => _constrained ??= Read();

    private static bool Read()
    {
        try
        {
            JSObject? navigator = JSHost.GlobalThis.GetPropertyAsJSObject("navigator");
            if (navigator is null || navigator.GetTypeOfProperty("deviceMemory") != "number")
            {
                return false;
            }

            double gigabytes = navigator.GetPropertyAsDouble("deviceMemory");
            return gigabytes > 0 && gigabytes <= ConstrainedAtOrBelow;
        }
        catch (JSException)
        {
            return false;
        }
    }

    /// <summary>
    /// How many bytes the faces kept for good may hold (<see cref="FaceCache"/>): the recordings, their rasters under the
    /// software renderer, and the art they draw.
    /// </summary>
    public static long DurableFaceBudget => Constrained ? 32L * 1024 * 1024 : 384L * 1024 * 1024;

    /// <summary>How many bytes the faces of the moment may hold (<see cref="FaceCache"/>).</summary>
    public static long MomentFaceBudget => Constrained ? 16L * 1024 * 1024 : 32L * 1024 * 1024;

    /// <summary>How many bytes of decoded art <see cref="CardArt"/> keeps ready to draw a face with.</summary>
    public static long ArtBudget => Constrained ? 24L * 1024 * 1024 : 256L * 1024 * 1024;
}
