//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! [Model Context Protocol][mcp] server exposed at the `/mcp` endpoint.
//!
//! Wraps every [`Command`] the [`crate::background_thread`] dispatcher
//! understands as an MCP **tool**, plus a handful of MCP **resources**
//! (game lists, enum dumps, examples, reference docs) and **prompts** for
//! common workflows ("open and inspect a pack", "edit a DB table",
//! "manage dependencies", …). Each MCP client gets its own dedicated
//! [`Session`] and [`McpServer`] — same isolation guarantees as the
//! WebSocket clients.
//!
//! Each tool call:
//!
//! 1. Translates its `*Args` payload into a [`Command`] and ships it
//!    through the session's
//!    [`background_loop`](crate::background_thread::background_loop) via
//!    the `send_and_respond!` helper.
//! 2. Wraps the resulting [`Response`] back into a [`CallToolResult`].
//!
//! The `*Args` structs are the canonical schema for every tool. Their
//! `JsonSchema` derive is what `rmcp` ships to clients to advertise tool
//! arguments, so docstrings on individual fields show up directly in MCP
//! tool listings.
//!
//! [mcp]: https://modelcontextprotocol.io/
//! [`Session`]: crate::session::Session
//! [`CallToolResult`]: rmcp::model::CallToolResult

use rmcp::ErrorData as McpError;
use rmcp::handler::server::{common::schema_for_output, router::prompt::PromptRouter, tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    CallToolResult, CompletionInfo, CompleteRequestParams, CompleteResult,
    ContentBlock, ErrorCode, ListResourcesResult, ListResourceTemplatesResult,
    PaginatedRequestParams, PromptMessage,
    ReadResourceRequestParams, ReadResourceResult, ReadResourceResponse,
    Resource, ResourceContents, Role, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router, RoleServer};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::path::PathBuf;

use rpfm_extensions::merge::MergeOptions;
use rpfm_extensions::translator::DEFAULT_SRC_LANG;

use rpfm_ipc::api::{ApiError, Done, Request, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::files::{
    AddFilesFromDisk, CopyFiles, CreateFile, DeleteFiles, DuplicateFiles, ExtractFiles, FileEntry, FileList, FilesAdded, FilesDeleted,
    FilesExtracted, FilesRenamed, ListFiles, RenameFiles,
};
use rpfm_ipc::api::packs::{ClosePack, CloseAllPacks, GetPackInfo, NewPack, OpenPack, OpenVanillaPacks, PackDetails, PackSummary, SavePack, UpdatePack};
use rpfm_ipc::api::session::{GetSessionStatus, SessionStatus, SetGame};
use rpfm_ipc::api::tables::{EditTable, GetTableDefinition, GetTableInfo, GetTableRows, TableDefinition, TableEdited, TableInfo, TableRows};
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
        let tx = start_tool_transaction($tool_name);
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
            Ok(CallToolResult::error(vec![ContentBlock::text(json)]))
        } else {
            Ok(CallToolResult::success(vec![ContentBlock::text(json)]))
        }
    }};
}

/// Starts the Sentry transaction of a tool call, following the MCP tracing spec.
fn start_tool_transaction(tool_name: &str) -> sentry::Transaction {
    let tx_ctx = sentry::TransactionContext::new(&format!("tools/call {}", tool_name), "mcp.server");
    let tx = sentry::start_transaction(tx_ctx);
    tx.set_data("mcp.method.name", sentry::protocol::Value::from("tools/call"));
    tx.set_data("mcp.tool.name", sentry::protocol::Value::from(tool_name));
    tx.set_data("mcp.transport", sentry::protocol::Value::from("streamable-http"));

    sentry::configure_scope(|scope| scope.set_span(Some(tx.clone().into())));
    tx
}

/// Build a `Resource` with common fields set.
fn resource(uri: &str, name: &str, description: &str, mime_type: &str) -> Resource {
    Resource::new(uri, name)
        .with_description(description)
        .with_mime_type(mime_type)
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
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Invalid JSON parameter: {e}"))])),
        }
    };
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// MCP server bound to a single [`Session`].
///
/// One instance is constructed per MCP client connection by the
/// `StreamableHttpService` factory wired in `main.rs`. The
/// `tool_router` and `prompt_router` fields are built once at construction
/// time from the `#[tool_router]` / `#[prompt_router]` attribute macros
/// applied further down in this module.
///
/// Cheap to clone — only `Arc` and small router structs.
#[derive(Clone)]
pub struct McpServer {
    /// The session this MCP client is bound to.
    session: Arc<Session>,
    /// The router auto-generated from `#[tool_router]` annotations.
    tool_router: ToolRouter<Self>,
    /// The router auto-generated from `#[prompt_router]` annotations.
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

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct TableColumnArgs {
    /// Name of the DB table, like `factions_tables`.
    pub table_name: String,
    /// Name of the column.
    pub column_name: String,
}

// -- Pack Key Args (multi-pack support) --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyArg {
    /// The key of the target pack. Use `session_status` to get available keys.
    pub pack_key: String,
}


#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackKeyStringArg {
    /// The key of the target pack.
    pub pack_key: String,
    /// A string value.
    pub value: String,
}


// -- Pack Metadata Args --



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
pub struct AddPackedFilesFromPackFileToAnimpackArgs {
    /// The key of the source pack the files are copied from.
    pub source_pack_key: String,
    /// The key of the pack that owns the target AnimPack (may differ from the source).
    pub pack_key: String,
    /// The animpack path.
    pub animpack_path: String,
    /// The JSON representation of Vec<ContainerPath> for files to add.
    pub container_paths: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct AddPackedFilesFromAnimpackArgs {
    /// The key of the pack that owns the AnimPack (only used when `source` is a PackFile).
    pub anim_pack_key: String,
    /// The key of the destination pack the files are copied into (may differ from the AnimPack's).
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
pub struct SavePackedFileFromViewArgs {
    /// The key of the target pack.
    pub pack_key: String,
    /// The path of the file inside the pack.
    pub path: String,
    /// The JSON representation of the RFileDecoded enum.
    pub data: String,
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
pub struct StringsArg {
    /// A list of string values.
    pub values: Vec<String>,
}

// -- Dependency Args --


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
    /// Merge rows by key instead of concatenating them. If some rows can't be reconciled
    /// automatically, nothing is written and the response is `Response::MergeConflicts` instead.
    /// Defaults to false.
    #[serde(default)]
    pub delta_merge: bool,
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
pub struct LuaRunTestsArgs {
    /// Code of the Lua test file.
    pub test_source: String,
    /// Campaign whose vanilla scripts to load, like "main_warhammer". Omit it to load only the script libraries and the mods.
    pub campaign: Option<String>,
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
    /// The source language code these translations are based on (e.g. "EN").
    #[serde(default = "default_src_lang")]
    pub src_lang: String,
    /// The target language code.
    pub language: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct SrcLangArg {
    /// The source language code (e.g. "SP").
    pub src_lang: String,
}

fn default_src_lang() -> String {
    DEFAULT_SRC_LANG.to_owned()
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl rmcp::ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_prompts()
            .enable_resources()
            .enable_completions()
            .build();

        // `ServerInfo` is `#[non_exhaustive]` in rmcp, so it must be built through its constructor instead of a struct literal.
        ServerInfo::new(capabilities).with_instructions("\
This is the MCP server for RPFM (Rusted PackFile Manager), a tool for modding Total War games by \
Creative Assembly. It lets you read, edit, create, and manage PackFiles (.pack) — the archive \
format used by all modern Total War titles.

## Key Concepts

- **PackFile**: An archive containing game data files (DB tables, localisation, textures, models, etc.). \
  Mods are distributed as PackFiles.
- **pack_key**: When you open one or more PackFiles, each gets a unique key string. Use `session_status` \
  to discover available keys. Most tools require a `pack_key` parameter.
- **Reading data**: `list_files`, `table_info`, `table_rows` and `table_definition` return small, \
  paginated, structured results. Prefer them over `decode_packed_file` for DB and Loc tables.
- **Editing tables**: `edit_table` inserts, updates and deletes rows by index, with values by column \
  name. To change a vanilla table, copy it into your pack with `copy_files` first.
- **Paths**: tools taking plain path strings treat a path as a file if one exists there, or as a \
  folder otherwise.
- **DataSource**: Where data lives — `\"PackFile\"` (the user's mod), `\"GameFiles\"` (vanilla game data), \
  `\"ParentFiles\"` (dependency mods), `\"AssKitFiles\"` (Assembly Kit data), `\"ExternalFile\"` (disk file).
- **ContainerPath**: A path inside a pack — either `{\"File\": \"db/land_units_tables/my_table\"}` or \
  `{\"Folder\": \"db/land_units_tables\"}`. Use an empty string for root folder.

## Required Initialization Sequence

1. **Set the game** — Call `set_game` with the game key (e.g. `\"warhammer_3\"`). This loads the \
   schema and the vanilla data.
2. **Open a pack** — Call `open_pack` with filesystem path(s), or `new_pack`. Note the returned pack key.
3. **Verify schema** — Call `session_status`; if `schema_loaded` is false, call `update_schemas` first.

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
")
    }

    //-----------------------------------------------------------------------//
    // Resources
    //-----------------------------------------------------------------------//

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
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
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
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
                    "pack_apply_compression": true,
                    "pack_apply_encryption": false,
                    "pack_remove_duplicated_files": false,
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
                    "pack_apply_compression": "Apply the most modern compression format the active game supports (overriding the pack's configured one), so the next save compresses the files.",
                    "pack_apply_encryption": "Enable both index and data encryption, so the next save encrypts the pack. No-op on packs older than PFH4.",
                    "pack_remove_duplicated_files": "Remove case-insensitively duplicated files (same name ignoring casing) when their contents are identical, keeping the all-lowercase one or, failing that, the last one.",
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
    Call: set_game(game: \"warhammer_3\")
    This loads the correct schemas and vanilla game data for the selected title.
    Valid game keys: pharaoh_dynasties, pharaoh, warhammer_3, troy, three_kingdoms,
    warhammer_2, warhammer, thrones_of_britannia, attila, rome_2, shogun_2,
    napoleon, empire, arena.

Step 2: Verify schema is loaded
    Call: session_status()
    If schema_loaded is false, call update_schemas() to download the latest schemas.

Step 3: Open a PackFile
    Call: open_pack(paths: [\"/path/to/my_mod.pack\"])
    The response includes the pack key you'll use for all subsequent operations.

Step 4: Verify dependencies (optional but recommended)
    Call: session_status()
    If dependencies.vanilla_loaded is false, call generate_dependencies_cache() to build the dependency database.

After initialization, use session_status() to see all open pack keys at any time.
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
    After editing a script, run `diagnostics_check`: it reports Lua syntax errors, invalid DB keys, unknown methods,
    wrong argument counts and unknown events (all but syntax errors need the game's Assembly Kit installed).
    To check what a script does, write tests for it and run them with `lua_run_tests`.

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

        Ok(ReadResourceResult::new(vec![ResourceContents::text(content, uri.clone())]).into())
    }

    //-----------------------------------------------------------------------//
    // Completions
    //-----------------------------------------------------------------------//

    async fn complete(
        &self,
        request: CompleteRequestParams,
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

        Ok(CompleteResult::new(CompletionInfo::with_pagination(
            values,
            Some(total),
            has_more,
        ).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to build completion info: {e}").into(),
            data: None,
        })?))
    }

}

impl McpServer {

    /// Runs a request of the version 2 API on the session.
    ///
    /// # Returns
    ///
    /// The response as structured content, or the error as a tool error, so a failed request doesn't end the MCP session.
    async fn call_api<R: Request>(&self, tool_name: &str, request: R) -> Result<CallToolResult, McpError> {
        let tx = start_tool_transaction(tool_name);

        let rpc_request = RpcRequest::new(0, &request).map_err(|error| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to serialize request: {error}").into(),
            data: None,
        })?;

        let response = self.session.call(rpc_request).recv().await
            .unwrap_or_else(|| RpcResponse::new(0, Err(ApiError::Internal("Session response channel closed unexpectedly".to_owned()))));

        tx.finish();

        Ok(match response.outcome {
            RpcOutcome::Result(value) => CallToolResult::structured(value),
            RpcOutcome::Error(error) => CallToolResult::error(vec![ContentBlock::text(serde_json::to_string(&error).unwrap_or(error.message))]),
        })
    }
}

#[tool_router]
impl McpServer {

    #[tool(
        name = "session_status",
        description = "Get the state of the session: the selected game, if its schema and dependencies (vanilla files, Assembly Kit tables, parent packs) are loaded, and the open packs with their keys. Call this first to know what's available.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<SessionStatus>(),
    )]
    pub async fn session_status(&self) -> Result<CallToolResult, McpError> {
        self.call_api("session_status", GetSessionStatus {}).await
    }

    #[tool(
        name = "pack_info",
        description = "Get the details of an open pack: type, format version, compression, encryption flags, the packs it depends on, and its MyMod mode.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<PackDetails>(),
    )]
    pub async fn pack_info(&self, params: Parameters<GetPackInfo>) -> Result<CallToolResult, McpError> {
        self.call_api("pack_info", params.0).await
    }

    #[tool(
        name = "list_files",
        description = "List the files of an open pack, the game files, the parent packs, or the Assembly Kit tables, sorted by path. Filter by path prefix and file type, and page through big sources with offset/limit (the total is always returned). With recursive=false, the subfolders of the prefix are listed instead of their files, to browse folder by folder.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<FileList>(),
    )]
    pub async fn list_files(&self, params: Parameters<ListFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("list_files", params.0).await
    }

    #[tool(
        name = "table_info",
        description = "Get the columns (name, type, key, referenced table and column, default value, description) and the row count of a DB or Loc table, from any source. Columns are listed in the order row values are returned and written.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<TableInfo>(),
    )]
    pub async fn table_info(&self, params: Parameters<GetTableInfo>) -> Result<CallToolResult, McpError> {
        self.call_api("table_info", params.0).await
    }

    #[tool(
        name = "table_rows",
        description = "Read rows of a DB or Loc table, from any source, as plain values (booleans, numbers and strings). Pick only the columns you need, filter rows by column values, and page with offset/limit (100 rows by default; the total of matching rows is always returned). Each row includes its index in the table.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<TableRows>(),
    )]
    pub async fn table_rows(&self, params: Parameters<GetTableRows>) -> Result<CallToolResult, McpError> {
        self.call_api("table_rows", params.0).await
    }

    #[tool(
        name = "table_definition",
        description = "Get the columns of a table as defined in the schema, without needing a file of it. Without a version, the version of the table in the game files is used.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<TableDefinition>(),
    )]
    pub async fn table_definition(&self, params: Parameters<GetTableDefinition>) -> Result<CallToolResult, McpError> {
        self.call_api("table_definition", params.0).await
    }

    #[tool(
        name = "set_game",
        description = "Select the game to work with, like `warhammer_3`, loading its schema and, by default, its dependencies (vanilla files, Assembly Kit tables, parent packs). Loading the dependencies can take a while. Call this before opening packs.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<SessionStatus>(),
    )]
    pub async fn set_game(&self, params: Parameters<SetGame>) -> Result<CallToolResult, McpError> {
        self.call_api("set_game", params.0).await
    }

    #[tool(
        name = "new_pack",
        description = "Create a new empty pack. It has no path on disk until you save it with `save_pack` and a `path`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<PackSummary>(),
    )]
    pub async fn new_pack(&self) -> Result<CallToolResult, McpError> {
        self.call_api("new_pack", NewPack {}).await
    }

    #[tool(
        name = "open_pack",
        description = "Open one or more packs from disk, merged into a single one. Returns its key, used by every tool that works on it.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<PackSummary>(),
    )]
    pub async fn open_pack(&self, params: Parameters<OpenPack>) -> Result<CallToolResult, McpError> {
        self.call_api("open_pack", params.0).await
    }

    #[tool(
        name = "open_vanilla_packs",
        description = "Open all the vanilla packs of the selected game, merged into a single one, to browse them like any open pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<PackSummary>(),
    )]
    pub async fn open_vanilla_packs(&self) -> Result<CallToolResult, McpError> {
        self.call_api("open_vanilla_packs", OpenVanillaPacks {}).await
    }

    #[tool(
        name = "close_pack",
        description = "Close an open pack. Unsaved changes are lost.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn close_pack(&self, params: Parameters<ClosePack>) -> Result<CallToolResult, McpError> {
        self.call_api("close_pack", params.0).await
    }

    #[tool(
        name = "close_all_packs",
        description = "Close all open packs. Unsaved changes are lost.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn close_all_packs(&self) -> Result<CallToolResult, McpError> {
        self.call_api("close_all_packs", CloseAllPacks {}).await
    }

    #[tool(
        name = "save_pack",
        description = "Save an open pack to disk, to its current path or to a new `path`. New packs need a path. Set `clean` to drop files that failed to decode if saving normally fails.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<PackSummary>(),
    )]
    pub async fn save_pack(&self, params: Parameters<SavePack>) -> Result<CallToolResult, McpError> {
        self.call_api("save_pack", params.0).await
    }

    #[tool(
        name = "update_pack",
        description = "Change properties of an open pack: type, compression, encryption and timestamp flags, the packs it depends on, and its MyMod mode. Only the fields you set are changed.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<PackDetails>(),
    )]
    pub async fn update_pack(&self, params: Parameters<UpdatePack>) -> Result<CallToolResult, McpError> {
        self.call_api("update_pack", params.0).await
    }

    #[tool(
        name = "create_file",
        description = "Create a new empty file in an open pack: a DB table (with the version of the game files by default), a Loc table, a text file or an AnimPack. Fill tables afterwards with `edit_table`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FileEntry>(),
    )]
    pub async fn create_file(&self, params: Parameters<CreateFile>) -> Result<CallToolResult, McpError> {
        self.call_api("create_file", params.0).await
    }

    #[tool(
        name = "add_files_from_disk",
        description = "Add files and folders from disk to an open pack, under a folder of the pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesAdded>(),
    )]
    pub async fn add_files_from_disk(&self, params: Parameters<AddFilesFromDisk>) -> Result<CallToolResult, McpError> {
        self.call_api("add_files_from_disk", params.0).await
    }

    #[tool(
        name = "copy_files",
        description = "Copy files and folders from an open pack, the game files, the parent packs or the Assembly Kit tables into an open pack, keeping their paths. Use it to start editing a vanilla table in your mod. Paths that match nothing are returned in `not_added`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesAdded>(),
    )]
    pub async fn copy_files(&self, params: Parameters<CopyFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("copy_files", params.0).await
    }

    #[tool(
        name = "delete_files",
        description = "Delete files and folders from an open pack.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<FilesDeleted>(),
    )]
    pub async fn delete_files(&self, params: Parameters<DeleteFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("delete_files", params.0).await
    }

    #[tool(
        name = "rename_files",
        description = "Rename or move files and folders of an open pack.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<FilesRenamed>(),
    )]
    pub async fn rename_files(&self, params: Parameters<RenameFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("rename_files", params.0).await
    }

    #[tool(
        name = "duplicate_files",
        description = "Copy files of an open pack in the same pack, adding a number to their names.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesAdded>(),
    )]
    pub async fn duplicate_files(&self, params: Parameters<DuplicateFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("duplicate_files", params.0).await
    }

    #[tool(
        name = "extract_files",
        description = "Extract files and folders of an open pack, the game files or the parent packs to a folder on disk, optionally with tables as TSV. Existing files on disk are overwritten.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<FilesExtracted>(),
    )]
    pub async fn extract_files(&self, params: Parameters<ExtractFiles>) -> Result<CallToolResult, McpError> {
        self.call_api("extract_files", params.0).await
    }

    #[tool(
        name = "edit_table",
        description = "Edit rows of a DB or Loc table in an open pack: insert, update and delete rows by index, with values given by column name (see `table_info` for the columns). Edits apply in order, each on the result of the previous ones; if any fails, none is applied. To edit a vanilla table, copy it into your pack first with `copy_files`.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<TableEdited>(),
    )]
    pub async fn edit_table(&self, params: Parameters<EditTable>) -> Result<CallToolResult, McpError> {
        self.call_api("edit_table", params.0).await
    }

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

    //-----------------------------------------------------------------------//
    // Pack Metadata
    //-----------------------------------------------------------------------//

    #[tool(description = "Get the settings of the pack identified by `pack_key`.")]
    pub async fn get_pack_settings(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_pack_settings", Command::GetPackSettings(params.0.pack_key))
    }

    #[tool(description = "Set the settings of the pack identified by `pack_key`. The `settings` is a PackSettings JSON object containing pack-level configuration.")]
    pub async fn set_pack_settings(&self, params: Parameters<SetPackSettingsArgs>) -> Result<CallToolResult, McpError> {
        let settings = parse_json!(&params.0.settings);
        send_and_respond!(self, "set_pack_settings", Command::SetPackSettings(params.0.pack_key, settings))
    }

    //-----------------------------------------------------------------------//
    // File Operations
    //-----------------------------------------------------------------------//

    #[tool(description = "Decode a file from the pack identified by `pack_key`. The `path` is the internal file path (e.g. \"db/land_units_tables/my_mod\"). The `source` is the data source: \"PackFile\" (user mod), \"GameFiles\" (vanilla), \"ParentFiles\" (dependency mods), \"AssKitFiles\", or \"ExternalFile\". Returns the whole decoded file as JSON (RFileDecoded). For DB and Loc tables, prefer `table_info` and `table_rows`, which return only the columns and rows you ask for.")]
    pub async fn decode_packed_file(&self, params: Parameters<DecodePackedFileArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "decode_packed_file", Command::DecodePackedFile(params.0.pack_key, params.0.path, params.0.source))
    }

    #[tool(description = "Create a new file inside the pack identified by `pack_key`. The `path` is the destination path (e.g. \"db/land_units_tables/my_mod\"). NewFile types: {\"DB\": [\"file_name\", \"table_name\", version]}, {\"Loc\": \"name\"}, {\"Text\": [\"name\", \"Plain\"]}, {\"AnimPack\": \"name\"}, {\"VMD\": \"name\"}, {\"WSModel\": \"name\"}, {\"PortraitSettings\": [\"name\", version, []]}.")]
    pub async fn new_packed_file(&self, params: Parameters<NewPackedFileArgs>) -> Result<CallToolResult, McpError> {
        let new_file = parse_json!(&params.0.new_file);
        send_and_respond!(self, "new_packed_file", Command::NewPackedFile(params.0.pack_key, params.0.path, new_file))
    }

    #[tool(description = "Copy files from the pack identified by `source_pack_key` into an AnimPack owned by `pack_key` (the two may differ). The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}]. The `animpack_path` is the AnimPack's internal path.")]
    pub async fn add_packed_files_from_pack_file_to_animpack(&self, params: Parameters<AddPackedFilesFromPackFileToAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "add_packed_files_from_pack_file_to_animpack", Command::AddPackedFilesFromPackFileToAnimpack(params.0.source_pack_key, params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Copy files from an AnimPack owned by `anim_pack_key` into the destination pack `pack_key` (the two may differ). The `source` is the DataSource (\"PackFile\", \"GameFiles\", etc.); `anim_pack_key` is only used when it is \"PackFile\". The `animpack_path` is the AnimPack's internal path. The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}].")]
    pub async fn add_packed_files_from_animpack(&self, params: Parameters<AddPackedFilesFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "add_packed_files_from_animpack", Command::AddPackedFilesFromAnimpack(params.0.anim_pack_key, params.0.pack_key, params.0.source, params.0.animpack_path, paths))
    }

    #[tool(description = "Delete files from an AnimPack in the pack identified by `pack_key`. The `animpack_path` is the AnimPack's internal path. The `container_paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"animations/anim.anim\"}].")]
    pub async fn delete_from_animpack(&self, params: Parameters<DeleteFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.container_paths);
        send_and_respond!(self, "delete_from_animpack", Command::DeleteFromAnimpack(params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Save an edited decoded file back to the pack identified by `pack_key`. The `path` is the internal path (e.g. \"db/land_units_tables/my_mod\"). The `data` is the modified RFileDecoded JSON (same structure returned by `decode_packed_file`).")]
    pub async fn save_packed_file_from_view(&self, params: Parameters<SavePackedFileFromViewArgs>) -> Result<CallToolResult, McpError> {
        let data: RFileDecoded = parse_json!(&params.0.data);
        send_and_respond!(self, "save_packed_file_from_view", Command::SavePackedFileFromView(params.0.pack_key, params.0.path, data))
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

    #[tool(description = "Clean the decode cache for the provided paths in the pack identified by `pack_key`. The `paths` is a JSON array of ContainerPath, e.g. [{\"File\": \"db/land_units_tables/my_mod\"}, {\"Folder\": \"db\"}].")]
    pub async fn clean_cache(&self, params: Parameters<ContainerPathsArg>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        send_and_respond!(self, "clean_cache", Command::CleanCache(params.0.pack_key, paths))
    }

    //-----------------------------------------------------------------------//
    // Game Selection
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Dependencies
    //-----------------------------------------------------------------------//

    #[tool(description = "Generate the dependencies cache for the selected game. This can take a long time (more than 30 seconds), depending on your CPU and disk read speed. If the client is not careful, it can take enough time that the client may trigger a timeout.")]
    pub async fn generate_dependencies_cache(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "generate_dependencies_cache", Command::GenerateDependenciesCache)
    }

    #[tool(description = "Rebuild dependencies. Pass true for full rebuild, false for mod-specific only.")]
    pub async fn rebuild_dependencies(&self, params: Parameters<BoolArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "rebuild_dependencies", Command::RebuildDependencies(params.0.value))
    }

    #[tool(description = "Get custom table names (start_pos_, twad_ prefixes) from the schema.")]
    pub async fn get_custom_table_list(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_custom_table_list", Command::GetCustomTableList)
    }

    #[tool(description = "Get local art set IDs from campaign_character_arts_tables in the pack identified by `pack_key`.")]
    pub async fn local_art_set_ids(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "local_art_set_ids", Command::LocalArtSetIds(params.0.pack_key))
    }

    #[tool(description = "Get art set IDs from dependencies' campaign_character_arts_tables.")]
    pub async fn dependencies_art_set_ids(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "dependencies_art_set_ids", Command::DependenciesArtSetIds)
    }

    #[tool(description = "Get the distinct values of the column `column_name` of the DB table `table_name` (like `factions_tables`), from the open packs, their parent packs and vanilla.")]
    pub async fn dependencies_column_values(&self, params: Parameters<TableColumnArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "dependencies_column_values", Command::DependenciesColumnValues(params.0.table_name, params.0.column_name))
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

    #[tool(description = "Get valid reference values for columns in a table definition for the pack identified by `pack_key`. The `definition` is a Definition JSON (as returned by `definition_by_table_name_and_version`). Set `force` to true to regenerate cached reference data.")]
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

    #[tool(description = "Get the current schema.")]
    pub async fn get_schema(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_schema", Command::Schema)
    }

    #[tool(description = "Get all definitions for a table name. NOTE: the returned `fields` list is the raw on-disk field layout, not what row data looks like (e.g. colour columns are split into separate r/g/b fields here). Use `table_definition` to get the columns rows actually have.")]
    pub async fn definitions_by_table_name(&self, params: Parameters<StringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "definitions_by_table_name", Command::DefinitionsByTableName(params.0.value))
    }

    #[tool(description = "Get a specific definition by table name and version. NOTE: the returned `fields` list is the raw on-disk field layout, not what row data looks like (e.g. colour columns are split into separate r/g/b fields here). Do not use `fields.len()` to size a row for saving — use `table_definition` to get the columns and types rows actually have.")]
    pub async fn definition_by_table_name_and_version(&self, params: Parameters<StringI32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "definition_by_table_name_and_version", Command::DefinitionByTableNameAndVersion(params.0.name, params.0.version))
    }

    #[tool(description = "Delete a definition by table name and version.")]
    pub async fn delete_definition(&self, params: Parameters<StringI32Args>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "delete_definition", Command::DeleteDefinition(params.0.name, params.0.version))
    }

    #[tool(description = "Get columns from other tables that reference the given table's definition. The `definition` is a Definition JSON (as returned by `definition_by_table_name_and_version` or `definitions_by_table_name`).")]
    pub async fn referencing_columns_for_definition(&self, params: Parameters<ReferencingColumnsForDefinitionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json!(&params.0.definition);
        send_and_respond!(self, "referencing_columns_for_definition", Command::ReferencingColumnsForDefinition(params.0.table_name, def))
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

    #[tool(description = "Merge multiple compatible tables into one in the pack identified by `pack_key`. The `paths` is a JSON array of ContainerPath for the tables to merge, e.g. [{\"File\": \"db/land_units_tables/table1\"}, {\"File\": \"db/land_units_tables/table2\"}]. The `merged_path` is the destination path. Set `delete_source` to true to remove the original files. DB Tables can also be enabled for Delta Merging (if two or more tables edit the same row, it merges their changes into a single row).")]
    pub async fn merge_files(&self, params: Parameters<MergeFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json!(&params.0.paths);
        let mut options = MergeOptions::default();
        options.set_delta_merge(params.0.delta_merge);
        send_and_respond!(self, "merge_files", Command::MergeFiles(params.0.pack_key, paths, params.0.merged_path, params.0.delete_source, options))
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

    #[tool(description = "Run Lua tests against the scripts of all open packs, outside of the game. Scripts run in Lua 5.1 with the game's real script libraries; the game's engine is emulated with objects typed after the Assembly Kit's scripting docs, which record every call made on them. Needs the game's Assembly Kit.

Test file API (a global `rpfm` table):
- Top-level code runs before the game boots, to set up the world: `local kislev = rpfm.faction { key = \"wh3_main_ksl_kislev\", is_human = true }`. Fields other than `key` are the values returned by the methods with the same name; lists like `region_list` can be plain arrays. Also `rpfm.region { key = ... }`, `rpfm.character { faction = kislev, ... }`, and `rpfm.object(\"TYPE_SCRIPT_INTERFACE\", methods)`. Factions and regions from the game's DB exist even if not set up.
- `rpfm.test(name, function)` registers a test. Each test runs in a fresh Lua state, after the libraries, the pack's mods (script/campaign/mod/) and the first tick have run.
- `rpfm.fire(event, { accessor = value, ... })` triggers an event, like `rpfm.fire(\"FactionTurnStart\", { faction = kislev })`.
- `rpfm.advance_time(seconds)` advances game time, triggering due time triggers, like the ones from `cm:callback`. `rpfm.end_turn()` plays a full round: `WorldStartRound`, `FactionRoundStart` for every faction, then for each faction in creation order its `FactionTurnStart`, the turn events of the regions and characters in its `region_list` and `character_list`, `FactionBeginTurnPhaseNormal`, `FactionAboutToEndTurn` and `FactionTurnEnd`.
- `rpfm.mock(object, method, value)` changes what a method of an engine object returns after boot, like `rpfm.mock(region, \"owning_faction\", kislev)`; `value` can be a function receiving the call's arguments. Scripts' own globals (like a mod's manager table) can be inspected and changed directly from tests.
- `rpfm.assert_called(method, args...)`, `rpfm.assert_not_called(method)`, `rpfm.calls_to(method)` and `rpfm.assert_equal(actual, expected)` check what the scripts did. Calls on `cm` are recorded by their method name, like `treasury_mod`.

The report lists each test with its errors (including errors of the pack's scripts, and in listeners), the undocumented methods it called (whose results are placeholders), and the scripts' output.")]
    pub async fn lua_run_tests(&self, params: Parameters<LuaRunTestsArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "lua_run_tests", Command::LuaRunTests(params.0.test_source, params.0.campaign))
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
        send_and_respond!(self, "get_pack_translation", Command::GetPackTranslation(params.0.pack_key, params.0.src_lang, params.0.language))
    }

    #[tool(description = "Generate the vanilla texts of a source language from the game's locale packs. Returns whether vanilla texts for that language are available.")]
    pub async fn generate_vanilla_translation_source(&self, params: Parameters<SrcLangArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "generate_vanilla_translation_source", Command::GenerateVanillaTranslationSource(params.0.src_lang))
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

    //-----------------------------------------------------------------------//
    // Additional tools
    //-----------------------------------------------------------------------//

}

//-------------------------------------------------------------------------------//
//                              MCP Prompts
//-------------------------------------------------------------------------------//

#[prompt_router]
impl McpServer {

    #[prompt(name = "open_and_inspect_pack", description = "Walk through opening a PackFile and inspecting its contents.")]
    pub async fn open_and_inspect_pack(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            "\
You are an assistant helping the user inspect a Total War PackFile using the RPFM MCP server.

Follow these steps in order:

1. **Select the game** – Call `set_game` with the correct game key (e.g. `\"warhammer_3\"`), so the
   schema and the vanilla data are loaded.

2. **Open the pack** – Call `open_pack` with the filesystem path(s) the user provides.
   Remember the returned `key`; the other tools take it as the pack key.

3. **List pack contents** – Call `list_files` with `source: {\"pack\": <pack key>}`. For big packs,
   browse folder by folder with `recursive: false` and a `prefix`, or filter by `file_types`.
   Present the files to the user in a readable format.

4. **Read tables** – For DB and Loc tables, call `table_info` to see their columns and row count,
   and `table_rows` to read rows, picking only the columns and rows you need with `columns` and
   `filters`. For other files, call `decode_packed_file` with the pack key, the internal path,
   and `source: \"PackFile\"`.

5. **Inspect metadata** – Use `pack_info` and `get_pack_settings` to answer questions about the pack itself.

Important notes:
- Always call `session_status` if you are unsure which pack key to use.
- If a table fails to decode, check `schema_loaded` in `session_status`; if false, call `update_schemas` first.
- When done, optionally call `close_pack` to free resources.
",
        )]
    }

    #[prompt(name = "edit_db_table", description = "Guide for reading, modifying, and saving a DB table inside a pack.")]
    pub async fn edit_db_table(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            "\
You are an assistant helping the user edit a DB table inside a Total War PackFile.

Workflow:

1. **Set the game** – `set_game` with the game key.
2. **Open the pack** – `open_pack` → note the pack `key`.
3. **Find the table** – `list_files` with `source: {\"pack\": <pack key>}`, `prefix: \"db/\"` and
   `file_types: [\"DB\"]`. To edit a vanilla table, copy it into the pack first:
   `copy_files` with `from: \"game_files\"`, its path, and `to_pack`.
4. **Inspect it** – `table_info` with `file: {\"source\": {\"pack\": <pack key>}, \"path\": <DB path>}`
   returns its columns (name, type, key, referenced table) and row count.
5. **Read the rows you need** – `table_rows` with `columns` and `filters`, to find the index of each
   row to change.
6. **Edit** – `edit_table` with a list of edits:
   - `{\"op\": \"update\", \"index\": 3, \"values\": {\"column\": value}}`
   - `{\"op\": \"insert\", \"values\": {\"key\": \"my_key\", ...}}` (missing columns get their default)
   - `{\"op\": \"delete\", \"indexes\": [5, 7]}`
   Edits apply in order, each on the result of the previous ones, and if any fails, none is applied.
7. **Save the pack** – `save_pack` (with a `path` to save it somewhere else).

Tips:
- Use `get_reference_data_from_definition` to discover valid values for referenced columns,
  or `table_rows` on the referenced table in the game files.
- Mods usually only keep the rows they change: `optimize_pack_file` removes rows identical to vanilla.
- After saving, you can run `diagnostics_check` to validate the pack.
",
        )]
    }

    #[prompt(name = "create_new_mod", description = "Step-by-step guide for creating a new mod PackFile from scratch.")]
    pub async fn create_new_mod(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            "\
You are an assistant helping the user create a new Total War mod from scratch.

Workflow:

1. **Set the game** – `set_game` with the target game key.

2. **Create the pack** – `new_pack` returns a new empty pack and its key.

3. **Add DB tables** – For each table you need, either:
   a. copy the vanilla table with `copy_files` (`from: \"game_files\"`) and edit it, or
   b. create an empty one with `create_file` (`kind: {\"type\": \"db\", \"table_name\": \"land_units_tables\"}`
      and a path like `\"db/land_units_tables/my_mod\"`), then add rows with `edit_table`.

4. **Add Loc files** – `create_file` with a path like `\"text/db/my_mod.loc\"` and
   `kind: {\"type\": \"loc\"}`, then add `key`/`text` rows with `edit_table`.

5. **Add other files** – `add_files_from_disk` to import assets from disk (images, models, scripts, etc.).

6. **Save the pack** – `save_pack` with a `path` to write the `.pack` file to disk.

Optional steps:
- `update_pack` to change the pack's type or the packs it depends on.
- `initialize_my_mod_folder` to set up a mod development folder with IDE support.
- `optimize_pack_file` to strip unchanged rows that match vanilla data.
- `diagnostics_check` to validate everything before release.
",
        )]
    }

    #[prompt(name = "search_and_replace", description = "Find and replace values across all files in a pack.")]
    pub async fn search_and_replace(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
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

6. **Save** – `save_pack` to persist changes.

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
            Role::User,
            "\
You are an assistant helping the user work with dependency data (vanilla game files).

Workflow:

1. **Set the game** – `set_game` with the game key.

2. **Check dependency database** – `session_status` shows if the vanilla files and the
   Assembly Kit tables are loaded. If `dependencies.vanilla_loaded` is false, call
   `generate_dependencies_cache` first.

3. **Browse vanilla files** – `list_files` with `source: \"game_files\"` (or `\"parent_files\"`,
   `\"assembly_kit\"`). Use `prefix: \"db/\"` and `file_types: [\"DB\"]` to list the vanilla tables.

4. **Read vanilla data** – `table_rows` with `file: {\"source\": \"game_files\", \"path\": <path>}`,
   filtering and picking columns to get only the rows you need.

5. **Get definitions** – `table_definition` with a table name, or `table_info` on a vanilla file.

6. **Import from vanilla** – `copy_files` with `from: \"game_files\"` (or `\"assembly_kit\"`) to copy
   specific files from vanilla into your mod pack.

7. **Open CA packs** – `open_vanilla_packs` opens all vanilla packs as one merged pack for full browsing.

Tips:
- `update_pack` with `dependencies` lets you mark other mods as dependencies of your pack,
  and `pack_info` shows the current ones.
- Columns returned by `table_info` and `table_definition` match the values of each row,
  in the same order.
",
        )]
    }

    #[prompt(name = "run_diagnostics", description = "Validate a pack and fix common issues.")]
    pub async fn run_diagnostics(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
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

5. **Fix issues** – For each issue in a table, find the affected rows with `table_rows` and fix them
   with `edit_table` (correct a reference, remove a duplicate row, etc.).

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
            Role::User,
            "\
You are an assistant helping the user manage RPFM table schemas.

Workflow:

1. **Check schema status** – `session_status` to verify a schema is loaded (`schema_loaded`).
   If not, call `update_schemas` to download the latest from the repository.

2. **Get the full schema** – `get_schema` returns the entire schema object.

3. **Inspect a table definition** – `definitions_by_table_name` with a table name
   returns all known versions. Use `definition_by_table_name_and_version` for a
   specific version.

4. **See the columns rows have** – `table_definition` returns the columns of a table
   as rows see them, with bitwise expansion, enum conversions, and colour-group merging
   applied. `definitions_by_table_name` and `definition_by_table_name_and_version` return
   the raw on-disk field list instead (e.g. a colour column split into separate r/g/b
   fields), which has a different length/order than actual row data. Use those only to
   edit the schema itself.

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
            Role::User,
            "\
You are an assistant helping the user manage files inside Total War PackFiles.

Paths are plain strings: a path is treated as a file if one exists there, or as a folder otherwise.

**Find files:**
- `list_files` – List files by path prefix and type, to find files or check if a path exists.

**Create files:**
- `create_file` – Create an empty DB table, Loc table, text file or AnimPack.
- `new_packed_file` – Create other file types, like portrait settings.

**Add files:**
- `add_files_from_disk` – Import files and folders from disk into a folder of the pack.
- `copy_files` – Copy files from another open pack, the game files, the parent packs or the
  Assembly Kit, keeping their paths.

**Delete, rename, duplicate:**
- `delete_files` – Remove files and folders.
- `rename_files` – Rename or move files and folders, with a list of `{from, to}`.
- `duplicate_files` – Clone files in the same pack with a numeric suffix.

**Extract to disk:**
- `extract_files` – Export files from a pack, the game files or the parent packs to a folder on disk.
  Set `as_tsv: true` to export tables as TSV files.

**AnimPack operations:**
- `add_packed_files_from_pack_file_to_animpack` – Add files to an AnimPack.
- `add_packed_files_from_animpack` – Extract files from an AnimPack.
- `delete_from_animpack` – Remove files from an AnimPack.

**Other:**
- `get_packed_file_raw_data` – Get the raw binary content of a file.
- `merge_files` – Combine multiple compatible tables into one.

Always call `save_pack` when done to persist changes.
",
        )]
    }

    #[prompt(name = "troubleshooting", description = "Diagnose and fix common issues with RPFM and PackFiles.")]
    pub async fn troubleshooting(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            "\
You are an assistant helping the user troubleshoot common RPFM and PackFile issues.

## Common Issues and Solutions

### 1. Schema not loaded
**Symptom**: Files fail to decode, or `decode_packed_file` returns raw data.
**Solution**:
- Call `session_status()` – if `schema_loaded` is false, call `update_schemas()`.
- Make sure `set_game` was called for the right game.

### 2. Dependencies not available
**Symptom**: References show as invalid, diagnostics report missing keys.
**Solution**:
- Call `session_status()` – if `dependencies.vanilla_loaded` is false, call `generate_dependencies_cache()`.
- Ensure the game path is configured correctly in settings.

### 3. Pack won't save
**Symptom**: `save_pack` returns an error.
**Solution**:
- Check if the file is read-only or locked by another process.
- Try `save_pack` with a different `path`.
- As a last resort, use `save_pack` with `clean: true` to drop the files that fail to decode.

### 4. Table version mismatch
**Symptom**: Table data looks wrong or has missing columns after a game update.
**Solution**:
- Call `update_schemas()` to get the latest table definitions.
- Use `update_table` to migrate the table to the current version.
- Check `table_definition` for the expected columns.

### 5. Wrong game selected
**Symptom**: Tables decode with wrong columns or fail to decode, dependencies are for a different game.
**Solution**:
- Call `session_status()` to verify the current game.
- Call `set_game` with the correct game key.

### 6. Diagnostics show many reference errors
**Symptom**: `diagnostics_check` reports hundreds of invalid references.
**Solution**:
- Ensure dependencies are loaded (`dependencies` in `session_status`).
- Check if the pack depends on other mods via `pack_info`.
- Some references are Assembly Kit only; re-run with `check_ak_only_refs: true`.
- Use `add_line_to_pack_ignored_diagnostics` for intentional deviations.

### Diagnostic Tools
- `diagnostics_check` – Full pack validation.
- `session_status` – Verify the game, schema, dependencies and open packs.
- `config_path` / `schemas_path` – Verify RPFM paths.
",
        )]
    }

    #[prompt(name = "tsv_workflow", description = "Import and export tables as TSV files for batch editing in spreadsheets.")]
    pub async fn tsv_workflow(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
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
   Call `extract_files` with `paths: [\"db\", \"text\"]` and `as_tsv: true`.
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

3. **Verify**: Call `table_rows` to confirm the data imported correctly.

4. **Save the pack**: Call `save_pack` to persist changes.

## Tips
- TSV files include metadata headers that RPFM uses for schema matching.
  Do not delete or modify these header rows.
- Use `table_info` or `table_definition` to understand column types before editing.
- After import, run `diagnostics_check` to validate references.
",
        )]
    }

    #[prompt(name = "translation_workflow", description = "Work with localisation and translation data in PackFiles.")]
    pub async fn translation_workflow(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            "\
You are an assistant helping the user work with localisation (translation) data in Total War mods.

## Understanding Loc Files

Loc files contain key-value pairs for in-game text. Each entry has:
- A **key** (unique identifier referenced by DB tables)
- A **value** (the displayed text in the game)

## Viewing Existing Translations

1. **Open the pack** and **set the game**.

2. **Read a loc file**:
   Call `table_rows` with `file: {\"source\": {\"pack\": <pack key>}, \"path\": \"text/db/my_mod.loc\"}`,
   filtering by `key` or `text` to find specific entries.

3. **Get translation overview**:
   Call `get_pack_translation` with the pack key and a language code
   (e.g. `\"en\"`, `\"fr\"`, `\"de\"`, `\"es\"`, `\"it\"`, `\"zh\"`, `\"ru\"`, etc.).

## Creating New Translations

1. **Create a new loc file**:
   Call `create_file` with path `\"text/db/my_mod.loc\"` and `kind: {\"type\": \"loc\"}`.

2. **Add entries**: `edit_table` with inserts like
   `{\"op\": \"insert\", \"values\": {\"key\": \"key_string\", \"text\": \"Displayed text in game\"}}`.

3. **Save the pack**: `save_pack`.

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
