<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# References

Methods to follow references between tables, and between tables and Loc files.

## `references.definition`

Finds the row where a value of a referenced table is defined.

Searches the open packs, then the parent packs, the game files and the Assembly Kit tables, and returns the first row found.

**Params** (`FindDefinition`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Referenced table, like `factions_tables`. |
| `column` | `string` | yes | Referenced column. If it's a localised column, the table's key is searched instead. |
| `value` | `string` | yes | Value to find. |
| `pack` | `string` or `null` | no | If set, only this open pack is searched, instead of all of them and the dependencies. |

**Result** (`FoundRow`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `values` | map of `any` | yes | Values of the row, by column name. |
| `column_index` | `integer` | yes | Index of the column the value is in. |
| `path` | `string` | yes | Path of the table in its source. |
| `row_index` | `integer` | yes | Index of the row. |
| `source` | [`FileSource`](#type-filesource) | yes | Where the table is. |

## `references.usages`

Finds the rows of other tables referencing a value of a table.

The referencing columns are taken from the schema. Searches the open packs, the parent packs and the game files.

**Params** (`FindUsages`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Table the value is from, like `factions_tables`. |
| `column` | `string` | yes | Column the value is from, like `key`. |
| `value` | `string` | yes | Value to find usages of. |
| `version` | `integer` or `null` | no | Version of the table's definition, to know which columns reference it. Defaults to the version in the game files, or the newest one. |
| `pack` | `string` or `null` | no | If set, only this open pack is searched, instead of all of them and the dependencies. |
| `offset` | `integer` | no, default `0` | Amount of usages to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of usages to return. Defaults to 200. |

**Result** (`Usages`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `usages` | array of [`Usage`](#type-usage) | yes | The usages in the page. |
| `total` | `integer` | yes | Amount of usages, in all pages. |

## `references.loc`

Finds the row of a Loc file with a key.

Searches the open packs, then the parent packs and the game files, and returns the first row found.

**Params** (`FindLoc`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key to find, like `factions_screen_name_wh_main_emp_empire`. |
| `pack` | `string` or `null` | no | If set, only this open pack is searched, instead of all of them and the dependencies. |

**Result** (`FoundRow`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `values` | map of `any` | yes | Values of the row, by column name. |
| `column_index` | `integer` | yes | Index of the column the value is in. |
| `path` | `string` | yes | Path of the table in its source. |
| `row_index` | `integer` | yes | Index of the row. |
| `source` | [`FileSource`](#type-filesource) | yes | Where the table is. |

## `references.loc_source`

Returns the table, column and row a loc key belongs to.

**Params** (`GetLocSource`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | The loc key, like `factions_screen_name_wh_main_emp_empire`. |

**Result** (`LocSourceLookup`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`LocSource`](#type-locsource) or `null` | yes | Table, column and row the key belongs to, if they were found. |

## `references.row_locs`

Returns the loc entries of a row of a DB table: the loc key of each of its localised columns, with its text.

**Params** (`GetRowLocs`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `row` | [`RowRef`](#type-rowref) | yes | The row. |

**Result** (`RowLocs`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `locs` | array of [`RowLoc`](#type-rowloc) | yes | The loc entry of each localised column of the row. |

## `references.table_data`

Returns the values each column of a table can reference, with their lookups.

Meant for clients editing whole tables, to fill their dropdowns. Others should use `references.values`.

**Params** (`GetTableReferenceData`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `version` | `integer` | yes | Version of the table's definition. |
| `regenerate` | `boolean` | no, default `false` | If the references to the tables of the open packs are generated again, instead of reusing the cached ones. |

**Result** (`TableReferenceData`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `columns` | map of `any` | yes | Referenceable values, by column index. |

## `references.values`

Returns the values a reference column of a table can have, with their display text.

The values come from the referenced column, in the open packs and the dependencies.

**Params** (`GetReferenceValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table with the reference column, like `units_tables`. |
| `column` | `string` | yes | Name of the reference column. |
| `prefix` | `string` | no, default `""` | Only values starting with this are returned. |
| `offset` | `integer` | no, default `0` | Amount of values to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of values to return. Defaults to 500. |

**Result** (`ReferenceValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `values` | array of [`ReferenceValue`](#type-referencevalue) | yes | The values in the page, sorted. |
| `total` | `integer` | yes | Amount of values matching the request, in all pages. |

## Types

### Type `FileSource`

Where a file is.

One of:

- `{ "pack": … }`: An open pack, by key.
    - `pack` (`string`, yes)
- `"game_files"`: The vanilla files of the selected game.
- `"parent_files"`: The files of the parent packs of the open packs.
- `"assembly_kit"`: The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.

### Type `LocSource`

Table, column and row a loc key belongs to.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `factions_tables`. |
| `column` | `string` | yes | Name of the localised column. |
| `key_values` | array of `string` | yes | Values of the key columns of the row. |

### Type `ReferenceValue`

A value a reference column can have.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `value` | `string` | yes | The value. |
| `lookup` | `string` | no | Text shown next to the value, like the name of the referenced row. Empty if there's none. |

### Type `RowLoc`

The loc entry of a localised column of a row.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `column` | `string` | yes | Name of the localised column, like `onscreen_name`. |
| `key` | `string` | yes | Loc key of the column, like `land_units_onscreen_name_my_unit`. |
| `text` | `string` or `null` | yes | Text of the loc key, from the open packs or the dependencies, or null if there is none. |

### Type `RowRef`

A row of a table, in any source.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The table. |
| `index` | `integer` | yes | Index of the row in the table. |

### Type `FileRef`

A file in one of the sources.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where the file is. |
| `path` | `string` | yes | Path of the file in its source. |

### Type `Usage`

A row referencing a value.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `column` | `string` | yes | Name of the column referencing the value. |
| `values` | map of `any` | yes | Values of the row, by column name. |
| `column_index` | `integer` | yes | Index of the column the value is in. |
| `path` | `string` | yes | Path of the table in its source. |
| `row_index` | `integer` | yes | Index of the row. |
| `source` | [`FileSource`](#type-filesource) | yes | Where the table is. |
