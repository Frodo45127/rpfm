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
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::service::ServiceExt;
use rmcp::transport::stdio;
use rmcp::{tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use std::path::PathBuf;

use rpfm_ipc::helpers::DataSource;
use rpfm_ipc::messages::{Command, Response};

pub use crate::comms::CentralCommand;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone)]
pub struct RpfmServer {
    central: Arc<CentralCommand<Response>>,
    tool_router: ToolRouter<Self>,
}

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
    /// The path of the TSV file to export to.
    pub tsv_path: PathBuf,
    /// The path of the table to export.
    pub table_path: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct TsvImportArgs {
    /// The path of the TSV file to import from.
    pub tsv_path: PathBuf,
    /// The path of the table to import to.
    pub table_path: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

#[tool_handler]
impl rmcp::ServerHandler for RpfmServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This is a Model Context Protocol (MCP) server for RPFM (Rusted PackFile Manager). It allows you to interact with RFile and PackFiles using various tools.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tool_router]
impl RpfmServer {

    pub fn new(central: Arc<CentralCommand<Response>>) -> Self {
        Self {
            central,
            tool_router: RpfmServer::tool_router()
        }
    }

    #[tool(name = "call_command", description = "Call any IPC command directly.")]
    pub async fn call_command(&self, params: Parameters<CallCommandArgs>) -> Result<CallToolResult, McpError> {
        let command: Command = serde_json::from_str(&params.0.command).unwrap();

        let mut receiver = self.central.send(command);
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }

    #[tool(description = "Open one or more PackFiles.")]
    pub async fn open_packfiles(&self, params: Parameters<OpenPackfilesArgs>) -> Result<CallToolResult, McpError> {
        let mut receiver = self.central.send(Command::OpenPackFiles(params.0.paths));
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }

    #[tool(description = "Set the current game selected.")]
    pub async fn set_game_selected(&self, params: Parameters<SetGameSelectedArgs>) -> Result<CallToolResult, McpError> {
        let mut receiver = self.central.send(Command::SetGameSelected(params.0.game_name, params.0.rebuild_dependencies));
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }

    #[tool(description = "Save the currently open PackFile.")]
    pub async fn save_packfile(&self) -> Result<CallToolResult, McpError> {
        let mut receiver = self.central.send(Command::SavePackFile);
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }

    #[tool(description = "Export a table to a TSV file.")]
    pub async fn export_tsv(&self, params: Parameters<TsvExportArgs>) -> Result<CallToolResult, McpError> {
        let mut receiver = self.central.send(Command::ExportTSV(params.0.table_path, params.0.tsv_path, DataSource::PackFile));
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }

    #[tool(description = "Import a TSV file to a table.")]
    pub async fn import_tsv(&self, params: Parameters<TsvImportArgs>) -> Result<CallToolResult, McpError> {
        let mut receiver = self.central.send(Command::ImportTSV(params.0.table_path, params.0.tsv_path));
        let response = CentralCommand::recv(&mut receiver).await;

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&response).unwrap())]))
    }
}
