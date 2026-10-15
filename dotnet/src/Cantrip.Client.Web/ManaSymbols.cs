using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// Mana symbols as coloured discs with the Mana icon font's marks on them, drawn with Skia. Costs, rules text and the dock all draw symbols through here, so a pip looks the same wherever it is.
/// </summary>
public static class ManaSymbols
{
    public static readonly SKColor Ink = SKColor.Parse("111111");

    private static readonly Dictionary<string, string> Glyphs = new(StringComparer.Ordinal)
    {
        ["W"] = "",
        ["U"] = "",
        ["B"] = "",
        ["R"] = "",
        ["G"] = "",
        ["0"] = "",
        ["1"] = "",
        ["2"] = "",
        ["3"] = "",
        ["4"] = "",
        ["5"] = "",
        ["6"] = "",
        ["7"] = "",
        ["8"] = "",
        ["9"] = "",
        ["10"] = "",
        ["11"] = "",
        ["12"] = "",
        ["13"] = "",
        ["14"] = "",
        ["15"] = "",
        ["16"] = "",
        ["17"] = "",
        ["18"] = "",
        ["19"] = "",
        ["20"] = "",
        ["X"] = "",
        ["S"] = "",
        ["P"] = "",
        ["T"] = "",
        ["Q"] = "",
        ["C"] = "",
        ["E"] = "",
    };

    private static readonly Dictionary<(string Glyph, int Size), SKRect> InkBounds = [];

    public static SKColor Fill(string symbol) => symbol switch
    {
        "W" => SKColor.Parse("f0f2c0"),
        "U" => SKColor.Parse("b5cde3"),
        "B" => SKColor.Parse("aca29a"),
        "R" => SKColor.Parse("db8664"),
        "G" => SKColor.Parse("93b483"),
        "Q" => Ink,
        _ => SKColor.Parse("beb9b2"),
    };

    /// <summary>Oracle notation split into plain text and symbols: "{T}: Add {G}." is a symbol, text, a symbol, text.</summary>
    public static IEnumerable<(string Text, bool IsSymbol)> Parse(string notation)
    {
        int start = 0;

        while (start < notation.Length)
        {
            int open = notation.IndexOf('{', start);
            int close = open < 0 ? -1 : notation.IndexOf('}', open + 1);

            if (open < 0 || close < 0)
            {
                yield return (notation[start..], false);
                yield break;
            }

            if (open > start)
            {
                yield return (notation[start..open], false);
            }

            yield return (notation[(open + 1)..close], true);
            start = close + 1;
        }
    }

    public static List<string> Symbols(string notation) =>
        [.. Parse(notation).Where(part => part.IsSymbol).Select(part => part.Text)];

    /// <summary>One symbol: its disc, a shadow down and to the left if asked for, and its mark.</summary>
    public static void Draw(SKCanvas canvas, SKPoint centre, float diameter, string symbol, bool shadow = true)
    {
        float radius = diameter / 2f;

        if (shadow)
        {
            // Down and to the left, as the font's stylesheet has it.
            using var shade = new SKPaint { Color = new SKColor(0, 0, 0, 153), IsAntialias = true };
            canvas.DrawCircle(centre.X - (diameter * 0.05f), centre.Y + (diameter * 0.07f), radius, shade);
        }

        // A two-colour hybrid symbol (CR 107.4e), {U/R}: the disc split corner to corner between its colours.
        if (symbol.Length == 3 && symbol[1] == '/' && Glyphs.ContainsKey(symbol[..1]) && Glyphs.ContainsKey(symbol[2..]))
        {
            DrawHybrid(canvas, centre, diameter, symbol[..1], symbol[2..]);
            return;
        }

        using (var disc = new SKPaint { Color = Fill(symbol), IsAntialias = true })
        {
            canvas.DrawCircle(centre, radius, disc);
        }

        SKColor ink = symbol == "Q" ? SKColors.White : Ink;

        if (Glyphs.TryGetValue(symbol, out string? glyph))
        {
            // The stylesheet sets the mark at 0.95em inside a 1.3em disc.
            int size = Math.Max(4, (int)MathF.Round(diameter * 0.73f));
            DrawGlyph(canvas, centre, glyph, size, ink);
            return;
        }

        // Letters on a plain disc, for a symbol the font has no mark for. Shrunk to fit: "W/U" is three wide.
        int letters = Math.Max(4, (int)MathF.Round(diameter * (symbol.Length > 1 ? 0.5f : 0.75f)));
        using var font = new SKFont(CardAssets.Title, letters);
        using var paint = new SKPaint { Color = ink, IsAntialias = true };
        float width = font.MeasureText(symbol);
        canvas.DrawText(symbol, centre.X - (width / 2f), centre.Y + ((-font.Metrics.Ascent - font.Metrics.Descent) / 2f) - 1, SKTextAlign.Left, font, paint);
    }

    /// <summary>A bare mana mark without its disc, for the oversized symbol printed in a basic land's text box.</summary>
    public static void DrawMark(SKCanvas canvas, SKPoint centre, float size, string symbol, SKColor ink)
    {
        if (!Glyphs.TryGetValue(symbol, out string? glyph))
        {
            return;
        }

        int fontSize = Math.Max(4, (int)MathF.Round(size));
        var shadow = new SKPoint(size * -0.035f, size * 0.045f);
        DrawGlyph(canvas, centre + shadow, glyph, fontSize, new SKColor(0, 0, 0, 115));
        DrawGlyph(canvas, centre, glyph, fontSize, ink);
    }

    private static void DrawHybrid(SKCanvas canvas, SKPoint centre, float diameter, string first, string second)
    {
        float radius = diameter / 2f;

        using (var disc = new SKPaint { Color = Fill(first), IsAntialias = true })
        {
            canvas.DrawCircle(centre, radius, disc);
        }

        // The lower-right half, from the top-right corner round to the bottom-left one.
        using (var path = new SKPath())
        {
            const int steps = 24;
            for (int i = 0; i <= steps; i++)
            {
                float angle = (-MathF.PI / 4f) + (MathF.PI * i / steps);
                var point = new SKPoint(centre.X + (MathF.Cos(angle) * (radius - 0.5f)), centre.Y + (MathF.Sin(angle) * (radius - 0.5f)));
                if (i == 0)
                {
                    path.MoveTo(point);
                }
                else
                {
                    path.LineTo(point);
                }
            }

            path.Close();
            using var half = new SKPaint { Color = Fill(second), IsAntialias = true };
            canvas.DrawPath(path, half);
        }

        int size = Math.Max(3, (int)MathF.Round(diameter * 0.4f));
        float apart = diameter * 0.2f;
        DrawGlyph(canvas, new SKPoint(centre.X - apart, centre.Y - apart), Glyphs[first], size, Ink);
        DrawGlyph(canvas, new SKPoint(centre.X + apart, centre.Y + apart), Glyphs[second], size, Ink);
    }

    /// <summary>The loyalty shields (ms-loyalty-up, -down, -zero and -start upstream): a cost's, and the loyalty a planeswalker has.</summary>
    public const string LoyaltyUp = "\ue627";
    public const string LoyaltyDown = "\ue625";
    public const string LoyaltyZero = "\ue626";
    public const string LoyaltyStart = "\ue628";

    /// <summary>The mark of a creature that has just arrived (ms-ability-summoning-sickness upstream).</summary>
    public const string SummoningSickness = "\ue96a";

    /// <summary>The font's artist's brush, which an illustration credit is printed after.</summary>
    public const string ArtistBrush = "";

    /// <summary>How wide a mark's ink is at <paramref name="size"/>.</summary>
    public static float InkWidth(string glyph, int size)
    {
        using var font = new SKFont(CardAssets.Mana, size);
        font.MeasureText(glyph, out SKRect bounds);
        return bounds.Width;
    }

    /// <summary>A mark centred on its ink rather than its advance box: several of this font's glyphs sit at the left of a wider advance.</summary>
    public static void DrawGlyph(SKCanvas canvas, SKPoint centre, string glyph, int size, SKColor ink)
    {
        using var font = new SKFont(CardAssets.Mana, size);
        if (!InkBounds.TryGetValue((glyph, size), out SKRect bounds))
        {
            font.MeasureText(glyph, out bounds);
            InkBounds[(glyph, size)] = bounds;
        }

        using var paint = new SKPaint { Color = ink, IsAntialias = true };
        canvas.DrawText(glyph, centre.X - bounds.MidX, centre.Y - bounds.MidY, SKTextAlign.Left, font, paint);
    }

    /// <summary>
    /// A mark as a printed loyalty shield is: a soft drop shadow below it, a white border round it, then its ink. The
    /// border is stroked about the glyph's edge, half of it over the ink, which the ink is then drawn over.
    /// </summary>
    public static void DrawGlyphBordered(SKCanvas canvas, SKPoint centre, string glyph, int size, SKColor ink)
    {
        using var font = new SKFont(CardAssets.Mana, size);
        if (!InkBounds.TryGetValue((glyph, size), out SKRect bounds))
        {
            font.MeasureText(glyph, out bounds);
            InkBounds[(glyph, size)] = bounds;
        }

        float x = centre.X - bounds.MidX;
        float y = centre.Y - bounds.MidY;
        float border = Math.Max(1f, size * 0.07f);

        using var shadow = new SKPaint
        {
            Color = new SKColor(0, 0, 0, 170),
            IsAntialias = true,
            Style = SKPaintStyle.StrokeAndFill,
            StrokeWidth = border * 2f,
            MaskFilter = SKMaskFilter.CreateBlur(SKBlurStyle.Normal, border * 0.9f),
        };
        canvas.DrawText(glyph, x + (border * 0.6f), y + (border * 1.4f), SKTextAlign.Left, font, shadow);

        using var edge = new SKPaint
        {
            Color = SKColors.White,
            IsAntialias = true,
            Style = SKPaintStyle.StrokeAndFill,
            StrokeWidth = border * 2f,
            StrokeJoin = SKStrokeJoin.Miter,
            StrokeMiter = 4f,
        };
        canvas.DrawText(glyph, x, y, SKTextAlign.Left, font, edge);

        using var fill = new SKPaint { Color = ink, IsAntialias = true };
        canvas.DrawText(glyph, x, y, SKTextAlign.Left, font, fill);
    }
}
