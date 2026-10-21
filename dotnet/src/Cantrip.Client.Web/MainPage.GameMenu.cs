using Cantrip.Client;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// The game menu: a button of three lines beside the side panel's arrow - in its header while it is open, and in the
/// table's top right corner beside the arrow that opens it again while it is hidden - that opens a short menu of what a
/// player can do to the game as a whole. <b>Concede Game</b> opens the panel's game tab to ask once more, as its own button
/// does; <b>Offer Draw</b> offers the other players a draw while the player has priority, and reads <b>Accept Draw</b> while
/// another player's offer is open to them (docs/rules-exceptions.md, RX-4). A row that cannot be taken now is drawn faint.
/// <para>
/// A draw offered to this player is asked on the dock, over whatever else it is asking, until they accept or decline; one
/// declined is accepted later from the menu, at any priority of theirs. The menu's buttons are lit while an offer waits on
/// this player.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    /// <summary>The game menu's rows, opened under its button.</summary>
    private readonly CardMenu _gameMenu = new();

    /// <summary>What the game menu's button says to a page driven by a test, and to a screen reader.</summary>
    public const string GameMenuName = "Game menu";

    /// <summary>A game menu button: three lines, quiet in the panel's header and edged on the table.</summary>
    internal static DockButton GameMenuButton(bool quiet)
    {
        var button = new DockButton("≡", primary: false, fontSize: 16);
        if (quiet)
        {
            button.Quiet();
        }

        button.Tooltip = "Game menu: concede, or offer a draw";
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(button, GameMenuName);
        return button;
    }

    /// <summary>Opens the game menu under the button it was opened from.</summary>
    private void OpenGameMenu(FrameworkElement from)
    {
        GameClient? client = _gameClient;
        bool concede = client is not null && CanConcede(client);
        (string caption, bool draw) = DrawEntry(client);
        _gameMenu.Open(
            from,
            new Windows.Foundation.Point(0, from.ActualHeight + 2),
            [
                ("Concede Game", () =>
                {
                    SetSideOpen(true);
                    _side.Select(SidePanel.Tab.Menu);
                    _side.Menu.AskConcede();
                }),
                (caption, OfferOrAcceptDraw),
            ],
            enabled: [concede, draw]);
    }

    /// <summary>What the draw entry says - Offer Draw, or Accept Draw while an offer is open to this player - and whether it can be taken now.</summary>
    private static (string Caption, bool Enabled) DrawEntry(GameClient? client)
    {
        if (client is null || !client.IsConnected || client.Table is { Phase: not TablePhase.Playing })
        {
            return ("Offer Draw", false);
        }

        if (client.DrawOffer is { } offer && offer.Offerer != client.Seat && !offer.Accepted.Contains(client.Seat))
        {
            return ("Accept Draw", client.CanAcceptDraw);
        }

        return ("Offer Draw", client.CanOfferDraw);
    }

    /// <summary>Offers a draw, or accepts the one offered, whichever the entry says.</summary>
    private void OfferOrAcceptDraw()
    {
        if (_gameClient is not { } client)
        {
            return;
        }

        if (client.CanAcceptDraw)
        {
            AnswerDraw(client, accept: true);
        }
        else if (client.CanOfferDraw)
        {
            client.OfferDraw();
        }

        RedrawNow();
    }

    private void AnswerDraw(GameClient client, bool accept)
    {
        client.AnswerDraw(accept);
    }

    /// <summary>
    /// Whether a draw offered waits on this player's first answer: the client says so only until it has answered, while its
    /// answer is on its way too (<see cref="GameClient.DrawAskedOfMe"/>).
    /// </summary>
    private static bool DrawAsked(GameClient client) => client.DrawAskedOfMe;

    /// <summary>The game tab's draw button, and the game menu's buttons lit while an offer waits on this player.</summary>
    private void ShowDrawOffer(GameClient client)
    {
        (string caption, bool enabled) = DrawEntry(client);
        _side.Menu.ShowDraw(caption, enabled, DrawNote(client));
        bool waiting = DrawAsked(client);
        _side.MenuButton.Lit = waiting;
        _cornerMenu.Lit = waiting;
    }

    /// <summary>Where the draw offered stands, for the line under the game tab's button.</summary>
    private static string DrawNote(GameClient client)
    {
        if (client.IsOver || client.Table is { Phase: not TablePhase.Playing })
        {
            return string.Empty;
        }

        if (client.DrawOffer is not { } offer)
        {
            return client.CanOfferDraw ? "Offering a draw passes priority." : "A draw is offered while you have priority.";
        }

        if (offer.Offerer == client.Seat)
        {
            string declined = offer.Declined.Count == 0 ? string.Empty
                : $" Declined by {string.Join(" and ", offer.Declined.Select(seat => NameOf(client, seat)))}.";
            return "Your offer of a draw stands until you do anything but pass." + declined;
        }

        return offer.Accepted.Contains(client.Seat)
            ? $"You accepted {NameOf(client, offer.Offerer)}'s offer of a draw."
            : $"{NameOf(client, offer.Offerer)} offers a draw. Accept it at any priority of yours.";
    }

    /// <summary>
    /// The dock's question while a draw offered waits on this player's first answer: accept, or decline and play on. True when
    /// it asked it, and nothing else is to be asked on the dock until it is answered.
    /// </summary>
    private bool AskDraw(GameClient client)
    {
        if (!DrawAsked(client) || client.DrawOffer is not { } offer)
        {
            return false;
        }

        int others = client.View?.Players.Count(player => player.Seat != client.Seat && player.Status == PlayerStatusView.Active) ?? 1;
        Prompt(
            $"{NameOf(client, offer.Offerer)} offers a draw",
            others > 1 ? "The game is a draw once every other player still in it accepts." : "Accept, and the game is a draw.");
        // Neither is the dock's main button, so the Pass key - pressed out of habit - answers nothing.
        AddButton("Accept Draw", () => AnswerDraw(client, accept: true));
        AddButton("Decline", () => AnswerDraw(client, accept: false));
        ShowErrors(client);
        return true;
    }
}
