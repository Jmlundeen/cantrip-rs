using Microsoft.UI.Text;
using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The settings tab: the player's preferences, each a row switched by clicking
/// anywhere on it, saved the moment it changes. There is no apply button, because a preference that has to be
/// confirmed is one the player can believe they set and did not.
/// </summary>
public sealed class SettingsPanel : ScrollViewer
{
    private readonly StackPanel _rows = new() { Orientation = Orientation.Vertical, Spacing = 2 };
    private readonly List<HotkeyRow> _hotkeyRows = [];

    public SettingsPanel()
    {
        HorizontalScrollMode = ScrollMode.Disabled;
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled;
        VerticalScrollMode = ScrollMode.Auto;
        VerticalScrollBarVisibility = ScrollBarVisibility.Auto;
        Content = new Border { Padding = new Thickness(8, 12, 8, 14), Child = _rows };

        Section("Cards");

        var animate = new SettingRow(
            "Animate cards",
            "Cards glide to where they are going: they turn as they tap, and grow and shrink back as they are held up to read.",
            PlayerSettings.AnimateCards);
        animate.Toggled += on => PlayerSettings.AnimateCards = on;
        _rows.Children.Add(animate);

        var autoScale = new SettingRow(
            "Fit cards to the table",
            "Permanents are drawn as large as the two rows of a half have room for, and follow the table as the "
                + "window, the hand strip or the side panel changes its height.",
            PlayerSettings.AutoCardScale);
        _rows.Children.Add(autoScale);

        var scale = new SliderRow(
            "Battlefield card size",
            "How large permanents are drawn on the table. The hand grows with its strip instead.",
            PlayerSettings.MinCardScale,
            PlayerSettings.MaxCardScale,
            0.05f,
            PlayerSettings.CardScale,
            value => $"{value * 100f:0}%")
        {
            // Shown saying what it would do rather than hidden, and doing nothing until the player takes the choice back.
            Enabled = !PlayerSettings.AutoCardScale,
        };

        scale.ValueChanged += value => PlayerSettings.CardScale = value;
        _rows.Children.Add(scale);

        autoScale.Toggled += on =>
        {
            PlayerSettings.AutoCardScale = on;
            scale.Enabled = !on;
        };

        var artQuality = new SliderRow(
            "Card art quality",
            "How large illustrations are fetched from the art server. Higher is sharper when a card is held up for a "
                + "close look, and costs more to download and to keep; lower fills a window full of cards sooner. "
                + "Nothing about a card but its picture changes.",
            0f,
            Enum.GetValues<ArtQuality>().Length - 1,
            1f,
            (float)(int)PlayerSettings.ArtQuality,
            value => PlayerSettings.Describe((ArtQuality)(int)value));

        artQuality.ValueChanged += value => PlayerSettings.ArtQuality = (ArtQuality)(int)value;
        _rows.Children.Add(artQuality);

        Section("Playing");

        var autoMana = new SettingRow(
            "Auto-mana",
            "A land that could be tapped for more than one colour is tapped for whichever colour the cost needs "
                + "and the rest of your board is least able to supply, instead of asking. The Auto-mana key then "
                + "does the opposite: hold it to be asked.",
            PlayerSettings.AutoManaByDefault);
        autoMana.Toggled += on => PlayerSettings.AutoManaByDefault = on;
        _rows.Children.Add(autoMana);

        Section("Hotkeys");

        foreach (Hotkey hotkey in Hotkeys.All)
        {
            var row = new HotkeyRow(hotkey);
            row.CaptureStarted += () =>
            {
                // One row waits at a time.
                if (Capturing is { } other && other != row)
                {
                    other.End();
                }

                Capturing = row;
            };
            row.CaptureEnded += () => Capturing = Capturing == row ? null : Capturing;
            _hotkeyRows.Add(row);
            _rows.Children.Add(row);
        }

        DockButton reset = QuietButton("Reset hotkeys to defaults");
        reset.HorizontalAlignment = HorizontalAlignment.Right;
        reset.Margin = new Thickness(4, 4, 4, 0);
        reset.Enabled = !PlayerSettings.HotkeysAreDefault;
        reset.Clicked += PlayerSettings.ResetHotkeys;
        PlayerSettings.Changed += () => reset.Enabled = !PlayerSettings.HotkeysAreDefault;
        _rows.Children.Add(reset);
    }

    /// <summary>
    /// The row waiting for a key, if one is. The page checks this before acting on a hotkey, so the key pressed to choose
    /// is never also the key that passes.
    /// </summary>
    public HotkeyRow? Capturing { get; private set; }

    /// <summary>Stops any row waiting for a key: a click somewhere else, or the panel closing.</summary>
    public void StopCapturing() => Capturing?.End();

    private void Section(string title) => _rows.Children.Add(new TextBlock
    {
        Text = title.ToUpperInvariant(),
        FontSize = 10,
        FontWeight = FontWeights.SemiBold,
        Foreground = Theme.Brush(Theme.FgMuted),
        Margin = new Thickness(8, 4, 8, 4),
    });

    /// <summary>A button with no ground until it is pointed at (Ui.Quiet), in small type.</summary>
    private static DockButton QuietButton(string text) => new DockButton(text, primary: false, fontSize: 12).Quiet();
}

/// <summary>
/// The panel down the right-hand edge: the game log, the settings and the game's menu as tabs, and a
/// button that hides it. One panel rather than several because none is looked at while another is.
/// </summary>
public sealed class SidePanel : Grid
{
    public const double PanelWidth = 320;

    public enum Tab
    {
        Log,
        Settings,
        Menu,
    }

    private readonly Dictionary<Tab, (Border Button, TextBlock Label, FrameworkElement Page)> _tabs = [];

    public SidePanel(LogPanelControl log, SettingsPanel settings, GameMenuPanel menu)
    {
        Log = log;
        Settings = settings;
        Menu = menu;
        Width = PanelWidth;
        HorizontalAlignment = HorizontalAlignment.Left;
        Background = Theme.Brush(Theme.Panel);
        BorderBrush = Theme.Brush(Theme.Line);
        BorderThickness = new Thickness(1, 0, 0, 0);

        RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });

        var header = new Grid
        {
            Background = Theme.Brush(Theme.Bar),
            BorderBrush = Theme.Brush(Theme.Line),
            BorderThickness = new Thickness(0, 0, 0, 1),
            Padding = new Thickness(8, 0, 8, 0),
        };
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        header.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Children.Add(header);

        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 2 };
        header.Children.Add(row);

        foreach ((Tab tab, string text, FrameworkElement page) in (ValueTuple<Tab, string, FrameworkElement>[])[(Tab.Log, "Game log", log), (Tab.Settings, "Settings", settings), (Tab.Menu, "Menu", menu)])
        {
            Grid.SetRow(page, 1);
            Children.Add(page);
            (Border button, TextBlock label) = TabButton(text, tab);
            row.Children.Add(button);
            _tabs[tab] = (button, label, page);
        }

        // The game menu (MainPage.GameMenu), beside the arrow that hides the panel.
        MenuButton = MainPage.GameMenuButton(quiet: true);
        MenuButton.VerticalAlignment = VerticalAlignment.Center;
        MenuButton.Clicked += () => MenuRequested?.Invoke();

        DockButton hide = new DockButton("›", primary: false, fontSize: 18).Quiet();
        hide.VerticalAlignment = VerticalAlignment.Center;
        hide.Tooltip = "Hide the panel" + Hotkeys.Suffix(Hotkeys.TogglePanel);
        hide.Clicked += () => HideRequested?.Invoke();
        var corner = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 2, Children = { MenuButton, hide } };
        Grid.SetColumn(corner, 2);
        header.Children.Add(corner);

        Select(Tab.Log);
    }

    public LogPanelControl Log { get; }

    public SettingsPanel Settings { get; }

    public GameMenuPanel Menu { get; }

    public Tab Current { get; private set; } = Tab.Log;

    /// <summary>Raised when the player presses the panel's own hide button.</summary>
    public event Action? HideRequested;

    /// <summary>The game menu's button, beside the hide button.</summary>
    public DockButton MenuButton { get; }

    /// <summary>Raised when the player presses the game menu's button.</summary>
    public event Action? MenuRequested;

    public void Select(Tab tab)
    {
        Current = tab;

        foreach ((Tab each, (Border button, TextBlock label, FrameworkElement page)) in _tabs)
        {
            bool on = each == tab;
            page.Visibility = on ? Visibility.Visible : Visibility.Collapsed;
            button.BorderBrush = Theme.Brush(on ? Theme.Accent : Color.FromArgb(0, 0, 0, 0));
            label.Foreground = Theme.Brush(on ? Theme.Fg : Theme.FgMuted);
        }

        if (tab != Tab.Settings)
        {
            Settings.StopCapturing();
        }
    }

    /// <summary>What a tab's button is tagged with, so the page's test state can name it as a tab.</summary>
    public const string TabTag = "side-tab";

    /// <summary>A tab as a word with a bar under it when it is the one showing, on the header's own bottom line.</summary>
    private (Border, TextBlock) TabButton(string text, Tab tab)
    {
        var label = new TextBlock { Text = text, FontSize = 13, FontWeight = FontWeights.SemiBold, VerticalAlignment = VerticalAlignment.Center };
        var button = new Border
        {
            Tag = TabTag,
            MinHeight = 38,
            Padding = new Thickness(10, 0, 10, 0),
            BorderThickness = new Thickness(0, 0, 0, 2),
            Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0)),
            Child = label,
        };

        button.PointerEntered += (_, _) =>
        {
            button.Background = Theme.Brush(Color.FromArgb(13, 255, 255, 255));
            label.Foreground = Theme.Brush(Theme.Fg);
        };
        button.PointerExited += (_, _) =>
        {
            button.Background = Theme.Brush(Color.FromArgb(0, 0, 0, 0));
            label.Foreground = Theme.Brush(Current == tab ? Theme.Fg : Theme.FgMuted);
        };
        button.Tapped += (_, _) => Select(tab);
        return (button, label);
    }
}
