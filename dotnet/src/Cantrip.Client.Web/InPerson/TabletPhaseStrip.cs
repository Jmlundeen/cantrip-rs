using Cantrip.Protocol;
using Cantrip.Protocol.InPerson;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml.Shapes;

namespace Cantrip.Client.Web.InPerson;

/// <summary>
/// The turn as a tablet shows it, along the edge of a seat's half nearest the middle of the table: whose turn it is, then a
/// pill per step, the current one filled - green on the turn of the player this half is for, amber on the other's - and
/// a dot under each step the table stops in this turn. At an in-person table a stop is the table's (docs/in-person.md,
/// "Stops"): the active player's stops on their own turn and every other player's on an opponent's, as each phone said
/// them (<see cref="TableStopsMessage"/>). Nothing on it is tapped: stops are set on the phones.
/// </summary>
public sealed class TabletPhaseStrip : Border
{
    private static readonly (TurnStepView Step, string Caption)[] Steps =
    [
        (TurnStepView.Untap, "Untap"),
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
        (TurnStepView.Cleanup, "Cleanup"),
    ];

    private readonly int _seat;
    private readonly TextBlock _turn;
    private readonly Dictionary<TurnStepView, (Border Pill, TextBlock Label, Ellipse Stop)> _steps = [];

    /// <param name="seat">The seat whose half this strip runs along, whose turn is shown in their own colour.</param>
    public TabletPhaseStrip(int seat)
    {
        _seat = seat;
        Background = Theme.Brush(Theme.Bar);
        BorderBrush = Theme.Brush(Theme.Line);
        BorderThickness = new Thickness(0, 0, 0, 1);
        Padding = new Thickness(12, 3, 12, 3);

        var grid = new Grid { ColumnSpacing = 2 };
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(PlayerRail.RailWidth - 12) });
        _turn = new TextBlock
        {
            FontSize = 13,
            FontWeight = FontWeights.SemiBold,
            Foreground = Theme.Brush(Theme.Fg),
            VerticalAlignment = VerticalAlignment.Center,
            TextTrimming = TextTrimming.CharacterEllipsis,
        };
        grid.Children.Add(_turn);

        foreach ((TurnStepView step, string caption) in Steps)
        {
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            var label = new TextBlock
            {
                Text = caption,
                FontSize = 12,
                HorizontalAlignment = HorizontalAlignment.Center,
                TextTrimming = TextTrimming.CharacterEllipsis,
            };
            var stop = new Ellipse { Width = 6, Height = 6, HorizontalAlignment = HorizontalAlignment.Center, Visibility = Visibility.Collapsed };
            var pill = new Border
            {
                CornerRadius = new CornerRadius(9),
                Padding = new Thickness(4, 2, 4, 2),
                Child = new StackPanel { Spacing = 1, Children = { label, stop } },
            };
            Grid.SetColumn(pill, grid.ColumnDefinitions.Count - 1);
            grid.Children.Add(pill);
            _steps[step] = (pill, label, stop);
        }

        Child = grid;
    }

    /// <summary>The step the strip shows as current, for a test.</summary>
    public TurnStepView Current { get; private set; }

    /// <summary>The steps marked as stops this turn, for a test.</summary>
    public IReadOnlyList<TurnStepView> Stops { get; private set; } = [];

    /// <summary>The turn as the view has it, and the stops each seat's phone set.</summary>
    /// <param name="turn">Whose turn and which, or how the game ended.</param>
    public void Show(PlayerView view, string turn, IReadOnlyList<SeatStops> stops)
    {
        _turn.Text = turn;
        Current = view.Step;
        int active = view.ActivePlayer;
        bool mine = active == _seat;
        List<TurnStepView> stopping = [];
        int index = Array.FindIndex(Steps, each => each.Step == view.Step);
        for (int i = 0; i < Steps.Length; i++)
        {
            TurnStepView step = Steps[i].Step;
            (Border pill, TextBlock label, Ellipse stop) = _steps[step];
            bool here = i == index;
            pill.Background = here ? Theme.Brush(mine ? Theme.Mine : Theme.Theirs) : Theme.Brush(Windows.UI.Color.FromArgb(0, 0, 0, 0));
            label.Foreground = Theme.Brush(here ? Theme.FgOnAccent : i < index ? Theme.FgMuted : Theme.FgSoft);
            label.FontWeight = here ? FontWeights.SemiBold : FontWeights.Normal;

            bool stopsHere = TurnStops.Stoppable(step) && StopsIn(stops, active, step);
            stop.Visibility = stopsHere ? Visibility.Visible : Visibility.Collapsed;
            stop.Fill = Theme.Brush(here ? Theme.FgOnAccent : mine ? Theme.Mine : Theme.Theirs);
            if (stopsHere)
            {
                stopping.Add(step);
            }
        }

        Stops = stopping;
    }

    /// <summary>Whether the table stops in a step of the active player's turn: their own stop there, or another player's on an opponent's turn.</summary>
    public static bool StopsIn(IReadOnlyList<SeatStops> stops, int active, TurnStepView step) =>
        stops.Any(each => (each.Seat == active ? each.Mine : each.Theirs).Contains(step));
}
