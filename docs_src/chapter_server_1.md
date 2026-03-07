# Shared Types

This page documents all the data types used in the RPFM server protocol. These types appear as parameters in [Commands](./chapter_server_2.md) and as payloads in [Responses](./chapter_server_3.md).

All types are serialized as JSON. Nullable fields use `null` when absent.

---

## Core Types

### DataSource

Discriminates where file data originates from. Serialized as a plain string.

| Value            | Description                    |
|------------------|--------------------------------|
| `"PackFile"`     | An open Pack file              |
| `"GameFiles"`    | Vanilla game files             |
| `"ParentFiles"`  | Parent mod files               |
| `"AssKitFiles"`  | Assembly Kit files             |
| `"ExternalFile"` | External file on disk          |

### ContainerPath

A file or folder path within a Pack. Serialized as a tagged enum:

```json
{ "File": "db/units_tables/data" }
{ "Folder": "db/units_tables" }
```

### RFileInfo

Metadata about a packed file within a container.

| Field            | Type            | Description                                      |
|------------------|-----------------|--------------------------------------------------|
| `path`           | string          | Internal path within the Pack                    |
| `container_name` | string or null  | Name of the containing Pack (null if unknown)    |
| `timestamp`      | number or null  | Last modification timestamp                      |
| `file_type`      | string          | File type enum value (e.g. `"DB"`, `"Loc"`, `"Text"`) |

<!-- langtabs-start -->
```typescript
interface RFileInfo {
  path: string;
  container_name: string | null;
  timestamp: number | null;
  file_type: string;
}
```
```csharp
public class RFileInfo
{
    public string Path { get; set; }
    public string? ContainerName { get; set; }
    public long? Timestamp { get; set; }
    public string FileType { get; set; }
}
```
<!-- langtabs-end -->

### ContainerInfo

Reduced representation of a Pack file (container-level metadata).

| Field           | Type    | Description                              |
|-----------------|---------|------------------------------------------|
| `file_name`     | string  | Name of the Pack file                    |
| `file_path`     | string  | Full path to the Pack file on disk       |
| `pfh_version`   | string  | PFH version enum value                   |
| `pfh_file_type` | string  | PFH file type enum value                 |
| `bitmask`       | unknown | PFH flags bitmask                        |
| `compress`      | string  | Compression format enum value            |
| `timestamp`     | number  | Pack file timestamp                      |

<!-- langtabs-start -->
```typescript
interface ContainerInfo {
  file_name: string;
  file_path: string;
  pfh_version: string;
  pfh_file_type: string;
  bitmask: unknown;
  compress: string;
  timestamp: number;
}
```
```csharp
public class ContainerInfo
{
    public string FileName { get; set; }
    public string FilePath { get; set; }
    public string PfhVersion { get; set; }
    public string PfhFileType { get; set; }
    public object Bitmask { get; set; }
    public string Compress { get; set; }
    public long Timestamp { get; set; }
}
```
<!-- langtabs-end -->

### DependenciesInfo

Information about loaded dependency files, used to populate dependency tree views.

| Field                  | Type         | Description                     |
|------------------------|--------------|---------------------------------|
| `asskit_tables`        | RFileInfo[]  | Assembly Kit table files        |
| `vanilla_packed_files` | RFileInfo[]  | Vanilla game files              |
| `parent_packed_files`  | RFileInfo[]  | Parent mod files                |

<!-- langtabs-start -->
```typescript
interface DependenciesInfo {
  asskit_tables: RFileInfo[];
  vanilla_packed_files: RFileInfo[];
  parent_packed_files: RFileInfo[];
}
```
```csharp
public class DependenciesInfo
{
    public List<RFileInfo> AsskitTables { get; set; }
    public List<RFileInfo> VanillaPackedFiles { get; set; }
    public List<RFileInfo> ParentPackedFiles { get; set; }
}
```
<!-- langtabs-end -->

### SessionInfo

Information about an active session on the server. Returned by the `GET /sessions` REST endpoint.

| Field                    | Type            | Description                                    |
|--------------------------|-----------------|------------------------------------------------|
| `session_id`             | number          | Unique session identifier                      |
| `connection_count`       | number          | Number of active WebSocket connections          |
| `timeout_remaining_secs` | number or null  | Seconds until timeout (null if connections exist) |
| `is_shutting_down`       | boolean         | Whether the session is shutting down            |
| `pack_names`             | string[]        | Names of the open packs                        |

<!-- langtabs-start -->
```typescript
interface SessionInfo {
  session_id: number;
  connection_count: number;
  timeout_remaining_secs: number | null;
  is_shutting_down: boolean;
  pack_names: string[];
}
```
```csharp
public class SessionInfo
{
    public int SessionId { get; set; }
    public int ConnectionCount { get; set; }
    public double? TimeoutRemainingSecs { get; set; }
    public bool IsShuttingDown { get; set; }
    public List<string> PackNames { get; set; }
}
```
<!-- langtabs-end -->

---

## File Types

### NewFile

Parameters for creating a new packed file. Serialized as a tagged enum — the variant name determines the file type:

| Variant            | Payload                                        | Description                                      |
|--------------------|------------------------------------------------|--------------------------------------------------|
| `AnimPack`         | `string`                                       | File name                                        |
| `DB`               | `[string, string, number]`                     | `[file_name, table_name, version]`               |
| `Loc`              | `string`                                       | Table name                                       |
| `PortraitSettings` | `[string, number, [string, string][]]`         | `[name, version, clone_entries]`                 |
| `Text`             | `[string, string]`                             | `[file_name, text_format]`                       |
| `VMD`              | `string`                                       | File name                                        |
| `WSModel`          | `string`                                       | File name                                        |

Example:

```json
{ "DB": ["my_table", "units_tables", 4] }
{ "Text": ["script.lua", "Lua"] }
```

### VideoInfo

Metadata specific to video files.

| Field           | Type   | Description                           |
|-----------------|--------|---------------------------------------|
| `format`        | string | Video format enum value               |
| `version`       | number | Video format version                  |
| `codec_four_cc` | string | Codec FourCC identifier               |
| `width`         | number | Video width in pixels                 |
| `height`        | number | Video height in pixels                |
| `num_frames`    | number | Total number of frames                |
| `framerate`     | number | Frames per second                     |

<!-- langtabs-start -->
```typescript
interface VideoInfo {
  format: string;
  version: number;
  codec_four_cc: string;
  width: number;
  height: number;
  num_frames: number;
  framerate: number;
}
```
```csharp
public class VideoInfo
{
    public string Format { get; set; }
    public int Version { get; set; }
    public string CodecFourCc { get; set; }
    public int Width { get; set; }
    public int Height { get; set; }
    public int NumFrames { get; set; }
    public double Framerate { get; set; }
}
```
<!-- langtabs-end -->

### FileType

Identifies the type of a packed file. Serialized as a plain string:

| Value                | Description                      |
|----------------------|----------------------------------|
| `"Anim"`             | Animation file                   |
| `"AnimFragmentBattle"` | Battle animation fragment      |
| `"AnimPack"`         | Animation pack                   |
| `"AnimsTable"`       | Animation table                  |
| `"Atlas"`            | Sprite sheet atlas               |
| `"Audio"`            | Audio file                       |
| `"BMD"`              | Battle map data                  |
| `"BMDVegetation"`    | Battle map vegetation data       |
| `"Dat"`              | Generic data file                |
| `"DB"`               | Database table                   |
| `"ESF"`              | Empire Save Format               |
| `"Font"`             | Font file                        |
| `"GroupFormations"`  | Group formations                 |
| `"HlslCompiled"`    | Compiled HLSL shader             |
| `"Image"`            | Image file (DDS, PNG, etc.)      |
| `"Loc"`              | Localisation file                |
| `"MatchedCombat"`    | Matched combat animations        |
| `"Pack"`             | Nested Pack file                 |
| `"PortraitSettings"` | Portrait settings               |
| `"RigidModel"`       | 3D model file                   |
| `"SoundBank"`        | Sound bank                      |
| `"Text"`             | Text file (Lua, XML, JSON, etc.)|
| `"UIC"`              | UI Component                     |
| `"UnitVariant"`      | Unit variant                     |
| `"Video"`            | Video file (CA_VP8)              |
| `"VMD"`              | VMD text file                    |
| `"WSModel"`          | WSModel text file                |
| `"Unknown"`          | Unrecognized file type           |

### CompressionFormat

Compression format for Pack files. Serialized as a plain string:

| Value      | Description                |
|------------|----------------------------|
| `"None"`   | No compression             |
| `"Lz4"`    | LZ4 compression            |
| `"Zstd"`   | Zstandard compression      |

### PFHFileType

Pack file type. Serialized as a plain string (e.g. `"Mod"`, `"Movie"`, `"Boot"`, etc.).

---

## Data Types

### DecodedData

Cell data in a table. Serialized as a tagged enum — the variant name indicates the data type:

| Variant            | Payload           | Description              |
|--------------------|-------------------|--------------------------|
| `Boolean`          | boolean           | Boolean value            |
| `F32`              | number            | 32-bit float             |
| `F64`              | number            | 64-bit float             |
| `I16`              | number            | 16-bit integer           |
| `I32`              | number            | 32-bit integer           |
| `I64`              | number            | 64-bit integer           |
| `ColourRGB`        | string            | RGB colour string        |
| `StringU8`         | string            | UTF-8 string             |
| `StringU16`        | string            | UTF-16 string            |
| `OptionalI16`      | number            | Optional 16-bit integer  |
| `OptionalI32`      | number            | Optional 32-bit integer  |
| `OptionalI64`      | number            | Optional 64-bit integer  |
| `OptionalStringU8` | string            | Optional UTF-8 string    |
| `OptionalStringU16`| string            | Optional UTF-16 string   |
| `SequenceU16`      | DecodedData[][]   | Nested sequence (u16 count) |
| `SequenceU32`      | DecodedData[][]   | Nested sequence (u32 count) |

Example:

```json
{ "StringU8": "hello" }
{ "I32": 42 }
{ "Boolean": true }
```

### TableInMemory

In-memory table data structure used by DB and Loc files.

| Field              | Type              | Description                                    |
|--------------------|-------------------|------------------------------------------------|
| `table_name`       | string            | Table type identifier (e.g. `"units_tables"`)  |
| `definition`       | Definition        | Schema definition for this table               |
| `definition_patch` | DefinitionPatch   | Runtime schema modifications                   |
| `table_data`       | DecodedData[][]   | Row data (outer = rows, inner = columns)       |
| `altered`          | boolean           | Whether data was altered during decoding        |

### RFile

A raw packed file.

| Field            | Type           | Description                          |
|------------------|----------------|--------------------------------------|
| `path`           | string         | Path of the file within a container  |
| `timestamp`      | number or null | Last modified timestamp (Unix epoch) |
| `file_type`      | FileType       | Detected or specified file type      |
| `container_name` | string or null | Name of the source container         |
| `data`           | unknown        | Internal data storage                |

### RFileDecoded

Decoded file content. Serialized as a tagged enum — the variant name indicates the file type. See [Decoded File Types](#decoded-file-types) for each type's structure.

Variants: `Anim`, `AnimFragmentBattle`, `AnimPack`, `AnimsTable`, `Atlas`, `Audio`, `BMD`, `BMDVegetation`, `Dat`, `DB`, `ESF`, `Font`, `GroupFormations`, `HlslCompiled`, `Image`, `Loc`, `MatchedCombat`, `Pack`, `PortraitSettings`, `RigidModel`, `SoundBank`, `Text`, `UIC`, `UnitVariant`, `Unknown`, `Video`, `VMD`, `WSModel`.

Example:

```json
{ "DB": { "mysterious_byte": true, "guid": "", "table": { ... } } }
{ "Text": { "encoding": "Utf8Bom", "format": "Lua", "contents": "-- script" } }
```

### TableReferences

Reference data for a column, used by lookup/autocomplete features.

| Field                          | Type                   | Description                                        |
|--------------------------------|------------------------|----------------------------------------------------|
| `field_name`                   | string                 | Name of the column these references are for        |
| `referenced_table_is_ak_only`  | boolean                | Whether the referenced table only exists in the AK |
| `referenced_column_is_localised` | boolean              | Whether the referenced column is localised         |
| `data`                         | Record<string, string> | Map of actual values to their display text         |

---

## Decoded File Types

### DB

Decoded database table file.

| Field            | Type          | Description                                      |
|------------------|---------------|--------------------------------------------------|
| `mysterious_byte`| boolean       | Boolean flag (setting to 0 can crash WH2)        |
| `guid`           | string        | GUID for this table instance (empty for older games) |
| `table`          | TableInMemory | The table data including definition and rows      |

### Loc

Decoded localisation file.

| Field   | Type          | Description                                         |
|---------|---------------|-----------------------------------------------------|
| `table` | TableInMemory | Table data with key, text, and tooltip columns      |

### Text

Decoded text file.

| Field      | Type         | Description                    |
|------------|--------------|--------------------------------|
| `encoding` | TextEncoding | Character encoding of the file |
| `format`   | TextFormat   | Detected file format           |
| `contents` | string       | Decoded text contents          |

**TextEncoding** values: `"Iso8859_1"`, `"Utf8"`, `"Utf8Bom"`, `"Utf16Le"`

**TextFormat** values: `"Bat"`, `"Cpp"`, `"Html"`, `"Hlsl"`, `"Json"`, `"Js"`, `"Css"`, `"Lua"`, `"Markdown"`, `"Plain"`, `"Python"`, `"Sql"`, `"Xml"`, `"Yaml"`

### Image

Decoded image file.

| Field            | Type             | Description                                      |
|------------------|------------------|--------------------------------------------------|
| `data`           | number[]         | Original raw image data in native format         |
| `converted_data` | number[] or null | PNG-converted data for DDS textures (for viewing)|

### RigidModel

Decoded RigidModel (3D model) file.

| Field         | Type      | Description                                        |
|---------------|-----------|----------------------------------------------------|
| `version`     | number    | File format version (6, 7, or 8)                   |
| `uk_1`        | number    | Unknown field                                      |
| `skeleton_id` | string    | Skeleton identifier for animation (empty if static)|
| `lods`        | unknown[] | LOD structures from highest to lowest quality      |

### ESF

Decoded ESF (Empire Save Format) file.

| Field           | Type    | Description                         |
|-----------------|---------|-------------------------------------|
| `signature`     | string  | Format signature (CAAB, CBAB, etc.) |
| `unknown_1`     | number  | Unknown header field, typically 0   |
| `creation_date` | number  | Creation timestamp                  |
| `root_node`     | unknown | Root node of the data tree          |

### Bmd

Decoded BMD (Battle Map Data) file.

| Field                | Type    | Description                    |
|----------------------|---------|--------------------------------|
| `serialise_version`  | number  | File format version (23-27)    |
| *(other fields)*     | unknown | Complex battlefield-related data |

### AnimFragmentBattle

Decoded AnimFragmentBattle file.

| Field           | Type      | Description                            |
|-----------------|-----------|----------------------------------------|
| `version`       | number    | File format version (2 or 4)           |
| `entries`       | unknown[] | List of animation entries              |
| `skeleton_name` | string    | Name of the skeleton                   |
| `subversion`    | number    | Format subversion (version 4 only)     |

### AnimsTable

Decoded AnimsTable file.

| Field     | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `version` | number    | File format version (currently 2)  |
| `entries` | unknown[] | List of animation table entries    |

### Atlas

Decoded Atlas (sprite sheet) file.

| Field     | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `version` | number    | File format version (currently 1)  |
| `unknown` | number    | Unknown field                      |
| `entries` | unknown[] | List of sprite entries             |

### Audio

Decoded Audio file.

| Field  | Type     | Description          |
|--------|----------|----------------------|
| `data` | number[] | Raw binary audio data|

### GroupFormations

Decoded GroupFormations file.

| Field        | Type      | Description                  |
|--------------|-----------|------------------------------|
| `formations` | unknown[] | List of formation definitions|

### MatchedCombat

Decoded MatchedCombat file.

| Field     | Type      | Description                           |
|-----------|-----------|---------------------------------------|
| `version` | number    | File format version (1 or 3)          |
| `entries` | unknown[] | List of matched combat entries        |

### PortraitSettings

Decoded PortraitSettings file.

| Field     | Type      | Description                          |
|-----------|-----------|--------------------------------------|
| `version` | number    | Format version (1 or 4)              |
| `entries` | unknown[] | Portrait entries, one per art set    |

### UIC

Decoded UIC (UI Component) file.

| Field                | Type                    | Description                             |
|----------------------|-------------------------|-----------------------------------------|
| `version`            | number                  | Format version number                   |
| `source_is_xml`      | boolean                 | Whether decoded from XML (true) or binary (false) |
| `comment`            | string                  | Optional comment/description            |
| `precache_condition` | string                  | Condition for precaching                |
| `hierarchy`          | Record<string, unknown> | Tree structure of UI element relationships |
| `components`         | Record<string, unknown> | Map of component IDs to definitions     |

### UnitVariant

Decoded UnitVariant file.

| Field       | Type      | Description              |
|-------------|-----------|--------------------------|
| `version`   | number    | Version of the UnitVariant |
| `unknown_1` | number    | Unknown field            |
| `categories`| unknown[] | Variant categories       |

---

## Schema Types

### FieldType

The data type of a field in a schema definition. Most values are plain strings; sequence types wrap a nested `Definition`:

| Value              | Description                          |
|--------------------|--------------------------------------|
| `"Boolean"`        | Boolean value                        |
| `"F32"`            | 32-bit float                         |
| `"F64"`            | 64-bit float                         |
| `"I16"`            | 16-bit signed integer                |
| `"I32"`            | 32-bit signed integer                |
| `"I64"`            | 64-bit signed integer                |
| `"ColourRGB"`      | RGB colour value                     |
| `"StringU8"`       | UTF-8 string (length-prefixed u8)    |
| `"StringU16"`      | UTF-16 string (length-prefixed u16)  |
| `"OptionalI16"`    | Optional 16-bit integer              |
| `"OptionalI32"`    | Optional 32-bit integer              |
| `"OptionalI64"`    | Optional 64-bit integer              |
| `"OptionalStringU8"`  | Optional UTF-8 string             |
| `"OptionalStringU16"` | Optional UTF-16 string            |
| `{ "SequenceU16": Definition }` | Nested sequence (u16 count) |
| `{ "SequenceU32": Definition }` | Nested sequence (u32 count) |

### Field

A single field descriptor within a Definition.

| Field                    | Type                    | Description                                            |
|--------------------------|-------------------------|--------------------------------------------------------|
| `name`                   | string                  | Field name                                             |
| `field_type`             | FieldType               | Data type                                              |
| `is_key`                 | boolean                 | Part of the table's primary key                        |
| `default_value`          | string or null          | Default value for new rows                             |
| `is_filename`            | boolean                 | Whether this field contains a filename/path            |
| `filename_relative_path` | string or null          | Semicolon-separated relative paths for file lookup     |
| `is_reference`           | [string, string] or null | Foreign key: `[table_name, column_name]`              |
| `lookup`                 | string[] or null        | Additional columns to show from the referenced table   |
| `description`            | string                  | Human-readable description                             |
| `ca_order`               | number                  | Position in CA's Assembly Kit editor (-1 = unknown)    |
| `is_bitwise`             | number                  | Number of boolean columns to split this field into     |
| `enum_values`            | Record<number, string>  | Named enum values (integer key to string name)         |
| `is_part_of_colour`      | number or null          | RGB colour group index (null if not a colour field)    |

### Definition

Schema definition for a specific version of a DB table.

| Field                          | Type     | Description                                                  |
|--------------------------------|----------|--------------------------------------------------------------|
| `version`                      | number   | Version number (-1 = fake, 0 = unversioned, 1+ = versioned) |
| `fields`                       | Field[]  | Fields in binary encoding order (see note below)             |
| `localised_fields`             | Field[]  | Fields extracted to LOC files during export                  |
| `localised_key_order`          | number[] | Order of key fields for constructing localisation keys       |

> **Note:** The `fields` list is ordered to match the **binary encoding** of the table, which is not necessarily the order columns appear in the decoded/displayed data. To get fields in decoded column order (with bitwise expansion, enum conversion, and colour merging applied), use the [`FieldsProcessed`](./chapter_server_2.md#fieldsprocessed) command, passing the `Definition` as input.

### Schema

The full schema containing all table definitions for a game.

| Field         | Type                                | Description                        |
|---------------|-------------------------------------|------------------------------------|
| `version`     | number                              | Schema format version (currently 5)|
| `definitions` | Record<string, Definition[]>        | Table name to version definitions  |
| `patches`     | Record<string, DefinitionPatch>     | Table name to patches              |

### DefinitionPatch

A patch applied to a schema definition. Serialized as a nested map:

```json
{
  "field_name": {
    "property_name": "property_value"
  }
}
```

Type: `Record<string, Record<string, string>>`

---

## Pack Settings Types

### PackSettings

Per-pack configuration stored inside the Pack file.

| Field             | Type                    | Description                                    |
|-------------------|-------------------------|------------------------------------------------|
| `settings_text`   | Record<string, string>  | Multi-line text settings (e.g., ignore lists)  |
| `settings_string` | Record<string, string>  | Single-line string settings                    |
| `settings_bool`   | Record<string, boolean> | Boolean flags                                  |
| `settings_number` | Record<string, number>  | Integer settings                               |

### Note

A note attached to a path within the Pack file.

| Field     | Type           | Description                                    |
|-----------|----------------|------------------------------------------------|
| `id`      | number         | Unique note identifier                         |
| `message` | string         | Note content/body                              |
| `url`     | string or null | Optional URL associated with the note          |
| `path`    | string         | Path within the Pack (empty string = global)   |

### OptimizerOptions

Configuration for the pack optimizer.

| Field                                       | Type    | Description                                               |
|---------------------------------------------|---------|-----------------------------------------------------------|
| `pack_remove_itm_files`                     | boolean | Remove files unchanged from vanilla                       |
| `db_import_datacores_into_twad_key_deletes` | boolean | Import datacored tables into twad_key_deletes             |
| `db_optimize_datacored_tables`              | boolean | Optimize datacored tables (not recommended)               |
| `table_remove_duplicated_entries`           | boolean | Remove duplicated rows from DB and Loc files              |
| `table_remove_itm_entries`                  | boolean | Remove Identical To Master rows                           |
| `table_remove_itnr_entries`                 | boolean | Remove Identical To New Row rows                          |
| `table_remove_empty_file`                   | boolean | Remove empty DB and Loc files                             |
| `text_remove_unused_xml_map_folders`        | boolean | Remove unused XML files in map folders                    |
| `text_remove_unused_xml_prefab_folder`      | boolean | Remove unused XML files in the prefab folder              |
| `text_remove_agf_files`                     | boolean | Remove unused AGF files                                   |
| `text_remove_model_statistics_files`        | boolean | Remove unused model_statistics files                      |
| `pts_remove_unused_art_sets`                | boolean | Remove unused art sets in Portrait Settings               |
| `pts_remove_unused_variants`                | boolean | Remove unused variants from Portrait Settings art sets    |
| `pts_remove_empty_masks`                    | boolean | Remove empty masks in Portrait Settings                   |
| `pts_remove_empty_file`                     | boolean | Remove empty Portrait Settings files                      |

---

## Translation Types

### Translation

A single translation entry for a Loc key.

| Field                | Type    | Description                                        |
|----------------------|---------|----------------------------------------------------|
| `key`                | string  | The Loc key identifying this string                |
| `value_original`     | string  | Original text in the base language                 |
| `value_translated`   | string  | Translated text in the target language             |
| `needs_retranslation`| boolean | Whether the source text has changed since translation |
| `removed`            | boolean | Whether this string has been removed from the source pack |

### PackTranslation

Translation data for a pack in a specific language.

| Field          | Type                          | Description                     |
|----------------|-------------------------------|---------------------------------|
| `language`     | string                        | Target language code (e.g. `"es"`, `"de"`) |
| `pack_name`    | string                        | Name of the pack               |
| `translations` | Record<string, Translation>   | Loc key to translation data    |

---

## Diagnostics Types

### Diagnostics

Diagnostics report configuration and results.

| Field                 | Type       | Description                                          |
|-----------------------|------------|------------------------------------------------------|
| `folders_ignored`     | string[]   | Folder paths excluded from checks                    |
| `files_ignored`       | string[]   | File paths excluded from checks                      |
| `fields_ignored`      | string[]   | Table fields excluded (`"table_name/field_name"`)    |
| `diagnostics_ignored` | string[]   | Diagnostic type identifiers to skip                  |
| `results`             | unknown[]  | Diagnostic results from the most recent check        |

---

## Update Types

### APIResponse

Response from a program update check. Serialized as a tagged enum:

| Variant              | Payload  | Description                        |
|----------------------|----------|------------------------------------|
| `NewBetaUpdate`      | string   | New beta version available         |
| `NewStableUpdate`    | string   | New stable version available       |
| `NewUpdateHotfix`    | string   | New hotfix available               |
| `NoUpdate`           | *(none)* | Already up to date                 |
| `UnknownVersion`     | *(none)* | Current version could not be determined |

### GitResponse

Response from a git-based update check (schemas, translations, etc.):

| Value            | Description                              |
|------------------|------------------------------------------|
| `"NewUpdate"`    | A new update is available on the remote  |
| `"NoUpdate"`     | The local repository is up to date       |
| `"NoLocalFiles"` | No local copy exists (needs cloning)     |
| `"Diverged"`     | Local and remote branches have diverged  |

---

## Search Types

### SearchSource

Which data source to search. Serialized as a plain string:

| Value            | Description              |
|------------------|--------------------------|
| `"Pack"`         | Currently loaded pack    |
| `"ParentFiles"`  | Parent mod dependencies  |
| `"GameFiles"`    | Vanilla game files       |
| `"AssKitFiles"`  | Assembly Kit files       |

### SearchOn

Boolean flags for which file types to include in a search. Each field corresponds to a file type:

`anim`, `anim_fragment_battle`, `anim_pack`, `anims_table`, `atlas`, `audio`, `bmd`, `db`, `esf`, `group_formations`, `image`, `loc`, `matched_combat`, `pack`, `portrait_settings`, `rigid_model`, `sound_bank`, `text`, `uic`, `unit_variant`, `unknown`, `video`, `schema`

All fields are `boolean`.

### GlobalSearch

Global search configuration and results.

| Field            | Type         | Description                              |
|------------------|--------------|------------------------------------------|
| `pattern`        | string       | Text pattern or regex to search for      |
| `replace_text`   | string       | Replacement text                         |
| `case_sensitive` | boolean      | Whether the search is case-sensitive     |
| `use_regex`      | boolean      | Whether the pattern is a regular expression |
| `source`         | SearchSource | Which data source to search              |
| `search_on`      | SearchOn     | Which file types to search               |
| `matches`        | Matches      | Results from the most recent search      |
| `game_key`       | string       | Game key for the files being searched    |

### Match Types

Search results use specialized match types per file format. All match containers share the same pattern: a `path` string and a `matches` array.

**TableMatch** (used for DB and Loc files):

| Field           | Type   | Description                           |
|-----------------|--------|---------------------------------------|
| `column_name`   | string | Column where the match is             |
| `column_number` | number | Logical column index (-1 if hidden)   |
| `row_number`    | number | Row number (-1 if hidden by filter)   |
| `start`         | number | Byte offset where match starts        |
| `end`           | number | Byte offset where match ends          |
| `text`          | string | Contents of the matched cell          |

**TextMatch** (used for text files):

| Field  | Type   | Description                           |
|--------|--------|---------------------------------------|
| `row`  | number | Row of the first character            |
| `start`| number | Byte offset where match starts        |
| `end`  | number | Byte offset where match ends          |
| `text` | string | Line containing the match             |

**UnknownMatch** (used for binary/unknown files):

| Field | Type   | Description                           |
|-------|--------|---------------------------------------|
| `pos` | number | Byte position of the match            |
| `len` | number | Length of the matched pattern in bytes |

**AnimFragmentBattleMatch** (used for AnimFragmentBattle files):

| Field               | Type                                                                               | Description                          |
|---------------------|-------------------------------------------------------------------------------------|--------------------------------------|
| `skeleton_name`     | boolean                                                                             | Match is in the skeleton name        |
| `table_name`        | boolean                                                                             | Match is in the table name           |
| `mount_table_name`  | boolean                                                                             | Match is in the mount table name     |
| `unmount_table_name`| boolean                                                                             | Match is in the unmount table name   |
| `locomotion_graph`  | boolean                                                                             | Match is in the locomotion graph     |
| `entry`             | `[number, [number, boolean, boolean, boolean] or null, boolean, boolean, boolean, boolean, boolean]` or null | Entry match details |
| `start`             | number                                                                              | Byte offset where match starts       |
| `end`               | number                                                                              | Byte offset where match ends         |
| `text`              | string                                                                              | The matched text                     |

**AtlasMatch** (used for Atlas files — same structure as TableMatch):

| Field           | Type   | Description                           |
|-----------------|--------|---------------------------------------|
| `column_name`   | string | Column where the match is             |
| `column_number` | number | Logical column index                  |
| `row_number`    | number | Row number of the match               |
| `start`         | number | Byte offset where match starts        |
| `end`           | number | Byte offset where match ends          |
| `text`          | string | Contents of the matched cell          |

**PortraitSettingsMatch** (used for PortraitSettings files):

| Field                   | Type                                                   | Description                          |
|-------------------------|--------------------------------------------------------|--------------------------------------|
| `entry`                 | number                                                 | Index of the entry                   |
| `id`                    | boolean                                                | Match is in the id field             |
| `camera_settings_head`  | boolean                                                | Match is in head camera skeleton node|
| `camera_settings_body`  | boolean                                                | Match is in body camera skeleton node|
| `variant`               | `[number, boolean, boolean, boolean, boolean, boolean]` or null | Variant match details     |
| `start`                 | number                                                 | Byte offset where match starts       |
| `end`                   | number                                                 | Byte offset where match ends         |
| `text`                  | string                                                 | The matched text                     |

**RigidModelMatch** (used for RigidModel files):

| Field                    | Type                 | Description                              |
|--------------------------|----------------------|------------------------------------------|
| `skeleton_id`            | boolean              | Match is in the skeleton id              |
| `mesh_value`             | `[number, number]` or null | LOD and mesh index, or null        |
| `mesh_name`              | boolean              | Match is in the mesh name                |
| `mesh_mat_name`          | boolean              | Match is in the material name            |
| `mesh_textute_directory` | boolean              | Match is in the texture directory        |
| `mesh_filters`           | boolean              | Match is in the mesh filters             |
| `mesh_att_point_name`    | number or null       | Attachment point index with match        |
| `mesh_texture_path`      | number or null       | Texture path index with match            |
| `start`                  | number               | Byte offset where match starts           |
| `end`                    | number               | Byte offset where match ends             |
| `text`                   | string               | The matched text                         |

**UnitVariantMatch** (used for UnitVariant files):

| Field    | Type                                    | Description                    |
|----------|-----------------------------------------|--------------------------------|
| `entry`  | number                                  | Index of the entry             |
| `name`   | boolean                                 | Match is in the name           |
| `variant`| `[number, boolean, boolean]` or null    | Variant match details          |
| `start`  | number                                  | Byte offset where match starts |
| `end`    | number                                  | Byte offset where match ends   |
| `text`   | string                                  | The matched text               |

**SchemaMatch** (used for schema searches):

| Field         | Type   | Description                       |
|---------------|--------|-----------------------------------|
| `table_name`  | string | The table name                    |
| `version`     | number | Version of the matched definition |
| `column`      | number | Column index of the match         |
| `column_name` | string | Full name of the matched column   |

All match container types (e.g. `TableMatches`, `TextMatches`, `AnimFragmentBattleMatches`, etc.) share the same structure: `{ path: string, matches: <MatchType>[] }`.

### MatchHolder

A tagged enum wrapping a single file type's matches. The variant name indicates the file type:

```json
{ "Db": { "path": "db/units_tables/data", "matches": [...] } }
{ "Text": { "path": "script/campaign/mod.lua", "matches": [...] } }
```

Variants: `Anim`, `AnimFragmentBattle`, `AnimPack`, `AnimsTable`, `Atlas`, `Audio`, `Bmd`, `Db`, `Esf`, `GroupFormations`, `Image`, `Loc`, `MatchedCombat`, `Pack`, `PortraitSettings`, `RigidModel`, `SoundBank`, `Text`, `Uic`, `UnitVariant`, `Unknown`, `Video`, `Schema`.
