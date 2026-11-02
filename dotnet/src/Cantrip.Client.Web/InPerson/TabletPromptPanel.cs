using Cantrip.Protocol.InPerson;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Automation;

namespace Cantrip.Client.Web.InPerson;

/// <summary>
/// The prompt on a tablet (docs/in-person.md, "The prompt on the table"): what the asked player's phone asks them and the
/// buttons that answer it, as the phone said them (<see cref="TablePromptMessage"/>), in the column down the board's left
/// edge between the players' rails, turned to face that player. One player is asked at a time, so there is one. A button or a toggle tapped here is sent to the relay for that phone to press
/// (<see cref="TablePressMessage"/>); nothing on the tablet answers it, and the phone and the table judge it. A choice among
/// cards only that player may see shows only that they are making it.
/// </summary>
public sealed class TabletPromptPanel : Border
{
    /// <summary>How wide the column it stands in is: a rail's width, with room for a few finger-sized buttons a row.</summary>
    public const double PanelWidth = 232;

    private readonly TextBlock _who;
    private readonly SymbolText _title;
    private readonly SymbolText _detail;
    private readonly TextBlock _error;
    private readonly StackPanel _rows;
    private readonly List<(PromptItem Item, FrameworkElement Shown)> _items = [];
    private PromptCard? _shown;
    private string _name = string.Empty;

    public TabletPromptPanel()
    {
        Background = Theme.Brush(Theme.Panel);
        BorderBrush = Theme.Brush(Theme.Line);
        BorderThickness = new Thickness(0, 2, 0, 2);
        Padding = new Thickness(10, 8, 10, 8);

        _who = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.FgMuted), TextTrimming = TextTrimming.CharacterEllipsis };
        _title = new SymbolText(16, Theme.Fg, semibold: true, wrap: true);
        _detail = new SymbolText(13, Theme.FgSoft, wrap: true);
        _error = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.Danger), TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed };
        _rows = new StackPanel { Spacing = 6, Margin = new Thickness(0, 4, 0, 0) };
        Child = new ScrollViewer
        {
            VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
            Content = new StackPanel { Spacing = 3, Children = { _who, _title, _detail, _error, _rows } },
        };
    }

    /// <summary>An item tapped here: the prompt's version, and the item's id.</summary>
    public event Action<int, int>? Pressed;

    /// <summary>The prompt shown, for a test; null before the phone has said one.</summary>
    public PromptCard? Shown => _shown;

    /// <summary>The items shown and where each is drawn, for a test.</summary>
    public IReadOnlyList<(PromptItem Item, FrameworkElement Shown)> Items => _items;

    /// <summary>Shows a seat's prompt, made again only when it changed; <paramref name="name"/> is the player's.</summary>
    public void Show(PromptCard? prompt, string name, bool connected)
    {
        Opacity = connected ? 1 : 0.55;

        // Edged in the accent while its player is the one the table waits on.
        BorderBrush = Theme.Brush(prompt is { Asking: true } ? Theme.Accent : Theme.Line);
        if (ReferenceEquals(prompt, _shown) && name == _name)
        {
            return;
        }

        _shown = prompt;
        _name = name;
        _items.Clear();
        _rows.Children.Clear();
        _who.Text = name;
        if (prompt is null)
        {
            _title.Text = "Waiting for their phone";
            _detail.Text = string.Empty;
            _error.Visibility = Visibility.Collapsed;
            return;
        }

        if (prompt.Private)
        {
            // Shown as a player at a table in paper is seen looking through their library: that they are, and no more.
            _title.Text = "Choosing on their phone";
            _detail.Text = "A choice only they may see.";
            _error.Visibility = Visibility.Collapsed;
            return;
        }

        _title.Text = prompt.Title;
        _detail.Text = prompt.Detail;
        _detail.Visibility = prompt.Detail.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
        _error.Text = prompt.Error;
        _error.Visibility = prompt.Error.Length > 0 ? Visibility.Visible : Visibility.Collapsed;

        int version = prompt.Version;
        foreach (PromptRow row in prompt.Rows)
        {
            var line = new FlowPanel { HorizontalSpacing = 6, VerticalSpacing = 6 };
            foreach (PromptItem item in row.Items)
            {
                FrameworkElement shown;
                if (item.Kind == PromptItemKind.Words)
                {
                    shown = new SymbolText(15, Theme.Fg, semibold: true) { Text = item.Text, VerticalAlignment = VerticalAlignment.Center, Margin = new Thickness(2, 10, 2, 0) };
                }
                else
                {
                    // A toggle is a button held down while it is on; every button is a finger's size.
                    var button = new DockButton(item.Text, item.Primary, fontSize: item.Kind == PromptItemKind.Toggle ? 12 : 15, enabled: item.Enabled)
                    {
                        MinHeight = item.Kind == PromptItemKind.Toggle ? 32 : 44,
                        MinWidth = item.Kind == PromptItemKind.Toggle ? 0 : 64,
                        On = item.On,
                    }.Wrapping(PanelWidth - 22);
                    AutomationProperties.SetName(button, item.Text);
                    int id = item.Id;
                    button.Clicked += () => Pressed?.Invoke(version, id);
                    shown = button;
                }

                line.Children.Add(shown);
                _items.Add((item, shown));
            }

            _rows.Children.Add(line);
        }
    }
}
