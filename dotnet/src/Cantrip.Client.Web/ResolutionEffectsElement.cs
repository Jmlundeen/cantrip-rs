using Cantrip.Protocol;
using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// What an ability did as it resolved, drawn over the whole table for a moment: its source lights up - a border that
/// flares in with a light running once round it and a ring going out from it, in the manner of a picked card's - and
/// an arrow grows from it to each card or player it touched, strikes it with a flash of the same colour, and draws
/// itself in. Red for harm, green for help, gold for anything else. Takes no clicks; it is only drawn.
/// </summary>
public sealed class ResolutionEffectsElement : SKCanvasElement
{
    /// <summary>Which way an effect cut: against what it touched, for it, or neither or both.</summary>
    public enum Feeling
    {
        Neutral,
        Good,
        Bad,
    }

    /// <summary>A card, by handle, or a player, by seat: something on the table an effect can be drawn on.</summary>
    public readonly record struct Mark(ViewHandle Card, int Player)
    {
        public static Mark Of(ViewHandle card) => new(card, 0);

        public static Mark OfPlayer(int seat) => new(ViewHandle.None, seat);

        public bool IsPlayer => Player != 0;
    }

    /// <summary>
    /// Where a card or a player is on this layer: its face as it would be unturned, and how far it is turned about its
    /// centre, in degrees - a card tapping is outlined as it turns.
    /// </summary>
    public readonly record struct Spot(SKRect Face, float Angle)
    {
        public SKPoint Centre => new(Face.MidX, Face.MidY);

        /// <summary>The upright box the turned face fills, for where an arrow meets it.</summary>
        public SKRect Bounds
        {
            get
            {
                float radians = Angle * MathF.PI / 180f;
                float cos = MathF.Abs(MathF.Cos(radians));
                float sin = MathF.Abs(MathF.Sin(radians));
                float width = (cos * Face.Width) + (sin * Face.Height);
                float height = (sin * Face.Width) + (cos * Face.Height);
                return SKRect.Create(Face.MidX - (width / 2f), Face.MidY - (height / 2f), width, height);
            }
        }
    }

    /// <summary>
    /// One ability's resolution: its source, which way it cut on the whole, and an arrow to each other thing it touched.
    /// <paramref name="Origin"/> is where the arrows set off from when the source cannot be found - where the ability was
    /// last seen on the stack - and the source is not lit then.
    /// </summary>
    public sealed record Effect(Mark Source, Feeling Overall, IReadOnlyList<(Mark To, Feeling Feeling)> Arrows, Mark? Origin = null);

    private sealed class Playing
    {
        public required Effect Effect;
        public required DateTime Start;

        /// <summary>Where each thing was when last found, so one that leaves the table part way through is drawn where it was.</summary>
        public readonly Dictionary<Mark, Spot> Last = [];
    }

    /// <summary>When the source has finished flaring in and the first arrow sets off, in seconds from the start.</summary>
    private const float ArrowsSetOff = 0.16f;

    /// <summary>How much later each arrow after the first sets off, so a spray of them reads as one after another.</summary>
    private const float ArrowStagger = 0.05f;

    private const float ArrowGrows = 0.3f;
    private const float ArrowHolds = 0.26f;
    private const float ArrowDrawsIn = 0.26f;

    private const float SourceRises = 0.16f;
    private const float SourceFades = 0.36f;
    private const float RippleLasts = 0.6f;
    private const float StrikeLasts = 0.5f;

    /// <summary>How soon after one effect the next may start, so abilities resolving together are seen one by one.</summary>
    private const float Spacing = 0.42f;

    /// <summary>The most arrows one effect draws: past this, a sweep of the board says no more for being drawn whole.</summary>
    private const int MostArrows = 12;

    private static readonly SKColor Gold = SKColor.Parse("f1b44c");
    private static readonly SKColor Green = SKColor.Parse("6fcf8f");
    private static readonly SKColor Red = SKColor.Parse("e5534b");

    /// <summary>How long a target is seen struck before what the ability did to it is drawn.</summary>
    private const float StrikeReads = 0.14f;

    /// <summary>The longest the table is ever held for effects, however many arrive at once.</summary>
    private const float MostHeld = 1.4f;

    private DateTime _holdUntil = DateTime.MinValue;

    /// <summary>
    /// Whether the table should go on being drawn as it was, so the effects' arrows reach what they point at before the
    /// table moves on - before a creature they killed leaves it.
    /// </summary>
    public bool HoldingTable => _enabled && DateTime.UtcNow < _holdUntil;

    private readonly List<Playing> _playing = [];
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(8) };
    private DateTime _nextStart = DateTime.MinValue;

    public ResolutionEffectsElement()
    {
        IsHitTestVisible = false;
        _timer.Tick += (_, _) => Invalidate();
    }

    /// <summary>Where a card or a player is on this layer now, or null when it is not on the table.</summary>
    public Func<Mark, Spot?> Locate { get; set; } = _ => null;

    /// <summary>Where a card was last drawn, for one gone from the table by the time its effect is shown.</summary>
    public Func<Mark, Spot?> LastSeen { get; set; } = _ => null;

    /// <summary>Whether effects are drawn at all (the Animate cards setting). Turning it off drops any being shown.</summary>
    public bool Enabled
    {
        get => _enabled;
        set
        {
            _enabled = value;
            if (!value)
            {
                _playing.Clear();
                _holdUntil = DateTime.MinValue;
                Invalidate();
            }
        }
    }

    private bool _enabled = true;

    /// <summary>Whether an effect is being shown, for the test state's word on whether the table has settled.</summary>
    public bool Showing => _playing.Count > 0;

    /// <summary>How many effects have been shown since the page opened, for a test to see one was.</summary>
    public int Shown { get; private set; }

    /// <summary>The latest effects shown, newest last: for a test to see what was shown, and in what colour.</summary>
    public IReadOnlyList<Effect> History => _history;

    private readonly List<Effect> _history = [];

    /// <summary>Shows an effect, once the one before it has had its moment.</summary>
    public void Play(Effect effect)
    {
        if (!_enabled)
        {
            return;
        }

        DateTime now = DateTime.UtcNow;
        DateTime start = now > _nextStart ? now : _nextStart;
        _nextStart = start.AddSeconds(Spacing);

        var playing = new Playing
        {
            Effect = effect with { Arrows = [.. effect.Arrows.Take(MostArrows)] },
            Start = start,
        };

        // Where everything is as the effect is set going, before anything it did takes a card off the table.
        foreach (Mark mark in Marks(playing.Effect))
        {
            if (Find(mark) is { } box)
            {
                playing.Last[mark] = box;
            }
        }

        // The table is held as it was until the arrows have struck: what they did - a creature dying of it, a counter
        // on it - is shown after them, not under them.
        if (playing.Effect.Arrows.Count > 0)
        {
            DateTime struck = start.AddSeconds(ArrowStarts(playing.Effect.Arrows.Count - 1) + ArrowGrows + StrikeReads);
            DateTime most = now.AddSeconds(MostHeld);
            DateTime until = struck < most ? struck : most;
            _holdUntil = until > _holdUntil ? until : _holdUntil;
        }

        _playing.Add(playing);
        _history.Add(playing.Effect);
        if (_history.Count > 32)
        {
            _history.RemoveAt(0);
        }

        Shown++;
        if (!_timer.IsEnabled)
        {
            _timer.Start();
        }

        Invalidate();
    }

    public static SKColor ColourOf(Feeling feeling) => feeling switch
    {
        Feeling.Good => Green,
        Feeling.Bad => Red,
        _ => Gold,
    };

    /// <summary>How long an effect lasts with this many arrows.</summary>
    private static float Duration(int arrows) => arrows == 0
        ? SourceRises + RippleLasts + SourceFades
        : MathF.Max(SourceRises + RippleLasts + SourceFades, ArrowEnds(arrows - 1) + 0.04f);

    private static float ArrowStarts(int index) => ArrowsSetOff + (index * ArrowStagger);

    private static float ArrowEnds(int index) => ArrowStarts(index) + ArrowGrows + ArrowHolds + ArrowDrawsIn;

    private static IEnumerable<Mark> Marks(Effect effect)
    {
        yield return effect.Source;
        if (effect.Origin is { } origin)
        {
            yield return origin;
        }

        foreach ((Mark to, Feeling _) in effect.Arrows)
        {
            yield return to;
        }
    }

    private Spot? Find(Mark mark) => Locate(mark) ?? (mark.IsPlayer ? null : LastSeen(mark));

    /// <summary>Where something is now, or where it was last found while this effect was being shown.</summary>
    private Spot? Where(Playing playing, Mark mark)
    {
        if (Locate(mark) is { } now)
        {
            playing.Last[mark] = now;
            return now;
        }

        return playing.Last.TryGetValue(mark, out Spot last) ? last : null;
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        DateTime now = DateTime.UtcNow;
        _playing.RemoveAll(playing => (float)(now - playing.Start).TotalSeconds > Duration(playing.Effect.Arrows.Count));

        if (_playing.Count == 0)
        {
            _timer.Stop();
            return;
        }

        foreach (Playing playing in _playing)
        {
            float t = (float)(now - playing.Start).TotalSeconds;
            if (t < 0f)
            {
                continue;
            }

            Effect effect = playing.Effect;
            bool lit = effect.Origin is null;
            Spot? source = effect.Origin is { } origin ? Where(playing, origin) : Where(playing, effect.Source);

            if (lit && source is { } box)
            {
                DrawSource(canvas, box, ColourOf(effect.Overall), t, effect.Arrows.Count);
            }

            if (source is not { } from)
            {
                continue;
            }

            for (int i = 0; i < effect.Arrows.Count; i++)
            {
                (Mark to, Feeling feeling) = effect.Arrows[i];

                // A card the strike took off the table is not struck again where it was: the arrow only draws in.
                bool there = Locate(to) is not null;
                if (Where(playing, to) is { } target)
                {
                    DrawStrike(canvas, from, target, to.IsPlayer, there, ColourOf(feeling), t - ArrowStarts(i));
                }
            }
        }
    }

    /// <summary>
    /// The source's light: a border that flares in and holds while its arrows are out, a glow round it, a light running
    /// once round the border as it flares, and a ring going out from it.
    /// </summary>
    private static void DrawSource(SKCanvas canvas, Spot spot, SKColor colour, float t, int arrows)
    {
        canvas.Save();
        canvas.Translate(spot.Face.MidX, spot.Face.MidY);
        canvas.RotateDegrees(spot.Angle);
        DrawSource(canvas, SKRect.Create(-spot.Face.Width / 2f, -spot.Face.Height / 2f, spot.Face.Width, spot.Face.Height), colour, t, arrows);
        canvas.Restore();
    }

    private static void DrawSource(SKCanvas canvas, SKRect face, SKColor colour, float t, int arrows)
    {
        float holdUntil = arrows == 0 ? SourceRises + RippleLasts : MathF.Max(SourceRises + RippleLasts, ArrowStarts(arrows - 1) + ArrowGrows + ArrowHolds);
        float envelope = t < SourceRises
            ? EaseOut(t / SourceRises)
            : t < holdUntil ? 1f : 1f - EaseIn(Math.Clamp((t - holdUntil) / SourceFades, 0f, 1f));

        if (envelope <= 0f)
        {
            return;
        }

        float width = MathF.Min(face.Width, face.Height);
        float border = MathF.Max(3f, MathF.Round(width * 0.045f));
        float corner = MathF.Max(3f, width * 0.05f);

        // The glow, outside the face: rings fading outwards, flaring further as it comes in and settling back.
        float flare = t < SourceRises ? 1f : MathF.Max(0f, 1f - ((t - SourceRises) / 0.3f));
        float reach = 3f + (4f * flare);
        const int rings = 4;
        using var ring = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = 2f, IsAntialias = true };
        for (int i = rings; i >= 1; i--)
        {
            float grow = reach * i / rings;
            float alpha = (0.14f + (0.2f * flare)) * (1f - ((i - 1f) / rings)) * envelope;
            ring.Color = colour.WithAlpha(Byte(alpha));
            canvas.DrawRoundRect(SKRect.Inflate(face, grow, grow), corner + grow, corner + grow, ring);
        }

        SKRect path = SKRect.Inflate(face, -border / 2f, -border / 2f);
        using var edge = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = border, Color = colour.WithAlpha(Byte(envelope)), IsAntialias = true };
        canvas.DrawRoundRect(path, corner, corner, edge);

        // The ring going out: thin, quick, gone by the time the arrows have reached what they point at.
        float ripple = t / RippleLasts;
        if (ripple is > 0f and < 1f)
        {
            float spread = 2f + (16f * EaseOut(ripple));
            using var wave = new SKPaint
            {
                Style = SKPaintStyle.Stroke,
                StrokeWidth = 2.5f - (1.5f * ripple),
                Color = colour.WithAlpha(Byte(0.7f * (1f - ripple) * (1f - ripple))),
                IsAntialias = true,
            };
            canvas.DrawRoundRect(SKRect.Inflate(face, spread, spread), corner + spread, corner + spread, wave);
        }

        // The light: a bright stretch of the border with a fading tail, once round, quickest at the start.
        const float lap = 0.62f;
        if (t < lap)
        {
            float perimeter = 2f * (path.Width + path.Height);
            float head = EaseOut(t / lap) * perimeter * 1.05f;
            float length = perimeter * 0.26f;
            float fade = 1f - EaseIn(t / lap);
            SKColor bright = Lighter(colour);
            const int points = 48;

            using var light = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = MathF.Max(2f, border - 1f), StrokeCap = SKStrokeCap.Round, IsAntialias = true };
            SKPoint previous = PointOn(path, head - length, perimeter);
            for (int i = 1; i < points; i++)
            {
                float along = i / (points - 1f);
                SKPoint next = PointOn(path, head - (length * (1f - along)), perimeter);
                light.Color = bright.WithAlpha(Byte(along * along * 0.95f * fade));
                canvas.DrawLine(previous, next, light);
                previous = next;
            }
        }
    }

    /// <summary>
    /// One arrow, <paramref name="t"/> seconds after it set off: growing from the source's edge, striking its target with a
    /// flash and a ring, holding, then drawing itself into the target.
    /// </summary>
    private static void DrawStrike(SKCanvas canvas, Spot source, Spot target, bool player, bool there, SKColor colour, float t)
    {
        if (t <= 0f)
        {
            return;
        }

        // From the source's middle, as an arrow from the stack is: its tail starts faint enough not to hide the card.
        SKPoint from = source.Centre;
        SKPoint to = TargetArrowsElement.Edge(SKRect.Inflate(target.Bounds, 2f, 2f), from);

        float grown = EaseOut(Math.Clamp(t / ArrowGrows, 0f, 1f));
        float drawnIn = EaseIn(Math.Clamp((t - ArrowGrows - ArrowHolds) / ArrowDrawsIn, 0f, 1f));
        TargetArrowsElement.DrawArrow(canvas, from, to, colour, drawnIn, grown, 1f - (0.35f * drawnIn));

        float struck = (t - ArrowGrows) / StrikeLasts;
        if (there && struck is > 0f and < 1f)
        {
            DrawHit(canvas, target, player, colour, struck);
        }
    }

    /// <summary>What an arrow does as it lands: the target washed with its colour, an edge round it, and a ring going out.</summary>
    private static void DrawHit(SKCanvas canvas, Spot spot, bool player, SKColor colour, float struck)
    {
        canvas.Save();
        canvas.Translate(spot.Face.MidX, spot.Face.MidY);
        canvas.RotateDegrees(spot.Angle);
        DrawHit(canvas, SKRect.Create(-spot.Face.Width / 2f, -spot.Face.Height / 2f, spot.Face.Width, spot.Face.Height), player, colour, struck);
        canvas.Restore();
    }

    private static void DrawHit(SKCanvas canvas, SKRect box, bool player, SKColor colour, float struck)
    {
        float width = MathF.Min(box.Width, box.Height);
        float corner = player ? 8f : MathF.Max(3f, width * 0.05f);
        float fade = 1f - EaseIn(struck);

        using var wash = new SKPaint { Style = SKPaintStyle.Fill, Color = colour.WithAlpha(Byte(0.2f * fade)), IsAntialias = true };
        canvas.DrawRoundRect(box, corner, corner, wash);

        float border = MathF.Max(2.5f, MathF.Round(width * 0.03f));
        using var edge = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = border, Color = colour.WithAlpha(Byte(0.95f * fade)), IsAntialias = true };
        SKRect inside = SKRect.Inflate(box, -border / 2f, -border / 2f);
        canvas.DrawRoundRect(inside, corner, corner, edge);

        float spread = 1f + (12f * EaseOut(struck));
        using var wave = new SKPaint
        {
            Style = SKPaintStyle.Stroke,
            StrokeWidth = 2.5f - (1.5f * struck),
            Color = colour.WithAlpha(Byte(0.75f * (1f - struck) * (1f - struck))),
            IsAntialias = true,
        };
        canvas.DrawRoundRect(SKRect.Inflate(box, spread, spread), corner + spread, corner + spread, wave);
    }

    /// <summary>The colour three quarters of the way to white, for the light that runs round a border.</summary>
    private static SKColor Lighter(SKColor colour) => new(
        (byte)(colour.Red + ((255 - colour.Red) * 0.75f)),
        (byte)(colour.Green + ((255 - colour.Green) * 0.75f)),
        (byte)(colour.Blue + ((255 - colour.Blue) * 0.75f)));

    private static byte Byte(float alpha) => (byte)MathF.Round(Math.Clamp(alpha, 0f, 1f) * 255f);

    private static float EaseOut(float t) => 1f - ((1f - t) * (1f - t) * (1f - t));

    private static float EaseIn(float t) => t * t;

    /// <summary>The point a distance along a rectangle's edge, clockwise from its top left corner.</summary>
    private static SKPoint PointOn(SKRect box, float distance, float perimeter)
    {
        float d = ((distance % perimeter) + perimeter) % perimeter;

        if (d < box.Width)
        {
            return new SKPoint(box.Left + d, box.Top);
        }

        d -= box.Width;
        if (d < box.Height)
        {
            return new SKPoint(box.Right, box.Top + d);
        }

        d -= box.Height;
        if (d < box.Width)
        {
            return new SKPoint(box.Right - d, box.Bottom);
        }

        d -= box.Width;
        return new SKPoint(box.Left, box.Bottom - d);
    }
}
