//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rmcp::ErrorData as McpError;
use rmcp::handler::server::{router::prompt::PromptRouter, tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    Annotated, CallToolResult, CompletionInfo, CompleteRequestParam, CompleteResult,
    Content, ErrorCode, GetPromptRequestParam, GetPromptResult,
    ListPromptsResult, ListResourcesResult, ListResourceTemplatesResult,
    PaginatedRequestParam, PromptMessage, PromptMessageRole,
    RawResource, ReadResourceRequestParam, ReadResourceResult,
    ResourceContents, ServerCapabilities, ServerInfo, SetLevelRequestParam,
};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router, RoleServer};
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::path::PathBuf;

use rpfm_ipc::helpers::DataSource;
use rpfm_ipc::messages::{Command, Response};
use rpfm_lib::files::{ContainerPath, RFile, RFileDecoded};
use rpfm_telemetry::sentry;

use crate::session::{Session, recv_response};

//-------------------------------------------------------------------------------//
//                              Helper macro
//-------------------------------------------------------------------------------//

/// Helper to send a command and return the JSON response.
///
/// Each tool call starts an independent Sentry transaction following the MCP tracing spec,
/// so it gets reported regardless of the long-lived rmcp service span.
macro_rules! send_and_respond {
    ($self:expr, $tool_name:expr, $cmd:expr) => {{
        let tx_ctx = sentry::TransactionContext::new(
            &format!("tools/call {}", $tool_name),
            "mcp.server",
        );
        let tx = sentry::start_transaction(tx_ctx);
        tx.set_data("mcp.method.name", sentry::protocol::Value::from("tools/call"));
        tx.set_data("mcp.tool.name", sentry::protocol::Value::from($tool_name));
        tx.set_data("mcp.transport", sentry::protocol::Value::from("streamable-http"));

        sentry::configure_scope(|scope| scope.set_span(Some(tx.clone().into())));

        let mut receiver = $self.session.send($cmd);
        let response = recv_response(&mut receiver).await;

        tx.finish();

        let is_error = matches!(&response, Response::Error(_));

        let json = serde_json::to_string(&response).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to serialize response: {e}").into(),
            data: None,
        })?;

        if is_error {
            Ok(CallToolResult::error(vec![Content::text(json)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
    }};
}

/// Build an Annotated<RawResource> with common fields set.
fn resource(uri: &str, name: &str, description: &str, mime_type: &str) -> Annotated<RawResource> {
    let mut raw = RawResource::new(uri, name);
    raw.description = Some(description.into());
    raw.mime_type = Some(mime_type.into());
    Annotated { raw, annotations: None }
}

/// Parse a JSON string into the expected type, returning a tool-level error on failure.
///
/// This is a macro (not a function) so that `return Ok(...)` exits the calling tool method,
/// keeping invalid-JSON errors as tool results instead of protocol-level `McpError`s that
/// would tear down the MCP session.
macro_rules! parse_json {
    ($input:expr) => {
        match serde_json::from_str($input) {
            Ok(v) => v,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Invalid JSON parameter: {e}"))])),
        }
    };
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone)]
pub struct McpServer {
    session: Arc<Session>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

// -- Generic / Existing Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
#[schemars(description = "Call any IPC command directly.")]
pub struct CallCommandArgs {
    /// The JSON representation of the Command enum.
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct OpenPackfilesArgs {
    /// The paths of the PackFiles to open.
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SetGameSelectedArgs {
    /// The name of the game to select.
    pub game_name: String,
    /// Whether to rebuild dependencies.
    pub rebuild_dependencies: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct TsvExportArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the TSV file to export to.
    pub tsv_path: PathBuf,
    /// The path of the table to export.
    pub table_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct TsvImportArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the TSV file to import from.
    pub tsv_path: PathBuf,
    /// The path of the table to import to.
    pub table_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DecodePackedFileArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the file inside the data source.
    pub path: String,
    /// The data source to decode from.
    pub source: DataSource,
}

// -- Pack Lifecycle Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PathArg {
    /// The file path.
    pub path: PathBuf,
}

// -- Pack Key Args (multi-pack support) --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyArg {
    /// The key of the target pack. Use `list_open_packs` to get available keys.
    pub pack_key: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyBoolArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// A boolean value.
    pub value: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyStringArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// A string value.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyStringsArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// A list of string values.
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyPathArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// The file path.
    pub path: PathBuf,
}

// -- Pack Metadata Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SetPackFileTypeArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the PFHFileType enum.
    pub pack_file_type: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ChangeCompressionFormatArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the CompressionFormat enum.
    pub format: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct BoolArg {
    /// A boolean value.
    pub value: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SetPackSettingsArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the PackSettings struct.
    pub settings: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SetDependencyPackFilesListArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<(bool, String)> for the dependency list.
    pub list: String,
}

// -- File Operations Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct NewPackedFileArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path for the new file inside the pack.
    pub path: String,
    /// The JSON representation of the NewFile enum.
    pub new_file: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddPackedFilesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The source filesystem paths.
    pub source_paths: Vec<PathBuf>,
    /// The JSON representation of Vec<ContainerPath> for destination paths.
    pub destination_paths: String,
    /// The optional paths to ignore (JSON representation of Option<Vec<PathBuf>>).
    pub ignore_paths: Option<Vec<PathBuf>>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddPackedFilesFromPackFileArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The key of the source PackFile.
    pub source_pack_path: String,
    /// The JSON representation of Vec<ContainerPath> for files to add.
    pub container_paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddPackedFilesFromPackFileToAnimpackArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The animpack path.
    pub animpack_path: String,
    /// The JSON representation of Vec<ContainerPath> for files to add.
    pub container_paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddPackedFilesFromAnimpackArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The data source to get the animpack from.
    pub source: DataSource,
    /// The animpack path.
    pub animpack_path: String,
    /// The JSON representation of Vec<ContainerPath> for files to add.
    pub container_paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ContainerPathsArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<ContainerPath>.
    pub paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DeleteFromAnimpackArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The animpack path.
    pub animpack_path: String,
    /// The JSON representation of Vec<ContainerPath> for files to delete.
    pub container_paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExtractPackedFilesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of BTreeMap<DataSource, Vec<ContainerPath>>.
    pub source_paths: String,
    /// The destination path on disk.
    pub destination_path: PathBuf,
    /// Whether to export tables as TSV.
    pub export_as_tsv: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct RenamePackedFilesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<(ContainerPath, ContainerPath)>.
    pub renames: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct CopyOrCutPackedFilesArgs {
    /// A JSON object mapping pack key to ContainerPath arrays, e.g. {"my_pack.pack": [{"File": "db/table/file"}]}.
    pub paths_by_pack: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PastePackedFilesArgs {
    /// The key of the target pack to paste into.
    pub pack_key: String,
    /// The destination folder path inside the pack (use empty string for root).
    pub destination_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DuplicatePackedFilesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<ContainerPath> for files to duplicate.
    pub paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SavePackedFileFromViewArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the file inside the pack.
    pub path: String,
    /// The JSON representation of the RFileDecoded enum.
    pub data: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SavePackedFileFromExternalViewArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The internal path of the file in the pack.
    pub internal_path: String,
    /// The external file path on disk.
    pub external_path: PathBuf,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SavePackedFilesToPackFileAndCleanArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<RFile>.
    pub files: String,
    /// Whether to optimize after saving.
    pub optimize: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct StringArg {
    /// A string value.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct OpenPackedFileInExternalProgramArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The data source of the file.
    pub source: DataSource,
    /// The JSON representation of the ContainerPath.
    pub container_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct StringsArg {
    /// A list of string values.
    pub values: Vec<String>,
}

// -- Dependency Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ImportDependenciesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of BTreeMap<DataSource, Vec<ContainerPath>>.
    pub paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GetRFilesFromAllSourcesArgs {
    /// The JSON representation of Vec<ContainerPath>.
    pub paths: String,
    /// Whether to lowercase paths.
    pub lowercase: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ContainerPathArg {
    /// The JSON representation of the ContainerPath.
    pub path: String,
}

// -- Search Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GlobalSearchArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the GlobalSearch struct.
    pub search: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GlobalSearchReplaceMatchesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the GlobalSearch struct.
    pub search: String,
    /// The JSON representation of Vec<MatchHolder>.
    pub matches: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SearchReferencesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of HashMap<String, Vec<String>>.
    pub reference_map: String,
    /// The value to search for.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GetReferenceDataFromDefinitionArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The table name.
    pub table_name: String,
    /// The JSON representation of the Definition struct.
    pub definition: String,
    /// Force local reference regeneration.
    pub force: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GoToDefinitionArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The table name.
    pub table_name: String,
    /// The column name.
    pub column_name: String,
    /// The values to search for.
    pub values: Vec<String>,
}

// -- Schema Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SaveSchemaArgs {
    /// The JSON representation of the Schema struct.
    pub schema: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct StringI32Args {
    /// A string value (e.g., table name).
    pub name: String,
    /// An integer value (e.g., version).
    pub version: i32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReferencingColumnsForDefinitionArgs {
    /// The table name.
    pub table_name: String,
    /// The JSON representation of the Definition struct.
    pub definition: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DefinitionArg {
    /// The JSON representation of the Definition struct.
    pub definition: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SchemaPatchArgs {
    /// The JSON representation of HashMap<String, DefinitionPatch>.
    pub patches: String,
}

// -- Table Ops Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct MergeFilesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of Vec<ContainerPath> for files to merge.
    pub paths: String,
    /// The path for the merged file.
    pub merged_path: String,
    /// Whether to delete source files after merging.
    pub delete_source: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct CascadeEditionArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The table name.
    pub table_name: String,
    /// The JSON representation of the Definition struct.
    pub definition: String,
    /// The JSON representation of Vec<(Field, String, String)> for field changes.
    pub changes: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddKeysToKeyDeletesArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The table file name.
    pub table_file_name: String,
    /// The key table name.
    pub key_table_name: String,
    /// The keys to add.
    pub keys: HashSet<String>,
}

// -- Diagnostics Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DiagnosticsCheckArgs {
    /// The list of ignored diagnostics.
    pub ignored: Vec<String>,
    /// Whether to check AK-only references.
    pub check_ak_only_refs: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DiagnosticsUpdateArgs {
    /// The JSON representation of the Diagnostics struct.
    pub diagnostics: String,
    /// The JSON representation of Vec<ContainerPath> for paths to check.
    pub paths: String,
    /// Whether to check AK-only references.
    pub check_ak_only_refs: bool,
}

// -- Notes Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddNoteArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the Note struct.
    pub note: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct DeleteNoteArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path the note belongs to.
    pub path: String,
    /// The note ID.
    pub id: u64,
}

// -- Optimization Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct OptimizePackFileArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The JSON representation of the OptimizerOptions struct.
    pub options: String,
}

// -- Settings Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetBoolArgs {
    /// The setting key.
    pub key: String,
    /// The boolean value.
    pub value: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetI32Args {
    /// The setting key.
    pub key: String,
    /// The integer value.
    pub value: i32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetF32Args {
    /// The setting key.
    pub key: String,
    /// The float value.
    pub value: f32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetStringArgs {
    /// The setting key.
    pub key: String,
    /// The string value.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetPathBufArgs {
    /// The setting key.
    pub key: String,
    /// The path value.
    pub value: PathBuf,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetVecStringArgs {
    /// The setting key.
    pub key: String,
    /// The list of string values.
    pub value: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SettingsSetVecRawArgs {
    /// The setting key.
    pub key: String,
    /// The raw byte values.
    pub value: Vec<u8>,
}

// -- Specialized Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct InitializeMyModFolderArgs {
    /// The mod name.
    pub name: String,
    /// The game key.
    pub game: String,
    /// Whether to add Sublime Text support.
    pub sublime: bool,
    /// Whether to add VS Code support.
    pub vscode: bool,
    /// Optional gitignore template content.
    pub gitignore: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackMapArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The tile map paths.
    pub tile_maps: Vec<PathBuf>,
    /// The JSON representation of Vec<(PathBuf, String)> for tile path/name pairs.
    pub tiles: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct BuildStarposArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The campaign ID.
    pub campaign_id: String,
    /// Whether to process HLP/SPD data.
    pub process_hlp_spd: bool,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct UpdateAnimIdsArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The starting animation ID.
    pub starting_id: i32,
    /// The offset to apply.
    pub offset: i32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExportRigidToGltfArgs {
    /// The JSON representation of the RigidModel struct.
    pub rigid_model: String,
    /// The output path.
    pub output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SetVideoFormatArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the video file in the pack.
    pub path: String,
    /// The JSON representation of the SupportedFormats enum.
    pub format: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct GetPackTranslationArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The language code.
    pub language: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

#[tool_handler]
#[prompt_handler]
impl rmcp::ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("\
This is the MCP server for RPFM (Rusted PackFile Manager), a tool for modding Total War games by \
Creative Assembly. It lets you read, edit, create, and manage PackFiles (.pack) — the archive \
format used by all modern Total War titles.

## Key Concepts

- **PackFile**: An archive containing game data files (DB tables, localisation, textures, models, etc.). \
  Mods are distributed as PackFiles.
- **pack_key**: When you open one or more PackFiles, each gets a unique key string. Use `list_open_packs` \
  to discover available keys. Most tools require a `pack_key` parameter.
- **DataSource**: Where data lives — `\"PackFile\"` (the user's mod), `\"GameFiles\"` (vanilla game data), \
  `\"ParentFiles\"` (dependency mods), `\"AssKitFiles\"` (Assembly Kit data), `\"ExternalFile\"` (disk file).
- **ContainerPath**: A path inside a pack — either `{\"File\": \"db/land_units_tables/my_table\"}` or \
  `{\"Folder\": \"db/land_units_tables\"}`. Use an empty string for root folder.

## Required Initialization Sequence

1. **Set the game** — Call `set_game_selected` with the game key (e.g. `\"warhammer_3\"`) and \
   `rebuild_dependencies: true`. This loads schemas and vanilla data.
2. **Open a pack** — Call `open_packfiles` with filesystem path(s). Note the returned pack key(s).
3. **Verify schema** — Call `is_schema_loaded`; if false, call `update_schemas` first.

## Supported Games

Valid game keys: `pharaoh_dynasties`, `pharaoh`, `warhammer_3`, `troy`, `three_kingdoms`, \
`warhammer_2`, `warhammer`, `thrones_of_britannia`, `attila`, `rome_2`, `shogun_2`, `napoleon`, \
`empire`, `arena`.

## Common File Path Conventions

- DB tables: `db/<table_name>/<file_name>` (e.g. `db/land_units_tables/my_mod`)
- Localisation: `text/db/<file_name>.loc`
- Scripts: `script/<path>.lua`
- Images: `ui/<path>.png`

## Pack File Types (PFHFileType)

`\"Boot\"`, `\"Release\"`, `\"Patch\"`, `\"Mod\"` (default for mods), `\"Movie\"`.

## Compression Formats

`\"None\"` (default), `\"Lzma1\"` (legacy), `\"Lz4\"` (WH3 6.2+), `\"Zstd\"` (WH3 6.2+).

## Creating New Files (NewFile)

- DB table: `{\"DB\": [\"file_name\", \"table_name\", version]}` — e.g. `{\"DB\": [\"my_mod\", \"land_units_tables\", 0]}`
- Loc file: `{\"Loc\": \"file_name\"}`
- Text file: `{\"Text\": [\"file_name\", \"Plain\"]}` — formats: `\"Plain\"`, `\"Html\"`, `\"Xml\"`, `\"Lua\"`, `\"Cpp\"`, `\"Json\"`, `\"Markdown\"`, `\"Smithy\"`
- AnimPack: `{\"AnimPack\": \"file_name\"}`
- PortraitSettings: `{\"PortraitSettings\": [\"file_name\", version, [[\"entry_key\", \"entry_value\"]]]}`
- VMD: `{\"VMD\": \"file_name\"}`
- WSModel: `{\"WSModel\": \"file_name\"}`

## Resources

Use `resources/list` and `resources/read` to browse reference data: valid enum values, game lists, \
and example JSON payloads without needing tool calls.

## Responses

All tool responses are JSON-serialized. On failure, an error message is returned instead of the expected data.
".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_completions()
                .enable_logging()
                .build(),
            ..Default::default()
        }
    }

    //-----------------------------------------------------------------------//
    // Resources
    //-----------------------------------------------------------------------//

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources = vec![
            resource("rpfm://games", "games", "List of all supported Total War game keys.", "application/json"),
            resource("rpfm://enums/PFHFileType", "PFHFileType", "Valid PackFile type values (Boot, Release, Patch, Mod, Movie).", "application/json"),
            resource("rpfm://enums/CompressionFormat", "CompressionFormat", "Valid compression format values (None, Lzma1, Lz4, Zstd).", "application/json"),
            resource("rpfm://enums/DataSource", "DataSource", "Valid data source values indicating where data comes from.", "application/json"),
            resource("rpfm://enums/ContainerPath", "ContainerPath", "ContainerPath enum variants with JSON examples.", "application/json"),
            resource("rpfm://enums/NewFile", "NewFile", "NewFile enum variants for creating files inside packs, with JSON examples.", "application/json"),
            resource("rpfm://enums/SupportedFormats", "SupportedFormats", "Valid video format values (CaVp8, Ivf).", "application/json"),
            resource("rpfm://examples/global_search", "GlobalSearch example", "Example JSON for the GlobalSearch struct used by search tools.", "application/json"),
            resource("rpfm://examples/optimizer_options", "OptimizerOptions example", "Example JSON for OptimizerOptions with all boolean fields.", "application/json"),
            resource("rpfm://reference/initialization", "Initialization guide", "Step-by-step guide for initializing the RPFM MCP server session.", "text/plain"),
            resource("rpfm://reference/path_conventions", "Path conventions", "Common file path conventions inside Total War PackFiles.", "text/plain"),
        ];
        Ok(ListResourcesResult {
            resources,
            ..Default::default()
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;
        let content = match uri.as_str() {
            "rpfm://games" => serde_json::json!({
                "supported_games": [
                    {"key": "pharaoh_dynasties", "display_name": "Total War: Pharaoh Dynasties"},
                    {"key": "pharaoh", "display_name": "Total War: Pharaoh"},
                    {"key": "warhammer_3", "display_name": "Total War: Warhammer III"},
                    {"key": "troy", "display_name": "A Total War Saga: Troy"},
                    {"key": "three_kingdoms", "display_name": "Total War: Three Kingdoms"},
                    {"key": "warhammer_2", "display_name": "Total War: Warhammer II"},
                    {"key": "warhammer", "display_name": "Total War: Warhammer"},
                    {"key": "thrones_of_britannia", "display_name": "A Total War Saga: Thrones of Britannia"},
                    {"key": "attila", "display_name": "Total War: Attila"},
                    {"key": "rome_2", "display_name": "Total War: Rome II"},
                    {"key": "shogun_2", "display_name": "Total War: Shogun 2"},
                    {"key": "napoleon", "display_name": "Total War: Napoleon"},
                    {"key": "empire", "display_name": "Total War: Empire"},
                    {"key": "arena", "display_name": "Total War: Arena"}
                ]
            }).to_string(),

            "rpfm://enums/PFHFileType" => serde_json::json!({
                "enum": "PFHFileType",
                "description": "The type/priority of a PackFile. Games load packs in type order (Boot first, Movie last).",
                "variants": [
                    {"name": "Boot", "value": 0, "description": "Core game boot files, loaded first."},
                    {"name": "Release", "value": 1, "description": "Main game data files."},
                    {"name": "Patch", "value": 2, "description": "Official patch and update files."},
                    {"name": "Mod", "value": 3, "description": "User mod files. This is the default for mods."},
                    {"name": "Movie", "value": 4, "description": "Cinematic and always-loaded files, loaded last."}
                ],
                "json_example": "\"Mod\""
            }).to_string(),

            "rpfm://enums/CompressionFormat" => serde_json::json!({
                "enum": "CompressionFormat",
                "description": "Compression algorithm for pack file data.",
                "variants": [
                    {"name": "None", "description": "No compression (default)."},
                    {"name": "Lzma1", "description": "Legacy LZMA compression (all PFH5 games)."},
                    {"name": "Lz4", "description": "LZ4 compression (Warhammer 3 v6.2+)."},
                    {"name": "Zstd", "description": "Zstandard compression (Warhammer 3 v6.2+)."}
                ],
                "json_example": "\"None\""
            }).to_string(),

            "rpfm://enums/DataSource" => serde_json::json!({
                "enum": "DataSource",
                "description": "Identifies where data comes from when working with files.",
                "variants": [
                    {"name": "PackFile", "description": "Data from the user's currently open pack (mod files)."},
                    {"name": "GameFiles", "description": "Data from vanilla game files."},
                    {"name": "ParentFiles", "description": "Data from parent/dependency pack files."},
                    {"name": "AssKitFiles", "description": "Data from the Assembly Kit (modding tools)."},
                    {"name": "ExternalFile", "description": "Data from an external file on disk."}
                ],
                "json_example": "\"PackFile\""
            }).to_string(),

            "rpfm://enums/ContainerPath" => serde_json::json!({
                "enum": "ContainerPath",
                "description": "A path reference inside a PackFile, pointing to either a file or a folder.",
                "variants": [
                    {
                        "name": "File",
                        "description": "Path to a single file inside the pack.",
                        "json_example": {"File": "db/land_units_tables/my_table"}
                    },
                    {
                        "name": "Folder",
                        "description": "Path to a folder inside the pack. Use empty string for root.",
                        "json_example": {"Folder": "db/land_units_tables"}
                    }
                ],
                "usage_notes": "Most tools accept a JSON array of ContainerPath objects, e.g. [{\"File\": \"path1\"}, {\"Folder\": \"path2\"}]"
            }).to_string(),

            "rpfm://enums/NewFile" => serde_json::json!({
                "enum": "NewFile",
                "description": "Specifies what type of file to create inside a pack.",
                "variants": [
                    {
                        "name": "DB",
                        "description": "Create a new DB table. Args: [file_name, table_name, version].",
                        "json_example": {"DB": ["my_mod", "land_units_tables", 0]}
                    },
                    {
                        "name": "Loc",
                        "description": "Create a new localisation file. Arg: file_name.",
                        "json_example": {"Loc": "my_mod"}
                    },
                    {
                        "name": "Text",
                        "description": "Create a new text file. Args: [file_name, format]. Formats: Bat, Cpp, Html, Hlsl, Json, Js, Css, Lua, Markdown, Plain, Python, Sql, Xml, Yaml.",
                        "json_example": {"Text": ["my_script", "Lua"]}
                    },
                    {
                        "name": "AnimPack",
                        "description": "Create a new AnimPack file. Arg: file_name.",
                        "json_example": {"AnimPack": "my_anim"}
                    },
                    {
                        "name": "PortraitSettings",
                        "description": "Create a new portrait settings file. Args: [file_name, version, entries].",
                        "json_example": {"PortraitSettings": ["my_portraits", 3, []]}
                    },
                    {
                        "name": "VMD",
                        "description": "Create a new VMD file. Arg: file_name.",
                        "json_example": {"VMD": "my_vmd"}
                    },
                    {
                        "name": "WSModel",
                        "description": "Create a new WSModel file. Arg: file_name.",
                        "json_example": {"WSModel": "my_model"}
                    }
                ]
            }).to_string(),

            "rpfm://enums/SupportedFormats" => serde_json::json!({
                "enum": "SupportedFormats",
                "description": "Video format options for CA VP8 video files.",
                "variants": [
                    {"name": "CaVp8", "description": "CA's custom VP8 format (default)."},
                    {"name": "Ivf", "description": "Standard VP8 IVF format."}
                ],
                "json_example": "\"CaVp8\""
            }).to_string(),

            "rpfm://examples/global_search" => serde_json::json!({
                "description": "Example GlobalSearch JSON for use with global_search, global_search_replace_all, etc.",
                "example": {
                    "pattern": "old_unit_name",
                    "replace_text": "new_unit_name",
                    "case_sensitive": false,
                    "use_regex": false,
                    "sources": [{"Pack": "my_mod.pack"}],
                    "search_on": {
                        "anim": false, "anim_fragment_battle": false, "anim_pack": false,
                        "anims_table": false, "atlas": false, "audio": false, "bmd": false,
                        "db": true, "esf": false, "group_formations": false, "image": false,
                        "loc": true, "matched_combat": false, "pack": false,
                        "portrait_settings": false, "rigid_model": false, "sound_bank": false,
                        "text": true, "uic": false, "unit_variant": false, "unknown": false,
                        "video": false, "schema": false
                    },
                    "matches": {
                        "anim": [], "anim_fragment_battle": [], "anim_pack": [],
                        "anims_table": [], "atlas": [], "audio": [], "bmd": [],
                        "db": [], "esf": [], "group_formations": [], "image": [],
                        "loc": [], "matched_combat": [], "pack": [],
                        "portrait_settings": [], "rigid_model": [], "sound_bank": [],
                        "text": [], "uic": [], "unit_variant": [], "unknown": [],
                        "video": [], "schema": {"matches": []}
                    },
                    "game_key": "warhammer_3"
                },
                "notes": "The `matches` field is populated by the search results. When calling `global_search`, pass it empty. The `sources` field uses SearchSource: {\"Pack\": \"key\"}, \"ParentFiles\", \"GameFiles\", \"AssKitFiles\"."
            }).to_string(),

            "rpfm://examples/optimizer_options" => serde_json::json!({
                "description": "OptimizerOptions struct with all boolean fields for pack optimization.",
                "example": {
                    "pack_remove_itm_files": true,
                    "db_import_datacores_into_twad_key_deletes": false,
                    "db_optimize_datacored_tables": false,
                    "table_remove_duplicated_entries": true,
                    "table_remove_itm_entries": true,
                    "table_remove_itnr_entries": true,
                    "table_remove_empty_file": true,
                    "text_remove_unused_xml_map_folders": false,
                    "text_remove_unused_xml_prefab_folder": false,
                    "text_remove_agf_files": false,
                    "text_remove_model_statistics_files": false,
                    "pts_remove_unused_art_sets": false,
                    "pts_remove_unused_variants": false,
                    "pts_remove_empty_masks": false,
                    "pts_remove_empty_file": false
                },
                "field_descriptions": {
                    "pack_remove_itm_files": "Remove files identical to vanilla (Identical To Master).",
                    "db_import_datacores_into_twad_key_deletes": "Import datacored tables into TWAD key deletes.",
                    "db_optimize_datacored_tables": "Optimize datacored tables.",
                    "table_remove_duplicated_entries": "Remove duplicate rows in tables.",
                    "table_remove_itm_entries": "Remove rows identical to vanilla.",
                    "table_remove_itnr_entries": "Remove rows identical to vanilla that are not referenced.",
                    "table_remove_empty_file": "Remove tables with no rows.",
                    "text_remove_unused_xml_map_folders": "Remove unused XML files in map folders.",
                    "text_remove_unused_xml_prefab_folder": "Remove unused XML files in prefab folders.",
                    "text_remove_agf_files": "Remove AGF files.",
                    "text_remove_model_statistics_files": "Remove model statistics files.",
                    "pts_remove_unused_art_sets": "Remove unused art sets in portrait settings.",
                    "pts_remove_unused_variants": "Remove unused variants in portrait settings.",
                    "pts_remove_empty_masks": "Remove empty masks in portrait settings.",
                    "pts_remove_empty_file": "Remove empty portrait settings files."
                }
            }).to_string(),

            "rpfm://reference/initialization" => "\
RPFM MCP Server Initialization Guide
=====================================

Before you can work with PackFiles, you must initialize the server session:

Step 1: Set the game
    Call: set_game_selected(game_name: \"warhammer_3\", rebuild_dependencies: true)
    This loads the correct schemas and vanilla game data for the selected title.
    Valid game keys: pharaoh_dynasties, pharaoh, warhammer_3, troy, three_kingdoms,
    warhammer_2, warhammer, thrones_of_britannia, attila, rome_2, shogun_2,
    napoleon, empire, arena.

Step 2: Verify schema is loaded
    Call: is_schema_loaded()
    If it returns false, call update_schemas() to download the latest schemas.

Step 3: Open a PackFile
    Call: open_packfiles(paths: [\"/path/to/my_mod.pack\"])
    The response returns pack info including the pack_key you'll use for all
    subsequent operations.

Step 4: Verify dependencies (optional but recommended)
    Call: is_there_a_dependency_database(value: true)
    If false, call generate_dependencies_cache() to build the dependency database.

After initialization, use list_open_packs() to see all open pack keys at any time.
".to_string(),

            "rpfm://reference/path_conventions" => "\
Total War PackFile Path Conventions
====================================

Files inside PackFiles follow specific path conventions:

DB Tables:
    db/<table_name>/<file_name>
    Example: db/land_units_tables/my_mod
    Example: db/unit_stats_land_tables/custom_units

Localisation (Loc) files:
    text/db/<file_name>.loc
    text/<file_name>.loc
    Example: text/db/my_mod.loc

Scripts:
    script/<path>.lua
    script/campaign/mod/<script_name>.lua
    Example: script/campaign/mod/my_mod_script.lua

UI Images:
    ui/<path>.png
    Path may vary depending on the purpose of the image.

Models and Animations:
    variantmeshes/<path>
    animations/<path>
    Example: variantmeshes/wh_variantmodels/hu1/my_unit/my_unit.wsmodel

Audio:
    audio/<path>.bnk

Maps:
    terrain/tiles/battle/<map_name>/
".to_string(),

            _ => {
                return Err(McpError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: format!("Unknown resource URI: {uri}").into(),
                    data: None,
                });
            }
        };

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(content, uri.clone())],
        })
    }

    //-----------------------------------------------------------------------//
    // Completions
    //-----------------------------------------------------------------------//

    async fn complete(
        &self,
        request: CompleteRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        let argument_name = &request.argument.name;
        let partial = &request.argument.value;

        let candidates: Vec<String> = match argument_name.as_str() {
            "game_name" | "game_key" | "game" => {
                let games = vec![
                    "pharaoh_dynasties", "pharaoh", "warhammer_3", "troy",
                    "three_kingdoms", "warhammer_2", "warhammer",
                    "thrones_of_britannia", "attila", "rome_2", "shogun_2",
                    "napoleon", "empire", "arena",
                ];
                games.into_iter()
                    .filter(|g| g.starts_with(partial))
                    .map(String::from)
                    .collect()
            },
            "pack_file_type" => {
                let types = vec!["\"Boot\"", "\"Release\"", "\"Patch\"", "\"Mod\"", "\"Movie\""];
                types.into_iter()
                    .filter(|t| t.starts_with(partial))
                    .map(String::from)
                    .collect()
            },
            "format" => {
                // Could be CompressionFormat or SupportedFormats depending on tool
                let formats = vec![
                    "\"None\"", "\"Lzma1\"", "\"Lz4\"", "\"Zstd\"",
                    "\"CaVp8\"", "\"Ivf\"",
                ];
                formats.into_iter()
                    .filter(|f| f.starts_with(partial))
                    .map(String::from)
                    .collect()
            },
            "source" => {
                let sources = vec![
                    "\"PackFile\"", "\"GameFiles\"", "\"ParentFiles\"",
                    "\"AssKitFiles\"", "\"ExternalFile\"",
                ];
                sources.into_iter()
                    .filter(|s| s.starts_with(partial))
                    .map(String::from)
                    .collect()
            },
            _ => vec![],
        };

        let total = candidates.len() as u32;
        let values: Vec<String> = candidates.into_iter().take(100).collect();
        let has_more = total > 100;

        Ok(CompleteResult {
            completion: CompletionInfo {
                values,
                total: Some(total),
                has_more: Some(has_more),
            },
        })
    }

    //-----------------------------------------------------------------------//
    // Logging
    //-----------------------------------------------------------------------//

    async fn set_level(
        &self,
        _request: SetLevelRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        // Acknowledge the logging level request. RPFM uses its own logging
        // infrastructure (rpfm_telemetry/sentry), so we accept the request but
        // don't change the internal log level.
        Ok(())
    }
}

#[tool_router]
impl McpServer {

    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            tool_router: McpServer::tool_router(),
            prompt_router: McpServer::prompt_router(),
        }
    }

    //-----------------------------------------------------------------------//
    // Existing tools
    //-----------------------------------------------------------------------//

    #[tool(name = "call_command", description = "Call any IPC command directly. Use this for commands not yet wrapped as named tools.")]
    pub async fn call_command(&self, params: Parameters<CallCommandArgs>) -> Result<CallToolResult, McpError> {
        let command: Command = parse_json!(&params.0.command);
        send_and_respond!(self, "call_command", command)
    }

    //-----------------------------------------------------------------------//
    // Pack Lifecycle
    //-----------------------------------------------------------------------//

    #[tool(description = "Create a new empty PackFile.")]
    pub async fn new_pack(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "new_pack", Command::NewPack)
    }

    #[tool(description = "Open one or more PackFiles. Returns the info about the open pack.")]
    pub async fn open_packfiles(&self, params: Parameters<OpenPackfilesArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "open_packfiles", Command::OpenPackFiles(params.0.paths))
    }

    #[tool(description = "Save the pack identified by `pack_key`.")]
    pub async fn save_packfile(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "save_packfile", Command::SavePack(params.0.pack_key))
    }

    #[tool(description = "Close the pack identified by `pack_key` without saving. Any unsaved changes will be lost.")]
    pub async fn close_pack(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "close_pack", Command::ClosePack(params.0.pack_key))
    }

    #[tool(description = "Save the pack identified by `pack_key` to a new path.")]
    pub async fn save_pack_as(&self, params: Parameters<PackKeyPathArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "save_pack_as", Command::SavePackAs(params.0.pack_key, params.0.path))
    }

    #[tool(description = "Clean the pack identified by `pack_key` from corrupted files and save to a path. Use if normal save fails.")]
    pub async fn clean_and_save_pack_as(&self, params: Parameters<PackKeyPathArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "clean_and_save_pack_as", Command::CleanAndSavePackAs(params.0.pack_key, params.0.path))
    }

    #[tool(description = "Trigger a backup autosave for the pack identified by `pack_key`.")]
    pub async fn trigger_backup_autosave(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "trigger_backup_autosave", Command::TriggerBackupAutosave(params.0.pack_key))
    }

    #[tool(description = "Open all CA (vanilla) PackFiles for the selected game as one merged PackFile.")]
    pub async fn load_all_ca_pack_files(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "load_all_ca_pack_files", Command::LoadAllCAPackFiles)
    }

    //-----------------------------------------------------------------------//
    // Pack Metadata
    //-----------------------------------------------------------------------//

    #[tool(description = "Set the type of the pack identified by `pack_key`. Valid PFHFileType values: \"Boot\", \"Release\", \"Patch\", \"Mod\", \"Movie\". Example: pack_file_type = \"\\\"Mod\\\"\"")]
    pub async fn set_pack_file_type(&self, params: Parameters<SetPackFileTypeArgs>) -> Result<CallToolResult, McpError> {
        let pfh_type = parse_json!(&params.0.pack_file_type);
        send_and_respond!(self, "set_pack_file_type", Command::SetPackFileType(params.0.pack_key, pfh_type))
    }

    #[tool(description = "Change the compression format of the pack identified by `pack_key`. Valid formats: \"None\", \"Lzma1\" (legacy), \"Lz4\" (WH3 6.2+), \"Zstd\" (WH3 6.2+). Example: format = \"\\\"None\\\"\"")]
    pub async fn change_compression_format(&self, params: Parameters<ChangeCompressionFormatArgs>) -> Result<CallToolResult, McpError> {
        let format = parse_json!(&params.0.format);
        send_and_respond!(self, "change_compression_format", Command::ChangeCompressionFormat(params.0.pack_key, format))
    }

    #[tool(description = "Change whether the pack index includes timestamps for the pack identified by `pack_key`.")]
    pub async fn change_index_includes_timestamp(&self, params: Parameters<PackKeyBoolArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "change_index_includes_timestamp", Command::ChangeIndexIncludesTimestamp(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Get the file path of the pack identified by `pack_key`.")]
    pub async fn get_pack_file_path(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_pack_file_path", Command::GetPackFilePath(params.0.pack_key))
    }

    #[tool(description = "Get the file name of the pack identified by `pack_key`.")]
    pub async fn get_pack_file_name(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_pack_file_name", Command::GetPackFileName(params.0.pack_key))
    }

    #[tool(description = "Get the settings of the pack identified by `pack_key`.")]
    pub async fn get_pack_settings(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_pack_settings", Command::GetPackSettings(params.0.pack_key))
    }

    #[tool(description = "Set the settings of the pack identified by `pack_key`. The `settings` is a PackSettings JSON object containing pack-level configuration.")]
    pub async fn set_pack_settings(&self, params: Parameters<SetPackSettingsArgs>) -> Result<CallToolResult, McpError> {
        let settings = parse_json!(&params.0.settings);
        send_and_respond!(self, "set_pack_settings", Command::SetPackSettings(params.0.pack_key, settings))
    }

    #[tool(description = "Get the list of PackFiles marked as dependencies of the pack identified by `pack_key`.")]
    pub async fn get_dependency_pack_files_list(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_dependency_pack_files_list", Command::GetDependencyPackFilesList(params.0.pack_key))
    }

    #[tool(description = "Set the list of PackFiles marked as dependencies for the pack identified by `pack_key`. The `list` is a JSON array of [enabled, pack_name] pairs, e.g. [[true, \"other_mod.pack\"], [false, \"disabled_mod.pack\"]].")]
    pub async fn set_dependency_pack_files_list(&self, params: Parameters<SetDependencyPackFilesListArgs>) -> Result<CallToolResult, McpError> {
        let list = parse_json!(&params.0.list);
        send_and_respond!(self, "set_dependency_pack_files_list", Command::SetDependencyPackFilesList(params.0.pack_key, list))
    }

    //-----------------------------------------------------------------------//
    // File Operations
    //-----------------------------------------------------------------------//

    #[tool(description = "Decode a file from the pack identified by `pack_key`. The `path` is the internal file path (e.g. \"db/land_units_tables/my_mod\"). The `source` is the data source: \"PackFile\" (user mod), \"GameFiles\" (vanilla), \"ParentFiles\" (dependency mods), \"AssKitFiles\", or \"ExternalFile\". Returns the decoded file content as JSON (RFileDecoded).")]
    pub async fn decode_packed_file(&self, params: Parameters<DecodePackedFileArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "decode_packed_file", Command::DecodePackedFile(params.0.pack_key, params.0.path, params.0.source))
    }

    #[tool(description = "Create a new file inside the pack identified by `pack_key`. The `path` is the destination path (e.g. \"db/land_units_tables/my_mod\"). NewFile types: {\"DB\": [\"file_name\", \"table_name\", version]}, {\"Loc\": \"name\"}, {\"Text\": [\"name\", \"Plain\"]}, {\"AnimPack\": \"name\"}, {\"VMD\": \"name\"}, {\"WSModel\": \"name\"}, {\"PortraitSettings\": [\"name\", version, []]}.")]
    pub async fn new_packed_file(&self, params: Parameters<NewPackedFileArgs>) -> Result<CallToolResult, McpError> {
        let new_file = parse_json!(&params.0.new_file);
        send_and_respond!(self, "new_packed_file", Command::NewPackedFile(params.0.pack_key, params.0.path, new_file))
    }

    #[tool(description = "Add files from disk to the pack identified by `pack_key`. The `source_paths` are filesystem paths. The `destination_paths` is a JSON array of ContainerPath: [{\"File\": \"db/table/file\"}, {\"Folder\": \"ui/images\"}]. Optionally set `ignore_paths` to skip certain files.")]
    pub async fn add_packed_files(&self, params: Parameters<AddPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let dest: Vec<ContainerPath> = parse_json!(&params.0.destination_paths);
        send_and_respond!(self, "add_packed_files", Command::AddPackedFiles(params.0.pack_key, params.0.source_paths, dest, params.0.ignore_paths))
    }

    #[tool(description = "Add files from another PackFile to the pack identified by `pack_key`. The `source_pack_path` is the pack path. The `container_paths` is a JSON array of ContainerPath: [{\"File\": \"path\"}].")]
    pub async fn add_packed_files_from_pack_file(&self, params: Parameters<AddPackedFilesFromPackFileArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "add_packed_files_from_pack_file", Command::AddPackedFilesFromPackFile(params.0.pack_key, params.0.source_pack_path, paths))
    }

    #[tool(description = "Add files from the pack identified by `pack_key` to an AnimPack. The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}]. The `animpack_path` is the AnimPack's internal path.")]
    pub async fn add_packed_files_from_pack_file_to_animpack(&self, params: Parameters<AddPackedFilesFromPackFileToAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "add_packed_files_from_pack_file_to_animpack", Command::AddPackedFilesFromPackFileToAnimpack(params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Add files from an AnimPack to the pack identified by `pack_key`. The `source` is the DataSource (\"PackFile\", \"GameFiles\", etc.). The `animpack_path` is the AnimPack's internal path. The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}].")]
    pub async fn add_packed_files_from_animpack(&self, params: Parameters<AddPackedFilesFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "add_packed_files_from_animpack", Command::AddPackedFilesFromAnimpack(params.0.pack_key, params.0.source, params.0.animpack_path, paths))
    }

    #[tool(description = "Delete files from the pack identified by `pack_key`. The `paths` is a JSON array of ContainerPath: [{\"File\": \"path/to/file\"}, {\"Folder\": \"path/to/folder\"}].")]
    pub async fn delete_packed_files(&self, params: Parameters<ContainerPathsArg>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "delete_packed_files", Command::DeletePackedFiles(params.0.pack_key, paths))
    }

    #[tool(description = "Delete files from an AnimPack in the pack identified by `pack_key`. The `animpack_path` is the AnimPack's internal path. The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}].")]
    pub async fn delete_from_animpack(&self, params: Parameters<DeleteFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "delete_from_animpack", Command::DeleteFromAnimpack(params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Extract files from the pack identified by `pack_key` to disk. The `source_paths` is a JSON object mapping DataSource to ContainerPath arrays, e.g. {\"PackFile\": [{\"File\": \"db/table/file\"}]}. Set `export_as_tsv: true` to export tables as TSV files.")]
    pub async fn extract_packed_files(&self, params: Parameters<ExtractPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let source: BTreeMap<DataSource, Vec<ContainerPath>> = parse_json!(&params.0.source_paths);
        send_and_respond!(self, "extract_packed_files", Command::ExtractPackedFiles(params.0.pack_key, source, params.0.destination_path, params.0.export_as_tsv))
    }

    #[tool(description = "Rename files in the pack identified by `pack_key`. The `renames` is a JSON array of [old, new] ContainerPath pairs, e.g. [[{\"File\": \"old/path\"}, {\"File\": \"new/path\"}]].")]
    pub async fn rename_packed_files(&self, params: Parameters<RenamePackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let renames: Vec<(ContainerPath, ContainerPath)> = parse_json!(&params.0.renames);
        send_and_respond!(self, "rename_packed_files", Command::RenamePackedFiles(params.0.pack_key, renames))
    }

    #[tool(description = "Copy files to the internal clipboard. The `paths_by_pack` is a JSON object mapping pack key to ContainerPath arrays, e.g. {\"my_pack.pack\": [{\"File\": \"db/table/file\"}]}. Use `paste_packed_files` to paste afterwards.")]
    pub async fn copy_packed_files(&self, params: Parameters<CopyOrCutPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths_by_pack: BTreeMap<String, Vec<ContainerPath>> = parse_json!(&params.0.paths_by_pack);
        send_and_respond!(self, "copy_packed_files", Command::CopyPackedFiles(paths_by_pack))
    }

    #[tool(description = "Cut files to the internal clipboard. Same as copy, but files will be removed from the source pack on paste. The `paths_by_pack` is a JSON object mapping pack key to ContainerPath arrays. Use `paste_packed_files` to paste afterwards.")]
    pub async fn cut_packed_files(&self, params: Parameters<CopyOrCutPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths_by_pack: BTreeMap<String, Vec<ContainerPath>> = parse_json!(&params.0.paths_by_pack);
        send_and_respond!(self, "cut_packed_files", Command::CutPackedFiles(paths_by_pack))
    }

    #[tool(description = "Paste files from the internal clipboard into the pack identified by `pack_key`. The `destination_path` is the folder path to paste into (empty string for root). Returns the added paths, any cut-deleted paths, and the source pack key.")]
    pub async fn paste_packed_files(&self, params: Parameters<PastePackedFilesArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "paste_packed_files", Command::PastePackedFiles(params.0.pack_key, params.0.destination_path))
    }

    #[tool(description = "Duplicate files in-place within the same pack. Files are cloned with a numeric suffix to avoid name collisions. The `paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"db/table/file\"}].")]
    pub async fn duplicate_packed_files(&self, params: Parameters<DuplicatePackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "duplicate_packed_files", Command::DuplicatePackedFiles(params.0.pack_key, paths))
    }

    #[tool(description = "Save an edited decoded file back to the pack identified by `pack_key`. The `path` is the internal path (e.g. \"db/land_units_tables/my_mod\"). The `data` is the modified RFileDecoded JSON (same structure returned by `decode_packed_file`).")]
    pub async fn save_packed_file_from_view(&self, params: Parameters<SavePackedFileFromViewArgs>) -> Result<CallToolResult, McpError> {
        let data: RFileDecoded = parse_json!(&params.0.data);
        send_and_respond!(self, "save_packed_file_from_view", Command::SavePackedFileFromView(params.0.pack_key, params.0.path, data))
    }

    #[tool(description = "Save a file from an external program back to the pack identified by `pack_key`.")]
    pub async fn save_packed_file_from_external_view(&self, params: Parameters<SavePackedFileFromExternalViewArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "save_packed_file_from_external_view", Command::SavePackedFileFromExternalView(params.0.pack_key, params.0.internal_path, params.0.external_path))
    }

    #[tool(description = "Save files to the pack identified by `pack_key` and optionally optimize afterward. The `files` is a JSON array of RFile objects (as returned by decode/get operations). Set `optimize` to true to remove unchanged data after saving.")]
    pub async fn save_packed_files_to_pack_file_and_clean(&self, params: Parameters<SavePackedFilesToPackFileAndCleanArgs>) -> Result<CallToolResult, McpError> {
        let files: Vec<RFile> = parse_json!(&params.0.files);
        send_and_respond!(self, "save_packed_files_to_pack_file_and_clean", Command::SavePackedFilesToPackFileAndClean(params.0.pack_key, files, params.0.optimize))
    }

    #[tool(description = "Get the raw binary data of a file in the pack identified by `pack_key`.")]
    pub async fn get_packed_file_raw_data(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_packed_file_raw_data", Command::GetPackedFileRawData(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Open a file in the system's default program from the pack identified by `pack_key`. The `source` is the DataSource (\"PackFile\", \"GameFiles\", etc.). The `container_path` is a ContainerPath JSON, e.g. {\"File\": \"db/table/file\"}.")]
    pub async fn open_packed_file_in_external_program(&self, params: Parameters<OpenPackedFileInExternalProgramArgs>) -> Result<CallToolResult, McpError> {
        let cp: ContainerPath = parse_json!(&params.0.container_path);
        send_and_respond!(self, "open_packed_file_in_external_program", Command::OpenPackedFileInExternalProgram(params.0.pack_key, params.0.source, cp))
    }

    #[tool(description = "Open the folder containing the pack identified by `pack_key` in the file manager.")]
    pub async fn open_containing_folder(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "open_containing_folder", Command::OpenContainingFolder(params.0.pack_key))
    }

    #[tool(description = "Clean the decode cache for the provided paths in the pack identified by `pack_key`. The `paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"db/land_units_tables/my_mod\"}, {\"Folder\": \"db\"}].")]
    pub async fn clean_cache(&self, params: Parameters<ContainerPathsArg>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "clean_cache", Command::CleanCache(params.0.pack_key, paths))
    }

    #[tool(description = "Check if a folder exists in the pack identified by `pack_key`.")]
    pub async fn folder_exists(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "folder_exists", Command::FolderExists(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Check if a file exists in the pack identified by `pack_key`.")]
    pub async fn packed_file_exists(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "packed_file_exists", Command::PackedFileExists(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Get the info of one or more files in the pack identified by `pack_key`.")]
    pub async fn get_packed_files_info(&self, params: Parameters<PackKeyStringsArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_packed_files_info", Command::GetPackedFilesInfo(params.0.pack_key, params.0.values))
    }

    #[tool(description = "Get the info of a single file in the pack identified by `pack_key`.")]
    pub async fn get_rfile_info(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_rfile_info", Command::GetRFileInfo(params.0.pack_key, params.0.value))
    }

    //-----------------------------------------------------------------------//
    // Game Selection
    //-----------------------------------------------------------------------//

    #[tool(description = "Get the currently selected game key.")]
    pub async fn get_game_selected(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_game_selected", Command::GetGameSelected)
    }

    #[tool(description = "Set the current game. Valid game keys: pharaoh_dynasties, pharaoh, warhammer_3, troy, three_kingdoms, warhammer_2, warhammer, thrones_of_britannia, attila, rome_2, shogun_2, napoleon, empire, arena. Set rebuild_dependencies to true on first call to load schemas and vanilla data.")]
    pub async fn set_game_selected(&self, params: Parameters<SetGameSelectedArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "set_game_selected", Command::SetGameSelected(params.0.game_name, params.0.rebuild_dependencies))
    }

    //-----------------------------------------------------------------------//
    // Dependencies
    //-----------------------------------------------------------------------//

    #[tool(description = "Generate the dependencies cache for the selected game.")]
    pub async fn generate_dependencies_cache(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "generate_dependencies_cache", Command::GenerateDependenciesCache)
    }

    #[tool(description = "Rebuild dependencies. Pass true for full rebuild, false for mod-specific only.")]
    pub async fn rebuild_dependencies(&self, params: Parameters<BoolArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "rebuild_dependencies", Command::RebuildDependencies(params.0.value))
    }

    #[tool(description = "Check if there is a dependency database loaded. Pass true to ensure AssKit data is included.")]
    pub async fn is_there_a_dependency_database(&self, params: Parameters<BoolArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "is_there_a_dependency_database", Command::IsThereADependencyDatabase(params.0.value))
    }

    #[tool(description = "Get the table names of all DB files in dependency PackFiles.")]
    pub async fn get_table_list_from_dependency_pack_file(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_table_list_from_dependency_pack_file", Command::GetTableListFromDependencyPackFile)
    }

    #[tool(description = "Get custom table names (start_pos_, twad_ prefixes) from the schema.")]
    pub async fn get_custom_table_list(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_custom_table_list", Command::GetCustomTableList)
    }

    #[tool(description = "Get the version of a table from the dependency database.")]
    pub async fn get_table_version_from_dependency_pack_file(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_table_version_from_dependency_pack_file", Command::GetTableVersionFromDependencyPackFile(params.0.value))
    }

    #[tool(description = "Get the definition of a table from the dependency database.")]
    pub async fn get_table_definition_from_dependency_pack_file(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_table_definition_from_dependency_pack_file", Command::GetTableDefinitionFromDependencyPackFile(params.0.value))
    }

    #[tool(description = "Get table data from dependencies by table name.")]
    pub async fn get_tables_from_dependencies(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_tables_from_dependencies", Command::GetTablesFromDependencies(params.0.value))
    }

    #[tool(description = "Import files from dependencies into the pack identified by `pack_key`. The `paths` is a JSON object mapping DataSource to ContainerPath arrays, e.g. {\"GameFiles\": [{\"File\": \"db/table/file\"}]}.")]
    pub async fn import_dependencies_to_open_pack_file(&self, params: Parameters<ImportDependenciesArgs>) -> Result<CallToolResult, McpError> {
        let paths: BTreeMap<DataSource, Vec<ContainerPath>> = parse_json!(&params.0.paths);
        send_and_respond!(self, "import_dependencies_to_open_pack_file", Command::ImportDependenciesToOpenPackFile(params.0.pack_key, paths))
    }

    #[tool(description = "Get files from all known sources (PackFile, GameFiles, ParentFiles). The `paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"db/land_units_tables/some_file\"}]. Set `lowercase` to true to normalize path casing.")]
    pub async fn get_rfiles_from_all_sources(&self, params: Parameters<GetRFilesFromAllSourcesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "get_rfiles_from_all_sources", Command::GetRFilesFromAllSources(paths, params.0.lowercase))
    }

    #[tool(description = "Get all file names under a path prefix across all data sources (PackFile, GameFiles, ParentFiles). The `path` is a ContainerPath JSON, e.g. {\"Folder\": \"db/land_units_tables\"} to list all files under that folder.")]
    pub async fn get_packed_files_names_starting_with_path_from_all_sources(&self, params: Parameters<ContainerPathArg>) -> Result<CallToolResult, McpError> {
        let path: ContainerPath = parse_json!(&params.0.path);
        send_and_respond!(self, "get_packed_files_names_starting_with_path_from_all_sources", Command::GetPackedFilesNamesStartingWitPathFromAllSources(path))
    }

    #[tool(description = "Get local art set IDs from campaign_character_arts_tables in the pack identified by `pack_key`.")]
    pub async fn local_art_set_ids(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "local_art_set_ids", Command::LocalArtSetIds(params.0.pack_key))
    }

    #[tool(description = "Get art set IDs from dependencies' campaign_character_arts_tables.")]
    pub async fn dependencies_art_set_ids(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "dependencies_art_set_ids", Command::DependenciesArtSetIds)
    }

    //-----------------------------------------------------------------------//
    // Search
    //-----------------------------------------------------------------------//

    #[tool(description = "Run a global search across the pack identified by `pack_key`. The `search` is a GlobalSearch JSON with fields: pattern (string), replace_text (string), case_sensitive (bool), use_regex (bool), search_on ({db: bool, loc: bool, text: bool, ...}), sources ([{\"Pack\": \"key\"}]), game_key (string). See the `rpfm://examples/global_search` resource for a full example.")]
    pub async fn global_search(&self, params: Parameters<GlobalSearchArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json!(&params.0.search);
        send_and_respond!(self, "global_search", Command::GlobalSearch(params.0.pack_key, search))
    }

    #[tool(description = "Replace specific matches in a global search for the pack identified by `pack_key`. The `search` is the same GlobalSearch JSON used in `global_search` (see `rpfm://examples/global_search` resource). The `matches` is a JSON array of MatchHolder objects from the search results — include only the matches you want to replace.")]
    pub async fn global_search_replace_matches(&self, params: Parameters<GlobalSearchReplaceMatchesArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json!(&params.0.search);
        let matches = parse_json!(&params.0.matches);
        send_and_respond!(self, "global_search_replace_matches", Command::GlobalSearchReplaceMatches(params.0.pack_key, search, matches))
    }

    #[tool(description = "Replace all matches in a global search for the pack identified by `pack_key`. The `search` is a GlobalSearch JSON with the `replace_text` field set to the replacement string. See `rpfm://examples/global_search` resource for the full structure.")]
    pub async fn global_search_replace_all(&self, params: Parameters<GlobalSearchArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json!(&params.0.search);
        send_and_respond!(self, "global_search_replace_all", Command::GlobalSearchReplaceAll(params.0.pack_key, search))
    }

    #[tool(description = "Find all references to a value in the pack identified by `pack_key`. The `reference_map` is a JSON object mapping table names to column name arrays, e.g. {\"land_units_tables\": [\"key\", \"unit\"]}. The `value` is the string to search for across those columns.")]
    pub async fn search_references(&self, params: Parameters<SearchReferencesArgs>) -> Result<CallToolResult, McpError> {
        let map: HashMap<String, Vec<String>> = parse_json!(&params.0.reference_map);
        send_and_respond!(self, "search_references", Command::SearchReferences(params.0.pack_key, map, params.0.value))
    }

    #[tool(description = "Get valid reference values for columns in a table definition for the pack identified by `pack_key`. The `definition` is a Definition JSON (as returned by `get_table_definition_from_dependency_pack_file`). Set `force` to true to regenerate cached reference data.")]
    pub async fn get_reference_data_from_definition(&self, params: Parameters<GetReferenceDataFromDefinitionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json!(&params.0.definition);
        send_and_respond!(self, "get_reference_data_from_definition", Command::GetReferenceDataFromDefinition(params.0.pack_key, params.0.table_name, def, params.0.force))
    }

    #[tool(description = "Go to the definition of a reference in the pack identified by `pack_key`. Provide table name, column name, and values to search.")]
    pub async fn go_to_definition(&self, params: Parameters<GoToDefinitionArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "go_to_definition", Command::GoToDefinition(params.0.pack_key, params.0.table_name, params.0.column_name, params.0.values))
    }

    #[tool(description = "Go to a loc key's location in the pack identified by `pack_key`.")]
    pub async fn go_to_loc(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "go_to_loc", Command::GoToLoc(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Get the source data of a loc key in the pack identified by `pack_key`.")]
    pub async fn get_source_data_from_loc_key(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_source_data_from_loc_key", Command::GetSourceDataFromLocKey(params.0.pack_key, params.0.value))
    }

    //-----------------------------------------------------------------------//
    // Schema
    //-----------------------------------------------------------------------//

    #[tool(description = "Save the provided schema to disk. The `schema` is the full Schema JSON object (as returned by `get_schema`). Use this after modifying definitions or applying patches.")]
    pub async fn save_schema(&self, params: Parameters<SaveSchemaArgs>) -> Result<CallToolResult, McpError> {
        let schema = parse_json!(&params.0.schema);
        send_and_respond!(self, "save_schema", Command::SaveSchema(schema))
    }

    #[tool(description = "Update the currently loaded schema with data from the game's Assembly Kit.")]
    pub async fn update_current_schema_from_asskit(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_current_schema_from_asskit", Command::UpdateCurrentSchemaFromAssKit)
    }

    #[tool(description = "Update schemas from the remote repository.")]
    pub async fn update_schemas(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_schemas", Command::UpdateSchemas)
    }

    #[tool(description = "Check if a schema is currently loaded.")]
    pub async fn is_schema_loaded(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "is_schema_loaded", Command::IsSchemaLoaded)
    }

    #[tool(description = "Get the current schema.")]
    pub async fn get_schema(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_schema", Command::Schema)
    }

    #[tool(description = "Get all definitions for a table name.")]
    pub async fn definitions_by_table_name(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "definitions_by_table_name", Command::DefinitionsByTableName(params.0.value))
    }

    #[tool(description = "Get a specific definition by table name and version.")]
    pub async fn definition_by_table_name_and_version(&self, params: Parameters<StringI32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "definition_by_table_name_and_version", Command::DefinitionByTableNameAndVersion(params.0.name, params.0.version))
    }

    #[tool(description = "Delete a definition by table name and version.")]
    pub async fn delete_definition(&self, params: Parameters<StringI32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "delete_definition", Command::DeleteDefinition(params.0.name, params.0.version))
    }

    #[tool(description = "Get columns from other tables that reference the given table's definition. The `definition` is a Definition JSON (as returned by `get_table_definition_from_dependency_pack_file` or `definitions_by_table_name`).")]
    pub async fn referencing_columns_for_definition(&self, params: Parameters<ReferencingColumnsForDefinitionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json!(&params.0.definition);
        send_and_respond!(self, "referencing_columns_for_definition", Command::ReferencingColumnsForDefinition(params.0.table_name, def))
    }

    #[tool(description = "Get the processed fields from a definition with bitwise expansion and enum conversions applied (useful for display). The `definition` is a Definition JSON (as returned by `get_table_definition_from_dependency_pack_file`).")]
    pub async fn fields_processed(&self, params: Parameters<DefinitionArg>) -> Result<CallToolResult, McpError> {
        let def = parse_json!(&params.0.definition);
        send_and_respond!(self, "fields_processed", Command::FieldsProcessed(def))
    }

    #[tool(description = "Save local schema patches to customize column metadata without modifying the upstream schema. The `patches` is a JSON object mapping table names to DefinitionPatch objects, e.g. {\"land_units_tables\": {\"field_patches\": {...}}}.")]
    pub async fn save_local_schema_patch(&self, params: Parameters<SchemaPatchArgs>) -> Result<CallToolResult, McpError> {
        let patches = parse_json!(&params.0.patches);
        send_and_respond!(self, "save_local_schema_patch", Command::SaveLocalSchemaPatch(patches))
    }

    #[tool(description = "Remove local schema patches for a table.")]
    pub async fn remove_local_schema_patches_for_table(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "remove_local_schema_patches_for_table", Command::RemoveLocalSchemaPatchesForTable(params.0.value))
    }

    #[tool(description = "Remove local schema patches for a specific field in a table.")]
    pub async fn remove_local_schema_patches_for_table_and_field(&self, params: Parameters<SettingsSetStringArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "remove_local_schema_patches_for_table_and_field", Command::RemoveLocalSchemaPatchesForTableAndField(params.0.key, params.0.value))
    }

    #[tool(description = "Import a schema patch from an external source. The `patches` is a JSON object mapping table names to DefinitionPatch objects (same format as `save_local_schema_patch`).")]
    pub async fn import_schema_patch(&self, params: Parameters<SchemaPatchArgs>) -> Result<CallToolResult, McpError> {
        let patches = parse_json!(&params.0.patches);
        send_and_respond!(self, "import_schema_patch", Command::ImportSchemaPatch(patches))
    }

    //-----------------------------------------------------------------------//
    // Table Operations
    //-----------------------------------------------------------------------//

    #[tool(description = "Merge multiple compatible tables into one in the pack identified by `pack_key`. The `paths` is a JSON array of ContainerPath for the tables to merge, e.g. [{\"File\": \"db/land_units_tables/table1\"}, {\"File\": \"db/land_units_tables/table2\"}]. The `merged_path` is the destination path. Set `delete_source` to true to remove the original files.")]
    pub async fn merge_files(&self, params: Parameters<MergeFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "merge_files", Command::MergeFiles(params.0.pack_key, paths, params.0.merged_path, params.0.delete_source))
    }

    #[tool(description = "Update a table to the latest schema version in the pack identified by `pack_key`. The `value` is a ContainerPath JSON, e.g. {\"File\": \"db/land_units_tables/my_mod\"}.")]
    pub async fn update_table(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        let path: ContainerPath = parse_json!(&params.0.value);
        send_and_respond!(self, "update_table", Command::UpdateTable(params.0.pack_key, path))
    }

    #[tool(description = "Trigger a cascade edition on all referenced data in the pack identified by `pack_key`. When a key value changes, this propagates the change to all referencing tables. The `definition` is a Definition JSON for the source table. The `changes` is a JSON array of [field, old_value, new_value] tuples, e.g. [[field_json, \"old_key\", \"new_key\"]].")]
    pub async fn cascade_edition(&self, params: Parameters<CascadeEditionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json!(&params.0.definition);
        let changes = parse_json!(&params.0.changes);
        send_and_respond!(self, "cascade_edition", Command::CascadeEdition(params.0.pack_key, params.0.table_name, def, changes))
    }

    #[tool(description = "Get table paths by table name from the pack identified by `pack_key`.")]
    pub async fn get_tables_by_table_name(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_tables_by_table_name", Command::GetTablesByTableName(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Add keys to the key_deletes table in the pack identified by `pack_key`.")]
    pub async fn add_keys_to_key_deletes(&self, params: Parameters<AddKeysToKeyDeletesArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "add_keys_to_key_deletes", Command::AddKeysToKeyDeletes(params.0.pack_key, params.0.table_file_name, params.0.key_table_name, params.0.keys))
    }

    #[tool(description = "Export a table from the pack identified by `pack_key` to a TSV file.")]
    pub async fn export_tsv(&self, params: Parameters<TsvExportArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "export_tsv", Command::ExportTSV(params.0.pack_key, params.0.table_path, params.0.tsv_path, DataSource::PackFile))
    }

    #[tool(description = "Import a TSV file to a table in the pack identified by `pack_key`.")]
    pub async fn import_tsv(&self, params: Parameters<TsvImportArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "import_tsv", Command::ImportTSV(params.0.pack_key, params.0.table_path, params.0.tsv_path))
    }

    //-----------------------------------------------------------------------//
    // Diagnostics
    //-----------------------------------------------------------------------//

    #[tool(description = "Run a full diagnostics check over all open packs.")]
    pub async fn diagnostics_check(&self, params: Parameters<DiagnosticsCheckArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "diagnostics_check", Command::DiagnosticsCheck(params.0.ignored, params.0.check_ak_only_refs))
    }

    #[tool(description = "Update diagnostics incrementally for changed files across all open packs. The `diagnostics` is the Diagnostics JSON from a previous `diagnostics_check` call. The `paths` is a JSON array of ContainerPath for the files that changed, e.g. [{\"File\": \"db/land_units_tables/my_mod\"}].")]
    pub async fn diagnostics_update(&self, params: Parameters<DiagnosticsUpdateArgs>) -> Result<CallToolResult, McpError> {
        let diag = parse_json!(&params.0.diagnostics);
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "diagnostics_update", Command::DiagnosticsUpdate(diag, paths, params.0.check_ak_only_refs))
    }

    #[tool(description = "Add a line to the ignored diagnostics list for the pack identified by `pack_key`.")]
    pub async fn add_line_to_pack_ignored_diagnostics(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "add_line_to_pack_ignored_diagnostics", Command::AddLineToPackIgnoredDiagnostics(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Export missing table definitions for the pack identified by `pack_key` to a file (for debugging).")]
    pub async fn get_missing_definitions(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_missing_definitions", Command::GetMissingDefinitions(params.0.pack_key))
    }

    //-----------------------------------------------------------------------//
    // Notes
    //-----------------------------------------------------------------------//

    #[tool(description = "Get all notes under a path in the pack identified by `pack_key`.")]
    pub async fn notes_for_path(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "notes_for_path", Command::NotesForPath(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Add a note to the pack identified by `pack_key`. The `note` is a Note JSON object with fields: path (string — the file or folder path to attach the note to), id (u64), text (string — the note content).")]
    pub async fn add_note(&self, params: Parameters<AddNoteArgs>) -> Result<CallToolResult, McpError> {
        let note = parse_json!(&params.0.note);
        send_and_respond!(self, "add_note", Command::AddNote(params.0.pack_key, note))
    }

    #[tool(description = "Delete a note by path and ID in the pack identified by `pack_key`.")]
    pub async fn delete_note(&self, params: Parameters<DeleteNoteArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "delete_note", Command::DeleteNote(params.0.pack_key, params.0.path, params.0.id))
    }

    //-----------------------------------------------------------------------//
    // Optimization
    //-----------------------------------------------------------------------//

    #[tool(description = "Optimize the pack identified by `pack_key` by removing unchanged/duplicate data. The `options` is an OptimizerOptions JSON with boolean fields: pack_remove_itm_files, table_remove_duplicated_entries, table_remove_itm_entries, table_remove_itnr_entries, table_remove_empty_file, db_optimize_datacored_tables, etc. See the `rpfm://examples/optimizer_options` resource for all fields.")]
    pub async fn optimize_pack_file(&self, params: Parameters<OptimizePackFileArgs>) -> Result<CallToolResult, McpError> {
        let options = parse_json!(&params.0.options);
        send_and_respond!(self, "optimize_pack_file", Command::OptimizePackFile(params.0.pack_key, options))
    }

    #[tool(description = "Get the default optimizer options.")]
    pub async fn get_optimizer_options(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_optimizer_options", Command::OptimizerOptions)
    }

    //-----------------------------------------------------------------------//
    // Updates
    //-----------------------------------------------------------------------//

    #[tool(description = "Check if there is an RPFM update available.")]
    pub async fn check_updates(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "check_updates", Command::CheckUpdates)
    }

    #[tool(description = "Check if there is a schema update available.")]
    pub async fn check_schema_updates(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "check_schema_updates", Command::CheckSchemaUpdates)
    }

    #[tool(description = "Check for Lua autogen updates.")]
    pub async fn check_lua_autogen_updates(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "check_lua_autogen_updates", Command::CheckLuaAutogenUpdates)
    }

    #[tool(description = "Check for Empire/Napoleon Assembly Kit updates.")]
    pub async fn check_empire_and_napoleon_ak_updates(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "check_empire_and_napoleon_ak_updates", Command::CheckEmpireAndNapoleonAKUpdates)
    }

    #[tool(description = "Check for translation updates.")]
    pub async fn check_translations_updates(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "check_translations_updates", Command::CheckTranslationsUpdates)
    }

    #[tool(description = "Update the Lua autogen repository.")]
    pub async fn update_lua_autogen(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_lua_autogen", Command::UpdateLuaAutogen)
    }

    #[tool(description = "Update the program to the latest version.")]
    pub async fn update_main_program(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_main_program", Command::UpdateMainProgram)
    }

    #[tool(description = "Update the Empire/Napoleon Assembly Kit files.")]
    pub async fn update_empire_and_napoleon_ak(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_empire_and_napoleon_ak", Command::UpdateEmpireAndNapoleonAK)
    }

    #[tool(description = "Update the translations repository.")]
    pub async fn update_translations(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_translations", Command::UpdateTranslations)
    }

    //-----------------------------------------------------------------------//
    // Settings Getters
    //-----------------------------------------------------------------------//

    #[tool(description = "Get a boolean setting value by key.")]
    pub async fn settings_get_bool(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_bool", Command::SettingsGetBool(params.0.value))
    }

    #[tool(description = "Get an i32 setting value by key.")]
    pub async fn settings_get_i32(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_i32", Command::SettingsGetI32(params.0.value))
    }

    #[tool(description = "Get an f32 setting value by key.")]
    pub async fn settings_get_f32(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_f32", Command::SettingsGetF32(params.0.value))
    }

    #[tool(description = "Get a string setting value by key.")]
    pub async fn settings_get_string(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_string", Command::SettingsGetString(params.0.value))
    }

    #[tool(description = "Get a PathBuf setting value by key.")]
    pub async fn settings_get_path_buf(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_path_buf", Command::SettingsGetPathBuf(params.0.value))
    }

    #[tool(description = "Get a Vec<String> setting value by key.")]
    pub async fn settings_get_vec_string(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_vec_string", Command::SettingsGetVecString(params.0.value))
    }

    #[tool(description = "Get a raw bytes setting value by key.")]
    pub async fn settings_get_vec_raw(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_vec_raw", Command::SettingsGetVecRaw(params.0.value))
    }

    #[tool(description = "Get all settings at once (bool, i32, f32, string, raw_data, and vec_string maps).")]
    pub async fn settings_get_all(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_get_all", Command::SettingsGetAll)
    }

    //-----------------------------------------------------------------------//
    // Settings Setters
    //-----------------------------------------------------------------------//

    #[tool(description = "Set a boolean setting value.")]
    pub async fn settings_set_bool(&self, params: Parameters<SettingsSetBoolArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_bool", Command::SettingsSetBool(params.0.key, params.0.value))
    }

    #[tool(description = "Set an i32 setting value.")]
    pub async fn settings_set_i32(&self, params: Parameters<SettingsSetI32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_i32", Command::SettingsSetI32(params.0.key, params.0.value))
    }

    #[tool(description = "Set an f32 setting value.")]
    pub async fn settings_set_f32(&self, params: Parameters<SettingsSetF32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_f32", Command::SettingsSetF32(params.0.key, params.0.value))
    }

    #[tool(description = "Set a string setting value.")]
    pub async fn settings_set_string(&self, params: Parameters<SettingsSetStringArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_string", Command::SettingsSetString(params.0.key, params.0.value))
    }

    #[tool(description = "Set a PathBuf setting value.")]
    pub async fn settings_set_path_buf(&self, params: Parameters<SettingsSetPathBufArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_path_buf", Command::SettingsSetPathBuf(params.0.key, params.0.value))
    }

    #[tool(description = "Set a Vec<String> setting value.")]
    pub async fn settings_set_vec_string(&self, params: Parameters<SettingsSetVecStringArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_vec_string", Command::SettingsSetVecString(params.0.key, params.0.value))
    }

    #[tool(description = "Set a raw bytes setting value.")]
    pub async fn settings_set_vec_raw(&self, params: Parameters<SettingsSetVecRawArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_set_vec_raw", Command::SettingsSetVecRaw(params.0.key, params.0.value))
    }

    #[tool(description = "Backup the current settings to memory.")]
    pub async fn backup_settings(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "backup_settings", Command::BackupSettings)
    }

    #[tool(description = "Clear all settings and reset to defaults.")]
    pub async fn clear_settings(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "clear_settings", Command::ClearSettings)
    }

    #[tool(description = "Restore settings from the backup.")]
    pub async fn restore_backup_settings(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "restore_backup_settings", Command::RestoreBackupSettings)
    }

    //-----------------------------------------------------------------------//
    // Path Queries
    //-----------------------------------------------------------------------//

    #[tool(description = "Get the config path.")]
    pub async fn config_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "config_path", Command::ConfigPath)
    }

    #[tool(description = "Get the Assembly Kit path for the current game.")]
    pub async fn assembly_kit_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "assembly_kit_path", Command::AssemblyKitPath)
    }

    #[tool(description = "Get the backup autosave path.")]
    pub async fn backup_autosave_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "backup_autosave_path", Command::BackupAutosavePath)
    }

    #[tool(description = "Get the old Assembly Kit data path.")]
    pub async fn old_ak_data_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "old_ak_data_path", Command::OldAkDataPath)
    }

    #[tool(description = "Get the schemas path.")]
    pub async fn schemas_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "schemas_path", Command::SchemasPath)
    }

    #[tool(description = "Get the table profiles path.")]
    pub async fn table_profiles_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "table_profiles_path", Command::TableProfilesPath)
    }

    #[tool(description = "Get the translations local path.")]
    pub async fn translations_local_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "translations_local_path", Command::TranslationsLocalPath)
    }

    #[tool(description = "Get the dependencies cache path.")]
    pub async fn dependencies_cache_path(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "dependencies_cache_path", Command::DependenciesCachePath)
    }

    #[tool(description = "Clear a config path.")]
    pub async fn settings_clear_path(&self, params: Parameters<PathArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "settings_clear_path", Command::SettingsClearPath(params.0.path))
    }

    //-----------------------------------------------------------------------//
    // Specialized
    //-----------------------------------------------------------------------//

    #[tool(description = "Get the info about the pack identified by `pack_key` and the list of files it contains.")]
    pub async fn open_pack_info(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "open_pack_info", Command::GetPackFileDataForTreeView(params.0.pack_key))
    }

    #[tool(description = "Initialize a MyMod folder for mod development.")]
    pub async fn initialize_my_mod_folder(&self, params: Parameters<InitializeMyModFolderArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "initialize_my_mod_folder", Command::InitializeMyModFolder(params.0.name, params.0.game, params.0.sublime, params.0.vscode, params.0.gitignore))
    }

    #[tool(description = "Live export the pack identified by `pack_key` to the game folder for testing.")]
    pub async fn live_export(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "live_export", Command::LiveExport(params.0.pack_key))
    }

    #[tool(description = "Patch the SiegeAI of a Siege Map in the pack identified by `pack_key` for Warhammer games.")]
    pub async fn patch_siege_ai(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "patch_siege_ai", Command::PatchSiegeAI(params.0.pack_key))
    }

    #[tool(description = "Pack map tiles into the pack identified by `pack_key`. The `tile_maps` is a list of tile map file paths on disk. The `tiles` is a JSON array of [path, name] pairs, e.g. [[\"/path/to/tile\", \"tile_name\"]].")]
    pub async fn pack_map(&self, params: Parameters<PackMapArgs>) -> Result<CallToolResult, McpError> {
        let tiles: Vec<(PathBuf, String)> = parse_json!(&params.0.tiles);
        send_and_respond!(self, "pack_map", Command::PackMap(params.0.pack_key, params.0.tile_maps, tiles))
    }

    #[tool(description = "Generate all missing loc entries for the pack identified by `pack_key`.")]
    pub async fn generate_missing_loc_data(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "generate_missing_loc_data", Command::GenerateMissingLocData(params.0.pack_key))
    }

    #[tool(description = "Get pack translation data for a language from the pack identified by `pack_key`.")]
    pub async fn get_pack_translation(&self, params: Parameters<GetPackTranslationArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_pack_translation", Command::GetPackTranslation(params.0.pack_key, params.0.language))
    }

    #[tool(description = "Get campaign IDs for starpos building in the pack identified by `pack_key`.")]
    pub async fn build_starpos_get_campaign_ids(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "build_starpos_get_campaign_ids", Command::BuildStarposGetCampaingIds(params.0.pack_key))
    }

    #[tool(description = "Check if victory conditions file exists for starpos building in the pack identified by `pack_key`.")]
    pub async fn build_starpos_check_victory_conditions(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "build_starpos_check_victory_conditions", Command::BuildStarposCheckVictoryConditions(params.0.pack_key))
    }

    #[tool(description = "Build starpos (pre-processing step) for the pack identified by `pack_key`.")]
    pub async fn build_starpos(&self, params: Parameters<BuildStarposArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "build_starpos", Command::BuildStarpos(params.0.pack_key, params.0.campaign_id, params.0.process_hlp_spd))
    }

    #[tool(description = "Build starpos (post-processing step) for the pack identified by `pack_key`.")]
    pub async fn build_starpos_post(&self, params: Parameters<BuildStarposArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "build_starpos_post", Command::BuildStarposPost(params.0.pack_key, params.0.campaign_id, params.0.process_hlp_spd))
    }

    #[tool(description = "Clean up starpos temporary files for the pack identified by `pack_key`.")]
    pub async fn build_starpos_cleanup(&self, params: Parameters<BuildStarposArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "build_starpos_cleanup", Command::BuildStarposCleanup(params.0.pack_key, params.0.campaign_id, params.0.process_hlp_spd))
    }

    #[tool(description = "Update animation IDs with an offset in the pack identified by `pack_key`.")]
    pub async fn update_anim_ids(&self, params: Parameters<UpdateAnimIdsArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "update_anim_ids", Command::UpdateAnimIds(params.0.pack_key, params.0.starting_id, params.0.offset))
    }

    #[tool(description = "Get animation paths by skeleton name.")]
    pub async fn get_anim_paths_by_skeleton_name(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_anim_paths_by_skeleton_name", Command::GetAnimPathsBySkeletonName(params.0.value))
    }

    #[tool(description = "Export a RigidModel to glTF format. The `rigid_model` is a RigidModel JSON object (as returned by decoding a .rigid_model_v2 file with `decode_packed_file`). The `output_path` is the destination file path on disk.")]
    pub async fn export_rigid_to_gltf(&self, params: Parameters<ExportRigidToGltfArgs>) -> Result<CallToolResult, McpError> {
        let rigid = parse_json!(&params.0.rigid_model);
        send_and_respond!(self, "export_rigid_to_gltf", Command::ExportRigidToGltf(rigid, params.0.output_path))
    }

    #[tool(description = "Change the format of a ca_vp8 video file in the pack identified by `pack_key`. Valid formats: \"CaVp8\" (CA custom VP8) or \"Ivf\" (standard VP8 IVF).")]
    pub async fn set_video_format(&self, params: Parameters<SetVideoFormatArgs>) -> Result<CallToolResult, McpError> {
        let format = parse_json!(&params.0.format);
        send_and_respond!(self, "set_video_format", Command::SetVideoFormat(params.0.pack_key, params.0.path, format))
    }

    //-----------------------------------------------------------------------//
    // Multi-Pack Management
    //-----------------------------------------------------------------------//

    #[tool(description = "List all currently open packs with their keys and metadata. Use this to get valid pack_key values for other tools.")]
    pub async fn list_open_packs(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "list_open_packs", Command::ListOpenPacks)
    }

    //-----------------------------------------------------------------------//
    // Additional tools
    //-----------------------------------------------------------------------//

    #[tool(description = "Close all currently open packs without saving. Any unsaved changes will be lost.")]
    pub async fn close_all_packs(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "close_all_packs", Command::CloseAllPacks)
    }

}

//-------------------------------------------------------------------------------//
//                              MCP Prompts
//-------------------------------------------------------------------------------//

#[prompt_router]
impl McpServer {

    #[prompt(name = "open_and_inspect_pack", description = "Walk through opening a PackFile and inspecting its contents.")]
    pub async fn open_and_inspect_pack(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user inspect a Total War PackFile using the RPFM MCP server.

Follow these steps in order:

1. **Open the pack** – Call `open_packfiles` with the filesystem path(s) the user provides.
   The response contains one or more pack keys; remember them for subsequent calls.

2. **Select the game** – Call `set_game_selected` with the correct game key (e.g. `\"warhammer_3\"`)
   and `rebuild_dependencies: true` so that schemas and dependency data are loaded.

3. **List pack contents** – Call `open_pack_info` with the pack key to get the full file tree.
   Present the tree to the user in a readable format.

4. **Decode specific files** – When the user asks about a file, call `decode_packed_file` with the
   pack key, the internal path (e.g. `\"db/land_units_tables/my_table\"`), and
   `source: \"PackFile\"`. The decoded JSON will contain the table rows, schema, etc.

5. **Inspect metadata** – Use `get_pack_settings`, `get_pack_file_name`, or
   `get_dependency_pack_files_list` to answer questions about the pack itself.

Important notes:
- Always call `list_open_packs` if you are unsure which pack key to use.
- If a file fails to decode, check `is_schema_loaded`; if false, call `update_schemas` first.
- When done, optionally call `close_pack` to free resources.
",
        )]
    }

    #[prompt(name = "edit_db_table", description = "Guide for reading, modifying, and saving a DB table inside a pack.")]
    pub async fn edit_db_table(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user edit a DB table inside a Total War PackFile.

Workflow:

1. **Open the pack** – `open_packfiles` → note the `pack_key`.
2. **Set the game** – `set_game_selected` with `rebuild_dependencies: true`.
3. **Decode the table** – `decode_packed_file` with the DB path
   (e.g. `\"db/unit_stats_land_tables/my_table\"`) and `source: \"PackFile\"`.
   The response is an `RFileDecoded` JSON containing the table data and definition.
4. **Modify rows** – Edit the decoded JSON: add, remove, or change rows/cells.
   Each row is typically a list of `DecodedData` values matching the table's
   fields processed list (retrievable via the `FieldsProcessed` message).
5. **Save back** – Call `save_packed_file_from_view` with the pack key, the same path,
   and the modified `RFileDecoded` JSON as the `data` parameter.
6. **Save the pack** – Call `save_packfile` (or `save_pack_as` for a new path).

Tips:
- Use `get_table_definition_from_dependency_pack_file` to see the expected column schema.
- Use `get_reference_data_from_definition` to discover valid values for referenced columns.
- After saving, you can run `diagnostics_check` to validate the pack.
",
        )]
    }

    #[prompt(name = "create_new_mod", description = "Step-by-step guide for creating a new mod PackFile from scratch.")]
    pub async fn create_new_mod(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user create a new Total War mod from scratch.

Workflow:

1. **Set the game** – `set_game_selected` with the target game key and
   `rebuild_dependencies: true`.

2. **Create the pack** – `new_pack` returns a new empty pack and its pack key.

3. **Set pack type** – `set_pack_file_type` to `\"Mod\"` (the standard type for mods).

4. **Add DB tables** – For each table you need:
   a. Call `new_packed_file` with the pack key, the path (e.g. `\"db/land_units_tables/my_mod\"`),
      and the `new_file` JSON set to `\"DB\"` with the table name.
   b. Decode, edit, and save as described in the `edit_db_table` workflow.

5. **Add Loc files** – For localisation:
   a. `new_packed_file` with path `\"text/db/my_mod.loc\"` and `new_file` set to `\"Loc\"`.
   b. Decode, add key/value rows, and save.

6. **Add other files** – Use `add_packed_files` to import assets from disk (images, models, etc.).

7. **Save the pack** – `save_pack_as` to write the final `.pack` file to disk.

Optional steps:
- `initialize_my_mod_folder` to set up a mod development folder with IDE support.
- `optimize_pack_file` to strip unchanged rows that match vanilla data.
- `diagnostics_check` to validate everything before release.
",
        )]
    }

    #[prompt(name = "search_and_replace", description = "Find and replace values across all files in a pack.")]
    pub async fn search_and_replace(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user search for and replace data across a PackFile.

Workflow:

1. **Open the pack** and **set the game** (see `open_and_inspect_pack` prompt).

2. **Run a global search** – Call `global_search` with the pack key and a `GlobalSearch`
   JSON object. The search object specifies the pattern, whether to use regex, which file
   types to include (DB, Loc, Text), and the replacement string.

3. **Review matches** – The response contains all matches grouped by file.
   Present them to the user for review.

4. **Replace selectively** – Call `global_search_replace_matches` with the same search
   object and a `Vec<MatchHolder>` containing only the matches the user approved.

5. **Or replace all** – If the user confirms a blanket replace, call
   `global_search_replace_all` with the search object.

6. **Save** – `save_packfile` to persist changes.

Related tools:
- `search_references` – Find all rows that reference a specific value across tables.
- `go_to_definition` – Jump to where a referenced key is defined.
- `go_to_loc` – Find the loc entry for a given key.
",
        )]
    }

    #[prompt(name = "manage_dependencies", description = "Set up and work with game dependencies and vanilla data.")]
    pub async fn manage_dependencies(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user work with dependency data (vanilla game files).

Workflow:

1. **Set the game** – `set_game_selected` with `rebuild_dependencies: true`.

2. **Check dependency database** – `is_there_a_dependency_database` with `true` to verify
   that game data (including Assembly Kit data) is loaded.
   If it returns false, call `generate_dependencies_cache` first.

3. **Browse vanilla tables** – `get_table_list_from_dependency_pack_file` returns all
   DB table names from the vanilla game files.

4. **Read vanilla data** – `get_tables_from_dependencies` with a table name to get
   all rows from vanilla for that table.

5. **Get definitions** – `get_table_definition_from_dependency_pack_file` to get the
   schema definition for any table.

6. **Import from vanilla** – `import_dependencies_to_open_pack_file` to copy specific
   files from vanilla into your mod pack.

7. **Open CA packs** – `load_all_ca_pack_files` opens all vanilla packs as one merged
   read-only pack for full browsing.

8. **Cross-source lookups** – `get_rfiles_from_all_sources` retrieves files by path
   from PackFile, GameFiles, and ParentFiles simultaneously.

Tips:
- Use `get_packed_files_names_starting_with_path_from_all_sources` to discover files
  under a given path prefix across all sources.
- `set_dependency_pack_files_list` lets you mark other mods as dependencies of your pack.
",
        )]
    }

    #[prompt(name = "run_diagnostics", description = "Validate a pack and fix common issues.")]
    pub async fn run_diagnostics(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user validate a Total War mod PackFile.

Workflow:

1. **Open the pack** and **set the game** with `rebuild_dependencies: true`.

2. **Generate dependencies** – If dependencies have not been generated yet,
   call `generate_dependencies` to build the dependency data needed for diagnostics.

3. **Run full diagnostics** – `diagnostics_check` with an empty `ignored` list
   and `check_ak_only_refs: false` (or `true` to include Assembly Kit references).
   The response contains all warnings and errors grouped by category.

4. **Review results** – Present the diagnostic results to the user, grouped by severity.
   Common issues include:
   - Invalid references (a column references a key that does not exist)
   - Duplicate keys
   - Empty loc entries
   - Outdated table versions

5. **Fix issues** – For each issue:
   - Decode the affected file with `decode_packed_file`.
   - Apply the fix (correct a reference, remove a duplicate row, etc.).
   - Save with `save_packed_file_from_view`.

6. **Ignore false positives** – Use `add_line_to_pack_ignored_diagnostics` to suppress
   specific diagnostic lines that are intentional.

7. **Re-check** – After fixes, call `diagnostics_check` again to confirm all issues
   are resolved.

8. **Optimize** – Optionally run `optimize_pack_file` to remove rows that are identical
   to vanilla, reducing pack size.
",
        )]
    }

    #[prompt(name = "schema_operations", description = "Work with table schemas: inspect, update, and patch definitions.")]
    pub async fn schema_operations(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user manage RPFM table schemas.

Workflow:

1. **Check schema status** – `is_schema_loaded` to verify a schema is loaded.
   If not, call `update_schemas` to download the latest from the repository.

2. **Get the full schema** – `get_schema` returns the entire schema object.

3. **Inspect a table definition** – `definitions_by_table_name` with a table name
   returns all known versions. Use `definition_by_table_name_and_version` for a
   specific version.

4. **See processed fields** – `fields_processed` takes a Definition JSON and returns
   fields with bitwise expansion and enum conversions applied (useful for display).

5. **Find referencing columns** – `referencing_columns_for_definition` shows which
   other tables reference a given table's columns.

6. **Patch a definition** – To customise column metadata (descriptions, references,
   default values) without modifying the upstream schema:
   a. Build a `HashMap<String, DefinitionPatch>` with your changes.
   b. Call `save_local_schema_patch` to persist it locally.
   c. Use `remove_local_schema_patches_for_table` or
      `remove_local_schema_patches_for_table_and_field` to undo patches.

7. **Import patches** – `import_schema_patch` applies a patch from another source.

8. **Update from Assembly Kit** – `update_current_schema_from_asskit` merges
   definition data from the game's Assembly Kit into the loaded schema.

9. **Save the schema** – `save_schema` writes the current in-memory schema to disk.
",
        )]
    }

    #[prompt(name = "file_operations", description = "Add, remove, rename, extract, and move files within packs.")]
    pub async fn file_operations(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user manage files inside a Total War PackFile.

Common operations:

**Add files from disk:**
- `add_packed_files` – Import files from the filesystem into the pack. Provide source
  filesystem paths and destination `ContainerPath` entries as JSON.

**Add files from another pack:**
- `add_packed_files_from_pack_file` – Copy files between two open packs.

**Create new files:**
- `new_packed_file` – Create a blank DB table, Loc file, or other file type inside the pack.

**Delete files:**
- `delete_packed_files` – Remove files by their `ContainerPath` list.

**Rename / move files:**
- `rename_packed_files` – Pass a list of `(old_path, new_path)` tuples.

**Copy / Cut / Paste / Duplicate:**
- `copy_packed_files` – Copy files to the internal clipboard for later pasting.
- `cut_packed_files` – Cut files to the internal clipboard (removed from source on paste).
- `paste_packed_files` – Paste clipboard contents into a pack at the given folder path.
- `duplicate_packed_files` – Clone files in-place with a numeric suffix.

**Extract to disk:**
- `extract_packed_files` – Export files from the pack to a folder on disk.
  Set `export_as_tsv: true` to export tables as TSV files.

**AnimPack operations:**
- `add_packed_files_from_pack_file_to_animpack` – Add files to an AnimPack.
- `add_packed_files_from_animpack` – Extract files from an AnimPack.
- `delete_from_animpack` – Remove files from an AnimPack.

**File info:**
- `get_packed_files_info` / `get_rfile_info` – Get metadata about files.
- `folder_exists` / `packed_file_exists` – Check if a path exists.
- `get_packed_file_raw_data` – Get the raw binary content of a file.

**Merge tables:**
- `merge_files` – Combine multiple compatible tables into one.

**External editing:**
- `open_packed_file_in_external_program` – Open a file in the system's default editor.
- `save_packed_file_from_external_view` – Re-import after external editing.

Always call `save_packfile` or `save_pack_as` when done to persist changes.
",
        )]
    }

    #[prompt(name = "troubleshooting", description = "Diagnose and fix common issues with RPFM and PackFiles.")]
    pub async fn troubleshooting(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user troubleshoot common RPFM and PackFile issues.

## Common Issues and Solutions

### 1. Schema not loaded
**Symptom**: Files fail to decode, or `decode_packed_file` returns raw data.
**Solution**:
- Call `is_schema_loaded()` – if false, call `update_schemas()`.
- Make sure `set_game_selected` was called with `rebuild_dependencies: true`.

### 2. Dependencies not available
**Symptom**: References show as invalid, diagnostics report missing keys.
**Solution**:
- Call `is_there_a_dependency_database(true)` – if false, call `generate_dependencies_cache()`.
- Ensure the game path is configured correctly in settings.

### 3. Pack won't save
**Symptom**: `save_packfile` returns an error.
**Solution**:
- Check if the file is read-only or locked by another process.
- Try `save_pack_as` to a different path.
- As a last resort, use `clean_and_save_pack_as` to recover from corruption.

### 4. Table version mismatch
**Symptom**: Table data looks wrong or has missing columns after a game update.
**Solution**:
- Call `update_schemas()` to get the latest table definitions.
- Use `update_table` to migrate the table to the current version.
- Check `get_table_definition_from_dependency_pack_file` for the expected schema.

### 5. Wrong game selected
**Symptom**: Tables decode with wrong columns or fail to decode, dependencies are for a different game.
**Solution**:
- Call `get_game_selected()` to verify the current game.
- Call `set_game_selected` with the correct game key and `rebuild_dependencies: true`.

### 6. Diagnostics show many reference errors
**Symptom**: `diagnostics_check` reports hundreds of invalid references.
**Solution**:
- Ensure dependencies are loaded (`is_there_a_dependency_database(true)`).
- Check if the pack depends on other mods via `get_dependency_pack_files_list`.
- Some references are Assembly Kit only; re-run with `check_ak_only_refs: true`.
- Use `add_line_to_pack_ignored_diagnostics` for intentional deviations.

### Diagnostic Tools
- `diagnostics_check` – Full pack validation.
- `get_game_selected` – Verify game context.
- `is_schema_loaded` – Check schema status.
- `is_there_a_dependency_database` – Check dependency database status.
- `list_open_packs` – Verify which packs are open.
- `config_path` / `schemas_path` – Verify RPFM paths.
",
        )]
    }

    #[prompt(name = "tsv_workflow", description = "Import and export tables as TSV files for batch editing in spreadsheets.")]
    pub async fn tsv_workflow(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user work with TSV (Tab-Separated Values) files for batch editing \
Total War mod data in spreadsheets.

## Export Workflow (Pack → TSV → Spreadsheet)

1. **Open the pack** and **set the game** with `rebuild_dependencies: true`.

2. **Export a single table as TSV**:
   Call `export_tsv` with:
   - `pack_key`: the pack key
   - `tsv_path`: destination path on disk (e.g. `/home/user/my_table.tsv`)
   - `table_path`: the internal path (e.g. `db/land_units_tables/my_mod`)

3. **Export all tables as TSV**:
   Call `extract_packed_files` with `export_as_tsv: true`.
   This exports all tables in the pack as TSV files to the destination folder.

4. **Edit in a spreadsheet**: Open the TSV file in LibreOffice Calc, Excel, or Google Sheets.
   - Keep the header rows intact (they contain schema metadata).
   - Tab-separated values — do not change the delimiter.

## Import Workflow (Spreadsheet → TSV → Pack)

1. **Save the spreadsheet as TSV** (tab-delimited, UTF-8 encoding).

2. **Import the TSV back**:
   Call `import_tsv` with:
   - `pack_key`: the target pack key
   - `tsv_path`: path to the TSV file on disk
   - `table_path`: the internal path where the table should go

3. **Verify**: Call `decode_packed_file` to confirm the data imported correctly.

4. **Save the pack**: Call `save_packfile` to persist changes.

## Tips
- TSV files include metadata headers that RPFM uses for schema matching.
  Do not delete or modify these header rows.
- Use `get_table_definition_from_dependency_pack_file` to understand column types
  before editing.
- After import, run `diagnostics_check` to validate references.
",
        )]
    }

    #[prompt(name = "translation_workflow", description = "Work with localisation and translation data in PackFiles.")]
    pub async fn translation_workflow(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "\
You are an assistant helping the user work with localisation (translation) data in Total War mods.

## Understanding Loc Files

Loc files contain key-value pairs for in-game text. Each entry has:
- A **key** (unique identifier referenced by DB tables)
- A **value** (the displayed text in the game)

## Viewing Existing Translations

1. **Open the pack** and **set the game**.

2. **Decode a loc file**:
   Call `decode_packed_file` with the loc file path (e.g. `text/db/my_mod.loc`)
   and `source: \"PackFile\"`.

3. **Get translation overview**:
   Call `get_pack_translation` with the pack key and a language code
   (e.g. `\"en\"`, `\"fr\"`, `\"de\"`, `\"es\"`, `\"it\"`, `\"zh\"`, `\"ru\"`, etc.).

## Creating New Translations

1. **Create a new loc file**:
   Call `new_packed_file` with path `\"text/db/my_mod.loc\"` and
   `new_file = {\"Loc\": \"my_mod\"}`.

2. **Decode it**: `decode_packed_file` to get the empty structure.

3. **Add entries**: Modify the decoded JSON to add key-value rows.
   Each row is typically `[\"key_string\", \"Displayed text in game\"]`.

4. **Save back**: `save_packed_file_from_view` with the modified data.

## Generating Missing Loc Data

Call `generate_missing_loc_data` with the pack key to auto-generate
loc entries for DB fields that reference loc keys but don't have entries yet.

## Finding Loc Keys

- Use `go_to_loc` with a loc key to find its source loc file.
- Use `get_source_data_from_loc_key` to find where a loc key is referenced.
- Use `global_search` with `search_on.loc: true` to search across all loc files.

## Tips
- Loc keys follow naming conventions like `<table>_<loc_column_name>_<keys_concatenated>`.
- Use `search_references` to find all DB columns that reference a specific loc key.
- After adding translations, run `diagnostics_check` to verify all references.
",
        )]
    }
}
