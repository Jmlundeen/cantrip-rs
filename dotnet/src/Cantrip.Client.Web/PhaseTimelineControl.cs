using Cantrip.Protocol;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Shapes;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The turn as a bar of steps: one pill per step, the current one filled green on
/// this player's turn or amber on an opponent's, steps already gone faded. A step anyone gets priority in carries a dot
/// for each turn it is a stop on, and a click opens the choice of whether to stop there.
/// </summary>
public sealed class PhaseTimelineControl : Border
{
    private static readonly (TurnStepView Step, string Caption)[] Steps =
    [
        (TurnStepView.Untap, "Untap"),
        (TurnStepView.Upkeep, "Upkeep"),
        (TurnStepView.Draw, "Draw"),
        (TurnStepView.PrecombatMain, "Main 1"),
        (TurnStepView.BeginCombat, "Combat"),
        (TurnStepView.DeclareAttackers, "Attack"),
        (TurnStepView.DeclareBlockers, "Block"),
        (TurnStepView.CombatDamage, "Damage"),
        (TurnStepView.EndOfCombat, "End combat"),
        (TurnStepView.PostcombatMain, "Main 2"),
        (TurnStepView.End, "End"),
        (TurnStepView.Cleanup, "Cleanup"),
    ];

    private static readonly SolidColorBrush Clear = new(Color.FromArgb(0, 0, 0, 0));

    private readonly TurnStops _stops;
    private readonly Dictionary<TurnStepView, Border> _pills = [];
    private readonly Dictionary<TurnStepView, TextBlock> _labels = [];
    private readonly Dictionary<(TurnStepView Step, bool Mine), Ellipse> _marks = [];
    private readonly HashSet<TurnStepView> _hovered = [];
    private readonly Popup _popup = new() { IsLightDismissEnabled = true };

    private TurnStepView _current;
    private bool _myTurn;

    public PhaseTimelineControl(TurnStops stops)
    {
        _stops = stops;
        _stops.Changed += () =>
        {
            foreach ((TurnStepView step, _) in Steps)
            {
                Mark(step);
            }
        };

        Background = Theme.Brush(Theme.Bar);
        BorderBrush = Theme.Brush(Theme.Line);
        BorderThickness = new Thickness(0, 1, 0, 0);
        Padding = new Thickness(12, 4, 12, 4);

        var grid = new Grid();
        Child = grid;

        foreach ((TurnStepView step, string caption) in Steps)
        {
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            Border pill = Pill(step, caption);
            Grid.SetColumn(pill, grid.ColumnDefinitions.Count - 1);
            grid.Children.Add(pill);
        }

        // Day and night (CR 731.1) belongs to the game rather than to any player, so it is a chip at the end of the bar,
        // absent until the game has a designation at all, which most games never get (PhaseTimeline._designation).
        _designationLabel = new TextBlock { FontSize = 11, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Accent) };
        _designation = Theme.Pill(new Border
        {
            Padding = new Thickness(8, 2, 8, 2),
            VerticalAlignment = VerticalAlignment.Center,
            Background = Theme.Brush(Theme.WithAlpha(Theme.Accent, 0.15)),
            Child = _designationLabel,
            Visibility = Visibility.Collapsed,
        });

        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Grid.SetColumn(_designation, grid.ColumnDefinitions.Count - 1);
        grid.Children.Add(_designation);
    }

    private readonly Border _designation;
    private readonly TextBlock _designationLabel;

    /// <summary>Says whether it is day or night (CR 731.1); nothing while the game is neither (PhaseTimeline.Designate).</summary>
    public void Designate(DayNightView designation)
    {
        _designation.Visibility = designation == DayNightView.Neither ? Visibility.Collapsed : Visibility.Visible;
        if (designation == DayNightView.Neither)
        {
            return;
        }

        bool night = designation == DayNightView.Night;
        _designationLabel.Text = night ? "Night" : "Day";
        ToolTipService.SetToolTip(_designation, night
            ? "It is night. Permanents with daybound are turned over, and it becomes day if the active player casts two or more spells in a turn."
            : "It is day. Permanents with nightbound are turned over, and it becomes night if the active player casts no spells in a turn.");
    }

    /// <summary>Marks where the game currently is.</summary>
    public void Follow(TurnStepView current, bool myTurn)
    {
        _current = current;
        _myTurn = myTurn;

        foreach ((TurnStepView step, _) in Steps)
        {
            Paint(step);
        }
    }

    private Border Pill(TurnStepView step, string caption)
    {
        Border pill = Theme.Pill(new Border
        {
            Padding = new Thickness(9, 2, 9, 2),
            HorizontalAlignment = HorizontalAlignment.Center,
            Background = Clear,
        });

        var inner = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 3 };
        pill.Child = inner;

        var label = new TextBlock { Text = caption, FontSize = 11, VerticalAlignment = VerticalAlignment.Center };
        inner.Children.Add(label);

        _pills[step] = pill;
        _labels[step] = label;

        if (TurnStops.Stoppable(step))
        {
            ToolTipService.SetToolTip(pill, "Click to choose whether to stop here.");

            foreach (bool mine in (bool[])[true, false])
            {
                // StopDot at 8: a filled dot of radius 0.36 of its size, drawn only while the stop is on.
                var mark = new Ellipse
                {
                    Width = 5.76,
                    Height = 5.76,
                    Margin = new Thickness(1.12),
                    Fill = Theme.Brush(mine ? Theme.Mine : Theme.Theirs),
                    VerticalAlignment = VerticalAlignment.Center,
                };

                inner.Children.Add(mark);
                _marks[(step, mine)] = mark;
            }

            pill.Tapped += (_, _) => OpenStops(step, caption, pill);
            pill.PointerEntered += (_, _) =>
            {
                _hovered.Add(step);
                Paint(step);
            };

            pill.PointerExited += (_, _) =>
            {
                _hovered.Remove(step);
                Paint(step);
            };
        }
        else
        {
            ToolTipService.SetToolTip(pill, "Nobody receives priority in this step, so there is nothing to stop for.");
        }

        Mark(step);
        Paint(step);
        return pill;
    }

    private void Mark(TurnStepView step)
    {
        foreach (bool mine in (bool[])[true, false])
        {
            if (_marks.TryGetValue((step, mine), out Ellipse? mark))
            {
                mark.Visibility = _stops.StopsAt(step, mine) ? Visibility.Visible : Visibility.Collapsed;
            }
        }
    }

    private void Paint(TurnStepView step)
    {
        bool here = step == _current;
        bool passed = !here && IndexOf(step) < IndexOf(_current);

        _pills[step].Background = here
            ? Theme.Brush(_myTurn ? Theme.Mine : Theme.Theirs)
            : _hovered.Contains(step) ? Theme.Brush(Color.FromArgb(18, 255, 255, 255)) : Clear;

        // Steps already gone this turn fade, so the bar reads left to right as time does.
        _labels[step].Foreground = Theme.Brush(here ? Theme.FgOnAccent : passed ? Theme.FgMuted : Theme.FgSoft);
        _labels[step].FontWeight = here ? FontWeights.SemiBold : FontWeights.Normal;
    }

    /// <summary>The StopPopup: the step's name, and a row for each turn to stop on, opened just above the pill.</summary>
    private void OpenStops(TurnStepView step, string caption, Border pill)
    {
        var column = new StackPanel { Orientation = Orientation.Vertical, Spacing = 2 };
        column.Children.Add(new TextBlock
        {
            Text = caption.ToUpperInvariant(),
            FontSize = 11,
            FontWeight = FontWeights.SemiBold,
            Foreground = Theme.Brush(Theme.FgMuted),
            Margin = new Thickness(8, 4, 8, 4),
        });

        column.Children.Add(StopRow(step, "Stop on my turn", mine: true));
        column.Children.Add(StopRow(step, "Stop on their turn", mine: false));

        var panel = new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.ControlBorder),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(8),
            Padding = new Thickness(6),
            Child = column,
        };

        _popup.XamlRoot = XamlRoot;
        _popup.Child = panel;
        panel.Measure(new Windows.Foundation.Size(double.PositiveInfinity, double.PositiveInfinity));

        Windows.Foundation.Point at = pill.TransformToVisual(null).TransformPoint(new Windows.Foundation.Point(0, 0));
        _popup.HorizontalOffset = at.X + (pill.ActualWidth / 2) - (panel.DesiredSize.Width / 2);
        _popup.VerticalOffset = at.Y - panel.DesiredSize.Height - 8;
        _popup.IsOpen = true;
    }

    private Border StopRow(TurnStepView step, string text, bool mine)
    {
        bool on = _stops.StopsAt(step, mine);
        Color tint = mine ? Theme.Mine : Theme.Theirs;

        // StopDot at 12: filled while on, a ring while off.
        var dot = new Ellipse
        {
            Width = 8.64,
            Height = 8.64,
            Margin = new Thickness(1.68),
            Fill = on ? Theme.Brush(tint) : Clear,
            Stroke = on ? null : Theme.Brush(Theme.WithAlpha(tint, 0.35)),
            StrokeThickness = 1.5,
            VerticalAlignment = VerticalAlignment.Center,
        };

        var content = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(8, 0, 8, 0) };
        content.Children.Add(dot);
        content.Children.Add(new TextBlock { Text = text, FontSize = 12, Foreground = Theme.Brush(Theme.Fg), VerticalAlignment = VerticalAlignment.Center });

        var row = new Border { MinWidth = 180, Height = 28, CornerRadius = new CornerRadius(6), Background = Clear, Child = content };
        row.PointerEntered += (_, _) => row.Background = Theme.Brush(Color.FromArgb(20, 255, 255, 255));
        row.PointerExited += (_, _) => row.Background = Clear;
        row.Tapped += (_, _) =>
        {
            _stops.Toggle(step, mine);
            _popup.IsOpen = false;
        };

        return row;
    }

    private static int IndexOf(TurnStepView step) => Array.FindIndex(Steps, s => s.Step == step);
}
