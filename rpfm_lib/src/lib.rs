//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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
    clippy::doc_markdown,                   // Disabled due to false positives on things that shouldn't be formated in the docs as it says.
    clippy::too_many_arguments,             // Disabled because you never have enough arguments.
    clippy::type_complexity,                // Disabled temporarily because there are other things to do before rewriting the types it warns about.
)]

use lazy_static::lazy_static;

use std::sync::{Arc, Mutex, RwLock};

use crate::games::{SupportedGames, get_supported_games_list};
use crate::packedfile::table::db::DB;
use crate::packfile::packedfile::PackedFile;
use crate::schema::Schema;
use crate::settings::Settings;

pub mod assembly_kit;
pub mod common;
pub mod config;
pub mod games;
pub mod global_search;
pub mod packedfile;
pub mod packfile;
pub mod schema;
pub mod settings;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
    /// for "MyMod" folders.
    #[derive(Debug)]
    pub static ref SUPPORTED_GAMES: SupportedGames = get_supported_games_list();

    /// The current Settings and Shortcuts. To avoid reference and lock issues, this should be edited ONLY in the background thread.
    pub static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load(None).unwrap_or_else(|_|Settings::new())));

    /// The current GameSelected. Same as the one above, only edited from the background thread.
    pub static ref GAME_SELECTED: Arc<RwLock<String>> = Arc::new(RwLock::new(SETTINGS.lock().unwrap().settings_string["default_game"].to_owned()));

    /// PackedFiles from the dependencies of the currently open PackFile.
    pub static ref DEPENDENCY_DATABASE: Mutex<Vec<PackedFile>> = Mutex::new(vec![]);

    /// DB Files from the Pak File of the current game. Only for dependency checking.
    pub static ref FAKE_DEPENDENCY_DATABASE: Mutex<Vec<DB>> = Mutex::new(vec![]);

    /// Currently loaded schema.
    pub static ref SCHEMA: Arc<RwLock<Option<Schema>>> = Arc::new(RwLock::new(None));
}

pub const DOCS_BASE_URL: &str = "https://frodo45127.github.io/rpfm/";
pub const PATREON_URL: &str = "https://www.patreon.com/RPFM";

// TODO: docs
