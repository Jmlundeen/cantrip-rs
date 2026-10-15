using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;
using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// What every card face is drawn with: the retro template's layout (content/res/templates/retro.json), its frame
/// bitmaps, and the three typefaces. All embedded from the tracked content files.
/// </summary>
public static class CardAssets
{
    private static readonly Dictionary<string, SKImage> Frames = new(StringComparer.Ordinal);

    public static CardTemplate Template { get; } = LoadTemplate();

    /// <summary>Card names and type lines, and power and toughness.</summary>
    public static SKTypeface Title { get; } = LoadTypeface("beleren-bold_P1.01.ttf");

    /// <summary>Rules text.</summary>
    public static SKTypeface Body { get; } = LoadTypeface("mplantin.ttf");

    /// <summary>The mana symbol icon font.</summary>
    public static SKTypeface Mana { get; } = LoadTypeface("mana.ttf");

    /// <summary>
    /// A frame at the size it was made, 1005 by 1407, decoded once and kept. It is drawn at whatever size a face is shown
    /// by the GPU, from mipmaps it makes once, so there is no copy of it at each size to make or keep.
    /// </summary>
    private static SKImage Frame(string name)
    {
        if (!Frames.TryGetValue(name, out SKImage? frame))
        {
            using Stream stream = Open($"frame_{name}.png");
            using SKBitmap bitmap = SKBitmap.Decode(stream);
            bitmap.SetImmutable();
            frame = SKImage.FromBitmap(bitmap);
            Frames[name] = frame;
        }

        return frame;
    }

    private static readonly Dictionary<string, SKImage> Icons = new(StringComparer.Ordinal);

    /// <summary>
    /// One of the template's icons (content/res/icons), at the size it was made in template units, decoded once and kept:
    /// a Saga's chapter badge and the line between its chapters.
    /// </summary>
    private static SKImage Icon(string name)
    {
        if (!Icons.TryGetValue(name, out SKImage? icon))
        {
            using Stream stream = Open($"icon_{name}.png");
            using SKBitmap bitmap = SKBitmap.Decode(stream);
            bitmap.SetImmutable();
            icon = SKImage.FromBitmap(bitmap);
            Icons[name] = icon;
        }

        return icon;
    }

    /// <summary>How tall an icon stands to how wide it is.</summary>
    public static float IconAspect(string name)
    {
        SKImage icon = Icon(name);
        return icon.Height / (float)icon.Width;
    }

    /// <summary>An icon over a box.</summary>
    public static void DrawIcon(SKCanvas canvas, string name, SKRect box, SKPaint? paint = null) =>
        canvas.DrawImage(Icon(name), box, FrameSampling, paint);

    /// <summary>Whether a frame is decoded already.</summary>
    public static bool HasFrame(string name) => Frames.ContainsKey(name);

    /// <summary>Decodes a frame ahead of the first face drawn with it.</summary>
    public static void PrepareFrame(string name) => _ = Frame(name);

    private static readonly SKSamplingOptions FrameSampling = new(SKFilterMode.Linear, SKMipmapMode.Linear);

    /// <summary>
    /// A frame cut down to one half of a split card (<see cref="CardTemplate.SplitLayout"/>), made once from the whole
    /// frame and kept: the template's bands taken out of its middle, and each seam cross-faded, so the border, the bevels
    /// and the title bar keep their thickness and only the stretches between them are shorter.
    /// </summary>
    private static SKImage HalfFrame(string name)
    {
        if (!HalfFrames.TryGetValue(name, out SKImage? half))
        {
            // The cuts are in template units, and a frame need not be drawn at one pixel to the unit.
            CardTemplate.SplitLayout split = Template.SplitHalf!;
            SKImage whole = Frame(name);
            float scale = whole.Width / Template.Width;
            using SKImage narrowed = Kitbash(whole, CardTemplate.Span.Between(split.ColumnCuts, Template.Width), split.CrossFade, scale, across: true);
            half = Kitbash(narrowed, CardTemplate.Span.Between(split.RowCuts, Template.Height), split.CrossFade, scale, across: false);
            HalfFrames[name] = half;
        }

        return half;
    }

    private static readonly Dictionary<string, SKImage> HalfFrames = new(StringComparer.Ordinal);

    /// <summary>
    /// A frame rebuilt for a token of its own (<see cref="CardTemplate.TokenLayout"/>), made once from the whole frame and
    /// kept: its rows laid down again as the template's spans say - a stretch of the art box's twice, a stretch of the
    /// rules box's not at all - so the card is as tall and the art box has what the rules box gave up.
    /// </summary>
    private static SKImage TokenFrame(string name)
    {
        if (!TokenFrames.TryGetValue(name, out SKImage? token))
        {
            CardTemplate.TokenLayout layout = Template.Token!;
            SKImage whole = Frame(name);
            token = Kitbash(whole, layout.Rows, layout.CrossFade, whole.Width / Template.Width, across: false);
            TokenFrames[name] = token;
        }

        return token;
    }

    private static readonly Dictionary<string, SKImage> TokenFrames = new(StringComparer.Ordinal);

    private static SKImage Cut(string name, FrameCut cut) => cut switch
    {
        FrameCut.SplitHalf => HalfFrame(name),
        FrameCut.Token => TokenFrame(name),
        _ => Frame(name),
    };

    /// <summary>
    /// An image made of stretches of another laid end to end along one direction - columns <paramref name="across"/>,
    /// rows otherwise - each after the first laid over the end of the one before it and faded in over
    /// <paramref name="fade"/> about the seam, so the frame's texture runs on without an edge. A stretch may go back over
    /// what the one before it covered, or skip ahead of it. Spans and fade are in template units, <paramref name="scale"/>
    /// pixels of the image to the unit.
    /// </summary>
    private static SKImage Kitbash(SKImage source, IReadOnlyList<CardTemplate.Span> templateSpans, float templateFade, float scale, bool across)
    {
        CardTemplate.Span[] spans = [.. templateSpans.Select(span => new CardTemplate.Span { From = span.From * scale, To = span.To * scale })];
        float fade = templateFade * scale;
        int length = across ? source.Width : source.Height;
        int kept = (int)MathF.Round(spans.Sum(span => span.To - span.From));
        var info = new SKImageInfo(across ? kept : source.Width, across ? source.Height : kept, SKColorType.Rgba8888, SKAlphaType.Premul);
        using SKSurface surface = SKSurface.Create(info) ?? throw new InvalidOperationException("Could not create an offscreen Skia surface.");
        SKCanvas canvas = surface.Canvas;
        canvas.Clear(SKColors.Transparent);

        SKRect Range(float from, float to) => across
            ? new SKRect(from, 0, to, source.Height)
            : new SKRect(0, from, source.Width, to);

        SKPoint At(float along) => across ? new SKPoint(along, 0) : new SKPoint(0, along);

        float laid = 0f;
        for (int i = 0; i < spans.Length; i++)
        {
            // Each stretch runs half the fade past its end, and the next starts half the fade before its own start.
            float start = i > 0 ? spans[i].From - (fade / 2f) : spans[i].From;
            float end = i < spans.Length - 1 ? spans[i].To + (fade / 2f) : MathF.Min(spans[i].To, length);
            float shift = laid - spans[i].From;
            SKRect from = Range(start, end);
            SKRect to = Range(start + shift, end + shift);

            if (i == 0)
            {
                canvas.DrawImage(source, from, to, FrameSampling);
            }
            else
            {
                canvas.SaveLayer(to, null);
                canvas.DrawImage(source, from, to, FrameSampling);
                using SKShader rise = Rise(At(start + shift), At(start + shift + fade));
                using var mask = new SKPaint { Shader = rise, BlendMode = SKBlendMode.DstIn };
                canvas.DrawRect(to, mask);
                canvas.Restore();
            }

            laid += spans[i].To - spans[i].From;
        }

        return surface.Snapshot();
    }

    /// <summary>A mask that rises from nothing to whole between two points along a smoothstep, and holds either side.</summary>
    private static SKShader Rise(SKPoint from, SKPoint to)
    {
        const int Stops = 17;
        var colors = new SKColor[Stops];
        float[] positions = new float[Stops];
        for (int i = 0; i < Stops; i++)
        {
            float t = i / (Stops - 1f);
            float smooth = t * t * (3f - (2f * t));
            positions[i] = t;
            colors[i] = new SKColor(255, 255, 255, (byte)MathF.Round(smooth * 255));
        }

        return SKShader.CreateLinearGradient(from, to, colors, positions, SKShaderTileMode.Clamp);
    }

    /// <summary>A frame over a face's box, whole or cut as <paramref name="cut"/> says.</summary>
    public static void DrawFrame(SKCanvas canvas, string name, SKRect face, SKPaint paint, FrameCut cut = FrameCut.Whole) =>
        canvas.DrawImage(Cut(name, cut), face, FrameSampling, paint);

    /// <summary>
    /// Two frames cross-faded left to right over the template's blend range with a smoothstep, for a two-coloured card:
    /// the left frame, and over it the right one through a mask that rises across the blend range along the smoothstep curve, which for the frame's opaque pixels is the same
    /// cross-fade. The mask is a shader on the right frame rather than a layer, so the GPU blends it as it draws.
    /// </summary>
    public static void DrawBlendedFrame(SKCanvas canvas, string left, string right, SKRect face, SKPaint paint, FrameCut cut = FrameCut.Whole)
    {
        CardTemplate.FrameBlend blend = Template.TwoColorFrameBlend!;
        DrawFrame(canvas, left, face, paint, cut);

        SKImage second = Cut(right, cut);
        var fit = SKMatrix.CreateScaleTranslation(face.Width / second.Width, face.Height / second.Height, face.Left, face.Top);
        using SKShader frame = second.ToShader(SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, FrameSampling, fit);
        using SKShader mask = Rise(
            new SKPoint(face.Left + (face.Width * blend.Start), 0),
            new SKPoint(face.Left + (face.Width * blend.End), 0));

        // The frame is the destination and the mask the source: DstIn keeps the frame where the mask is.
        using SKShader masked = SKShader.CreateCompose(frame, mask, SKBlendMode.DstIn);
        using SKPaint over = paint.Clone();
        over.Shader = masked;
        canvas.DrawRect(face, over);
    }

    private static CardTemplate LoadTemplate()
    {
        using Stream stream = Open("retro.json");
        return JsonSerializer.Deserialize(stream, TemplateJson.Default.CardTemplate)
            ?? throw new InvalidDataException("The retro card template is empty.");
    }

    private static SKTypeface LoadTypeface(string name)
    {
        using Stream stream = Open(name);
        return SKTypeface.FromStream(stream);
    }

    private static Stream Open(string name)
    {
        Assembly assembly = typeof(CardAssets).Assembly;
        return assembly.GetManifestResourceStream(name)
            ?? throw new FileNotFoundException($"'{name}' is not embedded in the web client.", name);
    }
}

/// <summary>How a frame is cut from the whole one before it is drawn.</summary>
public enum FrameCut
{
    Whole,

    /// <summary>To one half of a split card (<see cref="CardTemplate.SplitLayout"/>).</summary>
    SplitHalf,

    /// <summary>To a token of its own, with the taller art box (<see cref="CardTemplate.TokenLayout"/>).</summary>
    Token,
}

/// <summary>The shape of a card frame template file, in template units.</summary>
public sealed class CardTemplate
{
    public float Width { get; set; }
    public float Height { get; set; }
    public Box Art { get; set; } = new();
    public Box Title { get; set; } = new();
    public Box TypeLine { get; set; } = new();
    public Box Rules { get; set; } = new();
    public Box Body { get; set; } = new();
    public Box Status { get; set; } = new();
    public Box Artist { get; set; } = new();
    public float TitleFontSize { get; set; }
    public float MinimumTitleFontSize { get; set; }
    public float TypeFontSize { get; set; }
    public float MinimumTypeFontSize { get; set; }
    public float RulesFontSize { get; set; }
    public float MinimumRulesFontSize { get; set; }
    public float BodyFontSize { get; set; }
    public string BodyAlignment { get; set; } = "right";
    public float StatusFontSize { get; set; }
    public float ArtistFontSize { get; set; }
    public float ManaSymbolDiameter { get; set; }
    public float ManaSymbolGap { get; set; }
    public FrameBlend? TwoColorFrameBlend { get; set; }
    public SplitLayout? SplitHalf { get; set; }
    public TokenLayout? Token { get; set; }
    public float TextInset { get; set; }
    public float TextClipInset { get; set; } = 30f;
    public float RulesInset { get; set; }

    /// <summary>The narrower margin the rules box takes when its text would otherwise be set a size smaller.</summary>
    public float TightRulesInset { get; set; }

    /// <summary>The room between one ability and the next in rules text, in line heights, past the line's own.</summary>
    public float ParagraphGap { get; set; }

    /// <summary>Where a loyalty ability's shield is centred across the card - the text panel's left edge - and how wide it is.</summary>
    public float LoyaltyCentreX { get; set; }
    public float LoyaltyWidth { get; set; }

    /// <summary>The icon drawn in that room between a planeswalker's abilities; none when empty.</summary>
    public string AbilitySeparator { get; set; } = "";
    public string RulesAlignment { get; set; } = "left";
    public float LandSymbolSize { get; set; }
    public float OuterInset { get; set; }
    public float OuterTextShadowOffset { get; set; }
    public TextStyles TextStyle { get; set; } = new();
    public PanelLayout? Saga { get; set; }
    public PanelLayout? Class { get; set; }

    public sealed class Box
    {
        public float X { get; set; }
        public float Y { get; set; }
        public float Width { get; set; }
        public float Height { get; set; }
    }

    public sealed class TextStyles
    {
        public string OuterTextColor { get; set; } = "18130e";
        public string OuterTextShadowColor { get; set; } = "00000000";
        public string RulesVerticalAlignment { get; set; } = "top";
    }

    /// <summary>
    /// How one half of a split card is cut from the template (CR 709.1): the halves stand side by side along the card's
    /// length, each as tall as the card is wide, so each is the template with bands taken out of its middle - not shrunk,
    /// so its border, title bar and mana cost are the size a whole card's are at that size.
    /// <para>
    /// The cuts take the template to half the card's length plus the <see cref="Overlap"/> across, by the card's width
    /// down: the halves overlap by one black outer border, so the line between them is as thick as the card's edge
    /// rather than twice it. Boxes are carried across the cuts, so one a cut runs through is shorter by what was cut.
    /// </para>
    /// </summary>
    public sealed class SplitLayout
    {
        public float Overlap { get; set; }
        public float CrossFade { get; set; }
        public List<Cut> ColumnCuts { get; set; } = [];
        public List<Cut> RowCuts { get; set; } = [];

        /// <summary>Where a point across the template lands across a half.</summary>
        public float MapX(float x) => Map(x, ColumnCuts);

        /// <summary>Where a point down the template lands down a half.</summary>
        public float MapY(float y) => Map(y, RowCuts);

        private static float Map(float along, List<Cut> cuts) =>
            along - cuts.Sum(cut => Math.Clamp(along - cut.At, 0f, cut.Length));
    }

    /// <summary>
    /// A token's own face - a Food, a Clue, a 1/1 Human, any token that is not a copy of a card: the template's rows laid
    /// down again as <see cref="Rows"/> says, so the art box is taller by what the rules box, which a token's few words do
    /// not fill, is shorter. The card stays the template's height, and each box is carried along with the rows it is in.
    /// </summary>
    public sealed class TokenLayout
    {
        public float CrossFade { get; set; }

        /// <summary>The stretches of the template's rows the token's frame is laid down from, top to bottom.</summary>
        public List<Span> Rows { get; set; } = [];

        /// <summary>
        /// Where a point down the template lands down a token: in the first stretch that has it, or, in rows no stretch
        /// keeps, where the next one starts.
        /// </summary>
        public float MapY(float y)
        {
            float laid = 0f;
            foreach (Span span in Rows)
            {
                if (y < span.From)
                {
                    return laid;
                }

                if (y <= span.To)
                {
                    return laid + y - span.From;
                }

                laid += span.To - span.From;
            }

            return laid;
        }
    }

    /// <summary>
    /// A Saga's face (CR 714) or a Class's (CR 716): its own frames, with the chapters or levels down a panel beside a
    /// tall illustration and the type line at the foot. Its boxes take the place of the card's; the title stays where
    /// every card has it.
    /// <para>
    /// The panel reads top to bottom: the reminder text, in italics - the one reminder a face prints, since the chapters
    /// and levels do not read without it - then each chapter or level beside its badge on the ribbon, a line between each.
    /// </para>
    /// </summary>
    public sealed class PanelLayout
    {
        /// <summary>Whether the frames are drawn turned over left to right: a Class's are a Saga's, the panel on the right.</summary>
        public bool Mirrored { get; set; }

        /// <summary>The frame for each colour, "gold" for more than one, and "land" for a land; a colour missing is a Saga drawn as any other card.</summary>
        public Dictionary<string, string> Frames { get; set; } = [];
        public Box Art { get; set; } = new();
        public Box TypeLine { get; set; } = new();
        public Box Rules { get; set; } = new();
        public Box Status { get; set; } = new();
        public Box Artist { get; set; } = new();
        public float RulesFontSize { get; set; }
        public float MinimumRulesFontSize { get; set; }

        /// <summary>Room above and below each chapter's text, and below the reminder.</summary>
        public float ChapterPadding { get; set; }

        /// <summary>The badge a chapter's number is set on, its centre across the card, its width and the gap between two of one chapter.</summary>
        public string ChapterIcon { get; set; } = "saga_chapter";
        public float ChapterIconCentreX { get; set; }
        public float ChapterIconWidth { get; set; }
        public float ChapterIconGap { get; set; }
        public float ChapterNumberFontSize { get; set; }
        public string ChapterNumberColor { get; set; } = "24170d";

        /// <summary>The line drawn under the reminder and between chapters, from its left edge across its width.</summary>
        public string Separator { get; set; } = "separator";
        public float SeparatorX { get; set; }
        public float SeparatorWidth { get; set; }

        /// <summary>How far the reminder's letters lean: the rules face has no italic of its own.</summary>
        public float ReminderSkew { get; set; } = -0.2f;
    }

    /// <summary>A band taken out of the template: where it starts, and how long it is, in template units.</summary>
    public sealed class Cut
    {
        public float At { get; set; }
        public float Length { get; set; }
    }

    /// <summary>A stretch of the template along one direction, from where to where, in template units.</summary>
    public sealed class Span
    {
        public float From { get; set; }
        public float To { get; set; }

        /// <summary>What a template <paramref name="length"/> long keeps with <paramref name="cuts"/> taken out of it.</summary>
        public static List<Span> Between(IEnumerable<Cut> cuts, float length)
        {
            List<Span> spans = [];
            float start = 0f;
            foreach (Cut cut in cuts.OrderBy(cut => cut.At))
            {
                spans.Add(new Span { From = start, To = cut.At });
                start = cut.At + cut.Length;
            }

            spans.Add(new Span { From = start, To = length });
            return spans;
        }
    }

    public sealed class FrameBlend
    {
        public float Start { get; set; } = 0.4f;
        public float End { get; set; } = 0.6f;
        public List<string> ColorOrder { get; set; } = [];
        public Dictionary<string, string> CardFrames { get; set; } = [];
        public Dictionary<string, string> LandFrames { get; set; } = [];

        public int OrderOf(string color)
        {
            int position = ColorOrder.FindIndex(candidate => candidate.Equals(color, StringComparison.OrdinalIgnoreCase));
            return position < 0 ? int.MaxValue : position;
        }

        public bool TryFrame(string color, bool land, out string frame) =>
            (land ? LandFrames : CardFrames).TryGetValue(color, out frame!);
    }
}

/// <summary>Generated rather than reflected, so trimming a release build cannot strip the setters it fills in.</summary>
[JsonSourceGenerationOptions(PropertyNameCaseInsensitive = true)]
[JsonSerializable(typeof(CardTemplate))]
internal sealed partial class TemplateJson : JsonSerializerContext;
