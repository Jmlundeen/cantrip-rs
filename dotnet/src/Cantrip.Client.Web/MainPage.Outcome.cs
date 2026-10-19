using Cantrip.Client;
using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Microsoft.UI.Text;

namespace Cantrip.Client.Web;

/// <summary>
/// The outcome said once, large, over the board when the game ends, and put away with a button once read. The dock
/// keeps the outcome for as long as the page is up, so nothing is lost by closing it, and the final board is worth
/// looking at.
/// <para>
/// A table that names the page its players came from (the site's draft queues, its constructed play) is gone back to
/// from here (<see cref="GoingBack"/>): a button on the result does it at once, and the page does it by itself once the
/// result has been up a while, saying so under it. Putting the result away to look at the board stops that, and the game
/// tab of the side panel keeps the button.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    private Grid _outcome = null!;
    private TextBlock _outcomeTitle = null!;
    private TextBlock _outcomeDetail = null!;
    private bool _outcomeDismissed;
    private DockButton _outcomeBack = null!;
    private TextBlock _outcomeCountdown = null!;
    private readonly GoingBack _goingBack = new();

    /// <summary>
    /// A panel centred over the whole window. Only the panel takes the pointer; around it the board can still be
    /// hovered and its cards held up while the result is on screen.
    /// </summary>
    private Grid BuildOutcomeOverlay()
    {
        _outcomeTitle = new TextBlock
        {
            FontSize = 34,
            FontWeight = FontWeights.SemiBold,
            HorizontalAlignment = HorizontalAlignment.Center,
            TextAlignment = TextAlignment.Center,
        };

        _outcomeDetail = new TextBlock
        {
            FontSize = 14,
            Foreground = Theme.Brush(Theme.FgSoft),
            TextAlignment = TextAlignment.Center,
            TextWrapping = TextWrapping.WrapWholeWords,
            Width = 392,
            HorizontalAlignment = HorizontalAlignment.Center,
        };

        var close = new DockButton("Look at the board", primary: false) { HorizontalAlignment = HorizontalAlignment.Center };
        close.Clicked += () =>
        {
            _outcomeDismissed = true;
            _goingBack.Stay();
            _outcome.Visibility = Visibility.Collapsed;
            _needsRedraw = true;
        };

        _outcomeBack = new DockButton("Back", primary: true) { HorizontalAlignment = HorizontalAlignment.Center, Visibility = Visibility.Collapsed };
        _outcomeBack.Clicked += GoBack;

        _outcomeCountdown = new TextBlock
        {
            FontSize = 12,
            Foreground = Theme.Brush(Theme.FgMuted),
            TextAlignment = TextAlignment.Center,
            HorizontalAlignment = HorizontalAlignment.Center,
            Visibility = Visibility.Collapsed,
        };

        var panel = new Border
        {
            MinWidth = 460,
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(2),
            CornerRadius = new CornerRadius(12),
            Padding = new Thickness(34, 26, 34, 26),
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Child = new StackPanel { Orientation = Orientation.Vertical, Spacing = 14, Children = { _outcomeTitle, _outcomeDetail, _outcomeBack, close, _outcomeCountdown } },
        };

        _outcome = new Grid { Visibility = Visibility.Collapsed, Children = { panel } };
        return _outcome;
    }

    /// <summary>Raises the panel once the game is over, and keeps it down once it has been put away.</summary>
    private void ShowOutcome(GameClient client)
    {
        if (!client.IsOver || _outcomeDismissed)
        {
            _outcome.Visibility = Visibility.Collapsed;
            return;
        }

        _outcomeTitle.Text = DescribeOutcome(client).TrimEnd('.');
        _outcomeTitle.Foreground = Theme.Brush(
            client.Outcome!.Outcome == GameOutcomeView.Draw ? Theme.FgSoft
            : client.Outcome.WinningTeam == (client.Me?.Team ?? 0) ? Theme.Mine
            : Theme.Danger);

        _outcomeDetail.Text = WhyItEnded(client);
        _outcomeDetail.Visibility = _outcomeDetail.Text.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
        _outcome.Visibility = Visibility.Visible;
    }

    /// <summary>
    /// Follows the way back every tick, since its countdown moves with no message to redraw for: the button on the result
    /// and in the game tab, the line under the result, and going back once it is time.
    /// </summary>
    private void FollowGoingBack(GameClient client)
    {
        if (_goingBack.Follow(client.Table, client.IsOver, DateTimeOffset.UtcNow) is { } due)
        {
            TableMove.Back(due);
        }

        bool offered = _goingBack.Address is not null;
        _outcomeBack.Visibility = offered ? Visibility.Visible : Visibility.Collapsed;
        if (offered && _outcomeBack.Caption != _goingBack.Button)
        {
            _outcomeBack.Caption = _goingBack.Button;
        }

        string countdown = _goingBack.Countdown(DateTimeOffset.UtcNow);
        if (_outcomeCountdown.Text != countdown)
        {
            _outcomeCountdown.Text = countdown;
            _outcomeCountdown.Visibility = countdown.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
        }

        _side.Menu.ShowReturn(offered ? _goingBack.Button : null);
    }

    /// <summary>Goes back to the page the player came from, from either button.</summary>
    private void GoBack()
    {
        if (_goingBack.Go() is { } page)
        {
            TableMove.Back(page);
        }
    }

    /// <summary>
    /// What took each player out, one sentence each, in the order they went, and why a drawn game is drawn: the log's own
    /// lines for it, since the outcome says only which team won.
    /// </summary>
    private static string WhyItEnded(GameClient client) =>
        string.Join(
            " ",
            client.Log
                .Where(entry => entry.Kind is LogEntryKind.PlayerLost or LogEntryKind.GameDrawn)
                .Select(entry => entry.Text));
}
