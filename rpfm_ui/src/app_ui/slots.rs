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
Module with all the code related to the main `AppUISlot`.
!*/

use qt_widgets::QAction;
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QMessageBox;
use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;
use qt_gui::QDesktopServices;

use qt_core::QBox;
use qt_core::{SlotOfBool, SlotOfInt, SlotNoArgs};
use qt_core::QFlags;
use qt_core::QString;
use qt_core::QUrl;

use std::fs::{DirBuilder, copy, remove_file, remove_dir_all};
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_error::ErrorKind;

use rpfm_lib::common::*;
use rpfm_lib::config::get_config_path;
use rpfm_lib::DOCS_BASE_URL;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::games::*;
use rpfm_lib::packfile::{PFHFileType, CompressionState};
use rpfm_lib::PATREON_URL;
use rpfm_lib::SETTINGS;
use rpfm_lib::SCHEMA;
use rpfm_lib::SUPPORTED_GAMES;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{THREADS_COMMUNICATION_ERROR, Command, Response};
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, tr, tre};
use crate::mymod_ui::MyModUI;
use crate::pack_tree::{new_pack_file_tooltip, PackTree, TreeViewOperation};
use crate::packedfile_views::{View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::TreePathType;
use crate::settings_ui::SettingsUI;
use crate::ui::GameSelectedIcons;
use crate::{ui_state::op_mode::OperationalMode, UI_STATE};
use crate::utils::{log_to_status_bar, show_dialog};
use crate::VERSION;
use crate::views::table::utils::{get_reference_data, setup_item_delegates};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action created at the start of the program.
///
/// This means everything you can do with the stuff you have in the `AppUI` goes here.
pub struct AppUISlots {

    //-----------------------------------------------//
    // `PackFile` menu slots.
    //-----------------------------------------------//
    pub packfile_open_menu: QBox<SlotNoArgs>,
    pub packfile_new_packfile: QBox<SlotOfBool>,
    pub packfile_open_packfile: QBox<SlotOfBool>,
    pub packfile_save_packfile: QBox<SlotOfBool>,
    pub packfile_save_packfile_as: QBox<SlotOfBool>,
    pub packfile_load_all_ca_packfiles: QBox<SlotOfBool>,
    pub packfile_change_packfile_type: QBox<SlotOfBool>,
    pub packfile_index_includes_timestamp: QBox<SlotOfBool>,
    pub packfile_data_is_compressed: QBox<SlotOfBool>,
    pub packfile_preferences: QBox<SlotOfBool>,
    pub packfile_quit: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `MyMod` menu slots.
    //-----------------------------------------------//
    pub mymod_open_menu: QBox<SlotNoArgs>,
    pub mymod_open_mymod_folder: QBox<SlotOfBool>,
    pub mymod_new: QBox<SlotOfBool>,
    pub mymod_delete_selected: QBox<SlotOfBool>,
    pub mymod_install: QBox<SlotOfBool>,
    pub mymod_uninstall: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `View` menu slots.
    //-----------------------------------------------//
    pub view_toggle_packfile_contents: QBox<SlotOfBool>,
    pub view_toggle_global_search_panel: QBox<SlotOfBool>,
    pub view_toggle_diagnostics_panel: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Game Selected` menu slots.
    //-----------------------------------------------//
    pub game_selected_launch_game: QBox<SlotOfBool>,
    pub game_selected_open_game_data_folder: QBox<SlotOfBool>,
    pub game_selected_open_game_assembly_kit_folder: QBox<SlotOfBool>,
    pub game_selected_open_config_folder: QBox<SlotOfBool>,
    pub change_game_selected: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Special Stuff` menu slots.
    //-----------------------------------------------//
    pub special_stuff_generate_pak_file: QBox<SlotOfBool>,
    pub special_stuff_optimize_packfile: QBox<SlotOfBool>,
    pub special_stuff_patch_siege_ai: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Templates` menu slots.
    //-----------------------------------------------//
    pub templates_open_custom_templates_folder: QBox<SlotNoArgs>,
    pub templates_open_official_templates_folder: QBox<SlotNoArgs>,
    pub templates_save_packfile_to_template: QBox<SlotNoArgs>,

    //-----------------------------------------------//
    // `About` menu slots.
    //-----------------------------------------------//
    pub about_about_qt: QBox<SlotOfBool>,
    pub about_about_rpfm: QBox<SlotOfBool>,
    pub about_open_manual: QBox<SlotOfBool>,
    pub about_patreon_link: QBox<SlotOfBool>,
    pub about_check_updates: QBox<SlotOfBool>,
    pub about_check_schema_updates: QBox<SlotOfBool>,
    pub about_check_templates_updates: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Debug` menu slots.
    //-----------------------------------------------//
    pub debug_update_current_schema_from_asskit: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `PackedFileView` slots.
    //-----------------------------------------------//
    pub packed_file_hide: QBox<SlotOfInt>,
    pub packed_file_update: QBox<SlotOfInt>,
    pub packed_file_unpreview: QBox<SlotOfInt>,

    //-----------------------------------------------//
    // `Generic` slots.
    //-----------------------------------------------//
    pub pack_file_backup_autosave: QBox<SlotNoArgs>,

    pub tab_bar_packed_file_context_menu_show: QBox<SlotOfQPoint>,
    pub tab_bar_packed_file_close: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all_left: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all_right: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_prev: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_next: QBox<SlotNoArgs>,
}

pub struct AppUITempSlots {}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUISlots`.
impl AppUISlots {

	/// This function creates an entire `AppUISlots` struct. Used to create the logic of the starting UI.
	pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
    ) -> Self {

        //-----------------------------------------------//
        // `PackFile` menu logic.
        //-----------------------------------------------//

        // Slot to build the "Open from" submenus of the PackFile menu.
        let packfile_open_menu = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move || {
                AppUI::build_open_from_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
            }
        ));

        // What happens when we trigger the "New PackFile" action.
        let packfile_new_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui => move |_| {

                // Check first if there has been changes in the PackFile.
                if AppUI::are_you_sure(&app_ui, false) {

                    // Tell the Background Thread to create a new PackFile.
                    CENTRAL_COMMAND.send_message_qt(Command::NewPackFile);

                    // Reset the autosave timer.
                    let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
                    if timer > 0 {
                        app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                        app_ui.timer_backup_autosave.start_0a();
                    }

                    // Disable the main window, so the user can't interrupt the process or iterfere with it.
                    app_ui.main_window.set_enabled(false);

                    // Close any open PackedFile and clear the global search pannel.
                    let _ = AppUI::purge_them_all(&app_ui,  &pack_file_contents_ui, false);
                    GlobalSearchUI::clear(&global_search_ui);
                    diagnostics_ui.get_ref_diagnostics_table_model().clear();
                    //if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                    // Show the "Tips".
                    //display_help_tips(&app_ui);

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
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(None, None));

                    // Re-enable the Main Window.
                    app_ui.main_window.set_enabled(true);

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    AppUI::enable_packfile_actions(&app_ui, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    UI_STATE.set_operational_mode(&app_ui, None);
                    UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        let packfile_open_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // Check first if there has been changes in the PackFile.
                if AppUI::are_you_sure(&app_ui, false) {

                    // Create the FileDialog to get the PackFile to open and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &app_ui.main_window,
                        &qtr("open_packfiles"),
                    );
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                    file_dialog.set_file_mode(FileMode::ExistingFiles);

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Now the fun thing. We have to get all the selected files, and then open them one by one.
                        // For that we use the same logic as for the "Load All CA PackFiles" feature.
                        let mut paths = vec![];
                        for index in 0..file_dialog.selected_files().count_0a() {
                            paths.push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                        }

                        // Try to open it, and report it case of error.
                        if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &paths, "") {
                            return show_dialog(&app_ui.main_window, error, false);
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let packfile_save_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, false) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));

        // What happens when we trigger the "Save PackFile As" action.
        let packfile_save_packfile_as = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, true) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));

        // What happens when we trigger the "Load All CA PackFiles" action.
        let packfile_load_all_ca_packfiles = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {


            // Check first if there has been changes in the PackFile. If we accept, just take all the PackFiles in the data folder
            // and open them all together, skipping mods.
            if AppUI::are_you_sure(&app_ui, false) {

                // Reset the autosave timer.
                let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
                if timer > 0 {
                    app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                    app_ui.timer_backup_autosave.start_0a();
                }

                // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
                app_ui.main_window.set_enabled(false);

                // Destroy whatever it's in the PackedFile's views and clear the global search UI.
                GlobalSearchUI::clear(&global_search_ui);
                let _ = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, false);

                CENTRAL_COMMAND.send_message_qt(Command::LoadAllCAPackFiles);
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {

                    // If it's success....
                    Response::PackFileInfo(ui_data) => {

                        // Set this PackFile always to type `Other`.
                        app_ui.change_packfile_type_other.set_checked(true);

                        // Disable all of these.
                        app_ui.change_packfile_type_data_is_encrypted.set_checked(false);
                        app_ui.change_packfile_type_index_includes_timestamp.set_checked(false);
                        app_ui.change_packfile_type_index_is_encrypted.set_checked(false);
                        app_ui.change_packfile_type_header_is_extended.set_checked(false);

                        // Set the compression level correctly, because otherwise we may fuckup some files.
                        let compression_state = match ui_data.compression_state {
                            CompressionState::Enabled => true,
                            CompressionState::Partial | CompressionState::Disabled => false,
                        };
                        app_ui.change_packfile_type_data_is_compressed.set_checked(compression_state);

                        // Update the TreeView.
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(None, None));

                        let game_selected = GAME_SELECTED.read().unwrap().to_owned();
                        match &*game_selected {
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
                            KEY_ARENA => app_ui.game_selected_arena.trigger(),
                            _ => unreachable!(),
                        }

                        UI_STATE.set_operational_mode(&app_ui, None);
                        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);
                    }

                    // If we got an error...
                    Response::Error(error) => {
                        show_dialog(&app_ui.main_window, error, false);
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                // Always reenable the Main Window.
                app_ui.main_window.set_enabled(true);
            }
        }));

        // What happens when we trigger the "Change PackFile Type" action.
        let packfile_change_packfile_type = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {

                // Get the currently selected PackFile's Type.
                let packfile_type = match &*(app_ui.change_packfile_type_group
                    .checked_action().text().to_std_string()) {
                    "&Boot" => PFHFileType::Boot,
                    "&Release" => PFHFileType::Release,
                    "&Patch" => PFHFileType::Patch,
                    "&Mod" => PFHFileType::Mod,
                    "Mo&vie" => PFHFileType::Movie,
                    _ => PFHFileType::Other(99),
                };

                // Send the type to the Background Thread, and update the UI.
                CENTRAL_COMMAND.send_message_qt(Command::SetPackFileType(packfile_type));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we change the value of "Include Last Modified Date" action.
        let packfile_index_includes_timestamp = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui =>  move |_| {
                let state = app_ui.change_packfile_type_index_includes_timestamp.is_checked();
                CENTRAL_COMMAND.send_message_qt(Command::ChangeIndexIncludesTimestamp(state));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we enable/disable compression on the current PackFile.
        let packfile_data_is_compressed = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui =>  move |_| {
                let state = app_ui.change_packfile_type_data_is_compressed.is_checked();
                CENTRAL_COMMAND.send_message_qt(Command::ChangeDataIsCompressed(state));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we trigger the "Preferences" action.
        let packfile_preferences = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // We store a copy of the old settings (for checking changes) and trigger the new settings dialog.
                let old_settings = SETTINGS.read().unwrap().clone();
                if let Some(settings) = SettingsUI::new(&app_ui) {

                    // If we returned new settings, save them and wait for confirmation.
                    CENTRAL_COMMAND.send_message_qt(Command::SetSettings(settings.clone()));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    match response {

                        // If it worked, do some checks to ensure the UI keeps his consistency.
                        Response::Success => {

                            // If we changed the "MyMod's Folder" path, disable the MyMod mode and set it so the MyMod menu will be re-built
                            // next time we open the MyMod menu.
                            if settings.paths["mymods_base_path"] != old_settings.paths["mymods_base_path"] {
                                UI_STATE.set_operational_mode(&app_ui, None);
                                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
                            }

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // re-select the current `GameSelected` to force it to reload the game's files.
                            let has_game_selected_path_changed = settings.paths.iter()
                                .filter(|x| x.0 != "mymods_base_path" && &old_settings.paths[x.0] != x.1)
                                .any(|x| x.0 == &*GAME_SELECTED.read().unwrap());

                            if has_game_selected_path_changed {
                                QAction::trigger(&app_ui.game_selected_group.checked_action());
                            }
                        }

                        // If we got an error, report it.
                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),

                        // In ANY other situation, it's a message problem.
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                    }
                }
            }
        ));

        // What happens when we trigger the "Quit" action.
        let packfile_quit = SlotOfBool::new(&app_ui.main_window, clone!(
            mut app_ui => move |_| {
                if AppUI::are_you_sure(&app_ui, false) {
                    app_ui.main_window.close();
                }
            }
        ));

        //-----------------------------------------------//
        // `MyMod` menu logic.
        //-----------------------------------------------//

        // Slot to build the "Open from" submenus of the MyMod menu.
        let mymod_open_menu = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move || {
                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
            }
        ));

        // What happens when we trigger the "Open MyMod Folder" action.
        let mymod_open_mymod_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Some(ref path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {
                if open::that(&path).is_err() {
                    show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                };
            }
            else { show_dialog(&app_ui.main_window, ErrorKind::MyModPathNotConfigured, false); }
        }));

        // This slot is used for the "New MyMod" action.
        let mymod_new = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // Trigger the `New MyMod` Dialog, and get the result.
                if let Some((mod_name, mod_game)) = MyModUI::new(&app_ui) {
                    let full_mod_name = format!("{}.pack", mod_name);

                    // Change the Game Selected to match the one we chose for the new "MyMod".
                    // NOTE: Arena should not be on this list.
                    match &*mod_game {
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

                    // Disable the main window.
                    app_ui.main_window.set_enabled(false);

                    // Get his new path from the base "MyMod" path + `mod_game`.
                    let mut mymod_path = SETTINGS.read().unwrap().paths["mymods_base_path"].clone().unwrap();
                    mymod_path.push(&mod_game);

                    // Just in case the folder doesn't exist, we try to create it.
                    if DirBuilder::new().recursive(true).create(&mymod_path).is_err() {
                        app_ui.main_window.set_enabled(true);
                        return show_dialog(&app_ui.main_window, ErrorKind::IOCreateAssetFolder, false);
                    }

                    // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                    let mut mymod_path_private = mymod_path.to_path_buf();
                    mymod_path_private.push(&mod_name);
                    if DirBuilder::new().recursive(true).create(&mymod_path_private).is_err() {
                        app_ui.main_window.set_enabled(true);
                        return show_dialog(&app_ui.main_window, ErrorKind::IOCreateNestedAssetFolder, false);
                    };

                    // Complete the mymod PackFile path and create it.
                    mymod_path.push(&full_mod_name);

                    // Destroy whatever it's in the PackedFile's views and clear the global search UI.
                    let _ = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, false);
                    GlobalSearchUI::clear(&global_search_ui);

                    // Reset the autosave timer.
                    let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
                    if timer > 0 {
                        app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                        app_ui.timer_backup_autosave.start_0a();
                    }

                    CENTRAL_COMMAND.send_message_qt(Command::NewPackFile);
                    CENTRAL_COMMAND.send_message_qt(Command::SavePackFileAs(mymod_path.to_path_buf()));
                    let response = CENTRAL_COMMAND.recv_message_qt_try();
                    match response {
                        Response::PackFileInfo(pack_file_info) => {
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Build(None, None));
                            let packfile_item = pack_file_contents_ui.packfile_contents_tree_model.item_1a(0);
                            packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                            packfile_item.set_text(&QString::from_std_str(&full_mod_name));

                            // Set the UI to the state it should be in.
                            app_ui.change_packfile_type_mod.set_checked(true);
                            app_ui.change_packfile_type_data_is_encrypted.set_checked(false);
                            app_ui.change_packfile_type_index_includes_timestamp.set_checked(false);
                            app_ui.change_packfile_type_index_is_encrypted.set_checked(false);
                            app_ui.change_packfile_type_header_is_extended.set_checked(false);
                            app_ui.change_packfile_type_data_is_compressed.set_checked(false);

                            AppUI::enable_packfile_actions(&app_ui, true);

                            UI_STATE.set_operational_mode(&app_ui, Some(&mymod_path));
                            UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

                            // Close the Global Search stuff and reset the filter's history.
                            //if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                            // Show the "Tips".
                            //display_help_tips(&app_ui);
                            AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
                            app_ui.main_window.set_enabled(true);
                        }

                        Response::Error(error) => {
                            app_ui.main_window.set_enabled(true);
                            show_dialog(&app_ui.main_window, error, false);
                        }

                        // In ANY other situation, it's a message problem.
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
            }
        ));

        // This slot is used for the "Delete Selected MyMod" action.
        let mymod_delete_selected = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // Ask before doing it, as this will permanently delete the mod from the Disk.
                if AppUI::are_you_sure(&app_ui, true) {

                    // We want to keep our "MyMod" name for the success message, so we store it here.
                    let old_mod_name: String;

                    // Depending on our current "Mode", we choose what to do.
                    let mod_deleted = match UI_STATE.get_operational_mode() {

                        // If we have a "MyMod" selected, and everything we need it's configured,
                        // copy the PackFile to the data folder of the selected game.
                        OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                            old_mod_name = mod_name.to_owned();
                            let mymods_base_path = &SETTINGS.read().unwrap().paths["mymods_base_path"];
                            if let Some(ref mymods_base_path) = mymods_base_path {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                if !mymod_path.is_file() {
                                    return show_dialog(&app_ui.main_window, ErrorKind::MyModPackFileDoesntExist, false);
                                }

                                // Try to delete his PackFile. If it fails, return error.
                                if remove_file(&mymod_path).is_err() {
                                    return show_dialog(&app_ui.main_window, ErrorKind::IOGenericDelete(vec![mymod_path; 1]), false);
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.to_path_buf();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(&mymod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(&app_ui.main_window, ErrorKind::MyModPackFileDeletedFolderNotFound, false);
                                }

                                // If the assets folder exists, we try to delete it. Again, this is optional, so it should not stop the deleting process.
                                else if remove_dir_all(&mymod_assets_path).is_err() {
                                    show_dialog(&app_ui.main_window, ErrorKind::IOGenericDelete(vec![mymod_assets_path; 1]), false);
                                }

                                // Update the MyMod list and return true, as we have effectively deleted the MyMod.
                                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
                                true
                            }
                            else { return show_dialog(&app_ui.main_window, ErrorKind::MyModPathNotConfigured, false); }
                        }

                        // If we have no "MyMod" selected, return an error.
                        OperationalMode::Normal => return show_dialog(&app_ui.main_window, ErrorKind::MyModDeleteWithoutMyModSelected, false),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {
                        UI_STATE.set_operational_mode(&app_ui, None);
                        CENTRAL_COMMAND.send_message_qt(Command::ResetPackFile);
                        AppUI::enable_packfile_actions(&app_ui, false);
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Clear);
                        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

                        show_dialog(&app_ui.main_window, tre("mymod_delete_success", &[&old_mod_name]), true);
                    }
                }
            }
        ));

        // This slot is used for the "Install MyMod" action.
        let mymod_install = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected, and everything we need it's configured,
                    // copy the PackFile to the data folder of the selected game.
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                        let mymods_base_path = &SETTINGS.read().unwrap().paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {
                            if let Some(mut mymod_install_path) = get_mymod_install_path() {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                if !mymod_path.is_file() {
                                    return show_dialog(&app_ui.main_window, ErrorKind::MyModPackFileDoesntExist, false);
                                }

                                if !mymod_install_path.is_dir() {
                                    return show_dialog(&app_ui.main_window, ErrorKind::MyModInstallFolderDoesntExists, false);
                                }

                                // Get the destination path for the PackFile with the PackFile name included.
                                // And copy the PackFile to his destination. If the copy fails, return an error.
                                mymod_install_path.push(&mod_name);
                                if copy(mymod_path, &mymod_install_path).is_err() {
                                    return show_dialog(&app_ui.main_window, ErrorKind::IOGenericCopy(mymod_install_path), false);
                                }
                            }
                            else { show_dialog(&app_ui.main_window, ErrorKind::GamePathNotConfigured, false) }
                        }
                        else { show_dialog(&app_ui.main_window, ErrorKind::MyModPathNotConfigured, false); }
                    }

                    // If we have no "MyMod" selected, return an error.
                    OperationalMode::Normal => show_dialog(&app_ui.main_window, ErrorKind::MyModDeleteWithoutMyModSelected, false),
                }

            }
        ));

        // This slot is used for the "Uninstall MyMod" action.
        let mymod_uninstall = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected, and everything we need it's configured,
                    // try to delete the PackFile (if exists) from the data folder of the selected game.
                    OperationalMode::MyMod(_, ref mod_name) => {
                        if let Some(mut mymod_install_path) = get_mymod_install_path() {
                            mymod_install_path.push(&mod_name);

                            if !mymod_install_path.is_file() {
                                show_dialog(&app_ui.main_window, ErrorKind::MyModNotInstalled, false)
                            }

                            else if remove_file(&mymod_install_path).is_err() {
                                return show_dialog(&app_ui.main_window, ErrorKind::IOGenericDelete(vec![mymod_install_path; 1]), false);
                            }
                        }
                        else { show_dialog(&app_ui.main_window, ErrorKind::GamePathNotConfigured, false); }
                    }

                   // If we have no "MyMod" selected, return an error.
                    OperationalMode::Normal => show_dialog(&app_ui.main_window, ErrorKind::MyModDeleteWithoutMyModSelected, false),
                }
            }
        ));

        //-----------------------------------------------//
        // `View` menu logic.
        //-----------------------------------------------//
        let view_toggle_packfile_contents = SlotOfBool::new(&app_ui.main_window, clone!(
            pack_file_contents_ui => move |state| {
            if !state { pack_file_contents_ui.packfile_contents_dock_widget.hide(); }
            else { pack_file_contents_ui.packfile_contents_dock_widget.show();}
        }));

        let view_toggle_global_search_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            global_search_ui => move |state| {
            if !state { global_search_ui.global_search_dock_widget.hide(); }
            else {
                global_search_ui.global_search_dock_widget.show();
                global_search_ui.global_search_search_line_edit.set_focus_0a()
            }
        }));

        let view_toggle_diagnostics_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            diagnostics_ui => move |state| {
                if !state { diagnostics_ui.get_ref_diagnostics_dock_widget().hide(); }
                else { diagnostics_ui.get_ref_diagnostics_dock_widget().show();}
        }));

        //-----------------------------------------------//
        // `Game Selected` menu logic.
        //-----------------------------------------------//

        // What happens when we trigger the "Launch Game" action.
        let game_selected_launch_game = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            match get_game_selected_install_type().unwrap() {
                InstallType::Steam(steam_id) => {
                    if open::that(format!("steam://rungameid/{}", steam_id)).is_err() {
                        show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                    };
                }
                _ => show_dialog(&app_ui.main_window, ErrorKind::LaunchNotSupportedForThisGame, false),
            }
        }));

        // What happens when we trigger the "Open Game's Data Folder" action.
        let game_selected_open_game_data_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Some(path) = get_game_selected_data_path() {
                if open::that(&path).is_err() {
                    show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                };
            }
            else { show_dialog(&app_ui.main_window, ErrorKind::GamePathNotConfigured, false); }
        }));

        // What happens when we trigger the "Open Game's Assembly Kit Folder" action.
        let game_selected_open_game_assembly_kit_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Some(path) = get_game_selected_assembly_kit_path() {
                if open::that(&path).is_err() {
                    show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                };
            }
            else { show_dialog(&app_ui.main_window, ErrorKind::GamePathNotConfigured, false); }
        }));

        // What happens when we trigger the "Open Config Folder" action.
        let game_selected_open_config_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Ok(path) = get_config_path() {
                if open::that(&path).is_err() {
                    show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                };
            }
            else { show_dialog(&app_ui.main_window, ErrorKind::ConfigFolderCouldNotBeOpened, false); }
        }));

        // What happens when we trigger the "Change Game Selected" action.
        //
        // NOTE: NEVER EVER AGAIN SHALL YOU TRIGGER HERE A REBUILD OF THE GAME-SPECIFIC SLOTS!!!!!!!!!!
        let change_game_selected = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {

                // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
                let mut new_game_selected = app_ui.game_selected_group.checked_action().text().to_std_string();
                if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
                let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();
                if new_game_selected != *GAME_SELECTED.read().unwrap() || SCHEMA.read().unwrap().is_none() {

                    // Disable the Main Window (so we can't do other stuff).
                    app_ui.main_window.set_enabled(false);

                    // Send the command to the background thread to set the new `Game Selected`, and tell RPFM to rebuild the mymod menu when it can.
                    // We have to wait because we need the GameSelected update before updating the menus.
                    CENTRAL_COMMAND.send_message_qt(Command::SetGameSelected(new_game_selected));
                    let response = CENTRAL_COMMAND.recv_message_qt_try();
                    match response {
                        Response::Success => {}
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    // If we have a packfile open, set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                    if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() > 0 {
                        UI_STATE.set_operational_mode(&app_ui, None);
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::PackFile]));
                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                    }

                    // Re-enable the Main Window.
                    app_ui.main_window.set_enabled(true);

                    // Change the GameSelected Icon. Disabled until we find better icons.
                    GameSelectedIcons::set_game_selected_icon(&app_ui);
                }

                // Disable the `PackFile Management` actions and, if we have a `PackFile` open, re-enable them.
                AppUI::enable_packfile_actions(&app_ui, false);
                if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() != 0 {
                    AppUI::enable_packfile_actions(&app_ui, true);
                }

                // Always trigger the missing definitions code and the rebuilt for dependencies.
                CENTRAL_COMMAND.send_message_qt(Command::RebuildDependencies);
                CENTRAL_COMMAND.send_message_qt(Command::GetMissingDefinitions);
            }
        ));

        //-----------------------------------------------------//
        // `Special Stuff` menu logic.
        //-----------------------------------------------------//

        // What happens when we trigger the "Generate Pak File" action.
        let special_stuff_generate_pak_file = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {

                // For Rome 2+, we need the game path set. For other games, we have to ask for a path.
                let version = SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().raw_db_version;
                let path = match version {

                    // Post-Shogun 2 games.
                    2 => {
                        if let Some(ref path) = SETTINGS.read().unwrap().paths[&**GAME_SELECTED.read().unwrap()] {
                            let mut path = path.to_path_buf();
                            path.push("assembly_kit");
                            path.push("raw_data");
                            path.push("db");
                            path
                        }
                        else {
                            return show_dialog(&app_ui.main_window, ErrorKind::GamePathNotConfigured, false);
                        }
                    }

                    // Shogun 2.
                    1 => {

                        // Create the FileDialog to get the path of the Assembly Kit.
                        let file_dialog = QFileDialog::from_q_widget_q_string(
                            &app_ui.main_window,
                            &qtr("special_stuff_select_ak_folder"),
                        );

                        // Set it to only search Folders.
                        file_dialog.set_file_mode(FileMode::Directory);
                        file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        let mut path = if file_dialog.exec() == 1 {
                            PathBuf::from(file_dialog.selected_files().at(0).to_std_string())
                        } else {
                            return show_dialog(&app_ui.main_window, ErrorKind::AssemblyKitNotFound, false);
                        };

                        path.push("raw_data");
                        path.push("db");
                        path
                    }

                    // Empire and Napoleon. This is not really supported yet. It's leave here as a placeholder.
                    _ => return show_dialog(&app_ui.main_window, tr("game_selected_unsupported_operation"), false),
                };

                if path.is_dir() {

                    // If there is no problem, ere we go.
                    app_ui.main_window.set_enabled(false);

                    CENTRAL_COMMAND.send_message_qt(Command::GeneratePakFile(path, version));
                    let response = CENTRAL_COMMAND.recv_message_qt_try();
                    match response {
                        Response::Success => show_dialog(&app_ui.main_window, tr("generate_pak_success"), true),
                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    app_ui.main_window.set_enabled(true);
                }
                else {
                    show_dialog(&app_ui.main_window, ErrorKind::AssemblyKitNotFound, false);
                }
            }
        ));

        // What happens when we trigger the "Optimize PackFile" action.
        let special_stuff_optimize_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // If there is no problem, ere we go.
                app_ui.main_window.set_enabled(false);

                if let Err(error) = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, true) {
                    return show_dialog(&app_ui.main_window, error, false);
                }

                GlobalSearchUI::clear(&global_search_ui);

                CENTRAL_COMMAND.send_message_qt(Command::OptimizePackFile);
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {
                    Response::VecVecString(response) => {
                        let response = response.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();

                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(response));
                        show_dialog(&app_ui.main_window, tr("optimize_packfile_success"), true);
                    }
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                // Re-enable the Main Window.
                app_ui.main_window.set_enabled(true);
            }
        ));

        // What happens when we trigger the "Patch Siege AI" action.
        let special_stuff_patch_siege_ai = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                // Ask the background loop to patch the PackFile, and wait for a response.
                app_ui.main_window.set_enabled(false);

                if let Err(error) = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, true) {
                    return show_dialog(&app_ui.main_window, error, false);
                }

                GlobalSearchUI::clear(&global_search_ui);

                CENTRAL_COMMAND.send_message_qt(Command::PatchSiegeAI);
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {
                    Response::StringVecVecString(response) => {
                        let message = response.0;
                        let paths = response.1.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(paths));
                        show_dialog(&app_ui.main_window, &message, true);
                    }

                    // If the PackFile is empty or is not patchable, report it. Otherwise, praise the nine divines.
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }

                // Re-enable the Main Window.
                app_ui.main_window.set_enabled(true);
            }
        ));

        //-----------------------------------------------//
        // `Templates` menu logic.
        //-----------------------------------------------//

        let templates_open_custom_templates_folder = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                match get_custom_template_definitions_path() {
                    Ok(path) => if open::that(&path).is_err() {
                        show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                    },
                    Err(error) => show_dialog(&app_ui.main_window, error, false),
                }
            }
        ));

        let templates_open_official_templates_folder = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                match get_template_definitions_path() {
                    Ok(path) => if open::that(&path).is_err() {
                        show_dialog(&app_ui.main_window, ErrorKind::IOFolderCannotBeOpened, false);
                    },
                    Err(error) => show_dialog(&app_ui.main_window, error, false),
                }
            }
        ));

        let templates_save_packfile_to_template = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
                if let Err(error) = AppUI::save_to_template(&app_ui, &pack_file_contents_ui) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));
		//-----------------------------------------------//
        // `About` menu logic.
        //-----------------------------------------------//

        // What happens when we trigger the "About Qt" action.
        let about_about_qt = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                QMessageBox::about_qt_1a(&app_ui.main_window);
            }
        ));

        // What happens when we trigger the "About RPFM" action.
        let about_about_rpfm = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                QMessageBox::about(
                    &app_ui.main_window,
                    &qtr("about_about_rpfm"),

                    // NOTE: This one is hardcoded, because I don't want people attributing themselfs the program in the translations.
                    &QString::from_std_str(format!(
                        "<table>
                            <tr>
                                <td><h2><b>Rusted PackFile Manager</b></h2></td>
                                <td>{}</td>
                            </tr>
                        </table>

                        <p><b>Rusted PackFile Manager</b> (a.k.a. RPFM) is a modding tool for modern Total War Games, made by modders, for modders.</p>
                        <p>This program is <b>open-source</b>, under MIT License. You can always get the last version (or collaborate) here:</p>
                        <a href=\"https://github.com/Frodo45127/rpfm\">https://github.com/Frodo45127/rpfm</a>
                        <p>This program is also <b>free</b> (if you paid for this, sorry, but you got scammed), but if you want to help with money, here is <b>RPFM's Patreon</b>:</p>
                        <a href=\"https://www.patreon.com/RPFM\">https://www.patreon.com/RPFM</a>

                        <h3>Credits</h3>
                        <ul style=\"list-style-type: disc\">
                            <li>Created and Programmed by: <b>Frodo45127</b>.</li>
                            <li>Icon until 1.6.2 by: <b>Maruka</b>.</li>
                            <li>Icons since 2.0.0 by: <b>Jake Armitage</b>.</li>
                            <li>RigidModel research by: <b>Mr.Jox</b>, <b>Der Spaten</b>, <b>Maruka</b> and <b>Frodo45127</b>.</li>
                            <li>LUA functions by: <b>Aexrael Dex</b>.</li>
                            <li>LUA Types for Kailua: <b>DrunkFlamingo</b>.</li>
                            <li>TW: Arena research and coding: <b>Trolldemorted</b>.</li>
                            <li>Ca_vp8 research: <b>John Sirett</b>.</li>
                            <li>AnimPack research: <b>Marthenil</b> and <b>Frodo45127</b>.</li>
                            <li>TreeView Icons made by <a href=\"https://www.flaticon.com/authors/smashicons\" title=\"Smashicons\">Smashicons</a> from <a href=\"https://www.flaticon.com/\" title=\"Flaticon\">www.flaticon.com</a>. Licensed under <a href=\"http://creativecommons.org/licenses/by/3.0/\" title=\"Creative Commons BY 3.0\" target=\"_blank\">CC 3.0 BY</a>
                        </ul>

                        <h3>Special thanks</h3>
                        <ul style=\"list-style-type: disc\">
                            <li><b>PFM team</b>, for providing the community with awesome modding tools.</li>
                            <li><b>CA</b>, for being a mod-friendly company.</li>
                            <li><b>CnC discord guys</b>, for asking for features, helping with testing from time to time, etc...</li>
                        </ul>
                        ", &VERSION))
                    );
            }
        ));

        // What happens when we trigger the "Open Manual" action.
        let about_open_manual = SlotOfBool::new(&app_ui.main_window, |_| { QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(DOCS_BASE_URL))); });

        // What happens when we trigger the "Support me on Patreon" action.
        let about_patreon_link = SlotOfBool::new(&app_ui.main_window, |_| { QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(PATREON_URL))); });

        // What happens when we trigger the "Check Update" action.
        let about_check_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                AppUI::check_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Check Schema Update" action.
        let about_check_schema_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                AppUI::check_schema_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Update Templates" action.
        let about_check_templates_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                AppUI::check_template_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Update from AssKit" action.
        let debug_update_current_schema_from_asskit = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {

                // For Rome 2+, we need the game path set. For other games, we have to ask for a path.
                let version = SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().raw_db_version;
                let path = match version {
                    1| 0 => {

                        // Create the FileDialog to get the path of the Assembly Kit.
                        let file_dialog = QFileDialog::from_q_widget_q_string(
                            &app_ui.main_window,
                            &qtr("special_stuff_select_raw_db_folder"),
                        );

                        // Set it to only search Folders.
                        file_dialog.set_file_mode(FileMode::Directory);
                        file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 { Some(PathBuf::from(file_dialog.selected_files().at(0).to_std_string()))
                        } else { None }
                    }
                    _ => None,
                };

                // If there is no problem, ere we go.
                app_ui.main_window.set_enabled(false);

                CENTRAL_COMMAND.send_message_qt(Command::UpdateCurrentSchemaFromAssKit(path));
                let response = CENTRAL_COMMAND.recv_message_qt_try();
                match response {
                    Response::Success => show_dialog(&app_ui.main_window, tr("update_current_schema_from_asskit_success"), true),
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                app_ui.main_window.set_enabled(true);
            }
        ));

        //-----------------------------------------------//
        // `PackedFileView` logic.
        //-----------------------------------------------//
        let packed_file_hide = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |index| {
                AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &[index]);
            }
        ));

        // TODO: This lags the ui on switching tabs. Move to the backend + timer.
        let packed_file_update = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui => move |index| {
                if index == -1 { return; }
                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let widget = packed_file_view.get_mut_widget();
                    if app_ui.tab_bar_packed_file.index_of(widget) == index {
                        if let ViewType::Internal(view) = packed_file_view.get_view() {

                            // For tables, we have to update the dependency data, reset the dropdown's data, and recheck the entire table for errors.
                            if let View::Table(table) = view {
                                let table = table.get_ref_table();
                                let table_name = if let Some(name) = table.get_ref_table_name() { name.to_owned() } else { "".to_owned() };
                                if let Ok(data) = get_reference_data(&table_name, &table.get_ref_table_definition()) {
                                    table.set_dependency_data(&data);

                                    setup_item_delegates(
                                        &table.get_mut_ptr_table_view_primary(),
                                        &table.get_mut_ptr_table_view_frozen(),
                                        &table.get_ref_table_definition(),
                                        &data
                                    );
                                }
                            }
                        }
                        break;
                    }
                }

                // We also have to check for colliding packedfile names, so we can use their full path instead.
                AppUI::update_views_names(&app_ui);

                // Update the background icon.
                GameSelectedIcons::set_game_selected_icon(&app_ui);
            }
        ));

        let packed_file_unpreview = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui => move |index| {
                if index == -1 { return; }

                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let widget = packed_file_view.get_mut_widget();
                    if app_ui.tab_bar_packed_file.index_of(widget) == index {
                        if packed_file_view.get_is_preview() {
                            packed_file_view.set_is_preview(false);

                            let name = packed_file_view.get_ref_path().last().unwrap().to_owned();
                            app_ui.tab_bar_packed_file.set_tab_text(index, &QString::from_std_str(&name));
                        }
                        break;
                    }
                }
            }
        ));

        // Autosave slot.
        let pack_file_backup_autosave = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                CENTRAL_COMMAND.send_message_qt(Command::GetPackFileSettings);
                let response = CENTRAL_COMMAND.recv_message_qt();
                let settings = match response {
                    Response::PackFileSettings(settings) => settings,
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                };

                if let Some(disable_autosaves) = settings.settings_bool.get("disable_autosaves") {
                    if !disable_autosaves {
                        CENTRAL_COMMAND.send_message_qt(Command::TriggerBackupAutosave);
                        log_to_status_bar(&tr("autosaving"));
                        app_ui.main_window.set_enabled(false);
                        let response = CENTRAL_COMMAND.recv_message_notification_to_qt_try();
                        match response {
                            Response::Success => log_to_status_bar(&tr("autosaved")),
                            Response::Error(error) => if error.kind() == &ErrorKind::PackFileIsNonEditable {
                                log_to_status_bar(&tr("error_autosave_non_editable"))
                            } else { log_to_status_bar(&error.to_terminal()) },
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }
                        app_ui.main_window.set_enabled(true);

                        // Reset the timer.
                        let timer = SETTINGS.read().unwrap().settings_string["autosave_interval"].parse::<i32>().unwrap_or(10);
                        if timer > 0 {
                            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                            app_ui.timer_backup_autosave.start_0a();
                        }
                    }
                }
            }
        ));

        // When we want to show the context menu.
        let tab_bar_packed_file_context_menu_show = SlotOfQPoint::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            app_ui.tab_bar_packed_file_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let tab_bar_packed_file_close = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &[index]);
        }));

        let tab_bar_packed_file_close_all = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check != index && index_to_check != -1 {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();

            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_close_all_left = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check < index {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_close_all_right = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check > index {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_prev = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                let index = app_ui.tab_bar_packed_file.current_index();
                if index != -1 {
                    app_ui.tab_bar_packed_file.set_current_index(index - 1);
                }
            }
        ));

        let tab_bar_packed_file_next = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                let index = app_ui.tab_bar_packed_file.current_index();
                if index != -1 {
                    app_ui.tab_bar_packed_file.set_current_index(index + 1);
                }
            }
        ));

        // And here... we return all the slots.
		Self {

            //-----------------------------------------------//
            // `PackFile` menu slots.
            //-----------------------------------------------//
            packfile_open_menu,
            packfile_new_packfile,
            packfile_open_packfile,
            packfile_save_packfile,
            packfile_save_packfile_as,
            packfile_load_all_ca_packfiles,
            packfile_change_packfile_type,
            packfile_index_includes_timestamp,
            packfile_data_is_compressed,
            packfile_preferences,
            packfile_quit,

            //-----------------------------------------------//
            // `MyMod` menu slots.
            //-----------------------------------------------//
            mymod_open_menu,
            mymod_open_mymod_folder,
            mymod_new,
            mymod_delete_selected,
            mymod_install,
            mymod_uninstall,

            //-----------------------------------------------//
            // `View` menu slots.
            //-----------------------------------------------//
            view_toggle_packfile_contents,
            view_toggle_global_search_panel,
            view_toggle_diagnostics_panel,

            //-----------------------------------------------//
            // `Game Selected` menu slots.
            //-----------------------------------------------//
            game_selected_launch_game,
            game_selected_open_game_data_folder,
            game_selected_open_game_assembly_kit_folder,
            game_selected_open_config_folder,
            change_game_selected,

            //-----------------------------------------------//
            // `Special Stuff` menu slots.
            //-----------------------------------------------//
            special_stuff_generate_pak_file,
            special_stuff_optimize_packfile,
            special_stuff_patch_siege_ai,

            //-----------------------------------------------//
            // `Templates` menu slots.
            //-----------------------------------------------//
            templates_open_custom_templates_folder,
            templates_open_official_templates_folder,
            templates_save_packfile_to_template,

    		//-----------------------------------------------//
	        // `About` menu slots.
	        //-----------------------------------------------//
    		about_about_qt,
            about_about_rpfm,
            about_open_manual,
            about_patreon_link,
            about_check_updates,
            about_check_schema_updates,
            about_check_templates_updates,

            //-----------------------------------------------//
            // `Debug` menu slots.
            //-----------------------------------------------//
            debug_update_current_schema_from_asskit,

            //-----------------------------------------------//
            // `PackedFileView` slots.
            //-----------------------------------------------//
            packed_file_hide,
            packed_file_update,
            packed_file_unpreview,

            //-----------------------------------------------//
            // `Generic` slots.
            //-----------------------------------------------//
            pack_file_backup_autosave,

            tab_bar_packed_file_context_menu_show,
            tab_bar_packed_file_close,
            tab_bar_packed_file_close_all,
            tab_bar_packed_file_close_all_left,
            tab_bar_packed_file_close_all_right,
            tab_bar_packed_file_prev,
            tab_bar_packed_file_next,
		}
	}
}

impl AppUITempSlots {
    pub unsafe fn build(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
    ) {
        AppUI::build_open_from_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
        AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui);
    }
}
