//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::{QFileDialog, q_file_dialog::FileMode};
use qt_widgets::SlotOfQPoint;
use qt_widgets::QTreeView;

use qt_gui::QCursor;
use qt_gui::QGuiApplication;

use qt_core::QBox;
use qt_core::{SlotNoArgs, SlotOfBool, SlotOfQModelIndexInt, SlotOfQString};
use qt_core::QPtr;
use qt_core::QString;

use std::collections::HashSet;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_lib::files::{ContainerPath, FileType, pack::*};
use rpfm_lib::integrations::log::*;
use rpfm_lib::utils::*;

use rpfm_ui_common::clone;
use rpfm_ui_common::locale::{qtr, tre};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, SpecialView};
use crate::references_ui::ReferencesUI;
use crate::SCHEMA;
use crate::settings_ui::backend::*;
use crate::utils::{show_dialog, check_regex};
use crate::UI_STATE;
use crate::ui_state::OperationalMode;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub move_items: QBox<SlotOfQModelIndexInt>,

    pub open_packedfile_preview: QBox<SlotNoArgs>,
    pub open_packedfile_full: QBox<SlotNoArgs>,

    pub filter_trigger: QBox<SlotNoArgs>,
    pub filter_change_text: QBox<SlotOfQString>,
    pub filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub filter_change_case_sensitive: QBox<SlotOfBool>,
    pub filter_check_regex: QBox<SlotOfQString>,

    pub contextual_menu: QBox<SlotOfQPoint>,
    pub contextual_menu_enabler: QBox<SlotNoArgs>,

    pub contextual_menu_add_file: QBox<SlotOfBool>,
    pub contextual_menu_add_folder: QBox<SlotOfBool>,
    pub contextual_menu_add_from_packfile: QBox<SlotOfBool>,
    pub contextual_menu_delete: QBox<SlotOfBool>,
    pub contextual_menu_extract: QBox<SlotOfBool>,
    pub contextual_menu_rename: QBox<SlotOfBool>,
    pub contextual_menu_copy_path: QBox<SlotOfBool>,

    pub contextual_menu_new_packed_file_anim_pack: QBox<SlotOfBool>,
    pub contextual_menu_new_packed_file_db: QBox<SlotOfBool>,
    pub contextual_menu_new_packed_file_loc: QBox<SlotOfBool>,
    pub contextual_menu_new_packed_file_portrait_settings: QBox<SlotOfBool>,
    pub contextual_menu_new_packed_file_text: QBox<SlotOfBool>,
    pub contextual_menu_new_folder: QBox<SlotOfBool>,
    pub contextual_menu_new_queek_packed_file: QBox<SlotOfBool>,

    pub contextual_menu_open_decoder: QBox<SlotOfBool>,
    pub contextual_menu_open_dependency_manager: QBox<SlotOfBool>,
    pub contextual_menu_open_containing_folder: QBox<SlotOfBool>,
    pub contextual_menu_open_in_external_program: QBox<SlotOfBool>,
    pub contextual_menu_open_packfile_settings: QBox<SlotOfBool>,
    pub contextual_menu_open_notes: QBox<SlotOfBool>,

    pub contextual_menu_tables_merge_tables: QBox<SlotOfBool>,
    pub contextual_menu_tables_update_table: QBox<SlotOfBool>,
    pub contextual_menu_generate_missing_loc_data: QBox<SlotOfBool>,

    pub packfile_contents_tree_view_expand_all: QBox<SlotNoArgs>,
    pub packfile_contents_tree_view_collapse_all: QBox<SlotNoArgs>,

    pub packfile_contents_resize: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsSlots`.
impl PackFileContentsSlots {

	/// This function creates an entire `PackFileContentsSlots` struct.
	pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Self {

        // Slot to move stuff with drag and drop.
        let move_items = SlotOfQModelIndexInt::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |dest_parent, dest_row| {
                info!("Triggering `Move` By Drag&Drop By Slot");

                // Rare case, but possible due to selection weirdness.
                let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if selected_items.is_empty() {
                    return;
                }

                // First, check if it's yet another idiot trying to move the db folders, and give him a warning.
                if selected_items.iter()
                    .filter_map(|item_type| if let ContainerPath::Folder(ref path) = item_type { Some(path) } else { None })
                    .any(|path| path.to_lowercase().starts_with("db/")) &&
                    !AppUI::are_you_sure_edition(&app_ui, "are_you_sure_rename_db_folder") {
                    return;
                }

                // Limitation: we can only move together files/folders under the same base_path.
                if selected_items.iter()
                    .map(|container_path| {
                        let mut split_path = container_path.path_raw().split('/').collect::<Vec<_>>();
                        split_path.pop();
                        split_path.join("/")
                    })
                    .collect::<HashSet<_>>()
                    .len() != 1 {
                    return;
                }

                let dest_index_visual = dest_parent.child(dest_row, 0);
                let dest_index_logical = pack_file_contents_ui.packfile_contents_tree_model_filter().map_to_source(&dest_index_visual);
                let mut new_base_path = <QPtr<QTreeView> as PackTree>::get_path_from_index(dest_index_logical.as_ref(), &pack_file_contents_ui.packfile_contents_tree_model().static_upcast());
                if !new_base_path.ends_with('/') {
                    new_base_path.push('/');
                }

                // Warn people before moving things to the db folder.
                if new_base_path.to_lowercase().starts_with("db/") && !AppUI::are_you_sure_edition(&app_ui, "are_you_sure_rename_db_folder") {
                    return;
                }

                // Prepare the new paths using the rename sequence.
                let mut renaming_data_background: Vec<(ContainerPath, ContainerPath)> = vec![];
                for item_type in &selected_items {
                    let original_path = item_type.path_raw().split('/').to_owned().collect::<Vec<_>>();
                    let mut new_path = new_base_path.to_owned();
                    new_path.push_str(original_path.last().unwrap());

                    let new_path = match item_type {
                        ContainerPath::File(_) => ContainerPath::File(new_path),
                        ContainerPath::Folder(_) => ContainerPath::Folder(new_path),
                    };

                    renaming_data_background.push((item_type.clone(), new_path));
                }

                // Send the renaming data to the Background Thread, wait for a response.
                let receiver = CENTRAL_COMMAND.send_background(Command::RenamePackedFiles(renaming_data_background.to_vec()));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::VecContainerPathContainerPath(renamed_items) => {
                        let mut path_changes = vec![];

                        // TODO: Filter out reserved files with some generic logic.
                        for path in UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| x.path_read()) {
                            if !path.is_empty() {
                                for (old_path, new_path) in &renamed_items {

                                    // No need to check for path type here, as we can only get file paths.
                                    if old_path.path_raw() == *path {
                                        path_changes.push((old_path.path_raw(), new_path.path_raw()));
                                    }
                                }
                            }
                        }

                        {
                            let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                            for (path_before, path_after) in &path_changes {
                                let position = open_packedfiles.iter().position(|x| *x.path_read() == *path_before && x.data_source() == DataSource::PackFile).unwrap();
                                let data = open_packedfiles.remove(position);
                                let widget = data.main_widget();
                                let index = app_ui.tab_bar_packed_file().index_of(widget);
                                let path_split_before = path_before.split('/').collect::<Vec<_>>();
                                let path_split_after = path_after.split('/').collect::<Vec<_>>();
                                let old_name = path_split_before.last().unwrap();
                                let new_name = path_split_after.last().unwrap();
                                if old_name != new_name {
                                    app_ui.tab_bar_packed_file().set_tab_text(index, &QString::from_std_str(new_name));
                                }

                                data.set_path(path_after);
                                open_packedfiles.push(data);
                            }
                        }

                        // Move the items on the UI and mark the currently open Pack as modified.
                        let folders_to_move = selected_items.into_iter()
                            .filter(|path| matches!(path, ContainerPath::Folder(_)))
                            .collect::<Vec<_>>();

                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Move(renamed_items, folders_to_move), DataSource::PackFile);

                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    },
                    Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        ));

        // Slot to open the selected PackedFile as a preview.
        let open_packedfile_preview = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move || {
            info!("PackedFile opened as Preview By Slot");
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, None, true, false, DataSource::PackFile);
        }));

        // Slot to open the selected PackedFile as a permanent view.
        let open_packedfile_full = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move || {
            info!("PackedFile opened as Full By Slot");
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, None, false, false, DataSource::PackFile);
        }));

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotOfQString::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |_| {
                PackFileContentsUI::start_delayed_updates_timer(&pack_file_contents_ui);
            }
        ));
        let filter_change_autoexpand_matches = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |_| {
                PackFileContentsUI::filter_files(&pack_file_contents_ui);
            }
        ));
        let filter_change_case_sensitive = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |_| {
                PackFileContentsUI::filter_files(&pack_file_contents_ui);
            }
        ));

        // Function triggered by the filter timer.
        let filter_trigger = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move || {
                PackFileContentsUI::filter_files(&pack_file_contents_ui);
            }
        ));


        // What happens when we trigger the "Check Regex" action.
        let filter_check_regex = SlotOfQString::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |string| {
                check_regex(&string.to_std_string(), pack_file_contents_ui.filter_line_edit.static_upcast());
            }
        ));

        // Slot to show the Contextual Menu for the TreeView.
        let contextual_menu = SlotOfQPoint::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |_| {
            pack_file_contents_ui.packfile_contents_tree_view_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // Slot to enable/disable contextual actions depending on the selected item.
        let contextual_menu_enabler = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move || {
                let (contents, files, folders, _) = <QPtr<QTreeView> as PackTree>::get_combination_from_main_treeview_selection(&pack_file_contents_ui);
                match contents {

                    // Only one or more files selected.
                    1 => {

                        // These options are valid for 1 or more files.
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(true);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);

                        // These options are limited to only 1 file selected, and should not be usable if multiple files
                        // are selected.
                        let enabled = files == 1;
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(enabled);

                        // Only if we have multiple files selected, we give the option to merge. Further checks are done when clicked.
                        let enabled = files > 1;
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(enabled);
                    },

                    // Only one or more folders selected.
                    2 => {

                        // These options are valid for 1 or more folders.
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(true);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);

                        // These options are limited to only 1 folder selected.
                        let enabled = folders == 1;
                        pack_file_contents_ui.context_menu_add_file.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(enabled);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(enabled);
                    },

                    // One or more files and one or more folders selected.
                    3 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },

                    // One PackFile (you cannot have two in the same TreeView) selected.
                    4 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(true);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(true);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(true);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(true);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(true);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },

                    // PackFile and one or more files selected.
                    5 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },

                    // PackFile and one or more folders selected.
                    6 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },

                    // PackFile, one or more files, and one or more folders selected.
                    7 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(true);
                        pack_file_contents_ui.context_menu_extract.set_enabled(true);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(true);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value.
                    0 | 8..=255 => {
                        pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_add_from_packfile.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                        pack_file_contents_ui.context_menu_new_queek_packed_file.set_enabled(false);
                        pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                        pack_file_contents_ui.context_menu_delete.set_enabled(false);
                        pack_file_contents_ui.context_menu_extract.set_enabled(false);
                        pack_file_contents_ui.context_menu_rename.set_enabled(false);
                        pack_file_contents_ui.context_menu_copy_path.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                        pack_file_contents_ui.context_menu_open_notes.set_enabled(false);
                        pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                    },
                }

                // If there is anything selected, we can generate missing loc data.
                if files > 0 || folders > 0 {
                    pack_file_contents_ui.context_menu_generate_missing_loc_data.set_enabled(true);
                } else {
                    pack_file_contents_ui.context_menu_generate_missing_loc_data.set_enabled(false);
                }

                // Ask the other thread if there is a Dependency Database and a Schema loaded.
                let receiver = CENTRAL_COMMAND.send_background(Command::IsThereADependencyDatabase(false));
                let response = CentralCommand::recv(&receiver);
                let is_there_a_dependency_database = match response {
                    Response::Bool(it_is) => it_is,
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                };

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || SCHEMA.read().unwrap().is_none() {
                    pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                }
            }
        ));

        // What happens when we trigger the "Add File/s" action in the Contextual Menu.
        let contextual_menu_add_file = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Add File` By Slot");

                // Create the FileDialog to get the file/s to add and configure it.
                let file_dialog = QFileDialog::from_q_widget_q_string(
                    app_ui.main_window(),
                    &qtr("context_menu_add_files"),
                );
                file_dialog.set_file_mode(FileMode::ExistingFiles);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = setting_path("mymods_base_path");
                        if mymods_base_path.is_dir() {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path;
                            assets_folder.push(game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref());
                            file_dialog.set_directory_q_string(&QString::from_std_str(assets_folder.to_string_lossy()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() {
                                if let Err(error) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                    return show_dialog(app_ui.main_window(), format!("Error while creating the MyMod's Assets folder: {error}"), false);
                                }
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the files we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() {
                                    paths.push(PathBuf::from(paths_qt.at(index).to_std_string()));
                                }

                                // Check if the files are in the Assets Folder. The file chooser kinda guarantees that
                                // all are in the same folder, so we can just check the first one.
                                let paths_packedfile: Vec<ContainerPath> = if paths[0].starts_with(&assets_folder) {
                                    let mut paths_packedfile: Vec<ContainerPath> = vec![];
                                    for path in &paths {
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(ContainerPath::File(filtered_path.to_string_lossy().to_string()));
                                    }
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {
                                    let mut paths_packedfile: Vec<ContainerPath> = vec![];
                                    for path in &paths {
                                        paths_packedfile.append(&mut <QPtr<QTreeView> as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, path, true).iter().map(|x| ContainerPath::File(x.to_string())).collect());
                                    }
                                    paths_packedfile
                                };

                                app_ui.toggle_main_window(false);
                                PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &paths, &paths_packedfile, None);
                                app_ui.toggle_main_window(true);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else {
                            show_dialog(app_ui.main_window(), "MyMod path is not configured. Configure it in the settings and try again.", false)
                        }
                    }

                    // If it's in "Normal" mode...
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the files we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() {
                                paths.push(PathBuf::from(paths_qt.at(index).to_std_string()));
                            }

                            // Get their final paths in the PackFile and only proceed if all of them are closed.
                            let mut paths_packedfile: Vec<ContainerPath> = vec![];
                            for path in &paths {
                                paths_packedfile.append(&mut <QPtr<QTreeView> as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, path, true).iter().map(|x| ContainerPath::File(x.to_string())).collect());
                            }

                            app_ui.toggle_main_window(false);
                            PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &paths, &paths_packedfile, None);
                            app_ui.toggle_main_window(true);
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Add Folder/s" action in the Contextual Menu.
        let contextual_menu_add_folder = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Add Folder` By Slot");

                // Create the FileDialog to get the folder/s to add and configure it.
                let file_dialog = QFileDialog::from_q_widget_q_string(
                    app_ui.main_window(),
                    &qtr("context_menu_add_folders"),
                );
                file_dialog.set_file_mode(FileMode::Directory);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = setting_path("mymods_base_path");
                        if mymods_base_path.is_dir() {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path;
                            assets_folder.push(game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref());
                            file_dialog.set_directory_q_string(&QString::from_std_str(assets_folder.to_string_lossy()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() {
                                if let Err(error) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                    return show_dialog(app_ui.main_window(), format!("Error while creating the MyMod's Assets folder: {error}"), false);
                                }
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the folders we want to add.
                                let mut folder_paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Get the Paths of the files inside the folders we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                for path in &folder_paths { paths.append(&mut files_from_subdir(path, true).unwrap()); }

                                // Check to ensure we actually have a path, as you may try to add empty folders.
                                if let Some(path) = paths.get(0) {

                                    // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                                    if path.starts_with(&assets_folder) {
                                        let mut paths_packedfile: Vec<ContainerPath> = vec![];
                                        for path in &paths {
                                            let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                            paths_packedfile.push(ContainerPath::File(filtered_path.to_string_lossy().to_string()));
                                        }
                                        PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &paths, &paths_packedfile, None);
                                    }

                                    // Otherwise, they are added like normal files.
                                    else if let Some(selection) = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection().get(0) {
                                        app_ui.toggle_main_window(false);
                                        PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &folder_paths, &[ContainerPath::Folder(selection.to_string())], None);
                                        app_ui.toggle_main_window(true);
                                    }
                                }
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else {
                            show_dialog(app_ui.main_window(), "MyMod path is not configured. Configure it in the settings and try again.", false)
                        }
                    }

                    // If it's in "Normal" mode, we just get the paths of the files inside them and add those files.
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the folders we want to add.
                            let mut folder_paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() {
                                folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string()));
                            }

                            // Get the Paths of the files inside the folders we want to add.
                            if let Some(selection) = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection().get(0) {

                                app_ui.toggle_main_window(false);
                                PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &folder_paths, &[ContainerPath::Folder(selection.to_string())], None);
                                app_ui.toggle_main_window(true);
                            }
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Add From PackFile" action in the Contextual Menu.
        let contextual_menu_add_from_packfile = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
                info!("Triggering `Add From PackFile` By Slot");

                // Create the FileDialog to get the PackFile to open, configure it and run it.
                let file_dialog = QFileDialog::from_q_widget_q_string(
                    app_ui.main_window(),
                    &qtr("context_menu_select_packfile"),
                );

                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                if file_dialog.exec() == 1 {
                    let path_str = file_dialog.selected_files().at(0).to_std_string();

                    // DON'T ALLOW TO LOAD THE SAME PACKFILE WE HAVE ALREADY OPEN!!!!
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileDataForTreeView);
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::ContainerInfoVecRFileInfo((pack_file_info, _)) => {
                            if pack_file_info.file_path() == &path_str {
                                 return show_dialog(app_ui.main_window(), "You cannot add PackedFile to the same PackFile you're adding from. It's like putting a bag of holding into a bag of holding.", false);
                            }
                        },
                        Response::Error(error) => return show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }

                    app_ui.toggle_main_window(false);
                    AppUI::open_special_view(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, SpecialView::Pack(path_str));

                    app_ui.toggle_main_window(true);
                }
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let contextual_menu_delete = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if AppUI::are_you_sure_edition(&app_ui, "are_you_sure_delete") {
                    info!("Triggering `Delete` By Slot");

                    let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);

                    let receiver = CENTRAL_COMMAND.send_background(Command::DeletePackedFiles(selected_items));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecContainerPath(items) => {
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items.to_vec()), DataSource::PackFile);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(items.to_vec()), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);

                            // Remove all the deleted PackedFiles from the cache.
                            for item in &items {
                                match item {
                                    ContainerPath::File(path) => { let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, false); },
                                    ContainerPath::Folder(path) => {
                                        let mut paths_to_remove = vec![];
                                        {
                                            let open_packedfiles = UI_STATE.set_open_packedfiles();
                                            for packed_file_path in open_packedfiles.iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| x.path_read()) {
                                                if !packed_file_path.is_empty() && packed_file_path.starts_with(path) {
                                                    paths_to_remove.push(packed_file_path.to_owned());
                                                }
                                            }
                                        }

                                        for path in paths_to_remove {
                                            let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, &path, DataSource::PackFile, false);
                                        }

                                    }
                                }
                            }
                        },
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };
                }
            }
        ));

        // What happens when we trigger the "Extract" action in the Contextual Menu.
        let contextual_menu_extract = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Extract` By Slot");
                PackFileContentsUI::extract_packed_files(&app_ui, &pack_file_contents_ui, None, true);
            }
        ));


        // What happens when we trigger the "Rename" Action.
        let contextual_menu_rename = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Rename` By Slot");

                // Rare case, but possible due to selection weirdness.
                let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if selected_items.is_empty() {
                    return;
                }

                // First, check if it's yet another idiot trying to rename the db folders, and give him a warning.
                if selected_items.iter()
                    .filter_map(|item_type| if let ContainerPath::Folder(ref path) = item_type { Some(path) } else { None })
                    .any(|path| path.to_lowercase().starts_with("db/")) &&
                    !AppUI::are_you_sure_edition(&app_ui, "are_you_sure_rename_db_folder") {
                    return;
                }

                // Ask for the new path of the item to rename. Even if we select multiple items, this returns the rename sequence for all of them.
                match PackFileContentsUI::create_rename_dialog(&app_ui, &selected_items) {
                    Ok(rewrite_sequence) => {
                        if let Some((rewrite_sequence, full_path_movement)) = rewrite_sequence {

                            // Prepare the new paths using the rename sequence.
                            let mut renaming_data_background: Vec<(ContainerPath, ContainerPath)> = vec![];
                            for item_type in &selected_items {
                                let mut original_path = item_type.path_raw().split('/').to_owned().collect::<Vec<_>>();

                                // Replace {x} with the original name.
                                let new_path = match full_path_movement {
                                    true => rewrite_sequence.replace("{x}", original_path.last().unwrap()),
                                    false => {
                                        let new_name = rewrite_sequence.replace("{x}", original_path.last().unwrap());
                                        *original_path.last_mut().unwrap() = &new_name;
                                        original_path.join("/")
                                    },
                                };

                                let new_path = match item_type {
                                    ContainerPath::File(_) => ContainerPath::File(new_path),
                                    ContainerPath::Folder(_) => ContainerPath::Folder(new_path),
                                };

                                renaming_data_background.push((item_type.clone(), new_path));
                            }

                            // Send the renaming data to the Background Thread, wait for a response.
                            let receiver = CENTRAL_COMMAND.send_background(Command::RenamePackedFiles(renaming_data_background.to_vec()));
                            let response = CentralCommand::recv(&receiver);
                            match response {
                                Response::VecContainerPathContainerPath(renamed_items) => {
                                    let mut path_changes = vec![];

                                    // TODO: Filter out reserved files with some generic logic.
                                    for path in UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| x.path_read()) {
                                        if !path.is_empty() {
                                            for (old_path, new_path) in &renamed_items {

                                                // No need to check for path type here, as we can only get file paths.
                                                if old_path.path_raw() == *path {
                                                    path_changes.push((old_path.path_raw(), new_path.path_raw()));
                                                }
                                            }
                                        }
                                    }

                                    {
                                        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                                        for (path_before, path_after) in &path_changes {
                                            let position = open_packedfiles.iter().position(|x| *x.path_read() == *path_before && x.data_source() == DataSource::PackFile).unwrap();
                                            let data = open_packedfiles.remove(position);
                                            let widget = data.main_widget();
                                            let index = app_ui.tab_bar_packed_file().index_of(widget);
                                            let path_split_before = path_before.split('/').collect::<Vec<_>>();
                                            let path_split_after = path_after.split('/').collect::<Vec<_>>();
                                            let old_name = path_split_before.last().unwrap();
                                            let new_name = path_split_after.last().unwrap();
                                            if old_name != new_name {
                                                let mut new_name = new_name.to_string();
                                                if data.is_preview() {
                                                    new_name.push_str(" (Preview)");
                                                }

                                                app_ui.tab_bar_packed_file().set_tab_text(index, &QString::from_std_str(new_name));
                                            }

                                            data.set_path(path_after);
                                            open_packedfiles.push(data);
                                        }
                                    }

                                    // Move the items on the UI and mark the currently open Pack as modified.
                                    let folders_to_move = selected_items.into_iter()
                                        .filter(|path| matches!(path, ContainerPath::Folder(_)))
                                        .collect::<Vec<_>>();

                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Move(renamed_items, folders_to_move), DataSource::PackFile);

                                    UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                                },
                                Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                            }
                        }
                    }
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

        let contextual_menu_copy_path = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move |_| {
            let selected_paths = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection();
            if selected_paths.len() == 1 {
                QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(&selected_paths[0]));
            }
        }));

        // What happens when we trigger the "Create AnimPack" Action.
        let contextual_menu_new_packed_file_anim_pack = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New AnimPack` By Slot");
            AppUI::new_file(&app_ui, &pack_file_contents_ui, FileType::AnimPack);
        }));

        // What happens when we trigger the "Create DB PackedFile" Action.
        let contextual_menu_new_packed_file_db = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New DB` By Slot");
            AppUI::new_file(&app_ui, &pack_file_contents_ui, FileType::DB);
        }));

        // What happens when we trigger the "Create Loc PackedFile" Action.
        let contextual_menu_new_packed_file_loc = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New Loc` By Slot");
            AppUI::new_file(&app_ui, &pack_file_contents_ui, FileType::Loc);
        }));

        // What happens when we trigger the "Create Portrait Settings File" Action.
        let contextual_menu_new_packed_file_portrait_settings = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New Portrait Settings` By Slot");
            AppUI::new_file(&app_ui, &pack_file_contents_ui, FileType::PortraitSettings);
        }));

        // What happens when we trigger the "Create Text PackedFile" Action.
        let contextual_menu_new_packed_file_text = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New Text` By Slot");
            AppUI::new_file(&app_ui, &pack_file_contents_ui, FileType::Text);
        }));

        // What happens when we trigger the "New Folder" Action.
        let contextual_menu_new_folder = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {

                // Create the "New Folder" dialog and wait for a new name (or a cancellation).
                if let Some(new_folder_name) = AppUI::new_folder_dialog(&app_ui) {
                    info!("Triggering `New Folder` By Slot");

                    // Get the currently selected paths, and only continue if there is only one.
                    let selected_paths = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection();
                    if selected_paths.len() == 1 {

                        // Add the folder's name to the list.
                        let mut complete_path = selected_paths[0].to_owned();

                        if !complete_path.is_empty() && !complete_path.ends_with('/') {
                            complete_path.push('/');
                        }
                        complete_path.push_str(&new_folder_name);

                        // Check if the folder exists.
                        let receiver = CENTRAL_COMMAND.send_background(Command::FolderExists(complete_path.to_owned()));
                        let response = CentralCommand::recv(&receiver);
                        let folder_exists = if let Response::Bool(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

                        // If the folder already exists, return an error.
                        if folder_exists { return show_dialog(app_ui.main_window(), "That folder already exists in the current path.", false)}
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::Folder(complete_path); 1]), DataSource::PackFile);
                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // What happens when we trigger the "Create Text PackedFile" Action.
        let contextual_menu_new_queek_packed_file = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `New Queek File` By Slot");
            AppUI::new_queek_packed_file(&app_ui, &pack_file_contents_ui);
        }));

        // What happens when we trigger the "Open Decoder" Action.
        let contextual_menu_open_decoder = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
            info!("Triggering `Open Decoder` By Slot");
            let selected_items = pack_file_contents_ui.packfile_contents_tree_view().get_item_types_from_selection(true);
            if selected_items.len() == 1 {
                AppUI::open_special_view(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, SpecialView::Decoder(selected_items[0].path_raw().to_string()))
            }
        }));

        // What happens when we trigger the "Open Dependency Table" Action.
        let contextual_menu_open_dependency_manager = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
            info!("Triggering `Open Dependency Manager` By Slot");
            AppUI::open_special_view(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, SpecialView::PackDependencies);
        }));

        // What happens when we trigger the "Open Containing Folder" Action.
        let contextual_menu_open_containing_folder = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui => move |_| {
            let receiver = CENTRAL_COMMAND.send_background(Command::OpenContainingFolder);
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Success => {}
                Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }));

        // What happens when we trigger the "Open In External Program" Action.
        let contextual_menu_open_in_external_program = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
            info!("Triggering `Open In External Program` By Slot");
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, None, false, true, DataSource::PackFile);
        }));

        let contextual_menu_open_packfile_settings = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
            info!("Triggering `Open PackFile Settings` By Slot");
            AppUI::open_special_view(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, SpecialView::PackSettings);
        }));

        // What happens when we trigger the "Open Notes" Action.
        let contextual_menu_open_notes = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |_| {
            info!("Triggering `Open Notes` By Slot");
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, Some(RESERVED_NAME_NOTES.to_owned()), false, false, DataSource::PackFile);
        }));

        // What happens when we trigger the "Merge Tables" action in the Contextual Menu.
        let contextual_menu_tables_merge_tables = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `Merge Tables` By Slot");

            // Get the currently selected paths, and get how many we have of each type.
            let selected_paths = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection();

            // First, we check if we're merging locs, as it's far simpler.
            let mut loc_pass = true;
            for path in &selected_paths {
                if !path.to_lowercase().ends_with(".loc") {
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
                let path_split = path.split('/').collect::<Vec<_>>();
                if path_split.len() == 3 {
                    if path_split[0].to_lowercase() == "db" {
                        if db_folder.is_empty() {
                            db_folder = path_split[1].to_owned();
                        }

                        if path_split[1] != db_folder {
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
                if let Some((mut name, delete_source_files)) = AppUI::merge_tables_dialog(&app_ui) {

                    // If it's a loc file and the name doesn't end in a ".loc" termination, call it ".loc".
                    if loc_pass && !name.to_lowercase().ends_with(".loc") {
                        name.push_str(".loc");
                    }

                    // Close the open and selected files.
                    let mut paths_to_close = vec![];
                    {
                        let open_packedfiles = UI_STATE.set_open_packedfiles();
                        for path in open_packedfiles.iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| x.path_read())  {
                            if selected_paths.contains(&path) {
                                paths_to_close.push(ContainerPath::File(path.to_owned()));
                            }
                        }
                    }

                    for path in &paths_to_close {
                        if let Err(error) = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path.path_raw(), DataSource::PackFile, true) {
                            return show_dialog(app_ui.main_window(), error, false);
                        }
                    }

                    let mut path_to_add = selected_paths[0].rsplitn(2, '/').collect::<Vec<_>>()[1].to_string();
                    path_to_add.push('/');
                    path_to_add.push_str(&name);

                    let receiver = CENTRAL_COMMAND.send_background(Command::MergeFiles(paths_to_close.to_vec(), path_to_add, delete_source_files));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::String(path_to_add) => {

                            // If we want to delete the sources, do it now. Oh, and close them manually first, or the autocleanup will try to save them and fail miserably.
                            if delete_source_files {
                                let items_to_remove = selected_paths.iter().map(|x| ContainerPath::File(x.to_owned())).collect();
                                selected_paths.iter().for_each(|x| { let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, x, DataSource::PackFile, false); });
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items_to_remove), DataSource::PackFile);
                            }

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(path_to_add); 1]), DataSource::PackFile);

                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }
                }
            }

            else {
                show_dialog(app_ui.main_window(), "The files you selected are not all LOCs, neither DB Tables of the same type and version.", false);
            }
        }));

        // What happens when we trigger the "Update Table" action in the Contextual Menu.
        let contextual_menu_tables_update_table = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `Update Table` By Slot");

            let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &selected_items[0] } else { return };
            match item_type {
                ContainerPath::File(path) => {

                    // First, if the PackedFile is open, save it.
                    let close_path = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).any(|file_view| {
                        file_view.path_copy() == *path
                    });

                    if close_path {
                        if let Err(error) = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, true) {
                            return show_dialog(app_ui.main_window(), error, false);
                        }
                    }

                    let receiver = CENTRAL_COMMAND.send_background(Command::UpdateTable(item_type.clone()));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::I32I32(old_version, new_version) => {
                            let message = tre("update_table_success", &[&old_version.to_string(), &new_version.to_string()]);
                            show_dialog(app_ui.main_window(), message, true);

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Modify(vec![item_type.clone(); 1]), DataSource::PackFile);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![item_type.clone(); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }
                }
                _ => unimplemented!()
            }
        }));

        // What happens when we trigger the "Update Table" action in the Contextual Menu.
        let contextual_menu_generate_missing_loc_data = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `Generate Loc Data` By Slot");

            let receiver = CENTRAL_COMMAND.send_background(Command::GenerateMissingLocData);
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::OptionContainerPath(path_to_add) => {
                    if let Some(path_to_add) = path_to_add {
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![path_to_add; 1]), DataSource::PackFile);
                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    }
                }

                Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }));

        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move || {
                pack_file_contents_ui.packfile_contents_tree_view.expand_all();
            }
        ));
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move || {
                pack_file_contents_ui.packfile_contents_tree_view.collapse_all();
            }
        ));

        let packfile_contents_resize = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, move || {

            // Not yet working.
            if setting_bool("packfile_treeview_resize_to_fit") {
                //pack_file_contents_ui.packfile_contents_dock_widget.widget().adjust_size();
                //pack_file_contents_ui.packfile_contents_tree_view.header().resize_sections(ResizeMode::ResizeToContents);
            }
        });

        // And here... we return all the slots.
		Self {
            move_items,

            open_packedfile_preview,
            open_packedfile_full,

            filter_trigger,
            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,
            filter_check_regex,

            contextual_menu,
            contextual_menu_enabler,

            contextual_menu_add_file,
            contextual_menu_add_folder,
            contextual_menu_add_from_packfile,
            contextual_menu_delete,
            contextual_menu_extract,
            contextual_menu_rename,
            contextual_menu_copy_path,

            contextual_menu_new_packed_file_anim_pack,
            contextual_menu_new_packed_file_db,
            contextual_menu_new_packed_file_loc,
            contextual_menu_new_packed_file_portrait_settings,
            contextual_menu_new_packed_file_text,
            contextual_menu_new_folder,
            contextual_menu_new_queek_packed_file,

            contextual_menu_open_decoder,
            contextual_menu_open_dependency_manager,
            contextual_menu_open_containing_folder,
            contextual_menu_open_in_external_program,
            contextual_menu_open_packfile_settings,
            contextual_menu_open_notes,

            contextual_menu_tables_merge_tables,
            contextual_menu_tables_update_table,
            contextual_menu_generate_missing_loc_data,

            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,

            packfile_contents_resize
		}
	}
}
