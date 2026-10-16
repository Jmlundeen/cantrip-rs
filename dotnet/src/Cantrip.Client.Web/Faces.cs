using Cantrip.Core;
using Cantrip.Protocol;
using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// How card faces are drawn, named and paced, for every row on the page.
/// <para>
/// A face is recorded, not rasterized: <see cref="CardFace.Draw"/> is played into an <see cref="SKPicture"/>, which keeps
/// the drawing's operations - the art, the frame, each run of text where the layout put it - rather than its pixels. The
/// layout, which is what costs, is done once per look and size; the pixels are the GPU's, drawn afresh each frame at the
/// size and angle the card is shown at. So a face is sharp at any size, tapped or held up, and there is no copy of each
/// face in memory to make or keep: a picture is a few kilobytes, and shares its art and frames with every other.
/// </para>
/// <para>
/// A face is never laid out smaller than <see cref="MinWidth"/>: a card shown narrower than that - in the hand, on the
/// battlefield, on the stack, in a pile, in a library search - is that one layout scaled down. So a card has one face whatever it is shown at, and resizing a row finds it already made.
/// </para>
/// <para>
/// Laying out is on the UI thread, so it is paced: every row shares one budget per frame, spent first on faces in view
/// and then on faces a row has out of view (the rest of a library search, in its order). Cards not on any row - the rest
/// of a player's deck, a card just seen - are laid out ahead with what is left (<see cref="DeckLoadingPanel"/>).
/// </para>
/// </summary>
internal static class Faces
{
    /// <summary>The smallest width a face is laid out at: twice a card in a graveyard pile (PlayerRail's piles are 83 wide).</summary>
    public const float MinWidth = 166f;

    public static readonly float MinHeight = MathF.Round(MinWidth * 88f / 63f);

    /// <summary>
    /// How many pixels to the unit a face is meant to be seen at, for the size of art it asks for: twice its size on the
    /// page, since art at the literal size blurs once the display's pixel ratio is above one. Only the software renderer
    /// draws faces at this scale; on the GPU everything else is drawn at the size it is shown.
    /// </summary>
    public const float PixelScale = 2f;

    /// <summary>The size a face is drawn at to be shown at this size: its own, or the smallest size when it is smaller.</summary>
    public static (float Width, float Height) DrawnSize(float width, float height) =>
        width <= MinWidth ? (MinWidth, MinHeight) : (width, height);

    /// <summary>
    /// Everything a kept face shows, as text - exactly what <see cref="CardFace.Draw"/> reads, and nothing it does not, so
    /// a card that taps, untaps, changes zone or changes controller keeps its face, and every copy that looks the same
    /// shares one drawing. Power and toughness, and a planeswalker's loyalty, are left out: a kept face is drawn without
    /// them and they are drawn over it (<see cref="CardFace.DrawBodyOver"/>), so a creature pumped or a planeswalker
    /// activated keeps its face.
    /// <para>
    /// Worked out once for each object the client is sent, since a view is never changed once received: every row asks
    /// on every redraw, and two hundred tokens asking each time was a real part of a redraw's cost.
    /// </para>
    /// </summary>
    public static string LookOf(ObjectView card) => Looks.GetValue(card, WorkOutLook);

    private static readonly System.Runtime.CompilerServices.ConditionalWeakTable<ObjectView, string> Looks = new();

    private static string WorkOutLook(ObjectView card)
    {
        string? body = CardFace.BodyCounter(card);
        var look = new System.Text.StringBuilder();
        look.Append(card.Name).Append('|')
            .Append(card.Kind).Append('|')
            .Append(card.IsCopy ? 'C' : 'c')
            .Append(card.Known ? 'K' : 'k')
            .Append(card.FaceDown ? 'F' : 'f')
            .Append(card.Monstrous ? 'M' : 'm')
            .Append(card.ClassLevel)
            .Append(card.Harnessed ? 'H' : 'h')
            .Append(card.Exerted ? 'E' : 'e')
            .Append(card.Saddled ? 'D' : 'd')
            .Append(card.Goaded ? 'G' : 'g')
            .Append(card.ProducesMana ? 'P' : 'p').Append('|')
            .Append(card.FaceDownName).Append('|')
            // Only whether it has a body, which moves the damage marked and the counters, not what the body says.
            .Append(card.Power is null ? '-' : 'P').Append(card.Toughness is null ? '-' : 'T').Append('|')
            .Append(card.DamageMarked).Append('|')
            .AppendJoin(',', Marked(card, body).OrderBy(c => c.Name, StringComparer.Ordinal).Select(c => $"{c.Name}:{c.Count}")).Append('|')
            .AppendJoin(',', card.Colors.Order()).Append('|')
            .AppendJoin(',', card.Supertypes).Append('|')
            .AppendJoin(',', card.CardTypes).Append('|')
            .AppendJoin(',', card.Subtypes).Append('|')
            .Append(card.ManaCost).Append('|')
            .Append(card.OracleText).Append('|')
            .AppendJoin(',', card.Halves.Select(half => half.Name)).Append('|')
            .AppendJoin(',', card.UnlockedDoors).Append('|')
            .AppendJoin(',', card.GrantedAbilities.Select(a => a.ToString())).Append('|')
            .AppendJoin(',', card.LostAbilities).Append('|')
            .Append(card.CantBeCountered ? 'U' : 'u').Append(card.EveryCreatureType ? 'Y' : 'y').Append('|')
            .AppendJoin(',', card.TypeLineSubtypes).Append('|')
            .AppendJoin(',', card.SpentLines).Append('|')
            .AppendJoin('|', card.ChosenModes).Append('|')
            .Append(card.KnownBy.Count).Append('|')
            .AppendJoin(',', card.Choices.Select(choice => $"{choice.Key}={choice.Value}"));
        return look.ToString();
    }

    /// <summary>
    /// The counters drawn on a face: all but a planeswalker's loyalty or a battle's defense, which is its body, drawn
    /// over the face.
    /// </summary>
    private static IEnumerable<CounterView> Marked(ObjectView card, string? body) =>
        body is null ? card.Counters : card.Counters.Where(counter => counter.Name != body);

    /// <summary>
    /// The printed look of each card this client has been sent as printed, by name - its own deck, and every card it has
    /// seen (<see cref="GameClient.Printed"/>): a card that looks otherwise has been changed by the game - a cost
    /// reduced, a type added - and its face is not kept for good.
    /// </summary>
    private static readonly Dictionary<string, string> Printed = new(StringComparer.Ordinal);

    /// <summary>
    /// Notes a card as printed, and what else is printed on it - a double-faced card's other face, a prepare spell - for
    /// telling a changed one from it.
    /// </summary>
    public static void NotePrinted(ObjectView card)
    {
        if (card.Known && !card.FaceDown)
        {
            Printed[card.Name] = LookOf(card);
        }

        foreach (ObjectView other in card.AlsoPrinted())
        {
            NotePrinted(other);
        }
    }

    /// <summary>
    /// Whether a face is the card as it is, with nothing the game has done to it on it: kept for the whole session and
    /// never let go to make room, since every copy of the card shows it whenever nothing has happened to it. Anything on
    /// it - counters, damage, a status, a choice, an ability given it, faded while something is chosen, a line over its
    /// art, face down, or a look other than the one its decklist printed - makes it a face of the moment, kept while
    /// there is room. Power and toughness and loyalty are never on a kept face, so they never make it one.
    /// </summary>
    public static bool IsDurable(ObjectView card, string cardLook, bool dimmed, string? annotation) =>
        !dimmed
        && annotation is null
        && !card.FaceDown
        && card.DamageMarked == 0
        && !Marked(card, CardFace.BodyCounter(card)).Any()
        && card.Choices.Count == 0
        && card.KnownBy.Count == 0
        && card.GrantedAbilities.Count == 0
        && card.LostAbilities.Count == 0
        && !card.CantBeCountered
        && card.SpentLines.Count == 0
        && card.ChosenModes.Count == 0
        && card.ClassLevel == 0
        && !card.Monstrous && !card.Harnessed && !card.Saddled && !card.Exerted && !card.Goaded
        && (!Printed.TryGetValue(card.Name, out string? printed) || printed == cardLook);

    /// <summary>A face's look as drawn: the card's own, faded or not, with any line over its art.</summary>
    public static string Look(string cardLook, bool dimmed, string? annotation) =>
        string.Concat(dimmed ? "|d|" : "|-|", annotation ?? string.Empty, "|", cardLook);

    /// <summary>The key a face is kept under: its look, and the size it is drawn at.</summary>
    public static string Key(float drawnWidth, float drawnHeight, string look) =>
        string.Concat(
            drawnWidth.ToString(System.Globalization.CultureInfo.InvariantCulture), "x",
            drawnHeight.ToString(System.Globalization.CultureInfo.InvariantCulture), look);

    /// <summary>
    /// Lays out a face at a size (already a <see cref="DrawnSize"/>), without its body, keeps it - for good when it is
    /// <paramref name="durable"/> - and counts the time against the frame.
    /// </summary>
    public static SKPicture Draw(ObjectView card, float width, float height, bool dimmed, string? annotation, string key, string look, string slug, bool durable)
    {
        DateTime started = DateTime.UtcNow;
        SKPicture picture = Record(width, height, canvas => CardFace.Draw(canvas, card, width, height, dimmed, annotation, body: false, PixelScale));

        // Whether the art it was drawn with is all it will get at this size, or it wants drawing again when more comes.
        bool wantsArt = card.Known && !card.FaceDown
            && CardFace.ArtNames(card).Any(name => !CardArt.Complete(name, CardFace.ArtPixels(card, width, PixelScale)));

        FaceCache.Put(key, look, slug, picture, wantsArt, durable);
        Spent(DateTime.UtcNow - started);
        FacePerf.Drew(DateTime.UtcNow - started, durable);
        LastDrawn = DateTime.UtcNow;
        return picture;
    }

    /// <summary>When a face was last laid out, so a picture of the page can wait until faces have stopped being made.</summary>
    public static DateTime LastDrawn { get; private set; } = DateTime.MinValue;

    /// <summary>
    /// A drawing of <paramref name="width"/> by <paramref name="height"/> units, recorded to be played back at any size.
    /// Under the software renderer, which would rasterize every operation of every face on every frame, the recording is
    /// rasterized once at <see cref="PixelScale"/> and what is kept is that image, scaled as it is shown.
    /// </summary>
    public static SKPicture Record(float width, float height, Action<SKCanvas> draw)
    {
        var bounds = SKRect.Create(width, height);
        using var recorder = new SKPictureRecorder();
        draw(recorder.BeginRecording(bounds));
        SKPicture picture = recorder.EndRecording();

        if (OnGpu)
        {
            return picture;
        }

        using (picture)
        {
            var info = new SKImageInfo((int)MathF.Ceiling(width * PixelScale), (int)MathF.Ceiling(height * PixelScale), SKColorType.Rgba8888, SKAlphaType.Premul);
            using SKSurface surface = SKSurface.Create(info) ?? throw new InvalidOperationException("Could not create an offscreen Skia surface.");
            surface.Canvas.Clear(SKColors.Transparent);
            surface.Canvas.Scale(PixelScale);
            surface.Canvas.DrawPicture(picture);
            using SKImage image = surface.Snapshot();

            using var wrapper = new SKPictureRecorder();
            wrapper.BeginRecording(bounds).DrawImage(image, bounds, new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.Linear));
            return wrapper.EndRecording();
        }
    }

    /// <summary>
    /// Whether the page is drawn by the GPU (WebGL), rather than by Uno's software renderer, which it falls back to when
    /// the browser has no WebGL to give it. Asked each time rather than kept: before the renderer is made it cannot say.
    /// </summary>
    // Uno's reference assembly marks this not implemented, but its Skia runtime answers it: whether the compositor is
    // the software renderer.
#pragma warning disable Uno0001
    private static bool OnGpu => Microsoft.UI.Composition.CompositionCapabilities.GetForCurrentView().AreEffectsFast();
#pragma warning restore Uno0001


    // --- Pacing ------------------------------------------------------------------------------------------------------

    /// <summary>How urgently a face is wanted, which decides how much of the frame it may have.</summary>
    public enum Urgency
    {
        /// <summary>In view now: drawn while the frame has time, and always at least one a frame.</summary>
        Shown,

        /// <summary>On a row but scrolled out of view, as the rest of a library search is: a few a frame, in the row's order.</summary>
        Ahead,
    }

    /// <summary>A frame, for pacing: rows draw at different moments in one, so the time is counted over this long.</summary>
    private static readonly TimeSpan Frame = TimeSpan.FromMilliseconds(16);

    /// <summary>How long a frame may spend drawing faces in view. Past it the rest wait for the next frame behind a stand-in.</summary>
    private static readonly TimeSpan ShownBudget = TimeSpan.FromMilliseconds(8);

    /// <summary>How much of a frame drawing ahead may have, and how many faces.</summary>
    private static readonly TimeSpan AheadBudget = TimeSpan.FromMilliseconds(4);
    private const int MostAhead = 2;

    private static DateTime _frameStart = DateTime.MinValue;
    private static TimeSpan _spent;
    private static int _drawn;
    private static int _extra;

    /// <summary>Whether a face wanted this urgently may be drawn in this frame; if so, it is counted as one.</summary>
    public static bool MayDraw(Urgency urgency)
    {
        DateTime now = DateTime.UtcNow;
        if (now - _frameStart >= Frame)
        {
            _frameStart = now;
            _spent = TimeSpan.Zero;
            _drawn = 0;
            _extra = 0;
        }

        bool may = urgency switch
        {
            Urgency.Shown => _drawn == 0 || _spent < ShownBudget,
            _ => _spent < AheadBudget && _extra < MostAhead,
        };

        if (may)
        {
            _drawn++;
            if (urgency != Urgency.Shown)
            {
                _extra++;
            }
        }

        return may;
    }

    private static void Spent(TimeSpan took) => _spent += took;
}
