//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use open::that_in_background;
use rayon::prelude::*;
use uuid::Uuid;

use std::collections::BTreeMap;
use std::env::temp_dir;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use rpfm_error::{Error, ErrorKind};
use rpfm_lib::assembly_kit::*;
use rpfm_lib::DEPENDENCY_DATABASE;
use rpfm_lib::FAKE_DEPENDENCY_DATABASE;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::packedfile::*;
use rpfm_lib::packedfile::animpack::AnimPack;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::loc::{Loc, TSV_NAME_LOC};
use rpfm_lib::packedfile::text::{Text, TextType};
use rpfm_lib::packfile::{PackFile, PackFileInfo, packedfile::PackedFile, PathType, PFHFlags};
use rpfm_lib::schema::*;
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::template::Template;

use crate::app_ui::NewPackedFile;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::tre;
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
    // - `pack_file_decoded_extra`: This one will hold the PackFile opened for the `add_from_packfile` feature.
    let mut pack_file_decoded = PackFile::new();
    let mut pack_file_decoded_extra = PackFile::new();

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        let response = CENTRAL_COMMAND.recv_message_rust();
        match response {

            // In case we want to reset the PackFile to his original state (dummy)...
            Command::ResetPackFile => pack_file_decoded = PackFile::new(),

            // In case we want to reset the Secondary PackFile to his original state (dummy)...
            Command::ResetPackFileExtra => pack_file_decoded_extra = PackFile::new(),

            // In case we want to create a "New PackFile"...
            Command::NewPackFile => {
                let game_selected = GAME_SELECTED.read().unwrap();
                let pack_version = SUPPORTED_GAMES.get(&**game_selected).unwrap().pfh_version[0];
                pack_file_decoded = PackFile::new_with_name("unknown.pack", pack_version);
            }

            // In case we want to "Open one or more PackFiles"...
            Command::OpenPackFiles(paths) => {
                match PackFile::open_packfiles(&paths, SETTINGS.read().unwrap().settings_bool["use_lazy_loading"], false, false) {
                    Ok(pack_file) => {
                        pack_file_decoded = pack_file;
                        CENTRAL_COMMAND.send_message_rust(Response::PackFileInfo(PackFileInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to "Open an Extra PackFile" (for "Add from PackFile")...
            Command::OpenPackFileExtra(path) => {
                match PackFile::open_packfiles(&[path], true, false, true) {
                     Ok(pack_file) => {
                        pack_file_decoded_extra = pack_file;
                        CENTRAL_COMMAND.send_message_rust(Response::PackFileInfo(PackFileInfo::from(&pack_file_decoded_extra)));
                    }
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to "Load All CA PackFiles"...
            Command::LoadAllCAPackFiles => {
                match PackFile::open_all_ca_packfiles() {
                    Ok(pack_file) => {
                        pack_file_decoded = pack_file;
                        CENTRAL_COMMAND.send_message_rust(Response::PackFileInfo(PackFileInfo::from(&pack_file_decoded)));
                    }
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to "Save a PackFile"...
            Command::SavePackFile => {
                match pack_file_decoded.save(None) {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::PackFileInfo(From::from(&pack_file_decoded))),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::SavePackFileGeneric(error.to_string())))),
                }
            }

            // In case we want to "Save a PackFile As"...
            Command::SavePackFileAs(path) => {
                match pack_file_decoded.save(Some(path.to_path_buf())) {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::PackFileInfo(From::from(&pack_file_decoded))),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::SavePackFileGeneric(error.to_string())))),
                }
            }

            // In case we want to change the current settings...
            Command::SetSettings(settings) => {
                *SETTINGS.write().unwrap() = settings;
                match SETTINGS.read().unwrap().save() {
                    Ok(()) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to change the current shortcuts...
            Command::SetShortcuts(shortcuts) => {
                match shortcuts.save() {
                    Ok(()) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to get the data of a PackFile needed to form the TreeView...
            Command::GetPackFileDataForTreeView => {

                // Get the name and the PackedFile list, and send it.
                CENTRAL_COMMAND.send_message_rust(Response::PackFileInfoVecPackedFileInfo((
                    From::from(&pack_file_decoded),
                    pack_file_decoded.get_packed_files_all_info(),

                )));
            }

            // In case we want to get the data of a Secondary PackFile needed to form the TreeView...
            Command::GetPackFileExtraDataForTreeView => {

                // Get the name and the PackedFile list, and serialize it.
                CENTRAL_COMMAND.send_message_rust(Response::PackFileInfoVecPackedFileInfo((
                    From::from(&pack_file_decoded_extra),
                    pack_file_decoded_extra.get_packed_files_all_info(),

                )));
            }

            // In case we want to get the info of one PackedFile from the TreeView.
            Command::GetPackedFileInfo(path) => {
                CENTRAL_COMMAND.send_message_rust(Response::OptionPackedFileInfo(
                    pack_file_decoded.get_packed_file_info_by_path(&path)
                ));
            }

            // In case we want to get the info of more than one PackedFiles from the TreeView.
            Command::GetPackedFilesInfo(paths) => {
                CENTRAL_COMMAND.send_message_rust(Response::VecOptionPackedFileInfo(
                    paths.iter().map(|x| pack_file_decoded.get_packed_file_info_by_path(x)).collect()
                ));
            }

            // In case we want to launch a global search on a `PackFile`...
            Command::GlobalSearch(mut global_search) => {
                global_search.search(&mut pack_file_decoded);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CENTRAL_COMMAND.send_message_rust(Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to update the results of a global search on a `PackFile`...
            Command::GlobalSearchUpdate(mut global_search, path_types) => {
                global_search.update(&mut pack_file_decoded, &path_types);
                let packed_files_info = global_search.get_update_paths_packed_file_info(&mut pack_file_decoded, &path_types);
                CENTRAL_COMMAND.send_message_rust(Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to change the current `Game Selected`...
            Command::SetGameSelected(game_selected) => {
                *GAME_SELECTED.write().unwrap() = game_selected.to_owned();

                // Try to load the Schema for this game but, before it, PURGE THE DAMN SCHEMA-RELATED CACHE.
                pack_file_decoded.get_ref_mut_packed_files_by_type(PackedFileType::DB, false).iter_mut().for_each(|x| { let _ = x.encode_and_clean_cache(); });
                *SCHEMA.write().unwrap() = Schema::load(&SUPPORTED_GAMES.get(&*game_selected).unwrap().schema).ok();

                // Send a response, so we can unlock the UI.
                CENTRAL_COMMAND.send_message_rust(Response::Success);

                // Change the `dependency_database` for that game.
                *DEPENDENCY_DATABASE.lock().unwrap() = PackFile::load_all_dependency_packfiles(&pack_file_decoded.get_packfiles_list());

                // Change the `fake dependency_database` for that game.
                *FAKE_DEPENDENCY_DATABASE.write().unwrap() = DB::read_pak_file();

                // If there is a PackFile open, change his id to match the one of the new `Game Selected`.
                if !pack_file_decoded.get_file_name().is_empty() {
                    pack_file_decoded.set_pfh_version(SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().pfh_version[0]);
                }

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

                    // Try to save the file.
                    let path = RPFM_PATH.to_path_buf().join(PathBuf::from("missing_table_definitions.txt"));
                    let mut file = BufWriter::new(File::create(path).unwrap());
                    file.write_all(table_list.as_bytes()).unwrap();
                }
            }

            // In case we want to generate a new Pak File for our Game Selected...
            Command::GeneratePakFile(path, version) => {
                match generate_pak_file(&path, version) {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }

                // Reload the `fake dependency_database` for that game.
                *FAKE_DEPENDENCY_DATABASE.write().unwrap() = DB::read_pak_file();
            }

            // In case we want to update the Schema for our Game Selected...
            Command::UpdateCurrentSchemaFromAssKit(path) => {
                match update_schema_from_raw_files(path) {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to optimize our PackFile...
            Command::OptimizePackFile => {
                CENTRAL_COMMAND.send_message_rust(Response::VecVecString(pack_file_decoded.optimize()));
            }

            // In case we want to Patch the SiegeAI of a PackFile...
            Command::PatchSiegeAI => {
                match pack_file_decoded.patch_siege_ai() {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::StringVecVecString(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error))
                }
            }

            // In case we want to change the PackFile's Type...
            Command::SetPackFileType(new_type) => pack_file_decoded.set_pfh_file_type(new_type),

            // In case we want to change the "Include Last Modified Date" setting of the PackFile...
            Command::ChangeIndexIncludesTimestamp(state) => pack_file_decoded.get_ref_mut_bitmask().set(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, state),

            // In case we want to compress/decompress the PackedFiles of the currently open PackFile...
            Command::ChangeDataIsCompressed(state) => pack_file_decoded.toggle_compression(state),

            // In case we want to get the path of the currently open `PackFile`.
            Command::GetPackFilePath => CENTRAL_COMMAND.send_message_rust(Response::PathBuf(pack_file_decoded.get_file_path().to_path_buf())),

            // In case we want to get the Dependency PackFiles of our PackFile...
            Command::GetDependencyPackFilesList => CENTRAL_COMMAND.send_message_rust(Response::VecString(pack_file_decoded.get_packfiles_list().to_vec())),

            // In case we want to set the Dependency PackFiles of our PackFile...
            Command::SetDependencyPackFilesList(pack_files) => pack_file_decoded.set_packfiles_list(&pack_files),

            // In case we want to check if there is a Dependency Database loaded...
            Command::IsThereADependencyDatabase => CENTRAL_COMMAND.send_message_rust(Response::Bool(!DEPENDENCY_DATABASE.lock().unwrap().is_empty())),

            // In case we want to check if there is a Schema loaded...
            Command::IsThereASchema => CENTRAL_COMMAND.send_message_rust(Response::Bool(SCHEMA.read().unwrap().is_some())),

            // In case we want to create a PackedFile from scratch...
            Command::NewPackedFile(path, new_packed_file) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    let decoded = match new_packed_file {
                        NewPackedFile::DB(_, table, version) => {
                            match schema.get_ref_versioned_file_db(&table) {
                                Ok(versioned_file) => {
                                    match versioned_file.get_version(version) {
                                        Ok(definition) =>  DecodedPackedFile::DB(DB::new(&table, None, definition)),
                                        Err(error) => {
                                            CENTRAL_COMMAND.send_message_rust(Response::Error(error));
                                            continue;
                                        }
                                    }
                                }
                                Err(error) => {
                                    CENTRAL_COMMAND.send_message_rust(Response::Error(error));
                                    continue;
                                }
                            }
                        },
                        NewPackedFile::Loc(_) => {
                            match schema.get_ref_last_definition_loc() {
                                Ok(definition) => DecodedPackedFile::Loc(Loc::new(definition)),
                                Err(error) => {
                                    CENTRAL_COMMAND.send_message_rust(Response::Error(error));
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
                        Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                        Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                    }
                } else { CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::SchemaNotFound.into())); }
            }

            // When we want to add one or more PackedFiles to our PackFile...
            Command::AddPackedFiles((source_paths, destination_paths)) => {
                let mut broke = false;
                for (source_path, destination_path) in source_paths.iter().zip(destination_paths.iter()) {
                    if let Err(error) = pack_file_decoded.add_from_file(source_path, destination_path.to_vec(), true) {
                        CENTRAL_COMMAND.send_message_rust(Response::Error(error));
                        broke = true;
                        break;
                    }
                }

                // If nothing failed, send back success.
                if !broke {
                    CENTRAL_COMMAND.send_message_rust(Response::Success);
                }
            }

            // In case we want to add one or more entire folders to our PackFile...
            Command::AddPackedFilesFromFolder(paths) => {
                match pack_file_decoded.add_from_folders(&paths, true) {
                    Ok(paths) => CENTRAL_COMMAND.send_message_rust(Response::VecPathType(paths.iter().map(|x| PathType::File(x.to_vec())).collect())),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),

                }
            }

            // In case we want to move stuff from one PackFile to another...
            Command::AddPackedFilesFromPackFile(paths) => {

                // Try to add the PackedFile to the main PackFile.
                match pack_file_decoded.add_from_packfile(&pack_file_decoded_extra, &paths, true) {
                    Ok(paths) => CENTRAL_COMMAND.send_message_rust(Response::VecPathType(paths)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),

                }
            }

            // In case we want to decode a RigidModel PackedFile...
            Command::DecodePackedFile(path) => {

                if path == ["notes.rpfm_reserved".to_owned()] {
                    let mut note = Text::new();
                    note.set_text_type(TextType::Markdown);
                    match pack_file_decoded.get_notes() {
                        Some(notes) => {
                            note.set_contents(notes);
                            CENTRAL_COMMAND.send_message_rust(Response::Text(note));
                        }
                        None => CENTRAL_COMMAND.send_message_rust(Response::Text(note)),
                    }
                }
                else {

                    // Find the PackedFile we want and send back the response.
                    match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                        Some(ref mut packed_file) => {
                            match packed_file.decode_return_ref() {
                                Ok(packed_file_data) => {
                                    match packed_file_data {
                                        DecodedPackedFile::AnimFragment(data) => CENTRAL_COMMAND.send_message_rust(Response::AnimFragmentPackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::AnimPack(data) => CENTRAL_COMMAND.send_message_rust(Response::AnimPackPackedFileInfo((data.get_file_list(), From::from(&**packed_file)))),
                                        DecodedPackedFile::AnimTable(data) => CENTRAL_COMMAND.send_message_rust(Response::AnimTablePackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::CaVp8(data) => CENTRAL_COMMAND.send_message_rust(Response::CaVp8PackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::DB(table) => CENTRAL_COMMAND.send_message_rust(Response::DBPackedFileInfo((table.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::Image(image) => CENTRAL_COMMAND.send_message_rust(Response::ImagePackedFileInfo((image.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::Loc(table) => CENTRAL_COMMAND.send_message_rust(Response::LocPackedFileInfo((table.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::MatchedCombat(data) => CENTRAL_COMMAND.send_message_rust(Response::MatchedCombatPackedFileInfo((data.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::RigidModel(rigid_model) => CENTRAL_COMMAND.send_message_rust(Response::RigidModelPackedFileInfo((rigid_model.clone(), From::from(&**packed_file)))),
                                        DecodedPackedFile::Text(text) => CENTRAL_COMMAND.send_message_rust(Response::TextPackedFileInfo((text.clone(), From::from(&**packed_file)))),
                                        _ => CENTRAL_COMMAND.send_message_rust(Response::Unknown),

                                    }
                                }
                                Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                            }
                        }
                        None => CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::PackedFileNotFound))),
                    }
                }
            }

            // When we want to save a PackedFile from the view....
            Command::SavePackedFileFromView(path, decoded_packed_file) => {
                if path == ["notes.rpfm_reserved".to_owned()] {
                    if let DecodedPackedFile::Text(data) = decoded_packed_file {
                        let note = if data.get_ref_contents().is_empty() { None } else { Some(data.get_ref_contents().to_owned()) };
                        pack_file_decoded.set_notes(&note);
                    }
                }
                else if let Some(packed_file) = pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    *packed_file.get_ref_mut_decoded() = decoded_packed_file;
                }
                CENTRAL_COMMAND.send_message_rust(Response::Success);
            }

            // In case we want to delete PackedFiles from a PackFile...
            Command::DeletePackedFiles(item_types) => {
                CENTRAL_COMMAND.send_message_rust(Response::VecPathType(pack_file_decoded.remove_packed_files_by_type(&item_types)));
            }

            // In case we want to extract PackedFiles from a PackFile...
            Command::ExtractPackedFiles(item_types, path) => {
                match pack_file_decoded.extract_packed_files_by_type(&item_types, &path) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::String(tre("files_extracted_success", &[&result.to_string()]))),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to rename one or more PackedFiles...
            Command::RenamePackedFiles(renaming_data) => {
                CENTRAL_COMMAND.send_message_rust(Response::VecPathTypeVecString(pack_file_decoded.rename_packedfiles(&renaming_data, false)));
            }

            // In case we want to Mass-Import TSV Files...
            Command::MassImportTSV(paths, name) => {
                match pack_file_decoded.mass_import_tsv(&paths, name, true) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::VecVecStringVecVecString(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to Mass-Export TSV Files...
            Command::MassExportTSV(path_types, path) => {
                match pack_file_decoded.mass_export_tsv(&path_types, &path) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::String(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to know if a Folder exists, knowing his path...
            Command::FolderExists(path) => {
                CENTRAL_COMMAND.send_message_rust(Response::Bool(pack_file_decoded.folder_exists(&path)));
            }

            // In case we want to know if PackedFile exists, knowing his path...
            Command::PackedFileExists(path) => {
                CENTRAL_COMMAND.send_message_rust(Response::Bool(pack_file_decoded.packedfile_exists(&path)));
            }

            // In case we want to get the list of tables in the dependency database...
            Command::GetTableListFromDependencyPackFile => {
                let tables = (*DEPENDENCY_DATABASE.lock().unwrap()).par_iter().filter(|x| x.get_path().len() > 2).filter(|x| x.get_path()[1].ends_with("_tables")).map(|x| x.get_path()[1].to_owned()).collect::<Vec<String>>();
                CENTRAL_COMMAND.send_message_rust(Response::VecString(tables));
            }

            // In case we want to get the version of an specific table from the dependency database...
            Command::GetTableVersionFromDependencyPackFile(table_name) => {
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    match schema.get_ref_last_definition_db(&table_name) {
                        Ok(definition) => CENTRAL_COMMAND.send_message_rust(Response::I32(definition.get_version())),
                        Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                    }
                } else { CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::SchemaNotFound.into())); }
            }

            // In case we want to check the DB tables for dependency errors...
            Command::DBCheckTableIntegrity => {
                match pack_file_decoded.check_table_integrity() {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to merge DB or Loc Tables from a PackFile...
            Command::MergeTables(paths, name, delete_source_files) => {
                match pack_file_decoded.merge_tables(&paths, &name, delete_source_files) {
                    Ok(data) => CENTRAL_COMMAND.send_message_rust(Response::VecString(data)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // In case we want to update a table...
            Command::UpdateTable(path_type) => {
                if let PathType::File(path) = path_type {
                    if let Some(packed_file) = pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                        match packed_file.decode_return_ref_mut() {
                            Ok(packed_file) => match packed_file.update_table() {
                                    Ok(data) => CENTRAL_COMMAND.send_message_rust(Response::I32I32(data)),
                                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                }
                            Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        }
                    } else { CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())); }
                } else { CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())); }
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceMatches(mut global_search, matches) => {
                let _ = global_search.replace_matches(&mut pack_file_decoded, &matches);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CENTRAL_COMMAND.send_message_rust(Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to replace all matches in a Global Search...
            Command::GlobalSearchReplaceAll(mut global_search) => {
                let _ = global_search.replace_all(&mut pack_file_decoded);
                let packed_files_info = global_search.get_results_packed_file_info(&mut pack_file_decoded);
                CENTRAL_COMMAND.send_message_rust(Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)));
            }

            // In case we want to get the reference data for a definition...
            Command::GetReferenceDataFromDefinition(definition, files_to_ignore) => {
                let dependency_data = match &*SCHEMA.read().unwrap() {
                    Some(ref schema) => {
                        let mut dep_db = DEPENDENCY_DATABASE.lock().unwrap();
                        let fake_dep_db = FAKE_DEPENDENCY_DATABASE.read().unwrap();

                        DB::get_dependency_data(
                            &mut pack_file_decoded,
                            schema,
                            &definition,
                            &mut dep_db,
                            &fake_dep_db,
                            &files_to_ignore,
                        )
                    }
                    None => BTreeMap::new(),
                };
                CENTRAL_COMMAND.send_message_rust(Response::BTreeMapI32BTreeMapStringString(dependency_data));
            }

            // In case we want to return an entire PackedFile to the UI.
            Command::GetPackedFile(path) => CENTRAL_COMMAND.send_message_rust(Response::OptionPackedFile(pack_file_decoded.get_packed_file_by_path(&path))),

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
                            Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        }
                    }
                    None => CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::PackedFileNotFound))),
                }
            },

            // In case we want to save an schema to disk...
            Command::SaveSchema(mut schema) => {
                match schema.save(&SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().schema) {
                    Ok(_) => {
                        *SCHEMA.write().unwrap() = Some(schema);
                        CENTRAL_COMMAND.send_message_rust(Response::Success);
                    },
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
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
                        DecodedPackedFile::DB(data) => match data.export_tsv(&external_path, &internal_path[1]) {
                            Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },
                        DecodedPackedFile::Loc(data) => match data.export_tsv(&external_path, &TSV_NAME_LOC) {
                            Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },
                        /*
                        DecodedPackedFile::DependencyPackFileList(data) => match data.export_tsv(&[external_path]) {
                            Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },*/
                        _ => unimplemented!()
                    }
                    None => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // In case we want to import a TSV as a PackedFile...
            Command::ImportTSV((internal_path, external_path)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&internal_path) {
                    Some(packed_file) => match packed_file.get_decoded() {
                        DecodedPackedFile::DB(data) => match DB::import_tsv(&data.get_definition(), &external_path, &internal_path[1]) {
                            Ok(data) => CENTRAL_COMMAND.send_message_rust(Response::TableType(TableType::DB(data))),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },
                        DecodedPackedFile::Loc(data) => match Loc::import_tsv(&data.get_definition(), &external_path, &TSV_NAME_LOC) {
                            Ok(data) => CENTRAL_COMMAND.send_message_rust(Response::TableType(TableType::Loc(data))),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },
                        /*
                        DecodedPackedFile::DependencyPackFileList(data) => match data.export_tsv(&[external_path]) {
                            Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                            Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                        },*/
                        _ => unimplemented!()
                    }
                    None => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // In case we want to open a PackFile's location in the file manager...
            Command::OpenContainingFolder => {

                // If the path exists, try to open it. If not, throw an error.
                if pack_file_decoded.get_file_path().exists() {
                    let mut temp_path = pack_file_decoded.get_file_path().to_path_buf();
                    temp_path.pop();
                    if open::that(&temp_path).is_err() {
                        CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackFileIsNotAFile.into()));
                    }
                    else {
                        CENTRAL_COMMAND.send_message_rust(Response::Success);
                    }
                }
                else {
                    CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackFileIsNotAFile.into()));
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
                        match packed_file.get_packed_file_type_by_path() {

                            // Tables we extract them as TSV.
                            PackedFileType::DB => {
                                match packed_file.decode_return_clean_cache() {
                                    Ok(data) => {
                                        if let DecodedPackedFile::DB(data) = data {
                                            temporal_file_path.set_extension("tsv");
                                            match data.export_tsv(&temporal_file_path, &path[1]) {
                                                Ok(_) => {
                                                    that_in_background(&temporal_file_path);
                                                    CENTRAL_COMMAND.send_message_rust(Response::PathBuf(temporal_file_path));
                                                }
                                                Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                            }
                                        }
                                    },
                                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                }
                            },

                            PackedFileType::Loc => {
                                match packed_file.decode_return_clean_cache() {
                                    Ok(data) => {
                                        if let DecodedPackedFile::Loc(data) = data {
                                            temporal_file_path.set_extension("tsv");
                                            match data.export_tsv(&temporal_file_path, &TSV_NAME_LOC) {
                                                Ok(_) => {
                                                    that_in_background(&temporal_file_path);
                                                    CENTRAL_COMMAND.send_message_rust(Response::PathBuf(temporal_file_path));
                                                }
                                                Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                            }
                                        }
                                    },
                                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
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
                                                    CENTRAL_COMMAND.send_message_rust(Response::PathBuf(temporal_file_path));
                                                }
                                                else {
                                                    CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1]))));
                                                }
                                            }
                                            Err(_) => CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1])))),
                                        }
                                    }
                                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                }
                            }
                        }
                    }
                    None => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // When we want to save a PackedFile from the external view....
            Command::SavePackedFileFromExternalView((path, external_path)) => {
                match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(packed_file) => {
                        match packed_file.get_packed_file_type_by_path() {

                            // Tables we extract them as TSV.
                            PackedFileType::DB | PackedFileType::Loc => {
                                match packed_file.decode_return_ref_mut() {
                                    Ok(data) => {
                                        if let DecodedPackedFile::DB(ref mut data) = data {
                                            match DB::import_tsv(&data.get_definition(), &external_path, &path[1]) {
                                                Ok(new_data) => {
                                                    *data = new_data;
                                                    match packed_file.encode_and_clean_cache() {
                                                        Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                                                        Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                                    }
                                                }
                                                Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                            }
                                        }
                                        else if let DecodedPackedFile::Loc(ref mut data) = data {
                                            match Loc::import_tsv(&data.get_definition(), &external_path, &TSV_NAME_LOC) {
                                                Ok(new_data) => {
                                                    *data = new_data;
                                                    match packed_file.encode_and_clean_cache() {
                                                        Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                                                        Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                                    }
                                                }
                                                Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                            }
                                        }
                                        else {
                                            unimplemented!()
                                        }
                                    },
                                    Err(error) =>  CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                                }
                            },

                            _ => {
                                match File::open(external_path) {
                                    Ok(mut file) => {
                                        let mut data = vec![];
                                        match file.read_to_end(&mut data) {
                                            Ok(_) => {
                                                packed_file.set_raw_data(&data);
                                                CENTRAL_COMMAND.send_message_rust(Response::Success);
                                            }
                                            Err(_) => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::IOGeneric.into())),
                                        }
                                    }
                                    Err(_) => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::IOGeneric.into())),
                                }
                            }
                        }
                    }
                    None => CENTRAL_COMMAND.send_message_rust(Response::Error(ErrorKind::PackedFileNotFound.into())),
                }
            }

            // When we want to unpack an AnimPack...
            Command::AnimPackUnpack(path) => {
                let data = match pack_file_decoded.get_ref_mut_packed_file_by_path(&path) {
                    Some(ref mut packed_file) => {
                        match packed_file.decode_return_ref() {
                            Ok(packed_file_data) => {
                                match packed_file_data {
                                    DecodedPackedFile::AnimPack(data) => data.clone(),
                                    _ => { CENTRAL_COMMAND.send_message_rust(Response::Unknown); continue },
                                }
                            }
                            Err(error) => { CENTRAL_COMMAND.send_message_rust(Response::Error(error)); continue },
                        }
                    }
                    None => { CENTRAL_COMMAND.send_message_rust(Response::Error(Error::from(ErrorKind::PackedFileNotFound))); continue },
                };

                match data.unpack(&mut pack_file_decoded) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::VecVecString(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // When we want to generate a dummy AnimPack...
            Command::GenerateDummyAnimPack => {
                let anim_pack = DecodedPackedFile::AnimPack(AnimPack::new());
                let packed_file = PackedFile::new_from_decoded(&anim_pack, &animpack::DEFAULT_PATH.iter().map(|x| x.to_string()).collect::<Vec<String>>());
                match pack_file_decoded.add_packed_file(&packed_file, true) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::VecString(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // When we want to apply a template over the open PackFile...
            Command::ApplyTemplate(mut template, params) => {
                match template.apply_template(&params, &mut pack_file_decoded) {
                    Ok(result) => CENTRAL_COMMAND.send_message_rust(Response::VecVecString(result)),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // When we want to update the templates..
            Command::UpdateTemplates => {
                match Template::update() {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // When we want to update our schemas...
            Command::UpdateSchemas => {
                match Schema::update_schema_repo() {
                    Ok(_) => CENTRAL_COMMAND.send_message_rust(Response::Success),
                    Err(error) => CENTRAL_COMMAND.send_message_rust(Response::Error(error)),
                }
            }

            // These two belong to the network thread, not to this one!!!!
            Command::CheckUpdates | Command::CheckSchemaUpdates => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
}
