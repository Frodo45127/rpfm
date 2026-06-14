# Commands

This page documents all commands that can be sent to the RPFM server. Each command is wrapped in a [Message](./overview.md#message-format) with a unique `id`.

Commands with no parameters are serialized as plain strings. Commands with parameters use the `{ "CommandName": params }` format. See the [serialization convention](./overview.md#serialization-convention) for details.

Most commands that operate on a specific pack take a `pack_key` string as their first parameter. The pack key is returned by `OpenPackFiles` or `NewPack` when you open or create a pack.

---

## Lifecycle

### Exit

Close the background thread. **Do not use directly** — the server manages this internally.

Response: None (breaks the server loop).

<!-- langtabs-start -->
```typescript
type ExitRequest = "Exit";
// No response — the server loop terminates.
```
```csharp
// Request: send the literal string "Exit". No response.
```
<!-- langtabs-end -->

### ClientDisconnecting

Signal that the client is intentionally disconnecting. Allows the server to clean up the session immediately.

Response: `"Success"`

```json
{ "id": 1, "data": "ClientDisconnecting" }
```

<!-- langtabs-start -->
```typescript
type ClientDisconnectingRequest = "ClientDisconnecting";
type ClientDisconnectingResponse = "Success";
```
```csharp
// Request: send the literal string "ClientDisconnecting".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## PackFile Operations

### NewPack

Create a new empty Pack.

Response: `{ String: string }` — the assigned pack key.

```json
{ "id": 1, "data": "NewPack" }
```

<!-- langtabs-start -->
```typescript
type NewPackRequest = "NewPack";
type NewPackResponse = { String: string };
```
```csharp
// Request: send the literal string "NewPack".
public class NewPackResponse
{
    public string String { get; set; }
}
```
<!-- langtabs-end -->

### OpenPackFiles

Open one or more Pack files and merge them into the current session.

| Parameter | Type     | Description            |
|-----------|----------|------------------------|
| `paths`   | string[] | Filesystem paths to open |

Response: `{ StringContainerInfo: [string, ContainerInfo] }` — pack key and metadata.

```json
{ "id": 1, "data": { "OpenPackFiles": ["/path/to/my_mod.pack"] } }
```

<!-- langtabs-start -->
```typescript
type OpenPackFilesRequest = { OpenPackFiles: string[] };
type OpenPackFilesResponse = { StringContainerInfo: [string, ContainerInfo] };
```
```csharp
public class OpenPackFilesRequest
{
    public List<string> OpenPackFiles { get; set; }
}
public class OpenPackFilesResponse
{
    public Tuple<string, ContainerInfo> StringContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

### LoadAllCAPackFiles

Open all CA Pack files for the selected game as one merged Pack.

Response: `{ StringContainerInfo: [string, ContainerInfo] }`

```json
{ "id": 1, "data": "LoadAllCAPackFiles" }
```

<!-- langtabs-start -->
```typescript
type LoadAllCAPackFilesRequest = "LoadAllCAPackFiles";
type LoadAllCAPackFilesResponse = { StringContainerInfo: [string, ContainerInfo] };
```
```csharp
// Request: send the literal string "LoadAllCAPackFiles".
public class LoadAllCAPackFilesResponse
{
    public Tuple<string, ContainerInfo> StringContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

### ListOpenPacks

List all currently open packs with their keys and metadata.

Response: `{ VecStringContainerInfo: [string, ContainerInfo][] }`

```json
{ "id": 1, "data": "ListOpenPacks" }
```

<!-- langtabs-start -->
```typescript
type ListOpenPacksRequest = "ListOpenPacks";
type ListOpenPacksResponse = { VecStringContainerInfo: [string, ContainerInfo][] };
```
```csharp
// Request: send the literal string "ListOpenPacks".
public class ListOpenPacksResponse
{
    public List<Tuple<string, ContainerInfo>> VecStringContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

### ClosePack

Close a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to close |

Response: `"Success"`

```json
{ "id": 1, "data": { "ClosePack": "my_mod.pack" } }
```

<!-- langtabs-start -->
```typescript
type ClosePackRequest = { ClosePack: string };
type ClosePackResponse = "Success";
```
```csharp
public class ClosePackRequest
{
    public string ClosePack { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### CloseAllPacks

Close all currently open Packs.

Response: `"Success"`

```json
{ "id": 1, "data": "CloseAllPacks" }
```

<!-- langtabs-start -->
```typescript
type CloseAllPacksRequest = "CloseAllPacks";
type CloseAllPacksResponse = "Success";
```
```csharp
// Request: send the literal string "CloseAllPacks".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### SavePack

Save a specific open Pack to disk.

| Parameter  | Type   | Description  |
|------------|--------|--------------|
| `pack_key` | string | Pack to save |

Response: `{ ContainerInfo: ContainerInfo }`

```json
{ "id": 1, "data": { "SavePack": "my_mod.pack" } }
```

<!-- langtabs-start -->
```typescript
type SavePackRequest = { SavePack: string };
type SavePackResponse = { ContainerInfo: ContainerInfo };
```
```csharp
public class SavePackRequest
{
    public string SavePack { get; set; }
}
public class SavePackResponse
{
    public ContainerInfo ContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type SavePackAsRequest = { SavePackAs: [string, string] };
type SavePackAsResponse = { ContainerInfo: ContainerInfo };
```
```csharp
public class SavePackAsRequest
{
    public Tuple<string, string> SavePackAs { get; set; }
}
public class SavePackAsResponse
{
    public ContainerInfo ContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type CleanAndSavePackAsRequest = { CleanAndSavePackAs: [string, string] };
type CleanAndSavePackAsResponse = { ContainerInfo: ContainerInfo };
```
```csharp
public class CleanAndSavePackAsRequest
{
    public Tuple<string, string> CleanAndSavePackAs { get; set; }
}
public class CleanAndSavePackAsResponse
{
    public ContainerInfo ContainerInfo { get; set; }
}
```
<!-- langtabs-end -->

### GetPackFileDataForTreeView

Get tree view data (container info and file list) for a specific pack.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to query  |

Response: `{ ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }`

```json
{ "id": 1, "data": { "GetPackFileDataForTreeView": "my_mod.pack" } }
```

<!-- langtabs-start -->
```typescript
type GetPackFileDataForTreeViewRequest = { GetPackFileDataForTreeView: string };
type GetPackFileDataForTreeViewResponse = { ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] };
```
```csharp
public class GetPackFileDataForTreeViewRequest
{
    public string GetPackFileDataForTreeView { get; set; }
}
public class GetPackFileDataForTreeViewResponse
{
    public Tuple<ContainerInfo, List<RFileInfo>> ContainerInfoVecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->


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

<!-- langtabs-start -->
```typescript
type GetPackedFilesInfoRequest = { GetPackedFilesInfo: [string, string[]] };
type GetPackedFilesInfoResponse = { VecRFileInfo: RFileInfo[] };
```
```csharp
public class GetPackedFilesInfoRequest
{
    public Tuple<string, List<string>> GetPackedFilesInfo { get; set; }
}
public class GetPackedFilesInfoResponse
{
    public List<RFileInfo> VecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type GetRFileInfoRequest = { GetRFileInfo: [string, string] };
type GetRFileInfoResponse = { OptionRFileInfo: RFileInfo | null };
```
```csharp
public class GetRFileInfoRequest
{
    public Tuple<string, string> GetRFileInfo { get; set; }
}
public class GetRFileInfoResponse
{
    public RFileInfo? OptionRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

### GetPackFilePath

Get the filesystem path of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ PathBuf: string }`

<!-- langtabs-start -->
```typescript
type GetPackFilePathRequest = { GetPackFilePath: string };
type GetPackFilePathResponse = { PathBuf: string };
```
```csharp
public class GetPackFilePathRequest
{
    public string GetPackFilePath { get; set; }
}
public class GetPackFilePathResponse
{
    public string PathBuf { get; set; }
}
```
<!-- langtabs-end -->

### GetPackFileName

Get the file name of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ String: string }`

<!-- langtabs-start -->
```typescript
type GetPackFileNameRequest = { GetPackFileName: string };
type GetPackFileNameResponse = { String: string };
```
```csharp
public class GetPackFileNameRequest
{
    public string GetPackFileName { get; set; }
}
public class GetPackFileNameResponse
{
    public string String { get; set; }
}
```
<!-- langtabs-end -->

### SetPackFileType

Change the PFH type of a specific open Pack (e.g., Mod, Movie, Boot).

| Parameter  | Type        | Description      |
|------------|-------------|------------------|
| `pack_key` | string      | Pack to modify   |
| `type`     | PFHFileType | New file type    |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SetPackFileTypeRequest = { SetPackFileType: [string, PFHFileType] };
type SetPackFileTypeResponse = "Success";
```
```csharp
public class SetPackFileTypeRequest
{
    public Tuple<string, string> SetPackFileType { get; set; }  // second is PFHFileType enum serialized as string
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ChangeIndexIncludesTimestamp

Toggle the "Index Includes Timestamp" flag for a specific pack.

| Parameter  | Type    | Description   |
|------------|---------|---------------|
| `pack_key` | string  | Pack to modify|
| `enabled`  | boolean | New flag value|

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type ChangeIndexIncludesTimestampRequest = { ChangeIndexIncludesTimestamp: [string, boolean] };
type ChangeIndexIncludesTimestampResponse = "Success";
```
```csharp
public class ChangeIndexIncludesTimestampRequest
{
    public Tuple<string, bool> ChangeIndexIncludesTimestamp { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ChangeCompressionFormat

Change the compression format of a specific open Pack.

| Parameter  | Type              | Description       |
|------------|-------------------|-------------------|
| `pack_key` | string            | Pack to modify    |
| `format`   | CompressionFormat | New format        |

Response: `{ CompressionFormat: CompressionFormat }` — actual format set (may differ if unsupported).

<!-- langtabs-start -->
```typescript
type ChangeCompressionFormatRequest = { ChangeCompressionFormat: [string, CompressionFormat] };
type ChangeCompressionFormatResponse = { CompressionFormat: CompressionFormat };
```
```csharp
public class ChangeCompressionFormatRequest
{
    public Tuple<string, string> ChangeCompressionFormat { get; set; }  // second is CompressionFormat enum as string
}
public class ChangeCompressionFormatResponse
{
    public string CompressionFormat { get; set; }
}
```
<!-- langtabs-end -->

### OptimizePackFile

Run the optimizer over a specific open Pack.

| Parameter | Type             | Description         |
|-----------|------------------|---------------------|
| `pack_key`| string           | Pack to optimize    |
| `options` | OptimizerOptions | Optimization config |

Response: `{ HashSetStringHashSetString: [string[], string[]] }` — deleted and added paths.

<!-- langtabs-start -->
```typescript
type OptimizePackFileRequest = { OptimizePackFile: [string, OptimizerOptions] };
type OptimizePackFileResponse = { HashSetStringHashSetString: [string[], string[]] };
```
```csharp
public class OptimizePackFileRequest
{
    public Tuple<string, OptimizerOptions> OptimizePackFile { get; set; }
}
public class OptimizePackFileResponse
{
    public Tuple<HashSet<string>, HashSet<string>> HashSetStringHashSetString { get; set; }
}
```
<!-- langtabs-end -->

### PatchSiegeAI

Patch Siege AI for Warhammer siege maps in a specific pack.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to patch  |

Response: `{ StringVecContainerPath: [string, ContainerPath[]] }`

<!-- langtabs-start -->
```typescript
type PatchSiegeAIRequest = { PatchSiegeAI: string };
type PatchSiegeAIResponse = { StringVecContainerPath: [string, ContainerPath[]] };
```
```csharp
public class PatchSiegeAIRequest
{
    public string PatchSiegeAI { get; set; }
}
public class PatchSiegeAIResponse
{
    public Tuple<string, List<ContainerPath>> StringVecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

---

## Game Selection

### GetGameSelected

Get the currently selected game key.

Response: `{ String: string }`

```json
{ "id": 1, "data": "GetGameSelected" }
```

<!-- langtabs-start -->
```typescript
type GetGameSelectedRequest = "GetGameSelected";
type GetGameSelectedResponse = { String: string };
```
```csharp
// Request: send the literal string "GetGameSelected".
public class GetGameSelectedResponse
{
    public string String { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type SetGameSelectedRequest = { SetGameSelected: [string, boolean] };
type SetGameSelectedResponse = {
  CompressionFormatDependenciesInfo: [CompressionFormat, DependenciesInfo | null]
};
```
```csharp
public class SetGameSelectedRequest
{
    public Tuple<string, bool> SetGameSelected { get; set; }
}
public class SetGameSelectedResponse
{
    public Tuple<string, DependenciesInfo?> CompressionFormatDependenciesInfo { get; set; }
}
```
<!-- langtabs-end -->

### GenerateDependenciesCache

Generate the dependencies cache for the currently selected game.

Response: `{ DependenciesInfo: DependenciesInfo }`

<!-- langtabs-start -->
```typescript
type GenerateDependenciesCacheRequest = "GenerateDependenciesCache";
type GenerateDependenciesCacheResponse = { DependenciesInfo: DependenciesInfo };
```
```csharp
// Request: send the literal string "GenerateDependenciesCache".
public class GenerateDependenciesCacheResponse
{
    public DependenciesInfo DependenciesInfo { get; set; }
}
```
<!-- langtabs-end -->

### UpdateCurrentSchemaFromAssKit

Update the current schema with data from the game's Assembly Kit.

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type UpdateCurrentSchemaFromAssKitRequest = "UpdateCurrentSchemaFromAssKit";
type UpdateCurrentSchemaFromAssKitResponse = "Success";
```
```csharp
// Request: send the literal string "UpdateCurrentSchemaFromAssKit".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type NewPackedFileRequest = { NewPackedFile: [string, string, NewFile] };
type NewPackedFileResponse = "Success";
```
```csharp
public class NewPackedFileRequest
{
    public Tuple<string, string, NewFile> NewPackedFile { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### AddPackedFiles

Add files from the filesystem to a specific open Pack.

| Parameter      | Type                    | Description                         |
|----------------|-------------------------|-------------------------------------|
| `pack_key`     | string                  | Target pack                         |
| `source_paths` | string[]                | Filesystem paths to add             |
| `dest_paths`   | ContainerPath[]         | Destination paths inside the pack   |
| `ignore_paths` | string[] or null        | Paths to exclude (optional)         |

Response: `{ VecContainerPathOptionString: [ContainerPath[], string | null] }` — added paths and optional error.

<!-- langtabs-start -->
```typescript
type AddPackedFilesRequest = {
  AddPackedFiles: [string, string[], ContainerPath[], string[] | null]
};
type AddPackedFilesResponse = {
  VecContainerPathOptionString: [ContainerPath[], string | null]
};
```
```csharp
public class AddPackedFilesRequest
{
    public Tuple<string, List<string>, List<ContainerPath>, List<string>?> AddPackedFiles { get; set; }
}
public class AddPackedFilesResponse
{
    public Tuple<List<ContainerPath>, string?> VecContainerPathOptionString { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type DecodePackedFileRequest = { DecodePackedFile: [string, string, DataSource] };
// Response is one of many variants depending on the file type:
type DecodePackedFileResponse =
  | { AnimFragmentBattleRFileInfo: [AnimFragmentBattle, RFileInfo] }
  | { AnimPackRFileInfo: [RFileInfo[], RFileInfo] }
  | { AnimsTableRFileInfo: [AnimsTable, RFileInfo] }
  | { AtlasRFileInfo: [Atlas, RFileInfo] }
  | { AudioRFileInfo: [Audio, RFileInfo] }
  | { BmdRFileInfo: [Bmd, RFileInfo] }
  | { DBRFileInfo: [DB, RFileInfo] }
  | { ESFRFileInfo: [ESF, RFileInfo] }
  | { GroupFormationsRFileInfo: [GroupFormations, RFileInfo] }
  | { ImageRFileInfo: [Image, RFileInfo] }
  | { LocRFileInfo: [Loc, RFileInfo] }
  | { MatchedCombatRFileInfo: [MatchedCombat, RFileInfo] }
  | { PortraitSettingsRFileInfo: [PortraitSettings, RFileInfo] }
  | { RigidModelRFileInfo: [RigidModel, RFileInfo] }
  | { TextRFileInfo: [Text, RFileInfo] }
  | { UICRFileInfo: [UIC, RFileInfo] }
  | { UnitVariantRFileInfo: [UnitVariant, RFileInfo] }
  | { VideoInfoRFileInfo: [VideoInfo, RFileInfo] }
  | { VMDRFileInfo: [Text, RFileInfo] }
  | { WSModelRFileInfo: [Text, RFileInfo] }
  | { Text: Text }
  | "Unknown";
```
```csharp
public class DecodePackedFileRequest
{
    public Tuple<string, string, string> DecodePackedFile { get; set; }  // third is DataSource enum
}
// Response: deserialize into a class with nullable properties for each variant,
// or branch on the first JSON key. Example:
public class DecodePackedFileResponse
{
    public Tuple<DB, RFileInfo>? DBRFileInfo { get; set; }
    public Tuple<Text, RFileInfo>? TextRFileInfo { get; set; }
    public Tuple<Loc, RFileInfo>? LocRFileInfo { get; set; }
    // ... one property per decoded variant above.
}
```
<!-- langtabs-end -->

### SavePackedFileFromView

Save an edited packed file back to the Pack.

| Parameter | Type         | Description              |
|-----------|--------------|--------------------------|
| `pack_key`| string       | Target pack              |
| `path`    | string       | Internal path            |
| `data`    | RFileDecoded | Decoded file content     |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SavePackedFileFromViewRequest = { SavePackedFileFromView: [string, string, RFileDecoded] };
type SavePackedFileFromViewResponse = "Success";
```
```csharp
public class SavePackedFileFromViewRequest
{
    public Tuple<string, string, RFileDecoded> SavePackedFileFromView { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type DeletePackedFilesRequest = { DeletePackedFiles: [string, ContainerPath[]] };
type DeletePackedFilesResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class DeletePackedFilesRequest
{
    public Tuple<string, List<ContainerPath>> DeletePackedFiles { get; set; }
}
public class DeletePackedFilesResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### CopyPackedFiles

Copy one or more packed files to the server-side clipboard, so they can later be pasted into the same or a different pack with `PastePackedFiles`. Path references only — nothing is duplicated until paste.

| Parameter | Type                                        | Description                              |
|-----------|---------------------------------------------|------------------------------------------|
| `sources` | Record<string, ContainerPath[]>             | Map of pack key to paths to copy from it |

Response: `"Success"`

```json
{ "id": 1, "data": { "CopyPackedFiles": { "my_mod.pack": [{ "File": "db/units_tables/data" }] } } }
```

<!-- langtabs-start -->
```typescript
type CopyPackedFilesRequest = { CopyPackedFiles: Record<string, ContainerPath[]> };
type CopyPackedFilesResponse = "Success";
```
```csharp
public class CopyPackedFilesRequest
{
    public Dictionary<string, List<ContainerPath>> CopyPackedFiles { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### CutPackedFiles

Same as `CopyPackedFiles`, but the source files will be deleted from their originating pack once `PastePackedFiles` runs.

| Parameter | Type                                        | Description                             |
|-----------|---------------------------------------------|-----------------------------------------|
| `sources` | Record<string, ContainerPath[]>             | Map of pack key to paths to cut from it |

Response: `"Success"`

```json
{ "id": 1, "data": { "CutPackedFiles": { "my_mod.pack": [{ "File": "db/units_tables/data" }] } } }
```

<!-- langtabs-start -->
```typescript
type CutPackedFilesRequest = { CutPackedFiles: Record<string, ContainerPath[]> };
type CutPackedFilesResponse = "Success";
```
```csharp
public class CutPackedFilesRequest
{
    public Dictionary<string, List<ContainerPath>> CutPackedFiles { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### PastePackedFiles

Paste packed files from the internal clipboard into a pack. Works for both copied and cut files — on a cut paste, the source files are removed as part of this call.

| Parameter    | Type   | Description                               |
|--------------|--------|-------------------------------------------|
| `pack_key`   | string | Target pack                               |
| `dest_path`  | string | Destination folder path inside the target |

Response: `{ VecContainerPathVecContainerPathString: [ContainerPath[], ContainerPath[], string] }` — added paths, cut-deleted paths, source pack key.

```json
{ "id": 1, "data": { "PastePackedFiles": ["my_mod.pack", "db/units_tables/"] } }
```

<!-- langtabs-start -->
```typescript
type PastePackedFilesRequest = { PastePackedFiles: [string, string] };
type PastePackedFilesResponse = {
  VecContainerPathVecContainerPathString: [ContainerPath[], ContainerPath[], string]
};
```
```csharp
public class PastePackedFilesRequest
{
    public Tuple<string, string> PastePackedFiles { get; set; }
}
public class PastePackedFilesResponse
{
    public Tuple<List<ContainerPath>, List<ContainerPath>, string> VecContainerPathVecContainerPathString { get; set; }
}
```
<!-- langtabs-end -->

### DuplicatePackedFiles

Duplicate one or more packed files in place within the same pack. Each file is cloned with a numeric suffix appended to avoid name collisions.

| Parameter | Type            | Description                |
|-----------|-----------------|----------------------------|
| `pack_key`| string          | Pack to modify             |
| `paths`   | ContainerPath[] | Paths to duplicate         |

Response: `{ VecContainerPath: ContainerPath[] }` — new duplicated paths.

```json
{ "id": 1, "data": { "DuplicatePackedFiles": ["my_mod.pack", [{ "File": "db/units_tables/data" }]] } }
```

<!-- langtabs-start -->
```typescript
type DuplicatePackedFilesRequest = { DuplicatePackedFiles: [string, ContainerPath[]] };
type DuplicatePackedFilesResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class DuplicatePackedFilesRequest
{
    public Tuple<string, List<ContainerPath>> DuplicatePackedFiles { get; set; }
}
public class DuplicatePackedFilesResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->


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

<!-- langtabs-start -->
```typescript
type ExtractPackedFilesRequest = {
  ExtractPackedFiles: [string, Record<DataSource, ContainerPath[]>, string, boolean]
};
type ExtractPackedFilesResponse = { StringVecPathBuf: [string, string[]] };
```
```csharp
public class ExtractPackedFilesRequest
{
    public Tuple<string, Dictionary<string, List<ContainerPath>>, string, bool> ExtractPackedFiles { get; set; }
}
public class ExtractPackedFilesResponse
{
    public Tuple<string, List<string>> StringVecPathBuf { get; set; }
}
```
<!-- langtabs-end -->

### RenamePackedFiles

Rename packed files in a specific Pack.

| Parameter | Type                                | Description                              |
|-----------|-------------------------------------|------------------------------------------|
| `pack_key`| string                              | Pack to modify                           |
| `renames` | [ContainerPath, ContainerPath][]    | Array of `[old_path, new_path]` pairs    |

Response: `{ VecContainerPathContainerPath: [ContainerPath, ContainerPath][] }`

<!-- langtabs-start -->
```typescript
type RenamePackedFilesRequest = {
  RenamePackedFiles: [string, [ContainerPath, ContainerPath][]]
};
type RenamePackedFilesResponse = {
  VecContainerPathContainerPath: [ContainerPath, ContainerPath][]
};
```
```csharp
public class RenamePackedFilesRequest
{
    public Tuple<string, List<Tuple<ContainerPath, ContainerPath>>> RenamePackedFiles { get; set; }
}
public class RenamePackedFilesResponse
{
    public List<Tuple<ContainerPath, ContainerPath>> VecContainerPathContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### FolderExists

Check if a folder exists in a specific open Pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to check |
| `path`    | string | Folder path   |

Response: `{ Bool: boolean }`

<!-- langtabs-start -->
```typescript
type FolderExistsRequest = { FolderExists: [string, string] };
type FolderExistsResponse = { Bool: boolean };
```
```csharp
public class FolderExistsRequest
{
    public Tuple<string, string> FolderExists { get; set; }
}
public class FolderExistsResponse
{
    public bool Bool { get; set; }
}
```
<!-- langtabs-end -->

### PackedFileExists

Check if a packed file exists in a specific open Pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to check |
| `path`    | string | File path     |

Response: `{ Bool: boolean }`

<!-- langtabs-start -->
```typescript
type PackedFileExistsRequest = { PackedFileExists: [string, string] };
type PackedFileExistsResponse = { Bool: boolean };
```
```csharp
public class PackedFileExistsRequest
{
    public Tuple<string, string> PackedFileExists { get; set; }
}
public class PackedFileExistsResponse
{
    public bool Bool { get; set; }
}
```
<!-- langtabs-end -->

### GetPackedFileRawData

Get the raw binary data of a packed file.

| Parameter | Type   | Description        |
|-----------|--------|--------------------|
| `pack_key`| string | Pack to query      |
| `path`    | string | Internal file path |

Response: `{ VecU8: number[] }`

<!-- langtabs-start -->
```typescript
type GetPackedFileRawDataRequest = { GetPackedFileRawData: [string, string] };
type GetPackedFileRawDataResponse = { VecU8: number[] };
```
```csharp
public class GetPackedFileRawDataRequest
{
    public Tuple<string, string> GetPackedFileRawData { get; set; }
}
public class GetPackedFileRawDataResponse
{
    public byte[] VecU8 { get; set; }
}
```
<!-- langtabs-end -->

### AddPackedFilesFromPackFile

Copy packed files from one Pack into another.

| Parameter    | Type            | Description                |
|--------------|-----------------|----------------------------|
| `target_key` | string          | Destination pack key       |
| `source_key` | string          | Source pack key            |
| `paths`      | ContainerPath[] | Paths to copy              |

Response: `{ VecContainerPath: ContainerPath[] }`

<!-- langtabs-start -->
```typescript
type AddPackedFilesFromPackFileRequest = {
  AddPackedFilesFromPackFile: [string, string, ContainerPath[]]
};
type AddPackedFilesFromPackFileResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class AddPackedFilesFromPackFileRequest
{
    public Tuple<string, string, List<ContainerPath>> AddPackedFilesFromPackFile { get; set; }
}
public class AddPackedFilesFromPackFileResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### AddPackedFilesFromPackFileToAnimpack

Copy packed files from a Pack into an AnimPack, which may live in a different open Pack.

| Parameter         | Type            | Description                    |
|-------------------|-----------------|--------------------------------|
| `source_pack_key` | string          | Pack the files are copied from |
| `pack_key`        | string          | Pack containing the animpack   |
| `animpack_path`   | string          | Path to the AnimPack           |
| `paths`           | ContainerPath[] | Paths to copy                  |

Response: `{ VecContainerPath: ContainerPath[] }`

<!-- langtabs-start -->
```typescript
type AddPackedFilesFromPackFileToAnimpackRequest = {
  AddPackedFilesFromPackFileToAnimpack: [string, string, string, ContainerPath[]]
};
type AddPackedFilesFromPackFileToAnimpackResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class AddPackedFilesFromPackFileToAnimpackRequest
{
    public Tuple<string, string, string, List<ContainerPath>> AddPackedFilesFromPackFileToAnimpack { get; set; }
}
public class AddPackedFilesFromPackFileToAnimpackResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### AddPackedFilesFromAnimpack

Copy packed files from an AnimPack into a Pack, which may differ from the AnimPack's own.

| Parameter       | Type            | Description                                      |
|-----------------|-----------------|--------------------------------------------------|
| `anim_pack_key` | string          | Pack owning the AnimPack (used when source=PackFile) |
| `pack_key`      | string          | Destination pack                                 |
| `source`        | DataSource      | Data source                                      |
| `animpack_path` | string          | Path to the AnimPack                             |
| `paths`         | ContainerPath[] | Paths to copy                                    |

Response: `{ VecContainerPath: ContainerPath[] }`

<!-- langtabs-start -->
```typescript
type AddPackedFilesFromAnimpackRequest = {
  AddPackedFilesFromAnimpack: [string, string, DataSource, string, ContainerPath[]]
};
type AddPackedFilesFromAnimpackResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class AddPackedFilesFromAnimpackRequest
{
    public Tuple<string, string, string, string, List<ContainerPath>> AddPackedFilesFromAnimpack { get; set; }  // third is DataSource enum
}
public class AddPackedFilesFromAnimpackResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### DeleteFromAnimpack

Delete packed files from an AnimPack.

| Parameter       | Type            | Description              |
|-----------------|-----------------|--------------------------|
| `pack_key`      | string          | Pack containing animpack |
| `animpack_path` | string          | Path to the AnimPack     |
| `paths`         | ContainerPath[] | Paths to delete          |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type DeleteFromAnimpackRequest = {
  DeleteFromAnimpack: [string, string, ContainerPath[]]
};
type DeleteFromAnimpackResponse = "Success";
```
```csharp
public class DeleteFromAnimpackRequest
{
    public Tuple<string, string, List<ContainerPath>> DeleteFromAnimpack { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ImportDependenciesToOpenPackFile

Import files from dependencies into a specific open Pack.

| Parameter | Type                                | Description                   |
|-----------|-------------------------------------|-------------------------------|
| `pack_key`| string                              | Target pack                   |
| `sources` | Record<DataSource, ContainerPath[]> | Files to import by source     |

Response: `{ VecContainerPathVecString: [ContainerPath[], string[]] }` — added paths, failed paths.

<!-- langtabs-start -->
```typescript
type ImportDependenciesToOpenPackFileRequest = {
  ImportDependenciesToOpenPackFile: [string, Record<DataSource, ContainerPath[]>]
};
type ImportDependenciesToOpenPackFileResponse = {
  VecContainerPathVecString: [ContainerPath[], string[]]
};
```
```csharp
public class ImportDependenciesToOpenPackFileRequest
{
    public Tuple<string, Dictionary<string, List<ContainerPath>>> ImportDependenciesToOpenPackFile { get; set; }
}
public class ImportDependenciesToOpenPackFileResponse
{
    public Tuple<List<ContainerPath>, List<string>> VecContainerPathVecString { get; set; }
}
```
<!-- langtabs-end -->

### SavePackedFilesToPackFileAndClean

Save packed files to a specific Pack and optionally run optimizer.

| Parameter | Type     | Description        |
|-----------|----------|--------------------|
| `pack_key`| string   | Target pack        |
| `files`   | RFile[]  | Files to save      |
| `optimize`| boolean  | Run optimizer      |

Response: `{ VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }` — added and deleted paths.

<!-- langtabs-start -->
```typescript
type SavePackedFilesToPackFileAndCleanRequest = {
  SavePackedFilesToPackFileAndClean: [string, RFile[], boolean]
};
type SavePackedFilesToPackFileAndCleanResponse = {
  VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]]
};
```
```csharp
public class SavePackedFilesToPackFileAndCleanRequest
{
    public Tuple<string, List<RFile>, bool> SavePackedFilesToPackFileAndClean { get; set; }
}
public class SavePackedFilesToPackFileAndCleanResponse
{
    public Tuple<List<ContainerPath>, List<ContainerPath>> VecContainerPathVecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### GetPackedFilesNamesStartingWitPathFromAllSources

Get all file names under a path from all dependency sources.

| Parameter | Type          | Description        |
|-----------|---------------|--------------------|
| `path`    | ContainerPath | Path prefix        |

Response: `{ HashMapDataSourceHashSetContainerPath: Record<DataSource, ContainerPath[]> }`

<!-- langtabs-start -->
```typescript
type GetPackedFilesNamesStartingWitPathFromAllSourcesRequest = {
  GetPackedFilesNamesStartingWitPathFromAllSources: ContainerPath
};
type GetPackedFilesNamesStartingWitPathFromAllSourcesResponse = {
  HashMapDataSourceHashSetContainerPath: Record<DataSource, ContainerPath[]>
};
```
```csharp
public class GetPackedFilesNamesStartingWitPathFromAllSourcesRequest
{
    public ContainerPath GetPackedFilesNamesStartingWitPathFromAllSources { get; set; }
}
public class GetPackedFilesNamesStartingWitPathFromAllSourcesResponse
{
    public Dictionary<string, List<ContainerPath>> HashMapDataSourceHashSetContainerPath { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type RebuildDependenciesRequest = { RebuildDependencies: boolean };
type RebuildDependenciesResponse = { DependenciesInfo: DependenciesInfo };
```
```csharp
public class RebuildDependenciesRequest
{
    public bool RebuildDependencies { get; set; }
}
public class RebuildDependenciesResponse
{
    public DependenciesInfo DependenciesInfo { get; set; }
}
```
<!-- langtabs-end -->

### IsThereADependencyDatabase

Check if a dependency database is loaded.

| Parameter       | Type    | Description                           |
|-----------------|---------|---------------------------------------|
| `require_asskit`| boolean | Check that AssKit data is included    |

Response: `{ Bool: boolean }`

<!-- langtabs-start -->
```typescript
type IsThereADependencyDatabaseRequest = { IsThereADependencyDatabase: boolean };
type IsThereADependencyDatabaseResponse = { Bool: boolean };
```
```csharp
public class IsThereADependencyDatabaseRequest
{
    public bool IsThereADependencyDatabase { get; set; }
}
public class IsThereADependencyDatabaseResponse
{
    public bool Bool { get; set; }
}
```
<!-- langtabs-end -->

### GetTableListFromDependencyPackFile

Get all DB table names from dependency Pack files.

Response: `{ VecString: string[] }`

<!-- langtabs-start -->
```typescript
type GetTableListFromDependencyPackFileRequest = "GetTableListFromDependencyPackFile";
type GetTableListFromDependencyPackFileResponse = { VecString: string[] };
```
```csharp
// Request: send the literal string "GetTableListFromDependencyPackFile".
public class GetTableListFromDependencyPackFileResponse
{
    public List<string> VecString { get; set; }
}
```
<!-- langtabs-end -->

### GetCustomTableList

Get custom table names (start_pos_, twad_ prefixes) from the schema.

Response: `{ VecString: string[] }`

<!-- langtabs-start -->
```typescript
type GetCustomTableListRequest = "GetCustomTableList";
type GetCustomTableListResponse = { VecString: string[] };
```
```csharp
// Request: send the literal string "GetCustomTableList".
public class GetCustomTableListResponse
{
    public List<string> VecString { get; set; }
}
```
<!-- langtabs-end -->

### LocalArtSetIds

Get local art set IDs from campaign_character_arts_tables in a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ HashSetString: string[] }`

<!-- langtabs-start -->
```typescript
type LocalArtSetIdsRequest = { LocalArtSetIds: string };
type LocalArtSetIdsResponse = { HashSetString: string[] };
```
```csharp
public class LocalArtSetIdsRequest
{
    public string LocalArtSetIds { get; set; }
}
public class LocalArtSetIdsResponse
{
    public HashSet<string> HashSetString { get; set; }
}
```
<!-- langtabs-end -->

### DependenciesArtSetIds

Get art set IDs from dependencies' campaign_character_arts_tables.

Response: `{ HashSetString: string[] }`

<!-- langtabs-start -->
```typescript
type DependenciesArtSetIdsRequest = "DependenciesArtSetIds";
type DependenciesArtSetIdsResponse = { HashSetString: string[] };
```
```csharp
// Request: send the literal string "DependenciesArtSetIds".
public class DependenciesArtSetIdsResponse
{
    public HashSet<string> HashSetString { get; set; }
}
```
<!-- langtabs-end -->

### GetTableVersionFromDependencyPackFile

Get the version of a table from the dependency database.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ I32: number }`

<!-- langtabs-start -->
```typescript
type GetTableVersionFromDependencyPackFileRequest = { GetTableVersionFromDependencyPackFile: string };
type GetTableVersionFromDependencyPackFileResponse = { I32: number };
```
```csharp
public class GetTableVersionFromDependencyPackFileRequest
{
    public string GetTableVersionFromDependencyPackFile { get; set; }
}
public class GetTableVersionFromDependencyPackFileResponse
{
    public int I32 { get; set; }
}
```
<!-- langtabs-end -->

### GetTableDefinitionFromDependencyPackFile

Get the definition of a table from the dependency database.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ Definition: Definition }`

<!-- langtabs-start -->
```typescript
type GetTableDefinitionFromDependencyPackFileRequest = { GetTableDefinitionFromDependencyPackFile: string };
type GetTableDefinitionFromDependencyPackFileResponse = { Definition: Definition };
```
```csharp
public class GetTableDefinitionFromDependencyPackFileRequest
{
    public string GetTableDefinitionFromDependencyPackFile { get; set; }
}
public class GetTableDefinitionFromDependencyPackFileResponse
{
    public Definition Definition { get; set; }
}
```
<!-- langtabs-end -->

### MergeFiles

Merge multiple compatible tables into one.

| Parameter        | Type            | Description                  |
|------------------|-----------------|------------------------------|
| `pack_key`       | string          | Pack containing the files    |
| `paths`          | ContainerPath[] | Files to merge               |
| `merged_path`    | string          | Destination path for result  |
| `delete_sources` | boolean         | Delete source files after    |

Response: `{ String: string }` — merged path.

<!-- langtabs-start -->
```typescript
type MergeFilesRequest = { MergeFiles: [string, ContainerPath[], string, boolean] };
type MergeFilesResponse = { String: string };
```
```csharp
public class MergeFilesRequest
{
    public Tuple<string, List<ContainerPath>, string, bool> MergeFiles { get; set; }
}
public class MergeFilesResponse
{
    public string String { get; set; }
}
```
<!-- langtabs-end -->

### UpdateTable

Update a table to a newer schema version.

| Parameter | Type          | Description           |
|-----------|---------------|-----------------------|
| `pack_key`| string        | Pack containing table |
| `path`    | ContainerPath | Table path            |

Response: `{ I32I32VecStringVecString: [old_ver, new_ver, deleted_fields, added_fields] }`

<!-- langtabs-start -->
```typescript
type UpdateTableRequest = { UpdateTable: [string, ContainerPath] };
type UpdateTableResponse = {
  I32I32VecStringVecString: [number, number, string[], string[]]
};
```
```csharp
public class UpdateTableRequest
{
    public Tuple<string, ContainerPath> UpdateTable { get; set; }
}
public class UpdateTableResponse
{
    public Tuple<int, int, List<string>, List<string>> I32I32VecStringVecString { get; set; }
}
```
<!-- langtabs-end -->

### GetDependencyPackFilesList

Get the list of Pack files marked as dependencies of a specific Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ VecBoolString: [boolean, string][] }` — `[enabled, pack_name]` pairs.

<!-- langtabs-start -->
```typescript
type GetDependencyPackFilesListRequest = { GetDependencyPackFilesList: string };
type GetDependencyPackFilesListResponse = { VecBoolString: [boolean, string][] };
```
```csharp
public class GetDependencyPackFilesListRequest
{
    public string GetDependencyPackFilesList { get; set; }
}
public class GetDependencyPackFilesListResponse
{
    public List<Tuple<bool, string>> VecBoolString { get; set; }
}
```
<!-- langtabs-end -->

### SetDependencyPackFilesList

Set the list of Pack files marked as dependencies.

| Parameter | Type                 | Description                  |
|-----------|----------------------|------------------------------|
| `pack_key`| string               | Pack to modify               |
| `deps`    | [boolean, string][]  | `[enabled, pack_name]` pairs |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SetDependencyPackFilesListRequest = {
  SetDependencyPackFilesList: [string, [boolean, string][]]
};
type SetDependencyPackFilesListResponse = "Success";
```
```csharp
public class SetDependencyPackFilesListRequest
{
    public Tuple<string, List<Tuple<bool, string>>> SetDependencyPackFilesList { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### GetRFilesFromAllSources

Get packed files from all known sources.

| Parameter  | Type            | Description                  |
|------------|-----------------|------------------------------|
| `paths`    | ContainerPath[] | Paths to retrieve            |
| `lowercase`| boolean         | Normalize paths to lowercase |

Response: `{ HashMapDataSourceHashMapStringRFile: Record<DataSource, Record<string, RFile>> }`

<!-- langtabs-start -->
```typescript
type GetRFilesFromAllSourcesRequest = { GetRFilesFromAllSources: [ContainerPath[], boolean] };
type GetRFilesFromAllSourcesResponse = {
  HashMapDataSourceHashMapStringRFile: Record<DataSource, Record<string, RFile>>
};
```
```csharp
public class GetRFilesFromAllSourcesRequest
{
    public Tuple<List<ContainerPath>, bool> GetRFilesFromAllSources { get; set; }
}
public class GetRFilesFromAllSourcesResponse
{
    public Dictionary<string, Dictionary<string, RFile>> HashMapDataSourceHashMapStringRFile { get; set; }
}
```
<!-- langtabs-end -->

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
      "sources": [{ "Pack": "my_mod.pack" }],
      "search_on": { "db": true, "loc": true, "text": true },
      "matches": {},
      "game_key": "warhammer_3"
    }]
  }
}
```

<!-- langtabs-start -->
```typescript
type GlobalSearchRequest = { GlobalSearch: [string, GlobalSearch] };
type GlobalSearchResponse = { GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] };
```
```csharp
public class GlobalSearchRequest
{
    public Tuple<string, GlobalSearch> GlobalSearch { get; set; }
}
public class GlobalSearchResponse
{
    public Tuple<GlobalSearch, List<RFileInfo>> GlobalSearchVecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

### GlobalSearchReplaceMatches

Replace specific matches in a global search.

| Parameter | Type          | Description            |
|-----------|---------------|------------------------|
| `pack_key`| string        | Pack to modify         |
| `config`  | GlobalSearch  | Search config          |
| `matches` | MatchHolder[] | Matches to replace     |

Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }`

<!-- langtabs-start -->
```typescript
type GlobalSearchReplaceMatchesRequest = {
  GlobalSearchReplaceMatches: [string, GlobalSearch, MatchHolder[]]
};
type GlobalSearchReplaceMatchesResponse = {
  GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]]
};
```
```csharp
public class GlobalSearchReplaceMatchesRequest
{
    public Tuple<string, GlobalSearch, List<MatchHolder>> GlobalSearchReplaceMatches { get; set; }
}
public class GlobalSearchReplaceMatchesResponse
{
    public Tuple<GlobalSearch, List<RFileInfo>> GlobalSearchVecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

### GlobalSearchReplaceAll

Replace all matches in a global search.

| Parameter | Type         | Description          |
|-----------|--------------|----------------------|
| `pack_key`| string       | Pack to modify       |
| `config`  | GlobalSearch | Search config        |

Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }`

<!-- langtabs-start -->
```typescript
type GlobalSearchReplaceAllRequest = { GlobalSearchReplaceAll: [string, GlobalSearch] };
type GlobalSearchReplaceAllResponse = {
  GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]]
};
```
```csharp
public class GlobalSearchReplaceAllRequest
{
    public Tuple<string, GlobalSearch> GlobalSearchReplaceAll { get; set; }
}
public class GlobalSearchReplaceAllResponse
{
    public Tuple<GlobalSearch, List<RFileInfo>> GlobalSearchVecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

### GetReferenceDataFromDefinition

Get reference data for columns in a table definition.

| Parameter     | Type       | Description                    |
|---------------|------------|--------------------------------|
| `pack_key`    | string     | Pack to query                  |
| `table_name`  | string     | Name of the table              |
| `definition`  | Definition | Table definition               |
| `force_local` | boolean    | Force regeneration from local  |

Response: `{ HashMapI32TableReferences: Record<number, TableReferences> }`

<!-- langtabs-start -->
```typescript
type GetReferenceDataFromDefinitionRequest = {
  GetReferenceDataFromDefinition: [string, string, Definition, boolean]
};
type GetReferenceDataFromDefinitionResponse = {
  HashMapI32TableReferences: Record<number, TableReferences>
};
```
```csharp
public class GetReferenceDataFromDefinitionRequest
{
    public Tuple<string, string, Definition, bool> GetReferenceDataFromDefinition { get; set; }
}
public class GetReferenceDataFromDefinitionResponse
{
    public Dictionary<int, TableReferences> HashMapI32TableReferences { get; set; }
}
```
<!-- langtabs-end -->

### SearchReferences

Find all references to a value across tables in a specific pack.

| Parameter          | Type                         | Description                        |
|--------------------|------------------------------|------------------------------------|
| `pack_key`         | string                       | Pack to search                     |
| `table_columns`    | Record<string, string[]>     | Map of table name to column names  |
| `search_value`     | string                       | Value to search for                |

Response: `{ VecDataSourceStringStringStringUsizeUsize: [DataSource, string, string, string, number, number][] }`

Each tuple is `(data_source, pack_key, path, column_name, column_number, row_number)`. `pack_key` identifies which open Pack the hit came from for `PackFile` results, so a client looking at multiple open Packs can route the navigation back to the right one. For `ParentFiles` and `GameFiles` hits, `pack_key` is an empty string — those are read from the dependency cache and aren't tied to a specific open Pack.

<!-- langtabs-start -->
```typescript
type SearchReferencesRequest = {
  SearchReferences: [string, Record<string, string[]>, string]
};
type SearchReferencesResponse = {
  VecDataSourceStringStringStringUsizeUsize: [DataSource, string, string, string, number, number][]
};
```
```csharp
public class SearchReferencesRequest
{
    public Tuple<string, Dictionary<string, List<string>>, string> SearchReferences { get; set; }
}
public class SearchReferencesResponse
{
    public List<Tuple<string, string, string, string, long, long>> VecDataSourceStringStringStringUsizeUsize { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type GoToDefinitionRequest = { GoToDefinition: [string, string, string, string[]] };
type GoToDefinitionResponse = {
  DataSourceStringUsizeUsize: [DataSource, string, number, number]
};
```
```csharp
public class GoToDefinitionRequest
{
    public Tuple<string, string, string, List<string>> GoToDefinition { get; set; }
}
public class GoToDefinitionResponse
{
    public Tuple<string, string, long, long> DataSourceStringUsizeUsize { get; set; }
}
```
<!-- langtabs-end -->

### GoToLoc

Navigate to a loc key's location.

| Parameter | Type   | Description          |
|-----------|--------|----------------------|
| `pack_key`| string | Pack to search       |
| `loc_key` | string | Loc key to find      |

Response: `{ DataSourceStringUsizeUsize: [DataSource, string, number, number] }`

<!-- langtabs-start -->
```typescript
type GoToLocRequest = { GoToLoc: [string, string] };
type GoToLocResponse = {
  DataSourceStringUsizeUsize: [DataSource, string, number, number]
};
```
```csharp
public class GoToLocRequest
{
    public Tuple<string, string> GoToLoc { get; set; }
}
public class GoToLocResponse
{
    public Tuple<string, string, long, long> DataSourceStringUsizeUsize { get; set; }
}
```
<!-- langtabs-end -->

### GetSourceDataFromLocKey

Get the source data (table, column, values) of a loc key.

| Parameter | Type   | Description      |
|-----------|--------|------------------|
| `pack_key`| string | Pack to search   |
| `loc_key` | string | Loc key to look up|

Response: `{ OptionStringStringVecString: [string, string, string[]] | null }`

<!-- langtabs-start -->
```typescript
type GetSourceDataFromLocKeyRequest = { GetSourceDataFromLocKey: [string, string] };
type GetSourceDataFromLocKeyResponse = {
  OptionStringStringVecString: [string, string, string[]] | null
};
```
```csharp
public class GetSourceDataFromLocKeyRequest
{
    public Tuple<string, string> GetSourceDataFromLocKey { get; set; }
}
public class GetSourceDataFromLocKeyResponse
{
    public Tuple<string, string, List<string>>? OptionStringStringVecString { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type CascadeEditionRequest = {
  CascadeEdition: [string, string, Definition, [Field, string, string][]]
};
type CascadeEditionResponse = {
  VecContainerPathVecRFileInfo: [ContainerPath[], RFileInfo[]]
};
```
```csharp
public class CascadeEditionRequest
{
    public Tuple<string, string, Definition, List<Tuple<Field, string, string>>> CascadeEdition { get; set; }
}
public class CascadeEditionResponse
{
    public Tuple<List<ContainerPath>, List<RFileInfo>> VecContainerPathVecRFileInfo { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type SetVideoFormatRequest = { SetVideoFormat: [string, string, SupportedFormats] };
type SetVideoFormatResponse = "Success";
```
```csharp
public class SetVideoFormatRequest
{
    public Tuple<string, string, string> SetVideoFormat { get; set; }  // third is SupportedFormats enum
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Schema Commands

### SaveSchema

Save a schema to disk.

| Parameter | Type   | Description            |
|-----------|--------|------------------------|
| `schema`  | Schema | Complete schema to save|

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SaveSchemaRequest = { SaveSchema: Schema };
type SaveSchemaResponse = "Success";
```
```csharp
public class SaveSchemaRequest
{
    public Schema SaveSchema { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### CleanCache

Encode and clean the internal cache for specified paths.

| Parameter | Type            | Description            |
|-----------|-----------------|------------------------|
| `pack_key`| string          | Pack to clean          |
| `paths`   | ContainerPath[] | Paths to process       |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type CleanCacheRequest = { CleanCache: [string, ContainerPath[]] };
type CleanCacheResponse = "Success";
```
```csharp
public class CleanCacheRequest
{
    public Tuple<string, List<ContainerPath>> CleanCache { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### IsSchemaLoaded

Check if a schema is loaded in memory.

Response: `{ Bool: boolean }`

<!-- langtabs-start -->
```typescript
type IsSchemaLoadedRequest = "IsSchemaLoaded";
type IsSchemaLoadedResponse = { Bool: boolean };
```
```csharp
// Request: send the literal string "IsSchemaLoaded".
public class IsSchemaLoadedResponse
{
    public bool Bool { get; set; }
}
```
<!-- langtabs-end -->

### Schema

Get the currently loaded schema.

Response: `{ Schema: Schema }`

<!-- langtabs-start -->
```typescript
type SchemaRequest = "Schema";
type SchemaResponse = { Schema: Schema };
```
```csharp
// Request: send the literal string "Schema".
public class SchemaResponse
{
    public Schema Schema { get; set; }
}
```
<!-- langtabs-end -->

### DefinitionsByTableName

Get all definitions (all versions) for a table name.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ VecDefinition: Definition[] }`

<!-- langtabs-start -->
```typescript
type DefinitionsByTableNameRequest = { DefinitionsByTableName: string };
type DefinitionsByTableNameResponse = { VecDefinition: Definition[] };
```
```csharp
public class DefinitionsByTableNameRequest
{
    public string DefinitionsByTableName { get; set; }
}
public class DefinitionsByTableNameResponse
{
    public List<Definition> VecDefinition { get; set; }
}
```
<!-- langtabs-end -->

### DefinitionByTableNameAndVersion

Get a specific definition by table name and version.

| Parameter    | Type   | Description      |
|--------------|--------|------------------|
| `table_name` | string | Table name       |
| `version`    | number | Version number   |

Response: `{ Definition: Definition }`

<!-- langtabs-start -->
```typescript
type DefinitionByTableNameAndVersionRequest = {
  DefinitionByTableNameAndVersion: [string, number]
};
type DefinitionByTableNameAndVersionResponse = { Definition: Definition };
```
```csharp
public class DefinitionByTableNameAndVersionRequest
{
    public Tuple<string, int> DefinitionByTableNameAndVersion { get; set; }
}
public class DefinitionByTableNameAndVersionResponse
{
    public Definition Definition { get; set; }
}
```
<!-- langtabs-end -->

### DeleteDefinition

Delete a definition by table name and version.

| Parameter    | Type   | Description      |
|--------------|--------|------------------|
| `table_name` | string | Table name       |
| `version`    | number | Version number   |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type DeleteDefinitionRequest = { DeleteDefinition: [string, number] };
type DeleteDefinitionResponse = "Success";
```
```csharp
public class DeleteDefinitionRequest
{
    public Tuple<string, int> DeleteDefinition { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ReferencingColumnsForDefinition

Get columns from other tables that reference a given table/definition.

| Parameter    | Type       | Description       |
|--------------|------------|-------------------|
| `table_name` | string     | Referenced table  |
| `definition` | Definition | Table definition  |

Response: `{ HashMapStringHashMapStringVecString: Record<string, Record<string, string[]>> }`

<!-- langtabs-start -->
```typescript
type ReferencingColumnsForDefinitionRequest = {
  ReferencingColumnsForDefinition: [string, Definition]
};
type ReferencingColumnsForDefinitionResponse = {
  HashMapStringHashMapStringVecString: Record<string, Record<string, string[]>>
};
```
```csharp
public class ReferencingColumnsForDefinitionRequest
{
    public Tuple<string, Definition> ReferencingColumnsForDefinition { get; set; }
}
public class ReferencingColumnsForDefinitionResponse
{
    public Dictionary<string, Dictionary<string, List<string>>> HashMapStringHashMapStringVecString { get; set; }
}
```
<!-- langtabs-end -->

### FieldsProcessed

Get the processed fields from a definition (bitwise expansion, enum conversion, colour merging applied).

| Parameter    | Type       | Description          |
|--------------|------------|----------------------|
| `definition` | Definition | Definition to process|

Response: `{ VecField: Field[] }`

<!-- langtabs-start -->
```typescript
type FieldsProcessedRequest = { FieldsProcessed: Definition };
type FieldsProcessedResponse = { VecField: Field[] };
```
```csharp
public class FieldsProcessedRequest
{
    public Definition FieldsProcessed { get; set; }
}
public class FieldsProcessedResponse
{
    public List<Field> VecField { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type ExportTSVRequest = { ExportTSV: [string, string, string, DataSource] };
type ExportTSVResponse = "Success";
```
```csharp
public class ExportTSVRequest
{
    public Tuple<string, string, string, string> ExportTSV { get; set; }  // fourth is DataSource enum
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ImportTSV

Import a TSV file as a table.

| Parameter   | Type   | Description               |
|-------------|--------|---------------------------|
| `pack_key`  | string | Target pack               |
| `path`      | string | Internal destination path |
| `tsv_path`  | string | Filesystem TSV path       |

Response: `{ RFileDecoded: RFileDecoded }`

<!-- langtabs-start -->
```typescript
type ImportTSVRequest = { ImportTSV: [string, string, string] };
type ImportTSVResponse = { RFileDecoded: RFileDecoded };
```
```csharp
public class ImportTSVRequest
{
    public Tuple<string, string, string> ImportTSV { get; set; }
}
public class ImportTSVResponse
{
    public RFileDecoded RFileDecoded { get; set; }
}
```
<!-- langtabs-end -->

---

## External Program Commands

### OpenContainingFolder

Open the folder containing a specific open Pack in the file manager.

| Parameter  | Type   | Description             |
|------------|--------|-------------------------|
| `pack_key` | string | Pack whose folder to open|

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type OpenContainingFolderRequest = { OpenContainingFolder: string };
type OpenContainingFolderResponse = "Success";
```
```csharp
public class OpenContainingFolderRequest
{
    public string OpenContainingFolder { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### OpenPackedFileInExternalProgram

Open a packed file in an external program.

| Parameter | Type          | Description         |
|-----------|---------------|---------------------|
| `pack_key`| string        | Pack containing file|
| `source`  | DataSource    | Data source         |
| `path`    | ContainerPath | File path           |

Response: `{ PathBuf: string }` — extracted temporary path.

<!-- langtabs-start -->
```typescript
type OpenPackedFileInExternalProgramRequest = {
  OpenPackedFileInExternalProgram: [string, DataSource, ContainerPath]
};
type OpenPackedFileInExternalProgramResponse = { PathBuf: string };
```
```csharp
public class OpenPackedFileInExternalProgramRequest
{
    public Tuple<string, string, ContainerPath> OpenPackedFileInExternalProgram { get; set; }  // second is DataSource enum
}
public class OpenPackedFileInExternalProgramResponse
{
    public string PathBuf { get; set; }
}
```
<!-- langtabs-end -->

### SavePackedFileFromExternalView

Save a packed file that was edited in an external program.

| Parameter  | Type   | Description            |
|------------|--------|------------------------|
| `pack_key` | string | Target pack            |
| `path`     | string | Internal path          |
| `ext_path` | string | External file path     |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SavePackedFileFromExternalViewRequest = {
  SavePackedFileFromExternalView: [string, string, string]
};
type SavePackedFileFromExternalViewResponse = "Success";
```
```csharp
public class SavePackedFileFromExternalViewRequest
{
    public Tuple<string, string, string> SavePackedFileFromExternalView { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Diagnostics Commands

### DiagnosticsCheck

Run a full diagnostics check over every open pack.

| Parameter             | Type     | Description                                           |
|-----------------------|----------|-------------------------------------------------------|
| `diagnostics_ignored` | string[] | Diagnostic type identifiers to skip during the check  |
| `check_ak`            | boolean  | Also check Assembly Kit-only references               |

Response: `{ Diagnostics: Diagnostics }`

<!-- langtabs-start -->
```typescript
type DiagnosticsCheckRequest = { DiagnosticsCheck: [string[], boolean] };
type DiagnosticsCheckResponse = { Diagnostics: Diagnostics };
```
```csharp
public class DiagnosticsCheckRequest
{
    public Tuple<List<string>, bool> DiagnosticsCheck { get; set; }
}
public class DiagnosticsCheckResponse
{
    public Diagnostics Diagnostics { get; set; }
}
```
<!-- langtabs-end -->

### DiagnosticsUpdate

Run a partial diagnostics update on specific paths.

| Parameter     | Type            | Description                      |
|---------------|-----------------|----------------------------------|
| `diagnostics` | Diagnostics     | Existing diagnostics state       |
| `paths`       | ContainerPath[] | Paths to re-check                |
| `check_ak`    | boolean         | Check AssKit-only references     |

Response: `{ Diagnostics: Diagnostics }`

<!-- langtabs-start -->
```typescript
type DiagnosticsUpdateRequest = {
  DiagnosticsUpdate: [Diagnostics, ContainerPath[], boolean]
};
type DiagnosticsUpdateResponse = { Diagnostics: Diagnostics };
```
```csharp
public class DiagnosticsUpdateRequest
{
    public Tuple<Diagnostics, List<ContainerPath>, bool> DiagnosticsUpdate { get; set; }
}
public class DiagnosticsUpdateResponse
{
    public Diagnostics Diagnostics { get; set; }
}
```
<!-- langtabs-end -->

### AddLineToPackIgnoredDiagnostics

Add a line to a specific pack's ignored diagnostics list.

| Parameter | Type   | Description            |
|-----------|--------|------------------------|
| `pack_key`| string | Pack to modify         |
| `line`    | string | Diagnostic key to ignore|

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type AddLineToPackIgnoredDiagnosticsRequest = {
  AddLineToPackIgnoredDiagnostics: [string, string]
};
type AddLineToPackIgnoredDiagnosticsResponse = "Success";
```
```csharp
public class AddLineToPackIgnoredDiagnosticsRequest
{
    public Tuple<string, string> AddLineToPackIgnoredDiagnostics { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Pack Settings Commands

### GetPackSettings

Get the settings of a specific open Pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ PackSettings: PackSettings }`

<!-- langtabs-start -->
```typescript
type GetPackSettingsRequest = { GetPackSettings: string };
type GetPackSettingsResponse = { PackSettings: PackSettings };
```
```csharp
public class GetPackSettingsRequest
{
    public string GetPackSettings { get; set; }
}
public class GetPackSettingsResponse
{
    public PackSettings PackSettings { get; set; }
}
```
<!-- langtabs-end -->

### SetPackSettings

Set the settings of a specific open Pack.

| Parameter  | Type         | Description       |
|------------|--------------|-------------------|
| `pack_key` | string       | Pack to modify    |
| `settings` | PackSettings | New settings      |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SetPackSettingsRequest = { SetPackSettings: [string, PackSettings] };
type SetPackSettingsResponse = "Success";
```
```csharp
public class SetPackSettingsRequest
{
    public Tuple<string, PackSettings> SetPackSettings { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Notes Commands

### NotesForPath

Get all notes under a given path in a specific pack.

| Parameter | Type   | Description    |
|-----------|--------|----------------|
| `pack_key`| string | Pack to query  |
| `path`    | string | Path prefix    |

Response: `{ VecNote: Note[] }`

<!-- langtabs-start -->
```typescript
type NotesForPathRequest = { NotesForPath: [string, string] };
type NotesForPathResponse = { VecNote: Note[] };
```
```csharp
public class NotesForPathRequest
{
    public Tuple<string, string> NotesForPath { get; set; }
}
public class NotesForPathResponse
{
    public List<Note> VecNote { get; set; }
}
```
<!-- langtabs-end -->

### AddNote

Add a note to a specific pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Target pack   |
| `note`    | Note   | Note to add   |

Response: `{ Note: Note }`

<!-- langtabs-start -->
```typescript
type AddNoteRequest = { AddNote: [string, Note] };
type AddNoteResponse = { Note: Note };
```
```csharp
public class AddNoteRequest
{
    public Tuple<string, Note> AddNote { get; set; }
}
public class AddNoteResponse
{
    public Note Note { get; set; }
}
```
<!-- langtabs-end -->

### DeleteNote

Delete a note from a specific pack.

| Parameter | Type   | Description   |
|-----------|--------|---------------|
| `pack_key`| string | Pack to modify|
| `path`    | string | Note path     |
| `note_id` | number | Note ID       |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type DeleteNoteRequest = { DeleteNote: [string, string, number] };
type DeleteNoteResponse = "Success";
```
```csharp
public class DeleteNoteRequest
{
    public Tuple<string, string, ulong> DeleteNote { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Schema Patch Commands

### SaveLocalSchemaPatch

Save local schema patches to disk.

| Parameter | Type                               | Description              |
|-----------|-------------------------------------|--------------------------|
| `patches` | Record<string, DefinitionPatch>    | Table name to patches    |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SaveLocalSchemaPatchRequest = { SaveLocalSchemaPatch: Record<string, DefinitionPatch> };
type SaveLocalSchemaPatchResponse = "Success";
```
```csharp
public class SaveLocalSchemaPatchRequest
{
    public Dictionary<string, DefinitionPatch> SaveLocalSchemaPatch { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### RemoveLocalSchemaPatchesForTable

Remove all local schema patches for a table.

| Parameter    | Type   | Description   |
|--------------|--------|---------------|
| `table_name` | string | Table name    |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type RemoveLocalSchemaPatchesForTableRequest = { RemoveLocalSchemaPatchesForTable: string };
type RemoveLocalSchemaPatchesForTableResponse = "Success";
```
```csharp
public class RemoveLocalSchemaPatchesForTableRequest
{
    public string RemoveLocalSchemaPatchesForTable { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### RemoveLocalSchemaPatchesForTableAndField

Remove local schema patches for a specific field in a table.

| Parameter    | Type   | Description   |
|--------------|--------|---------------|
| `table_name` | string | Table name    |
| `field_name` | string | Field name    |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type RemoveLocalSchemaPatchesForTableAndFieldRequest = {
  RemoveLocalSchemaPatchesForTableAndField: [string, string]
};
type RemoveLocalSchemaPatchesForTableAndFieldResponse = "Success";
```
```csharp
public class RemoveLocalSchemaPatchesForTableAndFieldRequest
{
    public Tuple<string, string> RemoveLocalSchemaPatchesForTableAndField { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ImportSchemaPatch

Import schema patches into local patches.

| Parameter | Type                               | Description              |
|-----------|-------------------------------------|--------------------------|
| `patches` | Record<string, DefinitionPatch>    | Table name to patches    |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type ImportSchemaPatchRequest = { ImportSchemaPatch: Record<string, DefinitionPatch> };
type ImportSchemaPatchResponse = "Success";
```
```csharp
public class ImportSchemaPatchRequest
{
    public Dictionary<string, DefinitionPatch> ImportSchemaPatch { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Loc Generation Commands

### GenerateMissingLocData

Generate all missing loc entries for a specific open Pack.

| Parameter  | Type   | Description             |
|------------|--------|-------------------------|
| `pack_key` | string | Pack to generate for    |

Response: `{ VecContainerPath: ContainerPath[] }`

<!-- langtabs-start -->
```typescript
type GenerateMissingLocDataRequest = { GenerateMissingLocData: string };
type GenerateMissingLocDataResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class GenerateMissingLocDataRequest
{
    public string GenerateMissingLocData { get; set; }
}
public class GenerateMissingLocDataResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

---

## Update Commands

All update commands take no parameters (send the literal command name). The `Check*` variants return an [`APIResponse`](./ws-shared-types.md#apiresponse) (the main program) or a [`GitResponse`](./ws-shared-types.md#gitresponse) (git-backed repos: schemas, autogen, AK, translations). The `Update*` variants apply the update and return `"Success"`.

<!-- langtabs-start -->
```typescript
// Shared shapes for every command in this section:
type CheckAPIRequest = "CheckUpdates";
type CheckAPIResponse = { APIResponse: APIResponse };

type CheckGitRequest =
  | "CheckSchemaUpdates"
  | "CheckLuaAutogenUpdates"
  | "CheckEmpireAndNapoleonAKUpdates"
  | "CheckTranslationsUpdates";
type CheckGitResponse = { APIResponseGit: GitResponse };

type UpdateApplyRequest =
  | "UpdateSchemas"
  | "UpdateMainProgram"
  | "UpdateLuaAutogen"
  | "UpdateEmpireAndNapoleonAK"
  | "UpdateTranslations";
type UpdateApplyResponse = "Success";
```
```csharp
// Shared shapes for every command in this section:
public class CheckAPIResponse
{
    public APIResponse APIResponse { get; set; }
}
public class CheckGitResponse
{
    public GitResponse APIResponseGit { get; set; }
}
// Update apply requests take no parameters; the response is the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type InitializeMyModFolderRequest = {
  InitializeMyModFolder: [string, string, boolean, boolean, string | null]
};
type InitializeMyModFolderResponse = { PathBuf: string };
```
```csharp
public class InitializeMyModFolderRequest
{
    public Tuple<string, string, bool, bool, string?> InitializeMyModFolder { get; set; }
}
public class InitializeMyModFolderResponse
{
    public string PathBuf { get; set; }
}
```
<!-- langtabs-end -->

### LiveExport

Live-export a specific Pack to the game's data folder.

| Parameter  | Type   | Description    |
|------------|--------|----------------|
| `pack_key` | string | Pack to export |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type LiveExportRequest = { LiveExport: string };
type LiveExportResponse = "Success";
```
```csharp
public class LiveExportRequest
{
    public string LiveExport { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### GetPackOperationalMode

Get the operational mode for a specific pack. This controls whether the pack is treated as a MyMod (with its game/pack association) or as a plain pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ OperationalMode: OperationalMode }` — see [`OperationalMode`](./ws-shared-types.md#operationalmode).

```json
{ "id": 1, "data": { "GetPackOperationalMode": "my_mod.pack" } }
```

<!-- langtabs-start -->
```typescript
type GetPackOperationalModeRequest = { GetPackOperationalMode: string };
type GetPackOperationalModeResponse = { OperationalMode: OperationalMode };
```
```csharp
public class GetPackOperationalModeRequest
{
    public string GetPackOperationalMode { get; set; }
}
public class GetPackOperationalModeResponse
{
    public OperationalMode OperationalMode { get; set; }
}
```
<!-- langtabs-end -->

### SetPackOperationalMode

Set the operational mode for a specific pack.

| Parameter  | Type            | Description           |
|------------|-----------------|-----------------------|
| `pack_key` | string          | Pack to modify        |
| `mode`     | OperationalMode | New operational mode  |

Response: `"Success"`

```json
{ "id": 1, "data": { "SetPackOperationalMode": ["my_mod.pack", { "MyMod": ["warhammer_2", "my_mod.pack"] }] } }
```

<!-- langtabs-start -->
```typescript
type SetPackOperationalModeRequest = {
  SetPackOperationalMode: [string, OperationalMode]
};
type SetPackOperationalModeResponse = "Success";
```
```csharp
public class SetPackOperationalModeRequest
{
    public Tuple<string, OperationalMode> SetPackOperationalMode { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Translation Commands

### GetPackTranslation

Get pack translation data for a language from a specific pack.

| Parameter  | Type   | Description      |
|------------|--------|------------------|
| `pack_key` | string | Pack to query    |
| `language` | string | Language code    |

Response: `{ PackTranslation: PackTranslation }`

<!-- langtabs-start -->
```typescript
type GetPackTranslationRequest = { GetPackTranslation: [string, string] };
type GetPackTranslationResponse = { PackTranslation: PackTranslation };
```
```csharp
public class GetPackTranslationRequest
{
    public Tuple<string, string> GetPackTranslation { get; set; }
}
public class GetPackTranslationResponse
{
    public PackTranslation PackTranslation { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type BuildStarposRequest = { BuildStarpos: [string, string, boolean] };
type BuildStarposResponse = "Success";
```
```csharp
public class BuildStarposRequest
{
    public Tuple<string, string, bool> BuildStarpos { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### BuildStarposPost

Build starpos (post-processing step) for a specific pack.

| Parameter          | Type    | Description              |
|--------------------|---------|--------------------------|
| `pack_key`         | string  | Target pack              |
| `campaign_id`      | string  | Campaign identifier      |
| `process_hlp_spd`  | boolean | Process HLP/SPD data     |

Response: `{ VecContainerPath: ContainerPath[] }`

<!-- langtabs-start -->
```typescript
type BuildStarposPostRequest = { BuildStarposPost: [string, string, boolean] };
type BuildStarposPostResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class BuildStarposPostRequest
{
    public Tuple<string, string, bool> BuildStarposPost { get; set; }
}
public class BuildStarposPostResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### BuildStarposCleanup

Clean up starpos temporary files for a specific pack.

| Parameter          | Type    | Description              |
|--------------------|---------|--------------------------|
| `pack_key`         | string  | Target pack              |
| `campaign_id`      | string  | Campaign identifier      |
| `process_hlp_spd`  | boolean | Process HLP/SPD data     |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type BuildStarposCleanupRequest = { BuildStarposCleanup: [string, string, boolean] };
type BuildStarposCleanupResponse = "Success";
```
```csharp
public class BuildStarposCleanupRequest
{
    public Tuple<string, string, bool> BuildStarposCleanup { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### BuildStarposGetCampaingIds

Get campaign IDs available for starpos building from a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to query |

Response: `{ HashSetString: string[] }`

<!-- langtabs-start -->
```typescript
type BuildStarposGetCampaingIdsRequest = { BuildStarposGetCampaingIds: string };
type BuildStarposGetCampaingIdsResponse = { HashSetString: string[] };
```
```csharp
public class BuildStarposGetCampaingIdsRequest
{
    public string BuildStarposGetCampaingIds { get; set; }
}
public class BuildStarposGetCampaingIdsResponse
{
    public HashSet<string> HashSetString { get; set; }
}
```
<!-- langtabs-end -->

### BuildStarposCheckVictoryConditions

Check if victory conditions file exists in a specific pack.

| Parameter  | Type   | Description   |
|------------|--------|---------------|
| `pack_key` | string | Pack to check |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type BuildStarposCheckVictoryConditionsRequest = { BuildStarposCheckVictoryConditions: string };
type BuildStarposCheckVictoryConditionsResponse = "Success";
```
```csharp
public class BuildStarposCheckVictoryConditionsRequest
{
    public string BuildStarposCheckVictoryConditions { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type UpdateAnimIdsRequest = { UpdateAnimIds: [string, number, number] };
type UpdateAnimIdsResponse = { VecContainerPath: ContainerPath[] };
```
```csharp
public class UpdateAnimIdsRequest
{
    public Tuple<string, int, int> UpdateAnimIds { get; set; }
}
public class UpdateAnimIdsResponse
{
    public List<ContainerPath> VecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

### GetAnimPathsBySkeletonName

Get animation paths by skeleton name.

| Parameter       | Type   | Description    |
|-----------------|--------|----------------|
| `skeleton_name` | string | Skeleton name  |

Response: `{ HashSetString: string[] }`

<!-- langtabs-start -->
```typescript
type GetAnimPathsBySkeletonNameRequest = { GetAnimPathsBySkeletonName: string };
type GetAnimPathsBySkeletonNameResponse = { HashSetString: string[] };
```
```csharp
public class GetAnimPathsBySkeletonNameRequest
{
    public string GetAnimPathsBySkeletonName { get; set; }
}
public class GetAnimPathsBySkeletonNameResponse
{
    public HashSet<string> HashSetString { get; set; }
}
```
<!-- langtabs-end -->

---

## Table Commands

### GetTablesFromDependencies

Get tables from dependencies by table name.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `table_name` | string | Table to query |

Response: `{ VecRFile: RFile[] }`

<!-- langtabs-start -->
```typescript
type GetTablesFromDependenciesRequest = { GetTablesFromDependencies: string };
type GetTablesFromDependenciesResponse = { VecRFile: RFile[] };
```
```csharp
public class GetTablesFromDependenciesRequest
{
    public string GetTablesFromDependencies { get; set; }
}
public class GetTablesFromDependenciesResponse
{
    public List<RFile> VecRFile { get; set; }
}
```
<!-- langtabs-end -->

### GetTablesByTableName

Get table paths by table name from a specific Pack.

| Parameter    | Type   | Description    |
|--------------|--------|----------------|
| `pack_key`   | string | Pack to query  |
| `table_name` | string | Table name     |

Response: `{ VecString: string[] }`

<!-- langtabs-start -->
```typescript
type GetTablesByTableNameRequest = { GetTablesByTableName: [string, string] };
type GetTablesByTableNameResponse = { VecString: string[] };
```
```csharp
public class GetTablesByTableNameRequest
{
    public Tuple<string, string> GetTablesByTableName { get; set; }
}
public class GetTablesByTableNameResponse
{
    public List<string> VecString { get; set; }
}
```
<!-- langtabs-end -->

### AddKeysToKeyDeletes

Add keys to the key_deletes table in a specific pack.

| Parameter        | Type     | Description         |
|------------------|----------|---------------------|
| `pack_key`       | string   | Target pack         |
| `table_file_name`| string   | Table file name     |
| `key_table_name` | string   | Key table name      |
| `keys`           | string[] | Keys to add         |

Response: `{ OptionContainerPath: ContainerPath | null }`

<!-- langtabs-start -->
```typescript
type AddKeysToKeyDeletesRequest = {
  AddKeysToKeyDeletes: [string, string, string, string[]]
};
type AddKeysToKeyDeletesResponse = { OptionContainerPath: ContainerPath | null };
```
```csharp
public class AddKeysToKeyDeletesRequest
{
    public Tuple<string, string, string, HashSet<string>> AddKeysToKeyDeletes { get; set; }
}
public class AddKeysToKeyDeletesResponse
{
    public ContainerPath? OptionContainerPath { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type PackMapRequest = {
  PackMap: [string, string[], [string, string][]]
};
type PackMapResponse = {
  VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]]
};
```
```csharp
public class PackMapRequest
{
    public Tuple<string, List<string>, List<Tuple<string, string>>> PackMap { get; set; }
}
public class PackMapResponse
{
    public Tuple<List<ContainerPath>, List<ContainerPath>> VecContainerPathVecContainerPath { get; set; }
}
```
<!-- langtabs-end -->

---

## 3D Export Commands

### ExportRigidToGltf

Export a RigidModel to glTF format.

| Parameter    | Type       | Description       |
|--------------|------------|-------------------|
| `model`      | RigidModel | Model to export   |
| `output_path`| string     | Output file path  |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type ExportRigidToGltfRequest = { ExportRigidToGltf: [RigidModel, string] };
type ExportRigidToGltfResponse = "Success";
```
```csharp
public class ExportRigidToGltfRequest
{
    public Tuple<RigidModel, string> ExportRigidToGltf { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
// Shared shapes for every getter in this subsection:
type SettingsGetRequest =
  | { SettingsGetBool: string }
  | { SettingsGetI32: string }
  | { SettingsGetF32: string }
  | { SettingsGetString: string }
  | { SettingsGetPathBuf: string }
  | { SettingsGetVecString: string }
  | { SettingsGetVecRaw: string };

type SettingsGetResponse =
  | { Bool: boolean }
  | { I32: number }
  | { F32: number }
  | { String: string }
  | { PathBuf: string }
  | { VecString: string[] }
  | { VecU8: number[] };
```
```csharp
// Request: { "SettingsGetBool": "some_key" } (or any of the other getter names).
// Response wrappers — one per getter:
public class SettingsGetBoolResponse    { public bool Bool { get; set; } }
public class SettingsGetI32Response     { public int I32 { get; set; } }
public class SettingsGetF32Response     { public float F32 { get; set; } }
public class SettingsGetStringResponse  { public string String { get; set; } }
public class SettingsGetPathBufResponse { public string PathBuf { get; set; } }
public class SettingsGetVecStringResponse { public List<string> VecString { get; set; } }
public class SettingsGetVecRawResponse  { public byte[] VecU8 { get; set; } }
```
<!-- langtabs-end -->

### SettingsGetAll

Get all settings at once (batch loading). Much more efficient than individual calls when you need several settings — one IPC round-trip instead of one per key.

Response: `{ SettingsAll: SettingsSnapshot }` — see [`SettingsSnapshot`](./ws-shared-types.md#settingssnapshot) for the field layout (bool / i32 / f32 / string / raw_data / vec_string maps).

```json
{ "id": 1, "data": "SettingsGetAll" }
```

<!-- langtabs-start -->
```typescript
type SettingsGetAllRequest = "SettingsGetAll";
type SettingsGetAllResponse = { SettingsAll: SettingsSnapshot };
```
```csharp
// Request: send the literal string "SettingsGetAll".
public class SettingsGetAllResponse
{
    public SettingsSnapshot SettingsAll { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
// Shared shapes for every setter in this subsection:
type SettingsSetRequest =
  | { SettingsSetBool: [string, boolean] }
  | { SettingsSetI32: [string, number] }
  | { SettingsSetF32: [string, number] }
  | { SettingsSetString: [string, string] }
  | { SettingsSetPathBuf: [string, string] }
  | { SettingsSetVecString: [string, string[]] }
  | { SettingsSetVecRaw: [string, number[]] };

type SettingsSetResponse = "Success";
```
```csharp
// Request wrappers — one per setter. Every response is the literal string "Success".
public class SettingsSetBoolRequest    { public Tuple<string, bool> SettingsSetBool { get; set; } }
public class SettingsSetI32Request     { public Tuple<string, int> SettingsSetI32 { get; set; } }
public class SettingsSetF32Request     { public Tuple<string, float> SettingsSetF32 { get; set; } }
public class SettingsSetStringRequest  { public Tuple<string, string> SettingsSetString { get; set; } }
public class SettingsSetPathBufRequest { public Tuple<string, string> SettingsSetPathBuf { get; set; } }
public class SettingsSetVecStringRequest { public Tuple<string, List<string>> SettingsSetVecString { get; set; } }
public class SettingsSetVecRawRequest  { public Tuple<string, byte[]> SettingsSetVecRaw { get; set; } }
```
<!-- langtabs-end -->

### SettingsClearPath

Clear a specific config path entry.

| Parameter | Type   | Description    |
|-----------|--------|----------------|
| `path`    | string | Path to clear  |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type SettingsClearPathRequest = { SettingsClearPath: string };
type SettingsClearPathResponse = "Success";
```
```csharp
public class SettingsClearPathRequest
{
    public string SettingsClearPath { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
// Shared shapes for every command in this section:
type PathCommandRequest =
  | "ConfigPath"
  | "AssemblyKitPath"
  | "BackupAutosavePath"
  | "OldAkDataPath"
  | "SchemasPath"
  | "TableProfilesPath"
  | "TranslationsLocalPath"
  | "DependenciesCachePath";
type PathCommandResponse = { PathBuf: string };
```
```csharp
// Request: send the literal command name as a string.
public class PathCommandResponse
{
    public string PathBuf { get; set; }
}
```
<!-- langtabs-end -->

---

## Settings Backup Commands

### BackupSettings

Backup current settings to memory (for restore on cancel).

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type BackupSettingsRequest = "BackupSettings";
type BackupSettingsResponse = "Success";
```
```csharp
// Request: send the literal string "BackupSettings".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### ClearSettings

Clear all settings and reset to defaults.

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type ClearSettingsRequest = "ClearSettings";
type ClearSettingsResponse = "Success";
```
```csharp
// Request: send the literal string "ClearSettings".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### RestoreBackupSettings

Restore settings from the in-memory backup.

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type RestoreBackupSettingsRequest = "RestoreBackupSettings";
type RestoreBackupSettingsResponse = "Success";
```
```csharp
// Request: send the literal string "RestoreBackupSettings".
// Response: the literal string "Success".
```
<!-- langtabs-end -->

### OptimizerOptions

Get the optimizer options configuration.

Response: `{ OptimizerOptions: OptimizerOptions }`

<!-- langtabs-start -->
```typescript
type OptimizerOptionsRequest = "OptimizerOptions";
type OptimizerOptionsResponse = { OptimizerOptions: OptimizerOptions };
```
```csharp
// Request: send the literal string "OptimizerOptions".
public class OptimizerOptionsResponse
{
    public OptimizerOptions OptimizerOptions { get; set; }
}
```
<!-- langtabs-end -->

---

## Debug Commands

### GetMissingDefinitions

Export missing table definitions from a specific pack to a file (for debugging/development).

| Parameter  | Type   | Description        |
|------------|--------|--------------------|
| `pack_key` | string | Pack to export from|

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type GetMissingDefinitionsRequest = { GetMissingDefinitions: string };
type GetMissingDefinitionsResponse = "Success";
```
```csharp
public class GetMissingDefinitionsRequest
{
    public string GetMissingDefinitions { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->

---

## Autosave Commands

### TriggerBackupAutosave

Trigger an autosave backup of a specific Pack.

| Parameter  | Type   | Description          |
|------------|--------|----------------------|
| `pack_key` | string | Pack to back up      |

Response: `"Success"`

<!-- langtabs-start -->
```typescript
type TriggerBackupAutosaveRequest = { TriggerBackupAutosave: string };
type TriggerBackupAutosaveResponse = "Success";
```
```csharp
public class TriggerBackupAutosaveRequest
{
    public string TriggerBackupAutosave { get; set; }
}
// Response: the literal string "Success".
```
<!-- langtabs-end -->
