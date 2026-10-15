# Aborting a connection whose request has already ended

Found: 2026-09-21. Diagnosed over the three days below, and **fixed on 2026-09-23** — see *The fix* and
*What was measured*. It was titled "a shutdown race" for most of its life; it is not, and the last day of evidence
is what settled that.

`GameServer.Connection.Abort()` calls `socket.Abort()`, and the socket ASP.NET Core hands back forwards that to the
`HttpContext` of the request that accepted it. Once that request has ended the context's feature collection is
disposed and the call throws:

```
System.ObjectDisposedException : IFeatureCollection has been disposed.
Object name: 'Collection'.
   at Microsoft.AspNetCore.Http.Features.FeatureReferences`1.ThrowContextDisposed()
   at Microsoft.AspNetCore.Http.DefaultHttpContext.Abort()
   at Cantrip.Server.Host.GameServer.Connection.Abort()
```

Nothing calls `Abort()` from inside the request that owns the socket. There were four callers, all of them from
some other thread, and all of them able to reach a connection whose request has gone:

- `DisposeAsync`'s abort loop — it cancels first, which is what starts every connection tearing itself down, and
  then aborts the `_connections` snapshot it took before they finished.
- `Connection.Send`, when the bounded 256-frame outbound channel will not take another frame. That runs on the
  pump.
- The idle sweep, which drops connections that have not joined within fifteen seconds. Also the pump.
- `Resume`, which supersedes a membership's previous socket. Also the pump, and the only one a client can ask for
  directly.

`WriteLoopAsync`'s own `finally { socket.Abort(); }` is **not** one of them: it runs inside the request, so the
context is still alive.

## What was racing, and the window it left

The socket is owned by the request that accepted it, and is disposed when the handler returns. In `ServeAsync`
(the part of `HandleAsync` that is about the socket rather than the request):

```csharp
finally
{
    connection.Complete();
    await writer.ConfigureAwait(false);
    _connections.TryRemove(connection, out _);   // leaves the set here...
    connection.Detach();
    Queue(connection, null);
}
// ...and the socket is disposed here, after which Kestrel tears the request down.
```

`ReadLoopAsync` passes the shutdown token to `ReceiveAsync`, so cancelling unblocks it immediately: the handler runs
its `finally`, leaves `_connections`, returns, and the request completes. `ConcurrentDictionary.Keys` is a snapshot,
so `DisposeAsync` can still be holding a connection that has finished. That is the shutdown shape, and
`DisposeAsync` creates the race it then loses.

The live-table shape is the same window seen from the other side. The pump reads `member.Connection`, not
`_connections`, and the pump is one loop behind its queue — so between the handler queueing its disconnect and the
pump dequeuing it, a `Flush` writes to a connection whose channel `Complete()` has already closed. `TryWrite`
returns false on a *completed* channel as readily as on a full one, and the old `Send` could not tell the two
apart, so it aborted.

## The fix

All four callers funnel through `Connection.Abort`, and the frames they were trying to send funnel through
`Connection.Send`. Both now know what state the connection is in.

```csharp
public void Send(ServerMessage message)
{
    if (_completed) return;
    if (!_outbound.Writer.TryWrite(MessagePackSerializer.Serialize(message))) Abort();
}

public void Complete() { _completed = true; _outbound.Writer.TryComplete(); }

/// The request that accepted the socket has finished with it and is about to dispose it.
public void Detach() { _detached = true; Complete(); }

public void Abort()
{
    Complete();
    if (_detached) return;
    try { socket.Abort(); }
    catch (ObjectDisposedException) { }
}
```

`ServeAsync` calls `Detach()` in its `finally`, between leaving `_connections` and returning to the `using` that
disposes the socket. Three parts, each doing something the others do not:

- **`Detach` plus the guard in `Abort`** is the actual fix. After it, no abort from any other thread reaches a
  socket whose request the host knows has ended. This is what the earlier `try/catch`-only candidate could not do:
  swallowing says "that failed and it did not matter", while not calling says "that is not ours to touch".
- **The guard in `Send`** is what the live-table case needed. A closed outbound channel is a connection leaving,
  not a client falling behind, and aborting it buys nothing — the socket is going. The frame is dropped. Nothing
  depends on it arriving: a seat that comes back resumes, and `GameSession.Resynchronize` clears the seat's queue
  and rebuilds `Welcome` + full `State` + the event cursor + any live prompt, rather than replaying what was
  missed. A frame dropped here is exactly a frame dropped to a seat that has already been marked disconnected,
  which the host has always done (`member.Connection?.Send(...)` on a null connection). A *full* channel on a live
  connection still aborts: backpressure is unchanged, and a client that cannot keep up must not hold the table's
  memory.
- **The `catch`** is the floor, for the sliver between reading `_detached` and the call landing, in which the
  request can still end. It is the one-line candidate this document used to recommend, kept as what it is: a
  backstop, not the fix.

`DisposeAsync`'s abort loop was left alone — see *Do not remove the abort loop* below.

## The test

`dotnet/tests/Cantrip.Server.Tests/ConnectionAbortTests.cs`. It does not race for the state, it arranges it, and it needs
no load, no seeds and no parallel runs.

`ScriptedSocket` is a `WebSocket` whose `Abort()` throws exactly the `ObjectDisposedException` a real one throws
once its request is torn down, and whose receive and send sides the test drives frame by frame. `ServeAsync` — the
socket half of `HandleAsync`, split out for this — serves that socket through the host's own connection
bookkeeping, so the connection is registered, pumped and swept like any other.

Four tests then reach each caller in turn: shutdown with such a connection still in `_connections`; the idle sweep,
driven by a fake `TimeProvider` rather than by waiting; an outbound channel filled to its 256 frames by stalling
the socket's sends and then refusing 400 client messages; and a resume superseding a seated socket. Five more pin
`Connection`'s own contract — detached, completing, torn-down-but-not-detached, and a live connection that overruns
its channel and must still be dropped. Two more answer the pooled-context question below.

The reproduction in *Reproducing it* is kept because it is the only thing that exercises the real timing; it is not
the regression test.

## Reproducing it (the old way — needs load, and is not the regression test)

```powershell
cd D:\depot
$jobs = 1..4 | ForEach-Object {
  Start-Job -ScriptBlock {
    Set-Location D:\depot
    $o = dotnet test dotnet\tests\Cantrip.Server.Tests\Cantrip.Server.Tests.csproj -c Release -v q --nologo --no-build 2>&1
    ($o | Select-String -Pattern "\[FAIL\]|Passed!|Failed!") -join "`n"
  }
}
$jobs | Wait-Job | Receive-Job; $jobs | Remove-Job
```

A single run of the project passes reliably, which is why this was probably dismissed as noise before. The test in
the failure message is whichever one was unlucky; across the runs recorded here it has been
`MembershipTransportTests.Normal_table_requires_catalog_selection_and_readiness_and_freezes_setup_when_ready`,
`MembershipTransportTests.Lobby_timeout_releases_seat_revokes_proof_and_does_not_allow_early_reclaim`,
`MembershipTransportTests.Active_departure_or_grace_expiry_abandons_without_fabricating_a_game_outcome`,
`ExtensionCompositionTransportTests.Independent_keyword_replacement_and_recipe_compose_without_engine_or_client_edits`,
`WebSocketTransportTests.A_disconnect_leaves_the_game_intact_for_everyone_else` and
`WebSocketTransportTests.Each_client_only_receives_its_own_hand_over_the_wire`. **The test in the message is not
the bug.** Any test that disposes a server with a live connection was exposed, which is nearly all of them.

## What was measured

All Release, on the machine this was found on.

| | |
| --- | --- |
| `ConnectionAbortTests`, fix reverted (seam kept) | **8 of 11 fail, 10 runs out of 10** |
| `ConnectionAbortTests`, fixed | **11 pass, 10 runs out of 10**, 0.33 s a run |
| Each part of the fix removed on its own | catch only: 5 fail · `Detach` guard only: 1 fails · `Send` guard only: 1 fails |
| Parallel repro, fix reverted | **1 failure in 8 runs** (`Lobby_timeout_releases_seat_...`), against the 2 in 8 recorded on 09-21 |
| Parallel repro, fixed | **0 failures in 16 runs** (four batches of four) |
| `Cantrip.Server.Tests` whole project, fixed | 195 pass |
| `Four_sockets_play_concurrently_and_block_simultaneously` | 6 runs, 6 pass |
| Core, Cards, Datasets, Engine, Projection, Client projects | all green (65, 578, 420, 2182, 88, 223) |

## The open question, answered: the context is not recycled onto another request

`DefaultHttpContext` is pooled, and the guard we were tripping exists because the context outlives the features of
the request it served. The worry was that if the context had been handed to a *different* request by the time
`Abort()` landed, the guard would not fire at all and the abort would land on another player's live socket — a
random mid-game disconnect rather than a flake. It was an untested hypothesis; it has now been tested, and it does
not hold here.

`A_websocket_request_context_is_never_handed_to_a_second_request` runs 24 sequential WebSocket requests against a
Kestrel app configured as the host's is, keeps every `HttpContext` it served, and asserts that all 24 are distinct
by reference and that every one of them throws `ObjectDisposedException` from a late `Abort()`. Both hold.

`A_keep_alive_connection_does_reuse_its_request_context` is the control, and the reason to believe the first. Six
kept-alive HTTP requests on one connection **do** share `HttpContext` instances — so the probe can see recycling,
and its silence on the WebSocket path means something. The mechanism is that Kestrel reuses a context across the
requests of one connection, and a WebSocket request is the last request its connection will ever carry. The
recycling is real; it cannot reach a socket the host still holds.

That makes the `catch` safe as a backstop rather than a gamble. It does not make it the fix, which is why `Detach`
is there as well: the argument above is about this framework version and this configuration, and "not ours to
touch" does not depend on either.

## Do not remove the abort loop from `DisposeAsync`

Measured on 09-21 and rejected. The reasoning looked sound — cancellation alone unblocks `ReceiveAsync` and the
write loop, `WriteLoopAsync` already aborts the socket from inside the request, and `_app.StopAsync()` drains what
is left. It is not. With the loop removed,
`WebSocketTransportTests.Four_sockets_play_concurrently_and_block_simultaneously` failed 2 of 4 runs at about 20
seconds, which looks like a shutdown that stops draining rather than an assertion. A later batch of 5 passed, so
the measurement is ambiguous and the root cause was never chased. **Whatever the abort loop is doing for that
test, it is load-bearing.** The fix above does not touch it and does not make it redundant; the loop still aborts
every live connection, it simply no longer reaches ones that have detached.

The structurally correct version — have `ServeAsync` register its own task and have `DisposeAsync` cancel, await
the in-flight handlers with a timeout, then `StopAsync` — remains untried, and still needs that point answered
first.

## How the diagnosis got here

**2026-09-21.** Found as an `ObjectDisposedException` thrown out of `DisposeAsync`, failing whichever transport
test happened to be holding its server in an `await using`. 2 failures in 8 parallel runs; a single run of the
project passed 3 for 3.

**2026-09-22.** A second shape appeared: `Assert.False(server.Finished.IsFaulted)` failing as "expected False,
actual True", with the exception and the whole table thrown away, because the pump catches everything and hands
the exception to a `TaskCompletionSource`. Observed roughly once in a dozen `tools\test.cmd` runs and not
reproducible alone. `GameServer` was given `_step` and `_recent` — the pump step it is on and the last dozen things
that happened to the table — written to the log and to `GameServer.FaultDetail` before the task is completed, and
`ServerAssert.DidNotFault` was added to report both. Whether the two shapes were one bug was left open.

**2026-09-23.** They were. The logging caught it, and the faulted run printed the path:

```
Faulted while flushing to the seats. Phase Playing, 1 connection(s), seat 1 connected, seat 2 connected;
turn 2 Upkeep, in progress; 1 queued; shutdown False. Recently: the table started with 2 seat(s).

System.ObjectDisposedException: IFeatureCollection has been disposed.
   at Microsoft.AspNetCore.Http.DefaultHttpContext.Abort()
   at GameServer.Connection.Abort()
   at GameServer.Connection.Send(ServerMessage)
   at GameServer.Flush()
   at GameServer.PumpAsync()
```

`shutdown False`, phase `Playing`, turn 2, a message queued: nothing to do with `DisposeAsync`. And
`1 connection(s)` while both seats read as connected — the window itself, a connection that has left `_connections`
but whose socket is not yet disposed, with the pump flushing into the gap. That is what retitled this document and
what made `Send` part of the fix rather than a footnote to it.

## Not caused by the playtest work

Checked on 09-22, because the two failures that surfaced this showed up during that change. `tools\test.cmd` passed
2 for 2 on a clean stash and 3 for 3 with the changes applied, the parallel repro failed on unmodified source, and
nothing in that change touches `dotnet/src/Cantrip.Server.Host`. The protocol did move 72 → 73 for `ObjectView.OtherFace`,
which makes every view marginally larger on the wire and so shifts timing a little; that may change how often the
race is lost, but it is not what made it a race.
