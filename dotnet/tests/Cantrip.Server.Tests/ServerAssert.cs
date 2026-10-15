using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Assertions about a live table that say what went wrong rather than only that something did.
/// </summary>
internal static class ServerAssert
{
    /// <summary>
    /// That the table's pump has not faulted — and, when it has, the whole of what the server knows about it.
    /// <para>
    /// <c>Assert.False(server.Finished.IsFaulted)</c> reports "expected False, actual True" and throws the
    /// exception away, which for a fault that only appears on a loaded machine is the entire evidence gone. The
    /// pump records what it was doing and what had just happened to the table
    /// (<see cref="GameServer.FaultDetail"/>); this is what puts that in the failure.
    /// </para>
    /// </summary>
    public static void DidNotFault(GameServer server)
    {
        ArgumentNullException.ThrowIfNull(server);

        if (!server.Finished.IsFaulted)
        {
            return;
        }

        Assert.Fail(
            $"The table loop faulted.\n{server.FaultDetail}\n\n{server.Finished.Exception}");
    }
}
