using Cantrip.Protocol;
using Microsoft.UI.Text;
using Windows.Foundation;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// Text with the mana symbols in it drawn as pips: "Still owed: {G}" reads with a green pip where the braces were. Each word is its own text run and each symbol a pip a little larger than the text,
/// centred on its line; with wrapping on, the words flow onto as many lines as the width takes.
/// <para>
/// A line that names cards - the game log's, a draft's picks - writes each as its name in square brackets
/// (<see cref="LogEntryView.Text"/>); shown with the cards it names (<see cref="Show"/>), each name is drawn bold and in
/// <see cref="Theme.CardName"/>, without its brackets, as one word that carries its card (<see cref="CardMention"/>) for
/// the page to show when the name is pointed at.
/// </para>
/// </summary>
public sealed class SymbolText : Panel
{
    private string _text = string.Empty;
    private IReadOnlyList<LogReferenceView> _names = [];

    /// <summary>Whether the text names cards in square brackets (<see cref="Show"/>); elsewhere a bracket is only a bracket, as in a loyalty cost.</summary>
    private bool _naming;
    private double _fontSize;
    private Color _color;
    private bool _semibold;

    public SymbolText(double fontSize, Color color, bool semibold = false, bool wrap = false)
    {
        _fontSize = fontSize;
        _color = color;
        _semibold = semibold;
        Wrap = wrap;
    }

    public bool Wrap { get; }

    public string Text
    {
        get => _text;
        set
        {
            value ??= string.Empty;
            if (_text != value || _naming)
            {
                _text = value;
                _names = [];
                _naming = false;
                Rebuild();
            }
        }
    }

    public Color Color
    {
        get => _color;
        set
        {
            _color = value;
            IEnumerable<UIElement> words = Children.SelectMany<UIElement, UIElement>(child => child is StackPanel name ? name.Children : [child]);
            foreach (TextBlock word in words.OfType<TextBlock>().Where(word => word.Tag is not CardMention))
            {
                word.Foreground = Theme.Brush(value);
            }
        }
    }

    private double Diameter => Math.Round(_fontSize * 1.1);

    /// <summary>
    /// How wide a space is: Open Sans's own advance, 532 of its 2048 units to the em. Measuring "x x" against "xx" at
    /// run time came out far too wide in Uno, so the face's figure is used as it stands.
    /// </summary>
    private double SpaceWidth() => _fontSize * 532 / 2048;

    /// <summary>A line naming cards, each in square brackets, and the cards it names in the order it names them.</summary>
    public void Show(string text, IReadOnlyList<LogReferenceView> names)
    {
        text ??= string.Empty;
        if (_naming && _text == text && ReferenceEquals(_names, names))
        {
            return;
        }

        _text = text;
        _names = names;
        _naming = true;
        Rebuild();
    }

    private void Rebuild()
    {
        Children.Clear();
        if (!_naming)
        {
            AddWords(_text);
            InvalidateMeasure();
            return;
        }

        int named = 0;
        int at = 0;
        while (at < _text.Length)
        {
            int open = _text.IndexOf('[', at);
            int close = open < 0 ? -1 : _text.IndexOf(']', open + 1);
            if (close < 0)
            {
                AddWords(_text[at..]);
                break;
            }

            AddWords(_text[at..open]);
            string name = _text[(open + 1)..close];
            LogReferenceView? card = named < _names.Count && _names[named].Name == name ? _names[named] : null;
            if (card is not null)
            {
                named++;
            }

            // What follows a name up to the next space - its full stop, its "'s" - goes with it, never onto the next line alone.
            int after = close + 1;
            while (after < _text.Length && _text[after] is not (' ' or '[' or '{'))
            {
                after++;
            }

            AddName(name, card, _text[(close + 1)..after]);
            at = after;
        }

        InvalidateMeasure();
    }

    private void AddName(string name, LogReferenceView? card, string tail)
    {
        var block = new TextBlock
        {
            Text = name,
            FontSize = _fontSize,
            FontWeight = FontWeights.Bold,
            Foreground = Theme.Brush(Theme.CardName),
            Tag = new CardMention(card),
        };

        if (tail.Length == 0)
        {
            Children.Add(block);
            return;
        }

        Children.Add(new StackPanel
        {
            Orientation = Orientation.Horizontal,
            Children = { block, Word(tail, 0) },
        });
    }

    private TextBlock Word(string word, double space) => new()
    {
        Text = word,
        FontSize = _fontSize,
        FontWeight = _semibold ? FontWeights.SemiBold : FontWeights.Normal,
        Foreground = Theme.Brush(_color),
        Margin = new Thickness(0, 0, space, 0),
    };

    private void AddWords(string text)
    {
        foreach ((string part, bool isSymbol) in ManaSymbols.Parse(text))
        {
            if (isSymbol)
            {
                // A slot a little wider than the pip.
                Children.Add(new ManaPipElement(part, Diameter) { Margin = new Thickness(1, 0, 1, 0) });
                continue;
            }

            int start = 0;
            for (int i = 0; i <= part.Length; i++)
            {
                if (i == part.Length || part[i] == ' ')
                {
                    int end = Math.Min(i + 1, part.Length);
                    if (end > start)
                    {
                        // The space after a word as room beside it rather than as text: a text block trims whitespace
                        // at its end, a no-break space included, and the words would run together.
                        string run = part[start..end];
                        string word = run.TrimEnd(' ');
                        double space = (run.Length - word.Length) * SpaceWidth();

                        Children.Add(word.Length == 0 ? new Border { Width = space, Height = 1 } : Word(word, space));
                    }

                    start = end;
                }
            }
        }
    }

    protected override Size MeasureOverride(Size availableSize)
    {
        double limit = Wrap && !double.IsInfinity(availableSize.Width) ? availableSize.Width : double.PositiveInfinity;
        double x = 0;
        double lineHeight = 0;
        double width = 0;
        double height = 0;

        foreach (UIElement child in Children)
        {
            child.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
            Size size = child.DesiredSize;

            if (x > 0 && x + size.Width > limit)
            {
                height += lineHeight;
                width = Math.Max(width, x);
                x = 0;
                lineHeight = 0;
            }

            x += size.Width;
            lineHeight = Math.Max(lineHeight, size.Height);
        }

        width = Math.Max(width, x);
        height += lineHeight;
        return new Size(Wrap && !double.IsInfinity(availableSize.Width) ? Math.Min(width, availableSize.Width) : width, height);
    }

    protected override Size ArrangeOverride(Size finalSize)
    {
        double limit = Wrap ? finalSize.Width : double.PositiveInfinity;
        List<List<UIElement>> lines = [[]];
        double x = 0;

        foreach (UIElement child in Children)
        {
            if (x > 0 && x + child.DesiredSize.Width > limit)
            {
                lines.Add([]);
                x = 0;
            }

            lines[^1].Add(child);
            x += child.DesiredSize.Width;
        }

        double y = 0;
        foreach (List<UIElement> line in lines)
        {
            double lineHeight = line.Count == 0 ? 0 : line.Max(child => child.DesiredSize.Height);
            x = 0;

            foreach (UIElement child in line)
            {
                Size size = child.DesiredSize;
                child.Arrange(new Rect(x, y + ((lineHeight - size.Height) / 2), size.Width, size.Height));
                x += size.Width;
            }

            y += lineHeight;
        }

        return finalSize;
    }
}

/// <summary>
/// What a card's name in a line of text is: the card it names, as the line was given it, or null for a name the line
/// had nothing more to give for.
/// </summary>
public sealed record CardMention(LogReferenceView? Card);
