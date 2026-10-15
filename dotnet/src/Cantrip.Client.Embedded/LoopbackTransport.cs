using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server;

namespace Cantrip.Client.Embedded;

/// <summary>
/// A transport that hands messages straight to a <see cref="GameSession"/> in the same process.
/// <para>
/// This is how host-and-play works, and the point is that it is the <em>only</em> difference from joining a
/// remote server: the client code above this interface is identical either way. A separate single-player code
/// path would be a second implementation of the rules interaction to keep in step, and the bugs in it would
/// only ever appear in one of the two modes.
/// </para>
/// <para>
/// No serialisation happens here. That is a deliberate trade: the loopback path is fast and allocation-light,
/// but it also means it does not exercise the MessagePack contracts, so the wire format is covered by its own
/// round-trip tests rather than incidentally by host-and-play.
/// </para>
/// </summary>
public sealed class LoopbackTransport : IGameTransport
{
    private readonly GameSession _session;
    private readonly PlayerId _seat;
    private bool _disposed;

    public LoopbackTransport(GameSession session, PlayerId seat)
    {
        ArgumentNullException.ThrowIfNull(session);

        _session = session;
        _seat = seat;
    }

    public bool IsConnected => !_disposed;

    public void Send(ClientMessage message)
    {
        ArgumentNullException.ThrowIfNull(message);
        ObjectDisposedException.ThrowIf(_disposed, this);

        _session.Submit(_seat, message);
    }

    public IReadOnlyList<ServerMessage> Receive() =>
        _disposed ? [] : _session.Drain(_seat);

    public void Dispose() => _disposed = true;
}
