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
use rmcp::handler::server::{tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, Content, ErrorCode, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::path::PathBuf;

use rpfm_ipc::helpers::DataSource;
use rpfm_ipc::messages::Command;
use rpfm_lib::files::{ContainerPath, RFile, RFileDecoded};
use rpfm_log::sentry;

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

        let json = serde_json::to_string(&response).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to serialize response: {e}").into(),
            data: None,
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }};
}

/// Parse a JSON string into the expected type, returning an MCP INVALID_PARAMS error on failure.
fn parse_json<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, McpError> {
    serde_json::from_str(input).map_err(|e| McpError {
        code: ErrorCode::INVALID_PARAMS,
        message: format!("Invalid JSON parameter: {e}").into(),
        data: None,
    })
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone)]
pub struct McpServer {
    session: Arc<Session>,
    tool_router: ToolRouter<Self>,
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
impl rmcp::ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This is a Model Context Protocol (MCP) server for RPFM (Rusted PackFile Manager). It allows you to interact with RFile and PackFiles using various tools.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tool_router]
impl McpServer {

    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            tool_router: McpServer::tool_router()
        }
    }

    //-----------------------------------------------------------------------//
    // Existing tools
    //-----------------------------------------------------------------------//

    #[tool(name = "call_command", description = "Call any IPC command directly. Use this for commands not yet wrapped as named tools.")]
    pub async fn call_command(&self, params: Parameters<CallCommandArgs>) -> Result<CallToolResult, McpError> {
        let command: Command = parse_json(&params.0.command)?;
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

    #[tool(description = "Set the type of the pack identified by `pack_key`. Pass the PFHFileType as JSON.")]
    pub async fn set_pack_file_type(&self, params: Parameters<SetPackFileTypeArgs>) -> Result<CallToolResult, McpError> {
        let pfh_type = parse_json(&params.0.pack_file_type)?;
        send_and_respond!(self, "set_pack_file_type", Command::SetPackFileType(params.0.pack_key, pfh_type))
    }

    #[tool(description = "Change the compression format of the pack identified by `pack_key`. Pass the CompressionFormat as JSON.")]
    pub async fn change_compression_format(&self, params: Parameters<ChangeCompressionFormatArgs>) -> Result<CallToolResult, McpError> {
        let format = parse_json(&params.0.format)?;
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

    #[tool(description = "Set the settings of the pack identified by `pack_key`. Pass PackSettings as JSON.")]
    pub async fn set_pack_settings(&self, params: Parameters<SetPackSettingsArgs>) -> Result<CallToolResult, McpError> {
        let settings = parse_json(&params.0.settings)?;
        send_and_respond!(self, "set_pack_settings", Command::SetPackSettings(params.0.pack_key, settings))
    }

    #[tool(description = "Get the list of PackFiles marked as dependencies of the pack identified by `pack_key`.")]
    pub async fn get_dependency_pack_files_list(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_dependency_pack_files_list", Command::GetDependencyPackFilesList(params.0.pack_key))
    }

    #[tool(description = "Set the list of PackFiles marked as dependencies for the pack identified by `pack_key`. Pass Vec<(bool, String)> as JSON.")]
    pub async fn set_dependency_pack_files_list(&self, params: Parameters<SetDependencyPackFilesListArgs>) -> Result<CallToolResult, McpError> {
        let list = parse_json(&params.0.list)?;
        send_and_respond!(self, "set_dependency_pack_files_list", Command::SetDependencyPackFilesList(params.0.pack_key, list))
    }

    //-----------------------------------------------------------------------//
    // File Operations
    //-----------------------------------------------------------------------//

    #[tool(description = "Decode a file from the pack identified by `pack_key`. The parameters are the path of the file inside the data source, and in what data source it is.")]
    pub async fn decode_packed_file(&self, params: Parameters<DecodePackedFileArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "decode_packed_file", Command::DecodePackedFile(params.0.pack_key, params.0.path, params.0.source))
    }

    #[tool(description = "Create a new file inside the pack identified by `pack_key`. Pass the NewFile type as JSON.")]
    pub async fn new_packed_file(&self, params: Parameters<NewPackedFileArgs>) -> Result<CallToolResult, McpError> {
        let new_file = parse_json(&params.0.new_file)?;
        send_and_respond!(self, "new_packed_file", Command::NewPackedFile(params.0.pack_key, params.0.path, new_file))
    }

    #[tool(description = "Add files from disk to the pack identified by `pack_key`. Pass destination ContainerPaths as JSON.")]
    pub async fn add_packed_files(&self, params: Parameters<AddPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let dest: Vec<ContainerPath> = parse_json(&params.0.destination_paths)?;
        send_and_respond!(self, "add_packed_files", Command::AddPackedFiles(params.0.pack_key, params.0.source_paths, dest, params.0.ignore_paths))
    }

    #[tool(description = "Add files from another PackFile to the pack identified by `pack_key`. Pass ContainerPaths as JSON.")]
    pub async fn add_packed_files_from_pack_file(&self, params: Parameters<AddPackedFilesFromPackFileArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.container_paths)?;
        send_and_respond!(self, "add_packed_files_from_pack_file", Command::AddPackedFilesFromPackFile(params.0.pack_key, params.0.source_pack_path, paths))
    }

    #[tool(description = "Add files from the pack identified by `pack_key` to an AnimPack. Pass ContainerPaths as JSON.")]
    pub async fn add_packed_files_from_pack_file_to_animpack(&self, params: Parameters<AddPackedFilesFromPackFileToAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.container_paths)?;
        send_and_respond!(self, "add_packed_files_from_pack_file_to_animpack", Command::AddPackedFilesFromPackFileToAnimpack(params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Add files from an AnimPack to the pack identified by `pack_key`. Pass ContainerPaths as JSON.")]
    pub async fn add_packed_files_from_animpack(&self, params: Parameters<AddPackedFilesFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.container_paths)?;
        send_and_respond!(self, "add_packed_files_from_animpack", Command::AddPackedFilesFromAnimpack(params.0.pack_key, params.0.source, params.0.animpack_path, paths))
    }

    #[tool(description = "Delete files from the pack identified by `pack_key`. Pass Vec<ContainerPath> as JSON.")]
    pub async fn delete_packed_files(&self, params: Parameters<ContainerPathsArg>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.paths)?;
        send_and_respond!(self, "delete_packed_files", Command::DeletePackedFiles(params.0.pack_key, paths))
    }

    #[tool(description = "Delete files from an AnimPack in the pack identified by `pack_key`. Pass ContainerPaths as JSON.")]
    pub async fn delete_from_animpack(&self, params: Parameters<DeleteFromAnimpackArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.container_paths)?;
        send_and_respond!(self, "delete_from_animpack", Command::DeleteFromAnimpack(params.0.pack_key, params.0.animpack_path, paths))
    }

    #[tool(description = "Extract files from the pack identified by `pack_key` to disk. Pass source paths as JSON BTreeMap<DataSource, Vec<ContainerPath>>.")]
    pub async fn extract_packed_files(&self, params: Parameters<ExtractPackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let source: BTreeMap<DataSource, Vec<ContainerPath>> = parse_json(&params.0.source_paths)?;
        send_and_respond!(self, "extract_packed_files", Command::ExtractPackedFiles(params.0.pack_key, source, params.0.destination_path, params.0.export_as_tsv))
    }

    #[tool(description = "Rename files in the pack identified by `pack_key`. Pass Vec<(ContainerPath, ContainerPath)> as JSON.")]
    pub async fn rename_packed_files(&self, params: Parameters<RenamePackedFilesArgs>) -> Result<CallToolResult, McpError> {
        let renames: Vec<(ContainerPath, ContainerPath)> = parse_json(&params.0.renames)?;
        send_and_respond!(self, "rename_packed_files", Command::RenamePackedFiles(params.0.pack_key, renames))
    }

    #[tool(description = "Save an edited decoded file back to the pack identified by `pack_key`. Pass RFileDecoded as JSON.")]
    pub async fn save_packed_file_from_view(&self, params: Parameters<SavePackedFileFromViewArgs>) -> Result<CallToolResult, McpError> {
        let data: RFileDecoded = parse_json(&params.0.data)?;
        send_and_respond!(self, "save_packed_file_from_view", Command::SavePackedFileFromView(params.0.pack_key, params.0.path, data))
    }

    #[tool(description = "Save a file from an external program back to the pack identified by `pack_key`.")]
    pub async fn save_packed_file_from_external_view(&self, params: Parameters<SavePackedFileFromExternalViewArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "save_packed_file_from_external_view", Command::SavePackedFileFromExternalView(params.0.pack_key, params.0.internal_path, params.0.external_path))
    }

    #[tool(description = "Save files to the pack identified by `pack_key` and optionally clean. Pass Vec<RFile> as JSON.")]
    pub async fn save_packed_files_to_pack_file_and_clean(&self, params: Parameters<SavePackedFilesToPackFileAndCleanArgs>) -> Result<CallToolResult, McpError> {
        let files: Vec<RFile> = parse_json(&params.0.files)?;
        send_and_respond!(self, "save_packed_files_to_pack_file_and_clean", Command::SavePackedFilesToPackFileAndClean(params.0.pack_key, files, params.0.optimize))
    }

    #[tool(description = "Get the raw binary data of a file in the pack identified by `pack_key`.")]
    pub async fn get_packed_file_raw_data(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "get_packed_file_raw_data", Command::GetPackedFileRawData(params.0.pack_key, params.0.value))
    }

    #[tool(description = "Open a file in an external program from the pack identified by `pack_key`. Pass ContainerPath as JSON.")]
    pub async fn open_packed_file_in_external_program(&self, params: Parameters<OpenPackedFileInExternalProgramArgs>) -> Result<CallToolResult, McpError> {
        let cp: ContainerPath = parse_json(&params.0.container_path)?;
        send_and_respond!(self, "open_packed_file_in_external_program", Command::OpenPackedFileInExternalProgram(params.0.pack_key, params.0.source, cp))
    }

    #[tool(description = "Open the folder containing the pack identified by `pack_key` in the file manager.")]
    pub async fn open_containing_folder(&self, params: Parameters<PackKeyArg>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "open_containing_folder", Command::OpenContainingFolder(params.0.pack_key))
    }

    #[tool(description = "Clean the decode cache for the provided paths in the pack identified by `pack_key`. Pass Vec<ContainerPath> as JSON.")]
    pub async fn clean_cache(&self, params: Parameters<ContainerPathsArg>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.paths)?;
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

    #[tool(description = "Set the current game selected. You need to set this to one of the valid games after opening a pack.")]
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

    #[tool(description = "Import files from dependencies into the pack identified by `pack_key`. Pass BTreeMap<DataSource, Vec<ContainerPath>> as JSON.")]
    pub async fn import_dependencies_to_open_pack_file(&self, params: Parameters<ImportDependenciesArgs>) -> Result<CallToolResult, McpError> {
        let paths: BTreeMap<DataSource, Vec<ContainerPath>> = parse_json(&params.0.paths)?;
        send_and_respond!(self, "import_dependencies_to_open_pack_file", Command::ImportDependenciesToOpenPackFile(params.0.pack_key, paths))
    }

    #[tool(description = "Get files from all known sources (PackFile, GameFiles, ParentFiles). Pass Vec<ContainerPath> as JSON.")]
    pub async fn get_rfiles_from_all_sources(&self, params: Parameters<GetRFilesFromAllSourcesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.paths)?;
        send_and_respond!(self, "get_rfiles_from_all_sources", Command::GetRFilesFromAllSources(paths, params.0.lowercase))
    }

    #[tool(description = "Get all file names under a path in all dependencies. Pass ContainerPath as JSON.")]
    pub async fn get_packed_files_names_starting_with_path_from_all_sources(&self, params: Parameters<ContainerPathArg>) -> Result<CallToolResult, McpError> {
        let path: ContainerPath = parse_json(&params.0.path)?;
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

    #[tool(description = "Run a global search across the pack identified by `pack_key`. Pass GlobalSearch as JSON.")]
    pub async fn global_search(&self, params: Parameters<GlobalSearchArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json(&params.0.search)?;
        send_and_respond!(self, "global_search", Command::GlobalSearch(params.0.pack_key, search))
    }

    #[tool(description = "Replace specific matches in a global search for the pack identified by `pack_key`. Pass GlobalSearch and Vec<MatchHolder> as JSON.")]
    pub async fn global_search_replace_matches(&self, params: Parameters<GlobalSearchReplaceMatchesArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json(&params.0.search)?;
        let matches = parse_json(&params.0.matches)?;
        send_and_respond!(self, "global_search_replace_matches", Command::GlobalSearchReplaceMatches(params.0.pack_key, search, matches))
    }

    #[tool(description = "Replace all matches in a global search for the pack identified by `pack_key`. Pass GlobalSearch as JSON.")]
    pub async fn global_search_replace_all(&self, params: Parameters<GlobalSearchArgs>) -> Result<CallToolResult, McpError> {
        let search = parse_json(&params.0.search)?;
        send_and_respond!(self, "global_search_replace_all", Command::GlobalSearchReplaceAll(params.0.pack_key, search))
    }

    #[tool(description = "Find all references to a value in the pack identified by `pack_key`. Pass HashMap<String, Vec<String>> as JSON for the reference map.")]
    pub async fn search_references(&self, params: Parameters<SearchReferencesArgs>) -> Result<CallToolResult, McpError> {
        let map: HashMap<String, Vec<String>> = parse_json(&params.0.reference_map)?;
        send_and_respond!(self, "search_references", Command::SearchReferences(params.0.pack_key, map, params.0.value))
    }

    #[tool(description = "Get reference data for columns in a definition for the pack identified by `pack_key`. Pass Definition as JSON.")]
    pub async fn get_reference_data_from_definition(&self, params: Parameters<GetReferenceDataFromDefinitionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json(&params.0.definition)?;
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

    #[tool(description = "Save the provided schema to disk. Pass Schema as JSON.")]
    pub async fn save_schema(&self, params: Parameters<SaveSchemaArgs>) -> Result<CallToolResult, McpError> {
        let schema = parse_json(&params.0.schema)?;
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

    #[tool(description = "Get columns that reference a table's definition. Pass Definition as JSON.")]
    pub async fn referencing_columns_for_definition(&self, params: Parameters<ReferencingColumnsForDefinitionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json(&params.0.definition)?;
        send_and_respond!(self, "referencing_columns_for_definition", Command::ReferencingColumnsForDefinition(params.0.table_name, def))
    }

    #[tool(description = "Get the processed fields from a definition (bitwise expansion, enum conversion applied). Pass Definition as JSON.")]
    pub async fn fields_processed(&self, params: Parameters<DefinitionArg>) -> Result<CallToolResult, McpError> {
        let def = parse_json(&params.0.definition)?;
        send_and_respond!(self, "fields_processed", Command::FieldsProcessed(def))
    }

    #[tool(description = "Save local schema patches. Pass HashMap<String, DefinitionPatch> as JSON.")]
    pub async fn save_local_schema_patch(&self, params: Parameters<SchemaPatchArgs>) -> Result<CallToolResult, McpError> {
        let patches = parse_json(&params.0.patches)?;
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

    #[tool(description = "Import a schema patch. Pass HashMap<String, DefinitionPatch> as JSON.")]
    pub async fn import_schema_patch(&self, params: Parameters<SchemaPatchArgs>) -> Result<CallToolResult, McpError> {
        let patches = parse_json(&params.0.patches)?;
        send_and_respond!(self, "import_schema_patch", Command::ImportSchemaPatch(patches))
    }

    //-----------------------------------------------------------------------//
    // Table Operations
    //-----------------------------------------------------------------------//

    #[tool(description = "Merge multiple compatible tables into one in the pack identified by `pack_key`. Pass Vec<ContainerPath> as JSON.")]
    pub async fn merge_files(&self, params: Parameters<MergeFilesArgs>) -> Result<CallToolResult, McpError> {
        let paths: Vec<ContainerPath> = parse_json(&params.0.paths)?;
        send_and_respond!(self, "merge_files", Command::MergeFiles(params.0.pack_key, paths, params.0.merged_path, params.0.delete_source))
    }

    #[tool(description = "Update a table to a newer version in the pack identified by `pack_key`. Pass ContainerPath as JSON.")]
    pub async fn update_table(&self, params: Parameters<PackKeyStringArg>) -> Result<CallToolResult, McpError> {
        let path: ContainerPath = parse_json(&params.0.value)?;
        send_and_respond!(self, "update_table", Command::UpdateTable(params.0.pack_key, path))
    }

    #[tool(description = "Trigger a cascade edition on all referenced data in the pack identified by `pack_key`. Pass Definition and Vec<(Field, String, String)> as JSON.")]
    pub async fn cascade_edition(&self, params: Parameters<CascadeEditionArgs>) -> Result<CallToolResult, McpError> {
        let def = parse_json(&params.0.definition)?;
        let changes = parse_json(&params.0.changes)?;
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

    #[tool(description = "Update diagnostics for changed files across all open packs. Pass Diagnostics and Vec<ContainerPath> as JSON.")]
    pub async fn diagnostics_update(&self, params: Parameters<DiagnosticsUpdateArgs>) -> Result<CallToolResult, McpError> {
        let diag = parse_json(&params.0.diagnostics)?;
        let paths: Vec<ContainerPath> = parse_json(&params.0.paths)?;
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

    #[tool(description = "Add a note to the pack identified by `pack_key`. Pass Note as JSON.")]
    pub async fn add_note(&self, params: Parameters<AddNoteArgs>) -> Result<CallToolResult, McpError> {
        let note = parse_json(&params.0.note)?;
        send_and_respond!(self, "add_note", Command::AddNote(params.0.pack_key, note))
    }

    #[tool(description = "Delete a note by path and ID in the pack identified by `pack_key`.")]
    pub async fn delete_note(&self, params: Parameters<DeleteNoteArgs>) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "delete_note", Command::DeleteNote(params.0.pack_key, params.0.path, params.0.id))
    }

    //-----------------------------------------------------------------------//
    // Optimization
    //-----------------------------------------------------------------------//

    #[tool(description = "Optimize the pack identified by `pack_key`. Pass OptimizerOptions as JSON.")]
    pub async fn optimize_pack_file(&self, params: Parameters<OptimizePackFileArgs>) -> Result<CallToolResult, McpError> {
        let options = parse_json(&params.0.options)?;
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

    #[tool(description = "Get all settings at once (bool, i32, f32, and string maps).")]
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

    #[tool(description = "Pack map tiles into the pack identified by `pack_key`. Pass Vec<(PathBuf, String)> as JSON for tiles.")]
    pub async fn pack_map(&self, params: Parameters<PackMapArgs>) -> Result<CallToolResult, McpError> {
        let tiles: Vec<(PathBuf, String)> = parse_json(&params.0.tiles)?;
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

    #[tool(description = "Export a RigidModel to glTF format. Pass RigidModel as JSON.")]
    pub async fn export_rigid_to_gltf(&self, params: Parameters<ExportRigidToGltfArgs>) -> Result<CallToolResult, McpError> {
        let rigid = parse_json(&params.0.rigid_model)?;
        send_and_respond!(self, "export_rigid_to_gltf", Command::ExportRigidToGltf(rigid, params.0.output_path))
    }

    #[tool(description = "Change the format of a ca_vp8 video file in the pack identified by `pack_key`. Pass SupportedFormats as JSON.")]
    pub async fn set_video_format(&self, params: Parameters<SetVideoFormatArgs>) -> Result<CallToolResult, McpError> {
        let format = parse_json(&params.0.format)?;
        send_and_respond!(self, "set_video_format", Command::SetVideoFormat(params.0.pack_key, params.0.path, format))
    }

    //-----------------------------------------------------------------------//
    // Multi-Pack Management
    //-----------------------------------------------------------------------//

    #[tool(description = "List all currently open packs with their keys and metadata. Use this to get valid pack_key values for other tools.")]
    pub async fn list_open_packs(&self) -> Result<CallToolResult, McpError> {
        send_and_respond!(self, "list_open_packs", Command::ListOpenPacks)
    }
}
