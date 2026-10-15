using Cantrip.Protocol;

namespace Cantrip.Client.Presentation;

/// <summary>
/// What the browser tab says the page is showing: "Cantrip - Winston Draft (evt-12)", "Cantrip - Game vs Bob", so a
/// player with several tabs open can tell them apart.
/// </summary>
public static class PageTitle
{
    public const string App = "Cantrip";

    /// <param name="table">The table as last heard; null before the first word from it.</param>
    /// <param name="seat">The seat this page plays, to leave out of the names of the opponents.</param>
    public static string For(TableMessage? table, int seat, DraftMessage? draft, DeckBuildingMessage? building, PairingsMessage? pairings)
    {
        if (table is null)
        {
            return App;
        }

        string page = table.Phase switch
        {
            TablePhase.Drafting => draft is null ? "Draft" : DraftName(draft.Format),
            TablePhase.DeckBuilding => building is { Title.Length: > 0 } ? "Sideboarding" : "Deck Building",
            TablePhase.Pairings => pairings is null ? "Pairings"
                : pairings.Final ? "Final Standings"
                : $"Round {pairings.Round} of {pairings.Rounds}",
            TablePhase.Playing => Game(table, seat),
            TablePhase.Finished => "Game Over",
            TablePhase.Abandoned => "Table Closed",
            _ => table.Forming ? "Forming Table" : "Lobby",
        };

        return table.EventId.Length > 0 ? $"{App} - {page} ({table.EventId})" : $"{App} - {page}";
    }

    private static string DraftName(string format) => format switch
    {
        "winston" => "Winston Draft",
        "grid" => "Grid Draft",
        "booster" => "Booster Draft",
        _ => "Draft",
    };

    private static string Game(TableMessage table, int seat)
    {
        string[] opponents = [.. table.Players.Where(player => player.Seat != seat && player.Name.Length > 0).Select(player => player.Name)];
        return opponents.Length > 0 ? $"Game vs {string.Join(", ", opponents)}" : "Game";
    }
}
