# Shared Types

This page documents all the data types used in the RPFM server protocol. These types appear as parameters in [Commands](./ws-commands.md) and as payloads in [Responses](./ws-responses.md).

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

<!-- langtabs-start -->
```typescript
type DataSource = "PackFile" | "GameFiles" | "ParentFiles" | "AssKitFiles" | "ExternalFile";
```
```csharp
// Serialize as a plain string — idiomatic C# is an enum with [JsonStringEnumConverter]:
public enum DataSource
{
    PackFile,
    GameFiles,
    ParentFiles,
    AssKitFiles,
    ExternalFile,
}
```
<!-- langtabs-end -->

### ContainerPath

A file or folder path within a Pack. Serialized as a tagged enum:

```json
{ "File": "db/units_tables/data" }
{ "Folder": "db/units_tables" }
```

<!-- langtabs-start -->
```typescript
type ContainerPath =
  | { File: string }
  | { Folder: string };
```
```csharp
// Serialize as either { "File": "..." } or { "Folder": "..." }:
public abstract class ContainerPath { }
public class ContainerPathFile   : ContainerPath { public string File { get; set; } }
public class ContainerPathFolder : ContainerPath { public string Folder { get; set; } }
```
<!-- langtabs-end -->

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

| Field           | Type              | Description                              |
|-----------------|-------------------|------------------------------------------|
| `file_name`     | string            | Name of the Pack file                    |
| `file_path`     | string            | Full path to the Pack file on disk       |
| `pfh_version`   | PFHVersion        | PFH format version                       |
| `pfh_file_type` | PFHFileType       | Pack file type                           |
| `bitmask`       | PFHFlags          | PFH flags bitmask (u32)                  |
| `compress`      | CompressionFormat | Compression format                       |
| `timestamp`     | number            | Pack file timestamp (Unix epoch)         |

<!-- langtabs-start -->
```typescript
interface ContainerInfo {
  file_name: string;
  file_path: string;
  pfh_version: PFHVersion;
  pfh_file_type: PFHFileType;
  bitmask: PFHFlags;
  compress: CompressionFormat;
  timestamp: number;
}
```
```csharp
public class ContainerInfo
{
    public string FileName { get; set; }
    public string FilePath { get; set; }
    public PFHVersion PfhVersion { get; set; }
    public PFHFileType PfhFileType { get; set; }
    public PFHFlags Bitmask { get; set; }
    public CompressionFormat Compress { get; set; }
    public ulong Timestamp { get; set; }
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

<!-- langtabs-start -->
```typescript
type NewFile =
  | { AnimPack: string }
  | { DB: [string, string, number] }
  | { Loc: string }
  | { PortraitSettings: [string, number, [string, string][]] }
  | { Text: [string, string] }
  | { VMD: string }
  | { WSModel: string };
```
```csharp
// Tagged-enum payload — one class per variant:
public abstract class NewFile { }
public class NewFileAnimPack : NewFile { public string AnimPack { get; set; } }
public class NewFileDB : NewFile { public Tuple<string, string, int> DB { get; set; } }
public class NewFileLoc : NewFile { public string Loc { get; set; } }
public class NewFilePortraitSettings : NewFile
{
    public Tuple<string, uint, List<Tuple<string, string>>> PortraitSettings { get; set; }
}
public class NewFileText : NewFile { public Tuple<string, string> Text { get; set; } }  // second is TextFormat enum
public class NewFileVMD : NewFile { public string VMD { get; set; } }
public class NewFileWSModel : NewFile { public string WSModel { get; set; } }
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type FileType =
  | "Anim" | "AnimFragmentBattle" | "AnimPack" | "AnimsTable" | "Atlas" | "Audio"
  | "BMD" | "BMDVegetation" | "Dat" | "DB" | "ESF" | "Font" | "GroupFormations"
  | "HlslCompiled" | "Image" | "Loc" | "MatchedCombat" | "Pack" | "PortraitSettings"
  | "RigidModel" | "SoundBank" | "Text" | "UIC" | "UnitVariant" | "Video" | "VMD"
  | "WSModel" | "Unknown";
```
```csharp
public enum FileType
{
    Anim, AnimFragmentBattle, AnimPack, AnimsTable, Atlas, Audio,
    BMD, BMDVegetation, Dat, DB, ESF, Font, GroupFormations,
    HlslCompiled, Image, Loc, MatchedCombat, Pack, PortraitSettings,
    RigidModel, SoundBank, Text, UIC, UnitVariant, Video, VMD,
    WSModel, Unknown,
}
```
<!-- langtabs-end -->

### CompressionFormat

Compression format for Pack files. Serialized as a plain string:

| Value      | Description                |
|------------|----------------------------|
| `"None"`   | No compression             |
| `"Lzma1"`  | LZMA1 compression          |
| `"Lz4"`    | LZ4 compression            |
| `"Zstd"`   | Zstandard compression      |

<!-- langtabs-start -->
```typescript
type CompressionFormat = "None" | "Lzma1" | "Lz4" | "Zstd";
```
```csharp
public enum CompressionFormat { None, Lzma1, Lz4, Zstd }
```
<!-- langtabs-end -->

### PFHFileType

Pack file type. Serialized as a plain string:

| Value       | Description        |
|-------------|--------------------|
| `"Boot"`    | Boot Pack          |
| `"Release"` | Release Pack       |
| `"Patch"`   | Patch Pack         |
| `"Mod"`     | Mod Pack           |
| `"Movie"`   | Movie Pack         |

<!-- langtabs-start -->
```typescript
type PFHFileType = "Boot" | "Release" | "Patch" | "Mod" | "Movie";
```
```csharp
public enum PFHFileType { Boot, Release, Patch, Mod, Movie }
```
<!-- langtabs-end -->

### PFHVersion

PFH (Pack file header) format version. Serialized as a plain string:

| Value    | Description                                  |
|----------|----------------------------------------------|
| `"PFH0"` | Original format (Empire / Napoleon era)      |
| `"PFH2"` | Shogun 2 era                                 |
| `"PFH3"` | Rome 2 era                                   |
| `"PFH4"` | Attila / Warhammer era                       |
| `"PFH5"` | Warhammer 2 / Three Kingdoms era             |
| `"PFH6"` | Warhammer 3 / Pharaoh / current era          |

<!-- langtabs-start -->
```typescript
type PFHVersion = "PFH0" | "PFH2" | "PFH3" | "PFH4" | "PFH5" | "PFH6";
```
```csharp
public enum PFHVersion { PFH0, PFH2, PFH3, PFH4, PFH5, PFH6 }
```
<!-- langtabs-end -->

### PFHFlags

Pack file header flags. Serialized as a **single integer** (a u32 bitmask) with these bits:

| Bit              | Flag                         | Meaning                              |
|------------------|------------------------------|--------------------------------------|
| `0x0000_0100`    | `HAS_EXTENDED_HEADER`        | Pack has an extended header          |
| `0x0000_0080`    | `HAS_ENCRYPTED_INDEX`        | The file index is encrypted          |
| `0x0000_0040`    | `HAS_INDEX_WITH_TIMESTAMPS`  | Index entries include timestamps     |
| `0x0000_0010`    | `HAS_ENCRYPTED_DATA`         | File data is encrypted               |

<!-- langtabs-start -->
```typescript
// Raw u32 on the wire. Use bitwise-and against the constants to test flags.
type PFHFlags = number;

const PFH_HAS_EXTENDED_HEADER      = 0x0000_0100;
const PFH_HAS_ENCRYPTED_INDEX      = 0x0000_0080;
const PFH_HAS_INDEX_WITH_TIMESTAMPS = 0x0000_0040;
const PFH_HAS_ENCRYPTED_DATA       = 0x0000_0010;
```
```csharp
[Flags]
public enum PFHFlags : uint
{
    None = 0,
    HasExtendedHeader      = 0x0000_0100,
    HasEncryptedIndex      = 0x0000_0080,
    HasIndexWithTimestamps = 0x0000_0040,
    HasEncryptedData       = 0x0000_0010,
}
```
<!-- langtabs-end -->

### SupportedFormats

Video container format. Serialized as a plain string:

| Value      | Description                                |
|------------|--------------------------------------------|
| `"CaVp8"`  | Creative Assembly's CA_VP8 container       |
| `"Ivf"`    | Standard VP8 IVF container                 |

<!-- langtabs-start -->
```typescript
type SupportedFormats = "CaVp8" | "Ivf";
```
```csharp
public enum SupportedFormats { CaVp8, Ivf }
```
<!-- langtabs-end -->

### OperationalMode

Per-pack operational mode, controlling MyMod behaviour. Serialized as a tagged enum:

| Variant    | Payload              | Description                                     |
|------------|----------------------|-------------------------------------------------|
| `MyMod`    | `[string, string]`   | `[game_folder_name, mymod_pack_name]`           |
| `Normal`   | *(none)*             | Normal mode, no MyMod association               |

Example:

```json
{ "MyMod": ["warhammer_2", "my_mymod.pack"] }
"Normal"
```

<!-- langtabs-start -->
```typescript
type OperationalMode =
  | { MyMod: [string, string] }
  | "Normal";
```
```csharp
// Serialize as either { "MyMod": [string, string] } or the string "Normal".
public abstract class OperationalMode { }
public class OperationalModeMyMod : OperationalMode
{
    public string GameFolder { get; set; }
    public string MyModPackName { get; set; }
}
public class OperationalModeNormal : OperationalMode { }
```
<!-- langtabs-end -->

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
| `SequenceU16`      | number[]          | Raw bytes for a nested sequence (u16-prefixed count) |
| `SequenceU32`      | number[]          | Raw bytes for a nested sequence (u32-prefixed count) |

Example:

```json
{ "StringU8": "hello" }
{ "I32": 42 }
{ "Boolean": true }
```

<!-- langtabs-start -->
```typescript
type DecodedData =
  | { Boolean: boolean }
  | { F32: number } | { F64: number }
  | { I16: number } | { I32: number } | { I64: number }
  | { ColourRGB: string }
  | { StringU8: string } | { StringU16: string }
  | { OptionalI16: number } | { OptionalI32: number } | { OptionalI64: number }
  | { OptionalStringU8: string } | { OptionalStringU16: string }
  | { SequenceU16: number[] } | { SequenceU32: number[] };
```
```csharp
// Tagged-enum — one class per variant. Branch on the JSON key.
public abstract class DecodedData { }
public class DecodedDataBoolean         : DecodedData { public bool Boolean { get; set; } }
public class DecodedDataF32             : DecodedData { public float F32 { get; set; } }
public class DecodedDataF64             : DecodedData { public double F64 { get; set; } }
public class DecodedDataI16             : DecodedData { public short I16 { get; set; } }
public class DecodedDataI32             : DecodedData { public int I32 { get; set; } }
public class DecodedDataI64             : DecodedData { public long I64 { get; set; } }
public class DecodedDataColourRGB       : DecodedData { public string ColourRGB { get; set; } }
public class DecodedDataStringU8        : DecodedData { public string StringU8 { get; set; } }
public class DecodedDataStringU16       : DecodedData { public string StringU16 { get; set; } }
public class DecodedDataOptionalI16     : DecodedData { public short OptionalI16 { get; set; } }
public class DecodedDataOptionalI32     : DecodedData { public int OptionalI32 { get; set; } }
public class DecodedDataOptionalI64     : DecodedData { public long OptionalI64 { get; set; } }
public class DecodedDataOptionalStringU8 : DecodedData { public string OptionalStringU8 { get; set; } }
public class DecodedDataOptionalStringU16: DecodedData { public string OptionalStringU16 { get; set; } }
public class DecodedDataSequenceU16     : DecodedData { public byte[] SequenceU16 { get; set; } }
public class DecodedDataSequenceU32     : DecodedData { public byte[] SequenceU32 { get; set; } }
```
<!-- langtabs-end -->

### TableInMemory

In-memory table data structure used by DB and Loc files.

| Field              | Type              | Description                                    |
|--------------------|-------------------|------------------------------------------------|
| `table_name`       | string            | Table type identifier (e.g. `"units_tables"`)  |
| `definition`       | Definition        | Schema definition for this table               |
| `definition_patch` | DefinitionPatch   | Runtime schema modifications                   |
| `table_data`       | DecodedData[][]   | Row data (outer = rows, inner = columns)       |
| `altered`          | boolean           | Whether data was altered during decoding        |

<!-- langtabs-start -->
```typescript
interface TableInMemory {
  table_name: string;
  definition: Definition;
  definition_patch: DefinitionPatch;
  table_data: DecodedData[][];
  altered: boolean;
}
```
```csharp
public class TableInMemory
{
    public string TableName { get; set; }
    public Definition Definition { get; set; }
    public DefinitionPatch DefinitionPatch { get; set; }
    public List<List<DecodedData>> TableData { get; set; }
    public bool Altered { get; set; }
}
```
<!-- langtabs-end -->

### RFile

A raw packed file.

| Field            | Type           | Description                          |
|------------------|----------------|--------------------------------------|
| `path`           | string         | Path of the file within a container  |
| `timestamp`      | number or null | Last modified timestamp (Unix epoch) |
| `file_type`      | FileType       | Detected or specified file type      |
| `container_name` | string or null | Name of the source container         |
| `data`           | unknown        | Internal data storage                |

<!-- langtabs-start -->
```typescript
interface RFile {
  path: string;
  timestamp: number | null;
  file_type: FileType;
  container_name: string | null;
  data: unknown;
}
```
```csharp
public class RFile
{
    public string Path { get; set; }
    public long? Timestamp { get; set; }
    public FileType FileType { get; set; }
    public string? ContainerName { get; set; }
    public object Data { get; set; }
}
```
<!-- langtabs-end -->

### RFileDecoded

Decoded file content. Serialized as a tagged enum — the variant name indicates the file type. See [Decoded File Types](#decoded-file-types) for each type's structure.

Variants: `Anim`, `AnimFragmentBattle`, `AnimPack`, `AnimsTable`, `Atlas`, `Audio`, `BMD`, `BMDVegetation`, `Dat`, `DB`, `ESF`, `Font`, `GroupFormations`, `HlslCompiled`, `Image`, `Loc`, `MatchedCombat`, `Pack`, `PortraitSettings`, `RigidModel`, `SoundBank`, `Text`, `UIC`, `UnitVariant`, `Unknown`, `Video`, `VMD`, `WSModel`.

Example:

```json
{ "DB": { "mysterious_byte": true, "guid": "", "table": { ... } } }
{ "Text": { "encoding": "Utf8Bom", "format": "Lua", "contents": "-- script" } }
```

<!-- langtabs-start -->
```typescript
// One variant per known file type. Most carry the decoded type described below;
// a handful (VMD, WSModel) wrap a Text payload.
type RFileDecoded =
  | { Anim: unknown } | { AnimFragmentBattle: AnimFragmentBattle }
  | { AnimPack: unknown } | { AnimsTable: AnimsTable }
  | { Atlas: Atlas } | { Audio: Audio }
  | { BMD: Bmd } | { BMDVegetation: unknown }
  | { Dat: unknown } | { DB: DB }
  | { ESF: ESF } | { Font: unknown }
  | { GroupFormations: GroupFormations } | { HlslCompiled: unknown }
  | { Image: Image } | { Loc: Loc }
  | { MatchedCombat: MatchedCombat } | { Pack: unknown }
  | { PortraitSettings: PortraitSettings } | { RigidModel: RigidModel }
  | { SoundBank: unknown } | { Text: Text }
  | { UIC: UIC } | { UnitVariant: UnitVariant }
  | { Unknown: unknown } | { Video: unknown }
  | { VMD: Text } | { WSModel: Text };
```
```csharp
// Tagged enum — one wrapper class per variant. Only the most common ones shown.
public abstract class RFileDecoded { }
public class RFileDecodedDB : RFileDecoded { public DB DB { get; set; } }
public class RFileDecodedLoc : RFileDecoded { public Loc Loc { get; set; } }
public class RFileDecodedText : RFileDecoded { public Text Text { get; set; } }
public class RFileDecodedImage : RFileDecoded { public Image Image { get; set; } }
public class RFileDecodedRigidModel : RFileDecoded { public RigidModel RigidModel { get; set; } }
// ... one class per variant listed in the enum above.
```
<!-- langtabs-end -->

### TableReferences

Reference data for a column, used by lookup/autocomplete features.

| Field                          | Type                   | Description                                        |
|--------------------------------|------------------------|----------------------------------------------------|
| `field_name`                   | string                 | Name of the column these references are for        |
| `referenced_table_is_ak_only`  | boolean                | Whether the referenced table only exists in the AK |
| `referenced_column_is_localised` | boolean              | Whether the referenced column is localised         |
| `data`                         | Record<string, string> | Map of actual values to their display text         |

<!-- langtabs-start -->
```typescript
interface TableReferences {
  field_name: string;
  referenced_table_is_ak_only: boolean;
  referenced_column_is_localised: boolean;
  data: Record<string, string>;
}
```
```csharp
public class TableReferences
{
    public string FieldName { get; set; }
    public bool ReferencedTableIsAkOnly { get; set; }
    public bool ReferencedColumnIsLocalised { get; set; }
    public Dictionary<string, string> Data { get; set; }
}
```
<!-- langtabs-end -->

---

## Decoded File Types

### DB

Decoded database table file.

| Field            | Type          | Description                                      |
|------------------|---------------|--------------------------------------------------|
| `mysterious_byte`| boolean       | Boolean flag (setting to 0 can crash WH2)        |
| `guid`           | string        | GUID for this table instance (empty for older games) |
| `table`          | TableInMemory | The table data including definition and rows      |

<!-- langtabs-start -->
```typescript
interface DB {
  mysterious_byte: boolean;
  guid: string;
  table: TableInMemory;
}
```
```csharp
public class DB
{
    public bool MysteriousByte { get; set; }
    public string Guid { get; set; }
    public TableInMemory Table { get; set; }
}
```
<!-- langtabs-end -->

### Loc

Decoded localisation file.

| Field   | Type          | Description                                         |
|---------|---------------|-----------------------------------------------------|
| `table` | TableInMemory | Table data with key, text, and tooltip columns      |

<!-- langtabs-start -->
```typescript
interface Loc {
  table: TableInMemory;
}
```
```csharp
public class Loc
{
    public TableInMemory Table { get; set; }
}
```
<!-- langtabs-end -->

### Text

Decoded text file.

| Field      | Type         | Description                    |
|------------|--------------|--------------------------------|
| `encoding` | TextEncoding | Character encoding of the file |
| `format`   | TextFormat   | Detected file format           |
| `contents` | string       | Decoded text contents          |

<!-- langtabs-start -->
```typescript
interface Text {
  encoding: TextEncoding;
  format: TextFormat;
  contents: string;
}
```
```csharp
public class Text
{
    public TextEncoding Encoding { get; set; }
    public TextFormat Format { get; set; }
    public string Contents { get; set; }
}
```
<!-- langtabs-end -->

### TextEncoding

Character encoding of a text file's contents. Serialized as a plain string:

| Value          | Description                        |
|----------------|------------------------------------|
| `"Iso8859_1"`  | ISO/IEC 8859-1 (Latin-1)           |
| `"Utf8"`       | UTF-8 without BOM                  |
| `"Utf8Bom"`    | UTF-8 with BOM                     |
| `"Utf16Le"`    | UTF-16 little-endian               |

> **Rust source name:** `Encoding` (in `rpfm_lib::files::text`). This doc uses the more descriptive name `TextEncoding` for clarity — the on-the-wire JSON is identical.

<!-- langtabs-start -->
```typescript
type TextEncoding = "Iso8859_1" | "Utf8" | "Utf8Bom" | "Utf16Le";
```
```csharp
public enum TextEncoding { Iso8859_1, Utf8, Utf8Bom, Utf16Le }
```
<!-- langtabs-end -->

### TextFormat

Source-code format / syntax-highlighting language for a text file. Also used as the second element of the [`NewFile.Text`](#newfile) payload. Serialized as a plain string:

| Value          | Description                         |
|----------------|-------------------------------------|
| `"Bat"`        | Windows batch file                  |
| `"Cpp"`        | C / C++                             |
| `"Css"`        | CSS                                 |
| `"Hlsl"`       | HLSL shader                         |
| `"Html"`       | HTML                                |
| `"Js"`         | JavaScript                          |
| `"Json"`       | JSON                                |
| `"Lua"`        | Lua                                 |
| `"Markdown"`   | Markdown                            |
| `"Plain"`      | Plain text (no specific format)     |
| `"Python"`     | Python                              |
| `"Sql"`        | SQL                                 |
| `"Xml"`        | XML                                 |
| `"Yaml"`       | YAML                                |

<!-- langtabs-start -->
```typescript
type TextFormat =
  | "Bat" | "Cpp" | "Html" | "Hlsl" | "Json" | "Js" | "Css"
  | "Lua" | "Markdown" | "Plain" | "Python" | "Sql" | "Xml" | "Yaml";
```
```csharp
public enum TextFormat { Bat, Cpp, Html, Hlsl, Json, Js, Css, Lua, Markdown, Plain, Python, Sql, Xml, Yaml }
```
<!-- langtabs-end -->

### Image

Decoded image file.

| Field            | Type             | Description                                      |
|------------------|------------------|--------------------------------------------------|
| `data`           | number[]         | Original raw image data in native format         |
| `converted_data` | number[] or null | PNG-converted data for DDS textures (for viewing)|

<!-- langtabs-start -->
```typescript
interface Image {
  data: number[];
  converted_data: number[] | null;
}
```
```csharp
public class Image
{
    public byte[] Data { get; set; }
    public byte[]? ConvertedData { get; set; }
}
```
<!-- langtabs-end -->

### RigidModel

Decoded RigidModel (3D model) file.

| Field         | Type      | Description                                        |
|---------------|-----------|----------------------------------------------------|
| `version`     | number    | File format version (6, 7, or 8)                   |
| `uk_1`        | number    | Unknown field                                      |
| `skeleton_id` | string    | Skeleton identifier for animation (empty if static)|
| `lods`        | unknown[] | LOD structures from highest to lowest quality      |

<!-- langtabs-start -->
```typescript
interface RigidModel {
  version: number;
  uk_1: number;
  skeleton_id: string;
  lods: unknown[];
}
```
```csharp
public class RigidModel
{
    public uint Version { get; set; }
    public uint Uk1 { get; set; }
    public string SkeletonId { get; set; }
    public List<object> Lods { get; set; }
}
```
<!-- langtabs-end -->

### ESF

Decoded ESF (Empire Save Format) file.

| Field           | Type    | Description                         |
|-----------------|---------|-------------------------------------|
| `signature`     | string  | Format signature (CAAB, CBAB, etc.) |
| `unknown_1`     | number  | Unknown header field, typically 0   |
| `creation_date` | number  | Creation timestamp                  |
| `root_node`     | unknown | Root node of the data tree          |

<!-- langtabs-start -->
```typescript
interface ESF {
  signature: string;
  unknown_1: number;
  creation_date: number;
  root_node: unknown;
}
```
```csharp
public class ESF
{
    public string Signature { get; set; }
    public uint Unknown1 { get; set; }
    public uint CreationDate { get; set; }
    public object RootNode { get; set; }
}
```
<!-- langtabs-end -->

### Bmd

Decoded BMD (Battle Map Data) file.

| Field                | Type    | Description                    |
|----------------------|---------|--------------------------------|
| `serialise_version`  | number  | File format version (23-27)    |
| *(other fields)*     | unknown | Complex battlefield-related data |

<!-- langtabs-start -->
```typescript
// Only the stable header field is typed; the rest is format-specific and opaque.
interface Bmd {
  serialise_version: number;
  [other: string]: unknown;
}
```
```csharp
public class Bmd
{
    public uint SerialiseVersion { get; set; }
    // plus format-specific fields depending on serialise_version
}
```
<!-- langtabs-end -->

### AnimFragmentBattle

Decoded AnimFragmentBattle file.

| Field           | Type      | Description                            |
|-----------------|-----------|----------------------------------------|
| `version`       | number    | File format version (2 or 4)           |
| `entries`       | unknown[] | List of animation entries              |
| `skeleton_name` | string    | Name of the skeleton                   |
| `subversion`    | number    | Format subversion (version 4 only)     |

<!-- langtabs-start -->
```typescript
interface AnimFragmentBattle {
  version: number;
  entries: unknown[];
  skeleton_name: string;
  subversion: number;
}
```
```csharp
public class AnimFragmentBattle
{
    public uint Version { get; set; }
    public List<object> Entries { get; set; }
    public string SkeletonName { get; set; }
    public uint Subversion { get; set; }
}
```
<!-- langtabs-end -->

### AnimsTable

Decoded AnimsTable file.

| Field     | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `version` | number    | File format version (currently 2)  |
| `entries` | unknown[] | List of animation table entries    |

<!-- langtabs-start -->
```typescript
interface AnimsTable {
  version: number;
  entries: unknown[];
}
```
```csharp
public class AnimsTable
{
    public uint Version { get; set; }
    public List<object> Entries { get; set; }
}
```
<!-- langtabs-end -->

### Atlas

Decoded Atlas (sprite sheet) file.

| Field     | Type      | Description                        |
|-----------|-----------|------------------------------------|
| `version` | number    | File format version (currently 1)  |
| `unknown` | number    | Unknown field                      |
| `entries` | unknown[] | List of sprite entries             |

<!-- langtabs-start -->
```typescript
interface Atlas {
  version: number;
  unknown: number;
  entries: unknown[];
}
```
```csharp
public class Atlas
{
    public uint Version { get; set; }
    public uint Unknown { get; set; }
    public List<object> Entries { get; set; }
}
```
<!-- langtabs-end -->

### Audio

Decoded Audio file.

| Field  | Type     | Description          |
|--------|----------|----------------------|
| `data` | number[] | Raw binary audio data|

<!-- langtabs-start -->
```typescript
interface Audio {
  data: number[];
}
```
```csharp
public class Audio
{
    public byte[] Data { get; set; }
}
```
<!-- langtabs-end -->

### GroupFormations

Decoded GroupFormations file.

| Field        | Type      | Description                  |
|--------------|-----------|------------------------------|
| `formations` | unknown[] | List of formation definitions|

<!-- langtabs-start -->
```typescript
interface GroupFormations {
  formations: unknown[];
}
```
```csharp
public class GroupFormations
{
    public List<object> Formations { get; set; }
}
```
<!-- langtabs-end -->

### MatchedCombat

Decoded MatchedCombat file.

| Field     | Type      | Description                           |
|-----------|-----------|---------------------------------------|
| `version` | number    | File format version (1 or 3)          |
| `entries` | unknown[] | List of matched combat entries        |

<!-- langtabs-start -->
```typescript
interface MatchedCombat {
  version: number;
  entries: unknown[];
}
```
```csharp
public class MatchedCombat
{
    public uint Version { get; set; }
    public List<object> Entries { get; set; }
}
```
<!-- langtabs-end -->

### PortraitSettings

Decoded PortraitSettings file.

| Field     | Type      | Description                          |
|-----------|-----------|--------------------------------------|
| `version` | number    | Format version (1 or 4)              |
| `entries` | unknown[] | Portrait entries, one per art set    |

<!-- langtabs-start -->
```typescript
interface PortraitSettings {
  version: number;
  entries: unknown[];
}
```
```csharp
public class PortraitSettings
{
    public uint Version { get; set; }
    public List<object> Entries { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
interface UIC {
  version: number;
  source_is_xml: boolean;
  comment: string;
  precache_condition: string;
  hierarchy: Record<string, unknown>;
  components: Record<string, unknown>;
}
```
```csharp
public class UIC
{
    public uint Version { get; set; }
    public bool SourceIsXml { get; set; }
    public string Comment { get; set; }
    public string PrecacheCondition { get; set; }
    public Dictionary<string, object> Hierarchy { get; set; }
    public Dictionary<string, object> Components { get; set; }
}
```
<!-- langtabs-end -->

### UnitVariant

Decoded UnitVariant file.

| Field       | Type      | Description              |
|-------------|-----------|--------------------------|
| `version`   | number    | Version of the UnitVariant |
| `unknown_1` | number    | Unknown field            |
| `categories`| unknown[] | Variant categories       |

<!-- langtabs-start -->
```typescript
interface UnitVariant {
  version: number;
  unknown_1: number;
  categories: unknown[];
}
```
```csharp
public class UnitVariant
{
    public uint Version { get; set; }
    public uint Unknown1 { get; set; }
    public List<object> Categories { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type FieldType =
  | "Boolean" | "F32" | "F64" | "I16" | "I32" | "I64"
  | "ColourRGB" | "StringU8" | "StringU16"
  | "OptionalI16" | "OptionalI32" | "OptionalI64"
  | "OptionalStringU8" | "OptionalStringU16"
  | { SequenceU16: Definition }
  | { SequenceU32: Definition };
```
```csharp
// Mix of plain-string and tagged-enum variants. Model it as a tagged union:
public abstract class FieldType { }
public class FieldTypePrimitive : FieldType { public string Kind { get; set; } }  // e.g. "I32"
public class FieldTypeSequenceU16 : FieldType { public Definition SequenceU16 { get; set; } }
public class FieldTypeSequenceU32 : FieldType { public Definition SequenceU32 { get; set; } }
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
interface Field {
  name: string;
  field_type: FieldType;
  is_key: boolean;
  default_value: string | null;
  is_filename: boolean;
  filename_relative_path: string | null;
  is_reference: [string, string] | null;
  lookup: string[] | null;
  description: string;
  ca_order: number;
  is_bitwise: number;
  enum_values: Record<number, string>;
  is_part_of_colour: number | null;
}
```
```csharp
public class Field
{
    public string Name { get; set; }
    public FieldType FieldType { get; set; }
    public bool IsKey { get; set; }
    public string? DefaultValue { get; set; }
    public bool IsFilename { get; set; }
    public string? FilenameRelativePath { get; set; }
    public Tuple<string, string>? IsReference { get; set; }
    public List<string>? Lookup { get; set; }
    public string Description { get; set; }
    public short CaOrder { get; set; }
    public int IsBitwise { get; set; }
    public Dictionary<int, string> EnumValues { get; set; }
    public byte? IsPartOfColour { get; set; }
}
```
<!-- langtabs-end -->

### Definition

Schema definition for a specific version of a DB table.

| Field                          | Type     | Description                                                  |
|--------------------------------|----------|--------------------------------------------------------------|
| `version`                      | number   | Version number (-1 = fake, 0 = unversioned, 1+ = versioned) |
| `fields`                       | Field[]  | Fields in binary encoding order (see note below)             |
| `localised_fields`             | Field[]  | Fields extracted to LOC files during export                  |
| `localised_key_order`          | number[] | Order of key fields for constructing localisation keys       |

> **Note:** The `fields` list is ordered to match the **binary encoding** of the table, which is not necessarily the order columns appear in the decoded/displayed data. To get fields in decoded column order (with bitwise expansion, enum conversion, and colour merging applied), use the [`FieldsProcessed`](./ws-commands.md#fieldsprocessed) command, passing the `Definition` as input.

<!-- langtabs-start -->
```typescript
interface Definition {
  version: number;
  fields: Field[];
  localised_fields: Field[];
  localised_key_order: number[];
}
```
```csharp
public class Definition
{
    public int Version { get; set; }
    public List<Field> Fields { get; set; }
    public List<Field> LocalisedFields { get; set; }
    public List<uint> LocalisedKeyOrder { get; set; }
}
```
<!-- langtabs-end -->

### Schema

The full schema containing all table definitions for a game.

| Field         | Type                                | Description                        |
|---------------|-------------------------------------|------------------------------------|
| `version`     | number                              | Schema format version (currently 5)|
| `definitions` | Record<string, Definition[]>        | Table name to version definitions  |
| `patches`     | Record<string, DefinitionPatch>     | Table name to patches              |

<!-- langtabs-start -->
```typescript
interface Schema {
  version: number;
  definitions: Record<string, Definition[]>;
  patches: Record<string, DefinitionPatch>;
}
```
```csharp
public class Schema
{
    public ushort Version { get; set; }
    public Dictionary<string, List<Definition>> Definitions { get; set; }
    public Dictionary<string, DefinitionPatch> Patches { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type DefinitionPatch = Record<string, Record<string, string>>;
```
```csharp
// DefinitionPatch is a type alias for the nested map:
using DefinitionPatch = System.Collections.Generic.Dictionary<string, System.Collections.Generic.Dictionary<string, string>>;
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
interface PackSettings {
  settings_text: Record<string, string>;
  settings_string: Record<string, string>;
  settings_bool: Record<string, boolean>;
  settings_number: Record<string, number>;
}
```
```csharp
public class PackSettings
{
    public Dictionary<string, string> SettingsText { get; set; }
    public Dictionary<string, string> SettingsString { get; set; }
    public Dictionary<string, bool> SettingsBool { get; set; }
    public Dictionary<string, int> SettingsNumber { get; set; }
}
```
<!-- langtabs-end -->

### Note

A note attached to a path within the Pack file.

| Field     | Type           | Description                                    |
|-----------|----------------|------------------------------------------------|
| `id`      | number         | Unique note identifier                         |
| `message` | string         | Note content/body                              |
| `url`     | string or null | Optional URL associated with the note          |
| `path`    | string         | Path within the Pack (empty string = global)   |

<!-- langtabs-start -->
```typescript
interface Note {
  id: number;
  message: string;
  url: string | null;
  path: string;
}
```
```csharp
public class Note
{
    public ulong Id { get; set; }
    public string Message { get; set; }
    public string? Url { get; set; }
    public string Path { get; set; }
}
```
<!-- langtabs-end -->

### OptimizerOptions

Configuration for the pack optimizer.

| Field                                       | Type    | Description                                                           |
|---------------------------------------------|---------|-----------------------------------------------------------------------|
| `pack_remove_itm_files`                     | boolean | Remove files unchanged from vanilla                                   |
| `pack_remove_duplicated_files`              | boolean | Remove case-insensitively duplicated files with identical contents    |
| `db_import_datacores_into_twad_key_deletes` | boolean | Import datacored tables into twad_key_deletes                         |
| `db_optimize_datacored_tables`              | boolean | Optimize datacored tables (not recommended)                           |
| `table_remove_duplicated_entries`           | boolean | Remove duplicated rows from DB and Loc files                          |
| `table_remove_itm_entries`                  | boolean | Remove Identical To Master rows                                       |
| `table_remove_itnr_entries`                 | boolean | Remove Identical To New Row rows                                      |
| `table_remove_empty_file`                   | boolean | Remove empty DB and Loc files                                         |
| `text_remove_unused_xml_map_folders`        | boolean | Remove unused XML files in map folders                                |
| `text_remove_unused_xml_prefab_folder`      | boolean | Remove unused XML files in the prefab folder                          |
| `text_remove_agf_files`                     | boolean | Remove unused AGF files                                               |
| `text_remove_model_statistics_files`        | boolean | Remove unused model_statistics files                                  |
| `pts_remove_unused_art_sets`                | boolean | Remove unused art sets in Portrait Settings                           |
| `pts_remove_unused_variants`                | boolean | Remove unused variants from Portrait Settings art sets                |
| `pts_remove_empty_masks`                    | boolean | Remove empty masks in Portrait Settings                               |
| `pts_remove_empty_file`                     | boolean | Remove empty Portrait Settings files                                  |

<!-- langtabs-start -->
```typescript
interface OptimizerOptions {
  pack_remove_itm_files: boolean;
  pack_remove_duplicated_files: boolean;
  db_import_datacores_into_twad_key_deletes: boolean;
  db_optimize_datacored_tables: boolean;
  table_remove_duplicated_entries: boolean;
  table_remove_itm_entries: boolean;
  table_remove_itnr_entries: boolean;
  table_remove_empty_file: boolean;
  text_remove_unused_xml_map_folders: boolean;
  text_remove_unused_xml_prefab_folder: boolean;
  text_remove_agf_files: boolean;
  text_remove_model_statistics_files: boolean;
  pts_remove_unused_art_sets: boolean;
  pts_remove_unused_variants: boolean;
  pts_remove_empty_masks: boolean;
  pts_remove_empty_file: boolean;
}
```
```csharp
public class OptimizerOptions
{
    public bool PackRemoveItmFiles { get; set; }
    public bool PackRemoveDuplicatedFiles { get; set; }
    public bool DbImportDatacoresIntoTwadKeyDeletes { get; set; }
    public bool DbOptimizeDatacoredTables { get; set; }
    public bool TableRemoveDuplicatedEntries { get; set; }
    public bool TableRemoveItmEntries { get; set; }
    public bool TableRemoveItnrEntries { get; set; }
    public bool TableRemoveEmptyFile { get; set; }
    public bool TextRemoveUnusedXmlMapFolders { get; set; }
    public bool TextRemoveUnusedXmlPrefabFolder { get; set; }
    public bool TextRemoveAgfFiles { get; set; }
    public bool TextRemoveModelStatisticsFiles { get; set; }
    public bool PtsRemoveUnusedArtSets { get; set; }
    public bool PtsRemoveUnusedVariants { get; set; }
    public bool PtsRemoveEmptyMasks { get; set; }
    public bool PtsRemoveEmptyFile { get; set; }
}
```
<!-- langtabs-end -->

### SettingsSnapshot

A full batch of settings. Returned by the [`SettingsGetAll`](./ws-commands.md#settingsgetall) command — much cheaper than round-tripping per-key getters when you need many settings at once. Each field is a map from setting key to value for one primitive type.

| Field         | Type                      | Description                    |
|---------------|---------------------------|--------------------------------|
| `bool`        | Record<string, boolean>   | Boolean settings               |
| `i32`         | Record<string, number>    | Signed 32-bit integer settings |
| `f32`         | Record<string, number>    | 32-bit float settings          |
| `string`      | Record<string, string>    | String settings                |
| `raw_data`    | Record<string, number[]>  | Raw byte-array settings        |
| `vec_string`  | Record<string, string[]>  | String-list settings           |

<!-- langtabs-start -->
```typescript
interface SettingsSnapshot {
  bool: Record<string, boolean>;
  i32: Record<string, number>;
  f32: Record<string, number>;
  string: Record<string, string>;
  raw_data: Record<string, number[]>;
  vec_string: Record<string, string[]>;
}
```
```csharp
public class SettingsSnapshot
{
    public Dictionary<string, bool> Bool { get; set; }
    public Dictionary<string, int> I32 { get; set; }
    public Dictionary<string, float> F32 { get; set; }
    public Dictionary<string, string> String { get; set; }
    public Dictionary<string, byte[]> RawData { get; set; }
    public Dictionary<string, List<string>> VecString { get; set; }
}
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
interface Translation {
  key: string;
  value_original: string;
  value_translated: string;
  needs_retranslation: boolean;
  removed: boolean;
}
```
```csharp
public class Translation
{
    public string Key { get; set; }
    public string ValueOriginal { get; set; }
    public string ValueTranslated { get; set; }
    public bool NeedsRetranslation { get; set; }
    public bool Removed { get; set; }
}
```
<!-- langtabs-end -->

### PackTranslation

Translation data for a pack in a specific language.

| Field          | Type                          | Description                     |
|----------------|-------------------------------|---------------------------------|
| `language`     | string                        | Target language code (e.g. `"es"`, `"de"`) |
| `pack_name`    | string                        | Name of the pack               |
| `translations` | Record<string, Translation>   | Loc key to translation data    |

<!-- langtabs-start -->
```typescript
interface PackTranslation {
  language: string;
  pack_name: string;
  translations: Record<string, Translation>;
}
```
```csharp
public class PackTranslation
{
    public string Language { get; set; }
    public string PackName { get; set; }
    public Dictionary<string, Translation> Translations { get; set; }
}
```
<!-- langtabs-end -->

---

## Diagnostics Types

### Diagnostics

Diagnostics report configuration and results.

| Field                 | Type             | Description                                          |
|-----------------------|------------------|------------------------------------------------------|
| `folders_ignored`     | string[]         | Folder paths excluded from checks                    |
| `files_ignored`       | string[]         | File paths excluded from checks                      |
| `fields_ignored`      | string[]         | Table fields excluded (`"table_name/field_name"`)    |
| `diagnostics_ignored` | string[]         | Diagnostic type identifiers to skip                  |
| `results`             | DiagnosticType[] | Diagnostic results from the most recent check        |

<!-- langtabs-start -->
```typescript
interface Diagnostics {
  folders_ignored: string[];
  files_ignored: string[];
  fields_ignored: string[];
  diagnostics_ignored: string[];
  results: DiagnosticType[];
}
```
```csharp
public class Diagnostics
{
    public List<string> FoldersIgnored { get; set; }
    public List<string> FilesIgnored { get; set; }
    public List<string> FieldsIgnored { get; set; }
    public List<string> DiagnosticsIgnored { get; set; }
    public List<DiagnosticType> Results { get; set; }
}
```
<!-- langtabs-end -->

### DiagnosticType

A single diagnostic result. Serialized as a tagged enum — the variant name indicates which kind of diagnostic fired:

| Variant              | Payload                       | Description                           |
|----------------------|-------------------------------|---------------------------------------|
| `AnimFragmentBattle` | AnimFragmentBattleDiagnostic  | Issues in an AnimFragmentBattle file  |
| `Config`             | ConfigDiagnostic              | Pack-level configuration issue        |
| `Dependency`         | DependencyDiagnostic          | Broken/missing dependency reference   |
| `DB`                 | TableDiagnostic               | Issue in a DB table                   |
| `Loc`                | TableDiagnostic               | Issue in a Loc file                   |
| `Pack`               | PackDiagnostic                | Pack-wide issue                       |
| `PortraitSettings`   | PortraitSettingsDiagnostic    | Issue in a PortraitSettings file      |
| `Text`               | TextDiagnostic                | Issue in a text file                  |

Each payload is a struct with at least `path: string` and `results: <SpecificDiagnosticReport>[]` (the concrete inner fields vary by diagnostic kind and are treated as opaque by this doc). The full structures live in `rpfm_extensions::diagnostics` — e.g. `TableDiagnostic`, `PackDiagnostic`, `ConfigDiagnostic`, and so on. Clients that only need to display diagnostic results can round-trip the JSON without decoding these payloads.

<!-- langtabs-start -->
```typescript
// The inner payloads are left opaque here — their shape depends on the diagnostic kind.
type DiagnosticType =
  | { AnimFragmentBattle: unknown }
  | { Config: unknown }
  | { Dependency: unknown }
  | { DB: unknown }
  | { Loc: unknown }
  | { Pack: unknown }
  | { PortraitSettings: unknown }
  | { Text: unknown };
```
```csharp
public abstract class DiagnosticType { }
public class DiagnosticTypeAnimFragmentBattle : DiagnosticType { public object AnimFragmentBattle { get; set; } }
public class DiagnosticTypeConfig             : DiagnosticType { public object Config { get; set; } }
public class DiagnosticTypeDependency         : DiagnosticType { public object Dependency { get; set; } }
public class DiagnosticTypeDB                 : DiagnosticType { public object DB { get; set; } }
public class DiagnosticTypeLoc                : DiagnosticType { public object Loc { get; set; } }
public class DiagnosticTypePack               : DiagnosticType { public object Pack { get; set; } }
public class DiagnosticTypePortraitSettings   : DiagnosticType { public object PortraitSettings { get; set; } }
public class DiagnosticTypeText               : DiagnosticType { public object Text { get; set; } }
```
<!-- langtabs-end -->

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

<!-- langtabs-start -->
```typescript
type APIResponse =
  | { NewBetaUpdate: string }
  | { NewStableUpdate: string }
  | { NewUpdateHotfix: string }
  | "NoUpdate"
  | "UnknownVersion";
```
```csharp
// Serialize as either a wrapped object { "<Variant>": "version" } or a bare string.
public abstract class APIResponse { }
public class APIResponseNewBetaUpdate   : APIResponse { public string NewBetaUpdate { get; set; } }
public class APIResponseNewStableUpdate : APIResponse { public string NewStableUpdate { get; set; } }
public class APIResponseNewUpdateHotfix : APIResponse { public string NewUpdateHotfix { get; set; } }
public class APIResponseNoUpdate        : APIResponse { }
public class APIResponseUnknownVersion  : APIResponse { }
```
<!-- langtabs-end -->

### GitResponse

Response from a git-based update check (schemas, translations, etc.):

| Value            | Description                              |
|------------------|------------------------------------------|
| `"NewUpdate"`    | A new update is available on the remote  |
| `"NoUpdate"`     | The local repository is up to date       |
| `"NoLocalFiles"` | No local copy exists (needs cloning)     |
| `"Diverged"`     | Local and remote branches have diverged  |

<!-- langtabs-start -->
```typescript
type GitResponse = "NewUpdate" | "NoUpdate" | "NoLocalFiles" | "Diverged";
```
```csharp
public enum GitResponse { NewUpdate, NoUpdate, NoLocalFiles, Diverged }
```
<!-- langtabs-end -->

---

## Search Types

### SearchSource

Which data source to search. Serialized as a tagged enum — the `Pack` variant carries a pack key, the rest are plain strings:

| Variant          | Payload  | Description                                |
|------------------|----------|--------------------------------------------|
| `Pack`           | `string` | A specific open pack, identified by pack key |
| `ParentFiles`    | *(none)* | Parent mod dependencies                    |
| `GameFiles`      | *(none)* | Vanilla game files                         |
| `AssKitFiles`    | *(none)* | Assembly Kit files                         |

Example:

```json
{ "Pack": "my_mod.pack" }
"GameFiles"
```

<!-- langtabs-start -->
```typescript
type SearchSource =
  | { Pack: string }
  | "ParentFiles"
  | "GameFiles"
  | "AssKitFiles";
```
```csharp
// Serialize as either { "Pack": "pack_key" } or a bare string.
public abstract class SearchSource { }
public class SearchSourcePack        : SearchSource { public string Pack { get; set; } }
public class SearchSourceParentFiles : SearchSource { }
public class SearchSourceGameFiles   : SearchSource { }
public class SearchSourceAssKitFiles : SearchSource { }
```
<!-- langtabs-end -->

### SearchOn

Boolean flags for which file types to include in a search. Each field corresponds to a file type:

`anim`, `anim_fragment_battle`, `anim_pack`, `anims_table`, `atlas`, `audio`, `bmd`, `db`, `esf`, `group_formations`, `image`, `loc`, `matched_combat`, `pack`, `portrait_settings`, `rigid_model`, `sound_bank`, `text`, `uic`, `unit_variant`, `unknown`, `video`, `schema`

All fields are `boolean`.

<!-- langtabs-start -->
```typescript
interface SearchOn {
  anim: boolean;
  anim_fragment_battle: boolean;
  anim_pack: boolean;
  anims_table: boolean;
  atlas: boolean;
  audio: boolean;
  bmd: boolean;
  db: boolean;
  esf: boolean;
  group_formations: boolean;
  image: boolean;
  loc: boolean;
  matched_combat: boolean;
  pack: boolean;
  portrait_settings: boolean;
  rigid_model: boolean;
  sound_bank: boolean;
  text: boolean;
  uic: boolean;
  unit_variant: boolean;
  unknown: boolean;
  video: boolean;
  schema: boolean;
}
```
```csharp
public class SearchOn
{
    public bool Anim { get; set; }
    public bool AnimFragmentBattle { get; set; }
    public bool AnimPack { get; set; }
    public bool AnimsTable { get; set; }
    public bool Atlas { get; set; }
    public bool Audio { get; set; }
    public bool Bmd { get; set; }
    public bool Db { get; set; }
    public bool Esf { get; set; }
    public bool GroupFormations { get; set; }
    public bool Image { get; set; }
    public bool Loc { get; set; }
    public bool MatchedCombat { get; set; }
    public bool Pack { get; set; }
    public bool PortraitSettings { get; set; }
    public bool RigidModel { get; set; }
    public bool SoundBank { get; set; }
    public bool Text { get; set; }
    public bool Uic { get; set; }
    public bool UnitVariant { get; set; }
    public bool Unknown { get; set; }
    public bool Video { get; set; }
    public bool Schema { get; set; }
}
```
<!-- langtabs-end -->

### GlobalSearch

Global search configuration and results.

| Field            | Type         | Description                              |
|------------------|--------------|------------------------------------------|
| `pattern`        | string         | Text pattern or regex to search for      |
| `replace_text`   | string         | Replacement text                         |
| `case_sensitive` | boolean        | Whether the search is case-sensitive     |
| `use_regex`      | boolean        | Whether the pattern is a regular expression |
| `sources`        | SearchSource[] | One or more data sources to search       |
| `search_on`      | SearchOn       | Which file types to search               |
| `matches`        | Matches        | Results from the most recent search      |
| `game_key`       | string         | Game key for the files being searched    |

<!-- langtabs-start -->
```typescript
interface GlobalSearch {
  pattern: string;
  replace_text: string;
  case_sensitive: boolean;
  use_regex: boolean;
  sources: SearchSource[];
  search_on: SearchOn;
  matches: Matches;
  game_key: string;
}
```
```csharp
public class GlobalSearch
{
    public string Pattern { get; set; }
    public string ReplaceText { get; set; }
    public bool CaseSensitive { get; set; }
    public bool UseRegex { get; set; }
    public List<SearchSource> Sources { get; set; }
    public SearchOn SearchOn { get; set; }
    public Matches Matches { get; set; }
    public string GameKey { get; set; }
}
```
<!-- langtabs-end -->

### Matches

The results of a global search, grouped by file type. Each field is an array of per-type match containers (defined in the next section) — except `schema`, which is a single `SchemaMatches` container.

| Field                  | Type                          | Description                                      |
|------------------------|-------------------------------|--------------------------------------------------|
| `anim`                 | UnknownMatches[]              | Matches in animation files                       |
| `anim_fragment_battle` | AnimFragmentBattleMatches[]   | Matches in battle animation fragments            |
| `anim_pack`            | UnknownMatches[]              | Matches in animation packs                       |
| `anims_table`          | UnknownMatches[]              | Matches in animation tables                      |
| `atlas`                | AtlasMatches[]                | Matches in atlases                               |
| `audio`                | UnknownMatches[]              | Matches in audio files                           |
| `bmd`                  | UnknownMatches[]              | Matches in BMD files                             |
| `db`                   | TableMatches[]                | Matches in DB tables                             |
| `esf`                  | UnknownMatches[]              | Matches in ESF files                             |
| `group_formations`     | UnknownMatches[]              | Matches in group formations                      |
| `image`                | UnknownMatches[]              | Matches in images                                |
| `loc`                  | TableMatches[]                | Matches in Loc tables                            |
| `matched_combat`       | UnknownMatches[]              | Matches in matched combat files                  |
| `pack`                 | UnknownMatches[]              | Matches in nested Pack files                     |
| `portrait_settings`    | PortraitSettingsMatches[]     | Matches in portrait settings                     |
| `rigid_model`          | RigidModelMatches[]           | Matches in RigidModel files                      |
| `sound_bank`           | UnknownMatches[]              | Matches in sound banks                           |
| `text`                 | TextMatches[]                 | Matches in text files (Lua, XML, …)              |
| `uic`                  | UnknownMatches[]              | Matches in UIC files                             |
| `unit_variant`         | UnitVariantMatches[]          | Matches in UnitVariant files                     |
| `unknown`              | UnknownMatches[]              | Matches in unclassified files                    |
| `video`                | UnknownMatches[]              | Matches in video files                           |
| `schema`               | SchemaMatches                 | Matches in the schema (single container, not an array) |

<!-- langtabs-start -->
```typescript
interface Matches {
  anim: UnknownMatches[];
  anim_fragment_battle: AnimFragmentBattleMatches[];
  anim_pack: UnknownMatches[];
  anims_table: UnknownMatches[];
  atlas: AtlasMatches[];
  audio: UnknownMatches[];
  bmd: UnknownMatches[];
  db: TableMatches[];
  esf: UnknownMatches[];
  group_formations: UnknownMatches[];
  image: UnknownMatches[];
  loc: TableMatches[];
  matched_combat: UnknownMatches[];
  pack: UnknownMatches[];
  portrait_settings: PortraitSettingsMatches[];
  rigid_model: RigidModelMatches[];
  sound_bank: UnknownMatches[];
  text: TextMatches[];
  uic: UnknownMatches[];
  unit_variant: UnitVariantMatches[];
  unknown: UnknownMatches[];
  video: UnknownMatches[];
  schema: SchemaMatches;
}
```
```csharp
public class Matches
{
    public List<MatchContainer<UnknownMatch>> Anim { get; set; }
    public List<MatchContainer<AnimFragmentBattleMatch>> AnimFragmentBattle { get; set; }
    public List<MatchContainer<UnknownMatch>> AnimPack { get; set; }
    public List<MatchContainer<UnknownMatch>> AnimsTable { get; set; }
    public List<MatchContainer<AtlasMatch>> Atlas { get; set; }
    public List<MatchContainer<UnknownMatch>> Audio { get; set; }
    public List<MatchContainer<UnknownMatch>> Bmd { get; set; }
    public List<MatchContainer<TableMatch>> Db { get; set; }
    public List<MatchContainer<UnknownMatch>> Esf { get; set; }
    public List<MatchContainer<UnknownMatch>> GroupFormations { get; set; }
    public List<MatchContainer<UnknownMatch>> Image { get; set; }
    public List<MatchContainer<TableMatch>> Loc { get; set; }
    public List<MatchContainer<UnknownMatch>> MatchedCombat { get; set; }
    public List<MatchContainer<UnknownMatch>> Pack { get; set; }
    public List<MatchContainer<PortraitSettingsMatch>> PortraitSettings { get; set; }
    public List<MatchContainer<RigidModelMatch>> RigidModel { get; set; }
    public List<MatchContainer<UnknownMatch>> SoundBank { get; set; }
    public List<MatchContainer<TextMatch>> Text { get; set; }
    public List<MatchContainer<UnknownMatch>> Uic { get; set; }
    public List<MatchContainer<UnitVariantMatch>> UnitVariant { get; set; }
    public List<MatchContainer<UnknownMatch>> Unknown { get; set; }
    public List<MatchContainer<UnknownMatch>> Video { get; set; }
    public MatchContainer<SchemaMatch> Schema { get; set; }
}
```
<!-- langtabs-end -->

### Match Types

Search results use specialized match types per file format. All match containers share the same wrapper shape:

| Field            | Type         | Description                                |
|------------------|--------------|--------------------------------------------|
| `path`           | string       | Internal path of the file the matches came from |
| `source`         | SearchSource | Data source the file was searched in       |
| `container_name` | string       | Name of the containing Pack (or empty)     |
| `matches`        | `<Match>[]`  | Match entries — type depends on file kind  |

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

All match container types (`TableMatches`, `TextMatches`, `AnimFragmentBattleMatches`, etc.) use the wrapper shape from the top of this section — `{ path, source, container_name, matches }` — with `matches` being an array of the corresponding per-type match struct.

<!-- langtabs-start -->
```typescript
interface TableMatch {
  column_name: string;
  column_number: number;
  row_number: number;
  start: number;
  end: number;
  text: string;
}

interface TextMatch {
  row: number;
  start: number;
  end: number;
  text: string;
}

interface UnknownMatch {
  pos: number;
  len: number;
}

interface AnimFragmentBattleMatch {
  skeleton_name: boolean;
  table_name: boolean;
  mount_table_name: boolean;
  unmount_table_name: boolean;
  locomotion_graph: boolean;
  entry:
    | [number, [number, boolean, boolean, boolean] | null, boolean, boolean, boolean, boolean, boolean]
    | null;
  start: number;
  end: number;
  text: string;
}

interface AtlasMatch extends TableMatch {}  // same shape as TableMatch

interface PortraitSettingsMatch {
  entry: number;
  id: boolean;
  camera_settings_head: boolean;
  camera_settings_body: boolean;
  variant: [number, boolean, boolean, boolean, boolean, boolean] | null;
  start: number;
  end: number;
  text: string;
}

interface RigidModelMatch {
  skeleton_id: boolean;
  mesh_value: [number, number] | null;
  mesh_name: boolean;
  mesh_mat_name: boolean;
  mesh_textute_directory: boolean;
  mesh_filters: boolean;
  mesh_att_point_name: number | null;
  mesh_texture_path: number | null;
  start: number;
  end: number;
  text: string;
}

interface UnitVariantMatch {
  entry: number;
  name: boolean;
  variant: [number, boolean, boolean] | null;
  start: number;
  end: number;
  text: string;
}

interface SchemaMatch {
  table_name: string;
  version: number;
  column: number;
  column_name: string;
}

// The wrapper that groups matches for one file, used by every *Matches type:
interface MatchContainer<M> {
  path: string;
  source: SearchSource;
  container_name: string;
  matches: M[];
}
type TableMatches              = MatchContainer<TableMatch>;
type TextMatches               = MatchContainer<TextMatch>;
type UnknownMatches            = MatchContainer<UnknownMatch>;
type AnimFragmentBattleMatches = MatchContainer<AnimFragmentBattleMatch>;
type AtlasMatches              = MatchContainer<AtlasMatch>;
type PortraitSettingsMatches   = MatchContainer<PortraitSettingsMatch>;
type RigidModelMatches         = MatchContainer<RigidModelMatch>;
type UnitVariantMatches        = MatchContainer<UnitVariantMatch>;
type SchemaMatches             = MatchContainer<SchemaMatch>;
```
```csharp
public class TableMatch
{
    public string ColumnName { get; set; }
    public uint ColumnNumber { get; set; }
    public long RowNumber { get; set; }
    public long Start { get; set; }
    public long End { get; set; }
    public string Text { get; set; }
}

public class TextMatch
{
    public ulong Row { get; set; }
    public long Start { get; set; }
    public long End { get; set; }
    public string Text { get; set; }
}

public class UnknownMatch
{
    public long Pos { get; set; }
    public long Len { get; set; }
}

// AtlasMatch has the same shape as TableMatch.
public class AtlasMatch : TableMatch { }

public class SchemaMatch
{
    public string TableName { get; set; }
    public int Version { get; set; }
    public int Column { get; set; }
    public string ColumnName { get; set; }
}

// AnimFragmentBattleMatch, PortraitSettingsMatch, RigidModelMatch and UnitVariantMatch
// follow the tables above; field-for-field translations are straightforward
// (booleans → bool, nullable tuples → Tuple<...>? or a named class).

// Wrapper shared by every *Matches type:
public class MatchContainer<M>
{
    public string Path { get; set; }
    public SearchSource Source { get; set; }
    public string ContainerName { get; set; }
    public List<M> Matches { get; set; }
}
```
<!-- langtabs-end -->

### MatchHolder

A tagged enum wrapping a single file type's matches. The variant name indicates the file type:

```json
{ "Db": { "path": "db/units_tables/data", "matches": [...] } }
{ "Text": { "path": "script/campaign/mod.lua", "matches": [...] } }
```

Variants: `Anim`, `AnimFragmentBattle`, `AnimPack`, `AnimsTable`, `Atlas`, `Audio`, `Bmd`, `Db`, `Esf`, `GroupFormations`, `Image`, `Loc`, `MatchedCombat`, `Pack`, `PortraitSettings`, `RigidModel`, `SoundBank`, `Text`, `Uic`, `UnitVariant`, `Unknown`, `Video`, `Schema`.

<!-- langtabs-start -->
```typescript
// Most variants carry an UnknownMatches; only the rich-formatted file types carry their own match container.
type MatchHolder =
  | { Anim: UnknownMatches }
  | { AnimFragmentBattle: AnimFragmentBattleMatches }
  | { AnimPack: UnknownMatches }
  | { AnimsTable: UnknownMatches }
  | { Atlas: AtlasMatches }
  | { Audio: UnknownMatches }
  | { Bmd: UnknownMatches }
  | { Db: TableMatches }
  | { Esf: UnknownMatches }
  | { GroupFormations: UnknownMatches }
  | { Image: UnknownMatches }
  | { Loc: TableMatches }
  | { MatchedCombat: UnknownMatches }
  | { Pack: UnknownMatches }
  | { PortraitSettings: PortraitSettingsMatches }
  | { RigidModel: RigidModelMatches }
  | { SoundBank: UnknownMatches }
  | { Text: TextMatches }
  | { Uic: UnknownMatches }
  | { UnitVariant: UnitVariantMatches }
  | { Unknown: UnknownMatches }
  | { Video: UnknownMatches }
  | { Schema: SchemaMatches };
```
```csharp
// One wrapper class per variant. Only the most common ones shown — pattern is identical for the rest.
public abstract class MatchHolder { }
public class MatchHolderDb   : MatchHolder { public MatchContainer<TableMatch> Db { get; set; } }
public class MatchHolderLoc  : MatchHolder { public MatchContainer<TableMatch> Loc { get; set; } }
public class MatchHolderText : MatchHolder { public MatchContainer<TextMatch> Text { get; set; } }
public class MatchHolderSchema : MatchHolder { public MatchContainer<SchemaMatch> Schema { get; set; } }
// ... one class per variant listed above; most wrap MatchContainer<UnknownMatch>.
```
<!-- langtabs-end -->
