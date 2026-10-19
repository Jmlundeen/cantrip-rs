using System.Net;
using System.Runtime.InteropServices.JavaScript;
using System.Text;
using System.Text.Json;
using Cantrip.Client.Decks;
using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Microsoft.UI.Xaml.Automation;
using BinShape = Microsoft.UI.Xaml.Shapes.Path;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// The constructed deck editor (<c>?decks</c>, which the bot's site opens at <c>/decks</c>): a player's saved decks down
/// the left, the cards to build from on top, and the deck below in the same columns a draft's deck is built in
/// (<see cref="BuildPoolView"/>) - lands, then each mana value, the sideboard pulled out on the left, and for a Commander
/// deck the command zone beside it.
/// <para>
/// The cards are every card the engine plays, from the site's catalog (<see cref="CardCatalogView"/>), with where each is
/// legal: the search shows only what the deck's format allows, and for Commander, once a commander is chosen, only what
/// is within its color identity. A card is dragged from the results into a column, the sideboard or the command zone, or
/// double-clicked into the deck; dragged back onto the results, it leaves the deck. The deck is checked as it changes
/// (<see cref="DeckRules"/>) and said to be legal or why not, but it is saved however it stands.
/// </para>
/// <para>
/// Decks are kept on the site, for the player signed in with Discord: the list opens one, Save keeps it, the bin deletes
/// one after asking. Opening another deck with changes unsaved asks whether to save them first, and the browser asks
/// before the page is left with any (DeckEditor.js). Which column a card is in is this page's alone, as in a draft.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    /// <summary>The width the search results are drawn at, before the pool's card size.</summary>
    private const float SearchWidth = 100f;

    /// <summary>
    /// How many rows of results are laid out above and below those in view, so a card scrolled to is there before it
    /// shows: every result can be scrolled to, but only the rows near the view are made (<see cref="GridWindow"/>).
    /// </summary>
    private const int SpareRows = 2;

    /// <summary>Where the search results' handles start: one past every pool card's and basic land's.</summary>
    private const int SearchBase = 3_000_000;

    private const string DeckListOpenKey = "deck-list-open";

    private static readonly HttpClient DeckHttp = new();

    private static readonly string[] TypeNames = ["Any type", "Creature", "Instant", "Sorcery", "Artifact", "Enchantment", "Planeswalker", "Land", "Battle", "Legendary", "Can be a commander"];
    private static readonly string[] ManaNames = ["Any", "0", "1", "2", "3", "4", "5", "6", "7+"];
    private static readonly string[] SortNames = ["Name", "Mana value"];
    private static readonly (string Letter, string Name, ColorView? Color)[] ColorFilters =
    [
        ("W", "White", ColorView.White), ("U", "Blue", ColorView.Blue), ("B", "Black", ColorView.Black),
        ("R", "Red", ColorView.Red), ("G", "Green", ColorView.Green), ("C", "Colorless", null),
    ];

    private readonly Grid _deckPanel = new()
    {
        Background = Theme.Brush(Theme.Window),
        Visibility = Visibility.Collapsed,
        Padding = new Thickness(16, 12, 16, 12),
    };

    private readonly CardBoardElement _searchBoard = new(SearchWidth, Tall(SearchWidth)) { HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
    private ScrollViewer _searchScroll = null!;
    private TextBox _deckName = null!;
    private TextBox _searchText = null!;
    private ChoiceDropdown _formatChoice = null!;
    private ChoiceDropdown _typeChoice = null!;
    private ChoiceDropdown _manaChoice = null!;
    private ChoiceDropdown _sortChoice = null!;
    private readonly List<DockButton> _colorButtons = [];
    private DockButton _saveButton = null!;
    private DockButton _deckListToggle = null!;
    private TextBlock _editorStatus = null!;
    private TextBlock _editorProblems = null!;
    private TextBlock _searchCount = null!;
    private StackPanel _deckListRows = null!;
    private Border _deckListColumn = null!;
    private Grid _askOverlay = null!;
    private TextBlock _askTitle = null!;
    private TextBlock _askDetail = null!;
    private StackPanel _askButtons = null!;
    private bool _deckListOpen = LocalSettings.Flag(DeckListOpenKey, true);

    /// <summary>Every card a deck can be built from, by name and in name order; null until the site has sent them.</summary>
    private CardCatalogView? _catalog;
    private Dictionary<string, CatalogCard> _catalogByName = new(StringComparer.Ordinal);
    private List<CatalogCard> _catalogCards = [];

    /// <summary>Each card's place in <see cref="_catalogCards"/>, which its search handle is made from.</summary>
    private Dictionary<CatalogCard, int> _catalogPlaces = [];
    private string? _catalogFailure;

    /// <summary>
    /// The deck being edited: its id (empty until it is first saved), its format, and what it was when last saved or
    /// opened, which telling an unsaved change goes by. Its cards are the pool's (<see cref="_editorCards"/>).
    /// </summary>
    private string _editingId = string.Empty;
    private DeckFormat _editingFormat = DeckFormat.All[0];
    private string _savedSnapshot = string.Empty;
    private int _decksOpened;

    /// <summary>
    /// Every card put in the deck since it was opened, by place: the pool's cards. One taken out is gone from the
    /// arrangement and stays here, so every other card keeps its place and its handle.
    /// </summary>
    private readonly List<ObjectView> _editorCards = [];

    /// <summary>The player's saved decks, as the site lists them; null when nobody is signed in, and before the site answers.</summary>
    private List<DeckSummary>? _savedDecks;
    private bool _signedOut;

    /// <summary>Every card the search finds, by handle, whether or not its row is laid out now.</summary>
    private readonly Dictionary<int, CatalogCard> _searchCards = [];

    /// <summary>Every card the search finds, in the order shown; only the rows near the view are laid out.</summary>
    private List<CatalogCard> _searchFound = [];

    /// <summary>How many of each card the deck has, by name, as written over its result.</summary>
    private Dictionary<string, int> _searchInDeck = new(StringComparer.Ordinal);

    /// <summary>The results' grid as last laid out, and the cells of it laid out now.</summary>
    private GridWindow _searchGrid;
    private (int First, int End) _searchWindow = (0, 0);
    private readonly Dictionary<int, ObjectView> _searchViews = [];
    private (ViewHandle Handle, DateTime At)? _lastSearchClick;

    private bool DeckEditorShown => _deckPanel.Visibility == Visibility.Visible;

    private bool EditingCommander => DeckEditorShown && _editingFormat.IsCommander;

    private bool InSearch(ViewHandle handle) => DeckEditorShown && _searchCards.ContainsKey(handle.Value);

    private void BuildDeckEditor()
    {
        _deckPanel.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _deckPanel.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _deckPanel.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        _deckPanel.Children.Add(BuildEditorHeader());

        _editorProblems = EventText(string.Empty, 12, Theme.Danger);
        _editorProblems.Margin = new Thickness(0, 0, 0, 8);
        Grid.SetRow(_editorProblems, 1);
        _deckPanel.Children.Add(_editorProblems);

        var body = new Grid { ColumnSpacing = 14 };
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        Grid.SetRow(body, 2);
        _deckPanel.Children.Add(body);

        _deckListColumn = BuildDeckList();
        body.Children.Add(_deckListColumn);

        var work = new Grid { RowSpacing = 8 };
        work.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        work.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        work.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1.35, GridUnitType.Star) });
        Grid.SetColumn(work, 1);
        body.Children.Add(work);

        work.Children.Add(BuildSearchFilters());
        _searchScroll = new ScrollViewer
        {
            HorizontalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            Content = _searchBoard,
        };
        var results = new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(8),
            Child = _searchScroll,
        };
        Grid.SetRow(results, 1);
        work.Children.Add(results);
        _searchScroll.SizeChanged += (_, _) => LayOutSearch();
        _searchScroll.ViewChanged += (_, _) => PlaceSearch(again: false);

        // The deck in the draft's columns: the same view, moved here, since this page never shows an event.
        if (_poolView.Parent is Panel holder)
        {
            holder.Children.Remove(_poolView);
        }

        _poolHint.Text = "Drag cards from the search into the deck, between columns and to the sideboard  ·  double-click to move one to or from the sideboard  ·  drag a card back onto the search to take it out";
        Grid.SetRow(_poolView, 2);
        work.Children.Add(_poolView);

        // The search results are a pack to take from, and somewhere to put a card of the deck to take it out.
        MakeDraggable(_searchBoard);
        MakePickable();
        _searchBoard.CardClicked += SearchClicked;
        _dropZones.Add(new DropZone(_searchBoard, InPool, (handle, _) => TakeOut(handle)));
        MakeDraggable(_commandBoard);
        _commandBoard.CardClicked += handle => PoolClicked(_commandBoard, handle);
        _dropZones.Add(new DropZone(_commandBoard, CanCommand, (handle, row) => PutInCommand(handle, row)));

        _askOverlay = BuildAsk();
        Grid.SetRowSpan(_askOverlay, 3);
        _deckPanel.Children.Add(_askOverlay);
    }

    private Grid BuildEditorHeader()
    {
        var header = new Grid { ColumnSpacing = 12, Margin = new Thickness(0, 0, 0, 8) };
        foreach (GridLength width in (GridLength[])[GridLength.Auto, GridLength.Auto, GridLength.Auto, GridLength.Auto, GridLength.Auto, GridLength.Auto, new GridLength(1, GridUnitType.Star), GridLength.Auto])
        {
            header.ColumnDefinitions.Add(new ColumnDefinition { Width = width });
        }

        _deckListToggle = new DockButton(string.Empty, primary: false) { VerticalAlignment = VerticalAlignment.Center };
        _deckListToggle.Clicked += () =>
        {
            _deckListOpen = !_deckListOpen;
            LocalSettings.Store(DeckListOpenKey, _deckListOpen);
            ShowDeckList();
        };

        var title = EventText("Deck Editor", 20, Theme.Fg, semibold: true);
        title.VerticalAlignment = VerticalAlignment.Center;

        _deckName = new TextBox { PlaceholderText = "Deck name", Width = 260, FontSize = 14, MaxLength = 100, VerticalAlignment = VerticalAlignment.Center, RequestedTheme = ElementTheme.Dark };
        AutomationProperties.SetName(_deckName, "Deck name");
        _deckName.TextChanged += (_, _) => DeckEdited();

        _formatChoice = new ChoiceDropdown("Format", "The format the deck is built for", [.. DeckFormat.All.Select(format => format.Title)]);
        _formatChoice.Changed = index => ChangeFormat(DeckFormat.All[index]);

        _saveButton = new DockButton("Save", primary: true) { VerticalAlignment = VerticalAlignment.Center };
        _saveButton.Clicked += () => _ = SaveDeckAsync();

        var fresh = new DockButton("New deck", primary: false) { VerticalAlignment = VerticalAlignment.Center };
        fresh.Clicked += () => WhenSaved(() =>
        {
            OpenDeck(new SavedDeck { Name = string.Empty, Format = _editingFormat.Key });
            return Task.CompletedTask;
        });

        _editorStatus = EventText(string.Empty, 13, Theme.FgSoft, semibold: true);
        _editorStatus.VerticalAlignment = VerticalAlignment.Center;

        // Back to the site's deck library, where the editor is opened from.
        var home = new DockButton("All decks", primary: false) { VerticalAlignment = VerticalAlignment.Center };
        home.Clicked += () => DeckEditorJs.Go("decks");

        FrameworkElement[] parts = [_deckListToggle, title, _deckName, _formatChoice, _saveButton, fresh, _editorStatus, home];
        for (int column = 0; column < parts.Length; column++)
        {
            Grid.SetColumn(parts[column], column);
            header.Children.Add(parts[column]);
        }

        return header;
    }

    private FrameworkElement BuildSearchFilters()
    {
        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12 };
        _searchText = new TextBox { PlaceholderText = "Search cards: name, type or rules text", Width = 300, FontSize = 13, VerticalAlignment = VerticalAlignment.Center, RequestedTheme = ElementTheme.Dark };
        _searchText.TextChanged += (_, _) => SearchChanged();
        row.Children.Add(_searchText);

        _typeChoice = new ChoiceDropdown("Type", "Only cards of a type", TypeNames) { Changed = _ => SearchChanged() };
        row.Children.Add(_typeChoice);

        var colors = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4, VerticalAlignment = VerticalAlignment.Center };
        foreach ((string letter, string name, _) in ColorFilters)
        {
            var button = new DockButton(letter, primary: false, fontSize: 12) { Tooltip = $"{name} cards" };
            AutomationProperties.SetName(button, name);
            button.Clicked += () =>
            {
                button.On = !button.On;
                SearchChanged();
            };
            _colorButtons.Add(button);
            colors.Children.Add(button);
        }

        row.Children.Add(colors);
        _manaChoice = new ChoiceDropdown("Mana value", "Only cards of a mana value", ManaNames) { Changed = _ => SearchChanged() };
        row.Children.Add(_manaChoice);
        _sortChoice = new ChoiceDropdown("Sort", "The order the cards are shown in", SortNames) { Changed = _ => SearchChanged() };
        row.Children.Add(_sortChoice);
        _searchCount = EventText(string.Empty, 12, Theme.FgMuted);
        _searchCount.VerticalAlignment = VerticalAlignment.Center;
        row.Children.Add(_searchCount);

        return new ScrollViewer
        {
            HorizontalScrollMode = ScrollMode.Auto,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Hidden,
            VerticalScrollMode = ScrollMode.Disabled,
            VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
            Content = row,
        };
    }

    private Border BuildDeckList()
    {
        _deckListRows = new StackPanel { Spacing = 6 };
        return new Border
        {
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(10),
            Width = 250,
            Child = new ScrollViewer
            {
                HorizontalScrollMode = ScrollMode.Disabled,
                HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
                VerticalScrollMode = ScrollMode.Auto,
                VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
                Content = _deckListRows,
            },
        };
    }

    /// <summary>A question over the editor, answered with one of its buttons: unsaved changes, a deck to delete.</summary>
    private Grid BuildAsk()
    {
        _askTitle = EventText(string.Empty, 17, Theme.Fg, semibold: true);
        _askDetail = EventText(string.Empty, 13, Theme.FgSoft);
        _askButtons = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 10, HorizontalAlignment = HorizontalAlignment.Right };
        var box = new Border
        {
            Background = Theme.Brush(Theme.Bar),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(8),
            Padding = new Thickness(20, 16, 20, 16),
            MaxWidth = 460,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Child = new StackPanel { Spacing = 14, Children = { _askTitle, _askDetail, _askButtons } },
        };
        return new Grid
        {
            Background = Theme.Brush(Theme.WithAlpha(Windows.UI.Color.FromArgb(255, 0, 0, 0), 0.55)),
            Visibility = Visibility.Collapsed,
            Children = { box },
        };
    }

    /// <summary>Asks a question over the editor; each answer closes it and does what it says (nothing for one with no action).</summary>
    private void Ask(string title, string detail, params (string Caption, bool Primary, Func<Task>? Then)[] answers)
    {
        _askTitle.Text = title;
        _askDetail.Text = detail;
        _askDetail.Visibility = detail.Length == 0 ? Visibility.Collapsed : Visibility.Visible;
        _askButtons.Children.Clear();
        foreach ((string caption, bool primary, Func<Task>? then) in answers)
        {
            var button = new DockButton(caption, primary);
            button.Clicked += async () =>
            {
                _askOverlay.Visibility = Visibility.Collapsed;
                if (then is not null)
                {
                    await then();
                }
            };
            _askButtons.Children.Add(button);
        }

        _askOverlay.Visibility = Visibility.Visible;
    }

    /// <summary>Puts the editor up in place of a table, and fetches the cards and the player's decks.</summary>
    private async Task OpenDeckEditorAsync()
    {
        _deckPanel.Visibility = Visibility.Visible;
        BrowserTitle.Show("Cantrip - Deck Editor");
        OpenDeck(new SavedDeck { Name = string.Empty, Format = DeckFormat.Find(ClientConfig.DeckFormatKey)?.Key ?? DeckFormat.StandardKey });
        ShowDeckList();
        await Task.WhenAll(LoadCatalogAsync(), RefreshDeckListAsync());

        // Opened from the site's deck library on one of the player's decks.
        if (ClientConfig.DeckId is { } id && _savedDecks?.Any(deck => deck.Id == id) == true)
        {
            await OpenSavedAsync(id);
        }
    }

    private async Task LoadCatalogAsync()
    {
        _editorStatus.Text = "Loading the cards...";
        try
        {
            using HttpResponseMessage response = await DeckHttp.GetAsync(DeckApi("catalog"));
            response.EnsureSuccessStatusCode();
            byte[] bytes = await response.Content.ReadAsByteArrayAsync();
            CardCatalogView catalog = MessagePack.MessagePackSerializer.Deserialize<CardCatalogView>(bytes);
            _catalogByName = new Dictionary<string, CatalogCard>(StringComparer.Ordinal);
            foreach (CatalogCard card in catalog.Cards)
            {
                _catalogByName.TryAdd(card.Name, card);
            }

            _catalogCards = [.. _catalogByName.Values.OrderBy(card => card.Name, StringComparer.OrdinalIgnoreCase)];
            _catalogPlaces = _catalogCards.Select((card, place) => (card, place)).ToDictionary(pair => pair.card, pair => pair.place);
            _catalog = catalog;
            _catalogFailure = null;
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or MessagePack.MessagePackSerializationException)
        {
            _catalogFailure = "The cards could not be loaded from the site. Reload to try again.";
        }

        // A deck opened before the cards came is shown again with its cards as printed.
        RebuildCards();
        DeckEdited();
    }

    private async Task RefreshDeckListAsync()
    {
        try
        {
            using HttpResponseMessage response = await DeckHttp.GetAsync(DeckApi("list"));
            if (response.StatusCode == HttpStatusCode.Unauthorized)
            {
                _signedOut = true;
                _savedDecks = null;
            }
            else
            {
                response.EnsureSuccessStatusCode();
                _signedOut = false;
                _savedDecks = JsonSerializer.Deserialize(await response.Content.ReadAsStringAsync(), DeckJson.Default.ListDeckSummary) ?? [];
            }
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or JsonException)
        {
            _savedDecks = null;
        }

        ShowDeckList();
    }

    private static Uri DeckApi(string path) => new(ClientConfig.Page, "decks/api/" + path);

    /// <summary>The player's decks by format, the one open lit, each with its bin; or a way to sign in to have any.</summary>
    private void ShowDeckList()
    {
        _deckListColumn.Visibility = _deckListOpen ? Visibility.Visible : Visibility.Collapsed;
        _deckListToggle.Caption = _deckListOpen ? "‹ Decks" : "› Decks";
        AutomationProperties.SetName(_deckListToggle, _deckListOpen ? "Hide decks" : "Show decks");
        _deckListRows.Children.Clear();
        _deckListRows.Children.Add(EventText("My decks", 15, Theme.Fg, semibold: true));

        if (_signedOut)
        {
            _deckListRows.Children.Add(EventText("Log in with Discord to save your decks and open them again.", 12, Theme.FgSoft));
            var signIn = new DockButton("Log in with Discord", primary: true).Filling();
            signIn.Clicked += () => DeckEditorJs.Go("signin?then=decks/edit");
            _deckListRows.Children.Add(signIn);
            return;
        }

        if (_savedDecks is null)
        {
            _deckListRows.Children.Add(EventText("Loading your decks...", 12, Theme.FgMuted));
            return;
        }

        if (_savedDecks.Count == 0)
        {
            _deckListRows.Children.Add(EventText("No decks saved yet. Build one and press Save.", 12, Theme.FgMuted));
            return;
        }

        foreach (DeckFormat format in DeckFormat.All)
        {
            List<DeckSummary> decks = [.. _savedDecks.Where(deck => deck.Format == format.Key)];
            if (decks.Count == 0)
            {
                continue;
            }

            TextBlock heading = EventText(format.Title.ToUpperInvariant(), 11, Theme.FgSoft, semibold: true);
            heading.Margin = new Thickness(0, 8, 0, 0);
            _deckListRows.Children.Add(heading);
            foreach (DeckSummary deck in decks)
            {
                _deckListRows.Children.Add(DeckRow(deck));
            }
        }
    }

    private Grid DeckRow(DeckSummary deck)
    {
        var row = new Grid { ColumnSpacing = 6 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        var open = new DockButton($"{deck.Name}  ·  {deck.Cards}", primary: false, fontSize: 12).Filling();
        open.On = deck.Id == _editingId;
        AutomationProperties.SetName(open, $"Open {deck.Name}");
        open.Tooltip = $"Open {deck.Name}";
        open.Clicked += () => WhenSaved(() => OpenSavedAsync(deck.Id));
        row.Children.Add(open);

        var bin = new BinButton($"Delete {deck.Name}");
        bin.Clicked += () => Ask($"Delete {deck.Name}?", "The deck is deleted for good. This can't be undone.",
            ("Delete", true, () => DeleteDeckAsync(deck)),
            ("Cancel", false, null));
        Grid.SetColumn(bin, 1);
        row.Children.Add(bin);
        return row;
    }

    /// <summary>Does something that puts another deck in the editor - once the changes to this one are saved or let go, if there are any.</summary>
    private void WhenSaved(Func<Task> then)
    {
        if (!DeckUnsaved())
        {
            _ = then();
            return;
        }

        string name = DeckNameShown();
        Ask($"Save the changes to {name}?", "The deck you're editing has changes that aren't saved.",
            ("Save", true, async () =>
            {
                if (await SaveDeckAsync())
                {
                    await then();
                }
            }),
            ("Discard", false, then),
            ("Cancel", false, null));
    }

    private async Task OpenSavedAsync(string id)
    {
        try
        {
            using HttpResponseMessage response = await DeckHttp.GetAsync(DeckApi("deck/" + Uri.EscapeDataString(id)));
            if (response.StatusCode == HttpStatusCode.NotFound)
            {
                _editorStatus.Text = "That deck is gone.";
                await RefreshDeckListAsync();
                return;
            }

            response.EnsureSuccessStatusCode();
            if (JsonSerializer.Deserialize(await response.Content.ReadAsStringAsync(), DeckJson.Default.SavedDeck) is { } deck)
            {
                OpenDeck(deck);
                ShowDeckList();
            }
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or JsonException)
        {
            _editorStatus.Text = "The deck could not be opened. Try again.";
        }
    }

    /// <summary>Puts a deck in the editor, its cards in their default columns, as it was saved.</summary>
    private void OpenDeck(SavedDeck deck)
    {
        _editingId = deck.Id;
        _editingFormat = DeckFormat.Find(deck.Format) ?? DeckFormat.All[0];
        _formatChoice.SetSelected(DeckFormat.All.ToList().IndexOf(_editingFormat));
        _deckName.Text = deck.Name;

        var arrangement = new PoolArrangement { EventId = $"deck-{++_decksOpened}" };
        _editorCards.Clear();
        _deckPicks.Clear();
        foreach (DeckEntry entry in deck.Commanders)
        {
            for (int copy = 0; copy < entry.Count; copy++) arrangement.Command.Add(AddCard(entry.Card));
        }

        foreach (DeckEntry entry in deck.Main)
        {
            for (int copy = 0; copy < entry.Count; copy++)
            {
                int place = AddCard(entry.Card);
                arrangement.Columns[DefaultColumn(_editorCards[place])].Add(place);
            }
        }

        foreach (DeckEntry entry in deck.Side)
        {
            for (int copy = 0; copy < entry.Count; copy++) arrangement.Side.Add(AddCard(entry.Card));
        }

        _arrangement = arrangement;
        _poolCards = _editorCards;
        _poolIsDeck = true;
        _poolBasics = [];
        _savedSnapshot = Snapshot(CurrentDeck());
        SettleCards();
        DeckEdited();
    }

    /// <summary>Adds a card to the pool by name, as printed; a card the catalog does not have (yet) by its name alone.</summary>
    private int AddCard(string name)
    {
        _editorCards.Add(_catalogByName.TryGetValue(name, out CatalogCard? card) ? card.Card : new ObjectView { Known = true, CardId = name, Name = name });
        return _editorCards.Count - 1;
    }

    /// <summary>The cards of a deck opened before the catalog came, as printed now that it has.</summary>
    private void RebuildCards()
    {
        for (int place = 0; place < _editorCards.Count; place++)
        {
            if (_catalogByName.TryGetValue(_editorCards[place].Name, out CatalogCard? card))
            {
                _editorCards[place] = card.Card;
            }
        }
    }

    /// <summary>The deck as the editor has it now: name, format and the cards in the command zone, the columns and the sideboard.</summary>
    private SavedDeck CurrentDeck() => new()
    {
        Id = _editingId,
        Name = _deckName.Text.Trim(),
        Format = _editingFormat.Key,
        Commanders = Entries(_arrangement?.Command ?? []),
        Main = Entries(_arrangement?.Columns.SelectMany(column => column) ?? []),
        Side = Entries(_arrangement?.Side ?? []),
    };

    private List<DeckEntry> Entries(IEnumerable<int> places) =>
        [.. places.GroupBy(place => _editorCards[place].Name, StringComparer.Ordinal)
            .Select(named => new DeckEntry(named.Key, named.Count()))
            .OrderBy(entry => entry.Card, StringComparer.Ordinal)];

    /// <summary>What telling a change from the saved deck goes by: everything but where in the columns each card is.</summary>
    private static string Snapshot(SavedDeck deck)
    {
        var plain = new SavedDeck { Name = deck.Name, Format = deck.Format, Commanders = deck.Commanders, Main = deck.Main, Side = deck.Side };
        return JsonSerializer.Serialize(plain, DeckJson.Default.SavedDeck);
    }

    private bool DeckUnsaved() => DeckEditorShown && _arrangement is not null && Snapshot(CurrentDeck()) != _savedSnapshot;

    private string DeckNameShown() => _deckName.Text.Trim() is { Length: > 0 } name ? name : "this deck";

    /// <summary>The deck changed - a card, the name, the format: checked again, the search refreshed, and the browser told whether to ask before leaving.</summary>
    private void DeckEdited()
    {
        if (!DeckEditorShown || _arrangement is null)
        {
            return;
        }

        bool unsaved = DeckUnsaved();
        DeckEditorJs.Unsaved(unsaved);
        _saveButton.Caption = unsaved || _editingId.Length == 0 ? "Save" : "Saved";

        SavedDeck deck = CurrentDeck();
        IReadOnlyList<string> problems = _catalog is null ? [] : DeckRules.Problems(deck, _catalogByName);
        string state = _catalogFailure ?? (_catalog is null
            ? "Loading the cards..."
            : problems.Count == 0 ? $"Legal in {_editingFormat.Title}" : $"Not legal in {_editingFormat.Title}: {problems.Count} problem{(problems.Count == 1 ? string.Empty : "s")}");
        _editorStatus.Text = unsaved ? $"{state}  ·  unsaved changes" : state;
        _editorStatus.Foreground = Theme.Brush(_catalog is not null && problems.Count == 0 ? Theme.Ready : _catalog is null ? Theme.FgSoft : Theme.Accent);
        _editorProblems.Text = problems.Count == 0 ? string.Empty
            : string.Join("   ·   ", problems.Take(6)) + (problems.Count > 6 ? $"   ·   and {problems.Count - 6} more" : string.Empty);
        _editorProblems.Visibility = problems.Count == 0 ? Visibility.Collapsed : Visibility.Visible;

        LayOutPool();
        LayOutSearch();
    }

    private void ChangeFormat(DeckFormat format)
    {
        if (format == _editingFormat)
        {
            return;
        }

        _editingFormat = format;

        // Out of Commander, a commander is a card of the deck like any other.
        if (!format.IsCommander && _arrangement is { Command.Count: > 0 } arrangement)
        {
            foreach (int place in arrangement.Command.ToList())
            {
                arrangement.Remove(place);
                arrangement.Columns[DefaultColumn(_editorCards[place])].Add(place);
            }
        }

        _flights.Settle();
        DeckEdited();
    }

    private async Task<bool> SaveDeckAsync()
    {
        if (_arrangement is null)
        {
            return false;
        }

        SavedDeck deck = CurrentDeck();
        if (deck.Name.Length == 0)
        {
            deck.Name = "Untitled deck";
            _deckName.Text = deck.Name;
        }

        _saveButton.Enabled = false;
        try
        {
            using var body = new StringContent(JsonSerializer.Serialize(deck, DeckJson.Default.SavedDeck), Encoding.UTF8, "application/json");
            using HttpResponseMessage response = await DeckHttp.PostAsync(DeckApi("save"), body);
            if (response.StatusCode == HttpStatusCode.Unauthorized)
            {
                _signedOut = true;
                ShowDeckList();
                _editorStatus.Text = "Log in with Discord to save your decks.";
                return false;
            }

            response.EnsureSuccessStatusCode();
            DeckSaved? saved = JsonSerializer.Deserialize(await response.Content.ReadAsStringAsync(), DeckJson.Default.DeckSaved);
            if (saved?.Deck is not { } kept)
            {
                _editorStatus.Text = saved?.Refusal ?? "The deck could not be saved.";
                return false;
            }

            _editingId = kept.Id;
            _savedSnapshot = Snapshot(CurrentDeck());
            DeckEdited();
            await RefreshDeckListAsync();
            return true;
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or JsonException)
        {
            _editorStatus.Text = "The deck could not be saved: the site did not answer. Try again.";
            return false;
        }
        finally
        {
            _saveButton.Enabled = true;
        }
    }

    private async Task DeleteDeckAsync(DeckSummary deck)
    {
        try
        {
            using var body = new StringContent($"{{\"id\":\"{JsonEncodedText.Encode(deck.Id)}\"}}", Encoding.UTF8, "application/json");
            using HttpResponseMessage response = await DeckHttp.PostAsync(DeckApi("delete"), body);
            if (!response.IsSuccessStatusCode && response.StatusCode != HttpStatusCode.NotFound)
            {
                _editorStatus.Text = $"{deck.Name} could not be deleted. Try again.";
                return;
            }

            // The deck open is the one deleted: what is in the editor is kept, as a deck not saved.
            if (deck.Id == _editingId)
            {
                _editingId = string.Empty;
                _savedSnapshot = string.Empty;
                DeckEdited();
            }
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException)
        {
            _editorStatus.Text = $"{deck.Name} could not be deleted: the site did not answer. Try again.";
        }

        await RefreshDeckListAsync();
    }

    /// <summary>The cards the search finds, in the order chosen, legal in the deck's format and for Commander within its commanders' identity.</summary>
    private List<CatalogCard> SearchResults()
    {
        HashSet<ColorView>? identity = null;
        if (_editingFormat.IsCommander && _arrangement is { Command.Count: > 0 } arrangement)
        {
            identity = DeckRules.IdentityOf(arrangement.Command.Select(place => _catalogByName.GetValueOrDefault(_editorCards[place].Name)).OfType<CatalogCard>());
        }

        string[] words = _searchText.Text.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        List<(ColorView? Color, bool On)> colors = [.. ColorFilters.Select((filter, i) => (filter.Color, _colorButtons[i].On))];
        bool anyColor = colors.Any(color => color.On);
        int type = _typeChoice.Selected;
        int mana = _manaChoice.Selected;

        IEnumerable<CatalogCard> found = _catalogCards.Where(card =>
        {
            ObjectView face = card.Card;
            if (!DeckRules.Fits(card, _editingFormat, identity))
            {
                return false;
            }

            if (words.Length > 0)
            {
                string text = $"{face.Name}\n{CardFace.TypeLine(face)}\n{face.OracleText}\n{face.OtherFace?.Name}\n{face.OtherFace?.OracleText}";
                if (!words.All(word => text.Contains(word, StringComparison.OrdinalIgnoreCase)))
                {
                    return false;
                }
            }

            bool typed = type switch
            {
                0 => true,
                1 => face.CardTypes.Contains(CardTypeView.Creature),
                2 => face.CardTypes.Contains(CardTypeView.Instant),
                3 => face.CardTypes.Contains(CardTypeView.Sorcery),
                4 => face.CardTypes.Contains(CardTypeView.Artifact),
                5 => face.CardTypes.Contains(CardTypeView.Enchantment),
                6 => face.CardTypes.Contains(CardTypeView.Planeswalker),
                7 => face.CardTypes.Contains(CardTypeView.Land),
                8 => face.CardTypes.Contains(CardTypeView.Battle),
                9 => face.Supertypes.Contains(SupertypeView.Legendary),
                _ => DeckRules.CanBeCommander(card) || DeckRules.IsBackground(card),
            };
            if (!typed)
            {
                return false;
            }

            if (mana > 0 && (mana == ManaNames.Length - 1 ? face.ManaValue < 7 : face.ManaValue != mana - 1))
            {
                return false;
            }

            return !anyColor || colors.Any(color => color.On && (color.Color is { } hue ? face.Colors.Contains(hue) : face.Colors.Count == 0));
        });

        return _sortChoice.Selected == 1
            ? [.. found.OrderBy(card => card.Card.ManaValue).ThenBy(card => card.Name, StringComparer.OrdinalIgnoreCase)]
            : [.. found];
    }

    /// <summary>The search asks for other cards: shown from the top, as a new search is.</summary>
    private void SearchChanged()
    {
        _searchScroll?.ChangeView(null, 0, null, disableAnimation: true);
        LayOutSearch();
    }

    /// <summary>
    /// The search results in rows as wide as the pane, each with how many the deck has written over its art: every card
    /// found, as tall as they all stand, of which the rows near the view are laid out (<see cref="PlaceSearch"/>).
    /// </summary>
    private void LayOutSearch()
    {
        if (!DeckEditorShown || _catalog is null || _searchScroll is null)
        {
            _searchCount.Text = _catalogFailure ?? (_catalog is null ? "Loading the cards..." : string.Empty);
            return;
        }

        _searchFound = SearchResults();
        _searchCards.Clear();
        foreach (CatalogCard card in _searchFound)
        {
            _searchCards[SearchHandle(card)] = card;
        }

        _searchInDeck = new Dictionary<string, int>(StringComparer.Ordinal);
        if (_arrangement is { } arrangement)
        {
            foreach (int place in arrangement.Columns.SelectMany(column => column).Concat(arrangement.Side).Concat(arrangement.Command))
            {
                _searchInDeck[_editorCards[place].Name] = _searchInDeck.GetValueOrDefault(_editorCards[place].Name) + 1;
            }
        }

        _searchCount.Text = _searchFound.Count == 0
            ? $"No {_editingFormat.Title} card matches"
            : $"{_searchFound.Count} card{(_searchFound.Count == 1 ? string.Empty : "s")}";
        PlaceSearch(again: true);
        FlyPool();
    }

    /// <summary>
    /// Lays out the rows of results in view and a few either side, at the places they have among all of them: again on
    /// every scroll that brings other rows near, and otherwise only when <paramref name="again"/> says the results or
    /// their size changed.
    /// </summary>
    private void PlaceSearch(bool again)
    {
        if (!DeckEditorShown || _catalog is null || _searchScroll is null)
        {
            return;
        }

        float width = MathF.Round(SearchWidth * _poolScale);
        float height = Tall(width);
        float room = MathF.Max(width, (float)_searchScroll.ActualWidth - 4);
        int across = Math.Max(1, (int)((room + PoolGap) / (width + PoolGap)));
        var grid = new GridWindow(_searchFound.Count, across, height, PoolGap);
        double view = _searchScroll.ViewportHeight > 0 ? _searchScroll.ViewportHeight : _searchScroll.ActualHeight;
        (int First, int End) window = grid.Within(_searchScroll.VerticalOffset, view, SpareRows);
        if (!again && grid == _searchGrid && window == _searchWindow)
        {
            return;
        }

        _searchGrid = grid;
        _searchWindow = window;
        List<CardBoardElement.BoardCell> cells = [];
        for (int i = window.First; i < window.End; i++)
        {
            CatalogCard card = _searchFound[i];
            int handle = SearchHandle(card);
            if (!_searchViews.TryGetValue(handle, out ObjectView? shown))
            {
                shown = Handled(card.Card, handle);
                _searchViews[handle] = shown;
            }

            int had = _searchInDeck.GetValueOrDefault(card.Name);
            cells.Add(new CardBoardElement.BoardCell(shown, (i % across) * (width + PoolGap), (float)grid.Top(i), Tapped: false,
                Clickable: true, Annotation: had > 0 ? $"{had} in deck" : null));
        }

        _searchBoard.SetCellSize(width, height);
        _searchBoard.Width = room;
        _searchBoard.Height = Math.Max(grid.Height, (float)_searchScroll.ActualHeight - 4);
        _searchBoard.SetCells(cells);
    }

    private int SearchHandle(CatalogCard card) => SearchBase + _catalogPlaces[card];

    /// <summary>A search result clicked twice in a row goes into the deck, in its column.</summary>
    private void SearchClicked(ViewHandle handle)
    {
        DateTime now = DateTime.UtcNow;
        bool twice = _lastSearchClick is { } last && last.Handle == handle && now - last.At < TimeSpan.FromMilliseconds(450);
        _lastSearchClick = twice ? null : (handle, now);
        if (!twice || !_searchCards.TryGetValue(handle.Value, out CatalogCard? card))
        {
            return;
        }

        _dropPose = _searchBoard.Bounds(handle) is { } box && _searchBoard.XamlRoot is not null
            ? Middle(_searchBoard, box)
            : null;
        AddFromSearch(handle, arrangement => arrangement.Columns[DefaultColumn(card.Card)], int.MaxValue);
        _dropPose = null;
    }

    private CardFlights.Pose Middle(CardBoardElement board, Rect box)
    {
        Point middle = board.TransformToVisual(Root).TransformPoint(new Point(box.X + (box.Width / 2), box.Y + (box.Height / 2)));
        return new CardFlights.Pose((float)middle.X, (float)middle.Y, (float)box.Width, 0f);
    }

    /// <summary>A copy of a search result put in the deck, at a place in the list given; it flies there from where it was let go.</summary>
    private void AddFromSearch(ViewHandle handle, Func<PoolArrangement, List<int>> into, int row)
    {
        if (!_searchCards.TryGetValue(handle.Value, out CatalogCard? card) || _arrangement is not { } arrangement)
        {
            return;
        }

        _editorCards.Add(card.Card);
        int place = _editorCards.Count - 1;
        List<int> list = into(arrangement);
        list.Insert(Math.Clamp(row, 0, list.Count), place);
        if (_dropPose is { } from)
        {
            int copy = _editorCards.Take(place).Count(other => other.CardId == card.Card.CardId);
            _flights.Toss(new ViewHandle(EventHandle(card.Card.CardId, copy)), from);
        }

        DeckEdited();
    }

    /// <summary>Whether a card held could go in the command zone: in a Commander deck, one that may join the commanders there.</summary>
    private bool CanCommand(ViewHandle handle)
    {
        if (!EditingCommander || _arrangement is not { } arrangement || CatalogOf(handle) is not { } card)
        {
            return false;
        }

        int? place = _poolPlaces.TryGetValue(handle.Value, out int known) ? known : null;
        List<CatalogCard> others = [.. arrangement.Command.Where(other => other != place)
            .Select(other => _catalogByName.GetValueOrDefault(_editorCards[other].Name)).OfType<CatalogCard>()];
        return DeckRules.CanJoinCommanders(card, others);
    }

    /// <summary>The catalog's card for a card held: a search result, or a card of the deck.</summary>
    private CatalogCard? CatalogOf(ViewHandle handle) =>
        _searchCards.TryGetValue(handle.Value, out CatalogCard? found) ? found
        : InPool(handle) && _poolPlaces.TryGetValue(handle.Value, out int place) && place < _editorCards.Count ? _catalogByName.GetValueOrDefault(_editorCards[place].Name)
        : null;

    private void PutInCommand(ViewHandle handle, int row)
    {
        if (_arrangement is not { } arrangement)
        {
            return;
        }

        if (InSearch(handle))
        {
            AddFromSearch(handle, held => held.Command, row);
            return;
        }

        if (_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            arrangement.Remove(place);
            arrangement.Command.Insert(Math.Clamp(row, 0, arrangement.Command.Count), place);
            DeckEdited();
        }
    }

    /// <summary>A card of the deck put back on the search: out of the deck.</summary>
    private void TakeOut(ViewHandle handle)
    {
        if (_arrangement is not { } arrangement || !_poolPlaces.TryGetValue(handle.Value, out int place))
        {
            return;
        }

        arrangement.Remove(place);
        DeckEdited();
    }

    /// <summary>The command zone: a Commander deck's commanders, one under the other; elsewhere it is not shown.</summary>
    private void LayOutCommand(PoolArrangement arrangement, List<ObjectView> handled, float width, float height, float step)
    {
        if (!EditingCommander)
        {
            _commandColumn.Visibility = Visibility.Collapsed;
            _commandBoard.SetCells([]);
            return;
        }

        _commandColumn.Visibility = Visibility.Visible;
        float apart = MathF.Round(height * 0.55f);
        _commandBoard.SetCellSize(width, height);
        _commandBoard.Width = width;
        _commandBoard.Height = height + (apart * Math.Max(1, arrangement.Command.Count));
        _commandBoard.SetCells([.. arrangement.Command.Select((place, row) => new CardBoardElement.BoardCell(handled[place], 0, row * apart, Tapped: false, Clickable: true))]);
        _commandLabel.Text = arrangement.Command.Count switch
        {
            0 => "Commander  ·  drop one here",
            1 => "Commander",
            _ => $"Commanders  ·  {arrangement.Command.Count}",
        };
        _ = step;
    }

    /// <summary>"Deck: 60 cards (24 Land, 36 Non-land, 20 Creatures)  ·  sideboard 15", and the commander for Commander.</summary>
    private string EditorSummary(PoolArrangement arrangement)
    {
        List<ObjectView> cards = [.. arrangement.Columns.SelectMany(column => column).Select(place => _editorCards[place])];
        int lands = cards.Count(card => card.CardTypes.Contains(CardTypeView.Land));
        int creatures = cards.Count(card => card.CardTypes.Contains(CardTypeView.Creature));
        int commanders = arrangement.Command.Count;
        int total = cards.Count + (EditingCommander ? commanders : 0);
        string side = arrangement.Side.Count > 0 ? $"  ·  sideboard {arrangement.Side.Count}" : string.Empty;
        return $"Deck: {total} card{(total == 1 ? string.Empty : "s")} ({lands} Land, {cards.Count - lands} Non-land, {creatures} Creature{(creatures == 1 ? string.Empty : "s")}){side}";
    }
}

/// <summary>The page's own JavaScript the deck editor uses (DeckEditor.js).</summary>
internal static partial class DeckEditorJs
{
    [JSImport("globalThis.cantripDeckUnsaved")]
    private static partial void SetUnsaved(bool unsaved);

    [JSImport("globalThis.cantripGoTo")]
    private static partial void GoTo(string address);

    private static bool? _told;

    /// <summary>Whether the browser asks before the page is left: only while the deck has changes not saved.</summary>
    public static void Unsaved(bool unsaved)
    {
        if (_told == unsaved)
        {
            return;
        }

        _told = unsaved;
        try
        {
            SetUnsaved(unsaved);
        }
        catch (JSException)
        {
            // A page without the script asks nothing.
        }
    }

    /// <summary>Goes to another page of the site, relative to this one; the browser asks first if a deck is unsaved.</summary>
    public static void Go(string relative)
    {
        try
        {
            GoTo(new Uri(ClientConfig.Page, relative).AbsoluteUri);
        }
        catch (JSException)
        {
        }
    }
}

/// <summary>A small button with a bin drawn on it, for deleting a deck from the list.</summary>
internal sealed class BinButton : Border
{
    private bool _hovered;

    public BinButton(string name)
    {
        Width = 28;
        Height = 26;
        CornerRadius = new CornerRadius(4);
        BorderThickness = new Thickness(1);
        AutomationProperties.SetName(this, name);
        ToolTipService.SetToolTip(this, name);
        ProtectedCursor = Microsoft.UI.Input.InputSystemCursor.Create(Microsoft.UI.Input.InputSystemCursorShape.Hand);

        // A lid with its handle, and the can with its ribs.
        var bin = new BinShape
        {
            Data = (Microsoft.UI.Xaml.Media.Geometry)Microsoft.UI.Xaml.Markup.XamlBindingHelper.ConvertValue(typeof(Microsoft.UI.Xaml.Media.Geometry),
                "M 1,3 L 13,3 M 5,3 L 5,1 L 9,1 L 9,3 M 2.5,3 L 3.5,14 L 10.5,14 L 11.5,3 M 5.5,6 L 5.8,11.5 M 8.5,6 L 8.2,11.5"),
            StrokeThickness = 1.3,
            Width = 14,
            Height = 15,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
        };
        Child = bin;

        PointerEntered += (_, _) => { _hovered = true; Paint(); };
        PointerExited += (_, _) => { _hovered = false; Paint(); };
        Tapped += (_, e) =>
        {
            e.Handled = true;
            Clicked?.Invoke();
        };
        Paint();
    }

    public event Action? Clicked;

    private void Paint()
    {
        Background = Theme.Brush(_hovered ? Theme.ControlHover : Theme.ControlFill);
        BorderBrush = Theme.Brush(_hovered ? Theme.Danger : Theme.ControlBorder);
        ((BinShape)Child).Stroke = Theme.Brush(_hovered ? Theme.Danger : Theme.FgSoft);
    }
}
