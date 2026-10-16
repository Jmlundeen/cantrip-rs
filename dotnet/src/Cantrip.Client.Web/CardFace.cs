using Cantrip.Core;
using Cantrip.Protocol;
using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// A card's face, laid out from the retro template at the card's own logical width: a thumbnail on the table shrinks,
/// clips and drops lines by the layout's rounding and smallest sizes. The caller scales the canvas for the device, not
/// this.
/// <para>
/// Composited like the source artwork: illustration, then the transparent frame, then the printed text, then the marks of
/// game state over the printed card - counters, then damage last so nothing already on the permanent hides it.
/// </para>
/// </summary>
public static partial class CardFace
{
    /// <summary>The width the design sizes of state marks are given at.</summary>
    public const float StandardWidth = 104f;

    private static readonly SKColor Parchment = SKColor.Parse("e8e2cc");
    private static readonly SKColor InkColor = SKColor.Parse("14130f");
    private static readonly SKColor Grey = SKColor.Parse("808080");

    /// <summary>How much of a font size a digit actually stands.</summary>
    private const float FigureHeight = 0.72f;

    /// <param name="body">
    /// Whether power and toughness, or a planeswalker's loyalty, are drawn. A face kept to be shown again is drawn without
    /// them, since they change often, and <see cref="DrawBodyOver"/> draws them over it wherever it is shown.
    /// </param>
    /// <param name="pixelScale">
    /// How many pixels to the unit the face is meant to be seen at, for the size of art to ask for. Not read from the
    /// canvas: a face is usually recorded to be played back later, at a size its recording canvas does not know.
    /// </param>
    public static void Draw(SKCanvas canvas, ObjectView card, float width, float height, bool dimmed, string? annotation = null, bool body = true, float pixelScale = 2f)
    {
        var face = SKRect.Create(width, height);

        if (card.FaceDown)
        {
            DrawFaceDown(canvas, card, face, width, body);
            return;
        }

        if (!card.Known)
        {
            DrawBack(canvas, face, width);
            return;
        }

        Layout layout = Layout.Of(card, width, height);
        if (card.Halves.Count > 0 && CardAssets.Template.SplitHalf is not null)
        {
            DrawSplit(canvas, card, width, height, dimmed, body, pixelScale);
        }
        else
        {
            DrawPrinted(canvas, card, layout, face, dimmed, body, pixelScale);
        }

        DrawStatus(canvas, card, Ink(dimmed), layout);

        DrawCounters(canvas, card, layout, width);
        if (annotation is { Length: > 0 })
        {
            // What the choice being made does with it, such as where a card put back is going.
            DrawAnnotation(canvas, annotation, layout, width);
        }
        else if (card.Choices.Count == 1)
        {
            // One line fits over the art, so a permanent with a single chosen quality wears it.
            DrawAnnotation(canvas, card.Choices[0].Value, layout, width);
        }

        DrawCopyTag(canvas, card, layout, width);

        if (EyeBox(card, width, height) is { } eye)
        {
            DrawEye(canvas, eye);
        }

        DrawMarkedDamage(canvas, card, layout);
    }


    /// <summary>
    /// What is printed on a face: art, frame, name and cost, type line, rules, and power and toughness when
    /// <paramref name="body"/>. Laid out by <paramref name="layout"/> over <paramref name="face"/>, which is a whole card,
    /// or one half of a split card.
    /// </summary>
    private static void DrawPrinted(SKCanvas canvas, ObjectView card, Layout layout, SKRect face, bool dimmed, bool body, float pixelScale)
    {
        SKColor frame = FrameColor(card);
        if (dimmed)
        {
            frame = Lerp(frame, DimmedFrame, 0.55f);
        }

        SKColor text = Ink(dimmed);
        SKColor outerText = SKColor.Parse(CardAssets.Template.TextStyle.OuterTextColor);
        SKColor outerShadow = ParseAlpha(CardAssets.Template.TextStyle.OuterTextShadowColor);

        DrawArt(canvas, card, layout, frame, dimmed, pixelScale);

        using (var framePaint = new SKPaint { IsAntialias = true })
        {
            if (dimmed)
            {
                framePaint.ColorFilter = SKColorFilter.CreateBlendMode(new SKColor(158, 158, 158), SKBlendMode.Modulate);
            }

            DrawFrame(canvas, card, face, framePaint, layout.Cut);
        }

        // The cost first, top right where a card carries it, and the name takes what is left short of a symbol's gap.
        float costWidth = DrawCost(canvas, card, layout, !dimmed);
        float nameX = layout.Title.Left + layout.TextInset;
        float titleRoom = Room(layout, nameX, layout.Title) - (costWidth > 0f ? costWidth + (layout.ManaDiameter / 3f) : 0f);
        int nameSize = FittedSize(CardAssets.Title, card.Name, layout.TitleFontSize, layout.MinimumTitleFontSize, titleRoom);
        DrawFrameString(canvas, nameX, CapBaseline(CardAssets.Title, layout.Title, nameSize), Fit(CardAssets.Title, card.Name, nameSize, titleRoom), nameSize, outerText, outerShadow, layout.OuterTextShadowOffset);

        string typeLine = TypeLine(card);
        float typeX = layout.Type.Left + layout.TextInset;
        float typeRoom = Room(layout, typeX, layout.Type);
        int typeSize = FittedSize(CardAssets.Title, typeLine, layout.TypeFontSize, layout.MinimumTypeFontSize, typeRoom);
        DrawFrameString(canvas, typeX, CapBaseline(CardAssets.Title, layout.Type, typeSize), Fit(CardAssets.Title, typeLine, typeSize, typeRoom), typeSize, outerText, outerShadow, layout.OuterTextShadowOffset);

        if (layout.Panel is not null)
        {
            DrawChapters(canvas, card, text, layout, dimmed);
        }
        else
        {
            DrawOracle(canvas, card, text, layout);
            DrawLandSymbol(canvas, card, layout);
        }

        // Game state wears the credit's place at the foot of the card: a creature just arrived says NEW there instead.
        if (StatusWords(card).Count == 0)
        {
            DrawArtist(canvas, card, layout, outerText, outerShadow);
        }

        if (body)
        {
            DrawBody(canvas, card, frame, layout, outerText, outerShadow);
        }
    }

    private static SKColor Ink(bool dimmed) => dimmed ? Lerp(InkColor, Grey, 0.4f) : InkColor;

    /// <summary>
    /// A split card as it is printed (CR 709.1): its halves side by side along its length, each a card of its own as tall
    /// as this one is wide - cut down from the template rather than shrunk, so the border and the name and cost are the
    /// size a whole card's are here - and the pair turned a quarter turn, so the card is read by turning it clockwise.
    /// </summary>
    private static void DrawSplit(SKCanvas canvas, ObjectView card, float width, float height, bool dimmed, bool body, float pixelScale)
    {
        CardTemplate.SplitLayout split = CardAssets.Template.SplitHalf!;
        canvas.Save();

        // The pair's left end at the card's foot and its top down the card's left side.
        canvas.Translate(0f, height);
        canvas.RotateDegrees(-90f);

        Layout layout = Layout.HalfOf(width, height, card.Halves.Count);
        float overlap = split.Overlap * layout.Scale;
        for (int i = 0; i < card.Halves.Count; i++)
        {
            // Each laid over the edge of the one before it, so the line between them is one border.
            canvas.Save();
            canvas.Translate(i * (layout.Width - overlap), 0f);
            DrawPrinted(canvas, card.Halves[i], layout, SKRect.Create(layout.Width, layout.Height), dimmed, body, pixelScale);

            // A Room's door that is locked (CR 709.5c): its printed face is still all there to read, under a veil and a word.
            if (card.UnlockedDoors.Count == card.Halves.Count && !card.UnlockedDoors[i])
            {
                DrawLocked(canvas, SKRect.Create(layout.Width, layout.Height));
            }

            canvas.Restore();
        }

        canvas.Restore();
    }

    /// <summary>The mark of a locked door (CR 709.5): a translucent veil over the door's face, so what is printed on it still reads, and LOCKED across it.</summary>
    private static void DrawLocked(SKCanvas canvas, SKRect door)
    {
        using var veil = new SKPaint { Color = new SKColor(10, 10, 20, 120), IsAntialias = true };
        canvas.DrawRect(door, veil);

        float size = MathF.Max(8f, door.Height * 0.16f);
        using var font = new SKFont(CardAssets.Title, size);
        string word = "LOCKED";
        float x = door.MidX - (font.MeasureText(word) / 2f);
        float y = door.MidY + (size * FigureHeight / 2f);
        using var halo = new SKPaint { Color = new SKColor(0, 0, 0, 220), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = MathF.Max(1.5f, size * 0.12f) };
        using var ink = new SKPaint { Color = new SKColor(255, 214, 120), IsAntialias = true };
        canvas.DrawText(word, x, y, font, halo);
        canvas.DrawText(word, x, y, font, ink);
    }

    /// <summary>
    /// The names a card's face shows art for: each half's on a split card, which is drawn as its halves, and otherwise its
    /// own.
    /// </summary>
    public static IEnumerable<string> ArtNames(ObjectView card) =>
        card.Halves.Count > 0 ? card.Halves.Select(half => half.Name) : [card.Name];

    /// <summary>
    /// The art a card's face is drawn with, as one key: its slug, or a split card's halves' slugs together
    /// (<see cref="CardArt.Covers"/>).
    /// </summary>
    public static string ArtSlug(ObjectView card) => string.Join(' ', ArtNames(card).Select(CardArt.Slug));

    /// <summary>
    /// The damage share's band across the middle of the face, its number, and the two steps either side of it, in one
    /// place, so what is drawn and what is clicked cannot drift apart. The band runs from the top of the art to
    /// the bottom of the rules box, so the name, the power and toughness and the damage already marked - what the player
    /// weighs the share against - stay in view.
    /// </summary>
    public static (float Size, SKRect Band, SKRect Number, SKRect Lower, SKRect Raise) DamageBoxes(float width, float height, int share)
    {
        var layout = new Layout(width, height);
        var band = new SKRect(0f, layout.Art.Top, width, MathF.Max(layout.Art.Top + 1f, layout.Rules.Bottom));
        float gap = MathF.Max(2f, MathF.Round(width * 0.03f));
        float numberWidth = width * 0.5f;
        float left = MathF.Round((width - numberWidth) / 2f);

        // Monospaced by digit count: measured from a row of zeros, so a 1 and a 7 are drawn at one size.
        using var font = new SKFont(CardAssets.Title, 100);
        float unit = font.MeasureText(new string('0', share < 10 ? 1 : share < 100 ? 2 : 3)) / 100f;
        float minimum = Math.Max(7, (int)MathF.Round(11 * (width / StandardWidth)));
        float size = MathF.Max(minimum, MathF.Round(MathF.Min(numberWidth / MathF.Max(unit, 0.01f), band.Height / FigureHeight)));

        float stepWidth = MathF.Max(6f, left - (gap * 2f));
        float stepHeight = MathF.Min(band.Height, MathF.Round(stepWidth * 1.7f));
        float stepTop = MathF.Round(band.MidY - (stepHeight / 2f));

        return (
            size,
            band,
            SKRect.Create(left, band.Top, numberWidth, band.Height),
            SKRect.Create(gap, stepTop, stepWidth, stepHeight),
            SKRect.Create(left + numberWidth + gap, stepTop, stepWidth, stepHeight));
    }

    /// <summary>
    /// The share of a division this card is being sent - combat damage, a divided spell's damage, red, or distributed
    /// counters, green - large across the middle of the face, with a step down and up either side. Drawn over the face
    /// each frame, since its steps light under the pointer.
    /// </summary>
    public static void DrawDamageShare(SKCanvas canvas, float width, float height, int share, bool lowerLive, bool raiseLive, bool hoverLower, bool hoverRaise, bool counters = false)
    {
        SKColor tone = SKColor.Parse(counters ? "6fcf8f" : "e5534b");

        (float size, SKRect band, SKRect number, SKRect lower, SKRect raise) = DamageBoxes(width, height, share);

        using var ground = new SKPaint { Color = new SKColor(13, 13, 18, (byte)(0.82f * 255)), Style = SKPaintStyle.Fill };
        using var edge = new SKPaint { Color = tone.WithAlpha(128), Style = SKPaintStyle.Stroke, StrokeWidth = 1f };
        canvas.DrawRect(band, ground);
        canvas.DrawRect(SKRect.Inflate(band, -0.5f, -0.5f), edge);

        string text = share.ToString(System.Globalization.CultureInfo.InvariantCulture);
        using var font = new SKFont(CardAssets.Title, size);
        using var ink = new SKPaint { Color = tone, IsAntialias = true };

        // Centred on the digits' own height, so the number sits in the middle of the band.
        canvas.DrawText(text, number.MidX - (font.MeasureText(text) / 2f), number.MidY + (size * FigureHeight / 2f), font, ink);

        DrawStep(canvas, lower, "−", lowerLive, hoverLower);
        DrawStep(canvas, raise, "+", raiseLive, hoverRaise);
    }

    /// <summary>One step beside a damage share: a rounded box with a sign in it, lit while the pointer rests on it.</summary>
    private static void DrawStep(SKCanvas canvas, SKRect box, string sign, bool live, bool hover)
    {
        using var fill = new SKPaint { Color = SKColor.Parse(live && hover ? "353c48" : "2a303a"), Style = SKPaintStyle.Fill, IsAntialias = true };
        using var line = new SKPaint { Color = SKColor.Parse("3a4250"), Style = SKPaintStyle.Stroke, StrokeWidth = 1f, IsAntialias = true };
        canvas.DrawRoundRect(box, 5, 5, fill);
        canvas.DrawRoundRect(SKRect.Inflate(box, -0.5f, -0.5f), 5, 5, line);

        // Sized from the box's width, the narrow side, so the sign never spills across the number.
        float size = MathF.Max(8, MathF.Round(box.Width * 0.7f));
        using var font = new SKFont(SKTypeface.Default, size);
        using var ink = new SKPaint { Color = SKColor.Parse(live ? "e8eaee" : "6c7482"), IsAntialias = true };
        font.GetFontMetrics(out SKFontMetrics metrics);
        canvas.DrawText(sign, box.MidX - (font.MeasureText(sign) / 2f), box.MidY - ((metrics.Ascent + metrics.Descent) / 2f), font, ink);
    }

    /// <summary>The type line as the rules write it, with a token announced at the front.</summary>
    public static string TypeLine(ObjectView card)
    {
        string front = string.Join(' ', card.Supertypes.Select(s => s.ToString()).Concat(card.CardTypes.Select(t => t.ToString())));

        if (card.Kind == ObjectKindView.Token)
        {
            front = front.Length > 0 ? $"Token {front}" : "Token";
        }

        return card.Subtypes.Count == 0 ? front : $"{front} - {string.Join(' ', card.Subtypes)}";
    }

    /// <summary>
    /// The card's rules text line by line, then the abilities effects have given it, each with whether it is spent, then -
    /// for a modal spell or ability on the stack - the modes it was put there with (CR 700.2).
    /// </summary>
    public static List<(string Text, bool Spent)> RulesLines(ObjectView card)
    {
        List<(string Text, bool Spent)> lines = [];
        if (card.OracleText.Length > 0)
        {
            string[] printed = card.OracleText.Split('\n');
            for (int i = 0; i < printed.Length; i++)
            {
                lines.Add((printed[i], card.SpentLines.Contains(i)));
            }
        }

        lines.AddRange(card.GrantedAbilities.Select(granted => (granted.Text, granted.Spent)));
        lines.AddRange(card.ChosenModes.Select(mode => ($"Chosen: {mode}", false)));
        return lines;
    }

    // --- The face's regions ------------------------------------------------------------------------------------------

    /// <summary>The retro template's boxes and sizes at one card width, rounded.</summary>
    private sealed class Layout
    {
        /// <param name="saga">The Saga layout a Saga's face is drawn with (<see cref="SagaOf"/>), whose boxes take the place of the card's.</param>
        public Layout(float width, float height, CardTemplate.PanelLayout? saga = null)
            : this(width, height, width, null, saga, null)
        {
        }

        /// <summary>The layout <paramref name="card"/>'s face is drawn with: a Saga's or a Class's, a token's own, or a card's.</summary>
        public static Layout Of(ObjectView card, float width, float height) =>
            TokenOf(card) is { } token ? new Layout(width, height, width, null, null, token) : new Layout(width, height, PanelOf(card));

        /// <summary>
        /// One of <paramref name="count"/> halves of a split card <paramref name="width"/> by <paramref name="height"/>,
        /// standing as tall as the card is wide: laid out at the card's own scale, so its type is the size a whole card's
        /// is, in the template with the split's cuts taken out (<see cref="CardTemplate.SplitLayout"/>).
        /// </summary>
        public static Layout HalfOf(float width, float height, int count)
        {
            CardTemplate.SplitLayout split = CardAssets.Template.SplitHalf!;
            float overlap = split.Overlap * (width / CardAssets.Template.Width);
            return new Layout((height + (overlap * (count - 1))) / count, width, width, split, null, null);
        }

        private Layout(float width, float height, float cardWidth, CardTemplate.SplitLayout? half, CardTemplate.PanelLayout? saga, CardTemplate.TokenLayout? token)
        {
            CardTemplate t = CardAssets.Template;
            float scale = cardWidth / t.Width;
            Width = width;
            Height = height;
            CardWidth = cardWidth;
            Scale = scale;
            Cut = half is not null ? FrameCut.SplitHalf : token is not null ? FrameCut.Token : FrameCut.Whole;

            float Down(float y) => half?.MapY(y) ?? token?.MapY(y) ?? y;

            SKRect Rect(CardTemplate.Box b)
            {
                float left = half?.MapX(b.X) ?? b.X;
                float top = Down(b.Y);
                float right = half?.MapX(b.X + b.Width) ?? (b.X + b.Width);
                float bottom = Down(b.Y + b.Height);
                // Each edge rounded where it falls, so a box ends where the frame's opening it fills does.
                return new SKRect(
                    MathF.Round(left * scale), MathF.Round(top * scale), MathF.Round(right * scale), MathF.Round(bottom * scale));
            }

            int FontSize(float source) => Math.Max(5, (int)MathF.Round(source * scale));

            Title = Rect(t.Title);
            Art = Rect(saga?.Art ?? t.Art);
            Type = Rect(saga?.TypeLine ?? t.TypeLine);
            Rules = Rect(saga?.Rules ?? t.Rules);
            Body = Rect(t.Body);
            Status = Rect(saga?.Status ?? t.Status);
            Artist = Rect(saga?.Artist ?? t.Artist);
            Panel = saga;

            float clip = MathF.Max(1f, MathF.Round(t.TextClipInset * scale));
            TextClip = SKRect.Create(clip, clip, width - (2 * clip), height - (2 * clip));

            TextInset = MathF.Max(1f, t.TextInset * scale);
            RulesInset = MathF.Max(1f, t.RulesInset * scale);
            LoyaltyCentreX = t.LoyaltyCentreX * scale;
            LoyaltyWidth = MathF.Max(4f, t.LoyaltyWidth * scale);
            TightRulesInset = MathF.Max(1f, MathF.Min(t.RulesInset, t.TightRulesInset > 0f ? t.TightRulesInset : t.RulesInset) * scale);
            RulesCentred = t.TextStyle.RulesVerticalAlignment.Equals("center", StringComparison.OrdinalIgnoreCase);
            RulesAlignment = t.RulesAlignment;
            BodyAlignment = t.BodyAlignment;
            TitleFontSize = FontSize(t.TitleFontSize);
            MinimumTitleFontSize = FontSize(t.MinimumTitleFontSize);
            TypeFontSize = FontSize(t.TypeFontSize);
            MinimumTypeFontSize = FontSize(t.MinimumTypeFontSize);
            RulesFontSize = FontSize(saga?.RulesFontSize ?? t.RulesFontSize);
            MinimumRulesFontSize = FontSize(saga?.MinimumRulesFontSize ?? t.MinimumRulesFontSize);
            BodyFontSize = FontSize(t.BodyFontSize);
            StatusFontSize = FontSize(t.StatusFontSize);
            ArtistFontSize = FontSize(t.ArtistFontSize);
            ManaDiameter = MathF.Max(5f, MathF.Round(t.ManaSymbolDiameter * scale));
            ManaGap = MathF.Max(1f, MathF.Round(t.ManaSymbolGap * scale));
            OuterTextShadowOffset = MathF.Max(1f, t.OuterTextShadowOffset * scale);
            LandSymbolSize = MathF.Max(8f, t.LandSymbolSize * scale);

            if (saga is not null)
            {
                ChapterPadding = saga.ChapterPadding * scale;
                ChapterIconCentreX = saga.ChapterIconCentreX * scale;
                ChapterIconWidth = MathF.Max(3f, saga.ChapterIconWidth * scale);
                ChapterIconGap = saga.ChapterIconGap * scale;
                ChapterNumberFontSize = FontSize(saga.ChapterNumberFontSize);
                SeparatorX = saga.SeparatorX * scale;
                SeparatorWidth = saga.SeparatorWidth * scale;
            }
        }

        /// <summary>The size of what is laid out: the card, or one half of it.</summary>
        public float Width { get; }
        public float Height { get; }

        /// <summary>The width of the card this is laid out on, which the art asked for and the marks are sized by.</summary>
        public float CardWidth { get; }

        /// <summary>Drawn units to the template unit.</summary>
        public float Scale { get; }

        /// <summary>How the frame this is laid out on is cut: whole, to one half of a split card, or to a token's own.</summary>
        public FrameCut Cut { get; }

        public SKRect Title { get; }
        public SKRect Art { get; }
        public SKRect Type { get; }
        public SKRect Rules { get; }
        public SKRect Body { get; }
        public SKRect Status { get; }
        public SKRect Artist { get; }
        public SKRect TextClip { get; }
        public float TextInset { get; }
        public float RulesInset { get; }

        /// <summary>The narrower margin rules text takes when the regular one would cost it a size.</summary>
        public float TightRulesInset { get; }

        /// <summary>Where a loyalty ability's shield is centred across the card, on the rules box's edge, and its width.</summary>
        public float LoyaltyCentreX { get; }
        public float LoyaltyWidth { get; }
        public bool RulesCentred { get; }
        public string RulesAlignment { get; }
        public string BodyAlignment { get; }
        public int TitleFontSize { get; }
        public int MinimumTitleFontSize { get; }
        public int TypeFontSize { get; }
        public int MinimumTypeFontSize { get; }
        public int RulesFontSize { get; }
        public int MinimumRulesFontSize { get; }
        public int BodyFontSize { get; }
        public int StatusFontSize { get; }
        public int ArtistFontSize { get; }
        public float ManaDiameter { get; }
        public float ManaGap { get; }
        public float OuterTextShadowOffset { get; }
        public float LandSymbolSize { get; }

        /// <summary>A Saga's layout, which this is laid out with, or null for any other card.</summary>
        public CardTemplate.PanelLayout? Panel { get; }
        public float ChapterPadding { get; }
        public float ChapterIconCentreX { get; }
        public float ChapterIconWidth { get; }
        public float ChapterIconGap { get; }
        public int ChapterNumberFontSize { get; }
        public float SeparatorX { get; }
        public float SeparatorWidth { get; }
    }

    /// <summary>
    /// The widest illustration the art service holds, width over height. The art box is narrower (770 by 622), so a wide
    /// picture is cropped at its sides, and its longest side - the size the service is asked for - is wider than the box
    /// by as much as this over the box's own shape.
    /// </summary>
    private const float WidestArt = 1.45f;

    /// <summary>
    /// The art size to ask for to fill the art box of a card <paramref name="width"/> wide drawn at <paramref name="scale"/>
    /// pixels to the unit: the box's width in pixels, widened for the crop of the widest picture. Everything that asks
    /// for art on a face's behalf asks this, so a face drawn anywhere wants the same art.
    /// </summary>
    public static int ArtPixels(float width, float scale) => ArtPixels(null, width, scale);

    /// <summary>
    /// The art size to ask for to fill <paramref name="card"/>'s art box: a Saga's box is tall, and the art made for it
    /// tall too, so its longest side is the box's height; a token's own is taller than a card's, so more of a wide
    /// picture is cropped; any other card's is <see cref="ArtPixels(float, float)"/>.
    /// </summary>
    public static int ArtPixels(ObjectView? card, float width, float scale)
    {
        CardTemplate t = CardAssets.Template;
        if (card is not null && PanelOf(card) is { } saga)
        {
            return (int)MathF.Ceiling(width * saga.Art.Height / t.Width * scale);
        }

        float height = card is not null && TokenOf(card) is { } token
            ? token.MapY(t.Art.Y + t.Art.Height) - token.MapY(t.Art.Y)
            : t.Art.Height;
        float box = width * t.Art.Width / t.Width;
        float crop = MathF.Max(1f, WidestArt / (t.Art.Width / height));
        return (int)MathF.Ceiling(box * scale * crop);
    }

    private static void DrawArt(SKCanvas canvas, ObjectView card, Layout layout, SKColor frame, bool dimmed, float pixelScale)
    {
        // A half of a split card asks for its art at a whole card's size, as everything asking on a face's behalf does.
        float width = layout.CardWidth;
        SKRect art = layout.Art;

        // The illustration runs a pixel under the frame all round, which is drawn over it, so no seam of whatever is
        // beneath the card opens between them where the frame's opening and the box round to different pixels.
        SKRect bed = SKRect.Inflate(art, 1f, 1f);
        Fill(canvas, bed, Lerp(frame, InkColor, 0.35f));

        // A double-faced card's other face and a preparation card's prepare spell are printed on it and shown beside it
        // on a close look: their art is asked for with this face's, so that look does not open on them without.
        foreach (ObjectView other in card.AlsoPrinted())
        {
            _ = CardArt.Fetch(other.Name, ArtPixels(other, width, pixelScale));
        }

        if (CardArt.Get(card.Name, ArtPixels(card, width, pixelScale)) is { } bitmap)
        {
            // A few legacy assets are scans of a whole card: recover the printed art aperture first. Only for a wide art
            // box, since a tall one - a Saga's - is made for tall art.
            SKRect available = bitmap.Width < bitmap.Height && bed.Width >= bed.Height
                ? SKRect.Create(bitmap.Width * 0.14f, bitmap.Height * 0.13f, bitmap.Width * 0.72f, bitmap.Height * 0.42f)
                : SKRect.Create(bitmap.Width, bitmap.Height);

            float targetRatio = bed.Width / bed.Height;
            SKRect source = available.Width / available.Height > targetRatio
                ? SKRect.Create(available.MidX - (available.Height * targetRatio / 2f), available.Top, available.Height * targetRatio, available.Height)
                : SKRect.Create(available.Left, available.MidY - (available.Width / targetRatio / 2f), available.Width, available.Width / targetRatio);

            SKImage image = CardArt.ImageOf(bitmap);
            using var paint = new SKPaint { IsAntialias = true };
            if (dimmed)
            {
                paint.ColorFilter = SKColorFilter.CreateBlendMode(new SKColor(143, 143, 143), SKBlendMode.Modulate);
            }

            canvas.DrawImage(image, source, bed, new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.Linear), paint);
        }
        else
        {
            // A little depth without pretending missing art is an illustration.
            float inset = MathF.Max(2f, width * 0.025f);
            Stroke(canvas, SKRect.Inflate(art, -inset, -inset), Lerp(frame, Parchment, 0.12f), 1f);
        }

        Stroke(canvas, art, Lerp(frame, InkColor, 0.72f), MathF.Max(1f, width / StandardWidth));
    }

    private static float DrawCost(SKCanvas canvas, ObjectView card, Layout layout, bool shadow)
    {
        List<string> symbols = ManaSymbols.Symbols(card.ManaCost);
        if (symbols.Count == 0)
        {
            return 0f;
        }

        float centreY = layout.Title.MidY;
        float right = layout.Title.Right - layout.TextInset;
        float originalRight = right;

        for (int i = symbols.Count - 1; i >= 0; i--)
        {
            ManaSymbols.Draw(canvas, new SKPoint(right - (layout.ManaDiameter / 2f), centreY), layout.ManaDiameter, symbols[i], shadow);
            right -= layout.ManaDiameter + layout.ManaGap;
        }

        return originalRight - right - layout.ManaGap;
    }

    /// <summary>
    /// Rules text in whatever lines fit between the type line and the bottom row, symbols drawn as pips in the run of the
    /// text: shrunk a size at a time to fit, and past the smallest size the lines that do not fit are dropped.
    /// </summary>
    private static void DrawOracle(SKCanvas canvas, ObjectView card, SKColor text, Layout layout)
    {
        SKRect box = layout.Rules;

        // Printed without its reminder text, which the tooltip still gives: a line that was all reminder goes. A
        // planeswalker's loyalty ability is its cost on a shield out on the box's edge, and its text (CR 606.3).
        bool walker = card.CardTypes.Contains(CardTypeView.Planeswalker);
        List<(string Text, string? Loyalty, bool Spent)> rules = [.. RulesLines(card)
            .Select(rule => (Text: WithoutReminder(rule.Text), rule.Spent))
            .Where(rule => rule.Text.Length > 0)
            .Select(rule => walker && LoyaltyCost().Match(rule.Text) is { Success: true } cost
                ? (cost.Groups[2].Value, (string?)cost.Groups[1].Value, rule.Spent)
                : (rule.Text, null, rule.Spent))];
        if (rules.Count == 0)
        {
            return;
        }

        // The box's margin: the regular one, or the tight one where that keeps the text a size larger (CR has nothing
        // to say; printed cards give a crowded text box more of the box the same way).
        float top = 0f, available = 0f, left = 0f, width = 0f;
        void Margin(float inset)
        {
            top = box.Top + inset;
            available = MathF.Min(box.Bottom - inset, layout.TextClip.Bottom) - top;
            left = box.Left + inset;
            width = MathF.Min(box.Right - inset, layout.TextClip.Right) - left;
        }

        Margin(layout.TightRulesInset);
        if (width <= 0f || available < LineHeight(CardAssets.Body, layout.MinimumRulesFontSize))
        {
            return;
        }

        int size = layout.RulesFontSize;
        List<(List<List<Run>> Lines, string? Loyalty, bool Spent)> paragraphs = [];
        float lineHeight = 0f, gap = 0f, indent = 0f;

        // Abilities stand a little further apart than lines do, so where one ends and the next begins reads at a glance.
        // A loyalty ability stands at least as tall as its shield, so shields one under another never meet.
        float shieldHeight = layout.LoyaltyWidth * LoyaltyShieldAspect;
        float spread = 0f;
        float Block((List<List<Run>> Lines, string? Loyalty, bool Spent) paragraph) => spread + (paragraph.Loyalty is null
            ? paragraph.Lines.Count * lineHeight
            : MathF.Max(paragraph.Lines.Count * lineHeight, shieldHeight));
        float Height() => paragraphs.Sum(Block) + ((paragraphs.Count - 1) * gap);

        bool fitted = false;
        while (!fitted)
        {
            float diameter = MathF.Round(size * 1.05f);
            lineHeight = LineHeight(CardAssets.Body, size);
            gap = MathF.Round(lineHeight * CardAssets.Template.ParagraphGap);
            foreach (float inset in (float[])[layout.RulesInset, layout.TightRulesInset])
            {
                Margin(inset);

                // A shield stands on the box's edge, half out of it, and a loyalty ability's text starts clear of it.
                indent = walker ? MathF.Max(0f, layout.LoyaltyCentreX + (layout.LoyaltyWidth / 2f) + MathF.Round(size * 0.3f) - left) : 0f;
                float shieldIndent = indent;
                float wrapWidth = width;
                paragraphs = [.. rules.Select(rule => (
                    Wrap(rule.Text, size, diameter, rule.Loyalty is null ? wrapWidth : wrapWidth - shieldIndent), rule.Loyalty, rule.Spent))];
                if (Height() <= available)
                {
                    fitted = true;
                    break;
                }
            }

            if (!fitted && size <= layout.MinimumRulesFontSize)
            {
                break;
            }

            if (!fitted)
            {
                size--;
            }
        }

        // What fits, a whole line at a time, and always the first line.
        // A planeswalker's abilities share out the room left, as a printed one's stand evenly down the box.
        if (walker && paragraphs.Count > 0)
        {
            spread = MathF.Max(0f, available - Height()) / paragraphs.Count;
        }

        // Each ability's block, its lines centred in it, and of those what fits a whole line at a time - always the first.
        List<(int Paragraph, int Line, float Top)> shown = [];
        List<(float Top, float Height)> blocks = [];
        float y = 0f;
        for (int p = 0; p < paragraphs.Count; p++)
        {
            if (p > 0)
            {
                y += gap;
            }

            float block = Block(paragraphs[p]);
            float lineTop = y + ((block - (paragraphs[p].Lines.Count * lineHeight)) / 2f);
            blocks.Add((y, block));
            for (int l = 0; l < paragraphs[p].Lines.Count; l++)
            {
                if (shown.Count > 0 && lineTop + lineHeight > available + 0.5f)
                {
                    p = paragraphs.Count;
                    break;
                }

                shown.Add((p, l, lineTop));
                lineTop += lineHeight;
            }

            y += block;
        }

        float shownHeight = shown.Count == 0 ? 0f : shown[^1].Top + lineHeight;
        float originY = layout.RulesCentred ? box.MidY - (shownHeight / 2f) : top;

        SKColor ink = Lerp(text, Grey, 0.25f);
        SKColor spentInk = Lerp(text, Grey, 0.75f).WithAlpha((byte)(text.Alpha * 0.6f));
        float pip = MathF.Round(size * 1.05f);

        using var font = new SKFont(CardAssets.Body, size);
        using var paint = new SKPaint { Color = ink, IsAntialias = true };
        using var spentPaint = new SKPaint { Color = spentInk, IsAntialias = true };

        if (walker)
        {
            // Across the abilities' text, clear of the shields: a groove cut into the box, a shadow with the light
            // catching its lower lip, both fading out at the ends so it sinks into the text box.
            float grooveLeft = left + indent;
            float grooveWidth = width - indent;
            float stroke = Math.Max(1f, size * 0.09f);
            using var shade = new SKPaint { IsAntialias = true, Style = SKPaintStyle.Fill };
            using var lip = new SKPaint { IsAntialias = true, Style = SKPaintStyle.Fill };
            shade.Shader = SKShader.CreateLinearGradient(new SKPoint(grooveLeft, 0), new SKPoint(grooveLeft + grooveWidth, 0),
                [SKColors.Black.WithAlpha(0), SKColors.Black.WithAlpha(110), SKColors.Black.WithAlpha(110), SKColors.Black.WithAlpha(0)],
                [0f, 0.15f, 0.85f, 1f], SKShaderTileMode.Clamp);
            lip.Shader = SKShader.CreateLinearGradient(new SKPoint(grooveLeft, 0), new SKPoint(grooveLeft + grooveWidth, 0),
                [SKColors.White.WithAlpha(0), SKColors.White.WithAlpha(140), SKColors.White.WithAlpha(140), SKColors.White.WithAlpha(0)],
                [0f, 0.15f, 0.85f, 1f], SKShaderTileMode.Clamp);
            foreach ((int p, _, _) in shown.Where(each => each.Paragraph > 0 && each.Line == 0))
            {
                float middle = originY + blocks[p].Top - (gap / 2f);
                canvas.DrawRect(grooveLeft, middle - stroke, grooveWidth, stroke, shade);
                canvas.DrawRect(grooveLeft, middle, grooveWidth, stroke * 0.8f, lip);
            }
        }

        foreach ((int p, int l, float lineTop) in shown)
        {
            (List<List<Run>> lines, string? loyalty, bool spent) = paragraphs[p];
            List<Run> line = lines[l];
            float textLeft = loyalty is null ? left : left + indent;
            float textWidth = loyalty is null ? width : width - indent;

            if (loyalty is not null && l == 0)
            {
                // The shield stands against the ability's lines that are shown, centred on them. It stays inked when the
                // ability is spent: the text greys, and the cost is still what it costs.
                int shownLines = shown.Count(each => each.Paragraph == p);
                float centreY = shownLines == lines.Count
                    ? originY + blocks[p].Top + (blocks[p].Height / 2f)
                    : originY + lineTop + (shownLines * lineHeight / 2f);
                // Nudged towards its point, so the shield's square body - what the eye takes for it - is what centres.
                float nudge = loyalty.StartsWith('+') ? -layout.LoyaltyWidth * LoyaltyNudge
                    : loyalty.StartsWith('−') || loyalty.StartsWith('-') ? layout.LoyaltyWidth * LoyaltyNudge
                    : 0f;
                DrawLoyalty(canvas, new SKPoint(layout.LoyaltyCentreX, centreY + nudge), layout.LoyaltyWidth, loyalty, InkColor);
            }

            float lineWidth = line.Sum(run => run.Width);
            float x = layout.RulesAlignment.Equals("center", StringComparison.OrdinalIgnoreCase) ? textLeft + ((textWidth - lineWidth) / 2f)
                : layout.RulesAlignment.Equals("right", StringComparison.OrdinalIgnoreCase) ? textLeft + textWidth - lineWidth
                : textLeft;

            DrawRuns(canvas, line, x, originY + lineTop, lineHeight, font, spent ? spentPaint : paint, pip);
        }
    }

    /// <summary>How far a loyalty cost's shield moves towards its point, in its widths.</summary>
    private const float LoyaltyNudge = 0.06f;

    /// <summary>How much of its room a loyalty shield fills, leaving its border and shadow space.</summary>
    private const float LoyaltyShrink = 0.9f;

    /// <summary>How tall a loyalty cost's shield's ink stands to its width.</summary>
    private const float LoyaltyShieldAspect = 0.68f;

    /// <summary>
    /// A planeswalker's loyalty ability line (CR 606.3): its cost - a plus or minus sign and a number, or a bare 0 - and
    /// what it does.
    /// </summary>
    [System.Text.RegularExpressions.GeneratedRegex(@"^\s*([+\u2212-]?(?:\d+|X)):\s*(.*)$")]
    private static partial System.Text.RegularExpressions.Regex LoyaltyCost();

    /// <summary>
    /// A loyalty cost on its shield, as the mana font draws them: pointing up for a cost that adds loyalty, down for one
    /// that removes it, level for nothing, with the cost written across it in white.
    /// </summary>
    public static void DrawLoyalty(SKCanvas canvas, SKPoint centre, float width, string cost, SKColor ink, SKColor? figure = null)
    {
        width *= LoyaltyShrink;
        bool up = cost.StartsWith('+');
        bool down = cost.StartsWith('\u2212') || cost.StartsWith('-');
        string glyph = up ? ManaSymbols.LoyaltyUp : down ? ManaSymbols.LoyaltyDown : ManaSymbols.LoyaltyZero;
        ManaSymbols.DrawGlyphBordered(canvas, centre, glyph, (int)MathF.Round(width), ink);

        // A shield's point is not where its number sits: the number keeps to the square body of it, away from the point.
        // The title face has no minus sign, so a minus is written as its en dash.
        string written = down ? "\u2013" + cost[1..] : cost;
        int size = Math.Max(4, (int)MathF.Round(width * 0.36f));
        float shift = up ? width * 0.08f : down ? -width * 0.08f : 0f;
        DrawString(canvas, CardAssets.Title, written, centre.X - (MeasureWidth(CardAssets.Title, written, size) / 2f),
            centre.Y + shift + (CapHeight(CardAssets.Title, size) / 2f), size, figure ?? SKColors.White);
    }

    /// <summary>The loyalty a planeswalker has, on the shield its loyalty starts on (the mana font's ms-loyalty-start).</summary>
    public static void DrawLoyaltyStart(SKCanvas canvas, SKPoint centre, float width, string loyalty, SKColor ink, SKColor figure)
    {
        width *= LoyaltyShrink;
        ManaSymbols.DrawGlyphBordered(canvas, centre, ManaSymbols.LoyaltyStart, (int)MathF.Round(width), ink);
        int size = Math.Max(4, (int)MathF.Round(width * 0.42f));
        DrawString(canvas, CardAssets.Title, loyalty, centre.X - (MeasureWidth(CardAssets.Title, loyalty, size) / 2f),
            centre.Y + (CapHeight(CardAssets.Title, size) / 2f), size, figure);
    }

    /// <summary>
    /// A planeswalker's rules line as a loyalty ability (CR 606.3): its cost and what it does, or false for any other
    /// line.
    /// </summary>
    public static bool TryLoyaltyAbility(string rule, out string cost, out string text)
    {
        if (LoyaltyCost().Match(rule) is { Success: true } match)
        {
            cost = match.Groups[1].Value;
            text = match.Groups[2].Value;
            return true;
        }

        cost = text = string.Empty;
        return false;
    }

    private static float MeasureWidth(SKTypeface typeface, string value, int size)
    {
        using var font = new SKFont(typeface, size);
        return font.MeasureText(value);
    }

    /// <summary>One wrapped line of rules from its top left: its words in <paramref name="font"/>, its symbols as pips.</summary>
    private static void DrawRuns(SKCanvas canvas, List<Run> line, float x, float top, float lineHeight, SKFont font, SKPaint paint, float pip)
    {
        float ascent = -font.Metrics.Ascent;
        foreach (Run run in line)
        {
            if (run.IsSymbol)
            {
                ManaSymbols.Draw(canvas, new SKPoint(x + (run.Width / 2f), top + (lineHeight / 2f)), pip, run.Text, shadow: false);
            }
            else
            {
                canvas.DrawText(run.Text, x, top + ascent, SKTextAlign.Left, font, paint);
            }

            x += run.Width;
        }
    }

    /// <summary>
    /// One section of a panel: a Saga's chapter or a Class's level - the numbers its badges carry (a chapter's; a level
    /// is marked by its header alone), the line it is headed with (a level's cost to gain it), its abilities, and whether
    /// it is drawn spent.
    /// </summary>
    private sealed record Section(string[] Numbers, string? Header, List<string> Texts, bool Spent);

    /// <summary>
    /// A Saga's or a Class's rules as their panel lays them out. The reminder first, when the first line is all reminder;
    /// then a Saga's chapters (CR 714.2b), each its numbers and text, or a Class's levels (CR 716.2a), each headed by the
    /// cost to gain it and holding the abilities after it, the first level before any. Any other line is a section of its
    /// own. A Class's levels above the one it has are drawn spent, as abilities it does not yet have.
    /// </summary>
    private static (string? Reminder, List<Section> Sections) PanelSections(ObjectView card)
    {
        bool levels = card.Subtypes.Contains("Class");

        // On the battlefield a Class with no level is at its first (CR 716.2d); anywhere else no level is spent.
        int reached = card.Zone == ZoneKindView.Battlefield ? Math.Max(1, card.ClassLevel) : int.MaxValue;
        string? reminder = null;
        List<Section> sections = [];
        List<(string Text, bool Spent)> rules = RulesLines(card);

        for (int i = 0; i < rules.Count; i++)
        {
            (string rule, bool spent) = rules[i];
            if (i == 0 && card.OracleText.Length > 0 && rule.StartsWith('(') && WithoutReminder(rule).Length == 0)
            {
                reminder = rule;
            }
            else if (levels && LevelLine().Match(rule) is { Success: true } level)
            {
                int number = int.Parse(level.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture);
                sections.Add(new Section([], rule.Trim(), [], number > reached));
            }
            else if (!levels && ChapterLine().Match(rule) is { Success: true } chapter)
            {
                string[] numbers = chapter.Groups[1].Value.Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
                sections.Add(new Section(numbers, null, [WithoutReminder(chapter.Groups[2].Value)], spent));
            }
            else if (WithoutReminder(rule) is { Length: > 0 } text)
            {
                if (levels && sections.Count == 0)
                {
                    sections.Add(new Section([], null, [], false));
                }

                if (levels && i < card.OracleText.Split('\n').Length)
                {
                    // A level's abilities go with it, spent with it when the Class has not reached it.
                    Section last = sections[^1];
                    last.Texts.Add(text);
                    sections[^1] = last with { Spent = last.Spent || spent };
                }
                else
                {
                    sections.Add(new Section([], null, [text], spent));
                }
            }
        }

        return (reminder, sections);
    }

    /// <summary>
    /// A Saga's or a Class's rules down its panel: the reminder at the top in italics - the one reminder text a face
    /// prints, since the chapters and levels do not read without it - then each chapter beside its badges on the ribbon
    /// and each level headed by its cost and its "Level N", with a line under the reminder and between each section. The sections share out what
    /// room their text leaves, each centred in its share, shrunk a size at a time to fit; past the smallest size what does
    /// not fit is cut off at the foot of the panel.
    /// </summary>
    private static void DrawChapters(SKCanvas canvas, ObjectView card, SKColor text, Layout layout, bool dimmed)
    {
        CardTemplate.PanelLayout panel = layout.Panel!;
        SKRect box = layout.Rules;
        float bottom = MathF.Min(box.Bottom, layout.TextClip.Bottom);
        float available = bottom - box.Top;
        float width = MathF.Min(box.Right, layout.TextClip.Right) - box.Left;
        if (width <= 0f || available < LineHeight(CardAssets.Body, layout.MinimumRulesFontSize))
        {
            return;
        }

        (string? reminder, List<Section> sections) = PanelSections(card);

        float badgeWidth = layout.ChapterIconWidth;
        float badgeHeight = badgeWidth * CardAssets.IconAspect(panel.ChapterIcon);
        float Badges(Section section) => section.Numbers.Length == 0 ? 0f
            : (section.Numbers.Length * badgeHeight) + ((section.Numbers.Length - 1) * layout.ChapterIconGap);

        float padding = layout.ChapterPadding;
        int size = layout.RulesFontSize;
        List<List<Run>> reminderLines;
        List<(List<List<Run>> Header, List<List<Run>> Body)> sectionLines;
        float lineHeight;

        float Share(int i) => MathF.Max(
            ((sectionLines[i].Header.Count + sectionLines[i].Body.Count) * lineHeight) + (padding * 2f), Badges(sections[i]));

        while (true)
        {
            float diameter = MathF.Round(size * 1.05f);
            lineHeight = LineHeight(CardAssets.Body, size);
            reminderLines = reminder is null ? [] : Wrap(reminder, size, diameter, width);
            sectionLines = [.. sections.Select(section => (
                section.Header is null ? [] : Wrap(section.Header, size, diameter, width, CardAssets.Title),
                section.Texts.SelectMany(each => Wrap(each, size, diameter, width)).ToList()))];

            float needed = (reminderLines.Count * lineHeight) + (reminder is null ? 0f : padding)
                + Enumerable.Range(0, sections.Count).Sum(Share);
            if (needed <= available || size <= layout.MinimumRulesFontSize)
            {
                break;
            }

            size--;
        }

        float used = (reminderLines.Count * lineHeight) + (reminder is null ? 0f : padding) + Enumerable.Range(0, sections.Count).Sum(Share);
        float spare = sections.Count == 0 ? 0f : MathF.Max(0f, available - used) / sections.Count;

        SKColor ink = Lerp(text, Grey, 0.25f);
        SKColor spentInk = Lerp(text, Grey, 0.75f).WithAlpha((byte)(text.Alpha * 0.6f));
        float pip = MathF.Round(size * 1.05f);
        using var font = new SKFont(CardAssets.Body, size);
        using var headerFont = new SKFont(CardAssets.Title, size);
        using var paint = new SKPaint { Color = ink, IsAntialias = true };
        using var spentPaint = new SKPaint { Color = spentInk, IsAntialias = true };
        using var iconPaint = new SKPaint { IsAntialias = true };
        if (dimmed)
        {
            iconPaint.ColorFilter = SKColorFilter.CreateBlendMode(new SKColor(158, 158, 158), SKBlendMode.Modulate);
        }

        float separatorHeight = layout.SeparatorWidth * CardAssets.IconAspect(panel.Separator);
        void Separator(float y) => CardAssets.DrawIcon(
            canvas, panel.Separator, SKRect.Create(layout.SeparatorX, y - (separatorHeight / 2f), layout.SeparatorWidth, separatorHeight), iconPaint);

        canvas.Save();
        canvas.ClipRect(new SKRect(0f, box.Top, layout.Width, bottom));
        float top = box.Top;

        if (reminder is not null)
        {
            using var italic = new SKFont(CardAssets.Body, size) { SkewX = panel.ReminderSkew };
            foreach (List<Run> line in reminderLines)
            {
                DrawRuns(canvas, line, box.Left, top, lineHeight, italic, paint, pip);
                top += lineHeight;
            }

            top += padding;
            if (sections.Count > 0)
            {
                Separator(top);
            }
        }

        SKColor numberInk = SKColor.Parse(panel.ChapterNumberColor);
        int numberSize = layout.ChapterNumberFontSize;
        float capHeight = CapHeight(CardAssets.Title, numberSize);
        for (int i = 0; i < sections.Count; i++)
        {
            Section section = sections[i];
            (List<List<Run>> header, List<List<Run>> body) = sectionLines[i];
            float height = Share(i) + spare;
            SKPaint linePaint = section.Spent ? spentPaint : paint;

            float badgeTop = top + ((height - Badges(section)) / 2f);
            foreach (string number in section.Numbers)
            {
                var badge = SKRect.Create(layout.ChapterIconCentreX - (badgeWidth / 2f), badgeTop, badgeWidth, badgeHeight);
                CardAssets.DrawIcon(canvas, panel.ChapterIcon, badge, iconPaint);
                DrawString(canvas, CardAssets.Title, number, Aligned(CardAssets.Title, number, numberSize, badge, "center"), badge.MidY + (capHeight / 2f), numberSize, numberInk);
                badgeTop += badgeHeight + layout.ChapterIconGap;
            }

            float lineTop = top + ((height - ((header.Count + body.Count) * lineHeight)) / 2f);
            foreach (List<Run> line in header)
            {
                DrawRuns(canvas, line, box.Left, lineTop, lineHeight, headerFont, linePaint, pip);
                lineTop += lineHeight;
            }

            foreach (List<Run> line in body)
            {
                DrawRuns(canvas, line, box.Left, lineTop, lineHeight, font, linePaint, pip);
                lineTop += lineHeight;
            }

            top += height;
            if (i < sections.Count - 1)
            {
                Separator(top);
            }
        }

        canvas.Restore();
    }

    /// <summary>A Saga's chapter line (CR 714.2b): its chapter numbers, a dash, and what the chapter does.</summary>
    [System.Text.RegularExpressions.GeneratedRegex(@"^\s*([IVX]+(?:\s*,\s*[IVX]+)*)\s*[\u2014\u2013-]\s*(.*)$")]
    private static partial System.Text.RegularExpressions.Regex ChapterLine();

    /// <summary>A Class's level line (CR 716.2a): the cost to gain a level, and the level it is.</summary>
    [System.Text.RegularExpressions.GeneratedRegex(@"^.*:\s*Level\s+(\d+)\s*$")]
    private static partial System.Text.RegularExpressions.Regex LevelLine();

    /// <summary>
    /// A rules line with its reminder text - what is in parentheses - taken out (CR 207.2a), and the spaces it leaves
    /// closed up.
    /// </summary>
    public static string WithoutReminder(string rule) =>
        rule.Contains('(', StringComparison.Ordinal)
            ? Spaces().Replace(Reminder().Replace(rule, string.Empty), " ").Trim()
            : rule;

    [System.Text.RegularExpressions.GeneratedRegex(@"\([^()]*\)")]
    private static partial System.Text.RegularExpressions.Regex Reminder();

    [System.Text.RegularExpressions.GeneratedRegex(@" {2,}")]
    private static partial System.Text.RegularExpressions.Regex Spaces();

    /// <summary>One piece of a rules line: a word with the space after it, or a symbol with the room one takes.</summary>
    private readonly record struct Run(string Text, bool IsSymbol, float Width);

    /// <summary>Greedy word wrap with symbols as fixed-width pieces, for a line of rules.</summary>
    private static List<List<Run>> Wrap(string rule, int size, float diameter, float width, SKTypeface? typeface = null)
    {
        using var font = new SKFont(typeface ?? CardAssets.Body, size);
        List<Run> pieces = [];

        foreach ((string part, bool isSymbol) in ManaSymbols.Parse(rule))
        {
            if (isSymbol)
            {
                pieces.Add(new Run(part, true, diameter + 1));
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
                        string word = part[start..end];
                        pieces.Add(new Run(word, false, font.MeasureText(word)));
                    }

                    start = end;
                }
            }
        }

        List<List<Run>> lines = [[]];
        float used = 0f;

        foreach (Run piece in pieces)
        {
            float trimmed = piece.IsSymbol ? piece.Width : font.MeasureText(piece.Text.TrimEnd());
            if (used + trimmed > width && lines[^1].Count > 0)
            {
                lines.Add([]);
                used = 0f;
            }

            lines[^1].Add(piece);
            used += piece.Width;
        }

        return lines;
    }

    /// <summary>The single large mana mark an old-frame basic land carries instead of rules text.</summary>
    private static void DrawLandSymbol(SKCanvas canvas, ObjectView card, Layout layout)
    {
        if (card.OracleText.Length > 0 || card.GrantedAbilities.Count > 0 || !card.Supertypes.Contains(SupertypeView.Basic))
        {
            return;
        }

        HashSet<ColorView> colors = ProducedLandColors(card);
        if (colors.Count != 1)
        {
            return;
        }

        (string symbol, SKColor ink) = colors.Single() switch
        {
            ColorView.White => ("W", SKColor.Parse("d7cba5")),
            ColorView.Blue => ("U", SKColor.Parse("087fbd")),
            ColorView.Black => ("B", SKColor.Parse("343039")),
            ColorView.Red => ("R", SKColor.Parse("c44734")),
            ColorView.Green => ("G", SKColor.Parse("267447")),
            _ => (string.Empty, InkColor),
        };

        if (symbol.Length > 0)
        {
            ManaSymbols.DrawMark(canvas, new SKPoint(layout.Rules.MidX, layout.Rules.MidY), layout.LandSymbolSize, symbol, ink);
        }
    }

    private static void DrawStatus(SKCanvas canvas, ObjectView card, SKColor text, Layout layout)
    {
        List<string> parts = StatusWords(card);
        if (parts.Count == 0)
        {
            return;
        }

        SKRect box = layout.Status;
        int size = layout.StatusFontSize;
        string line = Fit(CardAssets.Title, string.Join(' ', parts), size, MathF.Min(box.Right, layout.TextClip.Right) - box.Left);
        DrawString(canvas, CardAssets.Title, line, box.Left + layout.TextInset, CapBaseline(CardAssets.Title, box, size), size, Lerp(text, SKColor.Parse("e2b04c"), 0.55f));
    }

    /// <summary>
    /// The words of game state written at the foot of the card. Tapped is not listed: the card is turned sideways, which
    /// says it more plainly than a word would. Nor is a creature just arrived: its mark is over its art, drawn by the row
    /// it is on (<see cref="DrawArrival"/>), so the face is the same before and after.
    /// </summary>
    private static List<string> StatusWords(ObjectView card)
    {
        List<string> parts = [];
        if (card.Monstrous)
        {
            parts.Add("MONSTROUS");
        }

        if (card.ClassLevel > 0)
        {
            parts.Add($"LEVEL {card.ClassLevel}");
        }

        if (card.Harnessed)
        {
            parts.Add("HARNESSED");
        }

        if (card.Saddled)
        {
            parts.Add("SADDLED");
        }

        if (card.Exerted)
        {
            parts.Add("EXERTED");
        }

        if (card.Goaded)
        {
            parts.Add("GOADED");
        }

        return parts;
    }

    /// <summary>
    /// The illustration's credit, bottom left as the old frame prints it: the artist's brush, then the name, in the frame's
    /// ink. It goes with the art, so a copy credits the artist of the art it shows.
    /// </summary>
    private static void DrawArtist(SKCanvas canvas, ObjectView card, Layout layout, SKColor ink, SKColor shadow)
    {
        if (CardArt.Artist(card.Name) is not { Length: > 0 } artist)
        {
            return;
        }

        SKRect box = layout.Artist;
        int size = layout.ArtistFontSize;
        float baseline = CapBaseline(CardAssets.Title, box, size);
        float x = box.Left + layout.TextInset;

        // The brush lies on its side, so it is drawn larger than the capitals beside it to read as one.
        int brushSize = Math.Max(4, (int)MathF.Round(size * 1.6f));
        float brushWidth = ManaSymbols.InkWidth(ManaSymbols.ArtistBrush, brushSize);
        var brushCentre = new SKPoint(x + (brushWidth / 2f), baseline - (CapHeight(CardAssets.Title, size) / 2f));
        if (shadow.Alpha > 0)
        {
            ManaSymbols.DrawGlyph(canvas, brushCentre + new SKPoint(layout.OuterTextShadowOffset, layout.OuterTextShadowOffset), ManaSymbols.ArtistBrush, brushSize, shadow);
        }

        ManaSymbols.DrawGlyph(canvas, brushCentre, ManaSymbols.ArtistBrush, brushSize, ink);

        float nameX = x + brushWidth + MathF.Round(size * 0.3f);
        float room = MathF.Min(box.Right, layout.TextClip.Right) - nameX;
        DrawFrameString(canvas, nameX, baseline, Fit(CardAssets.Title, artist, size, room), size, ink, shadow, layout.OuterTextShadowOffset);
    }

    /// <summary>Whether the card has power and toughness, or loyalty, to show: what <see cref="DrawBodyOver"/> draws.</summary>
    public static bool HasBody(ObjectView card) => (card.Known || card.FaceDown) && Body(card) is not null;

    /// <summary>
    /// Power and toughness, or a planeswalker's loyalty, over a face drawn without them (<see cref="Draw"/> with no body),
    /// exactly where and as <see cref="Draw"/> would have drawn them: in the face's own frame, <paramref name="width"/> by
    /// <paramref name="height"/> being the size the face was drawn at.
    /// </summary>
    public static void DrawBodyOver(SKCanvas canvas, ObjectView card, float width, float height, bool dimmed)
    {
        if (!HasBody(card))
        {
            return;
        }

        (string words, SKColor frame) = BodyLooks.GetValue(card, card => new(Body(card)!, card.FaceDown ? FaceDownFrame : FrameColor(card)));
        if (dimmed && !card.FaceDown)
        {
            frame = Lerp(frame, DimmedFrame, 0.55f);
        }

        // Everything the body shows is its words, its frame's colour and the size it is laid out at, so one drawing of
        // "2/2" serves every card that shows it: laid out afresh, a board of two hundred creatures spent half of every
        // frame setting the same few words.
        (string, SKColor, float, float) key = (words, frame, width, height);
        if (!Bodies.TryGetValue(key, out SKPicture? body))
        {
            if (Bodies.Count >= MostBodies)
            {
                foreach (SKPicture old in Bodies.Values)
                {
                    old.Dispose();
                }

                Bodies.Clear();
            }

            body = Faces.Record(width, height, recording => DrawBody(
                recording, card, frame, Layout.Of(card, width, height), SKColor.Parse(CardAssets.Template.TextStyle.OuterTextColor), ParseAlpha(CardAssets.Template.TextStyle.OuterTextShadowColor)));
            Bodies[key] = body;
        }

        canvas.DrawPicture(body);
    }

    /// <summary>A card's body's words and its frame's colour, worked out once for each object the client is sent.</summary>
    private static readonly System.Runtime.CompilerServices.ConditionalWeakTable<ObjectView, BodyLook> BodyLooks = new();

    private sealed record BodyLook(string Words, SKColor Frame);

    private static readonly SKColor DimmedFrame = SKColor.Parse("3a3a3a");

    /// <summary>The bodies drawn so far, by their words, frame colour and size (<see cref="DrawBodyOver"/>).</summary>
    private static readonly Dictionary<(string Words, SKColor Frame, float Width, float Height), SKPicture> Bodies = [];

    /// <summary>How many bodies are kept before they are all let go and drawn again as they are wanted.</summary>
    private const int MostBodies = 512;

    /// <summary>
    /// The counter the card's body shows on a badge rather than power and toughness: a planeswalker's loyalty
    /// (CR 306.5c), or a battle's defense (CR 310.4c). Null for a card whose body is its power and toughness, or none.
    /// </summary>
    public static string? BodyCounter(ObjectView card) =>
        card.Power is not null && card.Toughness is not null ? null
        : card.CardTypes.Contains(CardTypeView.Planeswalker) ? CounterNames.Loyalty
        : card.CardTypes.Contains(CardTypeView.Battle) ? CounterNames.Defense
        : null;

    private static string? Body(ObjectView card)
    {
        if (card.Power is not null && card.Toughness is not null)
        {
            return $"{card.Power}/{card.Toughness}";
        }

        if (BodyCounter(card) is { } counter)
        {
            int count = card.StartingCounter ?? card.Counters.FirstOrDefault(c => c.Name == counter)?.Count ?? 0;
            return count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        }

        return null;
    }

    /// <summary>Power and toughness with the frame's shadowed ink, or a planeswalker's loyalty or a battle's defense on a badge of its own.</summary>
    private static void DrawBody(SKCanvas canvas, ObjectView card, SKColor frame, Layout layout, SKColor outerText, SKColor outerShadow)
    {
        if (Body(card) is not { } body)
        {
            return;
        }

        SKRect box = layout.Body;
        int size = layout.BodyFontSize;
        float baseline = CapBaseline(CardAssets.Title, box, size);

        if (card.Power is not null && card.Toughness is not null)
        {
            float x = Aligned(CardAssets.Title, body, size, box, layout.BodyAlignment);
            if (outerShadow.Alpha > 0)
            {
                DrawString(canvas, CardAssets.Title, body, x + layout.OuterTextShadowOffset, baseline + layout.OuterTextShadowOffset, size, outerShadow);
            }

            DrawString(canvas, CardAssets.Title, body, x, baseline, size, outerText);
            return;
        }

        if (BodyCounter(card) == CounterNames.Loyalty)
        {
            // Loyalty on the mana font's shield, as wide as the box's height lets it stand.
            float shieldWidth = MathF.Min(box.Width, box.Height / LoyaltyStartAspect);
            DrawLoyaltyStart(canvas, new SKPoint(box.MidX, box.MidY), shieldWidth, body, InkColor, SKColors.White);
            return;
        }

        RoundedBox(canvas, box, Parchment, Lerp(frame, InkColor, 0.4f), 1, 5);
        DrawString(canvas, CardAssets.Title, body, Aligned(CardAssets.Title, body, size, box, "center"), baseline, size, InkColor);
    }

    /// <summary>How tall the loyalty shield's ink stands to its width.</summary>
    private const float LoyaltyStartAspect = 0.6f;

    private static void DrawMarkedDamage(SKCanvas canvas, ObjectView card, Layout layout)
    {
        if (card.DamageMarked <= 0 || Body(card) is null)
        {
            return;
        }

        string damage = $"-{card.DamageMarked}";
        int size = Math.Max(layout.BodyFontSize, (int)MathF.Round(layout.BodyFontSize * 1.18f));
        using var font = new SKFont(CardAssets.Title, size);
        float width = font.MeasureText(damage);
        DrawString(canvas, CardAssets.Title, damage, layout.Body.Left - width - 3, Baseline(CardAssets.Title, layout.Body, size), size, SKColor.Parse("e2483c"));
    }

    /// <summary>The counters on a permanent, one pill per kind across the art box, each with how many beside it: "4 × (+1/+1)".</summary>
    private static float DrawCounters(SKCanvas canvas, ObjectView card, Layout layout, float cardWidth)
    {
        string? body = BodyCounter(card);
        List<CounterView> shown = [.. card.Counters.Where(c => c.Count > 0 && c.Name != body)];
        if (shown.Count == 0)
        {
            return 0f;
        }

        SKRect art = layout.Art;
        int size = Scaled(10, cardWidth);
        float inset = MathF.Max(2f, MathF.Round(cardWidth * 0.03f));
        float padX = MathF.Round(size * 0.5f);
        float height = MathF.Min(art.Height - 2f, MathF.Round((size * FigureHeight) + (padX * 1.4f)));
        float gap = MathF.Round(size * 0.35f);
        float x = art.Left + inset;
        float y = art.Top + MathF.Round((art.Height - height) / 2f);
        float baseline = MathF.Round(y + (height / 2f) + (size * FigureHeight / 2f));

        using var font = new SKFont(CardAssets.Title, size);

        foreach (CounterView counter in shown)
        {
            string count = $"{counter.Count} ×";
            string kind = counter.Name;
            float countWidth = font.MeasureText(count);
            float kindWidth = font.MeasureText(kind);
            float width = countWidth + gap + kindWidth + (padX * 2f);

            if (x + width > art.Right - inset && x > art.Left + inset)
            {
                break;
            }

            SKColor edge = counter.Name switch
            {
                CounterNames.PlusOnePlusOne => SKColor.Parse("5fbf6a"),
                CounterNames.MinusOneMinusOne => SKColor.Parse("e2584c"),
                _ => SKColor.Parse("e2b04c"),
            };

            RoundedBox(canvas, SKRect.Create(x - 2f, y, countWidth + 4f, height), new SKColor(15, 15, 20, 140), SKColors.Transparent, 0, 3);
            DrawString(canvas, CardAssets.Title, count, x, baseline, size, SKColors.White);

            var pill = SKRect.Create(x + countWidth + gap, y, kindWidth + (padX * 2f), height);
            RoundedBox(canvas, pill, new SKColor(18, 18, 23, 235), edge, 1, MathF.Ceiling(height / 2f));
            DrawString(canvas, CardAssets.Title, kind, pill.Left + padX, baseline, size, Lerp(edge, SKColors.White, 0.45f));

            x = pill.Right + gap;
        }

        return x - art.Left;
    }

    /// <summary>
    /// Where the eye on a card of this player's that an opponent knows about is drawn, in the card's own frame: on the
    /// frame's left edge just below the name, outside the art box, so it covers nothing on the face. Null for a card no
    /// one else knows. The page asks here too, for the eye's tooltip, so what is drawn and what is pointed at cannot
    /// drift apart.
    /// </summary>
    public static SKRect? EyeBox(ObjectView card, float width, float height)
    {
        if (card.KnownBy.Count == 0 || card.FaceDown || !card.Known)
        {
            return null;
        }

        var layout = Layout.Of(card, width, height);
        SKRect art = layout.Art;

        // As wide as the border beside the art leaves, and no wider than a small mark.
        float size = MathF.Min(Scaled(8, width, floor: 5), art.Left * 0.9f);
        float radius = size * 0.56f;
        float x = art.Left / 2f;
        float y = art.Top + radius;
        return new SKRect(x - radius, y - radius, x + radius, y + radius);
    }

    /// <summary>
    /// The eye itself, filling its box: a dark disc close round a white almond, a red-brown iris, a dark pupil. There is no
    /// emoji font here, so it is drawn.
    /// </summary>
    private static void DrawEye(SKCanvas canvas, SKRect box)
    {
        var centre = new SKPoint(box.MidX, box.MidY);
        float radius = box.Width / 2f;

        using (var disc = new SKPaint { Color = new SKColor(15, 15, 20, 214), IsAntialias = true })
        {
            canvas.DrawCircle(centre, radius, disc);
        }

        float half = radius * 0.86f;
        float lid = half * 0.62f;
        using var almond = new SKPath();
        almond.MoveTo(centre.X - half, centre.Y);
        almond.QuadTo(centre.X, centre.Y - (lid * 1.6f), centre.X + half, centre.Y);
        almond.QuadTo(centre.X, centre.Y + (lid * 1.6f), centre.X - half, centre.Y);
        almond.Close();

        using (var white = new SKPaint { Color = new SKColor(240, 240, 244), IsAntialias = true })
        {
            canvas.DrawPath(almond, white);
        }

        using (var iris = new SKPaint { Color = new SKColor(150, 72, 62), IsAntialias = true })
        {
            canvas.DrawCircle(centre, lid * 0.95f, iris);
        }

        using var pupil = new SKPaint { Color = new SKColor(20, 20, 26), IsAntialias = true };
        canvas.DrawCircle(centre, lid * 0.45f, pupil);
    }

    /// <summary>
    /// The art box of <paramref name="card"/> drawn <paramref name="width"/> by <paramref name="height"/>, unturned, which
    /// a creature just arrived wears its mark over (<see cref="DrawArrival"/>); null for a card that does not wear it.
    /// </summary>
    public static SKRect? ArrivalBox(ObjectView card, float width, float height) =>
        card.SummoningSick && card.Known && !card.FaceDown ? Layout.Of(card, width, height).Art : null;

    /// <summary>
    /// The mark of a creature just arrived (the mana font's summoning sickness), over the middle of its art box, light on a
    /// soft dark halo so it reads on any illustration. <paramref name="breath"/>, from 0 to 1, is where it is in its slow
    /// rise and fall: drawn by the row's own layer each frame (<see cref="ArrivalsElement"/>), not on the face, so the
    /// face is not drawn again for it.
    /// </summary>
    public static void DrawArrival(SKCanvas canvas, SKRect art, float breath)
    {
        float swell = 0.5f - (0.5f * MathF.Cos(breath * 2f * MathF.PI));
        int size = Math.Max(8, (int)MathF.Round(MathF.Min(art.Width, art.Height) * (0.5f + (0.06f * swell))));
        var centre = new SKPoint(art.MidX, art.MidY);
        float alpha = 0.6f + (0.35f * swell);

        using var font = new SKFont(CardAssets.Mana, size);
        font.MeasureText(ManaSymbols.SummoningSickness, out SKRect ink);
        float x = centre.X - ink.MidX;
        float y = centre.Y - ink.MidY;

        using var halo = new SKPaint
        {
            Color = new SKColor(0, 0, 0, (byte)MathF.Round(150 * alpha)),
            IsAntialias = true,
            Style = SKPaintStyle.StrokeAndFill,
            StrokeWidth = MathF.Max(1f, size * 0.08f),
            MaskFilter = SKMaskFilter.CreateBlur(SKBlurStyle.Normal, MathF.Max(1f, size * 0.06f)),
        };
        canvas.DrawText(ManaSymbols.SummoningSickness, x, y, SKTextAlign.Left, font, halo);

        using var mark = new SKPaint { Color = SKColor.Parse("f4eee1").WithAlpha((byte)MathF.Round(255 * alpha)), IsAntialias = true };
        canvas.DrawText(ManaSymbols.SummoningSickness, x, y, SKTextAlign.Left, font, mark);
    }

    /// <summary>
    /// What a copy is, on a tag at the foot of its art: "Token" for a token copying a card, "Copy" for a card copying
    /// another or a copied spell. Otherwise it reads exactly as what it copies, art and all.
    /// </summary>
    private static void DrawCopyTag(SKCanvas canvas, ObjectView card, Layout layout, float cardWidth)
    {
        if (!card.IsCopy)
        {
            return;
        }

        string word = card.Kind == ObjectKindView.Token ? "Token" : "Copy";
        int size = Scaled(9, cardWidth);
        float padding = MathF.Round(size * 0.4f);
        float wide;
        using (var font = new SKFont(CardAssets.Title, size))
        {
            wide = font.MeasureText(word);
        }

        SKRect art = layout.Art;
        float high = LineHeight(CardAssets.Title, size);
        var tag = SKRect.Create(art.Left + padding, art.Bottom - high - padding, wide + (padding * 2), high);
        Fill(canvas, tag, new SKColor(20, 20, 26, 199));
        DrawString(canvas, CardAssets.Title, word, tag.Left + padding, Baseline(CardAssets.Title, tag, size), size, SKColor.Parse("ffd479"));
    }

    /// <summary>What was chosen for the permanent, written across the art box.</summary>
    private static void DrawAnnotation(SKCanvas canvas, string annotation, Layout layout, float cardWidth)
    {
        SKRect art = layout.Art;
        Fill(canvas, art, new SKColor(20, 20, 26, 199));

        // Wrapped at words to at most three lines, centred, the rest dropped.
        int size = Scaled(9, cardWidth);
        float step = LineHeight(CardAssets.Title, size);
        float baseline = art.Top + size + 2;

        foreach (string line in WrapWords(CardAssets.Title, annotation, size, art.Width - 6).Take(3))
        {
            DrawString(canvas, CardAssets.Title, line, Aligned(CardAssets.Title, line, size, art, "center"), baseline, size, SKColor.Parse("ffd479"));
            baseline += step;
        }
    }

    /// <summary>Plain text broken into lines at spaces to fit a width; a word wider than the width is broken where it must be.</summary>
    private static List<string> WrapWords(SKTypeface typeface, string text, int size, float maxWidth)
    {
        using var font = new SKFont(typeface, size);
        List<string> lines = [];

        foreach (string paragraph in text.Split('\n'))
        {
            string current = string.Empty;
            foreach (string word in paragraph.Split(' ', StringSplitOptions.RemoveEmptyEntries))
            {
                string joined = current.Length == 0 ? word : $"{current} {word}";
                if (font.MeasureText(joined) <= maxWidth)
                {
                    current = joined;
                    continue;
                }

                if (current.Length > 0)
                {
                    lines.Add(current);
                }

                current = word;
                while (current.Length > 1 && font.MeasureText(current) > maxWidth)
                {
                    int fits = current.Length - 1;
                    while (fits > 1 && font.MeasureText(current.AsSpan(0, fits).ToString()) > maxWidth)
                    {
                        fits--;
                    }

                    lines.Add(current[..fits]);
                    current = current[fits..];
                }
            }

            lines.Add(current);
        }

        return lines;
    }

    // --- Faces with nothing printed to show ---------------------------------------------------------------------------

    /// <summary>A plain geometric back: a card back is something every player already reads.</summary>
    public static void DrawBack(SKCanvas canvas, SKRect face, float width)
    {
        using (var shadow = new SKPaint { Color = new SKColor(0, 0, 0, 102), IsAntialias = true, MaskFilter = SKMaskFilter.CreateBlur(SKBlurStyle.Normal, 2f) })
        {
            canvas.DrawRoundRect(SKRect.Create(face.Left, face.Top + 2, face.Width, face.Height), 5, 5, shadow);
        }

        // Two browns, so a back stands apart from the purple table it lies on.
        RoundedBox(canvas, face, SKColor.Parse("4a2e17"), FaceDownFrame, width >= StandardWidth ? 2 : 1, 5);

        float inset = MathF.Round(width * 0.11f);
        SKRect inner = SKRect.Inflate(face, -inset, -inset);
        RoundedBox(canvas, inner, SKColor.Parse("8a5a30"), SKColor.Parse("a87545"), 1, 5);

        // The five colours in the middle, white, blue, black, red, green, clockwise from the top.
        float reach = MathF.Min(inner.Width, inner.Height) * 0.27f;
        float dot = MathF.Max(1.5f, reach * 0.36f);
        using var fill = new SKPaint { IsAntialias = true };
        using var rim = new SKPaint { Color = SKColor.Parse("3a2412"), IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = MathF.Max(0.6f, dot * 0.18f) };

        string[] colours = ["f4efe1", "2f6fc4", "1b1712", "cc3a2c", "2f8f4a"];
        for (int i = 0; i < colours.Length; i++)
        {
            float angle = -MathF.PI / 2 + i * 2 * MathF.PI / colours.Length;
            float x = inner.MidX + reach * MathF.Cos(angle);
            float y = inner.MidY + reach * MathF.Sin(angle);
            fill.Color = SKColor.Parse(colours[i]);
            canvas.DrawCircle(x, y, dot, fill);
            canvas.DrawCircle(x, y, dot, rim);
        }
    }

    /// <summary>Anything face down: the back, with whatever characteristics it has, and the name underneath for a viewer allowed it.</summary>
    /// <summary>The frame colour a face-down card's body is drawn against.</summary>
    private static readonly SKColor FaceDownFrame = SKColor.Parse("2e1b0c");

    private static void DrawFaceDown(SKCanvas canvas, ObjectView card, SKRect face, float width, bool body)
    {
        DrawBack(canvas, face, width);

        Layout layout = new(face.Width, face.Height);
        if (body)
        {
            DrawBody(canvas, card, FaceDownFrame, layout, SKColor.Parse(CardAssets.Template.TextStyle.OuterTextColor), ParseAlpha(CardAssets.Template.TextStyle.OuterTextShadowColor));
        }

        DrawStatus(canvas, card, SKColor.Parse("d8d4e6"), layout);

        if (card.FaceDownName.Length > 0)
        {
            DrawAnnotation(canvas, card.FaceDownName, layout, width);
        }

        DrawCounters(canvas, card, layout, width);
        DrawMarkedDamage(canvas, card, layout);
    }

    // --- Frames -------------------------------------------------------------------------------------------------------

    /// <summary>The card's frame, or its two-colour blend, over the face.</summary>
    private static void DrawFrame(SKCanvas canvas, ObjectView card, SKRect face, SKPaint paint, FrameCut cut)
    {
        (string first, string? second) = Frames(card);

        // A mirrored panel's frame is drawn turned over left to right, and a blend's frames swapped so its first colour
        // still reads on the left.
        bool mirrored = cut == FrameCut.Whole && PanelOf(card) is { Mirrored: true };
        if (mirrored)
        {
            canvas.Save();
            canvas.Scale(-1f, 1f, face.MidX, face.MidY);
            if (second is not null)
            {
                (first, second) = (second, first);
            }
        }

        if (second is not null)
        {
            CardAssets.DrawBlendedFrame(canvas, first, second, face, paint, cut);
        }
        else
        {
            CardAssets.DrawFrame(canvas, first, face, paint, cut);
        }

        if (mirrored)
        {
            canvas.Restore();
        }
    }

    /// <summary>
    /// The frames a card's face is drawn with: its own, or the two its blend crosses between - for a split card, its
    /// halves'.
    /// </summary>
    public static IEnumerable<string> FramesOf(ObjectView card)
    {
        if (card.Halves.Count > 0 && CardAssets.Template.SplitHalf is not null)
        {
            return card.Halves.SelectMany(FramesOf).Distinct();
        }

        (string first, string? second) = Frames(card);
        return second is null ? [first] : [first, second];
    }

    /// <summary>
    /// A card's one frame, or the two its blend crosses between left to right: a Saga's or a Class's own, or any card's.
    /// </summary>
    private static (string First, string? Second) Frames(ObjectView card)
    {
        if (PanelOf(card) is { } panel && PanelFrames(card, panel) is { } frames)
        {
            return frames;
        }

        return CardAssets.Template.TwoColorFrameBlend is { } blend && TwoColorFrameNames(card, blend) is { } pair
            ? (pair.Left, pair.Right)
            : (FrameName(card), null);
    }

    /// <summary>
    /// The panel layout a card's face is drawn with: a Saga's (CR 714) or a Class's (CR 716), when the template has a
    /// frame for it, and null for every other card, or one the template has no frame for, which is drawn as any other
    /// card is.
    /// </summary>
    private static CardTemplate.PanelLayout? PanelOf(ObjectView card)
    {
        CardTemplate.PanelLayout? panel = card.Subtypes.Contains("Saga") ? CardAssets.Template.Saga
            : card.Subtypes.Contains("Class") ? CardAssets.Template.Class
            : null;
        return panel is not null && PanelFrames(card, panel) is not null ? panel : null;
    }

    /// <summary>
    /// The token layout a token of its own is drawn with - a Food, a Clue, a 1/1 Human - whose few words leave its art the
    /// room. Null for a card, for a token that copies one, which reads as what it copies, and for a Saga's or a Class's
    /// panel, which lays its art out its own way.
    /// </summary>
    private static CardTemplate.TokenLayout? TokenOf(ObjectView card) =>
        card.Kind == ObjectKindView.Token && !card.IsCopy && card.Halves.Count == 0 && PanelOf(card) is null
            ? CardAssets.Template.Token
            : null;

    /// <summary>
    /// A panel's frames, picked as any card's are among its own: a land's, two colours blended, more than two gold, and
    /// one its own. Null for a card none of them is for.
    /// </summary>
    private static (string First, string? Second)? PanelFrames(ObjectView card, CardTemplate.PanelLayout saga)
    {
        if (card.CardTypes.Contains(CardTypeView.Land))
        {
            return saga.Frames.TryGetValue("land", out string? land) ? (land, null) : null;
        }

        string[] colors = [.. card.Colors.Distinct().Select(ColorName)];
        if (colors.Length == 2 && CardAssets.Template.TwoColorFrameBlend is { } blend)
        {
            string[] ordered = [.. colors.OrderBy(blend.OrderOf)];
            if (saga.Frames.TryGetValue(ordered[0], out string? left) && saga.Frames.TryGetValue(ordered[1], out string? right))
            {
                return (left, right);
            }
        }

        string key = colors.Length switch
        {
            0 => "colorless",
            1 => colors[0],
            _ => "gold",
        };

        return saga.Frames.TryGetValue(key, out string? frame) ? (frame, null) : null;
    }

    private static (string Left, string Right)? TwoColorFrameNames(ObjectView card, CardTemplate.FrameBlend blend)
    {
        bool land = card.CardTypes.Contains(CardTypeView.Land);
        IEnumerable<ColorView> colors = land ? ProducedLandColors(card) : card.Colors;
        string[] ordered = [.. colors.Distinct().Select(ColorName).OrderBy(blend.OrderOf)];

        if (ordered.Length != 2 || !blend.TryFrame(ordered[0], land, out string left) || !blend.TryFrame(ordered[1], land, out string right))
        {
            return null;
        }

        return (left, right);
    }

    private static string FrameName(ObjectView card)
    {
        if (card.CardTypes.Contains(CardTypeView.Land))
        {
            HashSet<ColorView> produced = ProducedLandColors(card);
            return produced.Count == 1 ? $"land_{ColorName(produced.Single())}" : "land";
        }

        if (card.Colors.Count > 1)
        {
            return "gold";
        }

        if (card.Colors.Count == 1)
        {
            return ColorName(card.Colors[0]);
        }

        return card.CardTypes.Contains(CardTypeView.Artifact) ? "artifact" : "colorless";
    }

    /// <summary>The colours a land makes: its basic land types (CR 305.6), and the coloured symbols its mana abilities add.</summary>
    private static HashSet<ColorView> ProducedLandColors(ObjectView card)
    {
        HashSet<ColorView> colors = [.. card.Subtypes.Select(LandColor).Where(c => c is not null).Select(c => c!.Value)];

        if (card.ProducesMana)
        {
            foreach ((string symbol, bool isSymbol) in RulesLines(card).SelectMany(rule => ManaSymbols.Parse(rule.Text)))
            {
                if (isSymbol && SymbolColor(symbol) is { } color)
                {
                    colors.Add(color);
                }
            }
        }

        return colors;
    }

    private static ColorView? LandColor(string subtype) => subtype switch
    {
        "Plains" => ColorView.White,
        "Island" => ColorView.Blue,
        "Swamp" => ColorView.Black,
        "Mountain" => ColorView.Red,
        "Forest" => ColorView.Green,
        _ => null,
    };

    private static ColorView? SymbolColor(string symbol) => symbol switch
    {
        "W" => ColorView.White,
        "U" => ColorView.Blue,
        "B" => ColorView.Black,
        "R" => ColorView.Red,
        "G" => ColorView.Green,
        _ => null,
    };

    private static string ColorName(ColorView color) => color.ToString().ToLowerInvariant();

    /// <summary>The frame colour a card's marks are tinted from: its colours, then the mana a land makes, then colourless.</summary>
    private static SKColor FrameColor(ObjectView card)
    {
        if (card.Colors.Count > 1)
        {
            return SKColor.Parse("9b8146");
        }

        if (card.Colors.Count == 1)
        {
            return Of(card.Colors[0]);
        }

        if (!card.CardTypes.Contains(CardTypeView.Land))
        {
            return SKColor.Parse("82786a");
        }

        List<SKColor> produced = [.. card.Subtypes.Select(LandColor).Where(c => c is not null).Select(c => Of(c!.Value))];
        SKColor land = SKColor.Parse("80694f");

        return produced.Count switch
        {
            0 => land,
            1 => Lerp(produced[0], land, 0.4f),
            _ => Lerp(SKColor.Parse("9b8146"), land, 0.3f),
        };
    }

    private static SKColor Of(ColorView color) => color switch
    {
        ColorView.White => SKColor.Parse("d9c99c"),
        ColorView.Blue => SKColor.Parse("557b98"),
        ColorView.Black => SKColor.Parse("514951"),
        ColorView.Red => SKColor.Parse("a6533f"),
        ColorView.Green => SKColor.Parse("527657"),
        _ => SKColor.Parse("82786a"),
    };

    // --- Text and shapes ----------------------------------------------------------------------------------------------

    /// <summary>A design size for a card drawn smaller than standard, with a floor so it stays legible.</summary>
    private static int Scaled(int atStandardWidth, float cardWidth, int floor = 7) =>
        Math.Max(floor, (int)MathF.Round(atStandardWidth * (cardWidth / StandardWidth)));

    private static float LineHeight(SKTypeface typeface, int size)
    {
        using var font = new SKFont(typeface, size);
        return font.Metrics.Descent - font.Metrics.Ascent;
    }

    private static float Baseline(SKTypeface typeface, SKRect box, int size)
    {
        using var font = new SKFont(typeface, size);
        return box.MidY + ((-font.Metrics.Ascent - font.Metrics.Descent) / 2f);
    }

    /// <summary>
    /// The baseline that centres a line's capitals in <paramref name="box"/>: what the eye reads as a line of a name or a
    /// type set in a band of the frame, where centring the font's whole height would sit it off by its descenders.
    /// </summary>
    private static float CapBaseline(SKTypeface typeface, SKRect box, int size) =>
        box.MidY + (CapHeight(typeface, size) / 2f);

    private static float CapHeight(SKTypeface typeface, int size)
    {
        using var font = new SKFont(typeface, size);
        return font.Metrics.CapHeight > 0f ? font.Metrics.CapHeight : -font.Metrics.Ascent * FigureHeight;
    }

    private static float Room(Layout layout, float from, SKRect box) =>
        MathF.Max(0f, MathF.Min(box.Right - layout.TextInset, layout.TextClip.Right) - from);

    /// <summary>Reduces display text only as far as needed, as printed cards do for unusually long names and types.</summary>
    private static int FittedSize(SKTypeface typeface, string value, int maximum, int minimum, float width)
    {
        for (int size = maximum; size > minimum; size--)
        {
            using var font = new SKFont(typeface, size);
            if (font.MeasureText(value) <= width)
            {
                return size;
            }
        }

        return minimum;
    }

    /// <summary>Truncates to fit, because a card is narrower than most card names.</summary>
    private static string Fit(SKTypeface typeface, string text, int size, float maxWidth)
    {
        using var font = new SKFont(typeface, size);
        if (font.MeasureText(text) <= maxWidth)
        {
            return text;
        }

        for (int length = text.Length - 1; length > 0; length--)
        {
            string candidate = string.Concat(text.AsSpan(0, length).TrimEnd(), "...");
            if (font.MeasureText(candidate) <= maxWidth)
            {
                return candidate;
            }
        }

        return string.Empty;
    }

    private static float Aligned(SKTypeface typeface, string text, int size, SKRect box, string alignment)
    {
        using var font = new SKFont(typeface, size);
        float width = font.MeasureText(text);
        return alignment.Equals("right", StringComparison.OrdinalIgnoreCase) ? box.Right - width
            : alignment.Equals("center", StringComparison.OrdinalIgnoreCase) ? box.Left + ((box.Width - width) / 2f)
            : box.Left;
    }

    private static void DrawFrameString(SKCanvas canvas, float x, float baseline, string value, int size, SKColor ink, SKColor shadow, float offset)
    {
        if (shadow.Alpha > 0)
        {
            DrawString(canvas, CardAssets.Title, value, x + offset, baseline + offset, size, shadow);
        }

        DrawString(canvas, CardAssets.Title, value, x, baseline, size, ink);
    }

    private static void DrawString(SKCanvas canvas, SKTypeface typeface, string value, float x, float baseline, int size, SKColor color)
    {
        using var font = new SKFont(typeface, size) { Subpixel = true };
        using var paint = new SKPaint { Color = color, IsAntialias = true };
        canvas.DrawText(value, x, baseline, SKTextAlign.Left, font, paint);
    }

    private static void Fill(SKCanvas canvas, SKRect rect, SKColor color)
    {
        using var paint = new SKPaint { Color = color, IsAntialias = true };
        canvas.DrawRect(rect, paint);
    }

    private static void Stroke(SKCanvas canvas, SKRect rect, SKColor color, float width)
    {
        using var paint = new SKPaint { Color = color, IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = width };
        canvas.DrawRect(SKRect.Inflate(rect, -width / 2f, -width / 2f), paint);
    }

    /// <summary>A filled rounded box with its border drawn inside its edge.</summary>
    private static void RoundedBox(SKCanvas canvas, SKRect rect, SKColor fill, SKColor border, float borderWidth, float radius)
    {
        using (var paint = new SKPaint { Color = fill, IsAntialias = true })
        {
            canvas.DrawRoundRect(rect, radius, radius, paint);
        }

        if (borderWidth > 0 && border.Alpha > 0)
        {
            using var paint = new SKPaint { Color = border, IsAntialias = true, Style = SKPaintStyle.Stroke, StrokeWidth = borderWidth };
            canvas.DrawRoundRect(SKRect.Inflate(rect, -borderWidth / 2f, -borderWidth / 2f), radius, radius, paint);
        }
    }

    private static SKColor Lerp(SKColor from, SKColor to, float amount) => new(
        (byte)MathF.Round(from.Red + ((to.Red - from.Red) * amount)),
        (byte)MathF.Round(from.Green + ((to.Green - from.Green) * amount)),
        (byte)MathF.Round(from.Blue + ((to.Blue - from.Blue) * amount)),
        (byte)MathF.Round(from.Alpha + ((to.Alpha - from.Alpha) * amount)));

    /// <summary>A hex colour string, which may carry alpha as a fourth byte.</summary>
    private static SKColor ParseAlpha(string hex) =>
        hex.Length == 8
            ? new SKColor(Convert.ToByte(hex[..2], 16), Convert.ToByte(hex[2..4], 16), Convert.ToByte(hex[4..6], 16), Convert.ToByte(hex[6..8], 16))
            : SKColor.Parse(hex);
}
