using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// What a player is in the middle of picking creatures for, drawn over the whole table: a red arrow from each creature
/// picked up to the pointer, the rectangle dragged out over an empty part of the battlefield, and the arrows that have
/// just been sent somewhere, drawing themselves in to what they were sent at. Takes no clicks; it is only drawn.
/// </summary>
public sealed class SelectionOverlayElement : SKCanvasElement
{
    private static readonly SKColor Red = SKColor.Parse("e5484d");

    /// <summary>How long an arrow takes to draw itself into what it was sent at.</summary>
    private static readonly TimeSpan Retirement = TimeSpan.FromMilliseconds(380);

    private sealed record Retiring(SKPoint From, SKPoint To, DateTime Started);

    private List<SKPoint> _held = [];
    private SKPoint? _pointer;
    private SKRect? _marquee;
    private readonly List<Retiring> _retiring = [];
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(16) };

    public SelectionOverlayElement()
    {
        IsHitTestVisible = false;
        _timer.Tick += (_, _) =>
        {
            _retiring.RemoveAll(arrow => DateTime.UtcNow - arrow.Started >= Retirement);
            if (_retiring.Count == 0)
            {
                _timer.Stop();
            }

            Invalidate();
        };
    }

    /// <summary>Arrows still being retired, for a test to wait on.</summary>
    public int Retirements => _retiring.Count;

    /// <summary>Whether anything is drawn, or about to be.</summary>
    public bool Busy => _held.Count > 0 || _marquee is not null || _retiring.Count > 0;

    /// <summary>
    /// The creatures picked up, where each stands, and where the pointer is, which every one of them points at; and the
    /// rectangle being dragged out, if one is.
    /// </summary>
    public void Hold(IReadOnlyList<SKPoint> from, SKPoint? pointer, SKRect? marquee)
    {
        if (_held.SequenceEqual(from) && _pointer == pointer && _marquee == marquee)
        {
            return;
        }

        _held = [.. from];
        _pointer = pointer;
        _marquee = marquee;
        Invalidate();
    }

    /// <summary>Sends arrows from where they stood to what they were sent at, to draw themselves in there.</summary>
    public void Retire(IEnumerable<(SKPoint From, SKPoint To)> arrows)
    {
        DateTime now = DateTime.UtcNow;
        _retiring.AddRange(arrows.Select(arrow => new Retiring(arrow.From, arrow.To, now)));

        if (_retiring.Count > 0)
        {
            _timer.Start();
        }

        Invalidate();
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        if (_marquee is { } box)
        {
            using var fill = new SKPaint { Color = Red.WithAlpha(40), IsAntialias = true };
            using var edge = new SKPaint { Color = Red.WithAlpha(190), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 1.5f };
            canvas.DrawRect(box, fill);
            canvas.DrawRect(box, edge);
        }

        if (_pointer is { } at)
        {
            foreach (SKPoint from in _held)
            {
                TargetArrowsElement.DrawArrow(canvas, from, at, Red, 0f, 1f, 1f);
            }
        }

        foreach (Retiring arrow in _retiring)
        {
            float done = (float)Math.Clamp((DateTime.UtcNow - arrow.Started) / Retirement, 0, 1);

            // Fast at first and slowing into the target, as the arrow is drawn in.
            float eased = 1f - ((1f - done) * (1f - done));
            TargetArrowsElement.DrawArrow(canvas, arrow.From, arrow.To, Red, eased, 1f, 1f - (0.5f * done));
        }
    }
}
