<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Tables

Methods to read DB and Loc tables, and their definitions.

Tables are described by their columns as rows see them: colour columns are merged,
bitwise columns split and enum columns converted, so each row has one value per column.

## `table.info`

Returns the definition and row count of a table.

**Params** (`GetTableInfo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The table. |

**Result** (`TableInfo`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`, or `loc` for Loc files. |
| `version` | `integer` | yes | Version of the table's definition. |
| `columns` | array of [`Field`](#type-field) | yes | Columns of the table, in the order row values are, with the schema patches applied. |
| `row_count` | `integer` | yes | Amount of rows in the table. |

## `table.rows`

Returns rows of a table, optionally filtered and with only some columns.

**Params** (`GetTableRows`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The table. |
| `columns` | array of `string` or `null` | no | If set, only these columns are returned, in this order. |
| `filters` | array of [`RowFilter`](#type-rowfilter) | no, default `[]` | Only rows matching all these filters are returned. |
| `offset` | `integer` | no, default `0` | Amount of matching rows to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of rows to return. Defaults to 100. |

**Result** (`TableRows`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `columns` | array of `string` | yes | Names of the columns of the values of each row. |
| `rows` | array of [`TableRow`](#type-tablerow) | yes | The rows in the page. |
| `total` | `integer` | yes | Amount of rows matching the filters, in all pages. |

## `schema.definition`

Returns the columns of a table as defined in the schema.

**Params** (`GetTableDefinition`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `version` | `integer` or `null` | no | Version of the definition. If not set, the version the table has in the game files is used, or the newest one if the game files are not loaded or don't have the table. |

**Result** (`TableDefinition`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table. |
| `version` | `integer` | yes | Version of the definition. |
| `columns` | array of [`Field`](#type-field) | yes | Columns of the table, in the order row values are, with the schema patches applied. |

## `table.edit`

Edits rows of a DB or Loc table in an open pack.

Edits are applied in order, each one on the result of the previous ones. If any of them fails, none is applied.

**Params** (`EditTable`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the table in the pack. |
| `edits` | array of [`RowEdit`](#type-rowedit) | yes | Edits to apply. |

**Result** (`TableEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `row_count` | `integer` | yes | Amount of rows the table has after the edits. |

## `table.column_values`

Returns the distinct values of a column of a table, sorted.

**Params** (`GetColumnValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `factions_tables`. |
| `column` | `string` | yes | Name of the column. |
| `include_packs` | `boolean` | no, default `true` | If the tables of the open packs are included. |
| `include_dependencies` | `boolean` | no, default `true` | If the tables of the game files and the parent packs are included. |
| `prefix` | `string` | no, default `""` | Only values starting with this are returned. |
| `offset` | `integer` | no, default `0` | Amount of values to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of values to return. Defaults to 500. |

**Result** (`ColumnValues`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `values` | array of `string` | yes | The values in the page, sorted. |
| `total` | `integer` | yes | Amount of values matching the request, in all pages. |

## `table.unused_numbers`

Returns numbers no row uses yet in an integer column of a table, for new rows needing unique ones, like ids.

**Params** (`GetUnusedNumbers`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `main_units_tables`. |
| `column` | `string` | yes | Name of the integer column, like `unique_index`. |
| `count` | `integer` | no, default `1` | Amount of numbers to return, up to 10000. Defaults to 1. |

**Result** (`UnusedNumbers`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `numbers` | array of `integer` | yes | The numbers, from lowest to highest, all above the highest one used in the open packs and the dependencies. |

## `table.merge`

Merges tables of the same type of an open pack into a new one.

**Params** (`MergeTables`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `paths` | array of `string` | yes | Paths of the tables to merge. |
| `merged_path` | `string` | yes | Path of the merged table. |
| `delete_sources` | `boolean` | no, default `false` | If the merged tables are deleted afterwards. |
| `delta` | `boolean` | no, default `false` | If rows are merged by key against the vanilla data, instead of concatenated. Rows that can't be reconciled automatically are returned as conflicts, and nothing is written. |
| `resolutions` | array of `any` | no, default `[]` | How to resolve the conflicts of a previous delta merge. |

**Result** (`TablesMerged`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `merged` | `string` or `null` | yes | Path of the merged table, if the merge was done. |
| `conflicts` | array of `any` | no | Rows that couldn't be reconciled, if the merge wasn't done. Pass resolutions for them to merge again. |

## `table.upgrade`

Updates a table of an open pack to the version it has in the game files.

**Params** (`UpgradeTable`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the table. |

**Result** (`TableUpgraded`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `old_version` | `integer` | yes | Version the table had. |
| `new_version` | `integer` | yes | Version the table has now. |
| `deleted_columns` | array of `string` | yes | Columns removed by the update. |
| `added_columns` | array of `string` | yes | Columns added by the update, with their default value. |

## `table.rename_key`

Changes a value of a key column of a table in every table of an open pack,
including the columns referencing it and the loc keys generated from it.

**Params** (`RenameKey`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `table_name` | `string` | yes | Name of the table the key is from, like `factions_tables`. |
| `column` | `string` | yes | Name of the key column. |
| `old_value` | `string` | yes | Current value. |
| `new_value` | `string` | yes | New value. |
| `version` | `integer` or `null` | no | Version of the table's definition. Defaults to the version in the game files, or the newest one. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `table.add_key_deletes`

Adds rows to a key deletes table of an open pack, to delete keys of a table in the game.

**Params** (`AddKeyDeletes`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `file_name` | `string` | yes | File name of the key deletes table, under `db/twad_key_deletes_tables/`. |
| `table_name` | `string` | yes | Name of the table the keys belong to, like `units_tables`. |
| `keys` | array of `string` | yes | Keys to delete. |

**Result** (`FilesEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `edited` | array of `string` | yes | Paths of the edited files. |

## `table.export_tsv`

Writes a table to a TSV file.

**Params** (`ExportTsv`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The table. Assembly Kit tables can't be exported. |
| `destination` | `string` | yes | Path of the TSV file to write. |
| `tsv_keys_first` | `boolean` or `null` | no | If the TSV uses the old column order, with keys first. Defaults to the server's setting. |

**Result** (`Done`)

Empty object: `{}`.

## `table.import_tsv`

Replaces a table of an open pack with the contents of a TSV file, keeping its GUID.

**Params** (`ImportTsv`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `path` | `string` | yes | Path of the table in the pack. |
| `tsv_path` | `string` | yes | Path of the TSV file to read. |

**Result** (`TableEdited`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `row_count` | `integer` | yes | Amount of rows the table has after the edits. |

## Types

### Type `Field`

A column of a table, as the schema describes it.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | `string` | yes | Name of the column. |
| `field_type` | `any` | yes | Type of the values of the column, like `StringU8`, `I32`, `F32`, `Boolean` or `ColourRGB`. Sequences are an object with the definition of their rows, like `{ "SequenceU32": definition }`. |
| `is_key` | `boolean` | yes | If the column is part of the key of the table. |
| `default_value` | `string` or `null` | yes | Value of the column in new rows, if it has one. |
| `is_filename` | `boolean` | yes | If the column holds paths of files. |
| `filename_relative_path` | `string` or `null` | yes | Folders the files of the column are in, separated by `;`. A `%` in them stands for the value of the column. |
| `is_reference` | [`string`, `string`] or `null` | yes | Table and column the values of this column must exist in, like `["factions", "key"]`. The table name has no `_tables` suffix. |
| `lookup` | array of `string` or `null` | yes | Columns of the referenced table shown next to the values of this one, to identify them. |
| `description` | `string` | yes | Description of the column. Empty if it has none. |
| `ca_order` | `integer` | yes | Position of the column in the Assembly Kit, or `-1` if unknown. |
| `is_bitwise` | `integer` | yes | If more than 1, the column is one of the boolean columns a bitwise column was split into, named `<column>_1` to `<column>_<is_bitwise>`. |
| `enum_values` | map of `string` | yes | Names of the values of a column converted from an integer one, by value. |
| `is_part_of_colour` | `integer` or `null` | yes | Group of the colour columns merged into this one, if it's a merged colour column. |
| `unused` | `boolean` | yes | If the game doesn't use the column. |

### Type `FileRef`

A file in one of the sources.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `source` | [`FileSource`](#type-filesource) | yes | Where the file is. |
| `path` | `string` | yes | Path of the file in its source. |

### Type `FileSource`

Where a file is.

One of:

- `{ "pack": … }`: An open pack, by key.
    - `pack` (`string`, yes)
- `"game_files"`: The vanilla files of the selected game.
- `"parent_files"`: The files of the parent packs of the open packs.
- `"assembly_kit"`: The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.

### Type `RowEdit`

An edit of the rows of a table.

Values are given by column name, as booleans, numbers or strings, and converted to the type of their column.

One of:

- `{ "op": "insert", "copy_from"?: …, "index"?: …, "values"?: … }`: Adds a row. Columns not set get the value of the copied row, or their default value.
    - `copy_from` ([`RowRef`](#type-rowref) or `null`, no): Row to copy, from any table with columns of the same names, like the vanilla one. Columns it doesn't have get their default value.
    - `index` (`integer` or `null`, no): Position of the new row. If not set, it's added at the end.
    - `values` (map of `any`, no, default `{}`): Values of the new row, by column name. They replace the ones of the copied row.
- `{ "op": "update", "index": …, "values": … }`: Changes values of a row.
    - `index` (`integer`, yes): Position of the row.
    - `values` (map of `any`, yes): New values, by column name. Columns not set keep their value.
- `{ "op": "delete", "indexes": … }`: Removes rows.
    - `indexes` (array of `integer`, yes): Positions of the rows to remove, as they're before this edit.

### Type `RowFilter`

A condition on the value of a column, compared as text.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `column` | `string` | yes | Name of the column. |
| `op` | [`FilterOp`](#type-filterop) | yes | How the value is compared. |
| `value` | `string` | yes | Value to compare with. |
| `ignore_case` | `boolean` | no, default `false` | If the comparison ignores case. |

### Type `FilterOp`

Comparison of a row filter.

One of:

- `"equals"`: The value is the same as the filter's.
- `"not_equals"`: The value is not the same as the filter's.
- `"contains"`: The value contains the filter's.
- `"starts_with"`: The value starts with the filter's.
- `"ends_with"`: The value ends with the filter's.

### Type `RowRef`

A row of a table, in any source.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | [`FileRef`](#type-fileref) | yes | The table. |
| `index` | `integer` | yes | Index of the row in the table. |

### Type `TableRow`

A row of a table.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `index` | `integer` | yes | Position of the row in the table. Used to edit it. |
| `values` | array of `any` | yes | Values of the row, one per returned column: booleans, numbers or strings. |
