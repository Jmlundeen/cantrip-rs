using Microsoft.UI.Input;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Shapes;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// A switch drawn as a pill with a knob, lit in the accent when on. The knob slides
/// rather than jumps, so the change a click made is seen happening.
/// </summary>
public sealed class ToggleSwitchPill : Grid
{
    private const double PillWidth = 34;
    private const double PillHeight = 18;
    private const double SlideSeconds = 0.14;

    private readonly Border _pill;
    private readonly Ellipse _knob;
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(8) };
    private bool _on;
    private double _travel;
    private double _from;
    private DateTime _started;

    public ToggleSwitchPill(bool on)
    {
        _on = on;
        _travel = on ? 1 : 0;
        Width = PillWidth;
        Height = PillHeight;
        VerticalAlignment = VerticalAlignment.Center;

        _pill = new Border { CornerRadius = new CornerRadius(PillHeight / 2), BorderThickness = new Thickness(1) };
        Children.Add(_pill);

        double radius = (PillHeight / 2) - 3;
        _knob = new Ellipse { Width = radius * 2, Height = radius * 2, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Center };
        Children.Add(_knob);

        _timer.Tick += (_, _) =>
        {
            double t = Math.Min(1, (DateTime.UtcNow - _started).TotalSeconds / SlideSeconds);
            double eased = 1 - Math.Pow(1 - t, 3);
            _travel = _from + (((_on ? 1 : 0) - _from) * eased);
            Paint();
            if (t >= 1)
            {
                _timer.Stop();
            }
        };

        Paint();
    }

    public bool On
    {
        get => _on;
        set
        {
            if (_on == value)
            {
                return;
            }

            _on = value;
            _from = _travel;
            _started = DateTime.UtcNow;
            _timer.Start();
        }
    }

    private void Paint()
    {
        _pill.Background = Theme.Brush(Lerp(Theme.ControlFill, Theme.Accent, _travel));
        _pill.BorderBrush = Theme.Brush(Lerp(Theme.ControlBorder, Theme.Accent, _travel));
        _knob.Fill = Theme.Brush(Lerp(Theme.FgSoft, Theme.FgOnAccent, _travel));

        double radius = (PillHeight / 2) - 3;
        double left = 3;
        double right = PillWidth - 3 - (radius * 2);
        _knob.Margin = new Thickness(left + ((right - left) * _travel), 0, 0, 0);
    }

    internal static Color Lerp(Color from, Color to, double t) => Color.FromArgb(
        (byte)Math.Round(from.A + ((to.A - from.A) * t)),
        (byte)Math.Round(from.R + ((to.R - from.R) * t)),
        (byte)Math.Round(from.G + ((to.G - from.G) * t)),
        (byte)Math.Round(from.B + ((to.B - from.B) * t)));
}

/// <summary>
/// One on-or-off preference: its name and what it does on the left, a switch on the right, and the whole row the
/// control, so the player aims at a line of text rather than at a small switch.
/// </summary>
public sealed class SettingRow : Border
{
    private readonly ToggleSwitchPill _switch;

    public event Action<bool>? Toggled;

    public SettingRow(string title, string description, bool on)
    {
        CornerRadius = new CornerRadius(6);
        Padding = new Thickness(8);
        Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Hand);

        var row = new Grid { ColumnSpacing = 12 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Child = row;

        row.Children.Add(SettingText.Column(title, description));

        _switch = new ToggleSwitchPill(on);
        Grid.SetColumn(_switch, 1);
        row.Children.Add(_switch);

        PointerEntered += (_, _) => Background = Theme.Brush(Color.FromArgb(13, 255, 255, 255));
        PointerExited += (_, _) => Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        Tapped += (_, e) =>
        {
            e.Handled = true;
            _switch.On = !_switch.On;
            Toggled?.Invoke(_switch.On);
        };
    }
}

/// <summary>
/// A preference that is an amount: its name and current value on one line, what it does under that, and a slider the
/// width of the row, applied as it moves so its effect is seen while it is chosen.
/// </summary>
public sealed class SliderRow : Border
{
    private readonly TextBlock _amount;
    private readonly Grid _track;
    private readonly Border _fill;
    private readonly Ellipse _grabber;
    private readonly float _min;
    private readonly float _max;
    private readonly float _step;
    private readonly Func<float, string> _format;
    private float _value;
    private bool _enabled = true;
    private bool _dragging;

    public event Action<float>? ValueChanged;

    public SliderRow(string title, string description, float min, float max, float step, float value, Func<float, string> format)
    {
        _min = min;
        _max = max;
        _step = step;
        _format = format;
        _value = value;

        CornerRadius = new CornerRadius(6);
        Padding = new Thickness(8);

        var column = new StackPanel { Orientation = Orientation.Vertical, Spacing = 2 };
        Child = column;

        var head = new Grid { ColumnSpacing = 12 };
        head.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        head.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        column.Children.Add(head);

        head.Children.Add(new TextBlock { Text = title, FontSize = 13, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Fg) });
        _amount = new TextBlock { FontSize = 12, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.FgSoft), VerticalAlignment = VerticalAlignment.Center };
        Grid.SetColumn(_amount, 1);
        head.Children.Add(_amount);

        column.Children.Add(SettingText.Caption(description));

        // A thin rounded track, the part up to the value in the accent, and a grabber on it.
        _track = new Grid { Height = 22, Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0)) };
        _track.Children.Add(new Border { Height = 4, CornerRadius = new CornerRadius(2), Background = Theme.Brush(Theme.ControlFill), VerticalAlignment = VerticalAlignment.Center });
        _fill = new Border { Height = 4, CornerRadius = new CornerRadius(2), Background = Theme.Brush(Theme.Accent), HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Center };
        _track.Children.Add(_fill);
        _grabber = new Ellipse { Width = 14, Height = 14, Fill = Theme.Brush(Theme.Fg), HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Center };
        _track.Children.Add(_grabber);
        column.Children.Add(_track);

        _track.SizeChanged += (_, _) => Paint();
        _track.PointerPressed += OnPressed;
        _track.PointerMoved += OnMoved;
        _track.PointerReleased += (_, e) =>
        {
            _dragging = false;
            _track.ReleasePointerCapture(e.Pointer);
        };

        Paint();
    }

    /// <summary>
    /// Whether the amount is the player's to choose. A disabled row is dimmed and takes no drags, so it reads as
    /// something another setting is deciding rather than as a control that has stopped working.
    /// </summary>
    public bool Enabled
    {
        get => _enabled;
        set
        {
            _enabled = value;
            Opacity = value ? 1 : 0.4;
        }
    }

    private void OnPressed(object sender, PointerRoutedEventArgs e)
    {
        if (!_enabled)
        {
            return;
        }

        _dragging = _track.CapturePointer(e.Pointer);
        SetFrom(e.GetCurrentPoint(_track).Position.X);
        e.Handled = true;
    }

    private void OnMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_dragging)
        {
            SetFrom(e.GetCurrentPoint(_track).Position.X);
        }
    }

    private void SetFrom(double x)
    {
        double usable = Math.Max(1, _track.ActualWidth - _grabber.Width);
        double t = Math.Clamp((x - (_grabber.Width / 2)) / usable, 0, 1);
        float value = _min + ((_max - _min) * (float)t);
        value = MathF.Round(value / _step) * _step;
        value = Math.Clamp(value, _min, _max);

        if (value != _value)
        {
            _value = value;
            Paint();
            ValueChanged?.Invoke(value);
        }
    }

    private void Paint()
    {
        _amount.Text = _format(_value);
        double usable = Math.Max(0, _track.ActualWidth - _grabber.Width);
        double t = _max > _min ? (_value - _min) / (_max - _min) : 0;
        _grabber.Margin = new Thickness(usable * t, 0, 0, 0);
        _fill.Width = Math.Max(0, (usable * t) + (_grabber.Width / 2));
    }
}

/// <summary>
/// One hotkey: its name and what it does on the left, the key it is bound to on the right. Clicking the row waits for
/// the next key press and binds that; Escape or a click anywhere else leaves the binding as it was. A key held with
/// modifiers binds with them, and a modifier on its own is waited past, so Ctrl+Z is chosen by holding Ctrl and
/// pressing Z.
/// </summary>
public sealed class HotkeyRow : Border
{
    private readonly Hotkey _hotkey;
    private readonly string _description;
    private readonly TextBlock _caption;
    private readonly Border _chip;
    private readonly TextBlock _key;
    private readonly DispatcherTimer _settle = new() { Interval = TimeSpan.FromSeconds(4) };

    public event Action? CaptureStarted;

    public event Action? CaptureEnded;

    public HotkeyRow(Hotkey hotkey)
    {
        _hotkey = hotkey;
        _description = Hotkeys.Description(hotkey);

        CornerRadius = new CornerRadius(6);
        Padding = new Thickness(8);
        Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Hand);
        ToolTipService.SetToolTip(this, "Click, then press the key to use");

        var row = new Grid { ColumnSpacing = 12 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Child = row;

        StackPanel text = SettingText.Column(Hotkeys.Title(hotkey), _description);
        _caption = (TextBlock)text.Children[1];
        row.Children.Add(text);

        _key = new TextBlock { FontSize = 12, FontWeight = FontWeights.SemiBold, TextAlignment = TextAlignment.Center, MinWidth = 44 };
        _chip = new Border { CornerRadius = new CornerRadius(6), Padding = new Thickness(10, 4, 10, 4), VerticalAlignment = VerticalAlignment.Center, Child = _key };
        Grid.SetColumn(_chip, 1);
        row.Children.Add(_chip);

        PointerEntered += (_, _) => Background = Theme.Brush(Color.FromArgb(13, 255, 255, 255));
        PointerExited += (_, _) => Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        Tapped += (_, e) =>
        {
            e.Handled = true;
            if (!Capturing)
            {
                Capturing = true;
                Caption(_hotkey == Hotkey.Cancel ? "Press a key" : "Press a key, or Escape to cancel", Theme.Accent);
                Refresh();
                CaptureStarted?.Invoke();
            }
        };

        _settle.Tick += (_, _) =>
        {
            _settle.Stop();
            if (!Capturing)
            {
                Caption(_description, Theme.FgSoft);
            }
        };

        PlayerSettings.Changed += Refresh;
        Refresh();
    }

    /// <summary>Whether this row is waiting for a key.</summary>
    public bool Capturing { get; private set; }

    /// <summary>
    /// A key went down while this row waits: bound, refused, or waited past. The press goes no further, so choosing
    /// Space does not also pass.
    /// </summary>
    public void Take(Windows.System.VirtualKey pressed, bool ctrl, bool shift, bool alt)
    {
        Windows.System.VirtualKey key = KeyBinding.Plain(pressed);
        bool modifier = KeyBinding.IsModifier(key);

        // A hotkey that acts while held may be a bare modifier, which is what Ctrl is for holding priority. Any other
        // waits past the modifier, so Ctrl+Z can be chosen by holding Ctrl and pressing Z.
        if (modifier && !Hotkeys.IsHeld(_hotkey))
        {
            return;
        }

        if (key == Hotkeys.StopChoosing && _hotkey != Hotkey.Cancel)
        {
            End();
            return;
        }

        KeyBinding binding = modifier ? new KeyBinding(key) : new KeyBinding(key, ctrl, shift, alt);
        if (Hotkeys.WhyReserved(binding) is { } reason)
        {
            Caption(reason, Theme.Danger);
            return;
        }

        End();
        if (PlayerSettings.Bind(_hotkey, binding) is { } swapped)
        {
            Caption($"Swapped with {Hotkeys.Title(swapped)}, which is now {PlayerSettings.KeyFor(swapped)}", Theme.Accent);
            _settle.Start();
        }
    }

    /// <summary>Stops waiting, leaving the binding as it was: a click elsewhere, or Escape.</summary>
    public void End()
    {
        if (!Capturing)
        {
            return;
        }

        Capturing = false;
        Caption(_description, Theme.FgSoft);
        Refresh();
        CaptureEnded?.Invoke();
    }

    private void Caption(string text, Color color)
    {
        _caption.Text = text;
        _caption.Foreground = Theme.Brush(color);
    }

    private void Refresh()
    {
        _key.Text = Capturing ? "…" : PlayerSettings.KeyFor(_hotkey).ToString();
        _key.Foreground = Theme.Brush(Capturing ? Theme.FgOnAccent : Theme.Fg);
        _chip.Background = Theme.Brush(Capturing ? Theme.Accent : Theme.ControlFill);
        _chip.BorderBrush = Theme.Brush(Theme.ControlBorder);
        _chip.BorderThickness = new Thickness(Capturing ? 0 : 1);
    }
}

/// <summary>A setting's name over what it does, as every settings row sets them.</summary>
internal static class SettingText
{
    public static StackPanel Column(string title, string description) => new()
    {
        Orientation = Orientation.Vertical,
        Spacing = 2,
        Children =
        {
            new TextBlock { Text = title, FontSize = 13, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Fg) },
            Caption(description),
        },
    };

    /// <summary>Ui.Caption: small, soft, wrapped.</summary>
    public static TextBlock Caption(string text) => new()
    {
        Text = text,
        FontSize = 11,
        Foreground = Theme.Brush(Theme.FgSoft),
        TextWrapping = TextWrapping.Wrap,
    };
}
