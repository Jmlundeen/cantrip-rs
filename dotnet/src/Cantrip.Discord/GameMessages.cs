using Cantrip.Server.Events;

namespace Cantrip.Discord;

/// <summary>
/// What the bot says, as Discord markdown. Kept apart from the adapter so the words can be read and tested without a
/// Discord connection. Players are written by the display name they had when they sat down (see <see cref="Name"/>),
/// and the adapter still sends every message with mentions suppressed, so nothing here can ping anyone.
/// </summary>
internal static class GameMessages
{
    /// <summary>
    /// A player by name, in bold. Not a mention: the messages suppress mentions so nobody is pinged, and a suppressed
    /// mention also leaves the message without the user it names, so any client that has not loaded that member shows
    /// "@unknown-user" instead.
    /// </summary>
    public static string Name(DiscordPlayer player) => $"**{Escape(player.Name)}**";

    /// <summary>A name as literal text: every character Discord markdown gives a meaning is escaped.</summary>
    internal static string Escape(string text)
    {
        var escaped = new System.Text.StringBuilder(text.Length);
        foreach (char character in text)
        {
            if (character is '\\' or '*' or '_' or '~' or '`' or '|' or '>' or '#' or '-' or '[' or ']' or '(' or ')' or ':' or '<' or '@')
                escaped.Append('\\');
            escaped.Append(character);
        }

        return escaped.ToString();
    }

    /// <summary>The public message on a game, as it stands.</summary>
    public static string Table(HostedGame game, string deck)
    {
        string host = Name(game.Host);
        string both = game.Guest is { } guest ? $"{host} vs {Name(guest)}" : host;
        return game.Stage switch
        {
            GameStage.Open => $"{host} is hosting a Cantrip game. Both players play **{deck}**.\nPress **Join** to take the other seat.",
            GameStage.Seated => $"{both} · **{deck}** mirror\nWaiting for both players to connect.",
            GameStage.Playing => $"{both} · **{deck}** mirror\nIn progress.",
            _ => $"{both} · **{deck}** mirror\n{Ending(game)}",
        };
    }

    private static string Ending(HostedGame game) => game.Ending switch
    {
        GameEnding.Won => $"{string.Join(" and ", game.Winners.Select(Name))} won.",
        GameEnding.Draw => "The game was a draw.",
        GameEnding.Abandoned => "Abandoned" + (game.Detail.Length > 0 ? $": {game.Detail}" : "."),
        GameEnding.Cancelled => $"Cancelled by {(game.CancelledBy is { } by ? Name(by) : "its host")}.",
        GameEnding.Expired => "Expired: the game never started.",
        GameEnding.Failed => "The table failed. The host's log has the details.",
        _ => "Ended.",
    };

    /// <summary>The public message on a draft, as it stands.</summary>
    public static string Draft(HostedDraft draft, string cube)
    {
        string host = Name(draft.Host);
        string both = draft.Guest is { } guest ? $"{host} vs {Name(guest)}" : host;
        string heading = $"Winston draft · **{Escape(cube)}**";
        return draft.Stage switch
        {
            EventStage.NotActive when draft.Guest is null =>
                $"{host} is opening a {heading}.\nPress **Join draft** to draft against them.",
            EventStage.NotActive => $"{both} · {heading}\n{Escape(draft.Detail)}",
            EventStage.Seating => $"{both} · {heading}\nBoth players are in. Press **My ticket** to open your seat; the draft starts when you are both there.",
            EventStage.Drafting => $"{both} · {heading}\nDrafting. **My ticket** opens your seat again, from any computer.",
            EventStage.DeckBuilding => $"{both} · {heading}\nBuilding decks. **My ticket** opens your seat again, from any computer.",
            EventStage.Playing => $"{both} · {heading}\nPlaying. **My ticket** opens your seat again, from any computer.",
            _ => $"{both} · {heading}\n{Escape(draft.Detail)}",
        };
    }

    /// <summary>A queue's message: what it drafts, how full the table being filled is, and what else is going on.</summary>
    public static string Queue(QueueStatus status, string cube)
    {
        QueueSettings settings = status.Settings;
        string cards = settings.Cards == 0 ? "the whole cube" : $"{settings.Cards} cards";
        List<string> lines = settings.Format switch
        {
            DraftFormats.Booster =>
            [
                $"## Booster draft \u00b7 {Escape(cube)}",
                $"A pod of {settings.Seats}: {settings.PacksDrafted} packs of {settings.PackSize}, then {settings.Rounds} Swiss rounds of best-of-three; decks of at least {settings.MinimumDeck}",
                AtTheTable(status),
            ],
            DraftFormats.Grid =>
            [
                $"## Grid draft \u00b7 {Escape(cube)}",
                $"{settings.PacksDrafted} grids of nine, a row or a column each; decks of at least {settings.MinimumDeck}",
                AtTheTable(status),
            ],
            _ =>
            [
                $"## Winston draft \u00b7 {Escape(cube)}",
                $"{cards}, {settings.Piles} piles, decks of at least {settings.MinimumDeck}",
                AtTheTable(status),
            ],
        };

        List<string> also = [];
        if (status.Checking > 0) also.Add($"{status.Checking} table{(status.Checking == 1 ? string.Empty : "s")} getting ready");
        if (status.Underway > 0) also.Add($"{status.Underway} draft{(status.Underway == 1 ? string.Empty : "s")} under way");
        if (also.Count > 0) lines.Add(string.Join(" \u00b7 ", also));
        lines.Add("Press **Join** for a seat. Your table opens in your browser; when every seat is taken, everyone has 30 seconds to press **Ready!**");
        return string.Join("\n", lines);
    }

    /// <summary>How full the table being filled is, and the seats its bots play.</summary>
    private static string AtTheTable(QueueStatus status)
    {
        string line = $"Players at the table: **{status.Waiting}/{status.Seats}**";
        return status.Settings.Bots switch
        {
            0 => line,
            1 when status.Seats == 1 => line + ", and a bot in the other seat",
            1 => line + ", and a bot",
            int bots => line + $", and {bots} bots",
        };
    }

    /// <summary>The private reply to Join: the table's link is in the button.</summary>
    public static string QueueTicket(QueueSeating seating) => $"""
        You have seat {seating.Seat} at a table.
        Press **Open in Cantrip** below to go to it. The button's link is your seat, so do not share it.
        Press **Join** again any time for a new link to the same seat.
        """;

    /// <summary>The private reply that takes a player back to their booster draft, wherever in it they should be.</summary>
    public static string PodTicket(PodSeating seating) => $"""
        You have seat {seating.Seat} in your booster draft.
        Press **Open in Cantrip** below to go back to it: the draft, the pairings, or the table of your match, wherever you should be.
        The button's link is your seat, so do not share it. Your old links keep working.
        """;

    /// <summary>The private reply that seats a player in a draft. As with a game, the ticket is only in the button.</summary>
    public static string DraftTicket(DraftSeating seating) => $"""
        You have seat {seating.Seat} in this draft.
        Press **Open in Cantrip** below to go to it. The button's link is your seat, so do not share it.
        Press **My ticket** on the draft any time for a new link: it works from any computer, and your old links keep working.
        """;

    /// <summary>
    /// The private reply that seats a player. The ticket itself is only in the reply's <b>Open in Cantrip</b> button,
    /// which the adapter attaches; it is a bearer credential, so the reply only ever goes to its owner, ephemerally.
    /// </summary>
    public static string Ticket(Seating seating)
    {
        string opening = seating.Again
            ? $"You have seat {seating.Seat} at this game."
            : seating.Seat == 1
                ? "Your game is up. You have seat 1; the other seat opens to whoever presses **Join**."
                : "You have seat 2.";
        return $"""
            {opening}
            Press **Open in Cantrip** below to play. The button's link is your seat, so do not share it.
            If your client closes, press it again to get back in.
            """;
    }
}
