namespace Cantrip.Server.Events;

/// <summary>
/// A player as a page lists them: the name they go by, their picture (<see cref="PortraitStore.PathFor"/>, null for none -
/// a bot, or a player with no picture kept), and - for a player who came from Discord - their Discord account's id, which
/// a page links to a message to them by.
/// </summary>
public sealed record PlayerFace(string Name, string? Picture = null, string? Discord = null)
{
    /// <summary>The prefix of a Discord player's id: <c>discord-{account}</c>.</summary>
    public const string DiscordPrefix = "discord-";

    /// <summary>
    /// The player with the id <paramref name="playerId"/> (null or empty for one not known by id), as the name
    /// <paramref name="name"/>; <paramref name="portraits"/> is each player's picture by id (<see cref="PortraitStore.PathFor"/>).
    /// </summary>
    public static PlayerFace Of(string? playerId, string name, Func<string, string?>? portraits) =>
        new(name, playerId is { Length: > 0 } && portraits is not null ? portraits(playerId) : null, DiscordOf(playerId));

    /// <summary>The Discord account a player id is (<c>discord-{account}</c>), as text; null for any other id.</summary>
    public static string? DiscordOf(string? playerId) =>
        playerId is not null && playerId.StartsWith(DiscordPrefix, StringComparison.Ordinal)
            && ulong.TryParse(playerId.AsSpan(DiscordPrefix.Length), System.Globalization.NumberStyles.None, System.Globalization.CultureInfo.InvariantCulture, out ulong account)
            ? account.ToString(System.Globalization.CultureInfo.InvariantCulture)
            : null;
}
