//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the background loop.

Basically, this does the heavy load of the program.
!*/

use anyhow::{anyhow, Result};

use itertools::Itertools;
use open::that;
use rayon::prelude::*;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env::temp_dir;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Cursor, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::SystemTime;

use rpfm_extensions::dependencies::*;
use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::gltf::{gltf_from_rigid, save_gltf_to_disk};
use rpfm_extensions::optimizer::OptimizableContainer;
use rpfm_extensions::translator::PackTranslation;

use rpfm_ipc::helpers::*;
use rpfm_ipc::messages::OperationalMode;
use rpfm_ipc::settings_keys::*;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{animpack::AnimPack, Container, ContainerPath, db::DB, DecodeableExtraData, EncodeableExtraData, FileType, loc::Loc, pack::*, portrait_settings::PortraitSettings, RFile, RFileDecoded, table::{DecodedData, Table}, text::*};
use rpfm_lib::games::{GameInfo, LUA_REPO, LUA_BRANCH, LUA_REMOTE, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE, pfh_file_type::PFHFileType, supported_games::*, VanillaDBTableNameLogic};
use rpfm_lib::games::{TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE};
use rpfm_lib::integrations::{assembly_kit::*, git::*};
use rpfm_log::*;
use rpfm_lib::schema::*;
use rpfm_lib::utils::*;

use crate::*;
use crate::ceo_builder::{build_ceo_entries, build_ceo_post, get_trait_ceos};
use crate::comms::CentralCommand;
use crate::session::Session;
use crate::settings::*;
use crate::updater;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub const VANILLA_LOC_NAME: &str = "vanilla_english.tsv";
pub const VANILLA_FIXES_NAME: &str = "vanilla_fixes_";

const DEFAULT_PACK_STEM: &str = "new_pack";
const DEFAULT_PACK_EXT: &str = ".pack";

/// Derives a unique pack name for new (unsaved) packs. Appends a numeric suffix (_2, _3, etc.)
/// to the stem if the base name is already taken. Returns a name like "new_pack.pack", "new_pack_2.pack", etc.
fn derive_new_pack_name(existing_keys: &BTreeMap<String, Pack>) -> String {
    let base = format!("{}{}", DEFAULT_PACK_STEM, DEFAULT_PACK_EXT);
    if !existing_keys.contains_key(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{}_{}{}", DEFAULT_PACK_STEM, suffix, DEFAULT_PACK_EXT);
        if !existing_keys.contains_key(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Converts a path to its string representation for use as a pack key.
fn pack_key_from_path(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

/// Expand selected paths into file path entries for the clipboard.
///
/// Returns `(file_path, base_path, source_pack_key)` per file. Only paths are stored,
/// not the file data itself — the actual `RFile` is cloned from the source pack at paste time.
///
/// - For a selected file `a/b/c`, the base path is `a/b` (parent folder), so pasting gives just `c`.
/// - For a selected folder `a/b`, the base path is `a` (parent of folder), so pasting preserves `b/...`.
fn clipboard_entries_from_paths(pack: &Pack, paths: &[ContainerPath], pack_key: &str) -> Vec<(String, String, String)> {
    let mut result = Vec::new();
    for path in paths {
        let base_path = match path.path_raw().rfind('/') {
            Some(pos) => path.path_raw()[..pos].to_string(),
            None => String::new(),
        };
        for file in pack.files_by_paths(&[path.clone()], false) {
            result.push((file.path_in_container_raw().to_string(), base_path.clone(), pack_key.to_string()));
        }
    }
    result
}

/// Looks up a pack by key. If not found, sends a "Pack not found" error and returns `None`.
fn get_pack<'a>(packs: &'a BTreeMap<String, Pack>, pack_key: &str, sender: &UnboundedSender<Response>) -> Option<&'a Pack> {
    match packs.get(pack_key) {
        Some(pack) => Some(pack),
        None => {
            CentralCommand::send_back(sender, Response::Error(format!("Pack not found: {}", pack_key)));
            None
        }
    }
}

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub async fn background_loop(mut receiver: UnboundedReceiver<(UnboundedSender<Response>, Command)>, session: Arc<Session>) {

    //---------------------------------------------------------------------------------------//
    // Initializing stuff...
    //---------------------------------------------------------------------------------------//

    let supported_games = SupportedGames::default();
    let mut game = supported_games.game(KEY_WARHAMMER_3).unwrap();
    let mut schema = None;
    let mut first_game_change_done = false;

    // All open packs, keyed by their full file path (or a generated name for new/unsaved packs).
    let mut packs: BTreeMap<String, Pack> = BTreeMap::new();

    // Per-pack operational mode (Normal or MyMod). Keyed by the same pack key as `packs`.
    let mut pack_modes: BTreeMap<String, OperationalMode> = BTreeMap::new();

    // Internal clipboard for copy/cut/paste operations.
    let mut clipboard_entries: Vec<(String, String, String)> = Vec::new(); // (file_path, base_path, source_pack_key) per entry.
    let mut clipboard_is_cut: bool = false;

    // Preload the default game's dependencies.
    let mut dependencies = Arc::new(RwLock::new(Dependencies::default()));

    // Load settings from disk or use defaults.
    let _ = init_config_path();
    let mut settings = Settings::init(false).unwrap();
    let mut backup_settings = settings.clone();

    // Load all the tips we have.
    //let mut tips = if let Ok(tips) = Tips::load() { tips } else { Tips::default() };

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("Background Thread looping around…");
    'background_loop: while let Some((sender, response)) = receiver.recv().await {
        match response {

            // Command to close the thread.
            Command::Exit => break,

            // ClientDisconnecting is handled at the WebSocket level in main.rs.
            // If it reaches here, just acknowledge it (shouldn't normally happen).
            Command::ClientDisconnecting => {
                CentralCommand::send_back(&sender, Response::Success);
            }

            // When we want to check if there is an update available for RPFM...
            Command::CheckUpdates => {
                let sender = sender.clone();
                let settings = settings.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        updater::check_updates_rpfm(&settings)
                    }).await.unwrap();

                    match result {
                        Ok(response) => CentralCommand::send_back(&sender, Response::APIResponse(response)),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            Command::CheckSchemaUpdates => {
                git_update_check(sender, schemas_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
            }

            Command::CheckLuaAutogenUpdates => {
                git_update_check(sender, lua_autogen_base_path, LUA_REPO, LUA_BRANCH, LUA_REMOTE);
            }

            Command::CheckEmpireAndNapoleonAKUpdates => {
                git_update_check(sender, old_ak_files_path, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE);
            }

            Command::CheckTranslationsUpdates => {
                git_update_check(sender, translations_remote_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
            }

            // Close a specific pack by key.
            Command::ClosePack(pack_key) => {
                if packs.remove(&pack_key).is_some() {
                    pack_modes.remove(&pack_key);
                    session.remove_pack_name(&pack_key);
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key)));
                }
            }

            Command::CloseAllPacks => {
                for pack_key in packs.keys().cloned().collect::<Vec<_>>() {
                    session.remove_pack_name(&pack_key);
                }
                packs.clear();
                pack_modes.clear();
                CentralCommand::send_back(&sender, Response::Success);
            }

            // List all currently open packs.
            Command::ListOpenPacks => {
                let pack_list: Vec<(String, ContainerInfo)> = packs.iter()
                    .map(|(key, pack)| (key.clone(), ContainerInfo::from(pack)))
                    .collect();
                CentralCommand::send_back(&sender, Response::VecStringContainerInfo(pack_list));
            }

            // Create a new empty PackFile and insert into the map.
            Command::NewPack => {
                let pack_version = game.pfh_version_by_file_type(PFHFileType::Mod);
                let key = derive_new_pack_name(&packs);
                let mut pack = Pack::new_with_name_and_version(&key, pack_version);

                if let Some(version_number) = game.game_version_number(&settings.path_buf(game.key())) {
                    pack.set_game_version(version_number);
                }
                session.add_pack_name(&key);
                packs.insert(key.clone(), pack);
                pack_modes.insert(key.clone(), OperationalMode::Normal);
                CentralCommand::send_back(&sender, Response::String(key));
            }

            // Open one or more PackFiles, merge them, and insert into the map.
            Command::OpenPackFiles(paths) => {
                let key = if let Some(first_path) = paths.first() {
                    pack_key_from_path(first_path)
                } else {
                    format!("{}{}", DEFAULT_PACK_STEM, DEFAULT_PACK_EXT)
                };

                if packs.contains_key(&key) {
                    CentralCommand::send_back(&sender, Response::Error(format!(
                        "Pack '{}' is already open. Close it first if you want to reopen it.", key
                    )));
                } else {
                    match Pack::read_and_merge(&paths, game, settings.bool("use_lazy_loading"), false, false) {
                        Ok(mut pack) => {

                            // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                            if let Some(ref schema) = schema {
                                let mut decode_extra_data = DecodeableExtraData::default();
                                decode_extra_data.set_schema(Some(schema));
                                let extra_data = Some(decode_extra_data);

                                let mut files = pack.files_by_type_mut(&[FileType::DB, FileType::Loc]);
                                files.par_iter_mut().for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                });
                            }

                            session.add_pack_name(&key);

                            let info = ContainerInfo::from(&pack);
                            packs.insert(key.clone(), pack);
                            pack_modes.insert(key.clone(), OperationalMode::Normal);
                            CentralCommand::send_back(&sender, Response::StringContainerInfo(key, info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                }
            }

            // Load All CA PackFiles and insert into the map.
            Command::LoadAllCAPackFiles => {
                let key = "CA PackFiles".to_string();

                if packs.contains_key(&key) {
                    CentralCommand::send_back(&sender, Response::Error(format!(
                        "Pack '{}' is already open. Close it first if you want to reopen it.", key
                    )));
                } else {
                    match Pack::read_and_merge_ca_packs(game, &settings.path_buf(game.key())) {
                        Ok(mut pack) => {

                            // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                            if let Some(ref schema) = schema {
                                let mut decode_extra_data = DecodeableExtraData::default();
                                decode_extra_data.set_schema(Some(schema));
                                let extra_data = Some(decode_extra_data);

                                let mut files = pack.files_by_type_mut(&[FileType::DB, FileType::Loc]);
                                files.par_iter_mut().for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                });
                            }

                            session.add_pack_name(&key);

                            let info = ContainerInfo::from(&pack);
                            packs.insert(key.clone(), pack);
                            pack_modes.insert(key.clone(), OperationalMode::Normal);
                            CentralCommand::send_back(&sender, Response::StringContainerInfo(key, info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                }
            }

            // Save a specific pack to disk.
            Command::SavePack(pack_key) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));

                        let pack_type = *pack.header().pfh_file_type();
                        if !settings.bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.").to_string()));
                            continue;
                        }

                        match pack.save(None, game, &extra_data) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&*pack))),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Save a specific pack to a new path.
            Command::SavePackAs(pack_key, path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));

                        let pack_type = *pack.header().pfh_file_type();
                        if !settings.bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.").to_string()));
                            continue;
                        }

                        match pack.save(Some(&path), game, &extra_data) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&*pack))),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Clean and save a specific pack to a path.
            Command::CleanAndSavePackAs(pack_key, path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        pack.clean_undecoded();

                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                        match pack.save(Some(&path), game, &extra_data) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&*pack))),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Get the data of a specific pack needed to form the TreeView.
            Command::GetPackFileDataForTreeView(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => {
                        CentralCommand::send_back(&sender, Response::ContainerInfoVecRFileInfo((
                            From::from(pack),
                            pack.files().par_iter().map(|(_, file)| From::from(file)).collect(),
                        )));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Get the info of one PackedFile from a specific pack.
            Command::GetRFileInfo(pack_key, path) => {
                match packs.get(&pack_key) {
                    Some(pack) => {
                        CentralCommand::send_back(&sender, Response::OptionRFileInfo(
                            pack.files().get(&path).map(From::from)
                        ));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Get the info of more than one PackedFiles from a specific pack.
            Command::GetPackedFilesInfo(pack_key, paths) => {
                match packs.get(&pack_key) {
                    Some(pack) => {
                        let paths = paths.iter().map(|path| ContainerPath::File(path.to_owned())).collect::<Vec<_>>();
                        CentralCommand::send_back(&sender, Response::VecRFileInfo(
                            pack.files_by_paths(&paths, false).into_iter().map(From::from).collect()
                        ));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to launch a global search on a `PackFile`...
            Command::GlobalSearch(_pack_key, mut global_search) => {
                match schema {
                    Some(ref schema) => {
                        global_search.search(game, schema, &mut packs, &mut dependencies.write().unwrap(), &[]);
                        let packed_files_info = RFileInfo::info_from_global_search(&global_search, &packs);
                        CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), packed_files_info));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string())),
                }
            }

            Command::GetGameSelected => CentralCommand::send_back(&sender, Response::String(game.key().to_owned())),
            Command::SetGameSelected(game_key, rebuild_dependencies) => {
                info!("Setting game selected.");
                let game_changed = game.key() != game_key || !first_game_change_done;
                game = match supported_games.game(&game_key) {
                    Some(gi) => gi,
                    None => {
                        CentralCommand::send_back(&sender, Response::Error(anyhow!("The selected game is not supported!").to_string()));
                        continue;
                    }
                };

                // We need to make sure the compression format is valid for our game for all open packs.
                for pack in packs.values_mut() {
                    let current_cf = pack.compression_format();
                    if current_cf != CompressionFormat::None && !game.compression_formats_supported().contains(&current_cf) {
                        if let Some(new_cf) = game.compression_formats_supported().first() {
                            pack.set_compression_format(*new_cf, game);
                        } else {
                            pack.set_compression_format(CompressionFormat::None, game);
                        }
                    }
                }

                // Optimization: If we know we need to rebuild the whole dependencies, load them in another thread
                // while we load the schema. That way we can speed-up the entire game-switching process.
                //
                // While this is fast, the rust compiler doesn't like the fact that we're moving out the dependencies,
                // then moving them back in an if, so we need two branches of code, depending on if rebuild is true or not.
                //
                // Branch 1: dependencies rebuilt.
                // Load the new schema and re-decode tables in all open packs.
                load_schema(&mut schema, &mut packs, game, &settings);

                if rebuild_dependencies {
                    info!("Branch 1.");
                    // Collect dependencies from all open packs.
                    let pack_dependencies: Vec<_> = packs.values()
                        .flat_map(|pack| pack.dependencies().iter().map(|x| x.1.clone()))
                        .collect();
                    // Get settings values before spawning thread since settings can't be moved into closure
                    let game_path = settings.path_buf(game.key());
                    let secondary_path = settings.path_buf(SECONDARY_PATH);
                    let game_clone = game.clone();
                    let handle = thread::spawn(move || {
                        let file_path = dependencies_cache_path().unwrap().join(game_clone.dependencies_cache_file_name());
                        let file_path = if game_changed { Some(&*file_path) } else { None };
                        let _ = dependencies.write().unwrap().rebuild(&None, &pack_dependencies, file_path, &game_clone, &game_path, &secondary_path);
                        dependencies
                    });

                    // Get the dependencies that were loading in parallel and send their info to the UI.
                    dependencies = handle.join().unwrap();
                    let dependencies_info = DependenciesInfo::new(&dependencies.read().unwrap(), game.vanilla_db_table_name_logic());
                    info!("Sending dependencies info after game selected change.");
                    // Use compression format from the first pack, or None if no packs open.
                    let cf = packs.values().next().map(|p| p.compression_format()).unwrap_or(CompressionFormat::None);
                    CentralCommand::send_back(&sender, Response::CompressionFormatDependenciesInfo(cf, Some(dependencies_info)));

                    // Decode the dependencies tables while the UI does its own thing.
                    dependencies.write().unwrap().decode_tables(&schema);
                }

                // Branch 2: no dependencies rebuild.
                else {
                    info!("Branch 2.");
                    let cf = packs.values().next().map(|p| p.compression_format()).unwrap_or(CompressionFormat::None);
                    CentralCommand::send_back(&sender, Response::CompressionFormatDependenciesInfo(cf, None));
                };

                // For all open packs, change their id to match the one of the new `Game Selected`.
                for pack in packs.values_mut() {
                    if !pack.disk_file_path().is_empty() {
                        let pfh_file_type = *pack.header().pfh_file_type();
                        pack.header_mut().set_pfh_version(game.pfh_version_by_file_type(pfh_file_type));

                        if let Some(version_number) = game.game_version_number(&settings.path_buf(game.key())) {
                            pack.set_game_version(version_number);
                        }
                    }
                }

                if !first_game_change_done {
                    first_game_change_done = true;
                }

                info!("Switching game selected done.");
            }

            // In case we want to generate the dependencies cache for our Game Selected...
            Command::GenerateDependenciesCache => {
                let game_path = settings.path_buf(game.key());
                let ignore_game_files_in_ak = settings.bool("ignore_game_files_in_ak");
                let asskit_path = settings.assembly_kit_path(game).ok();

                if game_path.is_dir() {
                    match Dependencies::generate_dependencies_cache(&schema, game, &game_path, &asskit_path, ignore_game_files_in_ak) {
                        Ok(mut cache) => {
                            let dependencies_path = dependencies_cache_path().unwrap().join(game.dependencies_cache_file_name());
                            match cache.save(&dependencies_path) {
                                Ok(_) => {
                                    let secondary_path = settings.path_buf(SECONDARY_PATH);
                                    let pack_dependencies: Vec<_> = packs.values()
                                        .flat_map(|pack| pack.dependencies().iter().map(|x| x.1.clone()))
                                        .collect();
                                    let _ = dependencies.write().unwrap().rebuild(&schema, &pack_dependencies, Some(&dependencies_path), game, &game_path, &secondary_path);
                                    let dependencies_info = DependenciesInfo::new(&dependencies.read().unwrap(), game.vanilla_db_table_name_logic());
                                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                                },
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                            }
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Game Path not configured. Go to <i>'PackFile/Settings'</i> and configure it.").to_string()));
                }
            }

            // In case we want to update the Schema for our Game Selected...
            Command::UpdateCurrentSchemaFromAssKit => {
                let ignore_game_files_in_ak = settings.bool("ignore_game_files_in_ak");

                if let Some(ref mut schema) = schema {
                    match settings.assembly_kit_path(game) {
                        Ok(asskit_path) => {
                            let schema_path = schemas_path().unwrap().join(game.schema_file_name());

                            let dependencies = dependencies.read().unwrap();
                            if let Ok(mut tables_to_check) = dependencies.db_and_loc_data(true, false, true, false) {

                                // If there are packs open, also add the packs' tables to it. That way we can treat some special tables, like starpos tables.
                                for pack in packs.values() {
                                    if !pack.disk_file_path().is_empty() {
                                        tables_to_check.append(&mut pack.files_by_type(&[FileType::DB]));
                                    }
                                }

                                // Split the tables to check by table name.
                                let mut tables_to_check_split: HashMap<String, Vec<DB>> = HashMap::new();
                                for table_to_check in tables_to_check {
                                    if let Ok(RFileDecoded::DB(table)) = table_to_check.decoded() {
                                        match tables_to_check_split.get_mut(table.table_name()) {
                                            Some(tables) => {

                                                // Merge tables of the same name and version, so we got more chances of loc data being found.
                                                match tables.iter_mut().find(|x| x.definition().version() == table.definition().version()) {
                                                    Some(db_source) => *db_source = DB::merge(&[db_source, table]).unwrap(),
                                                    None => tables.push((table.clone()).clone()),
                                                }
                                            }
                                            None => {
                                                tables_to_check_split.insert(table.table_name().to_owned(), vec![table.clone()]);
                                            }
                                        }
                                    }
                                }

                                let tables_to_skip = if ignore_game_files_in_ak {
                                    dependencies.vanilla_loose_tables().keys().chain(dependencies.vanilla_tables().keys()).map(|x| &**x).collect::<Vec<_>>()
                                } else {
                                    vec![]
                                };

                                match update_schema_from_raw_files(schema, game, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split) {
                                    Ok(possible_loc_fields) => {

                                        // NOTE: This deletes all loc fields first, so we need to get the loc fields AGAIN after this from the TExc_LocalisableFields.xml, if said file exists and it's readable.
                                        // That's why it does the update again, to re-populate the loc fields list with the ones not bruteforced. It's ineficient, but gets the job done.
                                        // Use the open packs for bruteforce, or None if no packs open.
                                        let local_packs = if packs.is_empty() { None } else { Some(&packs) };
                                        if dependencies.bruteforce_loc_key_order(schema, possible_loc_fields, local_packs, None).is_ok() {

                                            // Note: this shows the list of "missing" fields.
                                            let _ = update_schema_from_raw_files(schema, game, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split);

                                            // This generates the automatic patches in the schema (like ".png are files" kinda patches).
                                            if dependencies.generate_automatic_patches(schema, &packs).is_ok() {

                                                // Fix for old file relative paths using incorrect separators.
                                                schema.definitions_mut().par_iter_mut().for_each(|x| {
                                                    x.1.iter_mut().for_each(|y| {
                                                        y.fields_mut().iter_mut().for_each(|z| {
                                                            if let Some(path) = z.filename_relative_path(None) {
                                                                if path.len() == 1 && path[0].contains(",") {
                                                                    let new_paths = path[0].split(',').map(|x| x.trim()).join(";");
                                                                    z.set_filename_relative_path(Some(new_paths));
                                                                }
                                                            }
                                                        });
                                                    });
                                                });

                                                match schema.save(&schemas_path().unwrap().join(game.schema_file_name())) {
                                                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                                                }
                                            } else {
                                                CentralCommand::send_back(&sender, Response::Success)
                                            }
                                        } else {
                                            CentralCommand::send_back(&sender, Response::Success)
                                        }
                                    },
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                                }
                            }
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                }
            }

            // In case we want to optimize our PackFile...
            Command::OptimizePackFile(pack_key, options) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        if let Some(ref schema) = schema {
                            match pack.optimize(None, &mut dependencies.write().unwrap(), schema, game, &options) {
                                Ok((paths_to_delete, paths_to_add)) => CentralCommand::send_back(&sender, Response::HashSetStringHashSetString(paths_to_delete, paths_to_add)),
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to Patch the SiegeAI of a PackFile...
            Command::PatchSiegeAI(pack_key) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        match pack.patch_siege_ai() {
                            Ok(result) => CentralCommand::send_back(&sender, Response::StringVecContainerPath(result.0, result.1)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string()))
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to change the PackFile's Type...
            Command::SetPackFileType(pack_key, new_type) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        pack.set_pfh_file_type(new_type);
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to change the "Include Last Modified Date" setting of the PackFile...
            Command::ChangeIndexIncludesTimestamp(pack_key, state) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let mut bitmask = pack.bitmask();
                        bitmask.set(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, state);
                        pack.set_bitmask(bitmask);
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to compress/decompress the PackedFiles of the currently open PackFile...
            Command::ChangeCompressionFormat(pack_key, cf) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        CentralCommand::send_back(&sender, Response::CompressionFormat(pack.set_compression_format(cf, game)));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to get the path of the currently open `PackFile`.
            Command::GetPackFilePath(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::PathBuf(PathBuf::from(pack.disk_file_path()))),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to get the Dependency PackFiles of our PackFile...
            Command::GetDependencyPackFilesList(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::VecBoolString(pack.dependencies().to_vec())),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to set the Dependency PackFiles of our PackFile...
            Command::SetDependencyPackFilesList(pack_key, dep_packs) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        pack.set_dependencies(dep_packs);
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to check if there is a Dependency Database loaded...
            Command::IsThereADependencyDatabase(include_asskit) => {
                let are_dependencies_loaded = dependencies.read().unwrap().is_vanilla_data_loaded(include_asskit);
                CentralCommand::send_back(&sender, Response::Bool(are_dependencies_loaded))
            },

            // In case we want to create a PackedFile from scratch...
            Command::NewPackedFile(pack_key, path, new_packed_file) => {
                let decoded = match new_packed_file {
                    NewFile::AnimPack(_) => {
                        let file = AnimPack::default();
                        RFileDecoded::AnimPack(file)
                    },
                    NewFile::DB(_, table, version) => {
                        if let Some(ref schema) = schema {
                            match schema.definition_by_name_and_version(&table, version) {
                                Some(definition) => {
                                    let patches = schema.patches_for_table(&table);
                                    let file = DB::new(definition, patches, &table);
                                    RFileDecoded::DB(file)
                                }
                                None => {
                                    CentralCommand::send_back(&sender, Response::Error(format!("No definitions found for the table `{}`, version `{}` in the currently loaded schema.", table, version)));
                                    continue;
                                }
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string()));
                            continue;
                        }
                    },
                    NewFile::Loc(_) => {
                        let file = Loc::new();
                        RFileDecoded::Loc(file)
                    }
                    NewFile::PortraitSettings(_, version, entries) => {
                        let mut file = PortraitSettings::default();
                        file.set_version(version);

                        if !entries.is_empty() {

                            let mut dependencies = dependencies.write().unwrap();
                            let mut vanilla_files = dependencies.files_by_types_mut(&[FileType::PortraitSettings], true, true);
                            let vanilla_files_decoded = vanilla_files.iter_mut()
                                .filter_map(|(_, file)| file.decode(&None, false, true).ok().flatten())
                                .filter_map(|file| if let RFileDecoded::PortraitSettings(file) = file { Some(file) } else { None })
                                .collect::<Vec<_>>();

                            let vanilla_values = vanilla_files_decoded.iter()
                                .flat_map(|file| file.entries())
                                .map(|entry| (entry.id(), entry))
                                .collect::<HashMap<_,_>>();

                            for (from_id, to_id) in entries {
                                if let Some(from_entry) = vanilla_values.get(&from_id) {
                                    let mut new_entry = (*from_entry).clone();
                                    new_entry.set_id(to_id);
                                    file.entries_mut().push(new_entry);
                                }
                            }
                        }

                        RFileDecoded::PortraitSettings(file)
                    },
                    NewFile::Text(_, text_type) => {
                        let mut file = Text::default();
                        file.set_format(text_type);
                        RFileDecoded::Text(file)
                    },

                    NewFile::VMD(_) => {
                        let mut file = Text::default();
                        file.set_format(TextFormat::Xml);
                        RFileDecoded::VMD(file)
                    },

                    NewFile::WSModel(_) => {
                        let mut file = Text::default();
                        file.set_format(TextFormat::Xml);
                        RFileDecoded::WSModel(file)
                    },
                };
                let file = RFile::new_from_decoded(&decoded, 0, &path);
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        match pack.insert(file) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // When we want to add one or more PackedFiles to our PackFile.
            Command::AddPackedFiles(pack_key, source_paths, destination_paths, paths_to_ignore) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let mut added_paths = vec![];
                        let mut it_broke = None;

                        let paths = source_paths.iter().zip(destination_paths.iter()).collect::<Vec<(&PathBuf, &ContainerPath)>>();
                        for (source_path, destination_path) in paths {

                            // Skip ignored paths.
                            if let Some(ref paths_to_ignore) = paths_to_ignore {
                                if paths_to_ignore.iter().any(|x| source_path.starts_with(x)) {
                                    continue;
                                }
                            }

                            match destination_path {
                                ContainerPath::File(destination_path) => {
                                    match pack.insert_file(source_path, destination_path, &schema) {
                                        Ok(path) => if let Some(path) = path {
                                            added_paths.push(path);
                                        },
                                        Err(error) => it_broke = Some(error),
                                    }
                                },

                                // TODO: See what should we do with the ignored paths.
                                ContainerPath::Folder(destination_path) => {
                                    match pack.insert_folder(source_path, destination_path, &None, &schema, settings.bool("include_base_folder_on_add_from_folder")) {
                                        Ok(mut paths) => added_paths.append(&mut paths),
                                        Err(error) => it_broke = Some(error),
                                    }
                                },
                            }
                        }

                        CentralCommand::send_back(&sender, Response::VecContainerPathOptionString(added_paths.to_vec(), it_broke.map(|e| e.to_string())));

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = schema {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = pack.files_by_paths_mut(&added_paths, false);
                            files.par_iter_mut()
                                .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                                .for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                }
                            );
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to move stuff from one PackFile to another...
            Command::AddPackedFilesFromPackFile(target_key, source_key, paths) => {
                // First, clone files from the source pack.
                let files = match packs.get(&source_key) {
                    Some(source_pack) => {
                        source_pack.files_by_paths(&paths, false)
                            .into_iter()
                            .cloned()
                            .collect::<Vec<RFile>>()
                    }
                    None => {
                        CentralCommand::send_back(&sender, Response::Error(format!("Source pack not found: {}", source_key)));
                        continue;
                    }
                };

                // Then, insert the cloned files into the target pack.
                match packs.get_mut(&target_key) {
                    Some(target_pack) => {
                        let mut added_paths = Vec::with_capacity(files.len());
                        for file in files {
                            if let Ok(Some(path)) = target_pack.insert(file) {
                                added_paths.push(path);
                            }
                        }

                        CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths.to_vec()));

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = schema {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = target_pack.files_by_paths_mut(&added_paths, false);
                            files.par_iter_mut()
                                .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                                .for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                }
                            );
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Target pack not found: {}", target_key))),
                }
            }

            // In case we want to move stuff from our PackFile to an Animpack...
            Command::AddPackedFilesFromPackFileToAnimpack(pack_key, anim_pack_path, paths) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                let files = pack.files_by_paths(&paths, false)
                    .into_iter()
                    .map(|file| {
                        let mut file = file.clone();
                        let _ = file.load();
                        file
                    })
                    .collect::<Vec<RFile>>();

                match pack.files_mut().get_mut(&anim_pack_path) {
                    Some(file) => {

                        // Try to decode it using lazy_load if enabled.
                        let extra_data = DecodeableExtraData::default();
                        //extra_data.set_lazy_load(SETTINGS.read().unwrap().bool("use_lazy_loading"));
                        let _ = file.decode(&Some(extra_data), true, false);

                        match file.decoded_mut() {
                            Ok(decoded) => match decoded {
                                RFileDecoded::AnimPack(anim_pack) => {
                                    let mut paths = Vec::with_capacity(files.len());
                                    for file in files {
                                        if let Ok(Some(path)) = anim_pack.insert(file) {
                                            paths.push(path);
                                        }
                                    }

                                    CentralCommand::send_back(&sender, Response::VecContainerPath(paths.to_vec()));
                                }
                                _ => CentralCommand::send_back(&sender, Response::Error(format!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded)))),
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(format!("Failed to decode the file at the following path: {}", anim_pack_path))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("File not found in the Pack: {}.", anim_pack_path))),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to move stuff from an Animpack to our PackFile...
            Command::AddPackedFilesFromAnimpack(pack_key, data_source, anim_pack_path, paths) => {
                let mut dependencies = dependencies.write().unwrap();
                let anim_pack_file = match data_source {
                    DataSource::PackFile => packs.get_mut(&pack_key).and_then(|pack| pack.files_mut().get_mut(&anim_pack_path)),
                    DataSource::GameFiles => dependencies.file_mut(&anim_pack_path, true, false).ok(),
                    DataSource::ParentFiles => dependencies.file_mut(&anim_pack_path, false, true).ok(),
                    DataSource::AssKitFiles |
                    DataSource::ExternalFile => unreachable!("add_files_to_animpack"),
                };

                let files = match anim_pack_file {
                    Some(file) => {

                        // Try to decode it using lazy_load if enabled.
                        let extra_data = DecodeableExtraData::default();
                        //extra_data.set_lazy_load(SETTINGS.read().unwrap().bool("use_lazy_loading"));
                        let _ = file.decode(&Some(extra_data), true, false);

                        match file.decoded_mut() {
                            Ok(decoded) => match decoded {
                                RFileDecoded::AnimPack(anim_pack) => anim_pack.files_by_paths(&paths, false).into_iter().cloned().collect::<Vec<RFile>>(),
                                _ => {
                                    CentralCommand::send_back(&sender, Response::Error(format!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded))));
                                    continue;
                                },
                            }
                            _ => {
                                CentralCommand::send_back(&sender, Response::Error(format!("Failed to decode the file at the following path: {}", anim_pack_path)));
                                continue;
                            },
                        }
                    }
                    None => {
                        CentralCommand::send_back(&sender, Response::Error(format!("The file with the path {} doesn't exists on the open Pack.", anim_pack_path)));
                        continue;
                    }
                };

                let result_paths = files.iter().map(|file| file.path_in_container()).collect::<Vec<_>>();
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        for mut file in files {
                            let _ = file.guess_file_type();
                            let _ = pack.insert(file);
                        }
                        CentralCommand::send_back(&sender, Response::VecContainerPath(result_paths));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to delete files from an Animpack...
            Command::DeleteFromAnimpack(pack_key, anim_pack_path, paths) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                match pack.files_mut().get_mut(&anim_pack_path) {
                    Some(file) => {

                        // Try to decode it using lazy_load if enabled.
                        let extra_data = DecodeableExtraData::default();
                        //extra_data.set_lazy_load(SETTINGS.read().unwrap().bool("use_lazy_loading"));
                        let _ = file.decode(&Some(extra_data), true, false);

                        match file.decoded_mut() {
                            Ok(decoded) => match decoded {
                                RFileDecoded::AnimPack(anim_pack) => {
                                    for path in paths {
                                        anim_pack.remove(&path);
                                    }

                                    CentralCommand::send_back(&sender, Response::Success);
                                }
                                _ => CentralCommand::send_back(&sender, Response::Error(format!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded)))),
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(format!("Failed to decode the file at the following path: {}", anim_pack_path))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("File not found in the Pack: {}.", anim_pack_path))),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to decode a RigidModel PackedFile...
            Command::DecodePackedFile(pack_key, path, data_source) => {
                info!("Trying to decode a file. Path: {}", &path);
                info!("Trying to decode a file. Data Source: {}", &data_source);

                match data_source {
                    DataSource::PackFile => {
                        match packs.get_mut(&pack_key) {
                            Some(pack) => {
                                if path == RESERVED_NAME_NOTES {
                                    let mut note = Text::default();
                                    note.set_format(TextFormat::Markdown);
                                    note.set_contents(pack.notes().pack_notes().to_owned());
                                    CentralCommand::send_back(&sender, Response::Text(note));
                                }

                                else {

                                    // Find the PackedFile we want and send back the response.
                                    match pack.files_mut().get_mut(&path) {
                                        Some(file) => decode_and_send_file(file, &sender, &settings, game, &schema),
                                        None => CentralCommand::send_back(&sender, Response::Error(format!("The file with the path {} hasn't been found on this Pack.", path))),
                                    }
                                }
                            }
                            None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                        }
                    }

                    DataSource::ParentFiles => {
                        match dependencies.write().unwrap().file_mut(&path, false, true) {
                            Ok(file) => decode_and_send_file(file, &sender, &settings, game, &schema),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }

                    DataSource::GameFiles => {
                        match dependencies.write().unwrap().file_mut(&path, true, false) {
                            Ok(file) => decode_and_send_file(file, &sender, &settings, game, &schema),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }

                    DataSource::AssKitFiles => {
                        let path_split = path.split('/').collect::<Vec<_>>();
                        if path_split.len() > 2 {
                            match dependencies.read().unwrap().asskit_only_db_tables().get(path_split[1]) {
                                Some(db) => CentralCommand::send_back(&sender, Response::DBRFileInfo(db.clone(), RFileInfo::default())),
                                None => CentralCommand::send_back(&sender, Response::Error(format!("Table {} not found on Assembly Kit files.", path))),
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Error(format!("Path {} doesn't contain an identifiable table name.", path)));
                        }
                    }

                    DataSource::ExternalFile => {
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                }
            }

            // When we want to save a PackedFile from the view....
            Command::SavePackedFileFromView(pack_key, path, file_decoded) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        if path == RESERVED_NAME_NOTES {
                            if let RFileDecoded::Text(data) = file_decoded {
                                pack.notes_mut().set_pack_notes(data.contents().to_owned());
                            }
                        }
                        else if let Some(file) = pack.files_mut().get_mut(&path) {
                            if let Err(error) = file.set_decoded(file_decoded) {
                                CentralCommand::send_back(&sender, Response::Error(error.to_string()));
                                continue;
                            }
                        }
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to delete PackedFiles from a PackFile...
            Command::DeletePackedFiles(pack_key, paths) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths.iter().flat_map(|path| pack.remove(path)).collect())),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Copy files to the internal clipboard.
            Command::CopyPackedFiles(paths_by_pack) => {
                clipboard_entries.clear();
                for (pack_key, paths) in &paths_by_pack {
                    if let Some(pack) = packs.get(pack_key) {
                        clipboard_entries.extend(clipboard_entries_from_paths(pack, paths, pack_key));
                    }
                }
                clipboard_is_cut = false;
                CentralCommand::send_back(&sender, Response::Success);
            }

            // Cut files to the internal clipboard.
            Command::CutPackedFiles(paths_by_pack) => {
                clipboard_entries.clear();
                for (pack_key, paths) in &paths_by_pack {
                    if let Some(pack) = packs.get(pack_key) {
                        clipboard_entries.extend(clipboard_entries_from_paths(pack, paths, pack_key));
                    }
                }
                clipboard_is_cut = true;
                CentralCommand::send_back(&sender, Response::Success);
            }

            // Paste files from the internal clipboard into a pack.
            Command::PastePackedFiles(target_key, destination_path) => {
                if clipboard_entries.is_empty() {
                    CentralCommand::send_back(&sender, Response::Error("Clipboard is empty.".to_string()));
                } else {

                    // Clone files from their source packs and compute their new paths.
                    // We collect all cloned files first so we don't hold borrows while mutating.
                    let mut files_to_insert: Vec<RFile> = Vec::with_capacity(clipboard_entries.len());
                    for (file_path, base_path, source_key) in &clipboard_entries {
                        if let Some(source_pack) = packs.get(source_key) {
                            let path_as_container = ContainerPath::File(file_path.clone());
                            let found = source_pack.files_by_paths(&[path_as_container], false);
                            if let Some(file) = found.first() {
                                let mut new_file = (*file).clone();

                                // Compute relative path by stripping this file's base path.
                                let relative_path = if !base_path.is_empty() && file_path.starts_with(base_path) {
                                    file_path[base_path.len()..].trim_start_matches('/')
                                } else {
                                    file_path
                                };
                                let new_path = if destination_path.is_empty() {
                                    relative_path.to_string()
                                } else {
                                    format!("{}/{}", destination_path.trim_end_matches('/'), relative_path)
                                };
                                new_file.set_path_in_container_raw(&new_path);
                                files_to_insert.push(new_file);
                            }
                        }
                    }

                    // If it was a cut operation, delete the files from their respective source packs.
                    let mut cut_deleted_by_pack: BTreeMap<String, Vec<ContainerPath>> = BTreeMap::new();
                    if clipboard_is_cut {
                        for (file_path, _, source_key) in &clipboard_entries {
                            if let Some(source_pack) = packs.get_mut(source_key) {
                                let removed = source_pack.remove(&ContainerPath::File(file_path.clone()));
                                cut_deleted_by_pack.entry(source_key.clone()).or_default().extend(removed);
                            }
                        }
                    }

                    // Insert the cloned files into the target pack.
                    match packs.get_mut(&target_key) {
                        Some(target_pack) => {
                            let mut added_paths = Vec::with_capacity(files_to_insert.len());
                            for new_file in files_to_insert {
                                if let Ok(Some(path)) = target_pack.insert(new_file) {
                                    added_paths.push(path);
                                }
                            }

                            // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                            if let Some(ref schema) = schema {
                                let mut decode_extra_data = DecodeableExtraData::default();
                                decode_extra_data.set_schema(Some(schema));
                                let extra_data = Some(decode_extra_data);

                                let mut files = target_pack.files_by_paths_mut(&added_paths, false);
                                files.par_iter_mut()
                                    .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                                    .for_each(|file| {
                                        let _ = file.decode(&extra_data, true, false);
                                    });
                            }

                            CentralCommand::send_back(&sender, Response::VecContainerPathBTreeMapStringVecContainerPath(added_paths, cut_deleted_by_pack));

                            // Clear clipboard after a cut-paste operation.
                            if clipboard_is_cut {
                                clipboard_entries.clear();
                                clipboard_is_cut = false;
                            }
                        }
                        None => CentralCommand::send_back(&sender, Response::Error(format!("Target pack not found: {}", target_key))),
                    }
                }
            }

            // Duplicate files in-place within the same pack.
            Command::DuplicatePackedFiles(pack_key, paths) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        // First, clone all the files we want to duplicate.
                        let files_to_dup: Vec<RFile> = pack.files_by_paths(&paths, false)
                            .into_iter()
                            .cloned()
                            .collect();

                        let mut added_paths = Vec::with_capacity(files_to_dup.len());
                        for file in files_to_dup {
                            let old_path = file.path_in_container_raw().to_string();

                            // Generate a new name with a numeric suffix: "name.ext" -> "name1.ext", "name1.ext" -> "name2.ext", etc.
                            let new_path = if let Some(dot_pos) = old_path.rfind('.') {
                                let (base, ext) = old_path.split_at(dot_pos);

                                // Find and increment any trailing number in the base name.
                                let base_trimmed = base.trim_end_matches(|c: char| c.is_ascii_digit());
                                let suffix_str = &base[base_trimmed.len()..];
                                let mut counter = suffix_str.parse::<u32>().unwrap_or(0) + 1;

                                // Keep incrementing until we find a name that doesn't exist.
                                loop {
                                    let candidate = format!("{}{}{}", base_trimmed, counter, ext);
                                    if !pack.has_file(&candidate) {
                                        break candidate;
                                    }
                                    counter += 1;
                                }
                            } else {
                                // No extension, just append a number.
                                let base_trimmed = old_path.trim_end_matches(|c: char| c.is_ascii_digit());
                                let suffix_str = &old_path[base_trimmed.len()..];
                                let mut counter = suffix_str.parse::<u32>().unwrap_or(0) + 1;

                                loop {
                                    let candidate = format!("{}{}", base_trimmed, counter);
                                    if !pack.has_file(&candidate) {
                                        break candidate;
                                    }
                                    counter += 1;
                                }
                            };

                            let mut new_file = file;
                            new_file.set_path_in_container_raw(&new_path);

                            if let Ok(Some(path)) = pack.insert(new_file) {
                                added_paths.push(path);
                            }
                        }

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = schema {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = pack.files_by_paths_mut(&added_paths, false);
                            files.par_iter_mut()
                                .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                                .for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                });
                        }

                        CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to extract PackedFiles from a PackFile...
            Command::ExtractPackedFiles(pack_key, container_paths, path, extract_tables_to_tsv) => {
                let schema = if extract_tables_to_tsv { &schema } else { &None };
                let mut errors = 0;

                // Pack extraction.
                if let Some(container_paths) = container_paths.get(&DataSource::PackFile) {
                    match packs.get_mut(&pack_key) {
                        Some(pack) => {
                            let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                            let mut extracted_paths = vec![];

                            for container_path in container_paths {
                                match pack.extract(container_path.clone(), &path, true, schema, false, settings.bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                                    Ok(mut extracted_path) => extracted_paths.append(&mut extracted_path),
                                    Err(_) => {
                                        //error!("Error extracting {}: {}", container_path.path_raw(), error);
                                        errors += 1;
                                    },
                                }
                            }

                            if errors == 0 {
                                CentralCommand::send_back(&sender, Response::StringVecPathBuf(tr("files_extracted_success"), extracted_paths));
                            } else {
                                CentralCommand::send_back(&sender, Response::Error(format!("There were {} errors while extracting.", errors)));
                            }
                        }
                        None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                    }
                }

                // Dependencies extraction.
                else {

                    let dependencies = dependencies.read().unwrap();
                    let mut game_files = if let Some(container_paths) = container_paths.get(&DataSource::GameFiles) {
                        dependencies.files_by_path(container_paths, true, false, false)
                    } else {
                        HashMap::new()
                    };
                    let parent_files = if let Some(container_paths) = container_paths.get(&DataSource::ParentFiles) {
                        dependencies.files_by_path(container_paths, false, true, false)
                    } else {
                        HashMap::new()
                    };

                    game_files.extend(parent_files);

                    let mut pack = Pack::default();
                    let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                    let mut extracted_paths = vec![];
                    for (path_raw, file) in game_files {
                        if pack.insert(file.clone()).is_err() {
                            errors += 1;
                            continue;
                        }

                        let container_path = ContainerPath::File(path_raw);
                        match pack.extract(container_path, &path, true, schema, false, settings.bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                            Ok(mut extracted_path) => extracted_paths.append(&mut extracted_path),
                            Err(_) => errors += 1,
                        }
                    }

                    if errors == 0 {
                        CentralCommand::send_back(&sender, Response::StringVecPathBuf(tr("files_extracted_success"), extracted_paths));
                    } else {
                        CentralCommand::send_back(&sender, Response::Error(format!("There were {} errors while extracting.", errors)));
                    }
                }
            }

            // In case we want to rename one or more files/folders...
            Command::RenamePackedFiles(pack_key, renaming_data) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        match pack.move_paths(&renaming_data) {
                            Ok(data) => CentralCommand::send_back(&sender, Response::VecContainerPathContainerPath(data)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to know if a Folder exists, knowing his path...
            Command::FolderExists(pack_key, path) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::Bool(pack.has_folder(&path))),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to know if PackedFile exists, knowing his path...
            Command::PackedFileExists(pack_key, path) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::Bool(pack.has_file(&path))),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to get the list of tables in the dependency database...
            Command::GetTableListFromDependencyPackFile => {
                let dependencies = dependencies.read().unwrap();
                CentralCommand::send_back(&sender, Response::VecString(dependencies.vanilla_loose_tables().keys().chain(dependencies.vanilla_tables().keys()).map(|x| x.to_owned()).collect()))
            },
            Command::GetCustomTableList => match &schema {
                Some(schema) => {
                    let tables = schema.definitions().par_iter().filter(|(key, defintions)|
                        !defintions.is_empty() && (
                            key.starts_with("start_pos_") ||
                            key.starts_with("twad_")
                        )
                    ).map(|(key, _)| key.to_owned()).collect::<Vec<_>>();
                    CentralCommand::send_back(&sender, Response::VecString(tables));
                }
                None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()))
            },

            Command::LocalArtSetIds(_pack_key) => {
                CentralCommand::send_back(&sender, Response::HashSetString(dependencies.read().unwrap().db_values_from_table_name_and_column_name(Some(&packs), "campaign_character_arts_tables", "art_set_id", false, false)));
            }

            // TODO: This needs to use a list pulled from portrait settings files, not from a table.
            Command::DependenciesArtSetIds => CentralCommand::send_back(&sender, Response::HashSetString(dependencies.read().unwrap().db_values_from_table_name_and_column_name(None, "campaign_character_arts_tables", "art_set_id", true, true))),

            // In case we want to get the version of an specific table from the dependency database...
            Command::GetTableVersionFromDependencyPackFile(table_name) => {
                if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                    match dependencies.read().unwrap().db_version(&table_name) {
                        Some(version) => CentralCommand::send_back(&sender, Response::I32(version)),
                        None => {

                            // If the table is one of the starpos tables, we need to return the latest version of the table, even if it's not in the game files.
                            if table_name.starts_with("start_pos_") || table_name.starts_with("twad_") || table_name.starts_with("ceo") { // TEMP FIX FOR NOW 
                                match &schema {
                                    Some(schema) => {
                                        match schema.definitions_by_table_name(&table_name) {
                                            Some(definitions) => {
                                                if definitions.is_empty() {
                                                    CentralCommand::send_back(&sender, Response::Error("There are no definitions for this specific table.".to_string()));
                                                } else {
                                                    CentralCommand::send_back(&sender, Response::I32(*definitions.first().unwrap().version()));
                                                }
                                            }
                                            None => CentralCommand::send_back(&sender, Response::Error("There are no definitions for this specific table.".to_string())),
                                        }
                                    }
                                    None => CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string().to_string()))
                                }
                            } else {
                                CentralCommand::send_back(&sender, Response::Error("Table not found in the game files.".to_string()))
                            }
                        },
                    }
                } else { CentralCommand::send_back(&sender, Response::Error("Dependencies cache needs to be regenerated before this.".to_string().to_string())); }
            }

            Command::GetTableDefinitionFromDependencyPackFile(table_name) => {
                if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                    if let Some(ref schema) = schema {
                        if let Some(version) = dependencies.read().unwrap().db_version(&table_name) {
                            if let Some(definition) = schema.definition_by_name_and_version(&table_name, version) {
                                CentralCommand::send_back(&sender, Response::Definition(definition.clone()));
                            } else { CentralCommand::send_back(&sender, Response::Error(format!("No definition found for table {}.", table_name).to_string())); }
                        } else { CentralCommand::send_back(&sender, Response::Error(format!("Table version not found in dependencies for table {}.", table_name).to_string())); }
                    } else { CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string().to_string())); }
                } else { CentralCommand::send_back(&sender, Response::Error("Dependencies cache needs to be regenerated before this.".to_string().to_string())); }
            }

            // In case we want to merge DB or Loc Tables from a PackFile...
            Command::MergeFiles(pack_key, paths, merged_path, delete_source_files) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let files_to_merge = pack.files_by_paths(&paths, false);
                        match RFile::merge(&files_to_merge, &merged_path) {
                            Ok(file) => {
                                let _ = pack.insert(file);

                                // Make sure to only delete the files if they're not the destination file.
                                if delete_source_files {
                                    paths.iter()
                                        .filter(|path| merged_path != path.path_raw())
                                        .for_each(|path| { pack.remove(path); });
                                }

                                CentralCommand::send_back(&sender, Response::String(merged_path.to_string()));
                            },
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to update a table...
            Command::UpdateTable(pack_key, path) => {
                let path = path.path_raw();
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                if let Some(rfile) = pack.file_mut(path, false) {
                    if let Ok(decoded) = rfile.decoded_mut() {
                        match dependencies.write().unwrap().update_db(decoded) {
                            Ok((old_version, new_version, fields_deleted, fields_added)) => CentralCommand::send_back(&sender, Response::I32I32VecStringVecString(old_version, new_version, fields_deleted, fields_added)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path undecoded: {}", path).to_string())); }
                } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found in the open Pack: {}", path).to_string())); }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceMatches(_pack_key, mut global_search, matches) => {
                if let Some(ref schema) = schema {
                    match global_search.replace(game, schema, &mut packs, &mut dependencies.write().unwrap(), &matches) {
                        Ok(paths) => {
                            let files_info = paths.iter().flat_map(|path| {
                                packs.values().flat_map(|pack| pack.files_by_path(path, false).iter().map(|file| RFileInfo::from(*file)).collect::<Vec<RFileInfo>>()).collect::<Vec<_>>()
                            }).collect();
                            CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), files_info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string()));
                }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceAll(_pack_key, mut global_search) => {
                if let Some(ref schema) = schema {
                    match global_search.replace_all(game, schema, &mut packs, &mut dependencies.write().unwrap()) {
                        Ok(paths) => {
                            let files_info = paths.iter().flat_map(|path| {
                                packs.values().flat_map(|pack| pack.files_by_path(path, false).iter().map(|file| RFileInfo::from(*file)).collect::<Vec<RFileInfo>>()).collect::<Vec<_>>()
                            }).collect();
                            CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), files_info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string()));
                }
            }

            // In case we want to get the reference data for a definition...
            Command::GetReferenceDataFromDefinition(_pack_key, table_name, definition, force_local_ref_generation) => {
                let mut reference_data = HashMap::new();

                // Only generate the cache references if we don't already have them generated.
                if let Some(ref schema) = schema {
                    if dependencies.read().unwrap().local_tables_references().get(&table_name).is_none() || force_local_ref_generation {
                        dependencies.write().unwrap().generate_local_definition_references(schema, &table_name, &definition);
                    }

                    reference_data = dependencies.read().unwrap().db_reference_data(schema, &packs, &table_name, &definition, &None);
                }

                CentralCommand::send_back(&sender, Response::HashMapI32TableReferences(reference_data));
            }

            // In case we want to change the format of a ca_vp8 video...
            Command::SetVideoFormat(pack_key, path, format) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                match pack.files_mut().get_mut(&path) {
                    Some(ref mut rfile) => {
                        match rfile.decoded_mut() {
                            Ok(data) => {
                                if let RFileDecoded::Video(ref mut data) = data {
                                    data.set_format(format);
                                    CentralCommand::send_back(&sender, Response::Success);
                                } else {
                                    CentralCommand::send_back(&sender, Response::Error("The file is not a video.".to_string()));
                                }
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error("This Pack doesn't exists as a file in the disk.".to_string())),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // In case we want to save an schema to disk...
            Command::SaveSchema(mut schema_new) => {
                match schema_new.save(&schemas_path().unwrap().join(game.schema_file_name())) {
                    Ok(_) => {
                        schema = Some(schema_new);
                        CentralCommand::send_back(&sender, Response::Success);
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to clean the cache of one or more PackedFiles...
            Command::CleanCache(pack_key, paths) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let cf = pack.compression_format();
                        let mut files = pack.files_by_paths_mut(&paths, false);
                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                        files.iter_mut().for_each(|file| {
                            let _ = file.encode(&extra_data, true, true, false);
                        });
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to export a PackedFile as a TSV file...
            Command::ExportTSV(pack_key, internal_path, external_path, data_source) => {
                let mut dependencies = dependencies.write().unwrap();
                match &schema {
                    Some(ref schema) => {
                        let file = match data_source {
                            DataSource::PackFile => packs.get_mut(&pack_key).and_then(|pack| pack.file_mut(&internal_path, false)),
                            DataSource::ParentFiles => dependencies.file_mut(&internal_path, false, true).ok(),
                            DataSource::GameFiles => dependencies.file_mut(&internal_path, true, false).ok(),
                            DataSource::AssKitFiles => {
                                CentralCommand::send_back(&sender, Response::Error("Exporting a TSV from the Assembly Kit is not yet supported.".to_string()));
                                continue;
                            },
                            DataSource::ExternalFile => {
                                CentralCommand::send_back(&sender, Response::Error("Exporting a TSV from a external file is not yet supported.".to_string()));
                                continue;
                            },
                        };
                        match file {
                            Some(file) => match file.tsv_export_to_path(&external_path, schema, settings.bool("tables_use_old_column_order_for_tsv")) {
                                Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                            }
                            None => CentralCommand::send_back(&sender, Response::Error(format!("File with the following path not found in the Pack: {}", internal_path).to_string())),
                        }
                    },
                    None => CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string().to_string())),
                }
            }

            // In case we want to import a TSV as a PackedFile...
            // TODO: This is... unreliable at best, can break stuff at worst. Replace the set_decoded with proper type checking.
            Command::ImportTSV(pack_key, internal_path, external_path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                match pack.file_mut(&internal_path, false) {
                    Some(file) => {
                        match RFile::tsv_import_from_path(&external_path, &schema) {
                            Ok(imported) => {
                                let decoded = imported.decoded().unwrap();
                                file.set_decoded(decoded.clone()).unwrap();
                                CentralCommand::send_back(&sender, Response::RFileDecoded(decoded.clone()))
                            },
                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path not found in the Pack: {}", internal_path).to_string())),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to open a PackFile's location in the file manager...
            Command::OpenContainingFolder(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => {

                // If the path exists, try to open it. If not, throw an error.
                let mut path_str = pack.disk_file_path().to_owned();

                // Remove canonicalization, as it breaks the open thingy.
                if path_str.starts_with("//?/") || path_str.starts_with("\\\\?\\") {
                    path_str = path_str[4..].to_string();
                }

                let mut path = PathBuf::from(path_str);
                if path.exists() {
                    path.pop();
                    let _ = open::that(&path);
                    CentralCommand::send_back(&sender, Response::Success);
                }
                else {
                    CentralCommand::send_back(&sender, Response::Error("This Pack doesn't exists as a file in the disk.".to_string()));
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            // When we want to open a PackedFile in a external program...
            Command::OpenPackedFileInExternalProgram(pack_key, data_source, path) => {
                match data_source {
                    DataSource::PackFile => {
                        match packs.get_mut(&pack_key) {
                            Some(pack) => {
                        let folder = temp_dir().join(format!("rpfm_{}", pack.disk_file_name()));
                        let cf = pack.compression_format();
                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                        match pack.extract(path.clone(), &folder, true, &schema, false, settings.bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                            Ok(extracted_path) => {
                                let _ = that(&extracted_path[0]);
                                CentralCommand::send_back(&sender, Response::PathBuf(extracted_path[0].to_owned()));
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                            }
                            None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                        }
                    }
                    _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("Opening dependencies files in external programs is not yet supported.").to_string())),
                }
            }

            // When we want to save a PackedFile from the external view....
            Command::SavePackedFileFromExternalView(pack_key, path, external_path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                match pack.file_mut(&path, false) {
                    Some(file) => match file.encode_from_external_data(&schema, &external_path) {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found").to_string())),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // When we want to update our schemas...
            Command::UpdateSchemas => {

                // Run the git operation on the blocking thread pool to avoid blocking the async runtime.
                let git_result = tokio::task::spawn_blocking(|| {
                    match schemas_path() {
                        Ok(local_path) => {
                            let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
                            git_integration.update_repo().map(|_| ()).map_err(|e| anyhow::anyhow!(e.to_string()))
                        },
                        Err(error) => Err(error),
                    }
                }).await.unwrap();

                // Post-download state mutation must stay in the main loop (accesses local mutable state).
                match git_result {
                    Ok(_) => {
                        let schema_path = schemas_path().unwrap().join(game.schema_file_name());
                        let patches_path = table_patches_path().unwrap().join(game.schema_file_name());

                        // Encode the decoded tables with the old schema, then re-decode them with the new one for all open packs.
                        for pack in packs.values_mut() {
                            let cf = pack.compression_format();
                            let mut tables = pack.files_by_type_mut(&[FileType::DB]);
                            let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                            tables.par_iter_mut().for_each(|x| { let _ = x.encode(&extra_data, true, true, false); });
                        }

                        schema = Schema::load(&schema_path, Some(&patches_path)).ok();

                        for pack in packs.values_mut() {
                            let mut extra_data = DecodeableExtraData::default();
                            extra_data.set_schema(schema.as_ref());
                            let extra_data = Some(extra_data);

                            let mut tables = pack.files_by_type_mut(&[FileType::DB]);
                            tables.par_iter_mut().for_each(|x| {
                                let _ = x.decode(&extra_data, true, false);
                            });
                        }

                        // Then rebuild the dependencies stuff.
                        if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                            let game_path = settings.path_buf(game.key());
                            let secondary_path = settings.path_buf(SECONDARY_PATH);
                            let dependencies_file_path = dependencies_cache_path().unwrap().join(game.dependencies_cache_file_name());
                            let pack_dependencies: Vec<_> = packs.values()
                                .flat_map(|pack| pack.dependencies().iter().map(|x| x.1.clone()))
                                .collect();

                            match dependencies.write().unwrap().rebuild(&schema, &pack_dependencies, Some(&*dependencies_file_path), game, &game_path, &secondary_path) {
                                Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                Err(_) => CentralCommand::send_back(&sender, Response::Error("Schema updated, but dependencies cache rebuilding failed. You may need to regenerate it.".to_string())),
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Success)
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // When we want to update our lua setup...
            Command::UpdateLuaAutogen => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match lua_autogen_base_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, LUA_REPO, LUA_BRANCH, LUA_REMOTE);
                                git_integration.update_repo().map(|_| ()).map_err(|e| e.into())
                            },
                            Err(error) => Err(error),
                        }
                    }).await.unwrap();

                    match result {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            // When we want to update our program...
            Command::UpdateMainProgram => {
                let sender = sender.clone();
                let settings = settings.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::updater::update_main_program(&settings)
                    }).await.unwrap();

                    match result {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            // When we want to update our program...
            Command::TriggerBackupAutosave(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => {
                        let folder = backup_autosave_path().unwrap().join(pack.disk_file_name());
                        let _ = DirBuilder::new().recursive(true).create(&folder);

                        let game_path = settings.path_buf(game.key());
                        let ca_paths = game.ca_packs_paths(&game_path)
                            .unwrap_or_default()
                            .iter()
                            .map(|path| path.to_string_lossy().replace('\\', "/"))
                            .collect::<Vec<_>>();

                        let pack_disable_autosaves = pack.settings().setting_bool("disable_autosaves")
                            .unwrap_or(&true);

                        let pack_type = pack.pfh_file_type();
                        let pack_path = pack.disk_file_path().replace('\\', "/");

                        // Do not autosave vanilla packs, packs with autosave disabled, or non-mod or movie packs.
                        if folder.is_dir() &&
                            !pack_disable_autosaves &&
                            (pack_type == PFHFileType::Mod || pack_type == PFHFileType::Movie) &&
                            (ca_paths.is_empty() || !ca_paths.contains(&pack_path))
                        {
                            let date = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                            let new_name = format!("{date}.pack");
                            let new_path = folder.join(new_name);
                            let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, pack.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                            let _ = pack.clone().save(Some(&new_path), game, &extra_data);

                            // If we have more than the limit, delete the older one.
                            if let Ok(files) = files_in_folder_from_newest_to_oldest(&folder) {
                                let max_files = settings.i32("autosave_amount") as usize;
                                for (index, file) in files.iter().enumerate() {
                                    if index >= max_files {
                                        let _ = std::fs::remove_file(file);
                                    }
                                }
                            }
                        }
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // In case we want to perform a diagnostics check...
            Command::DiagnosticsCheck(diagnostics_ignored, check_ak_only_refs) => {
                let game_path = settings.path_buf(game.key());
                let mut diagnostics = Diagnostics::default();
                *diagnostics.diagnostics_ignored_mut() = diagnostics_ignored;

                if let Some(ref schema) = schema {
                    diagnostics.check(&mut packs, &mut dependencies.write().unwrap(), schema, game, &game_path, &[], check_ak_only_refs);
                }

                info!("Checking diagnostics: done.");

                CentralCommand::send_back(&sender, Response::Diagnostics(diagnostics));
            }

            Command::DiagnosticsUpdate(mut diagnostics, path_types, check_ak_only_refs) => {
                let game_path = settings.path_buf(game.key());

                if let Some(ref schema) = schema {
                    diagnostics.check(&mut packs, &mut dependencies.write().unwrap(), schema, game, &game_path, &path_types, check_ak_only_refs);
                }

                info!("Checking diagnostics (update): done.");

                CentralCommand::send_back(&sender, Response::Diagnostics(diagnostics));
            }

            // In case we want to get the open PackFile's Settings...
            Command::GetPackSettings(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::PackSettings(pack.settings().clone())),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }
            Command::SetPackSettings(pack_key, pack_settings) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        pack.set_settings(pack_settings);
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            Command::GetMissingDefinitions(pack_key) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        // Test to see if every DB Table can be decoded. This is slow and only useful when
                        // a new patch lands and you want to know what tables you need to decode.
                        let mut counter = 0;
                        let mut table_list = String::new();
                        if let Some(ref schema) = schema {
                            let mut extra_data = DecodeableExtraData::default();
                            extra_data.set_schema(Some(schema));
                            let extra_data = Some(extra_data);

                            let mut files = pack.files_by_type_mut(&[FileType::DB]);
                            files.sort_by_key(|file| file.path_in_container_raw().to_lowercase());

                            for file in files {
                                if file.decode(&extra_data, false, false).is_err() && file.load().is_ok() {
                                    if let Ok(raw_data) = file.cached() {
                                        let mut reader = Cursor::new(raw_data);
                                        if let Ok((_, _, _, entry_count)) = DB::read_header(&mut reader) {
                                            if entry_count > 0 {
                                                counter += 1;
                                                table_list.push_str(&format!("{}, {:?}\n", counter, file.path_in_container_raw()))
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Try to save the file. And I mean "try". Someone seems to love crashing here...
                        let path = exe_path().join("missing_table_definitions.txt");

                        if let Ok(file) = File::create(path) {
                            let mut file = BufWriter::new(file);
                            let _ = file.write_all(table_list.as_bytes());
                        }
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            // Ignore errors for now.
            Command::RebuildDependencies(rebuild_only_current_mod_dependencies) => {
                if schema.is_some() {
                    let game_path = settings.path_buf(game.key());
                    let dependencies_file_path = dependencies_cache_path().unwrap().join(game.dependencies_cache_file_name());
                    let file_path = if !rebuild_only_current_mod_dependencies { Some(&*dependencies_file_path) } else { None };
                    let pack_dependencies: Vec<_> = packs.values()
                        .flat_map(|pack| pack.dependencies().iter().map(|x| x.1.clone()))
                        .collect();

                    let secondary_path = settings.path_buf(SECONDARY_PATH);
                    let _ = dependencies.write().unwrap().rebuild(&schema, &pack_dependencies, file_path, game, &game_path, &secondary_path);
                    let dependencies_info = DependenciesInfo::new(&dependencies.read().unwrap(), game.vanilla_db_table_name_logic());
                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                }
            },

            Command::CascadeEdition(pack_key, table_name, definition, changes) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let edited_paths = if let Some(ref schema) = schema {
                            changes.iter().flat_map(|(field, value_before, value_after)| {
                                DB::cascade_edition(pack, schema, &table_name, field, &definition, value_before, value_after)
                            }).collect::<Vec<_>>()
                        } else { vec![] };

                        let packed_files_info = pack.files_by_paths(&edited_paths, false).into_par_iter().map(From::from).collect();
                        CentralCommand::send_back(&sender, Response::VecContainerPathVecRFileInfo(edited_paths, packed_files_info));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::GetTablesByTableName(pack_key, table_name) => {
                match packs.get(&pack_key) {
                    Some(pack) => {
                        let path = ContainerPath::Folder(format!("db/{table_name}/"));
                        let files = pack.files_by_type_and_paths(&[FileType::DB], &[path], true);
                        let paths = files.iter()
                            .map(|x| x.path_in_container_raw().to_owned())
                            .collect::<Vec<_>>();

                        CentralCommand::send_back(&sender, Response::VecString(paths));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::AddKeysToKeyDeletes(pack_key, table_file_name, key_table_name, keys) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                let path = ContainerPath::File(format!("db/{KEY_DELETES_TABLE_NAME}/{table_file_name}"));
                let mut files = pack.files_by_type_and_paths_mut(&[FileType::DB], &[path], true);

                let mut cont_path = None;
                if let Some(file) = files.first_mut() {
                    if let Ok(RFileDecoded::DB(db)) = file.decoded_mut() {
                        for key in &keys {
                            let row = vec![
                                DecodedData::StringU8(key.to_owned()),
                                DecodedData::StringU8(key_table_name.to_owned()),
                            ];

                            db.data_mut().push(row);
                        }

                        cont_path = Some(file.path_in_container());
                    }
                }

                CentralCommand::send_back(&sender, Response::OptionContainerPath(cont_path));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            Command::GoToDefinition(pack_key, ref_table, mut ref_column, ref_data) => {
                let table_name = format!("{ref_table}_tables");
                let table_folder = format!("db/{table_name}");
                let Some(pack) = get_pack(&packs, &pack_key, &sender) else { continue 'background_loop; };
                let packed_files = pack.files_by_path(&ContainerPath::Folder(table_folder.to_owned()), true);
                let mut found = false;
                for packed_file in &packed_files {
                    if let Ok(RFileDecoded::DB(data)) = packed_file.decoded() {

                        // If the column is a loc column, we need to search in the first key column instead.
                        if data.definition().localised_fields().iter().any(|x| x.name() == ref_column) {
                            if let Some(first_key_index) = data.definition().localised_key_order().first() {
                                if let Some(first_key_field) = data.definition().fields_processed().get(*first_key_index as usize) {
                                    ref_column = first_key_field.name().to_owned();
                                }
                            }
                        }

                        if let Some((column_index, row_index)) = data.table().rows_containing_data(&ref_column, &ref_data[0]) {
                            CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::PackFile, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.read().unwrap().db_data(&table_name, false, true) {
                        for packed_file in &packed_files {
                            if let Ok(RFileDecoded::DB(data)) = packed_file.decoded() {

                                // If the column is a loc column, we need to search in the first key column instead.
                                if data.definition().localised_fields().iter().any(|x| x.name() == ref_column) {
                                    if let Some(first_key_index) = data.definition().localised_key_order().first() {
                                        if let Some(first_key_field) = data.definition().fields_processed().get(*first_key_index as usize) {
                                            ref_column = first_key_field.name().to_owned();
                                        }
                                    }
                                }

                                if let Some((column_index, row_index)) = data.table().rows_containing_data(&ref_column, &ref_data[0]) {
                                    CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::ParentFiles, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.read().unwrap().db_data(&table_name, true, false) {
                        for packed_file in &packed_files {
                            if let Ok(RFileDecoded::DB(data)) = packed_file.decoded() {

                                // If the column is a loc column, we need to search in the first key column instead.
                                if data.definition().localised_fields().iter().any(|x| x.name() == ref_column) {
                                    if let Some(first_key_index) = data.definition().localised_key_order().first() {
                                        if let Some(first_key_field) = data.definition().fields_processed().get(*first_key_index as usize) {
                                            ref_column = first_key_field.name().to_owned();
                                        }
                                    }
                                }

                                if let Some((column_index, row_index)) = data.table().rows_containing_data(&ref_column, &ref_data[0]) {
                                    CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::GameFiles, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    if let Some(data) = dependencies.read().unwrap().asskit_only_db_tables().get(&table_name) {

                        // If the column is a loc column, we need to search in the first key column instead.
                        if data.definition().localised_fields().iter().any(|x| x.name() == ref_column) {
                            if let Some(first_key_index) = data.definition().localised_key_order().first() {
                                if let Some(first_key_field) = data.definition().fields_processed().get(*first_key_index as usize) {
                                    ref_column = first_key_field.name().to_owned();
                                }
                            }
                        }

                        if let Some((column_index, row_index)) = data.table().rows_containing_data(&ref_column, &ref_data[0]) {
                            let path = format!("{}/ak_data", &table_folder);
                            CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::AssKitFiles, path, column_index, row_index[0]));
                            found = true;
                        }
                    }
                }

                if !found {
                    CentralCommand::send_back(&sender, Response::Error(tr("source_data_for_field_not_found")));
                }
            },

            Command::SearchReferences(pack_key, reference_map, value) => {
                let paths = reference_map.keys().map(|x| ContainerPath::Folder(format!("db/{x}"))).collect::<Vec<ContainerPath>>();
                let Some(pack) = get_pack(&packs, &pack_key, &sender) else { continue 'background_loop; };
                let files = pack.files_by_paths(&paths, true);

                let mut references: Vec<(DataSource, String, String, usize, usize)> = vec![];

                // Pass for local tables.
                for (table_name, columns) in &reference_map {
                    for file in &files {
                        if file.db_table_name_from_path().unwrap() == table_name {
                            if let Ok(RFileDecoded::DB(data)) = file.decoded() {
                                for column_name in columns {
                                    if let Some((column_index, row_indexes)) = data.table().rows_containing_data(column_name, &value) {
                                        for row_index in &row_indexes {
                                            references.push((DataSource::PackFile, file.path_in_container_raw().to_owned(), column_name.to_owned(), column_index, *row_index));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Pass for parent tables.
                for (table_name, columns) in &reference_map {
                        if let Ok(tables) = dependencies.read().unwrap().db_data(table_name, false, true) {
                        references.append(&mut tables.par_iter().map(|table| {
                            let mut references = vec![];
                            if let Ok(RFileDecoded::DB(data)) = table.decoded() {
                                for column_name in columns {
                                    if let Some((column_index, row_indexes)) = data.table().rows_containing_data(column_name, &value) {
                                        for row_index in &row_indexes {
                                            references.push((DataSource::ParentFiles, table.path_in_container_raw().to_owned(), column_name.to_owned(), column_index, *row_index));
                                        }
                                    }
                                }
                            }

                            references
                        }).flatten().collect());
                    }
                }

                // Pass for vanilla tables.
                for (table_name, columns) in &reference_map {
                    if let Ok(tables) = dependencies.read().unwrap().db_data(table_name, true, false) {
                        references.append(&mut tables.par_iter().map(|table| {
                            let mut references = vec![];
                            if let Ok(RFileDecoded::DB(data)) = table.decoded() {
                                for column_name in columns {
                                    if let Some((column_index, row_indexes)) = data.table().rows_containing_data(column_name, &value) {
                                        for row_index in &row_indexes {
                                            references.push((DataSource::GameFiles, table.path_in_container_raw().to_owned(), column_name.to_owned(), column_index, *row_index));
                                        }
                                    }
                                }
                            }

                            references
                        }).flatten().collect());
                    }
                }

                CentralCommand::send_back(&sender, Response::VecDataSourceStringStringUsizeUsize(references));
            },

            Command::GoToLoc(pack_key, loc_key) => {
                let Some(pack) = get_pack(&packs, &pack_key, &sender) else { continue 'background_loop; };
                let packed_files = pack.files_by_type(&[FileType::Loc]);
                let mut found = false;
                for packed_file in &packed_files {
                    if let Ok(RFileDecoded::Loc(data)) = packed_file.decoded() {
                        if let Some((column_index, row_index)) = data.table().rows_containing_data("key", &loc_key) {
                            CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::PackFile, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.read().unwrap().loc_data(false, true) {
                        for packed_file in &packed_files {
                            if let Ok(RFileDecoded::Loc(data)) = packed_file.decoded() {
                                if let Some((column_index, row_index)) = data.table().rows_containing_data("key", &loc_key) {
                                    CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::ParentFiles, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.read().unwrap().loc_data(true, false) {
                        for packed_file in &packed_files {
                            if let Ok(RFileDecoded::Loc(data)) = packed_file.decoded() {
                                if let Some((column_index, row_index)) = data.table().rows_containing_data("key", &loc_key) {
                                    CentralCommand::send_back(&sender, Response::DataSourceStringUsizeUsize(DataSource::GameFiles, packed_file.path_in_container_raw().to_owned(), column_index, row_index[0]));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    CentralCommand::send_back(&sender, Response::Error(tr("loc_key_not_found")));
                }
            },

            Command::GetSourceDataFromLocKey(_pack_key, loc_key) => CentralCommand::send_back(&sender, Response::OptionStringStringVecString(dependencies.read().unwrap().loc_key_source(&loc_key))),
            Command::GetPackFileName(pack_key) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::String(pack.disk_file_name())),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }
            Command::GetPackedFileRawData(pack_key, path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                let cf = pack.compression_format();
                match pack.files_mut().get_mut(&path) {
                    Some(ref mut rfile) => {

                        // Make sure it's in memory.
                        match rfile.load() {
                            Ok(_) => match rfile.cached() {
                                Ok(data) => CentralCommand::send_back(&sender, Response::VecU8(data.to_vec())),

                                // If we don't have binary data, it may be decoded. Encode it and return the binary data.
                                //
                                // NOTE: This fucks up the table decoder if the table was badly decoded.
                                Err(_) =>  {
                                    let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));
                                    match rfile.encode(&extra_data, false, false, true) {
                                        Ok(data) => CentralCommand::send_back(&sender, Response::VecU8(data.unwrap())),
                                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                                    }
                                },
                            },
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("This PackedFile no longer exists in the PackFile.").to_string())),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::ImportDependenciesToOpenPackFile(pack_key, paths_by_data_source) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                let mut added_paths = vec![];
                let mut not_added_paths = vec![];

                let dependencies = dependencies.read().unwrap();
                for (data_source, paths) in &paths_by_data_source {
                    let files = match data_source {
                        DataSource::GameFiles => dependencies.files_by_path(paths, true, false, false),
                        DataSource::ParentFiles => dependencies.files_by_path(paths, false, true, false),
                        DataSource::AssKitFiles => HashMap::new(),
                        _ => {
                            CentralCommand::send_back(&sender, Response::Error("You can't import files from this source.".to_string()));
                            continue 'background_loop;
                        },
                    };

                    for file in files.into_values() {
                        let file_path = file.path_in_container_raw().to_owned();
                        let mut file = file.clone();
                        let _ = file.guess_file_type();
                        if let Ok(Some(path)) = pack.insert(file) {
                            added_paths.push(path);
                        } else {
                            not_added_paths.push(file_path);
                        }
                    }
                }

                // Once we're done with normal files, we process the ak ones.
                for (data_source, paths) in &paths_by_data_source {
                    match data_source {
                        DataSource::GameFiles | DataSource::ParentFiles => {},
                        DataSource::AssKitFiles => {
                            match &schema {
                                Some(ref schema) => {
                                    let mut files = vec![];

                                    // CEO table prefixes that should go into ceo_db/ instead of db/
                                    let is_ceo_table = |name: &str| -> bool {
                                        name.starts_with("ceo") || name == "ceos_tables" || name == "ceos_to_equipment_variants_tables"
                                    };

                                    for path in paths {

                                        // We only have tables. If it's a folder, it's either a table folder, db or the root.
                                        match path {
                                            ContainerPath::Folder(path) => {
                                                let mut path = path.to_owned();

                                                if path.ends_with('/') {
                                                    path.pop();
                                                }

                                                let path_split = path.split('/').collect::<Vec<_>>();
                                                let table_name_logic = game.vanilla_db_table_name_logic();

                                                // The db folder or the root folder directly.
                                                if path_split.len() == 1 {
                                                    let table_names = dependencies.asskit_only_db_tables().keys();
                                                    for table_name in table_names {
                                                        let table_file_name = match table_name_logic {
                                                            VanillaDBTableNameLogic::DefaultName(ref name) => name,
                                                            VanillaDBTableNameLogic::FolderName => table_name,
                                                        };

                                                        match dependencies.import_from_ak(table_name, schema) {
                                                            Ok(table) => {
                                                                let prefix = if is_ceo_table(table_name) { "ceo_db" } else { path_split[0] };
                                                                let file_path = if path_split.len() > 1 {
                                                                    format!("{}/{}/{}", prefix, &path_split[1..].join("/"), table_file_name)
                                                                } else {
                                                                    format!("{}/{}/{}", prefix, table_name, table_file_name)
                                                                };

                                                                let decoded = RFileDecoded::DB(table);
                                                                let file = RFile::new_from_decoded(&decoded, 0, &file_path);
                                                                files.push(file);
                                                            },
                                                            Err(_) => not_added_paths.push(path.clone()),
                                                        }
                                                    }
                                                }

                                                // A table folder.
                                                else if path_split.len() == 2 {

                                                    let table_name = path_split[1];
                                                    let table_file_name = match table_name_logic {
                                                        VanillaDBTableNameLogic::DefaultName(ref name) => name,
                                                        VanillaDBTableNameLogic::FolderName => table_name,
                                                    };

                                                    match dependencies.import_from_ak(table_name, schema) {
                                                        Ok(table) => {
                                                            let prefix = if is_ceo_table(table_name) { "ceo_db" } else { path_split[0] };
                                                            let file_path = format!("{}/{}/{}", prefix, table_name, table_file_name);

                                                            let decoded = RFileDecoded::DB(table);
                                                            let file = RFile::new_from_decoded(&decoded, 0, &file_path);
                                                            files.push(file);
                                                        },
                                                        Err(_) => not_added_paths.push(path.clone()),
                                                    }
                                                }

                                                // Any other situation is an error.
                                                else {
                                                    CentralCommand::send_back(&sender, Response::Error("No idea how you were able to trigger this.".to_string()));
                                                    continue 'background_loop;
                                                }

                                            }
                                            ContainerPath::File(path) => {
                                                let path_parts = path.split('/').collect::<Vec<_>>();
                                                let table_name = path_parts[1];
                                                match dependencies.import_from_ak(table_name, schema) {
                                                    Ok(table) => {
                                                        let file_path = if is_ceo_table(table_name) {
                                                            let file_name = path_parts.last().unwrap_or(&"data__");
                                                            format!("ceo_db/{}/{}", table_name, file_name)
                                                        } else {
                                                            path.clone()
                                                        };

                                                        let decoded = RFileDecoded::DB(table);
                                                        let file = RFile::new_from_decoded(&decoded, 0, &file_path);
                                                        files.push(file);
                                                    },
                                                    Err(_) => not_added_paths.push(path.clone()),
                                                }
                                            }
                                        }
                                    }

                                    for file in files {
                                        if let Ok(Some(path)) = pack.insert(file) {
                                            added_paths.push(path);
                                        }
                                    }
                                },
                                None => {
                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                                    continue 'background_loop;
                                }
                            }
                        },
                        _ => {
                            CentralCommand::send_back(&sender, Response::Error("You can't import files from this source.".to_string()));
                            continue 'background_loop;
                        },
                    }
                }

                CentralCommand::send_back(&sender, Response::VecContainerPathVecString(added_paths, not_added_paths));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::GetRFilesFromAllSources(paths, force_lowercased_paths) => {
                let mut packed_files = HashMap::new();
                let dependencies = dependencies.read().unwrap();

                // Get PackedFiles requested from the Parent Files.
                let mut packed_files_parent = HashMap::new();
                for (path, file) in dependencies.files_by_path(&paths, false, true, true) {
                    packed_files_parent.insert(if force_lowercased_paths { path.to_lowercase() } else { path }, file.clone());
                }

                // Get PackedFiles requested from the Game Files.
                let mut packed_files_game = HashMap::new();
                for (path, file) in dependencies.files_by_path(&paths, true, false, true) {
                    packed_files_game.insert(if force_lowercased_paths { path.to_lowercase() } else { path }, file.clone());
                }

                // Get PackedFiles requested from the AssKit Files.
                //let mut packed_files_asskit = HashMap::new();
                //if let Ok((packed_files_decoded, _)) = dependencies.get_packedfile_from_asskit_files(&paths) {
                //    for packed_file in packed_files_decoded {
                //        packed_files_asskit.insert(packed_file.get_path().to_vec(), packed_file);
                //    }
                //    packed_files.insert(DataSource::AssKitFiles, packed_files_asskit);
                //}

                // Get PackedFiles requested from all currently open packs.
                let mut packed_files_packfile = HashMap::new();
                for pack in packs.values() {
                    for file in pack.files_by_paths(&paths, true) {
                        packed_files_packfile.insert(if force_lowercased_paths { file.path_in_container_raw().to_lowercase() } else { file.path_in_container_raw().to_owned() }, file.clone());
                    }
                }

                packed_files.insert(DataSource::ParentFiles, packed_files_parent);
                packed_files.insert(DataSource::GameFiles, packed_files_game);
                packed_files.insert(DataSource::PackFile, packed_files_packfile);

                // Return the full list of PackedFiles requested, split by source.
                CentralCommand::send_back(&sender, Response::HashMapDataSourceHashMapStringRFile(packed_files));
            },

            Command::GetAnimPathsBySkeletonName(skeleton_name) => {
                let mut paths = HashSet::new();
                let mut dependencies = dependencies.write().unwrap();

                // Get PackedFiles requested from the Parent Files.
                let mut packed_files_parent = HashSet::new();
                for (path, file) in dependencies.files_by_types_mut(&[FileType::Anim], false, true) {
                    if let Ok(Some(RFileDecoded::Anim(file))) = file.decode(&None, false, true) {
                        if file.skeleton_name() == &skeleton_name {
                            packed_files_parent.insert(path);
                        }
                    }
                }

                // Get PackedFiles requested from the Game Files.
                let mut packed_files_game = HashSet::new();
                for (path, file) in dependencies.files_by_types_mut(&[FileType::Anim], true, false) {
                    if let Ok(Some(RFileDecoded::Anim(file))) = file.decode(&None, false, true) {
                        if file.skeleton_name() == &skeleton_name {
                            packed_files_game.insert(path);
                        }
                    }
                }

                // Get PackedFiles requested from all currently open packs.
                let mut packed_files_packfile = HashSet::new();
                for pack in packs.values_mut() {
                    for file in pack.files_by_type_mut(&[FileType::Anim]) {
                        if let Ok(Some(RFileDecoded::Anim(anim_file))) = file.decode(&None, false, true) {
                            if anim_file.skeleton_name() == &skeleton_name {
                                packed_files_packfile.insert(file.path_in_container_raw().to_owned());
                            }
                        }
                    }
                }

                paths.extend(packed_files_game);
                paths.extend(packed_files_parent);
                paths.extend(packed_files_packfile);

                // Return the full list of PackedFiles requested, split by source.
                CentralCommand::send_back(&sender, Response::HashSetString(paths));
            },

            Command::GetPackedFilesNamesStartingWitPathFromAllSources(path) => {
                let mut files: HashMap<DataSource, HashSet<ContainerPath>> = HashMap::new();
                let dependencies = dependencies.read().unwrap();

                let parent_files = dependencies.files_by_path(std::slice::from_ref(&path), false, true, true);
                if !parent_files.is_empty() {
                    files.insert(DataSource::ParentFiles, parent_files.into_keys().map(ContainerPath::File).collect());
                }

                let game_files = dependencies.files_by_path(std::slice::from_ref(&path), true, false, true);
                if !game_files.is_empty() {
                    files.insert(DataSource::GameFiles, game_files.into_keys().map(ContainerPath::File).collect());
                }

                let mut local_file_paths = HashSet::new();
                for pack in packs.values() {
                    for file in pack.files_by_path(&path, true) {
                        local_file_paths.insert(file.path_in_container());
                    }
                }
                if !local_file_paths.is_empty() {
                    files.insert(DataSource::PackFile, local_file_paths);
                }

                // Return the full list of PackedFile names requested, split by source.
                CentralCommand::send_back(&sender, Response::HashMapDataSourceHashSetContainerPath(files));
            },

            Command::SavePackedFilesToPackFileAndClean(pack_key, files, optimize) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                match &schema {
                    Some(ref schema) => {

                        // We receive a list of edited PackedFiles. The UI is the one that takes care of editing them to have the data we want where we want.
                        // Also, the UI is responsible for naming them in case they're new. Here we grab them and directly add them into the PackFile.
                        let mut added_paths = vec![];
                        for file in files {
                            if let Ok(Some(path)) = pack.insert(file) {
                                added_paths.push(path);
                            }
                        }

                        // Clean up duplicates from overwrites.
                        added_paths.sort();
                        added_paths.dedup();

                        if optimize {

                            // TODO: DO NOT CALL QT ON BACKEND.
                            let options = settings.optimizer_options();

                            // Then, optimize the PackFile. This should remove any non-edited rows/files.
                            match pack.optimize(None, &mut dependencies.write().unwrap(), schema, game, &options) {
                                Ok((paths_to_delete, paths_to_add)) => {
                                    added_paths.extend(paths_to_add.into_iter()
                                        .map(ContainerPath::File)
                                        .collect::<Vec<_>>());
                                    CentralCommand::send_back(&sender, Response::VecContainerPathVecContainerPath(added_paths, paths_to_delete.into_iter()
                                        .map(ContainerPath::File)
                                        .collect()));
                                },
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::VecContainerPathVecContainerPath(added_paths, vec![]));
                        }
                    },
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string())),
                }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::NotesForPath(pack_key, path) => {
                match packs.get(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::VecNote(pack.notes().notes_by_path(&path))),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }
            Command::AddNote(pack_key, note) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::Note(pack.notes_mut().add_note(note))),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }
            Command::DeleteNote(pack_key, path, id) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        pack.notes_mut().delete_note(&path, id);
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            Command::SaveLocalSchemaPatch(patches) => {
                let path = table_patches_path().unwrap().join(game.schema_file_name());
                match Schema::save_patches(&patches, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::RemoveLocalSchemaPatchesForTable(table_name) => {
                let path = table_patches_path().unwrap().join(game.schema_file_name());
                match Schema::remove_patches_for_table(&table_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::RemoveLocalSchemaPatchesForTableAndField(table_name, field_name) => {
                let path = table_patches_path().unwrap().join(game.schema_file_name());
                match Schema::remove_patches_for_table_and_field(&table_name, &field_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::ImportSchemaPatch(patch) => {
                match schema {
                    Some(ref mut schema) => {
                        Schema::add_patches_to_patch_set(schema.patches_mut(), &patch);
                        match schema.save(&schemas_path().unwrap().join(game.schema_file_name())) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string())),
                }
            }

            Command::GenerateMissingLocData(_pack_key) => {
                match dependencies.read().unwrap().generate_missing_loc_data(&mut packs) {
                    Ok(path) => CentralCommand::send_back(&sender, Response::VecContainerPath(path)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::PackMap(pack_key, tile_maps, tiles) => {
                match schema {
                    Some(ref schema) => {
                        let mut dependencies = dependencies.write().unwrap();
                        let options = settings.optimizer_options();
                        match dependencies.add_tile_maps_and_tiles(&mut packs, Some(&pack_key), game, schema, options, tile_maps, tiles) {
                            Ok((paths_to_add, paths_to_delete)) => CentralCommand::send_back(&sender, Response::VecContainerPathVecContainerPath(paths_to_add, paths_to_delete)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string())),
                }
            }

            // Initialize the folder for a MyMod, including the folder structure it needs.
            Command::InitializeMyModFolder(mod_name, mod_game, sublime_support, vscode_support, git_support)  => {
                let mut mymod_path = settings.path_buf(MYMOD_BASE_PATH);
                if !mymod_path.is_dir() {
                    CentralCommand::send_back(&sender, Response::Error("MyMod path is not configured. Configure it in the settings and try again.".to_string()));
                    continue;
                }

                mymod_path.push(&mod_game);

                // Just in case the folder doesn't exist, we try to create it.
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the MyMod's Game folder: {}.", error)));
                    continue;
                }

                // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                mymod_path.push(&mod_name);
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the MyMod's Assets folder: {}.", error)));
                    continue;
                };

                // Create a repo inside the MyMod's folder.
                if let Some(gitignore) = git_support {
                    let git_integration = GitIntegration::new(&mymod_path, "", "", "");
                    if let Err(error) = git_integration.init() {
                        CentralCommand::send_back(&sender, Response::Error(error.to_string()));
                        continue
                    }

                    if let Err(error) = git_integration.add_gitignore(&gitignore) {
                        CentralCommand::send_back(&sender, Response::Error(error.to_string()));
                        continue
                    }
                }

                // If the tw_autogen supports the game, create the vscode and sublime configs for lua mods.
                if sublime_support || vscode_support {
                    if let Ok(lua_autogen_folder) = lua_autogen_game_path(game) {
                        let lua_autogen_folder = lua_autogen_folder.to_string_lossy().to_string().replace('\\', "/");

                        // VSCode support.
                        if vscode_support {
                            let mut vscode_config_path = mymod_path.to_owned();
                            vscode_config_path.push(".vscode");

                            if let Err(error) = DirBuilder::new().recursive(true).create(&vscode_config_path) {
                                CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the VSCode Config folder: {}.", error)));
                                continue;
                            };

                            let mut vscode_extensions_path_file = vscode_config_path.to_owned();
                            vscode_extensions_path_file.push("extensions.json");
                            if let Ok(file) = File::create(vscode_extensions_path_file) {
                                let mut file = BufWriter::new(file);
                                let _ = file.write_all("
{
    \"recommendations\": [
        \"sumneko.lua\",
        \"formulahendry.code-runner\"
    ],
}".as_bytes());
                            }
                        }

                        // Sublime support.
                        if sublime_support {
                            let mut sublime_config_path = mymod_path.to_owned();
                            sublime_config_path.push(format!("{mod_name}.sublime-project"));
                            if let Ok(file) = File::create(sublime_config_path) {
                                let mut file = BufWriter::new(file);
                                let _ = file.write_all("
{
    \"folders\":
    [
        {
            \"path\": \".\"
        }
    ]
}".to_string().as_bytes());
                            }
                        }

                        // Generic lua support.
                        let mut luarc_config_path = mymod_path.to_owned();
                        luarc_config_path.push(".luarc.json");

                        if let Ok(file) = File::create(luarc_config_path) {
                            let mut file = BufWriter::new(file);
                            let _ = file.write_all(format!("
{{
    \"workspace.library\": [
        \"{lua_autogen_folder}/global/\",
        \"{lua_autogen_folder}/campaign/\",
        \"{lua_autogen_folder}/frontend/\",
        \"{lua_autogen_folder}/battle/\"
    ],
    \"runtime.version\": \"Lua 5.1\",
    \"completion.autoRequire\": false,
    \"workspace.preloadFileSize\": 1500,
    \"workspace.ignoreSubmodules\": false,
    \"diagnostics.workspaceDelay\": 500,
    \"diagnostics.workspaceRate\": 40,
    \"diagnostics.disable\": [
        \"lowercase-global\",
        \"trailing-space\"
    ],
    \"hint.setType\": true,
    \"workspace.ignoreDir\": [
        \".vscode\",
        \".git\"
    ]
}}").as_bytes());
                        }
                    }
                }

                // Return the name of the MyMod Pack.
                mymod_path.set_extension("pack");
                CentralCommand::send_back(&sender, Response::PathBuf(mymod_path));
            },

            Command::LiveExport(pack_key) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        let game_path = settings.path_buf(game.key());
                        let disable_regen_table_guid = settings.bool("disable_uuid_regeneration_on_db_tables");
                        let keys_first = settings.bool("tables_use_old_column_order_for_tsv");
                        match pack.live_export(game, &game_path, disable_regen_table_guid, keys_first) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::SetPackOperationalMode(pack_key, mode) => {
                if packs.contains_key(&pack_key) {
                    pack_modes.insert(pack_key, mode);
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key)));
                }
            },

            Command::GetPackOperationalMode(pack_key) => {
                let mode = pack_modes.get(&pack_key).cloned().unwrap_or(OperationalMode::Normal);
                CentralCommand::send_back(&sender, Response::OperationalMode(mode));
            },

            Command::AddLineToPackIgnoredDiagnostics(pack_key, line) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        if let Some(diagnostics_ignored) = pack.settings_mut().settings_text_mut().get_mut("diagnostics_files_to_ignore") {
                            diagnostics_ignored.push_str(&line);
                        } else {
                            pack.settings_mut().settings_text_mut().insert("diagnostics_files_to_ignore".to_owned(), line);
                        }
                        CentralCommand::send_back(&sender, Response::Success);
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            },

            Command::UpdateEmpireAndNapoleonAK => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match old_ak_files_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE);
                                git_integration.update_repo().map(|_| ()).map_err(|e| e.into())
                            },
                            Err(error) => Err(error),
                        }
                    }).await.unwrap();

                    match result {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            Command::GetPackTranslation(pack_key, language) => {
                let game_key = game.key();
                match translations_local_path() {
                    Ok(local_path) => {
                        let mut base_english = HashMap::new();
                        let mut base_local_fixes = HashMap::new();

                        match translations_remote_path() {
                            Ok(remote_path) => {

                                let vanilla_loc_path = remote_path.join(format!("{}/{}", game.key(), VANILLA_LOC_NAME));
                                if let Ok(mut vanilla_loc) = RFile::tsv_import_from_path(&vanilla_loc_path, &None) {
                                    let _ = vanilla_loc.guess_file_type();
                                    if let Ok(RFileDecoded::Loc(vanilla_loc)) = vanilla_loc.decoded() {

                                        // If we have a fixes file for the vanilla translation, apply it before everything else.
                                        let fixes_loc_path = remote_path.join(format!("{}/{}{}.tsv", game.key(), VANILLA_FIXES_NAME, language));
                                        if let Ok(mut fixes_loc) = RFile::tsv_import_from_path(&fixes_loc_path, &None) {
                                            let _ = fixes_loc.guess_file_type();

                                            if let Ok(RFileDecoded::Loc(fixes_loc)) = fixes_loc.decoded() {
                                                base_local_fixes.extend(fixes_loc.data().iter().map(|x| (x[0].data_to_string().to_string(), x[1].data_to_string().to_string())).collect::<Vec<_>>());
                                            }
                                        }

                                        base_english.extend(vanilla_loc.data().iter().map(|x| (x[0].data_to_string().to_string(), x[1].data_to_string().to_string())).collect::<Vec<_>>());
                                    }
                                }

                                let dependencies = dependencies.read().unwrap();
                                let paths = vec![local_path, remote_path];
                                let Some(pack_ref) = get_pack(&packs, &pack_key, &sender) else { continue 'background_loop; };
                                match PackTranslation::new(&paths, pack_ref, game_key, &language, &dependencies, &base_english, &base_local_fixes) {
                                    Ok(tr) => CentralCommand::send_back(&sender, Response::PackTranslation(tr)),
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                                }
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::UpdateTranslations => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match translations_remote_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
                                git_integration.update_repo().map(|_| ()).map_err(|e| e.into())
                            },
                            Err(error) => Err(error),
                        }
                    }).await.unwrap();

                    match result {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            Command::BuildStarposGetCampaingIds(_pack_key) => {
                let ids = dependencies.read().unwrap().db_values_from_table_name_and_column_name(Some(&packs), "campaigns_tables", "campaign_name", true, true);
                CentralCommand::send_back(&sender, Response::HashSetString(ids));
            }

            Command::BuildStarposCheckVictoryConditions(pack_key) => {
                let Some(pack_ref) = get_pack(&packs, &pack_key, &sender) else { continue 'background_loop; };
                if !GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) || (
                        GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) &&
                        pack_ref.file(VICTORY_OBJECTIVES_FILE_NAME, false).is_some()
                    ) {
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::Error("Missing \"db/victory_objectives.txt\" file. Processing the startpos without this file will result in issues in campaign. Add the file to the pack and try again.".to_string()));
                }
            }

            Command::BuildStarpos(pack_key, campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game_path = settings.path_buf(game.key());

                // 3K needs two passes, one per startpos, and there are two per campaign.
                if game.key() == KEY_THREE_KINGDOMS {
                    match dependencies.build_starpos_pre(&mut packs, Some(&pack_key), game, &game_path, &campaign_id, process_hlp_spd_data, "historical") {
                        Ok(_) => match dependencies.build_starpos_pre(&mut packs, Some(&pack_key), game, &game_path, &campaign_id, false, "romance") {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    match dependencies.build_starpos_pre(&mut packs, Some(&pack_key), game, &game_path, &campaign_id, process_hlp_spd_data, "") {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                }
            }

            Command::BuildStarposPost(pack_key, campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game_path = settings.path_buf(game.key());
                let asskit_path = Some(settings.path_buf(&(game.key().to_owned() + "_assembly_kit")));

                let sub_start_pos = if game.key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match dependencies.build_starpos_post(&mut packs, Some(&pack_key), game, &game_path, asskit_path, &campaign_id, process_hlp_spd_data, false, &sub_start_pos) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            },

            Command::BuildStarposCleanup(pack_key, campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game_path = settings.path_buf(game.key());
                let asskit_path = Some(settings.path_buf(&(game.key().to_owned() + "_assembly_kit")));

                let sub_start_pos = if game.key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match dependencies.build_starpos_post(&mut packs, Some(&pack_key), game, &game_path, asskit_path, &campaign_id, process_hlp_spd_data, true, &sub_start_pos) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            },

            Command::BuildCeo(pack_key, akit_path, bob_exe_path) => {
                use std::process::Command as SysCommand;
                use std::time::{Duration, Instant};

                let akit_root = PathBuf::from(&akit_path);
                let bob_exe = PathBuf::from(&bob_exe_path);
                let bob_dir = match bob_exe.parent() {
                    Some(d) => d.to_path_buf(),
                    None => { CentralCommand::send_back(&sender, Response::Error("Invalid BOB path".into())); continue 'background_loop; }
                };
                let raw_db = akit_root.join(r"raw_data\db");
                let ceo_ccd = akit_root.join(r"working_data\campaigns\ceo_data.ccd");

                // ── Step 1: Backup existing ceo_data.ccd ─────────────────────
                if ceo_ccd.exists() {
                    let bak = ceo_ccd.with_extension("ccd.bak1");
                    if let Err(e) = std::fs::copy(&ceo_ccd, &bak) {
                        CentralCommand::send_back(&sender, Response::Error(format!("Failed to backup ceo_data.ccd: {e}")));
                        continue 'background_loop;
                    }
                }

                // ── Step 2: Backup raw_data/db/ceo_*.xml files ────────────────
                let mut xml_backups: Vec<(PathBuf, PathBuf)> = Vec::new();
                if raw_db.exists() {
                    match std::fs::read_dir(&raw_db) {
                        Ok(entries) => {
                            for entry in entries.filter_map(|e| e.ok()) {
                                let fname = entry.file_name();
                                let s = fname.to_string_lossy().to_lowercase();
                                if s.starts_with("ceo") && s.ends_with(".xml") {
                                    let orig = entry.path();
                                    let bak = orig.with_extension("xml.bak");
                                    if std::fs::copy(&orig, &bak).is_ok() {
                                        xml_backups.push((orig, bak));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            CentralCommand::send_back(&sender, Response::Error(format!("Failed to read raw_data/db: {e}")));
                            continue 'background_loop;
                        }
                    }
                }

                // ── Step 3: Export CEO DB tables from pack → raw_data/db XML ──
                let pack_ref = match packs.get_mut(&pack_key) {
                    Some(p) => p,
                    None => {
                        for (orig, bak) in &xml_backups { let _ = std::fs::rename(bak, orig); }
                        CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {pack_key}")));
                        continue 'background_loop;
                    }
                };

                // Only export the tables that BOB actually reads when building ceo_data.ccd.
                let ceo_allowed_folders: std::collections::HashSet<&str> = [
                    "ceo_active_permissions_tables",
                    "ceo_anti_ceo_pairs_tables",
                    "ceo_can_equip_requirements_tables",
                    "ceo_categories_tables",
                    "ceo_effect_list_to_effects_tables",
                    "ceo_effect_lists_tables",
                    "ceo_equipment_category_managers_tables",
                    "ceo_equipment_manager_all_possible_ceos_tables",
                    "ceo_equipment_manager_campaign_lookups_tables",
                    "ceo_equipment_manager_to_category_managers_tables",
                    "ceo_equipment_manager_types_tables",
                    "ceo_equipment_managers_tables",
                    "ceo_equipped_set_bonus_ceos_tables",
                    "ceo_equipped_set_bonus_effect_bundles_tables",
                    "ceo_equipped_set_bonuses_tables",
                    "ceo_equipped_set_bonuses_to_incident_junctions_tables",
                    "ceo_event_feed_categories_tables",
                    "ceo_group_ceos_tables",
                    "ceo_group_spawners_tables",
                    "ceo_groups_tables",
                    "ceo_initial_data_active_ceos_tables",
                    "ceo_initial_data_active_spawners_tables",
                    "ceo_initial_data_equipments_tables",
                    "ceo_initial_data_scripted_permissions_tables",
                    "ceo_initial_data_stages_tables",
                    "ceo_initial_data_to_stages_tables",
                    "ceo_initial_data_triggers_tables",
                    "ceo_initial_datas_tables",
                    "ceo_location_enums_tables",
                    "ceo_nodes_tables",
                    "ceo_permissions_groups_tables",
                    "ceo_permissions_tables",
                    "ceo_post_battle_loot_chances_tables",
                    "ceo_rarities_tables",
                    "ceo_scripted_permissions_tables",
                    "ceo_scripted_permissions_to_permissions_tables",
                    "ceo_set_items_tables",
                    "ceo_sets_tables",
                    "ceo_spawner_can_spawn_requirements_tables",
                    "ceo_spawners_tables",
                    "ceo_template_manager_all_possible_ceos_tables",
                    "ceo_template_manager_campaign_lookups_tables",
                    "ceo_template_manager_ceo_limits_tables",
                    "ceo_template_manager_ceo_spawn_limits_tables",
                    "ceo_template_manager_supported_categories_tables",
                    "ceo_template_manager_types_tables",
                    "ceo_template_managers_tables",
                    "ceo_threshold_nodes_tables",
                    "ceo_thresholds_tables",
                    "ceo_to_target_ceo_junctions_tables",
                    "ceo_to_target_factions_tables",
                    "ceo_to_target_junction_reasons_tables",
                    "ceo_to_target_province_junctions_tables",
                    "ceo_to_ui_display_junctions_tables",
                    "ceo_trigger_behaviour_enums_tables",
                    "ceo_trigger_target_requirements_tables",
                    "ceo_trigger_targets_tables",
                    "ceo_trigger_to_trigger_targets_tables",
                    "ceo_triggers_tables",
                    "ceos_tables",
                    "ceos_to_equipment_variants_tables",
                ].iter().copied().collect();

                let ceo_table_paths: Vec<String> = pack_ref.files()
                    .keys()
                    .filter(|p| {
                        let mut parts = p.splitn(3, '/');
                        let prefix = parts.next().unwrap_or("");
                        if prefix != "db" && prefix != "ceo_db" {
                            return false;
                        }
                        parts.next()
                            .map(|folder| ceo_allowed_folders.contains(folder))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                let decode_extra = {
                    let mut d = DecodeableExtraData::default();
                    d.set_schema(schema.as_ref());
                    Some(d)
                };
                let mut export_errors: Vec<String> = Vec::new();

                // Group table paths by their target XML file so multiple
                // db tables (e.g. data__, data__01) are combined into one XML.
                let mut xml_groups: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
                for table_path in &ceo_table_paths {
                    // "db/ceos_tables/data__" -> folder="ceos_tables" -> xml="ceos.xml"
                    let parts: Vec<&str> = table_path.split('/').collect();
                    if parts.len() < 2 { continue; }
                    let folder = parts[1];
                    let xml_name = if folder.ends_with("_tables") {
                        folder[..folder.len() - 7].to_owned() + ".xml"
                    } else {
                        folder.to_owned() + ".xml"
                    };
                    xml_groups.entry(xml_name).or_default().push(table_path.clone());
                }

                for (xml_name, table_paths) in &xml_groups {
                    let xml_path = raw_db.join(xml_name);
                    let table_tag = xml_name.trim_end_matches(".xml");
                    let xsd_name = xml_name.replace(".xml", ".xsd");

                    let mut xml = format!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n\
                         <dataroot xmlns:od=\"urn:schemas-microsoft-com:officedata\" \
                         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" \
                         xsi:noNamespaceSchemaLocation=\"{xsd_name}\" \
                         export_time=\"\" revision=\"0\" export_branch=\"\" export_user=\"rpfm\">\r\n\
                         <edit_uuid>00000000-0000-0000-0000-000000000000</edit_uuid>\r\n"
                    );

                    for table_path in table_paths {
                        let rfile = match pack_ref.files_mut().get_mut(table_path.as_str()) {
                            Some(f) => f,
                            None => continue,
                        };

                        let _ = rfile.load();
                        let _ = rfile.decode(&decode_extra, true, false);

                        let db_table = match rfile.decoded() {
                            Ok(RFileDecoded::DB(db)) => db.clone(),
                            _ => { export_errors.push(format!("Could not decode {table_path}")); continue; }
                        };

                        let fields: Vec<_> = db_table.definition().fields_processed().to_vec();

                        for row in db_table.data().iter() {
                            let mut field_pairs: Vec<(String, String)> = Vec::new();
                            for (field_def, value) in fields.iter().zip(row.iter()) {
                                let fname = field_def.name().to_owned();
                                let val_str = match value {
                                    DecodedData::Boolean(b) => if *b { "1".to_owned() } else { "0".to_owned() },
                                    DecodedData::I16(v) => v.to_string(),
                                    DecodedData::I32(v) => v.to_string(),
                                    DecodedData::I64(v) => v.to_string(),
                                    DecodedData::OptionalI16(v) => v.to_string(),
                                    DecodedData::OptionalI32(v) => v.to_string(),
                                    DecodedData::OptionalI64(v) => v.to_string(),
                                    DecodedData::F32(v) => v.to_string(),
                                    DecodedData::F64(v) => v.to_string(),
                                    DecodedData::StringU8(s) | DecodedData::StringU16(s) |
                                    DecodedData::OptionalStringU8(s) | DecodedData::OptionalStringU16(s) => s.clone(),
                                    DecodedData::ColourRGB(s) => s.clone(),
                                    _ => String::new(),
                                };
                                let escaped = val_str
                                    .replace('&', "&amp;")
                                    .replace('<', "&lt;")
                                    .replace('>', "&gt;")
                                    .replace('"', "&quot;");
                                field_pairs.push((fname, escaped));
                            }
                            field_pairs.sort_by(|a, b| a.0.cmp(&b.0));

                            xml.push_str(&format!("<{table_tag}>\r\n"));
                            for (fname, val) in &field_pairs {
                                xml.push_str(&format!("<{fname}>{val}</{fname}>\r\n"));
                            }
                            xml.push_str(&format!("</{table_tag}>\r\n"));
                        }
                    }

                    xml.push_str("</dataroot>\r\n");

                    if let Err(e) = std::fs::write(&xml_path, xml.as_bytes()) {
                        export_errors.push(format!("Failed to write {xml_name}: {e}"));
                    }
                }

                if !export_errors.is_empty() {
                    for (orig, bak) in &xml_backups { let _ = std::fs::rename(bak, orig); }
                    CentralCommand::send_back(&sender, Response::Error(format!("Export errors:\n{}", export_errors.join("\n"))));
                    continue 'background_loop;
                }

                // ── Step 4: Write BOB config and launch ───────────────────────
                let cfg_path = bob_dir.join("BOB/default_configuration.xml");

                // Backup any existing config so we can restore it after BOB runs.
                let cfg_backup = bob_dir.join("BOB/default_configuration.xml.rpfm_bak");
                let cfg_existed = cfg_path.exists();
                if cfg_existed {
                    if let Err(e) = std::fs::rename(&cfg_path, &cfg_backup) {
                        for (orig, bak) in &xml_backups { let _ = std::fs::rename(bak, orig); }
                        CentralCommand::send_back(&sender, Response::Error(format!("Failed to backup BOB config: {e}")));
                        continue 'background_loop;
                    }
                }

                const BOB_CONFIG_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
                <bob_configuration><processors><processor>Campaign</processor></processors>
                <directories/><global_rules/><retail>0</retail><silent>1</silent>
                <get_latest>0</get_latest><connect_db>0</connect_db>
                <merge_for_checkin_mode>2</merge_for_checkin_mode>
                <selected_files><entry>&lt;working&gt;/campaigns/ceo_data.ccd</entry></selected_files>
                </bob_configuration>"#;

                if let Err(e) = std::fs::write(&cfg_path, BOB_CONFIG_XML) {
                    // Restore backup before bailing.
                    if cfg_existed { let _ = std::fs::rename(&cfg_backup, &cfg_path); }
                    for (orig, bak) in &xml_backups { let _ = std::fs::rename(bak, orig); }
                    CentralCommand::send_back(&sender, Response::Error(format!("Failed to write BOB config: {e}")));
                    continue 'background_loop;
                }

                let output = match SysCommand::new(&bob_exe).current_dir(&bob_dir).output() {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = std::fs::remove_file(&cfg_path);
                        if cfg_existed { let _ = std::fs::rename(&cfg_backup, &cfg_path); }
                        for (orig, bak) in &xml_backups { let _ = std::fs::rename(bak, orig); }
                        CentralCommand::send_back(&sender, Response::Error(format!("Failed to launch BOB: {e}")));
                        continue 'background_loop;
                    }
                };

                // Restore config regardless of BOB's result.
                let _ = std::fs::remove_file(&cfg_path);
                if cfg_existed { let _ = std::fs::rename(&cfg_backup, &cfg_path); }

                // ── Step 5: Poll for ceo_data.ccd (180s timeout) ─────────────
                let deadline = Instant::now() + Duration::from_secs(180);
                let found = loop {
                    if ceo_ccd.exists() { break true; }
                    if Instant::now() >= deadline { break false; }
                    std::thread::sleep(Duration::from_millis(500));
                };

                // ── Step 6: Restore original ceo_*.xml files ─────────────────
                for (orig, bak) in &xml_backups {
                    let _ = std::fs::rename(bak, orig);
                }

                // ── Step 7: Report result ─────────────────────────────────────
                if found {
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::Error(format!(
                        "ceo_data.ccd not generated within 180s. BOB exit: {:?}\nstdout: {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stdout)
                    )));
                }
            }

            Command::BuildCeoPost(pack_key, akit_path) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        match build_ceo_post(pack, &akit_path) {
                            Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                            Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {pack_key}"))),
                }
            }

            Command::GetTraitCeos => {
                let deps = dependencies.read().unwrap();
                let trait_ceos = get_trait_ceos(&deps);
                CentralCommand::send_back(&sender, Response::VecStringTuples(trait_ceos));
            }
            
            Command::BuildCeoEntries(pack_key, entries) => {
                let result = (|| -> Result<Vec<ContainerPath>> {
                    let schema = schema.as_ref()
                        .ok_or_else(|| anyhow!("No schema loaded for the current game."))?;
                    let pack = packs.get_mut(&pack_key)
                        .ok_or_else(|| anyhow!("Pack not found: {}", pack_key))?;
                    build_ceo_entries(pack, schema, &entries)
                })();

                match result {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::UpdateAnimIds(pack_key, starting_id, offset) => {
                match packs.get_mut(&pack_key) {
                    Some(pack) => {
                        match pack.update_anim_ids(game, starting_id, offset) {
                            Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Pack not found: {}", pack_key))),
                }
            }

            Command::GetTablesFromDependencies(table_name) => {
                let dependencies = dependencies.read().unwrap();
                match dependencies.db_data(&table_name, true, true) {
                    Ok(files) => CentralCommand::send_back(&sender, Response::VecRFile(files.iter().map(|x| (**x).clone()).collect::<Vec<_>>())),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::ExportRigidToGltf(rigid, path) => {
                let mut dependencies = dependencies.write().unwrap();
                match gltf_from_rigid(&rigid, &mut dependencies) {
                    Ok(gltf) => match save_gltf_to_disk(&gltf, &PathBuf::from(path)) {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // Settings IPC handlers - all settings are now managed locally in background_loop
            Command::SettingsGetBool(key) => {
                CentralCommand::send_back(&sender, Response::Bool(settings.bool(&key)));
            }
            Command::SettingsGetI32(key) => {
                CentralCommand::send_back(&sender, Response::I32(settings.i32(&key)));
            }
            Command::SettingsGetF32(key) => {
                CentralCommand::send_back(&sender, Response::F32(settings.f32(&key)));
            }
            Command::SettingsGetString(key) => {
                CentralCommand::send_back(&sender, Response::String(settings.string(&key)));
            }
            Command::SettingsGetPathBuf(key) => {
                CentralCommand::send_back(&sender, Response::PathBuf(settings.path_buf(&key)));
            }
            Command::SettingsGetVecString(key) => {
                CentralCommand::send_back(&sender, Response::VecString(settings.vec_string(&key)));
            }
            Command::SettingsGetVecRaw(key) => {
                CentralCommand::send_back(&sender, Response::VecU8(settings.raw_data(&key)));
            }
            Command::SettingsGetAll => {
                CentralCommand::send_back(&sender, Response::SettingsAll(SettingsSnapshot {
                    bool: settings.bool.clone(),
                    i32: settings.i32.clone(),
                    f32: settings.f32.clone(),
                    string: settings.string.clone(),
                    raw_data: settings.raw_data.clone(),
                    vec_string: settings.vec_string.clone(),
                }));
            }
            Command::SettingsSetBool(key, value) => {
                match settings.set_bool(&key, value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetI32(key, value) => {
                match settings.set_i32(&key, value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetF32(key, value) => {
                match settings.set_f32(&key, value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetString(key, value) => {
                match settings.set_string(&key, &value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetPathBuf(key, value) => {
                match settings.set_path_buf(&key, &value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetVecString(key, value) => {
                match settings.set_vec_string(&key, &value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            }
            Command::SettingsSetVecRaw(key, value) => {
                match settings.set_raw_data(&key, &value) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::ConfigPath => {
                match config_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::AssemblyKitPath => {
                match settings.assembly_kit_path(game) {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::BackupAutosavePath => {
                match backup_autosave_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::OldAkDataPath => {
                match old_ak_files_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::SchemasPath => {
                match schemas_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::TableProfilesPath => {
                match table_profiles_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::TranslationsLocalPath => {
                match translations_local_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::DependenciesCachePath => {
                match dependencies_cache_path() {
                    Ok(path) => CentralCommand::send_back(&sender, Response::PathBuf(path)),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::SettingsClearPath(path) => {
                match clear_config_path(&path) {
                    Ok(()) => CentralCommand::send_back(&sender, Response::Success),
                    Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
                }
            },
            Command::BackupSettings => {
                backup_settings = settings.clone();
                CentralCommand::send_back(&sender, Response::Success);
            }
            Command::ClearSettings => match Settings::init(true) {
                Ok(set) => {
                    settings = set;
                    CentralCommand::send_back(&sender, Response::Success);},
                Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
            },
            Command::RestoreBackupSettings => {
                settings = backup_settings.clone();
                CentralCommand::send_back(&sender, Response::Success);
            }
            Command::OptimizerOptions => CentralCommand::send_back(&sender, Response::OptimizerOptions(settings.optimizer_options())),

            Command::IsSchemaLoaded => CentralCommand::send_back(&sender, Response::Bool(schema.is_some())),
            Command::DefinitionsByTableName(name) => match schema {
                Some(ref schema) => {
                    match schema.definitions_by_table_name(&name) {
                        Some(defs) => CentralCommand::send_back(&sender, Response::VecDefinition(defs.to_vec())),
                        None => CentralCommand::send_back(&sender, Response::VecDefinition(vec![])),
                    }
                },
                None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string())),
            },
            Command::ReferencingColumnsForDefinition(name, definition) => match schema {
                Some(ref schema) => CentralCommand::send_back(&sender, Response::HashMapStringHashMapStringVecString(schema.referencing_columns_for_table(&name, &definition))),
                None => CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string())),
            },
            Command::Schema => match &schema {
                Some(schema) => CentralCommand::send_back(&sender, Response::Schema(schema.clone())),
                None => CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string())),
            }
            Command::DefinitionByTableNameAndVersion(name, version) => match schema {
                Some(ref schema) => match schema.definition_by_name_and_version(&name, version) {
                    Some(def) => CentralCommand::send_back(&sender, Response::Definition(def.clone())),
                    None => CentralCommand::send_back(&sender, Response::Error(format!("No definition found for table '{}' with version {}.", name, version))),
                },
                None => CentralCommand::send_back(&sender, Response::Error("There is no Schema for the Game Selected.".to_string())),
            },

            Command::DeleteDefinition(name, version) => {
                if let Some(ref mut schema) = schema {
                    schema.remove_definition(&name, version);
                }
                CentralCommand::send_back(&sender, Response::Success);
            }

            Command::FieldsProcessed(definition) => {
                CentralCommand::send_back(&sender, Response::VecField(definition.fields_processed()));
            }
        }
    }
}

/// Function to simplify logic for changing game selected.


fn load_schema(schema: &mut Option<Schema>, packs: &mut BTreeMap<String, Pack>, game: &GameInfo, settings: &Settings) {

    // Before loading the schema, make sure we don't have tables with definitions from the current schema.
    for pack in packs.values_mut() {
        let cf = pack.compression_format();
        let mut files = pack.files_by_type_mut(&[FileType::DB]);
        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

        files.par_iter_mut().for_each(|file| {
            let _ = file.encode(&extra_data, true, true, false);
        });
    }

    // Load the new schema.
    let schema_path = schemas_path().unwrap().join(game.schema_file_name());
    let local_patches_path = table_patches_path().unwrap().join(game.schema_file_name());
    *schema = Schema::load(&schema_path, Some(&local_patches_path)).ok();

    // Re-decode all the tables in the open packs.
    if let Some(ref schema) = schema {
        for pack in packs.values_mut() {
            let mut files = pack.files_by_type_mut(&[FileType::DB]);
            let mut extra_data = DecodeableExtraData::default();
            extra_data.set_schema(Some(schema));
            let extra_data = Some(extra_data);

            files.par_iter_mut().for_each(|file| {
                let _ = file.decode(&extra_data, true, false);
            });
        }
    }
}

fn decode_and_send_file(file: &mut RFile, sender: &UnboundedSender<Response>, settings: &Settings, game: &GameInfo, schema: &Option<Schema>) {
    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_schema(schema.as_ref());
    extra_data.set_game_info(Some(game));

    // Do not attempt to decode these.
    let mut ignored_file_types = vec![
        FileType::Anim,
        FileType::BMD,
        FileType::BMDVegetation,
        FileType::Dat,
        FileType::Font,
        FileType::HlslCompiled,
        FileType::Pack,
        FileType::SoundBank,
        FileType::Unknown
    ];

    // Do not even attempt to decode esf files if the editor is disabled.
    if !settings.bool("enable_esf_editor") {
        ignored_file_types.push(FileType::ESF);
    }

    if ignored_file_types.contains(&file.file_type()) {
        return CentralCommand::send_back(sender, Response::Unknown);
    }
    let result = file.decode(&Some(extra_data), true, true).transpose().unwrap();

    match result {
        Ok(RFileDecoded::AnimFragmentBattle(data)) => CentralCommand::send_back(sender, Response::AnimFragmentBattleRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::AnimPack(data)) => CentralCommand::send_back(sender, Response::AnimPackRFileInfo(data.files().values().map(From::from).collect(), From::from(&*file))),
        Ok(RFileDecoded::AnimsTable(data)) => CentralCommand::send_back(sender, Response::AnimsTableRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Anim(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Atlas(data)) => CentralCommand::send_back(sender, Response::AtlasRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Audio(data)) => CentralCommand::send_back(sender, Response::AudioRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::BMD(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::BMDVegetation(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Dat(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::DB(table)) => CentralCommand::send_back(sender, Response::DBRFileInfo(table, From::from(&*file))),
        Ok(RFileDecoded::ESF(data)) => CentralCommand::send_back(sender, Response::ESFRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Font(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::HlslCompiled(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::GroupFormations(data)) => CentralCommand::send_back(sender, Response::GroupFormationsRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Image(image)) => CentralCommand::send_back(sender, Response::ImageRFileInfo(image, From::from(&*file))),
        Ok(RFileDecoded::Loc(table)) => CentralCommand::send_back(sender, Response::LocRFileInfo(table, From::from(&*file))),
        Ok(RFileDecoded::MatchedCombat(data)) => CentralCommand::send_back(sender, Response::MatchedCombatRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Pack(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::PortraitSettings(data)) => CentralCommand::send_back(sender, Response::PortraitSettingsRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::RigidModel(data)) => CentralCommand::send_back(sender, Response::RigidModelRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::SoundBank(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Text(text)) => CentralCommand::send_back(sender, Response::TextRFileInfo(text, From::from(&*file))),
        Ok(RFileDecoded::UIC(uic)) => CentralCommand::send_back(sender, Response::UICRFileInfo(uic, From::from(&*file))),
        Ok(RFileDecoded::UnitVariant(data)) => CentralCommand::send_back(sender, Response::UnitVariantRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Unknown(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Video(data)) => CentralCommand::send_back(sender, Response::VideoInfoRFileInfo(From::from(&data), From::from(&*file))),
        Ok(RFileDecoded::VMD(data)) => CentralCommand::send_back(sender, Response::VMDRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::WSModel(data)) => CentralCommand::send_back(sender, Response::WSModelRFileInfo(data, From::from(&*file))),
        Err(error) => CentralCommand::send_back(sender, Response::Error(error.to_string())),
    }
}

/// In debug mode, this function returns the base folder of the repo.
/// In release mode, it returns the folder where the executable of the program is.
fn exe_path() -> PathBuf {
    if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    }
}

/// Spawns an async task that checks for git updates for the given repository configuration,
/// sending the result back through `sender`.
fn git_update_check(
    sender: UnboundedSender<Response>,
    path_fn: fn() -> Result<PathBuf>,
    repo: &'static str,
    branch: &'static str,
    remote: &'static str,
) {
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            match path_fn() {
                Ok(local_path) => {
                    let git_integration = GitIntegration::new(&local_path, repo, branch, remote);
                    git_integration.check_update().map_err(|e| e.into())
                }
                Err(error) => Err(error),
            }
        }).await.unwrap();

        match result {
            Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseGit(response)),
            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
        }
    });
}

// TODO: what do we do with this?
fn tr(s: &str) -> String {
    s.to_owned()
}
