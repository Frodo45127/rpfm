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

use crate::settings::Settings;

use super::{default_true, Done, Request};
use super::packs::PackSummary;

/// `session.status`: returns the selected game, what's loaded for it, and the open packs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub struct Configure {

    /// The settings.
    pub settings: Settings,
}

impl Request for Configure {
    const METHOD: &'static str = "session.configure";
    type Response = Done;
}

/// `session.set_game`: selects a game, loading its schema and, optionally, its dependencies. Runs as a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub struct GenerateDependenciesCache {

    /// If tables in the game files are skipped when reading the Assembly Kit. Defaults to the server's setting.
    #[serde(default)]
    pub ignore_game_files_in_assembly_kit: Option<bool>,
}

/// `dependencies.rebuild`: reloads the dependencies of the selected game, like after changing the packs
/// the open packs depend on. Runs as a job.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
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
