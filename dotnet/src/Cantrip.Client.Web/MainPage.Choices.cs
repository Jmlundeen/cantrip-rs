using Cantrip.Protocol;
using Microsoft.UI.Text;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The window for cards that are not on the board - a library being searched, a hand being chosen from, triggers being
/// put in order - centred over the table. Always five cards across, scrolling past three rows. When a prompt is about these cards the window carries the prompt too,
/// with its buttons at the bottom right, so the choice and the answer are in one place.
/// </summary>
public sealed partial class MainPage
{
    private const int ChoiceColumns = 5;
    private const int ChoiceRows = 3;
    private const float ChoiceGap = 8f;
    private const float MinChoiceWidth = 380f;

    private Grid _choiceHost = null!;
    private TextBlock _choiceTitle = null!;
    private CollapseButton _choiceToggle = null!;
    private DockButton _choiceClose = null!;
    private StackPanel _choicePrompt = null!;
    private SymbolText _choicePromptTitle = null!;
    private SymbolText _choicePromptDetail = null!;
    private DockToggle _choiceValidOnly = null!;

    /// <summary>How a library being searched is shown - its order and its sections - with the filter, in one row.</summary>
    private StackPanel _choiceViewRow = null!;
    private ChoiceDropdown _choiceSort = null!;
    private ChoiceDropdown _choiceGroup = null!;
    private CardSort _searchSort = CardSort.None;
    private CardGrouping _searchGrouping = CardGrouping.None;

    /// <summary>
    /// The window's sections, when its cards are grouped: each a heading with its count over a grid of its own, from
    /// that index in <see cref="_choiceCards"/>. Empty for one run of cards.
    /// </summary>
    private readonly List<(string Heading, int Count, int From)> _choiceSections = [];

    /// <summary>The section headings, over the cards in the same scrolling content so they move with them.</summary>
    private Canvas _choiceHeadings = null!;

    /// <summary>How tall a section heading stands.</summary>
    private const float SearchHeadingHeight = 18f;
    private ScrollViewer _choiceScroll = null!;
    private CardBoardElement _choiceBoard = null!;
    private TextBlock _choiceEmpty = null!;

    /// <summary>Content a prompt lays out itself in place of the grid of cards, as distributing cards does; empty otherwise.</summary>
    private ContentControl _choiceCustom = null!;

    /// <summary>The height a prompt with content of its own wants for it, in place of the grid's rows; null for the grid.</summary>
    private float? _choiceTall;
    private Grid _choiceFooter = null!;
    private Border _choicePicked = null!;
    private CardBoardElement _choicePickedBoard = null!;
    private FlowPanel _choiceButtons = null!;
    private bool _choiceCollapsed;
    private bool _choicesOpen;
    private bool _choicePrompted;
    private int _choiceCount;

    /// <summary>The cards in the window this redraw, each with its look, in the order laid out.</summary>
    private readonly List<(ObjectView Card, CardLook Look)> _choiceCards = [];
    private readonly Dictionary<ViewHandle, CardLook> _choiceLooks = [];
    private readonly Dictionary<ViewHandle, CardLook> _pickedLooks = [];

    private Grid BuildChoiceOverlay()
    {
        var window = new Border
        {
            Background = Theme.Brush(Theme.WithAlpha(Theme.Panel, 0.75)),
            BorderBrush = Theme.Brush(Theme.ControlBorder),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(10),
            Padding = new Thickness(14, 12, 14, 12),
        };

        var column = new StackPanel { Orientation = Orientation.Vertical, Spacing = 10 };
        window.Child = column;

        var header = new Grid { ColumnSpacing = 4, Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0)) };
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        header.Tapped += (_, _) => ToggleChoices();
        column.Children.Add(header);

        _choiceTitle = new TextBlock { FontSize = 13, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Fg), VerticalAlignment = VerticalAlignment.Center };
        header.Children.Add(_choiceTitle);

        _choiceToggle = new CollapseButton();
        Grid.SetColumn(_choiceToggle, 1);
        header.Children.Add(_choiceToggle);

        // A window opened to look into a zone is closed from here; one carrying a prompt closes by being answered.
        _choiceClose = new DockButton("×", primary: false, fontSize: 15) { Tooltip = "Close", VerticalAlignment = VerticalAlignment.Center }.Quiet();
        _choiceClose.Clicked += () =>
        {
            _peek = null;
            _needsRedraw = true;
            RedrawNow();
        };
        Grid.SetColumn(_choiceClose, 2);
        header.Children.Add(_choiceClose);

        _choicePrompt = new StackPanel { Orientation = Orientation.Vertical, Spacing = 4 };
        column.Children.Add(_choicePrompt);

        _choicePromptTitle = new SymbolText(15, Theme.Fg, semibold: true, wrap: true);
        _choicePrompt.Children.Add(_choicePromptTitle);

        _choicePromptDetail = new SymbolText(12, Theme.FgSoft, wrap: true);
        _choicePrompt.Children.Add(_choicePromptDetail);

        _choiceValidOnly = new DockToggle("Show only valid choices")
        {
            Tooltip = "Hide the cards this search cannot take. The library keeps its order either way.",
        };
        _choiceValidOnly.SetOn(_searchValidOnly);
        _choiceValidOnly.Toggled += on =>
        {
            _searchValidOnly = on;
            _needsRedraw = true;
            RedrawNow();
        };

        // A row of its own, so the prompt's detail keeps the width to read on one line. Sort and Group by come before the
        // filter, so the row reads as how the cards are shown and then which.
        _choiceSort = new ChoiceDropdown(
            "Sort",
            "Order the cards by name or mana value. None keeps the library's order.",
            ["None", "Name", "Mana Value"])
        {
            Changed = index => Rearrange(() => _searchSort = (CardSort)index),
        };

        _choiceGroup = new ChoiceDropdown(
            "Group by",
            "Split the cards into sections by colour or card type.",
            ["None", "Color", "Type"])
        {
            Changed = index => Rearrange(() => _searchGrouping = (CardGrouping)index),
        };

        _choiceViewRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16, Children = { _choiceSort, _choiceGroup, _choiceValidOnly } };
        _choicePrompt.Children.Add(_choiceViewRow);

        _choiceBoard = new CardBoardElement(100, 140) { FliesAlong = false };
        _choiceBoard.CardClicked += handle =>
        {
            NotePress(_choiceBoard);
            Clicked(_choiceLooks, handle);
        };

        _choiceEmpty = new TextBlock
        {
            FontSize = 12,
            Foreground = Theme.Brush(Theme.FgMuted),
            HorizontalAlignment = HorizontalAlignment.Center,
            Visibility = Visibility.Collapsed,
        };

        _choiceScroll = new ScrollViewer
        {
            HorizontalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            Content = new Grid
            {
                Children =
                {
                    _choiceBoard,
                    (_choiceHeadings = new Canvas { IsHitTestVisible = false, HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top }),
                    _choiceEmpty,
                    (_choiceCustom = new ContentControl { HorizontalContentAlignment = HorizontalAlignment.Stretch }),
                },
            },
        };

        column.Children.Add(_choiceScroll);

        _choiceFooter = new Grid { ColumnSpacing = 12 };
        _choiceFooter.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        _choiceFooter.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        column.Children.Add(_choiceFooter);

        // Outside the cards' scroll, so what has been picked stays in view however far down the library is read.
        _choicePickedBoard = new CardBoardElement(60, 84);
        _choicePickedBoard.CardClicked += handle => Clicked(_pickedLooks, handle);
        _choicePicked = new Border
        {
            Background = Theme.Brush(Color.FromArgb(46, 0, 0, 0)),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(8),
            VerticalAlignment = VerticalAlignment.Bottom,
            Child = _choicePickedBoard,
        };
        _choiceFooter.Children.Add(_choicePicked);

        _choiceButtons = new FlowPanel { HorizontalSpacing = 8, VerticalSpacing = 6, AlignEnd = true, VerticalAlignment = VerticalAlignment.Bottom };
        Grid.SetColumn(_choiceButtons, 1);
        _choiceFooter.Children.Add(_choiceButtons);

        var host = new Grid
        {
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Visibility = Visibility.Collapsed,
            Children = { window },
        };

        // Over the window's side edges, short of its rounded corners. Either one widens the window on both sides at once,
        // since it stays centred.
        _choiceGrips = [ChoiceGrip(outwards: -1, HorizontalAlignment.Left), ChoiceGrip(outwards: 1, HorizontalAlignment.Right)];
        foreach (ResizeGrip grip in _choiceGrips)
        {
            host.Children.Add(grip);
        }

        return host;
    }

    private ResizeGrip[] _choiceGrips = [];

    /// <summary>The width the window is being dragged to, while it is; the width it was left at is kept between visits.</summary>
    private float? _choiceDragFrom;
    private float _choiceDragWidth;
    private const string ChoiceWidthKey = "cantrip-choice-width";
    private float _choiceKept = LocalSettings.Number(ChoiceWidthKey, 0f);

    private ResizeGrip ChoiceGrip(int outwards, HorizontalAlignment edge)
    {
        var grip = new ResizeGrip("Drag to resize", sideways: true)
        {
            HorizontalAlignment = edge,
            Margin = new Thickness(outwards < 0 ? -ResizeGrip.Thickness / 2 : 0, 12, outwards > 0 ? -ResizeGrip.Thickness / 2 : 0, 12),
        };

        grip.Started += () =>
        {
            _choiceDragFrom = ChoiceWidth();
            _choiceDragWidth = _choiceDragFrom.Value;
        };

        grip.Moved += across =>
        {
            float width = MathF.Round(ClampChoiceWidth((_choiceDragFrom ?? ChoiceWidth()) + (2f * (float)across * outwards)));
            if (width != _choiceDragWidth)
            {
                _choiceDragWidth = width;
                SettleCards();
                _needsRedraw = true;
                RedrawNow();
            }
        };

        grip.Ended += () =>
        {
            _choiceDragFrom = null;
            _choiceKept = _choiceDragWidth;
            LocalSettings.Store(ChoiceWidthKey, _choiceKept);
        };

        return grip;
    }

    private void Clicked(Dictionary<ViewHandle, CardLook> looks, ViewHandle handle)
    {
        if (looks.TryGetValue(handle, out CardLook? look) && look.Clicked is { } clicked)
        {
            clicked();
            RedrawNow();
        }
    }

    private void ToggleChoices()
    {
        _choiceCollapsed = !_choiceCollapsed;
        _needsRedraw = true;
        RedrawNow();
    }

    /// <summary>Closes the window at the start of every dock redraw; a prompt that wants it opens it again.</summary>
    private void ClearChoices()
    {
        _promptDropZones.Clear();
        _promptBoards.Clear();
        _choicesOpen = false;
        _choiceCards.Clear();
        _choiceLooks.Clear();
        _pickedLooks.Clear();
        _choiceButtons.Children.Clear();
        _choiceEmpty.Visibility = Visibility.Collapsed;
        _choicePicked.Visibility = Visibility.Collapsed;
        _choiceViewRow.Visibility = Visibility.Collapsed;
        _choiceSections.Clear();
        _choiceCustom.Content = null;
        _choiceTall = null;
    }

    /// <summary>Opens the window; a window that is not closable carries the prompt and its buttons.</summary>
    /// <param name="viewControls">Whether the window offers the library search's Sort, Group by and filter.</param>
    /// <param name="headings">How much height the sections' headings take, counted while every row fits.</param>
    private void OpenChoices(string title, bool closable, int count, bool viewControls = false, float headings = 0f)
    {
        _choicesOpen = true;
        _choicePrompted = !closable;
        _choiceCount = count;
        _choiceHeadingsHeight = headings;
        _choiceTitle.Text = title;

        if (_choicePrompted)
        {
            _buttons = _choiceButtons;
        }

        _choiceViewRow.Visibility = viewControls && !_choiceCollapsed ? Visibility.Visible : Visibility.Collapsed;
        _choiceValidOnly.SetOn(_searchValidOnly);
        _choiceSort.SetSelected((int)_searchSort);
        _choiceGroup.SetSelected((int)_searchGrouping);
        MirrorPromptInChoices();
    }

    /// <summary>A window titled with what the prompt asks does not say it twice.</summary>
    private void MirrorPromptInChoices()
    {
        if (!_choicesOpen)
        {
            return;
        }

        _choicePromptTitle.Text = _choicePrompted ? _promptTitle.Text : string.Empty;
        _choicePromptTitle.Visibility = _choicePrompted && _promptTitle.Text != _choiceTitle.Text ? Visibility.Visible : Visibility.Collapsed;
        _choicePromptDetail.Text = _choicePrompted ? _promptDetail.Text : string.Empty;
        _choicePromptDetail.Visibility = _choicePrompted && _promptDetail.Text.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    private float _choiceHeadingsHeight;

    /// <summary>
    /// Changes how the window's cards are shown and lays them out again. Nothing moves in the game, so the cards are put
    /// where they now go rather than flown there.
    /// </summary>
    private void Rearrange(Action change)
    {
        change();
        SettleCards();
        _needsRedraw = true;
        RedrawNow();
    }

    /// <summary>Starts a section of the window's cards under a heading; the cards added after it are its cards.</summary>
    private void AddChoiceSection(string heading, int count) => _choiceSections.Add((heading, count, _choiceCards.Count));

    private void AddChoice(ObjectView card, CardLook look)
    {
        _choiceCards.Add((card, look));
        _choiceLooks[card.Handle] = look;
    }

    private void ShowChoicesEmpty(string text)
    {
        _choiceEmpty.Text = text;
        _choiceEmpty.Visibility = Visibility.Visible;
    }

    /// <summary>What a search has picked so far, in the order picked, in the footer; clicking one puts it back.</summary>
    private void ShowPicked(GameClient client, int limit)
    {
        if (_picked.Count == 0 || _choiceCollapsed)
        {
            return;
        }

        _choicePicked.Visibility = Visibility.Visible;
        foreach (int handle in _picked)
        {
            if (client.Object(new ViewHandle(handle)) is { } card)
            {
                _pickedLooks[card.Handle] = new CardLook(Selected: true, Clicked: () => Pick(handle, limit));
            }
        }
    }

    /// <summary>A section's heading in a grouped library search: its name, how many cards it holds, and a rule across.</summary>
    private static FrameworkElement SearchHeading(string heading, int count, float width)
    {
        var row = new Grid { Width = width, Height = SearchHeadingHeight, ColumnSpacing = 8 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

        row.Children.Add(new TextBlock { Text = heading, FontSize = 12, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Fg), VerticalAlignment = VerticalAlignment.Center });

        var many = new TextBlock
        {
            Text = count.ToString(System.Globalization.CultureInfo.InvariantCulture),
            FontSize = 11,
            Foreground = Theme.Brush(Theme.FgMuted),
            VerticalAlignment = VerticalAlignment.Center,
        };
        Grid.SetColumn(many, 1);
        row.Children.Add(many);

        var rule = new Border { Height = 1, Background = Theme.Brush(Theme.Line), VerticalAlignment = VerticalAlignment.Center };
        Grid.SetColumn(rule, 2);
        row.Children.Add(rule);
        return row;
    }

    /// <summary>
    /// How wide the window is: the width being dragged to, or else the width the player last left it at, or else most of
    /// the board. Never wider than the board leaves room for.
    /// </summary>
    private float ChoiceWidth()
    {
        float board = MathF.Max(0f, (float)_table.ActualWidth - BoardInset());
        float wanted = _choiceDragFrom is not null ? _choiceDragWidth : _choiceKept > 0f ? _choiceKept : board * 0.62f;
        return ClampChoiceWidth(wanted);
    }

    private float ClampChoiceWidth(float width)
    {
        float board = MathF.Max(0f, (float)_table.ActualWidth - BoardInset());
        return Math.Clamp(width, MinChoiceWidth, MathF.Max(MinChoiceWidth, board - 40f));
    }

    /// <summary>Each card's width in the window: its width shared by five columns, less its padding, the gaps and a scroll bar.</summary>
    private float ChoiceCardWidth() =>
        MathF.Floor((ChoiceWidth() - 28f - 14f - ((ChoiceColumns - 1) * ChoiceGap)) / ChoiceColumns);

    /// <summary>Sizes the window and lays its cards out, once the dock has filled it.</summary>
    private void LayOutChoices()
    {
        _choiceHost.Visibility = _choicesOpen ? Visibility.Visible : Visibility.Collapsed;
        if (!_choicesOpen || _gameClient is not { } client)
        {
            return;
        }

        _choiceToggle.Collapsed = _choiceCollapsed;
        _choicePrompt.Visibility = _choicePrompted && !_choiceCollapsed ? Visibility.Visible : Visibility.Collapsed;
        _choiceScroll.Visibility = _choiceCollapsed ? Visibility.Collapsed : Visibility.Visible;
        _choiceButtons.Visibility = !_choiceCollapsed && _choiceButtons.Children.Count > 0 ? Visibility.Visible : Visibility.Collapsed;
        _choiceFooter.Visibility = _choiceButtons.Visibility == Visibility.Visible || _choicePicked.Visibility == Visibility.Visible
            ? Visibility.Visible
            : Visibility.Collapsed;

        float width = ChoiceWidth();
        foreach (ResizeGrip grip in _choiceGrips)
        {
            grip.Visibility = _choiceCollapsed ? Visibility.Collapsed : Visibility.Visible;
        }

        _choiceHost.Margin = new Thickness(BoardInset(), 0, 0, 0);
        _choiceClose.Visibility = _choicePrompted ? Visibility.Collapsed : Visibility.Visible;
        _choiceHost.Width = _choiceCollapsed ? double.NaN : width;

        float cardWidth = ChoiceCardWidth();
        float cardHeight = MathF.Round(cardWidth * 88f / 63f);

        // As tall as the rows, up to three, past which it scrolls; never taller than the table leaves room for.
        int rows = Math.Max(1, (_choiceCount + ChoiceColumns - 1) / ChoiceColumns);
        float above = _choicePicked.Visibility == Visibility.Visible ? MathF.Round(cardWidth * 0.75f * 88f / 63f) + 16f - 30f : 0f;
        float tall = (Math.Min(rows, ChoiceRows) * (cardHeight + ChoiceGap)) - ChoiceGap + (rows <= ChoiceRows ? _choiceHeadingsHeight : 0f);
        float room = (float)_table.ActualHeight - 40f - (_choicePrompted ? 190f : 70f) - above;
        _choiceScroll.Height = _choiceTall ?? MathF.Max(MathF.Min(cardHeight, room), MathF.Min(tall, room));

        float gridWidth = (ChoiceColumns * cardWidth) + ((ChoiceColumns - 1) * ChoiceGap);
        _choiceBoard.SetCellSize(cardWidth, cardHeight);
        _choiceBoard.Width = gridWidth;
        _choiceBoard.HorizontalAlignment = HorizontalAlignment.Left;
        _choiceHeadings.Children.Clear();
        _choiceHeadings.Width = gridWidth;

        // One run of cards, or one grid per section under its heading, so every section starts a fresh row.
        List<(string? Heading, int Count, int From, int To)> runs = _choiceSections.Count == 0
            ? [(null, _choiceCards.Count, 0, _choiceCards.Count)]
            : [.. _choiceSections.Select((section, i) => ((string?)section.Heading, section.Count, section.From,
                i + 1 < _choiceSections.Count ? _choiceSections[i + 1].From : _choiceCards.Count))];

        List<CardBoardElement.BoardCell> cells = [];
        float y = 0f;
        foreach ((string? heading, int count, int from, int to) in runs)
        {
            if (heading is not null)
            {
                FrameworkElement row = SearchHeading(heading, count, gridWidth);
                Canvas.SetTop(row, y);
                _choiceHeadings.Children.Add(row);
                y += SearchHeadingHeight + ChoiceGap;
            }

            for (int i = from; i < to; i++)
            {
                (ObjectView card, CardLook look) = _choiceCards[i];
                int at = i - from;
                cells.Add(new CardBoardElement.BoardCell(
                    card,
                    (at % ChoiceColumns) * (cardWidth + ChoiceGap),
                    y + ((at / ChoiceColumns) * (cardHeight + ChoiceGap)),
                    Tapped: false,
                    look.Actionable,
                    look.Dimmed,
                    look.Selected,
                    Clickable: look.Clicked is not null,
                    look.Annotation));
            }

            int rowsHere = (to - from + ChoiceColumns - 1) / ChoiceColumns;
            y += (rowsHere * (cardHeight + ChoiceGap));
        }

        _choiceBoard.Height = _choiceCards.Count == 0 ? 0 : MathF.Max(0f, y - ChoiceGap);
        _choiceHeadings.Height = _choiceBoard.Height;
        _choiceBoard.SetCells(cells);

        // The picked row: every picked card in view however many, overlapping once they would not fit side by side.
        float pickedWidth = MathF.Floor(cardWidth * 0.75f);
        float pickedHeight = MathF.Round(pickedWidth * 88f / 63f);
        List<ObjectView> picked = [.. _picked.Select(handle => client.Object(new ViewHandle(handle))).OfType<ObjectView>()];
        float step = pickedWidth + 6f;
        _choicePickedBoard.SetCellSize(pickedWidth, pickedHeight);
        _choicePickedBoard.Height = pickedHeight;
        _choicePickedBoard.Width = picked.Count == 0 ? 0 : pickedWidth + (step * (picked.Count - 1));
        _choicePickedBoard.SetCells([.. picked.Select((card, i) =>
        {
            CardLook look = _pickedLooks.GetValueOrDefault(card.Handle) ?? Plain;
            return new CardBoardElement.BoardCell(card, i * step, 0, Tapped: false, Selected: look.Selected, Clickable: look.Clicked is not null);
        })]);
    }
}
