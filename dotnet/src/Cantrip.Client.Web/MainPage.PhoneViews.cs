using Cantrip.Client.Web.InPerson;
using Cantrip.Protocol;
using Cantrip.Protocol.InPerson;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Automation;

namespace Cantrip.Client.Web;

/// <summary>
/// A phone's views at an in-person table (docs/web-client.md, "In-person play"): the phone held sideways shows one view at a
/// time beside the prompt, which is always on the right. The public board is the tablets' to show, so the phone does not
/// spend its screen on it: it opens on the player's hand, and a drawer that slides out from the left edge goes to the
/// others - each player's battlefield, each player's public state (life, counters, mana, piles and graveyard, by the
/// player's name), the stack, and the stops.
/// </summary>
public sealed partial class MainPage
{
    /// <summary>What a phone shows beside its prompt.</summary>
    private enum PhoneView
    {
        Hand,
        Board,
        Player,
        Stack,
        Stops,
    }

    /// <summary>The width of the strip down the phone's left edge that holds the drawer's tab and the Fullscreen button.</summary>
    private const double DrawerStrip = 52;

    /// <summary>The view shown, and the seat it is of (a board or a player's state; zero otherwise).</summary>
    private (PhoneView View, int Seat) _phoneView = (PhoneView.Hand, 0);

    /// <summary>The view shown before the stack was opened for its triggers to be put in order, to go back to after.</summary>
    private (PhoneView View, int Seat)? _beforeOrdering;

    private DockButton _drawerTab = null!;
    private Grid _drawer = null!;
    private StackPanel _drawerList = null!;
    private TextBlock _drawerTurn = null!;
    private PhoneStopsPanel _phoneStops = null!;
    private TextBlock _phoneStackEmpty = null!;

    /// <summary>The seats and names the drawer's buttons were last made for, so they are made again only when those change.</summary>
    private string _drawerMadeFor = "-";

    /// <summary>
    /// The phone's layout, sideways: the prompt down the right (the dock, scrolling if it must), the view beside it, and the
    /// drawer's tab and the Fullscreen button in a strip down the left edge.
    /// </summary>
    private void BuildPhoneViews()
    {
        // The dock leaves the bottom strip for a column of its own down the right, over the whole height.
        _bottom.Children.Remove(_dock);
        UIElement content = _dock.Child;
        _dock.Child = null;
        _dock.Child = new ScrollViewer
        {
            VerticalScrollMode = ScrollMode.Auto,
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            HorizontalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            Content = content,
        };
        _dock.BorderThickness = new Thickness(1, 0, 0, 0);
        _dock.Padding = new Thickness(12, 10, 12, 10);
        _dock.HorizontalAlignment = HorizontalAlignment.Stretch;
        Grid.SetColumn(_dock, 1);
        Grid.SetRow(_dock, 0);
        Grid.SetRowSpan(_dock, 3);
        Root.Children.Insert(Root.Children.IndexOf(_sideSlot), _dock);

        // Undo is the prompt's, at the top of the dock, rather than in a corner of a battlefield the phone may not be showing.
        ((Grid)_undoButton.Parent).Children.Remove(_undoButton);
        _undoButton.Margin = new Thickness(0, 0, 0, 2);
        _undoButton.HorizontalAlignment = HorizontalAlignment.Left;
        _undoButton.VerticalAlignment = VerticalAlignment.Top;
        ((StackPanel)content).Children.Insert(0, _undoButton);

        // The view's own parts clear of the strip down the left edge.
        foreach (FrameworkElement part in (FrameworkElement[])[_table, _bottom, _phaseTimeline])
        {
            part.Margin = new Thickness(DrawerStrip, 0, 0, 0);
        }

        _phaseTimeline.Visibility = Visibility.Collapsed;
        _bottom.BorderThickness = new Thickness(0);

        // A choice of cards - a library searched, a scry, triggers to order - fills the phone, over the view and the dock.
        _table.Children.Remove(_choiceHost);
        Grid.SetRow(_choiceHost, 0);
        Grid.SetRowSpan(_choiceHost, 3);
        Grid.SetColumnSpan(_choiceHost, 2);
        _choiceHost.HorizontalAlignment = HorizontalAlignment.Stretch;
        _choiceHost.VerticalAlignment = VerticalAlignment.Stretch;
        _choiceHost.Background = Theme.Brush(Theme.Window);
        if (_choiceHost.Children.OfType<Border>().FirstOrDefault() is { } window)
        {
            window.Background = Theme.Brush(Theme.WithAlpha(Theme.Panel, 0.98));
            window.CornerRadius = new CornerRadius(0);
            window.BorderThickness = new Thickness(0);
        }

        foreach (ResizeGrip grip in _choiceGrips)
        {
            _choiceHost.Children.Remove(grip);
        }

        _choiceGrips = [];
        Root.Children.Insert(Root.Children.IndexOf(_flights), _choiceHost);

        // The stack's view says so when there is nothing on it.
        _phoneStackEmpty = new TextBlock
        {
            Text = "Nothing on the stack",
            FontSize = 16,
            Foreground = Theme.Brush(Theme.FgMuted),
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Visibility = Visibility.Collapsed,
        };
        Grid.SetRowSpan(_phoneStackEmpty, 3);
        _table.Children.Add(_phoneStackEmpty);

        _phoneStops = new PhoneStopsPanel(_stops, OthersStopAt)
        {
            Margin = new Thickness(DrawerStrip, 0, 0, 0),
            Visibility = Visibility.Collapsed,
        };
        Grid.SetRowSpan(_phoneStops, 3);
        Root.Children.Insert(Root.Children.IndexOf(_dock), _phoneStops);

        BuildDrawer();
        ShowPhoneView(PhoneView.Hand, 0);
    }

    /// <summary>The drawer's tab at the top of the strip down the left edge, and the drawer itself, over the view.</summary>
    private void BuildDrawer()
    {
        _drawerTab = new DockButton("Views", primary: false, fontSize: 13)
        {
            Width = DrawerStrip - 8,
            Height = DrawerStrip - 8,
            Margin = new Thickness(4, 6, 4, 4),
            Tooltip = "Hand, boards, players, stack and stops",
        }.Bars(18);
        AutomationProperties.SetName(_drawerTab, "Views");
        _drawerTab.Clicked += () => OpenDrawer(_drawer.Visibility != Visibility.Visible);
        Grid.SetRowSpan(_drawerTab, 3);
        Root.Children.Insert(Root.Children.IndexOf(_dock), _drawerTab);

        _drawerTurn = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.FgSoft), TextWrapping = TextWrapping.WrapWholeWords, Margin = new Thickness(4, 0, 4, 4) };
        _drawerList = new StackPanel { Spacing = 4 };

        // Beside the strip, so the tab that shuts it and the Fullscreen button stay in view.
        var panel = new Border
        {
            Width = 240,
            Margin = new Thickness(DrawerStrip, 0, 0, 0),
            HorizontalAlignment = HorizontalAlignment.Left,
            Background = Theme.Brush(Theme.Panel),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(0, 0, 1, 0),
            Padding = new Thickness(8, 8, 8, 8),
            Child = new ScrollViewer
            {
                VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
                HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
                Content = new StackPanel
                {
                    Spacing = 2,
                    Children =
                    {
                        _drawerTurn,
                        _drawerList,
                    },
                },
            },
        };

        // Over the view and the dock, under the cards held up and the choices; a tap off the drawer shuts it.
        var scrim = new Border { Background = Theme.Brush(Windows.UI.Color.FromArgb(130, 0, 0, 0)) };
        scrim.Tapped += (_, e) =>
        {
            e.Handled = true;
            OpenDrawer(false);
        };
        _drawer = new Grid { Visibility = Visibility.Collapsed, Children = { scrim, panel } };
        Grid.SetRowSpan(_drawer, 3);
        Grid.SetColumnSpan(_drawer, 2);
        Root.Children.Insert(Root.Children.IndexOf(_choiceHost), _drawer);
        MakeDrawerButtons(null);
    }

    private void OpenDrawer(bool open)
    {
        _drawer.Visibility = open ? Visibility.Visible : Visibility.Collapsed;
        _drawerTab.On = open;
    }

    /// <summary>
    /// The drawer's buttons: the hand; each player's battlefield; each player's public state, by name; the stack; the
    /// stops; and the game's menu. Made again when the players are first known.
    /// </summary>
    private void MakeDrawerButtons(PlayerView? view)
    {
        IReadOnlyList<PlayerSummaryView> players = view is null ? [] : [.. view.Players.OrderBy(player => player.Seat == (_gameClient?.Seat ?? 0) ? 0 : 1).ThenBy(player => player.Seat)];
        string madeFor = string.Join("|", players.Select(player => $"{player.Seat}:{player.Name}"));
        if (madeFor == _drawerMadeFor)
        {
            return;
        }

        _drawerMadeFor = madeFor;
        _drawerList.Children.Clear();
        Add("Hand", PhoneView.Hand, 0);
        foreach (PlayerSummaryView player in players)
        {
            Add($"Board · {player.Name}", PhoneView.Board, player.Seat);
        }

        foreach (PlayerSummaryView player in players)
        {
            Add(player.Name, PhoneView.Player, player.Seat);
        }

        Add("Stack", PhoneView.Stack, 0);
        Add("Stops", PhoneView.Stops, 0);

        var menu = new DockButton("Log, settings and menu", primary: false, fontSize: 13) { MinHeight = 34, Margin = new Thickness(0, 4, 0, 0) }.Filling().Quiet();
        AutomationProperties.SetName(menu, "Menu");
        menu.Clicked += () =>
        {
            OpenDrawer(false);
            SetSideOpen(true);
        };
        _drawerList.Children.Add(menu);
        PaintDrawer();

        void Add(string label, PhoneView shown, int seat)
        {
            var button = new DockButton(label, primary: false, fontSize: 14) { MinHeight = 36, Tag = (shown, seat) }.Filling();
            AutomationProperties.SetName(button, label);
            button.Clicked += () =>
            {
                OpenDrawer(false);
                _beforeOrdering = null;
                ShowPhoneView(shown, seat);
            };
            _drawerList.Children.Add(button);
        }
    }

    /// <summary>The drawer's buttons as things stand: the view shown held down, the hand's and the stack's counts.</summary>
    private void PaintDrawer()
    {
        GameClient? client = _gameClient;
        int stack = client?.View is null ? 0 : client.ZoneCount(ZoneKindView.Stack, 0);
        foreach (DockButton button in _drawerList.Children.OfType<DockButton>())
        {
            if (button.Tag is not (PhoneView shown, int seat))
            {
                continue;
            }

            button.On = _phoneView == (shown, seat);
            if (shown == PhoneView.Hand)
            {
                int hand = client?.View is null ? 0 : client.MyHand().Count();
                button.Caption = hand > 0 ? $"Hand · {hand}" : "Hand";
            }
            else if (shown == PhoneView.Stack)
            {
                button.Caption = stack > 0 ? $"Stack · {stack}" : "Stack";
                button.Lit = stack > 0 && _phoneView.View != PhoneView.Stack;
            }
        }

        // Something on the stack lights the tab, while the stack is not what is shown.
        _drawerTab.Lit = stack > 0 && _phoneView.View != PhoneView.Stack;
        _drawerTurn.Text = client?.View is { } view && !client.IsOver
            ? $"Turn {view.Turn} · {view.Players.FirstOrDefault(player => player.Seat == view.ActivePlayer)?.Name ?? "?"} · {StepName(view.Step)}"
            : string.Empty;
    }

    private static string StepName(TurnStepView step) => step switch
    {
        TurnStepView.Untap => "Untap",
        TurnStepView.Upkeep => "Upkeep",
        TurnStepView.Draw => "Draw",
        TurnStepView.PrecombatMain => "Main 1",
        TurnStepView.BeginCombat => "Beginning of combat",
        TurnStepView.DeclareAttackers => "Declare attackers",
        TurnStepView.DeclareBlockers => "Declare blockers",
        TurnStepView.CombatDamage => "Combat damage",
        TurnStepView.EndOfCombat => "End of combat",
        TurnStepView.PostcombatMain => "Main 2",
        TurnStepView.End => "End step",
        TurnStepView.Cleanup => "Cleanup",
        _ => step.ToString(),
    };

    /// <summary>
    /// Shows one view beside the prompt: the hand filling the phone's height; one player's battlefield; one player's public
    /// state - their rail and their graveyard, opened beside it; the stack, its cards as large as the phone allows; or the
    /// stops.
    /// </summary>
    private void ShowPhoneView(PhoneView view, int seat)
    {
        _phoneView = (view, seat);
        int me = _gameClient?.Seat is > 0 and var known ? known : 1;
        bool hand = view == PhoneView.Hand;
        bool table = view is PhoneView.Board or PhoneView.Player or PhoneView.Stack;
        bool mine = view is PhoneView.Board or PhoneView.Player && seat == me;
        bool theirs = view is PhoneView.Board or PhoneView.Player && seat != me;
        _boardShown = view == PhoneView.Board;

        // The table's row, the timeline's and the hand's: the one the view is in takes the height.
        Root.RowDefinitions[0].Height = hand ? new GridLength(0) : new GridLength(1, GridUnitType.Star);
        Root.RowDefinitions[2].Height = hand ? new GridLength(1, GridUnitType.Star) : new GridLength(0);
        _bottom.Visibility = hand ? Visibility.Visible : Visibility.Collapsed;
        _table.Visibility = table ? Visibility.Visible : Visibility.Collapsed;
        _phoneStops.Visibility = view == PhoneView.Stops ? Visibility.Visible : Visibility.Collapsed;

        _table.RowDefinitions[0].Height = theirs || view == PhoneView.Stack ? new GridLength(1, GridUnitType.Star) : new GridLength(0);
        _table.RowDefinitions[1].Height = new GridLength(0);
        _table.RowDefinitions[2].Height = mine ? new GridLength(1, GridUnitType.Star) : new GridLength(0);
        _opponentHalf.Panel.Visibility = theirs ? Visibility.Visible : Visibility.Collapsed;
        _myHalf.Panel.Visibility = mine ? Visibility.Visible : Visibility.Collapsed;

        // A player's state is their rail with their graveyard beside it, in place of their battlefield.
        bool state = view == PhoneView.Player;
        _opponentHalf.Scroll.Visibility = state ? Visibility.Collapsed : Visibility.Visible;
        _myHalf.Scroll.Visibility = state ? Visibility.Collapsed : Visibility.Visible;
        if (state != _graveyardsOpen)
        {
            SetGraveyardsOpen(state);
        }

        FillPhoneHand();
        PaintDrawer();
        _needsRedraw = true;
        RedrawNow();
    }

    /// <summary>How many cards the hand was last sized for.</summary>
    private int _phoneHandSizedFor = -1;

    /// <summary>
    /// The hand's cards as large as the hand's view lets every one of them show: as tall as the phone while a few fit side
    /// by side, and smaller as the hand grows, so each card laid over the next still shows half its width - its name and
    /// cost - to be read and tapped.
    /// </summary>
    private void FillPhoneHand()
    {
        if (_phoneView.View != PhoneView.Hand || Root.ActualHeight <= 0)
        {
            return;
        }

        int count = _gameClient?.View is null ? 0 : _gameClient.MyHand().Count();
        _phoneHandSizedFor = count;
        float tall = (float)(Root.ActualHeight - Root.Padding.Top - Root.Padding.Bottom);
        float across = (float)(Root.ActualWidth - Root.Padding.Left - Root.Padding.Right - DrawerStrip - PhoneDockWidth()) - (2f * HandMarginX);
        float wide = across / (1f + (0.5f * Math.Max(0, count - 1)));
        float fits = (wide * 88f / 63f) + (2f * HandMarginY) + 4f;
        SetHandHeight(MathF.Max(MinHandHeight, MathF.Min(tall, fits)));
    }

    /// <summary>
    /// Keeps the view with the game, on the page's timer: the drawer's counts; the stack opened while triggers are put in
    /// order there, since each one is tapped on it, and the view before it shown again once they are; the hand again when
    /// a game begins; the stops' step.
    /// </summary>
    private void FollowPhoneView()
    {
        if (_drawer is null)
        {
            return;
        }

        GameClient? client = _gameClient;
        if (_fullScreen is not null)
        {
            _fullScreen.Landscape = client?.View is not null && client.Table?.Phase is not (TablePhase.Drafting or TablePhase.DeckBuilding);
        }

        MakeDrawerButtons(client?.View);
        PaintDrawer();
        if ((client?.View is null ? 0 : client.MyHand().Count()) != _phoneHandSizedFor)
        {
            FillPhoneHand();
        }

        _phoneStops.Follow(client?.View is { } view && !client.IsOver ? view.Step : null);

        bool ordering = client is not null && client.View is not null && StagedTriggers(client).Any();
        if (ordering && _phoneView.View != PhoneView.Stack)
        {
            _beforeOrdering = _phoneView;
            ShowPhoneView(PhoneView.Stack, 0);
        }
        else if (!ordering && _beforeOrdering is { } before)
        {
            _beforeOrdering = null;
            ShowPhoneView(before.View, before.Seat);
        }
    }

    /// <summary>
    /// Whether another player's stop stops this player in a step, on this player's turn or another's: on this player's turn
    /// wherever another stops on an opponent's turn, and on another's wherever the player whose turn it is stops on their
    /// own, or someone else on an opponent's (docs/in-person.md, "Stops").
    /// </summary>
    private bool OthersStopAt(TurnStepView step, bool myTurn)
    {
        int me = _gameClient?.ActingSeat ?? _gameClient?.Seat ?? 0;
        int active = _gameClient?.View?.ActivePlayer ?? 0;
        return _tableStops.Where(each => each.Seat != me).Any(each =>
            (myTurn ? each.Theirs : active == me || active == 0 || each.Seat == active ? each.Mine : each.Theirs).Contains(step));
    }

    /// <summary>The stack's view: the window shown even empty, with its cards as tall as the table's row allows.</summary>
    private void FollowPhoneStack(int count)
    {
        bool shown = _phoneView.View == PhoneView.Stack;
        _stackOverlay.Visibility = shown && count > 0 ? Visibility.Visible : Visibility.Collapsed;
        _phoneStackEmpty.Visibility = shown && count == 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    /// <summary>The prompt's column: a third of the phone's width, within what its buttons and words need.</summary>
    private double PhoneDockWidth() => Math.Clamp(Math.Round((Root.ActualWidth - Root.Padding.Left - Root.Padding.Right) * 0.34), 240, 360);

    /// <summary>The phone's view for its test state: hand, board:N, player:N, stack or stops.</summary>
    private string PhoneViewName() => _phoneView.View switch
    {
        PhoneView.Board => $"board:{_phoneView.Seat}",
        PhoneView.Player => $"player:{_phoneView.Seat}",
        _ => _phoneView.View.ToString().ToLowerInvariant(),
    };
}
