using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A booster draft's table seen from above: an oval with every player in their seat, the packs waiting in front of
/// each, and arrows round the table the way the packs are going. This seat is at the bottom, so passing left - to the
/// next seat up - goes off to its left, as it would at a real table.
/// <para>
/// The packs in front of a seat are public, as they are at a real table: a player taking their time has packs piling
/// up in front of them, which everyone can see.
/// </para>
/// </summary>
public sealed class DraftTableElement : SKCanvasElement
{
    private static readonly SKColor Felt = SKColor.Parse("1b2b23");
    private static readonly SKColor FeltEdge = SKColor.Parse("3b5546");
    private static readonly SKColor Plate = SKColor.Parse("1d2127");
    private static readonly SKColor PlateEdge = SKColor.Parse("3a4250");
    private static readonly SKColor Mine = SKColor.Parse("f1b44c");
    private static readonly SKColor Text = SKColor.Parse("e8eaee");
    private static readonly SKColor Soft = SKColor.Parse("a6adb8");
    private static readonly SKColor Arrow = SKColor.Parse("6fcf8f");
    private static readonly SKColor Piling = SKColor.Parse("e8a33d");

    /// <summary>
    /// One seat at the table: who, how many packs wait for them, how many cards they have, and their picture as the
    /// table names it (empty for none).
    /// </summary>
    public sealed record Seat(int Number, string Name, int Queued, int Drafted, string Portrait = "");

    private const float PlateWidth = 146f;
    private const float PlateHeight = 40f;
    private const float PictureSide = 30f;

    private IReadOnlyList<Seat> _seats = [];
    private int _me;
    private int _direction = 1;
    private string _caption = string.Empty;

    /// <summary>Shows the table as it now stands. <paramref name="direction"/> is +1 when packs go to the next seat up.</summary>
    public void Show(IReadOnlyList<Seat> seats, int me, int direction, string caption)
    {
        _seats = seats;
        _me = me;
        _direction = direction >= 0 ? 1 : -1;
        _caption = caption;
        Invalidate();

        foreach (Seat seat in seats)
        {
            if (seat.Portrait.Length > 0 && Portraits.Peek(seat.Portrait) is null) _ = ShowWhenHereAsync(seat.Portrait);
        }
    }

    /// <summary>Draws the table again once a picture has come.</summary>
    private async Task ShowWhenHereAsync(string portrait)
    {
        if (await Portraits.Get(portrait) is not null) Invalidate();
    }

    /// <summary>Where a seat sits, as an angle round the oval: this seat at the bottom, the next seat up to its left.</summary>
    private float AngleOf(int seat) => (MathF.PI / 2f) + ((seat - _me) * 2f * MathF.PI / Math.Max(1, _seats.Count));

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        float width = (float)area.Width;
        float height = (float)area.Height;
        if (_seats.Count == 0 || width < 40 || height < 40)
        {
            return;
        }

        float cx = width / 2f;
        float cy = height / 2f;
        // The seats sit on an oval a little outside the table's edge.
        float seatRx = (width / 2f) - (PlateWidth / 2f) - 4f;
        float seatRy = (height / 2f) - 24f;
        var table = SKRect.Create(cx - (seatRx * 0.78f), cy - (seatRy * 0.62f), seatRx * 1.56f, seatRy * 1.24f);

        using (var felt = new SKPaint { Color = Felt, IsAntialias = true })
        using (var edge = new SKPaint { Color = FeltEdge, IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 3 })
        {
            canvas.DrawOval(table, felt);
            canvas.DrawOval(table, edge);
        }

        DrawArrows(canvas, cx, cy, table.Width / 2f * 0.80f, table.Height / 2f * 0.72f);

        using (var font = new SKFont(CardAssets.Body, 13))
        using (var caption = new SKPaint { Color = Soft, IsAntialias = true })
        {
            canvas.DrawText(_caption, cx, cy + 5, SKTextAlign.Center, font, caption);
        }

        foreach (Seat seat in _seats)
        {
            float angle = AngleOf(seat.Number);
            DrawSeat(canvas, seat, cx + (seatRx * MathF.Cos(angle)), cy + (seatRy * MathF.Sin(angle)));
        }
    }

    /// <summary>An arrow from each seat to the next round the table, the way the packs are going.</summary>
    private void DrawArrows(SKCanvas canvas, float cx, float cy, float rx, float ry)
    {
        using var line = new SKPaint { Color = Arrow.WithAlpha(170), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = 2.2f, StrokeCap = SKStrokeCap.Round };
        using var head = new SKPaint { Color = Arrow.WithAlpha(200), IsAntialias = true };
        float step = 2f * MathF.PI / _seats.Count;
        foreach (Seat seat in _seats)
        {
            // From just past this seat to just short of the next, round the inside of the table.
            float from = AngleOf(seat.Number);
            float sweep = step * _direction * 0.62f;
            float start = from + (step * _direction * 0.19f);

            using var path = new SKPath();
            const int segments = 12;
            SKPoint last = default;
            SKPoint before = default;
            for (int i = 0; i <= segments; i++)
            {
                float a = start + (sweep * i / segments);
                var point = new SKPoint(cx + (rx * MathF.Cos(a)), cy + (ry * MathF.Sin(a)));
                if (i == 0) path.MoveTo(point);
                else path.LineTo(point);
                before = last;
                last = point;
            }

            canvas.DrawPath(path, line);

            // The head, pointing along the last bit of the arc.
            float dx = last.X - before.X;
            float dy = last.Y - before.Y;
            float length = MathF.Max(0.001f, MathF.Sqrt((dx * dx) + (dy * dy)));
            dx /= length;
            dy /= length;
            using var arrowHead = new SKPath();
            arrowHead.MoveTo(last.X + (dx * 6), last.Y + (dy * 6));
            arrowHead.LineTo(last.X - (dx * 4) - (dy * 5), last.Y - (dy * 4) + (dx * 5));
            arrowHead.LineTo(last.X - (dx * 4) + (dy * 5), last.Y - (dy * 4) - (dx * 5));
            arrowHead.Close();
            canvas.DrawPath(arrowHead, head);
        }
    }

    /// <summary>A seat's plate: the player's picture, the name, how many cards drafted, and the packs waiting in front of it.</summary>
    private void DrawSeat(SKCanvas canvas, Seat seat, float x, float y)
    {
        bool mine = seat.Number == _me;
        var plate = SKRect.Create(x - (PlateWidth / 2f), y - (PlateHeight / 2f), PlateWidth, PlateHeight);

        using (var fill = new SKPaint { Color = Plate, IsAntialias = true })
        using (var border = new SKPaint { Color = mine ? Mine : seat.Queued >= 3 ? Piling : PlateEdge, IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = mine ? 2 : 1.2f })
        {
            canvas.DrawRoundRect(plate, 6, 6, fill);
            canvas.DrawRoundRect(plate, 6, 6, border);
        }

        float inset = (PlateHeight - PictureSide) / 2f;
        PortraitElement.Draw(canvas, SKRect.Create(plate.Left + inset, plate.Top + inset, PictureSide, PictureSide), 4, Portraits.Peek(seat.Portrait));
        float textLeft = plate.Left + inset + PictureSide + 7;

        using var nameFont = new SKFont(CardAssets.Body, 13) { Embolden = mine };
        using var nameInk = new SKPaint { Color = mine ? Mine : Text, IsAntialias = true };
        string name = Fit(mine ? "You" : seat.Name, nameFont, plate.Right - 36 - textLeft);
        canvas.DrawText(name, textLeft, plate.Top + 17, SKTextAlign.Left, nameFont, nameInk);

        using var smallFont = new SKFont(CardAssets.Body, 11);
        using var softInk = new SKPaint { Color = Soft, IsAntialias = true };
        canvas.DrawText($"{seat.Drafted} drafted", textLeft, plate.Top + 32, SKTextAlign.Left, smallFont, softInk);

        // The packs waiting, as a little stack of backs with the count on them.
        float backWidth = 18f;
        float backHeight = 25f;
        float left = plate.Right - 32f;
        float top = plate.Top + 7f;
        int shown = Math.Min(seat.Queued, 4);
        for (int i = 0; i < shown; i++)
        {
            CardFace.DrawBack(canvas, SKRect.Create(left + (i * 2.5f), top - (i * 2.5f), backWidth, backHeight), backWidth);
        }

        if (seat.Queued > 0)
        {
            var badge = new SKPoint(left + backWidth + 4, top + backHeight - 2);
            using var badgeFill = new SKPaint { Color = seat.Queued >= 3 ? Piling : Arrow, IsAntialias = true };
            canvas.DrawCircle(badge, 8, badgeFill);
            using var badgeFont = new SKFont(CardAssets.Body, 11) { Embolden = true };
            using var badgeInk = new SKPaint { Color = SKColor.Parse("17130a"), IsAntialias = true };
            canvas.DrawText(seat.Queued.ToString(System.Globalization.CultureInfo.InvariantCulture), badge.X, badge.Y + 4, SKTextAlign.Center, badgeFont, badgeInk);
        }
    }

    /// <summary>The name cut short with an ellipsis where it would not fit.</summary>
    private static string Fit(string text, SKFont font, float width)
    {
        if (font.MeasureText(text) <= width) return text;
        for (int length = text.Length - 1; length > 0; length--)
        {
            string cut = text[..length] + "…";
            if (font.MeasureText(cut) <= width) return cut;
        }

        return text[..1];
    }
}
