// Here should go just the Background loop.

use std::env::temp_dir;
use std::sync::mpsc::{Sender, Receiver};
use std::path::PathBuf;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::process::Command;
use regex::Regex;

use crate::RPFM_PATH;
use crate::SUPPORTED_GAMES;
use crate::SHOW_TABLE_ERRORS;
use crate::SHORTCUTS;
use crate::SETTINGS;
use crate::SCHEMA;
use crate::DEPENDENCY_DATABASE;
use crate::GAME_SELECTED;
use crate::GlobalMatch;
use crate::background_thread_extra;
use crate::common::*;
use crate::common::coding_helpers::*;
use crate::common::communications::*;
use crate::error::{Error, ErrorKind};
use crate::packfile::{PackFile, PFHFlags};
use crate::packedfile::*;
use crate::packedfile::loc::*;
use crate::packedfile::db::*;
use crate::packedfile::db::schemas::*;
use crate::packedfile::rigidmodel::*;
use crate::updater::*;

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
/// The sender is to send stuff back (from Data enum) to the UI.
/// The receiver is to receive orders to execute from the loop.
/// The receiver_data is to receive data (whatever data is needed) inside a Data variant from the UI Thread.
pub fn background_loop(
    sender: &Sender<Data>,
    receiver: &Receiver<Commands>,
    receiver_data: &Receiver<Data>
) {

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

    // Start the main loop.
    loop {

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
        match receiver.recv() {

            // If you got a message...
            Ok(data) => {

                // Act depending on what that message is.
                match data {

                    // In case we want to reset the PackFile to his original state (dummy)...
                    Commands::ResetPackFile => {

                        // Create the new PackFile.
                        pack_file_decoded = PackFile::new();
                    }

                    // In case we want to reset the Secondary PackFile to his original state (dummy)...
                    Commands::ResetPackFileExtra => {

                        // Create the new PackFile.
                        pack_file_decoded_extra = PackFile::new();
                    }

                    // In case we want to create a "New PackFile"...
                    Commands::NewPackFile => {
                        let game_selected = GAME_SELECTED.lock().unwrap();
                        let pack_version = SUPPORTED_GAMES.get(&**game_selected).unwrap().id;
                        pack_file_decoded = background_thread_extra::new_packfile("unknown.pack".to_string(), pack_version);
                        *SCHEMA.lock().unwrap() = Schema::load(&SUPPORTED_GAMES.get(&**game_selected).unwrap().schema).ok();
                        sender.send(Data::U32(pack_file_decoded.pfh_file_type.get_value())).unwrap();
                    }

                    // In case we want to "Open a PackFile"...
                    Commands::OpenPackFile => {
                        let path: PathBuf = if let Data::PathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match background_thread_extra::open_packfile(path, SETTINGS.lock().unwrap().settings_bool["use_lazy_loading"]) {
                            Ok(pack_file) => {
                                pack_file_decoded = pack_file;
                                sender.send(Data::PackFileUIData(pack_file_decoded.create_ui_data())).unwrap();
                            }
                            Err(error) => sender.send(Data::Error(Error::from(ErrorKind::OpenPackFileGeneric(format!("{}", error))))).unwrap(),
                        }
                    }

                    // In case we want to "Open an Extra PackFile" (for "Add from PackFile")...
                    Commands::OpenPackFileExtra => {

                        // Get the path to open.
                        let path: PathBuf = if let Data::PathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Open the PackFile as Read-Only (Or die trying it).
                        match background_thread_extra::open_packfile(path, true) {

                            // If we managed to open it...
                            Ok(result) => {
                                pack_file_decoded_extra = result;
                                sender.send(Data::Success).unwrap();
                            }

                            // If there is an error, send it back to the UI.
                            Err(error) => sender.send(Data::Error(Error::from(ErrorKind::OpenPackFileGeneric(format!("{}", error))))).unwrap(),
                        }
                    }

                    // In case we want to "Save a PackFile"...
                    Commands::SavePackFile => {

                        // If it passed all the checks, then try to save it and return the result.
                        match background_thread_extra::save_packfile(&mut pack_file_decoded, None, SETTINGS.lock().unwrap().settings_bool["allow_editing_of_ca_packfiles"]) {
                            Ok(_) => sender.send(Data::I64(pack_file_decoded.timestamp)).unwrap(),
                            Err(error) => {
                                match error.kind() {
                                    ErrorKind::PackFileIsNotAFile => sender.send(Data::Error(error)).unwrap(),
                                    _ => sender.send(Data::Error(Error::from(ErrorKind::SavePackFileGeneric(format!("{}", error))))).unwrap(),
                                }
                            }
                        }
                    }

                    // In case we want to "Save a PackFile As"...
                    Commands::SavePackFileAs => {

                        // If it's editable, we send the UI the "Extra data" of the PackFile, as the UI needs it for some stuff.
                        sender.send(Data::PathBuf(pack_file_decoded.file_path.to_path_buf())).unwrap();

                        // Wait until we get the new path for the PackFile.
                        let path = match check_message_validity_recv(&receiver_data) {
                            Data::PathBuf(data) => data,
                            Data::Cancel => continue,
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        };

                        // Try to save the PackFile and return the results.
                        match background_thread_extra::save_packfile(&mut pack_file_decoded, Some(path.to_path_buf()), SETTINGS.lock().unwrap().settings_bool["allow_editing_of_ca_packfiles"]) {
                            Ok(_) => sender.send(Data::I64(pack_file_decoded.timestamp)).unwrap(),
                            Err(error) => sender.send(Data::Error(Error::from(ErrorKind::SavePackFileGeneric(format!("{}", error))))).unwrap(),
                        }
                    }

                    // In case we want to "Load All CA PackFiles"...
                    Commands::LoadAllCAPackFiles => {
                        match background_thread_extra::load_all_ca_packfiles() {

                            // If we succeed at opening the PackFile...
                            Ok(pack_file) => {
                                pack_file_decoded = pack_file;
                                sender.send(Data::PackFileUIData(pack_file_decoded.create_ui_data())).unwrap();
                            }

                            // If there is an error, send it back to the UI.
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to change the PackFile's Type...
                    Commands::SetPackFileType => {

                        // Wait until we get the needed data from the UI thread.
                        let new_type = if let Data::PFHFileType(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Change the type of the PackFile.
                        pack_file_decoded.pfh_file_type = new_type;
                    }

                    // In case we want to change the "Include Last Modified Date" setting of the PackFile...
                    Commands::ChangeIndexIncludesTimestamp => {

                        // Wait until we get the needed data from the UI thread.
                        let state: bool = if let Data::Bool(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // If it can be deserialized as a bool, change the state of the "Include Last Modified Date" setting of the PackFile.
                        pack_file_decoded.bitmask.set(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, state);
                    }

                    // In case we want to save an schema...
                    Commands::SaveSchema => {

                        // Wait until we get the needed data from the UI thread.
                        let new_schema: Schema = if let Data::Schema(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match Schema::save(&new_schema, &SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().schema) {
                            Ok(_) => {
                                *SCHEMA.lock().unwrap() = Some(new_schema);
                                sender.send(Data::Success).unwrap();
                            },
                            Err(error) => sender.send(Data::Error(error)).unwrap()
                        }
                    }

                    // In case we want to change the current settings...
                    Commands::SetSettings => {
                        let new_settings = if let Data::Settings(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        *SETTINGS.lock().unwrap() = new_settings;
                        match SETTINGS.lock().unwrap().save() {
                            Ok(()) => sender.send(Data::Success).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to change the current shortcuts...
                    Commands::SetShortcuts => {

                        // Wait until we get the needed data from the UI thread, then save our Shortcuts to a shortcuts file, and report in case of error.
                        let new_shortcuts = if let Data::Shortcuts(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        loop { if let Ok(ref mut shortcuts) = SHORTCUTS.try_lock() { 
                            **shortcuts = new_shortcuts;
                            match shortcuts.save() {
                                Ok(()) => sender.send(Data::Success).unwrap(),
                                Err(error) => sender.send(Data::Error(error)).unwrap(),
                            }
                            break;
                        }};
                    }

                    // In case we want to change the current Game Selected...
                    Commands::SetGameSelected => {

                        // Wait until we get the needed data from the UI thread and change the GameSelected.
                        let game_selected = if let Data::String(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        *GAME_SELECTED.lock().unwrap() = game_selected.to_owned();

                        // Send back the new Game Selected, and a bool indicating if there is a PackFile open.
                        sender.send(Data::Bool(!pack_file_decoded.get_file_name().is_empty())).unwrap();

                        // Try to load the Schema for this game.
                        *SCHEMA.lock().unwrap() = Schema::load(&SUPPORTED_GAMES.get(&*game_selected).unwrap().schema).ok();

                        // Change the `dependency_database` for that game.
                        *DEPENDENCY_DATABASE.lock().unwrap() = background_thread_extra::load_dependency_packfiles(&pack_file_decoded.pack_files);

                        // If there is a PackFile open, change his id to match the one of the new GameSelected.
                        if !pack_file_decoded.get_file_name().is_empty() { pack_file_decoded.pfh_version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().id; }

                        // Test to see if every DB Table can be decoded. This is slow and only useful when
                        // a new patch lands and you want to know what tables you need to decode. So, unless you want 
                        // to decode new tables, leave the const as false
                        if SHOW_TABLE_ERRORS {
                            let mut counter = 0;
                            for i in pack_file_decoded.packed_files.iter_mut() {
                                if i.path.starts_with(&["db".to_owned()]) {
                                    if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                                        if let Err(error) = db::DB::read(&(i.get_data_and_keep_it().unwrap()), &i.path[1], &schema) {
                                            if error.kind() != ErrorKind::DBTableContainsListField {
                                                match db::DB::get_header_data(&i.get_data().unwrap()) {
                                                    Ok((_, entry_count, _)) => {
                                                        if entry_count > 0 {
                                                            counter += 1;
                                                            println!("{}, {:?}", counter, i.path);
                                                        }
                                                    }
                                                    Err(_) => println!("Error in {:?}", i.path),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // In case we want to check if there is a current Dependency Database loaded...
                    Commands::IsThereADependencyDatabase => {
                        if !DEPENDENCY_DATABASE.lock().unwrap().is_empty() { sender.send(Data::Bool(true)).unwrap(); }
                        else { sender.send(Data::Bool(false)).unwrap(); }
                    }

                    // In case we want to check if there is an Schema loaded...
                    Commands::IsThereASchema => {
                        match *SCHEMA.lock().unwrap() {
                            Some(_) => sender.send(Data::Bool(true)).unwrap(),
                            None => sender.send(Data::Bool(false)).unwrap(),
                        }
                    }

                    // In case we want to Patch the SiegeAI of a PackFile...
                    Commands::PatchSiegeAI => {

                        // First, we try to patch the PackFile.
                        match background_thread_extra::patch_siege_ai(&mut pack_file_decoded) {

                            // If we succeed, send back the result.
                            Ok(result) => sender.send(Data::StringVecTreePathType(result)).unwrap(),

                            // Otherwise, return an error.
                            Err(error) => sender.send(Data::Error(error)).unwrap()
                        }
                    }

                    // In case we want to update our Schemas...
                    Commands::UpdateSchemas => {

                        // Reload the currently loaded schema, just in case it was updated.
                        let data = if let Data::VersionsVersions(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match update_schemas(&data.0, &data.1) {
                            Ok(_) => {
                                *SCHEMA.lock().unwrap() = Schema::load(&SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().schema).ok();
                                sender.send(Data::Success).unwrap();
                            }
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to add PackedFiles into a PackFile...
                    Commands::AddPackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::VecPathBufVecVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // For each file...
                        for index in 0..data.0.len() {

                            // Try to add it to the PackFile. If it fails, report it and stop adding files.
                            if let Err(error) = background_thread_extra::add_file_to_packfile(&mut pack_file_decoded, &data.0[index], data.1[index].to_vec()) {
                                sender.send(Data::Error(error)).unwrap();
                                break;
                            }
                        }

                        // If nothing failed, send back success.
                        sender.send(Data::Success).unwrap();
                    }

                    // In case we want to delete PackedFiles from a PackFile...
                    Commands::DeletePackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Get the type of the Path we want to delete.
                        let path_type = get_type_of_selected_path(&path, &pack_file_decoded);

                        // Delete the PackedFiles from the PackFile, changing his return in case of success.
                        match background_thread_extra::delete_from_packfile(&mut pack_file_decoded, &path) {
                            Ok(_) => sender.send(Data::TreePathType(path_type)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to extract PackedFiles from a PackFile...
                    Commands::ExtractPackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::VecStringPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Try to extract the PackFile.
                        match background_thread_extra::extract_from_packfile(
                            &pack_file_decoded,
                            &data.0,
                            &data.1
                        ) {
                            Ok(result) => sender.send(Data::String(result)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to get the type of an item in the TreeView, from his path...
                    Commands::GetTypeOfPath => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Get the type of the selected item.
                        let path_type = get_type_of_selected_path(&path, &pack_file_decoded);

                        // Send the type back.
                        sender.send(Data::TreePathType(path_type)).unwrap();
                    }

                    // In case we want to know if a PackedFile exists, knowing his path...
                    Commands::PackedFileExists => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Check if the path exists as a PackedFile.
                        let exists = pack_file_decoded.packedfile_exists(&path);

                        // Send the result back.
                        sender.send(Data::Bool(exists)).unwrap();
                    }

                    // In case we want to know if a Folder exists, knowing his path...
                    Commands::FolderExists => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Check if the path exists as a folder.
                        let exists = pack_file_decoded.folder_exists(&path);

                        // Send the result back.
                        sender.send(Data::Bool(exists)).unwrap();
                    }

                    // In case we want to create a PackedFile from scratch...
                    Commands::CreatePackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::VecStringPackedFileType(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Create the PackedFile.
                        match create_packed_file(
                            &mut pack_file_decoded,
                            data.1,
                            data.0,
                        ) {
                            // Send the result back.
                            Ok(_) => sender.send(Data::Success).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // TODO: Move checkings here, from the UI.
                    // In case we want to create an empty folder...
                    Commands::CreateFolder => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Add the path to the "empty folder" list.
                        pack_file_decoded.empty_folders.push(path);
                    }

                    // In case we want to update the empty folder list...
                    Commands::UpdateEmptyFolders => {

                        // Update the empty folder list, if needed.
                        pack_file_decoded.update_empty_folders();
                    }

                    // In case we want to get the data of a PackFile needed to form the TreeView...
                    Commands::GetPackFileDataForTreeView => {

                        // Get the name and the PackedFile list, and send it.
                        sender.send(Data::StringI64VecVecString((
                            pack_file_decoded.get_file_name(), 
                            pack_file_decoded.timestamp,
                            pack_file_decoded.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                        ))).unwrap();
                    }

                    // In case we want to get the data of a Secondary PackFile needed to form the TreeView...
                    Commands::GetPackFileExtraDataForTreeView => {

                        // Get the name and the PackedFile list, and serialize it.
                        sender.send(Data::StringI64VecVecString((
                            pack_file_decoded_extra.get_file_name(), 
                            pack_file_decoded_extra.timestamp,
                            pack_file_decoded_extra.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                        ))).unwrap();
                    }

                    // In case we want to move stuff from one PackFile to another...
                    Commands::AddPackedFileFromPackFile => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Try to add the PackedFile to the main PackFile.
                        match background_thread_extra::add_packedfile_to_packfile(
                            &pack_file_decoded_extra,
                            &mut pack_file_decoded,
                            &path
                        ) {

                            // In case of success, get the list of copied PackedFiles and send it back.
                            Ok(_) => {

                                // Get the "real" path, without the PackFile on it. If the path is just the PackFile, leave it empty.
                                let real_path = if path.len() > 1 { &path[1..] } else { &[] };

                                // Get all the PackedFiles to copy.
                                let path_list: Vec<Vec<String>> = pack_file_decoded_extra
                                    .packed_files
                                    .iter()
                                    .filter(|x| x.path.starts_with(&real_path))
                                    .map(|x| x.path.to_vec())
                                    .collect();

                                // Send all of it back.
                                sender.send(Data::VecVecString(path_list)).unwrap();
                            }

                            // In case of error, report it.
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to Mass-Import TSV Files...
                    Commands::MassImportTSV => {

                        // Try to import all the importable files from the provided path.
                        let data = if let Data::StringVecPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match tsv_mass_import(&data.1, &data.0, &mut pack_file_decoded) {
                            Ok(result) => sender.send(Data::VecVecStringVecVecString(result)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to Mass-Export TSV Files...
                    Commands::MassExportTSV => {

                        // Try to export all the exportable files to the provided path.
                        let path = if let Data::PathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match tsv_mass_export(&path, &mut pack_file_decoded) {
                            Ok(result) => sender.send(Data::String(result)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to decode a Loc PackedFile...
                    Commands::DecodePackedFileLoc => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => {

                                // We try to decode it as a Loc PackedFile.
                                match packed_file.get_data_and_keep_it() {
                                    Ok(data) => {
                                        match Loc::read(&data) {
                                            Ok(packed_file_decoded) => sender.send(Data::Loc(packed_file_decoded)).unwrap(),
                                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                                        }
                                    }
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to encode a Loc PackedFile...
                    Commands::EncodePackedFileLoc => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::LocVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Update the PackFile to reflect the changes.
                        background_thread_extra::update_packed_file_data_loc(
                            &data.0,
                            &mut pack_file_decoded,
                            &data.1
                        );
                    }

                    // In case we want to import a TSV file into a Loc PackedFile...
                    Commands::ImportTSVPackedFileLoc => {

                        // Try to import the TSV into the open Loc PackedFile from the provided path, or die trying.
                        let mut data = if let Data::LocPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match data.0.import_tsv(&data.1, "") {
                            Ok(_) => sender.send(Data::Loc(data.0)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to export a Loc PackedFile into a TSV file...
                    Commands::ExportTSVPackedFileLoc => {

                        // Try to export the TSV from the open Loc PackedFile to the provided path, or die trying.
                        let data = if let Data::LocPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match data.0.export_tsv(&data.1, ("", 0)) {
                            Ok(_) => sender.send(Data::Success).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to decode a DB PackedFile...
                    Commands::DecodePackedFileDB => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Depending if there is an Schema for this game or not...
                        match *SCHEMA.lock().unwrap() {
                            Some(ref schema) => {
                                match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                                    Some(packed_file) => {

                                        // We try to decode it as a DB PackedFile.
                                        match packed_file.get_data_and_keep_it() {
                                            Ok(data) => {
                                                match DB::read(
                                                    &data,
                                                    &packed_file.path[1],
                                                    schema,
                                                ) {
                                                    Ok(packed_file_decoded) => sender.send(Data::DB(packed_file_decoded)).unwrap(),
                                                    Err(error) => sender.send(Data::Error(error)).unwrap(),
                                                }
                                            }
                                            Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                        }
                                    }
                                    None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                                }
                            }

                            // If there is no schema, return an error.
                            None => sender.send(Data::Error(Error::from(ErrorKind::SchemaNotFound))).unwrap(),
                        }
                    }

                    // In case we want to encode a DB PackedFile...
                    Commands::EncodePackedFileDB => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::DBVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Update the PackFile to reflect the changes.
                        background_thread_extra::update_packed_file_data_db(
                            &data.0,
                            &mut pack_file_decoded,
                            &data.1
                        );
                    }

                    // In case we want to import a TSV file into a DB PackedFile...
                    Commands::ImportTSVPackedFileDB => {

                        // Try to import the TSV into the open DB Table from the provided path, or die trying.
                        let mut data = if let Data::DBPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        let db_type = data.0.db_type.to_owned();
                        match data.0.import_tsv(&data.1, &db_type) {
                            Ok(_) => sender.send(Data::DB(data.0)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to export a DB PackedFile into a TSV file...
                    Commands::ExportTSVPackedFileDB => {

                        // Try to export the TSV into the open DB PackedFile to the provided path, or die trying.
                        let data = if let Data::DBPathBuf(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        match data.0.export_tsv(&data.1, (&data.0.db_type, data.0.version)) {
                            Ok(_) => sender.send(Data::Success).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to decode a Plain Text PackedFile...
                    Commands::DecodePackedFileText => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => {
                                match packed_file.get_data_and_keep_it() {
                                    Ok(data) => {
                                        
                                        // NOTE: This only works for UTF-8 and ISO_8859_1 encoded files. Check their encoding before adding them here to be decoded.
                                        // Try to decode the PackedFile as a normal UTF-8 string.
                                        let mut decoded_string = decode_string_u8(&data);

                                        // If there is an error, try again as ISO_8859_1, as there are some text files using that encoding.
                                        if decoded_string.is_err() {
                                            if let Ok(string) = decode_string_u8_iso_8859_1(&data) {
                                                decoded_string = Ok(string);
                                            }
                                        }

                                        // Depending if the decoding worked or not, send back the text file or an error.
                                        match decoded_string {
                                            Ok(text) => sender.send(Data::String(text)).unwrap(),
                                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                                        }
                                    }
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to encode a Text PackedFile...
                    Commands::EncodePackedFileText => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::StringVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Encode the text.
                        let encoded_text = encode_string_u8(&data.0);

                        // Update the PackFile to reflect the changes.
                        background_thread_extra::update_packed_file_data_text(
                            &encoded_text,
                            &mut pack_file_decoded,
                            &data.1
                        );
                    }

                    // In case we want to decode a RigidModel...
                    Commands::DecodePackedFileRigidModel => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => {
                                match packed_file.get_data_and_keep_it() {
                                    Ok(data) => {
                                        
                                        // We try to decode it as a RigidModel.
                                        match RigidModel::read(&data) {

                                            // If we succeed, store it and send it back.
                                            Ok(packed_file_decoded) => sender.send(Data::RigidModel(packed_file_decoded)).unwrap(),
                                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                                        }
                                    }
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to encode a RigidModel...
                    Commands::EncodePackedFileRigidModel => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::RigidModelVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Update the PackFile to reflect the changes.
                        background_thread_extra::update_packed_file_data_rigid(
                            &data.0,
                            &mut pack_file_decoded,
                            &data.1
                        );
                    }

                    // In case we want to patch a decoded RigidModel from Attila to Warhammer...
                    Commands::PatchAttilaRigidModelToWarhammer => {

                        // Wait until we get the needed data from the UI thread.
                        let mut data = if let Data::RigidModelVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter().find(|x| x.path == data.1) {
                            Some(_) => {

                                // We try to patch the RigidModel.
                                match background_thread_extra::patch_rigid_model_attila_to_warhammer(&mut data.0) {

                                    // If we succeed...
                                    Ok(_) => {

                                        // Update the PackFile to reflect the changes.
                                        background_thread_extra::update_packed_file_data_rigid(
                                            &data.0,
                                            &mut pack_file_decoded,
                                            &data.1
                                        );

                                        // Send back the patched PackedFile.
                                        sender.send(Data::RigidModel(data.0)).unwrap()
                                    }

                                    // In case of error, report it.
                                    Err(error) => sender.send(Data::Error(error)).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to decode an Image...
                    Commands::DecodePackedFileImage => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => {
                                match packed_file.get_data_and_keep_it() {
                                    Ok(image_data) => {
                                        
                                        let image_name = &packed_file.path.last().unwrap().to_owned();

                                        // Create a temporal file for the image in the TEMP directory of the filesystem.
                                        let mut temporal_file_path = temp_dir();
                                        temporal_file_path.push(image_name);
                                        match File::create(&temporal_file_path) {
                                            Ok(mut temporal_file) => {

                                                // If there is an error while trying to write the image to the TEMP folder, report it.
                                                if temporal_file.write_all(&image_data).is_err() {
                                                    sender.send(Data::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1])))).unwrap();
                                                }

                                                // If it worked, create an Image with the new file and show it inside a ScrolledWindow.
                                                else { sender.send(Data::PathBuf(temporal_file_path)).unwrap(); }
                                            }

                                            // If there is an error when trying to create the file into the TEMP folder, report it.
                                            Err(_) => sender.send(Data::Error(Error::from(ErrorKind::IOGenericWrite(vec![temporal_file_path.display().to_string();1])))).unwrap(),
                                        }
                                    }
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to "Rename a PackedFile"...
                    Commands::RenamePackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::VecStringString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };

                        // Try to rename it and report the result.
                        match background_thread_extra::rename_packed_file(&mut pack_file_decoded, &data.0, &data.1) {
                            Ok(_) => sender.send(Data::Success).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to "Rename multiple PackedFiles" at once...
                    Commands::ApplyPrefixToPackedFilesInPath => {

                        // Wait until we get the needed data from the UI thread.
                        let data = if let Data::VecStringString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };

                        // Try to rename it and report the result.
                        match background_thread_extra::apply_prefix_to_packed_files(&mut pack_file_decoded, &data.0, &data.1) {
                            Ok(result) => sender.send(Data::VecVecString(result)).unwrap(),
                            Err(error) => sender.send(Data::Error(error)).unwrap(),
                        }
                    }

                    // In case we want to get a PackedFile's data...
                    Commands::GetPackedFile => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };

                        // Find the PackedFile we want and send back the response.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => { 
                                match packed_file.load_data() {
                                    Ok(_) => sender.send(Data::PackedFile(packed_file.clone())).unwrap(),
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }

                    // In case we want to get the list of tables in the dependency database...
                    Commands::GetTableListFromDependencyPackFile => {

                        let tables = DEPENDENCY_DATABASE.lock().unwrap().iter().filter(|x| x.path.len() > 2).filter(|x| x.path[1].ends_with("_tables")).map(|x| x.path[1].to_owned()).collect::<Vec<String>>();
                        sender.send(Data::VecString(tables)).unwrap();
                    }

                    // In case we want to get the version of an specific table from the dependency database...
                    Commands::GetTableVersionFromDependencyPackFile => {

                        // Wait until we get the needed data from the UI thread.
                        let table_name = if let Data::String(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };
                        if let Some(vanilla_table) = DEPENDENCY_DATABASE.lock().unwrap().iter_mut().filter(|x| x.path.len() == 3).find(|x| x.path[1] == table_name) {
                            match DB::get_header_data(&vanilla_table.get_data_and_keep_it().unwrap()) {
                                Ok(data) => sender.send(Data::U32(data.0)).unwrap(),
                                Err(error) => sender.send(Data::Error(error)).unwrap(),
                            }
                        }

                        // If our table is not in the dependencies, we fall back to use the version in the schema.
                        else if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                            if let Some(definition) = schema.tables_definitions.iter().find(|x| x.name == table_name) {
                                let mut versions = definition.versions.to_vec();
                                versions.sort_unstable_by(|x, y| x.version.cmp(&y.version));
                                sender.send(Data::U32(versions.last().unwrap().version)).unwrap();
                            }

                            // If there is no table in the schema, we return an error.
                            else { sender.send(Data::Error(Error::from(ErrorKind::SchemaTableDefinitionNotFound))).unwrap(); }

                        }

                        // If there is no schema, we return an error.
                        else { sender.send(Data::Error(Error::from(ErrorKind::SchemaNotFound))).unwrap(); }
                    }

                    // In case we want to optimize our PackFile...
                    Commands::OptimizePackFile => {
                        match background_thread_extra::optimize_packfile(&mut pack_file_decoded) {
                            Ok(deleted_packed_files) => sender.send(Data::VecTreePathType(deleted_packed_files)).unwrap(),
                            Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                        }
                    }

                    // In case we want to get the PackFiles List of our PackFile...
                    Commands::GetPackFilesList => {
                        sender.send(Data::VecString(pack_file_decoded.pack_files.to_vec())).unwrap();
                    }

                    // In case we want to save the PackFiles List of our PackFile...
                    Commands::SetPackFilesList => {
                       
                        // Wait until we get the needed data from the UI thread.
                        let list = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };
                        pack_file_decoded.save_packfiles_list(list);

                        // Update the dependency database.
                        *DEPENDENCY_DATABASE.lock().unwrap() = background_thread_extra::load_dependency_packfiles(&pack_file_decoded.pack_files);
                    }

                    // In case we want to get the dependency data for a table's column....
                    Commands::DecodeDependencyDB => {

                        // Wait until we get the needed data from the UI thread.
                        let dependency_data = if let Data::StringString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };
                        let mut data = vec![];
                        let mut iter = DEPENDENCY_DATABASE.lock().unwrap();
                        let mut iter = iter.iter_mut();
                        if !dependency_data.0.is_empty() && !dependency_data.1.is_empty() {
                            while let Some(packed_file) = iter.find(|x| x.path.starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                                if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                                    if let Ok(table) = DB::read(&packed_file.get_data_and_keep_it().unwrap(), &format!("{}_tables", dependency_data.0), &schema) {
                                        if let Some(column_index) = table.table_definition.fields.iter().position(|x| x.field_name == dependency_data.1) {
                                            for row in table.entries.iter() {

                                                // For now we assume any dependency is a string.
                                                match row[column_index] { 
                                                    DecodedData::StringU8(ref entry) |
                                                    DecodedData::StringU16(ref entry) |
                                                    DecodedData::OptionalStringU8(ref entry) |
                                                    DecodedData::OptionalStringU16(ref entry) => data.push(entry.to_owned()),
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // The same for our own PackFile.
                        let mut iter = pack_file_decoded.packed_files.iter_mut();
                        if !dependency_data.0.is_empty() && !dependency_data.1.is_empty() {
                            while let Some(packed_file) = iter.find(|x| x.path.starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                                if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                                    if let Ok(packed_file_data) = packed_file.get_data_and_keep_it() {
                                        if let Ok(table) = DB::read(&packed_file_data, &format!("{}_tables", dependency_data.0), &schema) {
                                            if let Some(column_index) = table.table_definition.fields.iter().position(|x| x.field_name == dependency_data.1) {
                                                for row in table.entries.iter() {

                                                    // For now we assume any dependency is a string.
                                                    match row[column_index] { 
                                                        DecodedData::StringU8(ref entry) |
                                                        DecodedData::StringU16(ref entry) |
                                                        DecodedData::OptionalStringU8(ref entry) |
                                                        DecodedData::OptionalStringU16(ref entry) => data.push(entry.to_owned()),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Sort and dedup the data found.
                        data.sort_unstable_by(|a, b| a.cmp(&b));
                        data.dedup();

                        sender.send(Data::VecString(data)).unwrap();
                    }

                    // In case we want to use Kailua to check if your script has errors...
                    Commands::CheckScriptWithKailua => {

                        // This is for storing the results we have to send back.
                        let mut results = vec![];

                        // Get the paths we need.
                        if let Some(ref ca_types_file) = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().ca_types_file {
                            let types_path = RPFM_PATH.to_path_buf().join(PathBuf::from("lua_types")).join(PathBuf::from(ca_types_file));
                            let temp_folder_path = temp_dir().join(PathBuf::from("rpfm/scripts"));
                            let mut config_path = temp_folder_path.to_path_buf();
                            config_path.push("kailua.json");
                            if Command::new("kailua").output().is_ok() {

                                let mut error = false;

                                // Extract every lua file in the PackFile, respecting his path.
                                for packed_file in &mut pack_file_decoded.packed_files {
                                    if packed_file.path.last().unwrap().ends_with(".lua") {
                                        let path: PathBuf = temp_folder_path.to_path_buf().join(packed_file.path.iter().collect::<PathBuf>());

                                        // If the path doesn't exist, create it.
                                        let mut path_base = path.to_path_buf();
                                        path_base.pop();
                                        if !path_base.is_dir() { DirBuilder::new().recursive(true).create(&path_base).unwrap(); }

                                        match packed_file.get_data_and_keep_it() {
                                            Ok(data) => {
                                                File::create(&path).unwrap().write_all(&data).unwrap();
                                                
                                                // Create the Kailua config file.
                                                let config = format!("
                                                {{
                                                    \"start_path\": [\"{}\"],
                                                    \"preload\": {{
                                                        \"open\": [\"lua51\"],
                                                        \"require\": [\"{}\"]
                                                    }}
                                                }}", path.to_string_lossy().replace('\\', "\\\\"), types_path.to_string_lossy().replace('\\', "\\\\"));
                                                File::create(&config_path).unwrap().write_all(&config.as_bytes()).unwrap();
                                                results.push(String::from_utf8_lossy(&Command::new("kailua").arg("check").arg(&config_path.to_string_lossy().as_ref().to_owned()).output().unwrap().stderr).to_string());
                                            }
                                            Err(_) => {
                                                sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap();
                                                error = true;
                                                break;
                                            }
                                        }
                                    }
                                }
    
                                // Send back the result.
                                if !error { sender.send(Data::VecString(results)).unwrap(); }
                            }

                            else { sender.send(Data::Error(Error::from(ErrorKind::KailuaNotFound))).unwrap(); }
                        }

                        // If there is no Type's file, return an error.
                        else { sender.send(Data::Error(Error::from(ErrorKind::NoTypesFileFound))).unwrap(); }
                    }

                    // In case we want to perform a "Global Search"...
                    Commands::GlobalSearch => {
                       
                        // Wait until we get the needed data from the UI thread.
                        let pattern = if let Data::String(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };
                        let regex = Regex::new(&pattern);
                        let mut matches: Vec<GlobalMatch> = vec![];
                        let mut error = false;
                        for packed_file in &mut pack_file_decoded.packed_files {
                            let path = packed_file.path.to_vec();
                            let packedfile_name = path.last().unwrap().to_owned();
                            let packed_file_type: &str =

                                // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
                                if path[0] == "db" { "DB" }

                                // If it ends in ".loc", it's a localisation PackedFile.
                                else if packedfile_name.ends_with(".loc") { "LOC" }

                                // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
                                else if packedfile_name.ends_with(".rigid_model_v2") { "RIGIDMODEL" }

                                // If it ends in any of these, it's a plain text PackedFile.
                                else if packedfile_name.ends_with(".lua") ||
                                        packedfile_name.ends_with(".xml") ||
                                        packedfile_name.ends_with(".xml.shader") ||
                                        packedfile_name.ends_with(".xml.material") ||
                                        packedfile_name.ends_with(".variantmeshdefinition") ||
                                        packedfile_name.ends_with(".environment") ||
                                        packedfile_name.ends_with(".lighting") ||
                                        packedfile_name.ends_with(".wsmodel") ||
                                        packedfile_name.ends_with(".csv") ||
                                        packedfile_name.ends_with(".tsv") ||
                                        packedfile_name.ends_with(".inl") ||
                                        packedfile_name.ends_with(".battle_speech_camera") ||
                                        packedfile_name.ends_with(".bob") ||
                                        packedfile_name.ends_with(".cindyscene") ||
                                        packedfile_name.ends_with(".cindyscenemanager") ||
                                        //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                                        packedfile_name.ends_with(".txt") { "TEXT" }

                                // If it ends in any of these, it's an image.
                                else if packedfile_name.ends_with(".jpg") ||
                                        packedfile_name.ends_with(".jpeg") ||
                                        packedfile_name.ends_with(".tga") ||
                                        packedfile_name.ends_with(".dds") ||
                                        packedfile_name.ends_with(".png") { "IMAGE" }

                                // Otherwise, we don't have a decoder for that PackedFile... yet.
                                else { "None" };

                            // Then, depending of his type we decode it properly (if we have it implemented support
                            // for his type).
                            match packed_file_type {

                                // If the file is a Loc PackedFile, decode it and search in his key and text columns.
                                "LOC" => {

                                    let data = match packed_file.get_data_and_keep_it() {
                                        Ok(data) => data,
                                        Err(_) => {
                                            sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap();
                                            error = true;
                                            break;
                                        }
                                    };

                                    // We try to decode it as a Loc PackedFile.
                                    if let Ok(packed_file) = Loc::read(&data) {

                                        let mut matches_in_file = vec![];
                                        for (index, row) in packed_file.entries.iter().enumerate() {
                                            if let Ok(ref regex) = regex {
                                                if regex.is_match(&row.key) { matches_in_file.push((0, index as i64, row.key.to_owned())); }
                                                if regex.is_match(&row.text) { matches_in_file.push((1, index as i64, row.text.to_owned())); }
                                            }
                                            else {
                                                if row.key.contains(&pattern) { matches_in_file.push((0, index as i64, row.key.to_owned())); }
                                                if row.text.contains(&pattern) { matches_in_file.push((1, index as i64, row.text.to_owned())); }
                                            }
                                        }

                                        if !matches_in_file.is_empty() { matches.push(GlobalMatch::Loc((path.to_vec(), matches_in_file))); }
                                    }
                                }

                                // If the file is a DB PackedFile...
                                "DB" => {

                                    let data = match packed_file.get_data_and_keep_it() {
                                        Ok(data) => data,
                                        Err(_) => {
                                            sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap();
                                            error = true;
                                            break;
                                        }
                                    };

                                    if let Some(ref schema) = *SCHEMA.lock().unwrap() {   
                                        if let Ok(packed_file) = DB::read(&data, &path[1], &schema) {

                                            let mut matches_in_file = vec![];
                                            for (index, row) in packed_file.entries.iter().enumerate() {
                                                for (column, field) in packed_file.table_definition.fields.iter().enumerate() {
                                                    match row[column] {

                                                        // All these are Strings, so it can be together,
                                                        DecodedData::StringU8(ref data) |
                                                        DecodedData::StringU16(ref data) |
                                                        DecodedData::OptionalStringU8(ref data) |
                                                        DecodedData::OptionalStringU16(ref data) => 

                                                            if let Ok(ref regex) = regex {
                                                                if regex.is_match(&data) {
                                                                    matches_in_file.push((field.field_name.to_owned(), column as i32, index as i64, data.to_owned())); 
                                                                }
                                                            }
                                                            else if data.contains(&pattern) {
                                                                matches_in_file.push((field.field_name.to_owned(), column as i32, index as i64, data.to_owned())); 
                                                            }

                                                        _ => continue
                                                    }
                                                }
                                            }

                                            if !matches_in_file.is_empty() { matches.push(GlobalMatch::DB((path.to_vec(), matches_in_file))); }
                                        }
                                    }
                                }

                                // For any other PackedFile, skip it.
                                _ => continue,
                            }
                        }

                        // Send back the list of matches.
                        if !error { sender.send(Data::VecGlobalMatch(matches)).unwrap(); }
                    }

                    // In case we want to perform a "Global Search"...
                    Commands::UpdateGlobalSearchData => {
                       
                        // Wait until we get the needed data from the UI thread.
                        let (pattern, paths) = if let Data::StringVecVecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };
                        let regex = Regex::new(&pattern);
                        let mut matches: Vec<GlobalMatch> = vec![];
                        let mut error = false;
                        for packed_file in &mut pack_file_decoded.packed_files {

                            // We need to take into account that we may pass here incomplete paths.
                            let mut is_in_folder = false;
                            for path in &paths {
                                if !path.is_empty() && packed_file.path.starts_with(path) {
                                    is_in_folder = true;
                                    break;
                                }
                            }

                            if paths.contains(&packed_file.path) || is_in_folder {
                                let path = packed_file.path.to_vec();
                                let packedfile_name = path.last().unwrap().to_owned();
                                let packed_file_type: &str =

                                    // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
                                    if path[0] == "db" { "DB" }

                                    // If it ends in ".loc", it's a localisation PackedFile.
                                    else if packedfile_name.ends_with(".loc") { "LOC" }

                                    // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
                                    else if packedfile_name.ends_with(".rigid_model_v2") { "RIGIDMODEL" }

                                    // If it ends in any of these, it's a plain text PackedFile.
                                    else if packedfile_name.ends_with(".lua") ||
                                            packedfile_name.ends_with(".xml") ||
                                            packedfile_name.ends_with(".xml.shader") ||
                                            packedfile_name.ends_with(".xml.material") ||
                                            packedfile_name.ends_with(".variantmeshdefinition") ||
                                            packedfile_name.ends_with(".environment") ||
                                            packedfile_name.ends_with(".lighting") ||
                                            packedfile_name.ends_with(".wsmodel") ||
                                            packedfile_name.ends_with(".csv") ||
                                            packedfile_name.ends_with(".tsv") ||
                                            packedfile_name.ends_with(".inl") ||
                                            packedfile_name.ends_with(".battle_speech_camera") ||
                                            packedfile_name.ends_with(".bob") ||
                                            packedfile_name.ends_with(".cindyscene") ||
                                            packedfile_name.ends_with(".cindyscenemanager") ||
                                            //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                                            packedfile_name.ends_with(".txt") { "TEXT" }

                                    // If it ends in any of these, it's an image.
                                    else if packedfile_name.ends_with(".jpg") ||
                                            packedfile_name.ends_with(".jpeg") ||
                                            packedfile_name.ends_with(".tga") ||
                                            packedfile_name.ends_with(".dds") ||
                                            packedfile_name.ends_with(".png") { "IMAGE" }

                                    // Otherwise, we don't have a decoder for that PackedFile... yet.
                                    else { "None" };

                                // Then, depending of his type we decode it properly (if we have it implemented support
                                // for his type).
                                match packed_file_type {

                                    // If the file is a Loc PackedFile, decode it and search in his key and text columns.
                                    "LOC" => {

                                        let data = match packed_file.get_data_and_keep_it() {
                                            Ok(data) => data,
                                            Err(_) => {
                                                sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap();
                                                error = true;
                                                break;
                                            }
                                        };

                                        // We try to decode it as a Loc PackedFile.
                                        if let Ok(packed_file) = Loc::read(&data) {

                                            let mut matches_in_file = vec![];
                                            for (index, row) in packed_file.entries.iter().enumerate() {
                                                if let Ok(ref regex) = regex {
                                                    if regex.is_match(&row.key) { matches_in_file.push((0, index as i64, row.key.to_owned())); }
                                                    if regex.is_match(&row.text) { matches_in_file.push((1, index as i64, row.text.to_owned())); }
                                                }
                                                else {
                                                    if row.key.contains(&pattern) { matches_in_file.push((0, index as i64, row.key.to_owned())); }
                                                    if row.text.contains(&pattern) { matches_in_file.push((1, index as i64, row.text.to_owned())); }
                                                }
                                            }

                                            if !matches_in_file.is_empty() { matches.push(GlobalMatch::Loc((path.to_vec(), matches_in_file))); }
                                        }
                                    }

                                    // If the file is a DB PackedFile...
                                    "DB" => {

                                        let data = match packed_file.get_data_and_keep_it() {
                                            Ok(data) => data,
                                            Err(_) => {
                                                sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap();
                                                error = true;
                                                break;
                                            }
                                        };

                                        if let Some(ref schema) = *SCHEMA.lock().unwrap() {   
                                            if let Ok(packed_file) = DB::read(&data, &path[1], &schema) {

                                                let mut matches_in_file = vec![];
                                                for (index, row) in packed_file.entries.iter().enumerate() {
                                                    for (column, field) in packed_file.table_definition.fields.iter().enumerate() {
                                                        match row[column] {

                                                            // All these are Strings, so it can be together,
                                                            DecodedData::StringU8(ref data) |
                                                            DecodedData::StringU16(ref data) |
                                                            DecodedData::OptionalStringU8(ref data) |
                                                            DecodedData::OptionalStringU16(ref data) => 

                                                            if let Ok(ref regex) = regex {
                                                                if regex.is_match(&data) {
                                                                    matches_in_file.push((field.field_name.to_owned(), column as i32, index as i64, data.to_owned())); 
                                                                }
                                                            }
                                                            else if data.contains(&pattern) {
                                                                matches_in_file.push((field.field_name.to_owned(), column as i32, index as i64, data.to_owned()));     
                                                            }

                                                            _ => continue
                                                        }
                                                    }
                                                }

                                                if !matches_in_file.is_empty() { matches.push(GlobalMatch::DB((path.to_vec(), matches_in_file))); }
                                            }
                                        }
                                    }

                                    // For any other PackedFile, skip it.
                                    _ => continue,
                                }
                            }
                        }

                        // Send back the list of matches.
                        if !error { sender.send(Data::VecGlobalMatch(matches)).unwrap(); }
                    }

                    // In case we want to open a PackedFile with an external Program...
                    Commands::OpenWithExternalProgram => {

                        // Wait until we get the needed data from the UI thread.
                        let path = if let Data::VecString(data) = check_message_validity_recv(&receiver_data) { data } else { panic!(THREADS_MESSAGE_ERROR) };

                        // Find the PackedFile and get a mut ref to it, so we can "update" his data.
                        match pack_file_decoded.packed_files.iter_mut().find(|x| x.path == path) {
                            Some(packed_file) => {

                                // Create a temporal file for the PackedFile in the TEMP directory of the filesystem.
                                let mut temp_path = temp_dir();
                                temp_path.push(packed_file.path.last().unwrap().to_owned());
                                match File::create(&temp_path) {
                                    Ok(mut file) => {
                                        match packed_file.get_data_and_keep_it() {
                                            Ok(data) => {

                                                // If there is an error while trying to write the image to the TEMP folder, report it.
                                                if file.write_all(&data).is_err() {
                                                    sender.send(Data::Error(Error::from(ErrorKind::IOGenericWrite(vec![temp_path.display().to_string();1])))).unwrap();
                                                }

                                                // Otherwise...
                                                else { 

                                                    // No matter how many times I tried, it's IMPOSSIBLE to open a file on windows, so instead we use this magic crate that seems to work everywhere.
                                                    if open::that(&temp_path).is_err() { sender.send(Data::Error(Error::from(ErrorKind::IOGeneric))).unwrap(); }
                                                    else { sender.send(Data::Success).unwrap(); }
                                                }
                                            },
                                            Err(_) => sender.send(Data::Error(Error::from(ErrorKind::PackedFileDataCouldNotBeLoaded))).unwrap(),
                                        }
                                    }
                                    Err(_) => sender.send(Data::Error(Error::from(ErrorKind::IOGenericWrite(vec![temp_path.display().to_string();1])))).unwrap(),
                                }
                            }
                            None => sender.send(Data::Error(Error::from(ErrorKind::PackedFileNotFound))).unwrap(),
                        }
                    }
                }
            }

            // If you got an error, it means the main UI Thread is dead.
            Err(_) => {

                // Print a message in case we got a terminal to show it.
                println!("Main UI Thread dead. Exiting...");

                // Break the loop, effectively terminating the thread.
                break;
            },
        }
    }
}
