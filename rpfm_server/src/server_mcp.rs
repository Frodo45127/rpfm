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

use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;


use rpfm_ipc::api::{ApiError, Done, Request, RpcError, RpcOutcome, RpcRequest, RpcResponse};
use rpfm_ipc::api::diagnostics::{DiagnosticList, IgnoreDiagnostics, ListDiagnostics, RunDiagnostics};
use rpfm_ipc::api::search::{ListSearchMatches, ReplaceSearchMatches, RunSearch, SearchMatchList, SearchReplaced};
use rpfm_ipc::api::references::{FindDefinition, FindLoc, FindUsages, GetLocSource, GetReferenceValues, LocSourceLookup, ReferenceValues, RowLocation, Usages};
use rpfm_ipc::api::notes::{AddNote, DeleteNote, ListNotes, NoteEntry, NoteList};
use rpfm_ipc::api::schema::{
    DeleteDefinition, GetRawDefinitions, GetReferencingColumns, GetTablePatches, ImportPatches, ListSchemaTables, PatchColumn, RawDefinitions, ReferencingColumns, RemovePatches, TablePatches,
    SchemaTables, SetDefinition, UpdateSchemaFromAssemblyKit, UpdateSchemas,
};
use rpfm_ipc::api::translations::{GenerateVanillaTexts, ListTranslations, Translations, VanillaTextsAvailable};
use rpfm_ipc::api::tools::{
    AnimsBySkeleton, ExportGltf, FilePaths, FilesChanged, FinishStartpos, GenerateMissingLocs, GetOptimizerOptions, GetStartposCampaigns, InitMyMod, LiveExport,
    MyModCreated, OptimizePack, OptimizerOptionValues, PackMap, PatchSiegeAi, RunLuaTests, SetVideoFormat, SiegeAiPatched, StartStartpos, StartposCampaigns,
    UpdateAnimIds,
};
use rpfm_ipc::api::updates::{ApplyUpdate, CheckUpdate, UpdateStatus};
use rpfm_ipc::api::files::{
    AddToAnimPack, DeleteFromAnimPack, ExtractFromAnimPack, FileContents, ListAnimPack, ReadFile, WriteFile,
    AddFilesFromDisk, CopyFiles, CreateFile, DeleteFiles, DuplicateFiles, ExtractFiles, FileEntry, FileList, FilesAdded, FilesDeleted,
    FilesExtracted, FilesRenamed, ListFiles, RenameFiles,
};
use rpfm_ipc::api::packs::{ClosePack, CloseAllPacks, GetPackInfo, GetPackSettings, NewPack, OpenPack, OpenVanillaPacks, PackDetails, PackSettingsValues, PackSummary, SavePack, UpdatePack, UpdatePackSettings};
use rpfm_ipc::api::jobs::{CancelJob, GetJobStatus, JobStarted, JobState, JobStatus, WaitForJob};
use rpfm_ipc::api::session::{DependencyTables, GenerateDependenciesCache, GetSessionStatus, ListDependencyTables, RebuildDependencies, SessionStatus, SetGame};
use rpfm_ipc::api::tables::{
    AddKeyDeletes, ColumnValues, EditTable, ExportTsv, FilesEdited, GetColumnValues, GetTableDefinition, GetTableInfo, GetTableRows, ImportTsv, MergeTables,
    RenameKey, TableDefinition, TableEdited, TableInfo, TableRows, TablesMerged, TableUpgraded, UpgradeTable,
};
use rpfm_ipc::messages::{Command, Response};
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

/// How long tools that run as jobs wait for them before returning their state.
///
/// Kept under the usual timeout of MCP clients. Jobs still running after it can be waited for with `wait_for_job`.
const MCP_JOB_WAIT: Duration = Duration::from_secs(45);

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

/// Returns an API error as a tool error.
fn error_result(error: &RpcError) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(serde_json::to_string(error).unwrap_or_else(|_| error.message.clone()))])
}

/// Returns the state of a job as structured content, marked as a tool error if the job failed.
fn job_status_result(status: &JobStatus) -> CallToolResult {
    match serde_json::to_value(status) {
        Ok(value) if matches!(status.state, JobState::Failed { .. }) => CallToolResult::structured_error(value),
        Ok(value) => CallToolResult::structured(value),
        Err(error) => error_result(&ApiError::Internal(error.to_string()).into()),
    }
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




// -- Pack Lifecycle Args --

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct PathArg {
    /// The file path.
    pub path: PathBuf,
}


// -- Pack Key Args (multi-pack support) --



// -- Pack Metadata Args --


// -- File Operations Args --








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






// -- Schema Args --


#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct StringI32Args {
    /// A string value (e.g., table name).
    pub name: String,
    /// An integer value (e.g., version).
    pub version: i32,
}



// -- Table Ops Args --




// -- Diagnostics Args --




// -- Notes Args --



// -- Optimization Args --


// -- Specialized Args --










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
  to discover available keys. Tools working on a pack take its key as `pack`.
- **Reading data**: `list_files`, `table_info`, `table_rows` and `table_definition` return small, \
  paginated, structured results. `read_file` reads any other file, as text, decoded JSON or raw bytes.
- **Editing tables**: `edit_table` inserts, updates and deletes rows by index, with values by column \
  name. To change a vanilla table, copy it into your pack with `copy_files` first.
- **Paths**: tools taking plain path strings treat a path as a file if one exists there, or as a \
  folder otherwise.
- **Jobs**: slow tools (`set_game`, `generate_dependencies_cache`, `run_diagnostics`, `run_search`, `optimize_pack`...) run as jobs. \
  They wait up to 45 seconds and return the job's state, with its result if it finished. If it's still \
  running, call `wait_for_job` with its ID. Other tools called meanwhile wait for the job to end.
- **Sources**: where a file is — `{\"pack\": <pack key>}` (an open pack), `\"game_files\"` (vanilla game data), \
  `\"parent_files\"` (packs the open packs depend on), or `\"assembly_kit\"` (Assembly Kit tables). A file is \
  referenced as `{\"source\": <source>, \"path\": <path>}`.

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

## Resources

Use `resources/list` and `resources/read` to browse reference data: valid enum values, game lists, \
and example JSON payloads without needing tool calls.

## Responses

Tools return structured results. On failure, they return a tool error with a `code`, a `message` and, \
in `data`, the `kind` of error (like `pack_not_found` or `schema_not_loaded`).
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
            resource("rpfm://enums/SupportedFormats", "SupportedFormats", "Valid video format values (CaVp8, Ivf).", "application/json"),
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

            "rpfm://enums/SupportedFormats" => serde_json::json!({
                "enum": "SupportedFormats",
                "description": "Video format options for CA VP8 video files.",
                "variants": [
                    {"name": "CaVp8", "description": "CA's custom VP8 format (default)."},
                    {"name": "Ivf", "description": "Standard VP8 IVF format."}
                ],
                "json_example": "\"CaVp8\""
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
    After editing a script, run `run_diagnostics`: it reports Lua syntax errors, invalid DB keys, unknown methods,
    wrong argument counts and unknown events (all but syntax errors need the game's Assembly Kit installed).
    To check what a script does, write tests for it and run them with `run_lua_tests`.

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

        // Jobs answer with their ID right away, so wait a bit for them and return their state.
        let result = match response.outcome {
            RpcOutcome::Result(value) if R::IS_JOB => match serde_json::from_value::<JobStarted>(value) {
                Ok(started) => match self.session.jobs().wait(started.job, MCP_JOB_WAIT).await {
                    Some(status) => job_status_result(&status),
                    None => error_result(&ApiError::JobNotFound(started.job).into()),
                },
                Err(error) => error_result(&ApiError::Internal(error.to_string()).into()),
            },
            RpcOutcome::Result(value) => CallToolResult::structured(value),
            RpcOutcome::Error(error) => error_result(&error),
        };

        tx.finish();
        Ok(result)
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
        description = "Select the game to work with, like `warhammer_3`, loading its schema and, by default, its dependencies (vanilla files, Assembly Kit tables, parent packs). Call this before opening packs. Runs as a job: waits up to 45 seconds and returns its state, with the session status as result if it finished; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn set_game(&self, params: Parameters<SetGame>) -> Result<CallToolResult, McpError> {
        self.call_api("set_game", params.0).await
    }

    #[tool(
        name = "generate_dependencies_cache",
        description = "Generate the dependencies cache of the selected game from its files and Assembly Kit, and load it. Needed once per game, and after game updates. Takes a while. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn generate_dependencies_cache(&self, params: Parameters<GenerateDependenciesCache>) -> Result<CallToolResult, McpError> {
        self.call_api("generate_dependencies_cache", params.0).await
    }

    #[tool(
        name = "rebuild_dependencies",
        description = "Reload the dependencies of the selected game, like after changing the packs the open packs depend on. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn rebuild_dependencies(&self, params: Parameters<RebuildDependencies>) -> Result<CallToolResult, McpError> {
        self.call_api("rebuild_dependencies", params.0).await
    }

    #[tool(
        name = "dependency_tables",
        description = "List the tables of the selected game's files, and the startpos and twad tables of its schema, with the version new tables of each type should use. Empty if the dependencies aren't loaded.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<DependencyTables>(),
    )]
    pub async fn dependency_tables(&self, params: Parameters<ListDependencyTables>) -> Result<CallToolResult, McpError> {
        self.call_api("dependency_tables", params.0).await
    }

    #[tool(
        name = "run_diagnostics",
        description = "Check the open packs for problems (invalid references, duplicated keys, outdated tables, script errors, etc.), and return a summary by level and type. The results are kept: read them with `list_diagnostics`. After fixing some files, pass their `paths` to check only them again. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn run_diagnostics_tool(&self, params: Parameters<RunDiagnostics>) -> Result<CallToolResult, McpError> {
        self.call_api("run_diagnostics", params.0).await
    }

    #[tool(
        name = "list_diagnostics",
        description = "List results of the last `run_diagnostics`, filtered by level, report type, pack and path prefix, in pages (100 by default; the total of matching results is always returned). Table results include the affected cells as [row, column], to fix them with `edit_table`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<DiagnosticList>(),
    )]
    pub async fn list_diagnostics(&self, params: Parameters<ListDiagnostics>) -> Result<CallToolResult, McpError> {
        self.call_api("list_diagnostics", params.0).await
    }

    #[tool(
        name = "ignore_diagnostics",
        description = "Make the next diagnostics checks of a pack skip results of files under a path, optionally only for some columns and report types. Saved in the pack's settings, so it applies after saving the pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn ignore_diagnostics(&self, params: Parameters<IgnoreDiagnostics>) -> Result<CallToolResult, McpError> {
        self.call_api("ignore_diagnostics", params.0).await
    }

    #[tool(
        name = "run_search",
        description = "Search text (or a regex) in open packs, the game files, the parent packs, the Assembly Kit tables, or the schema's column names, and return a summary of the matches by file type. The matches are kept: read them with `list_search_matches`, and replace them with `replace_search_matches`. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn run_search(&self, params: Parameters<RunSearch>) -> Result<CallToolResult, McpError> {
        self.call_api("run_search", params.0).await
    }

    #[tool(
        name = "list_search_matches",
        description = "List matches of the last `run_search`, filtered by file type and path prefix, in pages (100 by default; the total of matching matches is always returned). Each match has an ID to replace it, and details with where it is in its file (row and column for tables, row for text files).",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<SearchMatchList>(),
    )]
    pub async fn list_search_matches(&self, params: Parameters<ListSearchMatches>) -> Result<CallToolResult, McpError> {
        self.call_api("list_search_matches", params.0).await
    }

    #[tool(
        name = "replace_search_matches",
        description = "Replace matches of the last `run_search` by ID, or all of them, with a text. Only matches in open packs are replaced. The edited files are searched again, so match IDs change afterwards: list them again before replacing more.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<SearchReplaced>(),
    )]
    pub async fn replace_search_matches(&self, params: Parameters<ReplaceSearchMatches>) -> Result<CallToolResult, McpError> {
        self.call_api("replace_search_matches", params.0).await
    }

    #[tool(
        name = "column_values",
        description = "Get the distinct values of a column of a table, like all the faction keys of `factions_tables`, from the open packs and the dependencies, sorted. Filter by prefix, and page with offset/limit (500 by default; the total is always returned). Useful to find valid values for a referenced column: `table_info` tells which table and column a column references.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<ColumnValues>(),
    )]
    pub async fn column_values(&self, params: Parameters<GetColumnValues>) -> Result<CallToolResult, McpError> {
        self.call_api("column_values", params.0).await
    }

    #[tool(
        name = "find_definition",
        description = "Find the row where a value of a referenced table is defined, like the row of `factions_tables` with a faction key. Searches the open packs (starting with `pack`), the parent packs, the game files and the Assembly Kit tables.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<RowLocation>(),
    )]
    pub async fn find_definition(&self, params: Parameters<FindDefinition>) -> Result<CallToolResult, McpError> {
        self.call_api("find_definition", params.0).await
    }

    #[tool(
        name = "find_usages",
        description = "Find the rows of other tables referencing a value of a table, like everything using a faction key. The referencing columns are taken from the schema. Searches the open packs (or only `pack`), the parent packs and the game files, in pages (200 by default; the total is always returned).",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<Usages>(),
    )]
    pub async fn find_usages(&self, params: Parameters<FindUsages>) -> Result<CallToolResult, McpError> {
        self.call_api("find_usages", params.0).await
    }

    #[tool(
        name = "find_loc",
        description = "Find the row of a Loc file with a key. Searches the open packs (starting with `pack`), the parent packs and the game files.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<RowLocation>(),
    )]
    pub async fn find_loc(&self, params: Parameters<FindLoc>) -> Result<CallToolResult, McpError> {
        self.call_api("find_loc", params.0).await
    }

    #[tool(
        name = "loc_source",
        description = "Get the table, localised column and key values a loc key belongs to, like `factions`, `screen_name` and `[\"wh_main_emp_empire\"]` for `factions_screen_name_wh_main_emp_empire`. `source` is null if it can't be found.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<LocSourceLookup>(),
    )]
    pub async fn loc_source(&self, params: Parameters<GetLocSource>) -> Result<CallToolResult, McpError> {
        self.call_api("loc_source", params.0).await
    }

    #[tool(
        name = "pack_settings",
        description = "Get the settings of an open pack, like its diagnostics ignore rules (`diagnostics_files_to_ignore`), files to ignore when importing, or if it has autosaves disabled.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<PackSettingsValues>(),
    )]
    pub async fn pack_settings(&self, params: Parameters<GetPackSettings>) -> Result<CallToolResult, McpError> {
        self.call_api("pack_settings", params.0).await
    }

    #[tool(
        name = "update_pack_settings",
        description = "Change settings of an open pack. Only the keys you set are changed. Returns all the settings afterwards.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<PackSettingsValues>(),
    )]
    pub async fn update_pack_settings(&self, params: Parameters<UpdatePackSettings>) -> Result<CallToolResult, McpError> {
        self.call_api("update_pack_settings", params.0).await
    }

    #[tool(
        name = "list_notes",
        description = "Get the notes (comments) attached to a file or folder of an open pack, or all of them.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<NoteList>(),
    )]
    pub async fn list_notes(&self, params: Parameters<ListNotes>) -> Result<CallToolResult, McpError> {
        self.call_api("list_notes", params.0).await
    }

    #[tool(
        name = "add_note",
        description = "Attach a note (comment, with an optional link) to a file or folder of an open pack, or replace one by passing its `id`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<NoteEntry>(),
    )]
    pub async fn add_note(&self, params: Parameters<AddNote>) -> Result<CallToolResult, McpError> {
        self.call_api("add_note", params.0).await
    }

    #[tool(
        name = "delete_note",
        description = "Delete a note of an open pack.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn delete_note(&self, params: Parameters<DeleteNote>) -> Result<CallToolResult, McpError> {
        self.call_api("delete_note", params.0).await
    }

    #[tool(
        name = "schema_tables",
        description = "List the tables of the selected game's schema whose name starts with a prefix, with the versions it has definitions for, newest first.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<SchemaTables>(),
    )]
    pub async fn schema_tables(&self, params: Parameters<ListSchemaTables>) -> Result<CallToolResult, McpError> {
        self.call_api("schema_tables", params.0).await
    }

    #[tool(
        name = "patch_column",
        description = "Change how the schema describes a column with a local patch: description, if it's a key, default value, referenced table and column (`is_reference` as `table;column`), if it holds file paths, if it can't be empty, or if it's unused. Local patches survive schema updates, and apply right away.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn patch_column(&self, params: Parameters<PatchColumn>) -> Result<CallToolResult, McpError> {
        self.call_api("patch_column", params.0).await
    }

    #[tool(
        name = "remove_patches",
        description = "Remove the local schema patches of a table, or of one of its columns.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn remove_patches(&self, params: Parameters<RemovePatches>) -> Result<CallToolResult, McpError> {
        self.call_api("remove_patches", params.0).await
    }

    #[tool(
        name = "update_schemas",
        description = "Download the latest schemas, reload the selected game's one, and rebuild the dependencies. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn update_schemas(&self, params: Parameters<UpdateSchemas>) -> Result<CallToolResult, McpError> {
        self.call_api("update_schemas", params.0).await
    }

    #[tool(
        name = "update_schema_from_assembly_kit",
        description = "Update the selected game's schema with the table definitions of its Assembly Kit, and save it. For schema maintainers. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn update_schema_from_assembly_kit(&self, params: Parameters<UpdateSchemaFromAssemblyKit>) -> Result<CallToolResult, McpError> {
        self.call_api("update_schema_from_assembly_kit", params.0).await
    }

    #[tool(
        name = "merge_tables",
        description = "Merge tables of the same type of an open pack into a new one. With `delta: true`, rows are merged by key against the vanilla data; rows that can't be reconciled are returned as conflicts and nothing is written, so call it again with `resolutions` for them.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<TablesMerged>(),
    )]
    pub async fn merge_tables(&self, params: Parameters<MergeTables>) -> Result<CallToolResult, McpError> {
        self.call_api("merge_tables", params.0).await
    }

    #[tool(
        name = "upgrade_table",
        description = "Update a table of an open pack to the version it has in the game files, after a game update. Returns the columns removed and added.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<TableUpgraded>(),
    )]
    pub async fn upgrade_table(&self, params: Parameters<UpgradeTable>) -> Result<CallToolResult, McpError> {
        self.call_api("upgrade_table", params.0).await
    }

    #[tool(
        name = "rename_key",
        description = "Change a key value of a table in every table of an open pack: the key itself, the columns referencing it, and the loc keys generated from it. Use it to rename things safely.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<FilesEdited>(),
    )]
    pub async fn rename_key(&self, params: Parameters<RenameKey>) -> Result<CallToolResult, McpError> {
        self.call_api("rename_key", params.0).await
    }

    #[tool(
        name = "add_key_deletes",
        description = "Add keys to a key deletes table (`db/twad_key_deletes_tables/<file_name>`) of an open pack, to delete those keys of a table in the game.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesEdited>(),
    )]
    pub async fn add_key_deletes(&self, params: Parameters<AddKeyDeletes>) -> Result<CallToolResult, McpError> {
        self.call_api("add_key_deletes", params.0).await
    }

    #[tool(
        name = "export_tsv",
        description = "Write a table of an open pack, the game files or the parent packs to a TSV file, to edit it in a spreadsheet.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn export_tsv(&self, params: Parameters<ExportTsv>) -> Result<CallToolResult, McpError> {
        self.call_api("export_tsv", params.0).await
    }

    #[tool(
        name = "import_tsv",
        description = "Replace a table of an open pack with the contents of a TSV file, keeping its GUID.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<TableEdited>(),
    )]
    pub async fn import_tsv(&self, params: Parameters<ImportTsv>) -> Result<CallToolResult, McpError> {
        self.call_api("import_tsv", params.0).await
    }

    #[tool(
        name = "optimizer_options",
        description = "Get the optimizer options as the server's settings have them, by name. Use the names to change them in `optimize_pack`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<OptimizerOptionValues>(),
    )]
    pub async fn optimizer_options(&self) -> Result<CallToolResult, McpError> {
        self.call_api("optimizer_options", GetOptimizerOptions {}).await
    }

    #[tool(
        name = "patch_siege_ai",
        description = "Patch the siege maps of an open pack so the AI can use them. Warhammer games only.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<SiegeAiPatched>(),
    )]
    pub async fn patch_siege_ai(&self, params: Parameters<PatchSiegeAi>) -> Result<CallToolResult, McpError> {
        self.call_api("patch_siege_ai", params.0).await
    }

    #[tool(
        name = "pack_map",
        description = "Add the tiles and tile maps of a map exported by Terry to an open pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesChanged>(),
    )]
    pub async fn pack_map(&self, params: Parameters<PackMap>) -> Result<CallToolResult, McpError> {
        self.call_api("pack_map", params.0).await
    }

    #[tool(
        name = "generate_missing_locs",
        description = "Add empty loc entries for the localised columns of the tables of the open packs that don't have them yet.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesEdited>(),
    )]
    pub async fn generate_missing_locs(&self, params: Parameters<GenerateMissingLocs>) -> Result<CallToolResult, McpError> {
        self.call_api("generate_missing_locs", params.0).await
    }

    #[tool(
        name = "update_anim_ids",
        description = "Offset the animation ids of an open pack from a starting id, like after a game update moves them.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<FilesEdited>(),
    )]
    pub async fn update_anim_ids(&self, params: Parameters<UpdateAnimIds>) -> Result<CallToolResult, McpError> {
        self.call_api("update_anim_ids", params.0).await
    }

    #[tool(
        name = "anims_by_skeleton",
        description = "Get the paths of the animations using a skeleton, in the open packs and the dependencies.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<FilePaths>(),
    )]
    pub async fn anims_by_skeleton(&self, params: Parameters<AnimsBySkeleton>) -> Result<CallToolResult, McpError> {
        self.call_api("anims_by_skeleton", params.0).await
    }

    #[tool(
        name = "export_gltf",
        description = "Export a RigidModel of any source to a glTF file, with its textures.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn export_gltf(&self, params: Parameters<ExportGltf>) -> Result<CallToolResult, McpError> {
        self.call_api("export_gltf", params.0).await
    }

    #[tool(
        name = "set_video_format",
        description = "Change the format of a ca_vp8 video of an open pack: `CaVp8` or `Ivf`.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn set_video_format(&self, params: Parameters<SetVideoFormat>) -> Result<CallToolResult, McpError> {
        self.call_api("set_video_format", params.0).await
    }

    #[tool(
        name = "live_export",
        description = "Export the scripts and UI files of an open pack to the game's data folder, to test them without saving the pack.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn live_export(&self, params: Parameters<LiveExport>) -> Result<CallToolResult, McpError> {
        self.call_api("live_export", params.0).await
    }

    #[tool(
        name = "init_mymod",
        description = "Create the folder of a new MyMod in the MyMods folder of the settings, with optional editor configs for Lua scripting and a git repository.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<MyModCreated>(),
    )]
    pub async fn init_mymod(&self, params: Parameters<InitMyMod>) -> Result<CallToolResult, McpError> {
        self.call_api("init_mymod", params.0).await
    }

    #[tool(
        name = "startpos_campaigns",
        description = "Get the campaigns a startpos can be built for, and the one the pack's last startpos was built for.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<StartposCampaigns>(),
    )]
    pub async fn startpos_campaigns(&self, params: Parameters<GetStartposCampaigns>) -> Result<CallToolResult, McpError> {
        self.call_api("startpos_campaigns", params.0).await
    }

    #[tool(
        name = "start_startpos",
        description = "Start building a startpos for a campaign with the tables of an open pack: prepares the Assembly Kit and launches the game. Tell the user to close the game once it's done loading, then call `finish_startpos`.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn start_startpos(&self, params: Parameters<StartStartpos>) -> Result<CallToolResult, McpError> {
        self.call_api("start_startpos", params.0).await
    }

    #[tool(
        name = "finish_startpos",
        description = "Finish building a startpos after the game was closed: imports it into the pack, or cancels the build with `cancel: true`. Cleans up the build files either way.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesEdited>(),
    )]
    pub async fn finish_startpos(&self, params: Parameters<FinishStartpos>) -> Result<CallToolResult, McpError> {
        self.call_api("finish_startpos", params.0).await
    }

    #[tool(
        name = "optimize_pack",
        description = "Remove data of an open pack that's identical to vanilla or unneeded (duplicated rows, unchanged rows and files, empty tables, etc.). Options not set keep the server's settings; see `optimizer_options`. Runs as a job: waits up to 45 seconds and returns its state; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn optimize_pack(&self, params: Parameters<OptimizePack>) -> Result<CallToolResult, McpError> {
        self.call_api("optimize_pack", params.0).await
    }

    #[tool(
        name = "run_lua_tests",
        description = "Run Lua tests against the scripts of all open packs, outside of the game. Scripts run in Lua 5.1 with the game's real script libraries; the game's engine is emulated with objects typed after the Assembly Kit's scripting docs, which record every call made on them. Needs the game's Assembly Kit.

Test file API (a global `rpfm` table):
- Top-level code runs before the game boots, to set up the world: `local kislev = rpfm.faction { key = \"wh3_main_ksl_kislev\", is_human = true }`. Fields other than `key` are the values returned by the methods with the same name; lists like `region_list` can be plain arrays. Also `rpfm.region { key = ... }`, `rpfm.character { faction = kislev, ... }`, and `rpfm.object(\"TYPE_SCRIPT_INTERFACE\", methods)`. Factions and regions from the game's DB exist even if not set up.
- `rpfm.test(name, function)` registers a test. Each test runs in a fresh Lua state, after the libraries, the pack's mods (script/campaign/mod/) and the first tick have run.
- `rpfm.fire(event, { accessor = value, ... })` triggers an event, like `rpfm.fire(\"FactionTurnStart\", { faction = kislev })`.
- `rpfm.advance_time(seconds)` advances game time, triggering due time triggers, like the ones from `cm:callback`. `rpfm.end_turn()` plays a full round: `WorldStartRound`, `FactionRoundStart` for every faction, then for each faction in creation order its `FactionTurnStart`, the turn events of the regions and characters in its `region_list` and `character_list`, `FactionBeginTurnPhaseNormal`, `FactionAboutToEndTurn` and `FactionTurnEnd`.
- `rpfm.mock(object, method, value)` changes what a method of an engine object returns after boot, like `rpfm.mock(region, \"owning_faction\", kislev)`; `value` can be a function receiving the call's arguments. Scripts' own globals (like a mod's manager table) can be inspected and changed directly from tests.
- `rpfm.assert_called(method, args...)`, `rpfm.assert_not_called(method)`, `rpfm.calls_to(method)` and `rpfm.assert_equal(actual, expected)` check what the scripts did. Calls on `cm` are recorded by their method name, like `treasury_mod`.

The report lists each test with its errors (including errors of the pack's scripts, and in listeners), the undocumented methods it called (whose results are placeholders), and the scripts' output.

Runs as a job: waits up to 45 seconds and returns its state, with the report as its result; if it's still running, call `wait_for_job`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn run_lua_tests(&self, params: Parameters<RunLuaTests>) -> Result<CallToolResult, McpError> {
        self.call_api("run_lua_tests", params.0).await
    }

    #[tool(
        name = "check_update",
        description = "Check if there is an update of RPFM (`program`), the `schemas`, the Lua type definitions (`lua_autogen`), the Empire and Napoleon Assembly Kit data (`old_assembly_kit`), or the community `translations`.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<UpdateStatus>(),
    )]
    pub async fn check_update(&self, params: Parameters<CheckUpdate>) -> Result<CallToolResult, McpError> {
        self.call_api("check_update", params.0).await
    }

    #[tool(
        name = "apply_update",
        description = "Download the update of the Lua type definitions (`lua_autogen`), the Empire and Napoleon Assembly Kit data (`old_assembly_kit`), the community `translations`, or RPFM itself (`program`, which replaces its files and needs a restart: only do it if the user asks). Schemas are updated with `update_schemas`.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn apply_update(&self, params: Parameters<ApplyUpdate>) -> Result<CallToolResult, McpError> {
        self.call_api("apply_update", params.0).await
    }

    #[tool(
        name = "read_file",
        description = "Read a file of an open pack, the game files or the parent packs: as `text` for scripts, XML, JSON and other text files; as `decoded` JSON for structured formats (portrait settings, unit variants, models, etc.); or as `raw` base64 bytes, best for images and other binary files. For DB and Loc tables, prefer `table_rows`, which returns only the rows and columns you ask for.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<FileContents>(),
    )]
    pub async fn read_file(&self, params: Parameters<ReadFile>) -> Result<CallToolResult, McpError> {
        self.call_api("read_file", params.0).await
    }

    #[tool(
        name = "write_file",
        description = "Replace the contents of a file of an open pack: `text` for text files, `decoded` JSON in the format `read_file` returns, or `raw` base64 bytes (which also creates the file if it doesn't exist). For DB and Loc tables, prefer `edit_table`.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn write_file(&self, params: Parameters<WriteFile>) -> Result<CallToolResult, McpError> {
        self.call_api("write_file", params.0).await
    }

    #[tool(
        name = "list_animpack",
        description = "List the files inside an AnimPack of an open pack, the game files or the parent packs.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<FileList>(),
    )]
    pub async fn list_animpack(&self, params: Parameters<ListAnimPack>) -> Result<CallToolResult, McpError> {
        self.call_api("list_animpack", params.0).await
    }

    #[tool(
        name = "add_to_animpack",
        description = "Copy files of an open pack into an AnimPack of an open pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesAdded>(),
    )]
    pub async fn add_to_animpack(&self, params: Parameters<AddToAnimPack>) -> Result<CallToolResult, McpError> {
        self.call_api("add_to_animpack", params.0).await
    }

    #[tool(
        name = "extract_from_animpack",
        description = "Copy files of an AnimPack of any source into an open pack.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<FilesAdded>(),
    )]
    pub async fn extract_from_animpack(&self, params: Parameters<ExtractFromAnimPack>) -> Result<CallToolResult, McpError> {
        self.call_api("extract_from_animpack", params.0).await
    }

    #[tool(
        name = "delete_from_animpack",
        description = "Delete files from an AnimPack of an open pack.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn delete_from_animpack(&self, params: Parameters<DeleteFromAnimPack>) -> Result<CallToolResult, McpError> {
        self.call_api("delete_from_animpack", params.0).await
    }

    #[tool(
        name = "raw_definitions",
        description = "Get the definitions of a table as the schema stores them, to edit them with `set_definition`. Their fields are the raw on-disk layout, which can differ from the columns rows have: use `table_definition` for those.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<RawDefinitions>(),
    )]
    pub async fn raw_definitions(&self, params: Parameters<GetRawDefinitions>) -> Result<CallToolResult, McpError> {
        self.call_api("raw_definitions", params.0).await
    }

    #[tool(
        name = "set_definition",
        description = "Add a definition to the selected game's schema, or replace the one with its version, then save and reload the schema. For schema maintainers; use `patch_column` to change column metadata locally instead.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn set_definition(&self, params: Parameters<SetDefinition>) -> Result<CallToolResult, McpError> {
        self.call_api("set_definition", params.0).await
    }

    #[tool(
        name = "delete_definition",
        description = "Remove a definition from the selected game's schema, then save and reload the schema. For schema maintainers.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn delete_definition(&self, params: Parameters<DeleteDefinition>) -> Result<CallToolResult, McpError> {
        self.call_api("delete_definition", params.0).await
    }

    #[tool(
        name = "referencing_columns",
        description = "Get the columns of other tables referencing each column of a table, according to the schema.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<ReferencingColumns>(),
    )]
    pub async fn referencing_columns(&self, params: Parameters<GetReferencingColumns>) -> Result<CallToolResult, McpError> {
        self.call_api("referencing_columns", params.0).await
    }

    #[tool(
        name = "table_patches",
        description = "Get the patches applied to the columns of a table definition: the local ones made with `patch_column`, and the ones included in the schema.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<TablePatches>(),
    )]
    pub async fn table_patches(&self, params: Parameters<GetTablePatches>) -> Result<CallToolResult, McpError> {
        self.call_api("table_patches", params.0).await
    }

    #[tool(
        name = "import_patches",
        description = "Add patches to the selected game's schema itself, and save it. Unlike `patch_column`, they're part of the schema, so a schema update replaces them. For schema maintainers.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn import_patches(&self, params: Parameters<ImportPatches>) -> Result<CallToolResult, McpError> {
        self.call_api("import_patches", params.0).await
    }

    #[tool(
        name = "reference_values",
        description = "Get the values a reference column of a table can have, with their display text (like the name of each referenced row), from the open packs and the dependencies. Filter by prefix and page with offset/limit (500 by default).",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<ReferenceValues>(),
    )]
    pub async fn reference_values(&self, params: Parameters<GetReferenceValues>) -> Result<CallToolResult, McpError> {
        self.call_api("reference_values", params.0).await
    }

    #[tool(
        name = "list_translations",
        description = "Get the translation of the texts of an open pack to a language, reusing vanilla and previous translations. Filter to the `untranslated` entries or the ones that `needs_review`, and page with offset/limit (200 by default).",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<Translations>(),
    )]
    pub async fn list_translations(&self, params: Parameters<ListTranslations>) -> Result<CallToolResult, McpError> {
        self.call_api("list_translations", params.0).await
    }

    #[tool(
        name = "generate_vanilla_texts",
        description = "Generate the vanilla texts of a language from the game's locale packs, so translations to and from it can reuse them.",
        annotations(read_only_hint = false, destructive_hint = false),
        output_schema = schema_for_output::<VanillaTextsAvailable>(),
    )]
    pub async fn generate_vanilla_texts(&self, params: Parameters<GenerateVanillaTexts>) -> Result<CallToolResult, McpError> {
        self.call_api("generate_vanilla_texts", params.0).await
    }

    #[tool(
        name = "job_status",
        description = "Get the state of a job: queued, running (with its current step), finished (with its result), failed (with its error) or cancelled.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn job_status(&self, params: Parameters<GetJobStatus>) -> Result<CallToolResult, McpError> {
        self.call_api("job_status", params.0).await
    }

    #[tool(
        name = "wait_for_job",
        description = "Wait for a job to end, up to `timeout_secs` (60 by default), and return its state.",
        annotations(read_only_hint = true),
        output_schema = schema_for_output::<JobStatus>(),
    )]
    pub async fn wait_for_job(&self, params: Parameters<WaitForJob>) -> Result<CallToolResult, McpError> {
        self.call_api("wait_for_job", params.0).await
    }

    #[tool(
        name = "cancel_job",
        description = "Cancel a job that hasn't started yet. Running jobs can't be cancelled.",
        annotations(read_only_hint = false, destructive_hint = true),
        output_schema = schema_for_output::<Done>(),
    )]
    pub async fn cancel_job(&self, params: Parameters<CancelJob>) -> Result<CallToolResult, McpError> {
        self.call_api("cancel_job", params.0).await
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

    //-----------------------------------------------------------------------//
    // File Operations
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Game Selection
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Dependencies
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Search
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Schema
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Table Operations
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Diagnostics
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Notes
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Optimization
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Updates
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Specialized
    //-----------------------------------------------------------------------//

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
   `filters`. For other files, call `read_file`: as `text` for scripts and other text files,
   or as `decoded` JSON for the rest.

5. **Inspect metadata** – Use `pack_info` and `pack_settings` to answer questions about the pack itself.

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
- Use `reference_values` to discover valid values for referenced columns, with their names.
- Mods usually only keep the rows they change: `optimize_pack` removes rows identical to vanilla.
- After saving, you can run `run_diagnostics` to validate the pack.
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
- `init_mymod` to set up a mod development folder with IDE support.
- `optimize_pack` to strip unchanged rows that match vanilla data.
- `run_diagnostics` to validate everything before release.
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

2. **Run a search** – Call `run_search` with the `pattern`, the `sources` to search
   (e.g. `[{\"pack\": <pack key>}]`), and optionally `file_types` (DB, Loc and text by default),
   `case_sensitive` and `use_regex`. It returns how many matches there are per file type.

3. **Review matches** – `list_search_matches`, filtered by `file_types` or `path_prefix` if there
   are many. Present them to the user for review, with their IDs.

4. **Replace selectively** – Call `replace_search_matches` with the `replace_text` and the IDs of
   the matches the user approved.

5. **Or replace all** – If the user confirms a blanket replace, call `replace_search_matches`
   without `matches`. Match IDs change after each replace, so list them again before replacing more.

6. **Save** – `save_pack` to persist changes.

Related tools:
- `find_usages` – Find all rows that reference a specific value across tables.
- `find_definition` – Find where a referenced key is defined.
- `find_loc` – Find the loc entry for a given key.
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

1. **Set the game** with `set_game` and **open the pack** with `open_pack`.

2. **Generate dependencies** – If `session_status` shows the vanilla data isn't loaded,
   call `generate_dependencies_cache` to build the dependency data needed for diagnostics.

3. **Run diagnostics** – `run_diagnostics`, optionally with `check_assembly_kit_only_references: true`
   to include Assembly Kit references. It returns a summary with the amount of results per level and type.

4. **Review results** – `list_diagnostics` filtered by `levels` (e.g. `[\"error\"]`), `report_types` or
   `path_prefix`, and present them to the user grouped by severity. Common issues include:
   - Invalid references (a column references a key that does not exist)
   - Duplicate keys
   - Empty loc entries
   - Outdated table versions

5. **Fix issues** – For each issue in a table, find the affected rows with `table_rows` and fix them
   with `edit_table` (correct a reference, remove a duplicate row, etc.).

6. **Ignore false positives** – Use `ignore_diagnostics` to skip results that are intentional,
   for a path and, optionally, only some columns and report types.

7. **Re-check** – After fixes, call `run_diagnostics` with the `paths` of the fixed files to check
   only them again, and confirm the issues are resolved.

8. **Optimize** – Optionally run `optimize_pack` to remove rows that are identical
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

2. **List tables** – `schema_tables` lists the tables of the schema and their versions.

3. **See the columns rows have** – `table_definition` returns the columns of a table
   as rows see them, with bitwise expansion, enum conversions, and colour-group merging
   applied. `raw_definitions` returns the raw on-disk field list instead (e.g. a colour
   column split into separate r/g/b fields), which has a different length/order than
   actual row data. Use it only to edit the schema itself.

4. **Find referencing columns** – `referencing_columns` shows which columns of other
   tables reference each column of a table.

5. **Edit definitions** – `set_definition` adds or replaces a definition and
   `delete_definition` removes one. Both save and reload the schema.

6. **Patch a column** – To customise column metadata (descriptions, references,
   default values) without modifying the upstream schema, call `patch_column` with the
   table, the column and the keys to set. Use `remove_patches` to undo them.

7. **Import patches** – `import_patches` adds patches to the schema itself.

8. **Update from Assembly Kit** – `update_schema_from_assembly_kit` merges
   definition data from the game's Assembly Kit into the loaded schema.
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
- `write_file` – Replace the contents of a file: as text, decoded JSON, or raw bytes (which also creates new files).

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
- `list_animpack` – List the files inside an AnimPack.
- `add_to_animpack` – Add files to an AnimPack.
- `extract_from_animpack` – Copy files from an AnimPack into a pack.
- `delete_from_animpack` – Remove files from an AnimPack.

**Other:**
- `read_file` – Read any file as text, decoded JSON, or raw base64 bytes.
- `merge_tables` – Combine multiple compatible tables into one.

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
**Symptom**: Files fail to decode, or `read_file` says they can't be decoded.
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
- Use `upgrade_table` to migrate the table to the current version.
- Check `table_definition` for the expected columns.

### 5. Wrong game selected
**Symptom**: Tables decode with wrong columns or fail to decode, dependencies are for a different game.
**Solution**:
- Call `session_status()` to verify the current game.
- Call `set_game` with the correct game key.

### 6. Diagnostics show many reference errors
**Symptom**: `run_diagnostics` reports hundreds of invalid references.
**Solution**:
- Ensure dependencies are loaded (`dependencies` in `session_status`).
- Check if the pack depends on other mods via `pack_info`.
- Some references are Assembly Kit only; re-run with `check_assembly_kit_only_references: true`.
- Use `ignore_diagnostics` for intentional deviations.

### Diagnostic Tools
- `run_diagnostics` / `list_diagnostics` – Full pack validation.
- `session_status` – Verify the game, schema, dependencies and open packs.
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

1. **Set the game** with `set_game` and **open the pack** with `open_pack`.

2. **Export a single table as TSV**:
   Call `export_tsv` with:
   - `file`: the table, like `{\"source\": {\"pack\": <pack key>}, \"path\": \"db/land_units_tables/my_mod\"}`
   - `destination`: destination path on disk (e.g. `/home/user/my_table.tsv`)

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
   - `pack`: the target pack key
   - `path`: the internal path of the table to replace
   - `source`: path to the TSV file on disk

3. **Verify**: Call `table_rows` to confirm the data imported correctly.

4. **Save the pack**: Call `save_pack` to persist changes.

## Tips
- TSV files include metadata headers that RPFM uses for schema matching.
  Do not delete or modify these header rows.
- Use `table_info` or `table_definition` to understand column types before editing.
- After import, run `run_diagnostics` to validate references.
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
   Call `list_translations` with the pack key and a language code (e.g. `\"FR\"`, `\"DE\"`,
   `\"ES\"`, `\"IT\"`, `\"ZH\"`, `\"RU\"`), filtering to the `untranslated` entries if there are many.

## Creating New Translations

1. **Create a new loc file**:
   Call `create_file` with path `\"text/db/my_mod.loc\"` and `kind: {\"type\": \"loc\"}`.

2. **Add entries**: `edit_table` with inserts like
   `{\"op\": \"insert\", \"values\": {\"key\": \"key_string\", \"text\": \"Displayed text in game\"}}`.

3. **Save the pack**: `save_pack`.

## Generating Missing Loc Data

Call `generate_missing_locs` to auto-generate
loc entries for DB fields that reference loc keys but don't have entries yet.

## Finding Loc Keys

- Use `find_loc` with a loc key to find the loc file with it.
- Use `loc_source` to find the table row a loc key belongs to.
- Use `run_search` with `file_types: [\"loc\"]` to search across all loc files.

## Tips
- Loc keys follow naming conventions like `<table>_<loc_column_name>_<keys_concatenated>`.
- Use `find_usages` to find all the rows referencing a specific key.
- After adding translations, run `run_diagnostics` to verify all references.
",
        )]
    }
}
