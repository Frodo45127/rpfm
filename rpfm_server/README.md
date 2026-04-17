# rpfm_server

The backend that does the actual PackFile, schema and filesystem work for ***Rusted PackFile Manager***.

`rpfm_server` is a long-running local process that exposes RPFM's capabilities over WebSocket and over the [Model Context Protocol][mcp]. The Qt6 frontend (`rpfm_ui`) talks to it over the WebSocket; AI tools (or other MCP clients) talk to it over `/mcp`. Multiple clients can be connected at the same time, each in its own session with its own set of open packs.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets developers working on the server itself.

[manual]: https://frodo45127.github.io/rpfm/
[mcp]: https://modelcontextprotocol.io/

## Architecture

```
┌──────────────────────────┐    WebSocket    ┌──────────────────────────┐
│        rpfm_ui           │ ◀────────────▶  │      rpfm_server         │
│  (Qt6 desktop app)       │                 │      (this crate)        │
└──────────────────────────┘                 └─────────────┬────────────┘
                                                           │
┌──────────────────────────┐    HTTP+SSE                   │
│  MCP clients (AI tools)  │ ◀──────────────▶  /mcp ◀──────┘
└──────────────────────────┘
```

- Built on `axum` (HTTP + WebSocket) and `tokio`. Listens on `127.0.0.1:45127` by default.
- Each client connection is wrapped in a **session**. A session owns a dedicated background thread that processes commands serially against its in-memory state (open packs, dependency cache, schema, settings).
- Heavy work (file I/O, schema decode, diagnostics, search, optimizer) is delegated to `rpfm_lib` and `rpfm_extensions`.
- Telemetry, panic capture and logging are wired through `rpfm_telemetry`.

## Module layout

Top-level modules in `src/`:

- `main.rs` — Entry point. Sets up telemetry, builds the `SessionManager`, mounts the `axum` router, binds to the listening address.
- `session.rs` — `SessionManager`, `Session`, session lifecycle (creation, reconnection, timeout cleanup). `DEFAULT_SESSION_TIMEOUT_SECS = 300`.
- `server_websocket.rs` — `/ws` upgrade handler. Multiplexes IPC `Message<Command>` / `Message<Response>` traffic over the WebSocket; sends `SessionConnected` immediately after the handshake; flushes telemetry on graceful disconnect (`Server Action Telemetry`).
- `server_mcp.rs` — `/mcp` endpoint. Implements the MCP server (tools, prompts, resources) on top of `rmcp`, and forwards work into the session's background thread.
- `background_thread.rs` — Central dispatcher. The `background_loop` async function pulls `(sender, Command)` pairs off a channel and runs the matching logic.
- `comms.rs` — `CentralCommand<T>`, the generic mpsc-based request/response abstraction used to talk to the background thread.
- `settings.rs` — JSON-backed settings store with batch-write optimisation.
- `updater.rs` — Self-update checks against GitHub releases.

## Endpoints

All on `127.0.0.1:45127`:

| Endpoint    | Method | Purpose                                                                                       |
|-------------|--------|-----------------------------------------------------------------------------------------------|
| `/ws`       | GET    | WebSocket. Carries the `rpfm_ipc` command/response protocol. Accepts `?session_id=…` for reconnection. |
| `/sessions` | GET    | JSON `Vec<SessionInfo>` of every live session. Used by the session picker in the UI.          |
| `/mcp`      | *      | MCP `StreamableHttpService`. Each client gets its own dedicated session and `McpServer`.      |

### `/sessions` response

`Vec<SessionInfo>` (defined in `rpfm_ipc::helpers`) — each entry exposes:

- `session_id` — Unique identifier.
- `connection_count` — Number of active WebSocket connections.
- `timeout_remaining_secs` — Seconds until cleanup (only set while disconnected).
- `is_shutting_down` — Marked-for-shutdown flag.
- `pack_names` — Names of every pack open in that session.

## Sessions

Lifecycle:

1. **Connect.** A client opens `/ws` without `session_id`. The server allocates a new `SessionId`, spawns a background thread, and immediately sends `Response::SessionConnected(id)` so the client can stash the ID for later reconnection.
2. **Reconnect.** A client opens `/ws?session_id=<id>`. If the session still exists and isn't shutting down, the new socket adopts it with all in-memory state preserved (open packs, loaded dependencies, settings cache).
3. **Disconnect.** When the WebSocket drops without a `Command::ClientDisconnecting`, the session enters a 5-minute grace period. Reconnecting cancels the timeout; otherwise the session and its background thread are torn down.
4. **Graceful disconnect.** `Command::ClientDisconnecting` removes the session immediately and flushes telemetry.
5. **Empty manager → process exit.** When the last session goes away the process exits, so no orphaned server lingers in the background.

```bash
# Connect:
ws://127.0.0.1:45127/ws

# Reconnect:
ws://127.0.0.1:45127/ws?session_id=12345

# List sessions:
curl http://127.0.0.1:45127/sessions
```

## Per-pack state

State that used to live on the UI side now lives per-pack on the server. The most visible example is `OperationalMode` (Normal vs. MyMod-bound): `Command::SetPackOperationalMode(pack_key, mode)` and `Command::GetPackOperationalMode(pack_key)` set and read it, and the matching MCP tools mirror the same shape.

This means a session with N open packs holds N independent operational modes, and the UI no longer has to keep that state in sync with which pack is currently focused.

## MCP support

`/mcp` exposes RPFM as an MCP server, so AI tools and other MCP clients can drive it the same way the UI does. The implementation lives entirely in `server_mcp.rs` and routes everything through the same per-session background thread the WebSocket uses, so MCP and UI clients see consistent state.

The server advertises a large surface — over 150 tools covering pack lifecycle, file operations, tables, search, diagnostics, schema, animations, media, operational mode, and so on, plus a handful of resources (game lists, enum dumps, examples, reference docs) and prompts (common workflows like "open and inspect a pack", "edit a DB table", "manage dependencies"). Look in `server_mcp.rs` for the authoritative list.

## Telemetry

The server uses `rpfm_telemetry` for logging, crash reporting and action counters:

- `Logger::init` is called at startup; the `ClientInitGuard` is held in `main.rs` for the process lifetime.
- Every dispatched command is recorded via `record_action(command_name(cmd))` in `background_loop`.
- On graceful disconnect, the WebSocket handler calls `flush("Server Action Telemetry")`. This label is what tells server-side counters apart from `rpfm_ui`'s `"UI Action Telemetry"` events in Sentry.

Both of `rpfm_telemetry`'s opt-out toggles (`enable_usage_telemetry`, `enable_crash_reports`) are read from settings and applied at runtime.

## Building & running

The server is normally spawned automatically by `rpfm_ui` (debug builds run `cargo build -p rpfm_server` first; release builds launch the bundled binary). To build or run it directly:

```bash
# Debug build:
cargo build -p rpfm_server

# Release build:
cargo build --release -p rpfm_server

# Run it:
./target/release/rpfm_server
```

Logs go to stderr.

## Cargo features

This crate has no feature flags. It enables `integration_git`, `integration_assembly_kit` and `support_error_bitcode` on `rpfm_lib`.

## Related crates

- **rpfm_ipc** — Wire protocol shared with `rpfm_ui` and any other client.
- **rpfm_lib** — Core file-format library doing the actual decode/encode.
- **rpfm_extensions** — Higher-level workflows (dependencies, diagnostics, search, optimizer, translator, glTF).
- **rpfm_telemetry** — Logging, crash reports, action counters.
- **rpfm_ui** — Qt6 desktop client.

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM
