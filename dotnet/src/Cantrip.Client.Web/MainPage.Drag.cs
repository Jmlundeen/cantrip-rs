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
        _carried = new CardBoardElement(from.CellWidth, from.CellHeight)
        {
            Width = from.CellWidth,
            Height = from.CellHeight,
            Opacity = 0.85,
            FliesAlong = false,
        };

        _carried.SetCells([new CardBoardElement.BoardCell(card, 0, 0, Tapped: false)]);
        _carryLayer.Children.Clear();
        _carryLayer.Children.Add(_carried);
    }

    /// <summary>Moves the copy with the pointer and marks where the card would go in the row it is over, if that row takes it.</summary>
    private void Carry(ViewHandle handle, Point at)
    {
        if (_carried is null)
        {
            return;
        }

        Point onLayer = _carryLayer.TransformToVisual(null).Inverse.TransformPoint(at);
        Canvas.SetLeft(_carried, onLayer.X - _grabbed.X);
        Canvas.SetTop(_carried, onLayer.Y - _grabbed.Y);

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
        bool packed = InPack(handle);
        bool pooled = (_poolPlaces.ContainsKey(handle.Value) || packed) && _carried is not null;
        (DropZone Zone, Point Local)? dropped = at is { } released ? ZoneAt(handle, released) : null;
        _dropPose = null;
        if (pooled && _carried!.XamlRoot is not null)
        {
            Point middle = _carried.TransformToVisual(Root).TransformPoint(new Point(_carried.Width / 2, _carried.Height / 2));
            _dropPose = new CardFlights.Pose((float)middle.X, (float)middle.Y, (float)_carried.Width, 0f);
            if (!packed || dropped is null)
            {
                _flights.Toss(handle, _dropPose.Value);
            }
        }

        CardBoardElement? copy = _carried;
        _carried = null;
        _carryLayer.Children.Clear();

        foreach (DropZone zone in DropZones())
        {
            zone.Board.DropMarker = null;
            zone.Board.DropBar = null;
        }

        if (dropped is { } target)
        {
            target.Zone.Put(handle, target.Zone.Locate is { } locate
                ? locate(handle, target.Local)
                : target.Zone.Board.DropIndex(handle, target.Local));
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
