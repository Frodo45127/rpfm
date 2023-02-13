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
Module with all the code related to the main `AppUI`.

This module contains all the code needed to initialize the main Window and its menus.
!*/

use qt_widgets::QAction;
use qt_widgets::QActionGroup;
use qt_widgets::QApplication;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QFileDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::{q_message_box, QMessageBox};
use qt_widgets::QScrollArea;
use qt_widgets::QPushButton;
use qt_widgets::QTabWidget;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QIcon;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QTimer;
use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QListOfQObject;
use qt_core::QPtr;
use qt_core::QStringList;
use qt_core::QRegExp;
use qt_core::{SlotNoArgs, SlotOfBool};
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;

use anyhow::Result;
use getset::Getters;
use itertools::Itertools;
use self_update::cargo_crate_version;

use std::cell::RefCell;
use std::collections::HashMap;
use std::env::current_exe;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;
use std::sync::{atomic::Ordering, RwLock};

use rpfm_lib::files::{animpack, ContainerPath, FileType, loc, text, pack::*, portrait_settings, text::TextFormat};
use rpfm_lib::games::{pfh_file_type::*, pfh_version::*, supported_games::*};
use rpfm_lib::integrations::{git::*, log::*};
use rpfm_lib::utils::*;

use crate::ASSETS_PATH;
use crate::backend::*;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::FIRST_GAME_CHANGE_DONE;
use crate::GAME_SELECTED;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, qtre, tre};
use crate::pack_tree::{BuildData, icons::IconType, new_pack_file_tooltip, PackTree, TreeViewOperation};
use crate::packedfile_views::matched_combat::FileMatchedCombatDebugView;
use crate::packedfile_views::{anim_fragment::*, animpack::*, audio::FileAudioView, DataSource, decoder::*, dependencies_manager::*, esf::*, external::*, image::*, PackedFileView, packfile::PackFileExtraView, packfile_settings::*, portrait_settings::PortraitSettingsView, SpecialView, table::*, text::*, unit_variant::*, video::*};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::RPFM_PATH;
use crate::SCHEMA;
use crate::settings_ui::backend::*;
use crate::STATUS_BAR;
use crate::SUPPORTED_GAMES;
use crate::TREEVIEW_ICONS;
use crate::UI_STATE;
use crate::ui::GameSelectedIcons;
use crate::ui_state::OperationalMode;
use crate::updater::{APIResponse, CHANGELOG_FILE};
use crate::utils::*;

#[cfg(feature = "support_rigidmodel")]
use crate::packedfile_views::rigidmodel::*;

#[cfg(feature = "support_uic")]
use crate::packedfile_views::uic::*;

const NEW_FILE_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_file_dialog.ui";
const NEW_FILE_VIEW_RELEASE: &str = "ui/new_file_dialog.ui";

pub mod connections;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to all the static widgets/actions created at the start of the program.
///
/// This means every widget/action that's static and created on start (menus, window,...) should be here.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // Main Window.
    //-------------------------------------------------------------------------------//
    main_window: QBox<QMainWindow>,
    tab_bar_packed_file: QBox<QTabWidget>,
    shortcuts: CppBox<QListOfQObject>,
    message_widget: QPtr<QWidget>,

    //-------------------------------------------------------------------------------//
    // Status bar stuff.
    //-------------------------------------------------------------------------------//
    discord_button: QBox<QPushButton>,
    github_button: QBox<QPushButton>,
    patreon_button: QBox<QPushButton>,
    manual_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // `MenuBar` menus.
    //-------------------------------------------------------------------------------//
    menu_bar_packfile: QPtr<QMenu>,
    menu_bar_mymod: QPtr<QMenu>,
    menu_bar_view: QPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
    packfile_new_packfile: QPtr<QAction>,
    packfile_open_packfile: QPtr<QAction>,
    packfile_save_packfile: QPtr<QAction>,
    packfile_save_packfile_as: QPtr<QAction>,
    packfile_install: QPtr<QAction>,
    packfile_uninstall: QPtr<QAction>,
    packfile_open_recent: QBox<QMenu>,
    packfile_open_from_content: QBox<QMenu>,
    packfile_open_from_data: QBox<QMenu>,
    packfile_open_from_autosave: QBox<QMenu>,
    packfile_load_all_ca_packfiles: QPtr<QAction>,
    packfile_preferences: QPtr<QAction>,
    packfile_quit: QPtr<QAction>,

    // "Change PackFile Type" submenu.
    change_packfile_type_boot: QPtr<QAction>,
    change_packfile_type_release: QPtr<QAction>,
    change_packfile_type_patch: QPtr<QAction>,
    change_packfile_type_mod: QPtr<QAction>,
    change_packfile_type_movie: QPtr<QAction>,
    change_packfile_type_other: QPtr<QAction>,

    change_packfile_type_header_is_extended: QPtr<QAction>,
    change_packfile_type_index_includes_timestamp: QPtr<QAction>,
    change_packfile_type_index_is_encrypted: QPtr<QAction>,
    change_packfile_type_data_is_encrypted: QPtr<QAction>,

    // Action to enable/disable compression on PackFiles. Only for PFH5+ PackFiles.
    change_packfile_type_data_is_compressed: QPtr<QAction>,

    // Action Group for the submenu.
    change_packfile_type_group: QBox<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `MyMod` menu.
    //-------------------------------------------------------------------------------//
    mymod_open_mymod_folder: QPtr<QAction>,
    mymod_new: QPtr<QAction>,
    mymod_delete_selected: QPtr<QAction>,
    mymod_import: QPtr<QAction>,
    mymod_export: QPtr<QAction>,

    mymod_open_warhammer_3: QPtr<QMenu>,
    mymod_open_troy: QPtr<QMenu>,
    mymod_open_three_kingdoms: QPtr<QMenu>,
    mymod_open_warhammer_2: QPtr<QMenu>,
    mymod_open_warhammer: QPtr<QMenu>,
    mymod_open_thrones_of_britannia: QPtr<QMenu>,
    mymod_open_attila: QPtr<QMenu>,
    mymod_open_rome_2: QPtr<QMenu>,
    mymod_open_shogun_2: QPtr<QMenu>,
    mymod_open_napoleon: QPtr<QMenu>,
    mymod_open_empire: QPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
    view_toggle_packfile_contents: QPtr<QAction>,
    view_toggle_global_search_panel: QPtr<QAction>,
    view_toggle_diagnostics_panel: QPtr<QAction>,
    view_toggle_dependencies_panel: QPtr<QAction>,
    view_toggle_references_panel: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
    game_selected_launch_game: QPtr<QAction>,

    game_selected_open_game_data_folder: QPtr<QAction>,
    game_selected_open_game_assembly_kit_folder: QPtr<QAction>,
    game_selected_open_config_folder: QPtr<QAction>,

    game_selected_warhammer_3: QPtr<QAction>,
    game_selected_troy: QPtr<QAction>,
    game_selected_three_kingdoms: QPtr<QAction>,
    game_selected_warhammer_2: QPtr<QAction>,
    game_selected_warhammer: QPtr<QAction>,
    game_selected_thrones_of_britannia: QPtr<QAction>,
    game_selected_attila: QPtr<QAction>,
    game_selected_rome_2: QPtr<QAction>,
    game_selected_shogun_2: QPtr<QAction>,
    game_selected_napoleon: QPtr<QAction>,
    game_selected_empire: QPtr<QAction>,
    game_selected_arena: QPtr<QAction>,

    game_selected_group: QBox<QActionGroup>,

    //-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//

    // Warhammer 3 actions.
    special_stuff_wh3_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_wh3_optimize_packfile: QPtr<QAction>,

    // Troy actions.
    special_stuff_troy_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_troy_optimize_packfile: QPtr<QAction>,

    // Three Kingdoms actions.
    special_stuff_three_k_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_three_k_optimize_packfile: QPtr<QAction>,

    // Warhammer 2's actions.
    special_stuff_wh2_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_wh2_optimize_packfile: QPtr<QAction>,
    special_stuff_wh2_patch_siege_ai: QPtr<QAction>,

    // Warhammer's actions.
    special_stuff_wh_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_wh_optimize_packfile: QPtr<QAction>,
    special_stuff_wh_patch_siege_ai: QPtr<QAction>,

    // Thrones of Britannia's actions.
    special_stuff_tob_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_tob_optimize_packfile: QPtr<QAction>,

    // Attila's actions.
    special_stuff_att_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_att_optimize_packfile: QPtr<QAction>,

    // Rome 2's actions.
    special_stuff_rom2_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_rom2_optimize_packfile: QPtr<QAction>,

    // Shogun 2's actions.
    special_stuff_sho2_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_sho2_optimize_packfile: QPtr<QAction>,

    // Napoleon's actions.
    special_stuff_nap_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_nap_optimize_packfile: QPtr<QAction>,

    // Empire's actions.
    special_stuff_emp_generate_dependencies_cache: QPtr<QAction>,
    special_stuff_emp_optimize_packfile: QPtr<QAction>,

    // Common operations.
    special_stuff_rescue_packfile: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Tools` menu.
    //-------------------------------------------------------------------------------//
    tools_faction_painter: QPtr<QAction>,
    tools_unit_editor: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    about_about_qt: QPtr<QAction>,
    about_about_rpfm: QPtr<QAction>,
    about_check_updates: QPtr<QAction>,
    about_check_schema_updates: QPtr<QAction>,
    about_check_message_updates: QPtr<QAction>,
    about_check_lua_autogen_updates: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // "Debug" menu.
    //-------------------------------------------------------------------------------//
    debug_update_current_schema_from_asskit: QPtr<QAction>,
    debug_import_schema_patch: QPtr<QAction>,
    debug_reload_style_sheet: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Extra stuff
    //-------------------------------------------------------------------------------//
    timer_backup_autosave: QBox<QTimer>,

    tab_bar_packed_file_context_menu: QBox<QMenu>,
    tab_bar_packed_file_close: QPtr<QAction>,
    tab_bar_packed_file_close_all: QPtr<QAction>,
    tab_bar_packed_file_close_all_left: QPtr<QAction>,
    tab_bar_packed_file_close_all_right: QPtr<QAction>,
    tab_bar_packed_file_prev: QPtr<QAction>,
    tab_bar_packed_file_next: QPtr<QAction>,
    tab_bar_packed_file_import_from_dependencies: QPtr<QAction>,
    tab_bar_packed_file_toggle_tips: QPtr<QAction>,

    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,
}

/// This enum contains the data needed to create a new PackedFile.
#[derive(Clone, Debug)]
pub enum NewFile {

    /// Name of the PackedFile.
    AnimPack(String),

    /// Name of the PackedFile, Name of the Table, Version of the Table.
    DB(String, String, i32),

    /// Name of the Table.
    Loc(String),

    /// Name of the Table.
    Text(String, TextFormat),

    /// Name of the file, version of the file, and a list of entries that must be cloned from existing values in vanilla files (from, to).
    PortraitSettings(String, u32, Vec<(String, String)>),
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
        let message_widget = kmessage_widget_new_safe(&widget.as_ptr());
        let tab_bar_packed_file = QTabWidget::new_1a(&widget);
        tab_bar_packed_file.set_tabs_closable(true);
        tab_bar_packed_file.set_movable(true);
        tab_bar_packed_file.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        status_bar.set_size_grip_enabled(false);
        layout.add_widget_5a(&tab_bar_packed_file, 0, 0, 1, 1);
        layout.add_widget_5a(&message_widget, 1, 0, 1, 1);
        layout.set_row_stretch(0, 10);

        let github_button = QPushButton::from_q_widget(&status_bar);
        github_button.set_flat(true);
        github_button.set_tool_tip(&qtr("github_link"));
        github_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&github_button);

        let manual_button = QPushButton::from_q_widget(&status_bar);
        manual_button.set_flat(true);
        manual_button.set_tool_tip(&qtr("open_manual"));
        manual_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/manual_icon.png", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&manual_button);

        let discord_button = QPushButton::from_q_widget(&status_bar);
        discord_button.set_flat(true);
        discord_button.set_tool_tip(&qtr("discord_link"));
        discord_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/discord.svg", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&discord_button);

        let patreon_button = QPushButton::from_q_widget(&status_bar);
        patreon_button.set_flat(true);
        patreon_button.set_tool_tip(&qtr("patreon_link"));
        patreon_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/patreon.png", ASSETS_PATH.to_string_lossy()))));
        status_bar.add_permanent_widget_1a(&patreon_button);

        STATUS_BAR.store(status_bar.as_mut_raw_ptr(), Ordering::SeqCst);

        let tab_bar_packed_file_context_menu = QMenu::from_q_widget(&tab_bar_packed_file);

        // Initialize shortcuts for the entire program.
        let shortcuts = QListOfQObject::new();
        shortcut_collection_init_safe(&main_window.static_upcast::<qt_widgets::QWidget>().as_ptr(), shortcuts.as_ptr());

        // Create the Contextual Menu Actions.
        let tab_bar_packed_file_close = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_tab", "close_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs", "close_all_other_tabs", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all_left = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs_left", "close_tabs_to_left", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all_right = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs_right", "close_tabs_to_right", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_prev = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "previus_tab", "prev_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_next = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "next_tab", "next_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_import_from_dependencies = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "import_from_dependencies", "import_from_dependencies", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_toggle_tips = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "toggle_tips", "toggle_tips", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));

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
        if !setting_bool("enable_debug_menu") {
            menu_bar_debug.menu_action().set_visible(false);
        }

        //-----------------------------------------------//
        // `PackFile` Menu.
        //-----------------------------------------------//

        // Populate the `PackFile` menu.
        let packfile_new_packfile = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "new_pack", "new_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_open_packfile = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "open_pack", "open_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_save_packfile = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "save_pack", "save_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_save_packfile_as = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "save_pack_as", "save_packfile_as", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_install = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "install_pack", "packfile_install", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_uninstall = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "uninstall_pack", "packfile_uninstall", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        let packfile_open_recent = QMenu::from_q_string_q_widget(&qtr("open_recent"), &menu_bar_packfile);
        let packfile_open_from_content = QMenu::from_q_string_q_widget(&qtr("open_from_content"), &menu_bar_packfile);
        let packfile_open_from_data = QMenu::from_q_string_q_widget(&qtr("open_from_data"), &menu_bar_packfile);
        let packfile_open_from_autosave = QMenu::from_q_string_q_widget(&qtr("open_from_autosave"), &menu_bar_packfile);
        let packfile_change_packfile_type = QMenu::from_q_string_q_widget(&qtr("change_packfile_type"), &menu_bar_packfile);

        let packfile_load_all_ca_packfiles = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "load_all_ca_packs", "load_all_ca_packfiles", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_preferences = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "settings", "preferences", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_quit = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "quit", "quit", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

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
        let mymod_open_mymod_folder = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "open_mymod_folder", "mymod_open_mymod_folder", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let mymod_new = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "new_mymod", "mymod_new", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let mymod_delete_selected = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "delete_mymod", "mymod_delete_selected", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let mymod_import = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "import_mymod", "mymod_import", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let mymod_export = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "export_mymod", "mymod_export", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

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
        let view_toggle_packfile_contents = add_action_to_menu(&menu_bar_view, shortcuts.as_ref(), "view_menu", "pack_contents_panel", "view_toggle_packfile_contents", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let view_toggle_global_search_panel = add_action_to_menu(&menu_bar_view, shortcuts.as_ref(), "view_menu", "global_search_panel", "view_toggle_global_search_panel", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let view_toggle_diagnostics_panel = add_action_to_menu(&menu_bar_view, shortcuts.as_ref(), "view_menu", "diagnostics_panel", "view_toggle_diagnostics_panel", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let view_toggle_dependencies_panel = add_action_to_menu(&menu_bar_view, shortcuts.as_ref(), "view_menu", "dependencies_panel", "view_toggle_dependencies_panel", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let view_toggle_references_panel = add_action_to_menu(&menu_bar_view, shortcuts.as_ref(), "view_menu", "references_panel", "view_toggle_references_panel", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        view_toggle_packfile_contents.set_checkable(true);
        view_toggle_global_search_panel.set_checkable(true);
        view_toggle_diagnostics_panel.set_checkable(true);
        view_toggle_dependencies_panel.set_checkable(true);
        view_toggle_references_panel.set_checkable(true);

        //-----------------------------------------------//
        // `Game Selected` Menu.
        //-----------------------------------------------//
        let game_selected_launch_game = add_action_to_menu(&menu_bar_game_selected, shortcuts.as_ref(), "game_selected_menu", "launch_game", "game_selected_launch_game", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let game_selected_open_game_data_folder = add_action_to_menu(&menu_bar_game_selected, shortcuts.as_ref(), "game_selected_menu", "open_game_data_folder", "game_selected_open_game_data_folder", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let game_selected_open_game_assembly_kit_folder = add_action_to_menu(&menu_bar_game_selected, shortcuts.as_ref(), "game_selected_menu", "open_game_ak_folder", "game_selected_open_game_assembly_kit_folder", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let game_selected_open_config_folder = add_action_to_menu(&menu_bar_game_selected, shortcuts.as_ref(), "game_selected_menu", "open_rpfm_config_folder", "game_selected_open_config_folder", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

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

        game_selected_warhammer_3.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_file_name()))).as_ref());
        game_selected_troy.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_file_name()))).as_ref());
        game_selected_three_kingdoms.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_file_name()))).as_ref());
        game_selected_warhammer_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_file_name()))).as_ref());
        game_selected_warhammer.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_file_name()))).as_ref());
        game_selected_thrones_of_britannia.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_file_name()))).as_ref());
        game_selected_attila.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_file_name()))).as_ref());
        game_selected_rome_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_file_name()))).as_ref());
        game_selected_shogun_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_file_name()))).as_ref());
        game_selected_napoleon.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_file_name()))).as_ref());
        game_selected_empire.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_file_name()))).as_ref());
        game_selected_arena.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_file_name()))).as_ref());

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
        let special_stuff_wh3_generate_dependencies_cache = add_action_to_menu(&menu_warhammer_3, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh3_optimize_packfile = add_action_to_menu(&menu_warhammer_3, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_troy_generate_dependencies_cache = add_action_to_menu(&menu_troy, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_troy_optimize_packfile = add_action_to_menu(&menu_troy, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_three_k_generate_dependencies_cache = add_action_to_menu(&menu_three_kingdoms, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_three_k_optimize_packfile = add_action_to_menu(&menu_three_kingdoms, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh2_generate_dependencies_cache = add_action_to_menu(&menu_warhammer_2, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh2_optimize_packfile = add_action_to_menu(&menu_warhammer_2, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh2_patch_siege_ai = add_action_to_menu(&menu_warhammer_2, shortcuts.as_ref(), "special_stuff_menu", "patch_siege_ai", "special_stuff_patch_siege_ai", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh_generate_dependencies_cache = add_action_to_menu(&menu_warhammer, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh_optimize_packfile = add_action_to_menu(&menu_warhammer, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_wh_patch_siege_ai = add_action_to_menu(&menu_warhammer, shortcuts.as_ref(), "special_stuff_menu", "patch_siege_ai", "special_stuff_patch_siege_ai", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_tob_generate_dependencies_cache = add_action_to_menu(&menu_thrones_of_britannia, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_tob_optimize_packfile = add_action_to_menu(&menu_thrones_of_britannia, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_att_generate_dependencies_cache = add_action_to_menu(&menu_attila, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_att_optimize_packfile = add_action_to_menu(&menu_attila, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_rom2_generate_dependencies_cache = add_action_to_menu(&menu_rome_2, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_rom2_optimize_packfile = add_action_to_menu(&menu_rome_2, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_sho2_generate_dependencies_cache = add_action_to_menu(&menu_shogun_2, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_sho2_optimize_packfile = add_action_to_menu(&menu_shogun_2, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_nap_generate_dependencies_cache = add_action_to_menu(&menu_napoleon, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_nap_optimize_packfile = add_action_to_menu(&menu_napoleon, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_emp_generate_dependencies_cache = add_action_to_menu(&menu_empire, shortcuts.as_ref(), "special_stuff_menu", "generate_dependencies_cache", "special_stuff_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let special_stuff_emp_optimize_packfile = add_action_to_menu(&menu_empire, shortcuts.as_ref(), "special_stuff_menu", "optimize_pack", "special_stuff_optimize_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        menu_bar_special_stuff.insert_separator(&special_stuff_rescue_packfile);

        //-----------------------------------------------//
        // `Tools` Menu.
        //-----------------------------------------------//

        // Populate the `Tools` menu.
        let tools_faction_painter = menu_bar_tools.add_action_q_string(&qtr("tools_faction_painter"));
        let tools_unit_editor = menu_bar_tools.add_action_q_string(&qtr("tools_unit_editor"));
        if !setting_bool("enable_unit_editor") {
            tools_unit_editor.set_enabled(false);
        }

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//
        let about_about_qt = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "about_qt", "about_about_qt", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_about_rpfm = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "about_rpfm", "about_about_rpfm", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_check_updates = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "check_updates", "about_check_updates", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_check_schema_updates = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "check_schema_updates", "about_check_schema_updates", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_check_message_updates = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "check_message_updates", "about_check_message_updates", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_check_lua_autogen_updates = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "check_tw_autogen_updates", "about_check_lua_autogen_updates", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        //-----------------------------------------------//
        // `Debug` Menu.
        //-----------------------------------------------//

        // Populate the `Debug` menu.
        let debug_update_current_schema_from_asskit = menu_bar_debug.add_action_q_string(&qtr("update_current_schema_from_asskit"));
        let debug_import_schema_patch = menu_bar_debug.add_action_q_string(&qtr("import_schema_patch"));
        let debug_reload_style_sheet = menu_bar_debug.add_action_q_string(&qtr("reload_style_sheet"));

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
            shortcuts,
            message_widget,

            //-------------------------------------------------------------------------------//
            // Status bar stuff.
            //-------------------------------------------------------------------------------//
            discord_button,
            github_button,
            patreon_button,
            manual_button,

            //-------------------------------------------------------------------------------//
            // `MenuBar` menus.
            //-------------------------------------------------------------------------------//
            menu_bar_packfile,
            menu_bar_mymod,
            menu_bar_view,

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
            view_toggle_references_panel,

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
            about_check_updates,
            about_check_schema_updates,
            about_check_message_updates,
            about_check_lua_autogen_updates,

            //-------------------------------------------------------------------------------//
            // "Debug" menu.
            //-------------------------------------------------------------------------------//
            debug_update_current_schema_from_asskit,
            debug_import_schema_patch,
            debug_reload_style_sheet,

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
            tab_bar_packed_file_toggle_tips,

            focused_widget: Rc::new(RwLock::new(None)),
            disabled_counter: Rc::new(RwLock::new(0)),
        }
    }

    /// Function to toggle the main window on and off, while keeping the stupid focus from breaking.
    pub unsafe fn toggle_main_window(&self, enable: bool) {
        if enable {
            if *self.disabled_counter.read().unwrap() == 0 {
                error!("Bug: disabled counter broke. Needs investigation.");
            }

            if *self.disabled_counter.read().unwrap() > 0 {
                *self.disabled_counter.write().unwrap() -= 1;
            }

            if *self.disabled_counter.read().unwrap() == 0 && !self.main_window().is_enabled() {
                self.main_window().set_enabled(true);
                if let Some(focus_widget) = &*self.focused_widget.read().unwrap() {
                    if !focus_widget.is_null() && focus_widget.is_visible() && focus_widget.is_enabled() {
                        focus_widget.set_focus_0a();
                    }
                }

                *self.focused_widget.write().unwrap() = None;
            }
        }

        // Disabling, so store the focused widget. Do nothing if the window was already disabled.
        else {
            *self.disabled_counter.write().unwrap() += 1;
            if self.main_window().is_enabled() {
                let focus_widget = QApplication::focus_widget();
                if !focus_widget.is_null() {
                    *self.focused_widget.write().unwrap() = Some(focus_widget);
                }

                self.main_window().set_enabled(false);
            }
        }
    }

    /// This function takes care of updating the Main Window's title to reflect the current state of the program.
    pub unsafe fn update_window_title(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // First check if we have a PackFile open. If not, just leave the default title.
        let current_version = cargo_crate_version!().split('.').map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();
        let appendix = if current_version[2] >= 99 { " - Beta - " } else { "" };
        let window_title =
            if pack_file_contents_ui.packfile_contents_tree_model().invisible_root_item().is_null() ||
            pack_file_contents_ui.packfile_contents_tree_model().invisible_root_item().row_count() == 0 {
            "Rusted PackFile Manager[*]".to_owned() + appendix
        }

        // If there is a `PackFile` open, check if it has been modified, and set the title accordingly.
        else {
            format!("{}[*]{}", pack_file_contents_ui.packfile_contents_tree_model().item_1a(0).text().to_std_string(), appendix)
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
            if save_before_deleting && packed_file_view.get_path() != RESERVED_NAME_EXTRA_PACKFILE {
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
        path: &str,
        data_source: DataSource,
        save_before_deleting: bool
    ) -> Result<()> {

        if path.is_empty() {
            info!("purging empty path? this is a bug.");
        }

        let mut did_it_worked = Ok(());

        // Black magic to remove widgets.
        let position = UI_STATE.get_open_packedfiles().iter().position(|x| *x.get_ref_path() == path && x.get_data_source() == data_source);
        if let Some(position) = position {
            if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().get(position) {

                // Do not try saving PackFiles.
                if save_before_deleting && !path.starts_with(RESERVED_NAME_EXTRA_PACKFILE) {
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
                if !path.starts_with(RESERVED_NAME_EXTRA_PACKFILE) {

                    // We check if there are more tables open. This is because we cannot change the GameSelected
                    // when there is a PackedFile using his Schema.
                    let mut enable_game_selected_menu = true;
                    for path in UI_STATE.get_open_packedfiles().iter().map(|x| x.get_ref_path()) {
                        let path = path.to_lowercase();
                        if path.starts_with("db") {
                            enable_game_selected_menu = false;
                            break;
                        }

                        else if path.ends_with(".loc") {
                            enable_game_selected_menu = false;
                            break;
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
        app_ui.toggle_main_window(false);
        let receiver = CENTRAL_COMMAND.send_background(Command::OpenPackFiles(pack_file_paths.to_vec()));

        // If it's only one packfile, store it in the recent file list.
        if pack_file_paths.len() == 1 {
            let q_settings = settings();

            let paths = if q_settings.contains(&QString::from_std_str("recentFileList")) {
                q_settings.value_1a(&QString::from_std_str("recentFileList")).to_string_list()
            } else {
                QStringList::new()
            };

            let pos = paths.index_of_1a(&QString::from_std_str(pack_file_paths[0].to_str().unwrap()));
            if pos != -1 {
                paths.remove_at(pos);
            }

            paths.prepend(&QString::from_std_str(pack_file_paths[0].to_str().unwrap()));

            while paths.count_0a() > 10 {
                paths.remove_last();
            }
            q_settings.set_value(&QString::from_std_str("recentFileList"), &QVariant::from_q_string_list(&paths));
        }

        let timer = setting_int("autosave_interval");
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Check what response we got.
        let response = CENTRAL_COMMAND.recv_try(&receiver);
        match response {

            // If it's success...
            Response::ContainerInfo(ui_data) => {

                // We choose the right option, depending on our PackFile.
                match ui_data.pfh_file_type() {
                    PFHFileType::Boot => app_ui.change_packfile_type_boot.set_checked(true),
                    PFHFileType::Release => app_ui.change_packfile_type_release.set_checked(true),
                    PFHFileType::Patch => app_ui.change_packfile_type_patch.set_checked(true),
                    PFHFileType::Mod => app_ui.change_packfile_type_mod.set_checked(true),
                    PFHFileType::Movie => app_ui.change_packfile_type_movie.set_checked(true),
                }

                // Enable or disable these, depending on what data we have in the header.
                app_ui.change_packfile_type_data_is_encrypted.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_ENCRYPTED_DATA));
                app_ui.change_packfile_type_index_includes_timestamp.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS));
                app_ui.change_packfile_type_index_is_encrypted.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_ENCRYPTED_INDEX));
                app_ui.change_packfile_type_header_is_extended.set_checked(ui_data.bitmask().contains(PFHFlags::HAS_EXTENDED_HEADER));

                // Set the compression level correctly, because otherwise we may fuckup some files.
                app_ui.change_packfile_type_data_is_compressed.set_checked(*ui_data.compress());

                // Update the TreeView.
                let mut build_data = BuildData::new();
                build_data.editable = true;
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

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
                    let game_selected = GAME_SELECTED.read().unwrap().game_key_name();
                    match ui_data.pfh_version() {

                        // PFH6 is for Troy and maybe WH3.
                        PFHVersion::PFH6 => {

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila. That's the logic.
                            match &*game_selected {
                                KEY_TROY => app_ui.game_selected_troy.trigger(),
                                _ => {
                                    show_message_warning(&app_ui.message_widget, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_TROY]));
                                    app_ui.game_selected_troy.trigger();
                                }
                            }
                        },

                        // PFH5 is for Warhammer 2/Arena.
                        PFHVersion::PFH5 => {

                            // If the PackFile has the mysterious byte enabled, it's from Arena.
                            if ui_data.bitmask().contains(PFHFlags::HAS_EXTENDED_HEADER) {
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
                                        show_message_warning(&app_ui.message_widget, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_WARHAMMER_3]));
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
                                    show_message_warning(&app_ui.message_widget, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_ROME_2]));
                                    app_ui.game_selected_rome_2.trigger();
                                }
                            }
                        },

                        // PFH3/2 is for Shogun 2.
                        PFHVersion::PFH3 | PFHVersion::PFH2 => {
                            match &*game_selected {
                                KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.trigger(),
                                _ => {
                                    show_message_warning(&app_ui.message_widget, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_SHOGUN_2]));
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
                                    show_message_warning(&app_ui.message_widget, tre("game_selected_changed_on_opening", &[DISPLAY_NAME_EMPIRE]));
                                    app_ui.game_selected_empire.trigger();
                                }
                            }
                        },
                    }
                }

                UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);

                // Re-enable the Main Window.
                app_ui.toggle_main_window(true);
            }

            // If we got an error...
            Response::Error(error) => {
                app_ui.toggle_main_window(true);
                return Err(error)
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        app_ui.toggle_main_window(false);

        // First, we need to save all open `PackedFiles` to the backend. If one fails, we want to know what one.
        AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui)?;

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
        let response = CentralCommand::recv(&receiver);
        let mut path = if let Response::PathBuf(path) = response { path } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}") };
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
            file_dialog.select_file(&QString::from_std_str(path.file_name().unwrap_or_else(|| OsStr::new("mod.pack")).to_string_lossy()));

            // If we are saving an existing PackFile with another name, we start in his current path.
            if path.is_file() {
                path.pop();
                file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref()));
            }

            // In case we have a default path for the Game Selected and that path is valid,
            // we use his data folder as base path for saving our PackFile.
            else if let Ok(ref path) = GAME_SELECTED.read().unwrap().local_mods_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                if path.is_dir() { file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref())); }
            }

            // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {
                let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let receiver = CENTRAL_COMMAND.send_background(Command::SavePackFileAs(path));
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::ContainerInfo(pack_file_info) => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);
                        let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                        packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                        packfile_item.set_text(&QString::from_std_str(file_name));

                        UI_STATE.set_operational_mode(app_ui, None);
                        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                    }
                    Response::Error(error) => result = Err(error),

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        }

        else {
            let receiver = CENTRAL_COMMAND.send_background(Command::SavePackFile);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::ContainerInfo(pack_file_info) => {
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);
                    let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                    UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                }
                Response::Error(error) => result = Err(error),

                // In ANY other situation, it's a message problem.
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        // Clean the treeview and the views from markers.
        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            packed_file_view.clean();
        }

        // Then we re-enable the main Window and return whatever we've received.
        app_ui.toggle_main_window(true);
        result
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub unsafe fn enable_packfile_actions(app_ui: &Rc<Self>, pack_path: &Path, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        let game_selected = GAME_SELECTED.read().unwrap().game_key_name();
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
            else if let Ok(game_data_path) = GAME_SELECTED.read().unwrap().local_mods_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                game_data_path.is_dir() && !pack_path.starts_with(&game_data_path)
            } else { false };
            app_ui.packfile_install.set_enabled(enable_install);

            let enable_uninstall = if !pack_path.is_file() { false }
            else if let Ok(mut game_data_path) = GAME_SELECTED.read().unwrap().local_mods_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                if !game_data_path.is_dir() || pack_path.starts_with(&game_data_path) { false }
                else {
                    game_data_path.push(pack_path.file_name().unwrap().to_string_lossy().to_string());
                    game_data_path.is_file()
                }
            } else { false };
            app_ui.packfile_uninstall.set_enabled(enable_uninstall);

            // If there is a "MyMod" path set in the settings...
            let path = PathBuf::from(setting_string(MYMOD_BASE_PATH));
            if path.is_dir() { app_ui.mymod_new.set_enabled(true); }
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
        let q_settings = settings();
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

                            if setting_bool("diagnostics_trigger_on_open") {

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
        let mut content_paths = GAME_SELECTED.read().unwrap().content_packs_paths(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name()));
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

                        if setting_bool("diagnostics_trigger_on_open") {

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
        let mut data_paths = GAME_SELECTED.read().unwrap().data_packs_paths(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name()));
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

                        if setting_bool("diagnostics_trigger_on_open") {

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
        if let Ok(autosave_paths) = backup_autosave_path() {
            let autosave_paths = files_in_folder_from_newest_to_oldest(&autosave_paths);
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

                            if setting_bool("diagnostics_trigger_on_open") {

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
        let mymod_base_path = setting_path(MYMOD_BASE_PATH);
        if mymod_base_path.is_dir() {
            if let Ok(game_folder_list) = mymod_base_path.read_dir() {
                for game_folder in game_folder_list.flatten() {

                    // If it's a valid folder, and it's in our supported games list, get all the PackFiles inside it and create an open action for them.
                    let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                    let is_supported = SUPPORTED_GAMES.games().iter().filter_map(|x| if x.supports_editing() { Some(x.game_key_name()) } else { None }).any(|x| x == game_folder_name);
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
                                    let open_mod_action = game_submenu.add_action_q_string(&QString::from_std_str(mod_name));

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

                                            if setting_bool("diagnostics_trigger_on_open") {

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

        let response = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response {
            Response::APIResponse(response) => {
                match response {
                    APIResponse::NewStableUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_stable_update", &[&last_release])
                    }
                    APIResponse::NewBetaUpdate(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_beta_update", &[&last_release])
                    }
                    APIResponse::NewUpdateHotfix(last_release) => {
                        update_button.set_enabled(true);
                        qtre("api_response_success_new_update_hotfix", &[&last_release])
                    }
                    APIResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("api_response_success_no_update")
                    }
                    APIResponse::UnknownVersion => {
                        if !use_dialog { return; }
                        qtr("api_response_success_unknown_version")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMainProgram);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => {
                    let restart_button = dialog.add_button_q_string_button_role(&qtr("restart_button"), q_message_box::ButtonRole::ApplyRole);

                    let changelog_path = RPFM_PATH.join(CHANGELOG_FILE);
                    dialog.set_text(&qtre("update_success_main_program", &[&changelog_path.to_string_lossy()]));
                    restart_button.set_enabled(true);
                    close_button.set_enabled(true);

                    // This closes the program and triggers a restart.
                    if dialog.exec() == 1 {

                        // Make sure we close both threads and the window. In windows the main window doesn't get closed for some reason.
                        CENTRAL_COMMAND.send_background(Command::Exit);
                        CENTRAL_COMMAND.send_network(Command::Exit);
                        QApplication::close_all_windows();

                        let rpfm_exe_path = current_exe().unwrap();
                        SystemCommand::new(rpfm_exe_path).spawn().unwrap();
                        exit(10);
                    }
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_button.set_enabled(true);
                        qtr("schema_new_update")
                    }
                    GitResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("schema_no_update")
                    }
                    GitResponse::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_schema")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response_thread:?}"),
        };

        // If we hit "Update", try to update the schemas.
        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateSchemas);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => {
                    dialog.set_text(&qtr("schema_update_success"));
                    close_button.set_enabled(true);
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_button.set_enabled(true);
                        qtr("messages_new_update")
                    }
                    GitResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("messages_no_update")
                    }
                    GitResponse::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_messages")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response_thread:?}"),
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

                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::Success => {
                        dialog.set_text(&qtr("messages_update_success"));
                        close_button.set_enabled(true);
                    },
                    Response::Error(error) => {
                        dialog.set_text(&QString::from_std_str(error.to_string()));
                        close_button.set_enabled(true);
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        } else {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateMessages);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => log_to_status_bar("messages_update_success"),
                Response::Error(error) => log_to_status_bar(&error.to_string()),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }
    }

    /// This function checks if there is any newer version of RPFM's schemas released.
    ///
    /// If the `use_dialog` is false, we only show a dialog in case of update available. Useful for checks at start.
    pub unsafe fn check_lua_autogen_updates(app_ui: &Rc<Self>, use_dialog: bool) {
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckLuaAutogenUpdates);

        // Create the dialog to show the response and configure it.
        let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
            q_message_box::Icon::Information,
            &qtr("update_lua_autogen_checker"),
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
        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);
        let message = match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_button.set_enabled(true);
                        qtr("lua_autogen_new_update")
                    }
                    GitResponse::NoUpdate => {
                        if !use_dialog { return; }
                        qtr("lua_autogen_no_update")
                    }
                    GitResponse::NoLocalFiles => {
                        update_button.set_enabled(true);
                        qtr("update_no_local_lua_autogen")
                    }
                }
            }

            Response::Error(error) => {
                if !use_dialog { return; }
                qtre("api_response_error", &[&error.to_string()])
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response_thread:?}"),
        };

        // If we hit "Update", try to update the schemas.
        dialog.set_text(&message);
        if dialog.exec() == 0 {
            let receiver = CENTRAL_COMMAND.send_background(Command::UpdateLuaAutogen);

            dialog.show();
            dialog.set_text(&qtr("update_in_prog"));
            update_button.set_enabled(false);
            close_button.set_enabled(false);

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::Success => {
                    dialog.set_text(&qtr("lua_autogen_update_success"));
                    close_button.set_enabled(true);
                },
                Response::Error(error) => {
                    dialog.set_text(&QString::from_std_str(error.to_string()));
                    close_button.set_enabled(true);
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        references_ui: &Rc<ReferencesUI>,
        packed_file_path: Option<String>,
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
                Some(packed_file_path) => ContainerPath::File(packed_file_path),

                // If none path has been provided, we have to do some magic to find out what we're opening.
                None => {
                    match data_source {
                        DataSource::PackFile => {
                            let selected_items = pack_file_contents_ui.packfile_contents_tree_view().get_item_types_from_selection(true);
                            if selected_items.len() == 1 { selected_items[0].clone() } else { return }
                        },
                        DataSource::ParentFiles |
                        DataSource::GameFiles |
                        DataSource::AssKitFiles => {
                            let selected_items = dependencies_ui.dependencies_tree_view().get_item_types_from_selection(true);
                            if selected_items.len() == 1 {
                                if let Some(data_source_tree) = dependencies_ui.dependencies_tree_view().get_root_source_type_from_selection(true) {
                                    data_source = data_source_tree;
                                    selected_items[0].clone()
                                } else { return }
                            } else { return }
                        }

                        DataSource::ExternalFile => unimplemented!(),
                    }
                }
            };

            if let ContainerPath::File(ref path) = item_type {

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
                            let icon_type = IconType::File(path.to_owned());
                            let icon = TREEVIEW_ICONS.icon(icon_type);
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
                    if GAME_SELECTED.read().unwrap().is_file_banned(path) {
                        tab.set_is_read_only(true);
                    } else {
                        tab.set_is_read_only(false);
                    }
                } else {
                    tab.set_is_read_only(true);
                }

                tab.set_data_source(data_source);

                if !is_external {
                    let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(path.to_string(), tab.get_data_source()));

                    tab.set_is_preview(is_preview);
                    let icon_type = IconType::File(path.to_owned());
                    let icon = TREEVIEW_ICONS.icon(icon_type);

                    let response = CentralCommand::recv(&receiver);
                    match response {

                        // If the file is an AnimFragment PackedFile...
                        Response::AnimFragmentRFileInfo(data, ref file_info) => {
                            let file_info = file_info.clone();
                            match FileAnimFragmentDebugView::new_view(&mut tab, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        },

                        Response::AnimPackRFileInfo(files_info, file_info) => {
                            match PackedFileAnimPackView::new_view(&mut tab, app_ui, pack_file_contents_ui, &file_info, &files_info) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },

                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        },

                        // If the file is an AnimTable PackedFile...
                        Response::AnimsTableRFileInfo(_, _) => {}
                        /*Response::AnimsTableRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }*/

                        Response::AudioRFileInfo(data, file_info) => {
                            match FileAudioView::new_view(&mut tab, &data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                }
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        // If the file is a DB PackedFile...
                        Response::DBRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => {

                                    // Try to get the data of the table to send it for decoding.
                                    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFileRawData(path.to_owned()));
                                    let response = CentralCommand::recv(&receiver);
                                    let data = match response {
                                        Response::VecU8(data) => data,
                                        Response::Error(_) => return show_dialog(&app_ui.main_window, error, false),
                                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                                    };

                                    return show_dialog_decode_button(app_ui.main_window.static_upcast::<qt_widgets::QWidget>().as_ptr(), error, file_info.table_name().unwrap(), &data);
                                },
                            }
                        }

                        Response::ESFRFileInfo(data, file_info) => {
                            if setting_bool("enable_esf_editor") {
                                match PackedFileESFView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, data) {
                                    Ok(_) => {

                                        // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                        // Fix the tips view.
                                        let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);

                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                        }
                                    },
                                    Err(error) => return show_dialog(&app_ui.main_window, error, false),
                                }
                            }
                        }

                        // If the file is a Image PackedFile, ignore failures while opening.
                        Response::ImageRFileInfo(data, file_info) => {
                            match PackedFileImageView::new_view(&mut tab, &data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                }
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        // If the file is a Loc PackedFile...
                        Response::LocRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        Response::MatchedCombatRFileInfo(data, ref file_info) => {
                            let file_info = file_info.clone();
                            match FileMatchedCombatDebugView::new_view(&mut tab, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        Response::PortraitSettingsRFileInfo(mut data, file_info) => {
                            match PortraitSettingsView::new_view(&mut tab, &mut data, app_ui, pack_file_contents_ui) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                    // Fix the tips view.
                                    let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                    }
                                },
                                Err(error) => return show_dialog(&app_ui.main_window, error, false),
                            }
                        }

                        // Generic files logic.
                        Response::RFileDecodedRFileInfo(data, file_info) => {
                            match file_info.file_type() {
                                FileType::UnitVariant => {
                                    match PackedFileUnitVariantView::new_view(&mut tab, data) {
                                        Ok(_) => {

                                            // Add the file to the 'Currently open' list and make it visible.
                                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                            // Fix the tips view.
                                            let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                            layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                            let mut open_list = UI_STATE.set_open_packedfiles();
                                            open_list.push(tab);

                                            if data_source == DataSource::PackFile {
                                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                            }
                                        }
                                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                                    }
                                }
                                _ => {},
                            }
                        }

                        // If the file is a RigidModel PackedFile...
                        #[cfg(feature = "support_rigidmodel")]
                        Response::RigidModelRFileInfo(data, file_info) => {
                            if setting_bool("enable_rigidmodel_editor") {
                                match PackedFileRigidModelView::new_view(&mut tab, &data) {
                                    Ok(_) => {

                                       // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                                        // Fix the tips view.
                                        let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);

                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                                        }
                                    },
                                    Err(error) => return show_dialog(&app_ui.main_window, error, false),
                                }
                            }
                        }

                        // If the file is a Text PackedFile...
                        Response::TextRFileInfo(data, file_info) => {
                            PackedFileTextView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                            // Fix the tips view.
                            let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);

                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                            }
                        }

                        // If the file is the notes...
                        Response::Text(data) => {
                            PackedFileTextView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                            // Fix the tips view.
                            let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                        }

                        // If the file is a CA_VP8 PackedFile...
                        Response::VideoInfoRFileInfo(data, file_info) => {
                            PackedFileVideoView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());

                            // Fix the tips view.
                            let layout = tab.get_mut_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.get_tips_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source);
                            }
                        }

                        Response::Unknown => {},
                        Response::Error(error) => return show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };
                    /*
                    let packed_file_type = get_packed_file_type(path);

                    match packed_file_type {







                        // If the file is a UI Component...
                        #[cfg(feature = "support_uic")]
                        FileType::UIC => {
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

                        FileType::PackFile => {
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
                    }*/
                }

                // If it's external, we just create a view with just one button: "Stop Watching External File".
                else {
                    let icon_type = IconType::File(path.to_owned());
                    let icon = TREEVIEW_ICONS.icon(icon_type);

                    let receiver = CENTRAL_COMMAND.send_background(Command::OpenPackedFileInExternalProgram(DataSource::PackFile, ContainerPath::File(path.to_owned())));
                    let path = Rc::new(RefCell::new(path.to_owned()));

                    let response = CentralCommand::recv(&receiver);
                    let external_path = match response {
                        Response::PathBuf(external_path) => external_path,
                        Response::Error(error) => return show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };

                    PackedFileExternalView::new_view(&path, app_ui, &mut tab, pack_file_contents_ui, &external_path);

                    // Add the file to the 'Currently open' list and make it visible.
                    app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(""));
                    app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                    let mut open_list = UI_STATE.set_open_packedfiles();
                    open_list.push(tab);
                }
            }
        }

        Self::update_views_names(app_ui);

        // Try to paint the diagnostics results, if any.
        for diagnostic_type in UI_STATE.get_diagnostics().results() {
            DiagnosticsUI::paint_diagnostics_to_table(app_ui, diagnostic_type);
        }
    }

    /// This function is used to open views that cannot be open with the normal open_file_view function.
    pub unsafe fn open_special_view(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        view_type: SpecialView,
    ) {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here on.
        if !UI_STATE.get_packfile_contents_read_only() {

            let (path, name) = match view_type {
                SpecialView::Decoder(ref path) => {

                    // If we don't have an schema, don't even try it.
                    if SCHEMA.read().unwrap().is_none() {
                        return show_dialog(&app_ui.main_window, "No schema found. You need one to open the decoder.", false);
                    }

                    let mut fake_path = path.to_owned();
                    fake_path.push_str(DECODER_EXTENSION);
                    (fake_path, qtr("decoder_title"))
                },
                SpecialView::Pack(ref path) => (path.to_owned(), QString::from_std_str(path)),
                SpecialView::PackSettings => (RESERVED_NAME_SETTINGS.to_owned(), qtr("settings")),
                SpecialView::PackDependencies => (RESERVED_NAME_DEPENDENCIES_MANAGER.to_owned(), qtr("table_dependency_manager_title")),
            };

            // Close all preview views except the file we're opening. The path used for the manager is empty.
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
                    let icon_type = IconType::Pack(true);
                    let icon = TREEVIEW_ICONS.icon(icon_type);
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
            let icon_type = IconType::Pack(true);
            let icon = TREEVIEW_ICONS.icon(icon_type);

            match view_type {
                SpecialView::Decoder(ref path) => {
                    match PackedFileDecoderView::new_view(&mut tab, path, app_ui, pack_file_contents_ui) {
                        Ok(_) => {

                            // Add the decoder to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                        },
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    }
                }
                SpecialView::Pack(ref path) => {
                    tab.set_path(&(RESERVED_NAME_EXTRA_PACKFILE.to_owned() + "/" + path));
                    let pathbuf = PathBuf::from(path.to_owned());
                    match PackFileExtraView::new_view(&mut tab, app_ui, pack_file_contents_ui, pathbuf) {
                        Ok(_) => {
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &QString::from_std_str(path));
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                            UI_STATE.set_open_packedfiles().push(tab);
                        }
                        Err(error) => show_dialog(&app_ui.main_window, error, false),
                    }
                },
                SpecialView::PackSettings => {
                    match PackFileSettingsView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                        Ok(_) => {
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                            UI_STATE.set_open_packedfiles().push(tab);
                        },
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    }
                },
                SpecialView::PackDependencies => {
                    match DependenciesManagerView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui) {
                        Ok(_) => {

                            // Add the manager to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.get_mut_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.get_mut_widget());
                            UI_STATE.set_open_packedfiles().push(tab);
                        },
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    }
                }
            }
        }

        Self::update_views_names(app_ui);
    }

    /// This function is the one that takes care of the creation of different Files, including making sure we can create them,
    /// and triggering the relevant dialogs for each file type.
    pub unsafe fn new_file(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>, file_type: FileType) {

        // DB Files require the dependencies cache to be generated, and the schemas to be downloaded.
        if file_type == FileType::DB {

            if SCHEMA.read().unwrap().is_none() {
                return show_dialog(&app_ui.main_window, "There is no Schema for the Game Selected.", false);
            }

            let receiver = CENTRAL_COMMAND.send_background(Command::IsThereADependencyDatabase(false));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Bool(it_is) => if !it_is { return show_dialog(&app_ui.main_window, "The dependencies cache for the Game Selected is either missing, outdated, or it was generated without the Assembly Kit. Please, re-generate it and try again.", false); },
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        if file_type == FileType::PortraitSettings && GAME_SELECTED.read().unwrap().portrait_settings_version().is_none() {
            return show_dialog(&app_ui.main_window, "Creating PortraitSettings files is currently not supported for this game.", false);
        }

        // Create the "New File" dialog and wait for his data (or a cancellation). If we receive None, we do nothing. If we receive Some,
        // we still have to check if it has been any error during the creation of the File (for example, no definition for DB Tables).
        match Self::new_file_dialog(app_ui, file_type) {
            Ok(new_file) => {
                if let Some(mut new_file) = new_file {

                    // Check for empty names.
                    match new_file {
                        NewFile::AnimPack(ref mut name) |
                        NewFile::Loc(ref mut name) |
                        NewFile::Text(ref mut name, _) |
                        NewFile::PortraitSettings(ref mut name, _, _) |
                        NewFile::DB(ref mut name, _, _) => {

                            // If the name is_empty, stop.
                            if name.is_empty() {
                                return show_dialog(&app_ui.main_window, "Only my hearth can be empty.", false)
                            }
                        }
                    }

                    // If we reach this place, we get the full path of the file.
                    let selected_paths = pack_file_contents_ui.packfile_contents_tree_view().get_path_from_selection();
                    let full_path = match new_file {
                        NewFile::AnimPack(ref name) |
                        NewFile::Loc(ref name) |
                        NewFile::PortraitSettings(ref name, _, _) |
                        NewFile::Text(ref name, _) => {

                            if selected_paths.len() == 1 {
                                let mut complete_path = selected_paths[0].to_owned();
                                if !complete_path.is_empty() && !complete_path.ends_with('/') {
                                    complete_path.push('/');
                                }
                                complete_path.push_str(name);
                                complete_path
                            }
                            else {
                                return show_dialog(&app_ui.main_window, "Multiple selected paths? This should never happen. Pls, report it, because its a bug.", false)
                            }
                        },
                        NewFile::DB(ref name, ref table, _) => {
                            format!("db/{table}/{name}")
                        }
                    };

                    // Check if the File already exists, and report it if so.
                    let receiver = CENTRAL_COMMAND.send_background(Command::PackedFileExists(full_path.to_owned()));
                    let response = CentralCommand::recv(&receiver);
                    let exists = if let Response::Bool(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
                    if exists {
                        return show_dialog(&app_ui.main_window, format!("A file with this path ({full_path})' already exists in the Pack."), false)
                    }

                    // Get the response, just in case it failed.
                    let receiver = CENTRAL_COMMAND.send_background(Command::NewPackedFile(full_path.to_owned(), new_file));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::Success => {
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(full_path); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                        }

                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }
                }
            },
            Err(error) => show_dialog(&app_ui.main_window, error, false),
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
        let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
        if selected_items.len() == 1 {
            let item = &selected_items[0];

            let path = match item {
                ContainerPath::File(_) => item.parent_path(),
                ContainerPath::Folder(path) => path.to_owned(),
            };
            let path_split = path.split('/').collect::<Vec<_>>();

            if let Some(mut name) = Self::new_packed_file_name_dialog(app_ui) {

                // DB Check.
                let (new_path, new_packed_file) = if path.starts_with("db") && (path_split.len() == 2 || path_split.len() == 3) {
                    let new_path = format!("{path}/{name}");
                    let table = path_split[1];

                    let receiver = CENTRAL_COMMAND.send_background(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CentralCommand::recv(&receiver);
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };

                    let new_packed_file = NewFile::DB(name.to_owned(), table.to_owned(), version);
                    (new_path, new_packed_file)
                }

                // Loc Check.
                else if path.starts_with("text") && !path.is_empty() {
                    if !name.ends_with(".loc") { name.push_str(".loc"); }
                    let mut new_path = path.to_owned();

                    if !new_path.ends_with('/') {
                        new_path.push('/');
                    }
                    new_path.push_str(&name);

                    let new_packed_file = NewFile::Loc(name.to_owned());
                    (new_path, new_packed_file)
                }

                // Lua Check.
                else if path.starts_with("script") && !path.is_empty() {
                    if !name.ends_with(".lua") { name.push_str(".lua"); }
                    let mut new_path = path.to_owned();

                    if !new_path.ends_with('/') {
                        new_path.push('/');
                    }
                    new_path.push_str(&name);

                    let new_packed_file = NewFile::Text(name.to_owned(), TextFormat::Lua);
                    (new_path, new_packed_file)
                }

                // VMD Check.
                else if path.starts_with("variantmeshes/variantmeshdefinitions") && !path.is_empty() {
                    if !name.ends_with(".variantmeshdefinition") { name.push_str(".variantmeshdefinition"); }
                    let mut new_path = path.to_owned();

                    if !new_path.ends_with('/') {
                        new_path.push('/');
                    }
                    new_path.push_str(&name);

                    let new_packed_file = NewFile::Text(name.to_owned(), TextFormat::Xml);
                    (new_path, new_packed_file)
                }

                // Neutral Check, for folders without a predefined type.
                else {
                    return show_dialog(&app_ui.main_window, "I don't know what type of file goes in that folder, boi.", false);
                };

                // Check if the PackedFile already exists, and report it if so.
                let receiver = CENTRAL_COMMAND.send_background(Command::PackedFileExists(new_path.to_owned()));
                let response = CentralCommand::recv(&receiver);
                let exists = if let Response::Bool(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
                if exists { return show_dialog(&app_ui.main_window, "The provided file/s already exists in the current path.", false)}

                // Create the PackFile.
                let receiver = CENTRAL_COMMAND.send_background(Command::NewPackedFile(new_path.to_owned(), new_packed_file));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(new_path); 1]), DataSource::PackFile);
                        UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                    }
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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

    /// This function creates all the "New File" dialogs.
    ///
    /// It returns the type/name of the new file, or None if the dialog is canceled or closed.
    pub unsafe fn new_file_dialog(app_ui: &Rc<Self>, file_type: FileType) -> Result<Option<NewFile>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { NEW_FILE_VIEW_DEBUG } else { NEW_FILE_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        // Create and configure the "New PackedFile" Dialog.
        match file_type {
            FileType::AnimPack => dialog.set_window_title(&qtr("new_animpack_file")),
            FileType::DB => dialog.set_window_title(&qtr("new_db_file")),
            FileType::Loc => dialog.set_window_title(&qtr("new_loc_file")),
            FileType::Text => dialog.set_window_title(&qtr("new_txt_file")),
            FileType::PortraitSettings => dialog.set_window_title(&qtr("new_portrait_settings_file")),
            _ => unimplemented!(),
        }

        // Common section.
        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let message_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "message_widget")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());
        kmessage_widget_close_safe(&message_widget.as_ptr());

        // DB section.
        let table_extra_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "db_widget")?;
        let table_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let table_dropdown: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "table_combo_box")?;
        let table_filter = QSortFilterProxyModel::new_1a(&dialog);
        let table_model = QStandardItemModel::new_1a(&dialog);
        table_filter.set_source_model(&table_model);
        table_dropdown.set_model(&table_filter);
        table_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        table_filter_line_edit.text_changed().connect(&SlotNoArgs::new(&dialog, move || {
            table_filter.set_filter_reg_exp_q_reg_exp(&QRegExp::new_1a(&table_filter_line_edit.text()));
        }));

        // Portrait Settings section.
        let portrait_settings_extra_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "portrait_settings_widget")?;
        let portrait_settings_scroll_area: QPtr<QScrollArea> = find_widget(&main_widget.static_upcast(), "portrait_settings_scroll_area")?;
        let portrait_settings_scroll_area_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "portrait_settings_scroll_area_widget")?;
        let portrait_settings_art_set_id_model = QStandardItemModel::new_1a(&dialog);
        let portrait_settings_copy_column_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "copy_column_label")?;
        let portrait_settings_copy_from_column_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "copy_from_column_label")?;
        let portrait_settings_copy_to_column_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "copy_to_column_label")?;
        let mut portrait_settings_widgets = vec![];
        portrait_settings_copy_column_label.set_text(&qtr("new_portrait_settings_copy_column"));
        portrait_settings_copy_from_column_label.set_text(&qtr("new_portrait_settings_copy_from_column"));
        portrait_settings_copy_to_column_label.set_text(&qtr("new_portrait_settings_copy_to_column"));

        // Hide all extra widgets by default, and only make the ones we need visible.
        table_extra_widget.set_visible(false);
        portrait_settings_extra_widget.set_visible(false);

        // The default file name is the Pack name.
        //
        // That's because usually modders name many of the mod files like that.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFileName);
        let response = CentralCommand::recv(&receiver);
        let pack_name = if let Response::String(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
        let pack_name = if pack_name.to_lowercase().ends_with(".pack") {
            let mut pack_name = pack_name;
            pack_name.pop();
            pack_name.pop();
            pack_name.pop();
            pack_name.pop();
            pack_name.pop();
            pack_name
        } else { pack_name };

        match file_type {
            FileType::AnimPack => name_line_edit.set_text(&QString::from_std_str(format!("{pack_name}.animpack"))),
            FileType::DB => {
                let receiver = CENTRAL_COMMAND.send_background(Command::GetTableListFromDependencyPackFile);
                let response = CentralCommand::recv(&receiver);
                let mut tables = if let Response::VecString(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
                tables.sort();
                tables.iter().for_each(|x| table_dropdown.add_item_q_string(&QString::from_std_str(x)));

                name_line_edit.set_text(&QString::from_std_str(&pack_name));
                table_extra_widget.set_visible(true);
            },
            FileType::Loc => name_line_edit.set_text(&QString::from_std_str(format!("{pack_name}.loc"))),
            FileType::Text => name_line_edit.set_text(&QString::from_std_str(format!("{pack_name}.txt"))),
            FileType::PortraitSettings => {
                let receiver = CENTRAL_COMMAND.send_background(Command::LocalArtSetIds);
                let response = CentralCommand::recv(&receiver);
                let local_art_set_ids = if let Response::HashSetString(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

                let receiver = CENTRAL_COMMAND.send_background(Command::DependenciesArtSetIds);
                let response = CentralCommand::recv(&receiver);
                let dependencies_art_set_ids = if let Response::HashSetString(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

                for art_set_id in dependencies_art_set_ids.iter().sorted_unstable() {
                    let item = QStandardItem::from_q_string(&QString::from_std_str(art_set_id));
                    portrait_settings_art_set_id_model.append_row_q_standard_item(item.into_ptr());
                }

                // We need one row for each art set id we need an entry for.
                let base_layout = portrait_settings_scroll_area_widget.layout().static_downcast::<QGridLayout>();
                for (index, art_set_id) in local_art_set_ids.iter().sorted_unstable().enumerate() {
                    let use_check_box = QCheckBox::from_q_widget(&portrait_settings_scroll_area);
                    let art_set_id_to_copy_combo_box = QComboBox::new_1a(&portrait_settings_scroll_area);
                    let arrow_label = QLabel::from_q_string_q_widget(&QString::from_std_str("<html><head/><body><p>→</p></body></html>"), &portrait_settings_scroll_area);
                    let art_set_id_new_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(art_set_id), &portrait_settings_scroll_area);

                    use_check_box.set_checked(true);
                    art_set_id_to_copy_combo_box.set_model(&portrait_settings_art_set_id_model);
                    art_set_id_new_line_edit.set_read_only(true);

                    base_layout.add_widget_5a(&use_check_box, index as i32 + 2, 0, 1, 1);
                    base_layout.add_widget_5a(&art_set_id_to_copy_combo_box, index as i32 + 2, 1, 1, 1);
                    base_layout.add_widget_5a(&arrow_label, index as i32 + 2, 2, 1, 1);
                    base_layout.add_widget_5a(&art_set_id_new_line_edit, index as i32 + 2, 3, 1, 1);

                    portrait_settings_widgets.push((use_check_box, art_set_id_to_copy_combo_box, art_set_id_new_line_edit));
                }

                name_line_edit.set_text(&QString::from_std_str(format!("portrait_settings_{pack_name}.bin")));
                portrait_settings_extra_widget.set_visible(true);
            },
            _ => unimplemented!(),
        }

        // Show the Dialog and, if we hit the "Ok" button, return the corresponding NewPackedFileType.
        if dialog.exec() == 1 {
            let mut file_name = name_line_edit.text().to_std_string();
            match file_type {
                FileType::AnimPack => {
                    if !file_name.ends_with(animpack::EXTENSION) {
                        file_name.push_str(animpack::EXTENSION);
                    }
                    Ok(Some(NewFile::AnimPack(file_name)))
                }

                FileType::DB => {
                    let table = table_dropdown.current_text().to_std_string();
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetTableVersionFromDependencyPackFile(table.to_owned()));
                    let response = CentralCommand::recv(&receiver);
                    let version = match response {
                        Response::I32(data) => data,
                        Response::Error(error) => return Err(error),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };
                    Ok(Some(NewFile::DB(file_name, table, version)))
                },
                FileType::Loc => {
                    if !file_name.ends_with(loc::EXTENSION) {
                        file_name.push_str(loc::EXTENSION);
                    }
                    Ok(Some(NewFile::Loc(file_name)))
                },
                FileType::Text => {
                    if !text::EXTENSIONS.iter().any(|(x, _)| file_name.ends_with(x)) {
                        file_name.push_str(".txt");
                    }

                    let text_format = if let Some((_, text_format)) = text::EXTENSIONS.iter().find(|(x, _)| file_name.ends_with(x)) {
                        *text_format
                    } else {
                        TextFormat::Plain
                    };

                    Ok(Some(NewFile::Text(file_name, text_format)))
                },
                FileType::PortraitSettings => {
                    if !file_name.ends_with(portrait_settings::EXTENSION) {
                        file_name.push_str(portrait_settings::EXTENSION);
                    }

                    if !file_name.starts_with("portrait_settings_") {
                        file_name = format!("portrait_settings_{file_name}");
                    }

                    let mut import_entries = vec![];
                    for (checkbox, source_combo, dest_line_edit) in &portrait_settings_widgets {
                        if checkbox.is_checked() {
                            import_entries.push((source_combo.current_text().to_std_string(), dest_line_edit.text().to_std_string()));
                        }
                    }

                    // Unwrap because we already check it's valid before calling this function. If it crashes here, it's a bug in the caller.
                    Ok(Some(NewFile::PortraitSettings(file_name, GAME_SELECTED.read().unwrap().portrait_settings_version().unwrap(), import_entries)))
                },
                _ => unimplemented!(),
            }
        }

        // Otherwise, return None.
        else { Ok(None) }
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
        let packfile_name = if let Response::String(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
        let packfile_name = if packfile_name.to_lowercase().ends_with(".pack") {
            let mut packfile_name = packfile_name;
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name
        } else { packfile_name };

        name_line_edit.set_text(&QString::from_std_str(packfile_name));

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
        let packfile_name = if let Response::String(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };
        let packfile_name = if packfile_name.to_lowercase().ends_with(".pack") {
            let mut packfile_name = packfile_name;
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name.pop();
            packfile_name
        } else { packfile_name };

        name_line_edit.set_text(&QString::from_std_str(packfile_name));

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
                let path_split = path.split('/').collect::<Vec<_>>();
                if *path == RESERVED_NAME_NOTES {
                    names.insert("Notes".to_owned(), 1);
                } else if let Some(name) = path_split.last() {
                    match names.get_mut(*name) {
                        Some(name) => *name += 1,
                        None => { names.insert(name.to_string(), 1); },
                    }
                }
            }
        }

        for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
            let widget = packed_file_view.get_mut_widget();
            let path = packed_file_view.get_ref_path();
            let path_split = path.split('/').collect::<Vec<_>>();
            let widget_name = if *path == RESERVED_NAME_NOTES {
                "Notes".to_owned()
            } else if let Some(widget_name) = path_split.last() {
                widget_name.to_string()
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
                    name.push_str(&path);
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
                    if path.starts_with(RESERVED_NAME_EXTRA_PACKFILE) {
                        purge_on_delete.push(path.to_owned());

                        let path_split = path.split('/').collect::<Vec<_>>();
                        let path = path_split[1..].join("/");
                        let _ = CENTRAL_COMMAND.send_background(Command::RemovePackFileExtra(PathBuf::from(&path)));
                    }
                    else if path.ends_with(DECODER_EXTENSION) {
                        purge_on_delete.push(path.to_owned());
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

    /// Function to change the game selected, changing schemas, dependencies, and all related stuff as needed.
    pub unsafe fn change_game_selected(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        rebuild_dependencies: bool
    ) {

        // Optimization: get this before starting the entire game change. Otherwise, we'll hang the thread near the end.
        // Mutable because we reuse this variable to store the other receiver we need to generate down below.
        let mut receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
        let response = CentralCommand::recv(&receiver);
        let pack_path = if let Response::PathBuf(pack_path) = response { pack_path } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}") };

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = app_ui.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();
        let mut game_changed = false;

        // Due to how the backend is optimised, we need to back our files before triggering the proper game change.
        let _ = AppUI::purge_them_all(app_ui, pack_file_contents_ui, true);

        // If the game changed or we're initializing the program, change the game selected.
        if new_game_selected != GAME_SELECTED.read().unwrap().game_key_name() || !FIRST_GAME_CHANGE_DONE.load(Ordering::SeqCst) {

            // Disable the main window if it's not yet disabled so we can avoid certain issues.
            app_ui.toggle_main_window(false);

            // Send the command to the background thread to set the new `Game Selected`.
            receiver = CENTRAL_COMMAND.send_background(Command::SetGameSelected(new_game_selected, rebuild_dependencies));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Success => {}
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }

            // If we have a pack open, set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
            // We do not really support changing game selected while keep treating a mymod as a mymod.
            if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() > 0 {
                UI_STATE.set_operational_mode(app_ui, None);
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::Folder(String::new())]), DataSource::PackFile);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }

            // Change the GameSelected Icon.
            GameSelectedIcons::set_game_selected_icon(app_ui);

            // Set this at the end, because the backend need to check if it's our first initialization or not first.
            FIRST_GAME_CHANGE_DONE.store(true, Ordering::SeqCst);
            game_changed = true;
        }

        // Regardless if the game changed or not, if we are asked to rebuild data, prepare for a rebuild.
        // Realistically, there are two reasons for calling this:
        // - Game changed, requires full dependencies rebuild.
        // - Pack changed, requires parent mod rebuild.
        //
        // The backend already differentiates between the two and acts accordingly.
        if rebuild_dependencies {

            // If the game didn't change, we need to make sure we generate a receiver for this.
            let receiver = if game_changed {
                info!("Reusing receiver.");
                receiver
            } else {
                info!("New receiver.");
                CENTRAL_COMMAND.send_background(Command::RebuildDependencies(true))
            };

            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::DependenciesInfo(response) => {
                    let mut parent_build_data = BuildData::new();
                    parent_build_data.data = Some((ContainerInfo::default(), response.parent_packed_files().to_vec()));
                    dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles);

                    // While the backend returns the data of the entire dependencies, game and asskit data only change on game change, so we don't need to
                    // rebuild them the game didn't change.
                    if game_changed {

                        // NOTE: We're MOVING, not copying nor referencing the RFileInfo. This info is big and moving it makes it faster.
                        let mut game_build_data = BuildData::new();
                        game_build_data.data = Some((ContainerInfo::default(), response.vanilla_packed_files));

                        let mut asskit_build_data = BuildData::new();
                        asskit_build_data.data = Some((ContainerInfo::default(), response.asskit_tables));
                        dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(game_build_data), DataSource::GameFiles);
                        dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(asskit_build_data), DataSource::AssKitFiles);
                    }
                }
                Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        // Reenable the main window once everything is reloaded, regardless of if we disabled it here or not.
        if game_changed {
            app_ui.toggle_main_window(true);
        }

        // Disable the pack-related actions and, if we have a pack open, re-enable them.
        AppUI::enable_packfile_actions(app_ui, &pack_path, false);
        if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() != 0 {
            AppUI::enable_packfile_actions(app_ui, &pack_path, true);
        }

        // If we have the setting enabled, ask the backend to generate the missing definition list.
        if setting_bool("check_for_missing_table_definitions") {
            let _ = CENTRAL_COMMAND.send_background(Command::GetMissingDefinitions);
        }
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
        let timer = setting_int("autosave_interval");
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Disable the main window, so the user can't interrupt the process or interfere with it.
        let window_was_disabled = app_ui.main_window.is_enabled();
        if !window_was_disabled {
            app_ui.toggle_main_window(false);
        }

        // Close any open PackedFile and clear the global search panel.
        let _ = AppUI::purge_them_all(app_ui,  pack_file_contents_ui, false);
        GlobalSearchUI::clear(global_search_ui);
        diagnostics_ui.diagnostics_table_model().clear();

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
        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

        // Enable the actions available for the PackFile from the `MenuBar`.
        AppUI::enable_packfile_actions(app_ui, &PathBuf::new(), true);

        // Set the current "Operational Mode" to Normal, as this is a "New" mod.
        UI_STATE.set_operational_mode(app_ui, None);
        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);

        // Force a dependency rebuild.
        let receiver = CENTRAL_COMMAND.send_background(Command::RebuildDependencies(true));
        let response = CENTRAL_COMMAND.recv_try(&receiver);
        match response {
            Response::DependenciesInfo(response) => {
                let mut parent_build_data = BuildData::new();
                parent_build_data.data = Some((ContainerInfo::default(), response.parent_packed_files().to_vec()));
                dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles);
            }
            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }

        // Re-enable the Main Window.
        if !window_was_disabled {
            app_ui.toggle_main_window(true);
        }
    }

    /// This function is used to perform MyḾod imports.
    pub unsafe fn import_mymod(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) {
        app_ui.toggle_main_window(false);

        match UI_STATE.get_operational_mode() {

            // If we have a "MyMod" selected...
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                let mymods_base_path = setting_path("mymods_base_path");
                if mymods_base_path.is_dir() {

                    // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                    let mut mod_name = mod_name.to_owned();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();

                    let mut assets_folder = mymods_base_path;
                    assets_folder.push(game_folder_name);
                    assets_folder.push(&mod_name);

                    // Get the Paths of the files inside the folders we want to add.
                    let paths: Vec<PathBuf> = match files_from_subdir(&assets_folder, true) {
                        Ok(paths) => paths,
                        Err(error) => {
                            app_ui.toggle_main_window(true);
                            return show_dialog(&app_ui.main_window, error, false);
                        }
                    };

                    // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                    let mut paths_packedfile: Vec<ContainerPath> = vec![];
                    for path in &paths {
                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                        paths_packedfile.push(ContainerPath::File(filtered_path.to_string_lossy().to_string()));
                    }

                    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackSettings);
                    let response = CentralCommand::recv(&receiver);
                    let settings = match response {
                        Response::PackSettings(settings) => settings,
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };

                    let files_to_ignore = settings.setting_text("import_files_to_ignore").map(|files_to_ignore| {
                        if files_to_ignore.is_empty() { vec![] } else {
                            files_to_ignore.split('\n')
                                .filter(|x| !x.is_empty())
                                .map(|x| assets_folder.to_path_buf().join(x))
                                .collect::<Vec<PathBuf>>()
                        }
                    });

                    PackFileContentsUI::add_files(app_ui, pack_file_contents_ui, &paths, &paths_packedfile, files_to_ignore);
                }

                // If there is no MyMod path configured, report it.
                else { show_dialog(&app_ui.main_window, "MyMod path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false) }
            }
            OperationalMode::Normal => show_dialog(&app_ui.main_window, "This action is only available for MyMods.", false),
        }

        app_ui.toggle_main_window(true);
    }

    /// This function is used to perform MyḾod exports.
    pub unsafe fn export_mymod(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        paths_to_extract: Option<Vec<ContainerPath>>
    ) {
        PackFileContentsUI::extract_packed_files(app_ui, pack_file_contents_ui, paths_to_extract, true)
    }
}
