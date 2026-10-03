# Sessions & connection lifecycle

Each WebSocket connection lives inside a **session**, and all MCP clients share one more. A session owns a dedicated background thread that runs its requests serially against its in-memory state (open Packs, dependency cache, settings).

Sessions are isolated: open packs in one session aren't visible in another. This is what makes "many UI clients talking to one server" safe.

## Lifecycle

1. **Connect.** A client opens `/ws` without a session ID. The server allocates a new `SessionId`, spins up a background thread, and immediately sends a `session.connected` notification so the client can stash the ID for later reconnection.

   ```json
   { "jsonrpc": "2.0", "method": "session.connected", "params": { "session_id": 12345 } }
   ```

2. **Reconnect.** A client opens `/ws?session_id=12345`. If the session still exists and isn't shutting down, the new socket adopts it with all in-memory state preserved (open Packs, loaded dependencies, settings).

3. **Disconnect.** When the WebSocket drops without a `session.disconnect` request, the session enters a **5-minute** grace period (`DEFAULT_SESSION_TIMEOUT_SECS = 300`). Reconnecting cancels the timeout; otherwise the session and its background thread are torn down.

4. **Graceful disconnect.** Call `session.disconnect` before closing your socket and the session is removed immediately, telemetry is flushed, and the background thread exits.

   ```json
   { "jsonrpc": "2.0", "id": 99, "method": "session.disconnect" }
   ```

5. **Empty manager → process exit.** When the last WebSocket session goes away and no other session is left, the `rpfm_server` process exits, so no orphaned server lingers in the background.

## The MCP session

MCP clients have no disconnect signal the server can rely on, and newer MCP versions have no sessions at all, so all MCP clients share one session. It's created by the first tool call, and removed after 5 minutes without tool calls, unless it has open packs or unfinished jobs. The next tool call creates a new one. Removing it never stops the server, so a server started by hand for MCP clients keeps running.

## Reconnection example

<!-- langtabs-start -->
```typescript
// First connect: store the session ID
const ws = new WebSocket("ws://127.0.0.1:45127/ws");
let sessionId: number | null = null;

ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  if (msg.method === "session.connected") {
    sessionId = msg.params.session_id;
    console.log("session", sessionId);
  }
};

// Later, after a network blip, reconnect with the stored ID:
const ws2 = new WebSocket(`ws://127.0.0.1:45127/ws?session_id=${sessionId}`);
```
```csharp
using var ws = new ClientWebSocket();
await ws.ConnectAsync(new Uri("ws://127.0.0.1:45127/ws"), CancellationToken.None);

int? sessionId = null;
// ... receive the session.connected notification and store sessionId ...

// Later, reconnect:
using var ws2 = new ClientWebSocket();
await ws2.ConnectAsync(
    new Uri($"ws://127.0.0.1:45127/ws?session_id={sessionId}"),
    CancellationToken.None);
```
<!-- langtabs-end -->

## Listing active sessions

The `/sessions` REST endpoint returns every live session as JSON:

```bash
curl http://127.0.0.1:45127/sessions
```

```json
[
  {
    "session_id": 12345,
    "connection_count": 0,
    "timeout_remaining_secs": 180,
    "is_shutting_down": false,
    "pack_names": ["my_mod.pack"]
  },
  {
    "session_id": 12346,
    "connection_count": 1,
    "timeout_remaining_secs": null,
    "is_shutting_down": false,
    "pack_names": []
  }
]
```

Fields:

| Field                    | Description                                                                      |
|--------------------------|----------------------------------------------------------------------------------|
| `session_id`             | Unique identifier.                                                               |
| `connection_count`       | Number of active WebSocket connections to this session.                          |
| `timeout_remaining_secs` | Seconds until the session is cleaned up. Only set while disconnected.            |
| `is_shutting_down`       | `true` when the session has been marked for shutdown.                            |
| `pack_names`             | Names of the Packs currently open in this session.                               |

The UI uses this endpoint to populate **Pack → Select Session…**, which lets a user reattach to a server session after the UI was killed or disconnected.

## Per-session state

Each session has its own:

- Open Packs (with their full in-memory contents and metadata).
- Dependency cache view (game files, parent files, AK files).
- Active game selection.
- Settings: sessions start with the ones in RPFM's `settings.json`, and clients can replace them with `session.configure`.
- Jobs and their results.

When you call something like `session.set_game`, you're changing it for *your* session, not globally.
