//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `PackFileContentsSlots`.
!*/

use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;

use qt_core::slots::{SlotItemSelectionRefItemSelectionRef, SlotNoArgs, SlotBool};

use std::fs::DirBuilder;
use std::path::{Path, PathBuf};

use rpfm_error::ErrorKind;
use rpfm_lib::common::get_files_from_subdir;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::{check_if_path_is_closed, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::utils::show_dialog;
use crate::UI_STATE;
use crate::ui_state::op_mode::OperationalMode;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub contextual_menu: SlotQtCorePointRef<'static>,
    pub contextual_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,

    pub contextual_menu_add_file: SlotBool<'static>,
    pub contextual_menu_add_folder: SlotBool<'static>,
    pub packfile_contents_tree_view_expand_all: SlotNoArgs<'static>,
    pub packfile_contents_tree_view_collapse_all: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsSlots`.
impl PackFileContentsSlots {

	/// This function creates an entire `PackFileContentsSlots` struct.
	pub fn new(app_ui: AppUI, pack_file_contents_ui: PackFileContentsUI) -> Self {

        // Slot to show the Contextual Menu for the TreeView.
        let contextual_menu = SlotQtCorePointRef::new(move |_| {
            unsafe { pack_file_contents_ui.packfile_contents_tree_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
        });

        // Slot to enable/disable contextual actions depending on the selected item.
        let contextual_menu_enabler = SlotItemSelectionRefItemSelectionRef::new(move |_,_| {
                let (contents, files, folders) = <*mut TreeView as PackTree>::get_combination_from_main_treeview_selection(&app_ui, &pack_file_contents_ui);
                match contents {

                    // Only one or more files selected.
                    1 => {

                        // These options are valid for 1 or more files.
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 file selected, and should not be usable if multiple files
                        // are selected.
                        let enabled = if files == 1 { true } else { false };
                        unsafe {
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(enabled);
                        }

                        // Only if we have multiple files selected, we give the option to merge. Further checks are done when clicked.
                        let enabled = if files > 1 { true } else { false };
                        unsafe { pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(enabled); }
                    },

                    // Only one or more folders selected.
                    2 => {

                        // These options are valid for 1 or more folders.
                        unsafe {
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 folder selected.
                        let enabled = if folders == 1 { true } else { false };
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(enabled);
                        }
                    },

                    // One or more files and one or more folders selected.
                    3 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // One PackFile (you cannot have two in the same TreeView) selected.
                    4 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more files selected.
                    5 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more folders selected.
                    6 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile, one or more files, and one or more folders selected.
                    7 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value.
                    0 | 8..=255 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(false);
                        }
                    },
                }

                // Ask the other thread if there is a Dependency Database and a Schema loaded.
                CENTRAL_COMMAND.send_message_qt(Command::IsThereADependencyDatabase);
                CENTRAL_COMMAND.send_message_qt(Command::IsThereASchema);
                let is_there_a_dependency_database = match CENTRAL_COMMAND.recv_message_qt() {
                    Response::Bool(it_is) => it_is,
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                };

                let is_there_a_schema = match CENTRAL_COMMAND.recv_message_qt() {
                    Response::Bool(it_is) => it_is,
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                };

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || !is_there_a_schema {
                    unsafe { pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false); }
                }
            }
        );

        // What happens when we trigger the "Add File/s" action in the Contextual Menu.
        let contextual_menu_add_file = SlotBool::new(move |_| {

                // Create the FileDialog to get the file/s to add and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Add File/s"),
                )) };
                file_dialog.set_file_mode(FileMode::ExistingFiles);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = &SETTINGS.lock().unwrap().paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::IOCreateAssetFolder, false);
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the files we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Check if the files are in the Assets Folder. The file chooser kinda guarantees that
                                // all are in the same folder, so we can just check the first one.
                                let paths_packedfile: Vec<Vec<String>> = if paths[0].starts_with(&assets_folder) {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&app_ui, &pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false); }
                    }

                    // If it's in "Normal" mode...
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the files we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get their final paths in the PackFile and only proceed if all of them are closed.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&app_ui, &pack_file_contents_ui, &path, true)); }

                            pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                        }
                    }
                }
            }
        );


        // What happens when we trigger the "Add Folder/s" action in the Contextual Menu.
        let contextual_menu_add_folder = SlotBool::new(move |_| {

                // Create the FileDialog to get the folder/s to add and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Add Folder/s"),
                )) };
                file_dialog.set_file_mode(FileMode::Directory);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = &SETTINGS.lock().unwrap().paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::IOCreateAssetFolder, false);
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the folders we want to add.
                                let mut folder_paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Get the Paths of the files inside the folders we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                                // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                                let paths_packedfile = if paths[0].starts_with(&assets_folder) {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&app_ui, &pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false); }
                    }

                    // If it's in "Normal" mode, we just get the paths of the files inside them and add those files.
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the folders we want to add.
                            let mut folder_paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get the Paths of the files inside the folders we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                            // Get their final paths in the PackFile and only proceed if all of them are closed.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&app_ui, &pack_file_contents_ui, &path, true)); }
                            pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                        }
                    }
                }
            }
        );

        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});


        // And here... we return all the slots.
		Self {
            contextual_menu,
            contextual_menu_enabler,

            contextual_menu_add_file,
            contextual_menu_add_folder,
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}
