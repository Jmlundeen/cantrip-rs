using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// The marks of creatures just arrived on a row of the battlefield (<see cref="CardFace.DrawArrival"/>), on a layer of
/// their own laid exactly over the row: each over its card's art, rising and falling slowly together. Only the marks are
/// drawn again each frame - the row, and every face on it, is not - and a mark is cut away where another card stands
/// over its own. Without the Animate cards setting the marks are still, and drawn only when the row is.
/// </summary>
public sealed class ArrivalsElement : SKCanvasElement
{
    /// <summary>How long one rise and fall takes.</summary>
    private const double BreathSeconds = 2.6;

    private readonly CardBoardElement _board;
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(33) };

    public ArrivalsElement(CardBoardElement board)
    {
        _board = board;
        IsHitTestVisible = false;
        _timer.Tick += (_, _) => Invalidate();

        // Followed only while on the page, so a row taken off it does not keep this alive.
        Loaded += (_, _) =>
        {
            _board.Drawn -= OnBoardDrawn;
            _board.Drawn += OnBoardDrawn;
        };
        Unloaded += (_, _) =>
        {
            _board.Drawn -= OnBoardDrawn;
            _timer.Stop();
        };
    }

    private void OnBoardDrawn(object? sender, EventArgs e) => Invalidate();

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        float breath = CardBoardElement.Animates ? (float)(DateTime.UtcNow.TimeOfDay.TotalSeconds / BreathSeconds % 1.0) : 0.5f;
        int shown = 0;

        foreach ((SKMatrix card, SKRect art, SKPath over) in _board.Arrivals())
        {
            using (over)
            {
                canvas.Save();
                canvas.ClipPath(over, SKClipOperation.Difference, antialias: true);
                canvas.Concat(in card);
                CardFace.DrawArrival(canvas, art, breath);
                canvas.Restore();
            }

            shown++;
        }

        if (shown > 0 && CardBoardElement.Animates)
        {
            if (!_timer.IsEnabled)
            {
                _timer.Start();
            }
        }
        else
        {
            _timer.Stop();
        }
    }
}
