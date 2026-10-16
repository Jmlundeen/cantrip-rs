using Microsoft.UI.Xaml.Controls.Primitives;
using Windows.Foundation;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The menu a card opens when it has more than one thing to offer or an ability that cannot be taken back: a panel of rows at the pointer, each a caption with its mana symbols drawn as pips, so "Add
/// {B}{B}{B}" shows three black pips. Dismissing it does nothing, which is the way out.
/// </summary>
public sealed class CardMenu
{
    private readonly Popup _popup = new() { IsLightDismissEnabled = true };
    private readonly StackPanel _rows = new() { Orientation = Orientation.Vertical };

    public CardMenu()
    {
        _popup.Child = new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(4),
            Child = _rows,
        };
    }

    public bool IsOpen => _popup.IsOpen;

    /// <summary>Opens at a point in <paramref name="over"/>'s coordinates, with one row per choice.</summary>
    /// <param name="fontSize">The rows' type size: 13 for a card's menu, 12 for a drop-down's list.</param>
    public void Open(UIElement over, Point at, IReadOnlyList<(string Text, Action OnChosen)> choices, double fontSize = 13, IReadOnlyList<int>? abilities = null)
    {
        ArgumentNullException.ThrowIfNull(choices);

        _rows.Children.Clear();

        int index = 0;
        foreach ((string text, Action onChosen) in choices)
        {
            var row = new Border
            {
                CornerRadius = new CornerRadius(4),
                Padding = new Thickness(12, 5, 12, 5),
                Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0)),
                Child = new SymbolText(fontSize, Theme.Fg) { Text = text },
                // Which ability of the card the row is, for a page driven by a test to tell rows that read alike.
                Tag = abilities is not null && index < abilities.Count ? abilities[index] : -1,
            };

            row.PointerEntered += (_, _) => row.Background = Theme.Brush(Theme.ControlHover);
            row.PointerExited += (_, _) => row.Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
            row.PointerPressed += (_, _) => row.Background = Theme.Brush(Theme.ControlPressed);

            // Closed before the action runs, since the action redraws.
            row.Tapped += (_, e) =>
            {
                e.Handled = true;
                _popup.IsOpen = false;
                onChosen();
            };

            _rows.Children.Add(row);
            index++;
        }

        _popup.XamlRoot = over.XamlRoot;
        Point origin = over.TransformToVisual(null).TransformPoint(at);
        _popup.HorizontalOffset = origin.X;
        _popup.VerticalOffset = origin.Y;
        _popup.IsOpen = true;

        // Kept on the window: a menu opened low on the screen, over the hand, goes up from the pointer instead.
        _popup.Child.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
        Size size = _popup.Child.DesiredSize;
        Rect bounds = over.XamlRoot?.Content is FrameworkElement root ? new Rect(0, 0, root.ActualWidth, root.ActualHeight) : new Rect(0, 0, 1e6, 1e6);
        _popup.HorizontalOffset = Math.Max(0, Math.Min(origin.X, bounds.Width - size.Width));
        _popup.VerticalOffset = origin.Y + size.Height > bounds.Height ? Math.Max(0, origin.Y - size.Height) : origin.Y;
    }

    public void Close() => _popup.IsOpen = false;
}
