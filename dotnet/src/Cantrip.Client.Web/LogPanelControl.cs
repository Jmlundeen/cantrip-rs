using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Microsoft.UI.Text;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// The game log, the side panel's first page: the journal's lines - each turn as an
/// accent heading set into a hairline rule, each step as a small muted caps line, and entries under them. Appended
/// rather than rebuilt, and only follows the newest line when the reader was already at the bottom, so scrolling back
/// to read an earlier turn is not undone by the next event.
/// <para>
/// Only the lines in view, and a quarter of a screen either side, are made as controls (<see cref="LogRows"/>); the rest are only
/// their heights. A long game's log kept whole cost every redraw of the page, whatever the redraw was for: a combo's
/// few hundred lines made tapping a land take a noticeable time, and more the longer the game went on.
/// </para>
/// </summary>
public sealed class LogPanelControl : Grid
{
    private readonly LogRows _rows;
    private readonly ScrollViewer _scroll;
    private int _revision;

    /// <summary>Whether the reader is at the newest line, and so is kept there as lines come in.</summary>
    private bool _following = true;

    public LogPanelControl()
    {
        _scroll = new ScrollViewer
        {
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            HorizontalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        };

        _rows = new LogRows(_scroll);
        _scroll.Content = _rows;

        // Scrolling brings other lines into view, so which lines are made is worked out again. Where a scroll comes to
        // rest says whether the reader is following the newest line; while they are still scrolling, the lines are made
        // where they are looking, not at the end.
        _scroll.ViewChanged += (_, e) =>
        {
            if (!e.IsIntermediate)
            {
                _following = _scroll.VerticalOffset >= _scroll.ScrollableHeight - 8;
            }

            _rows.FromBottom = _following && !e.IsIntermediate;
            _rows.InvalidateMeasure();
        };

        // The lines grew, by new ones or by ones measured taller than guessed: a reader following the newest line is taken
        // to the new end.
        _rows.SizeChanged += (_, _) => KeepToTheEnd();

        Children.Add(_scroll);

        _fontWatch.Tick += (_, _) => WatchTheFont();
        _fontWatch.Start();
    }

    /// <summary>
    /// How often, and for how long, a line of text is measured to see whether the text font has come in. A log sent with
    /// the page's first view is made before the font has loaded, and a line made then keeps the stand-in font it was made
    /// in; the old log, made in one slow pass, only ever missed this by being slow.
    /// </summary>
    private readonly DispatcherTimer _fontWatch = new() { Interval = TimeSpan.FromMilliseconds(250) };
    private const int FontLooks = 120;
    private (double Regular, double Semibold, double Bold)? _fontSeen;
    private int _fontLooked;

    /// <summary>Makes the lines again when the same words, in each weight the log uses, measure differently than before.</summary>
    private void WatchTheFont()
    {
        (double Regular, double Semibold, double Bold) now = (Measured(FontWeights.Normal), Measured(FontWeights.SemiBold), Measured(FontWeights.Bold));
        if (_fontSeen is { } seen && seen != now)
        {
            _rows.Remake();
        }

        _fontSeen = now;
        if (++_fontLooked >= FontLooks)
        {
            _fontWatch.Stop();
        }
    }

    private static double Measured(Windows.UI.Text.FontWeight weight)
    {
        var probe = new TextBlock { Text = "Hamburgefonstiv 0123", FontSize = 12, FontWeight = weight };
        probe.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
        return probe.DesiredSize.Width;
    }

    /// <summary>How many lines the log has, and how many of them are made as controls, for the page's test state.</summary>
    public (int Lines, int Shown) Extent => (_rows.Count, _rows.Shown);

    /// <summary>Starts the log over, for a new game or a new fixture.</summary>
    public void Clear()
    {
        _rows.Reset();
        _revision = 0;
        _following = true;
        _rows.FromBottom = true;
    }

    public void Show(IReadOnlyList<LogLine> lines, int revision)
    {
        if (revision != _revision)
        {
            _revision = revision;
            _rows.Reset();
        }

        _rows.Follow(lines);
    }

    private void KeepToTheEnd()
    {
        if (_following && _scroll.VerticalOffset < _scroll.ScrollableHeight - 1)
        {
            _scroll.ChangeView(null, _scroll.ScrollableHeight, null, disableAnimation: true);
        }
    }
}

/// <summary>
/// The log's lines, made only where they can be seen: a panel as tall as every line would stand, holding controls for
/// the lines in and around the view and reusing them as it scrolls. Each line's height is guessed until it has been shown
/// and measured (<see cref="RowExtents"/>); a line that comes into view above the reader measured other than its guess
/// moves the view by the difference, so what they were reading stays where it was.
/// </summary>
internal sealed class LogRows : Panel
{
    private const double Gap = 2;
    private static readonly Thickness Pad = new(14, 12, 14, 16);

    private readonly ScrollViewer _scroll;
    private readonly RowExtents _extents = new(Gap);
    private readonly Dictionary<int, Row> _shown = [];
    private readonly Dictionary<LogLineKind, Stack<Row>> _spare = [];
    private readonly Dictionary<LogLineKind, double> _guess = new()
    {
        [LogLineKind.Turn] = 28,
        [LogLineKind.Phase] = 21,
        [LogLineKind.Entry] = 17,
    };

    private IReadOnlyList<LogLine> _lines = [];
    private LogLine? _last;
    private double _inner = -1;

    public LogRows(ScrollViewer scroll)
    {
        _scroll = scroll;
    }

    public int Count => _extents.Count;

    public int Shown => _shown.Count;

    /// <summary>Whether the lines at the end are the ones made, for a log that is following its newest line.</summary>
    public bool FromBottom { get; set; } = true;

    /// <summary>
    /// Takes in the journal's lines: those added since, and the last one again if it was said once more and counted
    /// (<see cref="LogLine.Count"/>), which the journal does by replacing it. Returns whether anything changed.
    /// </summary>
    public bool Follow(IReadOnlyList<LogLine> lines)
    {
        bool changed = false;
        int last = _extents.Count - 1;
        if (last >= 0 && last < lines.Count && !ReferenceEquals(lines[last], _last))
        {
            changed = true;
            if (_shown.TryGetValue(last, out Row? row))
            {
                row.Fill(lines[last], last == 0);
                row.Element.Measure(new Size(_inner, double.PositiveInfinity));
                _extents.Measure(last, row.Element.DesiredSize.Height);
            }
        }

        _lines = lines;
        for (int i = _extents.Count; i < lines.Count; i++)
        {
            _extents.Add(_guess[lines[i].Kind]);
            changed = true;
        }

        _last = lines.Count > 0 ? lines[^1] : null;
        if (changed)
        {
            InvalidateMeasure();
        }

        return changed;
    }

    /// <summary>Lets go of every line's control, so the lines in view are made afresh.</summary>
    public void Remake()
    {
        _shown.Clear();
        _spare.Clear();
        Children.Clear();
        _extents.Remeasure();
        InvalidateMeasure();
    }

    public void Reset()
    {
        foreach (int index in _shown.Keys.ToList())
        {
            Put(index);
        }

        _extents.Clear();
        _lines = [];
        _last = null;
        InvalidateMeasure();
    }

    protected override Size MeasureOverride(Size availableSize)
    {
        double width = double.IsInfinity(availableSize.Width) ? Math.Max(0, _scroll.ViewportWidth) : availableSize.Width;
        double inner = Math.Max(0, width - Pad.Left - Pad.Right);
        if (inner != _inner)
        {
            // Lines that wrapped may not at another width, and ones that did not may.
            _inner = inner;
            _extents.Remeasure();
        }

        double viewport = _scroll.ViewportHeight > 0 ? _scroll.ViewportHeight : 800;
        double moved = 0;

        // Measuring a line can move the ones under it into or out of the window, so the window is looked at again until
        // nothing it covers is left to measure.
        for (int pass = 0; pass < 4; pass++)
        {
            double top = FromBottom ? _extents.Total - viewport : _scroll.VerticalOffset - Pad.Top;
            (int first, int last) = _extents.Within(top - (viewport / 4), top + (1.25 * viewport));

            foreach (int index in _shown.Keys.Where(index => index < first || index > last).ToList())
            {
                Put(index);
            }

            bool changed = false;
            for (int index = first; index <= last; index++)
            {
                Row row = Take(index);
                if (_extents.IsMeasured(index))
                {
                    continue;
                }

                row.Element.Measure(new Size(inner, double.PositiveInfinity));
                double change = _extents.Measure(index, row.Element.DesiredSize.Height);
                if (change != 0)
                {
                    changed = true;

                    // A line wholly above the view that turned out taller or shorter pushes what is being read.
                    if (!FromBottom && _extents.Top(index) + _extents.Height(index) - change <= top)
                    {
                        moved += change;
                    }
                }
            }

            if (!changed)
            {
                break;
            }
        }

        if (moved != 0)
        {
            double offset = _scroll.VerticalOffset + moved;
            DispatcherQueue.TryEnqueue(() => _scroll.ChangeView(null, offset, null, disableAnimation: true));
        }

        return new Size(width, Pad.Top + _extents.Total + Pad.Bottom);
    }

    protected override Size ArrangeOverride(Size finalSize)
    {
        foreach ((int index, Row row) in _shown)
        {
            row.Element.Arrange(new Rect(Pad.Left, Pad.Top + _extents.Top(index), _inner, _extents.Height(index)));
        }

        return finalSize;
    }

    /// <summary>The control showing a line, made or reused.</summary>
    private Row Take(int index)
    {
        if (_shown.TryGetValue(index, out Row? shown))
        {
            return shown;
        }

        LogLine line = _lines[index];
        Row row = _spare.TryGetValue(line.Kind, out Stack<Row>? spare) && spare.Count > 0 ? spare.Pop() : Make(line.Kind);
        row.Fill(line, index == 0);
        row.Element.Visibility = Visibility.Visible;
        _shown[index] = row;

        // A reused control keeps the height it had for its last line, so the line is measured whatever it was taken as.
        if (_extents.IsMeasured(index))
        {
            row.Element.Measure(new Size(_inner, double.PositiveInfinity));
            _extents.Measure(index, row.Element.DesiredSize.Height);
        }

        return row;
    }

    /// <summary>Puts a line's control aside for another line.</summary>
    private void Put(int index)
    {
        Row row = _shown[index];
        _shown.Remove(index);
        row.Element.Visibility = Visibility.Collapsed;
        if (!_spare.TryGetValue(row.Kind, out Stack<Row>? spare))
        {
            _spare[row.Kind] = spare = new Stack<Row>();
        }

        spare.Push(row);
    }

    private Row Make(LogLineKind kind)
    {
        Row row = kind switch
        {
            LogLineKind.Turn => TurnHeader(),
            LogLineKind.Phase => PhaseHeader(),
            _ => Entry(),
        };

        Children.Add(row.Element);
        return row;
    }

    /// <summary>A turn as a rule with the turn's name set into it, so turns are told apart while scrolling.</summary>
    private static Row TurnHeader()
    {
        var row = new Grid { ColumnSpacing = 8 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

        var text = new TextBlock { FontSize = 12, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Accent) };
        row.Children.Add(text);

        var rule = new Border { Height = 1, Background = Theme.Brush(Theme.Line), VerticalAlignment = VerticalAlignment.Center };
        Grid.SetColumn(rule, 1);
        row.Children.Add(rule);

        return new Row(LogLineKind.Turn, row, (line, first) =>
        {
            text.Text = line.Text;
            row.Margin = new Thickness(0, first ? 0 : 10, 0, 0);
        });
    }

    private static Row PhaseHeader()
    {
        var text = new TextBlock
        {
            FontSize = 10,
            FontWeight = FontWeights.SemiBold,
            Foreground = Theme.Brush(Theme.FgMuted),
            TextWrapping = TextWrapping.Wrap,
            Margin = new Thickness(0, 6, 0, 1),
        };

        return new Row(LogLineKind.Phase, text, (line, _) => text.Text = line.Text.TrimEnd(':').ToUpperInvariant());
    }

    /// <summary>
    /// An entry, which says how many times it was said when that is more than once. A line naming a card or a cost is
    /// drawn a word to a control (<see cref="SymbolText"/>): each card by its name, bold and coloured, to be pointed at and
    /// read, and the symbols as pips. Any other line is one block of text, which is a good deal quicker to make and lay out.
    /// </summary>
    private static Row Entry()
    {
        var plain = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.FgSoft), TextWrapping = TextWrapping.Wrap };
        var symbols = new SymbolText(12, Theme.FgSoft, wrap: true);
        var row = new Grid { Children = { plain, symbols } };

        return new Row(LogLineKind.Entry, row, (line, _) =>
        {
            string text = line.Count > 1 ? $"{line.Text} ×{line.Count}" : line.Text;
            IReadOnlyList<LogReferenceView> names = line.Entry?.References ?? [];
            bool words = names.Count > 0 || text.Contains('{', StringComparison.Ordinal);
            plain.Visibility = words ? Visibility.Collapsed : Visibility.Visible;
            symbols.Visibility = words ? Visibility.Visible : Visibility.Collapsed;
            plain.Text = words ? string.Empty : text;
            symbols.Show(words ? text : string.Empty, names);
        });
    }

    private sealed record Row(LogLineKind Kind, FrameworkElement Element, Action<LogLine, bool> Fill);
}
