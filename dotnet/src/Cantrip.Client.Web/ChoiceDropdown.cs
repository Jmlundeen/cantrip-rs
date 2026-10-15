using Microsoft.UI.Input;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A small captioned drop-down for the row over the choice window's cards - the library search's Sort and Group by -
/// a caption, and a button a line of text tall showing the item picked, whose list
/// opens under it. Sized to sit beside a <see cref="DockToggle"/> as a quiet strip of view settings.
/// </summary>
public sealed class ChoiceDropdown : StackPanel
{
    private const double FontSize = 11;

    private readonly IReadOnlyList<string> _items;
    private readonly Border _button;
    private readonly TextBlock _shown;
    private readonly CardMenu _menu = new();
    private bool _hovered;

    public ChoiceDropdown(string caption, string tooltip, IReadOnlyList<string> items)
    {
        _items = items;
        Orientation = Orientation.Horizontal;
        Spacing = 5;
        VerticalAlignment = VerticalAlignment.Center;
        ToolTipService.SetToolTip(this, tooltip);

        Children.Add(new TextBlock { Text = caption, FontSize = FontSize, Foreground = Theme.Brush(Theme.FgSoft), VerticalAlignment = VerticalAlignment.Center });

        _shown = new TextBlock { FontSize = FontSize, Foreground = Theme.Brush(Theme.Fg), VerticalAlignment = VerticalAlignment.Center };
        var arrow = new TextBlock { Text = "▾", FontSize = FontSize, Foreground = Theme.Brush(Theme.FgSoft), VerticalAlignment = VerticalAlignment.Center };

        // Fits its longest item, so picking one does not move what stands beside it.
        var measure = new TextBlock { FontSize = FontSize };
        double widest = 0;
        foreach (string item in items)
        {
            measure.Text = item;
            measure.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
            widest = Math.Max(widest, measure.DesiredSize.Width);
        }

        _shown.MinWidth = Math.Ceiling(widest);

        _button = new Border
        {
            CornerRadius = new CornerRadius(4),
            BorderThickness = new Thickness(1),
            Padding = new Thickness(8, 1, 8, 1),
            VerticalAlignment = VerticalAlignment.Center,
            Child = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6, Children = { _shown, arrow } },
        };
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Hand);
        Children.Add(_button);

        _button.PointerEntered += (_, _) => { _hovered = true; Paint(false); };
        _button.PointerExited += (_, _) => { _hovered = false; Paint(false); };
        _button.PointerPressed += (_, _) => Paint(true);
        _button.Tapped += (_, e) =>
        {
            e.Handled = true;
            Paint(false);
            Open();
        };

        SetSelected(0);
        Paint(false);
    }

    /// <summary>The part that opens the list when pressed, for a test to press (<see cref="MainPage"/>'s ?test state).</summary>
    internal FrameworkElement Button => _button;

    /// <summary>Called with the index of the item the player picked.</summary>
    public Action<int>? Changed { get; set; }

    public int Selected { get; private set; }

    /// <summary>Shows an item without saying so, for a change that did not come from the player.</summary>
    public void SetSelected(int index)
    {
        Selected = Math.Clamp(index, 0, _items.Count - 1);
        _shown.Text = _items[Selected];
    }

    /// <summary>The list, under the button, one row per item.</summary>
    private void Open()
    {
        List<(string, Action)> rows = [];
        for (int i = 0; i < _items.Count; i++)
        {
            int index = i;
            rows.Add((_items[index], () =>
            {
                SetSelected(index);
                Changed?.Invoke(index);
            }));
        }

        _menu.Open(_button, new Point(0, _button.ActualHeight + 2), rows, fontSize: 12);
    }

    private void Paint(bool pressed)
    {
        _button.Background = Theme.Brush(pressed ? Theme.ControlPressed : _hovered ? Theme.ControlHover : Theme.ControlFill);
        _button.BorderBrush = Theme.Brush(_hovered ? Theme.FgMuted : Theme.ControlBorder);
    }
}
