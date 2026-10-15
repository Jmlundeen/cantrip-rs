using Cantrip.Client;
using Cantrip.Protocol;
using Microsoft.UI.Xaml.Automation;

namespace Cantrip.Client.Web;

/// <summary>
/// An event's table between its rounds: the round's pairings and how every match is going, the clock on the next round,
/// and the standings - the final ones once the event is over. A player comes back here when their match is over, and
/// watches the other matches finish; the next round begins a minute after the last one does.
/// </summary>
public sealed partial class MainPage
{
    private readonly Grid _pairingsPanel = new()
    {
        Background = Theme.Brush(Theme.Window),
        Visibility = Visibility.Collapsed,
        Padding = new Thickness(28, 20, 28, 20),
    };

    private TextBlock _pairingsTitle = null!;
    private TextBlock _pairingsLine = null!;
    private TextBlock _roundClock = null!;
    private StackPanel _pairingsRows = null!;
    private StackPanel _standingsRows = null!;
    private TextBlock _pairingsHeading = null!;
    private DockButton _dropButton = null!;
    private StackPanel _dropConfirm = null!;
    private PairingsMessage? _pairingsShown;
    private DateTime? _roundBy;

    /// <summary>Whether the pairings are up: a page of its own, ready for a test to read.</summary>
    private bool PairingsShown => _pairingsPanel.Visibility == Visibility.Visible;

    private void BuildPairingsPanel()
    {
        _pairingsPanel.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        _pairingsPanel.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });

        var header = new StackPanel { Spacing = 4, Margin = new Thickness(0, 0, 0, 18) };
        _pairingsTitle = EventText(string.Empty, 24, Theme.Fg, semibold: true);
        _pairingsLine = EventText(string.Empty, 15, Theme.FgSoft);
        _roundClock = EventText(string.Empty, 18, Theme.Accent, semibold: true);
        AutomationProperties.SetName(_roundClock, "Round clock");
        header.Children.Add(_pairingsTitle);
        header.Children.Add(_pairingsLine);
        header.Children.Add(_roundClock);
        var top = new Grid { ColumnSpacing = 24 };
        top.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        top.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        top.Children.Add(header);
        StackPanel drop = BuildDrop();
        Grid.SetColumn(drop, 1);
        top.Children.Add(drop);
        _pairingsPanel.Children.Add(top);

        var body = new Grid { ColumnSpacing = 36 };
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        Grid.SetRow(body, 1);
        _pairingsPanel.Children.Add(body);

        var pairings = new StackPanel { Spacing = 8 };
        _pairingsHeading = EventText("Pairings", 16, Theme.Fg, semibold: true);
        pairings.Children.Add(_pairingsHeading);
        _pairingsRows = new StackPanel { Spacing = 6 };
        pairings.Children.Add(_pairingsRows);
        body.Children.Add(new ScrollViewer { Content = pairings, VerticalScrollBarVisibility = ScrollBarVisibility.Auto });

        var standings = new StackPanel { Spacing = 8 };
        standings.Children.Add(EventText("Standings", 16, Theme.Fg, semibold: true));
        _standingsRows = new StackPanel { Spacing = 2 };
        standings.Children.Add(_standingsRows);
        var standingsScroll = new ScrollViewer { Content = standings, VerticalScrollBarVisibility = ScrollBarVisibility.Auto };
        Grid.SetColumn(standingsScroll, 1);
        body.Children.Add(standingsScroll);
    }

    /// <summary>
    /// Dropping from the event, in the header's far corner: a button, and once it is pressed the question again, since a
    /// player who drops is never paired again.
    /// </summary>
    private StackPanel BuildDrop()
    {
        var corner = new StackPanel
        {
            HorizontalAlignment = HorizontalAlignment.Right,
            VerticalAlignment = VerticalAlignment.Top,
        };

        _dropButton = new DockButton("Drop", primary: false);
        _dropButton.HorizontalAlignment = HorizontalAlignment.Right;
        _dropButton.Tooltip = "Leave the event: you are not paired again, and your standing is kept as it is.";
        corner.Children.Add(_dropButton);

        var yes = new DockButton("Yes, drop", primary: true).Filling();
        var no = new DockButton("Stay in", primary: false).Filling();
        var answers = new Grid { ColumnSpacing = 8 };
        answers.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        answers.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        answers.Children.Add(yes);
        Grid.SetColumn(no, 1);
        answers.Children.Add(no);

        _dropConfirm = new StackPanel
        {
            Width = 300,
            Spacing = 10,
            Padding = new Thickness(12),
            Background = Theme.Brush(Theme.WithAlpha(Theme.Danger, 0.12)),
            BorderBrush = Theme.Brush(Theme.Danger),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Visibility = Visibility.Collapsed,
        };
        _dropConfirm.Children.Add(EventText("Drop from the event? You will not be paired again, and it cannot be taken back.", 13, Theme.Fg));
        _dropConfirm.Children.Add(answers);
        corner.Children.Add(_dropConfirm);

        void Asking(bool asking)
        {
            _dropConfirm.Visibility = asking ? Visibility.Visible : Visibility.Collapsed;
            _dropButton.Visibility = asking ? Visibility.Collapsed : Visibility.Visible;
        }

        _dropButton.Clicked += () => Asking(true);
        no.Clicked += () => Asking(false);
        yes.Clicked += () =>
        {
            Asking(false);
            _dropButton.Visibility = Visibility.Collapsed;
            _gameClient?.Drop();
        };

        return corner;
    }

    /// <summary>Whether this seat can drop: the event is still going, and it is still in it.</summary>
    private static bool CanDrop(PairingsMessage pairings, int me) =>
        !pairings.Final && pairings.Standings.Any(standing => standing.Seat == me && !standing.Dropped);

    /// <summary>Shows the pairings while the event's table is between rounds, and hides them otherwise.</summary>
    private void ShowPairings(GameClient client)
    {
        bool shown = client.Table?.Phase == TablePhase.Pairings && client.Pairings is not null;
        _pairingsPanel.Visibility = shown ? Visibility.Visible : Visibility.Collapsed;
        if (!shown || ReferenceEquals(_pairingsShown, client.Pairings))
        {
            return;
        }

        PairingsMessage pairings = client.Pairings!;
        _pairingsShown = pairings;
        if (!CanDrop(pairings, client.Seat))
        {
            _dropConfirm.Visibility = Visibility.Collapsed;
        }

        _dropButton.Visibility = CanDrop(pairings, client.Seat) && _dropConfirm.Visibility != Visibility.Visible
            ? Visibility.Visible
            : Visibility.Collapsed;
        string name = client.Table?.EventName is { Length: > 0 } eventName ? eventName : "Draft";
        _pairingsTitle.Text = pairings.Final ? $"{name}  ·  final standings" : $"{name}  ·  round {pairings.Round} of {pairings.Rounds}";
        _pairingsLine.Text = pairings.Detail;
        _roundBy = pairings.StartsInMs > 0 ? DateTime.UtcNow + TimeSpan.FromMilliseconds(pairings.StartsInMs) : null;
        ShowRoundClock();

        _pairingsHeading.Text = pairings.Final ? $"Round {pairings.Round}" : "Pairings";
        _pairingsRows.Children.Clear();
        foreach (PairingView match in pairings.Matches)
        {
            _pairingsRows.Children.Add(PairingRow(match, client.Seat));
        }

        _standingsRows.Children.Clear();
        _standingsRows.Children.Add(StandingRow(null, client.Seat));
        foreach (StandingView standing in pairings.Standings)
        {
            _standingsRows.Children.Add(StandingRow(standing, client.Seat));
        }
    }

    /// <summary>The clock on the next round, counted down here from what the server said was left.</summary>
    private void ShowRoundClock()
    {
        if (_roundClock is null)
        {
            return;
        }

        if (_roundBy is not { } by || _pairingsShown is not { } pairings)
        {
            _roundClock.Text = string.Empty;
            return;
        }

        TimeSpan left = by - DateTime.UtcNow;
        if (left < TimeSpan.Zero) left = TimeSpan.Zero;
        _roundClock.Text = $"Round {pairings.Round} begins in {(int)left.TotalMinutes}:{left.Seconds:00}";
    }

    /// <summary>One match: the table, both players and the games each has won, and how it stands. This seat's lit.</summary>
    private static Border PairingRow(PairingView match, int me)
    {
        bool mine = match.FirstSeat == me || match.SecondSeat == me;
        var row = new Grid { ColumnSpacing = 12 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(64) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1.1, GridUnitType.Star) });

        row.Children.Add(EventText($"Table {match.Table}", 13, Theme.FgMuted));

        string players = match.SecondSeat == 0
            ? $"{match.First}  ·  bye"
            : $"{match.First}  {match.FirstWins}-{match.SecondWins}{(match.Draws > 0 ? $"-{match.Draws}" : string.Empty)}  {match.Second}";
        TextBlock who = EventText(players, 15, Theme.Fg, semibold: mine);
        Grid.SetColumn(who, 1);
        row.Children.Add(who);

        TextBlock status = EventText(match.Status, 13, match.Complete ? Theme.FgSoft : Theme.Accent);
        Grid.SetColumn(status, 2);
        row.Children.Add(status);

        var border = new Border
        {
            Background = Theme.Brush(mine ? Theme.WithAlpha(Theme.Mine, 0.18) : Theme.Panel),
            BorderBrush = Theme.Brush(mine ? Theme.Mine : Theme.Line),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(12, 9, 12, 9),
            Child = row,
        };
        AutomationProperties.SetName(border, $"Table {match.Table}: {players}, {match.Status}");
        return border;
    }

    /// <summary>One line of the standings, or - with no standing - the line naming the columns.</summary>
    private static Grid StandingRow(StandingView? standing, int me)
    {
        var row = new Grid { ColumnSpacing = 10, Padding = new Thickness(8, 4, 8, 4) };
        foreach (double width in (double[])[34, -1, 52, 70, 64, 64, 64])
        {
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = width < 0 ? new GridLength(1, GridUnitType.Star) : new GridLength(width) });
        }

        bool mine = standing?.Seat == me;
        string[] cells = standing is null
            ? ["#", "Player", "Points", "Record", "OMW", "GW", "OGW"]
            :
            [
                standing.Rank.ToString(System.Globalization.CultureInfo.InvariantCulture),
                standing.Dropped ? $"{standing.Name} (dropped)" : standing.Name,
                standing.Points.ToString(System.Globalization.CultureInfo.InvariantCulture),
                $"{standing.Wins}-{standing.Losses}-{standing.Draws}",
                Percent(standing.OpponentMatchWin),
                Percent(standing.GameWin),
                Percent(standing.OpponentGameWin),
            ];

        for (int i = 0; i < cells.Length; i++)
        {
            TextBlock cell = EventText(cells[i], standing is null ? 12 : 14,
                standing is null ? Theme.FgMuted : standing.Dropped ? Theme.FgMuted : mine ? Theme.Mine : Theme.Fg, semibold: mine);
            Grid.SetColumn(cell, i);
            row.Children.Add(cell);
        }

        if (mine)
        {
            row.Background = Theme.Brush(Theme.WithAlpha(Theme.Mine, 0.12));
        }

        return row;
    }

    private static string Percent(double value) => $"{value * 100:0.0}%";
}
