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
Module with all the code for extra implementations of `AppUI`.

This module contains the implementation of custom functions for `AppUI`. The reason
they're here and not in the main file is because I don't want to polute that one,
as it's mostly meant for initialization and configuration.
!*/

use qt_widgets::file_dialog::FileDialog;
use qt_widgets::{message_box, message_box::MessageBox};
use qt_widgets::widget::Widget;

use qt_core::object::Object;

use std::path::PathBuf;

use rpfm_error::Result;

use rpfm_lib::common::get_game_selected_data_path;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::packfile::{PFHFileType, PFHFlags, CompressionState, PFHVersion};
use rpfm_lib::SETTINGS;

use super::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::{new_pack_file_tooltip, PackTree, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUI`.
impl AppUI {

    /// This function takes care of updating the Main Window's title to reflect the current state of the program.
    pub fn update_window_title(&self, packfile_contents_ui: &PackFileContentsUI) {

        // First check if we have a PackFile open. If not, just leave the default title.
        let model = unsafe { packfile_contents_ui.packfile_contents_tree_model.as_ref().unwrap() };
        let main_window = unsafe { self.main_window.as_mut().unwrap() };
        let window_title;

        if model.row_count(()) == 0 { window_title = "Rusted PackFile Manager".to_owned(); }

        // If there is a `PackFile` open, check if it has been modified, and set the title accordingly.
        else {
            let pack_file_name = unsafe { model.item(0).as_ref().unwrap().text().to_std_string() };
            if UI_STATE.get_is_modified() { window_title = format!("{} - Modified", pack_file_name); }
            else { window_title = format!("{} - Not Modified", pack_file_name); }
        }
        main_window.set_window_title(&QString::from_std_str(window_title));
    }

    /// This function pops up a modal asking you if you're sure you want to do an action that may result in unsaved data loss.
    ///
    /// If you are trying to delete the open MyMod, pass it true.
    pub fn are_you_sure(&self, is_delete_my_mod: bool) -> bool {
        let title = "Rusted PackFile Manager";
        let message = if is_delete_my_mod { "<p>You are about to delete this <i>'MyMod'</i> from your disk.</p><p>There is no way to recover it after that.</p><p>Are you sure?</p>" }
        else if UI_STATE.get_is_modified() { "<p>There are some changes yet to be saved.</p><p>Are you sure?</p>" }

        // In any other situation... just return true and forget about the dialog.
        else { return true };

        // Create the dialog and run it (Yes => 3, No => 4).
        unsafe { MessageBox::new_unsafe((
            &QString::from_std_str(title),
            &QString::from_std_str(message),
            message_box::Icon::Warning,
            65536, // No
            16384, // Yes
            1, // By default, select yes.
            self.main_window as *mut Widget,
        )) }.exec() == 3
    }

    /// This function deletes all the widgets corresponding to opened PackedFiles.
    pub fn purge_them_all(&self) {

        // Black magic.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        for ui in open_packedfiles.values_mut() {
            let ui: *mut Widget = &mut **ui;
            unsafe { (ui as *mut Object).as_mut().unwrap().delete_later(); }
        }

        // Set it as not having an opened PackedFile, just in case.
        open_packedfiles.clear();

        // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
        unsafe { self.game_selected_group.as_mut().unwrap().set_enabled(true); }

        // Just in case what was open before was the `Add From PackFile` TreeView, unlock it.
        UI_STATE.set_packfile_contents_read_only(false);
    }

    /// This function deletes all the widgets corresponding to the specified PackedFile, if exists.
    pub fn purge_that_one_specifically(app_ui: &AppUI, path: &[String]) {

        // Black magic to remove widgets.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        if let Some(ui) = open_packedfiles.get_mut(path) {
            let ui: *mut Widget = &mut **ui;
            unsafe { (ui as *mut Object).as_mut().unwrap().delete_later(); }
        }

        // Set it as not having an opened PackedFile, just in case.
        open_packedfiles.remove(path);

        // We check if there are more tables open. This is beacuse we cannot change the GameSelected
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

        if enable_game_selected_menu { unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(true); }}
    }

    /// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending on the situation.
    ///
    /// NOTE: The `game_folder` is for when using this function with *MyMods*. If you're opening a normal mod, pass it empty.
    pub fn open_packfile(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        pack_file_paths: &[PathBuf],
        game_folder: &str,
    ) -> Result<()> {

        // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
        unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFiles(pack_file_paths.to_vec()));

        // Check what response we got.
        match CENTRAL_COMMAND.recv_message_qt() {

            // If it's success....
            Response::PackFileInfo(ui_data) => {

                // We choose the right option, depending on our PackFile.
                match ui_data.pfh_file_type {
                    PFHFileType::Boot => unsafe { self.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                    PFHFileType::Release => unsafe { self.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                    PFHFileType::Patch => unsafe { self.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                    PFHFileType::Mod => unsafe { self.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                    PFHFileType::Movie => unsafe { self.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                    PFHFileType::Other(_) => unsafe { self.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                }

                // Enable or disable these, depending on what data we have in the header.
                unsafe { self.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA)); }
                unsafe { self.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS)); }
                unsafe { self.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX)); }
                unsafe { self.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER)); }

                // Set the compression level correctly, because otherwise we may fuckup some files.
                let compression_state = match ui_data.compression_state {
                    CompressionState::Enabled => true,
                    CompressionState::Partial | CompressionState::Disabled => false,
                };
                unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_checked(compression_state); }

                // Update the TreeView.
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(false));

                // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
                if !game_folder.is_empty() && pack_file_paths.len() == 1 {

                    // NOTE: Arena should never be here.
                    // Change the Game Selected in the UI.
                    match game_folder {
                        "three_kingdoms" => unsafe { self.game_selected_three_kingdoms.as_mut().unwrap().trigger(); }
                        "warhammer_2" => unsafe { self.game_selected_warhammer_2.as_mut().unwrap().trigger(); }
                        "warhammer" => unsafe { self.game_selected_warhammer.as_mut().unwrap().trigger(); }
                        "thrones_of_britannia" => unsafe { self.game_selected_thrones_of_britannia.as_mut().unwrap().trigger(); }
                        "attila" => unsafe { self.game_selected_attila.as_mut().unwrap().trigger(); }
                        "rome_2" => unsafe { self.game_selected_rome_2.as_mut().unwrap().trigger(); }
                        "shogun_2" => unsafe { self.game_selected_shogun_2.as_mut().unwrap().trigger(); }
                        "napoleon" => unsafe { self.game_selected_napoleon.as_mut().unwrap().trigger(); }
                        "empire" | _ => unsafe { self.game_selected_empire.as_mut().unwrap().trigger(); }
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
                                unsafe { self.game_selected_arena.as_mut().unwrap().trigger(); }
                            }

                            // Otherwise, it's from Three Kingdoms or Warhammer 2.
                            else {
                                let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                                match &*game_selected {
                                    "three_kingdoms" => unsafe { self.game_selected_three_kingdoms.as_mut().unwrap().trigger(); },
                                    "warhammer_2" | _ => unsafe { self.game_selected_warhammer_2.as_mut().unwrap().trigger(); },
                                }
                            }
                        },

                        // PFH4 is for Thrones of Britannia/Warhammer 1/Attila/Rome 2.
                        PFHVersion::PFH4 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                            match &*game_selected {
                                "warhammer" => unsafe { self.game_selected_warhammer.as_mut().unwrap().trigger(); },
                                "thrones_of_britannia" => unsafe { self.game_selected_thrones_of_britannia.as_mut().unwrap().trigger(); }
                                "attila" => unsafe { self.game_selected_attila.as_mut().unwrap().trigger(); }
                                "rome_2" | _ => unsafe { self.game_selected_rome_2.as_mut().unwrap().trigger(); }
                            }
                        },

                        // PFH3 is for Shogun 2.
                        PFHVersion::PFH3 => unsafe { self.game_selected_shogun_2.as_mut().unwrap().trigger(); }

                        // PFH0 is for Napoleon/Empire.
                        PFHVersion::PFH0 => {
                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                            match &*game_selected {
                                "napoleon" => unsafe { self.game_selected_napoleon.as_mut().unwrap().trigger(); },
                                "empire" | _ => unsafe { self.game_selected_empire.as_mut().unwrap().trigger(); }
                            }
                        },
                    }

                    // Set the current "Operational Mode" to `Normal`.
                    UI_STATE.set_operational_mode(self, None);
                }

                // Re-enable the Main Window.
                unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                self.purge_them_all();

                // Close the Global Search stuff and reset the filter's history.
                //unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                //if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                // Show the "Tips".
                //display_help_tips(&app_ui);

                // Clean the TableStateData.
                //*table_state_data.borrow_mut() = TableStateData::new();
            }

            // If we got an error...
            Response::Error(error) => {
                unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                return Err(error)
            }

            // In ANY other situation, it's a message problem.
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        }

        // Return success.
        Ok(())
    }


    /// This function is used to save the currently open `PackFile` to disk.
    ///
    /// If the PackFile doesn't exist or we pass `save_as = true`,
    /// it opens a dialog asking for a path.
    pub fn save_packfile(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        save_as: bool,
    ) -> Result<()> {

        let mut result = Ok(());
        let main_window = unsafe { self.main_window.as_mut().unwrap() as &mut Widget};
        main_window.set_enabled(false);

        CENTRAL_COMMAND.send_message_qt(Command::GetPackFilePath);
        let path = if let Response::PathBuf(path) = CENTRAL_COMMAND.recv_message_qt() { path } else { panic!(THREADS_COMMUNICATION_ERROR) };
        if !path.is_file() || save_as {

            // Create the FileDialog to save the PackFile and configure it.
            let mut file_dialog = unsafe { FileDialog::new_unsafe((
                self.main_window as *mut Widget,
                &QString::from_std_str("Save PackFile"),
            )) };
            file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);
            file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
            file_dialog.set_confirm_overwrite(true);
            file_dialog.set_default_suffix(&QString::from_std_str("pack"));
            file_dialog.select_file(&QString::from_std_str(&path.file_name().unwrap().to_string_lossy()));

            // If we are saving an existing PackFile with another name, we start in his current path.
            if path.is_file() {
                let mut path = path.to_path_buf();
                path.pop();
                file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
            }

            // In case we have a default path for the Game Selected and that path is valid,
            // we use his data folder as base path for saving our PackFile.
            else if let Some(ref path) = get_game_selected_data_path(&*GAME_SELECTED.lock().unwrap()) {
                if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned())); }
            }

            // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {
                let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                CENTRAL_COMMAND.send_message_qt(Command::SavePackFileAs(path));
                match CENTRAL_COMMAND.recv_message_qt_try() {
                    Response::PackFileInfo(pack_file_info) => {
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean);
                        let packfile_item = unsafe { pack_file_contents_ui.packfile_contents_tree_model.as_mut().unwrap().item(0).as_mut().unwrap() };
                        packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                        packfile_item.set_text(&QString::from_std_str(&file_name));

                        UI_STATE.set_operational_mode(self, None);
                    }
                    Response::Error(error) => result = Err(error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                }
            }
        }

        else {
            CENTRAL_COMMAND.send_message_qt(Command::SavePackFile);
            match CENTRAL_COMMAND.recv_message_qt_try() {
                Response::PackFileInfo(pack_file_info) => {
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clean);
                    let packfile_item = unsafe { pack_file_contents_ui.packfile_contents_tree_model.as_mut().unwrap().item(0).as_mut().unwrap() };
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                }
                Response::Error(error) => result = Err(error),

                // In ANY other situation, it's a message problem.
                _ => panic!(THREADS_COMMUNICATION_ERROR),
            }
        }

        // Then we re-enable the main Window and return whatever we've received.
        main_window.set_enabled(true);

        // Clean all the modified items EXCEPT those open. That way we can still undo changes there.
        /*
        if result.is_ok() {
            let iter = table_state_data.borrow().iter().map(|x| x.0).cloned().collect::<Vec<Vec<String>>>();
            for path in &iter {
                if !packedfiles_open_in_packedfile_view.borrow().values().any(|x| *x.borrow() == *path) {
                    table_state_data.borrow_mut().remove(path);
                }
            }
        }*/
        result
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub fn enable_packfile_actions(&self, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        if &**GAME_SELECTED.lock().unwrap() == "arena" {

            // Disable the actions that allow to create and save PackFiles.
            unsafe { self.packfile_new_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.packfile_save_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.packfile_save_packfile_as.as_mut().unwrap().set_enabled(false); }

            // This one too, though we had to deal with it specially later on.
            unsafe { self.mymod_new.as_mut().unwrap().set_enabled(false); }
        }

        // Otherwise...
        else {

            // Enable or disable the actions from "PackFile" Submenu.
            unsafe { self.packfile_new_packfile.as_mut().unwrap().set_enabled(true); }
            unsafe { self.packfile_save_packfile.as_mut().unwrap().set_enabled(enable); }
            unsafe { self.packfile_save_packfile_as.as_mut().unwrap().set_enabled(enable); }

            // If there is a "MyMod" path set in the settings...
            if let Some(ref path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                // And it's a valid directory, enable the "New MyMod" button.
                if path.is_dir() { unsafe { self.mymod_new.as_mut().unwrap().set_enabled(true); }}

                // Otherwise, disable it.
                else { unsafe { self.mymod_new.as_mut().unwrap().set_enabled(false); }}
            }

            // Otherwise, disable it.
            else { unsafe { self.mymod_new.as_mut().unwrap().set_enabled(false); }}
        }

        // These actions are common, no matter what game we have.
        unsafe { self.change_packfile_type_group.as_mut().unwrap().set_enabled(enable); }
        unsafe { self.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_enabled(enable); }

        // If we are enabling...
        if enable {

            // Check the Game Selected and enable the actions corresponding to out game.
            match &**GAME_SELECTED.lock().unwrap() {
                "three_kingdoms" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_three_k_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_three_k_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "warhammer_2" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "warhammer" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_wh_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "thrones_of_britannia" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_tob_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_tob_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "attila" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_att_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_att_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "rome_2" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_rom2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_rom2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "shogun_2" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_sho2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_sho2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                "napoleon" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_nap_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                },
                "empire" => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_emp_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                },
                _ => {},
            }
        }

        // If we are disabling...
        else {

            // Universal Actions.
            unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }

            // Disable Three Kingdoms actions...
            unsafe { self.special_stuff_three_k_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_three_k_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Warhammer 2 actions...
            unsafe { self.special_stuff_wh2_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_wh2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_wh2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Warhammer actions...
            unsafe { self.special_stuff_wh_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_wh_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_wh_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Thrones of Britannia actions...
            unsafe { self.special_stuff_tob_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_tob_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Attila actions...
            unsafe { self.special_stuff_att_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_att_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Rome 2 actions...
            unsafe { self.special_stuff_rom2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_rom2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Shogun 2 actions...
            unsafe { self.special_stuff_sho2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
            unsafe { self.special_stuff_sho2_generate_pak_file.as_mut().unwrap().set_enabled(false); }

            // Disable Napoleon actions...
            unsafe { self.special_stuff_nap_optimize_packfile.as_mut().unwrap().set_enabled(false); }

            // Disable Empire actions...
            unsafe { self.special_stuff_emp_optimize_packfile.as_mut().unwrap().set_enabled(false); }
        }

        // The assembly kit thing should only be available for Rome 2 and later games.
        match &**GAME_SELECTED.lock().unwrap() {
            "three_kingdoms" |
            "warhammer_2" |
            "warhammer" |
            "thrones_of_britannia" |
            "attila" |
            "rome_2" => unsafe { self.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(true); }
            _ => unsafe { self.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(false); }
        }
    }
}
