<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Session and dependencies

Methods about the state of the session.

## `session.status`

Returns the selected game, what's loaded for it, and the open packs.

**Params** (`GetSessionStatus`)

Empty object: `{}`.

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `session.configure`

Replaces the settings the session runs with.

Sessions start with the settings in the settings file. Clients owning their own settings send them
with this before anything else, and again whenever they change.

**Params** (`Configure`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `settings` | `any` | yes | The settings. |

**Result** (`Done`)

Empty object: `{}`.

## `session.disconnect`

Tells the server the client is closing, so its WebSocket session is removed right away
instead of waiting for the timeout. Only for WebSocket clients.

**Params** (`Disconnect`)

Empty object: `{}`.

**Result** (`Done`)

Empty object: `{}`.

## `session.set_game`

Selects a game, loading its schema and, optionally, its dependencies.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`SetGame`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the game, like `warhammer_3`. |
| `rebuild_dependencies` | `boolean` | no, default `true` | If the dependencies (vanilla files, Assembly Kit tables, parent packs) are loaded for the game. |

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `dependencies.generate_cache`

Generates the dependencies cache of the selected game from its files and
Assembly Kit, and loads it.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`GenerateDependenciesCache`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `ignore_game_files_in_assembly_kit` | `boolean` or `null` | no | If tables in the game files are skipped when reading the Assembly Kit. Defaults to the server's setting. |

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `dependencies.rebuild`

Reloads the dependencies of the selected game, like after changing the packs
the open packs depend on.

Runs as a job: answers right away with a job ID, and sends its result when the job ends (see [Jobs](../ws-protocol.md#jobs)).

**Params** (`RebuildDependencies`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `only_parent_packs` | `boolean` | no, default `false` | If only the parent packs are reloaded, instead of the whole dependencies. |

**Result** (`SessionStatus`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `string` | yes | Key of the selected game, like `warhammer_3`. |
| `schema_loaded` | `boolean` | yes | If a schema is loaded for the selected game. Tables can't be read without one. |
| `dependencies` | [`DependenciesStatus`](#type-dependenciesstatus) | yes | What's loaded of the dependencies of the selected game. |
| `packs` | array of [`PackSummary`](#type-packsummary) | yes | The open packs. |

## `dependencies.tables`

Returns the tables of the game files, and the startpos, twad and CEO tables of the schema, with their version.

**Params** (`ListDependencyTables`)

Empty object: `{}`.

**Result** (`DependencyTables`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `tables` | map of `integer` | yes | Version of each table, by table name. Empty if the dependencies are not loaded. |

## `dependencies.info`

Returns the files of the dependencies of the selected game.

Meant for clients showing the dependencies as a tree. Others should use `files.list`.

**Params** (`GetDependenciesInfo`)

Empty object: `{}`.

**Result** (`DependenciesInfo`)

One of RPFM's own types, without a schema. See its docs in `rpfm_lib` or `rpfm_extensions`.

## `dependencies.table_data`

Returns every decoded table of a type in the game files and the parent packs.

Meant for clients comparing whole tables. Others should use `table.rows`.

**Params** (`GetDependencyTableData`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `table_name` | `string` | yes | Name of the table, like `units_tables`. |

**Result** (`DependencyTableData`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `tables` | array of `any` | yes | The tables. |

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
