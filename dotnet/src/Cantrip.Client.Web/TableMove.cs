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

    [JSImport("globalThis.cantripGoBack")]
    private static partial void GoBack(string page);

    /// <summary>
    /// Leaves the table for the page the player came to it from, once its game is over (TableMessage.Return), as a link
    /// to it would: the table stays in the tab's history.
    /// </summary>
    public static void Back(Uri page)
    {
        try
        {
            GoBack(page.AbsoluteUri);
        }
        catch (JSException)
        {
            // A page without the script stays where it is.
        }
    }

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
