# Schemas & patches

Schemas describe the binary layout of every supported DB table for every supported game. Without them, RPFM has no idea what a `units_tables/data` file looks like — it's a binary blob.

## Where schemas come from

Schemas live in the [`rpfm-schemas`](https://github.com/Frodo45127/rpfm-schemas) repository. **About → Check Updates** pulls the latest version from upstream over Git as part of the unified update check (it also covers the program itself, lua autogen and the legacy AK definitions). The Git update path is gated behind the `integration_git` Cargo feature, which is enabled by default in the shipped UI build.

Each schema is a RON file: `schema_<game>.ron`. Plus:

- `anim_ids_<game>.csv` — per-game animation ID maps for `matched_combat` / `anim_fragment_battle`.

## When to update schemas

When RPFM says there's a new schema update available. There's no benefit on using older versions of the schemas.

## Patches

Schemas describe what a table *is*, and a large part of their information is extracted automatically from the Assembly Kit of the games. Patches are manually generated corrections for the schemas: lookup overrides, default values, tooltips, type-display hints, etc. They live alongside the schemas and overlay the base schema at runtime.

To contribute a patch upstream so everyone benefits, PR your changes to [`rpfm-schemas`](https://github.com/Frodo45127/rpfm-schemas). The repo's README covers the file format.

## Adding or fixing a schema

When a game patch changes a table layout (new columns, reorder, type change), the existing schema is missing the new definitions and the tables won't decode. Fix it:

1. Open one of the affected files in the [DB Decoder](../editors/decoder.md).
2. Adjust the column types until the first row decoded field shows sane values.
3. Save the definition into the schema.
4. Try to open the table again and see if it opens correctly.
5. PR your changes to [`rpfm-schemas`](https://github.com/Frodo45127/rpfm-schemas).

## API surface

If you're consuming schemas from Rust, the [`rpfm_lib::schema`](../../api/rpfm_lib/schema/index.html) module is the entry point. Key types:

- `Schema` — the full per-game schema document.
- `Definition` — a single table version's definition.
- `Field` and `FieldType` — column metadata.
