using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// Cards of the deck picked together in the deck editor, to be dragged as one. A rectangle dragged out from a place with no
/// card, over the deck's columns or its sideboard, picks every card it touches where that card shows (Ctrl or Shift held
/// adds them to what is picked); Ctrl- or Shift-click picks a card or lets it go; a click on no card, or a plain click on
/// a card, lets them all go. Picked cards wear the gold border. Taking hold of a picked card carries every picked card,
/// and they go down together, in the order they were in, wherever that one could go: a column, the sideboard, or back
/// onto the search, out of the deck. Taking hold of a card not picked carries it alone, and lets the others go.
/// <para>
/// The search's results are cards to take from, not cards of the deck, and are never picked: a rectangle is not drawn
/// over them. A touch on no card scrolls, as it always did; a rectangle is drawn with a mouse or a pen.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    /// <summary>The cards of the deck picked, by place.</summary>
    private readonly HashSet<int> _deckPicks = [];

    /// <summary>The picked cards being carried together, by place, in the order they go down in; null while none are.</summary>
    private List<int>? _carriedPicks;

    private bool Picked(int place) => DeckEditorShown && _deckPicks.Contains(place);

    /// <summary>Lets rectangles be drawn over the deck's columns and sideboard; wired once, with the editor.</summary>
    private void MakePickable()
    {
        foreach (CardBoardElement board in (CardBoardElement[])[_mainBoard, _sideBoard])
        {
            board.Marquees = true;
            board.Marqueed += (handles, modifiers) => MarqueePicked(handles, modifiers);
            board.EmptyClicked += modifiers =>
            {
                if (!Adds(modifiers))
                {
                    LetPicksGo();
                }
            };
        }
    }

    private static bool Adds(Windows.System.VirtualKeyModifiers modifiers) =>
        (modifiers & (Windows.System.VirtualKeyModifiers.Control | Windows.System.VirtualKeyModifiers.Shift)) != 0;

    /// <summary>A rectangle drawn: the cards it touched are what is picked, or are picked as well with Ctrl or Shift held.</summary>
    private void MarqueePicked(IReadOnlyList<ViewHandle> touched, Windows.System.VirtualKeyModifiers modifiers)
    {
        if (!DeckEditorShown)
        {
            return;
        }

        if (!Adds(modifiers))
        {
            _deckPicks.Clear();
        }

        foreach (ViewHandle handle in touched)
        {
            if (_poolPlaces.TryGetValue(handle.Value, out int place))
            {
                _deckPicks.Add(place);
            }
        }

        LayOutPool();
    }

    private void LetPicksGo()
    {
        if (_deckPicks.Count > 0)
        {
            _deckPicks.Clear();
            LayOutPool();
        }
    }

    /// <summary>Forgets picks of cards no longer in the deck or its sideboard.</summary>
    private void KeepPicks(PoolArrangement arrangement)
    {
        if (!DeckEditorShown)
        {
            _deckPicks.Clear();
            return;
        }

        HashSet<int> kept = [.. arrangement.Columns.SelectMany(column => column), .. arrangement.Side];
        _deckPicks.RemoveWhere(place => !kept.Contains(place));
    }

    /// <summary>
    /// A click on a card of the deck: with Ctrl or Shift, picks it or lets it go, and is nothing more (true); a plain
    /// click lets every pick go, and is a click as before (false).
    /// </summary>
    private bool PickClicked(CardBoardElement board, ViewHandle handle)
    {
        if (!DeckEditorShown || (board != _mainBoard && board != _sideBoard) || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return false;
        }

        if (Adds(board.LastModifiers))
        {
            if (!_deckPicks.Remove(place))
            {
                _deckPicks.Add(place);
            }

            LayOutPool();
            return true;
        }

        LetPicksGo();
        return false;
    }

    /// <summary>The picked cards in the order they are laid out: the columns from the left, each from the top, then the sideboard.</summary>
    private List<int> PicksInOrder() =>
        _arrangement is not { } arrangement
            ? []
            : [.. arrangement.Columns.SelectMany(column => column).Concat(arrangement.Side).Where(_deckPicks.Contains)];

    /// <summary>
    /// A card of the deck taken hold of: when it is picked with others, they are carried with it, and their faces are
    /// returned to ride with it; when it is not picked, the picks are let go and it goes alone.
    /// </summary>
    private List<ObjectView> PickUpPicks(CardBoardElement from, ViewHandle handle)
    {
        if (!DeckEditorShown || (from != _mainBoard && from != _sideBoard) || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return [];
        }

        if (!_deckPicks.Contains(place))
        {
            LetPicksGo();
            return [];
        }

        if (_deckPicks.Count < 2)
        {
            return [];
        }

        _carriedPicks = PicksInOrder();
        Dictionary<int, ViewHandle> handles = _poolPlaces.ToDictionary(pair => pair.Value, pair => new ViewHandle(pair.Key));
        HashSet<ViewHandle> others = [.. _carriedPicks.Where(other => other != place && handles.ContainsKey(other)).Select(other => handles[other])];
        _mainBoard.CarriedWith = others;
        _sideBoard.CarriedWith = others;
        return [.. others.Select(other => _mainBoard.CardOf(other) ?? _sideBoard.CardOf(other)).OfType<ObjectView>()];
    }

    /// <summary>The picked cards carried together, now let go; null when the card let go was carried alone.</summary>
    private List<int>? PutDownPicks()
    {
        List<int>? carried = _carriedPicks;
        _carriedPicks = null;
        if (carried is not null)
        {
            _mainBoard.CarriedWith = null;
            _sideBoard.CarriedWith = null;
        }

        return carried;
    }

    /// <summary>
    /// Puts picked cards carried together down where the one taken hold of was let go: into a column or the sideboard at
    /// that place, in their order, or back onto the search, out of the deck. False where a group does not go - the command
    /// zone takes the one card held, as it always has.
    /// </summary>
    private bool PutPicks(DropZone zone, IReadOnlyList<int> group, int slot)
    {
        if (_arrangement is not { } arrangement)
        {
            return false;
        }

        if (zone.Board == _searchBoard)
        {
            foreach (int place in group)
            {
                arrangement.Remove(place);
            }

            _deckPicks.Clear();
            DeckEdited();
            return true;
        }

        bool toSide = zone.Board == _sideBoard;
        if (!toSide && zone.Board != _mainBoard)
        {
            return false;
        }

        // A basic land never goes in the sideboard: it stays where it is.
        List<int> moving = [.. group.Where(place => !toSide || !IsBasic(place))];
        List<int> into = toSide ? arrangement.Side : arrangement.Columns[Math.Clamp(slot / 1000, 0, arrangement.Columns.Count - 1)];
        int row = toSide ? slot : slot % 1000;

        // The place is counted among the cards not moving: the group goes before the one it was let go above.
        List<int> staying = [.. into.Where(place => !moving.Contains(place))];
        int? before = row >= 0 && row < staying.Count ? staying[row] : null;
        bool sideChanged = moving.Any(place => arrangement.InSide(place) != toSide);
        foreach (int place in moving)
        {
            arrangement.Remove(place);
        }

        into.InsertRange(before is { } at ? into.IndexOf(at) : into.Count, moving);

        // Into a sideboard tucked away, each flies into the strip from where they were let go.
        if (toSide && _dropPose is { } dropped)
        {
            foreach ((int handle, _) in _poolPlaces.Where(pair => moving.Contains(pair.Value)).ToList())
            {
                IntoTuckedSide(new ViewHandle(handle), dropped);
            }
        }

        PoolMoved(sideChanged);
        return true;
    }
}
