# rpfm_server

`rpfm_server` is the backend server for RPFM (Rusted PackFile Manager). It provides a WebSocket and MCP (Model Context Protocol) interface for the frontend to interact with the filesystem and perform PackFile operations.

## Compilation

To compile the server, you can use the following commands:

**For a debug build:**
```bash
cargo build -p rpfm_server
```

**For a release build:**
```bash
cargo build --release --bin rpfm_server
```

## Usage

The server is usually spawned automatically by the `rpfm_ui` frontend. However, you can also run it manually.

When started, the server will listen on `127.0.0.1:45127` by default. It exposes three endpoints:

-   `/ws`: A WebSocket endpoint for general commands and responses. The frontend connects to this to send commands and receive results. It supports session management, allowing a client to disconnect and reconnect to the same session.
-   `/sessions`: A REST endpoint that returns a JSON array of all active sessions. Used by the UI to display available sessions for reconnection.
-   `/mcp`: An HTTP endpoint for the Model Context Protocol, used for more specialized, streamable operations.

To run the server, execute the compiled binary:

```bash
./target/release/rpfm_server
```

The server will log its output to the standard error stream.

## Session Management

The server uses a session-based architecture where each client connection is associated with a session. Sessions maintain state (open PackFile, settings, etc.) and can persist across disconnections.

### Session Lifecycle

1. **New Connection**: When a client connects to `/ws` without a `session_id` parameter, a new session is created with a unique ID.
2. **Session Connected Message**: Immediately after connection, the server sends a `SessionConnected` response containing the session ID:
   ```json
   { "id": 0, "data": { "SessionConnected": 12345 } }
   ```
   The client should store this ID for potential reconnection.
3. **Disconnection**: When a client disconnects unexpectedly, the session enters a timeout period (default: 5 minutes) during which it can be reconnected.
4. **Graceful Disconnect**: If a client sends the `ClientDisconnecting` command before closing the connection, the session is removed immediately.
5. **Timeout Cleanup**: Sessions that are not reconnected within the timeout period are automatically cleaned up.
6. **Server Shutdown**: When all sessions are removed (either by graceful disconnect or timeout), the server shuts down automatically.

### Reconnecting to a Session

To reconnect to an existing session, append the `session_id` query parameter to the WebSocket URL:

```
ws://127.0.0.1:45127/ws?session_id=12345
```

If the session exists and is still valid, the client will be reconnected to it with all state preserved (open PackFile, loaded dependencies, etc.).

### Listing Available Sessions

The `/sessions` REST endpoint returns information about all active sessions:

```bash
curl http://127.0.0.1:45127/sessions
```

Response:
```json
[
  {
    "session_id": 12345,
    "connection_count": 0,
    "timeout_remaining_secs": 180,
    "is_shutting_down": false,
    "pack_name": "my_mod.pack"
  },
  {
    "session_id": 12346,
    "connection_count": 1,
    "timeout_remaining_secs": null,
    "is_shutting_down": false,
    "pack_name": null
  }
]
```

Fields:
- `session_id`: Unique identifier for the session.
- `connection_count`: Number of active WebSocket connections to this session.
- `timeout_remaining_secs`: Seconds until the session is cleaned up (only present if the session has no active connections).
- `is_shutting_down`: Whether the session has been marked for shutdown.
- `pack_name`: Name of the PackFile currently open in this session, or `null` if no pack is open.
