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
Module with all the code for extra implementations of `AppUI`.

This module contains the implementation of custom functions for `AppUI`. The reason
they're here and not in the main file is because I don't want to polute that one,
as it's mostly meant for initialization and configuration.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QFileDialog;
use qt_widgets::QLineEdit;
use qt_widgets::{q_message_box, QMessageBox};
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;


use qt_gui::QStandardItemModel;
use qt_core::QFlags;
use qt_core::QRegExp;
use qt_core::{SlotOfBool, SlotOfQString};
use qt_core::QSortFilterProxyModel;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::common::{get_game_selected_data_path, get_game_selected_content_packfiles_paths, get_game_selected_data_packfiles_paths};
use rpfm_lib::DOCS_BASE_URL;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::games::*;
use rpfm_lib::packedfile::{PackedFileType, table::loc, text, text::TextType};
use rpfm_lib::packfile::{PFHFileType, PFHFlags, CompressionState, PFHVersion};
use rpfm_lib::schema::{versions::APIResponseSchema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use super::AppUI;
use super::NewPackedFile;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR, network::APIResponse};
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, tr};
use crate::pack_tree::{icons::IconType, new_pack_file_tooltip, PackTree, TreePathType, TreeViewOperation};
use crate::packedfile_views::{ca_vp8::*, decoder::*, external::*, image::*, PackedFileView, rigidmodel::*, table::*, TheOneSlot, text::*};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;
use crate::utils::{create_grid_layout, show_dialog};

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUI`.
impl AppUI {

    /// This function takes care of updating the Main Window's title to reflect the current state of the program.
    pub unsafe fn update_window_title(&mut self, packfile_contents_ui: &PackFileContentsUI) {

        // First check if we have a PackFile open. If not, just leave the default title.
        let window_title = if packfile_contents_ui.packfile_contents_tree_model.row_count_0a() == 0 {
            "Rusted PackFile Manager".to_owned()
        }

        // If there is a `PackFile` open, check if it has been modified, and set the title accordingly.
        else {
            let pack_file_name = packfile_contents_ui.packfile_contents_tree_model.item_1a(0).text().to_std_string();
            if UI_STATE.get_is_modified() { format!("{} - Modified", pack_file_name) }
            else { format!("{} - Not Modified", pack_file_name) }
        };
        self.main_window.set_window_title(&QString::from_std_str(window_title));
    }

    /// This function pops up a modal asking you if you're sure you want to do an action that may result in unsaved data loss.
    ///
    /// If you are trying to delete the open MyMod, pass it true.
    pub unsafe fn are_you_sure(&self, is_delete_my_mod: bool) -> bool {
        let title = "Rusted PackFile Manager";
        let message = if is_delete_my_mod { "<p>You are about to delete this <i>'MyMod'</i> from your disk.</p><p>There is no way to recover it after that.</p><p>Are you sure?</p>" }
        else if UI_STATE.get_is_modified() { "<p>There are some changes yet to be saved.</p><p>Are you sure?</p>" }

        // In any other situation... just return true and forget about the dialog.
        else { return true };

        // Create the dialog and run it (Yes => 3, No => 4).
        QMessageBox::from_2_q_string_icon3_int_q_widget(
            &QString::from_std_str(title),
            &QString::from_std_str(message),
            q_message_box::Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            self.main_window,
        ).exec() == 3
    }

    /// This function deletes all the widgets corresponding to opened PackedFiles.
    pub unsafe fn purge_them_all(&mut self,
        global_search_ui: GlobalSearchUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>
    ) {

        // Black magic.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        for (path, packed_file_view) in open_packedfiles.iter_mut() {

            // TODO: This should report an error.
            let _ = packed_file_view.save(path, global_search_ui, &mut pack_file_contents_ui);
            let mut widget = packed_file_view.get_mut_widget();
            let index = self.tab_bar_packed_file.index_of(widget);
            self.tab_bar_packed_file.remove_tab(index);

            // Delete the widget manually to free memory.
            widget.delete_later();
        }

        // Remove all open PackedFiles and their slots.
        open_packedfiles.clear();
        slot_holder.borrow_mut().clear();

        // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
        self.game_selected_group.set_enabled(true);

        // Just in case what was open before was the `Add From PackFile` TreeView, unlock it.
        UI_STATE.set_packfile_contents_read_only(false);
    }

    /// This function deletes all the widgets corresponding to the specified PackedFile, if exists.
    pub unsafe fn purge_that_one_specifically(&mut self,
        global_search_ui: GlobalSearchUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        path: &[String],
        save_before_deleting: bool
    ) {

        // Black magic to remove widgets.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        if let Some(packed_file_view) = open_packedfiles.get_mut(path) {
            if save_before_deleting && path != ["extra_packfile.rpfm_reserved".to_owned()] {

                // TODO: This should report an error.
                let _ = packed_file_view.save(path, global_search_ui, &mut pack_file_contents_ui);
            }
            let mut widget = packed_file_view.get_mut_widget();
            let index = self.tab_bar_packed_file.index_of(widget);
            self.tab_bar_packed_file.remove_tab(index);

            // Delete the widget manually to free memory.
            widget.delete_later();
        }

        if !path.is_empty() {
            open_packedfiles.remove(path);
            if path != ["extra_packfile.rpfm_reserved".to_owned()] {

                // We check if there are more tables open. This is because we cannot change the GameSelected
                // when there is a PackedFile using his Schema.
                let mut enable_game_selected_menu = true;
                for path in open_packedfiles.keys() {
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
                    self.game_selected_group.set_enabled(true);
                }
            }
        }
    }

    /// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending on the situation.
    ///
    /// NOTE: The `game_folder` is for when using this function with *MyMods*. If you're opening a normal mod, pass it empty.
    pub unsafe fn open_packfile(
        &mut self,
        pack_file_contents_ui: &mut PackFileContentsUI,
        global_search_ui: &mut GlobalSearchUI,
        pack_file_paths: &[PathBuf],
        game_folder: &str,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) -> Result<()> {

        // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
        self.purge_them_all(*global_search_ui, *pack_file_contents_ui, slot_holder);

        // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
        self.main_window.set_enabled(false);
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFiles(pack_file_paths.to_vec()));

        // Check what response we got.
        let response = CENTRAL_COMMAND.recv_message_qt_try();
        match response {

            // If it's success....
            Response::PackFileInfo(ui_data) => {

                // We choose the right option, depending on our PackFile.
                match ui_data.pfh_file_type {
                    PFHFileType::Boot => self.change_packfile_type_boot.set_checked(true),
                    PFHFileType::Release => self.change_packfile_type_release.set_checked(true),
                    PFHFileType::Patch => self.change_packfile_type_patch.set_checked(true),
                    PFHFileType::Mod => self.change_packfile_type_mod.set_checked(true),
                    PFHFileType::Movie => self.change_packfile_type_movie.set_checked(true),
                    PFHFileType::Other(_) => self.change_packfile_type_other.set_checked(true),
                }

                // Enable or disable these, depending on what data we have in the header.
                self.change_packfile_type_data_is_encrypted.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA));
                self.change_packfile_type_index_includes_timestamp.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS));
                self.change_packfile_type_index_is_encrypted.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX));
                self.change_packfile_type_header_is_extended.set_checked(ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER));

                // Set the compression level correctly, because otherwise we may fuckup some files.
                let compression_state = match ui_data.compression_state {
                    CompressionState::Enabled => true,
                    CompressionState::Partial | CompressionState::Disabled => false,
                };
                self.change_packfile_type_data_is_compressed.set_checked(compression_state);

                // Update the TreeView.
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(false));

                // Re-enable the Main Window.
                self.main_window.set_enabled(true);

                // Close the Global Search stuff and reset the filter's history.
                global_search_ui.clear();

                // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
                if !game_folder.is_empty() && pack_file_paths.len() == 1 {

                    // NOTE: Arena should never be here.
                    // Change the Game Selected in the UI.
                    match game_folder {
                        KEY_THREE_KINGDOMS => self.game_selected_three_kingdoms.trigger(),
                        KEY_WARHAMMER_2 => self.game_selected_warhammer_2.trigger(),
                        KEY_WARHAMMER => self.game_selected_warhammer.trigger(),
                        KEY_THRONES_OF_BRITANNIA => self.game_selected_thrones_of_britannia.trigger(),
                        KEY_ATTILA => self.game_selected_attila.trigger(),
                        KEY_ROME_2 => self.game_selected_rome_2.trigger(),
                        KEY_SHOGUN_2 => self.game_selected_shogun_2.trigger(),
                        KEY_NAPOLEON => self.game_selected_napoleon.trigger(),
                        KEY_EMPIRE => self.game_selected_empire.trigger(),
                        _ => unimplemented!()
                    }

                    // Set the current "Operational Mode" to `MyMod`.
                    UI_STATE.set_operational_mode(self, Some(&pack_file_paths[0]));
                }

                // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
                else {

                    // Depending on the Id, choose one game or another.
                    match ui_data.pfh_version {

                        // PFH5 is for Warhammer 2/Arena.
                        PFHVersion::PFH5 => {

                            // If the PackFile has the mysterious byte enabled, it's from Arena.
                            if ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
                                self.game_selected_arena.trigger();
                            }

                            // Otherwise, it's from Three Kingdoms or Warhammer 2.
                            else {
                                let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                                match &*game_selected {
                                    KEY_THREE_KINGDOMS => self.game_selected_three_kingdoms.trigger(),
                                    KEY_WARHAMMER_2 => self.game_selected_warhammer_2.trigger(),
                                    _ => unimplemented!()
                                }
                            }
                        },

                        // PFH4 is for Thrones of Britannia/Warhammer 1/Attila/Rome 2.
                        PFHVersion::PFH4 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                            match &*game_selected {
                                KEY_WARHAMMER => self.game_selected_warhammer.trigger(),
                                KEY_THRONES_OF_BRITANNIA => self.game_selected_thrones_of_britannia.trigger(),
                                KEY_ATTILA => self.game_selected_attila.trigger(),
                                KEY_ROME_2 => self.game_selected_rome_2.trigger(),
                                _ => unimplemented!()
                            }
                        },

                        // PFH3 is for Shogun 2.
                        PFHVersion::PFH3 | PFHVersion::PFH2 => self.game_selected_shogun_2.trigger(),

                        // PFH0 is for Napoleon/Empire.
                        PFHVersion::PFH0 => {
                            let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                            match &*game_selected {
                                KEY_NAPOLEON => self.game_selected_napoleon.trigger(),
                                KEY_EMPIRE => self.game_selected_empire.trigger(),
                                _ => unimplemented!()
                            }
                        },
                    }
                }

                //if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                // Show the "Tips".
                //display_help_tips(&app_ui);
            }

            // If we got an error...
            Response::Error(error) => {
                self.main_window.set_enabled(true);
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
        &mut self,
        pack_file_contents_ui: &mut PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        save_as: bool,
    ) -> Result<()> {

        let mut result = Ok(());
        self.main_window.set_enabled(false);

        // First, we need to save all open `PackedFiles` to the backend.
        UI_STATE.get_open_packedfiles().iter().try_for_each(|(path, packed_file)| packed_file.save(path, *global_search_ui, pack_file_contents_ui))?;

        CENTRAL_COMMAND.send_message_qt(Command::GetPackFilePath);
        let response = CENTRAL_COMMAND.recv_message_qt();
        let mut path = if let Response::PathBuf(path) = response { path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };
        if !path.is_file() || save_as {

            // Create the FileDialog to save the PackFile and configure it.
            let mut file_dialog = QFileDialog::from_q_widget_q_string(
                self.main_window,
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
            else if let Some(ref path) = get_game_selected_data_path() {
                if path.is_dir() { file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned())); }
            }

            // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {
                let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                CENTRAL_COMMAND.send_message_qt(Command::SavePackFileAs(path));
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {
                    Response::PackFileInfo(pack_file_info) => {
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean);
                        let mut packfile_item = pack_file_contents_ui.packfile_contents_tree_model.item_1a(0);
                        packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                        packfile_item.set_text(&QString::from_std_str(&file_name));

                        UI_STATE.set_operational_mode(self, None);
                    }
                    Response::Error(error) => result = Err(error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }

        else {
            CENTRAL_COMMAND.send_message_qt(Command::SavePackFile);
            let response = CENTRAL_COMMAND.recv_message_qt_try();
            match response {
                Response::PackFileInfo(pack_file_info) => {
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean);
                    let mut packfile_item = pack_file_contents_ui.packfile_contents_tree_model.item_1a(0);
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                }
                Response::Error(error) => result = Err(error),

                // In ANY other situation, it's a message problem.
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // Then we re-enable the main Window and return whatever we've received.
        self.main_window.set_enabled(true);
        result
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub unsafe fn enable_packfile_actions(&mut self, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        if &**GAME_SELECTED.read().unwrap() == KEY_ARENA {

            // Disable the actions that allow to create and save PackFiles.
            self.packfile_new_packfile.set_enabled(false);
            self.packfile_save_packfile.set_enabled(false);
            self.packfile_save_packfile_as.set_enabled(false);

            // This one too, though we had to deal with it specially later on.
            self.mymod_new.set_enabled(false);
        }

        // Otherwise...
        else {

            // Enable or disable the actions from "PackFile" Submenu.
            self.packfile_new_packfile.set_enabled(true);
            self.packfile_save_packfile.set_enabled(enable);
            self.packfile_save_packfile_as.set_enabled(enable);

            // If there is a "MyMod" path set in the settings...
            if let Some(ref path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {

                // And it's a valid directory, enable the "New MyMod" button.
                if path.is_dir() { self.mymod_new.set_enabled(true); }

                // Otherwise, disable it.
                else { self.mymod_new.set_enabled(false); }
            }

            // Otherwise, disable it.
            else { self.mymod_new.set_enabled(false); }
        }

        // These actions are common, no matter what game we have.
        self.change_packfile_type_group.set_enabled(enable);
        self.change_packfile_type_index_includes_timestamp.set_enabled(enable);

        // If we are enabling...
        if enable {

            // Check the Game Selected and enable the actions corresponding to out game.
            match &**GAME_SELECTED.read().unwrap() {
                KEY_THREE_KINGDOMS => {
                    self.change_packfile_type_data_is_compressed.set_enabled(true);
                    self.special_stuff_three_k_optimize_packfile.set_enabled(true);
                    self.special_stuff_three_k_generate_pak_file.set_enabled(true);
                },
                KEY_WARHAMMER_2 => {
                    self.change_packfile_type_data_is_compressed.set_enabled(true);
                    self.special_stuff_wh2_patch_siege_ai.set_enabled(true);
                    self.special_stuff_wh2_optimize_packfile.set_enabled(true);
                    self.special_stuff_wh2_generate_pak_file.set_enabled(true);
                },
                KEY_WARHAMMER => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_wh_patch_siege_ai.set_enabled(true);
                    self.special_stuff_wh_optimize_packfile.set_enabled(true);
                    self.special_stuff_wh_generate_pak_file.set_enabled(true);
                },
                KEY_THRONES_OF_BRITANNIA => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_tob_optimize_packfile.set_enabled(true);
                    self.special_stuff_tob_generate_pak_file.set_enabled(true);
                },
                KEY_ATTILA => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_att_optimize_packfile.set_enabled(true);
                    self.special_stuff_att_generate_pak_file.set_enabled(true);
                },
                KEY_ROME_2 => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_rom2_optimize_packfile.set_enabled(true);
                    self.special_stuff_rom2_generate_pak_file.set_enabled(true);
                },
                KEY_SHOGUN_2 => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_sho2_optimize_packfile.set_enabled(true);
                    self.special_stuff_sho2_generate_pak_file.set_enabled(true);
                },
                KEY_NAPOLEON => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_nap_optimize_packfile.set_enabled(true);
                },
                KEY_EMPIRE => {
                    self.change_packfile_type_data_is_compressed.set_enabled(false);
                    self.special_stuff_emp_optimize_packfile.set_enabled(true);
                },
                _ => {},
            }
        }

        // If we are disabling...
        else {

            // Universal Actions.
            self.change_packfile_type_data_is_compressed.set_enabled(false);

            // Disable Three Kingdoms actions...
            self.special_stuff_three_k_optimize_packfile.set_enabled(false);
            self.special_stuff_three_k_generate_pak_file.set_enabled(false);

            // Disable Warhammer 2 actions...
            self.special_stuff_wh2_patch_siege_ai.set_enabled(false);
            self.special_stuff_wh2_optimize_packfile.set_enabled(false);
            self.special_stuff_wh2_generate_pak_file.set_enabled(false);

            // Disable Warhammer actions...
            self.special_stuff_wh_patch_siege_ai.set_enabled(false);
            self.special_stuff_wh_optimize_packfile.set_enabled(false);
            self.special_stuff_wh_generate_pak_file.set_enabled(false);

            // Disable Thrones of Britannia actions...
            self.special_stuff_tob_optimize_packfile.set_enabled(false);
            self.special_stuff_tob_generate_pak_file.set_enabled(false);

            // Disable Attila actions...
            self.special_stuff_att_optimize_packfile.set_enabled(false);
            self.special_stuff_att_generate_pak_file.set_enabled(false);

            // Disable Rome 2 actions...
            self.special_stuff_rom2_optimize_packfile.set_enabled(false);
            self.special_stuff_rom2_generate_pak_file.set_enabled(false);

            // Disable Shogun 2 actions...
            self.special_stuff_sho2_optimize_packfile.set_enabled(false);
            self.special_stuff_sho2_generate_pak_file.set_enabled(false);

            // Disable Napoleon actions...
            self.special_stuff_nap_optimize_packfile.set_enabled(false);

            // Disable Empire actions...
            self.special_stuff_emp_optimize_packfile.set_enabled(false);
        }

        // The assembly kit thing should only be available for Rome 2 and later games.
        match &**GAME_SELECTED.read().unwrap() {
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 => self.game_selected_open_game_assembly_kit_folder.set_enabled(true),
            _ => self.game_selected_open_game_assembly_kit_folder.set_enabled(false),
        }
    }

    /// This function takes care of recreating the dynamic submenus under `PackFile` menu.
    pub unsafe fn build_open_from_submenus(mut self, mut pack_file_contents_ui: PackFileContentsUI, mut global_search_ui: GlobalSearchUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> Vec<SlotOfBool<'static>> {
        let mut packfile_open_from_content = self.packfile_open_from_content;
        let mut packfile_open_from_data = self.packfile_open_from_data;

        // First, we clear both menus, so we can rebuild them properly.
        packfile_open_from_content.clear();
        packfile_open_from_data.clear();

        // And we create the slots.
        let mut open_from_slots = vec![];

        //---------------------------------------------------------------------------------------//
        // Build the menus...
        //---------------------------------------------------------------------------------------//

        // Get the path of every PackFile in the content folder (if the game's path it's configured) and make an action for each one of them.
        let mut content_paths = get_game_selected_content_packfiles_paths();
        if let Some(ref mut paths) = content_paths {
            paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
            for path in paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let open_mod_action = packfile_open_from_content.add_action_q_string(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotOfBool::new(clone!(
                    path,
                    slot_holder => move |_| {
                    if self.are_you_sure(false) {
                        if let Err(error) = self.open_packfile(&mut pack_file_contents_ui, &mut global_search_ui, &[path.to_path_buf()], "", &slot_holder) {
                            show_dialog(self.main_window, error, false);
                        }
                    }
                }));

                // Connect the slot and store it.
                open_mod_action.triggered().connect(&slot_open_mod);
                open_from_slots.push(slot_open_mod);
            }
        }

        // Get the path of every PackFile in the data folder (if the game's path it's configured) and make an action for each one of them.
        let mut data_paths = get_game_selected_data_packfiles_paths();
        if let Some(ref mut paths) = data_paths {
            paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
            for path in paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let open_mod_action = packfile_open_from_data.add_action_q_string(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotOfBool::new(clone!(
                    path,
                    slot_holder => move |_| {
                    if self.are_you_sure(false) {
                        if let Err(error) = self.open_packfile(&mut pack_file_contents_ui, &mut global_search_ui, &[path.to_path_buf()], "", &slot_holder) {
                            show_dialog(self.main_window, error, false);
                        }
                    }
                }));

                // Connect the slot and store it.
                open_mod_action.triggered().connect(&slot_open_mod);
                open_from_slots.push(slot_open_mod);
            }
        }

        // Only if the submenu has items, we enable it.
        packfile_open_from_content.menu_action().set_visible(!packfile_open_from_content.actions().is_empty());
        packfile_open_from_data.menu_action().set_visible(!packfile_open_from_data.actions().is_empty());

        // Return the slots.
        open_from_slots
    }


    /// This function takes care of the re-creation of the `MyMod` list for each game.
    pub unsafe fn build_open_mymod_submenus(mut self, mut pack_file_contents_ui: PackFileContentsUI, mut global_search_ui: GlobalSearchUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> Vec<SlotOfBool<'static>> {

        // First, we need to reset the menu, which basically means deleting all the game submenus and hiding them.
        self.mymod_open_three_kingdoms.menu_action().set_visible(false);
        self.mymod_open_warhammer_2.menu_action().set_visible(false);
        self.mymod_open_warhammer.menu_action().set_visible(false);
        self.mymod_open_thrones_of_britannia.menu_action().set_visible(false);
        self.mymod_open_attila.menu_action().set_visible(false);
        self.mymod_open_rome_2.menu_action().set_visible(false);
        self.mymod_open_shogun_2.menu_action().set_visible(false);
        self.mymod_open_napoleon.menu_action().set_visible(false);
        self.mymod_open_empire.menu_action().set_visible(false);

        self.mymod_open_three_kingdoms.clear();
        self.mymod_open_warhammer_2.clear();
        self.mymod_open_warhammer.clear();
        self.mymod_open_thrones_of_britannia.clear();
        self.mymod_open_attila.clear();
        self.mymod_open_rome_2.clear();
        self.mymod_open_shogun_2.clear();
        self.mymod_open_napoleon.clear();
        self.mymod_open_empire.clear();

        let mut slots = vec![];

        // If we have the "MyMod" path configured, get all the packfiles under the `MyMod` folder, separated by supported game.
        let supported_folders = SUPPORTED_GAMES.iter().filter(|(_, x)| x.supports_editing).map(|(folder_name,_)| *folder_name).collect::<Vec<&str>>();
        if let Some(ref mymod_base_path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {
            if let Ok(game_folder_list) = mymod_base_path.read_dir() {
                for game_folder in game_folder_list {
                    if let Ok(game_folder) = game_folder {

                        // If it's a valid folder, and it's in our supported games list, get all the PackFiles inside it and create an open action for them.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        if game_folder.path().is_dir() && supported_folders.contains(&&*game_folder_name) {
                            let mut game_submenu = match &*game_folder_name {
                                KEY_THREE_KINGDOMS => self.mymod_open_three_kingdoms,
                                KEY_WARHAMMER_2 => self.mymod_open_warhammer_2,
                                KEY_WARHAMMER => self.mymod_open_warhammer,
                                KEY_THRONES_OF_BRITANNIA => self.mymod_open_thrones_of_britannia,
                                KEY_ATTILA => self.mymod_open_attila,
                                KEY_ROME_2 => self.mymod_open_rome_2,
                                KEY_SHOGUN_2 => self.mymod_open_shogun_2,
                                KEY_NAPOLEON => self.mymod_open_napoleon,
                                KEY_EMPIRE => self.mymod_open_empire,
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
                                        let slot_open_mod = SlotOfBool::new(clone!(
                                            slot_holder,
                                            game_folder_name => move |_| {
                                            if self.are_you_sure(false) {
                                                if let Err(error) = self.open_packfile(&mut pack_file_contents_ui, &mut global_search_ui, &[pack_file.to_path_buf()], &game_folder_name, &slot_holder) {
                                                    show_dialog(self.main_window, error, false);
                                                }
                                            }
                                        }));

                                        open_mod_action.triggered().connect(&slot_open_mod);
                                        slots.push(slot_open_mod);
                                    }
                                }
                            }

                            // Only if the submenu has items, we show it to the big menu.
                            if game_submenu.actions().count() > 0 {
                                game_submenu.menu_action().set_visible(true);
                            }
                        }
                    }
                }
            }
        }

        slots
    }

    /// This function checks if there is any newer version of RPFM released.
    ///
    /// If the `use_dialog` is false, we make the checks in the background, and pop up a dialog only in case there is an update available.
    pub unsafe fn check_updates(&self, use_dialog: bool) {
        CENTRAL_COMMAND.send_message_qt_to_network(Command::CheckUpdates);

        // If we want to use a Dialog to show the full searching process (clicking in the
        // menu button) we show the dialog, then change its text.
        if use_dialog {
            let mut dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                q_message_box::Icon::Information,
                &qtr("update_checker"),
                &qtr("update_searching"),
                QFlags::from(q_message_box::StandardButton::Close),
                self.main_window,
            );

            dialog.set_modal(true);
            dialog.show();

            let response = CENTRAL_COMMAND.recv_message_network_to_qt_try();
            let message = match response {
                Response::APIResponse(response) => {
                    match response {
                        APIResponse::SuccessNewUpdate(last_release) => format!("<h4>New major update found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
                        APIResponse::SuccessNewUpdateHotfix(last_release) => format!("<h4>New minor update/hotfix found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
                        APIResponse::SuccessNoUpdate => "<h4>No new updates available</h4> <p>More luck next time :)</p>".to_owned(),
                        APIResponse::SuccessUnknownVersion => "<h4>Error while checking new updates</h4> <p>There has been a problem when getting the lastest released version number, or the current version number. That means I fucked up the last release title. If you see this, please report it here:\n<a href=\"https://github.com/Frodo45127/rpfm/issues\">https://github.com/Frodo45127/rpfm/issues</a></p>".to_owned(),
                        APIResponse::Error => "<h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>".to_owned(),
                    }
                }

                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            };

            dialog.set_text(&QString::from_std_str(message));
            dialog.exec();
        }

        // Otherwise, we just wait until we got a response, and only then (and only in case of new update)... we show a dialog.
        else {
            let response = CENTRAL_COMMAND.recv_message_network_to_qt_try();
            let message = match response {
                Response::APIResponse(response) => {
                    match response {
                        APIResponse::SuccessNewUpdate(last_release) => format!("<h4>New major update found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
                        APIResponse::SuccessNewUpdateHotfix(last_release) => format!("<h4>New minor update/hotfix found: \"{}\"</h4> <p>Download and changelog available here:<br><a href=\"{}\">{}</a></p>", last_release.name, last_release.html_url, last_release.html_url),
                        _ => return,
                    }
                }

                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            };

            let mut dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                q_message_box::Icon::Information,
                &qtr("update_checker"),
                &QString::from_std_str(message),
                QFlags::from(q_message_box::StandardButton::Close),
                self.main_window,
            );

            dialog.set_modal(true);
            dialog.exec();
        }
    }

    /// This function checks if there is any newer version of RPFM's schemas released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub unsafe fn check_schema_updates(&self, use_dialog: bool) {
        CENTRAL_COMMAND.send_message_qt_to_network(Command::CheckSchemaUpdates);

        // If we want to use a Dialog to show the full searching process.
        if use_dialog {

            // Create the dialog to show the response and configure it.
            let mut dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                q_message_box::Icon::Information,
                &qtr("update_schema_checker"),
                &qtr("update_searching"),
                QFlags::from(q_message_box::StandardButton::Close),
                self.main_window,
            );

            let mut update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
            update_button.set_enabled(false);

            dialog.set_modal(true);
            dialog.show();

            // When we get a response, act depending on the kind of response we got.
            let response_thread = CENTRAL_COMMAND.recv_message_network_to_qt_try();
            let message = match response_thread {
                Response::APIResponseSchema(ref response) => {
                    match response {
                        APIResponseSchema::SuccessNewUpdate(ref local_versions, ref remote_versions) => {
                            update_button.set_enabled(true);

                            // Build a table with each one of the remote schemas to show what ones got updated.
                            let mut message = "<h4>New schema update available</h4> <table>".to_owned();
                            for (remote_schema_name, remote_schema_version) in remote_versions.get() {
                                message.push_str("<tr>");
                                message.push_str(&format!("<td>{}:</td>", remote_schema_name));

                                // If the game exist in the local version, show both versions.
                                let game_name = SUPPORTED_GAMES.iter().find(|x| &x.1.schema == remote_schema_name).unwrap().0;
                                if let Some(local_schema_version) = local_versions.get().get(remote_schema_name) {
                                    message.push_str(&format!("<td>{lsv} => <a href='{base_url}changelogs_tables/{game_name}/changelog.html#{rsv:03}'>{rsv}</a></td>",base_url = DOCS_BASE_URL.to_owned(), game_name = game_name, lsv = local_schema_version, rsv = remote_schema_version));
                                } else { message.push_str(&format!("<td>0 => <a href='{base_url}changelogs_tables/{game_name}/changelog.html#{rsv:03}'>{rsv}</a></td>",base_url = DOCS_BASE_URL.to_owned(), game_name = game_name, rsv = remote_schema_version)); }

                                message.push_str("</tr>");
                            }
                            message.push_str("</table>");
                            message.push_str("<p>Do you want to update the schemas?</p>");
                            message
                        }

                        APIResponseSchema::SuccessNoLocalUpdate => {
                            update_button.set_enabled(true);
                            tr("update_no_local_schema")
                        },
                        APIResponseSchema::SuccessNoUpdate => "<h4>No new schema updates available</h4> <p>More luck next time :)</p>".to_owned(),
                        APIResponseSchema::Error => "<h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>".to_owned(),
                    }
                }

                Response::Error(error) => return show_dialog(self.main_window, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
            };

            // If we hit "Update", try to update the schemas.
            dialog.set_text(&QString::from_std_str(message));
            if dialog.exec() == 0 {
                if let Response::APIResponseSchema(ref response) = response_thread {
                    match response {
                        APIResponseSchema::SuccessNewUpdate(_,_) | APIResponseSchema::SuccessNoLocalUpdate => {
                            CENTRAL_COMMAND.send_message_qt(Command::UpdateSchemas);

                            dialog.show();
                            dialog.set_text(&qtr("update_in_prog"));
                            update_button.set_enabled(false);

                            match CENTRAL_COMMAND.recv_message_qt_try() {
                                Response::Success => show_dialog(self.main_window, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>", true),
                                Response::Error(error) => show_dialog(self.main_window, error, false),
                                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
                            }
                        }
                        _ => return
                    }
                }
            }
        }

        // Otherwise, we just wait until we got a response, and only then (and only in case of new schema update) we show a dialog.
        else {
            let response_thread = CENTRAL_COMMAND.recv_message_network_to_qt_try();
            let message = match response_thread {
                Response::APIResponseSchema(ref response) => {
                    match response {
                        APIResponseSchema::SuccessNewUpdate(ref local_versions, ref remote_versions) => {

                            // Build a table with each one of the remote schemas to show what ones got updated.
                            let mut message = "<h4>New schema update available</h4> <table>".to_owned();
                            for (remote_schema_name, remote_schema_version) in remote_versions.get() {
                                message.push_str("<tr>");
                                message.push_str(&format!("<td>{}:</td>", remote_schema_name));

                                // If the game exist in the local version, show both versions.
                                let game_name = SUPPORTED_GAMES.iter().find(|x| &x.1.schema == remote_schema_name).unwrap().0;
                                if let Some(local_schema_version) = local_versions.get().get(remote_schema_name) {
                                    message.push_str(&format!("<td>{lsv} => <a href='{base_url}changelogs_tables/{game_name}/changelog.html#{rsv:03}'>{rsv}</a></td>",base_url = DOCS_BASE_URL.to_owned(), game_name = game_name, lsv = local_schema_version, rsv = remote_schema_version));
                                } else { message.push_str(&format!("<td>0 => <a href='{base_url}changelogs_tables/{game_name}/changelog.html#{rsv:03}'>{rsv}</a></td>",base_url = DOCS_BASE_URL.to_owned(), game_name = game_name, rsv = remote_schema_version)); }

                                message.push_str("</tr>");
                            }
                            message.push_str("</table>");
                            message.push_str("<p>Do you want to update the schemas?</p>");
                            message
                        }
                        APIResponseSchema::SuccessNoLocalUpdate => {
                            tr("update_no_local_schema")
                        }
                        _ => return
                    }
                }
                _ => return
            };

            // Create the dialog to show the response.
            let mut dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                q_message_box::Icon::Information,
                &qtr("update_schema_checker"),
                &QString::from_std_str(message),
                QFlags::from(q_message_box::StandardButton::Close),
                self.main_window,
            );

            let mut update_button = dialog.add_button_q_string_button_role(&qtr("update_button"), q_message_box::ButtonRole::AcceptRole);
            dialog.set_modal(true);

            // If we hit "Update", try to update the schemas.
            if dialog.exec() == 0 {
                if let Response::APIResponseSchema(response) = response_thread {
                    match response {
                        APIResponseSchema::SuccessNewUpdate(_,_) | APIResponseSchema::SuccessNoLocalUpdate => {

                            CENTRAL_COMMAND.send_message_qt(Command::UpdateSchemas);

                            dialog.show();
                            dialog.set_text(&qtr("update_in_prog"));
                            update_button.set_enabled(false);

                            match CENTRAL_COMMAND.recv_message_qt_try() {
                                Response::Success => show_dialog(self.main_window, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>", true),
                                Response::Error(error) => show_dialog(self.main_window, error, false),
                                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                            }
                        }
                        _ => return
                    }
                }
            }
        }
    }

    /// This function is used to open ANY supported PackedFiles in a DockWidget, docked in the Main Window.
    pub unsafe fn open_packedfile(
        &mut self,
        pack_file_contents_ui: &mut PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
        is_preview: bool,
        is_external: bool,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        // Also, only open the selection when there is only one thing selected.
        if !UI_STATE.get_packfile_contents_read_only() {
            let selected_items = <MutPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &selected_items[0] } else { return };
            if let TreePathType::File(path) = item_type {

                // Close all preview views except the file we're opening.
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = self.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if open_path != path && packed_file_view.get_is_preview() && index != -1 {
                        self.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the file we want to open is already open, or it's hidden, we show it/focus it, instead of opening it again.
                // If it was a preview, then we mark it as full. Index == -1 means it's not in a tab.
                if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(path) {
                    if !is_external {
                        let index = self.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                        // If we're trying to open as preview something already open as full, we don't do anything.
                        if !(index != -1 && is_preview && !tab_widget.get_is_preview()) {
                            tab_widget.set_is_preview(is_preview);
                        }

                        if index == -1 {
                            let icon_type = IconType::File(path.to_vec());
                            let icon = icon_type.get_icon_from_path();
                            self.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &QString::from_std_str(""));
                        }

                        let name = if tab_widget.get_is_preview() { format!("{} (Preview)", path.last().unwrap()) } else { path.last().unwrap().to_owned() };
                        let index = self.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());
                        self.tab_bar_packed_file.set_tab_text(index, &QString::from_std_str(&name));
                        self.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                        return;
                    }
                }

                // If we have a PackedFile open, but we want to open it as a External file, close it here.
                if is_external && UI_STATE.set_open_packedfiles().get_mut(path).is_some() {
                    self.purge_that_one_specifically(*global_search_ui, *pack_file_contents_ui, path, true)
                }

                let mut tab = PackedFileView::default();
                let tab_widget = tab.get_mut_widget();
                if !is_external {
                    tab.set_is_preview(is_preview);
                    let name = if tab.get_is_preview() { format!("{} (Preview)", path.last().unwrap()) } else { path.last().unwrap().to_owned() };
                    let icon_type = IconType::File(path.to_vec());
                    let icon = icon_type.get_icon_from_path();

                    // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                    let packed_file_type = PackedFileType::get_packed_file_type(&path);
                    let path = Rc::new(RefCell::new(path.to_vec()));

                    match packed_file_type {

                        // If the file is a CA_VP8 PackedFile...
                        PackedFileType::CaVp8 => {
                            match PackedFileCaVp8View::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::CaVp8Decode(format!("{}", error)), false),
                            }
                        }


                        // If the file is a Loc PackedFile...
                        PackedFileType::Loc => {
                            match PackedFileTableView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                    }
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::LocDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a DB PackedFile...
                        PackedFileType::DB => {
                            match PackedFileTableView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                    }
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::DBTableDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a Text PackedFile...
                        PackedFileType::Text(_) => {
                            match PackedFileTextView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    if let Some(packed_file_info) = packed_file_info {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                    }
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a RigidModel PackedFile...
                        PackedFileType::RigidModel => {
                            match PackedFileRigidModelView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
                            }
                        }

                        // If the file is a Image PackedFile...
                        PackedFileType::Image => {
                            match PackedFileImageView::new_view(&path, &mut tab) {
                                Ok((slots, packed_file_info)) => {
                                    slot_holder.borrow_mut().push(slots);

                                    // Add the file to the 'Currently open' list and make it visible.
                                    self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                                    self.tab_bar_packed_file.set_current_widget(tab_widget);
                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.insert(path.borrow().to_vec(), tab);
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                },
                                Err(error) => return show_dialog(self.main_window, ErrorKind::ImageDecode(format!("{}", error)), false),
                            }
                        }

                        // For any other PackedFile, just restore the display tips.
                        _ => {
                            //purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                            //display_help_tips(&app_ui);
                        }
                    }
                }

                // If it's external, we just create a view with just one button: "Stop Watching External File".
                else {
                    let name = path.last().unwrap().to_owned();
                    let icon_type = IconType::File(path.to_vec());
                    let icon = icon_type.get_icon_from_path();
                    let path = Rc::new(RefCell::new(path.to_vec()));

                    match PackedFileExternalView::new_view(&path, self,  &mut tab, global_search_ui, pack_file_contents_ui) {
                        Ok(slots) => {
                            slot_holder.borrow_mut().push(slots);

                            // Add the file to the 'Currently open' list and make it visible.
                            self.tab_bar_packed_file.add_tab_3a(tab_widget, icon, &QString::from_std_str(&name));
                            self.tab_bar_packed_file.set_current_widget(tab_widget);
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.insert(path.borrow().to_vec(), tab);
                        }
                        Err(error) => show_dialog(self.main_window, ErrorKind::LocDecode(format!("{}", error)), false),
                    }
                }
            }
        }
    }

    /// This function is used to open the PackedFile Decoder.
    pub unsafe fn open_decoder(
        &mut self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) {

        // If we don't have an schema, don't even try it.
        if SCHEMA.read().unwrap().is_none() {
            return show_dialog(self.main_window, ErrorKind::SchemaNotFound, false);
        }

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {
            let mut selected_items = <MutPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &mut selected_items[0] } else { return };
            if let TreePathType::File(ref mut path) = item_type {
                let mut fake_path = path.to_vec();
                *fake_path.last_mut().unwrap() = format!("{}-rpfm-decoder", fake_path.last_mut().unwrap());

                // Close all preview views except the file we're opening.
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = self.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if open_path != path && packed_file_view.get_is_preview() && index != -1 {
                        self.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // Close all preview views except the file we're opening. The path used for the decoder is empty.
                let name = qtr("decoder_title");
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = self.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                    if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                        self.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the decoder is already open, or it's hidden, we show it/focus it, instead of opening it again.
                if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(&fake_path) {
                    let index = self.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                    if index == -1 {
                        let icon_type = IconType::PackFile(true);
                        let icon = icon_type.get_icon_from_path();
                        self.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                    }

                    self.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                    return;
                }

                // If it's not already open/hidden, we create it and add it as a new tab.
                let mut tab = PackedFileView::default();
                tab.set_is_preview(false);
                let icon_type = IconType::PackFile(true);
                let icon = icon_type.get_icon_from_path();

                let path = Rc::new(RefCell::new(path.to_vec()));
                match PackedFileDecoderView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui, &self) {
                    Ok(slots) => {
                        slot_holder.borrow_mut().push(slots);

                        // Add the decoder to the 'Currently open' list and make it visible.
                        self.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                        self.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                        let mut open_list = UI_STATE.set_open_packedfiles();
                        open_list.insert(fake_path, tab);
                    },
                    Err(error) => return show_dialog(self.main_window, ErrorKind::DecoderDecode(format!("{}", error)), false),
                }
            }
        }
    }

    /// This function is used to open the dependency manager.
    pub unsafe fn open_dependency_manager(
        &mut self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            // Close all preview views except the file we're opening. The path used for the manager is empty.
            let path = vec![];
            let name = qtr("table_dependency_manager_title");
            for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                let index = self.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                    self.tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the manager is already open, or it's hidden, we show it/focus it, instead of opening it again.
            if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(&path) {
                let index = self.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                if index == -1 {
                    let icon_type = IconType::PackFile(true);
                    let icon = icon_type.get_icon_from_path();
                    self.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                }

                self.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                return;
            }

            // If it's not already open/hidden, we create it and add it as a new tab.
            let mut tab = PackedFileView::default();
            tab.set_is_preview(false);
            let icon_type = IconType::PackFile(true);
            let icon = icon_type.get_icon_from_path();

            let path = Rc::new(RefCell::new(path.to_vec()));
            match PackedFileTableView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                Ok((slots, _)) => {
                    slot_holder.borrow_mut().push(slots);

                    // Add the manager to the 'Currently open' list and make it visible.
                    self.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                    self.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                    let mut open_list = UI_STATE.set_open_packedfiles();
                    open_list.insert(path.borrow().to_vec(), tab);
                },
                Err(error) => return show_dialog(self.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
            }
        }
    }

    /// This function is used to open the notes embebed into a PackFile.
    pub unsafe fn open_notes(
        &mut self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            // Close all preview views except the file we're opening. The path used for the notes is reserved.
            let path = vec!["notes.rpfm_reserved".to_owned()];
            let name = qtr("notes");
            for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                let index = self.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if open_path != &path && packed_file_view.get_is_preview() && index != -1 {
                    self.tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the notes are already open, or are hidden, we show them/focus them, instead of opening them again.
            if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(&path) {
                let index = self.tab_bar_packed_file.index_of(tab_widget.get_mut_widget());

                if index == -1 {
                    let icon_type = IconType::PackFile(true);
                    let icon = icon_type.get_icon_from_path();
                    self.tab_bar_packed_file.add_tab_3a(tab_widget.get_mut_widget(), icon, &name);
                }

                self.tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget());
                return;
            }

            // If it's not already open/hidden, we create it and add it as a new tab.
            let mut tab = PackedFileView::default();
            tab.set_is_preview(false);
            let icon_type = IconType::PackFile(true);
            let icon = icon_type.get_icon_from_path();

            let path = Rc::new(RefCell::new(path.to_vec()));
            match PackedFileTextView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                Ok((slots, _)) => {
                    slot_holder.borrow_mut().push(slots);

                    // Add the manager to the 'Currently open' list and make it visible.
                    self.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                    self.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                    let mut open_list = UI_STATE.set_open_packedfiles();
                    open_list.insert(path.borrow().to_vec(), tab);
                },
                Err(error) => return show_dialog(self.main_window, ErrorKind::TextDecode(format!("{}", error)), false),
            }
        }
    }

    /// This function is the one that takes care of the creation of different PackedFiles.
    pub unsafe fn new_packed_file(&self, pack_file_contents_ui: &mut PackFileContentsUI, packed_file_type: PackedFileType) {

        // Create the "New PackedFile" dialog and wait for his data (or a cancelation). If we receive None, we do nothing. If we receive Some,
        // we still have to check if it has been any error during the creation of the PackedFile (for example, no definition for DB Tables).
        if let Some(new_packed_file) = self.new_packed_file_dialog(packed_file_type) {
            match new_packed_file {
                Ok(mut new_packed_file) => {

                    // First we make sure the name is correct, and fix it if needed.
                    match new_packed_file {
                        NewPackedFile::Loc(ref mut name) |
                        NewPackedFile::Text(ref mut name) |
                        NewPackedFile::DB(ref mut name, _, _) => {

                            // If the name is_empty, stop.
                            if name.is_empty() {
                                return show_dialog(self.main_window, ErrorKind::EmptyInput, false)
                            }

                            // Fix their name termination if needed.
                            if let PackedFileType::Loc = packed_file_type {
                                if !name.ends_with(loc::EXTENSION) { name.push_str(loc::EXTENSION); }
                            }
                            if let PackedFileType::Text(TextType::Plain) = packed_file_type {
                                if !text::EXTENSIONS.iter().any(|(x, _)| name.ends_with(x)) {
                                    name.push_str(".txt");
                                }
                            }
                        }
                    }

                    // If we reach this place, we got all alright.
                    match new_packed_file {
                        NewPackedFile::Loc(ref name) |
                        NewPackedFile::Text(ref name) |
                        NewPackedFile::DB(ref name, _, _) => {

                            // Get the currently selected paths (or the complete path, in case of DB Tables),
                            // and only continue if there is only one and it's not empty.
                            let selected_paths = <MutPtr<QTreeView> as PackTree>::get_path_from_main_treeview_selection(pack_file_contents_ui);
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
                                CENTRAL_COMMAND.send_message_qt(Command::PackedFileExists(complete_path.to_vec()));
                                let response = CENTRAL_COMMAND.recv_message_qt();
                                let exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                                if exists { return show_dialog(self.main_window, ErrorKind::FileAlreadyInPackFile, false)}

                                // Get the response, just in case it failed.
                                CENTRAL_COMMAND.send_message_qt(Command::NewPackedFile(complete_path.to_vec(), new_packed_file));
                                let response = CENTRAL_COMMAND.recv_message_qt();
                                match response {
                                    Response::Success => {
                                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(complete_path); 1]));

                                        /*
                                        // If, for some reason, there is a TableState data for this file, remove it.
                                        if table_state_data.borrow().get(&complete_path).is_some() {
                                            table_state_data.borrow_mut().remove(&complete_path);
                                        }

                                        // Set it to not remove his color.
                                        let data = TableStateData::new_empty();
                                        table_state_data.borrow_mut().insert(complete_path, data);
                                        */
                                    }

                                    Response::Error(error) => show_dialog(self.main_window, error, false),
                                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                                }
                            }
                        }
                    }
                }
                Err(error) => show_dialog(self.main_window, error, false),
            }
        }
    }

    /// This function creates a new PackedFile based on the current path selection, being:
    /// - `db/xxxx` -> DB Table.
    /// - `text/xxxx` -> Loc Table.
    /// - `script/xxxx` -> Lua PackedFile.
    /// The name used for each packfile is a generic one.
    pub unsafe fn new_queek_packed_file(&self, pack_file_contents_ui: &mut PackFileContentsUI) {

        // Get the currently selected path and, depending on the selected path, generate one packfile or another.
        let selected_paths = <MutPtr<QTreeView> as PackTree>::get_path_from_main_treeview_selection(pack_file_contents_ui);
        if selected_paths.len() == 1 {
            let path = &selected_paths[0];
            if let Some(mut name) = self.new_packed_file_name_dialog() {

                // DB Check.
                let (new_path, new_packed_file) = if path.starts_with(&["db".to_owned()]) && path.len() == 2 || path.len() == 3 {
                    let new_path = vec!["db".to_owned(), path[1].to_owned(), name];
                    let table = &path[1];

                    CENTRAL_COMMAND.send_message_qt(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return show_dialog(self.main_window, error, false),
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

                    let new_packed_file = NewPackedFile::Text(new_path.last().unwrap().to_owned());
                    (new_path, new_packed_file)
                } else { return show_dialog(self.main_window, ErrorKind::NoQueekPackedFileHere, false); };

                // Check if the PackedFile already exists, and report it if so.
                CENTRAL_COMMAND.send_message_qt(Command::PackedFileExists(new_path.to_vec()));
                let response = CENTRAL_COMMAND.recv_message_qt();
                let exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                if exists { return show_dialog(self.main_window, ErrorKind::FileAlreadyInPackFile, false)}

                // Create the PackFile.
                CENTRAL_COMMAND.send_message_qt(Command::NewPackedFile(new_path.to_vec(), new_packed_file));
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::Success => pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(new_path); 1])),
                    Response::Error(error) => show_dialog(self.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }
    }

    /// This function creates the entire "New Folder" dialog.
    ///
    /// It returns the new name of the Folder, or None if the dialog is canceled or closed.
    pub unsafe fn new_folder_dialog(&self) -> Option<String> {
        let mut dialog = QDialog::new_1a(self.main_window).into_ptr();
        dialog.set_window_title(&qtr("new_folder"));
        dialog.set_modal(true);

        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());

        let mut new_folder_line_edit = QLineEdit::new();
        new_folder_line_edit.set_text(&qtr("new_folder_default"));
        let mut new_folder_button = QPushButton::from_q_string(&qtr("new_folder"));

        main_grid.add_widget_5a(&mut new_folder_line_edit, 0, 0, 1, 1);
        main_grid.add_widget_5a(&mut new_folder_button, 0, 1, 1, 1);
        new_folder_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 { Some(new_folder_line_edit.text().to_std_string()) }
        else { None }
    }

    /// This function creates all the "New PackedFile" dialogs.
    ///
    /// It returns the type/name of the new file, or None if the dialog is canceled or closed.
    pub unsafe fn new_packed_file_dialog(&self, packed_file_type: PackedFileType) -> Option<Result<NewPackedFile>> {

        // Create and configure the "New PackedFile" Dialog.
        let mut dialog = QDialog::new_1a(self.main_window).into_ptr();
        match packed_file_type {
            PackedFileType::DB => dialog.set_window_title(&qtr("new_db_file")),
            PackedFileType::Loc => dialog.set_window_title(&qtr("new_loc_file")),
            PackedFileType::Text(_) => dialog.set_window_title(&qtr("new_txt_file")),
            _ => unimplemented!(),
        }
        dialog.set_modal(true);

        // Create the main Grid and his widgets.
        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        let mut name_line_edit = QLineEdit::new().into_ptr();
        let mut table_filter_line_edit = QLineEdit::new().into_ptr();
        let mut create_button = QPushButton::from_q_string(&qtr("gen_loc_create"));
        let mut table_dropdown = QComboBox::new_0a();
        let mut table_filter = QSortFilterProxyModel::new_0a();
        let mut table_model = QStandardItemModel::new_0a();

        name_line_edit.set_text(&qtr("new_file_default"));
        table_dropdown.set_model(&mut table_model);
        table_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        // Add all the widgets to the main grid, except those specific for a PackedFileType.
        main_grid.add_widget_5a(name_line_edit, 0, 0, 1, 1);
        main_grid.add_widget_5a(&mut create_button, 0, 1, 1, 1);

        // If it's a DB Table, add its widgets, and populate the table list.
        if let PackedFileType::DB = packed_file_type {
            CENTRAL_COMMAND.send_message_qt(Command::GetTableListFromDependencyPackFile);
            let response = CENTRAL_COMMAND.recv_message_qt();
            let tables = if let Response::VecString(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
            match *SCHEMA.read().unwrap() {
                Some(ref schema) => {

                    // Add every table to the dropdown if exists in the dependency database.
                    schema.get_ref_versioned_file_db_all().iter()
                        .filter_map(|x| if let VersionedFile::DB(name, _) = x { Some(name) } else { None })
                        .filter(|x| tables.contains(&x))
                        .for_each(|x| table_dropdown.add_item_q_string(&QString::from_std_str(&x)));
                    table_filter.set_source_model(&mut table_model);
                    table_dropdown.set_model(&mut table_filter);

                    main_grid.add_widget_5a(&mut table_dropdown, 1, 0, 1, 1);
                    main_grid.add_widget_5a(table_filter_line_edit, 2, 0, 1, 1);
                }
                None => return Some(Err(ErrorKind::SchemaNotFound.into())),
            }
        }

        // What happens when we search in the filter.
        let slot_table_filter_change_text = SlotOfQString::new(move |_| {
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
                PackedFileType::DB => {
                    let table = table_dropdown.current_text().to_std_string();
                    CENTRAL_COMMAND.send_message_qt(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return Some(Err(error)),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    };
                    Some(Ok(NewPackedFile::DB(packed_file_name, table, version)))
                },
                PackedFileType::Loc => Some(Ok(NewPackedFile::Loc(packed_file_name))),
                PackedFileType::Text(_) => Some(Ok(NewPackedFile::Text(packed_file_name))),
                _ => unimplemented!(),
            }
        }

        // Otherwise, return None.
        else { None }
    }

    /// This function creates the "New PackedFile's Name" dialog when creating a new QueeK PackedFile.
    ///
    /// It returns the new name of the PackedFile, or `None` if the dialog is canceled or closed.
    unsafe fn new_packed_file_name_dialog(&self) -> Option<String> {

        // Create and configure the dialog.
        let mut dialog = QDialog::new_1a(self.main_window).into_ptr();
        dialog.set_window_title(&qtr("new_packedfile_name"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);

        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        let mut name_line_edit = QLineEdit::new();
        let mut accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        name_line_edit.set_text(&qtr("trololol"));

        main_grid.add_widget_5a(&mut name_line_edit, 1, 0, 1, 1);
        main_grid.add_widget_5a(&mut accept_button, 1, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = name_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(name_line_edit.text().to_std_string()) }
        } else { None }
    }

    /// This function creates the entire "Merge Tables" dialog. It returns the stuff set in it.
    pub unsafe fn merge_tables_dialog(&self) -> Option<(String, bool)> {

        let mut dialog = QDialog::new_1a(self.main_window).into_ptr();
        dialog.set_window_title(&qtr("packedfile_merge_tables"));
        dialog.set_modal(true);

        // Create the main Grid.
        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        let mut name = QLineEdit::new();
        name.set_placeholder_text(&qtr("merge_tables_new_name"));

        let mut delete_source_tables = QCheckBox::from_q_string(&qtr("merge_tables_delete_option"));

        let mut accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));
        main_grid.add_widget_5a(&mut name, 0, 0, 1, 1);
        main_grid.add_widget_5a(&mut delete_source_tables, 1, 0, 1, 1);
        main_grid.add_widget_5a(&mut accept_button, 2, 0, 1, 1);

        // What happens when we hit the "Search" button.
        accept_button.released().connect(dialog.slot_accept());

        // Execute the dialog.
        if dialog.exec() == 1 {
            let text = name.text().to_std_string();
            let delete_source_tables = delete_source_tables.is_checked();
            if !text.is_empty() { Some((text, delete_source_tables)) }
            else { None }
        }

        // Otherwise, return None.
        else { None }
    }
}
