using Cantrip.Client;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// Getting the seat back after the connection drops mid-game: five tries, two seconds apart, each resuming the seat with the proof the server granted, and a line over the table saying so.
/// </summary>
public sealed partial class MainPage
{
    private const int ReconnectTries = 5;
    private static readonly TimeSpan ReconnectPause = TimeSpan.FromSeconds(2);

    private Border _toast = null!;
    private TextBlock _connection = null!;
    private bool _reconnecting;

    /// <summary>Set from a live reconnect's Resume until the server answers it with the seat or a refusal.</summary>
    private bool _restoring;

    /// <summary>When the seat was last got back after a drop, so a connection lost again at once is not chased.</summary>
    private DateTime? _restoredAt;

    /// <summary>
    /// A drop this soon after getting the seat back is not tried again. The likeliest cause is another tab of this one,
    /// duplicated with its session and so its seat: each resume takes the seat from the other, and two tabs that both
    /// reconnected would take it back and forth for ever.
    /// </summary>
    private static readonly TimeSpan SettledAfter = TimeSpan.FromSeconds(10);

    /// <summary>
    /// A line about the connection, floated over the top of the board while there is anything to say, and absent
    /// otherwise.
    /// </summary>
    private void BuildToast()
    {
        _connection = new TextBlock { FontSize = 12, Foreground = Theme.Brush(Theme.Fg) };
        _toast = new Border
        {
            Background = Theme.Brush(Theme.WithAlpha(Theme.Panel, 0.96)),
            BorderBrush = Theme.Brush(Theme.WithAlpha(Theme.Danger, 0.5)),
            BorderThickness = new Thickness(1),
            CornerRadius = new CornerRadius(8),
            Padding = new Thickness(14, 8, 14, 8),
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Top,
            Visibility = Visibility.Collapsed,
            IsHitTestVisible = false,
            Child = _connection,
        };

        Grid.SetRowSpan(_toast, 3);
        _table.Children.Add(_toast);
    }

    private void Say(string status)
    {
        _connection.Text = status;
        _toast.Visibility = status.Length > 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    /// <summary>Centred over the board, right of the rail and whatever dock is open beside it.</summary>
    private void PlaceToast(float inset) => _toast.Margin = new Thickness(inset, 12, 0, 0);

    /// <summary>
    /// Starts getting the seat back once a live connection is lost before the game is over. Without a seat granted there
    /// is nothing to resume, and the dock says the connection is gone.
    /// </summary>
    private void FollowConnection(GameClient client)
    {
        if (_reconnecting || client.IsConnected || client.IsOver || _remembered is not { } seat)
        {
            return;
        }

        // Whatever arrived before the socket closed is taken in first: a seat sent to another table is told so and then
        // let go at once, and the word of where it went can land after this frame's pump and before the close is seen.
        if (client.Pump() > 0)
        {
            _needsRedraw = true;
            if (client.Moved is not null || client.Released is not null || client.IsOver)
            {
                KeepSeat(client);
                return;
            }
        }

        _reconnecting = true;
        client.Disconnect();

        // The dock stops offering what can no longer be sent, and says the connection is gone.
        _needsRedraw = true;

        if (_restoredAt is { } restored && DateTime.UtcNow - restored < SettledAfter)
        {
            Say("The connection was lost again as soon as it came back. If this seat is open in another tab, play it there, or reload here to take it back.");
            return;
        }

        _ = ReconnectAsync(seat);
    }

    /// <summary>
    /// Follows a live reconnect's Resume to its answer (Session.Poll): the seat back, which ends the reconnect; or a
    /// refusal - the seat's grace period over, the table closed - which is said and not tried again. Unlike a reload's
    /// refused seat, this one does not ask for a new seat: the player was in a game, and a fresh lobby would be a
    /// different game.
    /// </summary>
    private void FollowRestore(GameClient client)
    {
        if (!_restoring)
        {
            return;
        }

        if (client.Membership is not null || client.View is not null)
        {
            _restoring = false;
            _reconnecting = false;
            _restoredAt = DateTime.UtcNow;
            Say(string.Empty);
        }
        else if (client.Errors.LastOrDefault() is { Code: ErrorCode.MembershipRefused or ErrorCode.MembershipRequired or ErrorCode.TableClosed or ErrorCode.ProtocolMismatch } refusal)
        {
            _restoring = false;
            _resuming = null;
            Say(refusal.Detail.Length > 0 ? refusal.Detail : "Could not reconnect to your seat.");
        }
        else if (!client.IsConnected)
        {
            _restoring = false;
            Say("Could not reconnect to your seat.");
        }
    }

    private async Task ReconnectAsync(MembershipMessage seat)
    {
        Say("Connection lost. Reconnecting to your seat...");

        for (int attempt = 1; attempt <= ReconnectTries; attempt++)
        {
            try
            {
                var transport = await WebSocketTransport.ConnectAsync(ClientConfig.Server);
                var client = new GameClient(transport);
                client.Resume(seat);
                TakeOver(client);
                _restoring = true;
                Say("Connected. Restoring your seat...");
                return;
            }
            catch (Exception)
            {
                if (attempt < ReconnectTries)
                {
                    Say($"Reconnecting to your seat (attempt {attempt + 1} of {ReconnectTries})...");
                    await Task.Delay(ReconnectPause);
                }
            }
        }

        // Left set, so a lost connection is not tried again: the dock says it is gone and the toast why.
        Say("Could not reconnect to your seat.");
    }

    /// <summary>
    /// Puts a new conversation with the server in place of the lost one. The journal, the log and what the board last
    /// showed carry over; what was half chosen for a prompt of the old one does not.
    /// </summary>
    private void TakeOver(GameClient client)
    {
        _gameClient = client;
        _landingTriggers = null;
        _selection.Clear();
        _peekedForTargets = false;
        _lastPromptId = -1;
        _needsRedraw = true;
    }
}
