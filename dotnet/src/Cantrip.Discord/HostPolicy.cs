namespace Cantrip.Discord;

/// <summary>
/// Who may host and cancel games. Decided here from what the interaction says about the member, never from whether
/// Discord happened to show them the command: command visibility is presentation, not permission.
/// </summary>
public sealed class HostPolicy(IReadOnlyCollection<ulong> hostRoles)
{
    /// <summary>The roles whose members may host. Empty means only server managers may.</summary>
    public IReadOnlyCollection<ulong> HostRoles { get; } = hostRoles;

    /// <summary>A server manager may always host; anyone else needs one of the host roles.</summary>
    public bool MayHost(IEnumerable<ulong> roles, bool managesServer) => managesServer || roles.Any(HostRoles.Contains);

    public string Refusal => HostRoles.Count == 0
        ? "Hosting a game needs the Manage Server permission."
        : $"Hosting a game needs one of these roles: {string.Join(", ", HostRoles.Select(role => $"<@&{role}>"))}.";
}
