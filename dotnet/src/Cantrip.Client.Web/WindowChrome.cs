using Microsoft.UI.Input;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The chevron that folds a floating window up to its title bar: a quiet 28 by 24 button, its point up while the window
/// is open and down while it is folded.
/// </summary>
public sealed class CollapseButton : Border
{
    private readonly Polyline _chevron;
    private bool _collapsed;

    public CollapseButton()
    {
        Width = 28;
        Height = 24;
        CornerRadius = new CornerRadius(6);
        Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        VerticalAlignment = VerticalAlignment.Center;

        _chevron = new Polyline { StrokeThickness = 2, StrokeLineJoin = PenLineJoin.Round, StrokeStartLineCap = PenLineCap.Round, StrokeEndLineCap = PenLineCap.Round };
        Child = new Canvas { Children = { _chevron } };

        PointerEntered += (_, _) => Paint(hovered: true);
        PointerExited += (_, _) => Paint(hovered: false);
        Paint(hovered: false);
    }

    public bool Collapsed
    {
        get => _collapsed;
        set
        {
            _collapsed = value;
            ToolTipService.SetToolTip(this, value ? "Expand" : "Collapse");
            Paint(hovered: false);
        }
    }

    private void Paint(bool hovered)
    {
        const double half = 5;
        const double rise = 3;
        double point = _collapsed ? rise : -rise;
        var centre = new Point(14, 12);

        _chevron.Points = [new Point(centre.X - half, centre.Y - point), new Point(centre.X, centre.Y + point), new Point(centre.X + half, centre.Y - point)];
        _chevron.Stroke = Theme.Brush(hovered ? Theme.Fg : Theme.FgSoft);
        Background = Theme.Brush(hovered ? Color.FromArgb(20, 255, 255, 255) : Color.FromArgb(0, 0, 0, 0));
    }
}

/// <summary>
/// A grip on a window's edge that is dragged to resize it: eight high, with a small
/// muted pill in its middle that lights up in the accent, and grows, while it is pointed at or dragged.
/// </summary>
public sealed class ResizeGrip : Grid
{
    public const double Thickness = 8;

    private readonly Border _line;
    private readonly Border _pill;
    private readonly bool _lights;
    private readonly bool _sideways;
    private bool _hovered;
    private bool _dragging;
    private double _from;

    /// <param name="lights">Whether it lights up while pointed at or dragged; not for an edge the pointer crosses all the time.</param>
    /// <param name="sideways">On a left or right edge, dragged across rather than up and down.</param>
    public ResizeGrip(string tooltip, bool lights = true, bool sideways = false)
    {
        _lights = lights;
        _sideways = sideways;
        Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
        ToolTipService.SetToolTip(this, tooltip);

        if (sideways)
        {
            Width = Thickness;
            ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeWestEast);
            _line = new Border { Width = 2, HorizontalAlignment = HorizontalAlignment.Center };
            _pill = new Border { Width = 4, CornerRadius = new CornerRadius(2), HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
        }
        else
        {
            Height = Thickness;
            ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeNorthSouth);
            _line = new Border { Height = 2, VerticalAlignment = VerticalAlignment.Center };
            _pill = new Border { Height = 4, CornerRadius = new CornerRadius(2), HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
        }

        Children.Add(_line);
        Children.Add(_pill);

        PointerEntered += (_, _) => { _hovered = true; Paint(); };
        PointerExited += (_, _) => { _hovered = false; Paint(); };
        PointerPressed += OnPressed;
        PointerMoved += OnMoved;
        PointerReleased += OnReleased;
        PointerCaptureLost += (_, _) => End();
        Paint();
    }

    public event Action? Started;

    /// <summary>How far the pointer has moved down (or right, sideways) since the drag began; negative for up or left.</summary>
    public event Action<double>? Moved;

    public event Action? Ended;

    private void OnPressed(object sender, PointerRoutedEventArgs e)
    {
        _dragging = CapturePointer(e.Pointer);
        _from = Along(e.GetCurrentPoint(null).Position);
        Started?.Invoke();
        Paint();
        e.Handled = true;
    }

    private void OnMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_dragging)
        {
            Moved?.Invoke(Along(e.GetCurrentPoint(null).Position) - _from);
            e.Handled = true;
        }
    }

    private double Along(Windows.Foundation.Point at) => _sideways ? at.X : at.Y;

    private void OnReleased(object sender, PointerRoutedEventArgs e)
    {
        ReleasePointerCapture(e.Pointer);
        End();
        e.Handled = true;
    }

    private void End()
    {
        if (!_dragging)
        {
            return;
        }

        _dragging = false;
        Ended?.Invoke();
        Paint();
    }

    private void Paint()
    {
        bool lit = _lights && (_hovered || _dragging);
        _line.Background = lit ? Theme.Brush(Theme.WithAlpha(Theme.Accent, _dragging ? 0.9 : 0.6)) : null;
        if (_sideways)
        {
            _pill.Height = lit ? 40 : 30;
        }
        else
        {
            _pill.Width = lit ? 40 : 30;
        }

        _pill.Background = Theme.Brush(lit ? Theme.Accent : Theme.WithAlpha(Theme.FgMuted, 0.7));
    }
}
