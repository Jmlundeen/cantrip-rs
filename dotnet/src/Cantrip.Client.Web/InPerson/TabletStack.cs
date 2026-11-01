using Cantrip.Protocol;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Media;

namespace Cantrip.Client.Web.InPerson;

/// <summary>
/// The stack on a tablet: one window over the board, shown only while something is on the stack, so the halves keep the
/// whole height and never move as a spell is cast. Its spells and abilities are an overlapping row, oldest at the left and
/// the top of the stack whole at the right; each is turned to face the player who controls it and set off toward that
/// player's side, so the row reads as whose is whose. Its caps say how much is on it and fold it to just them - one cap at
/// each end on the shared board, the right-hand one turned for the player across the table. Its cards are tapped as a
/// battlefield's are: a spell can be a target.
/// </summary>
public sealed class TabletStack : Grid
{
    /// <summary>How far the window keeps its cards from its edges.</summary>
    private const float Inset = 8f;

    /// <summary>How far a spell is set off toward its controller's side, as a share of a card's height.</summary>
    private const float Offset = 0.16f;

    private readonly List<(TextBlock Title, CollapseButton Toggle)> _caps = [];
    private readonly Border _cards;
    private bool _collapsed;

    /// <param name="mirrored">The shared board's: a cap at each end, the right-hand one turned to face seat 2.</param>
    public TabletStack(bool mirrored)
    {
        HorizontalAlignment = HorizontalAlignment.Center;
        VerticalAlignment = VerticalAlignment.Center;
        Visibility = Visibility.Collapsed;

        var window = new Border
        {
            Background = Theme.Brush(Theme.WithAlpha(Theme.Panel, 0.88)),
            BorderBrush = Theme.Brush(Theme.ControlBorder),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(10),
            Padding = new Thickness(6),
        };
        Children.Add(window);

        var row = new Grid { ColumnSpacing = 4 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        window.Child = row;

        row.Children.Add(Cap(turned: false));

        Board = new CardBoardElement(60, 84)
        {
            Slides = PlayerSettings.AnimateCards,
            HorizontalAlignment = HorizontalAlignment.Left,
            VerticalAlignment = VerticalAlignment.Center,
        };
        _cards = new Border { Padding = new Thickness(Inset), Child = Board };
        Grid.SetColumn(_cards, 1);
        row.Children.Add(_cards);

        if (mirrored)
        {
            FrameworkElement turned = Cap(turned: true);
            Grid.SetColumn(turned, 2);
            row.Children.Add(turned);
        }
    }

    public CardBoardElement Board { get; }

    /// <summary>Whether the window is folded to its caps.</summary>
    public bool Collapsed => _collapsed;

    /// <summary>The window's caps, for a test to find its fold by: each one's box is the button.</summary>
    public IEnumerable<CollapseButton> Toggles => _caps.Select(cap => cap.Toggle);

    /// <summary>A cap: how much is on the stack, over the button that folds it, turned for the player across the table.</summary>
    private FrameworkElement Cap(bool turned)
    {
        var title = new TextBlock
        {
            FontSize = 15,
            FontWeight = FontWeights.SemiBold,
            Foreground = Theme.Brush(Theme.Fg),
            TextAlignment = TextAlignment.Center,
            HorizontalAlignment = HorizontalAlignment.Center,
        };
        var toggle = new CollapseButton { Width = 44, Height = 40, HorizontalAlignment = HorizontalAlignment.Center };
        _caps.Add((title, toggle));

        var cap = new StackPanel
        {
            Spacing = 4,
            Padding = new Thickness(6, 4, 6, 4),
            VerticalAlignment = VerticalAlignment.Center,
            Background = Theme.Brush(Windows.UI.Color.FromArgb(0, 0, 0, 0)),
            Children = { title, toggle },
        };
        cap.Tapped += (_, e) =>
        {
            e.Handled = true;
            Fold(!_collapsed);
        };

        if (turned)
        {
            cap.RenderTransformOrigin = new Windows.Foundation.Point(0.5, 0.5);
            cap.RenderTransform = new RotateTransform { Angle = 180 };
        }

        return cap;
    }

    /// <summary>Folds the window to its caps, or opens it again.</summary>
    public void Fold(bool collapsed)
    {
        _collapsed = collapsed;
        _cards.Visibility = collapsed ? Visibility.Collapsed : Visibility.Visible;

        // The cards themselves too: folded, none of them is anywhere to be tapped, flown to or pointed from.
        Board.Visibility = _cards.Visibility;
        foreach ((_, CollapseButton toggle) in _caps)
        {
            toggle.Collapsed = collapsed;
        }
    }

    /// <summary>
    /// Lays the stack out, oldest first as the view lists it: shown only while there is something on it, each card turned
    /// to face its controller and set off toward their side of the board.
    /// </summary>
    /// <param name="cardHeight">How tall a card on it is drawn.</param>
    /// <param name="room">The most width the cards may take.</param>
    /// <param name="turnedSeat">The seat whose side of the board is turned, to face them across the table; zero for none.</param>
    /// <param name="upSeat">The seat whose side is toward the board's top edge; spells of every other seat go toward the bottom.</param>
    public void Show(IReadOnlyList<ObjectView> stack, float cardHeight, float room, int turnedSeat, int upSeat)
    {
        Visibility = stack.Count > 0 ? Visibility.Visible : Visibility.Collapsed;
        foreach ((TextBlock title, _) in _caps)
        {
            title.Text = $"Stack\n{stack.Count}";
        }

        float height = MathF.Max(40f, MathF.Round(cardHeight));
        float width = MathF.Round(height * 63f / 88f);
        float offset = MathF.Round(height * Offset);
        Board.SetCellSize(width, height);
        Board.Height = height + offset;

        // Each card whole while they fit; past that, closer together, the top of the stack still whole at the right.
        room = MathF.Max(width, room - (2f * Inset));
        float step = stack.Count <= 1 ? width : MathF.Min(width + 10f, (room - width) / (stack.Count - 1));

        List<CardBoardElement.BoardCell> cells = [];
        float right = 0f;
        for (int i = 0; i < stack.Count; i++)
        {
            float x = i * step;
            int controller = stack[i].Controller;
            cells.Add(new CardBoardElement.BoardCell(
                stack[i],
                x,
                controller == upSeat ? 0f : offset,
                Tapped: false,
                Turn: turnedSeat != 0 && controller == turnedSeat ? 180f : 0f));
            right = x + width;
        }

        Board.Width = MathF.Max(1f, right);
        Board.SetCells(cells);
    }
}
