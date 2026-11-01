using Cantrip.Protocol;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Automation;

namespace Cantrip.Client.Web.InPerson;

/// <summary>
/// A phone's Stops view (MainPage.PhoneViews.cs): every step a player gets priority in, with a button for stopping there on
/// this player's turn and one for the opponent's, a finger's size each, lit while the stop is set; and where another player
/// at the table set a stop of their own, which stops this player too (docs/in-person.md, "Stops"). The step being played is
/// marked. The stops are the page's own <see cref="TurnStops"/>, which the phone tells the table as they change.
/// </summary>
public sealed class PhoneStopsPanel : Grid
{
    private static readonly (TurnStepView Step, string Caption)[] Steps =
    [
        (TurnStepView.Upkeep, "Upkeep"),
        (TurnStepView.Draw, "Draw"),
        (TurnStepView.PrecombatMain, "Main 1"),
        (TurnStepView.BeginCombat, "Combat"),
        (TurnStepView.DeclareAttackers, "Attack"),
        (TurnStepView.DeclareBlockers, "Block"),
        (TurnStepView.CombatDamage, "Damage"),
        (TurnStepView.EndOfCombat, "End combat"),
        (TurnStepView.PostcombatMain, "Main 2"),
        (TurnStepView.End, "End"),
    ];

    private readonly TurnStops _stops;
    private readonly Func<TurnStepView, bool, bool> _othersStop;
    private readonly List<(TurnStepView Step, bool Mine, DockButton Button)> _buttons = [];
    private readonly Dictionary<TurnStepView, (TextBlock Name, TextBlock Others)> _rows = [];
    private TurnStepView? _current;

    /// <param name="stops">This player's stops.</param>
    /// <param name="othersStop">For a step and whether it is this player's turn, whether another player's stop stops them there.</param>
    public PhoneStopsPanel(TurnStops stops, Func<TurnStepView, bool, bool> othersStop)
    {
        _stops = stops;
        _othersStop = othersStop;
        Background = Theme.Brush(Theme.Window);
        Padding = new Thickness(12, 8, 12, 8);
        RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });

        Children.Add(new StackPanel
        {
            Spacing = 2,
            Margin = new Thickness(4, 0, 0, 6),
            Children =
            {
                new TextBlock { Text = "Stops", FontSize = 18, FontWeight = FontWeights.SemiBold, Foreground = Theme.Brush(Theme.Fg) },
                new TextBlock
                {
                    Text = "At this table every player stops wherever anyone set a stop.",
                    FontSize = 12,
                    Foreground = Theme.Brush(Theme.FgSoft),
                    TextWrapping = TextWrapping.WrapWholeWords,
                },
            },
        });

        // Two columns of steps, the turn's first half and its second, so every one fits a phone held sideways.
        var columns = new Grid { ColumnSpacing = 18 };
        columns.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        columns.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        Grid.SetRow(columns, 1);
        Children.Add(columns);

        int half = (Steps.Length + 1) / 2;
        for (int column = 0; column < 2; column++)
        {
            var list = new Grid { ColumnSpacing = 6, RowSpacing = 4 };
            list.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            list.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            list.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            Grid.SetColumn(list, column);
            columns.Children.Add(list);

            foreach ((TurnStepView step, string caption) in Steps.Skip(column * half).Take(half))
            {
                int row = list.RowDefinitions.Count;
                list.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
                var name = new TextBlock { Text = caption, FontSize = 14, Foreground = Theme.Brush(Theme.Fg), VerticalAlignment = VerticalAlignment.Center, TextWrapping = TextWrapping.WrapWholeWords };
                var others = new TextBlock { FontSize = 11, Foreground = Theme.Brush(Theme.FgMuted), TextTrimming = TextTrimming.CharacterEllipsis };
                var words = new StackPanel { VerticalAlignment = VerticalAlignment.Center, Children = { name, others } };
                Grid.SetRow(words, row);
                list.Children.Add(words);
                _rows[step] = (name, others);

                foreach (bool mine in (bool[])[true, false])
                {
                    var button = new DockButton(mine ? "My turn" : "Theirs", primary: false, fontSize: 13) { MinHeight = 38, MinWidth = 76 }.Filling();
                    AutomationProperties.SetName(button, $"Stop: {caption}, {(mine ? "my turn" : "their turn")}");
                    TurnStepView toggled = step;
                    bool turn = mine;
                    button.Clicked += () => _stops.Toggle(toggled, turn);
                    Grid.SetRow(button, row);
                    Grid.SetColumn(button, mine ? 1 : 2);
                    list.Children.Add(button);
                    _buttons.Add((step, mine, button));
                }
            }
        }

        _stops.Changed += Paint;
        Paint();
    }

    /// <summary>The step being played, marked in the list; null between games.</summary>
    public void Follow(TurnStepView? current)
    {
        if (current != _current)
        {
            _current = current;
            Paint();
        }
    }

    /// <summary>Lights each stop set, and says where another player's stop stops this player too.</summary>
    public void Paint()
    {
        foreach ((TurnStepView step, bool mine, DockButton button) in _buttons)
        {
            button.On = _stops.OwnStopAt(step, mine);
        }

        foreach ((TurnStepView step, (TextBlock name, TextBlock others)) in _rows)
        {
            bool onMine = _othersStop(step, true);
            bool onTheirs = _othersStop(step, false);
            others.Text = (onMine, onTheirs) switch
            {
                (true, true) => "Opponent stops: both turns",
                (true, false) => "Opponent stops: your turn",
                (false, true) => "Opponent stops: their turn",
                _ => string.Empty,
            };
            others.Visibility = others.Text.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
            name.FontWeight = step == _current ? FontWeights.SemiBold : FontWeights.Normal;
            name.Foreground = Theme.Brush(step == _current ? Theme.Accent : Theme.Fg);
        }
    }

    /// <summary>The panel's buttons for a test: each one's name and whether it is lit.</summary>
    public IEnumerable<(string Name, bool On)> Buttons() =>
        _buttons.Select(each => (AutomationProperties.GetName(each.Button), each.Button.On));
}
