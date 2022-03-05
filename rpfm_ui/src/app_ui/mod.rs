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
Module with all the code related to the main `AppUI`.

This module contains all the code needed to initialize the main Window and its menus.
!*/

use qt_widgets::QAction;
use qt_widgets::QActionGroup;
use qt_widgets::QApplication;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QMenuBar;
use qt_widgets::QStatusBar;
use qt_widgets::QTabWidget;
use qt_widgets::QWidget;

use qt_gui::QIcon;

use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QTimer;
use qt_core::QPtr;
use qt_core::QString;

use std::sync::atomic::Ordering;

use rpfm_lib::games::supported_games::*;
use rpfm_lib::packedfile::text::TextType;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::ffi::are_you_sure;
use crate::ffi::new_q_main_window_custom_safe;
use crate::locale::qtr;
use crate::ASSETS_PATH;
use crate::STATUS_BAR;
use crate::utils::create_grid_layout;

mod app_ui_extra;
pub mod connections;
pub mod shortcuts;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to all the static widgets/actions created at the start of the program.
///
/// This means every widget/action that's static and created on start (menus, window,...) should be here.
#[derive(Debug)]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // Main Window.
    //-------------------------------------------------------------------------------//
    pub main_window: QBox<QMainWindow>,
    pub tab_bar_packed_file: QBox<QTabWidget>,
    pub menu_bar: QPtr<QMenuBar>,
    pub status_bar: QPtr<QStatusBar>,

    //-------------------------------------------------------------------------------//
    // `MenuBar` menus.
    //-------------------------------------------------------------------------------//
    pub menu_bar_packfile: QPtr<QMenu>,
    pub menu_bar_mymod: QPtr<QMenu>,
    pub menu_bar_view: QPtr<QMenu>,
    pub menu_bar_game_selected: QPtr<QMenu>,
    pub menu_bar_special_stuff: QPtr<QMenu>,
    pub menu_bar_tools: QPtr<QMenu>,
    pub menu_bar_about: QPtr<QMenu>,
    pub menu_bar_debug: QPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
    pub packfile_new_packfile: QPtr<QAction>,
    pub packfile_open_packfile: QPtr<QAction>,
    pub packfile_save_packfile: QPtr<QAction>,
    pub packfile_save_packfile_as: QPtr<QAction>,
    pub packfile_install: QPtr<QAction>,
    pub packfile_uninstall: QPtr<QAction>,
    pub packfile_open_recent: QBox<QMenu>,
    pub packfile_open_from_content: QBox<QMenu>,
    pub packfile_open_from_data: QBox<QMenu>,
    pub packfile_open_from_autosave: QBox<QMenu>,
    pub packfile_change_packfile_type: QBox<QMenu>,
    pub packfile_load_all_ca_packfiles: QPtr<QAction>,
    pub packfile_preferences: QPtr<QAction>,
    pub packfile_quit: QPtr<QAction>,

    // "Change PackFile Type" submenu.
    pub change_packfile_type_boot: QPtr<QAction>,
    pub change_packfile_type_release: QPtr<QAction>,
    pub change_packfile_type_patch: QPtr<QAction>,
    pub change_packfile_type_mod: QPtr<QAction>,
    pub change_packfile_type_movie: QPtr<QAction>,
    pub change_packfile_type_other: QPtr<QAction>,

    pub change_packfile_type_header_is_extended: QPtr<QAction>,
    pub change_packfile_type_index_includes_timestamp: QPtr<QAction>,
    pub change_packfile_type_index_is_encrypted: QPtr<QAction>,
    pub change_packfile_type_data_is_encrypted: QPtr<QAction>,

    // Action to enable/disable compression on PackFiles. Only for PFH5+ PackFiles.
    pub change_packfile_type_data_is_compressed: QPtr<QAction>,

    // Action Group for the submenu.
    pub change_packfile_type_group: QBox<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `MyMod` menu.
    //-------------------------------------------------------------------------------//
    pub mymod_open_mymod_folder: QPtr<QAction>,
    pub mymod_new: QPtr<QAction>,
    pub mymod_delete_selected: QPtr<QAction>,
    pub mymod_import: QPtr<QAction>,
    pub mymod_export: QPtr<QAction>,

    pub mymod_open_warhammer_3: QPtr<QMenu>,
    pub mymod_open_troy: QPtr<QMenu>,
    pub mymod_open_three_kingdoms: QPtr<QMenu>,
    pub mymod_open_warhammer_2: QPtr<QMenu>,
    pub mymod_open_warhammer: QPtr<QMenu>,
    pub mymod_open_thrones_of_britannia: QPtr<QMenu>,
    pub mymod_open_attila: QPtr<QMenu>,
    pub mymod_open_rome_2: QPtr<QMenu>,
    pub mymod_open_shogun_2: QPtr<QMenu>,
    pub mymod_open_napoleon: QPtr<QMenu>,
    pub mymod_open_empire: QPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
    pub view_toggle_packfile_contents: QPtr<QAction>,
    pub view_toggle_global_search_panel: QPtr<QAction>,
    pub view_toggle_diagnostics_panel: QPtr<QAction>,
    pub view_toggle_dependencies_panel: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    pub game_selected_launch_game: QPtr<QAction>,

    pub game_selected_open_game_data_folder: QPtr<QAction>,
    pub game_selected_open_game_assembly_kit_folder: QPtr<QAction>,
    pub game_selected_open_config_folder: QPtr<QAction>,

    pub game_selected_warhammer_3: QPtr<QAction>,
    pub game_selected_troy: QPtr<QAction>,
    pub game_selected_three_kingdoms: QPtr<QAction>,
    pub game_selected_warhammer_2: QPtr<QAction>,
    pub game_selected_warhammer: QPtr<QAction>,
    pub game_selected_thrones_of_britannia: QPtr<QAction>,
    pub game_selected_attila: QPtr<QAction>,
    pub game_selected_rome_2: QPtr<QAction>,
    pub game_selected_shogun_2: QPtr<QAction>,
    pub game_selected_napoleon: QPtr<QAction>,
    pub game_selected_empire: QPtr<QAction>,
    pub game_selected_arena: QPtr<QAction>,

    pub game_selected_group: QBox<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//

    // Warhammer 3 actions.
    pub special_stuff_wh3_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_wh3_optimize_packfile: QPtr<QAction>,

    // Troy actions.
    pub special_stuff_troy_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_troy_optimize_packfile: QPtr<QAction>,

    // Three Kingdoms actions.
    pub special_stuff_three_k_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_three_k_optimize_packfile: QPtr<QAction>,

    // Warhammer 2's actions.
    pub special_stuff_wh2_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_wh2_optimize_packfile: QPtr<QAction>,
    pub special_stuff_wh2_patch_siege_ai: QPtr<QAction>,

    // Warhammer's actions.
    pub special_stuff_wh_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_wh_optimize_packfile: QPtr<QAction>,
    pub special_stuff_wh_patch_siege_ai: QPtr<QAction>,

    // Thrones of Britannia's actions.
    pub special_stuff_tob_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_tob_optimize_packfile: QPtr<QAction>,

    // Attila's actions.
    pub special_stuff_att_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_att_optimize_packfile: QPtr<QAction>,

    // Rome 2's actions.
    pub special_stuff_rom2_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_rom2_optimize_packfile: QPtr<QAction>,

    // Shogun 2's actions.
    pub special_stuff_sho2_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_sho2_optimize_packfile: QPtr<QAction>,

    // Napoleon's actions.
    pub special_stuff_nap_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_nap_optimize_packfile: QPtr<QAction>,

    // Empire's actions.
    pub special_stuff_emp_generate_dependencies_cache: QPtr<QAction>,
    pub special_stuff_emp_optimize_packfile: QPtr<QAction>,

    // Common operations.
    pub special_stuff_rescue_packfile: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Tools` menu.
    //-------------------------------------------------------------------------------//
    pub tools_faction_painter: QPtr<QAction>,
    pub tools_unit_editor: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    pub about_about_qt: QPtr<QAction>,
    pub about_about_rpfm: QPtr<QAction>,
    pub about_open_manual: QPtr<QAction>,
    pub about_patreon_link: QPtr<QAction>,
    pub about_check_updates: QPtr<QAction>,
    pub about_check_schema_updates: QPtr<QAction>,
    pub about_check_message_updates: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // "Debug" menu.
    //-------------------------------------------------------------------------------//
    pub debug_update_current_schema_from_asskit: QPtr<QAction>,
    pub debug_import_schema_patch: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Extra stuff
    //-------------------------------------------------------------------------------//
    pub timer_backup_autosave: QBox<QTimer>,

    pub tab_bar_packed_file_context_menu: QBox<QMenu>,
    pub tab_bar_packed_file_close: QPtr<QAction>,
    pub tab_bar_packed_file_close_all: QPtr<QAction>,
    pub tab_bar_packed_file_close_all_left: QPtr<QAction>,
    pub tab_bar_packed_file_close_all_right: QPtr<QAction>,
    pub tab_bar_packed_file_prev: QPtr<QAction>,
    pub tab_bar_packed_file_next: QPtr<QAction>,
    pub tab_bar_packed_file_import_from_dependencies: QPtr<QAction>,
    pub tab_bar_packed_file_toggle_tips: QPtr<QAction>,
}

/// This enum contains the data needed to create a new PackedFile.
#[derive(Clone, Debug)]
pub enum NewPackedFile {

    /// Name of the PackedFile.
    AnimPack(String),

    /// Name of the PackedFile, Name of the Table, Version of the Table.
    DB(String, String, i32),

    /// Name of the Table.
    Loc(String),

    /// Name of the Table.
    Text(String, TextType)
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUI`.
impl AppUI {

    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    pub unsafe fn new() -> Self {

        // Initialize and configure the main window.
        let main_window = new_q_main_window_custom_safe(are_you_sure);
        let widget = QWidget::new_1a(&main_window);
        let layout = create_grid_layout(widget.static_upcast());
        main_window.set_central_widget(&widget);
        main_window.resize_2a(1300, 800);
        QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/rpfm.png", ASSETS_PATH.to_string_lossy()))));

        // Get the menu and status bars.
        let menu_bar = main_window.menu_bar();
        let status_bar = main_window.status_bar();
        let tab_bar_packed_file = QTabWidget::new_1a(&widget);
        tab_bar_packed_file.set_tabs_closable(true);
        tab_bar_packed_file.set_movable(true);
        tab_bar_packed_file.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        layout.add_widget_5a(&tab_bar_packed_file, 0, 0, 1, 1);
        STATUS_BAR.store(status_bar.as_mut_raw_ptr(), Ordering::SeqCst);

        let tab_bar_packed_file_context_menu = QMenu::from_q_widget(&tab_bar_packed_file);

        // Create the Contextual Menu Actions.
        let tab_bar_packed_file_close = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("close_tab"));
        let tab_bar_packed_file_close_all = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("close_all_other_tabs"));
        let tab_bar_packed_file_close_all_left = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("close_tabs_to_left"));
        let tab_bar_packed_file_close_all_right = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("close_tabs_to_right"));
        let tab_bar_packed_file_prev = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("prev_tab"));
        let tab_bar_packed_file_next = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("next_tab"));
        let tab_bar_packed_file_import_from_dependencies = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("import_from_dependencies"));
        let tab_bar_packed_file_toggle_tips = tab_bar_packed_file_context_menu.add_action_q_string(&qtr("toggle_tips"));

        tab_bar_packed_file_close.set_enabled(true);
        tab_bar_packed_file_close_all.set_enabled(true);
        tab_bar_packed_file_close_all_left.set_enabled(true);
        tab_bar_packed_file_close_all_right.set_enabled(true);
        tab_bar_packed_file_prev.set_enabled(true);
        tab_bar_packed_file_next.set_enabled(true);
        tab_bar_packed_file_import_from_dependencies.set_enabled(true);
        tab_bar_packed_file_toggle_tips.set_enabled(true);

        tab_bar_packed_file_context_menu.insert_separator(&tab_bar_packed_file_prev);
        tab_bar_packed_file_context_menu.insert_separator(&tab_bar_packed_file_import_from_dependencies);

        //-----------------------------------------------//
        // Menu bar.
        //-----------------------------------------------//

        // Create the `MenuBar` menus.
        let menu_bar_packfile = menu_bar.add_menu_q_string(&qtr("menu_bar_packfile"));
        let menu_bar_mymod = menu_bar.add_menu_q_string(&qtr("menu_bar_mymod"));
        let menu_bar_view = menu_bar.add_menu_q_string(&qtr("menu_bar_view"));
        let menu_bar_game_selected = menu_bar.add_menu_q_string(&qtr("menu_bar_game_selected"));
        let menu_bar_special_stuff = menu_bar.add_menu_q_string(&qtr("menu_bar_special_stuff"));
        let menu_bar_tools = menu_bar.add_menu_q_string(&qtr("menu_bar_tools"));
        let menu_bar_about = menu_bar.add_menu_q_string(&qtr("menu_bar_about"));

        // This menu is hidden unless you enable it.
        let menu_bar_debug = menu_bar.add_menu_q_string(&qtr("menu_bar_debug"));
        if !SETTINGS.read().unwrap().settings_bool["enable_debug_menu"] {
            menu_bar_debug.menu_action().set_visible(false);
        }

        //-----------------------------------------------//
        // `PackFile` Menu.
        //-----------------------------------------------//

        // Populate the `PackFile` menu.
        let packfile_new_packfile = menu_bar_packfile.add_action_q_string(&qtr("new_packfile"));
        let packfile_open_packfile = menu_bar_packfile.add_action_q_string(&qtr("open_packfile"));
        let packfile_save_packfile = menu_bar_packfile.add_action_q_string(&qtr("save_packfile"));
        let packfile_save_packfile_as = menu_bar_packfile.add_action_q_string(&qtr("save_packfile_as"));
        let packfile_install = menu_bar_packfile.add_action_q_string(&qtr("packfile_install"));
        let packfile_uninstall = menu_bar_packfile.add_action_q_string(&qtr("packfile_uninstall"));
        let packfile_open_recent = QMenu::from_q_string_q_widget(&qtr("open_recent"), &menu_bar_packfile);
        let packfile_open_from_content = QMenu::from_q_string_q_widget(&qtr("open_from_content"), &menu_bar_packfile);
        let packfile_open_from_data = QMenu::from_q_string_q_widget(&qtr("open_from_data"), &menu_bar_packfile);
        let packfile_open_from_autosave = QMenu::from_q_string_q_widget(&qtr("open_from_autosave"), &menu_bar_packfile);
        let packfile_change_packfile_type = QMenu::from_q_string_q_widget(&qtr("change_packfile_type"), &menu_bar_packfile);
        let packfile_load_all_ca_packfiles = menu_bar_packfile.add_action_q_string(&qtr("load_all_ca_packfiles"));
        let packfile_preferences = menu_bar_packfile.add_action_q_string(&qtr("preferences"));
        let packfile_quit = menu_bar_packfile.add_action_q_string(&qtr("quit"));

        // Add the "Open..." submenus. These needs to be here because they have to be inserted in specific positions of the menu.
        menu_bar_packfile.insert_menu(&packfile_load_all_ca_packfiles, &packfile_open_recent);
        menu_bar_packfile.insert_menu(&packfile_load_all_ca_packfiles, &packfile_open_from_content);
        menu_bar_packfile.insert_menu(&packfile_load_all_ca_packfiles, &packfile_open_from_data);
        menu_bar_packfile.insert_menu(&packfile_load_all_ca_packfiles, &packfile_open_from_autosave);

        menu_bar_packfile.insert_separator(packfile_open_recent.menu_action());
        menu_bar_packfile.insert_separator(&packfile_preferences);
        menu_bar_packfile.insert_menu(&packfile_preferences, &packfile_change_packfile_type);
        menu_bar_packfile.insert_separator(&packfile_preferences);

        // `Change PackFile Type` submenu.
        let change_packfile_type_boot = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_boot"));
        let change_packfile_type_release = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_release"));
        let change_packfile_type_patch = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_patch"));
        let change_packfile_type_mod = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_mod"));
        let change_packfile_type_movie = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_movie"));
        let change_packfile_type_other = packfile_change_packfile_type.add_action_q_string(&qtr("packfile_type_other"));
        let change_packfile_type_header_is_extended = packfile_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_header_is_extended"));
        let change_packfile_type_index_includes_timestamp = packfile_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_includes_timestamp"));
        let change_packfile_type_index_is_encrypted = packfile_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_is_encrypted"));
        let change_packfile_type_data_is_encrypted = packfile_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_data_is_encrypted"));
        let change_packfile_type_data_is_compressed = packfile_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_data_is_compressed"));

        let change_packfile_type_group = QActionGroup::new(&packfile_change_packfile_type);

        // Configure the `PackFile` menu and his submenu.
        change_packfile_type_group.add_action_q_action(&change_packfile_type_boot);
        change_packfile_type_group.add_action_q_action(&change_packfile_type_release);
        change_packfile_type_group.add_action_q_action(&change_packfile_type_patch);
        change_packfile_type_group.add_action_q_action(&change_packfile_type_mod);
        change_packfile_type_group.add_action_q_action(&change_packfile_type_movie);
        change_packfile_type_group.add_action_q_action(&change_packfile_type_other);
        change_packfile_type_boot.set_checkable(true);
        change_packfile_type_release.set_checkable(true);
        change_packfile_type_patch.set_checkable(true);
        change_packfile_type_mod.set_checkable(true);
        change_packfile_type_movie.set_checkable(true);
        change_packfile_type_other.set_checkable(true);

        // These ones are individual, but they need to be checkable and not editable.
        change_packfile_type_data_is_encrypted.set_checkable(true);
        change_packfile_type_index_includes_timestamp.set_checkable(true);
        change_packfile_type_index_is_encrypted.set_checkable(true);
        change_packfile_type_header_is_extended.set_checkable(true);
        change_packfile_type_data_is_compressed.set_checkable(true);

        change_packfile_type_data_is_encrypted.set_enabled(false);
        change_packfile_type_index_is_encrypted.set_enabled(false);
        change_packfile_type_header_is_extended.set_enabled(false);
        change_packfile_type_data_is_compressed.set_enabled(false);

        // Put separators in the SubMenu.
        packfile_change_packfile_type.insert_separator(&change_packfile_type_other);
        packfile_change_packfile_type.insert_separator(&change_packfile_type_header_is_extended);
        packfile_change_packfile_type.insert_separator(&change_packfile_type_data_is_compressed);

        //-----------------------------------------------//
        // `MyMod` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let mymod_open_mymod_folder = menu_bar_mymod.add_action_q_string(&qtr("mymod_open_mymod_folder"));
        let mymod_new = menu_bar_mymod.add_action_q_string(&qtr("mymod_new"));
        let mymod_delete_selected = menu_bar_mymod.add_action_q_string(&qtr("mymod_delete_selected"));
        let mymod_import = menu_bar_mymod.add_action_q_string(&qtr("mymod_import"));
        let mymod_export = menu_bar_mymod.add_action_q_string(&qtr("mymod_export"));

        menu_bar_mymod.add_separator();

        let mymod_open_warhammer_3 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_3));
        let mymod_open_troy = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_TROY));
        let mymod_open_three_kingdoms = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_THREE_KINGDOMS));
        let mymod_open_warhammer_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_2));
        let mymod_open_warhammer = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER));
        let mymod_open_thrones_of_britannia = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_THRONES_OF_BRITANNIA));
        let mymod_open_attila = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_ATTILA));
        let mymod_open_rome_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_ROME_2));
        let mymod_open_shogun_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_SHOGUN_2));
        let mymod_open_napoleon = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_NAPOLEON));
        let mymod_open_empire = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_EMPIRE));

        menu_bar_mymod.insert_separator(&mymod_new);

        mymod_new.set_enabled(false);
        mymod_delete_selected.set_enabled(false);
        mymod_import.set_enabled(false);
        mymod_export.set_enabled(false);

        mymod_open_warhammer_3.menu_action().set_visible(false);
        mymod_open_troy.menu_action().set_visible(false);
        mymod_open_three_kingdoms.menu_action().set_visible(false);
        mymod_open_warhammer_2.menu_action().set_visible(false);
        mymod_open_warhammer.menu_action().set_visible(false);
        mymod_open_thrones_of_britannia.menu_action().set_visible(false);
        mymod_open_attila.menu_action().set_visible(false);
        mymod_open_rome_2.menu_action().set_visible(false);
        mymod_open_shogun_2.menu_action().set_visible(false);
        mymod_open_napoleon.menu_action().set_visible(false);
        mymod_open_empire.menu_action().set_visible(false);

        //-----------------------------------------------//
        // `View` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let view_toggle_packfile_contents = menu_bar_view.add_action_q_string(&qtr("view_toggle_packfile_contents"));
        let view_toggle_global_search_panel = menu_bar_view.add_action_q_string(&qtr("view_toggle_global_search_panel"));
        let view_toggle_diagnostics_panel = menu_bar_view.add_action_q_string(&qtr("view_toggle_diagnostics_panel"));
        let view_toggle_dependencies_panel = menu_bar_view.add_action_q_string(&qtr("view_toggle_dependencies_panel"));

        view_toggle_packfile_contents.set_checkable(true);
        view_toggle_global_search_panel.set_checkable(true);
        view_toggle_diagnostics_panel.set_checkable(true);
        view_toggle_dependencies_panel.set_checkable(true);

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let game_selected_launch_game = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_launch_game"));

        let game_selected_open_game_data_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_game_data_folder"));
        let game_selected_open_game_assembly_kit_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_game_assembly_kit_folder"));
        let game_selected_open_config_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_config_folder"));

        let game_selected_warhammer_3 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_3));
        let game_selected_troy = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_TROY));
        let game_selected_three_kingdoms = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_THREE_KINGDOMS));
        let game_selected_warhammer_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_2));
        let game_selected_warhammer = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER));
        let game_selected_thrones_of_britannia = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_THRONES_OF_BRITANNIA));
        let game_selected_attila = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_ATTILA));
        let game_selected_rome_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_ROME_2));
        let game_selected_shogun_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_SHOGUN_2));
        let game_selected_napoleon = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_NAPOLEON));
        let game_selected_empire = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_EMPIRE));
        let game_selected_arena = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_ARENA));

        game_selected_warhammer_3.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_WARHAMMER_3).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_troy.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_TROY).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_three_kingdoms.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_THREE_KINGDOMS).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_warhammer_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_WARHAMMER_2).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_warhammer.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_WARHAMMER).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_thrones_of_britannia.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_THRONES_OF_BRITANNIA).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_attila.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_ATTILA).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_rome_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_ROME_2).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_shogun_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_SHOGUN_2).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_napoleon.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_NAPOLEON).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_empire.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_EMPIRE).unwrap().get_game_selected_icon_file_name()))).as_ref());
        game_selected_arena.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get_supported_game_from_key(KEY_ARENA).unwrap().get_game_selected_icon_file_name()))).as_ref());

        let game_selected_group = QActionGroup::new(&menu_bar_game_selected);

        // Configure the `Game Selected` Menu.
        menu_bar_game_selected.insert_separator(&game_selected_warhammer_3);
        menu_bar_game_selected.insert_separator(&game_selected_arena);
        game_selected_group.add_action_q_action(&game_selected_warhammer_3);
        game_selected_group.add_action_q_action(&game_selected_troy);
        game_selected_group.add_action_q_action(&game_selected_three_kingdoms);
        game_selected_group.add_action_q_action(&game_selected_warhammer_2);
        game_selected_group.add_action_q_action(&game_selected_warhammer);
        game_selected_group.add_action_q_action(&game_selected_thrones_of_britannia);
        game_selected_group.add_action_q_action(&game_selected_attila);
        game_selected_group.add_action_q_action(&game_selected_rome_2);
        game_selected_group.add_action_q_action(&game_selected_shogun_2);
        game_selected_group.add_action_q_action(&game_selected_napoleon);
        game_selected_group.add_action_q_action(&game_selected_empire);
        game_selected_group.add_action_q_action(&game_selected_arena);
        game_selected_warhammer_3.set_checkable(true);
        game_selected_troy.set_checkable(true);
        game_selected_three_kingdoms.set_checkable(true);
        game_selected_warhammer_2.set_checkable(true);
        game_selected_warhammer.set_checkable(true);
        game_selected_thrones_of_britannia.set_checkable(true);
        game_selected_attila.set_checkable(true);
        game_selected_rome_2.set_checkable(true);
        game_selected_shogun_2.set_checkable(true);
        game_selected_napoleon.set_checkable(true);
        game_selected_empire.set_checkable(true);
        game_selected_arena.set_checkable(true);

        //-----------------------------------------------//
        // `Special Stuff` Menu.
        //-----------------------------------------------//

        // Populate the `Special Stuff` menu with submenus.
        let menu_warhammer_3 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_3));
        let menu_troy = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_TROY));
        let menu_three_kingdoms = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_THREE_KINGDOMS));
        let menu_warhammer_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER_2));
        let menu_warhammer = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_WARHAMMER));
        let menu_thrones_of_britannia = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_THRONES_OF_BRITANNIA));
        let menu_attila = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_ATTILA));
        let menu_rome_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_ROME_2));
        let menu_shogun_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_SHOGUN_2));
        let menu_napoleon = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_NAPOLEON));
        let menu_empire = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_EMPIRE));
        let special_stuff_rescue_packfile = menu_bar_special_stuff.add_action_q_string(&qtr("special_stuff_rescue_packfile"));

        // Populate the `Special Stuff` submenus.
        let special_stuff_wh3_generate_dependencies_cache = menu_warhammer_3.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_wh3_optimize_packfile = menu_warhammer_3.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_troy_generate_dependencies_cache = menu_troy.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_troy_optimize_packfile = menu_troy.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_three_k_generate_dependencies_cache = menu_three_kingdoms.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_three_k_optimize_packfile = menu_three_kingdoms.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh2_generate_dependencies_cache = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_wh2_optimize_packfile = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh2_patch_siege_ai = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_patch_siege_ai"));
        let special_stuff_wh_generate_dependencies_cache = menu_warhammer.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_wh_optimize_packfile = menu_warhammer.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh_patch_siege_ai = menu_warhammer.add_action_q_string(&qtr("special_stuff_patch_siege_ai"));
        let special_stuff_tob_generate_dependencies_cache = menu_thrones_of_britannia.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_tob_optimize_packfile = menu_thrones_of_britannia.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_att_generate_dependencies_cache = menu_attila.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_att_optimize_packfile = menu_attila.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_rom2_generate_dependencies_cache = menu_rome_2.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_rom2_optimize_packfile = menu_rome_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_sho2_generate_dependencies_cache = menu_shogun_2.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_sho2_optimize_packfile = menu_shogun_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_nap_generate_dependencies_cache = menu_napoleon.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_nap_optimize_packfile = menu_napoleon.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_emp_generate_dependencies_cache = menu_empire.add_action_q_string(&qtr("special_stuff_generate_dependencies_cache"));
        let special_stuff_emp_optimize_packfile = menu_empire.add_action_q_string(&qtr("special_stuff_optimize_packfile"));

        menu_bar_special_stuff.insert_separator(&special_stuff_rescue_packfile);

        //-----------------------------------------------//
        // `Tools` Menu.
        //-----------------------------------------------//

        // Populate the `Tools` menu.
        let tools_faction_painter = menu_bar_tools.add_action_q_string(&qtr("tools_faction_painter"));
        let tools_unit_editor = menu_bar_tools.add_action_q_string(&qtr("tools_unit_editor"));
        if !SETTINGS.read().unwrap().settings_bool["enable_unit_editor"] {
            tools_unit_editor.set_enabled(false);
        }

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//

        // Populate the `About` menu.
        let about_about_qt = menu_bar_about.add_action_q_string(&qtr("about_about_qt"));
        let about_about_rpfm = menu_bar_about.add_action_q_string(&qtr("about_about_rpfm"));
        let about_open_manual = menu_bar_about.add_action_q_string(&qtr("about_open_manual"));
        let about_patreon_link = menu_bar_about.add_action_q_string(&qtr("about_patreon_link"));
        let about_check_updates = menu_bar_about.add_action_q_string(&qtr("about_check_updates"));
        let about_check_schema_updates = menu_bar_about.add_action_q_string(&qtr("about_check_schema_updates"));
        let about_check_message_updates = menu_bar_about.add_action_q_string(&qtr("about_check_message_updates"));

        //-----------------------------------------------//
        // `Debug` Menu.
        //-----------------------------------------------//

        // Populate the `Debug` menu.
        let debug_update_current_schema_from_asskit = menu_bar_debug.add_action_q_string(&qtr("update_current_schema_from_asskit"));
        let debug_import_schema_patch = menu_bar_debug.add_action_q_string(&qtr("import_schema_patch"));

        //-------------------------------------------------------------------------------//
        // "Extra stuff" menu.
        //-------------------------------------------------------------------------------//
        let timer_backup_autosave = QTimer::new_1a(&main_window);
        timer_backup_autosave.set_single_shot(true);

        // Create ***Da monsta***.
        AppUI {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,
            tab_bar_packed_file,
            menu_bar,
            status_bar,

            //-------------------------------------------------------------------------------//
            // `MenuBar` menus.
            //-------------------------------------------------------------------------------//
            menu_bar_packfile,
            menu_bar_mymod,
            menu_bar_view,
            menu_bar_game_selected,
            menu_bar_special_stuff,
            menu_bar_tools,
            menu_bar_about,
            menu_bar_debug,

            //-------------------------------------------------------------------------------//
            // "PackFile" menu.
            //-------------------------------------------------------------------------------//

            // Menus.
            packfile_new_packfile,
            packfile_open_packfile,
            packfile_save_packfile,
            packfile_save_packfile_as,
            packfile_install,
            packfile_uninstall,
            packfile_open_recent,
            packfile_open_from_content,
            packfile_open_from_data,
            packfile_open_from_autosave,
            packfile_change_packfile_type,
            packfile_load_all_ca_packfiles,
            packfile_preferences,
            packfile_quit,

            // "Change PackFile Type" submenu.
            change_packfile_type_boot,
            change_packfile_type_release,
            change_packfile_type_patch,
            change_packfile_type_mod,
            change_packfile_type_movie,
            change_packfile_type_other,

            change_packfile_type_header_is_extended,
            change_packfile_type_index_includes_timestamp,
            change_packfile_type_index_is_encrypted,
            change_packfile_type_data_is_encrypted,

            // Action for the PackFile compression.
            change_packfile_type_data_is_compressed,

            // Action Group for the submenu.
            change_packfile_type_group,

            //-------------------------------------------------------------------------------//
            // `MyMod` menu.
            //-------------------------------------------------------------------------------//
            mymod_open_mymod_folder,
            mymod_new,
            mymod_delete_selected,
            mymod_import,
            mymod_export,

            mymod_open_warhammer_3,
            mymod_open_troy,
            mymod_open_three_kingdoms,
            mymod_open_warhammer_2,
            mymod_open_warhammer,
            mymod_open_thrones_of_britannia,
            mymod_open_attila,
            mymod_open_rome_2,
            mymod_open_shogun_2,
            mymod_open_napoleon,
            mymod_open_empire,

            //-------------------------------------------------------------------------------//
            // "View" menu.
            //-------------------------------------------------------------------------------//
            view_toggle_packfile_contents,
            view_toggle_global_search_panel,
            view_toggle_diagnostics_panel,
            view_toggle_dependencies_panel,

            //-------------------------------------------------------------------------------//
            // "Game Selected" menu.
            //-------------------------------------------------------------------------------//
            game_selected_launch_game,

            game_selected_open_game_data_folder,
            game_selected_open_game_assembly_kit_folder,
            game_selected_open_config_folder,

            game_selected_warhammer_3,
            game_selected_troy,
            game_selected_three_kingdoms,
            game_selected_warhammer_2,
            game_selected_warhammer,
            game_selected_thrones_of_britannia,
            game_selected_attila,
            game_selected_rome_2,
            game_selected_shogun_2,
            game_selected_napoleon,
            game_selected_empire,
            game_selected_arena,

            game_selected_group,

            //-------------------------------------------------------------------------------//
            // "Special Stuff" menu.
            //-------------------------------------------------------------------------------//

            // Warhammer 3 actions.
            special_stuff_wh3_generate_dependencies_cache,
            special_stuff_wh3_optimize_packfile,

            // Troy actions.
            special_stuff_troy_generate_dependencies_cache,
            special_stuff_troy_optimize_packfile,

            // Three Kingdoms actions.
            special_stuff_three_k_generate_dependencies_cache,
            special_stuff_three_k_optimize_packfile,

            // Warhammer 2's actions.
            special_stuff_wh2_generate_dependencies_cache,
            special_stuff_wh2_optimize_packfile,
            special_stuff_wh2_patch_siege_ai,

            // Warhammer's actions.
            special_stuff_wh_generate_dependencies_cache,
            special_stuff_wh_optimize_packfile,
            special_stuff_wh_patch_siege_ai,

            // Thrones of Britannia's actions.
            special_stuff_tob_generate_dependencies_cache,
            special_stuff_tob_optimize_packfile,

            // Attila's actions.
            special_stuff_att_generate_dependencies_cache,
            special_stuff_att_optimize_packfile,

            // Rome 2's actions.
            special_stuff_rom2_generate_dependencies_cache,
            special_stuff_rom2_optimize_packfile,

            // Shogun 2's actions.
            special_stuff_sho2_generate_dependencies_cache,
            special_stuff_sho2_optimize_packfile,

            // Napoleon's actions.
            special_stuff_nap_generate_dependencies_cache,
            special_stuff_nap_optimize_packfile,

            // Empire's actions.
            special_stuff_emp_generate_dependencies_cache,
            special_stuff_emp_optimize_packfile,

            // Common operations.
            special_stuff_rescue_packfile,

            //-------------------------------------------------------------------------------//
            // "Tools" menu.
            //-------------------------------------------------------------------------------//
            tools_faction_painter,
            tools_unit_editor,

            //-------------------------------------------------------------------------------//
            // "About" menu.
            //-------------------------------------------------------------------------------//
            about_about_qt,
            about_about_rpfm,
            about_open_manual,
            about_patreon_link,
            about_check_updates,
            about_check_schema_updates,
            about_check_message_updates,

            //-------------------------------------------------------------------------------//
            // "Debug" menu.
            //-------------------------------------------------------------------------------//
            debug_update_current_schema_from_asskit,
            debug_import_schema_patch,

            //-------------------------------------------------------------------------------//
            // "Extra stuff" menu.
            //-------------------------------------------------------------------------------//
            timer_backup_autosave,

            tab_bar_packed_file_context_menu,
            tab_bar_packed_file_close,
            tab_bar_packed_file_close_all,
            tab_bar_packed_file_close_all_left,
            tab_bar_packed_file_close_all_right,
            tab_bar_packed_file_prev,
            tab_bar_packed_file_next,
            tab_bar_packed_file_import_from_dependencies,
            tab_bar_packed_file_toggle_tips
        }
    }
}
