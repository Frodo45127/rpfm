# Schema explorer (frontend)

Browser page that shows an RPFM DB schema as an interactive graph: one node per table, one edge per reference. It only reads schema information (definitions, fields, references, patches), never table data. The design and the planned server side are in [`docs/server/schema_explorer.md`](../../../docs/server/schema_explorer.md).

`rpfm_server` doesn't serve this page yet. Until it does, open it straight from disk and load a schema file.

## Try it

1. Open `index.html` in a browser, e.g. `xdg-open rpfm_server/assets/schema_explorer/index.html`, or double-click it. No build step or local server is needed.
2. Click **Open schema file**, or drop a file on the page:
   - `schema_<game>.ron` from RPFM's `schemas` folder (on Linux, `~/.config/rpfm/schemas/schema_wh3.ron`), or
   - a schema saved as JSON, including the reply of the MCP `get_schema` tool.
3. Search for a table (e.g. `battles`) and press Enter.

Parsing the 11 MB Warhammer 3 schema takes well under a second.

## Using it

- **Search:** ranks exact names, then prefixes, word prefixes, substrings and loose matches. Arrow keys and Enter pick a result.
- **Graph:** shows the tables around the focused one.
  - Click a table to see its details, and double-click it to focus on it.
  - Hover a table to highlight its links. Click an edge to see the columns behind it.
  - Edges point from the referencing table to the referenced one.
- **Depth / Direction:** how far to expand, and whether to follow references, "referenced by", or both.
- **Hide hubs over N:** tables with more than N linked tables (e.g. `factions`) are never expanded through. With the box ticked, they're hidden. Path finding then avoids them too.
- **Details panel:**
  - Every definition version, with keys, types, references (click to add the target to the graph), lookups, defaults, descriptions and patch notes.
  - Localised fields.
  - "Referenced by", across all versions of all tables, like RPFM's references.
  - A path finder.
- **Show all:** draws every table. It's dense and slow to lay out, so prefer search.
- **Links:** the focused table is kept in the URL fragment (`#table=battles`), so a view can be bookmarked.
- **Not-in-schema tables:** dashed nodes are tables other tables reference that have no definition in the schema.

## Data sources

`js/data_source.js` picks one of two sources:

- **API mode:** when the page is served over HTTP and `api/games` answers. It reads the endpoints proposed in the design doc (`api/games`, `api/{game}/graph`, `api/{game}/tables/{table}`).
- **File mode:** otherwise. `js/schema_model.js` builds the same JSON in the browser.

Both produce the same shapes, so the server only has to return what `SchemaModel.graph()` and `SchemaModel.tableDetails()` return today.

Differences in file mode:

- Only the patches inside the schema file are applied. Your local schema patches (RPFM's `table_patches` folder) aren't loaded.
- Fields are listed as stored in the schema. The server could return `Definition::fields_processed()` instead, which merges colour columns and expands bitwise fields.

## Files

| Path | What it is |
|---|---|
| `index.html`, `css/explorer.css` | Page and styles. No inline scripts or styles, so a strict CSP works. |
| `js/ron.js` | RON parser that produces serde_json-shaped values. |
| `js/schema_model.js` | Graph and table details from a schema, with `rpfm_lib`'s patch rules. |
| `js/graph_index.js` | Neighbourhoods, shortest paths and search over the graph. |
| `js/data_source.js` | API mode and file mode. |
| `js/explorer.js` | UI: Cytoscape.js graph, details panel, search, URL fragment. |
| `vendor/` | Pinned Cytoscape.js and fcose layout builds, see `vendor/README.md`. |
| `tests/` | Unit tests for the parser, model and graph queries. |

Scripts are plain (non-module) scripts loaded in order, because browsers refuse ES modules from `file://` pages.

## Tests

```sh
node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"
```

They need Node 18 or newer and no packages. CI doesn't run them yet, and the UI itself is checked by hand.
