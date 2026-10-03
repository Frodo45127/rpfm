# Server overview

`rpfm_server` is a standalone backend process that exposes RPFM's functionality over a local network protocol. The desktop UI (`rpfm_ui`) is one client of this server, but it isn't the only one — anything that speaks JSON-RPC over WebSocket, or anything that speaks the [Model Context Protocol][mcp], can drive the same backend.

[mcp]: https://modelcontextprotocol.io/

This section is for **tool authors and integrators**. For end-user workflows, you don't need anything here.

## Architecture

```
┌──────────────────────┐    WebSocket    ┌──────────────────────┐
│       rpfm_ui        │ ◀────────────▶  │     rpfm_server      │
│  (Qt6 desktop app)   │                 │   (this binary)      │
└──────────────────────┘                 └──────────┬───────────┘
                                                    │
┌──────────────────────┐    HTTP/SSE                │
│   MCP clients (AI)   │ ◀──────────▶  /mcp ◀───────┘
└──────────────────────┘
```

- The server binds to `127.0.0.1:45127` by default.
- All clients talk to the same backend, but each connection has its own **session** (own open Packs, own dependency cache, own settings).
- Behind the scenes the heavy lifting is in `rpfm_lib` and `rpfm_extensions`. The server is mostly an IPC gateway plus session bookkeeping.

## What the server can do

Pretty much everything `rpfm_ui` can:

- Open, save, close and inspect Packs.
- Add, extract, rename, copy, delete, duplicate files.
- Read and write every supported file type (DB tables, Loc, text, animations, …).
- Run search, diagnostics, references against open Packs.
- Manage dependencies, schemas, MyMods.
- Drive the optimizer, the translator, startpos building, glTF export, etc.

The full surface is documented in [Methods](./methods.md).

## Endpoints

The server exposes three HTTP endpoints on `127.0.0.1:45127`:

| Endpoint    | Method | Purpose                                                                           |
|-------------|--------|-----------------------------------------------------------------------------------|
| `/ws`       | GET    | WebSocket upgrade. Carries the JSON-RPC 2.0 protocol.                             |
| `/sessions` | GET    | REST: list every active session. Used by the UI session picker.                   |
| `/version`  | GET    | REST: the server's version and process ID. Used by the UI to reuse a running server. |
| `/mcp`      | *      | MCP `StreamableHttpService`. Each client gets its own session and `McpServer`.   |

## Two ways in

You'll typically pick one of two integration paths:

- **WebSocket** — for full programmatic access. You write client code that sends JSON-RPC requests and matches their responses by ID. See [WebSocket protocol](./ws-protocol.md).
- **MCP** — for AI agents and other clients that already speak MCP. The server exposes most methods as tools, plus prompts and resources. See [MCP interface](./mcp.md).

Under the hood both pathways use the same `Session` abstraction. Each WebSocket connection gets its own isolated session with its own open Packs and dependency cache, and can reattach to it by passing its previous `session_id` back on the handshake. All MCP clients share one session, separate from the WebSocket ones. A WebSocket client and an MCP client running side by side don't share state — they share the server process.

## Spawning the server

`rpfm_ui` spawns `rpfm_server` automatically and the UI's lifecycle owns the server's. To run the server standalone (for tool development, automated tests, or to keep it warm between UI sessions):

```bash
./rpfm_server
```

The server logs to stderr and stays in the foreground until every session is gone.

## Where to next

- [Sessions & connection lifecycle](./sessions.md) — what happens on connect, disconnect, reconnect.
- [WebSocket protocol](./ws-protocol.md) — message envelope, errors, jobs and conventions.
- [Methods](./methods.md) — every method of the API.
- [MCP interface](./mcp.md) — what tools the MCP endpoint exposes.
- [Client example](./client-example.md) — a concrete TypeScript / C# client implementation.
