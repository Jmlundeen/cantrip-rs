using Cantrip.Protocol;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls.Primitives;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A pool of cards laid out in columns, as the draft shows what has been drafted and deck building shows the deck: lands
/// on the left, then a column for each mana value from nothing up to seven and more, with a sideboard that pulls out on
/// the far left. Cards are dragged between columns, into the sideboard - open or not - and out of it; a double-click
/// moves a card between the main deck and the sideboard. The card size is the player's (the slider beside the heading).
/// A card of a booster pack dragged in is picked, and goes where it was put.
/// <para>
/// Where each card sits is this page's alone, and goes with it: it is not saved, not sent, and starts again from the
/// default columns on a reload. What is in the sideboard is the server's once decks are built, since that is the deck;
/// during the draft it is this page's too, and becomes the deck's starting sideboard when the draft ends.
/// </para>
/// <para>
/// Each card keeps one handle for as long as the page is up - the same in the pile it was drafted from as in the pool -
/// so the flight layer flies a pile taken into the columns, and a card taken blind from the stack's tile.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    private const float PoolCardWidth = 110f;
    private const float PoolGap = 10f;
    private const float PoolHeaderHeight = 22f;
    private const float SideClosedWidth = 44f;
    private const string PoolScaleKey = "pool-card-scale";

    /// <summary>The columns a card goes in to start with: lands, then mana values 0 to 6, then 7 and more.</summary>
    private static readonly string[] PoolColumnNames = ["Lands", "0", "1", "2", "3", "4", "5", "6", "7+"];

    /// <summary>
    /// Where each card of a pool is, by its place in the pool: the main deck's columns and the sideboard. A basic land
    /// added is in the columns too, under a place of its own past every pool card's (<see cref="BasicPlace"/>), which
    /// is also its handle; it is never in the sideboard.
    /// </summary>
    private sealed class PoolArrangement
    {
        public string EventId = string.Empty;
        public List<List<int>> Columns = [.. PoolColumnNames.Select(_ => new List<int>())];
        public List<int> Side = [];

        /// <summary>The commanders, for a Commander deck in the deck editor: in the command zone, neither deck nor sideboard.</summary>
        public List<int> Command = [];

        /// <summary>The pool cards of the main deck, without the basic lands.</summary>
        public IEnumerable<int> Main => Columns.SelectMany(column => column).Where(place => !IsBasic(place));

        /// <summary>The basic lands added, wherever they are in the columns.</summary>
        public IEnumerable<int> Basics => Columns.SelectMany(column => column).Where(IsBasic);

        /// <summary>Takes a card out of wherever it is.</summary>
        public void Remove(int place)
        {
            foreach (List<int> column in Columns)
            {
                column.Remove(place);
            }

            Side.Remove(place);
            Command.Remove(place);
        }

        public bool InSide(int place) => Side.Contains(place);

        /// <summary>Moves every pool card to its new place in the pool, by <paramref name="to"/>; one with none (-1) leaves.</summary>
        public void Renumber(Func<int, int> to)
        {
            foreach (List<int> places in Columns.Append(Side))
            {
                List<int> moved = [.. places.Select(place => IsBasic(place) ? place : to(place)).Where(place => place >= 0)];
                places.Clear();
                places.AddRange(moved);
            }
        }
    }

    /// <summary>Where the basic lands' places (and handles) start: each is this, plus its kind * 1000, plus which copy it is.</summary>
    private const int BasicBase = 2_000_000;

    private static bool IsBasic(int place) => place >= BasicBase;

    private static int BasicPlace(int kind, int copy) => BasicBase + (kind * 1000) + copy;

    private static int BasicKind(int place) => (place - BasicBase) / 1000;

    private PoolArrangement? _arrangement;
    private bool _poolSideOpen;
    private float _poolScale = Math.Clamp(LocalSettings.Number(PoolScaleKey, 1f), 0.6f, 1.6f);

    /// <summary>The pool as the server last sent it, in pool order, and whether the deck is being built rather than drafted.</summary>
    private IReadOnlyList<ObjectView> _poolCards = [];
    private bool _poolIsDeck;

    /// <summary>Each basic land that may be added, with how many are; where it is in the list is its kind. Empty while drafting.</summary>
    private IReadOnlyList<(BasicLandView Land, int Count)> _poolBasics = [];

    /// <summary>A handle for each card for as long as the page is up, by the card and which copy of it it is.</summary>
    private readonly Dictionary<(string Card, int Copy), int> _eventHandles = [];
    private int _nextEventHandle = 1_000_000;

    /// <summary>The place in the pool of each handle laid out in the pool now.</summary>
    private readonly Dictionary<int, int> _poolPlaces = [];

    private readonly CardBoardElement _mainBoard = new(PoolCardWidth, Tall(PoolCardWidth)) { Slides = true, HidesCarried = true, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
    private readonly CardBoardElement _sideBoard = new(PoolCardWidth, Tall(PoolCardWidth)) { Slides = true, HidesCarried = true, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
    private readonly Canvas _poolHeadings = new() { IsHitTestVisible = false, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };

    private Grid _poolView = null!;
    private TextBlock _poolLabel = null!;
    private TextBlock _poolHint = null!;
    private Slider _poolSlider = null!;
    private ScrollViewer _mainScroll = null!;
    private ScrollViewer _sideScroll = null!;
    private Border _sideColumn = null!;

    /// <summary>The deck editor's command zone, beside the sideboard: a Commander deck's commanders (MainPage.Decks).</summary>
    private readonly CardBoardElement _commandBoard = new(PoolCardWidth, Tall(PoolCardWidth)) { Slides = true, HidesCarried = true, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
    private Border _commandColumn = null!;
    private TextBlock _commandLabel = null!;
    private DockButton _sideToggle = null!;
    private TextBlock _sideLabel = null!;

    /// <summary>Where the card being put down was let go, on the page; null when it was not carried.</summary>
    private CardFlights.Pose? _dropPose;

    /// <summary>The last card clicked, on which row and when, for telling a double-click.</summary>
    private (CardBoardElement Board, ViewHandle Handle, DateTime At)? _lastPoolClick;

    private float PoolWidth => MathF.Round(PoolCardWidth * _poolScale);

    private float PoolStep => MathF.Round(Tall(PoolWidth) * 0.17f);

    private Grid BuildPoolView()
    {
        var view = new Grid { RowSpacing = 6 };
        view.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        view.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });

        // The heading, with the card size beside it.
        var heading = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16 };
        _poolLabel = EventText(string.Empty, 14, Theme.Fg, semibold: true);
        _poolLabel.VerticalAlignment = VerticalAlignment.Center;
        _poolSlider = new Slider
        {
            Minimum = 0.6, Maximum = 1.6, StepFrequency = 0.05, Value = _poolScale, Width = 160,
            VerticalAlignment = VerticalAlignment.Center, RequestedTheme = ElementTheme.Dark,
        };
        AutomationProperties.SetName(_poolSlider, "Card size");
        ToolTipService.SetToolTip(_poolSlider, "Card size");
        _poolSlider.ValueChanged += (_, e) =>
        {
            _poolScale = (float)e.NewValue;
            LocalSettings.Store(PoolScaleKey, _poolScale);
            _flights.Settle();
            LayOutPool();
            LayOutSearch();
        };
        heading.Children.Add(_poolLabel);
        heading.Children.Add(_poolSlider);
        _poolHint = EventText("Drag cards between columns  ·  double-click to move one to or from the sideboard, or a basic land out of the deck", 12, Theme.FgMuted);
        _poolHint.VerticalAlignment = VerticalAlignment.Center;
        heading.Children.Add(_poolHint);
        view.Children.Add(heading);

        var body = new Grid { ColumnSpacing = 10 };
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        Grid.SetRow(body, 1);
        view.Children.Add(body);

        // The sideboard, pulled out or tucked away on the left. Tucked away it is still somewhere to drop a card.
        _sideToggle = new DockButton("›", primary: false).Filling();
        _sideToggle.Clicked += () =>
        {
            _poolSideOpen = !_poolSideOpen;
            _flights.Settle();
            LayOutPool();
        };
        _sideLabel = EventText(string.Empty, 13, Theme.FgSoft, semibold: true);
        _sideScroll = new ScrollViewer
        {
            HorizontalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            Content = _sideBoard,
        };
        var side = new Grid { RowSpacing = 6 };
        side.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        side.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        side.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        side.Children.Add(_sideToggle);
        Grid.SetRow(_sideLabel, 1);
        side.Children.Add(_sideLabel);
        Grid.SetRow(_sideScroll, 2);
        side.Children.Add(_sideScroll);
        _sideColumn = new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(6),
            Child = side,
        };
        body.Children.Add(_sideColumn);

        // The command zone, only for a Commander deck in the deck editor.
        _commandLabel = EventText("Commander", 13, Theme.FgSoft, semibold: true);
        var command = new Grid { RowSpacing = 6 };
        command.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        command.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        command.Children.Add(_commandLabel);
        Grid.SetRow(_commandBoard, 1);
        command.Children.Add(_commandBoard);
        _commandColumn = new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Accent),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(6),
            Visibility = Visibility.Collapsed,
            Child = command,
        };
        Grid.SetColumn(_commandColumn, 1);
        body.Children.Add(_commandColumn);

        _mainScroll = new ScrollViewer
        {
            HorizontalScrollMode = ScrollMode.Auto,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Auto,
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            Content = new Grid { Children = { _mainBoard, _poolHeadings } },
        };
        Grid.SetColumn(_mainScroll, 2);
        body.Children.Add(_mainScroll);

        _mainScroll.SizeChanged += (_, _) => LayOutPool();
        _sideScroll.SizeChanged += (_, _) => LayOutPool();

        // Picked up from either and put down in either: a column of the main deck, or the sideboard.
        MakeDraggable(_mainBoard);
        MakeDraggable(_sideBoard);
        _dropZones.Add(new DropZone(_mainBoard, handle => InPool(handle) || InPack(handle) || InSearch(handle), (handle, slot) => PutInColumn(handle, slot / 1000, slot % 1000), ColumnSlot, ColumnBar));
        _dropZones.Add(new DropZone(_sideBoard, handle => (InPool(handle) && !IsBasic(handle.Value)) || InPack(handle) || InSearch(handle), (handle, slot) => PutInSide(handle, slot), SideSlot, SideBar));
        _mainBoard.CardClicked += handle => PoolClicked(_mainBoard, handle);
        _sideBoard.CardClicked += handle => PoolClicked(_sideBoard, handle);

        _poolView = view;
        return view;
    }

    /// <summary>
    /// Shows a pool: the cards in pool order, which are in the main deck when the server says so (deck building) or as
    /// this page has them (the draft), and the basic lands added.
    /// </summary>
    private void ShowPool(string eventId, IReadOnlyList<ObjectView> cards, IReadOnlyCollection<int>? serverMain, IReadOnlyList<(BasicLandView Land, int Count)> basics)
    {
        // A card that left the pool - a Cogwork Librarian put into a pack - leaves a gap every later card closes up: each
        // keeps where it was laid out, found by the card and which copy of it it is.
        if (_arrangement is { } kept && kept.EventId == eventId
            && (cards.Count < _poolCards.Count || _poolCards.Where((card, place) => card.CardId != cards[place].CardId).Any()))
        {
            List<(string, int)> before = CopyKeys(_poolCards);
            Dictionary<(string, int), int> after = CopyKeys(cards).Select((key, place) => (key, place)).ToDictionary(pair => pair.key, pair => pair.place);
            kept.Renumber(place => place < before.Count && after.TryGetValue(before[place], out int now) ? now : -1);
        }

        _poolCards = cards;
        _poolIsDeck = serverMain is not null;
        _poolBasics = basics;

        if (_arrangement is null || _arrangement.EventId != eventId)
        {
            _arrangement = new PoolArrangement { EventId = eventId };
        }

        PoolArrangement arrangement = _arrangement;
        foreach (List<int> column in arrangement.Columns)
        {
            column.RemoveAll(place => place >= cards.Count && !IsBasic(place));
        }

        PlaceBasics(arrangement);

        arrangement.Side.RemoveAll(place => place >= cards.Count);

        HashSet<int> placed = [.. arrangement.Main, .. arrangement.Side];
        Dictionary<string, int> copies = new(StringComparer.Ordinal);
        bool added = false;
        for (int place = 0; place < cards.Count; place++)
        {
            bool side = serverMain is not null && !serverMain.Contains(place);
            int copy = copies.GetValueOrDefault(cards[place].CardId);
            copies[cards[place].CardId] = copy + 1;
            if (!placed.Contains(place))
            {
                added = true;
                if (serverMain is null && _pickInto is { } into && into.Handle == EventHandle(cards[place].CardId, copy))
                {
                    // A card dragged in from the pack goes where it was put, flying from where it was let go.
                    List<int> to = into.Side ? arrangement.Side : arrangement.Columns[Math.Clamp(into.Column, 0, arrangement.Columns.Count - 1)];
                    to.Insert(Math.Clamp(into.Row, 0, to.Count), place);
                    if (into.From is { } from)
                    {
                        _flights.Toss(new ViewHandle(into.Handle), from);
                    }
                }
                else
                {
                    (side ? arrangement.Side : arrangement.Columns[DefaultColumn(cards[place])]).Add(place);
                }
            }
            else if (serverMain is not null && side != arrangement.InSide(place))
            {
                // The server's deck is the deck: a card it has on the other side than this page goes there.
                arrangement.Remove(place);
                (side ? arrangement.Side : arrangement.Columns[DefaultColumn(cards[place])]).Add(place);
            }
        }

        // The pick a card was dragged in for is over, whichever card it took.
        if (added)
        {
            _pickInto = null;
        }

        LayOutPool();
    }

    /// <summary>
    /// Brings the basic lands in the columns to as many of each as the deck has: one taken away is the highest-numbered
    /// copy, wherever it was moved, and one added goes at the foot of the lands column. The others stay where they were put.
    /// </summary>
    private void PlaceBasics(PoolArrangement arrangement)
    {
        foreach (int place in arrangement.Basics.Where(place => BasicKind(place) >= _poolBasics.Count).ToList())
        {
            arrangement.Remove(place);
        }

        for (int kind = 0; kind < _poolBasics.Count; kind++)
        {
            int wanted = _poolBasics[kind].Count;
            List<int> laid = [.. arrangement.Basics.Where(place => BasicKind(place) == kind).Order()];
            for (int extra = laid.Count - 1; extra >= wanted; extra--)
            {
                arrangement.Remove(laid[extra]);
            }

            for (int copy = 0, have = laid.Count; have < wanted; copy++)
            {
                if (!laid.Contains(BasicPlace(kind, copy)))
                {
                    arrangement.Columns[0].Add(BasicPlace(kind, copy));
                    have++;
                }
            }
        }
    }

    /// <summary>Each card of a pool as the card it is and which copy of it, in pool order.</summary>
    private static List<(string Card, int Copy)> CopyKeys(IReadOnlyList<ObjectView> pool)
    {
        Dictionary<string, int> copies = new(StringComparer.Ordinal);
        List<(string, int)> keys = [];
        foreach (ObjectView card in pool)
        {
            int copy = copies.GetValueOrDefault(card.CardId);
            copies[card.CardId] = copy + 1;
            keys.Add((card.CardId, copy));
        }

        return keys;
    }

    private static int DefaultColumn(ObjectView card) =>
        card.CardTypes.Contains(CardTypeView.Land) ? 0 : 1 + Math.Clamp(card.ManaValue, 0, 7);

    /// <summary>The handle a card keeps on this page: by its card, and which copy of that card it is.</summary>
    private int EventHandle(string card, int copy)
    {
        if (!_eventHandles.TryGetValue((card, copy), out int handle))
        {
            handle = _nextEventHandle++;
            _eventHandles[(card, copy)] = handle;
        }

        return handle;
    }

    /// <summary>The pool's cards by place, each under its handle.</summary>
    private List<ObjectView> PoolHandled()
    {
        Dictionary<string, int> copies = new(StringComparer.Ordinal);
        List<ObjectView> handled = [];
        _poolPlaces.Clear();
        for (int place = 0; place < _poolCards.Count; place++)
        {
            ObjectView card = _poolCards[place];
            int copy = copies.GetValueOrDefault(card.CardId);
            copies[card.CardId] = copy + 1;
            ObjectView shown = Handled(card, EventHandle(card.CardId, copy));
            _poolPlaces[shown.Handle.Value] = place;
            handled.Add(shown);
        }

        foreach (int basic in _arrangement?.Basics ?? [])
        {
            _poolPlaces[basic] = basic;
        }

        return handled;
    }

    /// <summary>A pile's cards under the handles they will keep when they join the pool: the next copies of each.</summary>
    private List<ObjectView> PileHandled(IReadOnlyList<ObjectView> pool, IReadOnlyList<ObjectView> pile)
    {
        Dictionary<string, int> copies = new(StringComparer.Ordinal);
        foreach (ObjectView card in pool)
        {
            copies[card.CardId] = copies.GetValueOrDefault(card.CardId) + 1;
        }

        List<ObjectView> handled = [];
        foreach (ObjectView card in pile)
        {
            int copy = copies.GetValueOrDefault(card.CardId);
            copies[card.CardId] = copy + 1;
            handled.Add(Handled(card, EventHandle(card.CardId, copy)));
        }

        return handled;
    }

    /// <summary>Lays the pool out: the columns, the sideboard open or tucked away, and the counts.</summary>
    private void LayOutPool()
    {
        if (_arrangement is not { } arrangement || _poolView is null)
        {
            return;
        }

        float width = PoolWidth;
        float height = Tall(width);
        float step = PoolStep;
        _mainBoard.SetCellSize(width, height);
        _sideBoard.SetCellSize(width, height);
        List<ObjectView> handled = PoolHandled();
        KeepPicks(arrangement);

        // The columns, each a stack of cards showing their names. A basic land is a card of its own for every copy.
        List<CardBoardElement.BoardCell> cells = [];
        _poolHeadings.Children.Clear();
        int tallest = 1;
        for (int column = 0; column < arrangement.Columns.Count; column++)
        {
            float x = column * (width + PoolGap);
            List<int> places = arrangement.Columns[column];
            for (int row = 0; row < places.Count; row++)
            {
                int place = places[row];
                ObjectView card = IsBasic(place) ? Handled(_poolBasics[BasicKind(place)].Land.Card, place) : handled[place];
                cells.Add(new CardBoardElement.BoardCell(card, x, PoolHeaderHeight + (row * step), Tapped: false, Clickable: !_deckSubmitted, Selected: Picked(place)));
            }

            int count = places.Count;
            tallest = Math.Max(tallest, count);
            var heading = EventText(count == 0 ? PoolColumnNames[column] : $"{PoolColumnNames[column]}  ·  {count}", 12, count == 0 ? Theme.FgMuted : Theme.FgSoft, semibold: true);
            Canvas.SetLeft(heading, x + 2);
            _poolHeadings.Children.Add(heading);
        }

        float columnsWidth = (arrangement.Columns.Count * (width + PoolGap)) - PoolGap;
        _mainBoard.Width = Math.Max(columnsWidth, (float)_mainScroll.ActualWidth - 4);
        _mainBoard.Height = Math.Max(PoolHeaderHeight + ((tallest - 1) * step) + height, (float)_mainScroll.ActualHeight - 4);
        _poolHeadings.Width = _mainBoard.Width;
        _mainBoard.SetCells(cells);

        // The sideboard: a column of its own when pulled out, and only somewhere to drop a card when not.
        float sideWidth = _poolSideOpen ? width : SideClosedWidth - 14;
        _sideColumn.Width = sideWidth + 14;
        _sideToggle.Caption = _poolSideOpen ? "‹" : "›";
        AutomationProperties.SetName(_sideToggle, _poolSideOpen ? "Hide sideboard" : "Show sideboard");
        _sideLabel.Text = _poolSideOpen ? $"Sideboard  ·  {arrangement.Side.Count}" : arrangement.Side.Count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        _sideLabel.HorizontalAlignment = _poolSideOpen ? HorizontalAlignment.Left : HorizontalAlignment.Center;
        _sideBoard.Width = sideWidth;
        _sideBoard.Height = Math.Max(_poolSideOpen ? ((Math.Max(1, arrangement.Side.Count) - 1) * step) + height : 0, (float)_sideScroll.ActualHeight - 2);
        _sideBoard.SetCells(_poolSideOpen
            ? [.. arrangement.Side.Select((place, row) => new CardBoardElement.BoardCell(handled[place], 0, row * step, Tapped: false, Clickable: !_deckSubmitted, Selected: Picked(place)))]
            : []);

        LayOutCommand(arrangement, handled, width, height, step);
        _poolLabel.Text = DeckEditorShown ? EditorSummary(arrangement) : PoolSummary(arrangement);
        DeckChanged();
        FlyPool();
    }

    /// <summary>"Your pool: 41 cards (7 Land, 34 Non-land, 19 Creatures)", or the same of the deck while it is built.</summary>
    private string PoolSummary(PoolArrangement arrangement)
    {
        List<ObjectView> cards = _poolIsDeck ? [.. arrangement.Main.Select(place => _poolCards[place])] : [.. _poolCards];
        int basics = _poolBasics.Sum(basic => basic.Count);
        int lands = cards.Count(card => card.CardTypes.Contains(CardTypeView.Land)) + basics;
        int creatures = cards.Count(card => card.CardTypes.Contains(CardTypeView.Creature));
        int total = cards.Count + basics;
        return $"{(_poolIsDeck ? "Your deck" : "Your pool")}: {total} card{(total == 1 ? string.Empty : "s")} ({lands} Land, {total - lands} Non-land, {creatures} Creature{(creatures == 1 ? string.Empty : "s")})";
    }

    private bool InPool(ViewHandle handle) => !_deckSubmitted && _poolPlaces.ContainsKey(handle.Value);

    /// <summary>The slot a point in the columns is: which column, and where among its other cards, as column * 1000 + row.</summary>
    private int ColumnSlot(ViewHandle carried, Point at)
    {
        if (_arrangement is not { } arrangement)
        {
            return 0;
        }

        int column = Math.Clamp((int)(at.X / (PoolWidth + PoolGap)), 0, arrangement.Columns.Count - 1);
        return (column * 1000) + SlotByTop(OtherTops(arrangement.Columns[column], carried, PoolHeaderHeight), CarriedTop(at));
    }

    private Rect? ColumnBar(ViewHandle carried, int slot)
    {
        if (_arrangement is not { } arrangement)
        {
            return null;
        }

        int column = slot / 1000;
        return new Rect(column * (PoolWidth + PoolGap), BarAt(OtherTops(arrangement.Columns[column], carried, PoolHeaderHeight), slot % 1000, PoolHeaderHeight) - 3, PoolWidth, 4);
    }

    /// <summary>
    /// Where the top of the card being carried is, in the row's terms: the pointer less where on the card it was taken
    /// hold of. A card goes in among the others by its top against theirs, as it is seen to be, not by the pointer.
    /// </summary>
    private float CarriedTop(Point at) => (float)(at.Y - _grabbed.Y);

    /// <summary>The tops of a column's cards as they are drawn, the one being carried left out (its place stays empty).</summary>
    private List<float> OtherTops(List<int> places, ViewHandle carried, float origin) =>
        [.. places.Select((place, row) => (place, row)).Where(pair => !IsCarried(pair.place, carried)).Select(pair => origin + (pair.row * PoolStep))];

    /// <summary>The place among the others a card whose top is here goes: after every card whose top is above its own.</summary>
    private static int SlotByTop(List<float> tops, float top) => tops.Count(other => other < top);

    /// <summary>Where the bar showing a slot goes: at the top of the card it would go before, or a step below the last.</summary>
    private float BarAt(List<float> tops, int slot, float origin) =>
        slot < tops.Count ? tops[slot] : tops.Count == 0 ? origin : tops[^1] + PoolStep;

    private int SideSlot(ViewHandle carried, Point at)
    {
        if (_arrangement is not { } arrangement || !_poolSideOpen)
        {
            return int.MaxValue;
        }

        return SlotByTop(OtherTops(arrangement.Side, carried, 0), CarriedTop(at));
    }

    private Rect? SideBar(ViewHandle carried, int slot) => _poolSideOpen && _arrangement is { } arrangement
        ? new Rect(0, Math.Max(0, BarAt(OtherTops(arrangement.Side, carried, 0), slot, 0) - 3), PoolWidth, 4)
        : new Rect(0, 0, SideClosedWidth - 14, _sideBoard.ActualHeight);

    private bool IsCarried(int place, ViewHandle carried) => _poolPlaces.GetValueOrDefault(carried.Value, -1) == place || _carriedPicks?.Contains(place) == true;

    /// <summary>Puts a card in a column of the main deck at a place among the others there.</summary>
    private void PutInColumn(ViewHandle handle, int column, int row)
    {
        if (InPack(handle))
        {
            PickInto(handle, side: false, column, row);
            return;
        }

        if (InSearch(handle))
        {
            AddFromSearch(handle, arrangement => arrangement.Columns[Math.Clamp(column, 0, arrangement.Columns.Count - 1)], row);
            return;
        }

        if (_arrangement is not { } arrangement || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return;
        }

        bool fromSide = arrangement.InSide(place);
        arrangement.Remove(place);
        List<int> into = arrangement.Columns[Math.Clamp(column, 0, arrangement.Columns.Count - 1)];
        into.Insert(Math.Clamp(row, 0, into.Count), place);
        PoolMoved(fromSide);
    }

    /// <summary>Puts a card in the sideboard: at a place among the others when it is open, at its foot when not.</summary>
    private void PutInSide(ViewHandle handle, int row)
    {
        if (InPack(handle))
        {
            PickInto(handle, side: true, 0, row);
            return;
        }

        if (InSearch(handle))
        {
            AddFromSearch(handle, arrangement => arrangement.Side, row);
            return;
        }

        if (_arrangement is not { } arrangement || !_poolPlaces.TryGetValue(handle.Value, out int place) || IsBasic(place))
        {
            return;
        }

        bool fromMain =!arrangement.InSide(place);
        arrangement.Remove(place);
        arrangement.Side.Insert(Math.Clamp(row, 0, arrangement.Side.Count), place);
        if (_dropPose is { } dropped)
        {
            IntoTuckedSide(handle, dropped);
        }

        PoolMoved(fromMain);
    }

    /// <summary>
    /// A click on a card of the pool: a second on the same card soon after moves it to the other side, or takes a basic
    /// land out of the deck.
    /// </summary>
    private void PoolClicked(CardBoardElement board, ViewHandle handle)
    {
        // In the deck editor, Ctrl or Shift held picks the card or lets it go; a plain click lets every pick go.
        if (PickClicked(board, handle))
        {
            return;
        }

        DateTime now = DateTime.UtcNow;
        bool twice = _lastPoolClick is { } last && last.Board == board && last.Handle == handle && now - last.At < TimeSpan.FromMilliseconds(450);
        _lastPoolClick = twice ? null : (board, handle, now);
        if (!twice || _deckSubmitted || _arrangement is not { } arrangement || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return;
        }

        if (IsBasic(place))
        {
            // This copy goes, not whichever the count would take: the others stay where they were put.
            arrangement.Remove(place);
            ChangeBasic(_poolBasics[BasicKind(place)].Land.Name, -1);
            return;
        }

        if (arrangement.InSide(place) || arrangement.Command.Contains(place))
        {
            arrangement.Remove(place);
            arrangement.Columns[DefaultColumn(_poolCards[place])].Add(place);
        }
        else
        {
            arrangement.Remove(place);
            arrangement.Side.Add(place);
            if (board.Bounds(handle) is { } box && board.XamlRoot is not null)
            {
                Point middle = board.TransformToVisual(Root).TransformPoint(new Point(box.X + (box.Width / 2), box.Y + (box.Height / 2)));
                IntoTuckedSide(handle, new CardFlights.Pose((float)middle.X, (float)middle.Y, PoolWidth, 0f));
            }
        }

        PoolMoved(sideChanged: true);
    }

    /// <summary>A card going into the sideboard while it is tucked away flies into the strip and fades there.</summary>
    private void IntoTuckedSide(ViewHandle handle, CardFlights.Pose from)
    {
        if (_poolSideOpen || _sideBoard.XamlRoot is null || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return;
        }

        Point strip = _sideBoard.TransformToVisual(Root).TransformPoint(new Point(_sideBoard.ActualWidth / 2, Math.Min(_sideBoard.ActualHeight / 2, 120)));
        _flights.Leave(handle, from, new CardFlights.Pose((float)strip.X, (float)strip.Y, (float)_sideBoard.ActualWidth, 0f), Handled(_poolCards[place], handle.Value));
    }

    /// <summary>A card was moved: laid out again, and sent when it changed which side of the deck it is on.</summary>
    private void PoolMoved(bool sideChanged)
    {
        if (DeckEditorShown)
        {
            DeckEdited();
            return;
        }

        if (sideChanged && _poolIsDeck)
        {
            SendDeck(submit: false);
            return;
        }

        LayOutPool();
    }

    /// <summary>The pool's cards to the flight layer, which flies a pile just taken into the columns.</summary>
    private void FlyPool()
    {
        if (!EventShown && !DeckEditorShown)
        {
            return;
        }

        (DeckEditorShown ? _deckPanel : _eventPanel).UpdateLayout();
        List<CardFlights.Placed> placed = [];
        foreach (CardBoardElement board in (CardBoardElement[])[_pileBoard, _gridBoard, _mainBoard, _sideBoard, _commandBoard, _searchBoard])
        {
            if (board.XamlRoot is null || board.ActualWidth <= 0 || !IsShown(board))
            {
                continue;
            }

            GeneralTransform toPage = board.TransformToVisual(Root);
            foreach ((ViewHandle handle, ObjectView card, float x, float y, float width, float angle) in board.Laid())
            {
                Point at = toPage.TransformPoint(new Point(x, y));
                placed.Add(new CardFlights.Placed(handle, card, new CardFlights.Pose((float)at.X, (float)at.Y, width, angle), board, x, y));
            }
        }

        // A card taken blind comes from the stack's tile.
        _flights.Look(placed, (zone, _) =>
        {
            if (zone != ZoneKindView.Library || _stackTile is not { ActualWidth: > 0, XamlRoot: not null } tile)
            {
                return null;
            }

            Point middle = tile.TransformToVisual(Root).TransformPoint(new Point(tile.ActualWidth / 2, tile.ActualHeight / 2));
            return new CardFlights.Pose((float)middle.X, (float)middle.Y, BackWidth, 0f);
        });
    }
}
