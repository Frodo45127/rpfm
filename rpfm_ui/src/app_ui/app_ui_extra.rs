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
Module with all the code for extra implementations of `AppUI`.

This module contains the implementation of custom functions for `AppUI`. The reason
they're here and not in the main file is because I don't want to pollute that one,
as it's mostly meant for initialization and configuration.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QFileDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::{q_message_box, QMessageBox};
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;

use qt_gui::QStandardItemModel;

use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QStringList;
use qt_core::QRegExp;
use qt_core::{SlotOfBool, SlotOfQString};
use qt_core::QSettings;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::common::*;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::games::supported_games::*;
use rpfm_lib::packedfile::{PackedFileType, animpack, table::loc, text, text::TextType};
use rpfm_lib::packfile::{PathType, PackFileInfo, PFHFileType, PFHFlags, CompressionState, PFHVersion, RESERVED_NAME_EXTRA_PACKFILE, RESERVED_NAME_NOTES, RESERVED_NAME_SETTINGS, RESERVED_NAME_DEPENDENCIES_MANAGER};
use rpfm_lib::schema::{APIResponseSchema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::settings::MYMOD_BASE_PATH;
use rpfm_lib::tips::APIResponseTips;
use rpfm_lib::updater::{APIResponse, CHANGELOG_FILE};

use super::AppUI;
use super::NewPackedFile;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::FIRST_GAME_CHANGE_DONE;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, qtre, tre};
use crate::pack_tree::{BuildData, icons::IconType, new_pack_file_tooltip, PackTree, TreePathType, TreeViewOperation};
use crate::packedfile_views::dependencies_manager::DependenciesManagerView;
use crate::packedfile_views::{anim_fragment::*, animpack::*, ca_vp8::*, DataSource, decoder::*, esf::*, external::*, image::*, PackedFileView, packfile::PackFileExtraView, packfile_settings::*, table::*, text::*, unit_variant::*};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::QT_PROGRAM;
use crate::QT_ORG;
use crate::RPFM_PATH;
use crate::UI_STATE;
use crate::ui::GameSelectedIcons;
use crate::ui_state::OperationalMode;
use crate::utils::{create_grid_layout, get_packed_file_type, show_dialog, show_dialog_decode_button, log_to_status_bar};

#[cfg(feature = "support_rigidmodel")]
use crate::packedfile_views::rigidmodel::*;

#[cfg(feature = "support_uic")]
use crate::packedfile_views::uic::*;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUI`.
impl AppUI {

    /// This function takes care of updating the Main Window's title to reflect the current state of the program.
    pub unsafe fn update_window_title(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // First check if we have a PackFile open. If not, just leave the default title.
        let window_title =
            if pack_file_contents_ui.packfile_contents_tree_model.invisible_root_item().is_null() ||
            pack_file_contents_ui.packfile_contents_tree_model.invisible_root_item().row_count() == 0 {
            "Rusted PackFile Manager[*]".to_owned()
        }

        // If there is a `PackFile` open, check if it has been modified, and set the title accordingly.
        else {
            format!("{}[*]", pack_file_contents_ui.packfile_contents_tree_model.item_1a(0).text().to_std_string())
        };

        app_ui.main_window.set_window_modified(UI_STATE.get_is_modified());
        app_ui.main_window.set_window_title(&QString::from_std_str(window_title));
    }

    /// This function pops up a modal asking you if you're sure you want to do an action that may result in unsaved data loss.
    ///
    /// If you are trying to delete the open MyMod, pass it true.
    pub unsafe fn are_you_sure(app_ui: &Rc<Self>, is_delete_my_mod: bool) -> bool {
        are_you_sure(app_ui.main_window.as_mut_raw_ptr(), is_delete_my_mod)
    }

    /// This function pops up a modal asking you if you're sure you want to do an action that may result in loss of data.
    ///
    /// This one is for custom actions, not for closing window actions.
    pub unsafe fn are_you_sure_edition(app_ui: &Rc<AppUI>, message: &str) -> bool {

        // Create the dialog and run it (Yes => 3, No => 4).
        QMessageBox::from_2_q_string_icon3_int_q_widget(
            &qtr("rpfm_title"),
            &qtr(message),
            q_message_box::Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            &app_ui.main_window,
        ).exec() == 3
    }

    /// This function updates the backend of all open PackedFiles with their view's data.
    #[must_use = "If one of those mysterious save errors happen here and we don't use the result, we may be losing the new changes to a file."]
    pub unsafe fn back_to_back_end_all(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<()> {

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            packed_file_view.save(app_ui, pack_file_contents_ui)?;
        }
        Ok(())
    }

    /// This function deletes all the widgets corresponding to opened PackedFiles.
    #[must_use = "If one of those mysterious save errors happen here and we don't use the result, we may be losing the new changes to a file."]
    pub unsafe fn purge_them_all(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        save_before_deleting: bool,
    ) -> Result<()> {

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            if save_before_deleting && !packed_file_view.get_path().starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {
                packed_file_view.save(app_ui, pack_file_contents_ui)?;
            }
            let widget = packed_file_view.get_mut_widget();
            let index = app_ui.tab_bar_packed_file.index_of(widget);
            if index != -1 {
                app_ui.tab_bar_packed_file.remove_tab(index);
            }

            // Delete the widget manually to free memory.
            widget.delete_later();
        }

        // Remove all open PackedFiles and their slots.
        UI_STATE.set_open_packedfiles().clear();

        // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
        app_ui.game_selected_group.set_enabled(true);

        // Just in case what was open before was the `Add From PackFile` TreeView, unlock it.
        UI_STATE.set_packfile_contents_read_only(false);

        // Update the background icon.
        GameSelectedIcons::set_game_selected_icon(app_ui);

        Ok(())
    }

    /// This function deletes all the widgets corresponding to the specified PackedFile, if exists.
    #[must_use = "If one of those mysterious save errors happen here and we don't use the result, we may be losing the new changes to a file."]
    pub unsafe fn purge_that_one_specifically(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        path: &[String],
        data_source: DataSource,
        save_before_deleting: bool
    ) -> Result<()> {

        if path.is_empty() {
            log::info!("purging empty path? this is a bug.");
        }

        let mut did_it_worked = Ok(());

        // Black magic to remove widgets.
        let position = UI_STATE.get_open_packedfiles().iter().position(|x| *x.get_ref_path() == path && x.get_data_source() == data_source);
        if let Some(position) = position {
            if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().get(position) {

                // Do not try saving PackFiles.
                if save_before_deleting && !path.starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {
                    did_it_worked = packed_file_view.save(app_ui, pack_file_contents_ui);
                }
                let widget = packed_file_view.get_mut_widget();
                let index = app_ui.tab_bar_packed_file.index_of(widget);
                if index != -1 {
                    app_ui.tab_bar_packed_file.remove_tab(index);
                }

                // Delete the widget manually to free memory.
                widget.delete_later();
            }

            if !path.is_empty() {
                UI_STATE.set_open_packedfiles().remove(position);
                if !path.starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {

                    // We check if there are more tables open. This is because we cannot change the GameSelected
                    // when there is a PackedFile using his Schema.
                    let mut enable_game_selected_menu = true;
                    for path in UI_STATE.get_open_packedfiles().iter().map(|x| x.get_ref_path()) {
                        if let Some(folder) = path.get(0) {
                            if folder.to_lowercase() == "db" {
                                enable_game_selected_menu = false;
                                break;
                            }
                        }

                        else if let Some(file) = path.last() {
                            if !file.is_empty() && file.to_lowercase().ends_with(".loc") {
                                enable_game_selected_menu = false;
                                break;
                            }
                        }
                    }

                    if enable_game_selected_menu {
                        app_ui.game_selected_group.set_enabled(true);
                    }
                }
            }
        }

        // Update the background icon.
        GameSelectedIcons::set_game_selected_icon(app_ui);

        did_it_worked
    }

    /// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending on the situation.
    ///
    /// NOTE: The `game_folder` is for when using this function with *MyMods*. If you're opening a normal mod, pass it empty.
    pub unsafe fn open_packfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_paths: &[PathBuf],
        game_folder: &str,
    ) -> Result<()> {

        // Destroy whatever it's in the PackedFile's view, to avoid data corruption. We don't care about this result.
        let _ = Self::purge_them_all(app_ui, pack_file_contents_ui, false);

        // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
        app_ui.main_window.set_enabled(false);
        let receiver = CENTRAL_COMMAND.send_background(Command::OpenPackFiles(pack_file_paths.to_vec()));

        // If it's only one packfile, store it in the recent file list.
        if pack_file_paths.len() == 1 {
            let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));

            let paths = if q_settings.contains(&QString::from_std_str("recentFileList")) {
                q_settings.value_1a(&QString::from_std_str("recentFileList")).to_string_list()
            } else {
                QStringList::new()
            };

            let pos = paths.index_of_1a(&QString::from_std_str(&pack_file_paths[0].to_str().unwrap()));
            if pos != -1 {
                paths.remove_at(pos);
            }

            paths.prepend(&QString::from_std_str(&pack_file_paths[0].to_str().unwrap()));

            while paths.count_0a() > 10 {
                paths.remove_last();
            }
            q_settings.set_value(&QString::from_std_str("recentFileList"), &QVariant::from_q_string_list(&paths));
        }

        let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Check what response we got.
        let response = CentralCommand::recv_try(&receiver);
        match response {

            // If it's success....
            Response::PackFileInfo(ui_data) => {

                // We choose the right option, depending on our PackFile.
                match ui_data.pfh_file_type {
                    PFHFileType::Boot => app_ui.change_packfile_type_boot.set_checked(true),
                    PFHFileType::Release => app_ui.change_packfile_type_release.set_checked(true),
                    PFHFileType::Patch => app_ui.change_packfile_type_patch.set_checked(true),
                    PFHFileType::Mod => app_ui.change_packfile_type_mod.set_checked(true),
                    PFHFileType::Movie => app_ui.change_packfile_type_movie.set_checked(true),
                    PFHFileType::Other(_) => app_ui.change_packfile_type_other.set_checked(true),
                }

                // Enable or disable these, depending on what data we have in the header.
                app_ui.change_packfile_type_data_is_encrypted.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA));
                app_ui.change_packfile_type_index_includes_timestamp.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS));
                app_ui.change_packfile_type_index_is_encrypted.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX));
                app_ui.change_packfile_type_header_is_extended.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER));

                // Set the compression level correctly, because otherwise we may fuckup some files.
                let compression_state = match ui_data.compression_state {
                    CompressionState::Enabled => true,
                    CompressionState::Partial | CompressionState::Disabled => false,
                };
                app_ui.change_packfile_type_data_is_compressed.set_checked(compression_state);

                // Update the TreeView.
                let mut build_data = BuildData::new();
                build_data.editable = true;
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

                // Close the Global Search stuff and reset the filter's history.
                GlobalSearchUI::clear(global_search_ui);

                // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
                if !game_folder.is_empty() && pack_file_paths.len() == 1 {

                    // NOTE: Arena should never be here.
                    // Change the Game Selected in the UI.
                    match game_folder {
                        KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3.trigger(),
                        KEY_TROY => app_ui.game_selected_troy.trigger(),
                        KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms.trigger(),
                        KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2.trigger(),
                        KEY_WARHAMMER => app_ui.game_selected_warhammer.trigger(),
                        KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia.trigger(),
                        KEY_ATTILA => app_ui.game_selected_attila.trigger(),
                        KEY_ROME_2 => app_ui.game_selected_rome_2.trigger(),
                        KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.trigger(),
                        KEY_NAPOLEON => app_ui.game_selected_napoleon.trigger(),
                        KEY_EMPIRE => app_ui.game_selected_empire.trigger(),
                        _ => unimplemented!()
                    }

                    // Set the current "Operational Mode" to `MyMod`.
                    UI_STATE.set_operational_mode(app_ui, Some(&pack_file_paths[0]));
                }

                // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
                else {

                    // Reset the operational mode.
                    UI_STATE.set_operational_mode(app_ui, None);

                    // Depending on the Id, choose one game or another.
                    let game_selected = GAME_SELECTED.read().unwrap().get_game_key_name();
                    match ui_data.pfh_version {

                        // PFH6 is for Troy and maybe WH3.
                        PFHVersion::PFH6 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            match &*game_selected {
                                KEY_TROY => app_ui.game_selected_troy.trigger(),
                                _ => {
                                    show_dialog(&app_ui.main_window, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_TROY]), true);
                                    app_ui.game_selected_troy.trigger();
                                }
                            }
                        },

                        // PFH5 is for Warhammer 2/Arena.
                        PFHVersion::PFH5 => {

                            // If the PackFile has the mysterious byte enabled, it's from Arena.
                            if ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
                                app_ui.game_selected_arena.trigger();
                            }

                            // Otherwise, it's from Three Kingdoms, Warhammer 2, Troy or Warhammer 3.
                            else {
                                match &*game_selected {
                                    KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3.trigger(),
                                    KEY_TROY => app_ui.game_selected_troy.trigger(),
                                    KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms.trigger(),
                                    KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2.trigger(),
                                    _ => {
                                        show_dialog(&app_ui.main_window, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_WARHAMMER_3]), true);
                                        app_ui.game_selected_warhammer_3.trigger();
                                    }
                                }
                            }
                        },

                        // PFH4 is for Thrones of Britannia/Warhammer 1/Attila/Rome 2.
                        PFHVersion::PFH4 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            match &*game_selected {
                                KEY_WARHAMMER => app_ui.game_selected_warhammer.trigger(),
                                KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia.trigger(),
                                KEY_ATTILA => app_ui.game_selected_attila.trigger(),
                                KEY_ROME_2 => app_ui.game_selected_rome_2.trigger(),
                                _ => {
                                    show_dialog(&app_ui.main_window, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_ROME_2]), true);
                                    app_ui.game_selected_rome_2.trigger();
                                }
                            }
                        },

                        // PFH3/2 is for Shogun 2.
                        PFHVersion::PFH3 | PFHVersion::PFH2 => {
                            match &*game_selected {
                                KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.trigger(),
                                _ => {
                                    show_dialog(&app_ui.main_window, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_SHOGUN_2]), true);
                                    app_ui.game_selected_shogun_2.trigger();
                                }
                            }
                        }

                        // PFH0 is for Napoleon/Empire.
                        PFHVersion::PFH0 => {
                            match &*game_selected {
                                KEY_NAPOLEON => app_ui.game_selected_napoleon.trigger(),
                                KEY_EMPIRE => app_ui.game_selected_empire.trigger(),
                                _ => {
                                    show_dialog(&app_ui.main_window, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_EMPIRE]), true);
                                    app_ui.game_selected_empire.trigger();
                                }
                            }
                        },
                    }
                }

                UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);

                // Re-enable the Main Window.
                app_ui.main_window.set_enabled(true);
            }

            // If we got an error...
            Response::Error(error) => {
                app_ui.main_window.set_enabled(true);
                return Err(error)
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Return success.
        Ok(())
    }


    /// This function is used to save the currently open `PackFile` to disk.
    ///
    /// If the PackFile doesn't exist or we pass `save_as = true`,
    /// it opens a dialog asking for a path.
    pub unsafe fn save_packfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        save_as: bool,
    ) -> Result<()> {

        let mut result = Ok(());
        app_ui.main_window.set_enabled(false);

        // First, we need to save all open `PackedFiles` to the backend. If one fails, we want to know what one.
        AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui)?;

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
        let response = CentralCommand::recv(&receiver);
        let mut path = if let Response::PathBuf(path) = response { path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };
        if !path.is_file() || save_as {

            // Create the FileDialog to save the PackFile and configure it.
            let file_dialog = QFileDialog::from_q_widget_q_string(
                &app_ui.main_window,
                &qtr("save_packfile"),
            );
            file_dialog.set_accept_mode(qt_widgets::q_file_dialog::AcceptMode::AcceptSave);
            file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
            file_dialog.set_confirm_overwrite(true);
            file_dialog.set_default_suffix(&QString::from_std_str("pack"));
            file_dialog.select_file(&QString::from_std_str(&path.file_name().unwrap().to_string_lossy()));

            // If we are saving an existing PackFile with another name, we start in his current path.
            if path.is_file() {
                path.pop();
                file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
            }

            // In case we have a default path for the Game Selected and that path is valid,
            // we use his data folder as base path for saving our PackFile.
            else if let Ok(ref path) = GAME_SELECTED.read().unwrap().get_local_mods_path() {
                if path.is_dir() { file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned())); }
            }

            // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {
                let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let receiver = CENTRAL_COMMAND.send_background(Command::SavePackFileAs(path));
                let response = CentralCommand::recv_try(&receiver);
                match response {
                    Response::PackFileInfo(pack_file_info) => {
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);
                        let packfile_item = pack_file_contents_ui.packfile_contents_tree_model.item_1a(0);
                        packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                        packfile_item.set_text(&QString::from_std_str(&file_name));

                        UI_STATE.set_operational_mode(app_ui, None);
                        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                    }
                    Response::Error(error) => result = Err(error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }

        else {
            let receiver = CENTRAL_COMMAND.send_background(Command::SavePackFile);
            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::PackFileInfo(pack_file_info) => {
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);
                    let packfile_item = pack_file_contents_ui.packfile_contents_tree_model.item_1a(0);
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                    UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                }
                Response::Error(error) => result = Err(error),

                // In ANY other situation, it's a message problem.
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // Clean the treeview and the views from markers.
        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            packed_file_view.clean();
        }

        // Then we re-enable the main Window and return whatever we've received.
        app_ui.main_window.set_enabled(true);
        result
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub unsafe fn enable_packfile_actions(app_ui: &Rc<Self>, pack_path: &Path, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        let game_selected = GAME_SELECTED.read().unwrap().get_game_key_name();
        if game_selected == KEY_ARENA {

            // Disable the actions that allow to create and save PackFiles.
            app_ui.packfile_new_packfile.set_enabled(false);
            app_ui.packfile_save_packfile.set_enabled(false);
            app_ui.packfile_save_packfile_as.set_enabled(false);
            app_ui.packfile_install.set_enabled(false);
            app_ui.packfile_uninstall.set_enabled(false);

            // This one too, though we had to deal with it specially later on.
            app_ui.mymod_new.set_enabled(false);
        }

        // Otherwise...
        else {

            // Enable or disable the actions from "PackFile" Submenu.
            app_ui.packfile_new_packfile.set_enabled(true);
            app_ui.packfile_save_packfile.set_enabled(enable);
            app_ui.packfile_save_packfile_as.set_enabled(enable);

            // Ensure it's a file and it's not in data before proceeding.
            let enable_install = if !pack_path.is_file() { false }
            else if let Ok(game_data_path) = GAME_SELECTED.read().unwrap().get_local_mods_path() {
                game_data_path.is_dir() && !pack_path.starts_with(&game_data_path)
            } else { false };
            app_ui.packfile_install.set_enabled(enable_install);

            let enable_uninstall = if !pack_path.is_file() { false }
            else if let Ok(mut game_data_path) = GAME_SELECTED.read().unwrap().get_local_mods_path() {
                if !game_data_path.is_dir() || pack_path.starts_with(&game_data_path) { false }
                else {
                    game_data_path.push(pack_path.file_name().unwrap().to_string_lossy().to_string());
                    game_data_path.is_file()
                }
            } else { false };
            app_ui.packfile_uninstall.set_enabled(enable_uninstall);

            // If there is a "MyMod" path set in the settings...
            if let Some(ref path) = SETTINGS.read().unwrap().paths[MYMOD_BASE_PATH] {
                if path.is_dir() { app_ui.mymod_new.set_enabled(true); }
                else { app_ui.mymod_new.set_enabled(false); }
            }
            else { app_ui.mymod_new.set_enabled(false); }
        }

        // These actions are common, no matter what game we have.
        app_ui.change_packfile_type_group.set_enabled(enable);
        app_ui.change_packfile_type_index_includes_timestamp.set_enabled(enable);

        app_ui.special_stuff_rescue_packfile.set_enabled(enable);

        // If we are enabling...
        if enable {

            // Check the Game Selected and enable the actions corresponding to out game.
            match &*game_selected {
                KEY_WARHAMMER_3 => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(true);
                    app_ui.special_stuff_wh3_optimize_packfile.set_enabled(true);
                },
                KEY_TROY => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(true);
                    app_ui.special_stuff_troy_optimize_packfile.set_enabled(true);
                },
                KEY_THREE_KINGDOMS => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(true);
                    app_ui.special_stuff_three_k_optimize_packfile.set_enabled(true);
                },
                KEY_WARHAMMER_2 => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(true);
                    app_ui.special_stuff_wh2_patch_siege_ai.set_enabled(true);
                    app_ui.special_stuff_wh2_optimize_packfile.set_enabled(true);
                },
                KEY_WARHAMMER => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_wh_patch_siege_ai.set_enabled(true);
                    app_ui.special_stuff_wh_optimize_packfile.set_enabled(true);
                },
                KEY_THRONES_OF_BRITANNIA => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_tob_optimize_packfile.set_enabled(true);
                },
                KEY_ATTILA => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_att_optimize_packfile.set_enabled(true);
                },
                KEY_ROME_2 => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_rom2_optimize_packfile.set_enabled(true);
                },
                KEY_SHOGUN_2 => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_sho2_optimize_packfile.set_enabled(true);
                },
                KEY_NAPOLEON => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_nap_optimize_packfile.set_enabled(true);
                },
                KEY_EMPIRE => {
                    app_ui.change_packfile_type_data_is_compressed.set_enabled(false);
                    app_ui.special_stuff_emp_optimize_packfile.set_enabled(true);
                },
                _ => {},
            }
        }

        // If we are disabling...
        else {

            // Universal Actions.
            app_ui.change_packfile_type_data_is_compressed.set_enabled(false);

            // Disable Warhammer 3 actions...
            app_ui.special_stuff_wh3_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_wh3_generate_dependencies_cache.set_enabled(false);

            // Disable Troy actions...
            app_ui.special_stuff_troy_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_troy_generate_dependencies_cache.set_enabled(false);

            // Disable Three Kingdoms actions...
            app_ui.special_stuff_three_k_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_three_k_generate_dependencies_cache.set_enabled(false);

            // Disable Warhammer 2 actions...
            app_ui.special_stuff_wh2_patch_siege_ai.set_enabled(false);
            app_ui.special_stuff_wh2_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_wh2_generate_dependencies_cache.set_enabled(false);

            // Disable Warhammer actions...
            app_ui.special_stuff_wh_patch_siege_ai.set_enabled(false);
            app_ui.special_stuff_wh_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_wh_generate_dependencies_cache.set_enabled(false);

            // Disable Thrones of Britannia actions...
            app_ui.special_stuff_tob_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_tob_generate_dependencies_cache.set_enabled(false);

            // Disable Attila actions...
            app_ui.special_stuff_att_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_att_generate_dependencies_cache.set_enabled(false);

            // Disable Rome 2 actions...
            app_ui.special_stuff_rom2_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_rom2_generate_dependencies_cache.set_enabled(false);

            // Disable Shogun 2 actions...
            app_ui.special_stuff_sho2_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_sho2_generate_dependencies_cache.set_enabled(false);

            // Disable Napoleon actions...
            app_ui.special_stuff_nap_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_nap_generate_dependencies_cache.set_enabled(false);

            // Disable Empire actions...
            app_ui.special_stuff_emp_optimize_packfile.set_enabled(false);
            app_ui.special_stuff_emp_generate_dependencies_cache.set_enabled(false);
        }

        // The assembly kit thing should only be available for Rome 2 and later games.
        // And dependencies generation should be enabled for the current game.
        match &*game_selected {
            KEY_WARHAMMER_3 => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_wh3_generate_dependencies_cache.set_enabled(true);
            },
            KEY_TROY => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_troy_generate_dependencies_cache.set_enabled(true);
            },
            KEY_THREE_KINGDOMS => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_three_k_generate_dependencies_cache.set_enabled(true);
            },
            KEY_WARHAMMER_2 => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_wh2_generate_dependencies_cache.set_enabled(true);
            },
            KEY_WARHAMMER => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_wh_generate_dependencies_cache.set_enabled(true);
            },
            KEY_THRONES_OF_BRITANNIA => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_tob_generate_dependencies_cache.set_enabled(true);
            },
            KEY_ATTILA => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_att_generate_dependencies_cache.set_enabled(true);
            },
            KEY_ROME_2 => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
                app_ui.special_stuff_rom2_generate_dependencies_cache.set_enabled(true);
            },
            KEY_SHOGUN_2 => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(false);
                app_ui.special_stuff_sho2_generate_dependencies_cache.set_enabled(true);
            },
            KEY_NAPOLEON => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(false);
                app_ui.special_stuff_nap_generate_dependencies_cache.set_enabled(true);
            },
            KEY_EMPIRE => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(false);
                app_ui.special_stuff_emp_generate_dependencies_cache.set_enabled(true);
            },
            _ => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(false);
            },
        }
    }

    /// This function takes care of recreating the dynamic submenus under `PackFile` menu.
    pub unsafe fn build_open_from_submenus(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
    ) {

        // First, we clear both menus, so we can rebuild them properly.
        app_ui.packfile_open_recent.clear();
        app_ui.packfile_open_from_content.clear();
        app_ui.packfile_open_from_data.clear();
        app_ui.packfile_open_from_autosave.clear();

        //---------------------------------------------------------------------------------------//
        // Build the menus...
        //---------------------------------------------------------------------------------------//

        // Recent PackFiles.
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));
        if q_settings.contains(&QString::from_std_str("recentFileList")) {
            let paths = q_settings.value_1a(&QString::from_std_str("recentFileList")).to_string_list();

            for index in 0..paths.count_0a() {
                let path_str = paths.at(index).to_std_string();

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let path = PathBuf::from(&path_str);
                if path.is_file() {
                    let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                    let open_mod_action = app_ui.packfile_open_recent.add_action_q_string(&QString::from_std_str(mod_name));

                    // Create the slot for that action.
                    let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                        app_ui,
                        pack_file_contents_ui,
                        global_search_ui,
                        diagnostics_ui,
                        path => move |_| {
                        if Self::are_you_sure(&app_ui, false) {
                            if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "") {
                                return show_dialog(&app_ui.main_window, error, false);
                            }

                            if SETTINGS.read().unwrap().settings_bool["diagnostics_trigger_on_open"] {

                                // Disable the top menus before triggering the check. Otherwise, we may end up in a crash.
                                app_ui.menu_bar_packfile.set_enabled(false);

                                DiagnosticsUI::check(&app_ui, &diagnostics_ui);

                                app_ui.menu_bar_packfile.set_enabled(true);
                            }
                        }
                    }));

                    // Connect the slot and store it.
                    open_mod_action.triggered().connect(&slot_open_mod);
                }
            }
        }

        // Get the path of every PackFile in the content folder (if the game's path it's configured) and make an action for each one of them.
        let mut content_paths = GAME_SELECTED.read().unwrap().get_content_packfiles_paths();
        if let Some(ref mut paths) = content_paths {
            paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
            for path in paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let open_mod_action = app_ui.packfile_open_from_content.add_action_q_string(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                    app_ui,
                    pack_file_contents_ui,
                    global_search_ui,
                    diagnostics_ui,
                    path => move |_| {
                    if Self::are_you_sure(&app_ui, false) {
                        if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "") {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if SETTINGS.read().unwrap().settings_bool["diagnostics_trigger_on_open"] {

                            // Disable the top menus before triggering the check. Otherwise, we may end up in a crash.
                            app_ui.menu_bar_packfile.set_enabled(false);

                            DiagnosticsUI::check(&app_ui, &diagnostics_ui);

                            app_ui.menu_bar_packfile.set_enabled(true);
                        }
                    }
                }));

                // Connect the slot and store it.
                open_mod_action.triggered().connect(&slot_open_mod);
            }
        }

        // Get the path of every PackFile in the data folder (if the game's path it's configured) and make an action for each one of them.
        let mut data_paths = GAME_SELECTED.read().unwrap().get_data_packfiles_paths();
        if let Some(ref mut paths) = data_paths {
            paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
            for path in paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let open_mod_action = app_ui.packfile_open_from_data.add_action_q_string(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                    app_ui,
                    pack_file_contents_ui,
                    global_search_ui,
                    diagnostics_ui,
                    path => move |_| {
                    if Self::are_you_sure(&app_ui, false) {
                        if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "") {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if SETTINGS.read().unwrap().settings_bool["diagnostics_trigger_on_open"] {

                            // Disable the top menus before triggering the check. Otherwise, we may end up in a crash.
                            app_ui.menu_bar_packfile.set_enabled(false);

                            DiagnosticsUI::check(&app_ui, &diagnostics_ui);

                            app_ui.menu_bar_packfile.set_enabled(true);
                        }
                    }
                }));

                // Connect the slot and store it.
                open_mod_action.triggered().connect(&slot_open_mod);
            }
        }

        // Get the path of every PackFile in the autosave folder, sorted by modification date, and make an action for each one of them.
        if let Ok(autosave_paths) = get_backup_autosave_path() {
            let autosave_paths = get_files_in_folder_from_newest_to_oldest(&autosave_paths);
            if let Ok(ref paths) = autosave_paths {
                for path in paths {

                    // That means our file is a valid PackFile and it needs to be added to the menu.
                    let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                    let open_mod_action = app_ui.packfile_open_from_autosave.add_action_q_string(&QString::from_std_str(mod_name));

                    // Create the slot for that action.
                    let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                        app_ui,
                        pack_file_contents_ui,
                        global_search_ui,
                        diagnostics_ui,
                        path => move |_| {
                        if Self::are_you_sure(&app_ui, false) {
                            if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "") {
                                return show_dialog(&app_ui.main_window, error, false);
                            }

                            if SETTINGS.read().unwrap().settings_bool["diagnostics_trigger_on_open"] {

                                // Disable the top menus before triggering the check. Otherwise, we may end up in a crash.
                                app_ui.menu_bar_packfile.set_enabled(false);

                                DiagnosticsUI::check(&app_ui, &diagnostics_ui);

                                app_ui.menu_bar_packfile.set_enabled(true);
                            }
                        }
                    }));

                    // Connect the slot and store it.
                    open_mod_action.triggered().connect(&slot_open_mod);
                }
            }
        }

        // Only if the submenu has items, we enable it.
        app_ui.packfile_open_recent.menu_action().set_visible(!app_ui.packfile_open_recent.actions().is_empty());
        app_ui.packfile_open_from_content.menu_action().set_visible(!app_ui.packfile_open_from_content.actions().is_empty());
        app_ui.packfile_open_from_data.menu_action().set_visible(!app_ui.packfile_open_from_data.actions().is_empty());
        app_ui.packfile_open_from_autosave.menu_action().set_visible(!app_ui.packfile_open_from_autosave.actions().is_empty());
    }

    /// This function takes care of the re-creation of the `MyMod` list for each game.
    pub unsafe fn build_open_mymod_submenus(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        global_search_ui: &Rc<GlobalSearchUI>
    ) {

        // First, we need to reset the menu, which basically means deleting all the game submenus and hiding them.
        app_ui.mymod_open_warhammer_3.menu_action().set_visible(false);
        app_ui.mymod_open_troy.menu_action().set_visible(false);
        app_ui.mymod_open_three_kingdoms.menu_action().set_visible(false);
        app_ui.mymod_open_warhammer_2.menu_action().set_visible(false);
        app_ui.mymod_open_warhammer.menu_action().set_visible(false);
        app_ui.mymod_open_thrones_of_britannia.menu_action().set_visible(false);
        app_ui.mymod_open_attila.menu_action().set_visible(false);
        app_ui.mymod_open_rome_2.menu_action().set_visible(false);
        app_ui.mymod_open_shogun_2.menu_action().set_visible(false);
        app_ui.mymod_open_napoleon.menu_action().set_visible(false);
        app_ui.mymod_open_empire.menu_action().set_visible(false);

        app_ui.mymod_open_warhammer_3.clear();
        app_ui.mymod_open_troy.clear();
        app_ui.mymod_open_three_kingdoms.clear();
        app_ui.mymod_open_warhammer_2.clear();
        app_ui.mymod_open_warhammer.clear();
        app_ui.mymod_open_thrones_of_britannia.clear();
        app_ui.mymod_open_attila.clear();
        app_ui.mymod_open_rome_2.clear();
        app_ui.mymod_open_shogun_2.clear();
        app_ui.mymod_open_napoleon.clear();
        app_ui.mymod_open_empire.clear();

        // If we have the "MyMod" path configured, get all the packfiles under the `MyMod` folder, separated by supported game.
        if let Some(ref mymod_base_path) = SETTINGS.read().unwrap().paths[MYMOD_BASE_PATH] {
            if let Ok(game_folder_list) = mymod_base_path.read_dir() {
                for game_folder in game_folder_list {
                    if let Ok(game_folder) = game_folder {

                        // If it's a valid folder, and it's in our supported games list, get all the PackFiles inside it and create an open action for them.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        let is_supported = SUPPORTED_GAMES.get_games().iter().filter_map(|x| if x.get_supports_editing() { Some(x.get_game_key_name()) } else { None }).any(|x| x == game_folder_name);
                        if game_folder.path().is_dir() && is_supported {
                            let game_submenu = match &*game_folder_name {
                                KEY_WARHAMMER_3 => &app_ui.mymod_open_warhammer_3,
                                KEY_TROY => &app_ui.mymod_open_troy,
                                KEY_THREE_KINGDOMS => &app_ui.mymod_open_three_kingdoms,
                                KEY_WARHAMMER_2 => &app_ui.mymod_open_warhammer_2,
                                KEY_WARHAMMER => &app_ui.mymod_open_warhammer,
                                KEY_THRONES_OF_BRITANNIA => &app_ui.mymod_open_thrones_of_britannia,
                                KEY_ATTILA => &app_ui.mymod_open_attila,
                                KEY_ROME_2 => &app_ui.mymod_open_rome_2,
                                KEY_SHOGUN_2 => &app_ui.mymod_open_shogun_2,
                                KEY_NAPOLEON => &app_ui.mymod_open_napoleon,
                                KEY_EMPIRE => &app_ui.mymod_open_empire,
                                _ => unimplemented!()
                            };

                            if let Ok(game_folder_files) = game_folder.path().read_dir() {
                                let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|x| x.unwrap().path()).collect();
                                game_folder_files_sorted.sort();

                                for pack_file in &game_folder_files_sorted {
                                    if pack_file.is_file() && pack_file.extension().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy() == "pack" {
                                        let pack_file = pack_file.clone();
                                        let mod_name = pack_file.file_name().unwrap().to_string_lossy();
                                        let open_mod_action = game_submenu.add_action_q_string(&QString::from_std_str(&mod_name));

                                        // Create the slot for that action.
                                        let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                                            app_ui,
                                            pack_file_contents_ui,
                                            global_search_ui,
                                            diagnostics_ui,
                                            game_folder_name => move |_| {
                                            if Self::are_you_sure(&app_ui, false) {
                                                if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[pack_file.to_path_buf()], &game_folder_name) {
                                                    return show_dialog(&app_ui.main_window, error, false);
                                                }

                                                if SETTINGS.read().unwrap().settings_bool["diagnostics_trigger_on_open"] {

                                                    // Disable the top menus before triggering the check. Otherwise, we may end up in a crash.
                                                    app_ui.menu_bar_mymod.set_enabled(false);

                                                    DiagnosticsUI::check(&app_ui, &diagnostics_ui);

                                                    app_ui.menu_bar_mymod.set_enabled(true);
                                                }
                                            }
                                        }));

                                        open_mod_action.triggered().connect(&slot_open_mod);
                                    }
                                }
                            }

                            // Only if the submenu has items, we show it to the big menu.
                            if game_submenu.actions().count_0a() > 0 {
                                game_submenu.menu_action().set_visible(true);
                            }
                        }
                    }
                }
            }
        }
    }

    /// This function checks if there is any newer version of RPFM released.
    ///
    /// If the `use_dialog` is false, we make the checks in the background, and pop up a dialog only in case there is an update available.
    pub unsafe fn check_updates(app_ui: &Rc<Self>, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates);

        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_checker"),
            &qtr("update_searching"),
            QFlags::from(q_message_box::StandardButton::Close),
            &app_ui.main_window,
        );

        let close_button = dialog.button(q_message_box::StandardButton::Close);
        let update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
        update_button.set_enabled(false);

        dialog.set_modal(true);
        if use_dialog {
            dialog.show();
        }

        let response = CentralCommand::recv_try(&receiver);
        let message = match response {
            Response::APIResponse(response) => {
                match response {
                    APIResponse::SuccessNewStableUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_stable_update", &[&last_release])
                    }
                    APIResponse::SuccessNewBetaUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_beta_update", &[&last_release])
                    }
                    APIResponse::SuccessNewUpdateHotfix(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_update_hotfix", &[&last_release])
                    }
                    APIResponse::SuccessNoUpdate => {
                        if !use_dialog { return; }
                        qtr("api_response_success_no_update")
                    }
                    APIResponse::SuccessUnknownVersion => {
                        if !use_dialog { return; }
                        qtr("api_response_success_unknown_version")
                    }
                    APIResponse::Error => {
                        if !use_dialog { return; }
                        qtr("api_response_error")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMainProgram);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::Success => {
                    let restart_button = dialog.add_button_q_string_button_role(&qtr("restart_button"), q_message_box::ButtonRole::ApplyRole);

                    let changelog_path = RPFM_PATH.join(CHANGELOG_FILE);
                    dialog.set_text(&qtre("update_success_main_program", &[&changelog_path.to_string_lossy()]));
                    restart_button.set_enabled(true);
                    close_button.set_enabled(true);

                    // This closes the program and triggers a restart in the launcher.
                    if dialog.exec() == 1 {
                        exit(10);
                    }
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(&error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }
    }

    /// This function checks if there is any newer version of RPFM's schemas released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub unsafe fn check_schema_updates(app_ui: &Rc<Self>, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);

        // Create the dialog to show the response and configure it.
        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_schema_checker"),
            &qtr("update_searching"),
            QFlags::from(q_message_box::StandardButton::Close),
            &app_ui.main_window,
        );

        let close_button = dialog.button(q_message_box::StandardButton::Close);
        let update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
        update_button.set_enabled(false);

        dialog.set_modal(true);
        if use_dialog {
            dialog.show();
        }

        // When we get a response, act depending on the kind of response we got.
        let response_thread = CentralCommand::recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseSchema(ref response) => {
                match response {
                    APIResponseSchema::NewUpdate => {
                        update_button.set_enabled(true);
                        qtr("schema_new_update")
                    }
                    APIResponseSchema::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("schema_no_update")
                    }
                    APIResponseSchema::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_schema")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
        };

        // If we hit "Update", try to update the schemas.
        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateSchemas);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::Success => {
                    dialog.set_text(&qtr("schema_update_success"));
                    close_button.set_enabled(true);
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(&error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }
    }

    /// This function checks if there is any newer version of RPFM's messages released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub unsafe fn check_message_updates(app_ui: &Rc<Self>, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckMessageUpdates);

        // Create the dialog to show the response and configure it.
        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_messages_checker"),
            &qtr("update_searching"),
            QFlags::from(q_message_box::StandardButton::Close),
            &app_ui.main_window,
        );

        let close_button = dialog.button(q_message_box::StandardButton::Close);
        let update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
        update_button.set_enabled(false);

        dialog.set_modal(true);
        if use_dialog {
            dialog.show();
        }

        // When we get a response, act depending on the kind of response we got.
        let response_thread = CentralCommand::recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseTips(ref response) => {
                match response {
                    APIResponseTips::NewUpdate => {
                        update_button.set_enabled(true);
                        qtr("messages_new_update")
                    }
                    APIResponseTips::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("messages_no_update")
                    }
                    APIResponseTips::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_messages")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
        };

        // If we hit "Update", try to update the messages.
        if use_dialog {
            dialog.set_text(&message);
            if dialog.exec() == 0 {
                let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMessages);

                dialog.show();
                dialog.set_text(&qtr("update_in_prog"));
                update_button.set_enabled(false);
                close_button.set_enabled(false);

                let response = CentralCommand::recv_try(&receiver);
                match response {
                    Response::Success => {
                        dialog.set_text(&qtr("messages_update_success"));
                        close_button.set_enabled(true);
                    },
                    Response::Error(error) => {
                        dialog.set_text(&QString::from_std_str(&error.to_string()));
                        close_button.set_enabled(true);
                    }
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        } else {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMessages);
            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::Success => log_to_status_bar("messages_update_success"),
                Response::Error(error) => log_to_status_bar(&error.to_string()),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }
    }

    /// This function is used to open ANY supported PackedFiles in a DockWidget, docked in the Main Window.
    pub unsafe fn open_packedfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        packed_file_path: Option<Vec<String>>,
        is_preview: bool,
        is_external: bool,
        mut data_source: DataSource,
    ) {

        // Conditions to open:
        // - Local PackedFile && the treeview not being locked.
        // - Remote PackedFile.
        let should_be_opened = if let DataSource::PackFile = data_source {
            !UI_STATE.get_packfile_contents_read_only()
        } else {
            true
        };

        if should_be_opened {
            let item_type = match packed_file_path {
                Some(packed_file_path) => TreePathType::File(packed_file_path),

                // If none path has been provided, we have to do some magic to find out what we're opening.
                None => {
                    match data_source {
                        DataSource::PackFile => {
                            let selected_items = pack_file_contents_ui.packfile_contents_tree_view.get_item_types_from_selection(true);
                            if selected_items.len() == 1 { selected_items[0].clone() } else { return }
                        },
                        DataSource::ParentFiles |
                        DataSource::GameFiles |
                        DataSource::AssKitFiles => {
                            let selected_items = dependencies_ui.dependencies_tree_view.get_item_types_from_selection(true);
                            if selected_items.len() == 1 {
                                if let Some(data_source_tree) = dependencies_ui.dependencies_tree_view.get_root_source_type_from_selection(true) {
                                    data_source = data_source_tree;
                                    selected_items[0].clone()
                                } else { return }
                            } else { return }
                        }

                        DataSource::ExternalFile => unimplemented!(),
                    }
                }
            };

            if let TreePathType::File(ref path) = item_type {

                // Close all preview views except the file we're opening.
                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let open_path = packed_file_view.get_ref_path();
                    let index = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if (data_source != packed_file_view.get_data_source() ||
                        (data_source == packed_file_view.get_data_source() && *open_path != *path)) &&
                        packed_file_view.get_is_preview() && index != -1 {
                        app_ui.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the file we want to open is already open, or it's hidden, we show it/focus it, instead of opening it again.
                // If it was a preview, then we mark it as full. Index == -1 means it's not in a tab.
                if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *path && x.get_data_source() == data_source) {
                    if !is_external {
                        let index = app_ui.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                        // If we're trying to open as preview something already open as full, we don't do anything.
                        if !(index != -1 && is_preview && !tab_widget.get_is_preview()) {
                            tab_widget.set_is_preview(is_preview);
                        }

                        if index == -1 {
                            let icon_type = IconType::File(path.to_vec());
                            let icon = icon_type.get_icon_from_path();
                            app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &QString::from_std_str(""));
                        }

                        app_ui.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                        Self::update_views_names(app_ui);
                        return;
                    }
                }

                // If we have a PackedFile open, but we want to open it as a external file, close it here.
                if is_external && UI_STATE.get_open_packedfiles().iter().any(|x| *x.get_ref_path() == *path && x.get_data_source() == data_source) {
                    if let Err(error) = Self::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, data_source, true) {
                        show_dialog(&app_ui.main_window, error, false);
                    }
                }

                let mut tab = PackedFileView::default();
                tab.get_mut_widget().set_parent(&app_ui.tab_bar_packed_file);
                tab.get_mut_widget().set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
                tab.set_path(path);

                // Any table banned or from out of our PackFile should not be editable.
                if let DataSource::PackFile = data_source {
                    if GAME_SELECTED.read().unwrap().is_packedfile_banned(path) {
                        tab.set_is_read_only(true);
                    } else {
                        tab.set_is_read_only(false);
                    }
                } else {
                    tab.set_is_read_only(true);
                }

                tab.set_data_source(data_source);

                if !is_external {
                    tab.set_is_preview(is_preview);
                    let icon_type = IconType::File(path.to_vec());
                    let icon = icon_type.get_icon_from_path();

                    // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                    let packed_file_type = get_packed_file_type(path);

                    match packed_file_type {

                        // If the file is an AnimFragment PackedFile...
                        PackedFileType::AnimFragment => {
                            match PackedFileAnimFragmentView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                                    }
                                },

                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::AnimFragmentDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is an AnimPack PackedFile...
                        PackedFileType::AnimPack => {
                            match PackedFileAnimPackView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::AnimPackDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is an AnimTable PackedFile...
                        PackedFileType::AnimTable => {
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::AnimTableDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a CA_VP8 PackedFile...
                        PackedFileType::CaVp8 => {
                            match PackedFileCaVp8View::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::CaVp8Decode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a Loc PackedFile...
                        PackedFileType::Loc => {
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::LocDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a DB PackedFile...
                        PackedFileType::DB => {
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => {

                                    // Try to get the data of the table to send it for decoding.
                                    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFileRawData(path.to_vec()));
                                    let response = CentralCommand::recv(&receiver);
                                    let data = match response {
                                        Response::VecU8(data) => data,
                                        Response::Error(_) => return show_dialog(&app_ui.main_window, ErrorKind::DBTableDecode(format!("{}", error)), false),
                                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                                    };

                                    return show_dialog_decode_button(app_ui.main_window.static_upcast::<qt_widgets::QWidget>().as_ptr(), ErrorKind::DBTableDecode(format!("{}", error)), &path[1], &data);
                                },
                            }
                        }

                        // If the file is a MatchedCombat PackedFile...
                        PackedFileType::MatchedCombat => {
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::MatchedCombatDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a Text PackedFile...
                        PackedFileType::Text(_) => {
                            match PackedFileTextView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a RigidModel PackedFile...
                        #[cfg(feature = "support_rigidmodel")]
                        PackedFileType::RigidModel => {
                            if SETTINGS.read().unwrap().settings_bool["enable_rigidmodel_editor"] {
                                match PackedFileRigidModelView::new_view(&mut tab) {
                                    Ok(packed_file_info) => {

                                       // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                        // Fix the tips view.
                                        let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);

                                        if let Some(packed_file_info) = packed_file_info {
                                            if data_source == DataSource::PackFile {
                                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                            }
                                        }
                                    },
                                    Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::RigidModelDecode(format!("{}", error)), false),
                                }
                            }
                        }

                        // If the file is a Image PackedFile, ignore failures while opening.
                        PackedFileType::Image => {
                            if let Ok(packed_file_info) = PackedFileImageView::new_view(&mut tab) {

                                // Add the file to the 'Currently open' list and make it visible.
                                app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                // Fix the tips view.
                                let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.push(tab);

                                if data_source == DataSource::PackFile {
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                                }
                            }
                        }

                        // If the file is a UI Component...
                        #[cfg(feature = "support_uic")]
                        PackedFileType::UIC => {
                            match PackedFileUICView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::UICDecode(format!("{}", error)), false),
                            }
                        }

                        PackedFileType::UnitVariant => {
                            match PackedFileUnitVariantView::new_view(&mut tab) {
                                Ok(packed_file_info) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if let Some(packed_file_info) = packed_file_info {
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                        }
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::UnitVariantDecode(format!("{}", error)), false),
                            }
                        }

                        PackedFileType::ESF => {
                            if SETTINGS.read().unwrap().settings_bool["enable_esf_editor"] {
                                match PackedFileESFView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                                    Ok(packed_file_info) => {

                                        // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                        // Fix the tips view.
                                        let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);

                                        if let Some(packed_file_info) = packed_file_info {
                                            if data_source == DataSource::PackFile {
                                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), data_source);
                                            }
                                        }
                                    },
                                    Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::ESFDecode(format!("{}", error)), false),
                                }
                            }
                        }

                        PackedFileType::PackFile => {
                            let path_str = &tab.get_path()[1..].join("/");
                            let path = PathBuf::from(path_str.to_owned());
                            match PackFileExtraView::new_view(&mut tab, app_ui, pack_file_contents_ui, path) {
                                Ok(_) => {
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(&path_str));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                                    UI_STATE.set_open_packedfiles().push(tab);
                                }
                                Err(error) => show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        // Ignore anything else.
                        _ => {}
                    }
                }

                // If it's external, we just create a view with just one button: "Stop Watching External File".
                else {
                    let icon_type = IconType::File(path.to_vec());
                    let icon = icon_type.get_icon_from_path();
                    let path = Rc::new(RefCell::new(path.to_vec()));

                    match PackedFileExternalView::new_view(&path, app_ui,  &mut tab, pack_file_contents_ui) {
                        Ok(_) => {

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                        }
                        Err(error) => show_dialog(&app_ui.main_window, error, false),
                    }
                }
            }
        }

        Self::update_views_names(app_ui);

        // Try to paint the diagnostics results, if any.
        for diagnostic_type in UI_STATE.get_diagnostics().get_ref_diagnostics() {
            DiagnosticsUI::paint_diagnostics_to_table(app_ui, diagnostic_type);
        }
    }

    /// This function is used to open the PackedFile Decoder.
    pub unsafe fn open_decoder(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>
    ) {

        // If we don't have an schema, don't even try it.
        if SCHEMA.read().unwrap().is_none() {
            return show_dialog(&app_ui.main_window, ErrorKind::SchemaNotFound, false);
        }

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {
            let mut selected_items = <QBox<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &mut selected_items[0] } else { return };
            if let TreePathType::File(ref mut path) = item_type {
                let mut fake_path = path.to_vec();
                *fake_path.last_mut().unwrap() = fake_path.last().unwrap().to_owned() + DECODER_EXTENSION;

                // Close all preview views except the file we're opening.
                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let open_path = packed_file_view.get_ref_path();
                    let index = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if *open_path != *path && packed_file_view.get_is_preview() && index != -1 {
                        app_ui.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // Close all preview views except the file we're opening. The path used for the decoder is empty.
                let name = qtr("decoder_title");
                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let open_path = packed_file_view.get_ref_path();
                    let index = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                        app_ui.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the decoder is already open, or it's hidden, we show it/focus it, instead of opening it again.
                if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == fake_path) {
                    let index = app_ui.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                    if index == -1 {
                        let icon_type = IconType::PackFile(true);
                        let icon = icon_type.get_icon_from_path();
                        app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                    }

                    app_ui.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                    return;
                }

                // If it's not already open/hidden, we create it and add it as a new tab.
                let mut tab = PackedFileView::default();
                tab.get_mut_widget().set_parent(&app_ui.tab_bar_packed_file);
                tab.set_is_preview(false);
                let icon_type = IconType::PackFile(true);
                let icon = icon_type.get_icon_from_path();
                tab.set_path(path);

                match PackedFileDecoderView::new_view(&mut tab, pack_file_contents_ui, app_ui) {
                    Ok(_) => {

                        // Add the decoder to the 'Currently open' list and make it visible.
                        app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                        app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                        let mut open_list = UI_STATE.set_open_packedfiles();
                        open_list.push(tab);
                    },
                    Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::DecoderDecode(format!("{}", error)), false),
                }
            }
        }

        Self::update_views_names(app_ui);
    }

    /// This function is used to open the dependency manager.
    pub unsafe fn open_dependency_manager(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            // Close all preview views except the file we're opening. The path used for the manager is empty.
            let path = vec![RESERVED_NAME_DEPENDENCIES_MANAGER.to_owned()];
            let name = qtr("table_dependency_manager_title");
            for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                let open_path = packed_file_view.get_ref_path();
                let index = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                    app_ui.tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the manager is already open, or it's hidden, we show it/focus it, instead of opening it again.
            if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == path) {
                let index = app_ui.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                if index == -1 {
                    let icon_type = IconType::PackFile(true);
                    let icon = icon_type.get_icon_from_path();
                    app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                }

                app_ui.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                return;
            }

            // If it's not already open/hidden, we create it and add it as a new tab.
            let mut tab = PackedFileView::default();
            tab.get_mut_widget().set_parent(&app_ui.tab_bar_packed_file);
            tab.set_is_preview(false);
            tab.set_path(&path);
            let icon_type = IconType::PackFile(true);
            let icon = icon_type.get_icon_from_path();

            match DependenciesManagerView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui) {
                Ok(_) => {

                    // Add the manager to the 'Currently open' list and make it visible.
                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                    UI_STATE.set_open_packedfiles().push(tab);
                },
                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
            }
        }

        Self::update_views_names(app_ui);
    }

    /// This function is used to open the settings embedded into a PackFile.
    pub unsafe fn open_packfile_settings(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            // Close all preview views except the file we're opening. The path used for the settings is reserved.
            let path = vec![RESERVED_NAME_SETTINGS.to_owned()];
            let name = qtr("settings");
            for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                let open_path = packed_file_view.get_ref_path();
                let index = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if *open_path != path && packed_file_view.get_is_preview() && index != -1 {
                    app_ui.tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the settings are already open, or are hidden, we show them/focus them, instead of opening them again.
            if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == path) {
                let index = app_ui.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                if index == -1 {
                    let icon_type = IconType::PackFile(true);
                    let icon = icon_type.get_icon_from_path();
                    app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                }

                app_ui.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                return;
            }

            // If it's not already open/hidden, we create it and add it as a new tab.
            let mut tab = PackedFileView::default();
            tab.get_mut_widget().set_parent(&app_ui.tab_bar_packed_file);
            tab.set_is_preview(false);
            let icon_type = IconType::PackFile(true);
            let icon = icon_type.get_icon_from_path();
            tab.set_path(&path);

            match PackFileSettingsView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                Ok(_) => {
                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                    UI_STATE.set_open_packedfiles().push(tab);
                },
                Err(error) => return show_dialog(&app_ui.main_window, ErrorKind::PackFileSettingsDecode(format!("{}", error)), false),
            }
        }

        Self::update_views_names(app_ui);
    }

    /// This function is the one that takes care of the creation of different PackedFiles.
    pub unsafe fn new_packed_file(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>, packed_file_type: PackedFileType) {

        // DB Files require the dependencies cache to be generated, and the schemas to be downloaded.
        if packed_file_type == PackedFileType::DB {

            if SCHEMA.read().unwrap().is_none() {
                return show_dialog(&app_ui.main_window, ErrorKind::SchemaNotFound, false);
            }

            let receiver = CENTRAL_COMMAND.send_background(Command::IsThereADependencyDatabase(false));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Bool(it_is) => if !it_is { return show_dialog(&app_ui.main_window, ErrorKind::DependenciesCacheNotGeneratedorOutOfDate, false); },
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // Create the "New PackedFile" dialog and wait for his data (or a cancellation). If we receive None, we do nothing. If we receive Some,
        // we still have to check if it has been any error during the creation of the PackedFile (for example, no definition for DB Tables).
        if let Some(new_packed_file) = Self::new_packed_file_dialog(app_ui, packed_file_type) {
            match new_packed_file {
                Ok(mut new_packed_file) => {

                    // First we make sure the name is correct, and fix it if needed.
                    match new_packed_file {
                        NewPackedFile::AnimPack(ref mut name) |
                        NewPackedFile::Loc(ref mut name) |
                        NewPackedFile::Text(ref mut name, _) |
                        NewPackedFile::DB(ref mut name, _, _) => {

                            // If the name is_empty, stop.
                            if name.is_empty() {
                                return show_dialog(&app_ui.main_window, ErrorKind::EmptyInput, false)
                            }

                            // Fix their name termination if needed.
                            if let PackedFileType::AnimPack = packed_file_type {
                                if !name.ends_with(animpack::EXTENSION) { name.push_str(animpack::EXTENSION); }
                            }
                            if let PackedFileType::Loc = packed_file_type {
                                if !name.ends_with(loc::EXTENSION) { name.push_str(loc::EXTENSION); }
                            }
                            if let PackedFileType::Text(_) = packed_file_type {
                                if !text::EXTENSIONS.iter().any(|(x, _)| name.ends_with(x)) {
                                    name.push_str(".txt");
                                }
                            }
                        }
                    }

                    if let NewPackedFile::Text(ref mut name, ref mut text_type) = new_packed_file {
                        if let Some((_, text_type_real)) = text::EXTENSIONS.iter().find(|(x, _)| name.ends_with(x)) {
                            *text_type = *text_type_real
                        }
                    }

                    // If we reach this place, we got all alright.
                    match new_packed_file {
                        NewPackedFile::AnimPack(ref name) |
                        NewPackedFile::Loc(ref name) |
                        NewPackedFile::Text(ref name, _) |
                        NewPackedFile::DB(ref name, _, _) => {

                            // Get the currently selected paths (or the complete path, in case of DB Tables),
                            // and only continue if there is only one and it's not empty.
                            let selected_paths = pack_file_contents_ui.packfile_contents_tree_view.get_path_from_selection();
                            let complete_path = if let NewPackedFile::DB(name, table,_) = &new_packed_file {
                                vec!["db".to_owned(), table.to_owned(), name.to_owned()]
                            }
                            else {

                                // We want to be able to write relative paths with this so, if a `/` is detected, split the name.
                                if selected_paths.len() == 1 {
                                    let mut complete_path = selected_paths[0].to_vec();
                                    complete_path.append(&mut (name.split('/').map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>()));
                                    complete_path
                                }
                                else { vec![] }
                            };

                            // If and only if, after all these checks, we got a path to save the PackedFile, we continue.
                            if !complete_path.is_empty() {

                                // Check if the PackedFile already exists, and report it if so.
                                let receiver = CENTRAL_COMMAND.send_background(Command::PackedFileExists(complete_path.to_vec()));
                                let response = CentralCommand::recv(&receiver);
                                let exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                                if exists { return show_dialog(&app_ui.main_window, ErrorKind::FileAlreadyInPackFile, false)}

                                // Get the response, just in case it failed.
                                let receiver = CENTRAL_COMMAND.send_background(Command::NewPackedFile(complete_path.to_vec(), new_packed_file));
                                let response = CentralCommand::recv(&receiver);
                                match response {
                                    Response::Success => {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(complete_path.to_vec()); 1]), DataSource::PackFile);
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::File(complete_path); 1]), DataSource::PackFile);
                                        UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                                    }

                                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                                }
                            }
                        }
                    }
                }
                Err(error) => show_dialog(&app_ui.main_window, error, false),
            }
        }
    }

    /// This function creates a new PackedFile based on the current path selection, being:
    /// - `db/xxxx` -> DB Table.
    /// - `text/xxxx` -> Loc Table.
    /// - `script/xxxx` -> Lua PackedFile.
    /// - `variantmeshes/variantmeshdefinitions/xxxx` -> VMD PackedFile.
    /// The name used for each packfile is a generic one.
    pub unsafe fn new_queek_packed_file(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the currently selected path and, depending on the selected path, generate one packfile or another.
        let selected_items = <QBox<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
        if selected_items.len() == 1 {
            let item = &selected_items[0];

            let path = match item {
                TreePathType::File(ref path) => {
                    let mut path = path.to_vec();
                    path.pop();
                    path
                },
                TreePathType::Folder(path) => path.to_vec(),
                _ => return show_dialog(&app_ui.main_window, ErrorKind::NoQueekPackedFileHere, false),
            };

            if let Some(mut name) = Self::new_packed_file_name_dialog(app_ui) {

                // DB Check.
                let (new_path, new_packed_file) = if path.starts_with(&["db".to_owned()]) && (path.len() == 2 || path.len() == 3) {
                    let new_path = vec!["db".to_owned(), path[1].to_owned(), name];
                    let table = &path[1];

                    let receiver = CENTRAL_COMMAND.send_background(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CentralCommand::recv(&receiver);
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    };

                    let new_packed_file = NewPackedFile::DB(new_path.last().unwrap().to_owned(), table.to_owned(), version);
                    (new_path, new_packed_file)
                }

                // Loc Check.
                else if path.starts_with(&["text".to_owned()]) && !path.is_empty() {
                    if !name.ends_with(".loc") { name.push_str(".loc"); }
                    let mut new_path = path.to_vec();
                    let mut name = name.split('/').map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>();
                    new_path.append(&mut name);

                    let new_packed_file = NewPackedFile::Loc(new_path.last().unwrap().to_owned());
                    (new_path, new_packed_file)
                }

                // Lua Check.
                else if path.starts_with(&["script".to_owned()]) && !path.is_empty() {
                    if !name.ends_with(".lua") { name.push_str(".lua"); }
                    let mut new_path = path.to_vec();
                    let mut name = name.split('/').map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>();
                    new_path.append(&mut name);

                    let new_packed_file = NewPackedFile::Text(new_path.last().unwrap().to_owned(), TextType::Lua);
                    (new_path, new_packed_file)
                }

                // VMD Check.
                else if path.starts_with(&["variantmeshes".to_owned(), "variantmeshdefinitions".to_owned()]) && !path.is_empty() {
                    if !name.ends_with(".variantmeshdefinition") { name.push_str(".variantmeshdefinition"); }
                    let mut new_path = path.to_vec();
                    let mut name = name.split('/').map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>();
                    new_path.append(&mut name);

                    let new_packed_file = NewPackedFile::Text(new_path.last().unwrap().to_owned(), TextType::Xml);
                    (new_path, new_packed_file)
                }

                // Neutral Check, for folders without a predefined type.
                else {
                    return show_dialog(&app_ui.main_window, ErrorKind::NoQueekPackedFileHere, false);
                };

                // Check if the PackedFile already exists, and report it if so.
                let receiver = CENTRAL_COMMAND.send_background(Command::PackedFileExists(new_path.to_vec()));
                let response = CentralCommand::recv(&receiver);
                let exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                if exists { return show_dialog(&app_ui.main_window, ErrorKind::FileAlreadyInPackFile, false)}

                // Create the PackFile.
                let receiver = CENTRAL_COMMAND.send_background(Command::NewPackedFile(new_path.to_vec(), new_packed_file));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => {
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(new_path.to_vec()); 1]), DataSource::PackFile);
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::File(new_path); 1]), DataSource::PackFile);
                        UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                    }
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }
    }

    /// This function creates the entire "New Folder" dialog.
    ///
    /// It returns the new name of the Folder, or None if the dialog is canceled or closed.
    pub unsafe fn new_folder_dialog(app_ui: &Rc<Self>) -> Option<String> {
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("new_folder"));
        dialog.set_modal(true);
        dialog.resize_2a(600, 20);

        let main_grid = create_grid_layout(dialog.static_upcast());

        let new_folder_line_edit = QLineEdit::new();
        new_folder_line_edit.set_text(&qtr("new_folder_default"));
        let new_folder_button = QPushButton::from_q_string(&qtr("new_folder"));

        main_grid.add_widget_5a(& new_folder_line_edit, 0, 0, 1, 1);
        main_grid.add_widget_5a(& new_folder_button, 0, 1, 1, 1);
        new_folder_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 { Some(new_folder_line_edit.text().to_std_string()) }
        else { None }
    }

    /// This function creates all the "New PackedFile" dialogs.
    ///
    /// It returns the type/name of the new file, or None if the dialog is canceled or closed.
    pub unsafe fn new_packed_file_dialog(app_ui: &Rc<Self>, packed_file_type: PackedFileType) -> Option<Result<NewPackedFile>> {

        // Create and configure the "New PackedFile" Dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        match packed_file_type {
            PackedFileType::AnimPack => dialog.set_window_title(&qtr("new_animpack_file")),
            PackedFileType::DB => dialog.set_window_title(&qtr("new_db_file")),
            PackedFileType::Loc => dialog.set_window_title(&qtr("new_loc_file")),
            PackedFileType::Text(_) => dialog.set_window_title(&qtr("new_txt_file")),
            _ => unimplemented!(),
        }
        dialog.set_modal(true);
        dialog.resize_2a(600, 20);

        // Create the main Grid and his widgets.
        let main_grid = create_grid_layout(dialog.static_upcast());
        let name_line_edit = QLineEdit::from_q_widget(&dialog);
        let table_filter_line_edit = QLineEdit::from_q_widget(&dialog);
        let create_button = QPushButton::from_q_string_q_widget(&qtr("gen_loc_create"), &dialog);
        let table_dropdown = QComboBox::new_1a(&dialog);
        let table_filter = QSortFilterProxyModel::new_1a(&dialog);
        let table_model = QStandardItemModel::new_1a(&dialog);

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileName);
        let response = CentralCommand::recv(&receiver);
        let packfile_name = if let Response::String(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
        let packfile_name = if packfile_name.to_lowercase().ends_with(".pack") {
            let mut packfile_name = packfile_name.to_owned();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name
        } else { packfile_name };

        name_line_edit.set_text(&QString::from_std_str(&packfile_name));
        table_dropdown.set_model(&table_model);
        table_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        // Add all the widgets to the main grid, except those specific for a PackedFileType.
        main_grid.add_widget_5a(&name_line_edit, 0, 0, 1, 1);
        main_grid.add_widget_5a(&create_button, 0, 1, 1, 1);

        // If it's a DB Table, add its widgets, and populate the table list.
        if let PackedFileType::DB = packed_file_type {
            let receiver = CENTRAL_COMMAND.send_background(Command::GetTableListFromDependencyPackFile);
            let response = CentralCommand::recv(&receiver);
            let tables = if let Response::VecString(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
            match *SCHEMA.read().unwrap() {
                Some(ref schema) => {

                    // Add every table to the dropdown if exists in the dependency database.
                    schema.get_ref_versioned_file_db_all().iter()
                        .filter_map(|x| if let VersionedFile::DB(name, _) = x { Some(name) } else { None })
                        .filter(|x| tables.contains(x))
                        .for_each(|x| table_dropdown.add_item_q_string(&QString::from_std_str(&x)));
                    table_filter.set_source_model(&table_model);
                    table_dropdown.set_model(&table_filter);

                    main_grid.add_widget_5a(&table_dropdown, 1, 0, 1, 1);
                    main_grid.add_widget_5a(&table_filter_line_edit, 2, 0, 1, 1);
                }
                None => return Some(Err(ErrorKind::SchemaNotFound.into())),
            }
        }

        // Remember to hide the unused stuff. Otherwise, it'll be shown out of place due to parenting.
        else {
            table_dropdown.set_visible(false);
            table_filter_line_edit.set_visible(false);
        }

        // What happens when we search in the filter.
        let table_filter_line_edit = table_filter_line_edit.as_ptr();
        let slot_table_filter_change_text = SlotOfQString::new(&dialog, move |_| {
            let pattern = QRegExp::new_1a(&table_filter_line_edit.text());
            table_filter.set_filter_reg_exp_q_reg_exp(&pattern);
        });

        // What happens when we hit the "Create" button.
        create_button.released().connect(dialog.slot_accept());

        // What happens when we edit the search filter.
        table_filter_line_edit.text_changed().connect(&slot_table_filter_change_text);

        // Show the Dialog and, if we hit the "Create" button, return the corresponding NewPackedFileType.
        if dialog.exec() == 1 {
            let packed_file_name = name_line_edit.text().to_std_string();
            match packed_file_type {
                PackedFileType::AnimPack => Some(Ok(NewPackedFile::AnimPack(packed_file_name))),
                PackedFileType::DB => {
                    let table = table_dropdown.current_text().to_std_string();
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CentralCommand::recv(&receiver);
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return Some(Err(error)),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    };
                    Some(Ok(NewPackedFile::DB(packed_file_name, table, version)))
                },
                PackedFileType::Loc => Some(Ok(NewPackedFile::Loc(packed_file_name))),
                PackedFileType::Text(_) => Some(Ok(NewPackedFile::Text(packed_file_name, TextType::Plain))),
                _ => unimplemented!(),
            }
        }

        // Otherwise, return None.
        else { None }
    }

    /// This function creates the "New PackedFile's Name" dialog when creating a new QueeK PackedFile.
    ///
    /// It returns the new name of the PackedFile, or `None` if the dialog is canceled or closed.
    unsafe fn new_packed_file_name_dialog(app_ui: &Rc<Self>) -> Option<String> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("new_packedfile_name"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);

        let main_grid = create_grid_layout(dialog.static_upcast());
        let name_line_edit = QLineEdit::new();
        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileName);
        let response = CentralCommand::recv(&receiver);
        let packfile_name = if let Response::String(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
        let packfile_name = if packfile_name.to_lowercase().ends_with(".pack") {
            let mut packfile_name = packfile_name.to_owned();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name
        } else { packfile_name };

        name_line_edit.set_text(&QString::from_std_str(&packfile_name));

        main_grid.add_widget_5a(&name_line_edit, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 1, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = name_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(name_line_edit.text().to_std_string()) }
        } else { None }
    }

    /// This function creates the entire "Merge Tables" dialog. It returns the stuff set in it.
    pub unsafe fn merge_tables_dialog(app_ui: &Rc<Self>) -> Option<(String, bool)> {

        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("packedfile_merge_tables"));
        dialog.set_modal(true);

        // Create the main Grid.
        let main_grid = create_grid_layout(dialog.static_upcast());
        let name_line_edit = QLineEdit::new();

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileName);
        let response = CentralCommand::recv(&receiver);
        let packfile_name = if let Response::String(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
        let packfile_name = if packfile_name.to_lowercase().ends_with(".pack") {
            let mut packfile_name = packfile_name.to_owned();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name
        } else { packfile_name };

        name_line_edit.set_text(&QString::from_std_str(&packfile_name));

        let delete_source_tables = QCheckBox::from_q_string(&qtr("merge_tables_delete_option"));

        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));
        main_grid.add_widget_5a(&name_line_edit, 0, 0, 1, 1);
        main_grid.add_widget_5a(&delete_source_tables, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 2, 0, 1, 1);

        // What happens when we hit the "Search" button.
        accept_button.released().connect(dialog.slot_accept());

        // Execute the dialog.
        if dialog.exec() == 1 {
            let text = name_line_edit.text().to_std_string();
            let delete_source_tables = delete_source_tables.is_checked();
            if !text.is_empty() { Some((text, delete_source_tables)) }
            else { None }
        }

        // Otherwise, return None.
        else { None }
    }

    /// Update the PackedFileView names, to ensure we have no collisions.
    pub unsafe fn update_views_names(&self) {

        // We also have to check for colliding packedfile names, so we can use their full path instead.
        let mut names = HashMap::new();
        let open_packedfiles = UI_STATE.get_open_packedfiles();
        for packed_file_view in open_packedfiles.iter() {
            let widget = packed_file_view.get_mut_widget();
            if self.tab_bar_packed_file.index_of(widget) != -1 {

                // Reserved PackedFiles should have special names.
                let path = packed_file_view.get_ref_path();
                if *path == [RESERVED_NAME_NOTES.to_owned()] {
                    names.insert("Notes".to_owned(), 1);
                } else if let Some(name) = path.last() {
                    match names.get_mut(name) {
                        Some(name) => *name += 1,
                        None => { names.insert(name.to_owned(), 1); },
                    }
                }
            }
        }

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            let widget = packed_file_view.get_mut_widget();
            let widget_name = if *packed_file_view.get_ref_path() == [RESERVED_NAME_NOTES.to_owned()] {
                "Notes".to_owned()
            } else if let Some(widget_name) = packed_file_view.get_ref_path().last() {
                widget_name.to_owned()
            } else {
                "".to_owned()
            };

            if let Some(count) = names.get(&widget_name) {
                let mut name = String::new();
                match packed_file_view.get_data_source() {
                    DataSource::PackFile => {},
                    DataSource::ParentFiles => name.push_str("Parent"),
                    DataSource::GameFiles => name.push_str("Game"),
                    DataSource::AssKitFiles => name.push_str("AssKit"),
                    DataSource::ExternalFile => name.push_str("External"),
                }

                if !name.is_empty() {
                    if packed_file_view.get_is_read_only() {
                        name.push_str("-RO:");
                    } else  {
                        name.push(':');
                    }
                }

                if count > &1 {
                    name.push_str(&packed_file_view.get_ref_path().join("/"));
                } else {
                    name.push_str(&widget_name.to_owned());
                };

                if packed_file_view.get_is_preview() {
                    name.push_str(" (Preview)");
                }

                let index = self.tab_bar_packed_file.index_of(widget);
                self.tab_bar_packed_file.set_tab_text(index, &QString::from_std_str(&name));
            }
        }
    }

    /// This function hides all the provided packedfile views.
    pub unsafe fn packed_file_view_hide(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        indexes: &[i32]
    ) {

        let mut indexes = indexes.to_vec();
        indexes.sort_unstable();
        indexes.dedup();
        indexes.reverse();

        // PackFile and Decoder Views must be deleted on close, so get them apart if we find one.
        let mut purge_on_delete = vec![];

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            let widget = packed_file_view.get_mut_widget();
            let index_widget = app_ui.tab_bar_packed_file.index_of(widget);
            if indexes.contains(&index_widget) {
                let path = packed_file_view.get_ref_path();
                if !path.is_empty() {
                    if path.starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {
                        purge_on_delete.push(path.to_vec());
                        let _ = CENTRAL_COMMAND.send_background(Command::RemovePackFileExtra(PathBuf::from(&path[1])));
                    }
                    else if path.last().unwrap().ends_with(DECODER_EXTENSION) {
                        purge_on_delete.push(path.to_vec());
                    }
                }
            }
        }

        indexes.iter().for_each(|x| app_ui.tab_bar_packed_file.remove_tab(*x));

        // This is for cleaning up open PackFiles.
        purge_on_delete.iter().for_each(|x| { let _ = Self::purge_that_one_specifically(app_ui, pack_file_contents_ui, x, DataSource::ExternalFile, false); });

        // And this is for cleaning decoders.
        purge_on_delete.iter().for_each(|x| { let _ = Self::purge_that_one_specifically(app_ui, pack_file_contents_ui, x, DataSource::PackFile, false); });

        // Update the background icon.
        GameSelectedIcons::set_game_selected_icon(app_ui);
    }

    pub unsafe fn change_game_selected(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        rebuild_dependencies: bool
    ) {

        // Optimization: get this before starting the entire game change. Otherwise, we'll hang the thread near the end.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
        let response = CentralCommand::recv(&receiver);
        let pack_path = if let Response::PathBuf(pack_path) = response { pack_path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = app_ui.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();
        let mut game_changed = false;

        // Check if the window was previously disabled, to know if we can enable/disable it here, or will the parent function take care of it.
        let was_window_disabled = !app_ui.main_window.is_enabled();

        // If the game changed, change the game selected.
        if new_game_selected != GAME_SELECTED.read().unwrap().get_game_key_name() || !FIRST_GAME_CHANGE_DONE.load(Ordering::SeqCst) {
            FIRST_GAME_CHANGE_DONE.store(true, Ordering::SeqCst);
            game_changed = true;

            // Disable the main window if it's not yet disabled so we can avoid certain issues.
            if !was_window_disabled {
                app_ui.main_window.set_enabled(false);
            }

            // Send the command to the background thread to set the new `Game Selected`, and tell RPFM to rebuild the mymod menu when it can.
            // We have to wait because we need the GameSelected update before updating the menus.
            let receiver = CENTRAL_COMMAND.send_background(Command::SetGameSelected(new_game_selected));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Success => {}
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }

            // If we have a packfile open, set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
            if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() > 0 {
                UI_STATE.set_operational_mode(app_ui, None);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::PackFile]), DataSource::PackFile);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }

            // Change the GameSelected Icon. Disabled until we find better icons.
            GameSelectedIcons::set_game_selected_icon(app_ui);
        }

        // Regardless if the game changed or not, if we are asked to rebuild data, prepare for a rebuild.
        if rebuild_dependencies {

            // Purge all views that depend on the dependencies.
            let paths_to_close: Vec<(DataSource, Vec<String>)> = UI_STATE.set_open_packedfiles().iter()
                .filter_map(|x| if x.get_data_source() != DataSource::PackFile || x.get_data_source() != DataSource::ExternalFile { Some((x.get_data_source(), x.get_ref_path().to_vec()))} else { None })
                .collect();

            for (data_source, path) in paths_to_close {
                if let Err(error) = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, &path, data_source, true) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }

            // Request a rebuild. If thee game changed, do a full rebuild. If not, only rebuild the parent's data.
            let receiver = CENTRAL_COMMAND.send_background(Command::RebuildDependencies(!game_changed));
            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::DependenciesInfo(response) => {
                    let mut parent_build_data = BuildData::new();
                    parent_build_data.data = Some((PackFileInfo::default(), response.parent_packed_files));
                    dependencies_ui.dependencies_tree_view.update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles);

                    if game_changed {
                        let mut game_build_data = BuildData::new();
                        game_build_data.data = Some((PackFileInfo::default(), response.vanilla_packed_files));

                        let mut asskit_build_data = BuildData::new();
                        asskit_build_data.data = Some((PackFileInfo::default(), response.asskit_tables));

                        dependencies_ui.dependencies_tree_view.update_treeview(true, TreeViewOperation::Build(game_build_data), DataSource::GameFiles);
                        dependencies_ui.dependencies_tree_view.update_treeview(true, TreeViewOperation::Build(asskit_build_data), DataSource::AssKitFiles);
                    }
                }
                Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // Reenable the main window once everything is reloaded.
        if !was_window_disabled {
            app_ui.main_window.set_enabled(true);
        }

        // Disable the `PackFile Management` actions and, if we have a `PackFile` open, re-enable them.
        AppUI::enable_packfile_actions(app_ui, &pack_path, false);
        if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() != 0 {
            AppUI::enable_packfile_actions(app_ui, &pack_path, true);
        }
        let _ = CENTRAL_COMMAND.send_background(Command::GetMissingDefinitions);
    }

    /// This function creates a new PackFile and setups the UI for it.
    pub unsafe fn new_packfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>
    ) {

        // Tell the Background Thread to create a new PackFile.
        let _ = CENTRAL_COMMAND.send_background(Command::NewPackFile);

        // Reset the autosave timer.
        let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Disable the main window, so the user can't interrupt the process or interfere with it.
        let window_was_disabled = app_ui.main_window.is_enabled();
        if !window_was_disabled {
            app_ui.main_window.set_enabled(false);
        }

        // Close any open PackedFile and clear the global search panel.
        let _ = AppUI::purge_them_all(app_ui,  pack_file_contents_ui, false);
        GlobalSearchUI::clear(global_search_ui);
        diagnostics_ui.get_ref_diagnostics_table_model().clear();

        // New PackFiles are always of Mod type.
        app_ui.change_packfile_type_mod.set_checked(true);

        // By default, the four bitmask should be false.
        app_ui.change_packfile_type_data_is_encrypted.set_checked(false);
        app_ui.change_packfile_type_index_includes_timestamp.set_checked(false);
        app_ui.change_packfile_type_index_is_encrypted.set_checked(false);
        app_ui.change_packfile_type_header_is_extended.set_checked(false);

        // We also disable compression by default.
        app_ui.change_packfile_type_data_is_compressed.set_checked(false);

        // Update the TreeView.
        let mut build_data = BuildData::new();
        build_data.editable = true;
        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Enable the actions available for the PackFile from the `MenuBar`.
        AppUI::enable_packfile_actions(app_ui, &PathBuf::new(), true);

        // Set the current "Operational Mode" to Normal, as this is a "New" mod.
        UI_STATE.set_operational_mode(app_ui, None);
        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);

        // Force a dependency rebuild.
        let receiver = CENTRAL_COMMAND.send_background(Command::RebuildDependencies(true));
        let response = CentralCommand::recv_try(&receiver);
        match response {
            Response::DependenciesInfo(response) => {
                let mut parent_build_data = BuildData::new();
                parent_build_data.data = Some((PackFileInfo::default(), response.parent_packed_files));
                dependencies_ui.dependencies_tree_view.update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles);
            }
            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Re-enable the Main Window.
        if !window_was_disabled {
            app_ui.main_window.set_enabled(true);
        }
    }

    /// This function is used to perform MyḾod imports.
    pub unsafe fn import_mymod(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) {
        app_ui.main_window.set_enabled(false);

        match UI_STATE.get_operational_mode() {

            // If we have a "MyMod" selected...
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                if let Some(ref mymods_base_path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {

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

                    // Get the Paths of the files inside the folders we want to add.
                    let paths: Vec<PathBuf> = match get_files_from_subdir(&assets_folder, true) {
                        Ok(paths) => paths,
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    };

                    // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                    for path in &paths {
                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                    }

                    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileSettings(false));
                    let response = CentralCommand::recv(&receiver);
                    let settings = match response {
                        Response::PackFileSettings(settings) => settings,
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    };

                    let files_to_ignore = settings.settings_text.get("import_files_to_ignore").map(|files_to_ignore| {
                        if files_to_ignore.is_empty() { vec![] } else {
                            files_to_ignore.split('\n')
                                .map(|x| assets_folder.to_path_buf().join(x))
                                .collect::<Vec<PathBuf>>()
                        }
                    });

                    PackFileContentsUI::add_packedfiles(&app_ui, &pack_file_contents_ui, &paths, &paths_packedfile, files_to_ignore, true);
                }

                // If there is no MyMod path configured, report it.
                else { show_dialog(&app_ui.main_window, ErrorKind::MyModPathNotConfigured, false) }
            }
            OperationalMode::Normal => show_dialog(&app_ui.main_window, ErrorKind::PackFileIsNotAMyMod, false),
        }

        app_ui.main_window.set_enabled(true);
    }

    /// This function is used to perform MyḾod exports.
    pub unsafe fn export_mymod(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        paths_to_extract: Option<Vec<PathType>>
    ) {
        PackFileContentsUI::extract_packed_files(app_ui, pack_file_contents_ui, paths_to_extract, true)
    }
}
