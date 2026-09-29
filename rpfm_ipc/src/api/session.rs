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

use super::Request;
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
