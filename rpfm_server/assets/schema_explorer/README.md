# Schema explorer (frontend)

Browser page that shows an RPFM DB schema as an interactive graph: one node per table, one edge per reference. It only reads schema information (definitions, fields, references, patches), never table data. The design and the planned server side are in [`docs/server/schema_explorer.md`](../../../docs/server/schema_explorer.md).

`rpfm_server` doesn't serve this page yet. Until it does, open it straight from disk and load a schema file.

## Try it

1. Open `index.html` in a browser, e.g. `xdg-open rpfm_server/assets/schema_explorer/index.html`, or double-click it. No build step or local server is needed.
2. Click **Open schema file**, or drop a file on the page:
   - `schema_<game>.ron` from RPFM's `schemas` folder (on Linux, `~/.config/rpfm/schemas/schema_wh3.ron`), or
   - a schema saved as JSON, including the reply of the MCP `get_schema` tool.
3. The whole schema appears as a star map. Click a star, or search for a table (e.g. `battles`) and press Enter.

On Warhammer 3 (1,744 tables), the first load takes about 4 seconds: parsing, the layout and drawing. After that, the layout comes from the browser's cache and loading takes about 2 seconds.

## Using it

- **Star map:** every table is a white star, sized by how many tables it's linked to.
  - The positions come from a ForceAtlas2 layout of the whole schema (`js/layout.js`). It's computed once per schema, deterministic, and cached in the browser.
  - Tables with no references at all are grouped in the "Tables without references" cluster on the right.
  - Tables that are referenced but have no definition are grey stars.
- **Edges:** one curved arrow per pair of tables, pointing from the referencing table to the referenced one. When the two tables reference each other, the arrow has heads at both ends. Click an edge to see the columns behind it.
- **Selecting:** click a star (or a table link in the panel) and everything else dims:
  - **Purple:** the selected table.
  - **Red (in):** tables that reference it.
  - **Yellow (out):** tables it references.
  - **Orange:** both.

  Double-click a star, or pick a search result, to also zoom to it. Click empty space or **Overview** to go back to the whole map.
- **Depth / Direction:** how far around the selection to highlight (depth 2 and 3 add a dimmer ring), and whether to follow references, "referenced by", or both.
- **Hide hubs over N:** tables with more than N linked tables (e.g. `factions`) are never highlighted through. With the box ticked, they're left out entirely. Path finding then avoids them too.
- **Search:** ranks exact names, then prefixes, word prefixes, substrings and loose matches. Arrow keys and Enter pick a result.
- **Details panel:**
  - Every definition version, with keys, types, references (click to select the target), lookups, defaults, descriptions and patch notes.
  - Localised fields.
  - "Referenced by", across all versions of all tables, like RPFM's references.
  - A path finder, which draws the path in green.
- **Recompute layout:** forgets the cached positions and runs the layout again.
- **Links:** the selected table is kept in the URL fragment (`#table=battles`), so a view can be bookmarked.

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
| `js/graph_index.js` | Neighbourhoods, shortest paths, table pairs and search over the graph. |
| `js/layout.js` | ForceAtlas2 layout of the whole graph, star overlap removal, isolated-table cluster, layout cache. |
| `js/data_source.js` | API mode and file mode. |
| `js/explorer.js` | UI: Cytoscape.js star map, highlighting, details panel, search, URL fragment. |
| `vendor/` | Pinned Cytoscape.js and graphology (with ForceAtlas2) builds, see `vendor/README.md`. |
| `tests/` | Unit tests for the parser, model, graph queries and layout. |

Scripts are plain (non-module) scripts loaded in order, because browsers refuse ES modules from `file://` pages.

## Tests

```sh
node --test "rpfm_server/assets/schema_explorer/tests/*.test.js"
```

They need Node 18 or newer and no packages. CI doesn't run them yet, and the UI itself is checked by hand.
