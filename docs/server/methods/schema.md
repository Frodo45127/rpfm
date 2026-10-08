<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Schema

Methods about the schema of the selected game: its tables, local patches, and updates.

Definitions of single tables are read with `GetTableDefinition`.

## `schema.tables`

Lists the tables of the schema, with the versions it has definitions for.

**Params** (`ListSchemaTables`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `prefix` | `string` | no, default `""` | Only tables whose name starts with this are listed. |

**Result** (`SchemaTables`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `tables` | map of array of `integer` | yes | Versions with a definition, by table name, newest first. |

## `schema.patch_column`

Changes how the schema describes a column, with a local patch.

Local patches are kept apart from the schema, so they survive schema updates. The schema is reloaded
after saving the patch, so it applies right away.

**Params** (`PatchColumn`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `column` | `string` | yes | Name of the column. |
| `patch` | map of `string` | yes | Values to set, by patch key. The keys are `description`, `is_key`, `default_value`, `is_filename`, `filename_relative_path`, `is_reference`, `lookup`, `lookup_hardcoded`, `not_empty` and `unused`. |

**Result** (`Done`)

Empty object: `{}`.

## `schema.remove_patches`

Removes the local patches of a table, or of one of its columns, and reloads the schema.

**Params** (`RemovePatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `column` | `string` or `null` | no | If set, only the patches of this column are removed. |

**Result** (`Done`)

Empty object: `{}`.

## `schema.update`

Downloads the latest schemas, reloads the selected game's one, and rebuilds the dependencies.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`UpdateSchemas`)

Empty object: `{}`.

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `schema.update_from_assembly_kit`

Updates the schema of the selected game with the tables of its Assembly Kit, and saves it.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`UpdateSchemaFromAssemblyKit`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `ignore_game_files` | `boolean` or `null` | no | If tables in the game files are skipped. Defaults to the server's setting. |

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `schema.raw_definitions`

Returns definitions of a table as the schema stores them, to edit them.

Their fields are the raw on-disk layout, which can differ from the columns rows have
(see `schema.definition` for those).

**Params** (`GetRawDefinitions`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `version` | `integer` or `null` | no | If set, only the definition of this version is returned. |

**Result** (`RawDefinitions`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `definitions` | array of `any` | yes | The definitions, newest first. |

## `schema.set_definition`

Adds a definition to the schema of the selected game, or replaces the one with
its version, then saves and reloads the schema.

**Params** (`SetDefinition`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `definition` | `any` | yes | The definition, in the format `schema.raw_definitions` returns. |

**Result** (`Done`)

Empty object: `{}`.

## `schema.delete_definition`

Removes a definition from the schema of the selected game, then saves and reloads the schema.

**Params** (`DeleteDefinition`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |
| `version` | `integer` | yes | Version of the definition. |

**Result** (`Done`)

Empty object: `{}`.

## `schema.referencing_columns`

Returns the columns of other tables referencing each column of a table.

**Params** (`GetReferencingColumns`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `factions_tables`. |
| `version` | `integer` or `null` | no | Version of the table's definition. Defaults to the version in the game files, or the newest one. |

**Result** (`ReferencingColumns`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `columns` | map of map of array of `string` | yes | For each referenced column, the referencing columns by table name. |

## `schema.patches`

Returns the patches applied to the columns of a table.

**Params** (`GetTablePatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `factions_tables`. |
| `version` | `integer` | yes | Version of the table's definition. If the table has no definition with this version, the patches of the table are returned. |

**Result** (`TablePatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `patches` | map of map of `string` | yes | Patches by column name, as key-value pairs. See the valid keys in `schema.patch_column`. |

## `schema.import_patches`

Adds patches to the schema of the selected game itself, and saves it.

Unlike the ones of `schema.patch_column`, these patches are part of the schema, so a schema update replaces them.

**Params** (`ImportPatches`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `patches` | map of map of map of `string` | yes | Patches, by table name, column name and patch key. The keys are `description`, `is_key`, `default_value`, `is_filename`, `filename_relative_path`, `is_reference`, `lookup`, `lookup_hardcoded`, `not_empty` and `unused`. |

**Result** (`Done`)

Empty object: `{}`.

## `schema.missing_definitions`

Returns the tables of an open pack with rows that can't be decoded with the schema.

**Params** (`GetMissingDefinitions`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |

**Result** (`MissingDefinitions`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `paths` | array of `string` | yes | Paths of the tables, sorted. |

## Types

### Type `DependenciesStatus`

What's loaded of the dependencies of the selected game.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `vanilla_loaded` | `boolean` | yes | If the vanilla files of the game are loaded. |
| `assembly_kit_loaded` | `boolean` | yes | If the tables of the game's Assembly Kit are loaded. |
| `parent_files` | `integer` | yes | Amount of files loaded from the parent packs of the open packs. |
| `error` | `string` or `null` | no | Why the dependencies couldn't be fully loaded, like a missing or outdated dependencies cache, if they couldn't. |

### Type `PackSummary`

Short description of an open pack.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Key identifying the pack in every method that works on it: a UUID, like `0b4c5a62-7d3e-4f1a-9c2b-8e6d5f4a3b21`. It never changes while the pack is open. |
| `name` | `string` | yes | File name of the pack, like `my_mod.pack`. |
| `path` | `string` or `null` | yes | Path of the pack on disk, if it's saved there. |
| `pack_type` | `string` | yes | Type of the pack: `Boot`, `Release`, `Patch`, `Mod` or `Movie`. |
| `file_count` | `integer` | yes | Amount of files in the pack. |
