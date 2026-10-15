using Microsoft.UI.Text;

namespace Cantrip.Client.Web;

/// <summary>
/// The game tab of the side panel: what a player can do to the game as a whole rather than within it. For now that is
/// conceding, which asks once more before it is sent - a concession cannot be taken back, and the button is one stray
/// click from the log tab.
/// </summary>
public sealed class GameMenuPanel : ScrollViewer
{
    private readonly DockButton _concede;
    private readonly TextBlock _note;
    private readonly StackPanel _confirm;
    private bool _playing;

    public GameMenuPanel()
    {
        HorizontalScrollMode = ScrollMode.Disabled;
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled;
        VerticalScrollMode = ScrollMode.Auto;
        VerticalScrollBarVisibility = ScrollBarVisibility.Auto;

        var rows = new StackPanel { Orientation = Orientation.Vertical, Spacing = 8 };
        Content = new Border { Padding = new Thickness(16, 12, 16, 14), Child = rows };

        rows.Children.Add(new TextBlock
        {
            Text = "THIS GAME",
            FontSize = 10,
            FontWeight = FontWeights.SemiBold,
            Foreground = Theme.Brush(Theme.FgMuted),
        });

        _concede = new DockButton("Concede", primary: false).Filling();
        _concede.Tooltip = "Lose this game now. In a match, the next game is played as after any other loss.";
        _concede.Clicked += () => Confirming(true);
        rows.Children.Add(_concede);

        _note = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.FgMuted), TextWrapping = TextWrapping.Wrap };
        rows.Children.Add(_note);

        var question = new TextBlock
        {
            Text = "Concede this game? You lose it, and it cannot be taken back.",
            FontSize = 13,
            Foreground = Theme.Brush(Theme.Fg),
            TextWrapping = TextWrapping.Wrap,
        };

        var yes = new DockButton("Yes, concede", primary: true).Filling();
        yes.Clicked += () =>
        {
            Confirming(false);
            if (_playing)
            {
                ConcedeRequested?.Invoke();
            }
        };

        var no = new DockButton("Keep playing", primary: false).Filling();
        no.Clicked += () => Confirming(false);

        var answers = new Grid { ColumnSpacing = 8 };
        answers.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        answers.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        answers.Children.Add(yes);
        Grid.SetColumn(no, 1);
        answers.Children.Add(no);

        _confirm = new StackPanel
        {
            Spacing = 10,
            Padding = new Thickness(12),
            Background = Theme.Brush(Theme.WithAlpha(Theme.Danger, 0.12)),
            BorderBrush = Theme.Brush(Theme.Danger),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Visibility = Visibility.Collapsed,
        };
        _confirm.Children.Add(question);
        _confirm.Children.Add(answers);
        rows.Children.Add(_confirm);

        ShowPlaying(false);
    }

    /// <summary>Raised when the player has asked to concede and said yes when asked again.</summary>
    public event Action? ConcedeRequested;

    /// <summary>Whether there is a game going on to concede. When there is not, the button says so and does nothing.</summary>
    public void ShowPlaying(bool playing)
    {
        if (playing == _playing && _note.Text.Length > 0)
        {
            return;
        }

        _playing = playing;
        _concede.Enabled = playing;
        _note.Text = playing ? "You lose this game at once, whatever is being asked." : "There is no game being played to concede.";
        if (!playing)
        {
            Confirming(false);
        }
    }

    private void Confirming(bool asking)
    {
        _confirm.Visibility = asking && _playing ? Visibility.Visible : Visibility.Collapsed;
        _concede.Visibility = _confirm.Visibility == Visibility.Visible ? Visibility.Collapsed : Visibility.Visible;
    }
}
