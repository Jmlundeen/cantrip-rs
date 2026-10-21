using System.Net.WebSockets;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Dropping a table's connection from outside its request (KI-85). The socket forwards <c>Abort()</c> to the request that
/// accepted it, which may end on its own thread at any moment; an abort landing just after it threw a
/// NullReferenceException out of the table's disposal. So the pump and shutdown never touch the socket: they cancel, and
/// the request's own read and write loops end and abort it. A fake socket, so nothing here waits on the clock.
/// </summary>
public sealed class ConnectionAbortTests
{
    [Fact]
    public void Dropping_a_connection_from_outside_never_touches_its_socket()
    {
        var socket = new FakeSocket();
        var connection = new GameServer.Connection(socket, DateTimeOffset.UnixEpoch);

        connection.Abort();

        Assert.True(connection.Aborted);
        Assert.Equal(0, socket.Aborts);
    }

    [Fact]
    public async Task A_dropped_connections_own_loops_end_and_abort_its_socket()
    {
        var socket = new FakeSocket();
        var connection = new GameServer.Connection(socket, DateTimeOffset.UnixEpoch);
        Task writer = connection.WriteLoopAsync(CancellationToken.None);
        Task reader = connection.ReadLoopAsync(_ => { }, CancellationToken.None);

        connection.Abort();
        await reader;
        await writer;

        Assert.True(socket.Received, "the read loop was waiting on the socket when the connection was dropped");
        Assert.Equal(1, socket.Aborts);
    }

    /// <summary>
    /// A socket whose receive waits until it is cancelled, as a quiet client's does, and which counts its aborts. It leaves
    /// out one thing the real socket does: a cancelled receive or send aborts the real socket from its token's registration,
    /// on the cancelling thread, while the operation is still pending in the request (which is safe, GameServer.Connection
    /// says why). Here a cancelled receive only ends, so the aborts counted are the calls the connection makes itself.
    /// </summary>
    private sealed class FakeSocket : WebSocket
    {
        private WebSocketState _state = WebSocketState.Open;

        public int Aborts { get; private set; }

        public bool Received { get; private set; }

        public override WebSocketCloseStatus? CloseStatus => null;

        public override string? CloseStatusDescription => null;

        public override WebSocketState State => _state;

        public override string? SubProtocol => null;

        public override void Abort()
        {
            Aborts++;
            _state = WebSocketState.Aborted;
        }

        public override Task CloseAsync(WebSocketCloseStatus closeStatus, string? statusDescription, CancellationToken cancellationToken) =>
            Task.CompletedTask;

        public override Task CloseOutputAsync(WebSocketCloseStatus closeStatus, string? statusDescription, CancellationToken cancellationToken) =>
            Task.CompletedTask;

        public override void Dispose()
        {
        }

        public override async Task<WebSocketReceiveResult> ReceiveAsync(ArraySegment<byte> buffer, CancellationToken cancellationToken)
        {
            Received = true;
            await Task.Delay(Timeout.Infinite, cancellationToken);
            throw new InvalidOperationException("unreachable");
        }

        public override Task SendAsync(ArraySegment<byte> buffer, WebSocketMessageType messageType, bool endOfMessage, CancellationToken cancellationToken) =>
            Task.CompletedTask;
    }
}
