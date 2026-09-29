//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-session state, and the operations a session can run on it.
//!
//! [`SessionState`] owns everything a session works with: the open packs, the selected game,
//! its schema and its dependencies. The operations live in the submodules, split by domain,
//! and return plain results instead of protocol messages, so any protocol can call them.
//!
//! The legacy [`Command`](rpfm_ipc::messages::Command) protocol is translated into these
//! operations in [`crate::background_thread`].

use anyhow::Result;
use rayon::prelude::*;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::lua::{ASSEMBLY_KIT_SCRIPT_DOCS_PATH, LuaApi};

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::packs::PackSummary;
use rpfm_ipc::api::session::{DependenciesStatus, SessionStatus};
use rpfm_ipc::messages::OperationalMode;
use rpfm_ipc::settings_keys::ASSEMBLY_KIT_SUFFIX;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{Container, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile};
use rpfm_lib::games::{GameInfo, supported_games::{KEY_WARHAMMER_3, SupportedGames}};
use rpfm_lib::schema::Schema;

use rpfm_telemetry::info;

use crate::session::Session;
use crate::settings::{schemas_path, table_patches_path, Settings, SETTINGS};

mod dependencies;
mod diagnostics;
mod files;
mod packs;
mod schema;
mod search;
mod settings;
mod tables;
mod tools;

pub use self::files::{DecodedFile, PathsByPack};
use self::diagnostics::DiagnosticsResults;
pub use self::tables::{MergeOutcome, ReferenceLocation, RowLocation};
pub use self::tools::{MyModOptions, plugin_scripts};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Everything a session owns. One instance lives in each session's background loop.
pub struct SessionState {

    /// The session this state belongs to. Used to keep its list of open pack names in sync.
    session: Arc<Session>,

    /// Registry of every game RPFM supports.
    supported_games: SupportedGames,

    /// The game currently selected.
    game: GameInfo,

    /// Schema of the selected game, if it could be loaded.
    schema: Option<Schema>,

    /// If the game has already been selected once, so the first selection always rebuilds everything.
    first_game_change_done: bool,

    /// All open packs, keyed by their full file path (or a generated name for new/unsaved packs).
    packs: BTreeMap<String, Pack>,

    /// Operational mode (Normal or MyMod) of each open pack, keyed by the same pack key as `packs`.
    pack_modes: BTreeMap<String, OperationalMode>,

    /// Internal clipboard for copy/cut/paste operations.
    clipboard: Clipboard,

    /// Vanilla and parent files of the selected game.
    dependencies: Dependencies,

    /// Results of the last diagnostics check, if any.
    diagnostics: Option<DiagnosticsResults>,

    /// Lua scripting API of the selected game, built on first use. See [`cached_lua_api`].
    lua_api_cache: Option<LuaApiCache>,

    /// Snapshot of the settings, so the settings dialog can undo a "Restore Defaults".
    backup_settings: Settings,
}

/// Options for operations that write files of a pack to disk.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtractOptions {

    /// If tables keep their GUID when encoded.
    pub disable_uuid_regeneration: bool,

    /// If tables are written as TSV with the old column order, with keys first.
    pub tsv_keys_first: bool,
}

/// Options for saving a pack.
#[derive(Debug, Clone, Copy, Default)]
pub struct SaveOptions {

    /// If tables keep their GUID when encoded.
    pub disable_uuid_regeneration: bool,

    /// If packs of CA types can be saved.
    pub allow_editing_of_ca_packfiles: bool,

    /// If files that failed to decode are removed before saving.
    pub clean: bool,
}

/// Files copied or cut, waiting to be pasted.
#[derive(Debug, Default)]
struct Clipboard {
    entries: Vec<ClipboardEntry>,
    is_cut: bool,
}

/// A file in the clipboard. Only its path is stored: the file itself is cloned from its pack when pasting.
#[derive(Debug)]
struct ClipboardEntry {

    /// Path of the file in its pack.
    file_path: String,

    /// Part of `file_path` stripped when pasting, so the pasted path starts at the selected file or folder.
    base_path: String,

    /// Key of the pack the file comes from.
    source_pack_key: String,
}

/// Cached Lua scripting API: the docs path and dependencies build date it was built from, and the API itself.
type LuaApiCache = (PathBuf, u64, Option<LuaApi>);

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl SessionState {

    /// Creates the state of a new session, with Warhammer 3 selected and nothing loaded.
    ///
    /// # Arguments
    ///
    /// * `session` - The session this state belongs to.
    pub fn new(session: Arc<Session>) -> Self {
        let supported_games = SupportedGames::default();
        let game = supported_games.game(KEY_WARHAMMER_3)
            .expect("Warhammer 3 is always a supported game")
            .clone();

        Self {
            session,
            supported_games,
            game,
            schema: None,
            first_game_change_done: false,
            packs: BTreeMap::new(),
            pack_modes: BTreeMap::new(),
            clipboard: Clipboard::default(),
            dependencies: Dependencies::default(),
            diagnostics: None,
            lua_api_cache: None,
            backup_settings: SETTINGS.read().unwrap().clone(),
        }
    }

    /// Returns the game currently selected.
    pub fn game(&self) -> &GameInfo {
        &self.game
    }

    /// Returns if a schema is loaded for the selected game.
    pub fn is_schema_loaded(&self) -> bool {
        self.schema.is_some()
    }

    /// Returns the selected game, what's loaded for it, and the open packs.
    pub fn session_status(&self) -> SessionStatus {
        SessionStatus {
            game: self.game.key().to_owned(),
            schema_loaded: self.schema.is_some(),
            dependencies: DependenciesStatus {
                vanilla_loaded: self.dependencies.is_vanilla_data_loaded(false),
                assembly_kit_loaded: !self.dependencies.asskit_only_db_tables().is_empty(),
                parent_files: self.dependencies.parent_files().len(),
            },
            packs: self.packs.iter().map(|(key, pack)| pack_summary(key, pack)).collect(),
        }
    }
}

/// Returns the short description of an open pack.
fn pack_summary(key: &str, pack: &Pack) -> PackSummary {
    PackSummary {
        key: key.to_owned(),
        name: pack.disk_file_name(),
        path: pack.disk_file_path().to_owned(),
        pack_type: pack.pfh_file_type(),
        file_count: pack.files().len(),
    }
}

/// Returns the open pack with the provided key.
fn pack<'a>(packs: &'a BTreeMap<String, Pack>, pack_key: &str) -> Result<&'a Pack> {
    packs.get(pack_key).ok_or_else(|| ApiError::PackNotFound(pack_key.to_owned()).into())
}

/// Returns the open pack with the provided key, mutably.
fn pack_mut<'a>(packs: &'a mut BTreeMap<String, Pack>, pack_key: &str) -> Result<&'a mut Pack> {
    packs.get_mut(pack_key).ok_or_else(|| ApiError::PackNotFound(pack_key.to_owned()).into())
}

/// Returns the loaded schema, or an error if there is none.
fn loaded_schema(schema: &Option<Schema>) -> Result<&Schema> {
    schema.as_ref().ok_or_else(|| ApiError::SchemaNotLoaded.into())
}

/// Returns the loaded schema mutably, or an error if there is none.
fn loaded_schema_mut(schema: &mut Option<Schema>) -> Result<&mut Schema> {
    schema.as_mut().ok_or_else(|| ApiError::SchemaNotLoaded.into())
}

/// Returns the names of the parent packs of every open pack.
fn parent_pack_names(packs: &BTreeMap<String, Pack>) -> Vec<String> {
    packs.values()
        .flat_map(|pack| pack.dependencies().iter().map(|(_, name)| name.clone()))
        .collect()
}

/// Returns the data needed to encode files of a pack for the provided game.
fn encode_extra_data(game: &GameInfo, compression_format: CompressionFormat, disable_uuid_regeneration: bool) -> Option<EncodeableExtraData<'_>> {
    Some(EncodeableExtraData::new_from_game_info_and_settings(game, compression_format, disable_uuid_regeneration))
}

/// Decodes the DB and Loc files among `files`, so they're in memory for the diagnostics to work.
///
/// Files that fail to decode are left as they are. Does nothing if there is no schema.
fn decode_tables(files: &mut [&mut RFile], schema: &Option<Schema>) {
    if let Some(schema) = schema {
        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(Some(schema));
        let extra_data = Some(extra_data);

        files.par_iter_mut()
            .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
            .for_each(|file| {
                let _ = file.decode(&extra_data, true, false);
            });
    }
}

/// Returns the Lua scripting API of a game, rebuilding the cached one if it's outdated.
///
/// The API is rebuilt when the game or its Assembly Kit path change, or when the dependencies cache is regenerated.
///
/// # Arguments
///
/// * `cache` - Cached API, updated if outdated.
/// * `game` - Game whose API to return.
/// * `settings` - Settings, to find the game's Assembly Kit.
/// * `dependencies` - Dependencies cache with the vanilla scripts of the game.
///
/// # Returns
///
/// The API, or `None` if the game's Assembly Kit has no scripting docs.
fn cached_lua_api<'a>(cache: &'a mut Option<LuaApiCache>, game: &GameInfo, settings: &Settings, dependencies: &Dependencies) -> Option<&'a LuaApi> {
    let docs_path = settings.path_buf(&format!("{}{}", game.key(), ASSEMBLY_KIT_SUFFIX)).join(ASSEMBLY_KIT_SCRIPT_DOCS_PATH);
    let build_date = *dependencies.build_date();
    let outdated = cache.as_ref().is_none_or(|(path, date, _)| *path != docs_path || *date != build_date);

    if outdated {
        let api = match LuaApi::from_assembly_kit(&docs_path) {
            Ok(mut api) => {
                api.add_vanilla_scripts(dependencies);
                Some(api)
            }
            Err(error) => {
                info!("Lua scripting API not available, Lua scripts will only get their syntax checked: {error}");
                None
            }
        };

        *cache = Some((docs_path, build_date, api));
    }

    cache.as_ref().and_then(|(_, _, api)| api.as_ref())
}

/// Loads the schema of a game, re-decoding the tables of every open pack with it.
///
/// # Arguments
///
/// * `schema` - Schema to replace. Set to `None` if the game's schema can't be loaded.
/// * `packs` - Open packs whose tables need re-decoding.
/// * `game` - Game whose schema to load.
/// * `disable_uuid_regeneration` - If tables keep their GUID when encoded before the switch.
fn load_schema(schema: &mut Option<Schema>, packs: &mut BTreeMap<String, Pack>, game: &GameInfo, disable_uuid_regeneration: bool) {

    // Before loading the schema, make sure we don't have tables with definitions from the current schema.
    for pack in packs.values_mut() {
        let extra_data = encode_extra_data(game, pack.compression_format(), disable_uuid_regeneration);
        let mut files = pack.files_by_type_mut(&[FileType::DB]);

        files.par_iter_mut().for_each(|file| {
            let _ = file.encode(&extra_data, true, true, false);
        });
    }

    let schema_path = schemas_path().unwrap().join(game.schema_file_name());
    let local_patches_path = table_patches_path().unwrap().join(game.schema_file_name());
    *schema = Schema::load(&schema_path, Some(&local_patches_path)).ok();

    for pack in packs.values_mut() {
        decode_tables(&mut pack.files_by_type_mut(&[FileType::DB]), schema);
    }
}
