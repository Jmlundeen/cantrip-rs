using System.Globalization;
using System.Runtime.InteropServices.JavaScript;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// This tab's seat at a table, kept across a reload so the client resumes its own seat instead of asking for a new one
/// that the server is still holding for it (the server reserves a dropped seat for its reconnect grace period).
/// <para>
/// In sessionStorage, not localStorage: the proof is a credential for the seat, so it lives only as long as the tab,
/// and a second tab is a second player rather than the same one. One entry per server address.
/// </para>
/// </summary>
public static class SeatMemory
{
    private static JSObject Storage => JSHost.GlobalThis.GetPropertyAsJSObject("sessionStorage")!;

    /// <param name="who">Empty for this page's own seat, and a name of its own for each seat the page plays for a bot.</param>
    private static string Key(string who) => $"cantrip-seat:{ClientConfig.Server}{who}";

    public static MembershipMessage? Recall(string who = "")
    {
        string[] parts = (Storage.GetPropertyAsString(Key(who)) ?? string.Empty).Split('\n');

        return parts.Length == 3 && int.TryParse(parts[2], NumberStyles.Integer, CultureInfo.InvariantCulture, out int seat)
            ? new MembershipMessage { MembershipId = parts[0], ReconnectProof = parts[1], Seat = seat }
            : null;
    }

    public static void Remember(MembershipMessage membership, string who = "") =>
        Storage.SetProperty(Key(who), string.Create(CultureInfo.InvariantCulture, $"{membership.MembershipId}\n{membership.ReconnectProof}\n{membership.Seat}"));

    public static void Forget(string who = "") => Storage.SetProperty(Key(who), string.Empty);
}
