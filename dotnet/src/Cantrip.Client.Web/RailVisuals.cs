using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>A pile drawn as a card back, or as an empty outlined slot when there is nothing in it.</summary>
public sealed class PileElement : SKCanvasElement
{
    private static readonly SKColor SlotBorder = SKColor.Parse("2c313a");

    private bool _filled;

    public bool Filled
    {
        get => _filled;
        set
        {
            if (_filled != value)
            {
                _filled = value;
                Invalidate();
            }
        }
    }

    private bool _lit;

    /// <summary>Outlined in the ready colour, as an actionable card is: there is something in the pile to do or to look at.</summary>
    public bool Lit
    {
        get => _lit;
        set
        {
            if (_lit != value)
            {
                _lit = value;
                Invalidate();
            }
        }
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        var face = SKRect.Create(1, 1, (float)area.Width - 2, (float)area.Height - 4);

        if (_filled)
        {
            CardFace.DrawBack(canvas, face, face.Width);
            if (_lit)
            {
                using var ready = new SKPaint { Color = SKColor.Parse("6fcf8f"), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 2 };
                canvas.DrawRoundRect(SKRect.Inflate(face, -1, -1), 5, 5, ready);
            }

            return;
        }

        using var slot = new SKPaint { Color = SlotBorder, IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 1 };
        canvas.DrawRoundRect(face, 5, 5, slot);
    }
}

/// <summary>
/// An opponent's hand as the backs of its cards, fanned about a point below them: a count is all anyone else is
/// entitled to, and a fan says it at a glance. Sized to the width it is drawn at: a full hand spans it, and a smaller
/// hand is the same fan with fewer cards, so the cards stay one size.
/// </summary>
public sealed class HandFanElement : SKCanvasElement
{
    // The proportions of the fan at its standard size; the whole fan is scaled from these to the width it is given.
    private const float StandardCardWidth = 30f;
    private const float StandardCardHeight = 42f;
    private const float StandardReach = 130f;
    private const float MaxSpread = MathF.PI * 0.26f;
    private const float StepAngle = MathF.PI * 0.045f;
    private const float FanMargin = 4f;

    /// <summary>How tall a fan that spans a width is, so the rail can make room for it.</summary>
    public static float HeightFor(float width) => MathF.Ceiling((float)Extent(MaxSpread, ScaleFor(width)).Height);

    /// <summary>How wide one card of a fan spanning a width is, for cards flying into or out of it.</summary>
    public static float CardWidthFor(float width) => MathF.Round(StandardCardWidth * ScaleFor(width));

    /// <summary>The scale at which a full hand's fan is a given width: its standard size scaled, plus a fixed margin.</summary>
    private static float ScaleFor(float width) => (width - FanMargin) / ((float)Extent(MaxSpread, 1f).Width - FanMargin);

    /// <summary>The box a fan covering an angle fills, counting the outer cards as the tilted rectangles they are.</summary>
    private static Size Extent(float spread, float scale)
    {
        float tilt = spread / 2f;
        float w = StandardCardWidth * scale;
        float h = StandardCardHeight * scale;
        float reach = StandardReach * scale;
        float across = (2f * reach * MathF.Sin(tilt)) + (w * MathF.Cos(tilt)) + (h * MathF.Sin(tilt));

        // The middle card stands upright at the top; the outer ones are lower and lean.
        float outerBottom = (h / 2f) + (reach * (1f - MathF.Cos(tilt))) + (((w * MathF.Sin(tilt)) + (h * MathF.Cos(tilt))) / 2f);
        return new Size(MathF.Ceiling(across + FanMargin), MathF.Ceiling(MathF.Max(h, outerBottom) + FanMargin));
    }

    private int _count;

    public int Count
    {
        get => _count;
        set
        {
            if (_count != value)
            {
                _count = value;
                Invalidate();
            }
        }
    }

    private IReadOnlyList<Cantrip.Protocol.ObjectView> _known = [];

    /// <summary>
    /// The cards in the hand this player knows, drawn face up at the left of the fan. Where they sit says nothing about
    /// where they are in the hand; the server does not send that.
    /// </summary>
    public IReadOnlyList<Cantrip.Protocol.ObjectView> Known
    {
        get => _known;
        set
        {
            if (!_known.Select(card => card.Handle).SequenceEqual(value.Select(card => card.Handle)))
            {
                _known = value;
                Invalidate();
            }
        }
    }

    /// <summary>
    /// The right button went down on a known card, to hold it up and read it: the card, its box upright about its
    /// middle in this element, and how far it leans, in degrees.
    /// </summary>
    public event Action<Cantrip.Protocol.ObjectView, Rect, float>? CardInspected;

    public HandFanElement()
    {
        PointerPressed += (_, e) =>
        {
            Microsoft.UI.Input.PointerPoint point = e.GetCurrentPoint(this);
            if (!point.Properties.IsRightButtonPressed)
            {
                return;
            }

            e.Handled = true;
            if (CardAt(point.Position) is { } index && index < _known.Count)
            {
                (SKPoint centre, float angle, float width, float height) = Placed(index, new Size(ActualWidth, ActualHeight));
                CardInspected?.Invoke(
                    _known[index],
                    new Rect(centre.X - (width / 2), centre.Y - (height / 2), width, height),
                    angle * 180f / MathF.PI);
            }
        };
    }

    /// <summary>How many cards the fan shows: the hand, or the cards known in it if more are known than counted.</summary>
    private int Shown => Math.Max(_count, _known.Count);

    /// <summary>Where the card at an index stands in a fan drawn over an area: its middle, its lean in radians, and its size.</summary>
    private (SKPoint Centre, float Angle, float Width, float Height) Placed(int index, Size area)
    {
        int count = Shown;
        float scale = ScaleFor((float)area.Width);
        float cardWidth = MathF.Round(StandardCardWidth * scale);
        float cardHeight = MathF.Round(StandardCardHeight * scale);
        float reach = StandardReach * scale;

        float spread = MathF.Min(MaxSpread, StepAngle * (count - 1));
        var pivot = new SKPoint((float)area.Width / 2, (cardHeight / 2) + 2 + reach);
        float angle = count == 1 ? 0 : -spread / 2 + (spread * index / (count - 1));

        // Swung about the pivot below the fan, reach away from it.
        var centre = new SKPoint(pivot.X + (reach * MathF.Sin(angle)), pivot.Y - (reach * MathF.Cos(angle)));
        return (centre, angle, cardWidth, cardHeight);
    }

    /// <summary>Each known card and its box upright about its middle, for the page's test state.</summary>
    public IEnumerable<(Cantrip.Protocol.ObjectView Card, Rect Box)> Targets()
    {
        var area = new Size(ActualWidth, ActualHeight);
        for (int i = 0; i < _known.Count; i++)
        {
            (SKPoint centre, _, float width, float height) = Placed(i, area);
            yield return (_known[i], new Rect(centre.X - (width / 2), centre.Y - (height / 2), width, height));
        }
    }

    /// <summary>The index of the card under a point, the one drawn over the rest where they overlap; null for none.</summary>
    private int? CardAt(Point at)
    {
        Size area = new Size(ActualWidth, ActualHeight);
        for (int i = Shown - 1; i >= 0; i--)
        {
            (SKPoint centre, float angle, float width, float height) = Placed(i, area);

            // The point turned back with the card, so its box is upright.
            float dx = (float)at.X - centre.X;
            float dy = (float)at.Y - centre.Y;
            float along = (dx * MathF.Cos(angle)) + (dy * MathF.Sin(angle));
            float down = (-dx * MathF.Sin(angle)) + (dy * MathF.Cos(angle));
            if (MathF.Abs(along) <= width / 2 && MathF.Abs(down) <= height / 2)
            {
                return i;
            }
        }

        return null;
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        for (int i = 0; i < Shown; i++)
        {
            (SKPoint centre, float angle, float cardWidth, float cardHeight) = Placed(i, area);

            canvas.Save();
            canvas.Translate(centre.X, centre.Y);
            canvas.RotateRadians(angle);

            if (i < _known.Count)
            {
                canvas.Translate(-cardWidth / 2, -cardHeight / 2);
                CardFace.Draw(canvas, _known[i], cardWidth, cardHeight, dimmed: false);
            }
            else
            {
                CardFace.DrawBack(canvas, SKRect.Create(-cardWidth / 2, -cardHeight / 2, cardWidth, cardHeight), cardWidth);
            }

            canvas.Restore();
        }
    }
}

/// <summary>The player's picture, when the table has one (<see cref="Portraits"/>), and a generic head and shoulders until then.</summary>
public sealed class PortraitElement : SKCanvasElement
{
    private static readonly SKColor Ground = SKColor.Parse("262b34");
    private static readonly SKColor Figure = new(0x6c, 0x74, 0x82, 140);

    private string _source = string.Empty;
    private SKImage? _image;

    /// <summary>Where the picture is, as the table says (empty for none); drawn once it has come.</summary>
    public string Source
    {
        get => _source;
        set
        {
            if (_source == value) return;
            _source = value;
            _image = null;
            Invalidate();
            if (value.Length > 0) _ = ShowAsync(value);
        }
    }

    private async Task ShowAsync(string source)
    {
        SKImage? image = await Portraits.Get(source);
        if (image is null || _source != source) return;
        _image = image;
        Invalidate();
    }

    protected override void RenderOverride(SKCanvas canvas, Size area) =>
        Draw(canvas, SKRect.Create(0, 0, (float)area.Width, (float)area.Height), 6, _image);

    /// <summary>
    /// A portrait in <paramref name="face"/>, with corners of <paramref name="radius"/>: the middle of
    /// <paramref name="image"/>, as much of it as fills the square, or the head and shoulders without one.
    /// </summary>
    public static void Draw(SKCanvas canvas, SKRect face, float radius, SKImage? image)
    {
        canvas.Save();
        canvas.ClipRoundRect(new SKRoundRect(face, radius, radius), antialias: true);

        using var ground = new SKPaint { Color = Ground, IsAntialias = true };
        canvas.DrawRect(face, ground);

        if (image is not null)
        {
            float side = Math.Min(image.Width, image.Height);
            var middle = SKRect.Create((image.Width - side) / 2, (image.Height - side) / 2, side, side);
            canvas.DrawImage(image, middle, face, new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.Linear));
        }
        else
        {
            float w = face.Width;
            float h = face.Height;
            using var figure = new SKPaint { Color = Figure, IsAntialias = true };
            canvas.DrawCircle(face.Left + (w * 0.5f), face.Top + (h * 0.38f), w * 0.16f, figure);
            canvas.DrawOval(new SKPoint(face.Left + (w * 0.5f), face.Bottom - (h * 0.054f)), new SKSize(w * 0.32f, h * 0.36f), figure);
        }

        canvas.Restore();

        using var border = new SKPaint { Color = new SKColor(255, 255, 255, 15), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 1 };
        canvas.DrawRoundRect(SKRect.Inflate(face, -0.5f, -0.5f), radius, radius, border);
    }
}

/// <summary>One mana symbol on its own, for the mana pool, where a symbol says the colour.</summary>
public sealed class ManaPipElement : SKCanvasElement
{
    private readonly string _symbol;

    public ManaPipElement(string symbol, double diameter)
    {
        _symbol = symbol;
        Width = diameter;
        Height = diameter;
        VerticalAlignment = VerticalAlignment.Center;
        IsHitTestVisible = false;
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        float diameter = (float)Math.Min(area.Width, area.Height);
        ManaSymbols.Draw(canvas, new SKPoint((float)area.Width / 2f, (float)area.Height / 2f), diameter, _symbol, shadow: false);
    }
}
