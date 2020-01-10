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
Module with all the code related to the main `PackFileContentsSlots`.
!*/

use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;

use qt_core::qt::CaseSensitivity;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef};

use std::cell::RefCell;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_error::ErrorKind;
use rpfm_lib::common::get_files_from_subdir;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::text::TextType;
use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::{icons::IconType, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::packfile::PackFileExtraView;
use crate::packedfile_views::{PackedFileView, TheOneSlot};
use crate::QString;
use crate::utils::show_dialog;
use crate::UI_STATE;
use crate::ui_state::op_mode::OperationalMode;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub open_packedfile_preview: SlotNoArgs<'static>,
    pub open_packedfile_full: SlotNoArgs<'static>,

    pub filter_change_text: SlotStringRef<'static>,
    pub filter_change_autoexpand_matches: SlotBool<'static>,
    pub filter_change_case_sensitive: SlotBool<'static>,

    pub contextual_menu: SlotQtCorePointRef<'static>,
    pub contextual_menu_enabler: SlotNoArgs<'static>,

    pub contextual_menu_add_file: SlotBool<'static>,
    pub contextual_menu_add_folder: SlotBool<'static>,
    pub contextual_menu_add_from_packfile: SlotBool<'static>,
    pub contextual_menu_delete: SlotBool<'static>,
    pub contextual_menu_extract: SlotBool<'static>,
    pub contextual_menu_rename: SlotBool<'static>,

    pub contextual_menu_new_packed_file_db: SlotBool<'static>,
    pub contextual_menu_new_packed_file_loc: SlotBool<'static>,
    pub contextual_menu_new_packed_file_text: SlotBool<'static>,
    pub contextual_menu_new_folder: SlotBool<'static>,

    pub contextual_menu_new_queek_packed_file: SlotBool<'static>,
    pub contextual_menu_tables_check_integrity: SlotBool<'static>,
    pub contextual_menu_tables_merge_tables: SlotBool<'static>,
    pub contextual_menu_tables_update_table: SlotBool<'static>,

    pub contextual_menu_mass_import_tsv: SlotBool<'static>,
    pub contextual_menu_mass_export_tsv: SlotBool<'static>,

    pub packfile_contents_tree_view_expand_all: SlotNoArgs<'static>,
    pub packfile_contents_tree_view_collapse_all: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsSlots`.
impl PackFileContentsSlots {

	/// This function creates an entire `PackFileContentsSlots` struct.
	pub fn new(
        app_ui: AppUI,
        pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>
    ) -> Self {

        // Slot to open the selected PackedFile as a preview.
        let open_packedfile_preview = SlotNoArgs::new(clone!(slot_holder => move || {
            app_ui.open_packedfile(&pack_file_contents_ui, &global_search_ui, &slot_holder, true);
        }));

        // Slot to open the selected PackedFile as a permanent view.
        let open_packedfile_full = SlotNoArgs::new(clone!(slot_holder => move || {
            app_ui.open_packedfile(&pack_file_contents_ui, &global_search_ui, &slot_holder, false);
        }));

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotStringRef::new(move |_| {
            pack_file_contents_ui.filter_files();
        });
        let filter_change_autoexpand_matches = SlotBool::new(move |_| {
            pack_file_contents_ui.filter_files();
        });
        let filter_change_case_sensitive = SlotBool::new(move |_| {
            pack_file_contents_ui.filter_files();
        });

        // Slot to show the Contextual Menu for the TreeView.
        let contextual_menu = SlotQtCorePointRef::new(move |_| {
            unsafe { pack_file_contents_ui.packfile_contents_tree_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
        });

        // Slot to enable/disable contextual actions depending on the selected item.
        let contextual_menu_enabler = SlotNoArgs::new(move || {
                let (contents, files, folders) = <*mut TreeView as PackTree>::get_combination_from_main_treeview_selection(&pack_file_contents_ui);
                match contents {

                    // Only one or more files selected.
                    1 => {

                        // These options are valid for 1 or more files.
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
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
                        let enabled = files == 1;
                        unsafe {
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(enabled);
                        }

                        // Only if we have multiple files selected, we give the option to merge. Further checks are done when clicked.
                        let enabled = files > 1;
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
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);

                        }

                        // These options are limited to only 1 folder selected.
                        let enabled = folders == 1;
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(enabled);
                        }
                    },

                    // One or more files and one or more folders selected.
                    3 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // One PackFile (you cannot have two in the same TreeView) selected.
                    4 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // PackFile and one or more files selected.
                    5 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // PackFile and one or more folders selected.
                    6 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // PackFile, one or more files, and one or more folders selected.
                    7 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value.
                    0 | 8..=255 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_new_queek_packed_file.as_mut().unwrap().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false);
                        }
                    },
                }

                // Ask the other thread if there is a Dependency Database and a Schema loaded.
                CENTRAL_COMMAND.send_message_qt(Command::IsThereADependencyDatabase);
                CENTRAL_COMMAND.send_message_qt(Command::IsThereASchema);
                let response = CENTRAL_COMMAND.recv_message_qt();
                let is_there_a_dependency_database = match response {
                    Response::Bool(it_is) => it_is,
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                };

                let response = CENTRAL_COMMAND.recv_message_qt();
                let is_there_a_schema = match response {
                    Response::Bool(it_is) => it_is,
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                };

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || !is_there_a_schema {
                    unsafe { pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_update_table.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_new_packed_file_db.as_mut().unwrap().set_enabled(false); }
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
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &global_search_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false) }
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
                            for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }

                            pack_file_contents_ui.add_packedfiles(&app_ui, &global_search_ui, &paths, &paths_packedfile);
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
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &global_search_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false) }
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
                            let ui_base_path: Vec<String> = <*mut TreeView as PackTree>::get_path_from_main_treeview_selection(&pack_file_contents_ui)[0].to_vec();
                            pack_file_contents_ui.add_packed_files_from_folders(&app_ui, &global_search_ui, &folder_paths, &[ui_base_path]);
                        }
                    }
                }
            }
        );

        // What happens when we trigger the "Add From PackFile" action in the Contextual Menu.
        let contextual_menu_add_from_packfile = SlotBool::new(clone!(
            slot_holder => move |_| {

                // Create the FileDialog to get the PackFile to open, configure it and run it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Select PackFile"),
                )) };

                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                if file_dialog.exec() == 1 {
                    let path_str = file_dialog.selected_files().at(0).to_std_string();
                    let path = PathBuf::from(path_str.to_owned());
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    let mut tab = PackedFileView::default();
                    match PackFileExtraView::new_view(&mut tab, &app_ui, &pack_file_contents_ui, &global_search_ui, path) {
                        Ok(slots) => {
                            slot_holder.borrow_mut().push(slots);

                            // Add the file to the 'Currently open' list and make it visible.
                            let tab_widget = tab.get_mut_widget();
                            let name = path_str;
                            let icon_type = IconType::PackFile(false);
                            let icon = icon_type.get_icon_from_path();

                            // If there is another Extra PackFile already open, close it.
                            {
                                let open_packedfiles = UI_STATE.set_open_packedfiles();
                                if let Some(view) = open_packedfiles.get(&vec!["extra_packfile.rpfm_reserved".to_owned()]) {
                                    let widget = view.get_mut_widget();
                                    let index = unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };

                                    unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }
                                }
                            }
                            app_ui.purge_that_one_specifically(global_search_ui, &["extra_packfile.rpfm_reserved".to_owned()], false);

                            unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                            unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.insert(vec!["packfile_extra.rpfm_reserved".to_owned()], tab);

                        }
                        Err(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                    }
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let contextual_menu_delete = SlotBool::new(clone!(
            slot_holder => move |_| {
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                let selected_items = selected_items.iter().map(From::from).collect::<Vec<PathType>>();

                CENTRAL_COMMAND.send_message_qt(Command::DeletePackedFiles(selected_items));
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::VecPathType(deleted_items) => {
                        let items = deleted_items.iter().map(From::from).collect::<Vec<TreePathType>>();
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items.to_vec()));

                        // Remove all the deleted PackedFiles from the cache.
                        for item in &items {
                            match item {
                                TreePathType::File(path) => app_ui.purge_that_one_specifically(global_search_ui, path, false),
                                TreePathType::Folder(path) => {
                                    let mut paths_to_remove = vec![];
                                    {
                                        let open_packedfiles = UI_STATE.set_open_packedfiles();
                                        for packed_file_path in open_packedfiles.keys() {
                                            if !packed_file_path.is_empty() && packed_file_path.starts_with(path) {
                                                paths_to_remove.push(packed_file_path.to_vec());
                                            }
                                        }
                                    }

                                    for path in paths_to_remove {
                                        app_ui.purge_that_one_specifically(global_search_ui, &path, false);
                                    }

                                }
                                TreePathType::PackFile => app_ui.purge_them_all(global_search_ui, &slot_holder),
                                TreePathType::None => unreachable!(),
                            }
                        }
                    },
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                };
            }
        ));

        // What happens when we trigger the "Extract" action in the Contextual Menu.
        let contextual_menu_extract = SlotBool::new(move |_| {

                // Get the currently selected paths (and visible) paths.
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                let selected_items = selected_items.iter().map(From::from).collect::<Vec<PathType>>();
                let extraction_path = match UI_STATE.get_operational_mode() {

                    // In MyMod mode we extract directly to the folder of the selected MyMod, keeping the folder structure.
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                        if let Some(ref mymods_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                            // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                            let mut mod_name = mod_name.to_owned();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();

                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(&mod_name);
                            assets_folder
                        }

                        // If there is no MyMod path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, true); }
                    }

                    // In normal mode, we ask the user to provide us with a path.
                    OperationalMode::Normal => {
                        let extraction_path = unsafe { FileDialog::get_existing_directory_unsafe((
                            app_ui.main_window as *mut Widget,
                            &QString::from_std_str("Extract PackFile"),
                        )) };

                        if !extraction_path.is_empty() { PathBuf::from(extraction_path.to_std_string()) }
                        else { return }
                    }
                };

                // We have to save our data from cache to the backend before extracting it. Otherwise we would extract outdated data.
                // TODO: Make this more... optimal.
                UI_STATE.get_open_packedfiles().iter().for_each(|(path, packed_file)| packed_file.save(path, global_search_ui));

                CENTRAL_COMMAND.send_message_qt(Command::ExtractPackedFiles(selected_items, extraction_path));
                unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::String(result) => show_dialog(app_ui.main_window as *mut Widget, result, true),
                    Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
                unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        );


        // What happens when we trigger the "Rename" Action.
        let contextual_menu_rename = SlotBool::new(move |_| {

                // Get the currently selected items, and check how many of them are valid before trying to rewrite them.
                // Why? Because I'm sure there is an asshole out there that it's going to try to give the files duplicated
                // names, and if that happen, we have to stop right there that criminal scum.
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if let Some(rewrite_sequence) = PackFileContentsUI::create_rename_dialog(&app_ui, &selected_items) {
                    let mut renaming_data_background: Vec<(PathType, String)> = vec![];
                    for item_type in selected_items {
                        match item_type {
                            TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                                let original_name = path.last().unwrap();
                                let new_name = rewrite_sequence.to_owned().replace("{x}", &original_name);
                                renaming_data_background.push((From::from(&item_type), new_name));

                            },

                            // These two should, if everything works properly, never trigger.
                            TreePathType::PackFile | TreePathType::None => unimplemented!(),
                        }
                    }

                    // Send the renaming data to the Background Thread, wait for a response.
                    CENTRAL_COMMAND.send_message_qt(Command::RenamePackedFiles(renaming_data_background));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    match response {
                        Response::VecPathTypeVecString(renamed_items) => {
                            let renamed_items = renamed_items.iter().map(|x| (From::from(&x.0), x.1.to_owned())).collect::<Vec<(TreePathType, Vec<String>)>>();
                            let mut path_changes = vec![];
                            let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                            for (path, _) in open_packedfiles.iter_mut() {
                                if !path.is_empty() {
                                    for (item_type, new_path) in &renamed_items {

                                        // Due to how the backend is built (doing a Per-PackedFile movement) we will always receive here individual PackedFiles.
                                        // So we don't need to check the rest. But the name change can be in any place of the path, so we have to take that into account.
                                        if let TreePathType::File(ref current_path) = item_type {
                                            if current_path == path {
                                                path_changes.push((current_path.to_vec(), new_path.to_vec()));

                                                // Update the global search stuff, if needed.
                                                global_search_ui.search_on_path(vec![PathType::File(new_path.to_vec()); 1]);
                                            }
                                        }
                                    }
                                }
                            }

                            for (path_before, path_after) in &path_changes {
                                let data = open_packedfiles.remove(path_before).unwrap();
                                let widget = data.get_mut_widget();
                                let index = unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };
                                let old_name = path_before.last().unwrap();
                                let new_name = path_after.last().unwrap();
                                if old_name != new_name {
                                    unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().set_tab_text(index, &QString::from_std_str(new_name)); }
                                }
                                open_packedfiles.insert(path_after.to_vec(), data);
                            }

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Move(renamed_items));
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
            }
        );

        // What happens when we trigger the "Create DB PackedFile" Action.
        let contextual_menu_new_packed_file_db = SlotBool::new(move |_| {
            app_ui.new_packed_file(&pack_file_contents_ui, &PackedFileType::DB);
        });

        // What happens when we trigger the "Create Loc PackedFile" Action.
        let contextual_menu_new_packed_file_loc = SlotBool::new(move |_| {
            app_ui.new_packed_file(&pack_file_contents_ui, &PackedFileType::Loc);
        });

        // What happens when we trigger the "Create Text PackedFile" Action.
        let contextual_menu_new_packed_file_text = SlotBool::new(move |_| {
            app_ui.new_packed_file(&pack_file_contents_ui, &PackedFileType::Text(TextType::Plain));
        });

        // What happens when we trigger the "New Folder" Action.
        let contextual_menu_new_folder = SlotBool::new(move |_| {

                // Create the "New Folder" dialog and wait for a new name (or a cancelation).
                if let Some(new_folder_name) = app_ui.new_folder_dialog() {

                    // Get the currently selected paths, and only continue if there is only one.
                    let selected_paths = <*mut TreeView as PackTree>::get_path_from_main_treeview_selection(&pack_file_contents_ui);
                    if selected_paths.len() == 1 {

                        // Add the folder's name to the list.
                        let mut complete_path = selected_paths[0].to_vec();
                        complete_path.append(&mut (new_folder_name.split('/').map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>()));

                        // Check if the folder exists.
                        CENTRAL_COMMAND.send_message_qt(Command::FolderExists(complete_path.to_vec()));
                        let response = CENTRAL_COMMAND.recv_message_qt();
                        let folder_exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

                        // If the folder already exists, return an error.
                        if folder_exists { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::FolderAlreadyInPackFile, false)}
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::Folder(complete_path); 1]));
                    }
                }
            }
        );

        // What happens when we trigger the "Create Text PackedFile" Action.
        let contextual_menu_new_queek_packed_file = SlotBool::new(move |_| {
            app_ui.new_queek_packed_file(&pack_file_contents_ui);
        });

        // What happens when we trigger the "Check Tables" action in the Contextual Menu.
        let contextual_menu_tables_check_integrity = SlotBool::new(move |_| {

            // Disable the window and trigger the check for all tables in the PackFile.
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
            CENTRAL_COMMAND.send_message_qt(Command::DBCheckTableIntegrity);
            let response = CENTRAL_COMMAND.recv_message_qt();
            match response {
                Response::Success => show_dialog(app_ui.main_window as *mut Widget, "No errors detected.", true),
                Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
        });

        // What happens when we trigger the "Merge Tables" action in the Contextual Menu.
        let contextual_menu_tables_merge_tables = SlotBool::new(move |_| {

            // Get the currently selected paths, and get how many we have of each type.
            let selected_paths = <*mut TreeView as PackTree>::get_path_from_main_treeview_selection(&pack_file_contents_ui);

            // First, we check if we're merging locs, as it's far simpler.
            let mut loc_pass = true;
            for path in &selected_paths {
                if !path.last().unwrap().ends_with(".loc") {
                    loc_pass = false;
                    break;
                }
            }

            // Then DB Tables. The conditions are that they're in the same db folder and with the same version.
            // If ANY of these fails (until the "update table version" feature it's done), we fail the pass.
            // Due to performance reasons, the version thing will be done later.
            let mut db_pass = true;
            let mut db_folder = String::new();
            for path in &selected_paths {
                if path.len() == 3 {
                    if path[0] == "db" {
                        if db_folder.is_empty() {
                            db_folder = path[1].to_owned();
                        }

                        if path[1] != db_folder {
                            db_pass = false;
                            break;
                        }
                    }
                    else {
                        db_pass = false;
                        break;
                    }
                }
                else {
                    db_pass = false;
                    break;
                }
            }

            // If we got valid files, create the dialog to ask for the needed info.
            if (loc_pass || db_pass) && !(loc_pass && db_pass) {

                // Get the info for the merged file.
                if let Some((mut name, delete_source_files)) = app_ui.merge_tables_dialog() {

                    // If it's a loc file and the name doesn't end in a ".loc" termination, call it ".loc".
                    if loc_pass && !name.ends_with(".loc") {
                        name.push_str(".loc");
                    }

                    // Close the open and selected files.
                    let mut paths_to_close = vec![];
                    {
                        let open_packedfiles = UI_STATE.set_open_packedfiles();
                        for (path, _) in open_packedfiles.iter() {
                            if selected_paths.contains(path) {
                                paths_to_close.push(path.to_vec());
                            }
                        }
                    }

                    for path in paths_to_close {
                        app_ui.purge_that_one_specifically(global_search_ui, &path, true);
                    }

                    CENTRAL_COMMAND.send_message_qt(Command::MergeTables(selected_paths.to_vec(), name, delete_source_files));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    match response {
                        Response::VecString(path_to_add) => {

                            // If we want to delete the sources, do it now.
                            if delete_source_files {
                                let items_to_remove = selected_paths.iter().map(|x| TreePathType::File(x.to_vec())).collect();
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items_to_remove));
                            }

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(path_to_add.to_vec()); 1]));

                            // Update the global search stuff, if needed.
                            global_search_ui.search_on_path(vec![PathType::File(path_to_add); 1]);
                            /*

                            // Remove the added file from the data history if exists.
                            if table_state_data.borrow().get(&path_to_add).is_some() {
                                table_state_data.borrow_mut().remove(&path_to_add);
                            }
                            // Same with the deleted ones.
                            for item in &items_to_remove {
                                let path = if let TreePathType::File(path) = item { path.to_vec() } else { panic!("This should never happen.") };
                                if table_state_data.borrow().get(&path).is_some() {
                                    table_state_data.borrow_mut().remove(&path);
                                }

                                let data = TableStateData::new_empty();
                                table_state_data.borrow_mut().insert(path.to_vec(), data);
                            }*/
                        }

                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
            }

            else { show_dialog(app_ui.main_window as *mut Widget, ErrorKind::InvalidFilesForMerging, false); }
        });


        // What happens when we trigger the "Update Table" action in the Contextual Menu.
        let contextual_menu_tables_update_table = SlotBool::new(clone!(slot_holder => move |_| {
            let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &selected_items[0] } else { return };
            match item_type {
                TreePathType::File(_) => {

                    // First, if the PackedFile is open, save it.
                    app_ui.purge_them_all(global_search_ui, &slot_holder);

                    let path_type: PathType = From::from(item_type);
                    CENTRAL_COMMAND.send_message_qt(Command::UpdateTable(path_type.clone()));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    match response {
                        Response::I32I32((old_version, new_version)) => {
                            let message = format!("Table updated from version '{}' to version '{}'.", old_version, new_version);
                            show_dialog(app_ui.main_window as *mut Widget, message, true);
                            global_search_ui.search_on_path(vec![path_type; 1]);
                        }

                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
                _ => unimplemented!()
            }
        }));

        // What happens when we trigger the "Mass-Import TSV" Action.
        //
        // TODO: Make it so the name of the table is split off when importing keeping the original name.
        let contextual_menu_mass_import_tsv = SlotBool::new(move |_| {

                // Don't do anything if there is a PackedFile open. This fixes the situation where you could overwrite data already in the UI.
                //if !packedfiles_open_in_packedfile_view.borrow().is_empty() { return show_dialog(app_ui.window, false, ErrorKind::PackedFileIsOpen) }

                // Create the "Mass-Import TSV" dialog and wait for his data (or a cancelation).
                if let Some(data) = PackFileContentsUI::create_mass_import_tsv_dialog(&app_ui) {

                    // If there is no name provided, nor TSV file selected, return an error.
                    if let Some(ref name) = data.1 {
                        if name.is_empty() { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::EmptyInput, false) }
                    }
                    if data.0.is_empty() { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::NoFilesToImport, false) }

                    // Otherwise, try to import all of them and report the result.
                    else {
                        unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        CENTRAL_COMMAND.send_message_qt(Command::MassImportTSV(data.0, data.1));
                        let response = CENTRAL_COMMAND.recv_message_qt();
                        match response {

                            // If it's success....
                            Response::VecVecStringVecVecString(paths) => {

                                // Get the list of paths to add, removing those we "replaced".
                                let mut paths_to_add = paths.1.to_vec();
                                paths_to_add.retain(|x| !paths.0.contains(&x));
                                let paths_to_add2 = paths_to_add.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();

                                // Update the TreeView.
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths_to_add2));

                                // Update the global search stuff, if needed.
                                global_search_ui.search_on_path(paths_to_add.iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>());

                                // For each file added, remove it from the data history if exists.
                                /*
                                for path in &paths.1 {
                                    if table_state_data.borrow().get(path).is_some() {
                                        table_state_data.borrow_mut().remove(path);
                                    }

                                    let data = TableStateData::new_empty();
                                    table_state_data.borrow_mut().insert(path.to_vec(), data);
                                }*/
                            }

                            Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                        }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        );

        // What happens when we trigger the "Mass-Export TSV" Action.
        let contextual_menu_mass_export_tsv = SlotBool::new(move |_| {

                // Get a "Folder-only" FileDialog.
                let export_path = unsafe { FileDialog::get_existing_directory_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Select destination folder")
                )) };

                // If we got an export path and it's not empty, try to export all selected files there.
                if !export_path.is_empty() {
                    let export_path = PathBuf::from(export_path.to_std_string());
                    if export_path.is_dir() {
                        unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                        let selected_items = selected_items.iter().map(From::from).collect::<Vec<PathType>>();
                        CENTRAL_COMMAND.send_message_qt(Command::MassExportTSV(selected_items, export_path));
                        let response = CENTRAL_COMMAND.recv_message_qt();
                        match response {
                            Response::String(response) => show_dialog(app_ui.main_window as *mut Widget, response, true),
                            Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }

                        unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        );

        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});

        // And here... we return all the slots.
		Self {
            open_packedfile_preview,
            open_packedfile_full,

            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,

            contextual_menu,
            contextual_menu_enabler,

            contextual_menu_add_file,
            contextual_menu_add_folder,
            contextual_menu_add_from_packfile,
            contextual_menu_delete,
            contextual_menu_extract,
            contextual_menu_rename,

            contextual_menu_new_packed_file_db,
            contextual_menu_new_packed_file_loc,
            contextual_menu_new_packed_file_text,
            contextual_menu_new_folder,

            contextual_menu_new_queek_packed_file,
            contextual_menu_tables_check_integrity,
            contextual_menu_tables_merge_tables,
            contextual_menu_tables_update_table,

            contextual_menu_mass_import_tsv,
            contextual_menu_mass_export_tsv,

            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}
