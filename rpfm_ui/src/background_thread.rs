//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use crossbeam::channel::Sender;
use log::info;
use open::that_in_background;
use rayon::prelude::*;
use uuid::Uuid;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env::temp_dir;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::thread;

use rpfm_error::{Error, ErrorKind};

use rpfm_lib::assembly_kit::*;
use rpfm_lib::common::*;
use rpfm_lib::diagnostics::Diagnostics;
use rpfm_lib::dependencies::{Dependencies, DependenciesInfo};
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::packfile::PFHFileType;
use rpfm_lib::packedfile::*;
use rpfm_lib::packedfile::animpack::AnimPack;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::loc::{Loc, TSV_NAME_LOC};
use rpfm_lib::packedfile::text::{Text, TextType};
use rpfm_lib::packfile::{PackFile, PackFileInfo, packedfile::{PackedFile, PackedFileInfo, RawPackedFile}, PathType, PFHFlags, RESERVED_NAME_NOTES};
use rpfm_lib::schema::{*, patch::SchemaPatches};
use rpfm_lib::SCHEMA;
use rpfm_lib::SCHEMA_PATCHES;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::tips::Tips;

use crate::app_ui::NewPackedFile;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::{tr, tre};
use crate::packedfile_views::DataSource;
use crate::RPFM_PATH;
use crate::views::table::TableType;

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
    let mut pack_file_decoded = PackFile::new();
    let mut pack_files_decoded_extra = BTreeMap::new();

    // Preload the default game's dependencies.
    let mut dependencies = Dependencies::default();

    // Load all the tips we have.
    let mut tips = if let Ok(tips) = Tips::load() { tips } else { Tips::default() };

    // Try to load the schema patchs. Ignore them if fails due to missing file.
    if let Ok(schema_patches) = SchemaPatches::load() {
        *SCHEMA_PATCHES.write().unwrap() = schema_patches;
    }

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
            Command::Exit => return,

            // In case we want to reset the PackFile to his original state (dummy)...
            Command::ResetPackFile => pack_file_decoded = PackFile::new(),

            // In case we want to remove a Secondary Packfile from memory...
            Command::RemovePackFileExtra(path) => { pack_files_decoded_extra.remove(&path); },

            // In case we want to create a "New PackFile"...
            Command::NewPackFile => {
                let pack_version = GAME_SELECTED.read().unwrap().get_pfh_version_by_file_type(PFHFileType::Mod);
                pack_file_decoded = PackFile::new_with_name("unknown.pack", pack_version);

                if let Ok(version_number) = get_game_selected_exe_version_number() {
                    pack_file_decoded.set_game_version(version_number);
                }
            }

            // In case we want to "Open one or more PackFiles"...
            Command::OpenPackFiles(paths) => {
                match PackFile::open_packfiles(&paths, SETTINGS.read().unwrap().settings_bool["use_lazy_loading"], false, false) {
                    Ok(pack_file) => {
                        pack_file_decoded = pack_file;

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            let mut packed_files = pack_file_decoded.get_ref_mut_packed_files_by_types(&[PackedFileType::DB, PackedFileType::Loc], false);
                            packed_files.par_iter_mut().for_each(|x| {
                                let _ = x.decode_no_locks(schema);
                            });
                        }

                        CentralCommand::send_back(&sender, Response::PackFileInfo(PackFileInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to "Open an Extra PackFile" (for "Add from PackFile")...
            Command::OpenPackFileExtra(path) => {
                match pack_files_decoded_extra.get(&path) {
                    Some(pack_file) => CentralCommand::send_back(&sender, Response::PackFileInfo(PackFileInfo::from(pack_file))),
                    None => match PackFile::open_packfiles(&[path.to_path_buf()], true, false, true) {
                         Ok(pack_file) => {
                            CentralCommand::send_back(&sender, Response::PackFileInfo(PackFileInfo::from(&pack_file)));
                            pack_files_decoded_extra.insert(path.to_path_buf(), pack_file);
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    }
                }
            }

            // In case we want to "Load All CA PackFiles"...
            Command::LoadAllCAPackFiles => {
                match PackFile::open_all_ca_packfiles() {
                    Ok(pack_file) => {
                        pack_file_decoded = pack_file;
                        CentralCommand::send_back(&sender, Response::PackFileInfo(PackFileInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to "Save a PackFile"...
            Command::SavePackFile => {
                match pack_file_decoded.save(None) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::PackFileInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::SavePackFileGeneric(error.to_string())))),
                }
            }

            // In case we want to "Save a PackFile As"...
            Command::SavePackFileAs(path) => {
                match pack_file_decoded.save(Some(path.to_path_buf())) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::PackFileInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::SavePackFileGeneric(error.to_string())))),
                }
            }

            // If you want to perform a clean&save over a PackFile...
            Command::CleanAndSavePackFileAs(path) => {
                pack_file_decoded.clean_packfile();
                match pack_file_decoded.save(Some(path.to_path_buf())) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::PackFileInfo(From::from(&pack_file_decoded))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::SavePackFileGeneric(error.to_string())))),
                }
            }

            // In case we want to change the current settings...
            Command::SetSettings(settings) => {
                *SETTINGS.write().unwrap() = settings;
                match SETTINGS.read().unwrap().save() {
                    Ok(()) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to change the current shortcuts...
            Command::SetShortcuts(shortcuts) => {
                match shortcuts.save() {
                    Ok(()) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to get the data of a PackFile needed to form the TreeView...
            Command::GetPackFileDataForTreeView => {

                // Get the name and the PackedFile list, and send it.
                CentralCommand::send_back(&sender, Response::PackFileInfoVecPackedFileInfo((
                    From::from(&pack_file_decoded),
                    pack_file_decoded.get_packed_files_all_info(),

                )));
            }

            // In case we want to get the data of a Secondary PackFile needed to form the TreeView...
            Command::GetPackFileExtraDataForTreeView(path) => {

                // Get the name and the PackedFile list, and serialize it.
                match pack_files_decoded_extra.get(&path) {
                    Some(pack_file) => CentralCommand::send_back(&sender, Response::PackFileInfoVecPackedFileInfo((
                        From::from(pack_file),
                        pack_file.get_packed_files_all_info(),
                    ))),
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::CannotFindExtraPackFile(path).into())),
                }
            }

            // In case we want to get the info of one PackedFile from the TreeView.
            Command::GetPackedFileInfo(path) => {
                CentralCommand::send_back(&sender, Response::OptionPackedFileInfo(
                    pack_file_decoded.get_packed_file_info_by_path(&path)
                ));
            }

            // In case we want to get the info of more than one PackedFiles from the TreeView.
            Command::GetPackedFilesInfo(paths) => {
                CentralCommand::send_back(&sender, Response::VecOptionPackedFileInfo(
                    paths.iter().map(|x| pack_file_decoded.get_packed_file_info_by_path(x)).collect()
                ));
            }

            // In case we want to launch a global search on a `PackFile`...
            Command::GlobalSearch(mut global_search) => {
                global_search.search(&mut pack_file_decoded, &dependencies);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CentralCommand::send_back(&sender, Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to change the current `Game Selected`...
            Command::SetGameSelected(game_selected) => {
                *GAME_SELECTED.write().unwrap() = SUPPORTED_GAMES.get_supported_game_from_key(&game_selected).unwrap();

                // Try to load the Schema for this game but, before it, PURGE THE DAMN SCHEMA-RELATED CACHE AND REBUILD IT AFTERWARDS.
                pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false).par_iter_mut().for_each(|x| { let _ = x.encode_and_clean_cache(); });
                *SCHEMA.write().unwrap() = Schema::load(GAME_SELECTED.read().unwrap().get_schema_name()).ok();
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false).par_iter_mut().for_each(|x| { let _ = x.decode_no_locks(schema); });
                }

                // Send a response, so we can unlock the UI.
                CentralCommand::send_back(&sender, Response::Success);

                // If there is a PackFile open, change his id to match the one of the new `Game Selected`.
                if !pack_file_decoded.get_file_name().is_empty() {
                    pack_file_decoded.set_pfh_version(GAME_SELECTED.read().unwrap().get_pfh_version_by_file_type(pack_file_decoded.get_pfh_file_type()));

                    if let Ok(version_number) = get_game_selected_exe_version_number() {
                        pack_file_decoded.set_game_version(version_number);
                    }
                }
            }

            // In case we want to generate the dependencies cache for our Game Selected...
            Command::GenerateDependenciesCache(path, version) => {
                match dependencies.generate_dependencies_cache(&path, version) {
                    Ok(mut cache) => match cache.save_to_binary() {
                        Ok(_) => {
                            let _ = dependencies.rebuild(pack_file_decoded.get_packfiles_list(), false);
                            let dependencies_info = DependenciesInfo::from(&dependencies);
                            CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
                        },
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to update the Schema for our Game Selected...
            Command::UpdateCurrentSchemaFromAssKit(path) => {
                match update_schema_from_raw_files(path, &dependencies) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to optimize our PackFile...
            Command::OptimizePackFile => {
                match pack_file_decoded.optimize(&dependencies) {
                    Ok(paths_to_delete) => CentralCommand::send_back(&sender, Response::VecVecString(paths_to_delete)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to Patch the SiegeAI of a PackFile...
            Command::PatchSiegeAI => {
                match pack_file_decoded.patch_siege_ai() {
                    Ok(result) => CentralCommand::send_back(&sender, Response::StringVecVecString(result)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error))
                }
            }

            // In case we want to change the PackFile's Type...
            Command::SetPackFileType(new_type) => pack_file_decoded.set_pfh_file_type(new_type),

            // In case we want to change the "Include Last Modified Date" setting of the PackFile...
            Command::ChangeIndexIncludesTimestamp(state) => pack_file_decoded.get_ref_mut_bitmask().set(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, state),

            // In case we want to compress/decompress the PackedFiles of the currently open PackFile...
            Command::ChangeDataIsCompressed(state) => pack_file_decoded.toggle_compression(state),

            // In case we want to get the path of the currently open `PackFile`.
            Command::GetPackFilePath => CentralCommand::send_back(&sender, Response::PathBuf(pack_file_decoded.get_file_path().to_path_buf())),

            // In case we want to get the Dependency PackFiles of our PackFile...
            Command::GetDependencyPackFilesList => CentralCommand::send_back(&sender, Response::VecString(pack_file_decoded.get_packfiles_list().to_vec())),

            // In case we want to set the Dependency PackFiles of our PackFile...
            Command::SetDependencyPackFilesList(pack_files) => pack_file_decoded.set_packfiles_list(&pack_files),

            // In case we want to check if there is a Dependency Database loaded...
            Command::IsThereADependencyDatabase(include_asskit) => CentralCommand::send_back(&sender, Response::Bool(dependencies.game_has_vanilla_data_loaded(include_asskit))),

            // In case we want to create a PackedFile from scratch...
            Command::NewPackedFile(path, new_packed_file) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let decoded = match new_packed_file {
                        NewPackedFile::AnimPack(_) => {
                            let packed_file = AnimPack::new();
                            DecodedPackedFile::AnimPack(packed_file)
                        },
                        NewPackedFile::DB(_, table, version) => {
                            match schema.get_ref_versioned_file_db(&table) {
                                Ok(versioned_file) => {
                                    match versioned_file.get_version(version) {
                                        Ok(definition) =>  DecodedPackedFile::DB(DB::new(&table, None, definition)),
                                        Err(error) => {
                                            CentralCommand::send_back(&sender, Response::Error(error));
                                            continue;
                                        }
                                    }
                                }
                                Err(error) => {
                                    CentralCommand::send_back(&sender, Response::Error(error));
                                    continue;
                                }
                            }
                        },
                        NewPackedFile::Loc(_) => {
                            match schema.get_ref_last_definition_loc() {
                                Ok(definition) => DecodedPackedFile::Loc(Loc::new(definition)),
                                Err(error) => {
                                    CentralCommand::send_back(&sender, Response::Error(error));
                                    continue;
                                }
                            }
                        }
                        NewPackedFile::Text(_, text_type) => {
                            let mut packed_file = Text::new();
                            packed_file.set_text_type(text_type);
                            DecodedPackedFile::Text(packed_file)
                        },
                    };
                    let packed_file = PackedFile::new_from_decoded(&decoded, &path);
                    match pack_file_decoded.add_packed_file(&packed_file, false) {
                        Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                    }
                } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())); }
            }

            // When we want to add one or more PackedFiles to our PackFile.
            Command::AddPackedFiles(source_paths, destination_paths, paths_to_ignore, import_tables_from_tsv) => {
                let mut added_paths = vec![];
                let mut it_broke = None;

                // If we're going to import TSV, make sure to remove any collision between binary and TSV.
                let paths = if import_tables_from_tsv {
                    source_paths.iter().zip(destination_paths.iter())
                        .filter(|(source, _)| {
                            if let Some(extension) = source.extension() {
                                if extension == "tsv" {
                                    true
                                } else {
                                    let mut path = source.to_path_buf();
                                    path.set_extension("tsv");
                                    source_paths.par_iter().all(|source| source != &path)
                                }
                            } else {
                                let mut path = source.to_path_buf();
                                path.set_extension("tsv");
                                source_paths.par_iter().all(|source| source != &path)
                            }
                        })
                        .collect::<Vec<(&PathBuf, &Vec<String>)>>()
                } else {
                    source_paths.iter().zip(destination_paths.iter()).collect::<Vec<(&PathBuf, &Vec<String>)>>()
                };

                for (source_path, destination_path) in paths {

                    // Skip ignored paths.
                    if let Some(ref paths_to_ignore) = paths_to_ignore {
                        if paths_to_ignore.iter().any(|x| source_path.starts_with(x)) {
                            continue;
                        }
                    }

                    match pack_file_decoded.add_from_file(source_path, destination_path.to_vec(), true, import_tables_from_tsv) {
                        Ok(path) => if !path.is_empty() { added_paths.push(PathType::File(path.to_vec())) },
                        Err(error) => it_broke = Some(error),
                    }
                }
                if let Some(error) = it_broke {
                    CentralCommand::send_back(&sender, Response::VecPathType(added_paths.to_vec()));
                    CentralCommand::send_back(&sender, Response::Error(error));
                } else {
                    CentralCommand::send_back(&sender, Response::VecPathType(added_paths.to_vec()));
                    CentralCommand::send_back(&sender, Response::Success);
                }

                // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let paths = added_paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect::<Vec<&[String]>>();
                    let mut packed_files = pack_file_decoded.get_ref_mut_packed_files_by_paths(paths);
                    packed_files.par_iter_mut()
                        .filter(|x| [PackedFileType::DB, PackedFileType::Loc].contains(&x.get_packed_file_type(false)))
                        .for_each(|x| {
                        let _ = x.decode_no_locks(schema);
                    });
                }
            }

            // In case we want to add one or more entire folders to our PackFile...
            Command::AddPackedFilesFromFolder(paths, paths_to_ignore, import_tables_from_tsv) => {
                match pack_file_decoded.add_from_folders(&paths, &paths_to_ignore, true, import_tables_from_tsv) {
                    Ok(paths) => {
                        CentralCommand::send_back(&sender, Response::VecPathType(paths.iter().filter(|x| !x.is_empty()).map(|x| PathType::File(x.to_vec())).collect()));

                        // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            let paths = paths.iter().map(|x| &**x).collect::<Vec<&[String]>>();
                            let mut packed_files = pack_file_decoded.get_ref_mut_packed_files_by_paths(paths);
                            packed_files.par_iter_mut()
                                .filter(|x| [PackedFileType::DB, PackedFileType::Loc].contains(&x.get_packed_file_type(false)))
                                .for_each(|x| {
                                let _ = x.decode_no_locks(schema);
                            });
                        }
                    }
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }

            }

            // In case we want to move stuff from one PackFile to another...
            Command::AddPackedFilesFromPackFile((pack_file_path, paths)) => {

                match pack_files_decoded_extra.get(&pack_file_path) {

                    // Try to add the PackedFile to the main PackFile.
                    Some(pack_file) => match pack_file_decoded.add_from_packfile(pack_file, &paths, true) {
                        Ok(paths) => {
                            CentralCommand::send_back(&sender, Response::VecPathType(paths.to_vec()));

                            // Force decoding of table/locs, so they're in memory for the diagnostics to work.
                            if let Some(ref schema) = *SCHEMA.read().unwrap() {
                                let paths = paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect::<Vec<&[String]>>();
                                let mut packed_files = pack_file_decoded.get_ref_mut_packed_files_by_paths(paths);
                                packed_files.par_iter_mut()
                                    .filter(|x| [PackedFileType::DB, PackedFileType::Loc].contains(&x.get_packed_file_type(false)))
                                    .for_each(|x| {
                                    let _ = x.decode_no_locks(schema);
                                });
                            }
                        }
                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),

                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::CannotFindExtraPackFile(pack_file_path).into())),
                }
            }

            // In case we want to move stuff from our PackFile to an Animpack...
            Command::AddPackedFilesFromPackFileToAnimpack((anim_pack_path, paths)) => {
                let packed_files_to_add = pack_file_decoded.get_packed_files_by_path_type(&paths);
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&anim_pack_path) {
                    Some(packed_file) => {
                        let packed_file_decoded = packed_file.get_ref_mut_decoded();
                        match packed_file_decoded {
                            DecodedPackedFile::AnimPack(anim_pack) => match anim_pack.add_packed_files(&packed_files_to_add) {
                                Ok(paths) => CentralCommand::send_back(&sender, Response::VecPathType(paths)),
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileTypeIsNotWhatWeExpected(PackedFileType::AnimPack.to_string(), PackedFileType::from(&*packed_file_decoded).to_string()).into())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // In case we want to move stuff from an Animpack to our PackFile...
            Command::AddPackedFilesFromAnimpack((anim_pack_path, paths)) => {
                let packed_files_to_add = match pack_file_decoded.get_ref_packed_file_by_path(&anim_pack_path) {
                    Some(packed_file) => {
                        let packed_file_decoded = packed_file.get_ref_decoded();
                        match packed_file_decoded {
                            DecodedPackedFile::AnimPack(anim_pack) => anim_pack.get_anim_packed_as_packed_files(&paths),
                            _ => {
                                CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileTypeIsNotWhatWeExpected(PackedFileType::AnimPack.to_string(), PackedFileType::from(&*packed_file_decoded).to_string()).into()));
                                continue;
                            }
                        }
                    }
                    None => {
                        CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into()));
                        continue;
                    },
                };

                let packed_files_to_add = packed_files_to_add.iter().collect::<Vec<&PackedFile>>();
                match pack_file_decoded.add_packed_files(&packed_files_to_add, true, true) {
                    Ok(paths) => CentralCommand::send_back(&sender, Response::VecPathType(paths.iter().map(|x| PathType::File(x.to_vec())).collect())),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to delete files from an Animpack...
            Command::DeleteFromAnimpack((anim_pack_path, paths)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&anim_pack_path) {
                    Some(packed_file) => {
                        let packed_file_decoded = packed_file.get_ref_mut_decoded();
                        match packed_file_decoded {
                            DecodedPackedFile::AnimPack(anim_pack) => {
                                anim_pack.remove_packed_file_by_path_types(&paths);
                                CentralCommand::send_back(&sender, Response::Success);
                            }
                            _ => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileTypeIsNotWhatWeExpected(PackedFileType::AnimPack.to_string(), PackedFileType::from(&*packed_file_decoded).to_string()).into())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // In case we want to decode a RigidModel PackedFile...
            Command::DecodePackedFile(path, data_source) => {
                match data_source {
                    DataSource::PackFile => {
                        if path == [RESERVED_NAME_NOTES.to_owned()] {
                            let mut note = Text::new();
                            note.set_text_type(TextType::Markdown);
                            match pack_file_decoded.get_notes() {
                                Some(notes) => {
                                    note.set_contents(notes);
                                    CentralCommand::send_back(&sender, Response::Text(note));
                                }
                                None => CentralCommand::send_back(&sender, Response::Text(note)),
                            }
                        }

                        else {

                            // Find the PackedFile we want and send back the response.
                            match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                                Some(ref mut packed_file) => {
                                    match packed_file.decode_return_ref() {
                                        Ok(packed_file_data) => {
                                            match packed_file_data {
                                                DecodedPackedFile::AnimFragment(data) => CentralCommand::send_back(&sender, Response::AnimFragmentPackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::AnimPack(data) => CentralCommand::send_back(&sender, Response::AnimPackPackedFileInfo((data.get_as_pack_file_info(&path), From::from(&**packed_file)))),
                                                DecodedPackedFile::AnimTable(data) => CentralCommand::send_back(&sender, Response::AnimTablePackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::CaVp8(data) => CentralCommand::send_back(&sender, Response::CaVp8PackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::ESF(data) => CentralCommand::send_back(&sender, Response::ESFPackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::DB(table) => CentralCommand::send_back(&sender, Response::DBPackedFileInfo((table.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::Image(image) => CentralCommand::send_back(&sender, Response::ImagePackedFileInfo((image.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::Loc(table) => CentralCommand::send_back(&sender, Response::LocPackedFileInfo((table.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::MatchedCombat(data) => CentralCommand::send_back(&sender, Response::MatchedCombatPackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::RigidModel(rigid_model) => CentralCommand::send_back(&sender, Response::RigidModelPackedFileInfo((rigid_model.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::Text(text) => CentralCommand::send_back(&sender, Response::TextPackedFileInfo((text.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::UIC(uic) => CentralCommand::send_back(&sender, Response::UICPackedFileInfo((uic.clone(), From::from(&**packed_file)))),
                                                DecodedPackedFile::UnitVariant(_) => CentralCommand::send_back(&sender, Response::DecodedPackedFilePackedFileInfo((packed_file_data.clone(), From::from(&**packed_file)))),
                                                _ => CentralCommand::send_back(&sender, Response::Unknown),

                                            }
                                        }
                                        Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                    }
                                }
                                None => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::PackedFileNotFound))),
                            }
                        }
                    }

                    DataSource::ParentFiles => {
                        match dependencies.get_packedfile_from_parent_files(&path) {
                            Ok(mut packed_file) => {
                                match packed_file.decode_return_ref() {
                                    Ok(packed_file_data) => {
                                        match packed_file_data {
                                            DecodedPackedFile::AnimFragment(data) => CentralCommand::send_back(&sender, Response::AnimFragmentPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::AnimPack(data) => CentralCommand::send_back(&sender, Response::AnimPackPackedFileInfo((data.get_as_pack_file_info(&path), From::from(&packed_file)))),
                                            DecodedPackedFile::AnimTable(data) => CentralCommand::send_back(&sender, Response::AnimTablePackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::CaVp8(data) => CentralCommand::send_back(&sender, Response::CaVp8PackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::ESF(data) => CentralCommand::send_back(&sender, Response::ESFPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::DB(table) => CentralCommand::send_back(&sender, Response::DBPackedFileInfo((table.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Image(image) => CentralCommand::send_back(&sender, Response::ImagePackedFileInfo((image.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Loc(table) => CentralCommand::send_back(&sender, Response::LocPackedFileInfo((table.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::MatchedCombat(data) => CentralCommand::send_back(&sender, Response::MatchedCombatPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::RigidModel(rigid_model) => CentralCommand::send_back(&sender, Response::RigidModelPackedFileInfo((rigid_model.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Text(text) => CentralCommand::send_back(&sender, Response::TextPackedFileInfo((text.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::UIC(uic) => CentralCommand::send_back(&sender, Response::UICPackedFileInfo((uic.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::UnitVariant(_) => CentralCommand::send_back(&sender, Response::DecodedPackedFilePackedFileInfo((packed_file_data.clone(), From::from(&packed_file)))),
                                            _ => CentralCommand::send_back(&sender, Response::Unknown),

                                        }
                                    }
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }
                    DataSource::GameFiles => {
                        match dependencies.get_packedfile_from_game_files(&path) {
                            Ok(mut packed_file) => {
                                match packed_file.decode_return_ref() {
                                    Ok(packed_file_data) => {
                                        match packed_file_data {
                                            DecodedPackedFile::AnimFragment(data) => CentralCommand::send_back(&sender, Response::AnimFragmentPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::AnimPack(data) => CentralCommand::send_back(&sender, Response::AnimPackPackedFileInfo((data.get_as_pack_file_info(&path), From::from(&packed_file)))),
                                            DecodedPackedFile::AnimTable(data) => CentralCommand::send_back(&sender, Response::AnimTablePackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::CaVp8(data) => CentralCommand::send_back(&sender, Response::CaVp8PackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::ESF(data) => CentralCommand::send_back(&sender, Response::ESFPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::DB(table) => CentralCommand::send_back(&sender, Response::DBPackedFileInfo((table.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Image(image) => CentralCommand::send_back(&sender, Response::ImagePackedFileInfo((image.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Loc(table) => CentralCommand::send_back(&sender, Response::LocPackedFileInfo((table.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::MatchedCombat(data) => CentralCommand::send_back(&sender, Response::MatchedCombatPackedFileInfo((data.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::RigidModel(rigid_model) => CentralCommand::send_back(&sender, Response::RigidModelPackedFileInfo((rigid_model.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::Text(text) => CentralCommand::send_back(&sender, Response::TextPackedFileInfo((text.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::UIC(uic) => CentralCommand::send_back(&sender, Response::UICPackedFileInfo((uic.clone(), From::from(&packed_file)))),
                                            DecodedPackedFile::UnitVariant(_) => CentralCommand::send_back(&sender, Response::DecodedPackedFilePackedFileInfo((packed_file_data.clone(), From::from(&packed_file)))),
                                            _ => CentralCommand::send_back(&sender, Response::Unknown),

                                        }
                                    }
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }
                    DataSource::AssKitFiles => {
                        match dependencies.get_packedfile_from_asskit_files(&path) {
                            Ok(db) => CentralCommand::send_back(&sender, Response::DBPackedFileInfo((db, PackedFileInfo::default()))),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }

                    DataSource::ExternalFile => {}
                }
            }

            // When we want to save a PackedFile from the view....
            Command::SavePackedFileFromView(path, decoded_packed_file) => {
                if path == [RESERVED_NAME_NOTES.to_owned()] {
                    if let DecodedPackedFile::Text(data) = decoded_packed_file {
                        let note = if data.get_ref_contents().is_empty() { None } else { Some(data.get_ref_contents().to_owned()) };
                        pack_file_decoded.set_notes(&note);
                    }
                }
                else if let Some(packed_file) = pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    *packed_file.get_ref_mut_decoded() = decoded_packed_file;
                }
                CentralCommand::send_back(&sender, Response::Success);
            }

            // In case we want to delete PackedFiles from a PackFile...
            Command::DeletePackedFiles(item_types) => {
                CentralCommand::send_back(&sender, Response::VecPathType(pack_file_decoded.remove_packed_files_by_type(&item_types)));
            }

            // In case we want to extract PackedFiles from a PackFile...
            Command::ExtractPackedFiles(item_types, path, extract_tables_to_tsv) => {
                match pack_file_decoded.extract_packed_files_by_type(&item_types, &path, extract_tables_to_tsv) {
                    Ok(result) => CentralCommand::send_back(&sender, Response::String(tre("files_extracted_success", &[&result.to_string()]))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to rename one or more PackedFiles...
            Command::RenamePackedFiles(renaming_data) => {
                CentralCommand::send_back(&sender, Response::VecPathTypeVecString(pack_file_decoded.rename_packedfiles(&renaming_data, false)));
            }

            // In case we want to Mass-Import TSV Files...
            Command::MassImportTSV(paths, name) => {
                match pack_file_decoded.mass_import_tsv(&paths, name, true) {
                    Ok(result) => CentralCommand::send_back(&sender, Response::VecVecStringVecVecString(result)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to Mass-Export TSV Files...
            Command::MassExportTSV(path_types, path) => {
                match pack_file_decoded.mass_export_tsv(&path_types, &path) {
                    Ok(result) => CentralCommand::send_back(&sender, Response::String(result)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to know if a Folder exists, knowing his path...
            Command::FolderExists(path) => {
                CentralCommand::send_back(&sender, Response::Bool(pack_file_decoded.folder_exists(&path)));
            }

            // In case we want to know if PackedFile exists, knowing his path...
            Command::PackedFileExists(path) => {
                CentralCommand::send_back(&sender, Response::Bool(pack_file_decoded.packedfile_exists(&path)));
            }

            // In case we want to get the list of tables in the dependency database...
            Command::GetTableListFromDependencyPackFile => {
                let tables = if let Ok(tables) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true) {
                    tables.iter().map(|x| x.get_path()[1].to_owned()).collect::<Vec<String>>()
                } else { vec![] };
                CentralCommand::send_back(&sender, Response::VecString(tables));
            }

            // In case we want to get the version of an specific table from the dependency database...
            Command::GetTableVersionFromDependencyPackFile(table_name) => {
                if dependencies.game_has_vanilla_data_loaded(false) {
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        match schema.get_ref_last_definition_db(&table_name, &dependencies) {
                            Ok(definition) => CentralCommand::send_back(&sender, Response::I32(definition.get_version())),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())); }
                } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into())); }
            }

            // In case we want to get the definition of an specific table from the dependency database...
            Command::GetTableDefinitionFromDependencyPackFile(table_name) => {
                if dependencies.game_has_vanilla_data_loaded(false) {
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        match schema.get_ref_last_definition_db(&table_name, &dependencies) {
                            Ok(definition) => CentralCommand::send_back(&sender, Response::Definition(definition.clone())),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())); }
                } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into())); }
            }

            // In case we want to merge DB or Loc Tables from a PackFile...
            Command::MergeTables(paths, name, delete_source_files) => {
                match pack_file_decoded.merge_tables(&paths, &name, delete_source_files) {
                    Ok(data) => CentralCommand::send_back(&sender, Response::VecString(data)),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to update a table...
            Command::UpdateTable(path_type) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    if let PathType::File(path) = path_type {
                        if let Some(packed_file) = pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                            match packed_file.decode_return_ref_mut_no_locks(schema) {
                                Ok(packed_file_decoded) => match packed_file_decoded.update_table(&dependencies) {
                                    Ok(data) => {

                                        // Save it to binary, so the decoder will load the proper data if we open it with it.
                                        let _ = packed_file.encode_no_load();
                                        CentralCommand::send_back(&sender, Response::I32I32(data))
                                    },
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                            }
                        } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())); }
                    } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())); }
                } else { CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())); }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceMatches(mut global_search, matches) => {
                let _ = global_search.replace_matches(&mut pack_file_decoded, &matches);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CentralCommand::send_back(&sender, Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceAll(mut global_search) => {
                let _ = global_search.replace_all(&mut pack_file_decoded);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CentralCommand::send_back(&sender, Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to get the reference data for a definition...
            Command::GetReferenceDataFromDefinition(table_name, definition, files_to_ignore) => {

                // This is a heavy function, so first check if we have the data we want in the cache.
                let dependency_data = if dependencies.get_ref_cached_data().read().unwrap().get(&table_name).is_some() {
                    DB::get_dependency_data(
                        &pack_file_decoded,
                        &table_name,
                        &definition,
                        &[],
                        &[],
                        &dependencies,
                        &files_to_ignore,
                    )
                } else {
                    if let Ok(dependencies_vanilla) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true) {
                        DB::get_dependency_data(
                            &pack_file_decoded,
                            &table_name,
                            &definition,
                            &dependencies_vanilla,
                            dependencies.get_ref_asskit_only_db_tables(),
                            &dependencies,
                            &files_to_ignore,
                        )
                    } else { BTreeMap::new() }
                };

                CentralCommand::send_back(&sender, Response::BTreeMapI32DependencyData(dependency_data));
            }

            // In case we want to return an entire PackedFile to the UI.
            Command::GetPackedFile(path) => CentralCommand::send_back(&sender, Response::OptionPackedFile(pack_file_decoded.get_packed_file_by_path(&path))),

            // In case we want to change the format of a ca_vp8 video...
            Command::SetCaVp8Format((path, format)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(ref mut packed_file) => {
                        match packed_file.decode_return_ref_mut() {
                            Ok(data) => {
                                if let DecodedPackedFile::CaVp8(ref mut data) = data {
                                    data.set_format(format);
                                }
                                // TODO: Put an error here.
                            }
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::PackedFileNotFound))),
                }
            },

            // In case we want to save an schema to disk...
            Command::SaveSchema(mut schema) => {
                match schema.save(GAME_SELECTED.read().unwrap().get_schema_name()) {
                    Ok(_) => {
                        *SCHEMA.write().unwrap() = Some(schema);
                        CentralCommand::send_back(&sender, Response::Success);
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // In case we want to clean the cache of one or more PackedFiles...
            Command::CleanCache(paths) => {
                let mut packed_files = pack_file_decoded.get_ref_mut_packed_files_by_paths(paths.iter().map(|x| x.as_ref()).collect::<Vec<&[String]>>());
                packed_files.iter_mut().for_each(|x| { let _ = x.encode_and_clean_cache(); });
            }

            // In case we want to export a PackedFile as a TSV file...
            Command::ExportTSV((internal_path, external_path)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&internal_path) {
                    Some(packed_file) => match packed_file.get_decoded() {
                        DecodedPackedFile::DB(data) => match data.export_tsv(&external_path, &internal_path[1], &packed_file.get_path()) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                        },
                        DecodedPackedFile::Loc(data) => match data.export_tsv(&external_path, TSV_NAME_LOC, &packed_file.get_path()) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                        },
                        /*
                        DecodedPackedFile::DependencyPackFileList(data) => match data.export_tsv(&[external_path]) {
                            Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                        },*/
                        _ => unimplemented!()
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // In case we want to import a TSV as a PackedFile...
            Command::ImportTSV((internal_path, external_path)) => {
                match *SCHEMA.read().unwrap() {
                    Some(ref schema) => {
                        match pack_file_decoded.get_ref_mut_packed_file_by_path(&internal_path) {
                            Some(packed_file) => match packed_file.get_packed_file_type(false) {
                                PackedFileType::DB => match DB::import_tsv(&schema, &external_path) {
                                    Ok((data, _)) => CentralCommand::send_back(&sender, Response::TableType(TableType::DB(data))),
                                    Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                },
                                PackedFileType::Loc => match Loc::import_tsv(&schema, &external_path) {
                                    Ok((data, _)) => CentralCommand::send_back(&sender, Response::TableType(TableType::Loc(data))),
                                    Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                },
                                _ => unimplemented!()
                            }
                            None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())),
                }
            }

            // In case we want to open a PackFile's location in the file manager...
            Command::OpenContainingFolder => {

                // If the path exists, try to open it. If not, throw an error.
                if pack_file_decoded.get_file_path().exists() {
                    let mut temp_path = pack_file_decoded.get_file_path().to_path_buf();
                    temp_path.pop();
                    open::that_in_background(&temp_path);
                    CentralCommand::send_back(&sender, Response::Success);
                }
                else {
                    CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackFileIsNotAFile.into()));
                }
            },

            // When we want to open a PackedFile in a external program...
            Command::OpenPackedFileInExternalProgram(path) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(packed_file) => {
                        let extension = path.last().unwrap().rsplitn(2, '.').next().unwrap();
                        let name = format!("{}.{}", Uuid::new_v4(), extension);
                        let mut temporal_file_path = temp_dir();
                        temporal_file_path.push(name);
                        match packed_file.get_packed_file_type(false) {

                            // Tables we extract them as TSV.
                            PackedFileType::DB => {
                                match packed_file.decode_return_clean_cache() {
                                    Ok(data) => {
                                        if let DecodedPackedFile::DB(data) = data {
                                            temporal_file_path.set_extension("tsv");
                                            match data.export_tsv(&temporal_file_path, &path[1], &packed_file.get_path()) {
                                                Ok(_) => {
                                                    that_in_background(&temporal_file_path);
                                                    CentralCommand::send_back(&sender, Response::PathBuf(temporal_file_path));
                                                }
                                                Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                            }
                                        }
                                    },
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                            },

                            PackedFileType::Loc => {
                                match packed_file.decode_return_clean_cache() {
                                    Ok(data) => {
                                        if let DecodedPackedFile::Loc(data) = data {
                                            temporal_file_path.set_extension("tsv");
                                            match data.export_tsv(&temporal_file_path, TSV_NAME_LOC, &packed_file.get_path()) {
                                                Ok(_) => {
                                                    that_in_background(&temporal_file_path);
                                                    CentralCommand::send_back(&sender, Response::PathBuf(temporal_file_path));
                                                }
                                                Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                            }
                                        }
                                    },
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                            },

                            // The rest of the files, we extract them as we have them.
                            _ => {
                                match packed_file.get_raw_data_and_clean_cache() {
                                    Ok(data) => {
                                        match File::create(&temporal_file_path) {
                                            Ok(mut file) => {
                                                if file.write_all(&data).is_ok() {
                                                    that_in_background(&temporal_file_path);
                                                    CentralCommand::send_back(&sender, Response::PathBuf(temporal_file_path));
                                                }
                                                else {
                                                    CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1]))));
                                                }
                                            }
                                            Err(_) => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1])))),
                                        }
                                    }
                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                }
                            }
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // When we want to save a PackedFile from the external view....
            Command::SavePackedFileFromExternalView((path, external_path)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(packed_file) => {
                        match packed_file.get_packed_file_type(false) {

                            // Tables we extract them as TSV.
                            PackedFileType::DB | PackedFileType::Loc => {
                                match *SCHEMA.read().unwrap() {
                                    Some(ref schema) => {
                                        match packed_file.decode_return_ref_mut() {
                                            Ok(data) => {
                                                match data {
                                                    DecodedPackedFile::DB(ref mut data) => {
                                                        match DB::import_tsv(&schema, &external_path) {
                                                            Ok((new_data, _)) => {
                                                                *data = new_data;
                                                                match packed_file.encode_and_clean_cache() {
                                                                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                                                }
                                                            }
                                                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                                        }
                                                    }
                                                    DecodedPackedFile::Loc(ref mut data) => {
                                                        match Loc::import_tsv(&schema, &external_path) {
                                                            Ok((new_data, _)) => {
                                                                *data = new_data;
                                                                match packed_file.encode_and_clean_cache() {
                                                                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                                                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                                                                }
                                                            }
                                                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                                        }
                                                    }
                                                    _ => unimplemented!(),
                                                }
                                            },
                                            Err(error) =>  CentralCommand::send_back(&sender, Response::Error(error)),
                                        }
                                    }
                                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaNotFound.into())),
                                }
                            },

                            _ => {
                                match File::open(external_path) {
                                    Ok(mut file) => {
                                        let mut data = vec![];
                                        match file.read_to_end(&mut data) {
                                            Ok(_) => {
                                                packed_file.set_raw_data(&data);
                                                CentralCommand::send_back(&sender, Response::Success);
                                            }
                                            Err(_) => CentralCommand::send_back(&sender, Response::Error(ErrorKind::IOGeneric.into())),
                                        }
                                    }
                                    Err(_) => CentralCommand::send_back(&sender, Response::Error(ErrorKind::IOGeneric.into())),
                                }
                            }
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // When we want to update our schemas...
            Command::UpdateSchemas => {
                match Schema::update_schema_repo() {

                    // If it worked, we have to update the currently open schema with the one we just downloaded and rebuild cache/dependencies with it.
                    Ok(_) => {

                        // Encode the decoded tables with the old schema, then re-decode them with the new one.
                        pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false).par_iter_mut().for_each(|x| { let _ = x.encode_and_clean_cache(); });
                        *SCHEMA.write().unwrap() = Schema::load(GAME_SELECTED.read().unwrap().get_schema_name()).ok();
                        if let Some(ref schema) = *SCHEMA.read().unwrap() {
                            pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false).par_iter_mut().for_each(|x| { let _ = x.decode_no_locks(schema); });
                        }

                        // Try to reload the schema patchs. Ignore them if fails due to missing file.
                        if let Ok(schema_patches) = SchemaPatches::load() {
                            *SCHEMA_PATCHES.write().unwrap() = schema_patches;
                        }

                        // Then rebuild the dependencies stuff.
                        if dependencies.game_has_dependencies_generated() {
                            match dependencies.rebuild(pack_file_decoded.get_packfiles_list(), false) {
                                Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                                Err(error) => CentralCommand::send_back(&sender, Response::Error(ErrorKind::SchemaUpdateRebuildError(error.to_string()).into())),
                            }
                        }

                        // Otherwise, just report the schema update success, and don't leave the ui waiting eternally again...
                        else {
                            CentralCommand::send_back(&sender, Response::Success);
                        }
                    },
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our messages...
            Command::UpdateMessages => {

                // TODO: Properly reload all loaded tips.
                match Tips::update_from_repo() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our program...
            Command::UpdateMainProgram => {
                match rpfm_lib::updater::update_main_program() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            // When we want to update our program...
            Command::TriggerBackupAutosave => {

                // Note: we no longer notify the UI of success or error to not hang it up.
                if let Ok(Some(file)) = get_oldest_file_in_folder(&get_backup_autosave_path().unwrap()) {
                    let _ = pack_file_decoded.clone().save(Some(file));
                }
            }

            // In case we want to "Open one or more PackFiles"...
            Command::DiagnosticsCheck => {
                thread::spawn(clone!(
                    mut dependencies,
                    mut pack_file_decoded => move || {
                    let mut diag = Diagnostics::default();
                    if pack_file_decoded.get_pfh_file_type() == PFHFileType::Mod ||
                        pack_file_decoded.get_pfh_file_type() == PFHFileType::Movie {
                        diag.check(&pack_file_decoded, &dependencies);
                    }
                    CentralCommand::send_back(&sender, Response::Diagnostics(diag));
                }));
            }

            // In case we want to "Open one or more PackFiles"...
            Command::DiagnosticsUpdate((mut diagnostics, path_types)) => {
                diagnostics.update(&pack_file_decoded, &path_types, &dependencies);
                let packed_files_info = diagnostics.get_update_paths_packed_file_info(&pack_file_decoded, &path_types);
                CentralCommand::send_back(&sender, Response::DiagnosticsVecPackedFileInfo(diagnostics, packed_files_info));
            }

            // In case we want to get the open PackFile's Settings...
            Command::GetPackFileSettings(is_autosave) => {
                if is_autosave {
                    CentralCommand::send_back(&sender, Response::PackFileSettings(pack_file_decoded.get_settings().clone()));
                } else {
                    CentralCommand::send_back(&sender, Response::PackFileSettings(pack_file_decoded.get_settings().clone()));
                }
            }

            Command::SetPackFileSettings(settings) => {
                pack_file_decoded.set_settings(&settings);
            }

            Command::GetMissingDefinitions => {

                // Test to see if every DB Table can be decoded. This is slow and only useful when
                // a new patch lands and you want to know what tables you need to decode. So, unless you want
                // to decode new tables, leave the setting as false.
                if SETTINGS.read().unwrap().settings_bool["check_for_missing_table_definitions"] {
                    let mut counter = 0;
                    let mut table_list = String::new();
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        for packed_file in pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false) {
                            if packed_file.decode_return_ref_no_locks(schema).is_err() {
                                if let Ok(raw_data) = packed_file.get_raw_data() {
                                    if let Ok((_, _, _, entry_count, _)) = DB::read_header(&raw_data) {
                                        if entry_count > 0 {
                                            counter += 1;
                                            table_list.push_str(&format!("{}, {:?}\n", counter, packed_file.get_path()))
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Try to save the file. And I mean "try". Someone seems to love crashing here...
                    let path = RPFM_PATH.to_path_buf().join(PathBuf::from("missing_table_definitions.txt"));

                    if let Ok(file) = File::create(path) {
                        let mut file = BufWriter::new(file);
                        let _ = file.write_all(table_list.as_bytes());
                    }
                }
            }

            // Ignore errors for now.
            Command::RebuildDependencies(rebuild_only_current_mod_dependencies) => {
                let _ = dependencies.rebuild(pack_file_decoded.get_packfiles_list(), rebuild_only_current_mod_dependencies);
                let dependencies_info = DependenciesInfo::from(&dependencies);
                CentralCommand::send_back(&sender, Response::DependenciesInfo(dependencies_info));
            },

            Command::CascadeEdition(editions) => {
                let edited_paths = DB::cascade_edition(&editions, &mut pack_file_decoded);
                let edited_paths_2 = edited_paths.iter().map(|x| &**x).collect::<Vec<&[String]>>();
                let packed_files_info = pack_file_decoded.get_ref_packed_files_by_paths(edited_paths_2).iter().map(|x| PackedFileInfo::from(*x)).collect::<Vec<PackedFileInfo>>();
                CentralCommand::send_back(&sender, Response::VecVecStringVecPackedFileInfo(edited_paths, packed_files_info));
            }

            Command::GoToDefinition(ref_table, ref_column, ref_data) => {
                let table_folder = vec!["db".to_owned(), ref_table + "_tables"];
                let packed_files = pack_file_decoded.get_ref_packed_files_by_path_start(&table_folder);
                let mut found = false;
                for packed_file in &packed_files {
                    if let Ok(DecodedPackedFile::DB(data)) = packed_file.get_decoded_from_memory() {
                        if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data(&ref_column, &ref_data) {
                            CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::PackFile, packed_file.get_path().to_vec(), column_index, row_index));
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.get_db_and_loc_tables_from_cache(true, false, false, true) {
                        for packed_file in &packed_files {
                            if packed_file.get_path().starts_with(&table_folder) {
                                if let Ok(DecodedPackedFile::DB(data)) = packed_file.get_decoded_from_memory() {
                                    if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data(&ref_column, &ref_data) {
                                        CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::ParentFiles, packed_file.get_path().to_vec(), column_index, row_index));
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, false) {
                        for packed_file in &packed_files {
                            if packed_file.get_path().starts_with(&table_folder) {
                                if let Ok(DecodedPackedFile::DB(data)) = packed_file.get_decoded_from_memory() {
                                    if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data(&ref_column, &ref_data) {
                                        CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::GameFiles, packed_file.get_path().to_vec(), column_index, row_index));
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if !found {
                    let tables = dependencies.get_ref_asskit_only_db_tables();
                    for table in tables {
                        if table.get_ref_table_name() == table_folder[1] {
                            if let Some((column_index, row_index)) = table.get_ref_table().get_source_location_of_reference_data(&ref_column, &ref_data) {
                                let path = vec![table_folder[0].to_owned(), table_folder[1].to_owned(), "ak_data".to_owned()];
                                CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::AssKitFiles, path, column_index, row_index));
                                found = true;
                                break;
                            }
                        }
                    }
                }

                if !found {
                    CentralCommand::send_back(&sender, Response::Error(ErrorKind::GenericHTMLError(tr("source_data_for_field_not_found")).into()));
                }
            },

            Command::GoToLoc(loc_key) => {
                let packed_files = pack_file_decoded.get_ref_packed_files_by_type(PackedFileType::Loc, false);
                let mut found = false;
                for packed_file in &packed_files {
                    if let Ok(DecodedPackedFile::Loc(data)) = packed_file.get_decoded_from_memory() {
                        if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data("key", &loc_key) {
                            CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::PackFile, packed_file.get_path().to_vec(), column_index, row_index));
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.get_db_and_loc_tables_from_cache(false, true, false, true) {
                        for packed_file in &packed_files {
                            if let Ok(DecodedPackedFile::Loc(data)) = packed_file.get_decoded_from_memory() {
                                if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data("key", &loc_key) {
                                    CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::ParentFiles, packed_file.get_path().to_vec(), column_index, row_index));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    if let Ok(packed_files) = dependencies.get_db_and_loc_tables_from_cache(false, true, true, false) {
                        for packed_file in &packed_files {
                            if let Ok(DecodedPackedFile::Loc(data)) = packed_file.get_decoded_from_memory() {
                                if let Some((column_index, row_index)) = data.get_ref_table().get_source_location_of_reference_data("key", &loc_key) {
                                    CentralCommand::send_back(&sender, Response::DataSourceVecStringUsizeUsize(DataSource::GameFiles, packed_file.get_path().to_vec(), column_index, row_index));
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !found {
                    CentralCommand::send_back(&sender, Response::Error(ErrorKind::GenericHTMLError(tr("loc_key_not_found")).into()));
                }
            },

            Command::GetSourceDataFromLocKey(loc_key) => CentralCommand::send_back(&sender, Response::OptionStringStringString(Loc::get_source_location_of_loc_key(&loc_key, &dependencies))),
            Command::GetPackedFileType(path) => {
                let packed_file = RawPackedFile::read_from_vec(path, String::new(), 0, false, vec![]);
                CentralCommand::send_back(&sender, Response::PackedFileType(PackedFileType::get_packed_file_type(&packed_file, false)));
            }
            Command::GetPackFileName => CentralCommand::send_back(&sender, Response::String(pack_file_decoded.get_file_name())),
            Command::GetPackedFileRawData(path) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(ref mut packed_file) => {
                        match packed_file.get_ref_raw().get_raw_data() {
                            Ok(data) => CentralCommand::send_back(&sender, Response::VecU8(data.clone())),
                            Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                        }
                    }
                    None => CentralCommand::send_back(&sender, Response::Error(Error::from(ErrorKind::PackedFileNotFound))),
                }
            },

            Command::ImportDependenciesToOpenPackFile(paths_by_data_source) => {
                let mut added_paths = vec![];
                let mut error_paths = vec![];
                for (data_source, paths) in &paths_by_data_source {
                    let packed_files: Vec<PackedFile> = match data_source {
                        DataSource::GameFiles => {
                            match dependencies.get_packedfiles_from_game_files(paths) {
                                Ok((packed_files, mut errors)) => {
                                    error_paths.append(&mut errors);
                                    packed_files
                                }
                                Err(error) => {
                                    CentralCommand::send_back(&sender, Response::Error(error));
                                    CentralCommand::send_back(&sender, Response::Success);
                                    continue 'background_loop;
                                },
                            }
                        }
                        DataSource::ParentFiles => {
                            match dependencies.get_packedfiles_from_parent_files(paths) {
                                Ok((packed_files, mut errors)) => {
                                    error_paths.append(&mut errors);
                                    packed_files
                                }
                                Err(error) => {
                                    CentralCommand::send_back(&sender, Response::Error(error));
                                    CentralCommand::send_back(&sender, Response::Success);
                                    continue 'background_loop;
                                },
                            }
                        },

                        _ => {
                            CentralCommand::send_back(&sender, Response::Error(ErrorKind::Generic.into()));
                            CentralCommand::send_back(&sender, Response::Success);
                            continue 'background_loop;
                        },
                    };

                    let packed_files_ref = packed_files.iter().collect::<Vec<&PackedFile>>();
                    added_paths.append(&mut pack_file_decoded.add_packed_files(&packed_files_ref, true, true).unwrap());
                }

                if !error_paths.is_empty() {
                    CentralCommand::send_back(&sender, Response::VecPathType(added_paths.iter().map(|x| PathType::File(x.to_vec())).collect()));
                    CentralCommand::send_back(&sender, Response::VecVecString(error_paths));
                } else {
                    CentralCommand::send_back(&sender, Response::VecPathType(added_paths.iter().map(|x| PathType::File(x.to_vec())).collect()));
                    CentralCommand::send_back(&sender, Response::Success);
                }
            },

            Command::GetPackedFilesFromAllSources(paths) => {
                let mut packed_files = HashMap::new();

                // Get PackedFiles requested from the Parent Files.
                let mut packed_files_parent = HashMap::new();
                if let Ok((packed_files_decoded, _)) = dependencies.get_packedfiles_from_parent_files_unicased(&paths) {
                    for packed_file in packed_files_decoded {
                        packed_files_parent.insert(packed_file.get_path().to_vec(), packed_file);
                    }
                    packed_files.insert(DataSource::ParentFiles, packed_files_parent);
                }

                // Get PackedFiles requested from the Game Files.
                let mut packed_files_game = HashMap::new();
                if let Ok((packed_files_decoded, _)) = dependencies.get_packedfiles_from_game_files_unicased(&paths) {
                    for packed_file in packed_files_decoded {
                        packed_files_game.insert(packed_file.get_path().to_vec(), packed_file);
                    }
                    packed_files.insert(DataSource::GameFiles, packed_files_game);
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
                for packed_file in pack_file_decoded.get_packed_files_by_path_type_unicased(&paths) {
                    packed_files_packfile.insert(packed_file.get_path().to_vec(), packed_file );
                }
                packed_files.insert(DataSource::PackFile, packed_files_packfile);

                // Return the full list of PackedFiles requested, split by source.
                CentralCommand::send_back(&sender, Response::HashMapDataSourceHashMapVecStringPackedFile(packed_files));
            },

            Command::GetPackedFilesNamesStartingWitPathFromAllSources(path) => {
                let mut packed_files = HashMap::new();
                let base_path = if let PathType::Folder(ref path) = path { path.to_vec() } else { unimplemented!() };

                // Get PackedFiles requested from the Parent Files.
                let mut packed_files_parent = HashSet::new();
                if let Ok((packed_files_decoded, _)) = dependencies.get_packedfiles_from_parent_files_unicased(&[path.clone()]) {
                    for packed_file in packed_files_decoded {
                        let packed_file_path = packed_file.get_path()[base_path.len() - 1..].to_vec();
                        packed_files_parent.insert(packed_file_path);
                    }
                    packed_files.insert(DataSource::ParentFiles, packed_files_parent);
                }

                // Get PackedFiles requested from the Game Files.
                let mut packed_files_game = HashSet::new();
                if let Ok((packed_files_decoded, _)) = dependencies.get_packedfiles_from_game_files_unicased(&[path.clone()]) {
                    for packed_file in packed_files_decoded {
                        let packed_file_path = packed_file.get_path()[base_path.len() - 1..].to_vec();
                        packed_files_game.insert(packed_file_path);
                    }
                    packed_files.insert(DataSource::GameFiles, packed_files_game);
                }

                // Get PackedFiles requested from the currently open PackFile, if any.
                let mut packed_files_packfile = HashSet::new();
                for packed_file in pack_file_decoded.get_packed_files_by_path_type_unicased(&[path]) {
                    let packed_file_path = packed_file.get_path()[base_path.len() - 1..].to_vec();
                    packed_files_packfile.insert(packed_file_path);
                }
                packed_files.insert(DataSource::PackFile, packed_files_packfile);

                // Return the full list of PackedFile names requested, split by source.
                CentralCommand::send_back(&sender, Response::HashMapDataSourceHashSetVecString(packed_files));
            },

            Command::SavePackedFilesToPackFileAndClean(packed_files) => {

                // We receive a list of edited PackedFiles. The UI is the one that takes care of editing them to have the data we want where we want.
                // Also, the UI is responsible for naming them in case they're new. Here we grab them and directly add them into the PackFile.
                let packed_files = packed_files.iter().collect::<Vec<&PackedFile>>();
                let mut added_paths = vec![];
                if let Ok(mut paths) = pack_file_decoded.add_packed_files(&packed_files, true, true) {
                    added_paths.append(&mut paths);
                }

                // Clean up duplicates from overwrites.
                added_paths.sort();
                added_paths.dedup();

                // Then, optimize the PackFile. This should remove any non-edited rows/files.
                match pack_file_decoded.optimize(&dependencies) {
                    Ok(paths_to_delete) => CentralCommand::send_back(&sender, Response::VecVecStringVecVecString((added_paths, paths_to_delete))),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            },

            Command::GetTipsForPath(path) => {
                let local_tips = tips.get_local_tips_for_path(&path);
                let remote_tips = tips.get_remote_tips_for_path(&path);
                CentralCommand::send_back(&sender, Response::VecTipVecTip(local_tips, remote_tips));
            }

            Command::AddTipToLocalTips(tip) => {
                tips.add_tip_to_local_tips(tip);
                match tips.save() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::DeleteTipById(id) => {
                tips.delete_tip_by_id(id);
                match tips.save() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::PublishTipById(id) => {
                match tips.publish_tip_by_id(id) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::UploadSchemaPatch(patch) => {
                match patch.upload() {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }

            Command::ImportSchemaPatch(patch) => {
                match SCHEMA_PATCHES.write().unwrap().import(patch) {
                    Ok(_) => CentralCommand::send_back(&sender, Response::Success),
                    Err(error) => CentralCommand::send_back(&sender, Response::Error(error)),
                }
            }


            // These two belong to the network thread, not to this one!!!!
            Command::CheckUpdates | Command::CheckSchemaUpdates | Command::CheckMessageUpdates => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
}
