using Cantrip.Protocol;
using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// Makes cards travel from where they were to where they are, over everything else on the page.
/// <para>
/// The table is laid out afresh from the server's view on every change, so there is nothing on screen to move in place.
/// What carries over is the handle: the server keeps an object's handle for as long as this player can see it, through
/// the hand, the stack and the battlefield. So this remembers where each handle was last drawn, and when it is laid out
/// somewhere else it hides the card there and flies a copy of it from the old place to the new, showing the card as the
/// copy lands.
/// </para>
/// <para>
/// A card that was not on screen before - an opponent's spell out of their hand, a card drawn - starts from the pile or
/// fan of the zone the move event says it came from; a card leaving for somewhere not drawn as cards flies to that
/// zone's pile and fades there.
/// </para>
/// </summary>
public sealed class CardFlights : SKCanvasElement
{
    /// <summary>How long a flight takes.</summary>
    private const double Duration = 0.34;

    /// <summary>How long a place is remembered for a card no longer on screen.</summary>
    private static readonly TimeSpan Memory = TimeSpan.FromSeconds(30);

    private const float AspectRatio = 88f / 63f;

    /// <summary>A card's look on the page: its centre, its width, and how far it is turned, in degrees.</summary>
    public readonly record struct Pose(float X, float Y, float Width, float Angle);

    /// <summary>A card laid out now: where, on which row, and at what place on that row.</summary>
    public readonly record struct Placed(ViewHandle Handle, ObjectView Card, Pose Pose, CardBoardElement Board, float LocalX, float LocalY);

    private sealed record Sighting(Pose Pose, ObjectView Card, CardBoardElement Board, float LocalX, float LocalY, DateTime At);

    private sealed class Flight
    {
        public required Pose From;
        public required Pose To;
        public required SKPicture Face;
        public required bool Leaving;
        /// <summary>Where it is drawn among the other cards in the air: as its row draws it, over what is under it there.</summary>
        public required long Order;
        /// <summary>Where a card gathered under one being dragged is going now: the pointer moves it as it flies.</summary>
        public Func<Pose>? Following;
        /// <summary>When it set off: the first frame it is drawn in, so the page's work to launch it is not the flight's time.</summary>
        public DateTime? Started;
        public Pose Current;
    }

    private readonly Dictionary<ViewHandle, Sighting> _seen = [];
    private readonly Dictionary<ViewHandle, (ZoneKindView Zone, int Owner, DateTime At)> _arrivals = [];
    private readonly Dictionary<ViewHandle, (ZoneKindView Zone, int Owner)> _departures = [];
    private readonly Dictionary<ViewHandle, Flight> _flights = [];

    /// <summary>Cards let go somewhere by hand, which fly from there to wherever they are laid out next.</summary>
    private readonly Dictionary<ViewHandle, Pose> _tossed = [];

    /// <summary>The tossed cards that fly over all the others: the one dragged, over the cards carried with it.</summary>
    private readonly HashSet<ViewHandle> _tossedOnTop = [];
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromMilliseconds(8) };
    private DateTime _settleUntil;

    public CardFlights()
    {
        IsHitTestVisible = false;
        _timer.Tick += (_, _) => Invalidate();
    }

    /// <summary>Whether cards fly at all (the Animate cards setting). Turning it off lands everything at once.</summary>
    public bool Enabled
    {
        get => _enabled;
        set
        {
            _enabled = value;
            if (!value)
            {
                foreach (ViewHandle handle in _flights.Keys.ToList())
                {
                    Land(handle);
                }

                _arrivals.Clear();
                _departures.Clear();
            }
        }
    }

    private bool _enabled = true;

    /// <summary>The cards in the air, which their rows do not draw until they land.</summary>
    public IReadOnlySet<ViewHandle> Flying => _flying;

    private readonly HashSet<ViewHandle> _flying = [];

    /// <summary>Raised when a card takes off or lands, so the rows hide or show it.</summary>
    public event Action? FlyingChanged;

    /// <summary>
    /// Puts cards laid out over the next moment where they are, without flying them: for a change of layout rather than
    /// of the game - the window resized, the hand's strip dragged, the card size changed.
    /// </summary>
    public void Settle() => _settleUntil = DateTime.UtcNow.AddSeconds(0.25);

    /// <summary>
    /// Notes that a card was let go at a place on the page - dragged and dropped - so that the next look flies it from
    /// there to wherever it is laid out, even along the row it came from.
    /// </summary>
    public void Toss(ViewHandle handle, Pose from, bool onTop = false)
    {
        if (!_enabled)
        {
            return;
        }

        // A card still being gathered under the one dragged sets off from where it has got to.
        _tossed[handle] = _flights.TryGetValue(handle, out Flight? gathering) && gathering.Following is not null ? gathering.Current : from;
        if (onTop)
        {
            _tossedOnTop.Add(handle);
        }
    }

    /// <summary>
    /// Flies a card from where it is to a place that moves - under the card being dragged, carried along with it - and
    /// lets the copy go when it gets there, where the dragged copy draws it from then on.
    /// </summary>
    public void Gather(ViewHandle handle, Pose from, Func<Pose> to, ObjectView card)
    {
        if (!_enabled)
        {
            return;
        }

        Land(handle);
        Launch(handle, from, to(), card, leaving: false, order: 0);
        _flights[handle].Following = to;
        Refresh();
    }

    /// <summary>
    /// Flies a card from one place on the page to another that does not draw it - a sideboard tucked away - and fades it
    /// there, as a card leaving for a pile does.
    /// </summary>
    public void Leave(ViewHandle handle, Pose from, Pose to, ObjectView card)
    {
        if (!_enabled)
        {
            return;
        }

        _tossed.Remove(handle);
        Land(handle);
        Launch(handle, from, to, card, leaving: true, int.MaxValue + ++_leavings);
        Refresh();
    }

    /// <summary>Notes a move the server reported, for the cards arriving from, or leaving for, a zone not drawn as cards.</summary>
    public void Moved(ObjectMovedEvent moved)
    {
        // An opponent's draw names no card, and there is nothing on screen to move for it.
        if (!_enabled || moved.Object.IsNone)
        {
            return;
        }

        if (_seen.ContainsKey(moved.Object))
        {
            _departures[moved.Object] = (moved.To, moved.ToOwner);
        }
        else
        {
            // The first zone it came from is the one it was hidden in; a later move in the same batch is from somewhere
            // it was never drawn.
            _arrivals.TryAdd(moved.Object, (moved.From, moved.FromOwner, DateTime.UtcNow));
        }
    }

    /// <summary>
    /// Compares the cards laid out now with where they were, and sets off whatever the game moved.
    /// <paramref name="anchor"/> says where a zone not drawn as cards sits on the page.
    /// </summary>
    public void Look(IReadOnlyList<Placed> cards, Func<ZoneKindView, int, Pose?> anchor)
    {
        DateTime now = DateTime.UtcNow;
        bool settling = now < _settleUntil;
        bool changed = false;

        // A handle drawn twice at once has no single place it is, and is left out.
        // The cards come in the order their rows draw them, which is the order their copies are drawn in the air: a card
        // tucked under a permanent stays under it while the two fly along the row together.
        Dictionary<ViewHandle, Placed?> onScreen = [];
        Dictionary<ViewHandle, long> order = [];
        foreach (Placed card in cards)
        {
            onScreen[card.Handle] = onScreen.ContainsKey(card.Handle) ? null : card;
            order.TryAdd(card.Handle, order.Count);
        }

        List<(ViewHandle Handle, Pose From, Pose To, ObjectView Card)> launching = [];
        List<(ViewHandle Handle, Pose From, Pose To, ObjectView Card)> tossing = [];

        foreach ((ViewHandle handle, Placed? shown) in onScreen)
        {
            bool arrived = _arrivals.Remove(handle, out var arrival);
            _departures.Remove(handle);

            if (shown is not { } card)
            {
                changed |= Land(handle);
                continue;
            }

            if (_tossed.Remove(handle, out Pose tossed))
            {
                Land(handle);
                if (Differs(tossed, card.Pose))
                {
                    tossing.Add((handle, tossed, card.Pose, card.Card));
                }
            }
            else if (_flights.TryGetValue(handle, out Flight? flight))
            {
                flight.Order = order[handle];

                // Laid out again while in the air: somewhere new sets off again from where the copy is now.
                if (Differs(flight.To, card.Pose))
                {
                    flight.From = flight.Current;
                    flight.To = card.Pose;
                    flight.Started = null;
                }
            }
            else if (_enabled && !settling)
            {
                Pose? from = _seen.TryGetValue(handle, out Sighting? before) && Moved(before, card)
                    ? before.Pose
                    : before is null && arrived && anchor(arrival.Zone, arrival.Owner) is { } pile
                        ? pile
                        : null;

                if (from is { } start && Differs(start, card.Pose))
                {
                    launching.Add((handle, start, card.Pose, card.Card));
                }
            }

            _seen[handle] = new Sighting(card.Pose, card.Card, card.Board, card.LocalX, card.LocalY, now);
        }

        // Cards let go by hand all fly, however many were carried together; the one dragged flies over the others.
        foreach ((ViewHandle handle, Pose from, Pose to, ObjectView card) in tossing)
        {
            Launch(handle, from, to, card, leaving: false, order[handle] + (_tossedOnTop.Contains(handle) ? int.MaxValue : 0));
            changed = true;
        }

        if (launching.Count <= MostAtOnce)
        {
            foreach ((ViewHandle handle, Pose from, Pose to, ObjectView card) in launching)
            {
                Launch(handle, from, to, card, leaving: false, order[handle]);
                changed = true;
            }
        }

        // The cards that left for a zone drawn only as a pile or a fan go to that pile or fan, and fade there.
        foreach ((ViewHandle handle, (ZoneKindView zone, int owner)) in _departures)
        {
            if (_seen.Remove(handle, out Sighting? last) && _enabled && !settling && anchor(zone, owner) is { } pile)
            {
                Pose from = _flights.TryGetValue(handle, out Flight? flying) ? flying.Current : last.Pose;
                Land(handle);
                // Over everything laid out, and each after the last that left.
                Launch(handle, from, pile, last.Card, leaving: true, int.MaxValue + ++_leavings);
                changed = true;
            }
        }

        _departures.Clear();
        _tossed.Clear();
        _tossedOnTop.Clear();
        Forget(now);

        if (changed)
        {
            Refresh();
        }
    }

    /// <summary>
    /// Whether the game moved a card, rather than the page moving it: onto another row, or along a row that does not
    /// slide its cards itself. The hand and the stack slide theirs as the pointer moves along them, which is not a
    /// flight; and a card on the same row is compared where it stands on the row, so a scrolled board is not a move.
    /// </summary>
    private static bool Moved(Sighting before, Placed now) =>
        !ReferenceEquals(before.Board, now.Board)
        || (now.Board.FliesAlong && !now.Board.Slides
            && (MathF.Abs(before.LocalX - now.LocalX) > 2f || MathF.Abs(before.LocalY - now.LocalY) > 2f));

    /// <summary>
    /// The most cards that set off in one look. More than this at once is the page laid out again rather than the game
    /// playing a card - a window of hundreds re-sorted - and a copy drawn for each would hold the page up to fly them.
    /// </summary>
    private const int MostAtOnce = 12;

    private static bool Differs(Pose a, Pose b) =>
        MathF.Abs(a.X - b.X) > 2f || MathF.Abs(a.Y - b.Y) > 2f || MathF.Abs(a.Width - b.Width) > 2f;

    /// <summary>How many cards have left for a pile, for drawing each over the ones that left before it.</summary>
    private long _leavings;

    /// <summary>How many cards have set off since the page opened, for a test to see that a move flew (?test).</summary>
    public int Launched { get; private set; }

    /// <summary>Where each card last set off from, for a test to see where a move flew from (?test).</summary>
    public IReadOnlyDictionary<ViewHandle, Pose> FlewFrom => _flewFrom;

    private readonly Dictionary<ViewHandle, Pose> _flewFrom = [];

    private void Launch(ViewHandle handle, Pose from, Pose to, ObjectView card, bool leaving, long order)
    {
        Launched++;
        _flewFrom[handle] = from;
        // Drawn at the size it ends at, upright, and scaled and turned as it flies.
        float width = MathF.Max(to.Width, from.Width);
        _flights[handle] = new Flight
        {
            From = from,
            To = to,
            Current = from,
            Face = Record(card, width),
            Leaving = leaving,
            Order = order,
        };

        if (!leaving)
        {
            _flying.Add(handle);
        }

        _timer.Start();
    }

    /// <summary>
    /// Says that an object the server has just made is one already shown under a handle of the page's own: a triggered
    /// ability put in order, drawn on the stack as it was picked, whose real ability arrives once the order is sent. The
    /// real one carries on from wherever its stand-in was, mid-flight or at rest.
    /// </summary>
    public void Alias(ViewHandle from, ViewHandle to)
    {
        if (!_seen.Remove(from, out Sighting? last))
        {
            return;
        }

        Pose pose = _flights.TryGetValue(from, out Flight? flying) ? flying.Current : last.Pose;
        if (Land(from))
        {
            FlyingChanged?.Invoke();
        }

        _seen[to] = last with { Pose = pose, At = DateTime.UtcNow };
    }

    private bool Land(ViewHandle handle)
    {
        if (!_flights.Remove(handle, out Flight? flight))
        {
            return false;
        }

        flight.Face.Dispose();
        _flying.Remove(handle);
        return true;
    }

    private void Forget(DateTime now)
    {
        foreach ((ViewHandle handle, Sighting sighting) in _seen.ToList())
        {
            if (now - sighting.At > Memory && !_flights.ContainsKey(handle))
            {
                _seen.Remove(handle);
            }
        }

        // An arrival that never turned up on screen is not waited for.
        foreach ((ViewHandle handle, (_, _, DateTime at)) in _arrivals.ToList())
        {
            if (now - at > TimeSpan.FromSeconds(1))
            {
                _arrivals.Remove(handle);
            }
        }
    }

    private void Refresh()
    {
        FlyingChanged?.Invoke();
        Invalidate();
    }

    /// <summary>The card's face laid out once, at the size it flies at most, to be scaled and turned as it flies.</summary>
    private static SKPicture Record(ObjectView card, float width)
    {
        float height = MathF.Round(width * AspectRatio);
        return Faces.Record(width, height, canvas => CardFace.Draw(canvas, card, width, height, dimmed: false, pixelScale: Faces.PixelScale));
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        DateTime now = DateTime.UtcNow;
        bool landed = false;
        using var fade = new SKPaint();

        // In the order the rows draw them, not the order they set off in.
        foreach ((ViewHandle handle, Flight flight) in _flights.OrderBy(pair => pair.Value.Order).ToList())
        {
            flight.Started ??= now;
            if (flight.Following is { } follow)
            {
                flight.To = follow();
            }

            float t = (float)Math.Min(1.0, (now - flight.Started.Value).TotalSeconds / Duration);
            float eased = 1f - MathF.Pow(1f - t, 3f);
            flight.Current = new Pose(
                flight.From.X + ((flight.To.X - flight.From.X) * eased),
                flight.From.Y + ((flight.To.Y - flight.From.Y) * eased),
                flight.From.Width + ((flight.To.Width - flight.From.Width) * eased),
                flight.From.Angle + ((flight.To.Angle - flight.From.Angle) * eased));

            // A card leaving the board fades as it reaches its pile, rather than sitting on top of it face up.
            fade.Color = SKColors.White.WithAlpha((byte)(255 * (flight.Leaving ? Math.Clamp((1f - t) / 0.35f, 0f, 1f) : 1f)));

            Pose pose = flight.Current;
            float height = pose.Width * AspectRatio;
            canvas.Save();
            canvas.Translate(pose.X, pose.Y);
            canvas.RotateDegrees(pose.Angle);
            canvas.Translate(-pose.Width / 2f, -height / 2f);
            SKRect laid = flight.Face.CullRect;
            canvas.Scale(pose.Width / laid.Width, height / laid.Height);

            // Faded through a layer, so only a card that is fading pays for one.
            canvas.DrawPicture(flight.Face, flight.Leaving ? fade : null);
            canvas.Restore();

            if (t >= 1f)
            {
                landed |= Land(handle);
            }
        }

        if (_flights.Count == 0)
        {
            _timer.Stop();
        }

        if (landed)
        {
            // The rows draw the landed cards, and this layer draws itself once more without them.
            FlyingChanged?.Invoke();
            DispatcherQueue.TryEnqueue(Invalidate);
        }
    }
}
