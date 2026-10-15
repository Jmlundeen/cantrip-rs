using Cantrip.Protocol;
using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A card held up to be read: a copy of it, over everything else on the
/// page, that grows to a readable size and stands straight upright however the card itself is turned, for as long as
/// the right button is held. Letting go sends it back to where the card is. Only the copy moves: the card itself keeps
/// its place and is merely not drawn meanwhile, so nothing on the table is laid out again.
/// <para>
/// Both faces of a double-faced card are printed on it and public (CR 712.2), so a close look shows both, side by side,
/// the face that is up on the left. A card a copy effect has made another shows the same way: what it is now on the
/// left, and the card itself as printed beside it.
/// </para>
/// <para>
/// A split card's halves are printed side by side along its length (CR 709.1), so it is held up turned a quarter turn
/// clockwise, the halves upright, and as big as each half is tall where a whole card held up is.
/// </para>
/// </summary>
public sealed class CardInspector : SKCanvasElement
{
    /// <summary>How long the copy takes to grow or go back.</summary>
    private const double Duration = 0.17;

    /// <summary>How tall a close look is, as a fraction of the window's height.</summary>
    private const float CloseHeight = 0.58f;

    /// <summary>How much bigger than at rest a close look is at the least, so it is always a magnification.</summary>
    private const float CloseLeast = 1.6f;

    /// <summary>How wide a close look may grow, as a fraction of the window's width.</summary>
    private const float CloseWidest = 0.42f;

    /// <summary>How far from the window's edges the copy is kept.</summary>
    private const float CloseMargin = 12f;

    /// <summary>How far apart the faces of a close look stand, middle to middle, in card widths.</summary>
    private const float FaceSpread = 1.06f;

    private const float AspectRatio = 88f / 63f;

    private readonly record struct Pose(float X, float Y, float Width, float Angle);

    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(8) };
    /// <summary>The faces held up, left to right: the card as it is, then its other face or the card under a copy.</summary>
    private readonly List<SKPicture> _faces = [];
    /// <summary>The card each face held up is drawn from, and the width it is drawn at, to draw one again when its art comes.</summary>
    private readonly List<ObjectView> _cards = [];
    private float _width;
    private Pose _rest;
    private Pose _from;
    private Pose _current;
    private DateTime _started;
    private bool _returning;
    private bool _conceals;
    private bool _landscape;

    public CardInspector()
    {
        IsHitTestVisible = false;
        _timer.Tick += (_, _) => Invalidate();

        // A face held up before its art arrived - the other face of a double-faced card, most often, which nothing on
        // the table has drawn - is drawn again when it comes. The event is static, so it is listened to only while on the page.
        Loaded += (_, _) =>
        {
            CardArt.Changed -= OnArtChanged;
            CardArt.Changed += OnArtChanged;
        };
        Unloaded += (_, _) => CardArt.Changed -= OnArtChanged;
    }

    private void OnArtChanged(string slug)
    {
        if (Held is null || _returning)
        {
            return;
        }

        bool redrawn = false;
        for (int i = 0; i < _cards.Count; i++)
        {
            if (CardArt.Covers(CardFace.ArtSlug(_cards[i]), slug))
            {
                _faces[i].Dispose();
                _faces[i] = Record(_cards[i], _width);
                redrawn = true;
            }
        }

        if (redrawn)
        {
            Invalidate();
        }
    }

    /// <summary>The card being looked at, whose own face is not drawn meanwhile; null while nothing is held up.</summary>
    public ViewHandle? Held { get; private set; }

    /// <summary>
    /// The card not to draw on the table while it is held up, because the copy is it, lifted off its place; null while
    /// nothing is, or when what is held up was read from its name in a line of text and the card stays where it is.
    /// </summary>
    public ViewHandle? Concealed => _conceals ? Held : null;

    /// <summary>How many faces are held up: two for a double-faced card or a copy beside its own card, none while nothing is.</summary>
    public int FacesHeld => Held is null ? 0 : _faces.Count;

    /// <summary>Raised when what is held up changes, so the boards stop or start drawing the card underneath.</summary>
    public event Action? HeldChanged;

    /// <summary>
    /// Holds a card up from where it is drawn: its box on the page, and how far it is turned. A face this player may not
    /// see has nothing to read, and is not held up.
    /// </summary>
    public void Inspect(ObjectView card, Rect rest, float angle) => Inspect(card, rest, angle, conceals: true);

    /// <summary>
    /// Holds a card up from its name in a line of text - the game log, a draft's picks: it grows out of the name, from a
    /// card no taller than the words, and goes back into it. The card itself, if it is on the table, stays drawn there.
    /// </summary>
    public void InspectName(ObjectView card, Rect name)
    {
        double height = Math.Max(name.Height, 8);
        double width = height / AspectRatio;
        double x = name.X + (name.Width / 2);
        double y = name.Y + (name.Height / 2);
        Inspect(card, new Rect(x - (width / 2), y - (height / 2), width, height), 0f, conceals: false);
    }

    private void Inspect(ObjectView card, Rect rest, float angle, bool conceals)
    {
        if (!card.Known)
        {
            return;
        }

        _landscape = card.Halves.Count > 0;
        List<ObjectView> faces = _landscape ? [card] : [card, .. card.AlsoPrinted(), .. new[] { card.CopyingCard }.OfType<ObjectView>()];

        // Its box as drawn, turned back upright: a tapped card's box is as wide as the card is tall.
        bool turned = MathF.Abs(angle % 180f) > 45f;
        float restWidth = (float)(turned ? rest.Height : rest.Width);
        _rest = new Pose((float)(rest.X + (rest.Width / 2)), (float)(rest.Y + (rest.Height / 2)), restWidth, angle);

        // Drawn at the size it ends at and scaled down to meet the card: text blown up from a small face cannot be read.
        float width = Enlarged(_rest, faces.Count).Width;
        Forget();
        _width = width;
        _cards.AddRange(faces);
        _faces.AddRange(faces.Select(face => Record(face, width)));

        _from = _rest;
        _current = _rest;
        _returning = false;
        _conceals = conceals;
        _started = DateTime.UtcNow;
        Held = card.Handle;
        HeldChanged?.Invoke();
        _timer.Start();
        Invalidate();
    }

    /// <summary>Ends a close look: the copy travels back to where the card is, and the card is drawn again as it lands.</summary>
    public void Release()
    {
        if (Held is null || _returning)
        {
            return;
        }

        _returning = true;
        _from = _current;
        _started = DateTime.UtcNow;
    }

    /// <summary>Where a card held up sits: straight upright, as big as the window comfortably takes, over the card itself.</summary>
    private Pose Enlarged(Pose rest, int faces)
    {
        float windowWidth = (float)ActualWidth;
        float windowHeight = (float)ActualHeight;

        if (_landscape)
        {
            // Turned, the card's width is how tall it stands, and each half is as tall as a whole card held up is; as
            // long as two cards held up side by side may be.
            float across = MathF.Min(
                MathF.Max(windowHeight * CloseHeight, rest.Width * CloseLeast),
                windowWidth * CloseWidest * 2f / AspectRatio);

            return new Pose(
                Onscreen(rest.X, across * AspectRatio, windowWidth),
                Onscreen(rest.Y, across, windowHeight),
                across,
                90f);
        }

        float width = MathF.Min(
            MathF.Max(windowHeight * CloseHeight / AspectRatio, rest.Width * CloseLeast),
            windowWidth * CloseWidest / faces);

        return new Pose(
            Onscreen(rest.X, width * faces, windowWidth),
            Onscreen(rest.Y, width * AspectRatio, windowHeight),
            width,
            0f);
    }

    /// <summary>Keeps the copy within one side of the window, or centres it when there is no room to.</summary>
    private static float Onscreen(float centre, float length, float across)
    {
        float middle = across / 2f;
        float room = ((across - length) / 2f) - CloseMargin;
        return room > 0f ? Math.Clamp(centre, middle - room, middle + room) : middle;
    }

    private static SKPicture Record(ObjectView card, float width)
    {
        float height = MathF.Round(width * AspectRatio);
        return Faces.Record(width, height, canvas => CardFace.Draw(canvas, card, width, height, dimmed: false, pixelScale: Faces.PixelScale));
    }

    private void Forget()
    {
        foreach (SKPicture face in _faces)
        {
            face.Dispose();
        }

        _faces.Clear();
        _cards.Clear();
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        if (Held is null || _faces.Count == 0)
        {
            return;
        }

        Pose open = Enlarged(_rest, _faces.Count);
        // With cards set not to move, a look does not grow either: it is simply there, and then gone.
        float t = CardBoardElement.Animates ? (float)Math.Min(1.0, (DateTime.UtcNow - _started).TotalSeconds / Duration) : 1f;
        float eased = 1f - MathF.Pow(1f - t, 3f);
        Pose to = _returning ? _rest : open;
        _current = new Pose(
            _from.X + ((to.X - _from.X) * eased),
            _from.Y + ((to.Y - _from.Y) * eased),
            _from.Width + ((to.Width - _from.Width) * eased),
            _from.Angle + ((to.Angle - _from.Angle) * eased));

        // Measured from how far the copy has grown, so a look let go of half way brings the faces back together.
        float spread = open.Width > _rest.Width
            ? Math.Clamp((_current.Width - _rest.Width) / (open.Width - _rest.Width), 0f, 1f)
            : 1f;

        // The card as it is drawn last, over the others while they are still gathered behind it.
        float middle = (_faces.Count - 1) / 2f;
        for (int i = _faces.Count - 1; i >= 0; i--)
        {
            Draw(canvas, _faces[i], _current, (i - middle) * FaceSpread * spread);
        }

        if (t >= 1f && _returning)
        {
            _timer.Stop();
            Held = null;
            Forget();
            HeldChanged?.Invoke();

            // The frame just drawn is the copy at rest, and it would stay on screen over the card under it until
            // something else redrew this layer.
            Invalidate();
        }
    }

    /// <summary>One face at a pose, shifted along by a number of card widths.</summary>
    private static void Draw(SKCanvas canvas, SKPicture face, Pose pose, float along)
    {
        float height = pose.Width * AspectRatio;
        SKRect laid = face.CullRect;
        canvas.Save();
        canvas.Translate(pose.X + (along * pose.Width), pose.Y);
        canvas.RotateDegrees(pose.Angle);
        canvas.Translate(-pose.Width / 2f, -height / 2f);
        canvas.Scale(pose.Width / laid.Width, height / laid.Height);
        canvas.DrawPicture(face);
        canvas.Restore();
    }
}
