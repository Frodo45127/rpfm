//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the state of the session.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use rpfm_lib::files::db::DB;

use crate::helpers::DependenciesInfo;
use crate::settings::Settings;

use super::{default_true, Done, Request};
use super::packs::PackSummary;

/// Method of the notification sent to WebSocket clients right after they connect. Its params are a [`SessionConnected`].
pub const SESSION_CONNECTED_NOTIFICATION: &str = "session.connected";

/// The session a WebSocket client is connected to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConnected {

    /// ID of the session.
    pub session_id: u64,
}

/// `session.status`: returns the selected game, what's loaded for it, and the open packs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetSessionStatus {}

/// State of a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SessionStatus {

    /// Key of the selected game, like `warhammer_3`.
    pub game: String,

    /// If a schema is loaded for the selected game. Tables can't be read without one.
    pub schema_loaded: bool,

    /// What's loaded of the dependencies of the selected game.
    pub dependencies: DependenciesStatus,

    /// The open packs.
    pub packs: Vec<PackSummary>,
}

/// What's loaded of the dependencies of the selected game.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DependenciesStatus {

    /// If the vanilla files of the game are loaded.
    pub vanilla_loaded: bool,

    /// If the tables of the game's Assembly Kit are loaded.
    pub assembly_kit_loaded: bool,

    /// Amount of files loaded from the parent packs of the open packs.
    pub parent_files: usize,

    /// Why the dependencies couldn't be fully loaded, like a missing or outdated dependencies cache, if they couldn't.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Request for GetSessionStatus {
    const METHOD: &'static str = "session.status";
    type Response = SessionStatus;
}

/// `session.configure`: replaces the settings the session runs with.
///
/// Sessions start with the settings in the settings file. Clients owning their own settings send them
/// with this before anything else, and again whenever they change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configure {

    /// The settings.
    pub settings: Settings,
}

impl Request for Configure {
    const METHOD: &'static str = "session.configure";
    type Response = Done;
}

/// `session.disconnect`: tells the server the client is closing, so its WebSocket session is removed right away
/// instead of waiting for the timeout. Only for WebSocket clients.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disconnect {}

impl Request for Disconnect {
    const METHOD: &'static str = "session.disconnect";
    type Response = Done;
}

/// `session.set_game`: selects a game, loading its schema and, optionally, its dependencies. Runs as a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetGame {

    /// Key of the game, like `warhammer_3`.
    pub game: String,

    /// If the dependencies (vanilla files, Assembly Kit tables, parent packs) are loaded for the game.
    #[serde(default = "default_true")]
    pub rebuild_dependencies: bool,
}

/// `dependencies.generate_cache`: generates the dependencies cache of the selected game from its files and
/// Assembly Kit, and loads it. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenerateDependenciesCache {

    /// If tables in the game files are skipped when reading the Assembly Kit. Defaults to the server's setting.
    #[serde(default)]
    pub ignore_game_files_in_assembly_kit: Option<bool>,
}

/// `dependencies.rebuild`: reloads the dependencies of the selected game, like after changing the packs
/// the open packs depend on. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RebuildDependencies {

    /// If only the parent packs are reloaded, instead of the whole dependencies.
    #[serde(default)]
    pub only_parent_packs: bool,
}

impl Request for SetGame {
    const METHOD: &'static str = "session.set_game";
    type Response = SessionStatus;
    const IS_JOB: bool = true;
}

impl Request for GenerateDependenciesCache {
    const METHOD: &'static str = "dependencies.generate_cache";
    type Response = SessionStatus;
    const IS_JOB: bool = true;
}

impl Request for RebuildDependencies {
    const METHOD: &'static str = "dependencies.rebuild";
    type Response = SessionStatus;
    const IS_JOB: bool = true;
}

/// `dependencies.tables`: returns the tables of the game files, and the startpos, twad and CEO tables of the schema, with their version.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListDependencyTables {}

/// Tables new files of the selected game can be created for.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DependencyTables {

    /// Version of each table, by table name. Empty if the dependencies are not loaded.
    pub tables: BTreeMap<String, i32>,
}

impl Request for ListDependencyTables {
    const METHOD: &'static str = "dependencies.tables";
    type Response = DependencyTables;
}

/// `dependencies.info`: returns the files of the dependencies of the selected game.
///
/// Meant for clients showing the dependencies as a tree. Others should use `files.list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetDependenciesInfo {}

impl Request for GetDependenciesInfo {
    const METHOD: &'static str = "dependencies.info";
    type Response = DependenciesInfo;
}

/// `dependencies.table_data`: returns every decoded table of a type in the game files and the parent packs.
///
/// Meant for clients comparing whole tables. Others should use `table.rows`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetDependencyTableData {

    /// Name of the table, like `units_tables`.
    pub table_name: String,
}

/// Decoded tables of the dependencies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyTableData {

    /// The tables.
    pub tables: Vec<DB>,
}

impl Request for GetDependencyTableData {
    const METHOD: &'static str = "dependencies.table_data";
    type Response = DependencyTableData;
}
