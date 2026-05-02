# Responses

This page documents all possible responses from the RPFM server. Each response is wrapped in a [Message](./overview.md#message-format) with the same `id` as the originating command.

Responses follow the same [serialization convention](./overview.md#serialization-convention) as commands:

- Unit responses are plain strings: `"Success"`
- Responses with data use a named wrapper: `{ "Bool": true }`, `{ "Error": "File not found" }`

```json
{ "id": 1, "data": "Success" }
{ "id": 2, "data": { "Error": "File not found" } }
{ "id": 3, "data": { "ContainerInfoVecRFileInfo": [{ ... }, [{ ... }]] } }
```

---

## Generic Responses

| Response           | Payload  | Description                              |
|--------------------|----------|------------------------------------------|
| `Success`          | *(none)* | Operation completed successfully         |
| `Error`            | string   | Human-readable error message             |
| `SessionConnected` | number   | Session ID (unsolicited, sent on connect with id=0) |
| `Unknown`          | *(none)* | Returned for unsupported/unrecognized file types |

---

## File-Type Decoded Responses

These are returned by `DecodePackedFile`. Each carries a tuple of `[DecodedData, RFileInfo]`:

| Response                      | Payload Type                        |
|-------------------------------|-------------------------------------|
| `AnimFragmentBattleRFileInfo` | `[AnimFragmentBattle, RFileInfo]`   |
| `AnimPackRFileInfo`           | `[RFileInfo[], RFileInfo]`          |
| `AnimsTableRFileInfo`         | `[AnimsTable, RFileInfo]`           |
| `AtlasRFileInfo`              | `[Atlas, RFileInfo]`                |
| `AudioRFileInfo`              | `[Audio, RFileInfo]`                |
| `BmdRFileInfo`                | `[Bmd, RFileInfo]`                  |
| `DBRFileInfo`                 | `[DB, RFileInfo]`                   |
| `ESFRFileInfo`                | `[ESF, RFileInfo]`                  |
| `GroupFormationsRFileInfo`    | `[GroupFormations, RFileInfo]`       |
| `ImageRFileInfo`              | `[Image, RFileInfo]`                |
| `LocRFileInfo`                | `[Loc, RFileInfo]`                  |
| `MatchedCombatRFileInfo`      | `[MatchedCombat, RFileInfo]`        |
| `PortraitSettingsRFileInfo`   | `[PortraitSettings, RFileInfo]`     |
| `RigidModelRFileInfo`         | `[RigidModel, RFileInfo]`           |
| `TextRFileInfo`               | `[Text, RFileInfo]`                 |
| `UICRFileInfo`                | `[UIC, RFileInfo]`                  |
| `UnitVariantRFileInfo`        | `[UnitVariant, RFileInfo]`          |
| `VideoInfoRFileInfo`          | `[VideoInfo, RFileInfo]`            |
| `VMDRFileInfo`                | `[Text, RFileInfo]`                 |
| `WSModelRFileInfo`            | `[Text, RFileInfo]`                 |

Example:
```json
{
  "id": 5,
  "data": {
    "DBRFileInfo": [
      { "mysterious_byte": true, "guid": "", "table": { ... } },
      { "path": "db/units_tables/data", "container_name": "my_mod.pack", "timestamp": null, "file_type": "DB" }
    ]
  }
}
```

---

## Scalar Responses

| Response   | Payload Type      | Description                |
|------------|-------------------|----------------------------|
| `Bool`     | boolean           | Boolean value              |
| `F32`      | number            | 32-bit float               |
| `I32`      | number            | 32-bit integer             |
| `I32I32`   | [number, number]  | Pair of integers           |
| `String`   | string            | String value               |
| `PathBuf`  | string            | Filesystem path            |

---

## Collection Responses

| Response                        | Payload Type                         | Description                        |
|---------------------------------|--------------------------------------|------------------------------------|
| `VecBoolString`                 | [boolean, string][]                  | Boolean-string pairs               |
| `VecContainerPath`              | ContainerPath[]                      | List of container paths            |
| `VecContainerPathContainerPath` | [ContainerPath, ContainerPath][]     | Pairs of container paths (renames) |
| `VecContainerPathOptionString`  | [ContainerPath[], string or null]    | Paths with optional error message  |
| `VecContainerPathVecContainerPath` | [ContainerPath[], ContainerPath[]]| Two lists of paths (added, deleted)|
| `VecContainerPathVecContainerPathString` | [ContainerPath[], ContainerPath[], string] | Paste result: added paths, cut-deleted paths, source pack key |
| `VecContainerPathBTreeMapStringVecContainerPath` | [ContainerPath[], Record<string, ContainerPath[]>] | Paths + grouping by pack key |
| `VecContainerPathVecRFileInfo`  | [ContainerPath[], RFileInfo[]]       | Paths and file info                |
| `VecContainerPathVecString`     | [ContainerPath[], string[]]          | Paths and string list              |
| `VecDataSourceStringStringStringUsizeUsize` | [DataSource, string, string, string, number, number][] | Reference search results: `(data_source, pack_key, path, column_name, column_number, row_number)`. `pack_key` is empty for non-`PackFile` sources. |
| `VecDefinition`                 | Definition[]                         | List of definitions                |
| `VecField`                      | Field[]                              | List of fields                     |
| `VecNote`                       | Note[]                               | List of notes                      |
| `VecRFile`                      | RFile[]                              | List of raw files                  |
| `VecRFileInfo`                  | RFileInfo[]                          | List of file metadata              |
| `VecString`                     | string[]                             | List of strings                    |
| `VecStringContainerInfo`        | [string, ContainerInfo][]            | Pack key + metadata pairs          |
| `VecU8`                         | number[]                             | Raw byte data                      |
| `HashSetString`                 | string[]                             | Set of strings                     |
| `HashSetStringHashSetString`    | [string[], string[]]                 | Two sets of strings                |

---

## Compound Responses

| Response                              | Payload Type                                                                 | Description                           |
|---------------------------------------|------------------------------------------------------------------------------|---------------------------------------|
| `APIResponse`                         | APIResponse                                                                  | Program update check result           |
| `APIResponseGit`                      | GitResponse                                                                  | Git update check result               |
| `CompressionFormat`                   | CompressionFormat                                                            | Pack compression format               |
| `CompressionFormatDependenciesInfo`   | [CompressionFormat, DependenciesInfo or null]                                | Format + optional dependencies info   |
| `ContainerInfo`                       | ContainerInfo                                                                | Pack metadata                         |
| `ContainerInfoVecRFileInfo`           | [ContainerInfo, RFileInfo[]]                                                 | Pack metadata + file list             |
| `StringContainerInfo`                 | [string, ContainerInfo]                                                      | Pack key + metadata                   |
| `DataSourceStringUsizeUsize`          | [DataSource, string, number, number]                                         | Navigation result                     |
| `Definition`                          | Definition                                                                   | Table definition                      |
| `DependenciesInfo`                    | DependenciesInfo                                                             | Dependencies information              |
| `Diagnostics`                         | Diagnostics                                                                  | Diagnostics report                    |
| `GlobalSearchVecRFileInfo`            | [GlobalSearch, RFileInfo[]]                                                  | Search results + modified files       |
| `HashMapDataSourceHashMapStringRFile` | Record<DataSource, Record<string, RFile>>                                    | Files by source and path              |
| `HashMapDataSourceHashSetContainerPath` | Record<DataSource, ContainerPath[]>                                        | Paths by data source                  |
| `HashMapI32TableReferences`           | Record<number, TableReferences>                                              | Column references by index            |
| `HashMapStringHashMapStringVecString` | Record<string, Record<string, string[]>>                                     | Nested string maps                    |
| `I32I32VecStringVecString`            | [number, number, string[], string[]]                                         | Version change result                 |
| `Note`                                | Note                                                                         | Single note                           |
| `OperationalMode`                     | OperationalMode                                                              | Per-pack operational mode             |
| `OptimizerOptions`                    | OptimizerOptions                                                             | Optimizer configuration               |
| `OptionContainerPath`                 | ContainerPath or null                                                        | Optional container path               |
| `OptionRFileInfo`                     | RFileInfo or null                                                            | Optional file info                    |
| `OptionStringStringVecString`         | [string, string, string[]] or null                                           | Optional loc source data              |
| `PackSettings`                        | PackSettings                                                                 | Pack settings                         |
| `PackTranslation`                     | PackTranslation                                                              | Translation data                      |
| `RFileDecoded`                        | RFileDecoded                                                                 | Decoded file content                  |
| `Schema`                              | Schema                                                                       | Full schema                           |
| `StringVecContainerPath`              | [string, ContainerPath[]]                                                    | String + path list                    |
| `StringVecPathBuf`                    | [string, string[]]                                                           | String + filesystem paths             |
| `Text`                                | Text                                                                         | Text file content                     |

---

## Batch Settings Response

### SettingsAll

Returned by `SettingsGetAll`. Wraps a [`SettingsSnapshot`](./ws-shared-types.md#settingssnapshot) — a struct with six maps, one per primitive setting type:

```json
{
  "SettingsAll": {
    "bool":       { "enable_telemetry": true },
    "i32":        { "autosave_interval": 300 },
    "f32":        { "ui_scale": 1.0 },
    "string":     { "game_selected": "warhammer_3" },
    "raw_data":   { "some_blob_setting": [0, 1, 2] },
    "vec_string": { "favourite_mods": ["a.pack", "b.pack"] }
  }
}
```

See [`SettingsSnapshot`](./ws-shared-types.md#settingssnapshot) for the full field list.
