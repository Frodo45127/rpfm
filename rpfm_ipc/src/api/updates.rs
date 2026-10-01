//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to check for and download updates of RPFM and of the data it downloads.
//!
//! These methods don't touch the session's state, so they run without waiting for its other requests.
//! Schemas are updated with [`UpdateSchemas`](super::schema::UpdateSchemas), as they need reloading.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Done, Request};

/// Something that can be updated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpdateComponent {

    /// RPFM itself. Can only be checked: the program is updated by its UI.
    Program,

    /// The schemas with the table definitions of every game.
    Schemas,

    /// The Lua type definitions used for editor support in MyMods.
    LuaAutogen,

    /// The Assembly Kit data of Empire and Napoleon, which don't have one.
    OldAssemblyKit,

    /// The community translations of the Translation Hub.
    Translations,
}

/// `updates.check`: checks if there is an update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CheckUpdate {

    /// What to check.
    pub component: UpdateComponent,
}

/// State of the updates of something.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateStatus {

    /// If there is an update.
    pub available: bool,

    /// Details of the state, like `new_stable_update`, `no_update`, `no_local_files` or `diverged`.
    pub state: String,

    /// Version of the update, for program updates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// `updates.apply`: downloads an update. The program and the schemas can't be updated with this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApplyUpdate {

    /// What to update.
    pub component: UpdateComponent,
}

impl Request for CheckUpdate {
    const METHOD: &'static str = "updates.check";
    type Response = UpdateStatus;
}

impl Request for ApplyUpdate {
    const METHOD: &'static str = "updates.apply";
    type Response = Done;
}
