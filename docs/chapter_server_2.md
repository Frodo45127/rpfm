# Commands

This page documents all commands that can be sent to the RPFM server. Each command is wrapped in a [Message](./chapter_server_0.md#message-format) with a unique `id`.

Commands with no parameters are serialized as plain strings. Commands with parameters use the `{ "CommandName": params }` format. See the [serialization convention](./chapter_server_0.md#serialization-convention) for details.

Most commands that operate on a specific pack take a `pack_key` string as their first parameter. The pack key is returned by `OpenPackFiles` or `NewPack` when you open or create a pack.

---

## Lifecycle

### Exit

Close the background thread. **Do not use directly** — the server manages this internally.

Response: None (breaks the server loop).

### ClientDisconnecting

Signal that the client is intentionally disconnecting. Allows the server to clean up the session immediately.

Response: `"Success"`

```json
{ "id": 1, "data": "ClientDisconnecting" }
```

---

## PackFile Operations

### NewPack

Create a new empty Pack.

Response: `{ String: string }` — the assigned pack key.

```json
{ "id": 1, "data": "NewPack" }
```

### OpenPackFiles

Open one or more Pack files and merge them into the current session.

| Parameter | Type     | Description            |
|-----------|----------|------------------------|
| `paths`   | string[] | Filesystem paths to open |

Response: `{ StringContainerInfo: [string, ContainerInfo] }` — pack key and metadata.

```json
{ "id": 1, "data": { "OpenPackFiles": ["/path/to/my_mod.pack"] } }
```

### LoadAllCAPackFiles

Open all CA Pack files for the selected game as one merged Pack.

Response: `{ StringContainerInfo: [string, ContainerInfo] }`

```json
{ "id": 1, "data": "LoadAllCAPackFiles" }
```

### ListOpenPacks

List all currently open packs with their keys and metadata.

Response: `{ VecStringContainerInfo: [string, ContainerInfo][] }`

```json
{ "id": 1, "data": "ListOpenPacks" }
```

### ClosePack

Close a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to close |

Response: `"Success"`

```json
{ "id": 1, "data": { "ClosePack": "my_mod.pack" } }
```

### CloseAllPacks

Close all currently open Packs.

Response: `"Success"`

```json
{ "id": 1, "data": "CloseAllPacks" }
```

### SavePack

Save a specific open Pack to disk.

| Parameter  | Type   | Description  |
|------------|--------|--------------|
| `pack_key` | string | Pack to save |

Response: `{ ContainerInfo: ContainerInfo }`

```json
{ "id": 1, "data": { "SavePack": "my_mod.pack" } }
```

### SavePackAs

Save a specific open Pack to a new path.

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| `pack_key`| string | Pack to save     |
| `path`    | string | Destination path |

Response: `{ ContainerInfo: ContainerInfo }`

```json
{ "id": 1, "data": { "SavePackAs": ["my_mod.pack", "/path/to/new_mod.pack"] } }
```

### CleanAndSavePackAs

Clean a Pack from corrupted/undecoded files and save to disk. Only use if the Pack is otherwise unsaveable.

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| `pack_key`| string | Pack to clean    |
| `path`    | string | Destination path |

Response: `{ ContainerInfo: ContainerInfo }`

```json
{ "id": 1, "data": { "CleanAndSavePackAs": ["my_mod.pack", "/path/to/cleaned.pack"] } }
```

### GetPackFileDataForTreeView

Get tree view data (container info and file list) for a specific pack.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to query  |

Response: `{ ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }`

```json
{ "id": 1, "data": { "GetPackFileDataForTreeView": "my_mod.pack" } }
```

### GetPackedFilesInfo

Get metadata for one or more packed files by path.

| Parameter | Type     | Description         |
|-----------|----------|---------------------|
| `pack_key`| string   | Pack to query       |
| `paths`   | string[] | Internal file paths |

Response: `{ VecRFileInfo: RFileInfo[] }`

```json
{ "id": 1, "data": { "GetPackedFilesInfo": ["my_mod.pack", ["db/units_tables/data"]] } }
```

### GetRFileInfo

Get the info of a single packed file.

| Parameter | Type   | Description        |
|-----------|--------|--------------------|
| `pack_key`| string | Pack to query      |
| `path`    | string | Internal file path |

Response: `{ OptionRFileInfo: RFileInfo | null }`

```json
{ "id": 1, "data": { "GetRFileInfo": ["my_mod.pack", "db/units_tables/data"] } }
```

### GetPackFilePath

Get the filesystem path of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ PathBuf: string }`

### GetPackFileName

Get the file name of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ String: string }`

### SetPackFileType

Change the PFH type of a specific open Pack (e.g., Mod, Movie, Boot).

| Parameter  | Type        | Description      |
|------------|-------------|------------------|
| `pack_key` | string      | Pack to modify   |
| `type`     | PFHFileType | New file type    |

Response: `"Success"`

### ChangeIndexIncludesTimestamp

Toggle the "Index Includes Timestamp" flag for a specific pack.

| Parameter  | Type    | Description   |
|------------|---------|---------------|
| `pack_key` | string  | Pack to modify|
| `enabled`  | boolean | New flag value|

Response: `"Success"`

### ChangeCompressionFormat

Change the compression format of a specific open Pack.

| Parameter  | Type              | Description       |
|------------|-------------------|-------------------|
| `pack_key` | string            | Pack to modify    |
| `format`   | CompressionFormat | New format        |

Response: `{ CompressionFormat: CompressionFormat }` — actual format set (may differ if unsupported).

### OptimizePackFile

Run the optimizer over a specific open Pack.

| Parameter | Type             | Description         |
|-----------|------------------|---------------------|
| `pack_key`| string           | Pack to optimize    |
| `options` | OptimizerOptions | Optimization config |

Response: `{ HashSetStringHashSetString: [string[], string[]] }` — deleted and added paths.

### PatchSiegeAI

Patch Siege AI for Warhammer siege maps in a specific pack.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to patch  |

Response: `{ StringVecContainerPath: [string, ContainerPath[]] }`

---

## Game Selection

### GetGameSelected

Get the currently selected game key.

Response: `{ String: string }`

```json
{ "id": 1, "data": "GetGameSelected" }
```

### SetGameSelected

Change the selected game. Optionally rebuilds dependencies.

| Parameter    | Type    | Description                       |
|--------------|---------|-----------------------------------|
| `game_key`   | string  | Game identifier (e.g., `"warhammer_3"`) |
| `rebuild`    | boolean | Whether to rebuild dependencies   |

Response: `{ CompressionFormatDependenciesInfo: [CompressionFormat, DependenciesInfo | null] }`

```json
{ "id": 1, "data": { "SetGameSelected": ["warhammer_3", true] } }
```

### GenerateDependenciesCache

Generate the dependencies cache for the currently selected game.

Response: `{ DependenciesInfo: DependenciesInfo }`

### UpdateCurrentSchemaFromAssKit

Update the current schema with data from the game's Assembly Kit.

Response: `"Success"`

---

## PackedFile Operations

### NewPackedFile

Create a new packed file inside a specific open Pack.

| Parameter  | Type    | Description                |
|------------|---------|----------------------------|
| `pack_key` | string  | Target pack                |
| `path`     | string  | Internal path for the file |
| `spec`     | NewFile | File type specification    |

Response: `"Success"`

```json
{ "id": 1, "data": { "NewPackedFile": ["my_mod.pack", "db/units_tables/data", { "DB": ["data", "units_tables", 4] }] } }
```

### AddPackedFiles

Add files from the filesystem to a specific open Pack.

| Parameter      | Type                    | Description                         |
|----------------|-------------------------|-------------------------------------|
| `pack_key`     | string                  | Target pack                         |
| `source_paths` | string[]                | Filesystem paths to add             |
| `dest_paths`   | ContainerPath[]         | Destination paths inside the pack   |
| `ignore_paths` | string[] or null        | Paths to exclude (optional)         |

Response: `{ VecContainerPathOptionString: [ContainerPath[], string | null] }` — added paths and optional error.

### DecodePackedFile

Decode a packed file for display.

| Parameter  | Type       | Description                |
|------------|------------|----------------------------|
| `pack_key` | string     | Pack containing the file   |
| `path`     | string     | Internal path              |
| `source`   | DataSource | Data source to decode from |

Response: Type-specific (e.g., `{ DBRFileInfo: [DB, RFileInfo] }`, `{ TextRFileInfo: [Text, RFileInfo] }`, `"Unknown"`, etc.)

```json
{ "id": 1, "data": { "DecodePackedFile": ["my_mod.pack", "db/units_tables/data", "PackFile"] } }
```

### SavePackedFileFromView

Save an edited packed file back to the Pack.

| Parameter | Type         | Description              |
|-----------|--------------|--------------------------|
| `pack_key`| string       | Target pack              |
| `path`    | string       | Internal path            |
| `data`    | RFileDecoded | Decoded file content     |

Response: `"Success"`

### DeletePackedFiles

Delete packed files from a specific open Pack.

| Parameter | Type            | Description          |
|-----------|-----------------|----------------------|
| `pack_key`| string          | Pack to modify       |
| `paths`   | ContainerPath[] | Paths to delete      |

Response: `{ VecContainerPath: ContainerPath[] }` — deleted paths.

```json
{ "id": 1, "data": { "DeletePackedFiles": ["my_mod.pack", [{ "File": "db/units_tables/data" }]] } }
```

### ExtractPackedFiles

Extract packed files to the filesystem.

| Parameter        | Type                                  | Description                    |
|------------------|---------------------------------------|--------------------------------|
| `pack_key`       | string                                | Pack to extract from           |
| `paths_by_source`| Record<DataSource, ContainerPath[]>   | Files grouped by data source   |
| `dest_path`      | string                                | Filesystem destination         |
| `as_tsv`         | boolean                               | Export tables as TSV           |

Response: `{ StringVecPathBuf: [string, string[]] }`

```json
{ "id": 1, "data": { "ExtractPackedFiles": ["my_mod.pack", { "PackFile": [{ "File": "db/units_tables/data" }] }, "/tmp/extract", false] } }
```

### RenamePackedFiles

Rename packed files in a specific Pack.

| Parameter | Type                                | Description                              |
|-----------|-------------------------------------|------------------------------------------|
| `pack_key`| string                              | Pack to modify                           |
| `renames` | [ContainerPath, ContainerPath][]    | Array of `[old_path, new_path]` pairs    |

Response: `{ VecContainerPathContainerPath: [ContainerPath, ContainerPath][] }`

### FolderExists

Check if a folder exists in a specific open Pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to check |
| `path`    | string | Folder path   |

Response: `{ Bool: boolean }`

### PackedFileExists

Check if a packed file exists in a specific open Pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to check |
| `path`    | string | File path     |

Response: `{ Bool: boolean }`

### GetPackedFileRawData

Get the raw binary data of a packed file.

| Parameter | Type   | Description        |
|-----------|--------|--------------------|
| `pack_key`| string | Pack to query      |
| `path`    | string | Internal file path |

Response: `{ VecU8: number[] }`

### AddPackedFilesFromPackFile

Copy packed files from one Pack into another.

| Parameter    | Type            | Description                |
|--------------|-----------------|----------------------------|
| `target_key` | string          | Destination pack key       |
| `source_key` | string          | Source pack key            |
| `paths`      | ContainerPath[] | Paths to copy              |

Response: `{ VecContainerPath: ContainerPath[] }`

### AddPackedFilesFromPackFileToAnimpack

Copy packed files from the main Pack into an AnimPack.

| Parameter       | Type            | Description              |
|-----------------|-----------------|--------------------------|
| `pack_key`      | string          | Pack containing animpack |
| `animpack_path` | string          | Path to the AnimPack     |
| `paths`         | ContainerPath[] | Paths to copy            |

Response: `{ VecContainerPath: ContainerPath[] }`

### AddPackedFilesFromAnimpack

Copy packed files from an AnimPack into the main Pack.

| Parameter       | Type            | Description              |
|-----------------|-----------------|--------------------------|
| `pack_key`      | string          | Target pack              |
| `source`        | DataSource      | Data source              |
| `animpack_path` | string          | Path to the AnimPack     |
| `paths`         | ContainerPath[] | Paths to copy            |

Response: `{ VecContainerPath: ContainerPath[] }`

### DeleteFromAnimpack

Delete packed files from an AnimPack.

| Parameter       | Type            | Description              |
|-----------------|-----------------|--------------------------|
| `pack_key`      | string          | Pack containing animpack |
| `animpack_path` | string          | Path to the AnimPack     |
| `paths`         | ContainerPath[] | Paths to delete          |

Response: `"Success"`

### ImportDependenciesToOpenPackFile

Import files from dependencies into a specific open Pack.

| Parameter | Type                                | Description                   |
|-----------|-------------------------------------|-------------------------------|
| `pack_key`| string                              | Target pack                   |
| `sources` | Record<DataSource, ContainerPath[]> | Files to import by source     |

Response: `{ VecContainerPathVecString: [ContainerPath[], string[]] }` — added paths, failed paths.

### SavePackedFilesToPackFileAndClean

Save packed files to a specific Pack and optionally run optimizer.

| Parameter | Type     | Description        |
|-----------|----------|--------------------|
| `pack_key`| string   | Target pack        |
| `files`   | RFile[]  | Files to save      |
| `optimize`| boolean  | Run optimizer      |

Response: `{ VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }` — added and deleted paths.

### GetPackedFilesNamesStartingWitPathFromAllSources

Get all file names under a path from all dependency sources.

| Parameter | Type          | Description        |
|-----------|---------------|--------------------|
| `path`    | ContainerPath | Path prefix        |

Response: `{ HashMapDataSourceHashSetContainerPath: Record<DataSource, ContainerPath[]> }`

---

## Dependency Commands

### RebuildDependencies

Rebuild the dependencies by combining deps from all open packs.

| Parameter     | Type    | Description                                   |
|---------------|---------|-----------------------------------------------|
| `rebuild_all` | boolean | true = all dependencies, false = mod-specific |

Response: `{ DependenciesInfo: DependenciesInfo }`

```json
{ "id": 1, "data": { "RebuildDependencies": true } }
```

### IsThereADependencyDatabase

Check if a dependency database is loaded.

| Parameter       | Type    | Description                           |
|-----------------|---------|---------------------------------------|
| `require_asskit`| boolean | Check that AssKit data is included    |

Response: `{ Bool: boolean }`

### GetTableListFromDependencyPackFile

Get all DB table names from dependency Pack files.

Response: `{ VecString: string[] }`

### GetCustomTableList

Get custom table names (start_pos_, twad_ prefixes) from the schema.

Response: `{ VecString: string[] }`

### LocalArtSetIds

Get local art set IDs from campaign_character_arts_tables in a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ HashSetString: string[] }`

### DependenciesArtSetIds

Get art set IDs from dependencies' campaign_character_arts_tables.

Response: `{ HashSetString: string[] }`

### GetTableVersionFromDependencyPackFile

Get the version of a table from the dependency database.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ I32: number }`

### GetTableDefinitionFromDependencyPackFile

Get the definition of a table from the dependency database.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ Definition: Definition }`

### MergeFiles

Merge multiple compatible tables into one.

| Parameter        | Type            | Description                  |
|------------------|-----------------|------------------------------|
| `pack_key`       | string          | Pack containing the files    |
| `paths`          | ContainerPath[] | Files to merge               |
| `merged_path`    | string          | Destination path for result  |
| `delete_sources` | boolean         | Delete source files after    |

Response: `{ String: string }` — merged path.

### UpdateTable

Update a table to a newer schema version.

| Parameter | Type          | Description           |
|-----------|---------------|-----------------------|
| `pack_key`| string        | Pack containing table |
| `path`    | ContainerPath | Table path            |

Response: `{ I32I32VecStringVecString: [old_ver, new_ver, deleted_fields, added_fields] }`

### GetDependencyPackFilesList

Get the list of Pack files marked as dependencies of a specific Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ VecBoolString: [boolean, string][] }` — `[enabled, pack_name]` pairs.

### SetDependencyPackFilesList

Set the list of Pack files marked as dependencies.

| Parameter | Type                 | Description                  |
|-----------|----------------------|------------------------------|
| `pack_key`| string               | Pack to modify               |
| `deps`    | [boolean, string][]  | `[enabled, pack_name]` pairs |

Response: `"Success"`

### GetRFilesFromAllSources

Get packed files from all known sources.

| Parameter  | Type            | Description                  |
|------------|-----------------|------------------------------|
| `paths`    | ContainerPath[] | Paths to retrieve            |
| `lowercase`| boolean         | Normalize paths to lowercase |

Response: `{ HashMapDataSourceHashMapStringRFile: Record<DataSource, Record<string, RFile>> }`

---

## Search Commands

### GlobalSearch

Perform a global search across a specific pack.

| Parameter | Type         | Description          |
|-----------|--------------|----------------------|
| `pack_key`| string       | Pack to search       |
| `config`  | GlobalSearch | Search configuration |

Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }`

```json
{
  "id": 1,
  "data": {
    "GlobalSearch": ["my_mod.pack", {
      "pattern": "cavalry",
      "replace_text": "",
      "case_sensitive": false,
      "use_regex": false,
      "source": "Pack",
      "search_on": { "db": true, "loc": true, "text": true },
      "matches": {},
      "game_key": "warhammer_3"
    }]
  }
}
```

### GlobalSearchReplaceMatches

Replace specific matches in a global search.

| Parameter | Type          | Description            |
|-----------|---------------|------------------------|
| `pack_key`| string        | Pack to modify         |
| `config`  | GlobalSearch  | Search config          |
| `matches` | MatchHolder[] | Matches to replace     |

Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }`

### GlobalSearchReplaceAll

Replace all matches in a global search.

| Parameter | Type         | Description          |
|-----------|--------------|----------------------|
| `pack_key`| string       | Pack to modify       |
| `config`  | GlobalSearch | Search config        |

Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }`

### GetReferenceDataFromDefinition

Get reference data for columns in a table definition.

| Parameter     | Type       | Description                    |
|---------------|------------|--------------------------------|
| `pack_key`    | string     | Pack to query                  |
| `table_name`  | string     | Name of the table              |
| `definition`  | Definition | Table definition               |
| `force_local` | boolean    | Force regeneration from local  |

Response: `{ HashMapI32TableReferences: Record<number, TableReferences> }`

### SearchReferences

Find all references to a value across tables in a specific pack.

| Parameter          | Type                         | Description                        |
|--------------------|------------------------------|------------------------------------|
| `pack_key`         | string                       | Pack to search                     |
| `table_columns`    | Record<string, string[]>     | Map of table name to column names  |
| `search_value`     | string                       | Value to search for                |

Response: `{ VecDataSourceStringStringUsizeUsize: [DataSource, string, string, number, number][] }`

---

## Navigation Commands

### GoToDefinition

Go to the definition of a table reference.

| Parameter     | Type     | Description             |
|---------------|----------|-------------------------|
| `pack_key`    | string   | Pack to search          |
| `table_name`  | string   | Table name              |
| `column_name` | string   | Column name             |
| `values`      | string[] | Values to search for    |

Response: `{ DataSourceStringUsizeUsize: [DataSource, string, number, number] }`

### GoToLoc

Navigate to a loc key's location.

| Parameter | Type   | Description          |
|-----------|--------|----------------------|
| `pack_key`| string | Pack to search       |
| `loc_key` | string | Loc key to find      |

Response: `{ DataSourceStringUsizeUsize: [DataSource, string, number, number] }`

### GetSourceDataFromLocKey

Get the source data (table, column, values) of a loc key.

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| `pack_key`| string | Pack to search   |
| `loc_key` | string | Loc key to look up|

Response: `{ OptionStringStringVecString: [string, string, string[]] | null }`

---

## Cascade Edition

### CascadeEdition

Trigger a cascade edition on all referenced data in a specific pack.

| Parameter  | Type                           | Description                           |
|------------|--------------------------------|---------------------------------------|
| `pack_key` | string                         | Pack to modify                        |
| `table`    | string                         | Table name                            |
| `definition`| Definition                    | Table definition                      |
| `changes`  | [Field, string, string][]      | `[field, old_value, new_value]` tuples|

Response: `{ VecContainerPathVecRFileInfo: [ContainerPath[], RFileInfo[]] }`

---

## Video Commands

### SetVideoFormat

Change the format of a ca_vp8 video packed file.

| Parameter | Type             | Description           |
|-----------|------------------|-----------------------|
| `pack_key`| string           | Pack containing file  |
| `path`    | string           | Internal file path    |
| `format`  | SupportedFormats | Target video format   |

Response: `"Success"`

---

## Schema Commands

### SaveSchema

Save a schema to disk.

| Parameter | Type   | Description            |
|-----------|--------|------------------------|
| `schema`  | Schema | Complete schema to save|

Response: `"Success"`

### CleanCache

Encode and clean the internal cache for specified paths.

| Parameter | Type            | Description            |
|-----------|-----------------|------------------------|
| `pack_key`| string          | Pack to clean          |
| `paths`   | ContainerPath[] | Paths to process       |

Response: `"Success"`

### IsSchemaLoaded

Check if a schema is loaded in memory.

Response: `{ Bool: boolean }`

### Schema

Get the currently loaded schema.

Response: `{ Schema: Schema }`

### DefinitionsByTableName

Get all definitions (all versions) for a table name.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ VecDefinition: Definition[] }`

### DefinitionByTableNameAndVersion

Get a specific definition by table name and version.

| Parameter    | Type   | Description      |
|--------------|--------|------------------|
| `table_name` | string | Table name       |
| `version`    | number | Version number   |

Response: `{ Definition: Definition }`

### DeleteDefinition

Delete a definition by table name and version.

| Parameter    | Type   | Description      |
|--------------|--------|------------------|
| `table_name` | string | Table name       |
| `version`    | number | Version number   |

Response: `"Success"`

### ReferencingColumnsForDefinition

Get columns from other tables that reference a given table/definition.

| Parameter    | Type       | Description       |
|--------------|------------|-------------------|
| `table_name` | string     | Referenced table  |
| `definition` | Definition | Table definition  |

Response: `{ HashMapStringHashMapStringVecString: Record<string, Record<string, string[]>> }`

### FieldsProcessed

Get the processed fields from a definition (bitwise expansion, enum conversion, colour merging applied).

| Parameter    | Type       | Description          |
|--------------|------------|----------------------|
| `definition` | Definition | Definition to process|

Response: `{ VecField: Field[] }`

---

## TSV Commands

### ExportTSV

Export a table as a TSV file.

| Parameter | Type       | Description           |
|-----------|------------|-----------------------|
| `pack_key`| string     | Pack containing table |
| `path`    | string     | Internal table path   |
| `dest`    | string     | Filesystem output path|
| `source`  | DataSource | Data source           |

Response: `"Success"`

### ImportTSV

Import a TSV file as a table.

| Parameter   | Type   | Description               |
|-------------|--------|---------------------------|
| `pack_key`  | string | Target pack               |
| `path`      | string | Internal destination path |
| `tsv_path`  | string | Filesystem TSV path       |

Response: `{ RFileDecoded: RFileDecoded }`

---

## External Program Commands

### OpenContainingFolder

Open the folder containing a specific open Pack in the file manager.

| Parameter  | Type   | Description             |
|------------|--------|-------------------------|
| `pack_key` | string | Pack whose folder to open|

Response: `"Success"`

### OpenPackedFileInExternalProgram

Open a packed file in an external program.

| Parameter | Type          | Description         |
|-----------|---------------|---------------------|
| `pack_key`| string        | Pack containing file|
| `source`  | DataSource    | Data source         |
| `path`    | ContainerPath | File path           |

Response: `{ PathBuf: string }` — extracted temporary path.

### SavePackedFileFromExternalView

Save a packed file that was edited in an external program.

| Parameter  | Type   | Description            |
|------------|--------|------------------------|
| `pack_key` | string | Target pack            |
| `path`     | string | Internal path          |
| `ext_path` | string | External file path     |

Response: `"Success"`

---

## Diagnostics Commands

### DiagnosticsCheck

Run a full diagnostics check over the open packs.

| Parameter    | Type     | Description                      |
|--------------|----------|----------------------------------|
| `pack_keys`  | string[] | Pack keys to check               |
| `check_ak`   | boolean  | Check AssKit-only references     |

Response: `{ Diagnostics: Diagnostics }`

### DiagnosticsUpdate

Run a partial diagnostics update on specific paths.

| Parameter     | Type            | Description                      |
|---------------|-----------------|----------------------------------|
| `diagnostics` | Diagnostics     | Existing diagnostics state       |
| `paths`       | ContainerPath[] | Paths to re-check                |
| `check_ak`    | boolean         | Check AssKit-only references     |

Response: `{ Diagnostics: Diagnostics }`

### AddLineToPackIgnoredDiagnostics

Add a line to a specific pack's ignored diagnostics list.

| Parameter | Type   | Description            |
|-----------|--------|------------------------|
| `pack_key`| string | Pack to modify         |
| `line`    | string | Diagnostic key to ignore|

Response: `"Success"`

---

## Pack Settings Commands

### GetPackSettings

Get the settings of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ PackSettings: PackSettings }`

### SetPackSettings

Set the settings of a specific open Pack.

| Parameter  | Type         | Description       |
|------------|--------------|-------------------|
| `pack_key` | string       | Pack to modify    |
| `settings` | PackSettings | New settings      |

Response: `"Success"`

---

## Notes Commands

### NotesForPath

Get all notes under a given path in a specific pack.

| Parameter | Type   | Description    |
|-----------|--------|----------------|
| `pack_key`| string | Pack to query  |
| `path`    | string | Path prefix    |

Response: `{ VecNote: Note[] }`

### AddNote

Add a note to a specific pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Target pack   |
| `note`    | Note   | Note to add   |

Response: `{ Note: Note }`

### DeleteNote

Delete a note from a specific pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to modify|
| `path`    | string | Note path     |
| `note_id` | number | Note ID       |

Response: `"Success"`

---

## Schema Patch Commands

### SaveLocalSchemaPatch

Save local schema patches to disk.

| Parameter | Type                               | Description              |
|-----------|-------------------------------------|--------------------------|
| `patches` | Record<string, DefinitionPatch>    | Table name to patches    |

Response: `"Success"`

### RemoveLocalSchemaPatchesForTable

Remove all local schema patches for a table.

| Parameter    | Type   | Description   |
|--------------|--------|---------------|
| `table_name` | string | Table name    |

Response: `"Success"`

### RemoveLocalSchemaPatchesForTableAndField

Remove local schema patches for a specific field in a table.

| Parameter    | Type   | Description   |
|--------------|--------|---------------|
| `table_name` | string | Table name    |
| `field_name` | string | Field name    |

Response: `"Success"`

### ImportSchemaPatch

Import schema patches into local patches.

| Parameter | Type                               | Description              |
|-----------|-------------------------------------|--------------------------|
| `patches` | Record<string, DefinitionPatch>    | Table name to patches    |

Response: `"Success"`

---

## Loc Generation Commands

### GenerateMissingLocData

Generate all missing loc entries for a specific open Pack.

| Parameter  | Type   | Description             |
|------------|--------|-------------------------|
| `pack_key` | string | Pack to generate for    |

Response: `{ VecContainerPath: ContainerPath[] }`

---

## Update Commands

### CheckUpdates

Check if there is an RPFM update available.

Response: `{ APIResponse: APIResponse }`

### CheckSchemaUpdates

Check if there is a schema update available.

Response: `{ APIResponseGit: GitResponse }`

### UpdateSchemas

Download and apply schema updates.

Response: `"Success"`

### UpdateMainProgram

Update RPFM to the latest version.

Response: `"Success"`

### CheckLuaAutogenUpdates

Check for updates on the tw_autogen repository.

Response: `{ APIResponseGit: GitResponse }`

### UpdateLuaAutogen

Update the tw_autogen repository.

Response: `"Success"`

### CheckEmpireAndNapoleonAKUpdates

Check for updates on the old Assembly Kit files repository.

Response: `{ APIResponseGit: GitResponse }`

### UpdateEmpireAndNapoleonAK

Update the old Assembly Kit files repository.

Response: `"Success"`

### CheckTranslationsUpdates

Check for translation updates.

Response: `{ APIResponseGit: GitResponse }`

### UpdateTranslations

Update the translations repository.

Response: `"Success"`

---

## MyMod Commands

### InitializeMyModFolder

Initialize a MyMod folder structure.

| Parameter   | Type           | Description                        |
|-------------|----------------|------------------------------------|
| `mod_name`  | string         | Name of the mod                    |
| `game_key`  | string         | Target game                        |
| `sublime`   | boolean        | Create Sublime Text project        |
| `vscode`    | boolean        | Create VS Code project             |
| `gitignore` | string or null | Gitignore content (null = no git)  |

Response: `{ PathBuf: string }` — path to the new pack.

### LiveExport

Live-export a specific Pack to the game's data folder.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to export |

Response: `"Success"`

---

## Translation Commands

### GetPackTranslation

Get pack translation data for a language from a specific pack.

| Parameter  | Type   | Description      |
|------------|--------|------------------|
| `pack_key` | string | Pack to query    |
| `language` | string | Language code    |

Response: `{ PackTranslation: PackTranslation }`

---

## Starpos Commands

### BuildStarpos

Build starpos (pre-processing step) for a specific pack.

| Parameter          | Type    | Description              |
|--------------------|---------|--------------------------|
| `pack_key`         | string  | Target pack              |
| `campaign_id`      | string  | Campaign identifier      |
| `process_hlp_spd`  | boolean | Process HLP/SPD data     |

Response: `"Success"`

### BuildStarposPost

Build starpos (post-processing step) for a specific pack.

| Parameter          | Type    | Description              |
|--------------------|---------|--------------------------|
| `pack_key`         | string  | Target pack              |
| `campaign_id`      | string  | Campaign identifier      |
| `process_hlp_spd`  | boolean | Process HLP/SPD data     |

Response: `{ VecContainerPath: ContainerPath[] }`

### BuildStarposCleanup

Clean up starpos temporary files for a specific pack.

| Parameter          | Type    | Description              |
|--------------------|---------|--------------------------|
| `pack_key`         | string  | Target pack              |
| `campaign_id`      | string  | Campaign identifier      |
| `process_hlp_spd`  | boolean | Process HLP/SPD data     |

Response: `"Success"`

### BuildStarposGetCampaingIds

Get campaign IDs available for starpos building from a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ HashSetString: string[] }`

### BuildStarposCheckVictoryConditions

Check if victory conditions file exists in a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to check |

Response: `"Success"`

---

## Animation Commands

### UpdateAnimIds

Update animation IDs with an offset in a specific pack.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `pack_key`   | string | Pack to modify |
| `starting_id`| number | Starting ID    |
| `offset`     | number | ID offset      |

Response: `{ VecContainerPath: ContainerPath[] }`

### GetAnimPathsBySkeletonName

Get animation paths by skeleton name.

| Parameter       | Type   | Description    |
|-----------------|--------|----------------|
| `skeleton_name` | string | Skeleton name  |

Response: `{ HashSetString: string[] }`

---

## Table Commands

### GetTablesFromDependencies

Get tables from dependencies by table name.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ VecRFile: RFile[] }`

### GetTablesByTableName

Get table paths by table name from a specific Pack.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `pack_key`   | string | Pack to query  |
| `table_name` | string | Table name     |

Response: `{ VecString: string[] }`

### AddKeysToKeyDeletes

Add keys to the key_deletes table in a specific pack.

| Parameter        | Type     | Description         |
|------------------|----------|---------------------|
| `pack_key`       | string   | Target pack         |
| `table_file_name`| string   | Table file name     |
| `key_table_name` | string   | Key table name      |
| `keys`           | string[] | Keys to add         |

Response: `{ OptionContainerPath: ContainerPath | null }`

---

## Map Packing Commands

### PackMap

Pack map tiles into a specific Pack.

| Parameter   | Type                   | Description                   |
|-------------|------------------------|-------------------------------|
| `pack_key`  | string                 | Target pack                   |
| `tile_maps` | string[]               | Tile map paths                |
| `tiles`     | [string, string][]     | `[tile_path, tile_name]` pairs|

Response: `{ VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }` — added and deleted paths.

---

## 3D Export Commands

### ExportRigidToGltf

Export a RigidModel to glTF format.

| Parameter    | Type       | Description       |
|--------------|------------|-------------------|
| `model`      | RigidModel | Model to export   |
| `output_path`| string     | Output file path  |

Response: `"Success"`

---

## Settings Commands

### Settings Getters

All settings getters take a single `key` string parameter and return the value in a typed wrapper:

| Command            | Response Type           |
|--------------------|-------------------------|
| `SettingsGetBool`  | `{ Bool: boolean }`     |
| `SettingsGetI32`   | `{ I32: number }`       |
| `SettingsGetF32`   | `{ F32: number }`       |
| `SettingsGetString`| `{ String: string }`    |
| `SettingsGetPathBuf`| `{ PathBuf: string }`  |
| `SettingsGetVecString` | `{ VecString: string[] }` |
| `SettingsGetVecRaw`| `{ VecU8: number[] }`   |

```json
{ "id": 1, "data": { "SettingsGetString": "game_selected" } }
```

### SettingsGetAll

Get all settings at once (batch loading). Much more efficient than individual calls.

Response: `{ SettingsAll: [Record<string, boolean>, Record<string, number>, Record<string, number>, Record<string, string>] }` — bool, i32, f32, and string settings.

```json
{ "id": 1, "data": "SettingsGetAll" }
```

### Settings Setters

All settings setters take a `[key, value]` tuple and return `"Success"`:

| Command              | Value Type |
|----------------------|------------|
| `SettingsSetBool`    | boolean    |
| `SettingsSetI32`     | number     |
| `SettingsSetF32`     | number     |
| `SettingsSetString`  | string     |
| `SettingsSetPathBuf` | string     |
| `SettingsSetVecString` | string[] |
| `SettingsSetVecRaw`  | number[]   |

```json
{ "id": 1, "data": { "SettingsSetString": ["game_selected", "warhammer_3"] } }
```

### SettingsClearPath

Clear a specific config path entry.

| Parameter | Type   | Description    |
|-----------|--------|----------------|
| `path`    | string | Path to clear  |

Response: `"Success"`

---

## Path Commands

These commands return filesystem paths used by RPFM. All respond with `{ PathBuf: string }`:

| Command                | Description                        |
|------------------------|------------------------------------|
| `ConfigPath`           | Config directory path              |
| `AssemblyKitPath`      | Assembly Kit path for current game |
| `BackupAutosavePath`   | Backup autosave directory          |
| `OldAkDataPath`        | Old AK data directory              |
| `SchemasPath`          | Schemas directory                  |
| `TableProfilesPath`    | Table profiles directory           |
| `TranslationsLocalPath`| Translations local directory       |
| `DependenciesCachePath`| Dependencies cache directory       |

```json
{ "id": 1, "data": "SchemasPath" }
```

---

## Settings Backup Commands

### BackupSettings

Backup current settings to memory (for restore on cancel).

Response: `"Success"`

### ClearSettings

Clear all settings and reset to defaults.

Response: `"Success"`

### RestoreBackupSettings

Restore settings from the in-memory backup.

Response: `"Success"`

### OptimizerOptions

Get the optimizer options configuration.

Response: `{ OptimizerOptions: OptimizerOptions }`

---

## Debug Commands

### GetMissingDefinitions

Export missing table definitions from a specific pack to a file (for debugging/development).

| Parameter  | Type   | Description        |
|------------|--------|--------------------|
| `pack_key` | string | Pack to export from|

Response: `"Success"`

---

## Autosave Commands

### TriggerBackupAutosave

Trigger an autosave backup of a specific Pack.

| Parameter  | Type   | Description          |
|------------|--------|----------------------|
| `pack_key` | string | Pack to back up      |

Response: `"Success"`
