using Cantrip.Protocol;
using Microsoft.UI.Xaml.Media;
using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A row of cards drawn by one SKCanvasElement rather than one element per card. Cards are keyed by
/// <see cref="ViewHandle"/>, so a card keeps its recorded face and any turn it is part way through across updates,
/// and a face is drawn again only when something drawn on it changed.
/// </summary>
public sealed class CardBoardElement : SKCanvasElement
{
    /// <param name="Dimmed">Faded: it cannot be picked while something is being chosen.</param>
    /// <param name="Actionable">Outlined in the ready colour: there is something to do with it.</param>
    /// <param name="Selected">Picked for the answer being built: the gold border that breathes, with a light running round it.</param>
    /// <param name="Clickable">A click does something, so the pointer over it draws the hover line.</param>
    /// <param name="Annotation">One line over the art, as a card put back by Brainstorm says where it is going.</param>
    public readonly record struct BoardCell(
        ObjectView Card,
        float Left,
        float Top,
        bool Tapped,
        bool Actionable = false,
        bool Dimmed = false,
        bool Selected = false,
        bool Clickable = false,
        string? Annotation = null,
        int? DamageShare = null,
        bool LowerLive = false,
        bool RaiseLive = false,
        bool ShareCounters = false);

    private sealed class CellState
    {
        public required ObjectView Card;
        public required string Slug;
        public bool Dimmed;
        public bool Actionable;
        public bool Selected;
        public bool Clickable;
        public string? Annotation;
        public int? DamageShare;
        public bool LowerLive;
        public bool RaiseLive;
        public bool ShareCounters;
        public float Left;
        public float Top;
        public float TargetLeft;
        public float TargetTop;
        public float Angle;
        public float FromAngle;
        public float TargetAngle;
        public DateTime AnimStart;
        public bool Animating;
        /// <summary>Everything drawn on the face, as text: with the size, the key its drawing is shared under.</summary>
        public string FaceKey = string.Empty;
        /// <summary>The key of the face this cell last drew, to go on showing while its next face is made.</summary>
        public string? DrawnKey;
        /// <summary>
        /// The look and key its face is drawn under, and what they were worked out from: made again only when one of those
        /// changes, not on every frame for every card in view.
        /// </summary>
        public (string Look, string Key) Keyed;
        public (string FaceKey, bool Dimmed, string? Annotation, float Width, float Height) KeyedFrom;
    }

    /// <summary>
    /// The face last drawn for each card on any row, with the card's slug: a card arriving on a row - cast to the stack,
    /// played from the hand - shows the face it had where it came from until its face here is drawn.
    /// </summary>
    private static readonly Dictionary<ViewHandle, (string Key, string Slug)> LastDrawn = [];

    /// <summary>A card was clicked - the handle is a live board cell, not necessarily a legal action right now.</summary>
    public event Action<ViewHandle>? CardClicked;

    /// <summary>
    /// The right button went down on a card, to hold it up and read it: the card, its box in this element, and how far
    /// it is turned. Raised for any card, clickable or not, since any face may be read.
    /// </summary>
    public event Action<ViewHandle, Rect, float>? CardInspected;

    /// <summary>
    /// Where the press that raised the last click landed, in this element, and the modifiers held for it: the click is
    /// raised before the page hears of the press, so a menu opened by it asks here rather than there.
    /// </summary>
    public Point LastPress { get; private set; }

    public Windows.System.VirtualKeyModifiers LastModifiers { get; private set; }

    /// <summary>A card held up to be read elsewhere, which is not drawn here meanwhile; null while none is.</summary>
    public ViewHandle? Concealed
    {
        get => _concealed;
        set
        {
            if (_concealed != value)
            {
                _concealed = value;
                Invalidate();
            }
        }
    }

    private ViewHandle? _concealed;

    /// <summary>A step beside a card's damage share was pressed: up, or down. A dead step takes the press all the same.</summary>
    public event Action<ViewHandle, bool>? DamageStepped;

    /// <summary>Where the pointer is on the card under it, in that card's own unturned frame, for lighting a step.</summary>
    private SKPoint? _pointerOnCard;

    /// <summary>
    /// Whether this row's cards can be picked up and put down elsewhere. A click on one is then a
    /// click only once the button comes up without the card having been carried off: acting on the press would play a
    /// card from the hand that the player meant to move along it.
    /// </summary>
    public bool Draggable { get; set; }

    /// <summary>
    /// A card was picked up, with where it was grabbed on it, in its own unturned frame. <see cref="Carrying"/> follows
    /// the pointer, in the window's coordinates, until <see cref="Dropped"/> says where it was let go, or null if the
    /// drag was lost.
    /// </summary>
    public event Action<ViewHandle, Point>? PickedUp;

    public event Action<ViewHandle, Point>? Carrying;

    public event Action<ViewHandle, Point?>? Dropped;

    /// <summary>A card carried off, faded where it was until it is put down, so the row shows where it came from.</summary>
    public ViewHandle? Carried
    {
        get => _carried;
        set
        {
            if (_carried != value)
            {
                _carried = value;
                Invalidate();
            }
        }
    }

    private ViewHandle? _carried;

    /// <summary>
    /// Whether a card carried off is not drawn at all where it was, rather than faded there: for a layout where the place
    /// it left says nothing worth showing, such as a column of a pool, while the card itself rides under the pointer.
    /// </summary>
    public bool HidesCarried { get; init; }

    /// <summary>Where a card held over this row would be put down, as the x of the line drawn there; null while none is.</summary>
    public float? DropMarker
    {
        get => _dropMarker;
        set
        {
            if (_dropMarker != value)
            {
                _dropMarker = value;
                Invalidate();
            }
        }
    }

    private float? _dropMarker;

    /// <summary>
    /// Where a card held over this element would be put down, as a bar drawn there: for a layout that is not one row,
    /// such as a pool in columns, where the place is a slot in a column rather than an edge along a row. Null while none is.
    /// </summary>
    public Rect? DropBar
    {
        get => _dropBar;
        set
        {
            if (_dropBar != value)
            {
                _dropBar = value;
                Invalidate();
            }
        }
    }

    private Rect? _dropBar;

    /// <summary>The press a pick-up may come of: which card, and where in this element.</summary>
    private (ViewHandle Handle, Point At)? _press;
    private bool _carrying;

    /// <summary>How far the pointer moves with the button down before the card is picked up rather than clicked.</summary>
    private const double PickUpDistance = 5;

    /// <summary>
    /// Where a card dropped at a point along this row goes, counted among the row's other cards: before the first card
    /// whose showing part the point is short of the middle of.
    /// </summary>
    public int DropIndex(ViewHandle carried, Point at)
    {
        List<CellState> others = Others(carried);
        for (int i = 0; i < others.Count; i++)
        {
            float start = others[i].TargetLeft;
            float end = i + 1 < others.Count ? MathF.Max(others[i + 1].TargetLeft, start + 10f) : start + _cellWidth;
            if (at.X < (start + end) / 2f)
            {
                return i;
            }
        }

        return others.Count;
    }

    /// <summary>The edge a card put down at an index among the others would be put against.</summary>
    public float MarkerAt(ViewHandle carried, int index)
    {
        List<CellState> others = Others(carried);
        return others.Count == 0 ? 0f : index < others.Count ? others[index].TargetLeft : others[^1].TargetLeft + _cellWidth;
    }

    private List<CellState> Others(ViewHandle carried) =>
        [.. _drawOrder.Where(handle => handle != carried && _cells.ContainsKey(handle)).Select(handle => _cells[handle]).OrderBy(state => state.TargetLeft)];

    /// <summary>The card under the pointer changed; <see cref="Hovered"/> says which, if any.</summary>
    public event Action? HoverChanged;

    /// <summary>The card under the pointer, or null.</summary>
    public ViewHandle? Hovered { get; private set; }

    /// <summary>Where the pointer last was over this element, or null once it has left.</summary>
    private Point? _pointerAt;

    /// <summary>Whether the pointer is on the eye of the card under it: the mark that an opponent knows that card.</summary>
    public bool OverEye() =>
        Hovered is { } handle
        && _pointerAt is { } at
        && _cells.TryGetValue(handle, out CellState? state)
        && CardFace.EyeBox(state.Card, _cellWidth, _cellHeight) is { } eye
        && eye.Contains(OnCard(state, at));

    private readonly Dictionary<ViewHandle, CellState> _cells = [];
    private List<ViewHandle> _drawOrder = [];
    private float _cellWidth;
    private float _cellHeight;
    private readonly DispatcherTimer _timer;
    private int _animatingCount;

    private const double AnimationMs = 250;

    /// <summary>Whether a card turns smoothly as it taps and untaps (the Animate cards setting); at once when not.</summary>
    public static bool Animates { get; set; }

    public CardBoardElement(float cellWidth, float cellHeight)
    {
        _cellWidth = cellWidth;
        _cellHeight = cellHeight;
        Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent);

        // Runs only while a card is part way through turning (see SetCells).
        _timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(8) };
        _timer.Tick += OnAnimationTick;

        PointerPressed += OnPointerPressed;

        // A row inside a scroll viewer draws the faces scrolled into view as they arrive there.
        // Art is listened for only while the row is on the page: the event is static, and a row rebuilt on every redraw
        // - a distribute prompt's piles, a carried card - would otherwise be kept alive by it.
        Loaded += (_, _) =>
        {
            CardArt.Changed -= OnArtChanged;
            CardArt.Changed += OnArtChanged;
            _scroller = Ancestor<ScrollViewer>(this);
            if (_scroller is not null)
            {
                _scroller.ViewChanged += OnScrolled;
            }
        };
        Unloaded += (_, _) =>
        {
            CardArt.Changed -= OnArtChanged;
            if (_scroller is not null)
            {
                _scroller.ViewChanged -= OnScrolled;
                _scroller = null;
            }
        };

        PointerMoved += (_, e) =>
        {
            Point at = e.GetCurrentPoint(this).Position;
            _pointerAt = at;
            SetHovered(CardAt(at));

            // A card wearing a damage share lights the step under the pointer.
            SKPoint? onCard = Hovered is { } over && _cells.TryGetValue(over, out CellState? state) && state.DamageShare is not null
                ? OnCard(state, at)
                : null;
            if (onCard != _pointerOnCard)
            {
                _pointerOnCard = onCard;
                Invalidate();
            }
        };
        PointerExited += (_, _) =>
        {
            _pointerAt = null;
            SetHovered(null);
        };
        PointerMoved += OnPointerCarried;
        PointerReleased += OnPointerReleased;
        PointerCaptureLost += (_, _) => LetGo(null);
    }

    public float CellWidth => _cellWidth;

    public float CellHeight => _cellHeight;

    /// <summary>Changes how large the cards are drawn; every face is drawn again at the new size.</summary>
    public void SetCellSize(float width, float height)
    {
        if (width == _cellWidth && height == _cellHeight)
        {
            return;
        }

        // The faces at the new size are other drawings, found or made as they are drawn - once the size stops changing,
        // so a row being resized draws the faces it has, scaled, rather than every face at every size it passes.
        _cellWidth = width;
        _cellHeight = height;
        _resizedAt = DateTime.UtcNow;
        Invalidate();
    }

    /// <summary>Draws the row again when art arrives for a card on it; the shared face cache has let that face go.</summary>
    private void OnArtChanged(string slug)
    {
        if (_cells.Values.Any(state => CardArt.Covers(state.Slug, slug)))
        {
            Invalidate();
        }
    }

    private void OnPointerPressed(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        Microsoft.UI.Input.PointerPoint point = e.GetCurrentPoint(this);
        Point at = point.Position;
        if (CardAt(at) is not { } handle)
        {
            return;
        }

        LastPress = at;
        LastModifiers = e.KeyModifiers;

        if (point.Properties.IsRightButtonPressed)
        {
            if (Bounds(handle) is { } box)
            {
                CardInspected?.Invoke(handle, box, _cells[handle].Angle);
            }

            e.Handled = true;
            return;
        }

        // The steps beside a damage share are taken before the card's own click.
        if (_cells[handle] is { DamageShare: { } share } state)
        {
            SKPoint local = OnCard(state, at);
            (_, _, _, SKRect lower, SKRect raise) = CardFace.DamageBoxes(_cellWidth, _cellHeight, share);
            if (lower.Contains(local) || raise.Contains(local))
            {
                DamageStepped?.Invoke(handle, raise.Contains(local));
                return;
            }
        }

        if (Draggable && point.Properties.IsLeftButtonPressed)
        {
            _press = (handle, at);
            CapturePointer(e.Pointer);
            e.Handled = true;
            return;
        }

        CardClicked?.Invoke(handle);
    }

    /// <summary>Picks the pressed card up once the pointer has moved far enough with the button down, then carries it.</summary>
    private void OnPointerCarried(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (_press is not { } press)
        {
            return;
        }

        Point at = e.GetCurrentPoint(this).Position;
        if (!_carrying)
        {
            if (Math.Abs(at.X - press.At.X) < PickUpDistance && Math.Abs(at.Y - press.At.Y) < PickUpDistance)
            {
                return;
            }

            _carrying = true;
            Carried = press.Handle;
            SKPoint grabbed = _cells.TryGetValue(press.Handle, out CellState? state) ? OnCard(state, press.At) : new SKPoint(_cellWidth / 2, _cellHeight / 2);
            PickedUp?.Invoke(press.Handle, new Point(grabbed.X, grabbed.Y));
        }

        Carrying?.Invoke(press.Handle, e.GetCurrentPoint(null).Position);
    }

    private void OnPointerReleased(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (_press is not { } press)
        {
            return;
        }

        e.Handled = true;
        Point at = e.GetCurrentPoint(this).Position;
        bool carried = _carrying;
        LetGo(carried ? e.GetCurrentPoint(null).Position : null);
        ReleasePointerCapture(e.Pointer);

        // Let go without having been carried off, over the card it was pressed on: that was a click.
        if (!carried && CardAt(at) == press.Handle)
        {
            CardClicked?.Invoke(press.Handle);
        }
    }

    /// <summary>Ends a press, putting down a card being carried: where the pointer was let go, or nowhere.</summary>
    private void LetGo(Point? at)
    {
        if (_press is not { } press)
        {
            return;
        }

        _press = null;
        if (_carrying)
        {
            _carrying = false;
            Carried = null;
            Dropped?.Invoke(press.Handle, at);
        }
    }

    /// <summary>A point in this element, in a card's own unturned frame, its top left corner at the origin.</summary>
    private SKPoint OnCard(CellState state, Point p)
    {
        float dx = (float)p.X - (state.Left + (_cellWidth / 2));
        float dy = (float)p.Y - (state.Top + (_cellHeight / 2));
        float radians = -state.Angle * MathF.PI / 180;
        float localX = (dx * MathF.Cos(radians)) - (dy * MathF.Sin(radians));
        float localY = (dx * MathF.Sin(radians)) + (dy * MathF.Cos(radians));
        return new SKPoint(localX + (_cellWidth / 2), localY + (_cellHeight / 2));
    }

    private bool AnySelected => _cells.Values.Any(state => state.Selected);

    /// <summary>
    /// Forgets the card under the pointer: the pointer is somewhere else on the page, though this row never heard it
    /// leave - covered by another panel while the pointer rested on a card, say.
    /// </summary>
    public void Unhover()
    {
        _pointerAt = null;
        SetHovered(null);
    }

    private void SetHovered(ViewHandle? handle)
    {
        if (handle != Hovered)
        {
            Hovered = handle;
            HoverChanged?.Invoke();
            Invalidate();
        }
    }

    /// <summary>The topmost card at a point in this element, tested against its turned rectangle.</summary>
    public ViewHandle? CardAtPoint(Point p) => CardAt(p);

    private ViewHandle? CardAt(Point p)
    {
        float x = (float)p.X;
        float y = (float)p.Y;

        for (int i = _drawOrder.Count - 1; i >= 0; i--)
        {
            ViewHandle handle = _drawOrder[i];
            if (!_cells.TryGetValue(handle, out CellState? state))
            {
                continue;
            }

            // Into the card's own unrotated frame, about its centre, so a tapped card is hit where it is drawn.
            float dx = x - (state.Left + (_cellWidth / 2));
            float dy = y - (state.Top + (_cellHeight / 2));
            float radians = -state.Angle * MathF.PI / 180;
            float localX = (dx * MathF.Cos(radians)) - (dy * MathF.Sin(radians));
            float localY = (dx * MathF.Sin(radians)) + (dy * MathF.Cos(radians));

            if (MathF.Abs(localX) <= _cellWidth / 2 && MathF.Abs(localY) <= _cellHeight / 2)
            {
                return handle;
            }
        }

        return null;
    }

    /// <summary>Cards in the air to somewhere on this row, which it leaves to the flight until they land.</summary>
    public IReadOnlySet<ViewHandle>? Hidden
    {
        get => _hidden;
        set
        {
            _hidden = value;
            Invalidate();
        }
    }

    private IReadOnlySet<ViewHandle>? _hidden;

    /// <summary>
    /// Every card laid out here, where it is going rather than where a slide has got it to: its handle, its face, the
    /// middle of its place in this element, and how far it is turned when it gets there.
    /// </summary>
    public IEnumerable<(ViewHandle Handle, ObjectView Card, float X, float Y, float Width, float Angle)> Laid() =>
        _drawOrder.Where(_cells.ContainsKey).Select(handle =>
        {
            CellState state = _cells[handle];
            return (handle, state.Card, state.TargetLeft + (_cellWidth / 2), state.TargetTop + (_cellHeight / 2), _cellWidth, state.TargetAngle);
        });

    /// <summary>
    /// Every card drawn here now, for a test's clicks (<see cref="MainPage"/>'s ?test state): its box in this
    /// element however it is turned, whether any of it is in view, and for a card wearing a damage share where its two
    /// steps are. A card held up or in the air is not drawn here, so it is not one of them.
    /// </summary>
    public IEnumerable<(ObjectView Card, Rect Box, bool Visible, Point? Lower, Point? Raise)> Targets()
    {
        SKRect view = Viewport(new Size(ActualWidth, ActualHeight), margin: false);
        foreach (ViewHandle handle in _drawOrder)
        {
            if (!_cells.TryGetValue(handle, out CellState? state) || handle == _concealed || _hidden?.Contains(handle) == true
                || (HidesCarried && handle == _carried) || Bounds(handle) is not { } box)
            {
                continue;
            }

            Point? lower = null;
            Point? raise = null;
            if (state.DamageShare is { } share)
            {
                (_, _, _, SKRect down, SKRect up) = CardFace.DamageBoxes(_cellWidth, _cellHeight, share);
                lower = OnElement(state, down.MidX, down.MidY);
                raise = OnElement(state, up.MidX, up.MidY);
            }

            var centre = new SKPoint((float)(box.X + (box.Width / 2)), (float)(box.Y + (box.Height / 2)));
            yield return (state.Card, box, view.Contains(centre), lower, raise);
        }
    }

    /// <summary>A point in a card's own unturned frame, in this element: <see cref="OnCard"/> the other way.</summary>
    private Point OnElement(CellState state, float localX, float localY)
    {
        float dx = localX - (_cellWidth / 2);
        float dy = localY - (_cellHeight / 2);
        float radians = state.Angle * MathF.PI / 180;
        return new Point(
            state.Left + (_cellWidth / 2) + (dx * MathF.Cos(radians)) - (dy * MathF.Sin(radians)),
            state.Top + (_cellHeight / 2) + (dx * MathF.Sin(radians)) + (dy * MathF.Cos(radians)));
    }

    /// <summary>The card drawn here under a handle, or null: it may be a stand-in the game view has no object for.</summary>
    public ObjectView? CardOf(ViewHandle handle) => _cells.TryGetValue(handle, out CellState? state) ? state.Card : null;

    /// <summary>Where a card is drawn in this element, as the box around it however it is turned; null if it is not here.</summary>
    public Rect? Bounds(ViewHandle handle)
    {
        if (!_cells.TryGetValue(handle, out CellState? state))
        {
            return null;
        }

        float radians = state.Angle * MathF.PI / 180;
        float width = (MathF.Abs(MathF.Cos(radians)) * _cellWidth) + (MathF.Abs(MathF.Sin(radians)) * _cellHeight);
        float height = (MathF.Abs(MathF.Sin(radians)) * _cellWidth) + (MathF.Abs(MathF.Cos(radians)) * _cellHeight);
        float centreX = state.Left + (_cellWidth / 2);
        float centreY = state.Top + (_cellHeight / 2);

        return new Rect(centreX - (width / 2), centreY - (height / 2), width, height);
    }

    /// <summary>A card's face as drawn, before it is turned, and how far it is turned about its centre, in degrees.</summary>
    public (Rect Face, float Angle)? Placement(ViewHandle handle) =>
        _cells.TryGetValue(handle, out CellState? state) ? (new Rect(state.Left, state.Top, _cellWidth, _cellHeight), state.Angle) : null;

    /// <summary>
    /// Replaces the row's contents. A card present before keeps its face unless what is drawn on it changed, and turns
    /// smoothly from wherever it is to tapped or untapped; cards no longer present are dropped.
    /// </summary>
    public void SetCells(IReadOnlyList<BoardCell> cells)
    {
        var seen = new HashSet<ViewHandle>();

        foreach (BoardCell cell in cells)
        {
            ViewHandle handle = cell.Card.Handle;
            seen.Add(handle);
            float targetAngle = cell.Tapped ? 90 : 0;

            if (_cells.TryGetValue(handle, out CellState? state))
            {
                state.TargetLeft = cell.Left;
                state.TargetTop = cell.Top;
                if (!Slides || DateTime.UtcNow < _settleUntil)
                {
                    state.Left = cell.Left;
                    state.Top = cell.Top;
                }
                state.Actionable = cell.Actionable;
                state.Selected = cell.Selected;
                state.Clickable = cell.Clickable;
                state.DamageShare = cell.DamageShare;
                state.LowerLive = cell.LowerLive;
                state.RaiseLive = cell.RaiseLive;
                state.ShareCounters = cell.ShareCounters;

                // Always the card as it is now, for what is drawn over its face: its power and toughness, and whether
                // it can be picked. Its face is drawn again only when the look says it has changed - everything the face
                // shows is in the look, and it is worked out once per object - so a tapped, moved or pumped card keeps its face.
                state.Card = cell.Card;
                string faceKey = Faces.LookOf(cell.Card);
                if (state.Dimmed != cell.Dimmed || state.Annotation != cell.Annotation || state.FaceKey != faceKey)
                {
                    state.FaceKey = faceKey;
                    state.Dimmed = cell.Dimmed;
                    state.Annotation = cell.Annotation;
                    state.Slug = CardFace.ArtSlug(cell.Card);
                }

                if (state.TargetAngle != targetAngle && !Animates)
                {
                    state.Angle = targetAngle;
                    state.FromAngle = targetAngle;
                    state.TargetAngle = targetAngle;
                }
                else if (state.TargetAngle != targetAngle)
                {
                    state.FromAngle = state.Angle;
                    state.TargetAngle = targetAngle;
                    state.AnimStart = DateTime.UtcNow;

                    if (!state.Animating)
                    {
                        state.Animating = true;
                        _animatingCount++;
                    }
                }
            }
            else
            {
                _cells[handle] = new CellState
                {
                    Card = cell.Card,
                    FaceKey = Faces.LookOf(cell.Card),
                    Slug = CardFace.ArtSlug(cell.Card),
                    Dimmed = cell.Dimmed,
                    Actionable = cell.Actionable,
                    Selected = cell.Selected,
                    Clickable = cell.Clickable,
                    Annotation = cell.Annotation,
                    DamageShare = cell.DamageShare,
                    LowerLive = cell.LowerLive,
                    RaiseLive = cell.RaiseLive,
                    ShareCounters = cell.ShareCounters,
                    Left = cell.Left,
                    Top = cell.Top,
                    TargetLeft = cell.Left,
                    TargetTop = cell.Top,
                    Angle = targetAngle,
                    FromAngle = targetAngle,
                    TargetAngle = targetAngle,
                };
            }
        }

        foreach (ViewHandle stale in _cells.Keys.Where(h => !seen.Contains(h)).ToList())
        {
            _cells.Remove(stale);
        }

        _drawOrder = [.. cells.Select(c => c.Card.Handle)];

        if ((_animatingCount > 0 || AnySelected || AnySliding) && !_timer.IsEnabled)
        {
            _lastTick = DateTime.UtcNow;
            _timer.Start();
        }

        Invalidate();
    }

    /// <summary>
    /// Whether a card moved along the row slides there rather than jumping: the hand and the stack sliding their cards
    /// aside for the one under the pointer, and the battlefield's rows making room, closing up, and gathering a card
    /// tapped into its pile. A card new to the row still appears where it belongs, flown there if it came from another.
    /// </summary>
    public bool Slides { get; set; }

    /// <summary>Until when cards laid out are put where they belong rather than slid there (<see cref="Settle"/>).</summary>
    private DateTime _settleUntil;

    /// <summary>
    /// Puts the cards laid out over the next moment where they belong, without sliding them: for a change of layout rather
    /// than of the game - the window resized, the card size changed - as <see cref="CardFlights.Settle"/> is.
    /// </summary>
    public void Settle() => _settleUntil = DateTime.UtcNow.AddSeconds(0.25);

    /// <summary>
    /// Whether a card moved along this row by the game flies there: the battlefield's rows. A row laid out again by the
    /// page - a card browser sorted, filtered or widened, a hand or stack sliding - puts its cards in place instead.
    /// </summary>
    public bool FliesAlong { get; init; } = true;

    /// <summary>How quickly cards slide to where they belong: the rate at which the distance left shrinks.</summary>
    private const float SlideSpeed = 16f;

    private DateTime _lastTick = DateTime.UtcNow;

    private bool AnySliding => _cells.Values.Any(state => state.Left != state.TargetLeft || state.Top != state.TargetTop);

    /// <summary>Whether a card here is still sliding to its place or turning as it taps or untaps.</summary>
    public bool Moving => _animatingCount > 0 || AnySliding;

    private void OnAnimationTick(object? sender, object e)
    {
        DateTime now = DateTime.UtcNow;
        float blend = 1f - MathF.Exp(-SlideSpeed * (float)(now - _lastTick).TotalSeconds);
        _lastTick = now;

        foreach (CellState state in _cells.Values)
        {
            state.Left = Approach(state.Left, state.TargetLeft, blend);
            state.Top = Approach(state.Top, state.TargetTop, blend);
        }

        foreach (CellState state in _cells.Values)
        {
            if (!state.Animating)
            {
                continue;
            }

            double t = (DateTime.UtcNow - state.AnimStart).TotalMilliseconds / AnimationMs;
            if (t >= 1)
            {
                state.Angle = state.TargetAngle;
                state.Animating = false;
                _animatingCount--;
            }
            else
            {
                float eased = (float)(1 - ((1 - t) * (1 - t)));
                state.Angle = state.FromAngle + ((state.TargetAngle - state.FromAngle) * eased);
            }
        }

        Invalidate();

        // A picked card breathes for as long as it is picked, so the clock keeps running while one is.
        if (_animatingCount == 0 && !AnySelected && !AnySliding)
        {
            _timer.Stop();
        }
    }

    /// <summary>A step of the way from where a card is to where it belongs, arriving outright within half a pixel.</summary>
    private static float Approach(float from, float to, float blend)
    {
        float at = from + ((to - from) * blend);
        return MathF.Abs(to - at) < 0.5f ? to : at;
    }

    /// <summary>The outline of an actionable card: the ready colour, two wide, just inside the face.</summary>
    private static readonly SKPaint ActionablePaint = new()
    {
        Style = SKPaintStyle.Stroke,
        Color = SKColor.Parse("6fcf8f"),
        StrokeWidth = 2,
        IsAntialias = true,
    };

    private IReadOnlyList<BoardRows.Band> _bands = [];

    /// <summary>The combat ground drawn with the cards: under the fighters, and around them over the rest.</summary>
    public IReadOnlyList<BoardRows.Band> Bands
    {
        get => _bands;
        set
        {
            _bands = value;
            Invalidate();
        }
    }

    private static readonly SKColor BandFill = SKColor.Parse("e5534b").WithAlpha((byte)(0.13 * 255));
    private static readonly SKColor BandEdge = SKColor.Parse("e5534b").WithAlpha((byte)(0.5 * 255));
    private const float BandRadius = 8f;

    /// <summary>The ground behind the fighters, under the cards.</summary>
    private void DrawBandsUnder(SKCanvas canvas)
    {
        using var fill = new SKPaint { Color = BandFill, Style = SKPaintStyle.Fill };
        foreach (BoardRows.Band band in _bands)
        {
            var fighters = SKRect.Intersect(Fighters(band), Outer(band));
            if (!fighters.IsEmpty)
            {
                canvas.DrawRect(fighters, fill);
            }
        }
    }

    /// <summary>
    /// The ground around the fighters, over the other cards it reaches, and its outline: rounded on the side away from the
    /// line and open on the line itself, so the two halves of a zone meet flush.
    /// </summary>
    private void DrawBandsOver(SKCanvas canvas)
    {
        using var fill = new SKPaint { Color = BandFill, Style = SKPaintStyle.Fill, IsAntialias = true };
        using var edge = new SKPaint { Color = BandEdge, Style = SKPaintStyle.Stroke, StrokeWidth = 1, IsAntialias = true };

        foreach (BoardRows.Band band in _bands)
        {
            SKRect outer = Outer(band);
            using SKPath shape = RoundedAway(outer, band.OpensDownward);

            canvas.Save();
            canvas.ClipRect(SKRect.Intersect(Fighters(band), outer), SKClipOperation.Difference);
            canvas.DrawPath(shape, fill);
            canvas.Restore();

            // The outline, less its side on the line.
            SKRect inset = SKRect.Inflate(outer, -0.5f, -0.5f);
            using SKPath outline = OpenOnLine(inset, band.OpensDownward);
            canvas.DrawPath(outline, edge);
        }
    }

    private static SKRect Outer(BoardRows.Band band) => new(band.Left, band.Top, band.Right, band.Bottom);

    private static SKRect Fighters(BoardRows.Band band) => new(band.FightersLeft, band.FightersTop, band.FightersRight, band.FightersBottom);

    /// <summary>A rectangle rounded at its two corners away from the line.</summary>
    private static SKPath RoundedAway(SKRect r, bool opensDownward)
    {
        var round = new SKRoundRect();
        var corner = new SKPoint(BandRadius, BandRadius);
        SKPoint[] radii = opensDownward
            ? [corner, corner, SKPoint.Empty, SKPoint.Empty]
            : [SKPoint.Empty, SKPoint.Empty, corner, corner];
        round.SetRectRadii(r, radii);

        var path = new SKPath();
        path.AddRoundRect(round);
        return path;
    }

    /// <summary>The band's outline as three sides, the rounded far corners included, leaving the side on the line open.</summary>
    private static SKPath OpenOnLine(SKRect r, bool opensDownward)
    {
        float k = BandRadius;
        var path = new SKPath();

        if (opensDownward)
        {
            path.MoveTo(r.Left, r.Bottom);
            path.LineTo(r.Left, r.Top + k);
            path.ArcTo(new SKRect(r.Left, r.Top, r.Left + (2 * k), r.Top + (2 * k)), 180, 90, false);
            path.LineTo(r.Right - k, r.Top);
            path.ArcTo(new SKRect(r.Right - (2 * k), r.Top, r.Right, r.Top + (2 * k)), 270, 90, false);
            path.LineTo(r.Right, r.Bottom);
        }
        else
        {
            path.MoveTo(r.Left, r.Top);
            path.LineTo(r.Left, r.Bottom - k);
            path.ArcTo(new SKRect(r.Left, r.Bottom - (2 * k), r.Left + (2 * k), r.Bottom), 180, -90, false);
            path.LineTo(r.Right - k, r.Bottom);
            path.ArcTo(new SKRect(r.Right - (2 * k), r.Bottom - (2 * k), r.Right, r.Bottom), 90, -90, false);
            path.LineTo(r.Right, r.Top);
        }

        return path;
    }

    private ScrollViewer? _scroller;

    private void OnScrolled(object? sender, ScrollViewerViewChangedEventArgs e) => Invalidate();

    private static T? Ancestor<T>(DependencyObject start)
        where T : class
    {
        for (DependencyObject? at = VisualTreeHelper.GetParent(start); at is not null; at = VisualTreeHelper.GetParent(at))
        {
            if (at is T found)
            {
                return found;
            }
        }

        return null;
    }

    /// <summary>
    /// The part of this element that can be seen now: inside its scroll viewer's view, if it has one. Faces are drawn in
    /// full only here, so a library of hundreds costs what the few rows in view cost.
    /// </summary>
    private SKRect InView(Size area) => Viewport(area, margin: true);

    /// <param name="margin">Whether a card's size is added round the view, for drawing what is about to scroll into it.</param>
    private SKRect Viewport(Size area, bool margin)
    {
        var all = SKRect.Create((float)area.Width, (float)area.Height);
        if (_scroller is not { ActualWidth: > 0 } scroller || XamlRoot is null)
        {
            return all;
        }

        try
        {
            Rect view = scroller.TransformToVisual(this).TransformBounds(new Rect(0, 0, scroller.ViewportWidth, scroller.ViewportHeight));
            var seen = SKRect.Create((float)view.X, (float)view.Y, (float)view.Width, (float)view.Height);
            return SKRect.Intersect(all, margin ? SKRect.Inflate(seen, _cellWidth, _cellHeight) : seen);
        }
        catch (ArgumentException)
        {
            return all;
        }
    }

    /// <summary>How long a row's size must hold before its faces are drawn again at it.</summary>
    private static readonly TimeSpan ResizeSettle = TimeSpan.FromMilliseconds(150);

    private DateTime _resizedAt = DateTime.MinValue;

    private static readonly SKPaint PendingGround = new() { Color = SKColor.Parse("15181d"), Style = SKPaintStyle.Fill, IsAntialias = true };
    private static readonly SKPaint PendingEdge = new() { Color = SKColor.Parse("2c313a"), Style = SKPaintStyle.Stroke, StrokeWidth = 1, IsAntialias = true };

    private static readonly SKPaint CarriedPaint = new() { Color = SKColors.White.WithAlpha(89) };
    private static readonly SKPaint MarkerPaint = new() { Color = SKColor.Parse("f1b44c") };

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        var face = SKRect.Create(new SKSize(_cellWidth, _cellHeight));
        DrawBandsUnder(canvas);

        SKRect inView = InView(area);
        bool settling = DateTime.UtcNow - _resizedAt < ResizeSettle;
        bool pending = false;

        // Faces smaller than the smallest size are that one drawing, scaled: the same at every size this row passes through.
        (float drawnWidth, float drawnHeight) = Faces.DrawnSize(_cellWidth, _cellHeight);

        // The cards out of view, in the row's order, for drawing ahead once those in view have theirs.
        List<CellState>? ahead = null;

        foreach (ViewHandle handle in _drawOrder)
        {
            if (!_cells.TryGetValue(handle, out CellState? state))
            {
                continue;
            }

            // One drawing per look at one size, shared by every card that looks the same, here or on any other row.
            (string look, string key) = Keys(state, drawnWidth, drawnHeight);

            if (handle == _concealed || _hidden?.Contains(handle) == true || (HidesCarried && handle == _carried))
            {
                // Not drawn, but a card in the air to here has its face made while it flies, so it lands with it.
                if (_hidden?.Contains(handle) == true && FaceCache.Get(key) is null && Faces.MayDraw(Faces.Urgency.Shown))
                {
                    Faces.Draw(state.Card, drawnWidth, drawnHeight, state.Dimmed, state.Annotation, key, look, state.Slug, Durable(state));
                }

                continue;
            }

            // Out of view there is nothing to draw now, but its face is made ahead of it being scrolled to.
            var box = SKRect.Create(state.Left, state.Top, _cellWidth, _cellHeight);
            float reach = MathF.Max(_cellWidth, _cellHeight);
            if (!SKRect.Inflate(box, reach, reach).IntersectsWith(inView))
            {
                (ahead ??= []).Add(state);
                continue;
            }

            SKPicture? drawing = FaceCache.Get(key);
            if (drawing is null)
            {
                // Until its face is made, a card shows the nearest it has: its look at another size, what it last showed
                // here, or what it showed on the row it came from. A card never blanks for a face it already had.
                SKPicture? standIn = FaceCache.StandIn(key)
                    ?? FaceCache.AnySize(look)
                    ?? FaceCache.StandIn(state.DrawnKey)
                    ?? (LastDrawn.TryGetValue(handle, out var last) && last.Slug == state.Slug ? FaceCache.StandIn(last.Key) : null);

                if ((standIn is null || !settling) && Faces.MayDraw(Faces.Urgency.Shown))
                {
                    drawing = Faces.Draw(state.Card, drawnWidth, drawnHeight, state.Dimmed, state.Annotation, key, look, state.Slug, Durable(state));
                }
                else
                {
                    drawing = standIn;
                    pending = true;
                }
            }

            if (drawing is not null && FaceCache.Peek(key) == drawing)
            {
                state.DrawnKey = key;
                LastDrawn[handle] = (key, state.Slug);
            }

            canvas.Save();
            canvas.Translate(state.Left + (_cellWidth / 2), state.Top + (_cellHeight / 2));
            canvas.RotateDegrees(state.Angle);
            canvas.Translate(-(_cellWidth / 2), -(_cellHeight / 2));

            if (drawing is not null)
            {
                // A card carried off is faded where it was, its body with it.
                bool carried = handle == _carried;
                if (carried)
                {
                    canvas.SaveLayer(CarriedPaint);
                }

                // The face was laid out at the size it was drawn at, which a stand-in may not share with this row: it is
                // scaled to the cell by its own bounds. Kept faces are drawn without power and toughness or loyalty,
                // which change too often to keep: they are drawn over the face here, in the frame it was laid out in, as
                // it would have drawn them.
                SKRect laid = drawing.CullRect;
                canvas.Save();
                canvas.Scale(_cellWidth / laid.Width, _cellHeight / laid.Height);
                canvas.DrawPicture(drawing);
                if (CardFace.HasBody(state.Card))
                {
                    CardFace.DrawBodyOver(canvas, state.Card, laid.Width, laid.Height, state.Dimmed);
                }

                canvas.Restore();

                if (carried)
                {
                    canvas.Restore();
                }
            }
            else
            {
                float corner = MathF.Max(3f, _cellWidth * 0.05f);
                canvas.DrawRoundRect(face, corner, corner, PendingGround);
                canvas.DrawRoundRect(SKRect.Inflate(face, -0.5f, -0.5f), corner, corner, PendingEdge);
            }

            if (state.DamageShare is { } share)
            {
                (_, _, _, SKRect lower, SKRect raise) = CardFace.DamageBoxes(_cellWidth, _cellHeight, share);
                SKPoint? pointer = Hovered == handle ? _pointerOnCard : null;
                CardFace.DrawDamageShare(
                    canvas,
                    _cellWidth,
                    _cellHeight,
                    share,
                    state.LowerLive,
                    state.RaiseLive,
                    pointer is { } a && lower.Contains(a),
                    pointer is { } b && raise.Contains(b),
                    state.ShareCounters);
            }

            DrawOutline(canvas, face, state, hover: Hovered == handle && state.Clickable);
            canvas.Restore();
        }

        DrawBandsOver(canvas);

        // The line at the edge a card held over the row would be put down against.
        if (_dropMarker is { } edge)
        {
            float top = _cells.Count == 0 ? 0f : _cells.Values.Min(state => state.TargetTop);
            canvas.DrawRect(SKRect.Create(MathF.Round(edge - 1.5f), top, 3, _cellHeight), MarkerPaint);
        }

        if (_dropBar is { } bar)
        {
            canvas.DrawRect(SKRect.Create((float)bar.X, (float)bar.Y, (float)bar.Width, (float)bar.Height), MarkerPaint);
        }

        // Once every face in view is drawn, the ones out of view are drawn ahead of being scrolled to, a few a frame, in the
        // row's order - a library search fills in from its top whether or not it is scrolled.
        if (!pending && ahead is not null && DrawAhead(ahead, drawnWidth, drawnHeight))
        {
            pending = true;
        }

        // Faces still to draw are drawn on the frames after this one.
        if (pending)
        {
            DispatcherQueue.TryEnqueue(Microsoft.UI.Dispatching.DispatcherQueuePriority.Low, Invalidate);
        }

        Drawn?.Invoke(this, EventArgs.Empty);
    }

    /// <summary>Raised each time the row is drawn, for a layer over it that follows where its cards are.</summary>
    public event EventHandler? Drawn;

    /// <summary>
    /// Each creature just arrived on the row as it is drawn now: where its art box is, in the row's own coordinates and
    /// turned with its card, and the outline of every card drawn after it - piled on it, or standing over what is tucked
    /// under it - which its mark is not drawn over.
    /// </summary>
    public IEnumerable<(SKMatrix Card, SKRect Art, SKPath Over)> Arrivals()
    {
        List<CellState> drawn = [];
        foreach (ViewHandle handle in _drawOrder)
        {
            if (_cells.TryGetValue(handle, out CellState? state)
                && handle != _concealed && _hidden?.Contains(handle) != true && !(HidesCarried && handle == _carried))
            {
                drawn.Add(state);
            }
        }

        for (int i = 0; i < drawn.Count; i++)
        {
            if (CardFace.ArrivalBox(drawn[i].Card, _cellWidth, _cellHeight) is not { } art)
            {
                continue;
            }

            var over = new SKPath();
            for (int j = i + 1; j < drawn.Count; j++)
            {
                using var face = new SKPath();
                face.AddRect(SKRect.Create(_cellWidth, _cellHeight));
                face.Transform(CellMatrix(drawn[j]));
                over.AddPath(face);
            }

            yield return (CellMatrix(drawn[i]), art, over);
        }
    }

    /// <summary>From a cell's own unturned face to the row: to its place, turned about its centre, as it is drawn.</summary>
    private SKMatrix CellMatrix(CellState state) =>
        SKMatrix.CreateTranslation(state.Left + (_cellWidth / 2), state.Top + (_cellHeight / 2))
            .PreConcat(SKMatrix.CreateRotationDegrees(state.Angle))
            .PreConcat(SKMatrix.CreateTranslation(-(_cellWidth / 2), -(_cellHeight / 2)));

    /// <summary>The look and key a cell's face is drawn under at this size, worked out again only when they would differ.</summary>
    private static (string Look, string Key) Keys(CellState state, float drawnWidth, float drawnHeight)
    {
        (string, bool, string?, float, float) from = (state.FaceKey, state.Dimmed, state.Annotation, drawnWidth, drawnHeight);
        if (state.Keyed.Key is null || state.KeyedFrom != from)
        {
            string look = Faces.Look(state.FaceKey, state.Dimmed, state.Annotation);
            state.Keyed = (look, Faces.Key(drawnWidth, drawnHeight, look));
            state.KeyedFrom = from;
        }

        return state.Keyed;
    }

    /// <summary>
    /// Selection, availability and hover, over the face: a card that could be picked has a thin
    /// green line, a picked one the gold border, and the pointer over a card that answers a click a blue line inside
    /// whichever of those it has, so pointing at a picked card does not seem to unpick it.
    /// </summary>
    private void DrawOutline(SKCanvas canvas, SKRect face, CellState state, bool hover)
    {
        float inset = 1f;

        if (state.Selected)
        {
            inset = DrawSelected(canvas, face);
        }
        else if (state.Actionable && !hover)
        {
            canvas.DrawRect(SKRect.Inflate(face, -2f, -2f), ActionablePaint);
        }

        if (hover)
        {
            canvas.DrawRect(SKRect.Inflate(face, -inset - 1f, -inset - 1f), HoverPaint);
        }
    }

    /// <summary>How long the glow on a picked card takes to breathe in and out, and the light to go once round, in seconds.</summary>
    private const float PulsePeriod = 1.4f;
    private const float OrbitPeriod = 2.2f;

    /// <summary>
    /// The picked card's border, glow and running light. Timed from the clock rather than from
    /// when the card was picked, so every picked card breathes together.
    /// </summary>
    /// <returns>How far in from the face's edge the border reaches, for the hover line inside it.</returns>
    private float DrawSelected(SKCanvas canvas, SKRect face)
    {
        float seconds = Environment.TickCount64 % 1_000_000 / 1000f;
        float pulse = 0.5f + (0.5f * MathF.Sin(seconds * MathF.Tau / PulsePeriod));
        float border = MathF.Max(3f, MathF.Round(_cellWidth * 0.045f));

        // The glow, outside the face: a few rings fading outwards, reaching further and brighter as it breathes in.
        const int rings = 4;
        float reach = 2f + (4f * pulse);

        using var ring = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = 2f, IsAntialias = true };
        for (int i = rings; i >= 1; i--)
        {
            float grow = reach * i / rings;
            float alpha = (0.1f + (0.22f * pulse)) * (1f - ((i - 1f) / rings));
            ring.Color = Accent.WithAlpha((byte)(alpha * 255));
            canvas.DrawRect(SKRect.Inflate(face, grow, grow), ring);
        }

        SKRect path = SKRect.Inflate(face, -border / 2f, -border / 2f);
        using var edge = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = border, Color = Accent, IsAntialias = true };
        canvas.DrawRect(path, edge);

        // The light: a bright stretch of the border with a fading tail, going round clockwise.
        float perimeter = 2f * (path.Width + path.Height);
        float head = seconds % OrbitPeriod / OrbitPeriod * perimeter;
        float length = perimeter * 0.22f;
        const int points = 48;

        using var light = new SKPaint { Style = SKPaintStyle.Stroke, StrokeWidth = MathF.Max(2f, border - 1f), StrokeCap = SKStrokeCap.Round, IsAntialias = true };
        SKPoint previous = PointOn(path, head - length, perimeter);
        for (int i = 1; i < points; i++)
        {
            float along = i / (points - 1f);
            SKPoint next = PointOn(path, head - (length * (1f - along)), perimeter);
            light.Color = new SKColor(255, 247, 217, (byte)(along * along * 0.95f * 255));
            canvas.DrawLine(previous, next, light);
            previous = next;
        }

        return border + 1f;
    }

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

    private static readonly SKColor Accent = SKColor.Parse("f1b44c");

    private static readonly SKPaint HoverPaint = new()
    {
        Style = SKPaintStyle.Stroke,
        Color = SKColor.Parse("8ec9ff"),
        StrokeWidth = 2,
        IsAntialias = true,
    };

    /// <summary>
    /// Draws the faces of cards out of view, first first, as far as the frame allows and the face cache has room for.
    /// Nothing is let go to make that room: a face in view needs it more.
    /// </summary>
    /// <returns>Whether any are still to draw, and can be now. One waiting on its art is drawn when it comes, which
    /// draws the row again (<see cref="OnArtChanged"/>).</returns>
    private static bool DrawAhead(List<CellState> ahead, float drawnWidth, float drawnHeight)
    {
        foreach (CellState state in ahead)
        {
            int artPixels = CardFace.ArtPixels(state.Card, drawnWidth, Faces.PixelScale);
            string look = Faces.Look(state.FaceKey, state.Dimmed, state.Annotation);
            string key = Faces.Key(drawnWidth, drawnHeight, look);
            if (FaceCache.Peek(key) is not null)
            {
                continue;
            }

            // Its art first, rather than a face drawn without it and again with it.
            if (!CardFace.ArtNames(state.Card).All(name => CardArt.Settled(name, artPixels)))
            {
                foreach (string name in CardFace.ArtNames(state.Card))
                {
                    _ = CardArt.Get(name, artPixels);
                }

                continue;
            }

            bool durable = Durable(state);
            if (!FaceCache.HasRoomFor(durable))
            {
                return false;
            }

            if (!Faces.MayDraw(Faces.Urgency.Ahead))
            {
                return true;
            }

            Faces.Draw(state.Card, drawnWidth, drawnHeight, state.Dimmed, state.Annotation, key, look, state.Slug, durable);
        }

        return false;
    }

    /// <summary>Whether a card's face as this row shows it is kept for good (<see cref="Faces.IsDurable"/>).</summary>
    private static bool Durable(CellState state) => Faces.IsDurable(state.Card, state.FaceKey, state.Dimmed, state.Annotation);
}
