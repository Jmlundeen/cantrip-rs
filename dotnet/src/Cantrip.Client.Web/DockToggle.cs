using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// A small checkbox and its label, as drawn in the dock: filled in the accent with a
/// tick when on, an outlined control square when off, and the label lit while on or pointed at.
/// </summary>
public sealed class DockToggle : Border
{
    private const double BoxSize = 11;

    private readonly Border _box;
    private readonly Polyline _tick;
    private readonly TextBlock _label;
    private bool _on;
    private bool _hovered;

    public DockToggle(string label)
    {
        Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        MinHeight = 18;

        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 5 };
        Child = row;

        _tick = new Polyline
        {
            Points = { new Point(2.6, 5.6), new Point(4.6, 7.6), new Point(8.4, 3.4) },
            StrokeThickness = 1.6,
            Stroke = Theme.Brush(Theme.FgOnAccent),
            Visibility = Visibility.Collapsed,
        };

        _box = new Border
        {
            Width = BoxSize,
            Height = BoxSize,
            CornerRadius = new CornerRadius(3),
            VerticalAlignment = VerticalAlignment.Center,
            Child = new Canvas { Children = { _tick } },
        };

        row.Children.Add(_box);

        _label = new TextBlock { Text = label, FontSize = 11, VerticalAlignment = VerticalAlignment.Center };
        row.Children.Add(_label);

        PointerEntered += (_, _) =>
        {
            _hovered = true;
            Restyle();
        };

        PointerExited += (_, _) =>
        {
            _hovered = false;
            Restyle();
        };

        Tapped += (_, _) =>
        {
            _on = !_on;
            Restyle();
            Toggled?.Invoke(_on);
        };

        Restyle();
    }

    public event Action<bool>? Toggled;

    public bool On => _on;

    /// <summary>What the box does, ending with its key, so it is written again when the key is rebound.</summary>
    public string Tooltip
    {
        get => ToolTipService.GetToolTip(this) as string ?? string.Empty;
        set => ToolTipService.SetToolTip(this, value);
    }

    /// <summary>Clicks the box: turns it over and raises <see cref="Toggled"/>, as its hotkey does.</summary>
    public void Flip()
    {
        _on = !_on;
        Restyle();
        Toggled?.Invoke(_on);
    }

    /// <summary>Sets the box without raising <see cref="Toggled"/>: for the table turning a setting off by itself.</summary>
    public void SetOn(bool on)
    {
        _on = on;
        Restyle();
    }

    private void Restyle()
    {
        if (_on)
        {
            Color fill = _hovered ? Lighten(Theme.Accent, 0.1) : Theme.Accent;
            _box.Background = Theme.Brush(fill);
            _box.BorderThickness = new Thickness(0);
        }
        else
        {
            _box.Background = Theme.Brush(_hovered ? Theme.ControlHover : Theme.ControlFill);
            _box.BorderBrush = Theme.Brush(_hovered ? Theme.FgMuted : Theme.ControlBorder);
            _box.BorderThickness = new Thickness(1);
        }

        _tick.Visibility = _on ? Visibility.Visible : Visibility.Collapsed;
        _label.Foreground = Theme.Brush(_on || _hovered ? Theme.Fg : Theme.FgSoft);
    }

    private static Color Lighten(Color color, double amount) => Color.FromArgb(
        color.A,
        (byte)(color.R + ((255 - color.R) * amount)),
        (byte)(color.G + ((255 - color.G) * amount)),
        (byte)(color.B + ((255 - color.B) * amount)));
}
