# Schema explorer (proposal)

> **Status:** the frontend is implemented in `rpfm_server/assets/schema_explorer/` and works from a schema file. The server endpoints and Part 2 aren't implemented.

A browser-based, interactive graph of a game's DB schema, served by `rpfm_server`. Every table is a node and every reference (`Field::is_reference`) is an edge. Users can search for a table, expand the tables around it, follow references in both directions and read field definitions.

The document has two parts:

1. [The read-only explorer](#part-1-read-only-explorer): what to build first. It only serves `GET` requests and changes no state.
2. [Opening a table in the RPFM UI](#part-2-open-the-db-table-in-rpfm-separate-change): a separate follow-up change that lets the page ask the RPFM window that opened it to open a table. It's the first browser-reachable endpoint that makes RPFM *do* something, so it has its own security section.

## Why a web page

- The vendored Qt bindings (`qt_core`, `qt_gui`, `qt_widgets`, `qt_ui_tools`) don't include QtWebEngine. Adding it means new FFI and shipping Chromium on every platform.
- QML / `QJSEngine` have no DOM, so JS graph libraries can't run there.
- A native `QGraphicsScene` graph would need layout, pan/zoom and filtering written by hand.
- `rpfm_server` already runs an axum HTTP server on `127.0.0.1:45127`. Serving a static page plus a JSON API from it is a small change, and the browser provides the sandbox: the page can only make HTTP requests back to the server.

---

# Part 1: Read-only explorer

## Scope

- Only `GET` endpoints. No `POST`, no WebSocket, no MCP calls from the page.
- No session involved. The explorer reads schema files from disk and never touches a UI or MCP session's game selected, open packs or dependencies.
- Data shown: tables, their definition versions, fields and references. No table *rows*, so no game data is read.

## Architecture

```
browser ──GET /schema-explorer/...──▶ rpfm_server (axum)
                                       ├─ static page + vendored JS (embedded in the binary)
                                       └─ JSON API ──▶ in-memory SchemaGraph cache ──▶ schema file on disk
                                                                                       (schemas_path + local table patches)
```

## Data source

Load the schema directly with the same paths `background_thread::load_schema` uses:

```rust
let schema_path = schemas_path()?.join(game.schema_file_name());
let local_patches_path = table_patches_path()?.join(game.schema_file_name());
let schema = Schema::load(&schema_path, Some(&local_patches_path))?;
```

This matches what a session would see after `set_game_selected`, including the user's local schema patches, but doesn't need one.

**Caching:** keep one built graph per game in a `LazyLock<RwLock<HashMap<String, Arc<SchemaGraph>>>>`. Rebuild it when the schema file's or patch file's modification time changes, so a schema update or a new local patch shows up after a page reload. Building the graph is a single pass over the definitions, which is cheap next to loading the file.

## Size of the graph (Warhammer 3, measured)

| Metric | Value |
|---|---|
| Tables | 1,744 (2,342 definitions across all versions) |
| Reference edges (newest definition of each table) | 3,299, between 2,917 table pairs |
| Tables with no references in or out | 76 |
| Referenced tables with no definition in the schema | 214 (e.g. `tristates`, `audio_vo_actors`) |
| Busiest tables (edges in + out) | `factions` 121, `effects` 83, `main_units` 83, `building_levels` 72 |
| Full schema as JSON (`get_schema`) | 5.1 MB |

What this means for the design:

- Send a **compact graph** built on the server, not the raw schema. Field details are fetched per table, on demand.
- Don't draw all 1,744 nodes by default. Start from a search and show the **neighbourhood** of the selected table. Allow an "everything" view, but with hub tables collapsed.
- Show referenced tables that have no definition as distinct "missing" nodes. They're useful for finding schema gaps.

## HTTP API

All routes sit under `/schema-explorer` and accept only `GET` (and `HEAD`). Responses are JSON unless noted.

| Route | Returns |
|---|---|
| `GET /schema-explorer` | `index.html` |
| `GET /schema-explorer/assets/{file}` | Vendored JS/CSS, from a fixed allowlist of embedded files |
| `GET /schema-explorer/api/games` | `[{ "key": "warhammer_3", "display_name": "...", "schema_available": true }]` |
| `GET /schema-explorer/api/{game}/graph` | Compact graph, see below |
| `GET /schema-explorer/api/{game}/tables/{table}` | All definition versions of one table, plus the tables and columns that reference it |

`{game}` must be a key from `SupportedGames`. `{table}` is only ever looked up in the in-memory schema, never used to build a file path.

### `graph` response

```json
{
  "game": "warhammer_3",
  "schema_version": 5,
  "nodes": [
    { "id": "battles", "table": "battles_tables", "versions": [17, 16, 13, 11, 8],
      "fields": 28, "keys": ["key"], "localised": true, "missing": false }
  ],
  "edges": [
    { "from": "battles", "from_column": "type", "to": "battle_types", "to_column": "type", "lookup": true }
  ]
}
```

- Node IDs drop the `_tables` suffix, since `is_reference` targets are stored without it (`["battle_types", "type"]`).
- Edges come from the **newest** definition of each table. The per-table endpoint shows the older versions.
- `lookup` marks references that also have `Field::lookup` columns, i.e. the UI shows a human-readable value from the target table.

### `tables/{table}` response

- `definitions`: every version, with fields from `Definition::fields_processed()` (the row shape users see in the editor) and `localised_fields`.
- `referenced_by`: from `Schema::referencing_columns_for_table`, the same data the `referencing_columns_for_definition` MCP tool returns.

## Frontend

- **Libraries:** [Cytoscape.js](https://js.cytoscape.org/) draws the graph. [graphology](https://graphology.github.io/)'s ForceAtlas2 computes a layout of the **whole** schema once per schema. It's deterministic and cached in the browser. The page opens on the full star map and selecting a table only highlights, so the map never moves. Both libraries are single files with no build step.
- **Vendoring:** check pinned, minified copies into `rpfm_server/assets/schema_explorer/` and embed them with `include_bytes!`/`include_str!`. No CDN: the page must work offline and must not load third-party code.
- **Code:** implemented in `rpfm_server/assets/schema_explorer/` (see its `README.md`). It uses plain scripts with no bundler or npm build step, not ES modules, because browsers refuse modules on `file://` pages:
  - `index.html`, `css/explorer.css`: the page shell and styles.
  - `js/explorer.js`: the UI.
  - `js/graph_index.js`: neighbourhood, path, table-pair and search queries.
  - `js/layout.js`: the ForceAtlas2 layout, star overlap removal, the cluster of tables without references, and the layout cache.
  - `js/data_source.js`: API mode or file mode.
  - `js/schema_model.js` and `js/ron.js`: build this API's JSON in the browser from a schema file, so the page works before the server side exists.
  - `vendor/`: the vendored libraries.
  - `tests/`: Node unit tests.
- **File mode:** until the endpoints exist, the page loads a `schema_*.ron` or JSON schema the user picks. Once the server serves the page, it switches to the API automatically.
- **Interactions:**
  - Game picker. Remember the last game in `localStorage`.
  - Table search with fuzzy matching on table name.
  - The whole schema shown on load as a star map, one curved arrow per table pair (arrowheads at both ends when two tables reference each other).
  - Selecting a table highlights it in purple, the tables referencing it in red and the tables it references in yellow, to depth 1–3, following references, "referenced by" or both.
  - Click a node to open a side panel with versions, key fields, field types, descriptions, the target of each reference, and "referenced by".
  - Collapse or hide hub tables above a degree threshold (e.g. `factions`).
  - Shortest path between two tables (BFS on the client).
  - Deep links via the URL fragment (`#game=warhammer_3&table=battles`), so a view can be bookmarked or shared.

**Layout on the server (optional):** the layout takes about 2 seconds of browser time for Warhammer 3 on first load. The server could compute it once per schema and add `x`/`y` to each node of the `graph` response. The frontend would then skip the layout step.

## Server changes

- New module `rpfm_server/src/server_schema_explorer.rs`:
  - Graph types (`SchemaGraph`, `GraphNode`, `GraphEdge`) with `Serialize`.
  - `SchemaGraph::from_schema(&Schema) -> SchemaGraph`, a pure function that's easy to unit-test.
  - The cache and the axum handlers.
  - `pub fn router() -> Router` mounted in `main.rs` with `.nest("/schema-explorer", server_schema_explorer::router())`.
- Static assets under `rpfm_server/assets/schema_explorer/`, served with explicit `Content-Type` headers.
- No new crates: `axum` (with `Path` and `Json` extractors), `serde_json` and `rpfm_lib` are already dependencies.
- Docs: add this page's final version to `docs/server/` and a line in `docs/server/overview.md`.

**Optional UI entry point** (can come later): a `rpfm_ui` menu action, e.g. *Tools > Schema explorer*, that calls `QDesktopServices::open_url("http://127.0.0.1:45127/schema-explorer#<game>")`. This already works in `notes/mod.rs` and `github_ui`, and it sends nothing back to the UI.

## Security (read-only explorer)

The explorer only reads public schema data, but it's a page on a localhost server that any website in the same browser can try to reach:

- **DNS rebinding:** a hostile site can point its own domain at `127.0.0.1` and read responses as same-origin. Reject any request whose `Host` header isn't `127.0.0.1:45127`, `localhost:45127` or `[::1]:45127`. Use an axum middleware for this, using the same list `rmcp` already enforces on `/mcp` by default. It should be applied to the explorer routes and is worth applying to all routes.
- **Cross-origin reads:** send no `Access-Control-Allow-Origin` header, so the browser's same-origin policy stops other sites reading the JSON.
- **XSS:** schema strings (table names, field names, descriptions) come from schema files and the user's local patches, which can contain anything. Insert them with `textContent`, never `innerHTML`, and send a strict CSP:
  - `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'`
  - No inline scripts or styles.
- **Clickjacking:** `frame-ancestors 'none'` in the CSP above (plus `X-Frame-Options: DENY` for older browsers).
- **MIME sniffing:** `X-Content-Type-Options: nosniff` and explicit content types for all assets.
- **Path traversal:** validate `{game}` against `SupportedGames` before building the schema file name, and serve assets only from the embedded allowlist, never from the filesystem by URL path.
- **Method restriction:** register `GET` routes only, so axum rejects other methods with `405`.
- **What's exposed:** the schema itself is public (it ships in `rpfm-schemas`). The user's local schema patches are also visible, which is acceptable for a localhost tool, but the page shouldn't show filesystem paths.
- **Not covered here:** the existing `/ws` endpoint accepts handshakes from any `Origin` or `Host`. That's a separate issue and should be fixed on its own; the explorer doesn't depend on `/ws`.

## Testing

- **Unit tests** (`rpfm_server/src/server_schema_explorer_test.rs`), building a small `Schema` in code:
  - Nodes and edges from references, with `_tables` suffix handling.
  - Missing-target nodes.
  - Edges taken from the newest definition only.
  - The `lookup` flag.
- **Router tests:**
  - `GET` returns the expected JSON.
  - An unknown game returns `404`.
  - A foreign `Host` returns `403`.
  - `POST` returns `405`.
  - The CSP and `nosniff` headers are present.
- **Manual:** open `http://127.0.0.1:45127/schema-explorer` for WH3, search `battles_tables`, expand it, and check its edges to `battle_types`, `cultures` and `audio_battle_environments`, and its "referenced by" list (`battle_armies_junctions`, `battle_sources`, ...).
- CI builds and tests `rpfm_server`, so all of this runs in CI. Only the optional UI menu action needs a local build.

## Delivery steps

1. Graph model, `SchemaGraph::from_schema` and unit tests.
2. JSON API, Host middleware, security headers and router tests.
3. Static page with vendored Cytoscape.js: search, neighbourhood view, side panel.
4. Extras: path finding, hub collapsing, deep links.
5. Optional `rpfm_ui` menu action.

---

# Part 2: "Open the DB table in RPFM" (separate change)

Goal: from the explorer, click a table and have **the RPFM window that opened the page** open that table: the vanilla file from the dependencies, or the table in the user's open pack if it has one.

This is a separate change because it needs a state-changing endpoint, a server-to-UI push, and UI work that CI can't build.

## Flow

```
RPFM UI ──open_url(...#session=<id>&token=<t>)──▶ browser
browser ──POST /schema-explorer/api/open {table}──▶ rpfm_server
rpfm_server ──resolve table to (DataSource, path) in that session──▶ push Response::OpenFileRequested over that session's /ws
RPFM UI (tokio reader) ──crossbeam channel──▶ QTimer on the GUI thread ──▶ AppUI::open_packedfile(...)
```

## Required changes

**`rpfm_ipc`**
- `Response::OpenFileRequested(DataSource, String)`: an unsolicited push, sent with message `id: 0` like `SettingsChanged`.
- `Command::GetExplorerToken`: the UI asks its session for an explorer token. Reply: `Response::String(token)`.
- Document both in `docs/server/ws-commands.md` and `ws-responses.md`.

**`rpfm_server`**
- **Push channel per session:** add a `tokio::sync::broadcast::Sender<Response>` (or `mpsc`) to `Session`. In `server_websocket::handle_socket`, subscribe to it the same way the socket already forwards `SETTINGS_CHANGED`, so pushes reach only that session's UI.
- **Explorer token per session:** 128 random bits generated on `GetExplorerToken`, stored on the `Session` and dropped with it.
- **New endpoint `POST /schema-explorer/api/open`:** body `{"session": <id>, "table": "battles_tables"}`, token in an `X-RPFM-Explorer-Token` header. It:
  1. Checks `Host`, `Origin`, `Content-Type` and the token (see below).
  2. Finds the session. Returns `410 Gone` if the RPFM window is no longer connected.
  3. Resolves the table name to a `(DataSource, path)` inside that session: the open pack's table file first, otherwise the vanilla file from dependencies. This needs a small new command that returns paths only; the existing `get_tables_from_dependencies` returns full table data, which is too heavy.
  4. Pushes `OpenFileRequested` and returns `202 Accepted`.

**`rpfm_ui`**
- **WebSocket reader** (`communications/mod.rs`, next to the `SettingsChanged` handling): forward `OpenFileRequested` onto a crossbeam channel instead of a response slot.
- **GUI thread:** a `QTimer` (e.g. every 200 ms) reads that channel and calls `AppUI::open_packedfile(..., Some(path), is_preview, false, data_source, Pane::...)`, expanding the dependencies tree the way `go_to_definition` does (`views/table/mod.rs`).
- **Menu action:** request the token, then open `http://127.0.0.1:45127/schema-explorer#game=<game>&session=<id>&token=<token>`.
- **Window raising:** try `raise()` / `activateWindow()`. Many window managers (Wayland in particular) block focus stealing, so the page should also say "Opened in RPFM".

**Explorer page**
- If the fragment has `session` and `token`, show an "Open in RPFM" button in the side panel. Otherwise hide it, so the read-only explorer works as before.

## Security concerns

This is the first endpoint a web page can use to make RPFM act, and every website open in the same browser can send requests to `127.0.0.1`.

1. **Cross-site request forgery (CSRF):** a hostile page can submit a form or `fetch` with `mode: "no-cors"` to `http://127.0.0.1:45127/...` without the user noticing. Simple requests (`text/plain` or form bodies, no custom headers) skip the CORS preflight, so CORS alone doesn't stop them. Required:
   - Require `Content-Type: application/json` **and** the custom `X-RPFM-Explorer-Token` header. Both force a preflight, and the server never answers preflights with permissive CORS headers.
   - Require `Origin` to be exactly `http://127.0.0.1:45127` (or the `localhost` / `[::1]` forms). Reject requests with a missing or `null` origin.
2. **Guessable session IDs:** session IDs are small sequential integers, so the `session` value proves nothing. The **token** is the real credential:
   - 128 bits from a CSPRNG, one per session, compared in constant time.
   - It only works for its own session and dies when the session ends.
3. **Token leakage:**
   - Pass the token in the URL **fragment** (`#token=`), which browsers never send to servers or in `Referer` headers, and send it only in the header.
   - Never put it in a query string, logs (`info!` request logging), telemetry or Sentry breadcrumbs.
   - Set `Referrer-Policy: no-referrer` on the page.
   - Browser history keeps fragments, but the token dies with the session.
4. **DNS rebinding:** the `Host` allowlist from Part 1 must also cover this endpoint. Otherwise a rebound page counts as same-origin.
5. **No raw paths from the browser:** the body carries only a table name. The server resolves it to a path from known sources (open pack, dependencies). It never accepts a filesystem or pack path from the browser, never opens `ExternalFile` sources, and never triggers `OpenPackedFileInExternalProgram`. Even a stolen token can then only open DB tables in a preview tab.
6. **Abuse and UI spam:** a page with a valid token could open hundreds of tabs. Rate-limit per session (e.g. 5 opens per second) and open as a **preview** tab, which replaces itself instead of piling up.
7. **Clickjacking:** keep `frame-ancestors 'none'` so a hostile page can't frame the explorer and trick clicks onto "Open in RPFM".
8. **Prerequisite, the `/ws` Origin check:** `/ws` currently accepts handshakes from any `Origin` and `Host`, and it gives full command access (save to arbitrary paths, change settings, open packs). As long as that's true, this endpoint's protections add little, because a hostile page can use `/ws` directly. Before shipping Part 2:
   - Reject `/ws` handshakes that carry a browser `Origin` header. The UI's `tokio-tungstenite` client sends none.
   - Apply the `Host` allowlist to `/ws` and `/sessions`.

## Testing

- **Server unit/router tests:**
  - Missing, wrong or other-session tokens are rejected (`401`/`403`).
  - A foreign `Origin` or `Host` is rejected.
  - A `text/plain` body is rejected (`415`).
  - An unknown session returns `410`.
  - A valid request results in one `OpenFileRequested` on the right session's push channel only.
  - Rate limiting works.
- **Manual (UI isn't built in CI):**
  - Open the explorer from the menu and click "Open in RPFM" on a vanilla table and on a table in the open pack. Check that the right tab opens and the dependencies tree is expanded.
  - Close RPFM and check that the page reports the window is gone.
