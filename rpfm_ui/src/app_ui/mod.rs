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
Module with all the code related to the main `AppUI`.

This module contains all the code needed to initialize the main Window and its menus.
!*/

use qt_widgets::abstract_item_view::{AbstractItemView, SelectionBehavior};
use qt_widgets::action::Action;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::application::Application;
use qt_widgets::completer::Completer;
use qt_widgets::dock_widget::DockWidget;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::menu::Menu;
use qt_widgets::menu_bar::MenuBar;
use qt_widgets::status_bar::StatusBar;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::icon::Icon;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::flags::Flags;
use qt_core::object::Object;
use qt_core::qt::CaseSensitivity;

use crate::ffi::new_tableview_command_palette;
use crate::locale::tr;
use crate::QString;
use crate::RPFM_PATH;
use crate::utils::create_grid_layout_unsafe;

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
#[derive(Copy, Clone)]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // `Command Palette` DockWidget.
    //-------------------------------------------------------------------------------//
    pub command_palette: *mut DockWidget,
    pub command_palette_line_edit: *mut LineEdit,
    pub command_palette_completer: *mut Completer,
    pub command_palette_completer_view: *mut TableView,
    pub command_palette_completer_model: *mut StandardItemModel,

    pub command_palette_show: *mut Action,
    pub command_palette_hide: *mut Action,

    //-------------------------------------------------------------------------------//
    // Main Window.
    //-------------------------------------------------------------------------------//
    pub main_window: *mut MainWindow,
    pub menu_bar: *mut MenuBar,
    pub status_bar: *mut StatusBar,

    //-------------------------------------------------------------------------------//
    // `MenuBar` menus.
    //-------------------------------------------------------------------------------//
    pub menu_bar_packfile: *mut Menu,
    pub menu_bar_mymod: *mut Menu,
    pub menu_bar_view: *mut Menu,
    pub menu_bar_game_seleted: *mut Menu,
    pub menu_bar_special_stuff: *mut Menu,
    pub menu_bar_about: *mut Menu,

    //-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
    pub packfile_new_packfile: *mut Action,
    pub packfile_open_packfile: *mut Action,
    pub packfile_save_packfile: *mut Action,
    pub packfile_save_packfile_as: *mut Action,
    pub packfile_open_from_content: *mut Menu,
    pub packfile_open_from_data: *mut Menu,
    pub packfile_change_packfile_type: *mut Menu,
    pub packfile_load_all_ca_packfiles: *mut Action,
    pub packfile_preferences: *mut Action,
    pub packfile_quit: *mut Action,

    // "Change PackFile Type" submenu.
    pub change_packfile_type_boot: *mut Action,
    pub change_packfile_type_release: *mut Action,
    pub change_packfile_type_patch: *mut Action,
    pub change_packfile_type_mod: *mut Action,
    pub change_packfile_type_movie: *mut Action,
    pub change_packfile_type_other: *mut Action,

    pub change_packfile_type_header_is_extended: *mut Action,
    pub change_packfile_type_index_includes_timestamp: *mut Action,
    pub change_packfile_type_index_is_encrypted: *mut Action,
    pub change_packfile_type_data_is_encrypted: *mut Action,

    // Action to enable/disable compression on PackFiles. Only for PFH5+ PackFiles.
    pub change_packfile_type_data_is_compressed: *mut Action,

    // Action Group for the submenu.
    pub change_packfile_type_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // `MyMod` menu.
    //-------------------------------------------------------------------------------//
    pub mymod_new: *mut Action,
    pub mymod_delete_selected: *mut Action,
    pub mymod_install: *mut Action,
    pub mymod_uninstall: *mut Action,
    pub mymod_open_separator: *mut Action,

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
    pub view_toggle_packfile_contents: *mut Action,
    pub view_toggle_global_search_panel: *mut Action,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    pub game_selected_open_game_data_folder: *mut Action,
    pub game_selected_open_game_assembly_kit_folder: *mut Action,

    pub game_selected_three_kingdoms: *mut Action,
    pub game_selected_warhammer_2: *mut Action,
    pub game_selected_warhammer: *mut Action,
    pub game_selected_thrones_of_britannia: *mut Action,
    pub game_selected_attila: *mut Action,
    pub game_selected_rome_2: *mut Action,
    pub game_selected_shogun_2: *mut Action,
    pub game_selected_napoleon: *mut Action,
    pub game_selected_empire: *mut Action,
    pub game_selected_arena: *mut Action,

    pub game_selected_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//

    // Three Kingdoms actions.
    pub special_stuff_three_k_generate_pak_file: *mut Action,
    pub special_stuff_three_k_optimize_packfile: *mut Action,

    // Warhammer 2's actions.
    pub special_stuff_wh2_generate_pak_file: *mut Action,
    pub special_stuff_wh2_optimize_packfile: *mut Action,
    pub special_stuff_wh2_patch_siege_ai: *mut Action,

    // Warhammer's actions.
    pub special_stuff_wh_generate_pak_file: *mut Action,
    pub special_stuff_wh_optimize_packfile: *mut Action,
    pub special_stuff_wh_patch_siege_ai: *mut Action,

    // Thrones of Britannia's actions.
    pub special_stuff_tob_generate_pak_file: *mut Action,
    pub special_stuff_tob_optimize_packfile: *mut Action,

    // Attila's actions.
    pub special_stuff_att_generate_pak_file: *mut Action,
    pub special_stuff_att_optimize_packfile: *mut Action,

    // Rome 2's actions.
    pub special_stuff_rom2_generate_pak_file: *mut Action,
    pub special_stuff_rom2_optimize_packfile: *mut Action,

    // Shogun 2's actions.
    pub special_stuff_sho2_generate_pak_file: *mut Action,
    pub special_stuff_sho2_optimize_packfile: *mut Action,

    // Napoleon's actions.
    pub special_stuff_nap_optimize_packfile: *mut Action,

    // Empire's actions.
    pub special_stuff_emp_optimize_packfile: *mut Action,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    pub about_about_qt: *mut Action,
    pub about_about_rpfm: *mut Action,
    pub about_open_manual: *mut Action,
    pub about_patreon_link: *mut Action,
    pub about_check_updates: *mut Action,
    pub about_check_schema_updates: *mut Action,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `AppUI`.
impl Default for AppUI {

    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    fn default() -> Self {

        // Initialize and configure the main window.
        let mut main_window = MainWindow::new();
        let widget = Widget::new();
        unsafe { main_window.set_central_widget(widget.into_raw()); }
        main_window.resize((1100, 400));
        Application::set_window_icon(&Icon::new(&QString::from_std_str(format!("{}/img/rpfm.png", RPFM_PATH.to_string_lossy()))));

        // Get the menu and status bars.
        let menu_bar = main_window.menu_bar();
        let status_bar = main_window.status_bar();

        //-----------------------------------------------//
        // `Command Palette` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'Command Palette` Dock Widget and all his contents.
        let command_palette_window_flags = Flags::from_int(8);
        let mut command_palette_widget = unsafe { DockWidget::new_unsafe((main_window.as_mut_ptr() as *mut Widget, command_palette_window_flags)) };
        let command_palette_inner_widget = Widget::new();
        let command_palette_layout = create_grid_layout_unsafe(command_palette_inner_widget.as_mut_ptr() as *mut Widget);
        unsafe { command_palette_widget.set_widget(command_palette_inner_widget.into_raw()); }
        command_palette_widget.set_features(Flags::from_int(0));
        command_palette_widget.set_minimum_width(500);

        // Create and configure the `Command Palette` itself.
        let command_palette_line_edit = LineEdit::new(());
        let mut command_palette_completer = Completer::new(());
        let command_palette_completer_view = unsafe { new_tableview_command_palette() };
        let command_palette_completer_model = StandardItemModel::new(());

        // This means our completer search with case-insensitive and contains filters.
        command_palette_completer.set_filter_mode(Flags::from_int(1));
        command_palette_completer.set_case_sensitivity(CaseSensitivity::Insensitive);
        command_palette_completer.set_max_visible_items(8);

        unsafe { command_palette_completer_view.as_mut().unwrap().set_show_grid(false); }
        unsafe { command_palette_completer_view.as_mut().unwrap().set_selection_behavior(SelectionBehavior::Rows); }
        unsafe { command_palette_completer_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().hide(); }
        unsafe { command_palette_completer_view.as_mut().unwrap().vertical_header().as_mut().unwrap().hide(); }

        unsafe { command_palette_completer.set_popup(command_palette_completer_view as *mut AbstractItemView); }
        unsafe { command_palette_completer.set_model(command_palette_completer_model.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { command_palette_layout.as_mut().unwrap().add_widget((command_palette_line_edit.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }

        // Create the actions needed to show/hide the `Command Palette`.
        let command_palette_show = Action::new(());
        let command_palette_hide = Action::new(());

        //-----------------------------------------------//
        // Menu bar.
        //-----------------------------------------------//

        // Create the `MenuBar` menus.
        let menu_bar_ref_mut = unsafe { menu_bar.as_mut().unwrap() };
        let menu_bar_packfile = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-packfile")));
        let menu_bar_mymod = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-mymod")));
        let menu_bar_view = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-view")));
        let menu_bar_game_seleted = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-game-selected")));
        let menu_bar_special_stuff = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-special-stuff")));
        let menu_bar_about = menu_bar_ref_mut.add_menu(&QString::from_std_str(tr("menu-bar-about")));

        //-----------------------------------------------//
        // `PackFile` Menu.
        //-----------------------------------------------//

        // Populate the `PackFile` menu.
        let menu_bar_packfile_ref_mut = unsafe { menu_bar_packfile.as_mut().unwrap() };
        let packfile_new_packfile = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&New PackFile"));
        let packfile_open_packfile = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&Open PackFile"));
        let packfile_save_packfile = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&Save PackFile"));
        let packfile_save_packfile_as = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("Save PackFile &As..."));
        let mut packfile_menu_open_from_content = Menu::new(&QString::from_std_str("Open From Content"));
        let mut packfile_menu_open_from_data = Menu::new(&QString::from_std_str("Open From Data"));
        let mut packfile_menu_change_packfile_type = Menu::new(&QString::from_std_str("&Change PackFile Type"));
        let packfile_load_all_ca_packfiles = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&Load All CA PackFiles"));
        let packfile_preferences = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&Preferences"));
        let packfile_quit = menu_bar_packfile_ref_mut.add_action(&QString::from_std_str("&Quit"));

        // Add the "Open..." submenus. These needs to be here because they have to be inserted in specific positions of the menu.
        unsafe { menu_bar_packfile_ref_mut.insert_menu(packfile_load_all_ca_packfiles, packfile_menu_open_from_content.as_mut_ptr()); }
        unsafe { menu_bar_packfile_ref_mut.insert_menu(packfile_load_all_ca_packfiles, packfile_menu_open_from_data.as_mut_ptr()); }

        unsafe { menu_bar_packfile_ref_mut.insert_separator(packfile_menu_open_from_content.menu_action()); }
        unsafe { menu_bar_packfile_ref_mut.insert_separator(packfile_preferences); }
        unsafe { menu_bar_packfile_ref_mut.insert_menu(packfile_preferences, packfile_menu_change_packfile_type.as_mut_ptr()); }
        unsafe { menu_bar_packfile_ref_mut.insert_separator(packfile_preferences); }

        // `Change PackFile Type` submenu.
        let change_packfile_type_boot = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Boot"));
        let change_packfile_type_release = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Release"));
        let change_packfile_type_patch = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Patch"));
        let change_packfile_type_mod = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Mod"));
        let change_packfile_type_movie = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("Mo&vie"));
        let change_packfile_type_other = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Other"));
        let change_packfile_type_header_is_extended = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Header Is Extended"));
        let change_packfile_type_index_includes_timestamp = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Index Includes Timestamp"));
        let change_packfile_type_index_is_encrypted = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("Index Is &Encrypted"));
        let change_packfile_type_data_is_encrypted = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("&Data Is Encrypted"));
        let change_packfile_type_data_is_compressed = packfile_menu_change_packfile_type.add_action(&QString::from_std_str("Data Is &Compressed"));

        let mut change_packfile_type_group = unsafe { ActionGroup::new(packfile_menu_change_packfile_type.as_mut_ptr() as *mut Object) };

        // Configure the `PackFile` menu and his submenu.
        packfile_menu_open_from_content.set_enabled(false);
        packfile_menu_open_from_data.set_enabled(false);

        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_boot); }
        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_release); }
        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_patch); }
        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_mod); }
        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_movie); }
        unsafe { change_packfile_type_group.add_action_unsafe(change_packfile_type_other); }
        unsafe { change_packfile_type_boot.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_release.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_patch.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_mod.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_movie.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_other.as_mut().unwrap().set_checkable(true); }

        // These ones are individual, but they need to be checkable and not editable.
        unsafe { change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_header_is_extended.as_mut().unwrap().set_checkable(true); }
        unsafe { change_packfile_type_data_is_compressed.as_mut().unwrap().set_checkable(true); }

        unsafe { change_packfile_type_data_is_encrypted.as_mut().unwrap().set_enabled(false); }
        unsafe { change_packfile_type_index_is_encrypted.as_mut().unwrap().set_enabled(false); }
        unsafe { change_packfile_type_header_is_extended.as_mut().unwrap().set_enabled(false); }
        unsafe { change_packfile_type_data_is_compressed.as_mut().unwrap().set_enabled(false); }

        // Put separators in the SubMenu.
        unsafe { packfile_menu_change_packfile_type.insert_separator(change_packfile_type_other); }
        unsafe { packfile_menu_change_packfile_type.insert_separator(change_packfile_type_header_is_extended); }
        unsafe { packfile_menu_change_packfile_type.insert_separator(change_packfile_type_data_is_compressed); }

        //-----------------------------------------------//
        // `MyMod` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let menu_bar_mymod_ref_mut = unsafe { menu_bar_mymod.as_mut().unwrap() };
        let mymod_new = menu_bar_mymod_ref_mut.add_action(&QString::from_std_str("&New MyMod"));
        let mymod_delete_selected = menu_bar_mymod_ref_mut.add_action(&QString::from_std_str("&Delete Selected MyMod"));
        let mymod_install = menu_bar_mymod_ref_mut.add_action(&QString::from_std_str("&Install"));
        let mymod_uninstall = menu_bar_mymod_ref_mut.add_action(&QString::from_std_str("&Uninstall"));
        let mymod_open_separator = menu_bar_mymod_ref_mut.add_separator();

        unsafe { menu_bar_mymod_ref_mut.insert_separator(mymod_install); }

        // Disable all the Contextual Menu actions by default.
        unsafe { mymod_new.as_mut().unwrap().set_enabled(false); }
        unsafe { mymod_delete_selected.as_mut().unwrap().set_enabled(false); }
        unsafe { mymod_install.as_mut().unwrap().set_enabled(false); }
        unsafe { mymod_uninstall.as_mut().unwrap().set_enabled(false); }

        //-----------------------------------------------//
        // `View` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let menu_bar_view_ref_mut = unsafe { menu_bar_view.as_mut().unwrap() };
        let view_toggle_packfile_contents = menu_bar_view_ref_mut.add_action(&QString::from_std_str("Toggle &PackFile Contents"));
        let view_toggle_global_search_panel = menu_bar_view_ref_mut.add_action(&QString::from_std_str("Toggle Global Search Window"));

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//

        // Populate the `Game Selected` menu.
        let menu_bar_game_seleted_ref_mut = unsafe { menu_bar_game_seleted.as_mut().unwrap() };
        let game_selected_open_game_data_folder = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Open Game's Data Folder"));
        let game_selected_open_game_assembly_kit_folder = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("Open &Game's Assembly Kit Folder"));

        let game_selected_three_kingdoms = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("Three &Kingdoms"));
        let game_selected_warhammer_2 = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Warhammer 2"));
        let game_selected_warhammer = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("War&hammer"));
        let game_selected_thrones_of_britannia = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Thrones of Britannia"));
        let game_selected_attila = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Attila"));
        let game_selected_rome_2 = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("R&ome 2"));
        let game_selected_shogun_2 = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Shogun 2"));
        let game_selected_napoleon = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Napoleon"));
        let game_selected_empire = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("&Empire"));
        let game_selected_arena = menu_bar_game_seleted_ref_mut.add_action(&QString::from_std_str("A&rena"));

        let mut game_selected_group = unsafe { ActionGroup::new(menu_bar_game_seleted as *mut Object) };

        // Configure the `Game Selected` Menu.
        unsafe { menu_bar_game_seleted_ref_mut.insert_separator(game_selected_three_kingdoms); }
        unsafe { menu_bar_game_seleted_ref_mut.insert_separator(game_selected_arena); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_three_kingdoms); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_warhammer_2); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_warhammer); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_thrones_of_britannia); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_attila); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_rome_2); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_shogun_2); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_napoleon); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_empire); }
        unsafe { game_selected_group.add_action_unsafe(game_selected_arena); }
        unsafe { game_selected_three_kingdoms.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_warhammer_2.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_warhammer.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_thrones_of_britannia.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_attila.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_rome_2.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_shogun_2.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_napoleon.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_empire.as_mut().unwrap().set_checkable(true); }
        unsafe { game_selected_arena.as_mut().unwrap().set_checkable(true); }

        //-----------------------------------------------//
        // `Special Stuff` Menu.
        //-----------------------------------------------//

        // Populate the `Special Stuff` menu with submenus.
        let menu_bar_special_stuff_ref_mut = unsafe { menu_bar_special_stuff.as_mut().unwrap() };
        let menu_three_kingdoms = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("Three &Kingdoms"));
        let menu_warhammer_2 = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Warhammer 2"));
        let menu_warhammer = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("War&hammer"));
        let menu_thrones_of_britannia = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Thrones of Britannia"));
        let menu_attila = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Attila"));
        let menu_rome_2 = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Rome 2"));
        let menu_shogun_2 = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Shogun 2"));
        let menu_napoleon = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Napoleon"));
        let menu_empire = menu_bar_special_stuff_ref_mut.add_menu(&QString::from_std_str("&Empire"));

        // Populate the `Special Stuff` submenus.
        let menu_three_kingdoms_ref_mut = unsafe { menu_three_kingdoms.as_mut().unwrap() };
        let menu_warhammer_2_ref_mut = unsafe { menu_warhammer_2.as_mut().unwrap() };
        let menu_warhammer_ref_mut = unsafe { menu_warhammer.as_mut().unwrap() };
        let menu_thrones_of_britannia_ref_mut = unsafe { menu_thrones_of_britannia.as_mut().unwrap() };
        let menu_attila_ref_mut = unsafe { menu_attila.as_mut().unwrap() };
        let menu_rome_2_ref_mut = unsafe { menu_rome_2.as_mut().unwrap() };
        let menu_shogun_2_ref_mut = unsafe { menu_shogun_2.as_mut().unwrap() };
        let menu_napoleon_ref_mut = unsafe { menu_napoleon.as_mut().unwrap() };
        let menu_empire_ref_mut = unsafe { menu_empire.as_mut().unwrap() };

        let special_stuff_three_k_generate_pak_file = menu_three_kingdoms_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_three_k_optimize_packfile = menu_three_kingdoms_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_wh2_generate_pak_file = menu_warhammer_2_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_wh2_optimize_packfile = menu_warhammer_2_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_wh2_patch_siege_ai = menu_warhammer_2_ref_mut.add_action(&QString::from_std_str("&Patch Siege AI"));
        let special_stuff_wh_generate_pak_file = menu_warhammer_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_wh_optimize_packfile = menu_warhammer_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_wh_patch_siege_ai = menu_warhammer_ref_mut.add_action(&QString::from_std_str("&Patch Siege AI"));
        let special_stuff_tob_generate_pak_file = menu_thrones_of_britannia_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_tob_optimize_packfile = menu_thrones_of_britannia_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_att_generate_pak_file = menu_attila_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_att_optimize_packfile = menu_attila_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_rom2_generate_pak_file = menu_rome_2_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_rom2_optimize_packfile = menu_rome_2_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_sho2_generate_pak_file = menu_shogun_2_ref_mut.add_action(&QString::from_std_str("&Generate PAK File"));
        let special_stuff_sho2_optimize_packfile = menu_shogun_2_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_nap_optimize_packfile = menu_napoleon_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));
        let special_stuff_emp_optimize_packfile = menu_empire_ref_mut.add_action(&QString::from_std_str("&Optimize PackFile"));

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//

        // Populate the `About` menu.
        let menu_bar_about_ref_mut = unsafe { menu_bar_about.as_mut().unwrap() };
        let about_about_qt = menu_bar_about_ref_mut.add_action(&QString::from_std_str("About &Qt"));
        let about_about_rpfm = menu_bar_about_ref_mut.add_action(&QString::from_std_str("&About RPFM"));
        let about_open_manual = menu_bar_about_ref_mut.add_action(&QString::from_std_str("&Open Manual"));
        let about_patreon_link = menu_bar_about_ref_mut.add_action(&QString::from_std_str("&Support me on Patreon"));
        let about_check_updates = menu_bar_about_ref_mut.add_action(&QString::from_std_str("&Check Updates"));
        let about_check_schema_updates = menu_bar_about_ref_mut.add_action(&QString::from_std_str("Check Schema &Updates"));

        command_palette_widget.hide();

        // Create ***Da monsta***.
        AppUI {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window: main_window.into_raw(),
            menu_bar,
            status_bar,

            //-------------------------------------------------------------------------------//
            // `Command Palette` DockWidget.
            //-------------------------------------------------------------------------------//
            command_palette: command_palette_widget.into_raw(),
            command_palette_line_edit: command_palette_line_edit.into_raw(),
            command_palette_completer: command_palette_completer.into_raw(),
            command_palette_completer_view,
            command_palette_completer_model: command_palette_completer_model.into_raw(),

            command_palette_show: command_palette_show.into_raw(),
            command_palette_hide: command_palette_hide.into_raw(),

            //-------------------------------------------------------------------------------//
            // `MenuBar` menus.
            //-------------------------------------------------------------------------------//
            menu_bar_packfile,
            menu_bar_mymod,
            menu_bar_view,
            menu_bar_game_seleted,
            menu_bar_special_stuff,
            menu_bar_about,

            //-------------------------------------------------------------------------------//
            // "PackFile" menu.
            //-------------------------------------------------------------------------------//

            // Menus.
            packfile_new_packfile,
            packfile_open_packfile,
            packfile_save_packfile,
            packfile_save_packfile_as,
            packfile_open_from_content: packfile_menu_open_from_content.into_raw(),
            packfile_open_from_data: packfile_menu_open_from_data.into_raw(),
            packfile_change_packfile_type: packfile_menu_change_packfile_type.into_raw(),
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
            change_packfile_type_group: change_packfile_type_group.into_raw(),

            //-------------------------------------------------------------------------------//
            // `MyMod` menu.
            //-------------------------------------------------------------------------------//
            mymod_new,
            mymod_delete_selected,
            mymod_install,
            mymod_uninstall,
            mymod_open_separator,

            //-------------------------------------------------------------------------------//
            // "View" menu.
            //-------------------------------------------------------------------------------//
            view_toggle_packfile_contents,
            view_toggle_global_search_panel,

            //-------------------------------------------------------------------------------//
            // "Game Selected" menu.
            //-------------------------------------------------------------------------------//
            game_selected_open_game_data_folder,
            game_selected_open_game_assembly_kit_folder,

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

            game_selected_group: game_selected_group.into_raw(),

            //-------------------------------------------------------------------------------//
            // "Special Stuff" menu.
            //-------------------------------------------------------------------------------//

            // Three Kingdoms actions.
            special_stuff_three_k_generate_pak_file,
            special_stuff_three_k_optimize_packfile,

            // Warhammer 2's actions.
            special_stuff_wh2_generate_pak_file,
            special_stuff_wh2_optimize_packfile,
            special_stuff_wh2_patch_siege_ai,

            // Warhammer's actions.
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
        }
    }
}
