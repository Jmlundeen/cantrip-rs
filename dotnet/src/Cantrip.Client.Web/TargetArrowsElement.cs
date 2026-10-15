using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// Arrows from a spell on the stack to what it is aimed at, drawn over the whole table: a curve bowed to one side, thin and faint at the spell and thick and solid at the target, ending in a head
/// that sits on the target's edge. Takes no clicks; it is only drawn.
/// </summary>
public sealed class TargetArrowsElement : SKCanvasElement
{
    /// <summary>How far the middle of an arrow is pushed sideways, as a share of its length.</summary>
    private const float Bow = 0.12f;
    private const float Thickness = 7f;
    private const float HeadLength = 20f;
    private const float HeadWidth = 16f;

    private static readonly SKColor Accent = SKColor.Parse("f1b44c");

    private List<(SKPoint From, SKPoint To)> _arrows = [];

    public TargetArrowsElement()
    {
        IsHitTestVisible = false;
    }

    public void Show(IReadOnlyList<(SKPoint From, SKPoint To)> arrows)
    {
        if (arrows.Count == 0 && _arrows.Count == 0)
        {
            return;
        }

        if (arrows.SequenceEqual(_arrows))
        {
            return;
        }

        _arrows = [.. arrows];
        Invalidate();
    }

    /// <summary>Where a line from <paramref name="from"/> towards the box's centre leaves the box, so a head lands on its edge.</summary>
    public static SKPoint Edge(SKRect box, SKPoint from)
    {
        var centre = new SKPoint(box.MidX, box.MidY);
        SKPoint line = centre - from;

        if (line.LengthSquared < 1f)
        {
            return centre;
        }

        float half = MathF.Abs(line.X) > 0.001f ? box.Width * 0.5f / MathF.Abs(line.X) : float.MaxValue;
        float tall = MathF.Abs(line.Y) > 0.001f ? box.Height * 0.5f / MathF.Abs(line.Y) : float.MaxValue;
        float along = MathF.Min(half, tall);

        return new SKPoint(centre.X - (line.X * along), centre.Y - (line.Y * along));
    }

    /// <summary>
    /// Where the label of the arrow from <paramref name="from"/> to <paramref name="to"/> is centred: just behind its
    /// head, along the way it comes in (TargetArrows.DrawArrow).
    /// </summary>
    public static SKPoint LabelAt(SKPoint from, SKPoint to)
    {
        SKPoint line = to - from;
        float length = line.Length;
        if (length < 1f)
        {
            return to;
        }

        var normal = new SKPoint(-line.Y / length, line.X / length);
        var middle = new SKPoint(((from.X + to.X) / 2f) + (normal.X * length * Bow), ((from.Y + to.Y) / 2f) + (normal.Y * length * Bow));
        SKPoint towards = to - Bend(from, middle, to, 0.92f);
        var approach = new SKPoint(towards.X / towards.Length, towards.Y / towards.Length);
        return new SKPoint(to.X - (approach.X * (HeadLength + 6f)), to.Y - (approach.Y * (HeadLength + 6f)));
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        foreach ((SKPoint from, SKPoint to) in _arrows)
        {
            DrawArrow(canvas, from, to);
        }
    }

    private static void DrawArrow(SKCanvas canvas, SKPoint from, SKPoint to) => DrawArrow(canvas, from, to, Accent, 0f, 1f, 1f);

    /// <summary>
    /// The arrow from <paramref name="from"/> to <paramref name="to"/>, or the part of it from <paramref name="start"/> to
    /// <paramref name="end"/> - shares of its whole length, tail and head together - with the head always at the leading
    /// end: so an arrow can be drawn growing out of where it comes from, and then drawing itself into what it points at.
    /// Each part of the tail is as wide and as solid as it is on the whole arrow, so a part looks like a piece of it.
    /// </summary>
    public static void DrawArrow(SKCanvas canvas, SKPoint from, SKPoint to, SKColor color, float start, float end, float opacity)
    {
        SKPoint line = to - from;
        float length = line.Length;
        if (length < 1f || end <= 0f || opacity <= 0f)
        {
            return;
        }

        // Bowed to one side, so two arrows to nearby targets stay apart and an arrow reads as pointing.
        var normal = new SKPoint(-line.Y / length, line.X / length);
        var middle = new SKPoint(((from.X + to.X) / 2f) + (normal.X * length * Bow), ((from.Y + to.Y) / 2f) + (normal.Y * length * Bow));

        // The head sits back from the target by its own length, so the curve ends where the head begins.
        SKPoint towards = to - Bend(from, middle, to, 0.92f);
        var approach = new SKPoint(towards.X / towards.Length, towards.Y / towards.Length);
        var neck = new SKPoint(to.X - (approach.X * HeadLength), to.Y - (approach.Y * HeadLength));

        // The curve's length as it goes, so a share of the arrow is a share of the way along it.
        Span<float> along = stackalloc float[Samples + 1];
        along[0] = 0f;
        SKPoint previous = from;
        for (int i = 1; i <= Samples; i++)
        {
            SKPoint point = Bend(from, middle, neck, (float)i / Samples);
            along[i] = along[i - 1] + (point - previous).Length;
            previous = point;
        }

        float tail = along[Samples];
        float whole = tail + HeadLength;
        float tip = Math.Clamp(end, 0f, 1f) * whole;
        float back = Math.Clamp(start, 0f, 1f) * whole;

        // The head rides the leading end: on the curve while it is still growing, on the straight run in once it is there.
        SKPoint head;
        SKPoint heading;
        if (tip >= tail)
        {
            head = neck + Times(approach, tip - tail);
            heading = approach;
        }
        else
        {
            float u = Parameter(along, tip);
            head = Bend(from, middle, neck, u);
            SKPoint tangent = Bend(from, middle, neck, MathF.Min(1f, u + 0.01f)) - Bend(from, middle, neck, MathF.Max(0f, u - 0.01f));
            heading = tangent.Length > 0.0001f ? Times(tangent, 1f / tangent.Length) : approach;
        }

        // The tail runs from the back of what is shown to the head's base, and a head drawn into its target takes the
        // tail's end with it.
        float tailEnd = MathF.Min(tail, tip - HeadLength);
        if (tailEnd > back)
        {
            DrawTail(canvas, from, middle, neck, Parameter(along, back), Parameter(along, tailEnd), color, opacity);
        }

        // Shortened as the arrow's back passes into it, so the head too is drawn into what it points at.
        float headLength = MathF.Min(HeadLength, tip - back);
        float headWidth = HeadWidth * MathF.Min(1f, tip / HeadLength);
        if (headLength <= 0.5f)
        {
            return;
        }

        SKPoint headBase = head - Times(heading, headLength);
        float spread = headWidth * 0.5f * (headLength / HeadLength);
        var across = new SKPoint(-heading.Y * spread, heading.X * spread);
        using var path = new SKPath();
        path.MoveTo(head);
        path.LineTo(headBase + across);
        path.LineTo(headBase - across);
        path.Close();

        using var headPaint = new SKPaint { Color = color.WithAlpha((byte)MathF.Round(217 * opacity)), IsAntialias = true };
        canvas.DrawPath(path, headPaint);
    }

    /// <summary>How finely the curve is measured.</summary>
    private const int Samples = 48;

    /// <summary>Where on the curve a distance along it falls, from the lengths measured at <see cref="Samples"/> even steps.</summary>
    private static float Parameter(ReadOnlySpan<float> along, float distance)
    {
        if (distance <= 0f)
        {
            return 0f;
        }

        for (int i = 1; i < along.Length; i++)
        {
            if (along[i] >= distance)
            {
                float step = along[i] - along[i - 1];
                float within = step > 0.0001f ? (distance - along[i - 1]) / step : 0f;
                return (i - 1 + within) / (along.Length - 1);
            }
        }

        return 1f;
    }

    /// <summary>
    /// The curve as quads that widen from one to the full thickness and fade in towards the target, from
    /// <paramref name="first"/> to <paramref name="last"/> of the way along it.
    /// </summary>
    private static void DrawTail(SKCanvas canvas, SKPoint from, SKPoint middle, SKPoint to, float first, float last, SKColor color, float opacity)
    {
        const int segments = 24;
        int count = Math.Max(1, (int)MathF.Ceiling(segments * (last - first)));
        SKPoint previous = Bend(from, middle, to, first);
        float previousWidth = 1f + ((Thickness - 1f) * first);

        using var paint = new SKPaint { IsAntialias = true };

        for (int i = 1; i <= count; i++)
        {
            float u = first + ((last - first) * i / count);
            SKPoint point = Bend(from, middle, to, u);
            SKPoint step = point - previous;
            float stepLength = MathF.Max(0.0001f, step.Length);
            var across = new SKPoint(-step.Y / stepLength, step.X / stepLength);
            float width = 1f + ((Thickness - 1f) * u);

            using var quad = new SKPath();
            quad.MoveTo(previous + Times(across, previousWidth * 0.5f));
            quad.LineTo(point + Times(across, width * 0.5f));
            quad.LineTo(point - Times(across, width * 0.5f));
            quad.LineTo(previous - Times(across, previousWidth * 0.5f));
            quad.Close();

            paint.Color = color.WithAlpha((byte)MathF.Round(255 * opacity * (0.15f + (0.65f * u))));
            canvas.DrawPath(quad, paint);

            previous = point;
            previousWidth = width;
        }
    }

    private static SKPoint Times(SKPoint point, float by) => new(point.X * by, point.Y * by);

    private static SKPoint Bend(SKPoint from, SKPoint middle, SKPoint to, float along)
    {
        SKPoint a = Lerp(from, middle, along);
        SKPoint b = Lerp(middle, to, along);
        return Lerp(a, b, along);
    }

    private static SKPoint Lerp(SKPoint a, SKPoint b, float t) => new(a.X + ((b.X - a.X) * t), a.Y + ((b.Y - a.Y) * t));
}
