using Microsoft.UI.Xaml.Shapes;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// A button drawn in the client's own palette: the accent fill for the answer a prompt expects, a quiet control fill
/// with a hairline for anything beside it, and a darker, muted one while it cannot be pressed. Not the stock Button, whose template repaints its own hover and pressed states over any
/// background it is given. The caption is a <see cref="SymbolText"/>, so "Pay {2}" shows a pip.
/// </summary>
public sealed class DockButton : Border
{
    private static readonly Color DisabledFill = Theme.FromHex("#1f232a");
    private static readonly Color DisabledBorder = Theme.FromHex("#2a2f38");

    private readonly SymbolText _label;
    private readonly bool _primary;
    private bool _enabled = true;
    private bool _hovered;
    private bool _pressed;

    public DockButton(string caption, bool primary, double fontSize = 13, bool enabled = true)
    {
        _primary = primary;
        _enabled = enabled;
        CornerRadius = new CornerRadius(6);
        HorizontalAlignment = HorizontalAlignment.Left;
        VerticalAlignment = VerticalAlignment.Top;

        // A caption with symbols in it is set semibold; a plain one is the theme's face, made semibold on the accent
        // button.
        bool semibold = primary && enabled || caption.Contains('{', StringComparison.Ordinal);
        _label = new SymbolText(fontSize, Theme.Fg, semibold) { Text = caption, VerticalAlignment = VerticalAlignment.Center };
        Child = _label;

        PointerEntered += (_, _) => { _hovered = true; Restyle(); };
        PointerExited += (_, _) => { _hovered = false; _pressed = false; Restyle(); };
        PointerPressed += (_, _) => { _pressed = true; Restyle(); };
        PointerReleased += (_, _) => { _pressed = false; Restyle(); };
        Tapped += (_, e) =>
        {
            e.Handled = true;
            if (_enabled)
            {
                Clicked?.Invoke();
            }
        };

        Restyle();
    }

    public event Action? Clicked;

    /// <summary>
    /// Stretches the button across its column with the caption centred. Returns the button, so it composes.
    /// </summary>
    public DockButton Filling()
    {
        HorizontalAlignment = HorizontalAlignment.Stretch;
        _label.HorizontalAlignment = HorizontalAlignment.Center;
        return this;
    }

    private Shape? _arrow;

    /// <summary>
    /// The caption replaced by an arrow drawn as a shape, pointing right or down: a glyph such as ▸ is not in every
    /// device's fonts (an iPad's browser draws none for it), and a line's button with nothing on it says nothing.
    /// Returns the button, so it composes.
    /// </summary>
    public DockButton Arrow(bool down, double size = 14)
    {
        double half = size / 2;
        _arrow = new Polygon
        {
            Points = down
                ? [new Windows.Foundation.Point(0, 0), new Windows.Foundation.Point(size, 0), new Windows.Foundation.Point(half, size * 0.62)]
                : [new Windows.Foundation.Point(0, 0), new Windows.Foundation.Point(size * 0.62, half), new Windows.Foundation.Point(0, size)],
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            IsHitTestVisible = false,
        };
        Child = _arrow;
        Restyle();
        return this;
    }

    /// <summary>
    /// The caption replaced by three bars drawn as a shape: the mark of a menu that slides out, which no font need have.
    /// Returns the button, so it composes.
    /// </summary>
    public DockButton Bars(double size = 18)
    {
        var bars = new Microsoft.UI.Xaml.Media.GeometryGroup();
        double thick = Math.Max(2, Math.Round(size / 7));
        foreach (double at in (double[])[0, (size - thick) / 2, size - thick])
        {
            bars.Children.Add(new Microsoft.UI.Xaml.Media.RectangleGeometry { Rect = new Windows.Foundation.Rect(0, Math.Round(at), size, thick) });
        }

        _arrow = new Microsoft.UI.Xaml.Shapes.Path
        {
            Data = bars,
            Width = size,
            Height = size,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            IsHitTestVisible = false,
        };
        Child = _arrow;
        Restyle();
        return this;
    }

    public string Caption
    {
        get => _label.Text;
        set => _label.Text = value;
    }

    public bool Enabled
    {
        get => _enabled;
        set
        {
            if (_enabled != value)
            {
                _enabled = value;
                Restyle();
            }
        }
    }

    public string? Tooltip
    {
        set => ToolTipService.SetToolTip(this, value is { Length: > 0 } ? value : null);
    }

    private bool _quiet;
    private bool _on;

    /// <summary>Held down while what it opens is open.</summary>
    public bool On
    {
        get => _on;
        set
        {
            if (_on != value)
            {
                _on = value;
                Restyle();
            }
        }
    }

    private bool _lit;

    /// <summary>Edged in the ready colour: there is something behind it to do, as a card outlined on the table has.</summary>
    public bool Lit
    {
        get => _lit;
        set
        {
            if (_lit != value)
            {
                _lit = value;
                Restyle();
            }
        }
    }

    /// <summary>
    /// A button with no ground or edge until it is pointed at, and softer type that brightens then: for the
    /// side panel's hide arrow and small actions that should not compete with the page. Returns the button.
    /// </summary>
    public DockButton Quiet()
    {
        _quiet = true;
        Restyle();
        return this;
    }

    private void Restyle()
    {
        if (_quiet)
        {
            Padding = new Thickness(8, 3, 8, 3);
            BorderThickness = new Thickness(0);
            Background = Theme.Brush(_enabled && _pressed ? Color.FromArgb(31, 255, 255, 255)
                : _enabled && _hovered ? Color.FromArgb(20, 255, 255, 255)
                : Color.FromArgb(0, 0, 0, 0));
            _label.Color = !_enabled ? Theme.FgMuted : _hovered || _pressed ? Theme.Fg : Theme.FgSoft;
            return;
        }

        // The accent style is only put on a button that can be pressed; a disabled answer looks like any disabled button.
        bool accent = _primary && _enabled;

        // The accent button: no hairline and 14 across; the plain one: a hairline and 12 across, so the same outer size.
        Padding = accent ? new Thickness(14, 6, 14, 6) : new Thickness(12, 6, 12, 6);
        BorderThickness = new Thickness(accent ? 0 : 1);

        Color fill;
        if (!_enabled)
        {
            fill = DisabledFill;
        }
        else if (accent)
        {
            fill = _pressed ? Darkened(Theme.Accent, 0.12) : _hovered ? Lightened(Theme.Accent, 0.1) : Theme.Accent;
        }
        else
        {
            fill = _pressed || _on ? Theme.ControlPressed : _hovered ? Theme.ControlHover : Theme.ControlFill;
        }

        Background = Theme.Brush(fill);
        BorderBrush = Theme.Brush(!_enabled ? DisabledBorder : _lit ? Theme.Ready : _on ? Theme.Accent : Theme.ControlBorder);
        _label.Color = accent ? Theme.FgOnAccent : _enabled ? Theme.Fg : Theme.FgMuted;
        if (_arrow is not null)
        {
            _arrow.Fill = Theme.Brush(_label.Color);
        }
    }

    private static Color Lightened(Color c, double by) => Color.FromArgb(
        c.A,
        (byte)Math.Round(c.R + ((255 - c.R) * by)),
        (byte)Math.Round(c.G + ((255 - c.G) * by)),
        (byte)Math.Round(c.B + ((255 - c.B) * by)));

    private static Color Darkened(Color c, double by) => Color.FromArgb(
        c.A,
        (byte)Math.Round(c.R * (1 - by)),
        (byte)Math.Round(c.G * (1 - by)),
        (byte)Math.Round(c.B * (1 - by)));
}
