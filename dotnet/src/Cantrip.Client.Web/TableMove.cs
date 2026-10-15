using System.Runtime.InteropServices.JavaScript;

namespace Cantrip.Client.Web;

/// <summary>
/// Goes to another table, when the server has moved this player's seat there (Ticket.js): the page loads again at that
/// table's address with the ticket for the seat, as it would from a link.
/// </summary>
public static partial class TableMove
{
    [JSImport("globalThis.cantripGoToTable")]
    private static partial void GoToTable(string server, string ticket);

    public static void Go(string server, string ticket)
    {
        try
        {
            GoToTable(server, ticket);
        }
        catch (JSException)
        {
            // A page without the script - a test harness of its own - simply stays where it is.
        }
    }
}
