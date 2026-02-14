/**
 * RPFM IPC Protocol — TypeScript Reference
 *
 * This file documents the WebSocket-based IPC protocol used to communicate
 * with the RPFM server (rpfm_server). All messages are JSON-serialized and
 * sent/received over a WebSocket connection to ws://127.0.0.1:45127.
 *
 * Every request is wrapped in a {@link Message} with a unique `id` field.
 * The server responds with a {@link Message} carrying the same `id`, allowing
 * the client to correlate responses to requests even when multiple requests
 * are in flight simultaneously.
 *
 * ## Quick Start
 *
 * ```ts
 * const ws = new WebSocket("ws://127.0.0.1:45127");
 *
 * let nextId = 1;
 * let currentSessionId: number | null = null;
 *
 * function send(command: Command): number {
 *   const id = nextId++;
 *   ws.send(JSON.stringify({ id, data: command }));
 *   return id;
 * }
 *
 * // Listen for responses
 * ws.onmessage = (event) => {
 *   const msg: Message<Response> = JSON.parse(event.data);
 *
 *   // Handle the SessionConnected message sent immediately after connection
 *   if (typeof msg.data === "object" && "SessionConnected" in msg.data) {
 *     currentSessionId = msg.data.SessionConnected;
 *     console.log(`Connected to session ${currentSessionId}`);
 *     return;
 *   }
 *
 *   console.log(`Response for request ${msg.id}:`, msg.data);
 * };
 *
 * // Open a pack file
 * ws.onopen = () => {
 *   send({ OpenPackFiles: [["/path/to/my_mod.pack"]] });
 * };
 * ```
 *
 * ## REST Endpoints
 *
 * In addition to the WebSocket protocol, the server exposes REST endpoints:
 *
 * - `GET /sessions` - Returns a JSON array of {@link SessionInfo} objects
 *   describing all active sessions. Useful for session management UIs.
 *
 * ## Session Reconnection
 *
 * To reconnect to an existing session, append `?session_id=<id>` to the WebSocket URL:
 * ```ts
 * const ws = new WebSocket("ws://127.0.0.1:45127/ws?session_id=123");
 * ```
 *
 * ## Serialization Convention
 *
 * Rust enums are serialized by serde as follows:
 * - Unit variants:    `"VariantName"`
 * - Newtype variants: `{ "VariantName": value }`
 * - Tuple variants:   `{ "VariantName": [v1, v2, ...] }`
 *
 * Generated from: rpfm_ipc/src/messages.rs and rpfm_ipc/src/helpers.rs
 */

// ---------------------------------------------------------------------------
// Message Wrapper
// ---------------------------------------------------------------------------

/**
 * Generic message wrapper that adds request-response correlation via unique IDs.
 *
 * Every command sent to the server and every response received is wrapped in
 * this structure. The `id` field must be unique per request so the client can
 * match responses to their originating commands.
 */
export interface Message<T> {
  id: number;
  data: T;
}

// ---------------------------------------------------------------------------
// Helper / Shared Types
// ---------------------------------------------------------------------------

/** Discriminates where file data originates from. */
export type DataSource =
  | "PackFile"      // Current open PackFile
  | "GameFiles"     // Vanilla game files
  | "ParentFiles"   // Parent mod files
  | "AssKitFiles"   // Assembly Kit files
  | "ExternalFile"; // External file on disk

/** Metadata about a packed file within a container. */
export interface RFileInfo {
  path: string;
  container_name: string | null;
  timestamp: number | null;
  file_type: string; // FileType enum value, e.g. "DB", "Loc", "Text", etc.
}

/** Reduced representation of a PackFile (container-level metadata). */
export interface ContainerInfo {
  file_name: string;
  file_path: string;
  pfh_version: string;   // PFHVersion enum value
  pfh_file_type: string;  // PFHFileType enum value
  bitmask: unknown;       // PFHFlags bitmask
  compress: string;       // CompressionFormat enum value
  timestamp: number;
}

/** Metadata specific to video files. */
export interface VideoInfo {
  format: string;          // SupportedFormats enum value
  version: number;
  codec_four_cc: string;
  width: number;
  height: number;
  num_frames: number;
  framerate: number;
}

/** Dependency paths information for tree view population. */
export interface DependenciesInfo {
  asskit_tables: RFileInfo[];
  vanilla_packed_files: RFileInfo[];
  parent_packed_files: RFileInfo[];
}

/** Information about an active session on the server. */
export interface SessionInfo {
  /** Unique identifier for the session. */
  session_id: number;
  /** Number of active connections to this session. */
  connection_count: number;
  /** Seconds remaining until session timeout (null if session has active connections). */
  timeout_remaining_secs: number | null;
  /** Whether the session has been marked for shutdown. */
  is_shutting_down: boolean;
  /** Name of the pack file currently open in this session (if any). */
  pack_name: string | null;
}

/** Parameters for creating a new packed file. */
export type NewFile =
  | { AnimPack: string }                                   // file name
  | { DB: [string, string, number] }                       // [file_name, table_name, version]
  | { Loc: string }                                        // table name
  | { PortraitSettings: [string, number, [string, string][]] } // [name, version, clone_entries]
  | { Text: [string, string] }                             // [file_name, text_format]
  | { VMD: string }                                        // file name
  | { WSModel: string };                                   // file name

/** A file path within a container. Serialized as `{ File: "path" }` or `{ Folder: "path" }`. */
export type ContainerPath =
  | { File: string }
  | { Folder: string };

/** Response from an update check. */
export type APIResponse =
  | { NewBetaUpdate: string }
  | { NewStableUpdate: string }
  | { NewUpdateHotfix: string }
  | "NoUpdate"
  | "UnknownVersion";

/** Git operation response. */
export type GitResponse =
  | "NewUpdate"      // A new update is available on the remote
  | "NoUpdate"       // The local repository is up-to-date
  | "NoLocalFiles"   // No local copy exists (needs cloning)
  | "Diverged";      // Local and remote branches have diverged

/** Optimizer configuration options. */
export interface OptimizerOptions {
  /** Allow removing files unchanged from vanilla. */
  pack_remove_itm_files: boolean;
  /** Import datacored tables into twad_key_deletes (doesn't delete datacored tables). */
  db_import_datacores_into_twad_key_deletes: boolean;
  /** Allow optimizing datacored tables (not recommended). */
  db_optimize_datacored_tables: boolean;
  /** Allow removing duplicated rows from db and loc files. */
  table_remove_duplicated_entries: boolean;
  /** Allow removing ITM (Identical To Master) rows from db and loc files. */
  table_remove_itm_entries: boolean;
  /** Allow removing ITNR (Identical To New Row) rows from db and loc files. */
  table_remove_itnr_entries: boolean;
  /** Allow removing empty db and loc files. */
  table_remove_empty_file: boolean;
  /** Allow removing unused xml files in map folders. */
  text_remove_unused_xml_map_folders: boolean;
  /** Allow removing unused xml files in the prefab folder. */
  text_remove_unused_xml_prefab_folder: boolean;
  /** Allow removing unused agf files. */
  text_remove_agf_files: boolean;
  /** Allow removing unused model_statistics files. */
  text_remove_model_statistics_files: boolean;
  /** Allow removing unused art sets in Portrait Settings files. */
  pts_remove_unused_art_sets: boolean;
  /** Allow removing unused variants from art sets in Portrait Settings files. */
  pts_remove_unused_variants: boolean;
  /** Allow removing empty masks in Portrait Settings files. */
  pts_remove_empty_masks: boolean;
  /** Allow removing empty Portrait Settings files. */
  pts_remove_empty_file: boolean;
}

/** Field data type in schema definitions. */
export type FieldType =
  | "Boolean"
  | "F32"
  | "F64"
  | "I16"
  | "I32"
  | "I64"
  | "ColourRGB"
  | "StringU8"
  | "StringU16"
  | "OptionalI16"
  | "OptionalI32"
  | "OptionalI64"
  | "OptionalStringU8"
  | "OptionalStringU16"
  | { SequenceU16: Definition }
  | { SequenceU32: Definition };

/** A single field descriptor within a Definition. */
export interface Field {
  /** Name of the field. */
  name: string;
  /** Data type of the field. */
  field_type: FieldType;
  /** Whether this field is part of the table's primary key. */
  is_key: boolean;
  /** Default value for this field when creating new rows. */
  default_value: string | null;
  /** Whether this field contains a filename/path. */
  is_filename: boolean;
  /** Semicolon-separated list of relative paths where files can be found. */
  filename_relative_path: string | null;
  /** Foreign key reference to another table: [table_name, column_name]. */
  is_reference: [string, string] | null;
  /** Additional columns from the referenced table to show in lookups. */
  lookup: string[] | null;
  /** Human-readable description of the field's purpose. */
  description: string;
  /** Visual position in CA's Assembly Kit table editor (-1 = unknown). */
  ca_order: number;
  /** Number of boolean columns this field should be split into. */
  is_bitwise: number;
  /** Named values for this field when treated as an enum. Map of integer values to string names. */
  enum_values: Record<number, string>;
  /** Index of the RGB colour group this field belongs to (null if not part of a colour). */
  is_part_of_colour: number | null;
}

/** Patch applied to a schema definition. Map of field names to patch properties. */
export type DefinitionPatch = Record<string, Record<string, string>>;

/** Schema definition for a DB table version. */
export interface Definition {
  /** Version number (-1=fake, 0=unversioned, 1+=versioned). */
  version: number;
  /** List of fields in binary order. */
  fields: Field[];
  /** Processed fields with bitwise expansion, enum conversion, and RGB colour merging applied. */
  fields_processed_serialized: Field[];
  /** Fields extracted to LOC files during export. */
  localised_fields: Field[];
  /** Order of key fields when constructing localisation keys. */
  localised_key_order: number[];
}

/** Full schema containing all table definitions. */
export interface Schema {
  /** Structural version of the schema format (currently 5). */
  version: number;
  /** Map of table names to their version definitions. */
  definitions: Record<string, Definition[]>;
  /** Map of table names to their patches. */
  patches: Record<string, DefinitionPatch>;
}

/** A note attached to a path in the PackFile. */
export interface Note {
  /** Unique identifier for this note. */
  id: number;
  /** The main content/body of the note. */
  message: string;
  /** Optional URL associated with the note. */
  url: string | null;
  /** Path within the PackFile (empty string = global note). */
  path: string;
}

/** PackFile-level settings. */
export interface PackSettings {
  /** Multi-line text settings (e.g., file ignore lists). */
  settings_text: Record<string, string>;
  /** Single-line string settings (e.g., compression format). */
  settings_string: Record<string, string>;
  /** Boolean flag settings (e.g., disable_autosaves). */
  settings_bool: Record<string, boolean>;
  /** Integer settings (e.g., thresholds, limits). */
  settings_number: Record<string, number>;
}

/** Translation entry for a single Loc key. */
export interface Translation {
  /** The Loc key identifying this string. */
  key: string;
  /** Original text in the base language (typically English). */
  value_original: string;
  /** Translated text in the target language (may be empty). */
  value_translated: string;
  /** Whether this translation needs review due to source changes. */
  needs_retranslation: boolean;
}

/** Pack translation data for a language. */
export interface PackTranslation {
  /** Target language code (e.g., "es", "de", "fr"). */
  language: string;
  /** Name of the pack these translations belong to. */
  pack_name: string;
  /** Map of Loc keys to their translation data. */
  translations: Record<string, Translation>;
}

/** Diagnostics report for the open PackFile. */
export interface Diagnostics {
  /** Folder paths to exclude from diagnostic checks. */
  folders_ignored: string[];
  /** File paths to exclude from diagnostic checks. */
  files_ignored: string[];
  /** Table fields to exclude (format: "table_name/field_name"). */
  fields_ignored: string[];
  /** Diagnostic type identifiers to skip. */
  diagnostics_ignored: string[];
  /** Diagnostic results from the most recent check. */
  results: unknown[];
}

/** Data source for search operations. */
export type SearchSource =
  | "Pack"         // Currently loaded pack
  | "ParentFiles"  // Parent mod dependencies
  | "GameFiles"    // Vanilla game files
  | "AssKitFiles"; // Assembly Kit files

/** Configuration for which file types to include in a search. */
export interface SearchOn {
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

/** Match in an unknown/binary file. */
export interface UnknownMatch {
  /** Byte position of the match. */
  pos: number;
  /** Length of the matched pattern in bytes. */
  len: number;
}

/** Matches for unknown/binary files. */
export interface UnknownMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: UnknownMatch[];
}

/** Match in a table row. */
export interface TableMatch {
  /** Name of the column where the match is. */
  column_name: string;
  /** Logical index of the column (-1 if hidden). */
  column_number: number;
  /** Row number of this match (-1 if hidden by filter). */
  row_number: number;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The contents of the matched cell. */
  text: string;
}

/** Matches for table files (DB & Loc). */
export interface TableMatches {
  /** Path of the table. */
  path: string;
  /** List of matches within the table. */
  matches: TableMatch[];
}

/** Match in a text file. */
export interface TextMatch {
  /** Row of the first character of the match. */
  row: number;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** Line of text containing the match. */
  text: string;
}

/** Matches for text files. */
export interface TextMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: TextMatch[];
}

/** Match in an AnimFragmentBattle file. */
export interface AnimFragmentBattleMatch {
  /** Whether the match is in the skeleton name. */
  skeleton_name: boolean;
  /** Whether the match is in the table name. */
  table_name: boolean;
  /** Whether the match is in the mount table name. */
  mount_table_name: boolean;
  /** Whether the match is in the unmount table name. */
  unmount_table_name: boolean;
  /** Whether the match is in the locomotion graph. */
  locomotion_graph: boolean;
  /** Entry match: [entry_index, sub_entry, ...flags]. */
  entry: [number, [number, boolean, boolean, boolean] | null, boolean, boolean, boolean, boolean, boolean] | null;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The matched text. */
  text: string;
}

/** Matches for AnimFragmentBattle files. */
export interface AnimFragmentBattleMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: AnimFragmentBattleMatch[];
}

/** Match in an Atlas file (same structure as TableMatch). */
export interface AtlasMatch {
  /** Name of the column where the match is. */
  column_name: string;
  /** Logical index of the column. */
  column_number: number;
  /** Row number of this match. */
  row_number: number;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The contents of the matched cell. */
  text: string;
}

/** Matches for Atlas files. */
export interface AtlasMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: AtlasMatch[];
}

/** Match in a PortraitSettings file. */
export interface PortraitSettingsMatch {
  /** Index of the entry in the PortraitSettings file. */
  entry: number;
  /** Whether the match is in the id field. */
  id: boolean;
  /** Whether the match is in camera_settings_head (skeleton node). */
  camera_settings_head: boolean;
  /** Whether the match is in camera_settings_body (skeleton node). */
  camera_settings_body: boolean;
  /** Variant match: [variant_index, ...flags] or null. */
  variant: [number, boolean, boolean, boolean, boolean, boolean] | null;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The matched text. */
  text: string;
}

/** Matches for PortraitSettings files. */
export interface PortraitSettingsMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: PortraitSettingsMatch[];
}

/** Match in a RigidModel file. */
export interface RigidModelMatch {
  /** Whether the match is in the skeleton id. */
  skeleton_id: boolean;
  /** Mesh location: [lod_index, mesh_index] or null. */
  mesh_value: [number, number] | null;
  /** Whether the match is in the mesh name. */
  mesh_name: boolean;
  /** Whether the match is in the material name. */
  mesh_mat_name: boolean;
  /** Whether the match is in the texture directory. */
  mesh_textute_directory: boolean;
  /** Whether the match is in the mesh filters. */
  mesh_filters: boolean;
  /** Index of the attachment point with a match, or null. */
  mesh_att_point_name: number | null;
  /** Index of the texture path with a match, or null. */
  mesh_texture_path: number | null;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The matched text. */
  text: string;
}

/** Matches for RigidModel files. */
export interface RigidModelMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: RigidModelMatch[];
}

/** Match in a UnitVariant file. */
export interface UnitVariantMatch {
  /** Index of the entry in the UnitVariant file. */
  entry: number;
  /** Whether the match is in the name. */
  name: boolean;
  /** Variant match: [variant_index, ...flags] or null. */
  variant: [number, boolean, boolean] | null;
  /** Byte where the match starts. */
  start: number;
  /** Byte where the match ends. */
  end: number;
  /** The matched text. */
  text: string;
}

/** Matches for UnitVariant files. */
export interface UnitVariantMatches {
  /** Path of the file. */
  path: string;
  /** List of matches within the file. */
  matches: UnitVariantMatch[];
}

/** Match in a Schema (column name match). */
export interface SchemaMatch {
  /** The table name. */
  table_name: string;
  /** Version of the definition with a match. */
  version: number;
  /** Column index of the match. */
  column: number;
  /** Full name of the matched column. */
  column_name: string;
}

/** Matches within the schema. */
export interface SchemaMatches {
  /** List of matches. */
  matches: SchemaMatch[];
}

/** Search results organized by file type. */
export interface Matches {
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

/** Global search configuration and results. */
export interface GlobalSearch {
  /** The text pattern or regex to search for. */
  pattern: string;
  /** Text to use for replacements. */
  replace_text: string;
  /** Whether the search should be case-sensitive. */
  case_sensitive: boolean;
  /** Whether to interpret the pattern as a regular expression. */
  use_regex: boolean;
  /** Which data sources to include in the search. */
  source: SearchSource;
  /** Which file types to search within. */
  search_on: SearchOn;
  /** Results from the most recent search operation. */
  matches: Matches;
  /** Game key for the files being searched. */
  game_key: string;
}

/** A single match within a global search result. */
export type MatchHolder =
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

/** Table reference data keyed by column index. */
export interface TableReferences {
  /** Name of the column these references are for. */
  field_name: string;
  /** Whether the referenced table exists only in the Assembly Kit. */
  referenced_table_is_ak_only: boolean;
  /** Whether the referenced column has been localised. */
  referenced_column_is_localised: boolean;
  /** Map of actual values to their lookup/display text. */
  data: Record<string, string>;
}

/** Compression format for PackFiles. */
export type CompressionFormat = string; // Enum value, e.g. "None", "Lz4", "Zstd", etc.

/** PFH file type (mod, movie, boot, etc.). */
export type PFHFileType = string; // Enum value

/** File type enumeration. */
export type FileType =
  | "Anim"
  | "AnimFragmentBattle"
  | "AnimPack"
  | "AnimsTable"
  | "Atlas"
  | "Audio"
  | "BMD"
  | "BMDVegetation"
  | "Dat"
  | "DB"
  | "ESF"
  | "Font"
  | "GroupFormations"
  | "HlslCompiled"
  | "Image"
  | "Loc"
  | "MatchedCombat"
  | "Pack"
  | "PortraitSettings"
  | "RigidModel"
  | "SoundBank"
  | "Text"
  | "UIC"
  | "UnitVariant"
  | "Video"
  | "VMD"
  | "WSModel"
  | "Unknown";

/** Cell data in a table. */
export type DecodedData =
  | { Boolean: boolean }
  | { F32: number }
  | { F64: number }
  | { I16: number }
  | { I32: number }
  | { I64: number }
  | { ColourRGB: string }
  | { StringU8: string }
  | { StringU16: string }
  | { OptionalI16: number }
  | { OptionalI32: number }
  | { OptionalI64: number }
  | { OptionalStringU8: string }
  | { OptionalStringU16: string }
  | { SequenceU16: DecodedData[][] }
  | { SequenceU32: DecodedData[][] };

/** In-memory table data structure. */
export interface TableInMemory {
  /** Table type identifier (e.g., "units_tables"). */
  table_name: string;
  /** Schema definition for this table. */
  definition: Definition;
  /** Runtime schema modifications. */
  definition_patch: DefinitionPatch;
  /** All table rows (outer = rows, inner = columns). */
  table_data: DecodedData[][];
  /** Flag indicating data was altered during decoding. */
  altered: boolean;
}

/** Decoded database table file. */
export interface DB {
  /** Boolean flag of unknown purpose (0 can crash WH2). */
  mysterious_byte: boolean;
  /** GUID for this table instance (empty for older games). */
  guid: string;
  /** The table data including definition and rows. */
  table: TableInMemory;
}

/** Decoded localisation file. */
export interface Loc {
  /** The underlying table data with key, text, and tooltip columns. */
  table: TableInMemory;
}

/** Text encoding type. */
export type TextEncoding =
  | "Iso8859_1"  // ISO-8859-1 (Western European)
  | "Utf8"       // UTF-8 without BOM
  | "Utf8Bom"    // UTF-8 with BOM
  | "Utf16Le";   // UTF-16 Little Endian with BOM

/** Text file format. */
export type TextFormat =
  | "Bat"
  | "Cpp"
  | "Html"
  | "Hlsl"
  | "Json"
  | "Js"
  | "Css"
  | "Lua"
  | "Markdown"
  | "Plain"
  | "Python"
  | "Sql"
  | "Xml"
  | "Yaml";

/** Decoded text file. */
export interface Text {
  /** Character encoding of the file. */
  encoding: TextEncoding;
  /** Detected file format based on extension. */
  format: TextFormat;
  /** Decoded text contents. */
  contents: string;
}

/** Decoded image file. */
export interface Image {
  /** Original raw image data in native format. */
  data: number[];
  /** PNG-converted data for DDS textures (for viewing/editing). */
  converted_data: number[] | null;
}

/** Level of Detail structure for 3D models. */
export interface Lod {
  /** Groups of meshes at this LOD level. */
  groups: unknown[];  // Complex nested structure
}

/** Decoded RigidModel (3D model) file. */
export interface RigidModel {
  /** File format version (6, 7, or 8). */
  version: number;
  /** Unknown field. */
  uk_1: number;
  /** Skeleton identifier for animation (empty for static models). */
  skeleton_id: string;
  /** LOD structures from highest to lowest quality. */
  lods: Lod[];
}

/** ESF file signature. */
export type ESFSignature = string; // CAAB, CBAB, etc.

/** Node type in ESF tree structure. */
export type NodeType = unknown; // Complex recursive structure

/** Decoded ESF (Empire Save Format) file. */
export interface ESF {
  /** Format signature (CAAB, CBAB, etc.). */
  signature: ESFSignature;
  /** Unknown header field, typically 0. */
  unknown_1: number;
  /** Creation timestamp. */
  creation_date: number;
  /** Root node of the data tree. */
  root_node: NodeType;
}

/** Decoded BMD (Battle Map Data) file. */
export interface Bmd {
  /** File format version (23-27). */
  serialise_version: number;
  // Complex structure with many battlefield-related fields
  [key: string]: unknown;
}

/** Decoded AnimFragmentBattle file. */
export interface AnimFragmentBattle {
  /** File format version (2 or 4). */
  version: number;
  /** List of animation entries. */
  entries: unknown[];
  /** Name of the skeleton. */
  skeleton_name: string;
  /** Format subversion (version 4 only). */
  subversion: number;
}

/** Decoded AnimsTable file. */
export interface AnimsTable {
  /** File format version (currently 2). */
  version: number;
  /** List of animation table entries. */
  entries: unknown[];
}

/** Atlas entry for sprite coordinates. */
export interface AtlasEntry {
  [key: string]: unknown;
}

/** Decoded Atlas (sprite sheet) file. */
export interface Atlas {
  /** File format version (currently 1). */
  version: number;
  /** Unknown field. */
  unknown: number;
  /** List of sprite entries. */
  entries: AtlasEntry[];
}

/** Decoded Audio file (raw binary). */
export interface Audio {
  /** Raw binary audio data. */
  data: number[];
}

/** Decoded GroupFormations file. */
export interface GroupFormations {
  /** List of formation definitions. */
  formations: unknown[];
}

/** Decoded MatchedCombat file. */
export interface MatchedCombat {
  /** File format version (1 or 3). */
  version: number;
  /** List of matched combat entries. */
  entries: unknown[];
}

/** Decoded PortraitSettings file. */
export interface PortraitSettings {
  /** Format version (1 or 4). */
  version: number;
  /** Portrait entries, one per art set. */
  entries: unknown[];
}

/** Decoded UIC (UI Component) file. */
export interface UIC {
  /** Format version number. */
  version: number;
  /** Whether decoded from XML (true) or binary (false). */
  source_is_xml: boolean;
  /** Optional comment/description. */
  comment: string;
  /** Condition for precaching. */
  precache_condition: string;
  /** Tree structure of UI element relationships. */
  hierarchy: Record<string, unknown>;
  /** Map of component IDs to definitions. */
  components: Record<string, unknown>;
}

/** Decoded UnitVariant file. */
export interface UnitVariant {
  /** Version of the UnitVariant. */
  version: number;
  /** Unknown field. */
  unknown_1: number;
  /** Variant categories. */
  categories: unknown[];
}

/** Video format types. */
export type SupportedFormats = string;

/** Internal data storage for RFile. */
export type RFileInnerData = unknown; // OnDisk, Cached, or Decoded

/** A raw packed file. */
export interface RFile {
  /** Path of the file within a container. */
  path: string;
  /** Last modified timestamp (Unix epoch). */
  timestamp: number | null;
  /** Detected or specified file type. */
  file_type: FileType;
  /** Name of the source container. */
  container_name: string | null;
  /** Internal data storage. */
  data: RFileInnerData;
}

/** Decoded file content (polymorphic). */
export type RFileDecoded =
  | { Anim: unknown }
  | { AnimFragmentBattle: AnimFragmentBattle }
  | { AnimPack: unknown }
  | { AnimsTable: AnimsTable }
  | { Atlas: Atlas }
  | { Audio: Audio }
  | { BMD: Bmd }
  | { BMDVegetation: unknown }
  | { Dat: unknown }
  | { DB: DB }
  | { ESF: ESF }
  | { Font: unknown }
  | { GroupFormations: GroupFormations }
  | { HlslCompiled: unknown }
  | { Image: Image }
  | { Loc: Loc }
  | { MatchedCombat: MatchedCombat }
  | { Pack: unknown }
  | { PortraitSettings: PortraitSettings }
  | { RigidModel: RigidModel }
  | { SoundBank: unknown }
  | { Text: Text }
  | { UIC: UIC }
  | { UnitVariant: UnitVariant }
  | { Unknown: unknown }
  | { Video: unknown }
  | { VMD: Text }
  | { WSModel: Text };

// ---------------------------------------------------------------------------
// Command Enum
// ---------------------------------------------------------------------------

/**
 * All commands that can be sent to the RPFM server.
 *
 * Each variant is documented with its expected {@link Response}.
 *
 * ### Serialization
 *
 * Unit commands (no parameters) are serialized as plain strings:
 * ```json
 * { "id": 1, "data": "NewPack" }
 * ```
 *
 * Commands with a single parameter use a newtype wrapper:
 * ```json
 * { "id": 2, "data": { "SavePackAs": "/path/to/file.pack" } }
 * ```
 *
 * Commands with multiple parameters use a tuple (JSON array):
 * ```json
 * { "id": 3, "data": { "SetGameSelected": ["warhammer_3", true] } }
 * ```
 */
export type Command =
  // ---- Lifecycle ----

  /**
   * Close the background thread. **Do not use directly** — the server
   * manages this internally.
   *
   * Response: None (breaks the server loop).
   */
  | "Exit"

  /**
   * Signal that the client is intentionally disconnecting.
   * Allows the server to clean up the session immediately instead of
   * waiting for the timeout. If this was the last session, the server
   * shuts down.
   *
   * Response: `"Success"`
   */
  | "ClientDisconnecting"

  // ---- PackFile Operations ----

  /**
   * Close the currently open Pack.
   *
   * Response: `"Success"`
   */
  | "ClosePack"

  /**
   * Close an extra Pack (opened for "Add from PackFile").
   *
   * @param path — Filesystem path to the extra Pack.
   * Response: `"Success"`
   */
  | { ClosePackExtra: string }

  /**
   * Clean the open Pack from corrupted/undecoded files and save to disk.
   * Only use if the Pack is otherwise unsaveable.
   *
   * @param path — Destination path.
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | { CleanAndSavePackAs: string }

  /**
   * Create a new empty Pack.
   *
   * Response: `"Success"`
   */
  | "NewPack"

  /**
   * Save the currently open Pack to its current path.
   *
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | "SavePack"

  /**
   * Save the currently open Pack to a new path.
   *
   * @param path — Destination path.
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | { SavePackAs: string }

  /**
   * Get tree view data for the currently open Pack.
   *
   * Response: `{ ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }`
   */
  | "GetPackFileDataForTreeView"

  /**
   * Get tree view data for an extra (secondary) Pack.
   *
   * @param path — Path to the extra Pack.
   * Response: `{ ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }` | `{ Error: string }`
   */
  | { GetPackFileExtraDataForTreeView: string }

  /**
   * Open one or more PackFiles and merge them into the current session.
   *
   * @param paths — Array of filesystem paths.
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | { OpenPackFiles: string[] }

  /**
   * Open an extra Pack for "Add from PackFile" operations.
   *
   * @param path — Filesystem path to the Pack.
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | { OpenPackExtra: string }

  /**
   * Open all CA PackFiles for the selected game as one merged Pack.
   *
   * Response: `{ ContainerInfo: ContainerInfo }` | `{ Error: string }`
   */
  | "LoadAllCAPackFiles"

  /**
   * Get RFileInfo for one or more packed files by path.
   *
   * @param paths — Internal paths of the packed files.
   * Response: `{ VecRFileInfo: RFileInfo[] }`
   */
  | { GetPackedFilesInfo: string[] }

  /**
   * Perform a global search across the open Pack.
   *
   * @param config — GlobalSearch configuration object.
   * Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }` | `{ Error: string }`
   */
  | { GlobalSearch: GlobalSearch }

  /**
   * Get the currently selected game key.
   *
   * Response: `{ String: string }`
   */
  | "GetGameSelected"

  /**
   * Change the selected game. Optionally rebuilds dependencies.
   *
   * @param params — `[game_key, rebuild_dependencies]`
   * Response: `{ CompressionFormatDependenciesInfo: [CompressionFormat, DependenciesInfo | null] }` | `{ Error: string }`
   */
  | { SetGameSelected: [string, boolean] }

  /**
   * Change the PFH type of the currently open Pack (e.g., Mod, Movie, Boot).
   *
   * @param file_type — PFHFileType enum value.
   * Response: `"Success"`
   */
  | { SetPackFileType: PFHFileType }

  /**
   * Generate the dependencies cache for the currently selected game.
   *
   * Response: `{ DependenciesInfo: DependenciesInfo }` | `{ Error: string }`
   */
  | "GenerateDependenciesCache"

  /**
   * Update the current schema with data from the game's Assembly Kit.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateCurrentSchemaFromAssKit"

  /**
   * Run the optimizer over the currently open Pack.
   *
   * @param options — Optimizer configuration.
   * Response: `{ HashSetStringHashSetString: [string[], string[]] }` (deleted, added paths) | `{ Error: string }`
   */
  | { OptimizePackFile: OptimizerOptions }

  /**
   * Patch Siege AI for Warhammer siege maps.
   *
   * Response: `{ StringVecContainerPath: [string, ContainerPath[]] }` | `{ Error: string }`
   */
  | "PatchSiegeAI"

  /**
   * Toggle the "Index Includes Timestamp" flag.
   *
   * @param enabled — Whether timestamps should be included.
   * Response: `"Success"`
   */
  | { ChangeIndexIncludesTimestamp: boolean }

  /**
   * Change the compression format of the currently open Pack.
   *
   * @param format — Desired compression format.
   * Response: `{ CompressionFormat: CompressionFormat }` (actual format set, may differ if unsupported)
   */
  | { ChangeCompressionFormat: CompressionFormat }

  /**
   * Get the filesystem path of the currently open Pack.
   *
   * Response: `{ PathBuf: string }`
   */
  | "GetPackFilePath"

  /**
   * Get the info of a single packed file.
   *
   * @param path — Internal path of the packed file.
   * Response: `{ OptionRFileInfo: RFileInfo | null }`
   */
  | { GetRFileInfo: string }

  // ---- Update Commands ----

  /**
   * Check if there is an RPFM update available.
   *
   * Response: `{ APIResponse: APIResponse }` | `{ Error: string }`
   */
  | "CheckUpdates"

  /**
   * Check if there is a schema update available.
   *
   * Response: `{ APIResponseGit: GitResponse }` | `{ Error: string }`
   */
  | "CheckSchemaUpdates"

  /**
   * Download and apply schema updates from the remote repository.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateSchemas"

  /**
   * Check if a dependency database is loaded in memory.
   *
   * @param require_asskit — If true, checks that dependencies include AssKit data.
   * Response: `{ Bool: boolean }`
   */
  | { IsThereADependencyDatabase: boolean }

  // ---- PackedFile Operations ----

  /**
   * Create a new packed file inside the currently open Pack.
   *
   * @param params — `[path, new_file_spec]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { NewPackedFile: [string, NewFile] }

  /**
   * Add files from the filesystem to the currently open Pack.
   *
   * @param params — `[source_paths, destination_container_paths, optional_ignore_paths]`
   * Response: `{ VecContainerPathOptionString: [ContainerPath[], string | null] }` (added paths, optional error message)
   */
  | { AddPackedFiles: [string[], ContainerPath[], string[] | null] }

  /**
   * Decode a packed file for display in the UI.
   *
   * @param params — `[internal_path, data_source]`
   * Response: Type-specific (e.g., `{ DBRFileInfo: [DB, RFileInfo] }`,
   *   `{ LocRFileInfo: [Loc, RFileInfo] }`, `{ TextRFileInfo: [Text, RFileInfo] }`,
   *   `{ ImageRFileInfo: [Image, RFileInfo] }`, `{ RigidModelRFileInfo: [RigidModel, RFileInfo] }`,
   *   `"Unknown"`, etc.) | `{ Error: string }`
   */
  | { DecodePackedFile: [string, DataSource] }

  /**
   * Save an edited packed file back to the Pack.
   *
   * @param params — `[internal_path, decoded_file_content]`
   * Response: `"Success"`
   */
  | { SavePackedFileFromView: [string, RFileDecoded] }

  /**
   * Copy packed files from another Pack into the current one.
   *
   * @param params — `[source_pack_path, container_paths]`
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | { AddPackedFilesFromPackFile: [string, ContainerPath[]] }

  /**
   * Copy packed files from the main Pack into an AnimPack.
   *
   * @param params — `[animpack_path, container_paths]`
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | { AddPackedFilesFromPackFileToAnimpack: [string, ContainerPath[]] }

  /**
   * Copy packed files from an AnimPack into the main Pack.
   *
   * @param params — `[data_source, animpack_path, container_paths]`
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | { AddPackedFilesFromAnimpack: [DataSource, string, ContainerPath[]] }

  /**
   * Delete packed files from an AnimPack.
   *
   * @param params — `[animpack_path, container_paths]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { DeleteFromAnimpack: [string, ContainerPath[]] }

  /**
   * Delete packed files from the open Pack.
   *
   * @param paths — Container paths to delete.
   * Response: `{ VecContainerPath: ContainerPath[] }` (deleted paths)
   */
  | { DeletePackedFiles: ContainerPath[] }

  /**
   * Extract packed files from the Pack to the filesystem.
   *
   * @param params — `[paths_by_source, extraction_path, export_tables_as_tsv]`
   *
   * The first parameter is a map of DataSource → ContainerPath[].
   * In JSON: `{ "PackFile": [...], "GameFiles": [...] }`
   *
   * Response: `{ StringVecPathBuf: [string, string[]] }` | `{ Error: string }`
   */
  | { ExtractPackedFiles: [Record<DataSource, ContainerPath[]>, string, boolean] }

  /**
   * Rename packed files in the Pack.
   *
   * @param renames — Array of `[old_path, new_path]` pairs.
   * Response: `{ VecContainerPathContainerPath: [ContainerPath, ContainerPath][] }` | `{ Error: string }`
   */
  | { RenamePackedFiles: [ContainerPath, ContainerPath][] }

  /**
   * Check if a folder exists in the currently open Pack.
   *
   * @param path — Folder path to check.
   * Response: `{ Bool: boolean }`
   */
  | { FolderExists: string }

  /**
   * Check if a packed file exists in the currently open Pack.
   *
   * @param path — File path to check.
   * Response: `{ Bool: boolean }`
   */
  | { PackedFileExists: string }

  // ---- Dependency Commands ----

  /**
   * Get all DB table names from dependency PackFiles.
   *
   * Response: `{ VecString: string[] }`
   */
  | "GetTableListFromDependencyPackFile"

  /**
   * Get custom table names (start_pos_, twad_ prefixes) from the schema.
   *
   * Response: `{ VecString: string[] }` | `{ Error: string }`
   */
  | "GetCustomTableList"

  /**
   * Get local art set IDs from campaign_character_arts_tables.
   *
   * Response: `{ HashSetString: string[] }`
   */
  | "LocalArtSetIds"

  /**
   * Get art set IDs from dependencies' campaign_character_arts_tables.
   *
   * Response: `{ HashSetString: string[] }`
   */
  | "DependenciesArtSetIds"

  /**
   * Get the version of a table from the dependency database.
   *
   * @param table_name — Name of the table.
   * Response: `{ I32: number }` | `{ Error: string }`
   */
  | { GetTableVersionFromDependencyPackFile: string }

  /**
   * Get the definition of a table from the dependency database.
   *
   * @param table_name — Name of the table.
   * Response: `{ Definition: Definition }` | `{ Error: string }`
   */
  | { GetTableDefinitionFromDependencyPackFile: string }

  /**
   * Merge multiple compatible tables into one.
   *
   * @param params — `[paths_to_merge, merged_file_path, delete_sources]`
   * Response: `{ String: string }` (merged path) | `{ Error: string }`
   */
  | { MergeFiles: [ContainerPath[], string, boolean] }

  /**
   * Update a table to a newer schema version.
   *
   * @param path — Container path of the table.
   * Response: `{ I32I32VecStringVecString: [old_ver, new_ver, deleted_fields, added_fields] }` | `{ Error: string }`
   */
  | { UpdateTable: ContainerPath }

  // ---- Search Commands ----

  /**
   * Replace specific matches in a global search.
   *
   * @param params — `[search_config, matches_to_replace]`
   * Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }` | `{ Error: string }`
   */
  | { GlobalSearchReplaceMatches: [GlobalSearch, MatchHolder[]] }

  /**
   * Replace all matches in a global search.
   *
   * @param config — GlobalSearch configuration.
   * Response: `{ GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }` | `{ Error: string }`
   */
  | { GlobalSearchReplaceAll: GlobalSearch }

  /**
   * Get reference data for columns in a table definition.
   *
   * @param params — `[table_name, definition, force_local_regen]`
   * Response: `{ HashMapI32TableReferences: Record<number, TableReferences> }`
   */
  | { GetReferenceDataFromDefinition: [string, Definition, boolean] }

  /**
   * Get the list of PackFiles marked as dependencies of the current Pack.
   *
   * Response: `{ VecBoolString: [boolean, string][] }`
   */
  | "GetDependencyPackFilesList"

  /**
   * Set the list of PackFiles marked as dependencies of the current Pack.
   *
   * @param list — Array of `[enabled, pack_name]` pairs.
   * Response: `"Success"`
   */
  | { SetDependencyPackFilesList: [boolean, string][] }

  /**
   * Get packed files from all known sources (PackFile, GameFiles, ParentFiles).
   *
   * @param params — `[container_paths, lowercase_paths]`
   * Response: `{ HashMapDataSourceHashMapStringRFile: Record<DataSource, Record<string, RFile>> }`
   */
  | { GetRFilesFromAllSources: [ContainerPath[], boolean] }

  // ---- Video Commands ----

  /**
   * Change the format of a ca_vp8 video packed file.
   *
   * @param params — `[internal_path, target_format]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SetVideoFormat: [string, SupportedFormats] }

  // ---- Schema Commands ----

  /**
   * Save a schema to disk.
   *
   * @param schema — The complete schema to save.
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SaveSchema: Schema }

  /**
   * Encode and clean the internal cache for the specified paths.
   *
   * @param paths — Container paths to clean.
   * Response: `"Success"`
   */
  | { CleanCache: ContainerPath[] }

  // ---- TSV Commands ----

  /**
   * Export a table as a TSV file.
   *
   * @param params — `[internal_path, destination_path, data_source]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { ExportTSV: [string, string, DataSource] }

  /**
   * Import a TSV file as a table.
   *
   * @param params — `[internal_path, source_tsv_path]`
   * Response: `{ RFileDecoded: RFileDecoded }` | `{ Error: string }`
   */
  | { ImportTSV: [string, string] }

  // ---- External Program Commands ----

  /**
   * Open the folder containing the currently open Pack in the file manager.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "OpenContainingFolder"

  /**
   * Open a packed file in an external program.
   *
   * @param params — `[data_source, container_path]`
   * Response: `{ PathBuf: string }` (extracted temp path) | `{ Error: string }`
   */
  | { OpenPackedFileInExternalProgram: [DataSource, ContainerPath] }

  /**
   * Save a packed file that was edited in an external program.
   *
   * @param params — `[internal_path, external_file_path]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SavePackedFileFromExternalView: [string, string] }

  // ---- Program Update Commands ----

  /**
   * Update RPFM to the latest version.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateMainProgram"

  /**
   * Trigger an autosave backup of the current Pack.
   *
   * Response: `"Success"`
   */
  | "TriggerBackupAutosave"

  // ---- Diagnostics Commands ----

  /**
   * Run a full diagnostics check over the open Pack.
   *
   * @param params — `[ignored_diagnostic_keys, check_ak_only_refs]`
   * Response: `{ Diagnostics: Diagnostics }`
   */
  | { DiagnosticsCheck: [string[], boolean] }

  /**
   * Run a partial diagnostics update on specific paths.
   *
   * @param params — `[existing_diagnostics, paths_to_check, check_ak_only_refs]`
   * Response: `{ Diagnostics: Diagnostics }`
   */
  | { DiagnosticsUpdate: [Diagnostics, ContainerPath[], boolean] }

  // ---- Pack Settings Commands ----

  /**
   * Get the settings of the currently open Pack.
   *
   * Response: `{ PackSettings: PackSettings }`
   */
  | "GetPackSettings"

  /**
   * Set the settings of the currently open Pack.
   *
   * @param settings — The new pack settings.
   * Response: `"Success"`
   */
  | { SetPackSettings: PackSettings }

  // ---- Debug Commands ----

  /**
   * Export missing table definitions to a file (for debugging/development).
   *
   * Response: `"Success"`
   */
  | "GetMissingDefinitions"

  // ---- Dependencies Commands ----

  /**
   * Rebuild the dependencies of the open Pack.
   *
   * @param rebuild_all — true = all dependencies, false = mod-specific only.
   * Response: `{ DependenciesInfo: DependenciesInfo }` | `{ Error: string }`
   */
  | { RebuildDependencies: boolean }

  // ---- Cascade Edition Commands ----

  /**
   * Trigger a cascade edition on all referenced data.
   *
   * @param params — `[table_name, definition, field_changes]`
   *   where field_changes is `[field, old_value, new_value][]`
   * Response: `{ VecContainerPathVecRFileInfo: [ContainerPath[], RFileInfo[]] }`
   */
  | { CascadeEdition: [string, Definition, [Field, string, string][]] }

  // ---- Navigation Commands ----

  /**
   * Go to the definition of a table reference.
   *
   * @param params — `[table_name, column_name, values_to_search]`
   * Response: `{ DataSourceStringUsizeUsize: [DataSource, string, number, number] }` | `{ Error: string }`
   */
  | { GoToDefinition: [string, string, string[]] }

  /**
   * Get the source data (table, column, values) of a loc key.
   *
   * @param loc_key — The loc key to look up.
   * Response: `{ OptionStringStringVecString: [string, string, string[]] | null }`
   */
  | { GetSourceDataFromLocKey: string }

  /**
   * Navigate to a loc key's location.
   *
   * @param loc_key — The loc key to find.
   * Response: `{ DataSourceStringUsizeUsize: [DataSource, string, number, number] }` | `{ Error: string }`
   */
  | { GoToLoc: string }

  /**
   * Find all references to a value across tables.
   *
   * @param params — `[table_columns_map, search_value]`
   *   where table_columns_map is `Record<table_name, column_names[]>`
   * Response: `{ VecDataSourceStringStringUsizeUsize: [DataSource, string, string, number, number][] }`
   */
  | { SearchReferences: [Record<string, string[]>, string] }

  /**
   * Get the name of the currently open Pack.
   *
   * Response: `{ String: string }`
   */
  | "GetPackFileName"

  /**
   * Get the raw binary data of a packed file.
   *
   * @param path — Internal path.
   * Response: `{ VecU8: number[] }` | `{ Error: string }`
   */
  | { GetPackedFileRawData: string }

  /**
   * Import files from dependencies into the open Pack.
   *
   * @param sources — Map of DataSource → ContainerPath[].
   * Response: `{ VecContainerPathVecString: [ContainerPath[], string[]] }` (added paths, failed paths) | `{ Error: string }`
   */
  | { ImportDependenciesToOpenPackFile: Record<DataSource, ContainerPath[]> }

  /**
   * Save packed files to the Pack and optionally run optimizer.
   *
   * @param params — `[files, optimize]`
   * Response: `{ VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }` (added, deleted) | `{ Error: string }`
   */
  | { SavePackedFilesToPackFileAndClean: [RFile[], boolean] }

  /**
   * Get all file names under a path from all dependency sources.
   *
   * @param path — Container path prefix to search.
   * Response: `{ HashMapDataSourceHashSetContainerPath: Record<DataSource, ContainerPath[]> }`
   */
  | { GetPackedFilesNamesStartingWitPathFromAllSources: ContainerPath }

  // ---- Notes Commands ----

  /**
   * Get all notes under a given path.
   *
   * @param path — Path prefix.
   * Response: `{ VecNote: Note[] }`
   */
  | { NotesForPath: string }

  /**
   * Add a note.
   *
   * @param note — The note to add.
   * Response: `{ Note: Note }`
   */
  | { AddNote: Note }

  /**
   * Delete a note.
   *
   * @param params — `[path, note_id]`
   * Response: `"Success"`
   */
  | { DeleteNote: [string, number] }

  // ---- Schema Patch Commands ----

  /**
   * Save local schema patches to disk.
   *
   * @param patches — Map of table_name → DefinitionPatch.
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SaveLocalSchemaPatch: Record<string, DefinitionPatch> }

  /**
   * Remove all local schema patches for a table.
   *
   * @param table_name — Name of the table.
   * Response: `"Success"` | `{ Error: string }`
   */
  | { RemoveLocalSchemaPatchesForTable: string }

  /**
   * Remove local schema patches for a specific field in a table.
   *
   * @param params — `[table_name, field_name]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { RemoveLocalSchemaPatchesForTableAndField: [string, string] }

  /**
   * Import schema patches into local patches.
   *
   * @param patches — Map of table_name → DefinitionPatch.
   * Response: `"Success"` | `{ Error: string }`
   */
  | { ImportSchemaPatch: Record<string, DefinitionPatch> }

  // ---- Loc Generation Commands ----

  /**
   * Generate all missing loc entries for the open Pack.
   *
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | "GenerateMissingLocData"

  // ---- Lua Autogen Commands ----

  /**
   * Check for updates on the tw_autogen repository.
   *
   * Response: `{ APIResponseGit: GitResponse }` | `{ Error: string }`
   */
  | "CheckLuaAutogenUpdates"

  /**
   * Update the tw_autogen repository.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateLuaAutogen"

  // ---- MyMod Commands ----

  /**
   * Initialize a MyMod folder structure.
   *
   * @param params — `[mod_name, game_key, sublime_support, vscode_support, git_support_gitignore_content]`
   * Response: `{ PathBuf: string }` (path to the new pack) | `{ Error: string }`
   */
  | { InitializeMyModFolder: [string, string, boolean, boolean, string | null] }

  /**
   * Live-export the Pack to the game's data folder.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "LiveExport"

  // ---- Map Packing Commands ----

  /**
   * Pack map tiles into the current Pack.
   *
   * @param params — `[tile_map_paths, tile_entries]`
   *   where tile_entries is `[tile_path, tile_name][]`
   * Response: `{ VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }` (added, deleted) | `{ Error: string }`
   */
  | { PackMap: [string[], [string, string][]] }

  // ---- Diagnostics Ignore Commands ----

  /**
   * Add a line to the pack's ignored diagnostics list.
   *
   * @param line — Diagnostic key to ignore.
   * Response: `"Success"`
   */
  | { AddLineToPackIgnoredDiagnostics: string }

  // ---- Empire/Napoleon AK Commands ----

  /**
   * Check for updates on the old Assembly Kit files repository.
   *
   * Response: `{ APIResponseGit: GitResponse }` | `{ Error: string }`
   */
  | "CheckEmpireAndNapoleonAKUpdates"

  /**
   * Update the old Assembly Kit files repository.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateEmpireAndNapoleonAK"

  // ---- Translation Commands ----

  /**
   * Get pack translation data for a language.
   *
   * @param language — Language code.
   * Response: `{ PackTranslation: PackTranslation }` | `{ Error: string }`
   */
  | { GetPackTranslation: string }

  /**
   * Check for translation updates.
   *
   * Response: `{ APIResponseGit: GitResponse }` | `{ Error: string }`
   */
  | "CheckTranslationsUpdates"

  /**
   * Update the translations repository.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "UpdateTranslations"

  // ---- Starpos Commands ----

  /**
   * Build starpos (pre-processing step).
   *
   * @param params — `[campaign_id, process_hlp_spd_data]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { BuildStarpos: [string, boolean] }

  /**
   * Build starpos (post-processing step).
   *
   * @param params — `[campaign_id, process_hlp_spd_data]`
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | { BuildStarposPost: [string, boolean] }

  /**
   * Clean up starpos temporary files.
   *
   * @param params — `[campaign_id, process_hlp_spd_data]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { BuildStarposCleanup: [string, boolean] }

  /**
   * Get campaign IDs available for starpos building.
   *
   * Response: `{ HashSetString: string[] }`
   */
  | "BuildStarposGetCampaingIds"

  /**
   * Check if victory conditions file exists (required for some games).
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "BuildStarposCheckVictoryConditions"

  // ---- Animation Commands ----

  /**
   * Update animation IDs with an offset.
   *
   * @param params — `[starting_id, offset]`
   * Response: `{ VecContainerPath: ContainerPath[] }` | `{ Error: string }`
   */
  | { UpdateAnimIds: [number, number] }

  /**
   * Get animation paths by skeleton name.
   *
   * @param skeleton_name — Name of the skeleton.
   * Response: `{ HashSetString: string[] }`
   */
  | { GetAnimPathsBySkeletonName: string }

  // ---- Table Commands ----

  /**
   * Get tables from dependencies by table name.
   *
   * @param table_name — Name of the table.
   * Response: `{ VecRFile: RFile[] }` | `{ Error: string }`
   */
  | { GetTablesFromDependencies: string }

  /**
   * Get table paths by name from the current Pack.
   *
   * @param table_name — Name of the table.
   * Response: `{ VecString: string[] }`
   */
  | { GetTablesByTableName: string }

  /**
   * Add keys to the key_deletes table.
   *
   * @param params — `[table_file_name, key_table_name, keys_to_add]`
   * Response: `{ OptionContainerPath: ContainerPath | null }`
   */
  | { AddKeysToKeyDeletes: [string, string, string[]] }

  // ---- 3D Export Commands ----

  /**
   * Export a RigidModel to glTF format.
   *
   * @param params — `[rigid_model, output_path]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { ExportRigidToGltf: [RigidModel, string] }

  // ---- Settings Getter Commands ----

  /**
   * Get a boolean setting value.
   *
   * @param key — Setting key.
   * Response: `{ Bool: boolean }`
   */
  | { SettingsGetBool: string }

  /**
   * Get an i32 setting value.
   *
   * @param key — Setting key.
   * Response: `{ I32: number }`
   */
  | { SettingsGetI32: string }

  /**
   * Get an f32 setting value.
   *
   * @param key — Setting key.
   * Response: `{ F32: number }`
   */
  | { SettingsGetF32: string }

  /**
   * Get a string setting value.
   *
   * @param key — Setting key.
   * Response: `{ String: string }`
   */
  | { SettingsGetString: string }

  /**
   * Get a PathBuf setting value.
   *
   * @param key — Setting key.
   * Response: `{ PathBuf: string }`
   */
  | { SettingsGetPathBuf: string }

  /**
   * Get a string array setting value.
   *
   * @param key — Setting key.
   * Response: `{ VecString: string[] }`
   */
  | { SettingsGetVecString: string }

  /**
   * Get raw byte data setting value.
   *
   * @param key — Setting key.
   * Response: `{ VecU8: number[] }`
   */
  | { SettingsGetVecRaw: string }

  /**
   * Get all settings at once (batch loading).
   * Much more efficient than individual SettingsGet* calls.
   *
   * Response: `{ SettingsAll: [Record<string, boolean>, Record<string, number>, Record<string, number>, Record<string, string>] }`
   *   (bool_settings, i32_settings, f32_settings, string_settings)
   */
  | "SettingsGetAll"

  // ---- Settings Setter Commands ----

  /**
   * Set a boolean setting value.
   *
   * @param params — `[key, value]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetBool: [string, boolean] }

  /**
   * Set an i32 setting value.
   *
   * @param params — `[key, value]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetI32: [string, number] }

  /**
   * Set an f32 setting value.
   *
   * @param params — `[key, value]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetF32: [string, number] }

  /**
   * Set a string setting value.
   *
   * @param params — `[key, value]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetString: [string, string] }

  /**
   * Set a PathBuf setting value.
   *
   * @param params — `[key, value]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetPathBuf: [string, string] }

  /**
   * Set a string array setting value.
   *
   * @param params — `[key, values]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetVecString: [string, string[]] }

  /**
   * Set raw byte data setting value.
   *
   * @param params — `[key, data]`
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsSetVecRaw: [string, number[]] }

  // ---- Path Commands ----

  /**
   * Get the config directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "ConfigPath"

  /**
   * Get the Assembly Kit path for the current game.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "AssemblyKitPath"

  /**
   * Get the backup autosave directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "BackupAutosavePath"

  /**
   * Get the old AK data directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "OldAkDataPath"

  /**
   * Get the schemas directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "SchemasPath"

  /**
   * Get the table profiles directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "TableProfilesPath"

  /**
   * Get the translations local directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "TranslationsLocalPath"

  /**
   * Get the dependencies cache directory path.
   *
   * Response: `{ PathBuf: string }` | `{ Error: string }`
   */
  | "DependenciesCachePath"

  /**
   * Clear a specific config path entry.
   *
   * @param path — The path to clear.
   * Response: `"Success"` | `{ Error: string }`
   */
  | { SettingsClearPath: string }

  // ---- Settings Backup Commands ----

  /**
   * Backup current settings to memory (for restore on cancel).
   *
   * Response: `"Success"`
   */
  | "BackupSettings"

  /**
   * Clear all settings and reset to defaults.
   *
   * Response: `"Success"` | `{ Error: string }`
   */
  | "ClearSettings"

  /**
   * Restore settings from the in-memory backup.
   *
   * Response: `"Success"`
   */
  | "RestoreBackupSettings"

  /**
   * Get the optimizer options configuration.
   *
   * Response: `{ OptimizerOptions: OptimizerOptions }`
   */
  | "OptimizerOptions"

  // ---- Schema Query Commands ----

  /**
   * Check if a schema is loaded in memory.
   *
   * Response: `{ Bool: boolean }`
   */
  | "IsSchemaLoaded"

  /**
   * Get all definitions (all versions) for a table name.
   *
   * @param table_name — Name of the table.
   * Response: `{ VecDefinition: Definition[] }` | `{ Error: string }`
   */
  | { DefinitionsByTableName: string }

  /**
   * Get columns from other tables that reference a given table/definition.
   *
   * @param params — `[table_name, definition]`
   * Response: `{ HashMapStringHashMapStringVecString: Record<string, Record<string, string[]>> }` | `{ Error: string }`
   */
  | { ReferencingColumnsForDefinition: [string, Definition] }

  /**
   * Get the currently loaded schema.
   *
   * Response: `{ Schema: Schema }` | `{ Error: string }`
   */
  | "Schema"

  /**
   * Get a specific definition by table name and version.
   *
   * @param params — `[table_name, version]`
   * Response: `{ Definition: Definition }` | `{ Error: string }`
   */
  | { DefinitionByTableNameAndVersion: [string, number] }

  /**
   * Delete a definition by table name and version.
   *
   * @param params — `[table_name, version]`
   * Response: `"Success"`
   */
  | { DeleteDefinition: [string, number] };

// ---------------------------------------------------------------------------
// Response Enum
// ---------------------------------------------------------------------------

/**
 * All possible responses from the RPFM server.
 *
 * Each variant is named after the type(s) it carries.
 *
 * ### Serialization
 *
 * Unit responses are plain strings:
 * ```json
 * { "id": 1, "data": "Success" }
 * ```
 *
 * Responses with data use a newtype wrapper:
 * ```json
 * { "id": 2, "data": { "Bool": true } }
 * { "id": 3, "data": { "Error": "File not found" } }
 * { "id": 4, "data": { "ContainerInfoVecRFileInfo": [{ ... }, [{ ... }]] } }
 * ```
 */
export type Response =
  /** Generic success with no data. */
  | "Success"

  /** Error with a human-readable message. */
  | { Error: string }

  /**
   * Sent by the server immediately after a WebSocket connection is established.
   * Contains the session ID that the client is now connected to.
   * This is an unsolicited message (not a response to a command) with id=0.
   */
  | { SessionConnected: number }

  // ---- File-type decoded responses (returned by DecodePackedFile) ----
  | { BmdRFileInfo: [Bmd, RFileInfo] }
  | { AnimFragmentBattleRFileInfo: [AnimFragmentBattle, RFileInfo] }
  | { AnimPackRFileInfo: [RFileInfo[], RFileInfo] }
  | { AnimsTableRFileInfo: [AnimsTable, RFileInfo] }
  | { AtlasRFileInfo: [Atlas, RFileInfo] }
  | { AudioRFileInfo: [Audio, RFileInfo] }
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

  // ---- Scalar responses ----
  | { Bool: boolean }
  | { F32: number }
  | { I32: number }
  | { I32I32: [number, number] }
  | { String: string }
  | { PathBuf: string }

  // ---- Collection responses ----
  | { VecBoolString: [boolean, string][] }
  | { VecContainerPath: ContainerPath[] }
  | { VecContainerPathContainerPath: [ContainerPath, ContainerPath][] }
  | { VecContainerPathOptionString: [ContainerPath[], string | null] }
  | { VecContainerPathVecContainerPath: [ContainerPath[], ContainerPath[]] }
  | { VecContainerPathVecRFileInfo: [ContainerPath[], RFileInfo[]] }
  | { VecContainerPathVecString: [ContainerPath[], string[]] }
  | { VecDataSourceStringStringUsizeUsize: [DataSource, string, string, number, number][] }
  | { VecDefinition: Definition[] }
  | { VecNote: Note[] }
  | { VecRFile: RFile[] }
  | { VecRFileInfo: RFileInfo[] }
  | { VecString: string[] }
  | { VecU8: number[] }
  | { HashSetString: string[] }
  | { HashSetStringHashSetString: [string[], string[]] }

  // ---- Compound responses ----
  | { APIResponse: APIResponse }
  | { APIResponseGit: GitResponse }
  | { CompressionFormat: CompressionFormat }
  | { CompressionFormatDependenciesInfo: [CompressionFormat, DependenciesInfo | null] }
  | { ContainerInfo: ContainerInfo }
  | { ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }
  | { DataSourceStringUsizeUsize: [DataSource, string, number, number] }
  | { Definition: Definition }
  | { DependenciesInfo: DependenciesInfo }
  | { Diagnostics: Diagnostics }
  | { GlobalSearchVecRFileInfo: [GlobalSearch, RFileInfo[]] }
  | { HashMapDataSourceHashMapStringRFile: Record<DataSource, Record<string, RFile>> }
  | { HashMapDataSourceHashSetContainerPath: Record<DataSource, ContainerPath[]> }
  | { HashMapI32TableReferences: Record<number, TableReferences> }
  | { HashMapStringHashMapStringVecString: Record<string, Record<string, string[]>> }
  | { I32I32VecStringVecString: [number, number, string[], string[]] }
  | { Note: Note }
  | { OptimizerOptions: OptimizerOptions }
  | { OptionContainerPath: ContainerPath | null }
  | { OptionRFileInfo: RFileInfo | null }
  | { OptionStringStringVecString: [string, string, string[]] | null }
  | { PackSettings: PackSettings }
  | { PackTranslation: PackTranslation }
  | { RFileDecoded: RFileDecoded }
  | { Schema: Schema }
  | { StringVecContainerPath: [string, ContainerPath[]] }
  | { StringVecPathBuf: [string, string[]] }
  | { Text: Text }

  /** Returned for unsupported/unrecognized file types. */
  | "Unknown"

  /**
   * All settings in one batch response.
   * `[bool_settings, i32_settings, f32_settings, string_settings]`
   */
  | { SettingsAll: [Record<string, boolean>, Record<string, number>, Record<string, number>, Record<string, string>] };

// ---------------------------------------------------------------------------
// Usage Examples
// ---------------------------------------------------------------------------

/**
 * ## Example: Full client implementation
 *
 * ```ts
 * class RpfmClient {
 *   private ws: WebSocket;
 *   private nextId = 1;
 *   private pending = new Map<number, {
 *     resolve: (resp: Response) => void;
 *     reject: (err: Error) => void;
 *   }>();
 *   public sessionId: number | null = null;
 *
 *   constructor(url = "ws://127.0.0.1:45127/ws") {
 *     this.ws = new WebSocket(url);
 *     this.ws.onmessage = (event) => {
 *       const msg: Message<Response> = JSON.parse(event.data);
 *
 *       // Handle SessionConnected (unsolicited, id=0)
 *       if (typeof msg.data === "object" && "SessionConnected" in msg.data) {
 *         this.sessionId = msg.data.SessionConnected;
 *         console.log(`Connected to session ${this.sessionId}`);
 *         return;
 *       }
 *
 *       const handler = this.pending.get(msg.id);
 *       if (handler) {
 *         this.pending.delete(msg.id);
 *         if (typeof msg.data === "object" && "Error" in msg.data) {
 *           handler.reject(new Error(msg.data.Error));
 *         } else {
 *           handler.resolve(msg.data);
 *         }
 *       }
 *     };
 *   }
 *
 *   send(command: Command): Promise<Response> {
 *     return new Promise((resolve, reject) => {
 *       const id = this.nextId++;
 *       this.pending.set(id, { resolve, reject });
 *       this.ws.send(JSON.stringify({ id, data: command }));
 *     });
 *   }
 *
 *   // --- Typed convenience methods ---
 *
 *   async openPack(paths: string[]): Promise<ContainerInfo> {
 *     const resp = await this.send({ OpenPackFiles: paths });
 *     return (resp as { ContainerInfo: ContainerInfo }).ContainerInfo;
 *   }
 *
 *   async savePack(): Promise<ContainerInfo> {
 *     const resp = await this.send("SavePack");
 *     return (resp as { ContainerInfo: ContainerInfo }).ContainerInfo;
 *   }
 *
 *   async savePackAs(path: string): Promise<ContainerInfo> {
 *     const resp = await this.send({ SavePackAs: path });
 *     return (resp as { ContainerInfo: ContainerInfo }).ContainerInfo;
 *   }
 *
 *   async getTreeView(): Promise<[ContainerInfo, RFileInfo[]]> {
 *     const resp = await this.send("GetPackFileDataForTreeView");
 *     return (resp as { ContainerInfoVecRFileInfo: [ContainerInfo, RFileInfo[]] }).ContainerInfoVecRFileInfo;
 *   }
 *
 *   async setGame(gameKey: string, rebuildDeps: boolean): Promise<void> {
 *     await this.send({ SetGameSelected: [gameKey, rebuildDeps] });
 *   }
 *
 *   async decodeFile(path: string, source: DataSource = "PackFile"): Promise<Response> {
 *     return this.send({ DecodePackedFile: [path, source] });
 *   }
 *
 *   async deleteFiles(paths: ContainerPath[]): Promise<ContainerPath[]> {
 *     const resp = await this.send({ DeletePackedFiles: paths });
 *     return (resp as { VecContainerPath: ContainerPath[] }).VecContainerPath;
 *   }
 *
 *   async extractFiles(
 *     paths: Record<DataSource, ContainerPath[]>,
 *     destPath: string,
 *     asTsv = false,
 *   ): Promise<[string, string[]]> {
 *     const resp = await this.send({ ExtractPackedFiles: [paths, destPath, asTsv] });
 *     return (resp as { StringVecPathBuf: [string, string[]] }).StringVecPathBuf;
 *   }
 *
 *   async getSetting(key: string): Promise<string> {
 *     const resp = await this.send({ SettingsGetString: key });
 *     return (resp as { String: string }).String;
 *   }
 *
 *   async getAllSettings(): Promise<{
 *     bools: Record<string, boolean>;
 *     ints: Record<string, number>;
 *     floats: Record<string, number>;
 *     strings: Record<string, string>;
 *   }> {
 *     const resp = await this.send("SettingsGetAll");
 *     const [bools, ints, floats, strings] = (resp as {
 *       SettingsAll: [Record<string, boolean>, Record<string, number>, Record<string, number>, Record<string, string>]
 *     }).SettingsAll;
 *     return { bools, ints, floats, strings };
 *   }
 *
 *   async disconnect(): Promise<void> {
 *     await this.send("ClientDisconnecting");
 *     this.ws.close();
 *   }
 * }
 * ```
 */
export {};
