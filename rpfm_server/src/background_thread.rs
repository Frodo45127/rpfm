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

use anyhow::anyhow;

use itertools::Itertools;
use open::that;
use rayon::prelude::*;

use std::collections::{BTreeMap, HashMap};
use std::collections::HashSet;
use std::env::temp_dir;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Cursor, Write};
use std::path::PathBuf;
use std::sync::{Arc, atomic::Ordering, RwLock};
use std::thread;
use std::time::SystemTime;

use rpfm_extensions::dependencies::*;
use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::gltf::{gltf_from_rigid, save_gltf_to_disk};
use rpfm_extensions::optimizer::OptimizableContainer;
use rpfm_extensions::translator::PackTranslation;

use rpfm_ipc::{MYMOD_BASE_PATH, SECONDARY_PATH, helpers::*};

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{animpack::AnimPack, Container, ContainerPath, db::DB, DecodeableExtraData, EncodeableExtraData, FileType, loc::Loc, pack::*, portrait_settings::PortraitSettings, RFile, RFileDecoded, table::{DecodedData, Table}, text::*};
use rpfm_lib::games::{GameInfo, LUA_REPO, LUA_BRANCH, LUA_REMOTE, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE, pfh_file_type::PFHFileType, supported_games::*, VanillaDBTableNameLogic};
use rpfm_lib::games::{TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE};
use rpfm_lib::integrations::{assembly_kit::*, git::*, log::*};
use rpfm_lib::schema::*;
use rpfm_lib::utils::*;

use crate::*;
use crate::settings::*;
use crate::updater;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub const VANILLA_LOC_NAME: &str = "vanilla_english.tsv";
pub const VANILLA_FIXES_NAME: &str = "vanilla_fixes_";

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub async fn background_loop(mut receiver: UnboundedReceiver<(UnboundedSender<Response>, Command)>) {

    //---------------------------------------------------------------------------------------//
    // Initializing stuff...
    //---------------------------------------------------------------------------------------//

    // We need two PackFiles:
    // - `pack_file_decoded`: This one will hold our opened PackFile.
    // - `pack_files_decoded_extra`: This one will hold the PackFiles opened for the `add_from_packfile` feature, using their paths as keys.
    let mut pack_file_decoded = Pack::default();
    let mut pack_files_decoded_extra = BTreeMap::new();

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

            // When we want to check if there is a schema's update available...
            Command::CheckSchemaUpdates => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match schemas_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
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

            // When we want to check if there is a lua setup update available...
            Command::CheckLuaAutogenUpdates => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match lua_autogen_base_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, LUA_REPO, LUA_BRANCH, LUA_REMOTE);
                                git_integration.check_update().map_err(|e| e.into())
                            },
                            Err(error) => Err(error),
                        }
                    }).await.unwrap();

                    match result {
                        Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseGit(response)),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            Command::CheckEmpireAndNapoleonAKUpdates => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match old_ak_files_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE);
                                git_integration.check_update().map_err(|e| e.into())
                            },
                            Err(error) => Err(error),
                        }
                    }).await.unwrap();

                    match result {
                        Ok(response) => CentralCommand::send_back(&sender, Response::APIResponseGit(response)),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                });
            }

            Command::CheckTranslationsUpdates => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(|| {
                        match translations_remote_path() {
                            Ok(local_path) => {
                                let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
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

            // In case we want to reset the PackFile to his original state (dummy)...
            Command::ResetPackFile => pack_file_decoded = Pack::default(),

            // In case we want to remove a Secondary Packfile from memory...
            Command::RemovePackFileExtra(path) => { pack_files_decoded_extra.remove(&path); },

            // In case we want to create a "New PackFile"...
            Command::NewPackFile => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let pack_version = game_selected.pfh_version_by_file_type(PFHFileType::Mod);
                pack_file_decoded = Pack::new_with_name_and_version("unknown.pack", pack_version);

                if let Some(version_number) = game_selected.game_version_number(&settings.path_buf(game_selected.key())) {
                    pack_file_decoded.set_game_version(version_number);
                }
            }

            // In case we want to "Open one or more PackFiles"...
            Command::OpenPackFiles(paths) => {
                let game_selected = GAME_SELECTED.read().unwrap().clone();
                match Pack::read_and_merge(&paths, &game_selected, settings.bool("use_lazy_loading"), false, false) {
                    Ok(pack) => {
                        pack_file_decoded = pack;

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = pack_file_decoded.files_by_type_mut(&[FileType::DB, FileType::Loc]);
                            files.par_iter_mut().for_each(|file| {
                                let _ = file.decode(&extra_data, true, false);
                            });
                        }

                        CentralCommand::send_back(&sender, Response::ContainerInfo(ContainerInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to "Open an Extra PackFile" (for "Add from PackFile")...
            Command::OpenPackExtra(path) => {
                let game_selected = GAME_SELECTED.read().unwrap().clone();
                match pack_files_decoded_extra.get(&path) {
                    Some(pack) => CentralCommand::send_back(&sender, Response::ContainerInfo(ContainerInfo::from(pack))),
                    None => match Pack::read_and_merge(&[path.to_path_buf()], &game_selected, true, false, false) {
                         Ok(pack) => {
                            CentralCommand::send_back(&sender, Response::ContainerInfo(ContainerInfo::from(&pack)));
                            pack_files_decoded_extra.insert(path.to_path_buf(), pack);
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                }
            }

            // In case we want to "Load All CA PackFiles"...
            Command::LoadAllCAPackFiles => {
                let game_selected = GAME_SELECTED.read().unwrap();
                match Pack::read_and_merge_ca_packs(&game_selected, &settings.path_buf(game_selected.key())) {
                    Ok(pack) => {
                        pack_file_decoded = pack;

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = pack_file_decoded.files_by_type_mut(&[FileType::DB, FileType::Loc]);
                            files.par_iter_mut().for_each(|file| {
                                let _ = file.decode(&extra_data, true, false);
                            });
                        }

                        CentralCommand::send_back(&sender, Response::ContainerInfo(ContainerInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to "Save a PackFile"...
            Command::SavePackFile => {
                let game = GAME_SELECTED.read().unwrap();
                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, pack_file_decoded.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));

                let pack_type = *pack_file_decoded.header().pfh_file_type();
                if !settings.bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.").to_string()));
                    continue;
                }

                match pack_file_decoded.save(None, &game, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                }
            }

            // In case we want to "Save a PackFile As"...
            Command::SavePackFileAs(path) => {
                let game = GAME_SELECTED.read().unwrap();
                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, pack_file_decoded.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));

                let pack_type = *pack_file_decoded.header().pfh_file_type();
                if !settings.bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.").to_string()));
                    continue;
                }

                match pack_file_decoded.save(Some(&path), &game, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                }
            }

            // If you want to perform a clean&save over a PackFile...
            Command::CleanAndSavePackFileAs(path) => {
                pack_file_decoded.clean_undecoded();

                let game = GAME_SELECTED.read().unwrap();
                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, pack_file_decoded.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                match pack_file_decoded.save(Some(&path), &game, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error).to_string())),
                }
            }

            // In case we want to get the data of a PackFile needed to form the TreeView...
            Command::GetPackFileDataForTreeView => {

                // Get the name and the PackedFile list, and send it.
                CentralCommand::send_back(&sender, Response::ContainerInfoVecRFileInfo((
                    From::from(&pack_file_decoded),
                    pack_file_decoded.files().par_iter().map(|(_, file)| From::from(file)).collect(),

                )));
            }

            // In case we want to get the data of a Secondary PackFile needed to form the TreeView...
            Command::GetPackFileExtraDataForTreeView(path) => {

                // Get the name and the PackedFile list, and serialize it.
                match pack_files_decoded_extra.get(&path) {
                    Some(pack_file) => CentralCommand::send_back(&sender, Response::ContainerInfoVecRFileInfo((
                        From::from(pack_file),
                        pack_file.files().par_iter().map(|(_, file)| From::from(file)).collect(),
                    ))),
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Cannot find extra PackFile with path: {}", path.to_string_lossy()).to_string())),
                }
            }

            // In case we want to get the info of one PackedFile from the TreeView.
            Command::GetRFileInfo(path) => {
                CentralCommand::send_back(&sender, Response::OptionRFileInfo(
                    pack_file_decoded.files().get(&path).map(From::from)
                ));
            }

            // In case we want to get the info of more than one PackedFiles from the TreeView.
            Command::GetPackedFilesInfo(paths) => {
                let paths = paths.iter().map(|path| ContainerPath::File(path.to_owned())).collect::<Vec<_>>();
                CentralCommand::send_back(&sender, Response::VecRFileInfo(
                    pack_file_decoded.files_by_paths(&paths, false).into_iter().map(From::from).collect()
                ));
            }

            // In case we want to launch a global search on a `PackFile`...
            Command::GlobalSearch(mut global_search) => {
                let game_selected = GAME_SELECTED.read().unwrap();
                match *SCHEMA.read().unwrap() {
                    Some(ref schema) => {
                        global_search.search(&game_selected, schema, &mut pack_file_decoded, &mut dependencies.write().unwrap(), &[]);
                        let packed_files_info = RFileInfo::info_from_global_search(&global_search, &pack_file_decoded);
                        CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), packed_files_info));
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string())),
                }
            }

            Command::SetGameSelected(game_selected, rebuild_dependencies) => {
                info!("Setting game selected.");
                let game_changed = GAME_SELECTED.read().unwrap().key() != game_selected || !FIRST_GAME_CHANGE_DONE.load(Ordering::SeqCst);
                *GAME_SELECTED.write().unwrap() = SUPPORTED_GAMES.game(&game_selected).unwrap();
                let game = GAME_SELECTED.read().unwrap();

                // We need to make sure the compression format is valid for our game.
                let current_cf = pack_file_decoded.compression_format();
                if current_cf != CompressionFormat::None && !game.compression_formats_supported().contains(&current_cf) {
                    if let Some(new_cf) = game.compression_formats_supported().first() {
                        pack_file_decoded.set_compression_format(*new_cf, &game);
                    } else {
                        pack_file_decoded.set_compression_format(CompressionFormat::None, &game);
                    }
                }

                // Optimisation: If we know we need to rebuild the whole dependencies, load them in another thread
                // while we load the schema. That way we can speed-up the entire game-switching process.
                //
                // While this is fast, the rust compiler doesn't like the fact that we're moving out the dependencies,
                // then moving them back in an if, so we need two branches of code, depending on if rebuild is true or not.
                //
                // Branch 1: dependencies rebuilt.
                if rebuild_dependencies {
                    info!("Branch 1.");
                    let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();
                    // Get settings values before spawning thread since settings can't be moved into closure
                    let game = GAME_SELECTED.read().unwrap().clone();
                    let game_path = settings.path_buf(game.key());
                    let secondary_path = settings.path_buf(SECONDARY_PATH);

                    let handle = thread::spawn(move || {
                        let game_selected = GAME_SELECTED.read().unwrap();
                        let file_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                        let file_path = if game_changed { Some(&*file_path) } else { None };
                        let _ = dependencies.write().unwrap().rebuild(&None, &pack_dependencies, file_path, &game_selected, &game_path, &secondary_path);
                        dependencies
                    });

                    // Load the new schemas.
                    load_schemas(&mut pack_file_decoded, &game, &settings);

                    // Get the dependencies that were loading in parallel and send their info to the UI.
                    dependencies = handle.join().unwrap();
                    let dependencies_info = DependenciesInfo::new(&*dependencies.read().unwrap(), &GAME_SELECTED.read().unwrap().vanilla_db_table_name_logic());
                    info!("Sending dependencies info after game selected change.");
                    CentralCommand::send_back(&sender, Response::CompressionFormatDependenciesInfo(pack_file_decoded.compression_format(), Some(dependencies_info)));

                    // Decode the dependencies tables while the UI does its own thing.
                    dependencies.write().unwrap().decode_tables(&SCHEMA.read().unwrap());
                }

                // Branch 2: no dependecies rebuild.
                else {
                    info!("Branch 2.");

                    // Load the new schemas.
                    load_schemas(&mut pack_file_decoded, &game, &settings);
                    CentralCommand::send_back(&sender, Response::CompressionFormatDependenciesInfo(pack_file_decoded.compression_format(), None));
                };

                // If there is a Pack open, change his id to match the one of the new `Game Selected`.
                if !pack_file_decoded.disk_file_path().is_empty() {
                    let pfh_file_type = *pack_file_decoded.header().pfh_file_type();
                    pack_file_decoded.header_mut().set_pfh_version(game.pfh_version_by_file_type(pfh_file_type));

                    if let Some(version_number) = game.game_version_number(&settings.path_buf(game.key())) {
                        pack_file_decoded.set_game_version(version_number);
                    }
                }
                info!("Switching game selected done.");
            }

            // In case we want to generate the dependencies cache for our Game Selected...
            Command::GenerateDependenciesCache => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game_selected.key());
                let ignore_game_files_in_ak = settings.bool("ignore_game_files_in_ak");
                let asskit_path = settings.assembly_kit_path(&game_selected).ok();

                if game_path.is_dir() {
                    let schema = SCHEMA.read().unwrap();
                    match Dependencies::generate_dependencies_cache(&schema, &game_selected, &game_path, &asskit_path, ignore_game_files_in_ak) {
                        Ok(mut cache) => {
                            let dependencies_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                            match cache.save(&dependencies_path) {
                                Ok(_) => {
                                    let secondary_path = settings.path_buf(SECONDARY_PATH);
                                    let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();
                                    let _ = dependencies.write().unwrap().rebuild(&schema, &pack_dependencies, Some(&dependencies_path), &game_selected, &game_path, &secondary_path);
                                    let dependencies_info = DependenciesInfo::new(&*dependencies.read().unwrap(), &GAME_SELECTED.read().unwrap().vanilla_db_table_name_logic());
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

                if let Some(ref mut schema) = *SCHEMA.write().unwrap() {
                    let game_selected = GAME_SELECTED.read().unwrap();
                    match settings.assembly_kit_path(&game_selected) {
                        Ok(asskit_path) => {
                            let schema_path = schemas_path().unwrap().join(game_selected.schema_file_name());

                            let dependencies = dependencies.read().unwrap();
                            if let Ok(mut tables_to_check) = dependencies.db_and_loc_data(true, false, true, false) {

                                // If there's a pack open, also add the pack's tables to it. That way we can treat some special tables, like starpos tables.
                                if !pack_file_decoded.disk_file_path().is_empty() {
                                    tables_to_check.append(&mut pack_file_decoded.files_by_type(&[FileType::DB]));
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

                                match update_schema_from_raw_files(schema, &game_selected, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split) {
                                    Ok(possible_loc_fields) => {

                                        // NOTE: This deletes all loc fields first, so we need to get the loc fields AGAIN after this from the TExc_LocalisableFields.xml, if said file exists and it's readable.
                                        // That's why it does the update again, to re-populate the loc fields list with the ones not bruteforced. It's ineficient, but gets the job done.
                                        if dependencies.bruteforce_loc_key_order(schema, possible_loc_fields, Some(&pack_file_decoded), None).is_ok() {

                                            // Note: this shows the list of "missing" fields.
                                            let _ = update_schema_from_raw_files(schema, &game_selected, &asskit_path, &schema_path, &tables_to_skip, &tables_to_check_split);

                                            // This generates the automatic patches in the schema (like ".png are files" kinda patches).
                                            if dependencies.generate_automatic_patches(schema, &pack_file_decoded).is_ok() {

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

                                                match schema.save(&schemas_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name())) {
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
            Command::OptimizePackFile(options) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let game_info = GAME_SELECTED.read().unwrap();
                    match pack_file_decoded.optimize(None, &mut dependencies.write().unwrap(), schema, &game_info, &options) {
                        Ok((paths_to_delete, paths_to_add)) => CentralCommand::send_back(&sender, Response::HashSetStringHashSetString(paths_to_delete, paths_to_add)),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                }
            }

            // In case we want to Patch the SiegeAI of a PackFile...
            Command::PatchSiegeAI => {
                match pack_file_decoded.patch_siege_ai() {
                    Ok(result) => CentralCommand::send_back(&sender, Response::StringVecContainerPath(result.0, result.1)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string()))
                }
            }

            // In case we want to change the PackFile's Type...
            Command::SetPackFileType(new_type) => pack_file_decoded.set_pfh_file_type(new_type),

            // In case we want to change the "Include Last Modified Date" setting of the PackFile...
            Command::ChangeIndexIncludesTimestamp(state) => {
                let mut bitmask = pack_file_decoded.bitmask();
                bitmask.set(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, state);
                pack_file_decoded.set_bitmask(bitmask);
            },

            // In case we want to compress/decompress the PackedFiles of the currently open PackFile...
            Command::ChangeCompressionFormat(cf) => {
                let gs = GAME_SELECTED.read().unwrap().clone();
                CentralCommand::send_back(&sender, Response::CompressionFormat(pack_file_decoded.set_compression_format(cf, &gs)));
            },

            // In case we want to get the path of the currently open `PackFile`.
            Command::GetPackFilePath => CentralCommand::send_back(&sender, Response::PathBuf(PathBuf::from(pack_file_decoded.disk_file_path()))),

            // In case we want to get the Dependency PackFiles of our PackFile...
            Command::GetDependencyPackFilesList => CentralCommand::send_back(&sender, Response::VecBoolString(pack_file_decoded.dependencies().to_vec())),

            // In case we want to set the Dependency PackFiles of our PackFile...
            Command::SetDependencyPackFilesList(packs) => { pack_file_decoded.set_dependencies(packs); },

            // In case we want to check if there is a Dependency Database loaded...
            Command::IsThereADependencyDatabase(include_asskit) => {
                let are_dependencies_loaded = dependencies.read().unwrap().is_vanilla_data_loaded(include_asskit);
                CentralCommand::send_back(&sender, Response::Bool(are_dependencies_loaded))
            },

            // In case we want to create a PackedFile from scratch...
            Command::NewPackedFile(path, new_packed_file) => {
                let decoded = match new_packed_file {
                    NewFile::AnimPack(_) => {
                        let file = AnimPack::default();
                        RFileDecoded::AnimPack(file)
                    },
                    NewFile::DB(_, table, version) => {
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
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
                            CentralCommand::send_back(&sender, Response::Error(format!("There is no Schema for the Game Selected.")));
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
                match pack_file_decoded.insert(file) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // When we want to add one or more PackedFiles to our PackFile.
            Command::AddPackedFiles(source_paths, destination_paths, paths_to_ignore) => {
                let mut added_paths = vec![];
                let mut it_broke = None;

                let paths = source_paths.iter().zip(destination_paths.iter()).collect::<Vec<(&PathBuf, &ContainerPath)>>();
                let schema = SCHEMA.read().unwrap();
                for (source_path, destination_path) in paths {

                    // Skip ignored paths.
                    if let Some(ref paths_to_ignore) = paths_to_ignore {
                        if paths_to_ignore.iter().any(|x| source_path.starts_with(x)) {
                            continue;
                        }
                    }

                    match destination_path {
                        ContainerPath::File(destination_path) => {
                            match pack_file_decoded.insert_file(source_path, &destination_path, &schema) {
                                Ok(path) => if let Some(path) = path {
                                    added_paths.push(path);
                                },
                                Err(error) => it_broke = Some(error),
                            }
                        },

                        // TODO: See what should we do with the ignored paths.
                        ContainerPath::Folder(destination_path) => {
                            match pack_file_decoded.insert_folder(source_path, &destination_path, &None, &schema, settings.bool("include_base_folder_on_add_from_folder")) {
                                Ok(mut paths) => added_paths.append(&mut paths),
                                Err(error) => it_broke = Some(error),
                            }
                        },
                    }
                }

                if let Some(error) = it_broke {
                    CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths.to_vec()));
                    CentralCommand::send_back(&sender, Response::Error(error.to_string()));
                } else {
                    CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths.to_vec()));
                    CentralCommand::send_back(&sender, Response::Success);
                }

                // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let mut decode_extra_data = DecodeableExtraData::default();
                    decode_extra_data.set_schema(Some(schema));
                    let extra_data = Some(decode_extra_data);

                    let mut files = pack_file_decoded.files_by_paths_mut(&added_paths, false);
                    files.par_iter_mut()
                        .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                        .for_each(|file| {
                            let _ = file.decode(&extra_data, true, false);
                        }
                    );
                }
            }

            // In case we want to move stuff from one PackFile to another...
            Command::AddPackedFilesFromPackFile((pack_file_path, paths)) => {
                match pack_files_decoded_extra.get(&pack_file_path) {

                    // Try to add the PackedFile to the main PackFile.
                    Some(pack) => {
                        let files = pack.files_by_paths(&paths, false);
                        let mut paths = Vec::with_capacity(files.len());
                        for file in files {
                            if let Ok(Some(path)) = pack_file_decoded.insert(file.clone()) {
                                paths.push(path);
                            }
                        }

                        CentralCommand::send_back(&sender, Response::VecContainerPath(paths.to_vec()));

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            let mut decode_extra_data = DecodeableExtraData::default();
                            decode_extra_data.set_schema(Some(schema));
                            let extra_data = Some(decode_extra_data);

                            let mut files = pack_file_decoded.files_by_paths_mut(&paths, false);
                            files.par_iter_mut()
                                .filter(|file| file.file_type() == FileType::DB || file.file_type() == FileType::Loc)
                                .for_each(|file| {
                                    let _ = file.decode(&extra_data, true, false);
                                }
                            );
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("Cannot find extra PackFile with path: {}", pack_file_path.to_string_lossy()))),
                }
            }

            // In case we want to move stuff from our PackFile to an Animpack...
            Command::AddPackedFilesFromPackFileToAnimpack(anim_pack_path, paths) => {
                let files = pack_file_decoded.files_by_paths(&paths, false)
                    .into_iter()
                    .map(|file| {
                        let mut file = file.clone();
                        let _ = file.load();
                        file
                    })
                    .collect::<Vec<RFile>>();

                match pack_file_decoded.files_mut().get_mut(&anim_pack_path) {
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

            // In case we want to move stuff from an Animpack to our PackFile...
            Command::AddPackedFilesFromAnimpack(data_source, anim_pack_path, paths) => {
                let mut dependencies = dependencies.write().unwrap();
                let anim_pack_file = match data_source {
                    DataSource::PackFile => pack_file_decoded.files_mut().get_mut(&anim_pack_path),
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

                let paths = files.iter().map(|file| file.path_in_container()).collect::<Vec<_>>();
                for mut file in files {
                    let _ = file.guess_file_type();
                    let _ = pack_file_decoded.insert(file);
                }

                CentralCommand::send_back(&sender, Response::VecContainerPath(paths));
            }

            // In case we want to delete files from an Animpack...
            Command::DeleteFromAnimpack((anim_pack_path, paths)) => {
                match pack_file_decoded.files_mut().get_mut(&anim_pack_path) {
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

            // In case we want to decode a RigidModel PackedFile...
            Command::DecodePackedFile(path, data_source) => {
                info!("Trying to decode a file. Path: {}", &path);
                info!("Trying to decode a file. Data Source: {}", &data_source);

                match data_source {
                    DataSource::PackFile => {
                        if path == RESERVED_NAME_NOTES {
                            let mut note = Text::default();
                            note.set_format(TextFormat::Markdown);
                            note.set_contents(pack_file_decoded.notes().pack_notes().to_owned());
                            CentralCommand::send_back(&sender, Response::Text(note));
                        }

                        else {

                            // Find the PackedFile we want and send back the response.
                            match pack_file_decoded.files_mut().get_mut(&path) {
                                Some(file) => decode_and_send_file(file, &sender, &settings),
                                None => CentralCommand::send_back(&sender, Response::Error(format!("The file with the path {} hasn't been found on this Pack.", path))),
                            }
                        }
                    }

                    DataSource::ParentFiles => {
                        match dependencies.write().unwrap().file_mut(&path, false, true) {
                            Ok(file) => decode_and_send_file(file, &sender, &settings),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }

                    DataSource::GameFiles => {
                        match dependencies.write().unwrap().file_mut(&path, true, false) {
                            Ok(file) => decode_and_send_file(file, &sender, &settings),
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
                            CentralCommand::send_back(&sender, Response::Error(format!("Path {} doesn't contain an identificable table name.", path)));
                        }
                    }

                    DataSource::ExternalFile => {}
                }
            }

            // When we want to save a PackedFile from the view....
            Command::SavePackedFileFromView(path, file_decoded) => {
                if path == RESERVED_NAME_NOTES {
                    if let RFileDecoded::Text(data) = file_decoded {
                        pack_file_decoded.notes_mut().set_pack_notes(data.contents().to_owned());
                    }
                }
                else if let Some(file) = pack_file_decoded.files_mut().get_mut(&path) {
                    if let Err(error) = file.set_decoded(file_decoded) {
                        CentralCommand::send_back(&sender, Response::Error(error.to_string()));
                    }
                }
                CentralCommand::send_back(&sender, Response::Success);
            }

            // In case we want to delete PackedFiles from a PackFile...
            Command::DeletePackedFiles(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths.iter().flat_map(|path| pack_file_decoded.remove(path)).collect())),

            // In case we want to extract PackedFiles from a PackFile...
            Command::ExtractPackedFiles(container_paths, path, extract_tables_to_tsv) => {
                let schema = SCHEMA.read().unwrap();
                let schema = if extract_tables_to_tsv { &*schema } else { &None };
                let mut errors = 0;

                let game = GAME_SELECTED.read().unwrap();
                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, pack_file_decoded.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                let mut extracted_paths = vec![];

                // Pack extraction.
                if let Some(container_paths) = container_paths.get(&DataSource::PackFile) {
                    for container_path in container_paths {
                        match pack_file_decoded.extract(container_path.clone(), &path, true, schema, false, settings.bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
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
            Command::RenamePackedFiles(renaming_data) => {
                match pack_file_decoded.move_paths(&renaming_data) {
                    Ok(data) => CentralCommand::send_back(&sender, Response::VecContainerPathContainerPath(data)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to know if a Folder exists, knowing his path...
            Command::FolderExists(path) => {
                CentralCommand::send_back(&sender, Response::Bool(pack_file_decoded.has_folder(&path)));
            }

            // In case we want to know if PackedFile exists, knowing his path...
            Command::PackedFileExists(path) => {
                CentralCommand::send_back(&sender, Response::Bool(pack_file_decoded.has_file(&path)));
            }

            // In case we want to get the list of tables in the dependency database...
            Command::GetTableListFromDependencyPackFile => {
                let dependencies = dependencies.read().unwrap();
                CentralCommand::send_back(&sender, Response::VecString(dependencies.vanilla_loose_tables().keys().chain(dependencies.vanilla_tables().keys()).map(|x| x.to_owned()).collect()))
            },
            Command::GetCustomTableList => match &*SCHEMA.read().unwrap() {
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

            Command::LocalArtSetIds => CentralCommand::send_back(&sender, Response::HashSetString(dependencies.read().unwrap().db_values_from_table_name_and_column_name(Some(&pack_file_decoded), "campaign_character_arts_tables", "art_set_id", false, false))),

            // TODO: This needs to use a list pulled from portrait settings files, not from a table.
            Command::DependenciesArtSetIds => CentralCommand::send_back(&sender, Response::HashSetString(dependencies.read().unwrap().db_values_from_table_name_and_column_name(None, "campaign_character_arts_tables", "art_set_id", true, true))),

            // In case we want to get the version of an specific table from the dependency database...
            Command::GetTableVersionFromDependencyPackFile(table_name) => {
                if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                    match dependencies.read().unwrap().db_version(&table_name) {
                        Some(version) => CentralCommand::send_back(&sender, Response::I32(version)),
                        None => {

                            // If the table is one of the starpos tables, we need to return the latest version of the table, even if it's not in the game files.
                            if table_name.starts_with("start_pos_") || table_name.starts_with("twad_") {
                                match &*SCHEMA.read().unwrap() {
                                    Some(schema) => {
                                        match schema.definitions_by_table_name(&table_name) {
                                            Some(definitions) => {
                                                if definitions.is_empty() {
                                                    CentralCommand::send_back(&sender, Response::Error(format!("There are no definitions for this specific table.")));
                                                } else {
                                                    CentralCommand::send_back(&sender, Response::I32(*definitions.first().unwrap().version()));
                                                }
                                            }
                                            None => CentralCommand::send_back(&sender, Response::Error(format!("There are no definitions for this specific table."))),
                                        }
                                    }
                                    None => CentralCommand::send_back(&sender, Response::Error(format!("There is no Schema for the Game Selected.").to_string()))
                                }
                            } else {
                                CentralCommand::send_back(&sender, Response::Error(format!("Table not found in the game files.")))
                            }
                        },
                    }
                } else { CentralCommand::send_back(&sender, Response::Error(format!("Dependencies cache needs to be regenerated before this.").to_string())); }
            }

            Command::GetTableDefinitionFromDependencyPackFile(table_name) => {
                if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        if let Some(version) = dependencies.read().unwrap().db_version(&table_name) {
                            if let Some(definition) = schema.definition_by_name_and_version(&table_name, version) {
                                CentralCommand::send_back(&sender, Response::Definition(definition.clone()));
                            } else { CentralCommand::send_back(&sender, Response::Error(format!("No definition found for table {}.", table_name).to_string())); }
                        } else { CentralCommand::send_back(&sender, Response::Error(format!("Table version not found in dependencies for table {}.", table_name).to_string())); }
                    } else { CentralCommand::send_back(&sender, Response::Error(format!("There is no Schema for the Game Selected.").to_string())); }
                } else { CentralCommand::send_back(&sender, Response::Error(format!("Dependencies cache needs to be regenerated before this.").to_string())); }
            }

            // In case we want to merge DB or Loc Tables from a PackFile...
            Command::MergeFiles(paths, merged_path, delete_source_files) => {
                let files_to_merge = pack_file_decoded.files_by_paths(&paths, false);
                match RFile::merge(&files_to_merge, &merged_path) {
                    Ok(file) => {
                        let _ = pack_file_decoded.insert(file);

                        // Make sure to only delete the files if they're not the destination file.
                        if delete_source_files {
                            paths.iter()
                                .filter(|path| merged_path != path.path_raw())
                                .for_each(|path| { pack_file_decoded.remove(path); });
                        }

                        CentralCommand::send_back(&sender, Response::String(merged_path.to_string()));
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to update a table...
            Command::UpdateTable(path) => {
                let path = path.path_raw();
                if let Some(rfile) = pack_file_decoded.file_mut(path, false) {
                    if let Ok(decoded) = rfile.decoded_mut() {
                        match dependencies.write().unwrap().update_db(decoded) {
                            Ok((old_version, new_version, fields_deleted, fields_added)) => CentralCommand::send_back(&sender, Response::I32I32VecStringVecString(old_version, new_version, fields_deleted, fields_added)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path undecoded: {}", path).to_string())); }
                } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found in the open Pack: {}", path).to_string())); }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceMatches(mut global_search, matches) => {
                let game_info = GAME_SELECTED.read().unwrap();
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    match global_search.replace(&game_info, schema, &mut pack_file_decoded, &mut dependencies.write().unwrap(), &matches) {
                        Ok(paths) => {
                            let files_info = paths.iter().flat_map(|path| pack_file_decoded.files_by_path(path, false).iter().map(|file| RFileInfo::from(*file)).collect::<Vec<RFileInfo>>()).collect();
                            CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), files_info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string()));
                }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceAll(mut global_search) => {
                let game_info = GAME_SELECTED.read().unwrap();
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    match global_search.replace_all(&game_info, schema, &mut pack_file_decoded, &mut dependencies.write().unwrap()) {
                        Ok(paths) => {
                            let files_info = paths.iter().flat_map(|path| pack_file_decoded.files_by_path(path, false).iter().map(|file| RFileInfo::from(*file)).collect::<Vec<RFileInfo>>()).collect();
                            CentralCommand::send_back(&sender, Response::GlobalSearchVecRFileInfo(Box::new(global_search), files_info));
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?").to_string()));
                }
            }

            // In case we want to get the reference data for a definition...
            Command::GetReferenceDataFromDefinition(table_name, definition, force_local_ref_generation) => {
                let mut reference_data = HashMap::new();

                // Only generate the cache references if we don't already have them generated.
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    if dependencies.read().unwrap().local_tables_references().get(&table_name).is_none() || force_local_ref_generation {
                        dependencies.write().unwrap().generate_local_definition_references(schema, &table_name, &definition);
                    }

                    reference_data = dependencies.read().unwrap().db_reference_data(schema, &pack_file_decoded, &table_name, &definition, &None);
                }

                CentralCommand::send_back(&sender, Response::HashMapI32TableReferences(reference_data));
            }

            // In case we want to change the format of a ca_vp8 video...
            Command::SetVideoFormat(path, format) => {
                match pack_file_decoded.files_mut().get_mut(&path) {
                    Some(ref mut rfile) => {
                        match rfile.decoded_mut() {
                            Ok(data) => {
                                if let RFileDecoded::Video(ref mut data) = data {
                                    data.set_format(format);
                                }
                                // TODO: Put an error here.
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(format!("This Pack doesn't exists as a file in the disk."))),
                }
            },

            // In case we want to save an schema to disk...
            Command::SaveSchema(mut schema) => {
                match schema.save(&schemas_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name())) {
                    Ok(_) => {
                        *SCHEMA.write().unwrap() = Some(schema);
                        CentralCommand::send_back(&sender, Response::Success);
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // In case we want to clean the cache of one or more PackedFiles...
            Command::CleanCache(paths) => {
                let cf = pack_file_decoded.compression_format();
                let mut files = pack_file_decoded.files_by_paths_mut(&paths, false);
                let game = GAME_SELECTED.read().unwrap();
                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                files.iter_mut().for_each(|file| {
                    let _ = file.encode(&extra_data, true, true, false);
                });
            }

            // In case we want to export a PackedFile as a TSV file...
            Command::ExportTSV(internal_path, external_path, data_source) => {
                let mut dependencies = dependencies.write().unwrap();
                let schema = SCHEMA.read().unwrap();
                match &*schema {
                    Some(ref schema) => {
                        let file = match data_source {
                            DataSource::PackFile => pack_file_decoded.file_mut(&internal_path, false),
                            DataSource::ParentFiles => dependencies.file_mut(&internal_path, false, true).ok(),
                            DataSource::GameFiles => dependencies.file_mut(&internal_path, true, false).ok(),
                            DataSource::AssKitFiles => {
                                CentralCommand::send_back(&sender, Response::Error(format!("Exporting a TSV from the Assembly Kit is not yet supported.")));
                                continue;
                            },
                            DataSource::ExternalFile => {
                                CentralCommand::send_back(&sender, Response::Error(format!("Exporting a TSV from a external file is not yet supported.")));
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
                    None => CentralCommand::send_back(&sender, Response::Error(format!("There is no Schema for the Game Selected.").to_string())),
                }
            }

            // In case we want to import a TSV as a PackedFile...
            // TODO: This is... unreliable at best, can break stuff at worst. Replace the set_decoded with proper type checking.
            Command::ImportTSV(internal_path, external_path) => {
                match pack_file_decoded.file_mut(&internal_path, false) {
                    Some(file) => {
                        let schema = SCHEMA.read().unwrap();
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

            // In case we want to open a PackFile's location in the file manager...
            Command::OpenContainingFolder => {

                // If the path exists, try to open it. If not, throw an error.
                let mut path_str = pack_file_decoded.disk_file_path().to_owned();

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
                    CentralCommand::send_back(&sender, Response::Error(format!("This Pack doesn't exists as a file in the disk.")));
                }
            },

            // When we want to open a PackedFile in a external program...
            Command::OpenPackedFileInExternalProgram(data_source, path) => {
                match data_source {
                    DataSource::PackFile => {
                        let folder = temp_dir().join(format!("rpfm_{}", pack_file_decoded.disk_file_name()));
                        let game = GAME_SELECTED.read().unwrap();
                        let cf = pack_file_decoded.compression_format();
                        let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                        match pack_file_decoded.extract(path.clone(), &folder, true, &SCHEMA.read().unwrap(), false, settings.bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                            Ok(extracted_path) => {
                                let _ = that(&extracted_path[0]);
                                CentralCommand::send_back(&sender, Response::PathBuf(extracted_path[0].to_owned()));
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("Opening dependencies files in external programs is not yet supported.").to_string())),
                }
            }

            // When we want to save a PackedFile from the external view....
            Command::SavePackedFileFromExternalView(path, external_path) => {
                let schema = SCHEMA.read().unwrap();
                match pack_file_decoded.file_mut(&path, false) {
                    Some(file) => match file.encode_from_external_data(&schema, &external_path) {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found").to_string())),
                }
            }

            // When we want to update our schemas...
            Command::UpdateSchemas => {
                match schemas_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => {
                                let game = GAME_SELECTED.read().unwrap();
                                let schema_path = schemas_path().unwrap().join(game.schema_file_name());
                                let patches_path = table_patches_path().unwrap().join(game.schema_file_name());

                                // Encode the decoded tables with the old schema, then re-decode them with the new one.
                                let cf = pack_file_decoded.compression_format();
                                let mut tables = pack_file_decoded.files_by_type_mut(&[FileType::DB]);
                                let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

                                tables.par_iter_mut().for_each(|x| { let _ = x.encode(&extra_data, true, true, false); });

                                *SCHEMA.write().unwrap() = Schema::load(&schema_path, Some(&patches_path)).ok();

                                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                                    let mut extra_data = DecodeableExtraData::default();
                                    extra_data.set_schema(Some(schema));
                                    let extra_data = Some(extra_data);

                                    tables.par_iter_mut().for_each(|x| {
                                        let _ = x.decode(&extra_data, true, false);
                                    });

                                    // Then rebuild the dependencies stuff.
                                    if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                                        let game_path = settings.path_buf(game.key());
                                        let secondary_path = settings.path_buf(SECONDARY_PATH);
                                        let dependencies_file_path = dependencies_cache_path().unwrap().join(game.dependencies_cache_file_name());
                                        let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();

                                        match dependencies.write().unwrap().rebuild(&SCHEMA.read().unwrap(), &pack_dependencies, Some(&*dependencies_file_path), &game, &game_path, &secondary_path) {
                                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                            Err(_) => CentralCommand::send_back(&sender, Response::Error(format!("Schema updated, but dependencies cache rebuilding failed. You may need to regenerate it."))),
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
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // When we want to update our lua setup...
            Command::UpdateLuaAutogen => {
                match lua_autogen_base_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, LUA_REPO, LUA_BRANCH, LUA_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // When we want to update our program...
            Command::UpdateMainProgram => {
                match crate::updater::update_main_program(&settings) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            // When we want to update our program...
            Command::TriggerBackupAutosave => {
                let folder = backup_autosave_path().unwrap().join(pack_file_decoded.disk_file_name());
                let _ = DirBuilder::new().recursive(true).create(&folder);

                // Note: we no longer notify the UI of success or error to not hang it up.
                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game_selected.key());
                let ca_paths = game_selected.ca_packs_paths(&game_path)
                    .unwrap_or_default()
                    .iter()
                    .map(|path| path.to_string_lossy().replace('\\', "/"))
                    .collect::<Vec<_>>();

                let pack_disable_autosaves = pack_file_decoded.settings().setting_bool("disable_autosaves")
                    .unwrap_or(&true);

                let pack_type = pack_file_decoded.pfh_file_type();
                let pack_path = pack_file_decoded.disk_file_path().replace('\\', "/");

                // Do not autosave vanilla packs, packs with autosave disabled, or non-mod or movie packs.
                if folder.is_dir() &&
                    !pack_disable_autosaves &&
                    (pack_type == PFHFileType::Mod || pack_type == PFHFileType::Movie) &&
                    (ca_paths.is_empty() || !ca_paths.contains(&pack_path))
                {
                    let date = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                    let new_name = format!("{date}.pack");
                    let new_path = folder.join(new_name);
                    let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game_selected, pack_file_decoded.compression_format(), settings.bool("disable_uuid_regeneration_on_db_tables")));
                    let _ = pack_file_decoded.clone().save(Some(&new_path), &game_selected, &extra_data);

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
            }

            // In case we want to perform a diagnostics check...
            Command::DiagnosticsCheck(diagnostics_ignored, check_ak_only_refs) => {

                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game_selected.key());

                let mut diagnostics = Diagnostics::default();
                *diagnostics.diagnostics_ignored_mut() = diagnostics_ignored;

                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    if pack_file_decoded.pfh_file_type() == PFHFileType::Mod ||
                        pack_file_decoded.pfh_file_type() == PFHFileType::Movie {
                        diagnostics.check(&mut pack_file_decoded, &mut dependencies.write().unwrap(), schema, &game_selected, &game_path, &[], check_ak_only_refs);
                    }
                }

                info!("Checking diagnostics: done.");

                CentralCommand::send_back(&sender, Response::Diagnostics(diagnostics));
            }

            Command::DiagnosticsUpdate(mut diagnostics, path_types, check_ak_only_refs) => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game_selected.key());

                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    if pack_file_decoded.pfh_file_type() == PFHFileType::Mod ||
                        pack_file_decoded.pfh_file_type() == PFHFileType::Movie {
                        diagnostics.check(&mut pack_file_decoded, &mut dependencies.write().unwrap(), schema, &game_selected, &game_path, &path_types, check_ak_only_refs);
                    }
                }

                info!("Checking diagnostics (update): done.");

                CentralCommand::send_back(&sender, Response::Diagnostics(diagnostics));
            }

            // In case we want to get the open PackFile's Settings...
            Command::GetPackSettings => CentralCommand::send_back(&sender, Response::PackSettings(pack_file_decoded.settings().clone())),
            Command::SetPackSettings(settings) => { pack_file_decoded.set_settings(settings); }

            Command::GetMissingDefinitions => {

                // Test to see if every DB Table can be decoded. This is slow and only useful when
                // a new patch lands and you want to know what tables you need to decode.
                let mut counter = 0;
                let mut table_list = String::new();
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let mut extra_data = DecodeableExtraData::default();
                    extra_data.set_schema(Some(schema));
                    let extra_data = Some(extra_data);

                    let mut files = pack_file_decoded.files_by_type_mut(&[FileType::DB]);
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
                let path = PROGRAM_PATH.to_path_buf().join(PathBuf::from("missing_table_definitions.txt"));

                if let Ok(file) = File::create(path) {
                    let mut file = BufWriter::new(file);
                    let _ = file.write_all(table_list.as_bytes());
                }
            }

            // Ignore errors for now.
            Command::RebuildDependencies(rebuild_only_current_mod_dependencies) => {
                if SCHEMA.read().unwrap().is_some() {
                    let game_selected = GAME_SELECTED.read().unwrap();
                    let game_path = settings.path_buf(game_selected.key());
                    let dependencies_file_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                    let file_path = if !rebuild_only_current_mod_dependencies { Some(&*dependencies_file_path) } else { None };
                    let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();

                    let secondary_path = settings.path_buf(SECONDARY_PATH);
                    let _ = dependencies.write().unwrap().rebuild(&SCHEMA.read().unwrap(), &pack_dependencies, file_path, &game_selected, &game_path, &secondary_path);
                    let dependencies_info = DependenciesInfo::new(&*dependencies.read().unwrap(), &GAME_SELECTED.read().unwrap().vanilla_db_table_name_logic());
                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                }
            },

            Command::CascadeEdition(table_name, definition, changes) => {
                let edited_paths = if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    changes.iter().flat_map(|(field, value_before, value_after)| {
                        DB::cascade_edition(&mut pack_file_decoded, schema, &table_name, field, &definition, value_before, value_after)
                    }).collect::<Vec<_>>()
                } else { vec![] };

                let packed_files_info = pack_file_decoded.files_by_paths(&edited_paths, false).into_par_iter().map(From::from).collect();
                CentralCommand::send_back(&sender, Response::VecContainerPathVecRFileInfo(edited_paths, packed_files_info));
            },

            Command::GetTablesByTableName(table_name) => {
                let path = ContainerPath::Folder(format!("db/{table_name}/"));
                let files = pack_file_decoded.files_by_type_and_paths(&[FileType::DB], &[path], true);
                let paths = files.iter()
                    .map(|x| x.path_in_container_raw().to_owned())
                    .collect::<Vec<_>>();

                CentralCommand::send_back(&sender, Response::VecString(paths));
            },

            Command::AddKeysToKeyDeletes(table_file_name, key_table_name, keys) => {
                let path = ContainerPath::File(format!("db/{KEY_DELETES_TABLE_NAME}/{table_file_name}"));
                let mut files = pack_file_decoded.files_by_type_and_paths_mut(&[FileType::DB], &[path], true);

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

            Command::GoToDefinition(ref_table, mut ref_column, ref_data) => {
                let table_name = format!("{ref_table}_tables");
                let table_folder = format!("db/{table_name}");
                let packed_files = pack_file_decoded.files_by_path(&ContainerPath::Folder(table_folder.to_owned()), true);
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

            Command::SearchReferences(reference_map, value) => {
                let paths = reference_map.keys().map(|x| ContainerPath::Folder(format!("db/{x}"))).collect::<Vec<ContainerPath>>();
                let files = pack_file_decoded.files_by_paths(&paths, true);

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

            Command::GoToLoc(loc_key) => {
                let packed_files = pack_file_decoded.files_by_type(&[FileType::Loc]);
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

            Command::GetSourceDataFromLocKey(loc_key) => CentralCommand::send_back(&sender, Response::OptionStringStringVecString(dependencies.read().unwrap().loc_key_source(&loc_key))),
            Command::GetPackFileName => CentralCommand::send_back(&sender, Response::String(pack_file_decoded.disk_file_name())),
            Command::GetPackedFileRawData(path) => {
                let cf = pack_file_decoded.compression_format();
                let game = GAME_SELECTED.read().unwrap();
                match pack_file_decoded.files_mut().get_mut(&path) {
                    Some(ref mut rfile) => {

                        // Make sure it's in memory.
                        match rfile.load() {
                            Ok(_) => match rfile.cached() {
                                Ok(data) => CentralCommand::send_back(&sender, Response::VecU8(data.to_vec())),

                                // If we don't have binary data, it may be decoded. Encode it and return the binary data.
                                //
                                // NOTE: This fucks up the table decoder if the table was badly decoded.
                                Err(_) =>  {
                                    let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(&game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));
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
            },

            Command::ImportDependenciesToOpenPackFile(paths_by_data_source) => {
                let mut added_paths = vec![];
                let mut not_added_paths = vec![];

                let dependencies = dependencies.read().unwrap();
                for (data_source, paths) in &paths_by_data_source {
                    let files = match data_source {
                        DataSource::GameFiles => dependencies.files_by_path(paths, true, false, false),
                        DataSource::ParentFiles => dependencies.files_by_path(paths, false, true, false),
                        DataSource::AssKitFiles => HashMap::new(),
                        _ => {
                            CentralCommand::send_back(&sender, Response::Error(format!("You can't import files from this source.")));
                            CentralCommand::send_back(&sender, Response::Success);
                            continue 'background_loop;
                        },
                    };

                    for file in files.into_values() {
                        let file_path = file.path_in_container_raw().to_owned();
                        let mut file = file.clone();
                        let _ = file.guess_file_type();
                        if let Ok(Some(path)) = pack_file_decoded.insert(file) {
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
                            let schema = SCHEMA.read().unwrap();
                            match &*schema {
                                Some(ref schema) => {
                                    let mut files = vec![];
                                    for path in paths {

                                        // We only have tables. If it's a folder, it's either a table folder, db or the root.
                                        match path {
                                            ContainerPath::Folder(path) => {
                                                let mut path = path.to_owned();

                                                if path.ends_with('/') {
                                                    path.pop();
                                                }

                                                let path_split = path.split('/').collect::<Vec<_>>();
                                                let table_name_logic = GAME_SELECTED.read().unwrap().vanilla_db_table_name_logic();

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
                                                                let mut path = path_split.to_vec();
                                                                path.push(table_file_name);
                                                                let path = path.join("/");

                                                                let file = RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path);
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
                                                            let mut path = path_split.to_vec();
                                                            path.push(table_file_name);
                                                            let path = path.join("/");

                                                            let file = RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path);
                                                            files.push(file);
                                                        },
                                                        Err(_) => not_added_paths.push(path.clone()),
                                                    }
                                                }

                                                // Any other situation is an error.
                                                else {
                                                    CentralCommand::send_back(&sender, Response::Error(format!("No idea how you were able to trigger this.")));
                                                    CentralCommand::send_back(&sender, Response::Success);
                                                    continue 'background_loop;
                                                }

                                            }
                                            ContainerPath::File(path) => {
                                                let table_name = path.split('/').collect::<Vec<_>>()[1];
                                                match dependencies.import_from_ak(table_name, schema) {
                                                    Ok(table) => {
                                                        let file = RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path);
                                                        files.push(file);
                                                    },
                                                    Err(_) => not_added_paths.push(path.clone()),
                                                }
                                            }
                                        }
                                    }

                                    for file in files {
                                        if let Ok(Some(path)) = pack_file_decoded.insert(file) {
                                            added_paths.push(path);
                                        }
                                    }
                                },
                                None => {
                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string()));
                                    CentralCommand::send_back(&sender, Response::Success);
                                    continue 'background_loop;
                                }
                            }
                        },
                        _ => {
                            CentralCommand::send_back(&sender, Response::Error(format!("You can't import files from this source.")));
                            CentralCommand::send_back(&sender, Response::Success);
                            continue 'background_loop;
                        },
                    }
                }

                CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths));
                if not_added_paths.is_empty() {
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::VecString(not_added_paths));
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

                // Get PackedFiles requested from the currently open PackFile, if any.
                let mut packed_files_packfile = HashMap::new();
                for file in pack_file_decoded.files_by_paths(&paths, true) {
                    packed_files_packfile.insert(if force_lowercased_paths { file.path_in_container_raw().to_lowercase() } else { file.path_in_container_raw().to_owned() }, file.clone());
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

                // Get PackedFiles requested from the currently open PackFile, if any.
                let mut packed_files_packfile = HashSet::new();
                for file in pack_file_decoded.files_by_type_mut(&[FileType::Anim]) {
                    if let Ok(Some(RFileDecoded::Anim(anim_file))) = file.decode(&None, false, true) {
                        if anim_file.skeleton_name() == &skeleton_name {
                            packed_files_packfile.insert(file.path_in_container_raw().to_owned());
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

                let local_files = pack_file_decoded.files_by_path(&path, true);
                if !local_files.is_empty() {
                    files.insert(DataSource::PackFile, local_files.into_iter().map(|file| file.path_in_container()).collect());
                }

                // Return the full list of PackedFile names requested, split by source.
                CentralCommand::send_back(&sender, Response::HashMapDataSourceHashSetContainerPath(files));
            },

            Command::SavePackedFilesToPackFileAndClean(files, optimize) => {
                let schema = SCHEMA.read().unwrap();
                match &*schema {
                    Some(ref schema) => {

                        // We receive a list of edited PackedFiles. The UI is the one that takes care of editing them to have the data we want where we want.
                        // Also, the UI is responsible for naming them in case they're new. Here we grab them and directly add them into the PackFile.
                        let mut added_paths = vec![];
                        for file in files {
                            if let Ok(Some(path)) = pack_file_decoded.insert(file) {
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
                            let game_info = GAME_SELECTED.read().unwrap();
                            match pack_file_decoded.optimize(None, &mut dependencies.write().unwrap(), schema, &game_info, &options) {
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
            },

            Command::NotesForPath(path) => CentralCommand::send_back(&sender, Response::VecNote(pack_file_decoded.notes().notes_by_path(&path))),
            Command::AddNote(note) => CentralCommand::send_back(&sender, Response::Note(pack_file_decoded.notes_mut().add_note(note))),
            Command::DeleteNote(path, id) => pack_file_decoded.notes_mut().delete_note(&path, id),

            Command::SaveLocalSchemaPatch(patches) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::new_patch(&patches, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::RemoveLocalSchemaPatchesForTable(table_name) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::remove_patch_for_table(&table_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::RemoveLocalSchemaPatchesForTableAndField(table_name, field_name) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::remove_patch_for_field(&table_name, &field_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }
            Command::ImportSchemaPatch(patch) => {
                match *SCHEMA.write().unwrap() {
                    Some(ref mut schema) => {
                        Schema::add_patch_to_patch_set(schema.patches_mut(), &patch);
                        match schema.save(&schemas_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name())) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.").to_string())),
                }
            }

            Command::GenerateMissingLocData => {
                match dependencies.read().unwrap().generate_missing_loc_data(&mut pack_file_decoded) {
                    Ok(path) => CentralCommand::send_back(&sender, Response::VecContainerPath(path)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::PackMap(tile_maps, tiles) => {
                match *SCHEMA.read().unwrap() {
                    Some(ref schema) => {
                        let game = GAME_SELECTED.read().unwrap().clone();
                        let mut dependencies = dependencies.write().unwrap();
                        let options = settings.optimizer_options();
                        match dependencies.add_tile_maps_and_tiles(&mut pack_file_decoded, &game, schema, options, tile_maps, tiles) {
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
                    CentralCommand::send_back(&sender, Response::Error(format!("MyMod path is not configured. Configure it in the settings and try again.")));
                    continue;
                }

                mymod_path.push(&mod_game);

                // Just in case the folder doesn't exist, we try to create it.
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the MyMod's Game folder: {}.", error.to_string())));
                    continue;
                }

                // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                mymod_path.push(&mod_name);
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the MyMod's Assets folder: {}.", error.to_string())));
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
                    if let Ok(lua_autogen_folder) = lua_autogen_game_path(&GAME_SELECTED.read().unwrap()) {
                        let lua_autogen_folder = lua_autogen_folder.to_string_lossy().to_string().replace('\\', "/");

                        // VSCode support.
                        if vscode_support {
                            let mut vscode_config_path = mymod_path.to_owned();
                            vscode_config_path.push(".vscode");

                            if let Err(error) = DirBuilder::new().recursive(true).create(&vscode_config_path) {
                                CentralCommand::send_back(&sender, Response::Error(format!("Error while creating the VSCode Config folder: {}.", error.to_string())));
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

            Command::LiveExport => {
                let game = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game.key());
                let disable_regen_table_guid = settings.bool("disable_uuid_regeneration_on_db_tables");
                let keys_first = settings.bool("tables_use_old_column_order_for_tsv");
                match pack_file_decoded.live_export(&*game, &game_path, disable_regen_table_guid, keys_first) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            },

            Command::AddLineToPackIgnoredDiagnostics(line) => {
                if let Some(diagnostics_ignored) = pack_file_decoded.settings_mut().settings_text_mut().get_mut("diagnostics_files_to_ignore") {
                    diagnostics_ignored.push_str(&line);
                } else {
                    pack_file_decoded.settings_mut().settings_text_mut().insert("diagnostics_files_to_ignore".to_owned(), line);
                }
            },

            Command::UpdateEmpireAndNapoleonAK => {
                match old_ak_files_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::GetPackTranslation(language) => {
                let game_key = GAME_SELECTED.read().unwrap().key();
                match translations_local_path() {
                    Ok(local_path) => {
                        let mut base_english = HashMap::new();
                        let mut base_local_fixes = HashMap::new();

                        match translations_remote_path() {
                            Ok(remote_path) => {

                                let vanilla_loc_path = remote_path.join(format!("{}/{}", GAME_SELECTED.read().unwrap().key(), VANILLA_LOC_NAME));
                                if let Ok(mut vanilla_loc) = RFile::tsv_import_from_path(&vanilla_loc_path, &None) {
                                    let _ = vanilla_loc.guess_file_type();
                                    if let Ok(RFileDecoded::Loc(vanilla_loc)) = vanilla_loc.decoded() {

                                        // If we have a fixes file for the vanilla translation, apply it before everything else.
                                        let fixes_loc_path = remote_path.join(format!("{}/{}{}.tsv", GAME_SELECTED.read().unwrap().key(), VANILLA_FIXES_NAME, language));
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
                                match PackTranslation::new(&paths, &pack_file_decoded, game_key, &language, &dependencies, &base_english, &base_local_fixes) {
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
                match translations_remote_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            }

            Command::BuildStarposGetCampaingIds => {
                let ids = dependencies.read().unwrap().db_values_from_table_name_and_column_name(Some(&pack_file_decoded), "campaigns_tables", "campaign_name", true, true);
                CentralCommand::send_back(&sender, Response::HashSetString(ids));
            }

            Command::BuildStarposCheckVictoryConditions => {
                let game = GAME_SELECTED.read().unwrap();
                if !GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) || (
                        GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) &&
                        pack_file_decoded.file(VICTORY_OBJECTIVES_FILE_NAME, false).is_some()
                    ) {
                    CentralCommand::send_back(&sender, Response::Success);
                } else {
                    CentralCommand::send_back(&sender, Response::Error(format!("Missing \"db/victory_objectives.txt\" file. Processing the startpos without this file will result in issues in campaign. Add the file to the pack and try again.")));
                }
            }

            Command::BuildStarpos(campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game.key());

                // 3K needs two passes, one per startpos, and there are two per campaign.
                if game.key() == KEY_THREE_KINGDOMS {
                    match dependencies.build_starpos_pre(&mut pack_file_decoded, &game, &game_path, &campaign_id, process_hlp_spd_data, "historical") {
                        Ok(_) => match dependencies.build_starpos_pre(&mut pack_file_decoded, &game, &game_path, &campaign_id, false, "romance") {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                } else {
                    match dependencies.build_starpos_pre(&mut pack_file_decoded, &game, &game_path, &campaign_id, process_hlp_spd_data, "") {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                    }
                }
            }

            Command::BuildStarposPost(campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game.key());
                let asskit_path = Some(settings.path_buf(&(game.key().to_owned() + "_assembly_kit")));

                let sub_start_pos = if game.key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match dependencies.build_starpos_post(&mut pack_file_decoded, &game, &game_path, asskit_path, &campaign_id, process_hlp_spd_data, false, &sub_start_pos) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            },

            Command::BuildStarposCleanup(campaign_id, process_hlp_spd_data) => {
                let dependencies = dependencies.read().unwrap();
                let game = GAME_SELECTED.read().unwrap();
                let game_path = settings.path_buf(game.key());
                let asskit_path = Some(settings.path_buf(&(game.key().to_owned() + "_assembly_kit")));

                let sub_start_pos = if game.key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match dependencies.build_starpos_post(&mut pack_file_decoded, &game, &game_path, asskit_path, &campaign_id, process_hlp_spd_data, true, &sub_start_pos) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
                }
            },

            Command::UpdateAnimIds(starting_id, offset) => {
                let game = GAME_SELECTED.read().unwrap();
                match pack_file_decoded.update_anim_ids(&game, starting_id, offset) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error.to_string())),
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
                CentralCommand::send_back(&sender, Response::SettingsBool(settings.bool(&key)));
            }
            Command::SettingsGetI32(key) => {
                CentralCommand::send_back(&sender, Response::SettingsI32(settings.i32(&key)));
            }
            Command::SettingsGetF32(key) => {
                CentralCommand::send_back(&sender, Response::SettingsF32(settings.f32(&key)));
            }
            Command::SettingsGetString(key) => {
                CentralCommand::send_back(&sender, Response::SettingsString(settings.string(&key)));
            }
            Command::SettingsGetPathBuf(key) => {
                CentralCommand::send_back(&sender, Response::SettingsPathBuf(settings.path_buf(&key)));
            }
            Command::SettingsGetVecString(key) => {
                CentralCommand::send_back(&sender, Response::SettingsVecString(settings.vec_string(&key)));
            }
            Command::SettingsGetVecRaw(key) => {
                CentralCommand::send_back(&sender, Response::SettingsVecRaw(settings.raw_data(&key)));
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
                let game = GAME_SELECTED.read().unwrap();
                match settings.assembly_kit_path(&game) {
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
            Command::BackupSettings => backup_settings = settings.clone(),
            Command::ClearSettings => match Settings::init(true) {
                Ok(set) => {
                    settings = set;
                    CentralCommand::send_back(&sender, Response::Success);},
                Err(e) => CentralCommand::send_back(&sender, Response::Error(e.to_string())),
            },
            Command::RestoreBackupSettings => settings = backup_settings.clone(),
            Command::OptimizerOptions => CentralCommand::send_back(&sender, Response::OptimizerOptions(settings.optimizer_options())),
        }
    }
}

/// Function to simplify logic for changing game selected.
fn load_schemas(pack: &mut Pack, game: &GameInfo, settings: &Settings) {
    let cf = pack.compression_format();

    // Before loading the schema, make sure we don't have tables with definitions from the current schema.
    let mut files = pack.files_by_type_mut(&[FileType::DB]);
    let extra_data = Some(EncodeableExtraData::new_from_game_info_and_settings(game, cf, settings.bool("disable_uuid_regeneration_on_db_tables")));

    files.par_iter_mut().for_each(|file| {
        let _ = file.encode(&extra_data, true, true, false);
    });

    // Load the new schema.
    let schema_path = schemas_path().unwrap().join(game.schema_file_name());
    let local_patches_path = table_patches_path().unwrap().join(game.schema_file_name());
    *SCHEMA.write().unwrap() = Schema::load(&schema_path, Some(&local_patches_path)).ok();

    // Redecode all the tables in the open file.
    if let Some(ref schema) = *SCHEMA.read().unwrap() {
        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(Some(schema));
        let extra_data = Some(extra_data);

        files.par_iter_mut().for_each(|file| {
            let _ = file.decode(&extra_data, true, false);
        });
    }
}

fn decode_and_send_file(file: &mut RFile, sender: &UnboundedSender<Response>, settings: &Settings) {
    let mut extra_data = DecodeableExtraData::default();
    let schema = SCHEMA.read().unwrap();
    extra_data.set_schema(schema.as_ref());

    let game_info = GAME_SELECTED.read().unwrap().clone();
    extra_data.set_game_info(Some(&game_info));

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

fn tr(s: &str) -> String {
    s.to_owned()
}
