using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Threading.Channels;
using Cantrip.Protocol;
using MessagePack;

namespace Cantrip.Client;

/// <summary>
/// Talks to a remote game over a WebSocket.
/// <para>
/// All the threading lives behind the polling interface: a reader task fills a queue that
/// <see cref="Receive"/> drains, and a writer task empties a channel that <see cref="Send"/> fills. The client
/// above this never sees a task, a lock, or a callback, which is what lets the web client drain it on a timer
/// and stay entirely single-threaded.
/// </para>
/// <para>
/// The transport reconnects nothing and retries nothing. A dropped connection surfaces as
/// <see cref="IsConnected"/> going false with <see cref="LastError"/> set, and resynchronising is the session's
/// business: a new transport sends Resume with its private membership proof and receives an authorized snapshot and
/// still-outstanding prompt. An already accepted reply is refused as stale; unresolved choices keep their request ID.
/// </para>
/// </summary>
public sealed class WebSocketTransport : IGameTransport
{
    private readonly ClientWebSocket _socket = new();
    private readonly ConcurrentQueue<ServerMessage> _inbound = new();
    private readonly Channel<byte[]> _outbound = Channel.CreateUnbounded<byte[]>();
    private readonly CancellationTokenSource _shutdown = new();

    private volatile bool _connected;
    private volatile string? _lastError;

    private WebSocketTransport()
    {
    }

    public bool IsConnected => _connected;

    /// <summary>Why the connection ended, or null while it is healthy.</summary>
    public string? LastError => _lastError;

    /// <summary>
    /// Connects to a server and starts pumping.
    /// </summary>
    public static async Task<WebSocketTransport> ConnectAsync(Uri address, CancellationToken cancellation = default)
    {
        ArgumentNullException.ThrowIfNull(address);

        var transport = new WebSocketTransport();
        try
        {
            await transport._socket.ConnectAsync(address, cancellation).ConfigureAwait(false);
        }
        catch
        {
            transport.Dispose();
            throw;
        }
        transport._connected = true;

        // Deliberately not awaited: these run for the life of the connection.
        _ = Task.Run(transport.ReadLoopAsync, CancellationToken.None);
        _ = Task.Run(transport.WriteLoopAsync, CancellationToken.None);

        return transport;
    }

    public void Send(ClientMessage message)
    {
        ArgumentNullException.ThrowIfNull(message);

        if (!_connected)
        {
            return;
        }

        _outbound.Writer.TryWrite(MessagePackSerializer.Serialize(message));
    }

    public IReadOnlyList<ServerMessage> Receive()
    {
        if (_inbound.IsEmpty)
        {
            return [];
        }

        List<ServerMessage> messages = [];
        while (_inbound.TryDequeue(out ServerMessage? message))
        {
            messages.Add(message);
        }

        return messages;
    }

    private async Task ReadLoopAsync()
    {
        var buffer = new byte[16 * 1024];

        try
        {
            // Read until the close is read, not while the socket says it is open: in the browser it says closed as soon as
            // the close arrives, with the last messages before it - a move to another table, why a seat was let go -
            // still waiting to be read.
            while (!_shutdown.IsCancellationRequested && _socket.State is not (WebSocketState.Aborted or WebSocketState.None))
            {
                using var frame = new MemoryStream();
                WebSocketReceiveResult result;

                // A message can arrive in several frames, so it is reassembled before being deserialised.
                do
                {
                    result = await _socket
                        .ReceiveAsync(new ArraySegment<byte>(buffer), _shutdown.Token)
                        .ConfigureAwait(false);

                    if (result.MessageType == WebSocketMessageType.Close)
                    {
                        Fault("the server closed the connection.");
                        return;
                    }

                    if (result.MessageType != WebSocketMessageType.Binary || frame.Length + result.Count > 4 * 1024 * 1024)
                    {
                        Fault("The server sent an invalid or oversized frame.");
                        return;
                    }

                    frame.Write(buffer, 0, result.Count);
                }
                while (!result.EndOfMessage);

                if (frame.Length == 0)
                {
                    continue;
                }

                _inbound.Enqueue(MessagePackSerializer.Deserialize<ServerMessage>(frame.ToArray()));
            }
        }
        catch (OperationCanceledException)
        {
            // Expected on disposal.
        }
        catch (Exception error)
        {
            Fault(error.Message);
        }
        finally
        {
            _connected = false;
            _outbound.Writer.TryComplete();
        }
    }

    private async Task WriteLoopAsync()
    {
        try
        {
            await foreach (byte[] payload in _outbound.Reader.ReadAllAsync(_shutdown.Token).ConfigureAwait(false))
            {
                if (_socket.State != WebSocketState.Open)
                {
                    return;
                }

                await _socket
                    .SendAsync(payload, WebSocketMessageType.Binary, endOfMessage: true, _shutdown.Token)
                    .ConfigureAwait(false);
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (Exception error)
        {
            Fault(error.Message);
        }
    }

    private void Fault(string reason)
    {
        _lastError ??= reason;
        _connected = false;
        _outbound.Writer.TryComplete();
        _socket.Abort();
    }

    public void Dispose()
    {
        _connected = false;
        _outbound.Writer.TryComplete();

        try
        {
            _shutdown.Cancel();
        }
        catch (ObjectDisposedException)
        {
        }

        _socket.Dispose();
        _shutdown.Dispose();
    }
}
