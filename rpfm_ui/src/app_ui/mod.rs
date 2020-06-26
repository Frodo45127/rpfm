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
Module with all the code related to the main `AppUI`.

This module contains all the code needed to initialize the main Window and its menus.
!*/

use qt_widgets::q_abstract_item_view::SelectionBehavior;
use qt_widgets::QAction;
use qt_widgets::QActionGroup;
use qt_widgets::QApplication;
use qt_widgets::QCompleter;
use qt_widgets::QDockWidget;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QMenuBar;
use qt_widgets::QStatusBar;
use qt_widgets::QTabWidget;
use qt_widgets::QTableView;
use qt_widgets::QWidget;
use qt_widgets::q_dock_widget::DockWidgetFeature;

use qt_gui::QIcon;
use qt_gui::QStandardItemModel;

use qt_core::QFlags;
use qt_core::CaseSensitivity;
use qt_core::QString;
use qt_core::WindowType;
use qt_core::MatchFlag;

use cpp_core::MutPtr;

use std::sync::atomic::Ordering;

use rpfm_lib::games::*;
use rpfm_lib::packedfile::text::TextType;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::ffi::new_tableview_command_palette_safe;
use crate::locale::qtr;
use crate::ASSETS_PATH;
use crate::STATUS_BAR;
use crate::utils::create_grid_layout;

mod app_ui_extra;
pub mod connections;
pub mod shortcuts;
pub mod slots;
pub mod tips;

// Display name, adapted to support Pnemonics.
const GAME_SELECTED_THREE_KINGDOMS: &str = "Three &Kingdoms";
const GAME_SELECTED_WARHAMMER_2: &str = "&Warhammer 2";
const GAME_SELECTED_WARHAMMER: &str = "War&hammer";
const GAME_SELECTED_THRONES_OF_BRITANNIA: &str = "&Thrones of Britannia";
const GAME_SELECTED_ATTILA: &str = "&Attila";
const GAME_SELECTED_ROME_2: &str = "R&ome 2";
const GAME_SELECTED_SHOGUN_2: &str = "&Shogun 2";
const GAME_SELECTED_NAPOLEON: &str = "&Napoleon";
const GAME_SELECTED_EMPIRE: &str = "&Empire";
const GAME_SELECTED_ARENA: &str = "A&rena";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to all the static widgets/actions created at the start of the program.
///
/// This means every widget/action that's static and created on start (menus, window,...) should be here.
#[derive(Copy, Clone, Debug)]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // `Command Palette` DockWidget.
    //-------------------------------------------------------------------------------//
    pub command_palette: MutPtr<QDockWidget>,
    pub command_palette_line_edit: MutPtr<QLineEdit>,
    pub command_palette_completer: MutPtr<QCompleter>,
    pub command_palette_completer_view: MutPtr<QTableView>,
    pub command_palette_completer_model: MutPtr<QStandardItemModel>,

    pub command_palette_show: MutPtr<QAction>,
    pub command_palette_hide: MutPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Main Window.
    //-------------------------------------------------------------------------------//
    pub main_window: MutPtr<QMainWindow>,
    pub tab_bar_packed_file: MutPtr<QTabWidget>,
    pub menu_bar: MutPtr<QMenuBar>,
    pub status_bar: MutPtr<QStatusBar>,

    //-------------------------------------------------------------------------------//
    // `MenuBar` menus.
    //-------------------------------------------------------------------------------//
    pub menu_bar_packfile: MutPtr<QMenu>,
    pub menu_bar_mymod: MutPtr<QMenu>,
    pub menu_bar_view: MutPtr<QMenu>,
    pub menu_bar_game_selected: MutPtr<QMenu>,
    pub menu_bar_special_stuff: MutPtr<QMenu>,
    pub menu_bar_about: MutPtr<QMenu>,
    pub menu_bar_debug: MutPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
    pub packfile_new_packfile: MutPtr<QAction>,
    pub packfile_open_packfile: MutPtr<QAction>,
    pub packfile_save_packfile: MutPtr<QAction>,
    pub packfile_save_packfile_as: MutPtr<QAction>,
    pub packfile_open_from_content: MutPtr<QMenu>,
    pub packfile_open_from_data: MutPtr<QMenu>,
    pub packfile_change_packfile_type: MutPtr<QMenu>,
    pub packfile_load_all_ca_packfiles: MutPtr<QAction>,
    pub packfile_load_template: MutPtr<QMenu>,
    pub packfile_preferences: MutPtr<QAction>,
    pub packfile_quit: MutPtr<QAction>,

    // "Change PackFile Type" submenu.
    pub change_packfile_type_boot: MutPtr<QAction>,
    pub change_packfile_type_release: MutPtr<QAction>,
    pub change_packfile_type_patch: MutPtr<QAction>,
    pub change_packfile_type_mod: MutPtr<QAction>,
    pub change_packfile_type_movie: MutPtr<QAction>,
    pub change_packfile_type_other: MutPtr<QAction>,

    pub change_packfile_type_header_is_extended: MutPtr<QAction>,
    pub change_packfile_type_index_includes_timestamp: MutPtr<QAction>,
    pub change_packfile_type_index_is_encrypted: MutPtr<QAction>,
    pub change_packfile_type_data_is_encrypted: MutPtr<QAction>,

    // Action to enable/disable compression on PackFiles. Only for PFH5+ PackFiles.
    pub change_packfile_type_data_is_compressed: MutPtr<QAction>,

    // Action Group for the submenu.
    pub change_packfile_type_group: MutPtr<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `MyMod` menu.
    //-------------------------------------------------------------------------------//
    pub mymod_new: MutPtr<QAction>,
    pub mymod_delete_selected: MutPtr<QAction>,
    pub mymod_install: MutPtr<QAction>,
    pub mymod_uninstall: MutPtr<QAction>,

    pub mymod_open_three_kingdoms: MutPtr<QMenu>,
    pub mymod_open_warhammer_2: MutPtr<QMenu>,
    pub mymod_open_warhammer: MutPtr<QMenu>,
    pub mymod_open_thrones_of_britannia: MutPtr<QMenu>,
    pub mymod_open_attila: MutPtr<QMenu>,
    pub mymod_open_rome_2: MutPtr<QMenu>,
    pub mymod_open_shogun_2: MutPtr<QMenu>,
    pub mymod_open_napoleon: MutPtr<QMenu>,
    pub mymod_open_empire: MutPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
    pub view_toggle_packfile_contents: MutPtr<QAction>,
    pub view_toggle_global_search_panel: MutPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    pub game_selected_launch_game: MutPtr<QAction>,

    pub game_selected_open_game_data_folder: MutPtr<QAction>,
    pub game_selected_open_game_assembly_kit_folder: MutPtr<QAction>,
    pub game_selected_open_config_folder: MutPtr<QAction>,

    pub game_selected_three_kingdoms: MutPtr<QAction>,
    pub game_selected_warhammer_2: MutPtr<QAction>,
    pub game_selected_warhammer: MutPtr<QAction>,
    pub game_selected_thrones_of_britannia: MutPtr<QAction>,
    pub game_selected_attila: MutPtr<QAction>,
    pub game_selected_rome_2: MutPtr<QAction>,
    pub game_selected_shogun_2: MutPtr<QAction>,
    pub game_selected_napoleon: MutPtr<QAction>,
    pub game_selected_empire: MutPtr<QAction>,
    pub game_selected_arena: MutPtr<QAction>,

    pub game_selected_group: MutPtr<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//

    // Three Kingdoms actions.
    pub special_stuff_three_k_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_three_k_optimize_packfile: MutPtr<QAction>,

    // Warhammer 2's actions.
    pub special_stuff_wh2_create_dummy_animpack: MutPtr<QAction>,
    pub special_stuff_wh2_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_wh2_optimize_packfile: MutPtr<QAction>,
    pub special_stuff_wh2_patch_siege_ai: MutPtr<QAction>,

    // Warhammer's actions.
    pub special_stuff_wh_create_dummy_animpack: MutPtr<QAction>,
    pub special_stuff_wh_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_wh_optimize_packfile: MutPtr<QAction>,
    pub special_stuff_wh_patch_siege_ai: MutPtr<QAction>,

    // Thrones of Britannia's actions.
    pub special_stuff_tob_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_tob_optimize_packfile: MutPtr<QAction>,

    // Attila's actions.
    pub special_stuff_att_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_att_optimize_packfile: MutPtr<QAction>,

    // Rome 2's actions.
    pub special_stuff_rom2_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_rom2_optimize_packfile: MutPtr<QAction>,

    // Shogun 2's actions.
    pub special_stuff_sho2_generate_pak_file: MutPtr<QAction>,
    pub special_stuff_sho2_optimize_packfile: MutPtr<QAction>,

    // Napoleon's actions.
    pub special_stuff_nap_optimize_packfile: MutPtr<QAction>,

    // Empire's actions.
    pub special_stuff_emp_optimize_packfile: MutPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    pub about_about_qt: MutPtr<QAction>,
    pub about_about_rpfm: MutPtr<QAction>,
    pub about_open_manual: MutPtr<QAction>,
    pub about_patreon_link: MutPtr<QAction>,
    pub about_check_updates: MutPtr<QAction>,
    pub about_check_schema_updates: MutPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // "Debug" menu.
    //-------------------------------------------------------------------------------//
    pub debug_update_current_schema_from_asskit: MutPtr<QAction>,
    pub debug_generate_schema_diff: MutPtr<QAction>,
}

/// This enum contains the data needed to create a new PackedFile.
#[derive(Clone, Debug)]
pub enum NewPackedFile {

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
        let mut main_window = QMainWindow::new_0a().into_ptr();
        let widget = QWidget::new_0a().into_ptr();
        let mut layout = create_grid_layout(widget);
        main_window.set_central_widget(widget);
        main_window.resize_2a(1100, 400);
        QApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/img/rpfm.png", ASSETS_PATH.to_string_lossy()))));

        // Get the menu and status bars.
        let mut menu_bar = main_window.menu_bar();
        let status_bar = main_window.status_bar();
        let mut tab_bar_packed_file = QTabWidget::new_0a();
        tab_bar_packed_file.set_tabs_closable(true);
        tab_bar_packed_file.set_movable(true);
        layout.add_widget_5a(&mut tab_bar_packed_file, 0, 0, 1, 1);
        STATUS_BAR.store(status_bar.as_mut_raw_ptr(), Ordering::SeqCst);

        //-----------------------------------------------//
        // `Command Palette` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'Command Palette` Dock Widget and all his contents.
        let command_palette_window_flags = QFlags::from(WindowType::Popup);
        let mut command_palette_widget = QDockWidget::from_q_widget_q_flags_window_type(main_window, command_palette_window_flags);
        let command_palette_inner_widget = QWidget::new_0a().into_ptr();
        let mut command_palette_layout = create_grid_layout(command_palette_inner_widget);
        command_palette_widget.set_widget(command_palette_inner_widget);
        command_palette_widget.set_features(QFlags::from(DockWidgetFeature::NoDockWidgetFeatures));
        command_palette_widget.set_minimum_width(500);

        // Create and configure the `Command Palette` itself.
        let mut command_palette_line_edit = QLineEdit::new();
        let mut command_palette_completer = QCompleter::new();
        let mut command_palette_completer_view = new_tableview_command_palette_safe();
        let mut command_palette_completer_model = QStandardItemModel::new_0a();

        // This means our completer search with case-insensitive and contains filters.
        command_palette_completer.set_filter_mode(QFlags::from(MatchFlag::MatchContains));
        command_palette_completer.set_case_sensitivity(CaseSensitivity::CaseInsensitive);
        command_palette_completer.set_max_visible_items(8);

        command_palette_completer_view.set_show_grid(false);
        command_palette_completer_view.set_selection_behavior(SelectionBehavior::SelectRows);
        command_palette_completer_view.horizontal_header().hide();
        command_palette_completer_view.vertical_header().hide();

        command_palette_completer.set_popup(command_palette_completer_view);
        command_palette_completer.set_model(&mut command_palette_completer_model);
        command_palette_layout.add_widget_5a(&mut command_palette_line_edit, 0, 0, 1, 1);

        // Create the actions needed to show/hide the `Command Palette`.
        let command_palette_show = QAction::new();
        let command_palette_hide = QAction::new();

        //-----------------------------------------------//
        // Menu bar.
        //-----------------------------------------------//

        // Create the `MenuBar` menus.
        let mut menu_bar_packfile = menu_bar.add_menu_q_string(&qtr("menu_bar_packfile"));
        let mut menu_bar_mymod = menu_bar.add_menu_q_string(&qtr("menu_bar_mymod"));
        let mut menu_bar_view = menu_bar.add_menu_q_string(&qtr("menu_bar_view"));
        let mut menu_bar_game_selected = menu_bar.add_menu_q_string(&qtr("menu_bar_game_selected"));
        let mut menu_bar_special_stuff = menu_bar.add_menu_q_string(&qtr("menu_bar_special_stuff"));
        let mut menu_bar_about = menu_bar.add_menu_q_string(&qtr("menu_bar_about"));

        // This menu is hidden unless you enable it.
        let mut menu_bar_debug = menu_bar.add_menu_q_string(&qtr("menu_bar_debug"));
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
        let packfile_menu_open_from_content = QMenu::from_q_string(&qtr("open_from_content")).into_ptr();
        let packfile_menu_open_from_data = QMenu::from_q_string(&qtr("open_from_data")).into_ptr();
        let mut packfile_menu_change_packfile_type = QMenu::from_q_string(&qtr("change_packfile_type")).into_ptr();
        let packfile_load_all_ca_packfiles = menu_bar_packfile.add_action_q_string(&qtr("load_all_ca_packfiles"));
        let packfile_menu_load_template = QMenu::from_q_string(&qtr("load_template")).into_ptr();
        let packfile_preferences = menu_bar_packfile.add_action_q_string(&qtr("preferences"));
        let packfile_quit = menu_bar_packfile.add_action_q_string(&qtr("quit"));

        // Add the "Open..." submenus. These needs to be here because they have to be inserted in specific positions of the menu.
        menu_bar_packfile.insert_menu(packfile_load_all_ca_packfiles, packfile_menu_open_from_content);
        menu_bar_packfile.insert_menu(packfile_load_all_ca_packfiles, packfile_menu_open_from_data);

        menu_bar_packfile.insert_separator(packfile_menu_open_from_content.menu_action());
        menu_bar_packfile.insert_separator(packfile_preferences);
        menu_bar_packfile.insert_menu(packfile_preferences, packfile_menu_change_packfile_type);
        menu_bar_packfile.insert_menu(packfile_preferences, packfile_menu_load_template);
        menu_bar_packfile.insert_separator(packfile_preferences);

        // `Change PackFile Type` submenu.
        let mut change_packfile_type_boot = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_boot"));
        let mut change_packfile_type_release = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_release"));
        let mut change_packfile_type_patch = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_patch"));
        let mut change_packfile_type_mod = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_mod"));
        let mut change_packfile_type_movie = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_movie"));
        let mut change_packfile_type_other = packfile_menu_change_packfile_type.add_action_q_string(&qtr("packfile_type_other"));
        let mut change_packfile_type_header_is_extended = packfile_menu_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_header_is_extended"));
        let mut change_packfile_type_index_includes_timestamp = packfile_menu_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_includes_timestamp"));
        let mut change_packfile_type_index_is_encrypted = packfile_menu_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_is_encrypted"));
        let mut change_packfile_type_data_is_encrypted = packfile_menu_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_data_is_encrypted"));
        let mut change_packfile_type_data_is_compressed = packfile_menu_change_packfile_type.add_action_q_string(&qtr("change_packfile_type_data_is_compressed"));

        let mut change_packfile_type_group = QActionGroup::new(packfile_menu_change_packfile_type);

        // Configure the `PackFile` menu and his submenu.
        change_packfile_type_group.add_action_q_action(change_packfile_type_boot);
        change_packfile_type_group.add_action_q_action(change_packfile_type_release);
        change_packfile_type_group.add_action_q_action(change_packfile_type_patch);
        change_packfile_type_group.add_action_q_action(change_packfile_type_mod);
        change_packfile_type_group.add_action_q_action(change_packfile_type_movie);
        change_packfile_type_group.add_action_q_action(change_packfile_type_other);
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
        packfile_menu_change_packfile_type.insert_separator(change_packfile_type_other);
        packfile_menu_change_packfile_type.insert_separator(change_packfile_type_header_is_extended);
        packfile_menu_change_packfile_type.insert_separator(change_packfile_type_data_is_compressed);

        //-----------------------------------------------//
        // `MyMod` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let mut mymod_new = menu_bar_mymod.add_action_q_string(&qtr("mymod_new"));
        let mut mymod_delete_selected = menu_bar_mymod.add_action_q_string(&qtr("mymod_delete_selected"));
        let mut mymod_install = menu_bar_mymod.add_action_q_string(&qtr("mymod_install"));
        let mut mymod_uninstall = menu_bar_mymod.add_action_q_string(&qtr("mymod_uninstall"));

        menu_bar_mymod.add_separator();

        let mymod_open_three_kingdoms = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_THREE_KINGDOMS));
        let mymod_open_warhammer_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER_2));
        let mymod_open_warhammer = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER));
        let mymod_open_thrones_of_britannia = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_THRONES_OF_BRITANNIA));
        let mymod_open_attila = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_ATTILA));
        let mymod_open_rome_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_ROME_2));
        let mymod_open_shogun_2 = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_SHOGUN_2));
        let mymod_open_napoleon = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_NAPOLEON));
        let mymod_open_empire = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_EMPIRE));

        menu_bar_mymod.insert_separator(mymod_install);

        // Disable all the Contextual Menu actions by default.
        mymod_new.set_enabled(false);
        mymod_delete_selected.set_enabled(false);
        mymod_install.set_enabled(false);
        mymod_uninstall.set_enabled(false);

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

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let game_selected_launch_game = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_launch_game"));

        let game_selected_open_game_data_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_game_data_folder"));
        let game_selected_open_game_assembly_kit_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_game_assembly_kit_folder"));
        let game_selected_open_config_folder = menu_bar_game_selected.add_action_q_string(&qtr("game_selected_open_config_folder"));

        let mut game_selected_three_kingdoms = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_THREE_KINGDOMS));
        let mut game_selected_warhammer_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER_2));
        let mut game_selected_warhammer = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER));
        let mut game_selected_thrones_of_britannia = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_THRONES_OF_BRITANNIA));
        let mut game_selected_attila = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_ATTILA));
        let mut game_selected_rome_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_ROME_2));
        let mut game_selected_shogun_2 = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_SHOGUN_2));
        let mut game_selected_napoleon = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_NAPOLEON));
        let mut game_selected_empire = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_EMPIRE));
        let mut game_selected_arena = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(GAME_SELECTED_ARENA));

        game_selected_three_kingdoms.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_THREE_KINGDOMS).unwrap().game_selected_icon))).as_ref());
        game_selected_warhammer_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_WARHAMMER_2).unwrap().game_selected_icon))).as_ref());
        game_selected_warhammer.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_WARHAMMER).unwrap().game_selected_icon))).as_ref());
        game_selected_thrones_of_britannia.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_THRONES_OF_BRITANNIA).unwrap().game_selected_icon))).as_ref());
        game_selected_attila.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_ATTILA).unwrap().game_selected_icon))).as_ref());
        game_selected_rome_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_ROME_2).unwrap().game_selected_icon))).as_ref());
        game_selected_shogun_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_SHOGUN_2).unwrap().game_selected_icon))).as_ref());
        game_selected_napoleon.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_NAPOLEON).unwrap().game_selected_icon))).as_ref());
        game_selected_empire.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_EMPIRE).unwrap().game_selected_icon))).as_ref());
        game_selected_arena.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}", ASSETS_PATH.to_string_lossy().to_string(), SUPPORTED_GAMES.get(KEY_ARENA).unwrap().game_selected_icon))).as_ref());

        let mut game_selected_group = QActionGroup::new(menu_bar_game_selected);

        // Configure the `Game Selected` Menu.
        menu_bar_game_selected.insert_separator(game_selected_three_kingdoms);
        menu_bar_game_selected.insert_separator(game_selected_arena);
        game_selected_group.add_action_q_action(game_selected_three_kingdoms);
        game_selected_group.add_action_q_action(game_selected_warhammer_2);
        game_selected_group.add_action_q_action(game_selected_warhammer);
        game_selected_group.add_action_q_action(game_selected_thrones_of_britannia);
        game_selected_group.add_action_q_action(game_selected_attila);
        game_selected_group.add_action_q_action(game_selected_rome_2);
        game_selected_group.add_action_q_action(game_selected_shogun_2);
        game_selected_group.add_action_q_action(game_selected_napoleon);
        game_selected_group.add_action_q_action(game_selected_empire);
        game_selected_group.add_action_q_action(game_selected_arena);
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
        let mut menu_three_kingdoms = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_THREE_KINGDOMS));
        let mut menu_warhammer_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER_2));
        let mut menu_warhammer = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_WARHAMMER));
        let mut menu_thrones_of_britannia = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_THRONES_OF_BRITANNIA));
        let mut menu_attila = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_ATTILA));
        let mut menu_rome_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_ROME_2));
        let mut menu_shogun_2 = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_SHOGUN_2));
        let mut menu_napoleon = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_NAPOLEON));
        let mut menu_empire = menu_bar_special_stuff.add_menu_q_string(&QString::from_std_str(GAME_SELECTED_EMPIRE));

        // Populate the `Special Stuff` submenus.
        let special_stuff_three_k_generate_pak_file = menu_three_kingdoms.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_three_k_optimize_packfile = menu_three_kingdoms.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh2_create_dummy_animpack = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_create_dummy_animpack"));
        let special_stuff_wh2_generate_pak_file = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_wh2_optimize_packfile = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh2_patch_siege_ai = menu_warhammer_2.add_action_q_string(&qtr("special_stuff_patch_siege_ai"));
        let special_stuff_wh_create_dummy_animpack = menu_warhammer.add_action_q_string(&qtr("special_stuff_create_dummy_animpack"));
        let special_stuff_wh_generate_pak_file = menu_warhammer.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_wh_optimize_packfile = menu_warhammer.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_wh_patch_siege_ai = menu_warhammer.add_action_q_string(&qtr("special_stuff_patch_siege_ai"));
        let special_stuff_tob_generate_pak_file = menu_thrones_of_britannia.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_tob_optimize_packfile = menu_thrones_of_britannia.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_att_generate_pak_file = menu_attila.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_att_optimize_packfile = menu_attila.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_rom2_generate_pak_file = menu_rome_2.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_rom2_optimize_packfile = menu_rome_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_sho2_generate_pak_file = menu_shogun_2.add_action_q_string(&qtr("special_stuff_generate_pak_file"));
        let special_stuff_sho2_optimize_packfile = menu_shogun_2.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_nap_optimize_packfile = menu_napoleon.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let special_stuff_emp_optimize_packfile = menu_empire.add_action_q_string(&qtr("special_stuff_optimize_packfile"));

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

        //-----------------------------------------------//
        // `Debug` Menu.
        //-----------------------------------------------//

        // Populate the `Debug` menu.
        let debug_update_current_schema_from_asskit = menu_bar_debug.add_action_q_string(&qtr("update_current_schema_from_asskit"));
        let debug_generate_schema_diff = menu_bar_debug.add_action_q_string(&qtr("generate_schema_diff"));

        command_palette_widget.hide();

        // Create ***Da monsta***.
        AppUI {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,
            tab_bar_packed_file: tab_bar_packed_file.into_ptr(),
            menu_bar,
            status_bar,

            //-------------------------------------------------------------------------------//
            // `Command Palette` DockWidget.
            //-------------------------------------------------------------------------------//
            command_palette: command_palette_widget.into_ptr(),
            command_palette_line_edit: command_palette_line_edit.into_ptr(),
            command_palette_completer: command_palette_completer.into_ptr(),
            command_palette_completer_view,
            command_palette_completer_model: command_palette_completer_model.into_ptr(),

            command_palette_show: command_palette_show.into_ptr(),
            command_palette_hide: command_palette_hide.into_ptr(),

            //-------------------------------------------------------------------------------//
            // `MenuBar` menus.
            //-------------------------------------------------------------------------------//
            menu_bar_packfile,
            menu_bar_mymod,
            menu_bar_view,
            menu_bar_game_selected,
            menu_bar_special_stuff,
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
            packfile_open_from_content: packfile_menu_open_from_content,
            packfile_open_from_data: packfile_menu_open_from_data,
            packfile_change_packfile_type: packfile_menu_change_packfile_type,
            packfile_load_all_ca_packfiles,
            packfile_load_template: packfile_menu_load_template,
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
            change_packfile_type_group: change_packfile_type_group.into_ptr(),

            //-------------------------------------------------------------------------------//
            // `MyMod` menu.
            //-------------------------------------------------------------------------------//
            mymod_new,
            mymod_delete_selected,
            mymod_install,
            mymod_uninstall,

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

            //-------------------------------------------------------------------------------//
            // "Game Selected" menu.
            //-------------------------------------------------------------------------------//
            game_selected_launch_game,

            game_selected_open_game_data_folder,
            game_selected_open_game_assembly_kit_folder,
            game_selected_open_config_folder,

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

            game_selected_group: game_selected_group.into_ptr(),

            //-------------------------------------------------------------------------------//
            // "Special Stuff" menu.
            //-------------------------------------------------------------------------------//

            // Three Kingdoms actions.
            special_stuff_three_k_generate_pak_file,
            special_stuff_three_k_optimize_packfile,

            // Warhammer 2's actions.
            special_stuff_wh2_create_dummy_animpack,
            special_stuff_wh2_generate_pak_file,
            special_stuff_wh2_optimize_packfile,
            special_stuff_wh2_patch_siege_ai,

            // Warhammer's actions.
            special_stuff_wh_create_dummy_animpack,
            special_stuff_wh_generate_pak_file,
            special_stuff_wh_optimize_packfile,
            special_stuff_wh_patch_siege_ai,

            // Thrones of Britannia's actions.
            special_stuff_tob_generate_pak_file,
            special_stuff_tob_optimize_packfile,

            // Attila's actions.
            special_stuff_att_generate_pak_file,
            special_stuff_att_optimize_packfile,

            // Rome 2's actions.
            special_stuff_rom2_generate_pak_file,
            special_stuff_rom2_optimize_packfile,

            // Shogun 2's actions.
            special_stuff_sho2_generate_pak_file,
            special_stuff_sho2_optimize_packfile,

            // Napoleon's actions.
            special_stuff_nap_optimize_packfile,

            // Empire's actions.
            special_stuff_emp_optimize_packfile,

            //-------------------------------------------------------------------------------//
            // "About" menu.
            //-------------------------------------------------------------------------------//
            about_about_qt,
            about_about_rpfm,
            about_open_manual,
            about_patreon_link,
            about_check_updates,
            about_check_schema_updates,

            //-------------------------------------------------------------------------------//
            // "Debug" menu.
            //-------------------------------------------------------------------------------//
            debug_update_current_schema_from_asskit,
            debug_generate_schema_diff,
        }
    }
}
