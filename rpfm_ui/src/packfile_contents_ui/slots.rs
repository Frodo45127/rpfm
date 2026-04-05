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
Module with all the code related to the main `PackFileContentsSlots`.
!*/

use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as FileDialogOption}};
use qt_widgets::QListView;
use qt_widgets::SlotOfQPoint;
use qt_widgets::QTreeView;

use qt_gui::{QCursor, SlotOfQAction};
use qt_gui::QGuiApplication;

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::{SlotNoArgs, SlotOfBool, SlotOfQModelIndexInt, SlotOfQString};

use itertools::Itertools;

use std::collections::HashSet;
use std::fs::{copy, remove_dir_all, remove_file, DirBuilder};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_ipc::{MYMOD_BASE_PATH, helpers::DataSource};

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{ContainerPath, FileType, pack::*};
use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::games::supported_games::*;
use rpfm_log::*;
use rpfm_lib::utils::*;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::communications::{Command, Response, send_ipc_command, send_ipc_command_result, send_ipc_command_result_async, send_ipc_command_async};
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::SpecialView;
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::{is_schema_loaded, settings_bool, settings_path_buf};
use crate::GAME_SELECTED;
use crate::UI_STATE;
use crate::ui_state::OperationalMode;
use crate::pack_tree::{BuildData, new_pack_file_tooltip};
use crate::utils::{check_regex, log_to_status_bar, qtr, show_dialog, show_message_info, tr, tre};

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
    pub contextual_menu_copy_to_pack_about_to_show: QBox<SlotNoArgs>,
    pub contextual_menu_copy_to_pack: QBox<SlotOfQAction>,
    pub contextual_menu_delete: QBox<SlotOfBool>,
    pub contextual_menu_extract: QBox<SlotOfBool>,
    pub contextual_menu_rename: QBox<SlotOfBool>,
    pub contextual_menu_copy_path: QBox<SlotOfBool>,
    pub contextual_menu_copy: QBox<SlotOfBool>,
    pub contextual_menu_cut: QBox<SlotOfBool>,
    pub contextual_menu_paste: QBox<SlotOfBool>,
    pub contextual_menu_duplicate: QBox<SlotOfBool>,

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

    pub context_menu_install: QBox<SlotOfBool>,
    pub context_menu_uninstall: QBox<SlotOfBool>,
    pub context_menu_change_packfile_type: QBox<SlotOfBool>,
    pub context_menu_change_compression_format: QBox<SlotOfBool>,
    pub context_menu_index_includes_timestamp: QBox<SlotOfBool>,
    pub context_menu_optimize_packfile: QBox<SlotOfBool>,
    pub context_menu_patch_siege_ai: QBox<SlotOfBool>,
    pub context_menu_live_export: QBox<SlotOfBool>,
    pub context_menu_pack_map: QBox<SlotOfBool>,
    pub context_menu_rescue_packfile: QBox<SlotOfBool>,
    pub context_menu_build_starpos: QBox<SlotOfBool>,
    pub context_menu_update_anim_ids: QBox<SlotOfBool>,

    pub context_menu_mymod_import: QBox<SlotOfBool>,
    pub context_menu_mymod_export: QBox<SlotOfBool>,
    pub context_menu_mymod_delete: QBox<SlotOfBool>,
    pub context_menu_mymod_open_folder: QBox<SlotOfBool>,

    pub packfile_contents_tree_view_expand_all: QBox<SlotNoArgs>,
    pub packfile_contents_tree_view_collapse_all: QBox<SlotNoArgs>,
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

                let dest_index_visual = pack_file_contents_ui.packfile_contents_tree_model_filter().index_3a(dest_row, 0, dest_parent);
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
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                match send_ipc_command_result(Command::RenamePackedFiles(pack_key.clone(), renaming_data_background.to_vec()), response_extractor!(Response::VecContainerPathContainerPath)) {
                    Ok(renamed_items) => {
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

                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Move(renamed_items, folders_to_move), DataSource::PackFile, &pack_key);

                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    },
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
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
                check_regex(&string.to_std_string(), pack_file_contents_ui.filter_line_edit.static_upcast(), true);
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
                let (contents, files, folders, _, multi_pack) = <QPtr<QTreeView> as PackTree>::get_combination_from_main_treeview_selection(&pack_file_contents_ui);

                // Disable all actions if we're selecting stuff from different packs.
                if multi_pack {
                    pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                    pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                    pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(false);
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
                    pack_file_contents_ui.context_menu_copy.set_enabled(true);
                    pack_file_contents_ui.context_menu_cut.set_enabled(true);
                    pack_file_contents_ui.context_menu_paste.set_enabled(false);
                    pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_decoder.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_dependency_manager.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_containing_folder.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_packfile_settings.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_with_external_program.set_enabled(false);
                    pack_file_contents_ui.context_menu_open_notes.set_enabled(false);
                    pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                } else {
                    match contents {

                        // Only one or more files selected.
                        1 => {

                            // These options are valid for 1 or more files.
                            pack_file_contents_ui.context_menu_add_file.set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.set_enabled(false);
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_folder.set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_anim_pack.set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_loc.set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_portrait_settings.set_enabled(false);
                            pack_file_contents_ui.context_menu_new_packed_file_text.set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.set_enabled(true);
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
                            pack_file_contents_ui.context_menu_new_packed_file_db.set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.set_enabled(true);
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(false);
                            pack_file_contents_ui.context_menu_cut.set_enabled(false);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(true);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(true);
                            pack_file_contents_ui.context_menu_cut.set_enabled(true);
                            pack_file_contents_ui.context_menu_paste.set_enabled(true);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy_to_pack.menu_action().set_enabled(false);
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
                            pack_file_contents_ui.context_menu_copy.set_enabled(false);
                            pack_file_contents_ui.context_menu_cut.set_enabled(false);
                            pack_file_contents_ui.context_menu_paste.set_enabled(false);
                            pack_file_contents_ui.context_menu_duplicate.set_enabled(false);
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
                }

                // Ask the other thread if there is a Dependency Database and a Schema loaded.
                let is_there_a_dependency_database = send_ipc_command(Command::IsThereADependencyDatabase(false), response_extractor!(Response::Bool));

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || !is_schema_loaded() {
                    pack_file_contents_ui.context_menu_update_table.set_enabled(false);
                }

                // Pack-level actions: only visible when exactly one pack root is selected.
                if contents == 4 {
                    pack_file_contents_ui.context_menu_install.set_visible(true);
                    pack_file_contents_ui.context_menu_uninstall.set_visible(true);
                    pack_file_contents_ui.context_menu_packfile_type_menu.menu_action().set_visible(true);
                    pack_file_contents_ui.context_menu_compression_menu.menu_action().set_visible(true);
                    pack_file_contents_ui.context_menu_optimize_packfile.set_visible(true);
                    pack_file_contents_ui.context_menu_rescue_packfile.set_visible(true);
                    pack_file_contents_ui.context_menu_build_starpos.set_visible(true);

                    // Game-specific actions.
                    let game_key = GAME_SELECTED.read().unwrap().key().to_owned();
                    pack_file_contents_ui.context_menu_patch_siege_ai.set_visible(game_key == KEY_WARHAMMER_2 || game_key == KEY_WARHAMMER);
                    pack_file_contents_ui.context_menu_live_export.set_visible(game_key == KEY_WARHAMMER_3);
                    pack_file_contents_ui.context_menu_pack_map.set_visible(game_key == KEY_WARHAMMER_3);
                    pack_file_contents_ui.context_menu_update_anim_ids.set_visible(game_key == KEY_WARHAMMER_3);

                    // Update pack type/compression/flags to reflect the selected pack's current state.
                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

                    // Query the pack's operational mode for MyMod action visibility.
                    let mode = send_ipc_command(Command::GetPackOperationalMode(pack_key.clone()), response_extractor!(Response::OperationalMode));
                    let is_mymod = matches!(mode, OperationalMode::MyMod(..));
                    pack_file_contents_ui.context_menu_mymod_import.set_visible(is_mymod);
                    pack_file_contents_ui.context_menu_mymod_export.set_visible(is_mymod);
                    pack_file_contents_ui.context_menu_mymod_delete.set_visible(is_mymod);
                    pack_file_contents_ui.context_menu_mymod_open_folder.set_visible(is_mymod);

                    let (ui_data, _) = send_ipc_command(Command::GetPackFileDataForTreeView(pack_key), response_extractor!(Response::ContainerInfoVecRFileInfo));
                    pack_file_contents_ui.context_menu_packfile_type_group.block_signals(true);
                    match ui_data.pfh_file_type() {
                        PFHFileType::Boot => pack_file_contents_ui.context_menu_packfile_type_boot.set_checked(true),
                        PFHFileType::Release => pack_file_contents_ui.context_menu_packfile_type_release.set_checked(true),
                        PFHFileType::Patch => pack_file_contents_ui.context_menu_packfile_type_patch.set_checked(true),
                        PFHFileType::Mod => pack_file_contents_ui.context_menu_packfile_type_mod.set_checked(true),
                        PFHFileType::Movie => pack_file_contents_ui.context_menu_packfile_type_movie.set_checked(true),
                    }
                    pack_file_contents_ui.context_menu_packfile_type_group.block_signals(false);

                    pack_file_contents_ui.context_menu_compression_group.block_signals(true);
                    match ui_data.compress() {
                        CompressionFormat::None => pack_file_contents_ui.context_menu_compression_none.set_checked(true),
                        CompressionFormat::Lzma1 => pack_file_contents_ui.context_menu_compression_lzma1.set_checked(true),
                        CompressionFormat::Lz4 => pack_file_contents_ui.context_menu_compression_lz4.set_checked(true),
                        CompressionFormat::Zstd => pack_file_contents_ui.context_menu_compression_zstd.set_checked(true),
                    }
                    pack_file_contents_ui.context_menu_compression_group.block_signals(false);

                    pack_file_contents_ui.context_menu_data_is_encrypted.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_ENCRYPTED_DATA));
                    pack_file_contents_ui.context_menu_index_includes_timestamp.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS));
                    pack_file_contents_ui.context_menu_index_is_encrypted.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_ENCRYPTED_INDEX));
                    pack_file_contents_ui.context_menu_header_is_extended.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_EXTENDED_HEADER));
                } else {
                    pack_file_contents_ui.context_menu_install.set_visible(false);
                    pack_file_contents_ui.context_menu_uninstall.set_visible(false);
                    pack_file_contents_ui.context_menu_packfile_type_menu.menu_action().set_visible(false);
                    pack_file_contents_ui.context_menu_compression_menu.menu_action().set_visible(false);
                    pack_file_contents_ui.context_menu_optimize_packfile.set_visible(false);
                    pack_file_contents_ui.context_menu_rescue_packfile.set_visible(false);
                    pack_file_contents_ui.context_menu_build_starpos.set_visible(false);
                    pack_file_contents_ui.context_menu_patch_siege_ai.set_visible(false);
                    pack_file_contents_ui.context_menu_live_export.set_visible(false);
                    pack_file_contents_ui.context_menu_pack_map.set_visible(false);
                    pack_file_contents_ui.context_menu_update_anim_ids.set_visible(false);
                    pack_file_contents_ui.context_menu_mymod_import.set_visible(false);
                    pack_file_contents_ui.context_menu_mymod_export.set_visible(false);
                    pack_file_contents_ui.context_menu_mymod_delete.set_visible(false);
                    pack_file_contents_ui.context_menu_mymod_open_folder.set_visible(false);
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

                // Query the selected pack's operational mode to set the initial directory.
                let selected_pack_key_for_mode = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let pack_mode = send_ipc_command(Command::GetPackOperationalMode(selected_pack_key_for_mode), response_extractor!(Response::OperationalMode));

                match pack_mode {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = settings_path_buf(MYMOD_BASE_PATH);
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

                // Wonky workaround to allow multiple folder selection.
                if settings_bool("enable_multifolder_filepicker") {
                    file_dialog.set_options(QFlags::from(FileDialogOption::DontUseNativeDialog.to_int() | file_dialog.options().to_int()));

                    if let Ok(list_view) = file_dialog.find_child::<QListView>("listView") {
                        list_view.set_selection_mode(qt_widgets::q_abstract_item_view::SelectionMode::MultiSelection);
                    }

                    if let Ok(tree_view) = file_dialog.find_child::<QTreeView>("treeView") {
                        tree_view.set_selection_mode(qt_widgets::q_abstract_item_view::SelectionMode::MultiSelection);
                    }
                }

                // Query the selected pack's operational mode to set the initial directory.
                let selected_pack_key_for_mode = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let pack_mode = send_ipc_command(Command::GetPackOperationalMode(selected_pack_key_for_mode), response_extractor!(Response::OperationalMode));

                match pack_mode {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = settings_path_buf(MYMOD_BASE_PATH);
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
                                for index in 0..paths_qt.size() {
                                    folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string()));
                                }

                                // Make sure all folders are part of the same subfolder. The multifolder selector can accidentally add folders with different base paths,
                                // and we need to avoid that.
                                if let Some(base_path) = folder_paths.first() {
                                    let mut base_path = base_path.to_path_buf();
                                    base_path.pop();

                                    for folder_path in &folder_paths {
                                        let mut second_path = folder_path.to_path_buf();
                                        second_path.pop();

                                        if base_path != second_path {
                                            return show_dialog(app_ui.main_window(), "Error: adding multiple folders from different parent folders is not supported.".to_string(), false);
                                        }
                                    }
                                }

                                // Get the Paths of the files inside the folders we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                for path in &folder_paths {
                                    paths.append(&mut files_from_subdir(path, true).unwrap());
                                }

                                // Check to ensure we actually have a path, as you may try to add empty folders.
                                if let Some(path) = paths.first() {

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
                                    else if let Some(selection) = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection().first() {
                                        let destination_paths = (0..folder_paths.len()).map(|_| ContainerPath::Folder(selection.to_string())).collect::<Vec<_>>();

                                        app_ui.toggle_main_window(false);
                                        PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &folder_paths, &destination_paths, None);
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

                            // Make sure all folders are part of the same subfolder. The multifolder selector can accidentally add folders with different base paths,
                            // and we need to avoid that.
                            if let Some(base_path) = folder_paths.first() {
                                let mut base_path = base_path.to_path_buf();
                                base_path.pop();

                                for folder_path in &folder_paths {
                                    let mut second_path = folder_path.to_path_buf();
                                    second_path.pop();

                                    if base_path != second_path {
                                        return show_dialog(app_ui.main_window(), "Error: adding multiple folders from different parent folders is not supported.".to_string(), false);
                                    }
                                }
                            }

                            // Get the Paths of the files inside the folders we want to add.
                            if let Some(selection) = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection().first() {
                                let destination_paths = (0..folder_paths.len()).map(|_| ContainerPath::Folder(selection.to_string())).collect::<Vec<_>>();

                                app_ui.toggle_main_window(false);
                                PackFileContentsUI::add_files(&app_ui, &pack_file_contents_ui, &folder_paths, &destination_paths, None);
                                app_ui.toggle_main_window(true);
                            }
                        }
                    }
                }
            }
        ));

        // What happens when the "Copy To Pack" submenu is about to show.
        // We populate it dynamically with the list of other open packs.
        let contextual_menu_copy_to_pack_about_to_show = SlotNoArgs::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            pack_file_contents_ui => move || {
                info!("Triggering `Copy To Pack About To Show` By Slot");

                let menu = &pack_file_contents_ui.context_menu_copy_to_pack;
                menu.clear();

                // Get the source pack key from current selection.
                let source_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

                // Query all open packs from the server.
                let pack_list = send_ipc_command(Command::ListOpenPacks, response_extractor!(Response::VecStringContainerInfo));
                for (pack_key, pack_info) in &pack_list {
                    // Skip the source pack itself.
                    if *pack_key == source_key {
                        continue;
                    }

                    // Use the file name from the pack path as the display name, or fallback to the key.
                    let display_name = std::path::Path::new(pack_info.file_path())
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| pack_key.clone());

                    let action = menu.add_action_q_string(&QString::from_std_str(&display_name));
                    action.set_data(&qt_core::QVariant::from_q_string(&QString::from_std_str(pack_key)));
                }

                // If the menu is empty, add a disabled placeholder.
                if menu.is_empty() {
                    let action = menu.add_action_q_string(&qtr("context_menu_copy_to_pack_no_packs"));
                    action.set_enabled(false);
                }
            }
        ));

        // What happens when a pack is selected from the "Copy To Pack" submenu.
        let contextual_menu_copy_to_pack = SlotOfQAction::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |action| {
                info!("Triggering `Copy To Pack` By Slot");

                let target_key = action.data().to_string().to_std_string();
                if target_key.is_empty() {
                    return;
                }

                let source_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if selected_items.is_empty() {
                    return;
                }

                app_ui.toggle_main_window(false);
                match send_ipc_command_result(Command::AddPackedFilesFromPackFile(target_key.clone(), source_key, selected_items), response_extractor!(Response::VecContainerPath)) {
                    Ok(paths_ok) => {

                        // If any of the files were already open in the target pack, reload their views.
                        for path in &paths_ok {
                            if let ContainerPath::File(path) = path {
                                let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                                if let Some(file_view) = open_packedfiles.iter_mut().find(|x| *x.path_read() == *path && x.data_source() == DataSource::PackFile) {
                                    if file_view.reload(path, &pack_file_contents_ui).is_err() {
                                        let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, false);
                                    }
                                }
                            }
                        }

                        // Update the target pack's tree view.
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile, &target_key);
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile, &target_key);
                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    },
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }

                app_ui.toggle_main_window(true);
                PackFileContentsUI::start_delayed_updates_timer(&pack_file_contents_ui);
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let contextual_menu_delete = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if AppUI::are_you_sure_edition(&app_ui, "are_you_sure_delete") {
                    info!("Triggering `Delete` By Slot");

                    let mut selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);

                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    let items = send_ipc_command(Command::DeletePackedFiles(pack_key.clone(), selected_items.clone()), response_extractor!(Response::VecContainerPath));

                    selected_items.extend_from_slice(&items);
                    let items = ContainerPath::dedup(&selected_items);
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items.to_vec(), settings_bool("delete_empty_folders_on_delete")), DataSource::PackFile, &pack_key);
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(items.to_vec()), DataSource::PackFile, &pack_key);
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
                            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                            match send_ipc_command_result(Command::RenamePackedFiles(pack_key.clone(), renaming_data_background.to_vec()), response_extractor!(Response::VecContainerPathContainerPath)) {
                                Ok(renamed_items) => {
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

                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Move(renamed_items, folders_to_move), DataSource::PackFile, &pack_key);

                                    UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                                },
                                Err(error) => show_dialog(app_ui.main_window(), error, false),
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

        // What happens when we trigger the "Copy" action in the Contextual Menu.
        let contextual_menu_copy = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Copy` By Slot");

                let paths_by_pack = pack_file_contents_ui.selected_items_grouped_by_pack_key();
                if paths_by_pack.is_empty() {
                    return;
                }

                match send_ipc_command_result(Command::CopyPackedFiles(paths_by_pack), response_extractor!()) {
                    Ok(()) => log_to_status_bar(&tr("copy_success")),
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

        // What happens when we trigger the "Cut" action in the Contextual Menu.
        let contextual_menu_cut = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Cut` By Slot");

                let paths_by_pack = pack_file_contents_ui.selected_items_grouped_by_pack_key();
                if paths_by_pack.is_empty() {
                    return;
                }

                match send_ipc_command_result(Command::CutPackedFiles(paths_by_pack), response_extractor!()) {
                    Ok(()) => log_to_status_bar(&tr("cut_success")),
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

        // What happens when we trigger the "Paste" action in the Contextual Menu.
        let contextual_menu_paste = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Paste` By Slot");

                // Get the destination path from the selection. Do nothing if nothing is selected.
                let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if selected_items.is_empty() {
                    return;
                }

                let destination_path = match &selected_items[0] {
                    ContainerPath::Folder(path) => path.clone(),
                    ContainerPath::File(path) => {
                        // It's a file, so use its parent folder.
                        match path.rfind('/') {
                            Some(pos) => path[..pos].to_string(),
                            None => String::new(),
                        }
                    }
                };

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                match send_ipc_command_result(Command::PastePackedFiles(pack_key.clone(), destination_path), response_extractor!(Response::VecContainerPathBTreeMapStringVecContainerPath, added_paths, deleted_by_pack)) {
                    Ok((added_paths, deleted_by_pack)) => {
                        if !added_paths.is_empty() {
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(added_paths.to_vec()), DataSource::PackFile, &pack_key);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        // If it was a cut operation, remove deleted items from each source pack's tree.
                        for (source_pack_key, deleted_paths) in &deleted_by_pack {
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(deleted_paths.to_vec(), false), DataSource::PackFile, source_pack_key);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(deleted_paths.to_vec()), DataSource::PackFile, source_pack_key);

                            // Remove all the deleted PackedFiles from the cache, but only for views from the source pack.
                            for item in deleted_paths {
                                match item {
                                    ContainerPath::File(path) => { let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, false); },
                                    ContainerPath::Folder(path) => {
                                        let mut paths_to_remove = vec![];
                                        {
                                            let open_packedfiles = UI_STATE.set_open_packedfiles();
                                            for view in open_packedfiles.iter().filter(|x| x.data_source() == DataSource::PackFile && x.pack_key_copy() == *source_pack_key) {
                                                let packed_file_path = view.path_read();
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
                        }
                    },
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

        // What happens when we trigger the "Duplicate" action in the Contextual Menu.
        let contextual_menu_duplicate = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Duplicate` By Slot");

                let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if selected_items.is_empty() {
                    return;
                }

                // Only duplicate files, not folders.
                let file_items: Vec<ContainerPath> = selected_items.into_iter()
                    .filter(|item| matches!(item, ContainerPath::File(_)))
                    .collect();

                if file_items.is_empty() {
                    return;
                }

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                match send_ipc_command_result(Command::DuplicatePackedFiles(pack_key.clone(), file_items), response_extractor!(Response::VecContainerPath)) {
                    Ok(added_paths) => {
                        if !added_paths.is_empty() {
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(added_paths.to_vec()), DataSource::PackFile, &pack_key);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }
                    },
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

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
                        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                        let folder_exists = send_ipc_command(Command::FolderExists(pack_key.clone(), complete_path.to_owned()), response_extractor!(Response::Bool));

                        // If the folder already exists, return an error.
                        if folder_exists { return show_dialog(app_ui.main_window(), "That folder already exists in the current path.", false)}
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::Folder(complete_path); 1]), DataSource::PackFile, &pack_key);
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
            app_ui,
            pack_file_contents_ui => move |_| {
            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            if let Err(error) = send_ipc_command_result(Command::OpenContainingFolder(pack_key), response_extractor!()) {
                show_dialog(app_ui.main_window(), error, false);
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
                if let Some((mut name, delete_source_files)) = AppUI::merge_tables_dialog(&app_ui, &pack_file_contents_ui) {

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

                    let selected_paths_cont = selected_paths.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<_>>();
                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    match send_ipc_command_result(Command::MergeFiles(pack_key.clone(), selected_paths_cont.to_vec(), path_to_add, delete_source_files), response_extractor!(Response::String)) {
                        Ok(path_to_add) => {

                            // If we want to delete the sources, do it now. Oh, and close them manually first, or the autocleanup will try to save them and fail miserably.
                            if delete_source_files {
                                selected_paths.iter().for_each(|x| { let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, x, DataSource::PackFile, false); });
                                let paths_to_delete = selected_paths_cont.iter().filter(|path| path.path_raw() != path_to_add).cloned().collect::<Vec<_>>();
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(paths_to_delete, true), DataSource::PackFile, &pack_key);
                            }

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(path_to_add); 1]), DataSource::PackFile, &pack_key);

                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Err(error) => show_dialog(app_ui.main_window(), error, false),
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

                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    match send_ipc_command_result(Command::UpdateTable(pack_key.clone(), item_type.clone()), response_extractor!(Response::I32I32VecStringVecString, old_version, new_version, fields_deleted, fields_added)) {
                        Ok((old_version, new_version, fields_deleted, fields_added)) => {
                            let mut message = tre("update_table_success", &[&old_version.to_string(), &new_version.to_string()]);
                            if !fields_deleted.is_empty() {
                                message.push_str(&tre("update_table_success_files_deleted", &[&fields_deleted.iter().map(|x| format!("<li>{x}</li>")).join("")]));
                            }

                            if !fields_added.is_empty() {
                                message.push_str(&tre("update_table_success_files_added", &[&fields_added.iter().map(|x| format!("<li>{x}</li>")).join("")]));
                            }

                            show_dialog(app_ui.main_window(), message, true);

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Modify(vec![item_type.clone(); 1]), DataSource::PackFile, &pack_key);
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![item_type.clone(); 1]), DataSource::PackFile, &pack_key);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Err(error) => show_dialog(app_ui.main_window(), error, false),
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

            // Make sure the backend has all the data updated.
            let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);

            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            match send_ipc_command_result(Command::GenerateMissingLocData(pack_key.clone()), response_extractor!(Response::VecContainerPath)) {
                Ok(paths_to_add) => {
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths_to_add.to_vec()), DataSource::PackFile, &pack_key);

                    // Reload correctly the UI.
                    for path in &paths_to_add {
                        UI_STATE.set_open_packedfiles()
                            .iter_mut()
                            .filter(|view| view.data_source() == DataSource::PackFile && view.path_copy() == path.path_raw())
                            .for_each(|view| { let _ = view.reload(&view.path_copy(), &pack_file_contents_ui); });
                    }

                    UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                }

                Err(error) => show_dialog(app_ui.main_window(), error, false),
            }
        }));

        //-----------------------------------------------------------------------//
        // Pack-level context menu slots.
        //-----------------------------------------------------------------------//

        let context_menu_install = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Install` By Slot");

                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, false, false) {
                    return show_dialog(app_ui.main_window(), error, false);
                }

                let pack_key = match pack_file_contents_ui.pack_key_from_selection_or_first() {
                    Some(key) => key,
                    None => return show_dialog(app_ui.main_window(), "No pack is open.", false),
                };
                let pack_path = match send_ipc_command_result(Command::GetPackFilePath(pack_key), response_extractor!(Response::PathBuf)) {
                    Ok(path) => path,
                    Err(error) => return show_dialog(app_ui.main_window(), error, false),
                };
                let mut pack_image_path = pack_path.clone();
                pack_image_path.set_extension("png");

                if !pack_path.is_file() {
                    return show_dialog(app_ui.main_window(), "Pack to install not found on disk.", false);
                }

                if let Ok(mut game_local_mods_path) = GAME_SELECTED.read().unwrap().local_mods_path(&settings_path_buf(GAME_SELECTED.read().unwrap().key())) {
                    if !game_local_mods_path.is_dir() {
                        return show_dialog(app_ui.main_window(), "Game Path not configured. Go to <i>'PackFile/Settings'</i> and configure it.", false);
                    }

                    if pack_path.starts_with(&game_local_mods_path) {
                        return show_dialog(app_ui.main_window(), "This Pack is already being edited from the data folder of the game. You cannot install/uninstall it.", false);
                    }

                    if let Some(ref mod_name) = pack_path.file_name() {
                        game_local_mods_path.push(mod_name);

                        let ca_paths = match GAME_SELECTED.read().unwrap().ca_packs_paths(&settings_path_buf(GAME_SELECTED.read().unwrap().key())) {
                            Ok(paths) => paths,
                            Err(_) => return show_dialog(app_ui.main_window(), "You can't do that to a CA PackFile, you monster!", false),
                        };

                        if ca_paths.contains(&game_local_mods_path) {
                            return show_dialog(app_ui.main_window(), "You can't do that to a CA PackFile, you monster!", false);
                        }

                        if copy(&pack_path, &game_local_mods_path).is_err() {
                            return show_dialog(app_ui.main_window(), "Error installing a Pack. Make sure the game/assembly kit is close and try again.", false);
                        }

                        game_local_mods_path.pop();
                        game_local_mods_path.push(pack_image_path.file_name().unwrap());
                        if pack_image_path.is_file() && copy(&pack_image_path, &game_local_mods_path).is_err() {
                            return show_dialog(app_ui.main_window(), "Error installing the thumbnail of a Pack. Make sure the game/assembly kit is close and try again.", false);
                        }

                        log_to_status_bar(&tr("install_success"));
                    }
                }
            }
        ));

        let context_menu_uninstall = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Uninstall` By Slot");

                let pack_key = match pack_file_contents_ui.pack_key_from_selection_or_first() {
                    Some(key) => key,
                    None => return show_dialog(app_ui.main_window(), "No pack is open.", false),
                };
                let pack_path = match send_ipc_command_result(Command::GetPackFilePath(pack_key), response_extractor!(Response::PathBuf)) {
                    Ok(path) => path,
                    Err(error) => return show_dialog(app_ui.main_window(), error, false),
                };

                if !pack_path.is_file() {
                    return show_dialog(app_ui.main_window(), "Pack to install not found on disk.", false);
                }

                if let Ok(game_local_mods_path) = GAME_SELECTED.read().unwrap().local_mods_path(&settings_path_buf(GAME_SELECTED.read().unwrap().key())) {
                    if !game_local_mods_path.is_dir() {
                        return show_dialog(app_ui.main_window(), "Game Path not configured. Go to <i>'PackFile/Settings'</i> and configure it.", false);
                    }

                    if pack_path.starts_with(&game_local_mods_path) {
                        return show_dialog(app_ui.main_window(), "This Pack is already being edited from the data folder of the game. You cannot install/uninstall it.", false);
                    }

                    if let Some(ref mod_name) = pack_path.file_name() {
                        let mut data_pack_path = game_local_mods_path.to_path_buf();
                        data_pack_path.push(mod_name);

                        let mut data_image_path = data_pack_path.clone();
                        data_image_path.set_extension("png");

                        let ca_paths = match GAME_SELECTED.read().unwrap().ca_packs_paths(&settings_path_buf(GAME_SELECTED.read().unwrap().key())) {
                            Ok(paths) => paths,
                            Err(_) => return show_dialog(app_ui.main_window(), "You can't do that to a CA PackFile, you monster!", false),
                        };

                        if ca_paths.contains(&data_pack_path) {
                            return show_dialog(app_ui.main_window(), "You can't do that to a CA PackFile, you monster!", false);
                        }

                        if remove_file(&data_pack_path).is_err() {
                            return show_dialog(app_ui.main_window(), "Error uninstalling the Pack from the game's folder. Make sure nothing else is using it and try again.", false);
                        }

                        let mut source_image_path = pack_path.to_path_buf();
                        source_image_path.set_extension("png");
                        if source_image_path.is_file() {
                            if remove_file(&data_image_path).is_err() {
                                return show_dialog(app_ui.main_window(), "Error uninstalling the thumbnail of the Pack from the game's folder. Make sure nothing else is using it and try again.", false);
                            }
                        }

                        log_to_status_bar(&tr("uninstall_success"));
                    }
                }
            }
        ));

        let context_menu_change_packfile_type = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Change PackFile Type` By Slot");

                let packfile_type = match &*(pack_file_contents_ui.context_menu_packfile_type_group.checked_action().text().remove_q_string(&QString::from_std_str("&")).to_std_string()) {
                    "Boot" => PFHFileType::Boot,
                    "Release" => PFHFileType::Release,
                    "Patch" => PFHFileType::Patch,
                    "Mod" => PFHFileType::Mod,
                    "Movie" => PFHFileType::Movie,
                    _ => unreachable!("change_pack_type with string {}", pack_file_contents_ui.context_menu_packfile_type_group.checked_action().text().remove_q_string(&QString::from_std_str("&")).to_std_string())
                };

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let _ = CENTRAL_COMMAND.read().unwrap().send(Command::SetPackFileType(pack_key, packfile_type));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        let context_menu_change_compression_format = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                let compression_format = CompressionFormat::from(pack_file_contents_ui.context_menu_compression_group.checked_action().text().remove_q_string(&QString::from_std_str("&")).to_std_string().as_str());
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let cf = send_ipc_command_async(Command::ChangeCompressionFormat(pack_key, compression_format), response_extractor!(Response::CompressionFormat));
                pack_file_contents_ui.context_menu_compression_group.block_signals(true);
                match cf {
                    CompressionFormat::None => pack_file_contents_ui.context_menu_compression_none.set_checked(true),
                    CompressionFormat::Lzma1 => pack_file_contents_ui.context_menu_compression_lzma1.set_checked(true),
                    CompressionFormat::Lz4 => pack_file_contents_ui.context_menu_compression_lz4.set_checked(true),
                    CompressionFormat::Zstd => pack_file_contents_ui.context_menu_compression_zstd.set_checked(true),
                }
                pack_file_contents_ui.context_menu_compression_group.block_signals(false);
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        let context_menu_index_includes_timestamp = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                let state = pack_file_contents_ui.context_menu_index_includes_timestamp.is_checked();
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let _ = CENTRAL_COMMAND.read().unwrap().send(Command::ChangeIndexIncludesTimestamp(pack_key, state));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        let context_menu_optimize_packfile = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {
                info!("Triggering `Optimize PackFile` By Slot");

                app_ui.toggle_main_window(false);

                match AppUI::optimizer_dialog(&app_ui, &pack_file_contents_ui, &global_search_ui) {
                    Ok(Some(_)) => show_dialog(app_ui.main_window(), tr("optimize_packfile_success"), true),
                    Ok(None) => {},
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_patch_siege_ai = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {
                info!("Triggering `Patch SiegeAI` By Slot");

                app_ui.toggle_main_window(false);

                if let Err(error) = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, true) {
                    return show_dialog(app_ui.main_window(), error, false);
                }

                GlobalSearchUI::clear(&global_search_ui);

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                match send_ipc_command_result_async(Command::PatchSiegeAI(pack_key.clone()), response_extractor!(Response::StringVecContainerPath, message, paths)) {
                    Ok((message, paths)) => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(paths, true), DataSource::PackFile, &pack_key);
                        show_dialog(app_ui.main_window(), message, true);
                    }
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_live_export = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Live Export` By Slot");

                app_ui.toggle_main_window(false);

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                match send_ipc_command_result_async(Command::LiveExport(pack_key), response_extractor!()) {
                    Ok(()) => show_message_info(app_ui.message_widget(), tr("live_export_success")),
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_pack_map = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Pack Map` By Slot");

                app_ui.toggle_main_window(false);

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);

                if let Ok(Some((tile_maps, tiles))) = AppUI::pack_map_dialog(&app_ui, &pack_file_contents_ui) {
                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    match send_ipc_command_result_async(Command::PackMap(pack_key.clone(), tile_maps, tiles), response_extractor!(Response::VecContainerPathVecContainerPath, paths_to_add, paths_to_delete)) {
                        Ok((paths_to_add, paths_to_delete)) => {
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths_to_add.to_vec()), DataSource::PackFile, &pack_key);

                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);

                            let failed_paths = UI_STATE.set_open_packedfiles()
                                .iter_mut()
                                .filter(|view| view.data_source() == DataSource::PackFile && (paths_to_add.iter().any(|path| path.path_raw() == *view.path_read() || *view.path_read() == RESERVED_NAME_NOTES)))
                                .filter_map(|view| if view.reload(&view.path_copy(), &pack_file_contents_ui).is_err() { Some(view.path_copy()) } else { None })
                                .collect::<Vec<_>>();

                            for path in &failed_paths {
                                let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, false);
                            }

                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(paths_to_delete.to_vec(), settings_bool("delete_empty_folders_on_delete")), DataSource::PackFile, &pack_key);

                            for path in &paths_to_delete {
                                let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path.path_raw(), DataSource::PackFile, false);
                            }
                        }
                        Err(error) => show_dialog(app_ui.main_window(), error, false),
                    }
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_rescue_packfile = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if AppUI::are_you_sure_edition(&app_ui, "are_you_sure_rescue_packfile") {
                    info!("Triggering `Rescue PackFile` By Slot");

                    app_ui.toggle_main_window(false);

                    if let Err(error) = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui) {
                        return show_dialog(app_ui.main_window(), error, false);
                    }

                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        app_ui.main_window(),
                        &qtr("save_packfile"),
                    );
                    file_dialog.set_accept_mode(qt_widgets::q_file_dialog::AcceptMode::AcceptSave);
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                    file_dialog.set_default_suffix(&QString::from_std_str("pack"));

                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                        match send_ipc_command_result_async(Command::CleanAndSavePackAs(pack_key.clone(), path), response_extractor!(Response::ContainerInfo)) {
                            Ok(pack_file_info) => {
                                let mut build_data = BuildData::new();
                                build_data.editable = true;
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile, &pack_key);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile, &pack_key);

                                let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                                packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                                packfile_item.set_text(&QString::from_std_str(file_name));

                                UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);
                            }
                            Err(error) => show_dialog(app_ui.main_window(), error, false),
                        }
                    }

                    app_ui.toggle_main_window(true);
                }
            }
        ));

        let context_menu_build_starpos = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                app_ui.toggle_main_window(false);

                if let Err(error) = AppUI::build_starpos(&app_ui, &pack_file_contents_ui) {
                    show_dialog(app_ui.main_window(), error, false);
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_update_anim_ids = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                app_ui.toggle_main_window(false);

                if let Err(error) = AppUI::update_anim_ids(&app_ui, &pack_file_contents_ui) {
                    show_dialog(app_ui.main_window(), error, false);
                }

                app_ui.toggle_main_window(true);
            }
        ));

        let context_menu_mymod_import = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                AppUI::import_mymod(&app_ui, &pack_file_contents_ui, &pack_key);
            }
        ));

        let context_menu_mymod_export = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                AppUI::export_mymod(&app_ui, &pack_file_contents_ui, Some(vec![ContainerPath::Folder("".to_owned())]));
            }
        ));

        let context_menu_mymod_delete = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui,
            dependencies_ui => move |_| {
                if AppUI::are_you_sure(&app_ui, true, false) {
                    info!("Triggering `Delete MyMod` By Context Menu");

                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    let mode = send_ipc_command(Command::GetPackOperationalMode(pack_key.clone()), response_extractor!(Response::OperationalMode));

                    if let OperationalMode::MyMod(ref game_folder_name, ref mod_name) = mode {
                        let old_mod_name = mod_name.clone();
                        let mymods_base_path = settings_path_buf(MYMOD_BASE_PATH);
                        if mymods_base_path.is_dir() {
                            let mut mymod_path = mymods_base_path;
                            mymod_path.push(game_folder_name);
                            mymod_path.push(mod_name);

                            if !mymod_path.is_file() {
                                return show_dialog(app_ui.main_window(), "The Pack of the selected MyMod doesn't exist, so it can't be deleted.", false);
                            }

                            if remove_file(&mymod_path).is_err() {
                                return show_dialog(app_ui.main_window(), "Error deleting the MyMod's Pack.", false);
                            }

                            let mut mymod_assets_path = mymod_path.clone();
                            mymod_assets_path.pop();
                            mymod_assets_path.push(mymod_path.file_stem().unwrap().to_string_lossy().as_ref());

                            if !mymod_assets_path.is_dir() {
                                show_dialog(app_ui.main_window(), "The Mod's Pack has been deleted, but its assets folder is nowhere to be found.", false);
                            } else if remove_dir_all(&mymod_assets_path).is_err() {
                                show_dialog(app_ui.main_window(), "Error deleting the MyMod's Asset Folder.", false);
                            }

                            AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &diagnostics_ui, &global_search_ui, &dependencies_ui);

                            let _ = CENTRAL_COMMAND.read().unwrap().send(Command::ClosePack(pack_key.clone()));
                            AppUI::enable_packfile_actions(&app_ui, false);
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clear, DataSource::PackFile, &pack_key);
                            global_search_ui.update_pack_sources(&pack_file_contents_ui);
                            UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

                            show_dialog(app_ui.main_window(), tre("mymod_delete_success", &[&old_mod_name]), true);
                        } else {
                            show_dialog(app_ui.main_window(), "MyMod path not configured. Go to <i>'PackFile/Settings'</i> and configure it.", false);
                        }
                    }
                }
            }
        ));

        let context_menu_mymod_open_folder = SlotOfBool::new(&pack_file_contents_ui.packfile_contents_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let mode = send_ipc_command(Command::GetPackOperationalMode(pack_key), response_extractor!(Response::OperationalMode));

                if let OperationalMode::MyMod(ref game_folder_name, ref mod_name) = mode {
                    let mymods_base_path = settings_path_buf(MYMOD_BASE_PATH);
                    if mymods_base_path.is_dir() {
                        let mut assets_folder = mymods_base_path;
                        assets_folder.push(game_folder_name);
                        assets_folder.push(Path::new(mod_name).file_stem().unwrap().to_string_lossy().as_ref());
                        let _ = open::that(&assets_folder);
                    } else {
                        show_dialog(app_ui.main_window(), "MyMod path not configured. Go to <i>'PackFile/Settings'</i> and configure it.", false);
                    }
                }
            }
        ));

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
            contextual_menu_copy_to_pack_about_to_show,
            contextual_menu_copy_to_pack,
            contextual_menu_delete,
            contextual_menu_extract,
            contextual_menu_rename,
            contextual_menu_copy_path,
            contextual_menu_copy,
            contextual_menu_cut,
            contextual_menu_paste,
            contextual_menu_duplicate,

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

            context_menu_install,
            context_menu_uninstall,
            context_menu_change_packfile_type,
            context_menu_change_compression_format,
            context_menu_index_includes_timestamp,
            context_menu_optimize_packfile,
            context_menu_patch_siege_ai,
            context_menu_live_export,
            context_menu_pack_map,
            context_menu_rescue_packfile,
            context_menu_build_starpos,
            context_menu_update_anim_ids,

            context_menu_mymod_import,
            context_menu_mymod_export,
            context_menu_mymod_delete,
            context_menu_mymod_open_folder,

            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}
