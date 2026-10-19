using Cantrip.Protocol;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// Cards picked up and put down elsewhere: the hand dragged into the
/// order its player wants it in, and a distribute prompt's cards dragged into, along and between its piles. A copy of
/// the card rides under the pointer where it was grabbed, the card left behind is faded, and the row it is held over
/// draws a line where it would go.
/// </summary>
public sealed partial class MainPage
{
    /// <summary>
    /// A row cards can be put down on: which it takes, and what putting one down at a place among its cards does. A zone
    /// that is not one row - a pool in columns - says itself which place a point is (<paramref name="Locate"/>) and
    /// where to show it (<paramref name="Bar"/>); a row counts places along itself.
    /// </summary>
    private sealed record DropZone(
        CardBoardElement Board,
        Func<ViewHandle, bool> Accepts,
        Action<ViewHandle, int> Put,
        Func<ViewHandle, Point, int>? Locate = null,
        Func<ViewHandle, int, Rect?>? Bar = null);

    /// <summary>The rows that take drops for as long as the page is up: the hand.</summary>
    private readonly List<DropZone> _dropZones = [];

    /// <summary>The rows the dock's prompt made, taken down with it: a distribute prompt's piles.</summary>
    private readonly List<DropZone> _promptDropZones = [];

    /// <summary>
    /// The rows of cards the dock's prompt made, taken down with it - a distribute prompt's piles and the cards still to
    /// place - read like any other row: a tooltip, and right-press to hold a card up.
    /// </summary>
    private readonly List<CardBoardElement> _promptBoards = [];

    private readonly Canvas _carryLayer = new() { IsHitTestVisible = false };

    /// <summary>The copy of the card being carried, and where on it the pointer took hold; null while nothing is.</summary>
    private CardBoardElement? _carried;
    private Point _grabbed;

    /// <summary>Where the copy being carried is on the page: its top left, on the carry layer, which covers the page as the flight layer does.</summary>
    private Point _carriedAt;

    /// <summary>The cards carried with the one taken hold of, each with how far down and right of it it rides.</summary>
    private List<(ViewHandle Handle, float Offset)> _carriedStack = [];

    /// <summary>Where each card carried with the one let go was in the stack, on the page; empty when it went alone.</summary>
    private Dictionary<ViewHandle, CardFlights.Pose> _dropPoses = [];

    /// <summary>Lets a row's cards be picked up; wired once per row.</summary>
    private void MakeDraggable(CardBoardElement board)
    {
        board.Draggable = true;
        board.PickedUp += (handle, grabbed) => PickUp(board, handle, grabbed);
        board.Carrying += Carry;
        board.Dropped += PutDown;
    }

    private void PickUp(CardBoardElement from, ViewHandle handle, Point grabbed)
    {
        if (from.CardOf(handle) is not { } card)
        {
            return;
        }

        _tooltip.Hide();
        _grabbed = grabbed;

        // Cards of the deck picked together go together: the others ride behind the one taken hold of.
        List<ObjectView> with = PickUpPicks(from, handle);
        float offset = MathF.Round(from.CellWidth * 0.07f);
        int behind = Math.Min(with.Count, 3);
        _carried = new CardBoardElement(from.CellWidth, from.CellHeight)
        {
            Width = from.CellWidth + (behind * offset),
            Height = from.CellHeight + (behind * offset),
            Opacity = 0.85,
            FliesAlong = false,
        };

        _carried.SetCells([
            .. with.Take(behind).Select((other, i) => new CardBoardElement.BoardCell(other, (behind - i) * offset, (behind - i) * offset, Tapped: false)),
            new CardBoardElement.BoardCell(card, 0, 0, Tapped: false, Annotation: with.Count > 0 ? $"{with.Count + 1} cards" : null),
        ]);
        _carried.Hidden = _flights.Flying;
        _carryLayer.Children.Clear();
        _carryLayer.Children.Add(_carried);

        // The others are scooped up into the stack: each flies from its place to its own in the stack, following the
        // pointer - those the stack has no room to show, to under the card taken hold of.
        _carriedStack = [.. with.Select((other, i) => (other.Handle, i < behind ? (behind - i) * offset : 0f))];
        foreach ((ViewHandle other, float down) in _carriedStack)
        {
            foreach (CardBoardElement board in (CardBoardElement[])[_mainBoard, _sideBoard])
            {
                if (board.Bounds(other) is { } box && board.XamlRoot is not null && board.CardOf(other) is { } face)
                {
                    _flights.Gather(other, Middle(board, box), () => StackPose(from, down), face);
                }
            }
        }
    }

    /// <summary>Where a card riding a distance down and right of the one carried is on the page.</summary>
    private CardFlights.Pose StackPose(CardBoardElement from, float offset) =>
        new((float)_carriedAt.X + offset + (from.CellWidth / 2), (float)_carriedAt.Y + offset + (from.CellHeight / 2), from.CellWidth, 0f);

    /// <summary>Moves the copy with the pointer and marks where the card would go in the row it is over, if that row takes it.</summary>
    private void Carry(ViewHandle handle, Point at)
    {
        if (_carried is null)
        {
            return;
        }

        Point onLayer = _carryLayer.TransformToVisual(null).Inverse.TransformPoint(at);
        _carriedAt = new Point(onLayer.X - _grabbed.X, onLayer.Y - _grabbed.Y);
        Canvas.SetLeft(_carried, _carriedAt.X);
        Canvas.SetTop(_carried, _carriedAt.Y);

        (DropZone Zone, Point Local)? over = ZoneAt(handle, at);
        foreach (DropZone zone in DropZones())
        {
            bool here = over is { } target && target.Zone == zone;
            if (zone.Locate is { } locate)
            {
                zone.Board.DropBar = here && zone.Bar is { } bar ? bar(handle, locate(handle, over!.Value.Local)) : null;
                continue;
            }

            zone.Board.DropMarker = here
                ? zone.Board.MarkerAt(handle, zone.Board.DropIndex(handle, over!.Value.Local))
                : null;
        }
    }

    /// <summary>Puts the card down in the row it was let go over, if that row takes it; anywhere else, it goes back.</summary>
    private void PutDown(ViewHandle handle, Point? at)
    {
        // A card of the pool flies from where it was let go to wherever it is put, or back where it came from. A card
        // of the pack put in the pool flies from there once it is picked (ShowPool); let go anywhere else, back to the pack.
        // A card of the deck editor's search results is put in the deck as a pack's card is picked: a copy flies from
        // where it was let go (MainPage.Decks).
        bool packed = InPack(handle) || InSearch(handle);
        bool pooled = (_poolPlaces.ContainsKey(handle.Value) || packed) && _carried is not null;
        (DropZone Zone, Point Local)? dropped = at is { } released ? ZoneAt(handle, released) : null;
        _dropPose = null;
        _dropPoses = [];
        if (pooled && _carried!.XamlRoot is not null)
        {
            Point middle = _carried.TransformToVisual(Root).TransformPoint(new Point(_carried.CellWidth / 2, _carried.CellHeight / 2));
            _dropPose = new CardFlights.Pose((float)middle.X, (float)middle.Y, _carried.CellWidth, 0f);

            // The cards carried with it fly out of the stack from where each rode in it, under the one taken hold of.
            foreach ((ViewHandle other, float down) in _carriedStack)
            {
                _dropPoses[other] = _dropPose.Value with { X = _dropPose.Value.X + down, Y = _dropPose.Value.Y + down };
                _flights.Toss(other, _dropPoses[other]);
            }

            if (!packed || dropped is null)
            {
                _flights.Toss(handle, _dropPose.Value, onTop: _carriedStack.Count > 0);
            }
        }

        _carriedStack = [];

        CardBoardElement? copy = _carried;
        _carried = null;
        _carryLayer.Children.Clear();

        foreach (DropZone zone in DropZones())
        {
            zone.Board.DropMarker = null;
            zone.Board.DropBar = null;
        }

        // Where it goes is counted among the cards not carried; the cards carried with it are put down with it, where they
        // can go, and are not carried any longer.
        int? slot = dropped is { } over
            ? over.Zone.Locate is { } locate ? locate(handle, over.Local) : over.Zone.Board.DropIndex(handle, over.Local)
            : null;
        IReadOnlyList<int>? group = PutDownPicks();
        if (dropped is { } target && slot is { } place)
        {
            if (group is null || !PutPicks(target.Zone, group, place))
            {
                target.Zone.Put(handle, place);
            }
        }
        else if (pooled)
        {
            LayOutPool();
        }

        // A card of the pack put in the pool waits where it was let go, gone from the pack, until the pick is answered.
        if (packed && _pickInto is { } into && into.Handle == handle.Value && copy is not null)
        {
            _parkedPick = copy;
            _carryLayer.Children.Add(copy);
            _pileBoard.Carried = handle;
        }

        _needsRedraw = true;
        RedrawNow();
    }

    private IEnumerable<DropZone> DropZones() => _dropZones.Concat(_promptDropZones);

    /// <summary>The row under a point in the window that will take the card, with the point in that row's own terms.</summary>
    private (DropZone Zone, Point Local)? ZoneAt(ViewHandle handle, Point at)
    {
        foreach (DropZone zone in DropZones())
        {
            CardBoardElement board = zone.Board;
            if (board.XamlRoot is null || !IsShown(board) || !zone.Accepts(handle))
            {
                continue;
            }

            Point local = board.TransformToVisual(null).Inverse.TransformPoint(at);
            if (local.X >= 0 && local.Y >= 0 && local.X <= board.ActualWidth && local.Y <= board.ActualHeight)
            {
                return (zone, local);
            }
        }

        return null;
    }

    /// <summary>
    /// The hand in the order its player has dragged it into. A hand has no order in the rules, so the server never hears
    /// of it: cards keep the places they were put in, and a card drawn joins at the right.
    /// </summary>
    private readonly List<ViewHandle> _handOrder = [];

    private List<ObjectView> InHandOrder(List<ObjectView> held)
    {
        HashSet<ViewHandle> present = [.. held.Select(card => card.Handle)];
        _handOrder.RemoveAll(handle => !present.Contains(handle));
        _handOrder.AddRange(held.Select(card => card.Handle).Where(handle => !_handOrder.Contains(handle)));
        return [.. held.OrderBy(card => _handOrder.IndexOf(card.Handle))];
    }

    /// <summary>Puts a card at a place in an ordered list, taking it out of wherever it was first.</summary>
    private static void Place<T>(List<T> order, T item, int index)
    {
        order.Remove(item);
        order.Insert(Math.Clamp(index, 0, order.Count), item);
    }
}
