# Schema explorer (frontend)

Browser page that shows an RPFM DB schema as an interactive graph: one node per table, one edge per reference. It only reads schema information (definitions, fields, references, patches), never table data. The design and the planned server side are in [`docs/server/schema_explorer.md`](../../../docs/server/schema_explorer.md).

`rpfm_server` doesn't serve this page yet. Until it does, open it straight from disk and load a schema file.

## Try it

1. Open `index.html` in a browser, e.g. `xdg-open rpfm_server/assets/schema_explorer/index.html`, or double-click it. No build step or local server is needed.
2. Click **Open schema file**, or drop a file on the page:
   - `schema_<game>.ron` from RPFM's `schemas` folder (on Linux, `~/.config/rpfm/schemas/schema_wh3.ron`), or
   - a schema saved as JSON, including the reply of the MCP `get_schema` tool.
3. The whole schema appears as a star map. Click a star, or search for a table (e.g. `battles`) and press Enter.

The first time a schema is loaded, the layout runs until it settles (on Warhammer 3, about 1,300 iterations and 10–20 seconds, with progress shown). After that, the saved layout is used and loading takes a couple of seconds.

## Using it

- **Star map:** every table is a white star, sized by how many tables it's linked to. Names only show on highlighted stars (selected, hovered, on a path or matching the search).
  - The positions come from a ForceAtlas2 layout of the whole schema (`js/layout.js`), run until the stars stop moving, then saved in the browser (one layout per game or schema file name).
  - Tables with no references at all sit on a spiral inside the "Tables without references" circle on the right. New ones continue the spiral outward.
  - Tables that are referenced but have no definition are grey stars.
- **Moving stars:** drag a star to rearrange the map. The new position is saved and the star is pinned, so later layout runs leave it where you put it. Drag the circle to move the whole cluster.
- **Zooming:** you can't zoom out further than the circle around all stars filling the view.
- **Schema updates:** when a schema with the same name changes (new or removed tables or references), the saved layout is kept. New tables start next to the tables they're linked to, and ForceAtlas2 continues from the existing map instead of starting over.
- **Edges:** one curved arrow per pair of tables, pointing from the referencing table to the referenced one. When the two tables reference each other, the arrow has heads at both ends. Click an edge to see the columns behind it.
- **Selecting:** click a star (or a table link in the panel). Its details panel opens and everything else dims. Only the colour of the stars changes:
  - **Purple:** the selected table.
  - **Red (in):** tables that reference it.
  - **Yellow (out):** tables it references.
  - **Orange:** both.

  Each of the selected table's arrows takes a darker shade of its neighbour's colour. Hovering a star previews the same colouring at half opacity, on top of any selection.

  Double-click a star, or pick a search result, to also zoom to it. Click empty space or **Close** in the panel to deselect. The panel is only shown while a table is selected.
- **Search:** ranks exact names, then prefixes, word prefixes, substrings and loose matches. Arrow keys and Enter pick a result. The listed results light up blue on the map and stay lit until the search box is emptied (Escape clears it); the current selection is left as it is.
- **Controls:** the title, the schema button (it shows the loaded schema's name; click it to open another) and the search bar float over the top of the map.
- **Tooltips:** hover a star or a table link for a summary of the table (the schema has no table-level comments). Hover a field row for the schema's comment on it; fields that have one are underlined with dots.
- **Details panel** (a floating card on the right, while a table is selected; Fields and Referenced by fold away and stay folded for the next table):
  - Every definition version, with keys, types, references (click to select the target), lookups, defaults, descriptions and patch notes.
  - Localised fields.
  - "Referenced by", across all versions of all tables, like RPFM's references.
  - A path finder, which draws the path in green.
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
| `js/layout.js` | ForceAtlas2 layout run to convergence, incremental updates for schema changes, pinned (dragged) stars, star overlap removal, isolated-table cluster, saved layouts. |
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
