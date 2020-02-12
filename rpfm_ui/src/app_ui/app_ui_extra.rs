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

use qt_widgets::check_box::CheckBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::{message_box, message_box::MessageBox};
use qt_widgets::push_button::PushButton;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::object::Object;
use qt_core::reg_exp::RegExp;
use qt_core::slots::{SlotBool, SlotStringRef};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;

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
use crate::locale::qtr;
use crate::pack_tree::{icons::IconType, new_pack_file_tooltip, PackTree, TreePathType, TreeViewOperation};
use crate::packedfile_views::{decoder::*, image::*, PackedFileView, rigidmodel::*, table::*, TheOneSlot, text::*};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;
use crate::utils::{create_grid_layout_unsafe, show_dialog};

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
    pub fn purge_them_all(&self,
        global_search_ui: GlobalSearchUI,
        pack_file_contents_ui: PackFileContentsUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>
    ) {

        // Black magic.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        for (path, packed_file_view) in open_packedfiles.iter_mut() {

            // TODO: This should report an error.
            let _ = packed_file_view.save(path, global_search_ui, &pack_file_contents_ui);
            let widget: *mut Widget = packed_file_view.get_mut_widget();
            let index = unsafe { self.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };
            unsafe { self.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }

            // Delete the widget manually to free memory.
            unsafe { (widget as *mut Object).as_mut().unwrap().delete_later(); }
        }

        // Remove all open PackedFiles and their slots.
        open_packedfiles.clear();
        slot_holder.borrow_mut().clear();

        // Just in case what was open before this was a DB Table, make sure the "Game Selected" menu is re-enabled.
        unsafe { self.game_selected_group.as_mut().unwrap().set_enabled(true); }

        // Just in case what was open before was the `Add From PackFile` TreeView, unlock it.
        UI_STATE.set_packfile_contents_read_only(false);
    }

    /// This function deletes all the widgets corresponding to the specified PackedFile, if exists.
    pub fn purge_that_one_specifically(&self,
        global_search_ui: GlobalSearchUI,
        pack_file_contents_ui: PackFileContentsUI,
        path: &[String],
        save_before_deleting: bool
    ) {

        // Black magic to remove widgets.
        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
        if let Some(packed_file_view) = open_packedfiles.get_mut(path) {
            if save_before_deleting && path != ["extra_packfile.rpfm_reserved".to_owned()] {

                // TODO: This should report an error.
                let _ = packed_file_view.save(path, global_search_ui, &pack_file_contents_ui);
            }
            let widget: *mut Widget = packed_file_view.get_mut_widget();
            let index = unsafe { self.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };
            unsafe { self.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }

            // Delete the widget manually to free memory.
            unsafe { (widget as *mut Object).as_mut().unwrap().delete_later(); }
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
                    unsafe { self.game_selected_group.as_mut().unwrap().set_enabled(true); }
                }
            }
        }
    }

    /// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending on the situation.
    ///
    /// NOTE: The `game_folder` is for when using this function with *MyMods*. If you're opening a normal mod, pass it empty.
    pub fn open_packfile(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_paths: &[PathBuf],
        game_folder: &str,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) -> Result<()> {

        // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
        unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
        CENTRAL_COMMAND.send_message_qt(Command::OpenPackFiles(pack_file_paths.to_vec()));

        // Check what response we got.
        let response = CENTRAL_COMMAND.recv_message_qt_try();
        match response {

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

                // Re-enable the Main Window.
                unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                self.purge_them_all(*global_search_ui, *pack_file_contents_ui, slot_holder);

                // Close the Global Search stuff and reset the filter's history.
                global_search_ui.clear();

                // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
                if !game_folder.is_empty() && pack_file_paths.len() == 1 {

                    // NOTE: Arena should never be here.
                    // Change the Game Selected in the UI.
                    match game_folder {
                        KEY_THREE_KINGDOMS => unsafe { self.game_selected_three_kingdoms.as_mut().unwrap().trigger(); }
                        KEY_WARHAMMER_2 => unsafe { self.game_selected_warhammer_2.as_mut().unwrap().trigger(); }
                        KEY_WARHAMMER => unsafe { self.game_selected_warhammer.as_mut().unwrap().trigger(); }
                        KEY_THRONES_OF_BRITANNIA => unsafe { self.game_selected_thrones_of_britannia.as_mut().unwrap().trigger(); }
                        KEY_ATTILA => unsafe { self.game_selected_attila.as_mut().unwrap().trigger(); }
                        KEY_ROME_2 => unsafe { self.game_selected_rome_2.as_mut().unwrap().trigger(); }
                        KEY_SHOGUN_2 => unsafe { self.game_selected_shogun_2.as_mut().unwrap().trigger(); }
                        KEY_NAPOLEON => unsafe { self.game_selected_napoleon.as_mut().unwrap().trigger(); }
                        KEY_EMPIRE | _ => unsafe { self.game_selected_empire.as_mut().unwrap().trigger(); }
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
                                let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                                match &*game_selected {
                                    KEY_THREE_KINGDOMS => unsafe { self.game_selected_three_kingdoms.as_mut().unwrap().trigger(); },
                                    KEY_WARHAMMER_2 | _ => unsafe { self.game_selected_warhammer_2.as_mut().unwrap().trigger(); },
                                }
                            }
                        },

                        // PFH4 is for Thrones of Britannia/Warhammer 1/Attila/Rome 2.
                        PFHVersion::PFH4 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                            match &*game_selected {
                                KEY_WARHAMMER => unsafe { self.game_selected_warhammer.as_mut().unwrap().trigger(); },
                                KEY_THRONES_OF_BRITANNIA => unsafe { self.game_selected_thrones_of_britannia.as_mut().unwrap().trigger(); }
                                KEY_ATTILA => unsafe { self.game_selected_attila.as_mut().unwrap().trigger(); }
                                KEY_ROME_2 | _ => unsafe { self.game_selected_rome_2.as_mut().unwrap().trigger(); }
                            }
                        },

                        // PFH3 is for Shogun 2.
                        PFHVersion::PFH3 | PFHVersion::PFH2 => unsafe { self.game_selected_shogun_2.as_mut().unwrap().trigger(); }

                        // PFH0 is for Napoleon/Empire.
                        PFHVersion::PFH0 => {
                            let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                            match &*game_selected {
                                KEY_NAPOLEON => unsafe { self.game_selected_napoleon.as_mut().unwrap().trigger(); },
                                KEY_EMPIRE | _ => unsafe { self.game_selected_empire.as_mut().unwrap().trigger(); }
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
                unsafe { (self.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
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
    pub fn save_packfile(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        save_as: bool,
    ) -> Result<()> {

        let mut result = Ok(());
        let main_window = unsafe { self.main_window.as_mut().unwrap() as &mut Widget};
        main_window.set_enabled(false);

        // First, we need to save all open `PackedFiles` to the backend.
        UI_STATE.get_open_packedfiles().iter().try_for_each(|(path, packed_file)| packed_file.save(path, *global_search_ui, pack_file_contents_ui))?;

        CENTRAL_COMMAND.send_message_qt(Command::GetPackFilePath);
        let response = CENTRAL_COMMAND.recv_message_qt();
        let mut path = if let Response::PathBuf(path) = response { path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };
        if !path.is_file() || save_as {

            // Create the FileDialog to save the PackFile and configure it.
            let mut file_dialog = unsafe { FileDialog::new_unsafe((
                self.main_window as *mut Widget,
                &qtr("save_packfile"),
            )) };
            file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);
            file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
            file_dialog.set_confirm_overwrite(true);
            file_dialog.set_default_suffix(&QString::from_std_str("pack"));
            file_dialog.select_file(&QString::from_std_str(&path.file_name().unwrap().to_string_lossy()));

            // If we are saving an existing PackFile with another name, we start in his current path.
            if path.is_file() {
                path.pop();
                file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
            }

            // In case we have a default path for the Game Selected and that path is valid,
            // we use his data folder as base path for saving our PackFile.
            else if let Some(ref path) = get_game_selected_data_path() {
                if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned())); }
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
                        let packfile_item = unsafe { pack_file_contents_ui.packfile_contents_tree_model.as_mut().unwrap().item(0).as_mut().unwrap() };
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
                    let packfile_item = unsafe { pack_file_contents_ui.packfile_contents_tree_model.as_mut().unwrap().item(0).as_mut().unwrap() };
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                }
                Response::Error(error) => result = Err(error),

                // In ANY other situation, it's a message problem.
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // Then we re-enable the main Window and return whatever we've received.
        main_window.set_enabled(true);
        result
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub fn enable_packfile_actions(&self, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        if &**GAME_SELECTED.read().unwrap() == KEY_ARENA {

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
            match &**GAME_SELECTED.read().unwrap() {
                KEY_THREE_KINGDOMS => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_three_k_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_three_k_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_WARHAMMER_2 => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_WARHAMMER => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_wh_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_wh_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_THRONES_OF_BRITANNIA => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_tob_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_tob_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_ATTILA => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_att_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_att_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_ROME_2 => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_rom2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_rom2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_SHOGUN_2 => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_sho2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                    unsafe { self.special_stuff_sho2_generate_pak_file.as_mut().unwrap().set_enabled(true); }
                },
                KEY_NAPOLEON => {
                    unsafe { self.change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }
                    unsafe { self.special_stuff_nap_optimize_packfile.as_mut().unwrap().set_enabled(true); }
                },
                KEY_EMPIRE => {
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
        match &**GAME_SELECTED.read().unwrap() {
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 => unsafe { self.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(true); }
            _ => unsafe { self.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_enabled(false); }
        }
    }

    /// This function takes care of recreating the dynamic submenus under `PackFile` menu.
    pub fn build_open_from_submenus(self, pack_file_contents_ui: PackFileContentsUI, global_search_ui: GlobalSearchUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> Vec<SlotBool<'static>> {
        let packfile_open_from_content = unsafe { self.packfile_open_from_content.as_mut().unwrap() };
        let packfile_open_from_data = unsafe { self.packfile_open_from_data.as_mut().unwrap() };

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
                let open_mod_action = packfile_open_from_content.add_action(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotBool::new(clone!(
                    path,
                    slot_holder => move |_| {
                    if self.are_you_sure(false) {
                        if let Err(error) = self.open_packfile(&pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "", &slot_holder) {
                            show_dialog(self.main_window as *mut Widget, error, false);
                        }
                    }
                }));

                // Connect the slot and store it.
                unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(&slot_open_mod); }
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
                let open_mod_action = packfile_open_from_data.add_action(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotBool::new(clone!(
                    path,
                    slot_holder => move |_| {
                    if self.are_you_sure(false) {
                        if let Err(error) = self.open_packfile(&pack_file_contents_ui, &global_search_ui, &[path.to_path_buf()], "", &slot_holder) {
                            show_dialog(self.main_window as *mut Widget, error, false);
                        }
                    }
                }));

                // Connect the slot and store it.
                unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(&slot_open_mod); }
                open_from_slots.push(slot_open_mod);
            }
        }

        // Only if the submenu has items, we enable it.
        unsafe { packfile_open_from_content.menu_action().as_mut().unwrap().set_visible(!packfile_open_from_content.actions().is_empty()); }
        unsafe { packfile_open_from_data.menu_action().as_mut().unwrap().set_visible(!packfile_open_from_data.actions().is_empty()); }

        // Return the slots.
        open_from_slots
    }


    /// This function takes care of the re-creation of the `MyMod` list for each game.
    pub fn build_open_mymod_submenus(self, pack_file_contents_ui: PackFileContentsUI, global_search_ui: GlobalSearchUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> Vec<SlotBool<'static>> {

        // First, we need to reset the menu, which basically means deleting all the game submenus and hiding them.
        unsafe { self.mymod_open_three_kingdoms.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_warhammer_2.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_warhammer.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_thrones_of_britannia.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_attila.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_rome_2.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_shogun_2.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_napoleon.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }
        unsafe { self.mymod_open_empire.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(false); }

        unsafe { self.mymod_open_three_kingdoms.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_warhammer_2.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_warhammer.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_thrones_of_britannia.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_attila.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_rome_2.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_shogun_2.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_napoleon.as_mut().unwrap().clear(); }
        unsafe { self.mymod_open_empire.as_mut().unwrap().clear(); }

        let mut slots = vec![];

        // If we have the "MyMod" path configured, get all the packfiles under the `MyMod` folder, separated by supported game.
        let supported_folders = SUPPORTED_GAMES.iter().filter(|(_, x)| x.supports_editing).map(|(folder_name,_)| *folder_name).collect::<Vec<&str>>();
        if let Some(ref mymod_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {
            if let Ok(game_folder_list) = mymod_base_path.read_dir() {
                for game_folder in game_folder_list {
                    if let Ok(game_folder) = game_folder {

                        // If it's a valid folder, and it's in our supported games list, get all the PackFiles inside it and create an open action for them.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        if game_folder.path().is_dir() && supported_folders.contains(&&*game_folder_name) {
                            let game_submenu = match &*game_folder_name {
                                KEY_THREE_KINGDOMS => unsafe { self.mymod_open_three_kingdoms.as_mut().unwrap() },
                                KEY_WARHAMMER_2 => unsafe { self.mymod_open_warhammer_2.as_mut().unwrap() },
                                KEY_WARHAMMER => unsafe { self.mymod_open_warhammer.as_mut().unwrap() },
                                KEY_THRONES_OF_BRITANNIA => unsafe { self.mymod_open_thrones_of_britannia.as_mut().unwrap() },
                                KEY_ATTILA => unsafe { self.mymod_open_attila.as_mut().unwrap() },
                                KEY_ROME_2 => unsafe { self.mymod_open_rome_2.as_mut().unwrap() },
                                KEY_SHOGUN_2 => unsafe { self.mymod_open_shogun_2.as_mut().unwrap() },
                                KEY_NAPOLEON => unsafe { self.mymod_open_napoleon.as_mut().unwrap() },
                                KEY_EMPIRE | _ => unsafe { self.mymod_open_empire.as_mut().unwrap() },
                            };

                            if let Ok(game_folder_files) = game_folder.path().read_dir() {
                                let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|x| x.unwrap().path()).collect();
                                game_folder_files_sorted.sort();

                                for pack_file in &game_folder_files_sorted {
                                    if pack_file.is_file() && pack_file.extension().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy() == "pack" {
                                        let pack_file = pack_file.clone();
                                        let mod_name = pack_file.file_name().unwrap().to_string_lossy();
                                        let open_mod_action = game_submenu.add_action(&QString::from_std_str(&mod_name));

                                        // Create the slot for that action.
                                        let slot_open_mod = SlotBool::new(clone!(
                                            slot_holder,
                                            game_folder_name => move |_| {
                                            if self.are_you_sure(false) {
                                                if let Err(error) = self.open_packfile(&pack_file_contents_ui, &global_search_ui, &[pack_file.to_path_buf()], &game_folder_name, &slot_holder) {
                                                    show_dialog(self.main_window as *mut Widget, error, false);
                                                }
                                            }
                                        }));

                                        unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(&slot_open_mod); }
                                        slots.push(slot_open_mod);
                                    }
                                }
                            }

                            // Only if the submenu has items, we show it to the big menu.
                            if game_submenu.actions().count() > 0 {
                                unsafe { game_submenu.menu_action().as_mut().unwrap().set_visible(true); }
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
    pub fn check_updates(&self, use_dialog: bool) {
        CENTRAL_COMMAND.send_message_qt_to_network(Command::CheckUpdates);

        // If we want to use a Dialog to show the full searching process (clicking in the
        // menu button) we show the dialog, then change its text.
        if use_dialog {
            let mut dialog = unsafe { MessageBox::new_unsafe((
                message_box::Icon::Information,
                &qtr("update_checker"),
                &qtr("update_searching"),
                Flags::from_int(2_097_152), // Close button.
                self.main_window as *mut Widget,
            )) };

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

            let mut dialog = unsafe { MessageBox::new_unsafe((
                message_box::Icon::Information,
                &qtr("update_checker"),
                &QString::from_std_str(message),
                Flags::from_int(2_097_152), // Close button.
                self.main_window as *mut Widget,
            )) };

            dialog.set_modal(true);
            dialog.exec();
        }
    }

    /// This function checks if there is any newer version of RPFM's schemas released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub fn check_schema_updates(&self, use_dialog: bool) {
        CENTRAL_COMMAND.send_message_qt_to_network(Command::CheckSchemaUpdates);

        // If we want to use a Dialog to show the full searching process.
        if use_dialog {

            // Create the dialog to show the response and configure it.
            let mut dialog = unsafe { MessageBox::new_unsafe((
                message_box::Icon::Information,
                &qtr("update_schema_checker"),
                &qtr("update_searching"),
                Flags::from_int(2_097_152), // Close button.
                self.main_window as *mut Widget,
            )) };

            let update_button = dialog.add_button((&qtr("update_button"), message_box::ButtonRole::AcceptRole));
            unsafe { update_button.as_mut().unwrap().set_enabled(false); }

            dialog.set_modal(true);
            dialog.show();

            // When we get a response, act depending on the kind of response we got.
            let response_thread = CENTRAL_COMMAND.recv_message_network_to_qt_try();
            let message = match response_thread {
                Response::APIResponseSchema(ref response) => {
                    match response {
                        APIResponseSchema::SuccessNewUpdate(ref local_versions, ref remote_versions) => {
                            unsafe { update_button.as_mut().unwrap().set_enabled(true); }

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
                        APIResponseSchema::SuccessNoUpdate => "<h4>No new schema updates available</h4> <p>More luck next time :)</p>".to_owned(),
                        APIResponseSchema::Error => "<h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>".to_owned(),
                    }
                }

                Response::Error(error) => return show_dialog(self.main_window as *mut Widget, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
            };

            // If we hit "Update", try to update the schemas.
            dialog.set_text(&QString::from_std_str(message));
            if dialog.exec() == 0 {
                if let Response::APIResponseSchema(ref response) = response_thread {
                    if let APIResponseSchema::SuccessNewUpdate(_,_) = response {

                        CENTRAL_COMMAND.send_message_qt(Command::UpdateSchemas);

                        dialog.show();
                        dialog.set_text(&qtr("update_in_prog"));
                        unsafe { update_button.as_mut().unwrap().set_enabled(false); }

                        match CENTRAL_COMMAND.recv_message_qt_try() {
                            Response::Success => show_dialog(self.main_window as *mut Widget, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>", true),
                            Response::Error(error) => show_dialog(self.main_window as *mut Widget, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response_thread),
                        }
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
                        _ => return
                    }
                }
                _ => return
            };

            // Create the dialog to show the response.
            let mut dialog = unsafe { MessageBox::new_unsafe((
                message_box::Icon::Information,
                &qtr("update_schema_checker"),
                &QString::from_std_str(message),
                Flags::from_int(2_097_152), // Close button.
                self.main_window as *mut Widget,
            )) };

            let update_button = dialog.add_button((&qtr("update_button"), message_box::ButtonRole::AcceptRole));
            dialog.set_modal(true);

            // If we hit "Update", try to update the schemas.
            if dialog.exec() == 0 {
                if let Response::APIResponseSchema(response) = response_thread {
                    if let APIResponseSchema::SuccessNewUpdate(_,_) = response {

                        CENTRAL_COMMAND.send_message_qt(Command::UpdateSchemas);

                        dialog.show();
                        dialog.set_text(&qtr("update_in_prog"));
                        unsafe { update_button.as_mut().unwrap().set_enabled(false); }

                        match CENTRAL_COMMAND.recv_message_qt_try() {
                            Response::Success => show_dialog(self.main_window as *mut Widget, "<h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>", true),
                            Response::Error(error) => show_dialog(self.main_window as *mut Widget, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }
                    }
                }
            }
        }
    }

    /// This function is used to open ANY supported PackedFiles in a DockWidget, docked in the Main Window.
    pub fn open_packedfile(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
        is_preview: bool
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        // Also, only open the selection when there is only one thing selected.
        if !UI_STATE.get_packfile_contents_read_only() {
            let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &selected_items[0] } else { return };
            if let TreePathType::File(path) = item_type {

                // Close all preview views except the file we're opening.
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = unsafe { self.tab_bar_packed_file.as_ref().unwrap().index_of(packed_file_view.get_mut_widget()) };
                    if open_path != path && packed_file_view.get_is_preview() && index != -1 {
                        unsafe { self.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }
                    }
                }

                // If the file we want to open is already open, or it's hidden, we show it/focus it, instead of opening it again.
                // If it was a preview, then we mark it as full. Index == -1 means it's not in a tab.
                if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(path) {
                    let index = unsafe { self.tab_bar_packed_file.as_ref().unwrap().index_of(tab_widget.get_mut_widget()) };

                    // If we're trying to open as preview something already open as full, we don't do anything.
                    if !(index != -1 && is_preview && !tab_widget.get_is_preview()) {
                        tab_widget.set_is_preview(is_preview);
                    }

                    if index == -1 {
                        let icon_type = IconType::File(path.to_vec());
                        let icon = icon_type.get_icon_from_path();
                        unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget.get_mut_widget(), icon, &QString::from_std_str(""))); }
                    }

                    let name = if tab_widget.get_is_preview() { format!("{} (Preview)", path.last().unwrap()) } else { path.last().unwrap().to_owned() };
                    let index = unsafe { self.tab_bar_packed_file.as_ref().unwrap().index_of(tab_widget.get_mut_widget()) };
                    unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_tab_text(index, &QString::from_std_str(&name)); }
                    unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget.get_mut_widget()); }
                    return;
                }

                let mut tab = PackedFileView::default();
                let tab_widget = tab.get_mut_widget();
                tab.set_is_preview(is_preview);
                let name = if tab.get_is_preview() { format!("{} (Preview)", path.last().unwrap()) } else { path.last().unwrap().to_owned() };
                let icon_type = IconType::File(path.to_vec());
                let icon = icon_type.get_icon_from_path();

                // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                let packed_file_type = PackedFileType::get_packed_file_type(&path);
                let path = Rc::new(RefCell::new(path.to_vec()));

                match packed_file_type {

                    // If the file is a Loc PackedFile...
                    PackedFileType::Loc => {
                        match PackedFileTableView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                            Ok((slots, packed_file_info)) => {
                                slot_holder.borrow_mut().push(slots);

                                // Add the file to the 'Currently open' list and make it visible.
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.insert(path.borrow().to_vec(), tab);
                                if let Some(packed_file_info) = packed_file_info {
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                }
                            },
                            Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::LocDecode(format!("{}", error)), false),
                        }
                    }

                    // If the file is a DB PackedFile...
                    PackedFileType::DB => {
                        match PackedFileTableView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                            Ok((slots, packed_file_info)) => {
                                slot_holder.borrow_mut().push(slots);

                                // Add the file to the 'Currently open' list and make it visible.
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.insert(path.borrow().to_vec(), tab);
                                if let Some(packed_file_info) = packed_file_info {
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                                }
                            },
                            Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::DBTableDecode(format!("{}", error)), false),
                        }
                    }

                    // If the file is a Text PackedFile...
                    PackedFileType::Text(_) => {
                        match PackedFileTextView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                            Ok((slots, packed_file_info)) => {
                                slot_holder.borrow_mut().push(slots);

                                // Add the file to the 'Currently open' list and make it visible.
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.insert(path.borrow().to_vec(), tab);
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                            },
                            Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::TextDecode(format!("{}", error)), false),
                        }
                    }

                    // If the file is a RigidModel PackedFile...
                    PackedFileType::RigidModel => {
                        match PackedFileRigidModelView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                            Ok((slots, packed_file_info)) => {
                                slot_holder.borrow_mut().push(slots);

                                // Add the file to the 'Currently open' list and make it visible.
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.insert(path.borrow().to_vec(), tab);
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                            },
                            Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::TextDecode(format!("{}", error)), false),
                        }
                    }

                    // If the file is a Image PackedFile...
                    PackedFileType::Image => {
                        match PackedFileImageView::new_view(&path, &mut tab) {
                            Ok((slots, packed_file_info)) => {
                                slot_holder.borrow_mut().push(slots);

                                // Add the file to the 'Currently open' list and make it visible.
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                                unsafe { self.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                                let mut open_list = UI_STATE.set_open_packedfiles();
                                open_list.insert(path.borrow().to_vec(), tab);
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]));
                            },
                            Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::ImageDecode(format!("{}", error)), false),
                        }
                    }

                    // For any other PackedFile, just restore the display tips.
                    _ => {
                        //purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                        //display_help_tips(&app_ui);
                    }
                }
            }
        }
    }

    /// This function is used to open the PackedFile Decoder.
    pub fn open_decoder(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {
            let mut selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
            let item_type = if selected_items.len() == 1 { &mut selected_items[0] } else { return };
            if let TreePathType::File(ref mut path) = item_type {
                let mut fake_path = path.to_vec();
                *fake_path.last_mut().unwrap() = format!("{}-rpfm-decoder", fake_path.last_mut().unwrap());

                // Close all preview views except the file we're opening.
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = unsafe { self.tab_bar_packed_file.as_ref().unwrap().index_of(packed_file_view.get_mut_widget()) };
                    if open_path != path && packed_file_view.get_is_preview() && index != -1 {
                        unsafe { self.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }
                    }
                }

                // Close all preview views except the file we're opening. The path used for the decoder is empty.
                let name = qtr("decoder_title");
                let tab_bar_packed_file = unsafe { self.tab_bar_packed_file.as_mut().unwrap() };
                for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                    let index = unsafe { tab_bar_packed_file.index_of(packed_file_view.get_mut_widget()) };
                    if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                        tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the decoder is already open, or it's hidden, we show it/focus it, instead of opening it again.
                if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(&fake_path) {
                    let index = unsafe { tab_bar_packed_file.index_of(tab_widget.get_mut_widget()) };

                    if index == -1 {
                        let icon_type = IconType::PackFile(true);
                        let icon = icon_type.get_icon_from_path();
                        unsafe { tab_bar_packed_file.add_tab((tab_widget.get_mut_widget(), icon, &name)); }
                    }

                    unsafe { tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget()); }
                    return;
                }

                // If it's not already open/hidden, we create it and add it as a new tab.
                let mut tab = PackedFileView::default();
                tab.set_is_preview(false);
                let icon_type = IconType::PackFile(true);
                let icon = icon_type.get_icon_from_path();

                let path = Rc::new(RefCell::new(path.to_vec()));
                match PackedFileDecoderView::new_view(&path, &mut tab, global_search_ui, pack_file_contents_ui) {
                    Ok(slots) => {
                        slot_holder.borrow_mut().push(slots);

                        // Add the decoder to the 'Currently open' list and make it visible.
                        unsafe { tab_bar_packed_file.add_tab((tab.get_mut_widget(), icon, &name)); }
                        unsafe { tab_bar_packed_file.set_current_widget(tab.get_mut_widget()); }
                        let mut open_list = UI_STATE.set_open_packedfiles();
                        open_list.insert(fake_path, tab);
                    },
                    Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::DecoderDecode(format!("{}", error)), false),
                }
            }
        }
    }

    /// This function is used to open the dependency manager.
    pub fn open_dependency_manager(
        &self,
        pack_file_contents_ui: &PackFileContentsUI,
        global_search_ui: &GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            // Close all preview views except the file we're opening. The path used for the manager is empty.
            let path = vec![];
            let name = qtr("table_dependency_manager_title");
            let tab_bar_packed_file = unsafe { self.tab_bar_packed_file.as_mut().unwrap() };
            for (open_path, packed_file_view) in UI_STATE.get_open_packedfiles().iter() {
                let index = unsafe { tab_bar_packed_file.index_of(packed_file_view.get_mut_widget()) };
                if !open_path.is_empty() && packed_file_view.get_is_preview() && index != -1 {
                    tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the manager is already open, or it's hidden, we show it/focus it, instead of opening it again.
            if let Some(ref mut tab_widget) = UI_STATE.set_open_packedfiles().get_mut(&path) {
                let index = unsafe { tab_bar_packed_file.index_of(tab_widget.get_mut_widget()) };

                if index == -1 {
                    let icon_type = IconType::PackFile(true);
                    let icon = icon_type.get_icon_from_path();
                    unsafe { tab_bar_packed_file.add_tab((tab_widget.get_mut_widget(), icon, &name)); }
                }

                unsafe { tab_bar_packed_file.set_current_widget(tab_widget.get_mut_widget()); }
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
                    unsafe { tab_bar_packed_file.add_tab((tab.get_mut_widget(), icon, &name)); }
                    unsafe { tab_bar_packed_file.set_current_widget(tab.get_mut_widget()); }
                    let mut open_list = UI_STATE.set_open_packedfiles();
                    open_list.insert(path.borrow().to_vec(), tab);
                },
                Err(error) => return show_dialog(self.main_window as *mut Widget, ErrorKind::DependencyManagerDecode(format!("{}", error)), false),
            }
        }
    }

    /// This function is the one that takes care of the creation of different PackedFiles.
    pub fn new_packed_file(&self, pack_file_contents_ui: &PackFileContentsUI, packed_file_type: &PackedFileType) {

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
                                return show_dialog(self.main_window as *mut Widget, ErrorKind::EmptyInput, false)
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
                            let selected_paths = <*mut TreeView as PackTree>::get_path_from_main_treeview_selection(pack_file_contents_ui);
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
                                if exists { return show_dialog(self.main_window as *mut Widget, ErrorKind::FileAlreadyInPackFile, false)}

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

                                    Response::Error(error) => show_dialog(self.main_window as *mut Widget, error, false),
                                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                                }
                            }
                        }
                    }
                }
                Err(error) => show_dialog(self.main_window as *mut Widget, error, false),
            }
        }
    }

    /// This function creates a new PackedFile based on the current path selection, being:
    /// - `db/xxxx` -> DB Table.
    /// - `text/xxxx` -> Loc Table.
    /// - `script/xxxx` -> Lua PackedFile.
    /// The name used for each packfile is a generic one.
    pub fn new_queek_packed_file(&self, pack_file_contents_ui: &PackFileContentsUI) {

        // Get the currently selected path and, depending on the selected path, generate one packfile or another.
        let selected_paths = <*mut TreeView as PackTree>::get_path_from_main_treeview_selection(pack_file_contents_ui);
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
                        Response::Error(error) => return show_dialog(self.main_window as *mut Widget, error, false),
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
                } else { return show_dialog(self.main_window as *mut Widget, ErrorKind::NoQueekPackedFileHere, false); };

                // Check if the PackedFile already exists, and report it if so.
                CENTRAL_COMMAND.send_message_qt(Command::PackedFileExists(new_path.to_vec()));
                let response = CENTRAL_COMMAND.recv_message_qt();
                let exists = if let Response::Bool(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
                if exists { return show_dialog(self.main_window as *mut Widget, ErrorKind::FileAlreadyInPackFile, false)}

                // Create the PackFile.
                CENTRAL_COMMAND.send_message_qt(Command::NewPackedFile(new_path.to_vec(), new_packed_file));
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::Success => pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(vec![TreePathType::File(new_path); 1])),
                    Response::Error(error) => show_dialog(self.main_window as *mut Widget, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }
    }

    /// This function creates the entire "New Folder" dialog.
    ///
    /// It returns the new name of the Folder, or None if the dialog is canceled or closed.
    pub fn new_folder_dialog(&self) -> Option<String> {
        let mut dialog = unsafe { Dialog::new_unsafe(self.main_window as *mut Widget) };
        dialog.set_window_title(&qtr("new_folder"));
        dialog.set_modal(true);

        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);

        let mut new_folder_line_edit = LineEdit::new(());
        new_folder_line_edit.set_text(&qtr("new_folder_default"));
        let new_folder_button = PushButton::new(&qtr("new_folder")).into_raw();

        unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_line_edit.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_button as *mut Widget, 0, 1, 1, 1)); }
        unsafe { new_folder_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

        if dialog.exec() == 1 { Some(new_folder_line_edit.text().to_std_string()) }
        else { None }
    }

    /// This function creates all the "New PackedFile" dialogs.
    ///
    /// It returns the type/name of the new file, or None if the dialog is canceled or closed.
    pub fn new_packed_file_dialog(&self, packed_file_type: &PackedFileType) -> Option<Result<NewPackedFile>> {

        // Create and configure the "New PackedFile" Dialog.
        let mut dialog = unsafe { Dialog::new_unsafe(self.main_window as *mut Widget) };
        match packed_file_type {
            PackedFileType::DB => dialog.set_window_title(&qtr("new_db_file")),
            PackedFileType::Loc => dialog.set_window_title(&qtr("new_loc_file")),
            PackedFileType::Text(_) => dialog.set_window_title(&qtr("new_txt_file")),
            _ => unimplemented!(),
        }
        dialog.set_modal(true);

        // Create the main Grid and his widgets.
        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);
        let mut name_line_edit = LineEdit::new(());
        let table_filter_line_edit = LineEdit::new(()).into_raw();
        let create_button = PushButton::new(&qtr("gen_loc_create"));
        let mut table_dropdown = ComboBox::new();
        let table_filter = SortFilterProxyModel::new().into_raw();
        let table_model = StandardItemModel::new(());

        name_line_edit.set_text(&qtr("new_file_default"));
        unsafe { table_dropdown.set_model(table_model.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { table_filter_line_edit.as_mut().unwrap().set_placeholder_text(&qtr("packedfile_filter")); }

        // Add all the widgets to the main grid, except those specific for a PackedFileType.
        unsafe { main_grid.as_mut().unwrap().add_widget((name_line_edit.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((create_button.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }

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
                        .for_each(|x| table_dropdown.add_item(&QString::from_std_str(&x)));
                    unsafe { table_filter.as_mut().unwrap().set_source_model(table_model.as_mut_ptr() as *mut AbstractItemModel); }
                    unsafe { table_dropdown.set_model(table_filter as *mut AbstractItemModel); }

                    unsafe { main_grid.as_mut().unwrap().add_widget((table_dropdown.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
                    unsafe { main_grid.as_mut().unwrap().add_widget((table_filter_line_edit as *mut Widget, 2, 0, 1, 1)); }
                }
                None => return Some(Err(ErrorKind::SchemaNotFound.into())),
            }
        }

        // What happens when we search in the filter.
        let slot_table_filter_change_text = SlotStringRef::new(move |_| {
            let pattern = unsafe { RegExp::new(&table_filter_line_edit.as_ref().unwrap().text()) };
            unsafe { table_filter.as_mut().unwrap().set_filter_reg_exp(&pattern); }
        });

        // What happens when we hit the "Create" button.
        create_button.signals().released().connect(&dialog.slots().accept());

        // What happens when we edit the search filter.
        unsafe { table_filter_line_edit.as_ref().unwrap().signals().text_changed().connect(&slot_table_filter_change_text); }

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
    fn new_packed_file_name_dialog(&self) -> Option<String> {

        // Create and configure the dialog.
        let mut dialog = unsafe { Dialog::new_unsafe(self.main_window as *mut Widget) };
        dialog.set_window_title(&qtr("new_packedfile_name"));
        dialog.set_modal(true);
        dialog.resize((400, 50));

        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);
        let mut name_line_edit = LineEdit::new(());
        let accept_button = PushButton::new(&qtr("gen_loc_accept"));

        name_line_edit.set_text(&qtr("trololol"));

        unsafe { main_grid.as_mut().unwrap().add_widget((name_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((accept_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }

        accept_button.signals().released().connect(&dialog.slots().accept());

        if dialog.exec() == 1 {
            let new_text = name_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(name_line_edit.text().to_std_string()) }
        } else { None }
    }

    /// This function creates the entire "Merge Tables" dialog. It returns the stuff set in it.
    pub fn merge_tables_dialog(&self) -> Option<(String, bool)> {

        let mut dialog = unsafe { Dialog::new_unsafe(self.main_window as *mut Widget) };
        dialog.set_window_title(&qtr("packedfile_merge_tables"));
        dialog.set_modal(true);

        // Create the main Grid.
        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);
        let mut name = LineEdit::new(());
        name.set_placeholder_text(&qtr("merge_tables_new_name"));

        let delete_source_tables = CheckBox::new(&qtr("merge_tables_delete_option"));

        let accept_button = PushButton::new(&qtr("gen_loc_accept"));
        unsafe { main_grid.as_mut().unwrap().add_widget((name.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((delete_source_tables.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((accept_button.as_mut_ptr() as *mut Widget, 2, 0, 1, 1)); }

        // What happens when we hit the "Search" button.
        accept_button.signals().released().connect(&dialog.slots().accept());

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
