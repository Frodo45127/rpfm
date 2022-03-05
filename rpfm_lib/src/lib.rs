//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is the RPFM Lib, a lib to decode/encode any kind of PackFile CA has to offer, including his contents.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::cognitive_complexity,           // Disabled due to useless warnings.
    //clippy::cyclomatic_complexity,          // Disabled due to useless warnings.
    clippy::doc_markdown,                   // Disabled due to false positives on things that shouldn't be formatted in the docs as it says.
    clippy::too_many_arguments,             // Disabled because you never have enough arguments.
    clippy::type_complexity,                // Disabled temporarily because there are other things to do before rewriting the types it warns about.
    clippy::suspicious_else_formatting,     // Disabled because it's more or less useless.
    clippy::large_enum_variant              // Not useful in our case.
)]

use lazy_static::lazy_static;
use sentry::ClientInitGuard;

use std::sync::{Arc, RwLock};

use crate::games::{GameInfo, supported_games::{SupportedGames, KEY_THREE_KINGDOMS}};
use crate::logger::Logger;
use crate::packedfile::table::db::DB;
use crate::schema::patch::SchemaPatches;
use crate::schema::Schema;
use crate::settings::Settings;

pub mod assembly_kit;
pub mod common;
pub mod dependencies;
pub mod diagnostics;
pub mod games;
pub mod global_search;
pub mod logger;
pub mod packedfile;
pub mod packfile;
pub mod schema;
pub mod settings;
pub mod tips;
pub mod updater;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
    /// for "MyMod" folders.
    #[derive(Debug)]
    pub static ref SUPPORTED_GAMES: SupportedGames = SupportedGames::new();

    /// The current Settings and Shortcuts. To avoid reference and lock issues, this should be edited ONLY in the background thread.
    pub static ref SETTINGS: Arc<RwLock<Settings>> = Arc::new(RwLock::new(Settings::load(None).unwrap_or_else(|_|Settings::new())));

    /// The current GameSelected. If invalid, it uses 3K as default.
    pub static ref GAME_SELECTED: Arc<RwLock<&'static GameInfo>> = Arc::new(RwLock::new(
        match SUPPORTED_GAMES.get_supported_game_from_key(&SETTINGS.read().unwrap().settings_string["default_game"]) {
            Ok(game) => game,
            Err(_) => SUPPORTED_GAMES.get_supported_game_from_key(KEY_THREE_KINGDOMS).unwrap(),
        }
    ));

    /// Currently loaded schema.
    pub static ref SCHEMA: Arc<RwLock<Option<Schema>>> = Arc::new(RwLock::new(None));
    pub static ref SCHEMA_PATCHES: Arc<RwLock<SchemaPatches>> = Arc::new(RwLock::new(SchemaPatches::default()));

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    pub static ref SENTRY_GUARD: Arc<RwLock<ClientInitGuard>> = Arc::new(RwLock::new(Logger::init().unwrap()));
}

pub const DOCS_BASE_URL: &str = "https://frodo45127.github.io/rpfm/";
pub const PATREON_URL: &str = "https://www.patreon.com/RPFM";

// TODO: docs
