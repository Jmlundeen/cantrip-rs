using Cantrip.Core;
using Cantrip.Protocol;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// The full face as text, shown when the pointer rests on a card, so nothing drawn in an abbreviated form is left
/// unreadable. One line per fact, the first - name and cost - a size up, and mana symbols drawn as pips. The panel is
/// near black, edged, and rounded.
/// </summary>
public sealed class CardTooltip : Canvas
{
    /// <summary>How long the pointer rests on a card before its tooltip shows.</summary>
    private static readonly TimeSpan Delay = TimeSpan.FromSeconds(0.5);

    /// <summary>How wide a tooltip's lines wrap at.</summary>
    private const double WrapWidth = 300;

    /// <summary>Leads a line whose ability has done all it may this turn, so it is greyed.</summary>
    private const char SpentMark = '\u0001';

    /// <summary>Leads a planeswalker's loyalty ability, drawn with its cost on a shield beside its text.</summary>
    private const char LoyaltyMark = '\u0002';

    /// <summary>Leads the loyalty a planeswalker has, drawn on the shield its loyalty starts on.</summary>
    private const char LoyaltyNowMark = '\u0003';

    /// <summary>How wide a loyalty shield is in a tooltip, and the room between it and the text.</summary>
    private const double ShieldWidth = 34;
    private const double ShieldGap = 6;

    private readonly Border _panel;
    private readonly StackPanel _lines = new() { Orientation = Orientation.Vertical, Spacing = 3 };
    private readonly DispatcherTimer _timer = new() { Interval = Delay };
    private string _pending = string.Empty;
    private Point _at;

    public CardTooltip()
    {
        IsHitTestVisible = false;

        _panel = new Border
        {
            Background = Theme.Brush(Theme.WithAlpha(Theme.FromHex("#0d0f12"), 0.97)),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(10, 7, 10, 7),
            MaxWidth = WrapWidth + 22,
            Child = _lines,
            Visibility = Visibility.Collapsed,
        };

        Children.Add(_panel);
        _timer.Tick += (_, _) =>
        {
            _timer.Stop();
            Show();
        };
    }

    /// <summary>The pointer came to rest on a card, at a point on the page: its tooltip shows after the delay.</summary>
    public void Rest(string text, Point at)
    {
        _at = at;
        if (text == _pending && (_panel.Visibility == Visibility.Visible || _timer.IsEnabled))
        {
            Place();
            return;
        }

        Hide();
        _pending = text;
        if (text.Length > 0)
        {
            _timer.Start();
        }
    }

    /// <summary>The first line of the tooltip showing - a card's name and cost - or empty when none is.</summary>
    public string Showing => _panel.Visibility == Visibility.Visible ? _pending.Split('\n')[0] : string.Empty;

    public void Hide()
    {
        _timer.Stop();
        _pending = string.Empty;
        _panel.Visibility = Visibility.Collapsed;
    }

    private void Show()
    {
        _lines.Children.Clear();
        string[] parts = _pending.Split('\n');

        for (int i = 0; i < parts.Length; i++)
        {
            bool spent = parts[i].StartsWith(SpentMark);
            string part = spent ? parts[i][1..] : parts[i];
            var color = i == 0 ? Theme.Fg : spent ? Theme.FgMuted : Theme.FgSoft;

            if (part.StartsWith(LoyaltyMark) && CardFace.TryLoyaltyAbility(part[1..], out string cost, out string ability))
            {
                // The cost on its shield, centred against the ability's lines, as the face sets it.
                var row = new Grid { ColumnSpacing = ShieldGap, MaxWidth = WrapWidth };
                row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
                row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
                var shield = new LoyaltyShieldElement(cost, ShieldWidth);
                var text = new SymbolText(12, color, wrap: true) { Text = ability, MaxWidth = WrapWidth - ShieldWidth - ShieldGap };
                Grid.SetColumn(text, 1);
                row.Children.Add(shield);
                row.Children.Add(text);
                _lines.Children.Add(row);
                continue;
            }

            if (part.StartsWith(LoyaltyNowMark))
            {
                var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = ShieldGap };
                row.Children.Add(new SymbolText(12, color) { Text = "Loyalty", VerticalAlignment = VerticalAlignment.Center });
                row.Children.Add(new LoyaltyShieldElement(part[1..], ShieldWidth, start: true));
                _lines.Children.Add(row);
                continue;
            }

            _lines.Children.Add(new SymbolText(i == 0 ? 13 : 12, color, semibold: i == 0, wrap: true)
            {
                Text = part,
                MaxWidth = WrapWidth,
            });
        }

        _panel.Visibility = Visibility.Visible;
        Place();
    }

    /// <summary>Below and to the right of the pointer, and turned back inside the page at its edges.</summary>
    private void Place()
    {
        _panel.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
        Size size = _panel.DesiredSize;
        double x = _at.X + 12;
        double y = _at.Y + 16;

        if (x + size.Width > ActualWidth)
        {
            x = Math.Max(0, _at.X - size.Width - 4);
        }

        if (y + size.Height > ActualHeight)
        {
            y = Math.Max(0, _at.Y - size.Height - 4);
        }

        SetLeft(_panel, x);
        SetTop(_panel, y);
    }

    /// <summary>
    /// A card's tooltip text: name and cost, type line, rules, then what is true of it now, and a note the board added,
    /// such as what the spell being paid for is waiting on.
    /// </summary>
    public static string TextFor(ObjectView card, string? note)
    {
        if (card.FaceDown)
        {
            // A face-down permanent is a 2/2 with no text (CR 708.2a); a card exiled face down has no characteristics
            // at all (CR 406.3a).
            bool permanent = card.Zone == ZoneKindView.Battlefield;
            List<string> said =
            [
                permanent
                    ? $"A face-down {card.Power}/{card.Toughness} creature with no name, text or cost."
                    : "A card exiled face down. It has no name, text, types or cost.",
            ];

            if (card.FaceDownName.Length > 0)
            {
                said.Add(permanent
                    ? $"It is {card.FaceDownName}. Only you can see that."
                    : $"You are allowed to look at it: it is {card.FaceDownName}.");
            }

            if (card.DamageMarked > 0)
            {
                said.Add($"{card.DamageMarked} damage marked.");
            }

            return string.Join('\n', said);
        }

        if (!card.Known)
        {
            return note ?? string.Empty;
        }

        List<string> lines = card.ManaCost.Length > 0
            ? [$"{card.Name}   {card.ManaCost}", CardFace.TypeLine(card)]
            : [card.Name, CardFace.TypeLine(card)];

        bool walker = card.CardTypes.Contains(CardTypeView.Planeswalker);
        foreach ((string rule, bool spent) in CardFace.RulesLines(card))
        {
            string line = walker && CardFace.TryLoyaltyAbility(rule, out _, out _) ? LoyaltyMark + rule : rule;
            lines.Add(spent ? SpentMark + line : line);
        }

        if (card.Power is not null && card.Toughness is not null)
        {
            lines.Add($"{card.Power}/{card.Toughness}" + (card.DamageMarked > 0 ? $", {card.DamageMarked} damage marked" : string.Empty));
        }

        foreach (CounterView counter in card.Counters)
        {
            lines.Add(counter.Name switch
            {
                CounterNames.Loyalty => $"{LoyaltyNowMark}{counter.Count}",
                CounterNames.Defense => $"Defense {counter.Count}",
                _ => $"{counter.Count} {counter.Name} counter(s)",
            });
        }

        foreach (ChoiceView choice in card.Choices)
        {
            lines.Add($"{ObjectChoice.Label(choice.Key)}: {choice.Value}.");
        }

        if (card.Tapped)
        {
            lines.Add("Tapped.");
        }

        if (card.Monstrous)
        {
            lines.Add("Monstrous (CR 701.37b).");
        }

        if (card.ClassLevel > 0)
        {
            lines.Add($"Level {card.ClassLevel} (CR 716).");
        }

        if (card.Harnessed)
        {
            lines.Add("Harnessed (CR 701.64).");
        }

        if (card.Saddled)
        {
            lines.Add("Saddled until end of turn (CR 702.171b).");
        }

        if (card.Exerted)
        {
            lines.Add("Exerted: it won't untap during the next untap step (CR 701.43a).");
        }

        if (card.Goaded)
        {
            lines.Add("Goaded: it attacks each combat if able, and a player other than its goader if able (CR 701.15b).");
        }

        if (card.SummoningSick)
        {
            lines.Add("Entered this turn.");
        }

        if (card.KnownBy.Count > 0)
        {
            lines.Add(card.KnownBy.Count == 1 ? "Your opponent knows this card." : $"{card.KnownBy.Count} opponents know this card.");
        }

        if (note is { Length: > 0 })
        {
            lines.Add(note);
        }

        return string.Join('\n', lines);
    }
}

/// <summary>
/// A loyalty cost on its shield, or with <c>start</c> the loyalty a planeswalker has, for a tooltip: light on its dark
/// panel, where the face's black shield would not show, with the number in the panel's own dark.
/// </summary>
public sealed class LoyaltyShieldElement : Uno.WinUI.Graphics2DSK.SKCanvasElement
{
    private static readonly SkiaSharp.SKColor Shield = SkiaSharp.SKColor.Parse("d9dce2");
    private static readonly SkiaSharp.SKColor Figure = SkiaSharp.SKColor.Parse("0d0f12");

    private readonly string _value;
    private readonly bool _start;

    public LoyaltyShieldElement(string value, double width, bool start = false)
    {
        _value = value;
        _start = start;
        Width = width;
        Height = Math.Round(width * 0.72);
        VerticalAlignment = VerticalAlignment.Center;
        IsHitTestVisible = false;
    }

    protected override void RenderOverride(SkiaSharp.SKCanvas canvas, Size area)
    {
        var centre = new SkiaSharp.SKPoint((float)area.Width / 2f, (float)area.Height / 2f);
        if (_start)
        {
            CardFace.DrawLoyaltyStart(canvas, centre, (float)area.Width, _value, Shield, Figure);
        }
        else
        {
            CardFace.DrawLoyalty(canvas, centre, (float)area.Width, _value, Shield, Figure);
        }
    }
}
