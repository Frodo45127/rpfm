//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, &which can be &found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the background loop.

Basically, this does the heavy load of the program.
!*/

use anyhow::{anyhow, Result};
use crossbeam::channel::{Sender, unbounded};
use itertools::Itertools;
use open::that;
use rayon::prelude::*;

use std::collections::{BTreeMap, HashMap, hash_map::DefaultHasher};
#[cfg(any(feature = "enable_tools", feature = "support_model_renderer"))] use std::collections::HashSet;
use std::env::temp_dir;
use std::fs::{DirBuilder, File};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command as SystemCommand;
use std::sync::{Arc, atomic::Ordering, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::optimizer::{OptimizableContainer, OptimizerOptions};
#[cfg(feature = "enable_tools")] use rpfm_extensions::translator::PackTranslation;

use rpfm_lib::binary::WriteBytes;
use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{animpack::AnimPack, Container, ContainerPath, db::DB, DecodeableExtraData, FileType, loc::Loc, pack::*, portrait_settings::PortraitSettings, RFile, RFileDecoded, table::Table, text::*};
use rpfm_lib::games::{GameInfo, LUA_REPO, LUA_BRANCH, LUA_REMOTE, OLD_AK_REPO, OLD_AK_BRANCH, OLD_AK_REMOTE, pfh_file_type::PFHFileType, supported_games::*, VanillaDBTableNameLogic};
#[cfg(feature = "enable_tools")] use rpfm_lib::games::{TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE};
use rpfm_lib::integrations::{assembly_kit::*, git::*, log::*};
use rpfm_lib::schema::*;
use rpfm_lib::utils::*;

use rpfm_ui_common::locale::tr;
use rpfm_ui_common::PROGRAM_PATH;

use crate::app_ui::NewFile;
use crate::backend::*;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::FIRST_GAME_CHANGE_DONE;
use crate::GAME_SELECTED;
use crate::packedfile_views::DataSource;
use crate::SCHEMA;
use crate::settings_ui::backend::*;
use crate::START_POS_WORKAROUND_THREAD;
use crate::SUPPORTED_GAMES;
#[cfg(feature = "enable_tools")]use crate::tools::translator::*;
use crate::utils::initialize_encodeable_extra_data;

#[allow(dead_code)] const USER_SCRIPT_FILE_NAME: &str = "user.script.txt";
#[allow(dead_code)] const VICTORY_OBJECTIVES_FILE_NAME: &str = "db/victory_objectives.txt";
#[allow(dead_code)] const VICTORY_OBJECTIVES_EXTRACTED_FILE_NAME: &str = "victory_objectives.txt";

const GAMES_NEEDING_VICTORY_OBJECTIVES: [&str; 9] = [
    KEY_PHARAOH_DYNASTIES,
    KEY_PHARAOH,
    KEY_TROY,
    KEY_THREE_KINGDOMS,
    KEY_WARHAMMER_2,
    KEY_WARHAMMER,
    KEY_THRONES_OF_BRITANNIA,
    KEY_ATTILA,
    KEY_ROME_2
];

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
///
/// All communication between this and the UI thread is done use the `CENTRAL_COMMAND` static.
pub fn background_loop() {

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

    // Load all the tips we have.
    //let mut tips = if let Ok(tips) = Tips::load() { tips } else { Tips::default() };

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    info!("Background Thread looping around…");
    'background_loop: loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let (sender, response): (Sender<Response>, Command) = CENTRAL_COMMAND.recv_background();
        match response {

            // Command to close the thread.
            Command::Exit => break,

            // In case we want to reset the PackFile to his original state (dummy)...
            Command::ResetPackFile => pack_file_decoded = Pack::default(),

            // In case we want to remove a Secondary Packfile from memory...
            Command::RemovePackFileExtra(path) => { pack_files_decoded_extra.remove(&path); },

            // In case we want to create a "New PackFile"...
            Command::NewPackFile => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let pack_version = game_selected.pfh_version_by_file_type(PFHFileType::Mod);
                pack_file_decoded = Pack::new_with_name_and_version("unknown.pack", pack_version);

                if let Some(version_number) = game_selected.game_version_number(&setting_path(game_selected.key())) {
                    pack_file_decoded.set_game_version(version_number);
                }
            }

            // In case we want to "Open one or more PackFiles"...
            Command::OpenPackFiles(paths) => {
                let game_selected = GAME_SELECTED.read().unwrap().clone();
                match Pack::read_and_merge(&paths, &game_selected, setting_bool("use_lazy_loading"), false, false) {
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
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
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
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                    }
                }
            }

            // In case we want to "Load All CA PackFiles"...
            Command::LoadAllCAPackFiles => {
                let game_selected = GAME_SELECTED.read().unwrap();
                match Pack::read_and_merge_ca_packs(&game_selected, &setting_path(game_selected.key())) {
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
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }

            // In case we want to "Save a PackFile"...
            Command::SavePackFile => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let extra_data = Some(initialize_encodeable_extra_data(&game_selected, pack_file_decoded.compression_format()));

                let pack_type = *pack_file_decoded.header().pfh_file_type();
                if !setting_bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.")));
                    continue;
                }

                match pack_file_decoded.save(None, &game_selected, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error))),
                }
            }

            // In case we want to "Save a PackFile As"...
            Command::SavePackFileAs(path) => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let extra_data = Some(initialize_encodeable_extra_data(&game_selected, pack_file_decoded.compression_format()));

                let pack_type = *pack_file_decoded.header().pfh_file_type();
                if !setting_bool("allow_editing_of_ca_packfiles") && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings.")));
                    continue;
                }

                match pack_file_decoded.save(Some(&path), &game_selected, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error))),
                }
            }

            // If you want to perform a clean&save over a PackFile...
            Command::CleanAndSavePackFileAs(path) => {
                pack_file_decoded.clean_undecoded();

                let game_selected = GAME_SELECTED.read().unwrap();
                let extra_data = Some(initialize_encodeable_extra_data(&game_selected, pack_file_decoded.compression_format()));
                match pack_file_decoded.save(Some(&path), &game_selected, &extra_data) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::ContainerInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while trying to save the currently open PackFile: {}", error))),
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
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Cannot find extra PackFile with path: {}", path.to_string_lossy()))),
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
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?"))),
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

                CentralCommand::send_back(&sender, Response::CompressionFormat(pack_file_decoded.compression_format()));

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
                    let handle = thread::spawn(move || {
                        let game_selected = GAME_SELECTED.read().unwrap();
                        let game_path = setting_path(game_selected.key());
                        let secondary_path = setting_path(SECONDARY_PATH);
                        let file_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                        let file_path = if game_changed { Some(&*file_path) } else { None };
                        let _ = dependencies.write().unwrap().rebuild(&None, &pack_dependencies, file_path, &game_selected, &game_path, &secondary_path);
                        dependencies
                    });

                    // Load the new schemas.
                    load_schemas(&sender, &mut pack_file_decoded, &game);

                    // Get the dependencies that were loading in parallel and send their info to the UI.
                    dependencies = handle.join().unwrap();
                    let dependencies_info = DependenciesInfo::from(&*dependencies.read().unwrap());
                    info!("Sending dependencies info after game selected change.");
                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));

                    // Decode the dependencies tables while the UI does its own thing.
                    dependencies.write().unwrap().decode_tables(&SCHEMA.read().unwrap());
                }

                // Branch 2: no dependecies rebuild.
                else {
                    info!("Branch 2.");

                    // Load the new schemas.
                    load_schemas(&sender, &mut pack_file_decoded, &game);
                };

                // If there is a Pack open, change his id to match the one of the new `Game Selected`.
                if !pack_file_decoded.disk_file_path().is_empty() {
                    let pfh_file_type = *pack_file_decoded.header().pfh_file_type();
                    pack_file_decoded.header_mut().set_pfh_version(game.pfh_version_by_file_type(pfh_file_type));

                    if let Some(version_number) = game.game_version_number(&setting_path(game.key())) {
                        pack_file_decoded.set_game_version(version_number);
                    }
                }
                info!("Switching game selected done.");
            }

            // In case we want to generate the dependencies cache for our Game Selected...
            Command::GenerateDependenciesCache => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = setting_path(game_selected.key());
                let ignore_game_files_in_ak = setting_bool("ignore_game_files_in_ak");
                let asskit_path = assembly_kit_path().ok();

                if game_path.is_dir() {
                    let schema = SCHEMA.read().unwrap();
                    match Dependencies::generate_dependencies_cache(&schema, &game_selected, &game_path, &asskit_path, ignore_game_files_in_ak) {
                        Ok(mut cache) => {
                            let dependencies_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                            match cache.save(&dependencies_path) {
                                Ok(_) => {
                                    let secondary_path = setting_path(SECONDARY_PATH);
                                    let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();
                                    let _ = dependencies.write().unwrap().rebuild(&schema, &pack_dependencies, Some(&dependencies_path), &game_selected, &game_path, &secondary_path);
                                    let dependencies_info = DependenciesInfo::from(&*dependencies.read().unwrap());
                                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                                },
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                            }
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Game Path not configured. Go to <i>'PackFile/Settings'</i> and configure it.")));
                }
            }

            // In case we want to update the Schema for our Game Selected...
            Command::UpdateCurrentSchemaFromAssKit => {
                let ignore_game_files_in_ak = setting_bool("ignore_game_files_in_ak");

                if let Some(ref mut schema) = *SCHEMA.write().unwrap() {
                    match assembly_kit_path() {
                        Ok(asskit_path) => {
                            let game_selected = GAME_SELECTED.read().unwrap();
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
                                            if dependencies.generate_automatic_patches(schema).is_ok() {

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
                                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                                                }
                                            } else {
                                                CentralCommand::send_back(&sender, Response::Success)
                                            }
                                        } else {
                                            CentralCommand::send_back(&sender, Response::Success)
                                        }
                                    },
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                                }
                            }
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")));
                }
            }

            // In case we want to optimize our PackFile...
            Command::OptimizePackFile(options) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    match pack_file_decoded.optimize(None, &mut dependencies.write().unwrap(), schema, &options) {
                        Ok(paths_to_delete) => CentralCommand::send_back(&sender, Response::HashSetString(paths_to_delete)),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")));
                }
            }

            // In case we want to Patch the SiegeAI of a PackFile...
            Command::PatchSiegeAI => {
                match pack_file_decoded.patch_siege_ai() {
                    Ok(result) => CentralCommand::send_back(&sender, Response::StringVecContainerPath(result.0, result.1)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error)))
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
                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("No definitions found for the table `{}`, version `{}` in the currently loaded schema.", table, version)));
                                    continue;
                                }
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")));
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
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
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
                            match pack_file_decoded.insert_file(source_path, destination_path, &schema) {
                                Ok(path) => if let Some(path) = path {
                                    added_paths.push(path);
                                },
                                Err(error) => it_broke = Some(error),
                            }
                        },

                        // TODO: See what should we do with the ignored paths.
                        ContainerPath::Folder(destination_path) => {
                            match pack_file_decoded.insert_folder(source_path, destination_path, &None, &schema, setting_bool("include_base_folder_on_add_from_folder")) {
                                Ok(mut paths) => added_paths.append(&mut paths),
                                Err(error) => it_broke = Some(error),
                            }
                        },
                    }
                }

                if let Some(error) = it_broke {
                    CentralCommand::send_back(&sender, Response::VecContainerPath(added_paths.to_vec()));
                    CentralCommand::send_back(&sender, Response::Error(From::from(error)));
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
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Cannot find extra PackFile with path: {}", pack_file_path.to_string_lossy()))),
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
                        //extra_data.set_lazy_load(setting_bool("use_lazy_loading"));
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
                                _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded)))),
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("Failed to decode the file at the following path: {}", anim_pack_path))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found in the Pack: {}.", anim_pack_path))),
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
                        //extra_data.set_lazy_load(setting_bool("use_lazy_loading"));
                        let _ = file.decode(&Some(extra_data), true, false);

                        match file.decoded_mut() {
                            Ok(decoded) => match decoded {
                                RFileDecoded::AnimPack(anim_pack) => anim_pack.files_by_paths(&paths, false).into_iter().cloned().collect::<Vec<RFile>>(),
                                _ => {
                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded))));
                                    continue;
                                },
                            }
                            _ => {
                                CentralCommand::send_back(&sender, Response::Error(anyhow!("Failed to decode the file at the following path: {}", anim_pack_path)));
                                continue;
                            },
                        }
                    }
                    None => {
                        CentralCommand::send_back(&sender, Response::Error(anyhow!("The file with the path {} doesn't exists on the open Pack.", anim_pack_path)));
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
                        //extra_data.set_lazy_load(setting_bool("use_lazy_loading"));
                        let _ = file.decode(&Some(extra_data), true, false);

                        match file.decoded_mut() {
                            Ok(decoded) => match decoded {
                                RFileDecoded::AnimPack(anim_pack) => {
                                    for path in paths {
                                        anim_pack.remove(&path);
                                    }

                                    CentralCommand::send_back(&sender, Response::Success);
                                }
                                _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded)))),
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("Failed to decode the file at the following path: {}", anim_pack_path))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found in the Pack: {}.", anim_pack_path))),
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
                                Some(file) => decode_and_send_file(file, &sender),
                                None => CentralCommand::send_back(&sender, Response::Error(anyhow!("The file with the path {} hasn't been found on this Pack.", path))),
                            }
                        }
                    }

                    DataSource::ParentFiles => {
                        match dependencies.write().unwrap().file_mut(&path, false, true) {
                            Ok(file) => decode_and_send_file(file, &sender),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }

                    DataSource::GameFiles => {
                        match dependencies.write().unwrap().file_mut(&path, true, false) {
                            Ok(file) => decode_and_send_file(file, &sender),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }

                    DataSource::AssKitFiles => {
                        let path_split = path.split('/').collect::<Vec<_>>();
                        if path_split.len() > 2 {
                            match dependencies.read().unwrap().asskit_only_db_tables().get(path_split[1]) {
                                Some(db) => CentralCommand::send_back(&sender, Response::DBRFileInfo(db.clone(), RFileInfo::default())),
                                None => CentralCommand::send_back(&sender, Response::Error(anyhow!("Table {} not found on Assembly Kit files.", path))),
                            }
                        } else {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("Path {} doesn't contain an identificable table name.", path)));
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
                        CentralCommand::send_back(&sender, Response::Error(From::from(error)));
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

                let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), pack_file_decoded.compression_format()));
                let mut extracted_paths = vec![];

                // Pack extraction.
                if let Some(container_paths) = container_paths.get(&DataSource::PackFile) {
                    for container_path in container_paths {
                        match pack_file_decoded.extract(container_path.clone(), &path, true, schema, false, setting_bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
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
                        CentralCommand::send_back(&sender, Response::Error(anyhow!("There were {} errors while extracting.", errors)));
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
                        match pack.extract(container_path, &path, true, schema, false, setting_bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                            Ok(mut extracted_path) => extracted_paths.append(&mut extracted_path),
                            Err(_) => errors += 1,
                        }
                    }

                    if errors == 0 {
                        CentralCommand::send_back(&sender, Response::StringVecPathBuf(tr("files_extracted_success"), extracted_paths));
                    } else {
                        CentralCommand::send_back(&sender, Response::Error(anyhow!("There were {} errors while extracting.", errors)));
                    }
                }
            }

            // In case we want to rename one or more files/folders...
            Command::RenamePackedFiles(renaming_data) => {
                match pack_file_decoded.move_paths(&renaming_data) {
                    Ok(data) => CentralCommand::send_back(&sender, Response::VecContainerPathContainerPath(data)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
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
                    let tables = schema.definitions().par_iter().filter(|(key, defintions)| !defintions.is_empty() && key.starts_with("start_pos_")).map(|(key, _)| key.to_owned()).collect::<Vec<_>>();
                    CentralCommand::send_back(&sender, Response::VecString(tables));
                }
                None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")))
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
                            if table_name.starts_with("start_pos_") {
                                match &*SCHEMA.read().unwrap() {
                                    Some(schema) => {
                                        match schema.definitions_by_table_name(&table_name) {
                                            Some(definitions) => {
                                                if definitions.is_empty() {
                                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There are no definitions for this specific table.")));
                                                } else {
                                                    CentralCommand::send_back(&sender, Response::I32(*definitions.first().unwrap().version()));
                                                }
                                            }
                                            None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There are no definitions for this specific table."))),
                                        }
                                    }
                                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")))
                                }
                            } else {
                                CentralCommand::send_back(&sender, Response::Error(anyhow!("Table not found in the game files.")))
                            }
                        },
                    }
                } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("Dependencies cache needs to be regenerated before this."))); }
            }

            #[cfg(feature = "enable_tools")] Command::GetTableDefinitionFromDependencyPackFile(table_name) => {
                if dependencies.read().unwrap().is_vanilla_data_loaded(false) {
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        if let Some(version) = dependencies.read().unwrap().db_version(&table_name) {
                            if let Some(definition) = schema.definition_by_name_and_version(&table_name, version) {
                                CentralCommand::send_back(&sender, Response::Definition(definition.clone()));
                            } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("No definition found for table {}.", table_name))); }
                        } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("Table version not found in dependencies for table {}.", table_name))); }
                    } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected."))); }
                } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("Dependencies cache needs to be regenerated before this."))); }
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
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }

            // In case we want to update a table...
            Command::UpdateTable(path) => {
                let path = path.path_raw();
                if let Some(rfile) = pack_file_decoded.file_mut(path, false) {
                    if let Ok(decoded) = rfile.decoded_mut() {
                        match dependencies.write().unwrap().update_db(decoded) {
                            Ok((old_version, new_version)) => CentralCommand::send_back(&sender, Response::I32I32(old_version, new_version)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path undecoded: {}", path))); }
                } else { CentralCommand::send_back(&sender, Response::Error(anyhow!("File not found in the open Pack: {}", path))); }
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
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.into())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?")));
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
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error.into())),
                    }
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema not found. Maybe you need to download it?")));
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
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("This Pack doesn't exists as a file in the disk."))),
                }
            },

            // In case we want to save an schema to disk...
            Command::SaveSchema(mut schema) => {
                match schema.save(&schemas_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name())) {
                    Ok(_) => {
                        *SCHEMA.write().unwrap() = Some(schema);
                        CentralCommand::send_back(&sender, Response::Success);
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }

            // In case we want to clean the cache of one or more PackedFiles...
            Command::CleanCache(paths) => {
                let cf = pack_file_decoded.compression_format();
                let mut files = pack_file_decoded.files_by_paths_mut(&paths, false);
                let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), cf));

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
                                CentralCommand::send_back(&sender, Response::Error(anyhow!("Exporting a TSV from the Assembly Kit is not yet supported.")));
                                continue;
                            },
                            DataSource::ExternalFile => {
                                CentralCommand::send_back(&sender, Response::Error(anyhow!("Exporting a TSV from a external file is not yet supported.")));
                                continue;
                            },
                        };
                        match file {
                            Some(file) => match file.tsv_export_to_path(&external_path, schema, setting_bool("tables_use_old_column_order_for_tsv")) {
                                Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                Err(error) =>  CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                            }
                            None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path not found in the Pack: {}", internal_path))),
                        }
                    },
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected."))),
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
                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("File with the following path not found in the Pack: {}", internal_path))),
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
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("This Pack doesn't exists as a file in the disk.")));
                }
            },

            // When we want to open a PackedFile in a external program...
            Command::OpenPackedFileInExternalProgram(data_source, path) => {
                match data_source {
                    DataSource::PackFile => {
                        let folder = temp_dir().join(format!("rpfm_{}", pack_file_decoded.disk_file_name()));
                        let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), pack_file_decoded.compression_format()));

                        match pack_file_decoded.extract(path.clone(), &folder, true, &SCHEMA.read().unwrap(), false, setting_bool("tables_use_old_column_order_for_tsv"), &extra_data, true) {
                            Ok(extracted_path) => {
                                let _ = that(&extracted_path[0]);
                                CentralCommand::send_back(&sender, Response::PathBuf(extracted_path[0].to_owned()));
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    _ => CentralCommand::send_back(&sender, Response::Error(anyhow!("Opening dependencies files in external programs is not yet supported."))),
                }
            }

            // When we want to save a PackedFile from the external view....
            Command::SavePackedFileFromExternalView(path, external_path) => {
                match save_files_from_external_path(&mut pack_file_decoded, &path, &external_path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
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
                                let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), cf));

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
                                        let game_path = setting_path(game.key());
                                        let secondary_path = setting_path(SECONDARY_PATH);
                                        let dependencies_file_path = dependencies_cache_path().unwrap().join(game.dependencies_cache_file_name());
                                        let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();

                                        match dependencies.write().unwrap().rebuild(&SCHEMA.read().unwrap(), &pack_dependencies, Some(&*dependencies_file_path), &game, &game_path, &secondary_path) {
                                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                            Err(_) => CentralCommand::send_back(&sender, Response::Error(anyhow!("Schema updated, but dependencies cache rebuilding failed. You may need to regenerate it."))),
                                        }
                                    } else {
                                        CentralCommand::send_back(&sender, Response::Success)
                                    }
                                } else {
                                    CentralCommand::send_back(&sender, Response::Success)
                                }
                            },
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our lua setup...
            Command::UpdateLuaAutogen => {
                match lua_autogen_base_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, LUA_REPO, LUA_BRANCH, LUA_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our program...
            Command::UpdateMainProgram => {
                match crate::updater_ui::update_main_program() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our program...
            Command::TriggerBackupAutosave => {
                let folder = backup_autosave_path().unwrap().join(pack_file_decoded.disk_file_name());
                let _ = DirBuilder::new().recursive(true).create(&folder);

                // Note: we no longer notify the UI of success or error to not hang it up.
                let game_selected = GAME_SELECTED.read().unwrap();
                let game_path = setting_path(game_selected.key());
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
                    let extra_data = Some(initialize_encodeable_extra_data(&game_selected, pack_file_decoded.compression_format()));
                    let _ = pack_file_decoded.clone().save(Some(&new_path), &game_selected, &extra_data);

                    // If we have more than the limit, delete the older one.
                    if let Ok(files) = files_in_folder_from_newest_to_oldest(&folder) {
                        let max_files = setting_int("autosave_amount") as usize;
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
                let game_path = setting_path(game_selected.key());

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
                let game_path = setting_path(game_selected.key());

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
                    let game_path = setting_path(game_selected.key());
                    let dependencies_file_path = dependencies_cache_path().unwrap().join(game_selected.dependencies_cache_file_name());
                    let file_path = if !rebuild_only_current_mod_dependencies { Some(&*dependencies_file_path) } else { None };
                    let pack_dependencies = pack_file_decoded.dependencies().iter().map(|x| x.1.clone()).collect::<Vec<_>>();

                    let secondary_path = setting_path(SECONDARY_PATH);
                    let _ = dependencies.write().unwrap().rebuild(&SCHEMA.read().unwrap(), &pack_dependencies, file_path, &game_selected, &game_path, &secondary_path);
                    let dependencies_info = DependenciesInfo::from(&*dependencies.read().unwrap());
                    CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                } else {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")));
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
                    CentralCommand::send_back(&sender, Response::Error(anyhow!(tr("source_data_for_field_not_found"))));
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
                    CentralCommand::send_back(&sender, Response::Error(anyhow!(tr("loc_key_not_found"))));
                }
            },

            Command::GetSourceDataFromLocKey(loc_key) => CentralCommand::send_back(&sender, Response::OptionStringStringVecString(dependencies.read().unwrap().loc_key_source(&loc_key))),
            Command::GetPackFileName => CentralCommand::send_back(&sender, Response::String(pack_file_decoded.disk_file_name())),
            Command::GetPackedFileRawData(path) => {
                let cf = pack_file_decoded.compression_format();
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
                                    let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), cf));
                                    match rfile.encode(&extra_data, false, false, true) {
                                        Ok(data) => CentralCommand::send_back(&sender, Response::VecU8(data.unwrap())),
                                        Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                                    }
                                },
                            },
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("This PackedFile no longer exists in the PackFile."))),
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
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("You can't import files from this source.")));
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
                                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("No idea how you were able to trigger this.")));
                                                    CentralCommand::send_back(&sender, Response::Success);
                                                    continue 'background_loop;
                                                }

                                            }
                                            ContainerPath::File(path) => {
                                                let table_name = path.split('/').collect::<Vec<_>>()[1];
                                                match dependencies.import_from_ak(table_name, schema) {
                                                    Ok(table) => {
                                                        let file = RFile::new_from_decoded(&RFileDecoded::DB(table), 0, path);
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
                                    CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected.")));
                                    CentralCommand::send_back(&sender, Response::Success);
                                    continue 'background_loop;
                                }
                            }
                        },
                        _ => {
                            CentralCommand::send_back(&sender, Response::Error(anyhow!("You can't import files from this source.")));
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

            #[cfg(feature = "support_model_renderer")] Command::GetAnimPathsBySkeletonName(skeleton_name) => {
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

            #[cfg(feature = "enable_tools")] Command::GetPackedFilesNamesStartingWitPathFromAllSources(path) => {
                let mut files: HashMap<DataSource, HashSet<ContainerPath>> = HashMap::new();
                let dependencies = dependencies.read().unwrap();

                let parent_files = dependencies.files_by_path(&[path.clone()], false, true, true);
                if !parent_files.is_empty() {
                    files.insert(DataSource::ParentFiles, parent_files.into_keys().map(ContainerPath::File).collect());
                }

                let game_files = dependencies.files_by_path(&[path.clone()], true, false, true);
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

            #[cfg(feature = "enable_tools")] Command::SavePackedFilesToPackFileAndClean(files) => {
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

                        // TODO: DO NOT CALL QT ON BACKEND.
                        let mut options = OptimizerOptions::default();
                        options.set_optimize_datacored_tables(setting_bool("optimize_not_renamed_packedfiles"));
                        options.set_remove_unused_art_sets(setting_bool("remove_unused_art_sets"));
                        options.set_remove_unused_variants(setting_bool("remove_unused_variants"));
                        options.set_remove_empty_masks(setting_bool("remove_empty_masks"));

                        // Then, optimize the PackFile. This should remove any non-edited rows/files.
                        match pack_file_decoded.optimize(None, &mut dependencies.write().unwrap(), schema, &options) {
                            Ok(paths_to_delete) => CentralCommand::send_back(&sender, Response::VecContainerPathVecContainerPath(added_paths, paths_to_delete.into_iter()
                                .map(ContainerPath::File)
                                .collect())),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected."))),
                }
            },

            Command::NotesForPath(path) => CentralCommand::send_back(&sender, Response::VecNote(pack_file_decoded.notes().notes_by_path(&path))),
            Command::AddNote(note) => CentralCommand::send_back(&sender, Response::Note(pack_file_decoded.notes_mut().add_note(note))),
            Command::DeleteNote(path, id) => pack_file_decoded.notes_mut().delete_note(&path, id),

            Command::SaveLocalSchemaPatch(patches) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::new_patch(&patches, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }
            Command::RemoveLocalSchemaPatchesForTable(table_name) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::remove_patch_for_table(&table_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }
            Command::RemoveLocalSchemaPatchesForTableAndField(table_name, field_name) => {
                let path = table_patches_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name());
                match Schema::remove_patch_for_field(&table_name, &field_name, &path) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }
            Command::ImportSchemaPatch(patch) => {
                match *SCHEMA.write().unwrap() {
                    Some(ref mut schema) => {
                        Schema::add_patch_to_patch_set(schema.patches_mut(), &patch);
                        match schema.save(&schemas_path().unwrap().join(GAME_SELECTED.read().unwrap().schema_file_name())) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected."))),
                }
            }

            Command::GenerateMissingLocData => {
                match generate_missing_loc_data(&mut pack_file_decoded, &dependencies.read().unwrap()) {
                    Ok(path) => CentralCommand::send_back(&sender, Response::VecContainerPath(path)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::PackMap(tile_maps, tiles) => {
                match *SCHEMA.read().unwrap() {
                    Some(ref schema) => {
                        match add_tile_maps_and_tiles(&mut pack_file_decoded, &mut dependencies.write().unwrap(), schema, tile_maps, tiles) {
                            Ok((paths_to_add, paths_to_delete)) => CentralCommand::send_back(&sender, Response::VecContainerPathVecContainerPath(paths_to_add, paths_to_delete)),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(anyhow!("There is no Schema for the Game Selected."))),
                }
            }

            // Initialize the folder for a MyMod, including the folder structure it needs.
            Command::InitializeMyModFolder(mod_name, mod_game, sublime_support, vscode_support, git_support)  => {
                let mut mymod_path = setting_path(MYMOD_BASE_PATH);
                if !mymod_path.is_dir() {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("MyMod path is not configured. Configure it in the settings and try again.")));
                    continue;
                }

                mymod_path.push(&mod_game);

                // Just in case the folder doesn't exist, we try to create it.
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while creating the MyMod's Game folder: {}.", error.to_string())));
                    continue;
                }

                // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                mymod_path.push(&mod_name);
                if let Err(error) = DirBuilder::new().recursive(true).create(&mymod_path) {
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while creating the MyMod's Assets folder: {}.", error.to_string())));
                    continue;
                };

                // Create a repo inside the MyMod's folder.
                if let Some(gitignore) = git_support {
                    let git_integration = GitIntegration::new(&mymod_path, "", "", "");
                    if let Err(error) = git_integration.init() {
                        CentralCommand::send_back(&sender, Response::Error(From::from(error)));
                        continue
                    }

                    if let Err(error) = git_integration.add_gitignore(&gitignore) {
                        CentralCommand::send_back(&sender, Response::Error(From::from(error)));
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
                                CentralCommand::send_back(&sender, Response::Error(anyhow!("Error while creating the VSCode Config folder: {}.", error.to_string())));
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

            Command::LiveExport => match live_export(&mut pack_file_decoded) {
                Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
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
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            #[cfg(feature = "enable_tools")]
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
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                                }
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            #[cfg(feature = "enable_tools")]
            Command::UpdateTranslations => {
                match translations_remote_path() {
                    Ok(local_path) => {
                        let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
                        match git_integration.update_repo() {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
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
                    CentralCommand::send_back(&sender, Response::Error(anyhow!("Missing \"db/victory_objectives.txt\" file. Processing the startpos without this file will result in issues in campaign. Add the file to the pack and try again.")));
                }
            }

            Command::BuildStarpos(campaign_id, process_hlp_spd_data) => {

                // 3K needs two passes, one per startpos, and there are two per campaign.
                if GAME_SELECTED.read().unwrap().key() == KEY_THREE_KINGDOMS {
                    match build_starpos(&dependencies.read().unwrap(), &mut pack_file_decoded, &campaign_id, process_hlp_spd_data, "historical") {
                        Ok(_) => match build_starpos(&dependencies.read().unwrap(), &mut pack_file_decoded, &campaign_id, false, "romance") {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    }
                } else {
                    match build_starpos(&dependencies.read().unwrap(), &mut pack_file_decoded, &campaign_id, process_hlp_spd_data, "") {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    }
                }
            }

            Command::BuildStarposPost(campaign_id, process_hlp_spd_data) => {
                let sub_start_pos = if GAME_SELECTED.read().unwrap().key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match build_starpos_post(&dependencies.read().unwrap(), &mut pack_file_decoded, &campaign_id, process_hlp_spd_data, false, &sub_start_pos) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            },

            Command::BuildStarposCleanup(campaign_id, process_hlp_spd_data) => {
                let sub_start_pos = if GAME_SELECTED.read().unwrap().key() == KEY_THREE_KINGDOMS {
                    vec!["historical".to_owned(), "romance".to_owned()]
                } else {
                    vec![]
                };

                match build_starpos_post(&dependencies.read().unwrap(), &mut pack_file_decoded, &campaign_id, process_hlp_spd_data, true, &sub_start_pos) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            },

            Command::UpdateAnimIds(starting_id, offset) => {
                match update_anim_ids(&mut pack_file_decoded, starting_id, offset) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecContainerPath(paths)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::GetTablesFromDependencies(table_name) => {
                let dependencies = dependencies.read().unwrap();
                match dependencies.db_data(&table_name, true, true) {
                    Ok(files) => CentralCommand::send_back(&sender, Response::VecRFile(files.iter().map(|x| (**x).clone()).collect::<Vec<_>>())),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(From::from(error))),
                }
            }

            // These two belong to the network thread, not to this one!!!!
            Command::CheckUpdates | Command::CheckSchemaUpdates | Command::CheckLuaAutogenUpdates | Command::CheckEmpireAndNapoleonAKUpdates => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            #[cfg(feature = "enable_tools")] Command::CheckTranslationsUpdates => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }
}

fn update_anim_ids(pack_file: &mut Pack, starting_id: i32, offset: i32) -> Result<Vec<ContainerPath>> {
    if offset == 0 {
        return Err(anyhow!("Offset must be different than 0."))
    }

    if starting_id < 0 {
        return Err(anyhow!("Starting Id must be greater than 0."))
    }

    // First, do a pass over sparse files.
    let game_info = GAME_SELECTED.read().unwrap().clone();
    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_game_info(Some(&game_info));
    let extra_data = Some(extra_data);

    let mut files = pack_file.files_by_type_mut(&[FileType::AnimFragmentBattle]);
    let mut paths = files.par_iter_mut()
        .filter_map(|file| {
            let mut changed = false;
            if let Ok(Some(RFileDecoded::AnimFragmentBattle(mut table))) = file.decode(&extra_data, false, true) {
                if *table.max_id() >= starting_id as u32 {
                    table.set_max_id(*table.max_id() + offset as u32);
                    changed = true;
                }

                for entry in table.entries_mut() {
                    if *entry.animation_id() >= starting_id as u32 {
                        entry.set_animation_id(*entry.animation_id() + offset as u32);
                        changed = true;
                    }

                    if *entry.slot_id() >= starting_id as u32 {
                        entry.set_slot_id(*entry.slot_id() + offset as u32);
                        changed = true;
                    }
                }

                if changed {
                    let _ = file.set_decoded(RFileDecoded::AnimFragmentBattle(table));
                    Some(file.path_in_container())
                } else {
                    None
                }
            } else {
                None
            }
        }
    ).collect::<Vec<_>>();

    // Then, do another pass over files in AnimPacks. No need to do a par_iter because there is often less than 10 animpacks in packs.
    let mut anim_packs = pack_file.files_by_type_mut(&[FileType::AnimPack]);

    for anim_pack in anim_packs.iter_mut() {
        let mut changed = false;
        if let Ok(Some(RFileDecoded::AnimPack(mut pack))) = anim_pack.decode(&extra_data, false, true) {

            let mut files = pack.files_by_type_mut(&[FileType::AnimFragmentBattle]);
            for file in files.iter_mut() {
                if let Ok(Some(RFileDecoded::AnimFragmentBattle(mut table))) = file.decode(&extra_data, false, true) {
                    if *table.max_id() >= starting_id as u32 {
                        table.set_max_id(*table.max_id() + offset as u32);
                        changed = true;
                    }

                    for entry in table.entries_mut() {
                        if *entry.animation_id() >= starting_id as u32 {
                            entry.set_animation_id(*entry.animation_id() + offset as u32);
                            changed = true;
                        }

                        if *entry.slot_id() >= starting_id as u32 {
                            entry.set_slot_id(*entry.slot_id() + offset as u32);
                            changed = true;
                        }
                    }

                    if changed {
                        let _ = file.set_decoded(RFileDecoded::AnimFragmentBattle(table));
                    }
                }
            }

            if changed {
                let _ = anim_pack.set_decoded(RFileDecoded::AnimPack(pack));
                paths.push(anim_pack.path_in_container());
            }
        }
    }

    Ok(paths)
}

// About the sub-startpos thing: 3k uses 2 startpos per campaign, so we need to do two passes for it, one for each campaign.
fn build_starpos(dependencies: &Dependencies, pack_file: &mut Pack, campaign_id: &str, process_hlp_spd_data: bool, sub_start_pos: &str) -> Result<()> {
    let pack_name = pack_file.disk_file_name();
    if pack_name.is_empty() {
        return Err(anyhow!("The Pack needs to be saved to disk in order to build a startpos. Save it and try again."));
    }

    if campaign_id.is_empty() {
        return Err(anyhow!("campaign_id not provided."));
    }

    let process_hlp_spd_data_string = if process_hlp_spd_data {
        String::from("process_campaign_ai_map_data;")
    } else {
        String::new()
    };

    let game = GAME_SELECTED.read().unwrap();

    // Note: 3K uses 2 passes per campaign, each one with a different startpos, but both share the hlp/spd process, so that only needs to be generated once.
    // Also, extra folders is to fix a bug in Rome 2, Attila and possibly Thrones where objectives are not processed if certain folders are missing.
    let extra_folders = "add_working_directory assembly_kit\\working_data;";
    let mut user_script_contents = if game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA { extra_folders.to_owned() } else { String::new() };

    user_script_contents.push_str(&format!("
mod {pack_name};
process_campaign_startpos {campaign_id} {sub_start_pos};
{process_hlp_spd_data_string}
quit_after_campaign_processing;"
    ));

    // Games may fail to launch if we don't have this path created, which is done the first time we start the game.
    let game_path = setting_path(game.key());
    let game_data_path = game.data_path(&game_path)?;
    if !game_path.is_dir() {
        return Err(anyhow!("Game path incorrect. Fix it in the settings and try again."));
    }

    if !PathBuf::from(pack_file.disk_file_path()).starts_with(&game_data_path) {
        return Err(anyhow!("The Pack needs to be in /data. Install it there and try again."));
    }

    // We need to extract the victory_objectives.txt file to "data/campaign_id/". Warhammer 3 doesn't use this file.
    if GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) {
        let mut game_campaign_path = game_data_path.to_path_buf();
        game_campaign_path.push(campaign_id);
        DirBuilder::new().recursive(true).create(&game_campaign_path)?;

        game_campaign_path.push(VICTORY_OBJECTIVES_EXTRACTED_FILE_NAME);
        pack_file.extract(ContainerPath::File(VICTORY_OBJECTIVES_FILE_NAME.to_owned()), &game_campaign_path, false, &None, true, false, &None, true)?;
    }

    let config_path = game.config_path(&game_path).ok_or(anyhow!("Error getting the game's config path."))?;
    let scripts_path = config_path.join("scripts");
    DirBuilder::new().recursive(true).create(&scripts_path)?;

    // Rome 2 is bugged when generating startpos using the userscript. We need to pass it to the game through args in a cmd terminal instead of by file.
    //
    // So don't do any userscript change for Rome 2.
    if game.key() != KEY_ROME_2 {

        // Make a backup before editing the script, so we can restore it later.
        let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
        let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");

        if uspa.is_file() {
            std::fs::copy(&uspa, uspb)?;
        }

        let mut file = BufWriter::new(File::create(uspa)?);

        // Napoleon, Empire and Shogun 2 require the user.script.txt or mod list file (for Shogun's latest update) to be in UTF-16 LE. What the actual fuck.
        if *game.raw_db_version() < 2 {
            file.write_string_u16(&user_script_contents)?;
        } else {
            file.write_all(user_script_contents.as_bytes())?;
        }

        file.flush()?;
    }

    // Due to how the starpos is generated, if we generate it on vanilla campaigns it'll overwrite existing files if it's generated on /data.
    // So we must backup the vanilla files, then restore them after.
    //
    // Only needed from Warhammer 1 onwards, and in Rome 2 due to how is generated there.
    if game.key() != KEY_THRONES_OF_BRITANNIA &&
        game.key() != KEY_ATTILA &&
        game.key() != KEY_SHOGUN_2 {

        let sub_start_pos_suffix = if sub_start_pos.is_empty() {
            String::new()
        } else {
            format!("_{sub_start_pos}")
        };

        let starpos_path = game_data_path.join(format!("campaigns/{campaign_id}/startpos{sub_start_pos_suffix}.esf"));
        if starpos_path.is_file() {
            let starpos_path_bak = game_data_path.join(format!("campaigns/{campaign_id}/startpos{sub_start_pos_suffix}.esf.bak"));
            std::fs::copy(&starpos_path, starpos_path_bak)?;
            std::fs::remove_file(starpos_path)?;
        }
    }

    // Same for the other two files, if we're generating them. We need to get the campaign name from the campaigns table first, then get the files generated.
    if process_hlp_spd_data {
        let map_names = dependencies.db_values_from_table_name_and_column_name_for_value(Some(pack_file), "campaigns_tables", "campaign_name", "map_name", true, true);
        if let Some(map_name) = map_names.get(campaign_id) {
            match game.key() {

                // For generating the hlp data, from Warhammer 1 onwards the game outputs it to /data, which may not exists and may conflict with existing files.
                //
                // Create the folder just in case, and back any file found.
                KEY_PHARAOH_DYNASTIES |
                KEY_PHARAOH |
                KEY_WARHAMMER_3 |
                KEY_TROY |
                KEY_THREE_KINGDOMS |
                KEY_WARHAMMER_2 |
                KEY_WARHAMMER => {
                    let hlp_folder_path = game_data_path.join(format!("campaign_maps/{map_name}"));
                    if !hlp_folder_path.is_dir() {
                        DirBuilder::new().recursive(true).create(&hlp_folder_path)?;
                    }

                    let hlp_path = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf"));
                    if hlp_path.is_file() {
                        let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));
                        std::fs::copy(&hlp_path, hlp_path_bak)?;
                        std::fs::remove_file(hlp_path)?;
                    }
                },

                // For Thrones and Attila is more tricky, because the game itself is bugged when processing this file.
                //
                // It's generated in the game's config folder, but we need to manually keep recreating the folder for a while because the game deletes it
                // in the middle of the process and causes an error when trying to write the file. The way we do it is with a background thread
                // that keeps recreating it every 100ms if it ever detects it's gone.
                //
                // Keep in mind this thread is kept alive for as long as the program runs unless it's intentionally stopped. So remember to stop it.
                KEY_THRONES_OF_BRITANNIA |
                KEY_ATTILA => {
                    let folder_path = config_path.join(format!("maps/campaign_maps/{map_name}"));

                    let (sender, receiver) = unbounded::<bool>();
                    let join = thread::spawn(move || {
                        loop {
                            match receiver.try_recv() {
                                Ok(stop) => if stop {
                                    break;
                                }
                                Err(_) => {
                                    if !folder_path.is_dir() {
                                        let _ = DirBuilder::new().recursive(true).create(&folder_path);
                                    }

                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    });

                     *START_POS_WORKAROUND_THREAD.write().unwrap() = Some(vec![(sender, join)]);
                },

                // For rome 2 is a weird one. It generates the file in config (like Attila), but them moves it to /data (like Warhammer).
                //
                // So we need to first, ensure the config folder is created (it may not exists, but it's not deleted mid-process like in Attile)
                // and it's empty, and then backup the hlp file, if exists, from /data.
                KEY_ROME_2 => {
                    let hlp_folder = game_data_path.join(format!("campaign_maps/{map_name}/"));
                    if hlp_folder.is_dir() {
                        let _ = DirBuilder::new().recursive(true).create(&hlp_folder);
                    }

                    let hlp_path = hlp_folder.join("hlp_data.esf");
                    if hlp_path.is_file() {
                        let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));
                        std::fs::copy(&hlp_path, hlp_path_bak)?;
                        std::fs::remove_file(hlp_path)?;
                    }

                }
                KEY_SHOGUN_2 => return Err(anyhow!("Unsupported... yet. If you want to test support for this game, let me know.")),
                KEY_NAPOLEON => return Err(anyhow!("Unsupported... yet. If you want to test support for this game, let me know.")),
                KEY_EMPIRE => return Err(anyhow!("Unsupported... yet. If you want to test support for this game, let me know.")),
                _ => return Err(anyhow!("How the fuck did you trigger this?")),
            }

            // This file is only from Warhammer 1 onwards. No need to check if the path exists because the hlp process should have created the folder.
            if game.key() != KEY_THRONES_OF_BRITANNIA &&
                game.key() != KEY_ATTILA &&
                game.key() != KEY_ROME_2 &&
                game.key() != KEY_SHOGUN_2 &&
                game.key() != KEY_NAPOLEON &&
                game.key() != KEY_EMPIRE {

                let spd_path = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf"));
                if spd_path.is_file() {
                    let spd_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf.bak"));
                    std::fs::copy(&spd_path, spd_path_bak)?;
                    std::fs::remove_file(spd_path)?;
                }
            }
        }
    }

    // Then launch the game. 3K needs to be launched manually and in a blocking manner to make sure it does each pass it has to do correctly.
    if game.key() == KEY_THREE_KINGDOMS {
        let exe_path = game.executable_path(&game_path).ok_or_else(|| anyhow!("Game exe path not found."))?;
        let exe_name = exe_path.file_name().ok_or_else(|| anyhow!("Game exe name not found."))?.to_string_lossy();

        // NOTE: This uses a non-existant load order file on purpouse, so no mod in the load order interferes with generating the startpos.
        let mut command = SystemCommand::new("cmd");
        command.arg("/C");
        command.arg("start");
        command.arg("/wait");
        command.arg("/d");
        command.arg(game_path.to_string_lossy().replace('\\', "/"));
        command.arg(exe_name.to_string());
        command.arg("temp_file.txt;");

        let _ = command.output()?;

        // In multipass, we need to clean the user script after each pass.
        let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
        let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");
        if uspb.is_file() {
            std::fs::copy(uspb, uspa)?;
        }

        // If there's no backup, means there was no file to begin with, so we delete the custom file.
        else if uspa.is_file() {
            std::fs::remove_file(uspa)?;
        }

    // Rome 2 needs to be launched manually through the cmd with params. The rest can be launched through their regular launcher.
    } else if game.key() == KEY_ROME_2 {
        let exe_path = game.executable_path(&game_path).ok_or_else(|| anyhow!("Game exe path not found."))?;
        let exe_name = exe_path.file_name().ok_or_else(|| anyhow!("Game exe name not found."))?.to_string_lossy();

        // NOTE: This uses a non-existant load order file on purpouse, so no mod in the load order interferes with generating the startpos.
        let mut command = SystemCommand::new("cmd");
        command.arg("/C");
        command.arg("start");
        command.arg("/d");
        command.arg(game_path.to_string_lossy().replace('\\', "/"));
        command.arg(exe_name.to_string());
        command.arg("temp_file.txt;");

        // We need to turn the user script contents into a oneliner or the command will ignore it.
        #[cfg(target_os = "windows")] {
            use std::os::windows::process::CommandExt;

            // Rome 2 needs the working_data folder in order to throw the startpos file there.
            command.raw_arg(extra_folders);
            command.raw_arg(user_script_contents.replace("\n", " "));
        }

        command.spawn()?;
    } else {
        match GAME_SELECTED.read().unwrap().game_launch_command(&setting_path(GAME_SELECTED.read().unwrap().key())) {
            Ok(command) => { let _ = open::that(command); },
            _ => return Err(anyhow!("The currently selected game cannot be launched from Steam.")),
        }
    }

    Ok(())
}

fn build_starpos_post(dependencies: &Dependencies, pack_file: &mut Pack, campaign_id: &str, process_hlp_spd_data: bool, cleanup_mode: bool, sub_start_pos: &[String]) -> Result<Vec<ContainerPath>> {

    let mut startpos_failed = false;
    let mut sub_startpos_failed = vec![];
    let mut hlp_failed = false;
    let mut spd_failed = false;

    // Before anything else, close the workaround thread.
    if let Some(data) = START_POS_WORKAROUND_THREAD.write().unwrap().as_mut() {
        let (sender, handle) = data.remove(0);
        let _ = sender.send(true);
        let _ = handle.join();
    }

    *START_POS_WORKAROUND_THREAD.write().unwrap() = None;

    let game = GAME_SELECTED.read().unwrap();
    let game_path = setting_path(game.key());

    if !game_path.is_dir() {
        return Err(anyhow!("Game path incorrect. Fix it in the settings and try again."));
    }

    let game_data_path = game.data_path(&game_path)?;

    // Warhammer 3 doesn't use this folder.
    if GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) {

        // We need to delete the "data/campaign_id/" folder.
        let mut game_campaign_path = game_data_path.to_path_buf();
        game_campaign_path.push(campaign_id);
        if game_campaign_path.is_dir() {
            let _ = std::fs::remove_dir_all(game_campaign_path);
        }
    }

    let config_path = game.config_path(&game_path).ok_or(anyhow!("Error getting the game's config path."))?;
    let scripts_path = config_path.join("scripts");
    if !scripts_path.is_dir() {
        DirBuilder::new().recursive(true).create(&scripts_path)?;
    }

    // Restore the userscript backup, if any.
    let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
    let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");
    if uspb.is_file() {
        std::fs::copy(uspb, uspa)?;
    }

    // If there's no backup, means there was no file to begin with, so we delete the custom file.
    else if uspa.is_file() {
        std::fs::remove_file(uspa)?;
    }

    let mut added_paths = vec![];

    // Add the starpos file. As some games have multiple startpos per campaign (3K) we return a vector with all the paths we have to generate.
    let starpos_paths = match game.key() {
        KEY_PHARAOH_DYNASTIES |
        KEY_PHARAOH |
        KEY_WARHAMMER_3 |
        KEY_TROY |
        KEY_THREE_KINGDOMS |
        KEY_WARHAMMER_2 |
        KEY_WARHAMMER => {
            if sub_start_pos.is_empty() {
                vec![game_data_path.join(format!("campaigns/{campaign_id}/startpos.esf"))]
            } else {
                let mut paths = vec![];
                for sub in sub_start_pos {
                    paths.push(game_data_path.join(format!("campaigns/{campaign_id}/startpos_{sub}.esf")));

                }
                paths
            }
        }
        KEY_THRONES_OF_BRITANNIA |
        KEY_ATTILA => vec![config_path.join(format!("maps/campaigns/{campaign_id}/startpos.esf"))],

        // Rome 2 outputs the startpos in the assembly kit folder.
        KEY_ROME_2 => {
            let asskit_path = setting_path(&(game.key().to_owned() + "_assembly_kit"));
            if !asskit_path.is_dir() {
                return Err(anyhow!("Assembly Kit path incorrect. Fix it in the settings and try again."));
            }

            vec![asskit_path.join(format!("working_data/campaigns/{campaign_id}/startpos.esf"))]
        },

        // Shogun 2 outputs to data, but unlike modern names, vanilla startpos are packed, so there's no rist of overwrite.
        // We still need to clean it up later though. Napoleon and Empire override vanilla files, so those are backed.
        KEY_SHOGUN_2 |
        KEY_NAPOLEON |
        KEY_EMPIRE => vec![game_data_path.join(format!("campaigns/{campaign_id}/startpos.esf"))],
        _ => return Err(anyhow!("How the fuck did you trigger this?")),
    };

    let starpos_paths_pack = if sub_start_pos.is_empty() {
        vec![format!("campaigns/{}/startpos.esf", campaign_id)]
    } else {
        let mut paths = vec![];
        for sub in sub_start_pos {
            paths.push(format!("campaigns/{campaign_id}/startpos_{sub}.esf"));
        }
        paths
    };

    if !cleanup_mode {
        for (index, starpos_path) in starpos_paths.iter().enumerate() {
            if !starpos_path.is_file() {
                if sub_start_pos.is_empty() {
                    startpos_failed = true;
                } else {
                    sub_startpos_failed.push(sub_start_pos[index].to_owned());
                }
            } else {

                let mut rfile = RFile::new_from_file_path(starpos_path)?;
                rfile.set_path_in_container_raw(&starpos_paths_pack[index]);
                rfile.load()?;
                rfile.guess_file_type()?;

                added_paths.push(pack_file.insert(rfile).map(|x| x.unwrap())?);
            }
        }
    }

    // Restore the old starpos if there was one, and delete the new one if it has already been added.
    //
    // Only needed from Warhammer 1 onwards, and for Rome 2, Napoleon and Empire. Other games generate the startpos outside that folder.
    //
    // 3K uses 2 startpos, so we need to restore them both.
    if game.key() != KEY_THRONES_OF_BRITANNIA &&
        game.key() != KEY_ATTILA &&
        game.key() != KEY_SHOGUN_2 {

        for starpos_path in &starpos_paths {
            let file_name = starpos_path.file_name().unwrap().to_string_lossy().to_string();
            let file_name_bak = file_name + ".bak";

            let mut starpos_path_bak = starpos_path.to_path_buf();
            starpos_path_bak.set_file_name(file_name_bak);

            if starpos_path_bak.is_file() {
                std::fs::copy(&starpos_path_bak, starpos_path)?;
                std::fs::remove_file(starpos_path_bak)?;
            }
        }
    }

    // In Shogun 2, we need to cleanup the generated file as to not interfere with the packed one.
    if game.key() == KEY_SHOGUN_2 {
        for starpos_path in &starpos_paths {
            if starpos_path.is_file() {
                std::fs::remove_file(starpos_path)?;
            }
        }
    }

    // Same with the other two files.
    if process_hlp_spd_data {
        let map_names = dependencies.db_values_from_table_name_and_column_name_for_value(Some(pack_file), "campaigns_tables", "campaign_name", "map_name", true, true);
        if let Some(map_name) = map_names.get(campaign_id) {

            // Same as with startpos. It's different depending on the game.
            let hlp_path = match game.key() {
                KEY_PHARAOH_DYNASTIES |
                KEY_PHARAOH |
                KEY_WARHAMMER_3 |
                KEY_TROY |
                KEY_THREE_KINGDOMS |
                KEY_WARHAMMER_2 |
                KEY_WARHAMMER => game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf")),
                KEY_THRONES_OF_BRITANNIA |
                KEY_ATTILA => config_path.join(format!("maps/campaign_maps/{map_name}/hlp_data.esf")),
                KEY_ROME_2 => game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf")),
                _ => return Err(anyhow!("How the fuck did you trigger this?")),
            };

            let hlp_path_pack = format!("campaign_maps/{map_name}/hlp_data.esf");

            if !cleanup_mode {

                if !hlp_path.is_file() {
                    hlp_failed = true;
                } else {

                    let mut rfile_hlp = RFile::new_from_file_path(&hlp_path)?;
                    rfile_hlp.set_path_in_container_raw(&hlp_path_pack);
                    rfile_hlp.load()?;
                    rfile_hlp.guess_file_type()?;

                    added_paths.push(pack_file.insert(rfile_hlp).map(|x| x.unwrap())?);
                }
            }

            // Only needed from Warhammer 1 onwards, and in Rome 2. Other games generate the hlp file outside that folder.
            if game.key() != KEY_THRONES_OF_BRITANNIA &&
                game.key() != KEY_ATTILA {

                let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));

                if hlp_path_bak.is_file() {
                    std::fs::copy(&hlp_path_bak, hlp_path)?;
                    std::fs::remove_file(hlp_path_bak)?;
                }
            }

            // The spd file was introduced in Warhammer 1. Don't expect it on older games.
            if game.key() != KEY_THRONES_OF_BRITANNIA &&
                game.key() != KEY_ATTILA &&
                game.key() != KEY_ROME_2 {

                let spd_path = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf"));
                let spd_path_pack = format!("campaign_maps/{map_name}/spd_data.esf");

                if !cleanup_mode {

                    if !spd_path.is_file() {
                        spd_failed = true;
                    } else {

                        let mut rfile_spd = RFile::new_from_file_path(&spd_path)?;
                        rfile_spd.set_path_in_container_raw(&spd_path_pack);
                        rfile_spd.load()?;
                        rfile_spd.guess_file_type()?;

                        added_paths.push(pack_file.insert(rfile_spd).map(|x| x.unwrap())?);
                    }
                }

                let spd_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf.bak"));
                if spd_path_bak.is_file() {
                    std::fs::copy(&spd_path_bak, spd_path)?;
                    std::fs::remove_file(spd_path_bak)?;
                }
            }
        }
    }

    let mut error = String::new();
    if startpos_failed || (!sub_start_pos.is_empty() && !sub_startpos_failed.is_empty()) || hlp_failed || spd_failed {
        error.push_str("<p>One or more files failed to generate:</p><ul>")
    }
    if startpos_failed {
        error.push_str("<li>Startpos file failed to generate.</li>");
    }

    for sub_failed in &sub_startpos_failed {
        error.push_str(&format!("<li>\"{sub_failed}\" Startpos file failed to generate.</li>"));
    }

    if hlp_failed {
        error.push_str("<li>HLP file failed to generate.</li>");
    }

    if spd_failed {
        error.push_str("<li>SPD file failed to generate.</li>");
    }

    if startpos_failed || hlp_failed || spd_failed {
        error.push_str("</ul><p>No files were added and the related files were restored to their pre-build state. Check your tables are correct before trying to generate them again.</p>")
    }

    if error.is_empty() {
        Ok(added_paths)
    } else {
        Err(anyhow!(error))
    }
}

/// Function to perform a live extraction.
fn live_export(pack: &mut Pack) -> Result<()> {

    // If there are no files, directly return an error.
    if pack.files().is_empty() {
        return Err(anyhow!("No files to export."));
    }

    let extra_data = Some(initialize_encodeable_extra_data(&GAME_SELECTED.read().unwrap(), pack.compression_format()));
    let game_path = setting_path(GAME_SELECTED.read().unwrap().key());
    let data_path = GAME_SELECTED.read().unwrap().data_path(&game_path)?;

    // We're interested in lua and xml files only, not those entire folders.
    let files = pack.files_by_type_and_paths(&[FileType::Text], &[ContainerPath::Folder("script/".to_string()), ContainerPath::Folder("ui/".to_string())], true)
        .into_iter()
        .cloned()
        .collect::<Vec<RFile>>();

    let mut correlations = HashMap::new();
    for mut file in files.into_iter() {
        let mut path_split = file.path_in_container_split().iter().map(|x| x.to_owned()).collect::<Vec<_>>();
        let mut hasher = DefaultHasher::new();

        // Use time to ensure we never collide with a previous live export.
        std::time::SystemTime::now().hash(&mut hasher);
        let value = hasher.finish();
        let new_name = format!("{}_{}", value, path_split.last().unwrap());

        *path_split.last_mut().unwrap() = &new_name;
        let new_path = path_split.join("/");

        correlations.insert(file.path_in_container_raw().to_owned(), new_path.to_owned());
        file.set_path_in_container_raw(&new_path);

        // To avoid duplicating logic, we insert these files into the pack, extract them, then delete them from the Pack.
        let container_path = file.path_in_container();
        pack.insert(file)?;
        pack.extract(container_path.clone(), &data_path, true, &None, false, setting_bool("tables_use_old_column_order_for_tsv"), &extra_data, true)?;

        pack.remove(&container_path);
    }

    // This is the file you have to call from lua later on.
    let summary_data_str = correlations.iter().map(|(key, value)| format!("    [\"{key}\"] = \"{value}\",")).join("\n");
    let summary_data_lua = format!("return {{\n{summary_data_str}\n}}");
    let summary_path = game_path.join("lua_path_mappings.txt");
    let mut file = BufWriter::new(File::create(summary_path)?);
    file.write_all(summary_data_lua.as_bytes())?;

    Ok(())
}

/// Function to simplify logic for changing game selected.
fn load_schemas(sender: &Sender<Response>, pack: &mut Pack, game: &GameInfo) {
    let cf = pack.compression_format();

    // Before loading the schema, make sure we don't have tables with definitions from the current schema.
    let mut files = pack.files_by_type_mut(&[FileType::DB]);
    let extra_data = Some(initialize_encodeable_extra_data(game, cf));

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

    // Send a response, so the UI continues working while we finish things here.
    info!("Sending success after game selected change.");
    CentralCommand::send_back(sender, Response::Success);
}

/// Function to simplify logic for changing game selected.
fn add_tile_maps_and_tiles(pack: &mut Pack, dependencies: &mut Dependencies, schema: &Schema, tile_maps: Vec<PathBuf>, tiles: Vec<(PathBuf, String)>) -> Result<(Vec<ContainerPath>, Vec<ContainerPath>)> {
    let mut added_paths = vec![];

    // Tile Maps are from assembly_kit/working_data/terrain/battles/.
    for tile_map in &tile_maps {
        added_paths.append(&mut pack.insert_folder(tile_map, "terrain/battles", &None, &None, true)?);
    }

    // Tiles are from assembly_kit/working_data/terrain/tiles/battle/, and can be in a subfolder if they're part of a tileset.
    for (tile, subpath) in &tiles {

        let (internal_path, needs_tile_database) = if subpath.is_empty() {
            ("terrain/tiles/battle".to_owned(), false)
        } else {
            (format!("terrain/tiles/battle/{}", subpath.replace('\\', "/")), true)
        };
        added_paths.append(&mut pack.insert_folder(tile, &internal_path, &None, &None, true)?);

        // If it's part of a tile set, we need to add the relevant tile database file for the tileset or the map will load as blank ingame.
        if needs_tile_database {

            // We only need the database for out map, not the full database folder.
            let subpath_len = subpath.replace('\\', "/").split('/').count();
            let mut tile_database = tile.to_path_buf();

            (0..=subpath_len).for_each(|_| {
                tile_database.pop();
            });

            let file_name = format!("{}_{}.bin", subpath.replace('/', "_"), tile.file_name().unwrap().to_string_lossy());
            tile_database.push(format!("_tile_database/TILES/{file_name}"));
            let tile_database_path = format!("terrain/tiles/battle/_tile_database/TILES/{file_name}");

            added_paths.push(pack.insert_file(&tile_database, &tile_database_path, &None)?.unwrap());
        }
    }

    // TODO: DO NOT CALL QT ON BACKEND.
    let mut options = OptimizerOptions::default();
    options.set_optimize_datacored_tables(setting_bool("optimize_not_renamed_packedfiles"));
    options.set_remove_unused_art_sets(setting_bool("remove_unused_art_sets"));
    options.set_remove_unused_variants(setting_bool("remove_unused_variants"));
    options.set_remove_empty_masks(setting_bool("remove_empty_masks"));

    let paths_to_delete = pack.optimize(Some(added_paths.clone()), dependencies, schema, &options)?
        .iter()
        .map(|path| ContainerPath::File(path.to_string()))
        .collect::<Vec<_>>();

    Ok((added_paths, paths_to_delete))
}

/// Function to save files from external paths, so it's easier to use in the big loop.
///
/// NOTE: If TSV is detected and fails to import, this returns an error.
fn save_files_from_external_path(pack: &mut Pack, internal_path: &str, external_path: &Path) -> Result<()> {

    // We do it manually instead of using insert_file because insert_file replaces the file's metadata.
    let mut file = BufReader::new(File::open(external_path)?);
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    match pack.file_mut(internal_path, false) {
        Some(file) => {

            // If we're dealing with a TSV, make sure to import it before setting up the data.
            match external_path.extension() {
                Some(extension) => {
                    if extension.to_string_lossy() == "tsv" {
                        let schema = SCHEMA.read().unwrap();
                        if let Ok(rfile) = RFile::tsv_import_from_path(external_path, &schema) {
                            file.set_decoded(rfile.decoded()?.clone())?;
                        } else {
                            file.set_cached(&data);
                        }
                    } else {
                        file.set_cached(&data);
                    }
                }
                None => {
                    file.set_cached(&data);
                }
            }

            // If they're tables, make sure they're left decoded.
            if file.file_type() == FileType::DB || file.file_type() == FileType::Loc {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let mut extra_data = DecodeableExtraData::default();
                    extra_data.set_schema(Some(schema));
                    let extra_data = Some(extra_data);
                    let _ = file.decode(&extra_data, true, false);
                }
            }

            Ok(())
        }
        None => Err(anyhow!("Failed to find file with path in pack: {}", internal_path)),
    }
}

fn decode_and_send_file(file: &mut RFile, sender: &Sender<Response>) {
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
        #[cfg(all(not(feature = "support_rigidmodel"), not(feature = "support_model_renderer")))] FileType::RigidModel,
        FileType::SoundBank,
        FileType::Unknown
    ];

    // Do not even attempt to decode esf files if the editor is disabled.
    if !setting_bool("enable_esf_editor") {
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
        #[cfg(all(not(feature = "support_rigidmodel"), not(feature = "support_model_renderer")))] Ok(RFileDecoded::RigidModel(_)) => CentralCommand::send_back(sender, Response::Unknown),
        #[cfg(any(feature = "support_rigidmodel", feature = "support_model_renderer"))]Ok(RFileDecoded::RigidModel(rigid_model)) => CentralCommand::send_back(sender, Response::RigidModelRFileInfo(rigid_model, From::from(&*file))),
        Ok(RFileDecoded::SoundBank(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Text(text)) => CentralCommand::send_back(sender, Response::TextRFileInfo(text, From::from(&*file))),
        Ok(RFileDecoded::UIC(uic)) => CentralCommand::send_back(sender, Response::UICRFileInfo(uic, From::from(&*file))),
        Ok(RFileDecoded::UnitVariant(data)) => CentralCommand::send_back(sender, Response::UnitVariantRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::Unknown(_)) => CentralCommand::send_back(sender, Response::Unknown),
        Ok(RFileDecoded::Video(data)) => CentralCommand::send_back(sender, Response::VideoInfoRFileInfo(From::from(&data), From::from(&*file))),
        Ok(RFileDecoded::VMD(data)) => CentralCommand::send_back(sender, Response::VMDRFileInfo(data, From::from(&*file))),
        Ok(RFileDecoded::WSModel(data)) => CentralCommand::send_back(sender, Response::WSModelRFileInfo(data, From::from(&*file))),
        Err(error) => CentralCommand::send_back(sender, Response::Error(From::from(error))),
    }
}

fn generate_missing_loc_data(pack: &mut Pack, dependencies: &Dependencies) -> Result<Vec<ContainerPath>> {
    let loc_data = dependencies.loc_data(true, true)?;
    let mut existing_locs = HashMap::new();

    for loc in &loc_data {
        if let Ok(RFileDecoded::Loc(ref data)) = loc.decoded() {
            existing_locs.extend(data.table().data().iter().map(|x| (x[0].data_to_string().to_string(), x[1].data_to_string().to_string())));
        }
    }

    pack.generate_missing_loc_data(&existing_locs).map_err(From::from)
}
