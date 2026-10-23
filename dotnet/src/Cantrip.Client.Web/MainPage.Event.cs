using Cantrip.Client;
using Cantrip.Protocol;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Automation;

namespace Cantrip.Client.Web;

/// <summary>
/// An event's table before its cards are dealt: the draft, and then building a deck from what was drafted. Over the
/// whole window, as the lobby is, but under a card held up to be read, so every card here can be read the way a card
/// on the table is - a right-press holds it up, and the pointer resting on it says what it does.
/// <para>
/// The draft shows the piles face down with how many cards each has, the stack, and whose turn it is; on this seat's
/// turn the pile it is looking at, face up and large, with Take and Pass. Below is the pool drafted so far, in columns
/// (<see cref="BuildPoolView"/>), which the cards taken fly into.
/// </para>
/// <para>
/// A grid draft shows its grid face up to both seats, with a button over each column and beside each row: on this
/// seat's pick one chooses that line, its cards lit, and Take takes it. Everything said about the grid is in a column
/// beside it, so the grid has the height of the window.
/// </para>
/// <para>
/// Deck building shows the same columns as the deck, with the sideboard beside them, and the basic lands to add and
/// Submit on the right. Every change to what is in the deck goes to the server as it is made, which keeps it, so a
/// reload or another computer picks it up where it was. Submit says it is the deck to play; the game is dealt when
/// everyone has.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    private const float PileWidth = 168f;
    private const float BackWidth = 64f;
    private const float EventGap = 10f;

    private static float Tall(float width) => MathF.Round(width * 88f / 63f);

    private readonly Grid _eventPanel = new()
    {
        Background = Theme.Brush(Theme.Window),
        Visibility = Visibility.Collapsed,
        Padding = new Thickness(20, 16, 20, 16),
    };

    private readonly CardBoardElement _pileBoard = new(PileWidth, Tall(PileWidth)) { HidesCarried = true, HorizontalAlignment = HorizontalAlignment.Left };

    private Grid _draftTop = null!;
    private Grid _eventHeader = null!;
    private StackPanel _looking = null!;
    private StackPanel _deckSide = null!;
    private TextBlock _eventTitle = null!;
    private TextBlock _eventSeats = null!;
    private TextBlock _draftStatus = null!;
    private StackPanel _pilesRow = null!;
    private StackPanel _draftRecent = null!;
    private StackPanel _pileButtons = null!;
    private StackPanel _basicsColumn = null!;
    private TextBlock _deckClock = null!;
    private TextBlock _deckStatus = null!;
    private DockButton _submitButton = null!;

    /// <summary>A booster draft's table, seen from above, where the Winston piles go.</summary>
    private readonly DraftTableElement _draftTable = new() { Width = 552, Height = 236, HorizontalAlignment = HorizontalAlignment.Left };

    /// <summary>
    /// A booster draft's table and its log of picks, with the event's name over the log: out of the way unless pulled
    /// down (<see cref="DraftTableOpenKey"/>), since the pack and the pool are what a pick is made from.
    /// </summary>
    private Border _draftTablePanel = null!;
    private TextBlock _draftTableTitle = null!;
    private DockButton _draftTableToggle = null!;
    private bool _draftTableOpen = LocalSettings.Flag(DraftTableOpenKey, false);
    private const string DraftTableOpenKey = "draft-table-open";

    /// <summary>The size the pack's cards are drawn at, the player's, as the pool's is.</summary>
    private Slider _packSlider = null!;
    private float _packScale = Math.Clamp(LocalSettings.Number(PackScaleKey, 1f), 0.6f, 1.6f);
    private const string PackScaleKey = "pack-card-scale";

    /// <summary>How long is left on the clock that matters now - the pick, deck building, sideboarding - counted down here.</summary>
    private TextBlock _eventClock = null!;
    private DateTime? _clockBy;
    private string _clockWhat = string.Empty;
    private readonly DispatcherTimer _eventTicker = new() { Interval = TimeSpan.FromMilliseconds(250) };

    /// <summary>The card of the pack in front of this seat chosen to pick, by handle, and the pick it was chosen for.</summary>
    private int? _packChoice;
    private int _packChoiceFor = -1;
    private (int Handle, DateTime At)? _lastPackClick;

    /// <summary>
    /// Whether this pick puts a card of the pool into the pack for another card from it (Cogwork Librarian,
    /// <see cref="DraftMessage.AdditionalPicks"/>), and the other cards chosen to go with the pick, by handle.
    /// </summary>
    private bool _packAdditional;
    private readonly List<int> _packAlso = [];

    /// <summary>The pick (<see cref="DraftMessage.Decision"/>) this page last sent a card, or word of none, for.</summary>
    private int _packSentFor = -1;

    /// <summary>The width a booster pack's cards are drawn at, before the player's scale.</summary>
    private const float PackWidth = 124f;

    private float PackCardWidth => MathF.Round(PackWidth * _packScale);

    /// <summary>
    /// A card of the pack dragged into the pool, waiting for the server to say it was taken: where it was put - a column
    /// and a place in it, or the sideboard - and where it was let go, which it flies from to there.
    /// </summary>
    private (int Handle, bool Side, int Column, int Row, CardFlights.Pose? From)? _pickInto;

    /// <summary>The copy of a card of the pack put in the pool, left where it was let go until the pick is answered.</summary>
    private CardBoardElement? _parkedPick;

    /// <summary>The width a grid draft's cards are drawn at, before the player's scale.</summary>
    private const float GridWidth = 104f;

    /// <summary>The thickness of the buttons that choose a grid's column, and of those beside its rows: room for the arrow.</summary>
    private const float LineButton = 26f;
    private const float RowButton = 30f;

    /// <summary>The space between a grid's cards, and between them and the buttons that choose their lines.</summary>
    private const float GridGap = 5f;
    private const float GridButtonGap = 4f;

    private float GridCardWidth => MathF.Round(GridWidth * _packScale);

    /// <summary>
    /// A grid draft's grid, face up, and the buttons over its columns and beside its rows. A card never moves along the
    /// grid: one drawn at another place than before is a copy that took over a handle - another copy of the card taken,
    /// or the next grid dealt - and is put there, not flown.
    /// </summary>
    private readonly CardBoardElement _gridBoard = new(GridWidth, Tall(GridWidth)) { FliesAlong = false, HidesCarried = true, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
    private Canvas _gridPicker = null!;

    /// <summary>
    /// Beside the grid, everything said about it: the event, what each seat has drafted, whose pick it is and the clock
    /// on it, Take, the log of picks and the card size - so the grid has the window's height to itself.
    /// </summary>
    private StackPanel _gridSide = null!;
    private TextBlock _gridTitle = null!;
    private TextBlock _gridSeats = null!;
    private TextBlock _gridStatus = null!;
    private TextBlock _gridClock = null!;
    private StackPanel _gridActions = null!;
    private Border _gridTools = null!;
    private StackPanel _boosterTools = null!;
    private Grid _statusRow = null!;

    /// <summary>The row (0 to 2) or column (3 to 5) of the grid chosen to take, and the pick it was chosen for.</summary>
    private int? _gridLine;
    private int _gridLineFor = -1;

    /// <summary>The pick (<see cref="DraftMessage.Decision"/>) this page last sent a line for.</summary>
    private int _gridSentFor = -1;

    /// <summary>
    /// The other seat's pick shown for a moment - the line it took lit on the grid it took it from - before the grid as
    /// it now is, so the pick is seen being made rather than only its cards gone.
    /// </summary>
    private readonly DispatcherTimer _gridHold = new() { Interval = TimeSpan.FromMilliseconds(1200) };

    /// <summary>The stack's tile, which a card taken blind flies from.</summary>
    private FrameworkElement? _stackTile;

    /// <summary>The messages last laid out, so a redraw with nothing new rebuilds nothing.</summary>
    private DraftMessage? _draftShown;
    private DeckBuildingMessage? _deckShown;

    /// <summary>The basic lands as this page has them, and whether the deck is submitted: the server's, with this page's changes.</summary>
    private Dictionary<string, int> _deckBasics = new(StringComparer.Ordinal);
    private bool _deckSubmitted;

    /// <summary>Plays this seat's draft and deck building under <c>?auto</c>.</summary>
    private DraftPlay? _selfDrafter;

    /// <summary>The event rows, as <see cref="CardBoards"/> lists them for reading, tooltips and tests.</summary>
    private IEnumerable<(CardBoardElement Board, Func<ViewHandle, CardLook?> Look)> EventBoards() =>
    [
        (_pileBoard, _ => null),
        (_gridBoard, _ => null),
        (_mainBoard, _ => null),
        (_sideBoard, _ => new CardLook(Note: "In your sideboard.")),
        (_commandBoard, _ => new CardLook(Note: "Your commander.")),
        (_companionBoard, _ => new CardLook(Note: "Your companion: a sideboard card, revealed before each game.")),
        (_searchBoard, _ => null),
    ];

    /// <summary>Whether the event panel is up: what a test reads as the page being ready before there is a game.</summary>
    private bool EventShown => _eventPanel.Visibility == Visibility.Visible;

    private void BuildEventPanel()
    {
        _eventPanel.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _eventPanel.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _eventPanel.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });

        var header = new Grid { Margin = new Thickness(0, 0, 0, 10) };
        _eventHeader = header;
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        _eventTitle = EventText(string.Empty, 22, Theme.Fg, semibold: true);
        _eventSeats = EventText(string.Empty, 14, Theme.FgSoft);
        _eventSeats.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(_eventSeats, 1);
        header.Children.Add(_eventTitle);
        header.Children.Add(_eventSeats);
        _eventPanel.Children.Add(header);

        _draftTop = BuildDraftTop();
        Grid.SetRow(_draftTop, 1);
        _eventPanel.Children.Add(_draftTop);

        var lower = new Grid { ColumnSpacing = 24 };
        lower.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        lower.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        lower.Children.Add(BuildPoolView());
        _deckSide = BuildDeckSide();
        Grid.SetColumn(_deckSide, 1);
        lower.Children.Add(_deckSide);
        Grid.SetRow(lower, 2);
        _eventPanel.Children.Add(lower);
    }

    private Grid BuildDraftTop()
    {
        var top = new Grid { RowSpacing = 10, Margin = new Thickness(0, 0, 0, 12) };
        foreach (GridLength height in (GridLength[])[GridLength.Auto, GridLength.Auto, GridLength.Auto])
        {
            top.RowDefinitions.Add(new RowDefinition { Height = height });
        }

        _draftStatus = EventText(string.Empty, 16, Theme.Accent, semibold: true);
        _eventClock = EventText(string.Empty, 16, Theme.Fg, semibold: true);
        AutomationProperties.SetName(_eventClock, "Clock");
        var status = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 18, VerticalAlignment = VerticalAlignment.Center };
        status.Children.Add(_draftStatus);
        status.Children.Add(_eventClock);

        // A booster draft's pack card size, and its table pulled down or put away.
        _packSlider = new Slider
        {
            Minimum = 0.6, Maximum = 1.6, StepFrequency = 0.05, Value = _packScale, Width = 160,
            VerticalAlignment = VerticalAlignment.Center, RequestedTheme = ElementTheme.Dark,
        };
        AutomationProperties.SetName(_packSlider, "Pack card size");
        ToolTipService.SetToolTip(_packSlider, "Card size");
        _packSlider.ValueChanged += (_, e) =>
        {
            _packScale = (float)e.NewValue;
            LocalSettings.Store(PackScaleKey, _packScale);
            _flights.Settle();
            if (_gameClient is { Draft: { Format: "booster" } draft } client)
            {
                LayOutPack(client, draft);
            }
            else if (_gameClient is { Draft: { Format: "grid" } grid } gridClient)
            {
                LayOutGrid(gridClient, grid);
            }
        };
        _draftTableToggle = new DockButton(string.Empty, primary: false) { VerticalAlignment = VerticalAlignment.Center };
        _draftTableToggle.Clicked += () =>
        {
            _draftTableOpen = !_draftTableOpen;
            LocalSettings.Store(DraftTableOpenKey, _draftTableOpen);
            ShowDraftTablePanel(booster: true);
        };
        _boosterTools = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16 };
        _boosterTools.Children.Add(_packSlider);
        _boosterTools.Children.Add(_draftTableToggle);
        _statusRow = new Grid();
        _statusRow.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        _statusRow.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Grid.SetColumn(_boosterTools, 1);
        _statusRow.Children.Add(status);
        _statusRow.Children.Add(_boosterTools);
        top.Children.Add(_statusRow);

        // The piles and the stack, face down, with what has happened beside them; or a booster draft's table, with the
        // event's name over its log of picks.
        var piles = new Grid { ColumnSpacing = 24 };
        piles.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        piles.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        _pilesRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 18 };
        _draftTableTitle = EventText(string.Empty, 16, Theme.Fg, semibold: true);
        _draftRecent = new StackPanel { Spacing = 2 };
        var log = new StackPanel { Spacing = 8, VerticalAlignment = VerticalAlignment.Top, Children = { _draftTableTitle, _draftRecent } };
        Grid.SetColumn(log, 1);
        piles.Children.Add(_pilesRow);
        piles.Children.Add(_draftTable);
        piles.Children.Add(log);
        _draftTablePanel = new Border { CornerRadius = new CornerRadius(6), Child = piles };
        Grid.SetRow(_draftTablePanel, 1);
        top.Children.Add(_draftTablePanel);

        // The pile or pack being looked at, face up, and what to do with it: beside a pile, under a pack.
        _looking = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 20, MinHeight = Tall(PileWidth) };
        _pileButtons = new StackPanel { Spacing = 10, VerticalAlignment = VerticalAlignment.Center, HorizontalAlignment = HorizontalAlignment.Left, Width = 260 };
        _gridPicker = new Canvas { Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
        _gridPicker.Children.Add(_gridBoard);
        _gridTitle = EventText(string.Empty, 20, Theme.Fg, semibold: true);
        _gridSeats = EventText(string.Empty, 14, Theme.FgSoft);
        _gridStatus = EventText(string.Empty, 16, Theme.Accent, semibold: true);
        _gridClock = EventText(string.Empty, 16, Theme.Fg, semibold: true);
        _gridActions = new StackPanel { Spacing = 8 };
        _gridTools = new Border { Margin = new Thickness(0, 4, 0, 0) };
        _gridSide = new StackPanel
        {
            Spacing = 8, Width = 340, Visibility = Visibility.Collapsed, VerticalAlignment = VerticalAlignment.Top,
            Children = { _gridTitle, _gridSeats, _gridStatus, _gridClock, _gridActions, _gridTools },
        };
        _looking.Children.Add(_pileBoard);
        _looking.Children.Add(_gridPicker);
        _looking.Children.Add(_gridSide);
        _looking.Children.Add(_pileButtons);
        Grid.SetRow(_looking, 2);
        top.Children.Add(_looking);

        _pileBoard.CardClicked += PackClicked;

        // A card of a booster pack can be dragged straight into the pool: a column of it, or the sideboard.
        MakeDraggable(_pileBoard);

        // The pack is laid out in rows as wide as the panel: again whenever that changes, and once it is first known.
        _eventPanel.SizeChanged += (_, _) =>
        {
            if (_gameClient is { Draft: { Format: "booster" } draft } client)
            {
                LayOutPack(client, draft);
            }
        };
        _eventTicker.Tick += (_, _) => ShowClock();
        _eventTicker.Start();
        return top;
    }

    private StackPanel BuildDeckSide()
    {
        var side = new StackPanel { Spacing = 10, Width = 250 };
        side.Children.Add(EventText("Basic lands", 14, Theme.Fg, semibold: true));
        _basicsColumn = new StackPanel { Spacing = 6 };
        side.Children.Add(_basicsColumn);
        _submitButton = new DockButton("Submit deck", primary: true).Filling();
        _submitButton.Margin = new Thickness(0, 12, 0, 0);
        _submitButton.Clicked += () => SendDeck(submit: !_deckSubmitted);
        side.Children.Add(_submitButton);
        _deckClock = EventText(string.Empty, 14, Theme.Accent, semibold: true);
        side.Children.Add(_deckClock);
        _deckStatus = EventText(string.Empty, 13, Theme.FgSoft);
        side.Children.Add(_deckStatus);
        return side;
    }

    /// <summary>Shows the draft or the deck being built while the table is at that point, and hides itself otherwise.</summary>
    private void ShowEvent(GameClient client)
    {
        TablePhase? phase = client.Table?.Phase;
        bool drafting = phase == TablePhase.Drafting && client.Draft is not null;
        bool building = phase == TablePhase.DeckBuilding && client.DeckBuilding is not null;
        _eventPanel.Visibility = drafting || building ? Visibility.Visible : Visibility.Collapsed;
        _draftTop.Visibility = drafting ? Visibility.Visible : Visibility.Collapsed;
        _eventHeader.Visibility = drafting && client.Draft!.Format is "booster" or "grid" ? Visibility.Collapsed : Visibility.Visible;
        _deckSide.Visibility = building ? Visibility.Visible : Visibility.Collapsed;

        if (drafting && !ReferenceEquals(_draftShown, client.Draft))
        {
            DraftMessage? before = _draftShown;
            _draftShown = client.Draft;
            ShowDraft(client, client.Draft!, before);
        }

        if (building && !ReferenceEquals(_deckShown, client.DeckBuilding))
        {
            UnparkPick();
            StartClock(client.DeckBuilding!.LeftMs, client.DeckBuilding.Title.Length > 0 ? "to sideboard" : "to build");
            // A view from before this page's latest changes came back shows a deck already changed here: who has
            // submitted is taken from it, and the deck is left as this page has it until the rest come back.
            bool first = _deckShown is null;
            _deckShown = client.DeckBuilding!;
            if (first && CarrySideboard(client, _deckShown))
            {
                ShowDeck(client, fromServer: false);
            }
            else
            {
                ShowDeck(client, fromServer: first || client.DeckCaughtUp);
            }
        }
    }

    private void ShowDraft(GameClient client, DraftMessage draft, DraftMessage? before)
    {
        bool booster = draft.Format == "booster";
        bool grid = draft.Format == "grid";
        _pileBoard.Visibility = grid ? Visibility.Collapsed : Visibility.Visible;
        _gridPicker.Visibility = grid ? Visibility.Visible : Visibility.Collapsed;
        _gridSide.Visibility = grid ? Visibility.Visible : Visibility.Collapsed;
        _pileButtons.Visibility = grid ? Visibility.Collapsed : Visibility.Visible;
        _statusRow.Visibility = grid ? Visibility.Collapsed : Visibility.Visible;
        _draftTop.RowSpacing = grid ? 0 : 10;

        // The card size goes with whichever draft is showing: beside a grid, or over a pack.
        if (grid && _gridTools.Child != _packSlider)
        {
            _boosterTools.Children.Remove(_packSlider);
            _gridTools.Child = _packSlider;
        }
        else if (!grid && _gridTools.Child == _packSlider)
        {
            _gridTools.Child = null;
            _boosterTools.Children.Insert(0, _packSlider);
        }

        if (grid)
        {
            _pilesRow.Visibility = Visibility.Collapsed;
            _draftTable.Visibility = Visibility.Collapsed;
            _draftTablePanel.Visibility = Visibility.Collapsed;
            _draftTableToggle.Visibility = Visibility.Collapsed;
            _packSlider.Visibility = Visibility.Visible;
            _looking.Orientation = Orientation.Horizontal;
            _looking.Spacing = 20;
            ShowGrid(client, draft, before);
            return;
        }

        _pilesRow.Visibility = booster ? Visibility.Collapsed : Visibility.Visible;
        _draftTable.Visibility = booster ? Visibility.Visible : Visibility.Collapsed;
        _draftTableTitle.Visibility = booster ? Visibility.Visible : Visibility.Collapsed;
        _packSlider.Visibility = booster ? Visibility.Visible : Visibility.Collapsed;
        _draftTableToggle.Visibility = booster ? Visibility.Visible : Visibility.Collapsed;
        _pileBoard.Draggable = booster;
        _looking.Orientation = booster ? Orientation.Vertical : Orientation.Horizontal;
        _looking.Spacing = booster ? 10 : 20;
        _draftStatus.FontSize = booster ? 22 : 16;
        _eventClock.FontSize = booster ? 22 : 16;
        ShowDraftTablePanel(booster);
        if (booster)
        {
            ShowBooster(client, draft);
            return;
        }

        string name = client.Table?.EventName is { Length: > 0 } eventName ? eventName : "Winston draft";
        _eventTitle.Text = $"{name}  ·  {draft.Total - draft.Stack - draft.Piles.Sum()} of {draft.Total} cards drafted";
        _eventSeats.Text = string.Join("   ", draft.Seats.Select(seat => $"{(seat.Seat == client.Seat ? "You" : seat.Name)}: {seat.Drafted}"));

        bool mine = draft.Active == client.Seat;
        string activeName = draft.Seats.FirstOrDefault(seat => seat.Seat == draft.Active)?.Name ?? "Your opponent";
        StartClock(draft.PickLeftMs, mine ? "to finish your turn" : $"for {activeName}'s turn");
        bool nextPile = draft.Piles.Skip(draft.Looking + 1).Any(count => count > 0);
        _draftStatus.Text = !mine
            ? $"{activeName} is looking at pile {draft.Looking + 1}."
            : !draft.CanPass
                ? $"Pile {draft.Looking + 1} is the last of the cards: take it."
                : nextPile
                    ? $"Your pick: take pile {draft.Looking + 1}, or pass it - a card from the stack goes onto it - and look at the next."
                    : $"Your pick: take pile {draft.Looking + 1}, or pass it and take the top card of the stack instead.";
        _draftStatus.Foreground = Theme.Brush(mine ? Theme.Accent : Theme.FgSoft);

        // The piles and the stack, face down, the pile being looked at lit.
        _pilesRow.Children.Clear();
        for (int i = 0; i < draft.Piles.Count; i++)
        {
            _pilesRow.Children.Add(PileStack($"Pile {i + 1}", draft.Piles[i], lit: i == draft.Looking));
        }

        _stackTile = PileStack("Stack", draft.Stack, lit: false);
        _pilesRow.Children.Add(_stackTile);

        _draftRecent.Children.Clear();
        foreach ((string line, IReadOnlyList<LogReferenceView> cards) in Recent(draft))
        {
            _draftRecent.Children.Add(Said(line, cards, _draftRecent.Children.Count == 0 ? Theme.Fg : Theme.FgMuted));
        }

        // The pile this seat is looking at, face up, under the handles its cards will keep in the pool.
        List<ObjectView> pile = mine ? PileHandled(draft.Pool, draft.Pile) : [];
        _pileBoard.SetCellSize(PileWidth, Tall(PileWidth));
        _pileBoard.Width = Math.Max(PileWidth, pile.Count * (PileWidth + EventGap));
        _pileBoard.Height = Tall(PileWidth);
        _pileBoard.SetCells([.. pile.Select((card, i) => new CardBoardElement.BoardCell(card, i * (PileWidth + EventGap), 0, Tapped: false))]);

        _pileButtons.Children.Clear();
        if (mine)
        {
            var take = new DockButton($"Take pile {draft.Looking + 1} ({pile.Count} card{(pile.Count == 1 ? string.Empty : "s")})", primary: true).Filling();
            take.Clicked += () => Pick(client, take: true);
            _pileButtons.Children.Add(take);
            if (draft.CanPass)
            {
                var pass = new DockButton(nextPile ? "Pass" : "Pass and draw from the stack", primary: false).Filling();
                pass.Clicked += () => Pick(client, take: false);
                _pileButtons.Children.Add(pass);
            }
        }
        else
        {
            _pileButtons.Children.Add(EventText($"Waiting for {activeName}.", 14, Theme.FgSoft));
        }

        // A card that joined the pool without having been shown in a pile was taken blind, from the stack.
        if (before is not null && draft.Pool.Count > before.Pool.Count)
        {
            HashSet<int> shown = [.. PileHandled(before.Pool, before.Pile).Select(card => card.Handle.Value)];
            Dictionary<string, int> copies = new(StringComparer.Ordinal);
            for (int place = 0; place < draft.Pool.Count; place++)
            {
                string card = draft.Pool[place].CardId;
                int copy = copies.GetValueOrDefault(card);
                copies[card] = copy + 1;
                int handle = EventHandle(card, copy);
                if (place >= before.Pool.Count && !shown.Contains(handle))
                {
                    _flights.Moved(new ObjectMovedEvent { Object = new ViewHandle(handle), From = ZoneKindView.Library, To = ZoneKindView.Hand });
                }
            }
        }

        ShowPool(draft.EventId, draft.Pool, serverMain: null, basics: []);
    }

    /// <summary>
    /// A booster draft: the table with everyone at it and the packs waiting for them, the pack in front of this seat to
    /// pick a card from, and the clock on it.
    /// </summary>
    private void ShowBooster(GameClient client, DraftMessage draft)
    {
        string name = client.Table?.EventName is { Length: > 0 } eventName ? eventName : "Booster draft";
        _draftTableTitle.Text = (draft.PackNumber > 0 ? $"{name}  ·  pack {draft.PackNumber} of {draft.Packs}" : name)
            + $"  ·  {draft.Pool.Count} card{(draft.Pool.Count == 1 ? string.Empty : "s")} drafted";

        string passing = draft.Direction > 0 ? "passing left" : "passing right";
        _draftTable.Show([.. draft.Seats.Select(seat => new DraftTableElement.Seat(seat.Seat, seat.Name, seat.Queued, seat.Drafted, PortraitOf(client, seat.Seat)))],
            client.Seat, draft.Direction, draft.PackNumber > 0 ? $"Pack {draft.PackNumber}  ·  {passing}" : string.Empty);

        bool mine = draft.Pile.Count > 0;
        if (_packChoiceFor != draft.Decision)
        {
            _packChoice = null;
            _packChoiceFor = draft.Decision;
            _packAdditional = false;
            _packAlso.Clear();
        }

        string pack = $"Pack {draft.PackNumber}/{draft.Packs}";
        _draftStatus.Text = mine
            ? $"{pack}, Pick {draft.PickNumber}/{draft.PackSize}"
            : $"{pack}, waiting for the next pack";
        _draftStatus.Foreground = Theme.Brush(mine ? Theme.Accent : Theme.FgSoft);
        StartClock(mine ? draft.PickLeftMs : 0, "to pick");

        _draftRecent.Children.Clear();
        foreach ((string line, IReadOnlyList<LogReferenceView> cards) in Recent(draft))
        {
            _draftRecent.Children.Add(Said(line, cards, _draftRecent.Children.Count == 0 ? Theme.Fg : Theme.FgMuted));
        }

        LayOutPack(client, draft);
        ShowPool(draft.EventId, draft.Pool, serverMain: null, basics: []);

        // The pick a card was put in the pool for is answered - taken, flying from where it was let go, or not - or
        // another pack is in front of this seat: the card waits there no longer.
        if (_pickInto is null || _packSentFor != draft.Decision)
        {
            UnparkPick();
        }

        // The clock ran out and the server asks for the card chosen: that card, or word that there is none.
        if (mine && draft.PickDue && _packSentFor != draft.Decision && !TakeFromPack(client))
        {
            _packSentFor = draft.Decision;
            client.PickNothing();
            _pileButtons.Children.Clear();
            _pileButtons.Children.Add(EventText("Taking...", 14, Theme.FgSoft));
        }
    }

    /// <summary>
    /// A grid draft: the grid, face up to both seats, whose pick it is, and on this seat's the line chosen to take; what
    /// each player took beside it.
    /// </summary>
    private void ShowGrid(GameClient client, DraftMessage draft, DraftMessage? before, bool held = false)
    {
        _gridHold.Stop();

        // From the server's word as it comes, not again once the other seat's pick has been held up to be seen.
        if (!held)
        {
            string picking = draft.Seats.FirstOrDefault(seat => seat.Seat == draft.Active)?.Name ?? "Your opponent";
            StartClock(draft.PickLeftMs, draft.Active == client.Seat ? "to pick" : $"for {picking} to pick");
        }

        // The other seat has just picked: the grid it picked from, its line lit, for a moment.
        if (before is { Format: "grid", Active: > 0 } picked && picked.Active != client.Seat && draft.Decision == picked.Decision + 1
            && draft.LastLine is >= 0 and < 6)
        {
            string picker = picked.Seats.FirstOrDefault(seat => seat.Seat == picked.Active)?.Name ?? "Your opponent";
            _gridStatus.Text = $"{picker} takes {LineName(draft.LastLine).ToLowerInvariant()}.";
            _gridStatus.Foreground = Theme.Brush(Theme.FgSoft);
            LayOutGrid(client, picked, lit: draft.LastLine);
            _gridHold.Tick -= GridHeld;
            _gridHold.Tick += GridHeld;
            _gridHold.Start();
            return;
        }

        string name = client.Table?.EventName is { Length: > 0 } eventName ? eventName : "Grid draft";
        _gridTitle.Text = draft.PackNumber > 0 ? $"{name}  ·  grid {draft.PackNumber} of {draft.Packs}" : name;
        _gridSeats.Text = string.Join("   ", draft.Seats.Select(seat => $"{(seat.Seat == client.Seat ? "You" : seat.Name)}: {seat.Drafted}"));

        if (_gridLineFor != draft.Decision)
        {
            _gridLine = null;
            _gridLineFor = draft.Decision;
        }

        bool mine = draft.Active == client.Seat;
        string activeName = draft.Seats.FirstOrDefault(seat => seat.Seat == draft.Active)?.Name ?? "Your opponent";
        _gridStatus.Text = mine
            ? draft.PickNumber == 1
                ? "Your pick: take a row or a column of the grid."
                : "Your pick: take a row or a column of what is left. The rest is discarded."
            : draft.PickNumber == 1
                ? $"{activeName} picks first from this grid."
                : $"{activeName} picks from what is left.";
        _gridStatus.Foreground = Theme.Brush(mine ? Theme.Accent : Theme.FgSoft);

        TossTakenLine(draft, before);
        LayOutGrid(client, draft);
        ShowPool(draft.EventId, draft.Pool, serverMain: null, basics: []);
    }

    /// <summary>The other seat's pick has been seen: the grid as it now is.</summary>
    private void GridHeld(object? sender, object e)
    {
        _gridHold.Stop();
        if (_gameClient is { Draft: { Format: "grid" } draft } client && client.Table?.Phase == TablePhase.Drafting)
        {
            ShowGrid(client, draft, before: null, held: true);
        }
    }

    /// <summary>
    /// This seat's pick has been taken: each card it put in the pool flies there from the very card taken. A handle goes
    /// by which copy of a card it is, so with another copy of a card left in the grid the handle that joins the pool can
    /// be the one the other copy was drawn under.
    /// </summary>
    private void TossTakenLine(DraftMessage draft, DraftMessage? before)
    {
        if (before is not { Format: "grid" } taken || draft.Decision != taken.Decision + 1 || draft.LastLine is not (>= 0 and < 6)
            || draft.Pool.Count <= taken.Pool.Count || _gridBoard.XamlRoot is null)
        {
            return;
        }

        List<int> known = [.. Enumerable.Range(0, taken.Pile.Count).Where(place => taken.Pile[place].Known)];
        List<ObjectView> handled = PileHandled(taken.Pool, [.. known.Select(place => taken.Pile[place])]);
        Dictionary<int, (float X, float Y, float Width, float Angle)> laid = _gridBoard.Laid().ToDictionary(card => card.Handle.Value, card => (card.X, card.Y, card.Width, card.Angle));
        GeneralTransform toPage = _gridBoard.TransformToVisual(Root);
        List<int> line = [.. GridPlaces(draft.LastLine).Where(known.Contains)];

        List<(string Card, int Copy)> keys = CopyKeys(draft.Pool);
        for (int place = taken.Pool.Count; place < keys.Count; place++)
        {
            int from = line.FindIndex(spot => taken.Pile[spot].CardId == keys[place].Card);
            if (from < 0)
            {
                continue;
            }

            int handle = handled[known.IndexOf(line[from])].Handle.Value;
            line.RemoveAt(from);
            if (laid.TryGetValue(handle, out var card))
            {
                Windows.Foundation.Point at = toPage.TransformPoint(new Windows.Foundation.Point(card.X, card.Y));
                _flights.Toss(new ViewHandle(EventHandle(keys[place].Card, keys[place].Copy)), new CardFlights.Pose((float)at.X, (float)at.Y, card.Width, card.Angle));
            }
        }
    }

    /// <summary>The places in the grid of a line: a row, 0 to 2, or a column, 3 to 5.</summary>
    private static IEnumerable<int> GridPlaces(int line) => line < 3
        ? Enumerable.Range(0, 3).Select(column => (line * 3) + column)
        : Enumerable.Range(0, 3).Select(row => (row * 3) + (line - 3));

    private static string LineName(int line) => line < 3 ? $"Row {line + 1}" : $"Column {line - 2}";

    /// <summary>The grid at the player's card size, the line chosen lit, its buttons, and Take with what was taken under it.</summary>
    /// <param name="lit">A line to light rather than the one chosen: the other seat's pick, being shown.</param>
    private void LayOutGrid(GameClient client, DraftMessage draft, int? lit = null)
    {
        bool mine = draft.Active == client.Seat;
        float width = GridCardWidth;
        float height = Tall(width);
        float step = width + GridGap;
        float rise = height + GridGap;
        float left = RowButton + GridButtonGap;
        float top = LineButton + GridButtonGap;

        // The grid's cards under the handles they will keep in the pool, each at its place; a taken place is left empty.
        List<int> known = [.. Enumerable.Range(0, draft.Pile.Count).Where(place => draft.Pile[place].Known)];
        List<ObjectView> handled = PileHandled(draft.Pool, [.. known.Select(place => draft.Pile[place])]);
        HashSet<int> lighting = (lit ?? _gridLine) is { } chosen ? [.. GridPlaces(chosen)] : [];
        _gridBoard.SetCellSize(width, height);
        _gridBoard.Width = (3 * step) - GridGap;
        _gridBoard.Height = (3 * rise) - GridGap;
        _gridBoard.SetCells([.. known.Select((place, i) => new CardBoardElement.BoardCell(handled[i],
            (place % 3) * step, (place / 3) * rise, Tapped: false, Selected: lighting.Contains(place)))]);
        Canvas.SetLeft(_gridBoard, left);
        Canvas.SetTop(_gridBoard, top);

        // A button over each column and beside each row, which choose it; only on this seat's pick, and only a line with
        // a card left in it.
        foreach (UIElement old in _gridPicker.Children.Where(child => child != _gridBoard).ToList())
        {
            _gridPicker.Children.Remove(old);
        }

        for (int line = 0; line < 6; line++)
        {
            bool row = line < 3;
            int cards = GridPlaces(line).Count(place => place < draft.Pile.Count && draft.Pile[place].Known);
            var button = new DockButton(row ? "▸" : "▾", primary: _gridLine == line, fontSize: 22, enabled: mine && cards > 0).Filling();
            button.Width = row ? RowButton : width;
            button.Height = row ? height : LineButton;
            AutomationProperties.SetName(button, LineName(line));
            ToolTipService.SetToolTip(button, cards == 0 ? $"{LineName(line)} has been taken." : $"{LineName(line)}: {cards} card{(cards == 1 ? string.Empty : "s")}");
            int chosenLine = line;
            button.Clicked += () => LineClicked(chosenLine);
            Canvas.SetLeft(button, row ? 0 : left + ((line - 3) * step));
            Canvas.SetTop(button, row ? top + (line * rise) : 0);
            _gridPicker.Children.Add(button);
        }

        _gridPicker.Width = left + _gridBoard.Width;
        _gridPicker.Height = top + _gridBoard.Height;

        _gridActions.Children.Clear();
        if (mine && draft.PackNumber > 0)
        {
            int count = _gridLine is { } line ? GridPlaces(line).Count(place => draft.Pile[place].Known) : 0;
            var take = new DockButton(_gridLine is { } taking ? $"Take {LineName(taking).ToLowerInvariant()} ({count} card{(count == 1 ? string.Empty : "s")})" : "Choose a row or a column",
                primary: true, enabled: _gridLine is not null).Filling();
            take.Clicked += () => TakeLine(client);
            _gridActions.Children.Add(take);
        }
        else if (draft.PackNumber > 0)
        {
            string activeName = draft.Seats.FirstOrDefault(seat => seat.Seat == draft.Active)?.Name ?? "your opponent";
            _gridActions.Children.Add(EventText($"Waiting for {activeName}.", 14, Theme.FgSoft));
        }

        foreach ((string said, IReadOnlyList<LogReferenceView> cards) in Recent(draft).Take(6))
        {
            _gridActions.Children.Add(Said(said, cards, _gridActions.Children.Count == 1 ? Theme.Fg : Theme.FgMuted));
        }
    }

    /// <summary>A row or column's button pressed: that line chosen, for Take to take.</summary>
    private void LineClicked(int line)
    {
        if (_gameClient is not { Draft: { Format: "grid" } draft } client || draft.Active != client.Seat || _gridSentFor == draft.Decision)
        {
            return;
        }

        _gridLine = line;
        _gridLineFor = draft.Decision;
        client.ChooseLine(line);
        LayOutGrid(client, draft);
    }

    /// <summary>Takes the row or column chosen, if there is one.</summary>
    private void TakeLine(GameClient client)
    {
        if (client.Draft is not { Format: "grid" } draft || draft.Active != client.Seat || _gridLine is not { } line
            || _gridLineFor != draft.Decision || _gridSentFor == draft.Decision)
        {
            return;
        }

        _gridSentFor = draft.Decision;
        client.PickLine(line);
        // Nothing more to click until the server says what happened.
        _gridActions.Children.Clear();
        _gridActions.Children.Add(EventText("Taking...", 14, Theme.FgSoft));
    }

    /// <summary>The pack in front of this seat, in rows as wide as the window allows, the card chosen lit.</summary>
    private void LayOutPack(GameClient client, DraftMessage draft)
    {
        List<ObjectView> pack = draft.Pile.Count > 0 ? PileHandled(draft.Pool, draft.Pile) : [];
        float width = PackCardWidth;
        float room = MathF.Max(width, (float)_eventPanel.ActualWidth - 40f);
        int across = Math.Max(1, (int)((room + EventGap) / (width + EventGap)));
        int rows = Math.Max(1, (pack.Count + across - 1) / across);
        _pileBoard.SetCellSize(width, Tall(width));
        _pileBoard.Width = Math.Max(width, Math.Min(pack.Count, across) * (width + EventGap));
        _pileBoard.Height = (rows * (Tall(width) + EventGap)) - EventGap;
        _pileBoard.SetCells([.. pack.Select((card, i) => new CardBoardElement.BoardCell(card,
            (i % across) * (width + EventGap), (i / across) * (Tall(width) + EventGap), Tapped: false,
            Selected: PackChosen().Contains(card.Handle.Value), Clickable: true))]);

        _pileButtons.Children.Clear();
        if (pack.Count == 0)
        {
            return;
        }

        List<string> chosen = [.. PackChosen().Select(handle => pack.FirstOrDefault(card => card.Handle.Value == handle)?.Name).OfType<string>()];
        var pick = new DockButton(chosen switch { [] => "Pick", [string one] => $"Pick {one}", _ => $"Pick {chosen.Count} cards" }, primary: true, enabled: chosen.Count > 0).Filling();
        pick.Clicked += () => TakeFromPack(client);
        _pileButtons.Children.Add(pick);

        // A Cogwork Librarian in the pool: put it into this pack for another card from it.
        if (draft.AdditionalPicks is [int first, ..] && first < draft.Pool.Count)
        {
            string spent = draft.Pool[first].Name;
            int most = Math.Min(draft.AdditionalPicks.Count, draft.Pile.Count - 1);
            var additional = new DockToggle(most == 1
                ? $"Use {spent} for a second card"
                : $"Use {spent} for up to {most} more cards")
            {
                Tooltip = $"Take another card from this pack with your pick, and put {spent} from your pool into the pack in its place.",
            };
            additional.SetOn(_packAdditional);
            additional.Toggled += on =>
            {
                _packAdditional = on;
                _packAlso.Clear();
                if (client.Draft is { } shown)
                {
                    LayOutPack(client, shown);
                }
            };
            _pileButtons.Children.Add(additional);
        }
    }

    /// <summary>The cards of the pack chosen to take, by handle: the pick first, then any taken with it.</summary>
    private List<int> PackChosen() => _packChoice is { } choice ? [choice, .. _packAlso] : [];

    /// <summary>
    /// Chooses a card of the pack: the pick, or - with a Cogwork Librarian being put into the pack - one more card to go
    /// with it, up to as many as there are to put in; a card already chosen is let go.
    /// </summary>
    private void ChoosePackCard(DraftMessage draft, int handle, bool toggle)
    {
        int more = _packAdditional ? draft.AdditionalPicks.Count : 0;
        if (more == 0 || _packChoice is null)
        {
            _packChoice = handle;
            _packAlso.Clear();
        }
        else if (handle == _packChoice || _packAlso.Contains(handle))
        {
            if (!toggle)
            {
                return;
            }

            if (_packAlso.Remove(handle))
            {
                return;
            }

            _packChoice = _packAlso is [int next, ..] ? next : null;
            if (_packAlso.Count > 0)
            {
                _packAlso.RemoveAt(0);
            }
        }
        else
        {
            if (_packAlso.Count >= more)
            {
                _packAlso.RemoveAt(_packAlso.Count - 1);
            }

            _packAlso.Add(handle);
        }

        _packChoiceFor = draft.Decision;
    }

    /// <summary>Shows a booster draft's table pulled down or put away, and how to do the other; a Winston draft's piles always.</summary>
    private void ShowDraftTablePanel(bool booster)
    {
        _draftTablePanel.Visibility = !booster || _draftTableOpen ? Visibility.Visible : Visibility.Collapsed;
        _draftTablePanel.Background = booster ? Theme.Brush(Theme.Panel) : null;
        _draftTablePanel.BorderBrush = booster ? Theme.Brush(Theme.Line) : null;
        _draftTablePanel.BorderThickness = new Thickness(booster ? 1 : 0);
        _draftTablePanel.Padding = new Thickness(booster ? 12 : 0);
        _draftTableToggle.Caption = _draftTableOpen ? "Hide draft table" : "Show draft table";
    }

    /// <summary>Takes away the copy of a card put in the pool from where it was let go, and shows it in the pack again if it is there.</summary>
    private void UnparkPick()
    {
        if (_parkedPick is null)
        {
            return;
        }

        _carryLayer.Children.Remove(_parkedPick);
        _parkedPick = null;
        _pileBoard.Carried = null;
    }

    /// <summary>Whether a card is in the pack in front of this seat waiting to be picked: one that can be dragged into the pool.</summary>
    private bool InPack(ViewHandle handle) =>
        _gameClient is { Draft: { Format: "booster", Pile.Count: > 0 } draft }
        && _packSentFor != draft.Decision
        && !_poolPlaces.ContainsKey(handle.Value)
        && _pileBoard.CardOf(handle) is not null;

    /// <summary>Picks a card of the pack dragged into the pool, to go where it was put once the server says it is taken.</summary>
    private void PickInto(ViewHandle handle, bool side, int column, int row)
    {
        if (_gameClient is not { Draft: { } draft } client)
        {
            return;
        }

        _pickInto = (handle.Value, side, column, row, _dropPose);

        // The card dragged is the pick; any others chosen go with it, to their own columns.
        List<int> others = [.. PackChosen().Where(other => other != handle.Value).Take(_packAdditional ? draft.AdditionalPicks.Count : 0)];
        _packChoice = handle.Value;
        _packAlso.Clear();
        _packAlso.AddRange(others);
        _packChoiceFor = draft.Decision;
        if (!TakeFromPack(client))
        {
            _pickInto = null;
        }
    }

    /// <summary>A card of the pack clicked: chosen, or - clicked twice - taken.</summary>
    private void PackClicked(ViewHandle handle)
    {
        if (_gameClient is not { Draft: { Format: "booster", Pile.Count: > 0 } draft } client)
        {
            return;
        }

        DateTime now = DateTime.UtcNow;
        bool twice = _lastPackClick is { } last && last.Handle == handle.Value && now - last.At < TimeSpan.FromMilliseconds(450);
        _lastPackClick = (handle.Value, now);
        ChoosePackCard(draft, handle.Value, toggle: !twice);
        if (twice)
        {
            TakeFromPack(client);
            return;
        }

        LayOutPack(client, draft);
    }

    /// <summary>Takes the card chosen from the pack in front of this seat, if there is one.</summary>
    /// <returns>Whether a card was chosen to take.</returns>
    private bool TakeFromPack(GameClient client)
    {
        if (client.Draft is not { Format: "booster", Pile.Count: > 0 } draft || _packChoice is not { } chosen)
        {
            return false;
        }

        List<ObjectView> pack = PileHandled(draft.Pool, draft.Pile);
        int place = pack.FindIndex(card => card.Handle.Value == chosen);
        if (place < 0)
        {
            return false;
        }

        List<int> also = _packAdditional
            ? [.. _packAlso.Select(handle => pack.FindIndex(card => card.Handle.Value == handle)).Where(other => other >= 0).Take(draft.AdditionalPicks.Count)]
            : [];
        _packSentFor = draft.Decision;
        client.PickCard(place, also);
        // Nothing more to click until the server says what happened.
        _pileButtons.Children.Clear();
        _pileButtons.Children.Add(EventText("Taking...", 14, Theme.FgSoft));
        return true;
    }

    /// <summary>Starts the clock shown counting down from what the server said is left; zero shows no clock.</summary>
    private void StartClock(int leftMs, string what)
    {
        _clockBy = leftMs > 0 ? DateTime.UtcNow + TimeSpan.FromMilliseconds(leftMs) : null;
        _clockWhat = what;
        ShowClock();
    }

    private void ShowClock()
    {
        string text = string.Empty;
        if (_clockBy is { } by)
        {
            TimeSpan left = by - DateTime.UtcNow;
            if (left < TimeSpan.Zero) left = TimeSpan.Zero;
            text = $"{(int)left.TotalMinutes}:{left.Seconds:00} {_clockWhat}";
        }

        var clockColour = Theme.Brush(_clockBy is { } soon && soon - DateTime.UtcNow < TimeSpan.FromSeconds(10) ? Theme.Danger : Theme.Fg);
        _eventClock.Text = text;
        _eventClock.Foreground = clockColour;
        _gridClock.Text = text;
        _gridClock.Foreground = clockColour;
        _deckClock.Text = text.Length > 0 ? $"{text}: the deck as it stands is submitted when time runs out." : string.Empty;
        ShowRoundClock();
        ShowMatchClocks();
    }

    private void Pick(GameClient client, bool take)
    {
        if (client.Draft is { } draft && draft.Active == client.Seat)
        {
            client.Pick(take);
            // Nothing more to click until the server says what happened.
            _pileButtons.Children.Clear();
            _pileButtons.Children.Add(EventText(take ? "Taking..." : "Passing...", 14, Theme.FgSoft));
        }
    }

    /// <summary>
    /// The sideboard made during the draft becomes the deck's, if this page has one and the server's deck is still the
    /// whole pool: sent at once, so the deck is built from where the draft left it. Returns whether it was sent.
    /// </summary>
    private bool CarrySideboard(GameClient client, DeckBuildingMessage building)
    {
        if (_arrangement is not { Side.Count: > 0 } arrangement || arrangement.EventId != building.EventId
            || client.DeckChangesSent > 0 || building.Main.Count != building.Pool.Count)
        {
            return false;
        }

        _deckBasics = new Dictionary<string, int>(StringComparer.Ordinal);
        _deckSubmitted = false;
        client.BuildDeck(arrangement.Main, _deckBasics, submit: false);
        return true;
    }

    /// <summary>A pile as a few card backs, one on another, with what it is and how many cards it has.</summary>
    private static FrameworkElement PileStack(string label, int count, bool lit)
    {
        float height = Tall(BackWidth);
        int shown = Math.Min(count, 3);
        var backs = new CardBoardElement(BackWidth, height) { FliesAlong = false, Width = BackWidth + 8, Height = height + 8, IsHitTestVisible = false };
        backs.SetCells([.. Enumerable.Range(0, shown).Select(i => new CardBoardElement.BoardCell(
            new ObjectView { Handle = new ViewHandle(9000 + i), Known = false }, i * 4f, 8f - (i * 4f), Tapped: false, Selected: lit && i == shown - 1))]);

        var column = new StackPanel { Spacing = 4, HorizontalAlignment = HorizontalAlignment.Center };
        column.Children.Add(count == 0
            ? new Border
            {
                Width = BackWidth, Height = height, Margin = new Thickness(0, 8, 8, 0), CornerRadius = new CornerRadius(4),
                BorderBrush = Theme.Brush(Theme.Line), BorderThickness = new Thickness(1),
            }
            : backs);
        column.Children.Add(EventText(label, 13, lit ? Theme.Accent : Theme.Fg, semibold: lit, centred: true));
        column.Children.Add(EventText(count == 1 ? "1 card" : $"{count} cards", 12, Theme.FgMuted, centred: true));
        return column;
    }

    /// <summary>Shows the deck: the server's when <paramref name="fromServer"/>, else this page's own as it stands.</summary>
    private void ShowDeck(GameClient client, bool fromServer)
    {
        if (_deckShown is not { } building)
        {
            return;
        }

        if (fromServer)
        {
            _deckBasics = building.Basics.ToDictionary(land => land.Name, land => land.Count, StringComparer.Ordinal);
            _deckSubmitted = building.Submitted;
        }

        _eventTitle.Text = building.Title.Length > 0 ? building.Title : _deckSubmitted ? "Your deck is submitted" : "Build your deck";
        _eventSeats.Text = string.Join("   ", building.Seats.Select(seat =>
            $"{(seat.Seat == client.Seat ? "You" : seat.Name)}: {(seat.Submitted ? "submitted" : "building")}"));

        // The server's deck is the deck only when it has caught up; otherwise the page's own sideboard stands.
        HashSet<int>? main = fromServer ? [.. building.Main] : null;
        ShowPool(building.EventId, building.Pool, main ?? (IReadOnlyCollection<int>)[.. _arrangement?.Main ?? []],
            [.. building.Basics.Select(land => (land, _deckBasics.GetValueOrDefault(land.Name)))],
            fromServer ? building.Companion : _arrangement?.Companion.FirstOrDefault(-1) ?? -1);
    }

    /// <summary>Brings the basic lands, Submit and what is said about the deck up to date with the deck as laid out.</summary>
    private void DeckChanged()
    {
        if (!_poolIsDeck || _deckShown is not { } building || _arrangement is not { } arrangement || _gameClient is not { } client)
        {
            return;
        }

        int size = arrangement.Main.Count() + _deckBasics.Values.Sum();
        _basicsColumn.Children.Clear();
        foreach (BasicLandView land in building.Basics)
        {
            _basicsColumn.Children.Add(BasicRow(land.Name, _deckBasics.GetValueOrDefault(land.Name)));
        }

        _submitButton.Caption = _deckSubmitted ? "Change my deck" : $"Submit deck ({size})";
        _submitButton.Enabled = _deckSubmitted || size >= building.Minimum;

        List<string> waiting = [.. building.Seats.Where(seat => seat.Seat != client.Seat && !seat.Submitted).Select(seat => seat.Name)];
        _deckStatus.Text = _deckSubmitted
            ? waiting.Count == 0 ? "Everyone has submitted. Dealing..." : $"Waiting for {string.Join(" and ", waiting)} to submit. You can still change your deck until then."
            : size < building.Minimum
                ? $"A deck needs at least {building.Minimum} cards. Add {building.Minimum - size} more to submit."
                : $"At least {building.Minimum} cards. Ready to submit.";
    }

    /// <summary>One basic land: how many, with buttons to add and take away one.</summary>
    private FrameworkElement BasicRow(string land, int count)
    {
        var row = new Grid { ColumnSpacing = 8 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(36) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(30) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(36) });

        var name = new SymbolText(13, Theme.Fg, semibold: false) { Text = $"{Pip(land)} {land}", VerticalAlignment = VerticalAlignment.Center };
        row.Children.Add(name);

        var less = new DockButton("−", primary: false, enabled: !_deckSubmitted && count > 0).Filling();
        AutomationProperties.SetName(less, $"Remove {land}");
        less.Clicked += () => ChangeBasic(land, -1);
        Grid.SetColumn(less, 1);
        row.Children.Add(less);

        TextBlock shown = EventText(count.ToString(System.Globalization.CultureInfo.InvariantCulture), 14, Theme.Fg, semibold: true, centred: true);
        shown.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(shown, 2);
        row.Children.Add(shown);

        var more = new DockButton("+", primary: false, enabled: !_deckSubmitted && count < 99).Filling();
        AutomationProperties.SetName(more, $"Add {land}");
        more.Clicked += () => ChangeBasic(land, +1);
        Grid.SetColumn(more, 3);
        row.Children.Add(more);
        return row;
    }

    private static string Pip(string land) => land switch
    {
        "Plains" => "{W}",
        "Island" => "{U}",
        "Swamp" => "{B}",
        "Mountain" => "{R}",
        "Forest" => "{G}",
        _ => string.Empty,
    };

    private void ChangeBasic(string land, int by)
    {
        if (_deckSubmitted)
        {
            return;
        }

        _deckBasics[land] = Math.Clamp(_deckBasics.GetValueOrDefault(land) + by, 0, 99);
        SendDeck(submit: false);
    }

    /// <summary>Sends the deck as it now stands, and shows it at once rather than when the server's answer comes.</summary>
    private void SendDeck(bool submit)
    {
        if (_gameClient is not { DeckBuilding: not null } client || _arrangement is not { } arrangement)
        {
            return;
        }

        client.BuildDeck(arrangement.Main, _deckBasics, submit, arrangement.Companion.FirstOrDefault(-1));
        _deckSubmitted = submit;
        ShowDeck(client, fromServer: false);
    }

    /// <summary>A card as printed, under a handle of its own on the row it is drawn in.</summary>
    private static ObjectView Handled(ObjectView printed, int handle)
    {
        ObjectView card = MessagePack.MessagePackSerializer.Deserialize<ObjectView>(MessagePack.MessagePackSerializer.Serialize(printed));
        card.Handle = new ViewHandle(handle);
        return card;
    }

    /// <summary>A draft's record of picks, newest first, each line with the cards it names.</summary>
    private static IEnumerable<(string Text, IReadOnlyList<LogReferenceView> Cards)> Recent(DraftMessage draft) =>
        draft.Recent.Select((line, i) => (line, (IReadOnlyList<LogReferenceView>)(i < draft.RecentCards.Count ? draft.RecentCards[i] : []))).Reverse();

    /// <summary>A line of a draft's record of picks, its cards named to be pointed at and read, as in the game log.</summary>
    private static SymbolText Said(string text, IReadOnlyList<LogReferenceView> cards, Windows.UI.Color color)
    {
        var said = new SymbolText(13, color, wrap: true);
        said.Show(text, cards);
        return said;
    }

    private static TextBlock EventText(string text, double size, Windows.UI.Color color, bool semibold = false, bool centred = false) => new()
    {
        Text = text,
        FontSize = size,
        FontWeight = semibold ? FontWeights.SemiBold : FontWeights.Normal,
        Foreground = Theme.Brush(color),
        TextWrapping = TextWrapping.Wrap,
        HorizontalAlignment = centred ? HorizontalAlignment.Center : HorizontalAlignment.Left,
        TextAlignment = centred ? TextAlignment.Center : TextAlignment.Left,
    };
}
