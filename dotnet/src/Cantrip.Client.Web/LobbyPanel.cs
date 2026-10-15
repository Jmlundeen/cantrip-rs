using Cantrip.Protocol;
using Microsoft.UI.Text;

namespace Cantrip.Client.Web;

/// <summary>
/// Setting up a table before the game: who is seated and whether they are ready,
/// this player's name and deck, the host's choice of sets, and Ready. Centred over the whole window while the table is
/// being set up. The server validates every message it sends; this only offers them.
/// </summary>
public sealed class LobbyPanel : Grid
{
    private TableMessage? _shown;
    private GameClient? _client;
    private int _errorCount = -1;
    private SeatReleasedMessage? _released;

    /// <summary>When the ready check of a forming table runs out, by this page's clock, and the line that counts down to it.</summary>
    private DateTime? _readyBy;
    private TextBlock? _countdown;
    private readonly DispatcherTimer _ticker = new() { Interval = TimeSpan.FromMilliseconds(200) };

    public LobbyPanel()
    {
        HorizontalAlignment = HorizontalAlignment.Stretch;
        VerticalAlignment = VerticalAlignment.Stretch;
        Visibility = Visibility.Collapsed;

        // The dark theme's own controls - the name field, the deck list, the switches - rather than the light default.
        RequestedTheme = ElementTheme.Dark;
        _ticker.Tick += (_, _) => Count();
    }

    /// <summary>Shows the panel while the table is being set up, rebuilding it only when the table or an error changes.</summary>
    public bool Show(GameClient client)
    {
        // A seat given up - left, or not ready in time - is gone: said, over everything, with nothing more to do here.
        if (client.Released is { } released)
        {
            Visibility = Visibility.Visible;
            Background = Theme.Brush(Theme.Window);
            if (!ReferenceEquals(_released, released))
            {
                _released = released;
                _ticker.Stop();
                RenderReleased(released);
            }

            return true;
        }

        bool setup = client.Table is { Phase: TablePhase.Setup };
        Visibility = setup ? Visibility.Visible : Visibility.Collapsed;

        // A table forming from a queue is a page of its own, with no empty board showing behind it.
        Background = client.Table is { Forming: true } ? Theme.Brush(Theme.Window) : null;
        if (!setup)
        {
            return false;
        }

        if (ReferenceEquals(_shown, client.Table) && ReferenceEquals(_client, client) && _errorCount == client.Errors.Count)
        {
            return true;
        }

        _shown = client.Table;
        _client = client;
        _errorCount = client.Errors.Count;
        Render(client, client.Table!);
        return true;
    }

    private void Render(GameClient client, TableMessage table)
    {
        Children.Clear();

        var column = new StackPanel { Orientation = Orientation.Vertical, Spacing = 12 };
        Children.Add(new Border
        {
            Background = Theme.Brush(Theme.Window),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(20),
            MinWidth = 480,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Child = column,
        });

        _ticker.Stop();
        if (table.Forming)
        {
            RenderForming(client, table, column);
            return;
        }

        if (table.EventId.Length > 0)
        {
            RenderSeating(client, table, column);
            return;
        }

        column.Children.Add(Text("Set up the table", 24, Theme.Fg, semibold: true));

        foreach (TablePlayerView player in table.Players)
        {
            column.Children.Add(Text(
                $"{player.Name} · seat {player.Seat} · {(player.Ready ? "ready" : "choosing")}{(player.Connected ? "" : " · disconnected")}",
                14,
                Theme.Fg));
        }

        TablePlayerView? me = table.Players.FirstOrDefault(player => player.Seat == client.Seat);
        bool locked = me?.Ready == true;

        var name = new TextBox { Text = me?.Name ?? "", PlaceholderText = "Your name", IsReadOnly = locked, FontSize = 13 };
        column.Children.Add(name);

        var deck = new ComboBox { IsEnabled = !locked, HorizontalAlignment = HorizontalAlignment.Stretch, FontSize = 13 };
        foreach (string available in table.Decks)
        {
            deck.Items.Add(available);
        }

        deck.SelectedIndex = table.Decks.IndexOf(me?.Deck ?? "");
        column.Children.Add(deck);

        DockButton save = Button(column, "Apply name and deck", disabled: locked || table.Decks.Count == 0);
        save.Clicked += () =>
        {
            if (deck.SelectedIndex >= 0)
            {
                client.ConfigurePlayer(name.Text, table.Decks[deck.SelectedIndex]);
            }
        };

        if (client.Seat == table.HostSeat)
        {
            column.Children.Add(Text("Sets for this table", 14, Theme.Fg, semibold: true));

            bool anyReady = table.Players.Any(player => player.Ready);
            List<(string Name, ToggleSwitch Toggle)> sets = [];

            foreach (string set in table.AvailableSets)
            {
                var toggle = new ToggleSwitch { IsOn = table.SelectedSets.Contains(set), IsEnabled = !anyReady, OnContent = null, OffContent = null, MinWidth = 0 };
                var row = new Grid();
                row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
                row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
                row.Children.Add(Text(set, 13, Theme.Fg));
                Grid.SetColumn(toggle, 1);
                row.Children.Add(toggle);
                ((TextBlock)row.Children[0]).VerticalAlignment = VerticalAlignment.Center;

                sets.Add((set, toggle));
                column.Children.Add(row);
            }

            DockButton apply = Button(column, "Apply sets", disabled: anyReady);
            apply.Clicked += () => client.ConfigureTable(sets.Where(set => set.Toggle.IsOn).Select(set => set.Name));
        }

        DockButton ready = Button(column, locked ? "Not ready" : "Ready", disabled: !locked && string.IsNullOrEmpty(me?.Deck));
        ready.Clicked += () => client.SetReady(!locked);

        if (table.Detail.Length > 0)
        {
            column.Children.Add(Text(table.Detail, 13, Theme.FgMuted));
        }

        if (client.Errors.LastOrDefault() is { } error)
        {
            column.Children.Add(Text(error.Detail, 13, Theme.FgMuted));
        }
    }

    /// <summary>
    /// An event's table while its players take their seats: who is here and who is still to come. There is nothing to
    /// choose - the event set everything up - so this only says what it is waiting for.
    /// </summary>
    private static void RenderSeating(GameClient client, TableMessage table, StackPanel column)
    {
        column.Children.Add(Text(table.EventName.Length > 0 ? table.EventName : "Event", 24, Theme.Fg, semibold: true));
        column.Children.Add(Text("The draft starts when everyone has taken their seat.", 14, Theme.FgSoft));
        foreach (TablePlayerView player in table.Players)
        {
            string who = player.Seat == client.Seat ? $"{player.Name} (you)" : player.Name;
            var line = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 10 };
            line.Children.Add(Portrait(player));
            TextBlock text = Text($"{who}  ·  {(player.Connected ? "here" : "not here yet")}", 14, player.Connected ? Theme.Fg : Theme.FgMuted);
            text.VerticalAlignment = VerticalAlignment.Center;
            line.Children.Add(text);
            column.Children.Add(line);
        }

        if (client.Errors.LastOrDefault() is { } error)
        {
            column.Children.Add(Text(error.Detail, 13, Theme.FgMuted));
        }
    }

    /// <summary>
    /// A table forming from a queue: every seat, taken or waiting for a player; Leave Table; and once every seat is taken,
    /// Ready! and how long is left to press it.
    /// </summary>
    private void RenderForming(GameClient client, TableMessage table, StackPanel column)
    {
        column.Children.Add(Text(table.EventName.Length > 0 ? table.EventName : "Draft table", 24, Theme.Fg, semibold: true));

        int empty = table.Players.Count(player => player.Name.Length == 0);
        bool checking = table.ReadyLeftMs > 0;
        foreach (TablePlayerView player in table.Players.OrderBy(player => player.Seat))
        {
            var row = new Grid { ColumnSpacing = 12, Padding = new Thickness(12, 8, 12, 8), Background = Theme.Brush(Theme.Panel), CornerRadius = new CornerRadius(6) };
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(64) });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            TextBlock number = Text($"Seat {player.Seat}", 13, Theme.FgMuted);
            number.VerticalAlignment = VerticalAlignment.Center;
            row.Children.Add(number);

            // A seat still waiting for a player keeps the picture's room, so the names line up.
            bool taken = player.Name.Length > 0;
            FrameworkElement picture = Portrait(player);
            picture.Opacity = taken ? 1 : 0;
            Grid.SetColumn(picture, 1);
            row.Children.Add(picture);

            TextBlock who = Text(!taken ? "Waiting for a player\u2026" : player.Seat == client.Seat ? $"{player.Name} (you)" : player.Name, 15, taken ? Theme.Fg : Theme.FgMuted, semibold: taken);
            who.VerticalAlignment = VerticalAlignment.Center;
            Grid.SetColumn(who, 2);
            row.Children.Add(who);

            if (taken && (checking || !player.Connected))
            {
                // Whether they are ready, once asked; until then, whether they have opened the table yet.
                TextBlock state = checking
                    ? Text(player.Ready ? "Ready" : "Not ready yet", 13, player.Ready ? Theme.Ready : Theme.FgSoft, semibold: player.Ready)
                    : Text("Opening the table…", 13, Theme.FgMuted);
                state.VerticalAlignment = VerticalAlignment.Center;
                Grid.SetColumn(state, 3);
                row.Children.Add(state);
            }

            column.Children.Add(row);
        }

        TablePlayerView? me = table.Players.FirstOrDefault(player => player.Seat == client.Seat);
        if (checking)
        {
            _readyBy = DateTime.UtcNow + TimeSpan.FromMilliseconds(table.ReadyLeftMs);
            column.Children.Add(Text("Every seat is taken. Everyone has to press Ready! to start the draft; anyone who doesn't in time loses their seat.", 14, Theme.FgSoft));
            _countdown = Text(string.Empty, 18, Theme.Accent, semibold: true);
            column.Children.Add(_countdown);
            Count();
            _ticker.Start();

            if (me is { Ready: false })
            {
                var ready = new DockButton("Ready!", primary: true, fontSize: 16).Filling();
                ready.Clicked += () => client.SetReady(true);
                column.Children.Add(ready);
            }
            else
            {
                column.Children.Add(Text("You're ready. Waiting for everyone else.", 14, Theme.Ready, semibold: true));
            }
        }
        else
        {
            _readyBy = null;
            column.Children.Add(Text(
                empty == 0 ? "Every seat is taken. Once everyone has the table open, everyone has 30 seconds to press Ready!"
                : empty == 1 ? "Waiting for 1 more player to join the queue."
                : $"Waiting for {empty} more players to join the queue.",
                14,
                Theme.FgSoft));
        }

        var leave = new DockButton("Leave Table", primary: false).Filling();
        leave.Clicked += client.Leave;
        column.Children.Add(leave);
    }

    /// <summary>Counts the ready check down, a few times a second.</summary>
    private void Count()
    {
        if (_readyBy is not { } by || _countdown is null)
        {
            _ticker.Stop();
            return;
        }

        int left = (int)Math.Ceiling(Math.Max(0, (by - DateTime.UtcNow).TotalSeconds));
        _countdown.Text = left == 1 ? "1 second left" : $"{left} seconds left";
    }

    /// <summary>The seat is gone: why, and where to go from here.</summary>
    private void RenderReleased(SeatReleasedMessage released)
    {
        Children.Clear();
        var column = new StackPanel { Orientation = Orientation.Vertical, Spacing = 12 };
        Children.Add(new Border
        {
            Background = Theme.Brush(Theme.Window),
            CornerRadius = new CornerRadius(6),
            Padding = new Thickness(20),
            MinWidth = 480,
            MaxWidth = 560,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Child = column,
        });

        column.Children.Add(Text("You're no longer at this table", 24, Theme.Fg, semibold: true));
        column.Children.Add(Text(released.Detail, 15, Theme.FgSoft));
        column.Children.Add(Text("You can close this page. To play, press Join in the queue's channel in Discord.", 14, Theme.FgMuted));
    }

    /// <summary>A player's picture beside their name, or the stand-in for someone without one.</summary>
    private static PortraitElement Portrait(TablePlayerView player) => new() { Width = 32, Height = 32, Source = player.Portrait };

    private static TextBlock Text(string text, double size, Windows.UI.Color color, bool semibold = false) => new()
    {
        Text = text,
        FontSize = size,
        FontWeight = semibold ? FontWeights.SemiBold : FontWeights.Normal,
        Foreground = Theme.Brush(color),
        TextWrapping = TextWrapping.Wrap,
    };

    private static DockButton Button(Panel parent, string text, bool disabled)
    {
        DockButton button = new DockButton(text, primary: false, enabled: !disabled).Filling();
        parent.Children.Add(button);
        return button;
    }
}
