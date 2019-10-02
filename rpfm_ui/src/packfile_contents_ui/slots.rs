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

use qt_widgets::tree_view::TreeView;
use qt_widgets::slots::SlotQtCorePointRef;

use qt_gui::cursor::Cursor;

use qt_core::slots::{SlotItemSelectionRefItemSelectionRef, SlotNoArgs};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::PackTree;
use crate::packfile_contents_ui::PackFileContentsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub contextual_menu: SlotQtCorePointRef<'static>,
    pub contextual_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
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

        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});


        // And here... we return all the slots.
		Self {
            contextual_menu,
            contextual_menu_enabler,
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}
