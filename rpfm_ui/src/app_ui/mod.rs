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
Module with all the code related to the main `AppUI`.

This module contains all the code needed to initialize the main Window and its menus.
!*/

use qt_widgets::QApplication;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::{ButtonRole, StandardButton};
use qt_widgets::QFileDialog;
use qt_widgets::QGroupBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QListWidget;
use qt_widgets::QListWidgetItem;
use qt_widgets::QTableWidget;
use qt_widgets::QTableWidgetItem;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QSpinBox;
use qt_widgets::{q_message_box, QMessageBox};
use qt_widgets::QScrollArea;
use qt_widgets::QPushButton;
use qt_widgets::QTabWidget;
use qt_widgets::QTreeView;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QAction;
use qt_gui::QActionGroup;
use qt_gui::QGuiApplication;
use qt_gui::QIcon;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QTimer;
use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QEventLoop;
use qt_core::ItemFlag;
use qt_core::QListOfQObject;
use qt_core::QPtr;
use qt_core::QRegularExpression;
use qt_core::{SlotNoArgs, SlotOfBool};
use qt_core::QSortFilterProxyModel;
use qt_core::SortOrder;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;

use anyhow::{anyhow, Result};
use getset::Getters;
use itertools::Itertools;
use self_update::cargo_crate_version;
use time::OffsetDateTime;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{atomic::Ordering, RwLock};

use rpfm_ipc::settings_keys::*;
use rpfm_ipc::helpers::{ContainerInfo, DataSource, NewFile};
use rpfm_ipc::messages::CeoEntryData;

use rpfm_lib::files::{animpack, ContainerPath, FileType, loc, text, pack::*, portrait_settings, text::TextFormat};
use rpfm_lib::games::supported_games::*;
use rpfm_log::*;
use rpfm_lib::utils::*;

use rpfm_ui_common::utils::{create_grid_layout, find_widget, load_template};
use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::FULL_DATE_FORMAT;
use rpfm_ui_common::icons::IconType;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR, send_ipc_command, send_ipc_command_result, send_ipc_command_result_async, send_ipc_command_async};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::FIRST_GAME_CHANGE_DONE;
use crate::GAME_SELECTED;
use crate::global_search_ui::GlobalSearchUI;
use crate::NEW_FILE_VIEW_CREATED;
use crate::pack_tree::{BuildData, new_pack_file_tooltip, PackTree, TreeViewOperation};
use crate::packedfile_views::{anim_fragment_battle::*, animpack::*, anims_table::*, audio::FileAudioView, bmd::FileBMDView, decoder::*, dependencies_manager::*, esf::*, external::*, group_formations::*, image::*, matched_combat::*, FileView, packfile_settings::*, portrait_settings::PortraitSettingsView, rigidmodel::*, SpecialView, table::*, text::*, unit_variant::*, video::*, vmd::*};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::STATUS_BAR;
use crate::SUPPORTED_GAMES;
use crate::TREEVIEW_ICONS;
use crate::UI_STATE;
use crate::settings_ui::backend::*;
use crate::ui::GameSelectedIcons;
use crate::ui_state::OperationalMode;
use crate::utils::*;

#[cfg(feature = "support_model_renderer")]
use crate::packedfile_views::{View, ViewType};

#[cfg(feature = "support_uic")]
use crate::packedfile_views::uic::*;

const NEW_FILE_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_file_dialog.ui";
const NEW_FILE_VIEW_RELEASE: &str = "ui/new_file_dialog.ui";

const PACK_MAP_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/pack_map_dialog.ui";
const PACK_MAP_VIEW_RELEASE: &str = "ui/pack_map_dialog.ui";

const BUILD_STARPOS_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/build_starpos_view.ui";
const BUILD_STARPOS_VIEW_RELEASE: &str = "ui/build_starpos_view.ui";

const BUILD_CEO_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/build_ceo_view.ui";
const BUILD_CEO_VIEW_RELEASE: &str = "ui/build_ceo_view.ui";

const BUILD_CEO_BUILDER_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/build_ceo_builder_view.ui";
const BUILD_CEO_BUILDER_VIEW_RELEASE: &str = "ui/build_ceo_builder_view.ui";

const UPDATE_ANIM_IDS_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/update_anim_ids_dialog.ui";
const UPDATE_ANIM_IDS_VIEW_RELEASE: &str = "ui/update_anim_ids_dialog.ui";

const OPTIMIZER_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/optimizer_dialog.ui";
const OPTIMIZER_VIEW_RELEASE: &str = "ui/optimizer_dialog.ui";

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
    welcome_page_ui: crate::welcome_page_ui::WelcomePageUI,
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
    menu_bar_debug: QPtr<QMenu>,

    //-------------------------------------------------------------------------------//
    // Command Palette actions.
    //-------------------------------------------------------------------------------//
    command_palette_open_files: QPtr<QAction>,
    command_palette_open_commands: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
    packfile_new_packfile: QPtr<QAction>,
    packfile_open_packfiles: QPtr<QAction>,
    packfile_open_and_merge_packs: QPtr<QAction>,
    packfile_load_all_ca_packfiles: QPtr<QAction>,
    packfile_open_recent: QBox<QMenu>,
    packfile_open_from_content: QBox<QMenu>,
    packfile_open_from_secondary: QBox<QMenu>,
    packfile_open_from_data: QBox<QMenu>,
    packfile_open_from_autosave: QBox<QMenu>,
    packfile_close_pack_menu: QBox<QMenu>,
    packfile_save_pack_menu: QBox<QMenu>,
    packfile_save_pack_as_menu: QBox<QMenu>,
    packfile_save_pack_for_release: QBox<QMenu>,
    packfile_save_all: QPtr<QAction>,
    packfile_select_session: QPtr<QAction>,
    packfile_settings: QPtr<QAction>,
    packfile_quit: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `MyMod` menu.
    //-------------------------------------------------------------------------------//
    mymod_open_mymod_folder: QPtr<QAction>,
    mymod_new: QPtr<QAction>,

    mymod_open_pharaoh_dynasties: QPtr<QMenu>,
    mymod_open_pharaoh: QPtr<QMenu>,
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

    game_selected_pharaoh_dynasties: QPtr<QAction>,
    game_selected_pharaoh: QPtr<QAction>,
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
    // `Game Selected` menu (continued).
    //-------------------------------------------------------------------------------//
    game_selected_generate_dependencies_cache: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `Tools` menu.
    //-------------------------------------------------------------------------------//
    tools_faction_painter: QPtr<QAction>,
    tools_unit_editor: QPtr<QAction>,
    tools_translator: QPtr<QAction>,
    tools_ceo_builder: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
    about_about_qt: QPtr<QAction>,
    about_about_rpfm: QPtr<QAction>,
    about_check_updates: QPtr<QAction>,

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
    timer_server_status: QBox<QTimer>,

    tab_bar_packed_file_context_menu: QBox<QMenu>,
    tab_bar_packed_file_close: QPtr<QAction>,
    tab_bar_packed_file_close_all: QPtr<QAction>,
    tab_bar_packed_file_close_all_other: QPtr<QAction>,
    tab_bar_packed_file_close_all_left: QPtr<QAction>,
    tab_bar_packed_file_close_all_right: QPtr<QAction>,
    tab_bar_packed_file_prev: QPtr<QAction>,
    tab_bar_packed_file_next: QPtr<QAction>,
    tab_bar_packed_file_import_from_dependencies: QPtr<QAction>,
    tab_bar_packed_file_toggle_quick_notes: QPtr<QAction>,

    focused_widget: Rc<RwLock<Option<QPtr<QWidget>>>>,
    disabled_counter: Rc<RwLock<u32>>,
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
        QGuiApplication::set_window_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/rpfm.png", ASSETS_PATH.to_string_lossy()))));

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

        // Welcome widget, shown when no tabs are open.
        let welcome_page_ui = crate::welcome_page_ui::WelcomePageUI::new(&widget);
        layout.add_widget_5a(welcome_page_ui.welcome_widget(), 0, 0, 1, 1);
        tab_bar_packed_file.hide();
        welcome_page_ui.welcome_widget().show();

        let github_button = QPushButton::from_q_widget(&status_bar);
        github_button.set_flat(true);
        github_button.set_tool_tip(&qtr("github_link"));
        if is_dark_theme() {
            github_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
        } else {
            github_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github-dark.svg", ASSETS_PATH.to_string_lossy()))));
        }
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
        let shortcuts = QListOfQObject::new_0a();
        shortcut_collection_init_safe(&main_window.static_upcast::<qt_widgets::QWidget>().as_ptr(), shortcuts.as_ptr());

        // Create the Contextual Menu Actions.
        let tab_bar_packed_file_close = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_tab", "close_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all_left = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs_left", "close_tabs_to_left", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all_right = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs_right", "close_tabs_to_right", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all_other = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_other_tabs", "close_all_other_tabs", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_close_all = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "close_all_tabs", "close_all_tabs", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_prev = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "previus_tab", "prev_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_next = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "next_tab", "next_tab", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_import_from_dependencies = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "import_from_dependencies", "import_from_dependencies", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));
        let tab_bar_packed_file_toggle_quick_notes = add_action_to_menu(&tab_bar_packed_file_context_menu.static_upcast(), shortcuts.as_ref(), "file_tab", "toggle_quick_notes", "toggle_quick_notes", Some(tab_bar_packed_file.static_upcast::<qt_widgets::QWidget>()));

        tab_bar_packed_file_close.set_enabled(true);
        tab_bar_packed_file_close_all.set_enabled(true);
        tab_bar_packed_file_close_all_other.set_enabled(true);
        tab_bar_packed_file_close_all_left.set_enabled(true);
        tab_bar_packed_file_close_all_right.set_enabled(true);
        tab_bar_packed_file_prev.set_enabled(true);
        tab_bar_packed_file_next.set_enabled(true);
        tab_bar_packed_file_import_from_dependencies.set_enabled(true);
        tab_bar_packed_file_toggle_quick_notes.set_enabled(true);

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
        let menu_bar_tools = menu_bar.add_menu_q_string(&qtr("menu_bar_tools"));
        let menu_bar_about = menu_bar.add_menu_q_string(&qtr("menu_bar_about"));

        // This menu is hidden unless you enable it.
        let menu_bar_debug = menu_bar.add_menu_q_string(&qtr("menu_bar_debug"));
        menu_bar_debug.menu_action().set_visible(false);

        //-----------------------------------------------//
        // Command Palette actions (not in any menu, just shortcuts).
        //-----------------------------------------------//
        let command_palette_open_files = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "command_palette", "open_file_palette", "command_palette_open_files", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let command_palette_open_commands = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "command_palette", "open_command_palette", "command_palette_open_commands", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        // Remove these from the menu — they're only needed for their shortcuts.
        menu_bar_packfile.remove_action(&command_palette_open_files);
        menu_bar_packfile.remove_action(&command_palette_open_commands);

        //-----------------------------------------------//
        // `PackFile` Menu.
        //-----------------------------------------------//

        // Populate the `PackFile` menu.
        let packfile_new_packfile = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "new_pack", "new_packfile", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_open_packfiles = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "open_packs", "open_packs", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_open_and_merge_packs = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "open_and_merge_packs", "open_and_merge_packs", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_load_all_ca_packfiles = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "load_all_ca_packs", "load_all_ca_packfiles", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_save_all = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "save_all", "save_all", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        let packfile_open_recent = QMenu::from_q_string_q_widget(&qtr("open_recent"), &menu_bar_packfile);
        let packfile_open_from_content = QMenu::from_q_string_q_widget(&qtr("open_from_content"), &menu_bar_packfile);
        let packfile_open_from_secondary = QMenu::from_q_string_q_widget(&qtr("open_from_secondary"), &menu_bar_packfile);
        let packfile_open_from_data = QMenu::from_q_string_q_widget(&qtr("open_from_data"), &menu_bar_packfile);
        let packfile_open_from_autosave = QMenu::from_q_string_q_widget(&qtr("open_from_autosave"), &menu_bar_packfile);
        let packfile_close_pack_menu = QMenu::from_q_string_q_widget(&qtr("close_pack_menu"), &menu_bar_packfile);
        let packfile_save_pack_menu = QMenu::from_q_string_q_widget(&qtr("save_pack_menu"), &menu_bar_packfile);
        let packfile_save_pack_as_menu = QMenu::from_q_string_q_widget(&qtr("save_pack_as_menu"), &menu_bar_packfile);
        let packfile_save_pack_for_release = QMenu::from_q_string_q_widget(&qtr("save_pack_for_release"), &menu_bar_packfile);

        let packfile_select_session = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "select_session", "select_session", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_settings = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "settings", "settings", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let packfile_quit = add_action_to_menu(&menu_bar_packfile, shortcuts.as_ref(), "pack_menu", "quit", "quit", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_open_recent);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_open_from_content);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_open_from_secondary);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_open_from_data);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_open_from_autosave);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_close_pack_menu);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_save_pack_menu);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_save_pack_as_menu);
        menu_bar_packfile.insert_menu(&packfile_save_all, &packfile_save_pack_for_release);

        menu_bar_packfile.insert_separator(&packfile_open_recent.menu_action());
        menu_bar_packfile.insert_separator(&packfile_close_pack_menu.menu_action());
        menu_bar_packfile.insert_separator(&packfile_save_pack_menu.menu_action());
        menu_bar_packfile.insert_separator(&packfile_select_session);

        //-----------------------------------------------//
        // `MyMod` Menu.
        //-----------------------------------------------//
        let mymod_open_mymod_folder = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "open_mymod_folder", "mymod_open_mymod_folder", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let mymod_new = add_action_to_menu(&menu_bar_mymod, shortcuts.as_ref(), "mymod_menu", "new_mymod", "mymod_new", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        menu_bar_mymod.add_separator();

        let mymod_open_pharaoh_dynasties = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_PHARAOH_DYNASTIES));
        let mymod_open_pharaoh = menu_bar_mymod.add_menu_q_string(&QString::from_std_str(DISPLAY_NAME_PHARAOH));
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

        mymod_open_pharaoh_dynasties.menu_action().set_visible(false);
        mymod_open_pharaoh.menu_action().set_visible(false);
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
        let game_selected_generate_dependencies_cache = add_action_to_menu(&menu_bar_game_selected, shortcuts.as_ref(), "game_selected_menu", "generate_dependencies_cache", "game_selected_generate_dependencies_cache", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

        let game_selected_pharaoh_dynasties = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_PHARAOH_DYNASTIES));
        let game_selected_pharaoh = menu_bar_game_selected.add_action_q_string(&QString::from_std_str(DISPLAY_NAME_PHARAOH));
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

        game_selected_pharaoh_dynasties.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH_DYNASTIES).unwrap().icon_small()))).as_ref());
        game_selected_pharaoh.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH).unwrap().icon_small()))).as_ref());
        game_selected_warhammer_3.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_small()))).as_ref());
        game_selected_troy.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_small()))).as_ref());
        game_selected_three_kingdoms.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_small()))).as_ref());
        game_selected_warhammer_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_small()))).as_ref());
        game_selected_warhammer.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_small()))).as_ref());
        game_selected_thrones_of_britannia.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_small()))).as_ref());
        game_selected_attila.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_small()))).as_ref());
        game_selected_rome_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_small()))).as_ref());
        game_selected_shogun_2.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_small()))).as_ref());
        game_selected_napoleon.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_small()))).as_ref());
        game_selected_empire.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_small()))).as_ref());
        game_selected_arena.set_icon(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_small()))).as_ref());

        let game_selected_group = QActionGroup::new(&menu_bar_game_selected);

        // Configure the `Game Selected` Menu.
        menu_bar_game_selected.insert_separator(&game_selected_pharaoh_dynasties);
        menu_bar_game_selected.insert_separator(&game_selected_arena);
        game_selected_group.add_action_q_action(&game_selected_pharaoh_dynasties);
        game_selected_group.add_action_q_action(&game_selected_pharaoh);
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
        game_selected_pharaoh_dynasties.set_checkable(true);
        game_selected_pharaoh.set_checkable(true);
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
        // `Tools` Menu.
        //-----------------------------------------------//

        // Populate the `Tools` menu.
        let tools_faction_painter = menu_bar_tools.add_action_q_string(&qtr("tools_faction_painter"));
        let tools_unit_editor = menu_bar_tools.add_action_q_string(&qtr("tools_unit_editor"));
        let tools_translator = menu_bar_tools.add_action_q_string(&qtr("tools_translator"));
        let tools_ceo_builder = menu_bar_tools.add_action_q_string(&qtr("tools_ceo_builder"));
        if !settings_bool(ENABLE_UNIT_EDITOR) {
            tools_unit_editor.set_enabled(false);
        }

        //-----------------------------------------------//
        // `About` Menu.
        //-----------------------------------------------//
        let about_about_qt = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "about_qt", "about_about_qt", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_about_rpfm = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "about_rpfm", "about_about_rpfm", Some(main_window.static_upcast::<qt_widgets::QWidget>()));
        let about_check_updates = add_action_to_menu(&menu_bar_about, shortcuts.as_ref(), "about_menu", "check_updates", "about_check_updates", Some(main_window.static_upcast::<qt_widgets::QWidget>()));

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

        let timer_server_status = QTimer::new_1a(&main_window);
        timer_server_status.set_interval(5000);
        timer_server_status.start_0a();

        // Create ***Da monsta***.
        AppUI {

            //-------------------------------------------------------------------------------//
            // Main Window.
            //-------------------------------------------------------------------------------//
            main_window,
            tab_bar_packed_file,
            welcome_page_ui,
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
            menu_bar_debug,

            command_palette_open_files,
            command_palette_open_commands,

            //-------------------------------------------------------------------------------//
            // "PackFile" menu.
            //-------------------------------------------------------------------------------//

            // Menus.
            packfile_new_packfile,
            packfile_open_packfiles,
            packfile_open_and_merge_packs,
            packfile_load_all_ca_packfiles,
            packfile_open_recent,
            packfile_open_from_content,
            packfile_open_from_secondary,
            packfile_open_from_data,
            packfile_open_from_autosave,
            packfile_close_pack_menu,
            packfile_save_pack_menu,
            packfile_save_pack_as_menu,
            packfile_save_pack_for_release,
            packfile_save_all,
            packfile_select_session,
            packfile_settings,
            packfile_quit,

            //-------------------------------------------------------------------------------//
            // `MyMod` menu.
            //-------------------------------------------------------------------------------//
            mymod_open_mymod_folder,
            mymod_new,

            mymod_open_pharaoh_dynasties,
            mymod_open_pharaoh,
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

            game_selected_pharaoh_dynasties,
            game_selected_pharaoh,
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

            game_selected_generate_dependencies_cache,

            //-------------------------------------------------------------------------------//
            // "Tools" menu.
            //-------------------------------------------------------------------------------//
            tools_faction_painter,
            tools_unit_editor,
            tools_translator,
            tools_ceo_builder,

            //-------------------------------------------------------------------------------//
            // "About" menu.
            //-------------------------------------------------------------------------------//
            about_about_qt,
            about_about_rpfm,
            about_check_updates,

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
            timer_server_status,

            tab_bar_packed_file_context_menu,
            tab_bar_packed_file_close,
            tab_bar_packed_file_close_all,
            tab_bar_packed_file_close_all_other,
            tab_bar_packed_file_close_all_left,
            tab_bar_packed_file_close_all_right,
            tab_bar_packed_file_prev,
            tab_bar_packed_file_next,
            tab_bar_packed_file_import_from_dependencies,
            tab_bar_packed_file_toggle_quick_notes,

            focused_widget: Rc::new(RwLock::new(None)),
            disabled_counter: Rc::new(RwLock::new(0)),
        }
    }

    /// This function toggles visibility between the welcome widget and the tab widget.
    pub unsafe fn toggle_welcome_visibility(&self) {
        self.welcome_page_ui.toggle_visibility(&self.tab_bar_packed_file);
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
    pub unsafe fn are_you_sure(app_ui: &Rc<Self>, is_delete_my_mod: bool, is_full_close: bool) -> bool {
        are_you_sure(app_ui.main_window.as_mut_raw_ptr(), is_delete_my_mod, is_full_close)
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

        for file_view in UI_STATE.get_open_packedfiles().iter() {
            file_view.save(app_ui, pack_file_contents_ui)?;
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

        for file_view in UI_STATE.get_open_packedfiles().iter() {
            if save_before_deleting && file_view.path_copy() != RESERVED_NAME_EXTRA_PACKFILE {
                file_view.save(app_ui, pack_file_contents_ui)?;
            }
            let widget = file_view.main_widget();
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

    /// This function deletes all the widgets corresponding to opened PackedFiles from the local Pack.
    #[must_use = "If one of those mysterious save errors happen here and we don't use the result, we may be losing the new changes to a file."]
    pub unsafe fn purge_the_local_ones(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        save_before_deleting: bool,
    ) -> Result<()> {

        let mut delete_indexes = vec![];
        for (file_index, file_view) in UI_STATE.get_open_packedfiles().iter().enumerate() {
            if file_view.data_source() == DataSource::PackFile {
                if save_before_deleting && file_view.path_copy() != RESERVED_NAME_EXTRA_PACKFILE {
                    file_view.save(app_ui, pack_file_contents_ui)?;
                }
                let widget = file_view.main_widget();
                let index = app_ui.tab_bar_packed_file.index_of(widget);
                if index != -1 {
                    app_ui.tab_bar_packed_file.remove_tab(index);
                }

                // Delete the widget manually to free memory.
                widget.delete_later();
                delete_indexes.push(file_index);
            }
        }

        // Remove all open PackedFiles and their slots.
        delete_indexes.reverse();
        for index in &delete_indexes {
            UI_STATE.set_open_packedfiles().remove(*index);
        }

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
        let position = UI_STATE.get_open_packedfiles().iter().position(|x| *x.path_read() == path && x.data_source() == data_source);
        if let Some(position) = position {
            if let Some(file_view) = UI_STATE.get_open_packedfiles().get(position) {

                // Do not try saving PackFiles.
                if save_before_deleting && !path.starts_with(RESERVED_NAME_EXTRA_PACKFILE) {
                    did_it_worked = file_view.save(app_ui, pack_file_contents_ui);
                }
                let widget = file_view.main_widget();
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
                    for path in UI_STATE.get_open_packedfiles().iter().map(|x| x.path_read()) {
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

    /// This function opens the PackFile(s) at the provided Path, and sets all the stuff needed, depending on the situation.
    ///
    /// NOTE: The `game_folder` is for when using this function with *MyMods*. If you're opening a normal mod, pass it empty.
    /// NOTE: When `additive` is true, packs are added alongside any already-open packs instead of replacing them.
    /// When additive and no packs are currently open, game selection logic is still applied.
    pub unsafe fn open_packfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        pack_file_paths: &[PathBuf],
        game_folder: &str,
        additive: bool,
    ) -> Result<()> {

        // Destroy whatever it's in the PackedFile's view, to avoid data corruption. We don't care about this result.
        // Only needed when replacing packs, not when adding alongside existing ones.
        if !additive {
            let _ = Self::purge_them_all(app_ui, pack_file_contents_ui, false);
        }

        app_ui.toggle_main_window(false);

        // Track all opened files in the recent file list.
        for pack_file_path in pack_file_paths {
            if let Some(path_str) = pack_file_path.to_str() {
                let mut paths = settings_vec_string(RECENT_FILE_LIST);
                if let Some(pos) = paths.iter().position(|x| x == path_str) {
                    paths.remove(pos);
                }
                paths.insert(0, path_str.to_owned());
                while paths.len() > 10 {
                    paths.pop();
                }
                let _ = settings_set_vec_string(RECENT_FILE_LIST, &paths);
            }
        }

        let timer = settings_i32(AUTOSAVE_INTERVAL);
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Open the packs and update the tree view.
        // In additive mode, each pack is opened individually and added to the tree.
        // In non-additive mode, close all existing packs on the backend first, then open the new ones.
        if !additive {
            send_ipc_command(Command::CloseAllPacks, response_extractor!());
        }

        if additive {
            for pack_file_path in pack_file_paths {
                let (pack_key, _) = match send_ipc_command_result_async(Command::OpenPackFiles(vec![pack_file_path.clone()]), response_extractor!(Response::StringContainerInfo, v1, v2)) {
                    Ok(result) => result,
                    Err(error) => {
                        app_ui.toggle_main_window(true);
                        return Err(error);
                    }
                };
                let mut build_data = BuildData::new();
                build_data.editable = true;
                build_data.is_mymod = !game_folder.is_empty();
                build_data.pack_key = Some(pack_key.clone());
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::AddPack(build_data), DataSource::PackFile, &pack_key);
            }
        } else {
            let (pack_key, _) = match send_ipc_command_result_async(Command::OpenPackFiles(pack_file_paths.to_vec()), response_extractor!(Response::StringContainerInfo, v1, v2)) {
                Ok(result) => result,
                Err(error) => {
                    app_ui.toggle_main_window(true);
                    return Err(error);
                }
            };
            let mut build_data = BuildData::new();
            build_data.editable = true;
            build_data.is_mymod = !game_folder.is_empty();
            build_data.pack_key = Some(pack_key.clone());
            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile, &pack_key);
        }

        Self::enable_packfile_actions(app_ui, true);

        // Close the Global Search stuff and reset the filter's history.
        GlobalSearchUI::clear(global_search_ui);
        global_search_ui.update_pack_sources(pack_file_contents_ui);

        // Operational mode logic: mark the pack as MyMod on the server when opened from the MyMod menu.
        if !game_folder.is_empty() && pack_file_paths.len() == 1 {
            let path = &pack_file_paths[0];
            let pack_key = path.to_string_lossy().to_string();
            let mod_name = path.file_name().unwrap().to_string_lossy().to_string();
            let game_folder_name = path.parent().and_then(|p| p.file_name()).map(|f| f.to_string_lossy().to_string()).unwrap_or_default();

            send_ipc_command(Command::SetPackOperationalMode(pack_key, OperationalMode::MyMod(game_folder_name, mod_name)), response_extractor!());
        }

        // Rebuild parent packs in the dependencies so they reflect the newly opened pack.
        Self::rebuild_parent_packs(app_ui, pack_file_contents_ui, dependencies_ui);

        if additive {
            UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
        } else {
            UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile, "");
        }

        // Re-enable the Main Window.
        app_ui.toggle_main_window(true);

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
        optimize: bool
    ) -> Result<()> {
        Self::save_packfile_by_key(app_ui, pack_file_contents_ui, None, save_as, optimize)
    }

    /// This function is used to save a specific `PackFile` to disk, identified by pack key.
    ///
    /// If `target_pack_key` is `None`, it uses the selected/first pack.
    /// If the PackFile doesn't exist or we pass `save_as = true`,
    /// it opens a dialog asking for a path.
    pub unsafe fn save_packfile_by_key(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        target_pack_key: Option<String>,
        save_as: bool,
        optimize: bool
    ) -> Result<()> {

        let mut result: Result<()> = Ok(());
        app_ui.toggle_main_window(false);

        // Resolve the pack key once, up front.
        let pack_key = match target_pack_key.or_else(|| pack_file_contents_ui.pack_key_from_selection_or_first()) {
            Some(key) => key,
            None => {
                app_ui.toggle_main_window(true);
                return Err(anyhow!("No pack is open."));
            }
        };

        // First, we need to save all open `PackedFiles` to the backend. If one fails, we want to know what one.
        AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui)?;

        if optimize {
            let _ = AppUI::purge_them_all(app_ui, pack_file_contents_ui, true);

            let options = optimizer_options();
            match send_ipc_command_result_async(Command::OptimizePackFile(pack_key.clone(), options), response_extractor!(Response::HashSetStringHashSetString, v1, v2)) {
                Ok((response_1, response_2)) => {
                    let response_1 = response_1.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<ContainerPath>>();
                    let response_2 = response_2.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<ContainerPath>>();
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(response_1, true), DataSource::PackFile, &pack_key);
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(response_2), DataSource::PackFile, &pack_key);
                }
                Err(error) => show_dialog(&app_ui.main_window, error, false),
            }
        }

        let mut path = match send_ipc_command_result(Command::GetPackFilePath(pack_key.clone()), response_extractor!(Response::PathBuf)) {
            Ok(path) => path,
            Err(error) => {
                app_ui.toggle_main_window(true);
                return Err(error);
            }
        };
        if !path.is_file() || save_as {

            // Create the FileDialog to save the PackFile and configure it.
            let file_dialog = QFileDialog::from_q_widget_q_string(
                &app_ui.main_window,
                &qtr("save_pack_as_menu"),
            );
            file_dialog.set_accept_mode(qt_widgets::q_file_dialog::AcceptMode::AcceptSave);
            file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
            file_dialog.set_default_suffix(&QString::from_std_str("pack"));
            file_dialog.select_file(&QString::from_std_str(path.file_name().unwrap_or_else(|| OsStr::new("mod.pack")).to_string_lossy()));

            // If we are saving an existing PackFile with another name, we start in his current path.
            if path.is_file() {
                path.pop();
                file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref()));
            }

            // In case we have a default path for the Game Selected and that path is valid,
            // we use his data folder as base path for saving our PackFile.
            else if let Ok(ref path) = GAME_SELECTED.read().unwrap().local_mods_path(&settings_path_buf(GAME_SELECTED.read().unwrap().key())) {
                if path.is_dir() { file_dialog.set_directory_q_string(&QString::from_std_str(path.to_string_lossy().as_ref())); }
            }

            // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
            if file_dialog.exec() == 1 {
                let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                match send_ipc_command_result_async(Command::SavePackAs(pack_key.clone(), path), response_extractor!(Response::ContainerInfo)) {
                    Ok(pack_file_info) => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile, &pack_key);
                        let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                        packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                        packfile_item.set_text(&QString::from_std_str(file_name));

                        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                    }
                    Err(error) => result = Err(error),
                }
            }
        }

        else {
            match send_ipc_command_result_async(Command::SavePack(pack_key.clone()), response_extractor!(Response::ContainerInfo)) {
                Ok(pack_file_info) => {
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile, &pack_key);
                    let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                    packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                    UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);
                }
                Err(error) => result = Err(error),
            }
        }

        // Clean the treeview and the views from markers.
        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile, &pack_key);

        for file_view in UI_STATE.get_open_packedfiles().iter() {
            file_view.clean();
        }

        // Then we re-enable the main Window and return whatever we've received.
        app_ui.toggle_main_window(true);
        result
    }

    /// This function closes a pack identified by `pack_key`, removing it from the backend,
    /// the tree view, any open file tabs, and the global search sources.
    pub unsafe fn close_pack(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_key: &str,
    ) {
        let _ = CENTRAL_COMMAND.read().unwrap().send(Command::ClosePack(pack_key.to_string()));
        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::RemovePack(pack_key.to_string()), DataSource::PackFile, pack_key);
        global_search_ui.update_pack_sources(pack_file_contents_ui);

        // Close any open file views belonging to this pack.
        let mut tabs_to_close = vec![];
        {
            let open_packedfiles = UI_STATE.get_open_packedfiles();
            for file_view in open_packedfiles.iter() {
                if file_view.pack_key_copy() == pack_key {
                    let widget = file_view.main_widget();
                    let index = app_ui.tab_bar_packed_file.index_of(widget);
                    if index != -1 {
                        tabs_to_close.push(index);
                    }
                }
            }
        }

        tabs_to_close.sort_unstable();
        tabs_to_close.reverse();

        for index in tabs_to_close {
            app_ui.tab_bar_packed_file.remove_tab(index);
        }

        // Also remove them from the open file views list.
        UI_STATE.set_open_packedfiles().retain(|v| v.pack_key_copy() != pack_key);

        // If no packs remain, disable pack-dependent actions.
        let remaining = send_ipc_command(Command::ListOpenPacks, response_extractor!(Response::VecStringContainerInfo));
        if remaining.is_empty() {
            Self::enable_packfile_actions(app_ui, false);
        }

        GameSelectedIcons::set_game_selected_icon(app_ui);
    }

    /// This function enables/disables the actions on the main window, depending on the current state of the Application.
    ///
    /// You have to pass `enable = true` if you are trying to enable actions, and `false` to disable them.
    pub unsafe fn enable_packfile_actions(app_ui: &Rc<Self>, enable: bool) {

        // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
        let game_selected = GAME_SELECTED.read().unwrap().clone();
        if game_selected.key() == KEY_ARENA {

            // Disable the actions that allow to create and save PackFiles.
            app_ui.packfile_new_packfile.set_enabled(false);
            app_ui.packfile_close_pack_menu.set_enabled(true);
            app_ui.packfile_save_pack_menu.set_enabled(false);
            app_ui.packfile_save_pack_as_menu.set_enabled(false);
            app_ui.packfile_save_pack_for_release.set_enabled(false);
            app_ui.packfile_save_all.set_enabled(false);

            // This one too, though we had to deal with it specially later on.
            app_ui.mymod_new.set_enabled(false);
        }

        // Otherwise...
        else {

            // Enable or disable the actions from "PackFile" Submenu.
            app_ui.packfile_new_packfile.set_enabled(true);
            app_ui.packfile_close_pack_menu.set_enabled(enable);
            app_ui.packfile_save_pack_menu.set_enabled(enable);
            app_ui.packfile_save_pack_as_menu.set_enabled(enable);
            app_ui.packfile_save_pack_for_release.set_enabled(enable);
            app_ui.packfile_save_all.set_enabled(enable);

            // If there is a "MyMod" path set in the settings...
            let path = settings_path_buf(MYMOD_BASE_PATH);
            if path.is_dir() { app_ui.mymod_new.set_enabled(true); }
            else { app_ui.mymod_new.set_enabled(false); }
        }

        // If we are disabling...
        if !enable {
            app_ui.game_selected_generate_dependencies_cache.set_enabled(false);
        }

        // Dependencies generation should be enabled for the current game.
        app_ui.game_selected_generate_dependencies_cache.set_enabled(true);

        // The assembly kit thing should only be available for Rome 2 and later games.
        match game_selected.key() {
            KEY_PHARAOH_DYNASTIES |
            KEY_PHARAOH |
            KEY_WARHAMMER_3 |
            KEY_TROY |
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(true);
            },
            _ => {
                app_ui.game_selected_open_game_assembly_kit_folder.set_enabled(false);
            },
        }
    }

    /// This function takes care of recreating the dynamic submenus under `PackFile` menu.
    pub unsafe fn build_pack_submenus(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) {

        // First, we clear both menus, so we can rebuild them properly.
        app_ui.packfile_open_recent.clear();
        app_ui.packfile_open_from_content.clear();
        app_ui.packfile_open_from_secondary.clear();
        app_ui.packfile_open_from_data.clear();
        app_ui.packfile_open_from_autosave.clear();
        app_ui.packfile_close_pack_menu.clear();
        app_ui.packfile_save_pack_menu.clear();
        app_ui.packfile_save_pack_as_menu.clear();
        app_ui.packfile_save_pack_for_release.clear();

        //---------------------------------------------------------------------------------------//
        // Build the menus...
        //---------------------------------------------------------------------------------------//

        // Recent PackFiles.
        let recent_file_paths = settings_vec_string(RECENT_FILE_LIST);
        if !recent_file_paths.is_empty() {

            for path_str in recent_file_paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let path = PathBuf::from(&path_str);
                if path.is_file() {
                    let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                    let open_mod_action = app_ui.packfile_open_recent.add_action_q_string(&QString::from_std_str(mod_name));

                    // Create the slot for that action.
                    let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                        app_ui,
                        pack_file_contents_ui,
                        dependencies_ui,
                        global_search_ui,
                        diagnostics_ui,
                        path => move |_| {
                        if Self::are_you_sure(&app_ui, false, false) {
                            if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                                return show_dialog(&app_ui.main_window, error, false);
                            }

                            if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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
        let mut content_paths = GAME_SELECTED.read().unwrap().content_packs_paths(&settings_path_buf(GAME_SELECTED.read().unwrap().key()));
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
                    dependencies_ui,
                    global_search_ui,
                    diagnostics_ui,
                    path => move |_| {
                    if Self::are_you_sure(&app_ui, false, false) {
                        if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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

        // Get the path of every PackFile in the secondary folder (if it's configured) and make an action for each one of them.
        let mut secondary_paths = GAME_SELECTED.read().unwrap().secondary_packs_paths(&settings_path_buf(SECONDARY_PATH));
        if let Some(ref mut paths) = secondary_paths {
            paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
            for path in paths {

                // That means our file is a valid PackFile and it needs to be added to the menu.
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                let open_mod_action = app_ui.packfile_open_from_secondary.add_action_q_string(&QString::from_std_str(mod_name));

                // Create the slot for that action.
                let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                    app_ui,
                    pack_file_contents_ui,
                    dependencies_ui,
                    global_search_ui,
                    diagnostics_ui,
                    path => move |_| {
                    if Self::are_you_sure(&app_ui, false, false) {
                        if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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
        let mut data_paths = GAME_SELECTED.read().unwrap().data_packs_paths(&settings_path_buf(GAME_SELECTED.read().unwrap().key()));
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
                    dependencies_ui,
                    global_search_ui,
                    diagnostics_ui,
                    path => move |_| {
                    if Self::are_you_sure(&app_ui, false, false) {
                        if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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
            if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() > 0 {
                let pack_name = pack_file_contents_ui.packfile_contents_tree_model().index_2a(0, 0).data_0a().to_string().to_std_string();
                let pack_autosave_paths = autosave_paths.join(pack_name);
                if pack_autosave_paths.is_dir() {
                    let autosave_paths = files_in_folder_from_newest_to_oldest(&pack_autosave_paths);
                    if let Ok(ref paths) = autosave_paths {
                        for path in paths {

                            // That means our file is a valid PackFile and it needs to be added to the menu.
                            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                            let mod_name_no_pack = mod_name.replace(".pack", "");
                            if let Ok(date_numeric) = mod_name_no_pack.parse::<i64>() {
                                if let Ok(date_formatted) = OffsetDateTime::from_unix_timestamp(date_numeric) {
                                    let date_formatted = date_formatted.format(&FULL_DATE_FORMAT).unwrap();
                                    let open_mod_action = app_ui.packfile_open_from_autosave.add_action_q_string(&QString::from_std_str(date_formatted));

                                    // Create the slot for that action.
                                    let slot_open_mod = SlotOfBool::new(&open_mod_action, clone!(
                                        app_ui,
                                        pack_file_contents_ui,
                                        dependencies_ui,
                                        global_search_ui,
                                        diagnostics_ui,
                                        path => move |_| {
                                        if Self::are_you_sure(&app_ui, false, false) {
                                            if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                                                return show_dialog(&app_ui.main_window, error, false);
                                            }

                                            if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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
                    }
                }
            }
        }

        // Query the server for all open packs.
        let pack_list = send_ipc_command(Command::ListOpenPacks, response_extractor!(Response::VecStringContainerInfo));

        for (pack_key, _container_info) in &pack_list {

            // Close Pack action.
            let close_action = app_ui.packfile_close_pack_menu.add_action_q_string(&QString::from_std_str(pack_key));
            let slot_close = SlotOfBool::new(&close_action, clone!(
                app_ui,
                pack_file_contents_ui,
                global_search_ui,
                pack_key => move |_| {
                    Self::close_pack(&app_ui, &pack_file_contents_ui, &global_search_ui, &pack_key);
                }
            ));
            close_action.triggered().connect(&slot_close);

            // Save Pack action (per-pack).
            let save_action = app_ui.packfile_save_pack_menu.add_action_q_string(&QString::from_std_str(pack_key));
            let slot_save = SlotOfBool::new(&save_action, clone!(
                app_ui,
                pack_file_contents_ui,
                pack_key => move |_| {
                    if let Err(error) = Self::save_packfile_by_key(&app_ui, &pack_file_contents_ui, Some(pack_key.clone()), false, false) {
                        show_dialog(&app_ui.main_window, error, false);
                    }
                }
            ));
            save_action.triggered().connect(&slot_save);

            // Save Pack As action (per-pack).
            let save_as_action = app_ui.packfile_save_pack_as_menu.add_action_q_string(&QString::from_std_str(pack_key));
            let slot_save_as = SlotOfBool::new(&save_as_action, clone!(
                app_ui,
                pack_file_contents_ui,
                pack_key => move |_| {
                    if let Err(error) = Self::save_packfile_by_key(&app_ui, &pack_file_contents_ui, Some(pack_key.clone()), true, false) {
                        show_dialog(&app_ui.main_window, error, false);
                    }
                }
            ));
            save_as_action.triggered().connect(&slot_save_as);

            // Save Pack For Release action (per-pack).
            let save_for_release_action = app_ui.packfile_save_pack_for_release.add_action_q_string(&QString::from_std_str(pack_key));
            let slot_save_for_release = SlotOfBool::new(&save_for_release_action, clone!(
                app_ui,
                pack_file_contents_ui,
                pack_key => move |_| {
                    if let Err(error) = Self::save_packfile_by_key(&app_ui, &pack_file_contents_ui, Some(pack_key.clone()), false, true) {
                        show_dialog(&app_ui.main_window, error, false);
                    }
                }
            ));
            save_for_release_action.triggered().connect(&slot_save_for_release);
        }

        app_ui.packfile_open_recent.menu_action().set_visible(!app_ui.packfile_open_recent.actions().is_empty());
        app_ui.packfile_open_from_content.menu_action().set_visible(!app_ui.packfile_open_from_content.actions().is_empty());
        app_ui.packfile_open_from_secondary.menu_action().set_visible(!app_ui.packfile_open_from_secondary.actions().is_empty());
        app_ui.packfile_open_from_data.menu_action().set_visible(!app_ui.packfile_open_from_data.actions().is_empty());
        app_ui.packfile_open_from_autosave.menu_action().set_visible(!app_ui.packfile_open_from_autosave.actions().is_empty());

        let has_packs = !pack_list.is_empty();
        app_ui.packfile_close_pack_menu.set_enabled(has_packs);
        app_ui.packfile_save_pack_menu.set_enabled(has_packs);
        app_ui.packfile_save_pack_as_menu.set_enabled(has_packs);
        app_ui.packfile_save_pack_for_release.set_enabled(has_packs);
    }

    /// This function takes care of the re-creation of the `MyMod` list for each game.
    pub unsafe fn build_open_mymod_submenus(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) {

        // First, we need to reset the menu, which basically means deleting all the game submenus and hiding them.
        app_ui.mymod_open_pharaoh_dynasties.menu_action().set_visible(false);
        app_ui.mymod_open_pharaoh.menu_action().set_visible(false);
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

        app_ui.mymod_open_pharaoh_dynasties.clear();
        app_ui.mymod_open_pharaoh.clear();
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
        let mymod_base_path = settings_path_buf(MYMOD_BASE_PATH);
        if mymod_base_path.is_dir() {
            if let Ok(game_folder_list) = mymod_base_path.read_dir() {
                for game_folder in game_folder_list.flatten() {

                    // If it's a valid folder, and it's in our supported games list, get all the PackFiles inside it and create an open action for them.
                    let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                    let is_supported = SUPPORTED_GAMES.games().iter().filter_map(|x| if *x.supports_editing() { Some(x.key()) } else { None }).any(|x| x == game_folder_name);
                    if game_folder.path().is_dir() && is_supported {
                        let game_submenu = match &*game_folder_name {
                            KEY_PHARAOH_DYNASTIES => &app_ui.mymod_open_pharaoh_dynasties,
                            KEY_PHARAOH => &app_ui.mymod_open_pharaoh,
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
                                        dependencies_ui,
                                        global_search_ui,
                                        diagnostics_ui,
                                        game_folder_name => move |_| {
                                        if Self::are_you_sure(&app_ui, false, false) {
                                            if let Err(error) = Self::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[pack_file.to_path_buf()], &game_folder_name, true) {
                                                return show_dialog(&app_ui.main_window, error, false);
                                            }

                                            if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {

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
                        if game_submenu.actions().count() > 0 {
                            game_submenu.menu_action().set_visible(true);
                        }
                    }
                }
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
                for file_view in UI_STATE.get_open_packedfiles().iter() {
                    let open_path = file_view.path_read();
                    let index = app_ui.tab_bar_packed_file.index_of(file_view.main_widget());
                    if (data_source != file_view.data_source() ||
                        (data_source == file_view.data_source() && *open_path != *path)) &&
                        file_view.is_preview() && index != -1 {

                        // If they're a rigid view, we need to pause their rendering.
                        #[cfg(feature = "support_model_renderer")] {
                            if let ViewType::Internal(View::RigidModel(view)) = file_view.view_type() {
                                crate::ffi::pause_rendering(&view.renderer().as_ptr());
                            } else if let ViewType::Internal(View::VMD(view)) = file_view.view_type() {
                                crate::ffi::pause_rendering(&view.renderer().as_ptr());
                            } else if let ViewType::Internal(View::WSModel(view)) = file_view.view_type() {
                                crate::ffi::pause_rendering(&view.renderer().as_ptr());
                            }

                        }

                        app_ui.tab_bar_packed_file.remove_tab(index);
                    }
                }

                // If the file we want to open is already open, or it's hidden, we show it/focus it, instead of opening it again.
                // If it was a preview, then we mark it as full. Index == -1 means it's not in a tab.
                if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *path && x.data_source() == data_source) {
                    if !is_external {
                        let index = app_ui.tab_bar_packed_file.index_of(tab_widget.main_widget());

                        // If we're trying to open as preview something already open as full, we don't do anything.
                        if !(index != -1 && is_preview && !tab_widget.is_preview()) {
                            tab_widget.set_is_preview(is_preview);
                        }

                        if index == -1 {
                            let icon_type = IconType::File(path.to_owned());
                            let icon = TREEVIEW_ICONS.icon(icon_type);
                            app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.main_widget(), icon, &QString::from_std_str(""));
                        }

                        // If they're a rigid view, we need to pause their rendering.
                        #[cfg(feature = "support_model_renderer")] {
                            if let ViewType::Internal(View::RigidModel(view)) = tab_widget.view_type() {
                                crate::ffi::resume_rendering(&view.renderer().as_ptr());
                            } else if let ViewType::Internal(View::VMD(view)) = tab_widget.view_type() {
                                crate::ffi::resume_rendering(&view.renderer().as_ptr());
                            } else if let ViewType::Internal(View::WSModel(view)) = tab_widget.view_type() {
                                crate::ffi::resume_rendering(&view.renderer().as_ptr());
                            }
                        }

                        app_ui.tab_bar_packed_file.set_current_widget(tab_widget.main_widget());
                        Self::update_views_names(app_ui);
                        return;
                    }
                }

                // If we have a PackedFile open, but we want to open it as a external file, close it here.
                if is_external && UI_STATE.get_open_packedfiles().iter().any(|x| *x.path_read() == *path && x.data_source() == data_source) {
                    if let Err(error) = Self::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, data_source, true) {
                        show_dialog(&app_ui.main_window, error, false);
                    }
                }

                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let mut tab = FileView::new(path, &pack_key);
                tab.main_widget().set_parent(&app_ui.tab_bar_packed_file);
                tab.main_widget().set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

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
                    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::DecodePackedFile(pack_key.clone(), path.to_string(), tab.data_source()));

                    tab.set_is_preview(is_preview);
                    let icon_type = IconType::File(path.to_owned());
                    let icon = TREEVIEW_ICONS.icon(icon_type);

                    // If we're here, it's always a new file view. The line next to this one disables the variable.
                    NEW_FILE_VIEW_CREATED.store(true, std::sync::atomic::Ordering::SeqCst);
                    let tab_index = app_ui.tab_bar_packed_file.add_tab_3a(tab.main_widget(), icon, &QString::from_std_str(""));
                    let response = CentralCommand::recv(&receiver);
                    match response {

                        Response::AnimFragmentBattleRFileInfo(data, ref file_info) => {
                            let file_info = file_info.clone();
                            match FileAnimFragmentBattleView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        },

                        Response::AnimPackRFileInfo(files_info, file_info) => {
                            match PackedFileAnimPackView::new_view(&mut tab, app_ui, pack_file_contents_ui, &file_info, &files_info) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },

                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        },

                        Response::AnimsTableRFileInfo(data, ref file_info) => {
                            let file_info = file_info.clone();
                            match FileAnimsTableDebugView::new_view(&mut tab, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        },

                        Response::AtlasRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::AudioRFileInfo(data, file_info) => {
                            match FileAudioView::new_view(&mut tab, &data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                }
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::BmdRFileInfo(data, file_info) => {
                            match FileBMDView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                }
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        // If the file is a DB PackedFile...
                        Response::DBRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);

                                    // Try to get the data of the table to send it for decoding.
                                    /*let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::GetPackedFileRawData(path.to_owned()));
                                    let response = CentralCommand::recv(&receiver);
                                    let data = match response {
                                        Response::VecU8(data) => data,
                                        Response::Error(_) => return show_dialog(&app_ui.main_window, error, false),
                                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                                    };*/

                                    return show_dialog_decode_button(app_ui.main_window.static_upcast::<qt_widgets::QWidget>().as_ptr(), error);
                                },
                            }
                        }

                        Response::ESFRFileInfo(data, file_info) => {
                            match PackedFileESFView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::GroupFormationsRFileInfo(data, file_info) => {
                            let file_info = file_info.clone();
                            match FileGroupFormationsDebugView::new_view(&mut tab, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        // If the file is a Image PackedFile, ignore failures while opening.
                        Response::ImageRFileInfo(data, file_info) => {
                            match PackedFileImageView::new_view(&mut tab, &data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                }
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        // If the file is a Loc PackedFile...
                        Response::LocRFileInfo(_, ref file_info) => {
                            let file_info = file_info.clone();
                            match PackedFileTableView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, response) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::MatchedCombatRFileInfo(data, ref file_info) => {
                            let file_info = file_info.clone();
                            match FileMatchedCombatDebugView::new_view(&mut tab, data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::PortraitSettingsRFileInfo(mut data, file_info) => {
                            match PortraitSettingsView::new_view(&mut tab, &mut data, app_ui, pack_file_contents_ui) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        // If the file is a RigidModel PackedFile...
                        Response::RigidModelRFileInfo(data, file_info) => {
                            match RigidModelView::new_view(&mut tab, &data, app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui) {
                                Ok(_) => {

                                   // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);

                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        // If the file is a Text PackedFile...
                        Response::TextRFileInfo(data, file_info) => {
                            PackedFileTextView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                            // Fix the quick notes view.
                            let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);

                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                            }
                        }

                        // If the file is the notes...
                        Response::Text(data) => {
                            PackedFileTextView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                            // Fix the quick notes view.
                            let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                        }

                        Response::UnitVariantRFileInfo(mut data, file_info) => {
                            if settings_bool(USE_DEBUG_VIEW_UNIT_VARIANT) {
                                match UnitVariantDebugView::new_view(&mut tab, data.clone()) {
                                    Ok(_) => {

                                        // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                        // Fix the quick notes view.
                                        let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                        }
                                    },
                                    Err(error) => {
                                        app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                        return show_dialog(&app_ui.main_window, error, false);
                                    }
                                }
                            } else {
                                match UnitVariantView::new_view(&mut tab, &mut data, app_ui, pack_file_contents_ui) {
                                    Ok(_) => {

                                        // Add the file to the 'Currently open' list and make it visible.
                                        app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                        // Fix the quick notes view.
                                        let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                        layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                        let mut open_list = UI_STATE.set_open_packedfiles();
                                        open_list.push(tab);
                                        if data_source == DataSource::PackFile {
                                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                        }
                                    },
                                    Err(error) => {
                                        app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                        return show_dialog(&app_ui.main_window, error, false);
                                    }
                                }
                            }
                        }

                        #[cfg(feature = "support_uic")]
                        Response::UICRFileInfo(mut data, file_info) => {
                            match FileUICView::new_view(&mut tab, &mut data) {
                                Ok(_) => {

                                    // Add the file to the 'Currently open' list and make it visible.
                                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                                    // Fix the quick notes view.
                                    let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                                    layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                                    let mut open_list = UI_STATE.set_open_packedfiles();
                                    open_list.push(tab);
                                    if data_source == DataSource::PackFile {
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                                    }
                                },
                                Err(error) => {
                                    app_ui.tab_bar_packed_file.remove_tab(tab_index);
                                    return show_dialog(&app_ui.main_window, error, false);
                                }
                            }
                        }

                        Response::Unknown => app_ui.tab_bar_packed_file.remove_tab(tab_index),

                        // If the file is a CA_VP8 PackedFile...
                        Response::VideoInfoRFileInfo(data, file_info) => {
                            PackedFileVideoView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                            // Fix the quick notes view.
                            let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                            }
                        }

                        Response::VMDRFileInfo(data, file_info) => {
                            FileVMDView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data, FileType::VMD);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                            // Fix the quick notes view.
                            let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);

                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                            }
                        },

                        Response::WSModelRFileInfo(data, file_info) => {
                            FileVMDView::new_view(&mut tab, app_ui, pack_file_contents_ui, &data, FileType::WSModel);

                            // Add the file to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());

                            // Fix the quick notes view.
                            let layout = tab.main_widget().layout().static_downcast::<QGridLayout>();
                            layout.add_widget_5a(tab.notes_widget(), 0, 99, layout.row_count(), 1);

                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);

                            if data_source == DataSource::PackFile {
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), data_source, &pack_key);
                            }
                        },

                        Response::Error(error) => {
                            app_ui.tab_bar_packed_file.remove_tab(tab_index);
                            return show_dialog(&app_ui.main_window, error, false);
                        }
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };
                }

                // If it's external, we just create a view with just one button: "Stop Watching External File".
                else {
                    let icon_type = IconType::File(path.to_owned());
                    let icon = TREEVIEW_ICONS.icon(icon_type);

                    let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::OpenPackedFileInExternalProgram(pack_key, DataSource::PackFile, ContainerPath::File(path.to_owned())));
                    let path = Rc::new(RefCell::new(path.to_owned()));

                    let response = CentralCommand::recv(&receiver);
                    let external_path = match response {
                        Response::PathBuf(external_path) => external_path,
                        Response::Error(error) => return show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };

                    PackedFileExternalView::new_view(&path, app_ui, &mut tab, pack_file_contents_ui, &external_path);

                    // Add the file to the 'Currently open' list and make it visible.
                    app_ui.tab_bar_packed_file.add_tab_3a(tab.main_widget(), icon, &QString::from_std_str(""));
                    app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());
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

        // This forces the UI to process the events related to making the file view's visible before returning,
        // so stuff that opens a file and scrolls its view actually works.
        let event_loop = QEventLoop::new_0a();
        event_loop.process_events();
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
                    if !is_schema_loaded() {
                        return show_dialog(&app_ui.main_window, "No schema found. You need one to open the decoder.", false);
                    }

                    let mut fake_path = path.to_owned();
                    fake_path.push_str(DECODER_EXTENSION);
                    (fake_path, qtr("decoder_title"))
                },
                SpecialView::PackSettings => (RESERVED_NAME_SETTINGS.to_owned(), qtr("settings")),
                SpecialView::PackDependencies => (RESERVED_NAME_DEPENDENCIES_MANAGER_V2.to_owned(), qtr("table_dependency_manager_title")),
            };

            // Close all preview views except the file we're opening. The path used for the manager is empty.
            for file_view in UI_STATE.get_open_packedfiles().iter() {
                let open_path = file_view.path_read();
                let index = app_ui.tab_bar_packed_file.index_of(file_view.main_widget());
                if !open_path.is_empty() && file_view.is_preview() && index != -1 {
                    app_ui.tab_bar_packed_file.remove_tab(index);
                }
            }

            // If the manager is already open, or it's hidden, we show it/focus it, instead of opening it again.
            if let Some(tab_widget) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).find(|x| *x.path_read() == path) {
                let index = app_ui.tab_bar_packed_file.index_of(tab_widget.main_widget());

                if index == -1 {
                    let icon_type = IconType::Pack(true);
                    let icon = TREEVIEW_ICONS.icon(icon_type);
                    app_ui.tab_bar_packed_file.add_tab_3a(tab_widget.main_widget(), icon, &name);
                }

                app_ui.tab_bar_packed_file.set_current_widget(tab_widget.main_widget());
                return;
            }

            // If it's not already open/hidden, we create it and add it as a new tab.
            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            let mut tab = FileView::new(&path, &pack_key);
            tab.main_widget().set_parent(&app_ui.tab_bar_packed_file);
            tab.set_is_preview(false);
            let icon_type = IconType::Pack(true);
            let icon = TREEVIEW_ICONS.icon(icon_type);

            match view_type {
                SpecialView::Decoder(ref path) => {
                    match PackedFileDecoderView::new_view(&mut tab, path, app_ui, pack_file_contents_ui) {
                        Ok(_) => {

                            // Add the decoder to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.main_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.push(tab);
                        },
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    }
                }
                SpecialView::PackSettings => {
                    match PackFileSettingsView::new_view(&mut tab, app_ui, pack_file_contents_ui) {
                        Ok(_) => {
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.main_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());
                            UI_STATE.set_open_packedfiles().push(tab);
                        },
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
                    }
                },
                SpecialView::PackDependencies => {
                    match DependenciesManagerView::new_view(&mut tab, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui) {
                        Ok(_) => {

                            // Add the manager to the 'Currently open' list and make it visible.
                            app_ui.tab_bar_packed_file.add_tab_3a(tab.main_widget(), icon, &name);
                            app_ui.tab_bar_packed_file.set_current_widget(tab.main_widget());
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

            if !is_schema_loaded() {
                return show_dialog(&app_ui.main_window, "There is no Schema for the Game Selected.", false);
            }

            let it_is = send_ipc_command(Command::IsThereADependencyDatabase(false), response_extractor!(Response::Bool));
            if !it_is { return show_dialog(&app_ui.main_window, "The dependencies cache for the Game Selected is either missing, outdated, or it was generated without the Assembly Kit. Please, re-generate it and try again.", false); }
        }

        if file_type == FileType::PortraitSettings && GAME_SELECTED.read().unwrap().portrait_settings_version().is_none() {
            return show_dialog(&app_ui.main_window, "Creating PortraitSettings files is currently not supported for this game.", false);
        }

        // Create the "New File" dialog and wait for his data (or a cancellation). If we receive None, we do nothing. If we receive Some,
        // we still have to check if it has been any error during the creation of the File (for example, no definition for DB Tables).
        match Self::new_file_dialog(app_ui, pack_file_contents_ui, file_type) {
            Ok(new_file) => {
                if let Some(mut new_file) = new_file {

                    // Check for empty names.
                    match new_file {
                        NewFile::AnimPack(ref mut name) |
                        NewFile::Loc(ref mut name) |
                        NewFile::VMD(ref mut name) |
                        NewFile::WSModel(ref mut name) |
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
                        NewFile::VMD(ref name) |
                        NewFile::WSModel(ref name) |
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
                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    let exists = send_ipc_command(Command::PackedFileExists(pack_key.clone(), full_path.to_owned()), response_extractor!(Response::Bool));
                    if exists {
                        return show_dialog(&app_ui.main_window, format!("A file with this path ({full_path})' already exists in the Pack."), false)
                    }

                    // Get the response, just in case it failed.
                    match send_ipc_command_result(Command::NewPackedFile(pack_key.clone(), full_path.to_owned(), new_file), response_extractor!()) {
                        Ok(_) => {
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(full_path); 1]), DataSource::PackFile, &pack_key);
                            UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                        }
                        Err(error) => show_dialog(&app_ui.main_window, error, false),
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
    ///
    /// The name used for each packfile is a generic one.
    pub unsafe fn new_queek_packed_file(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the currently selected path and, depending on the selected path, generate one packfile or another.
        let selected_items = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
        if selected_items.len() == 1 {
            let item = &selected_items[0];

            let path = match item {
                ContainerPath::File(_) => item.parent_path().to_lowercase(),
                ContainerPath::Folder(path) => path.to_owned().to_lowercase(),
            };
            let path_split = path.split('/').collect::<Vec<_>>();

            if let Some(mut name) = Self::new_packed_file_name_dialog(app_ui, pack_file_contents_ui) {

                // DB Check.
                let (new_path, new_packed_file) = if path.starts_with("db") && (path_split.len() == 2 || path_split.len() == 3) {
                    let new_path = format!("{path}/{name}");
                    let table = path_split[1];

                    let version = match send_ipc_command_result(Command::GetTableVersionFromDependencyPackFile(table.to_owned()), response_extractor!(Response::I32)) {
                        Ok(data) => data,
                        Err(error) => return show_dialog(&app_ui.main_window, error, false),
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

                    let new_packed_file = NewFile::VMD(name.to_owned());
                    (new_path, new_packed_file)
                }

                // Neutral Check, for folders without a predefined type.
                else {
                    return show_dialog(&app_ui.main_window, "I don't know what type of file goes in that folder, boi.", false);
                };

                // Check if the PackedFile already exists, and report it if so.
                let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                let exists = send_ipc_command(Command::PackedFileExists(pack_key.clone(), new_path.to_owned()), response_extractor!(Response::Bool));
                if exists { return show_dialog(&app_ui.main_window, "The provided file/s already exists in the current path.", false)}

                // Create the PackFile.
                match send_ipc_command_result(Command::NewPackedFile(pack_key.clone(), new_path.to_owned(), new_packed_file), response_extractor!()) {
                    Ok(_) => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(vec![ContainerPath::File(new_path); 1]), DataSource::PackFile, &pack_key);
                        UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                    }
                    Err(error) => show_dialog(&app_ui.main_window, error, false),
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
    pub unsafe fn new_file_dialog(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>, file_type: FileType) -> Result<Option<NewFile>> {

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
        table_filter_line_edit.set_focus_0a();
        table_filter_line_edit.text_changed().connect(&SlotNoArgs::new(&dialog, move || {
            table_filter.set_filter_regular_expression_q_regular_expression(&QRegularExpression::new_1a(&table_filter_line_edit.text()));
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
        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let pack_name = send_ipc_command(Command::GetPackFileName(pack_key.clone()), response_extractor!(Response::String));
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
                let mut tables = send_ipc_command(Command::GetTableListFromDependencyPackFile, response_extractor!(Response::VecString));

                // Also get the custom tables (start_pos) if there's any supported for the game selected.
                //
                // These may come duplicated, so we need to dedup them later.
                let mut custom_tables = send_ipc_command(Command::GetCustomTableList, response_extractor!(Response::VecString));

                tables.append(&mut custom_tables);
                tables.sort();
                tables.dedup();
                tables.iter().for_each(|x| table_model.append_row_q_standard_item(QStandardItem::from_q_string(&QString::from_std_str(x)).into_ptr()));

                name_line_edit.set_text(&QString::from_std_str(&pack_name));
                table_extra_widget.set_visible(true);
            },
            FileType::Loc => name_line_edit.set_text(&QString::from_std_str(format!("{pack_name}.loc"))),
            FileType::Text => name_line_edit.set_text(&QString::from_std_str(format!("{pack_name}.txt"))),
            FileType::PortraitSettings => {
                let local_art_set_ids = send_ipc_command(Command::LocalArtSetIds(pack_key.clone()), response_extractor!(Response::HashSetString));
                let dependencies_art_set_ids = send_ipc_command(Command::DependenciesArtSetIds, response_extractor!(Response::HashSetString));

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
                name_line_edit.set_focus_0a();
                portrait_settings_extra_widget.set_visible(true);
            },
            _ => unimplemented!(),
        }

        // Force resize down to fix issues with certain modes.
        dialog.resize_2a(500, 100);

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
                    let version = send_ipc_command_result(Command::GetTableVersionFromDependencyPackFile(table.to_owned()), response_extractor!(Response::I32))?;
                    Ok(Some(NewFile::DB(file_name, table, version)))
                },
                FileType::Loc => {
                    if !file_name.ends_with(loc::EXTENSION) {
                        file_name.push_str(loc::EXTENSION);
                    }
                    Ok(Some(NewFile::Loc(file_name)))
                },
                FileType::Text => {
                    if !text::EXTENSIONS.iter().any(|(x, _)| file_name.ends_with(x)) && !file_name.ends_with(text::EXTENSION_VMD.0) && !file_name.ends_with(text::EXTENSION_WSMODEL.0) {
                        file_name.push_str(".txt");
                    }

                    if let Some((_, text_format)) = text::EXTENSIONS.iter().find(|(x, _)| file_name.ends_with(x)) {
                        Ok(Some(NewFile::Text(file_name, *text_format)))
                    } else if file_name.ends_with(text::EXTENSION_VMD.0) {
                        Ok(Some(NewFile::VMD(file_name)))
                    } else if file_name.ends_with(text::EXTENSION_WSMODEL.0) {
                        Ok(Some(NewFile::WSModel(file_name)))
                    } else {
                        Ok(Some(NewFile::Text(file_name, TextFormat::Plain)))
                    }
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
    unsafe fn new_packed_file_name_dialog(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Option<String> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("new_packedfile_name"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);

        let main_grid = create_grid_layout(dialog.static_upcast());
        let name_line_edit = QLineEdit::new();
        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let packfile_name = send_ipc_command(Command::GetPackFileName(pack_key), response_extractor!(Response::String));
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
    pub unsafe fn merge_tables_dialog(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Option<(String, bool)> {

        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("merge_tables"));
        dialog.set_modal(true);

        // Create the main Grid.
        let main_grid = create_grid_layout(dialog.static_upcast());
        let name_line_edit = QLineEdit::new();

        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let packfile_name = send_ipc_command(Command::GetPackFileName(pack_key), response_extractor!(Response::String));
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

    /// This function creates the "Pack Map" dialog.
    ///
    /// It returns the tile maps and tiles to add, or `None` if the dialog is canceled or closed.
    pub unsafe fn pack_map_dialog(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<Option<(Vec<PathBuf>, Vec<(PathBuf, String)>)>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { PACK_MAP_VIEW_DEBUG } else { PACK_MAP_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        // Create and configure the dialog.
        let tile_maps_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "tile_maps_groupbox")?;
        let tile_maps_add_selected: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "tile_maps_add_selected")?;
        let tile_maps_remove_selected: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "tile_maps_remove_selected")?;
        let tile_maps_available: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "tile_maps_available")?;
        let tile_maps_to_add: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "tile_maps_to_add")?;

        let tiles_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "tiles_groupbox")?;
        let tiles_add_selected: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "tiles_add_selected")?;
        let tiles_remove_selected: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "tiles_remove_selected")?;
        let tiles_available: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "tiles_available")?;
        let tiles_to_add: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "tiles_to_add")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        let tile_maps_available_model = QStandardItemModel::new_1a(&tile_maps_available);
        let tile_maps_to_add_model = QStandardItemModel::new_1a(&tile_maps_to_add);
        let tiles_available_model = QStandardItemModel::new_1a(&tiles_available);
        let tiles_to_add_model = QStandardItemModel::new_1a(&tiles_to_add);

        let tile_maps_available_filter = QSortFilterProxyModel::new_1a(&tile_maps_available_model);
        let tile_maps_to_add_filter = QSortFilterProxyModel::new_1a(&tile_maps_to_add_model);
        let tiles_available_filter = QSortFilterProxyModel::new_1a(&tiles_available_model);
        let tiles_to_add_filter = QSortFilterProxyModel::new_1a(&tiles_to_add_model);
        tile_maps_available_filter.set_source_model(&tile_maps_available_model);
        tile_maps_to_add_filter.set_source_model(&tile_maps_to_add_model);
        tiles_available_filter.set_source_model(&tiles_available_model);
        tiles_to_add_filter.set_source_model(&tiles_to_add_model);
        tile_maps_available.set_model(&tile_maps_available_filter);
        tile_maps_to_add.set_model(&tile_maps_to_add_filter);
        tiles_available.set_model(&tiles_available_filter);
        tiles_to_add.set_model(&tiles_to_add_filter);

        tile_maps_available_filter.sort_2a(0, SortOrder::AscendingOrder);
        tile_maps_to_add_filter.sort_2a(0, SortOrder::AscendingOrder);
        tiles_available_filter.sort_2a(0, SortOrder::AscendingOrder);
        tiles_to_add_filter.sort_2a(0, SortOrder::AscendingOrder);

        dialog.set_window_title(&qtr("pack_map"));
        tile_maps_groupbox.set_title(&qtr("tile_maps"));
        tiles_groupbox.set_title(&qtr("tiles"));

        // Populate the lists with the available tile maps and tiles from the assembly kit.
        let game_key = GAME_SELECTED.read().unwrap().key();
        let ak_path = settings_path_buf(&format!("{game_key}_assembly_kit"));

        let tile_maps_path = ak_path.join("working_data/terrain/battles");
        let tile_maps = final_folders_from_subdir(&tile_maps_path, true)?;
        let tile_maps_strip_name = tile_maps.iter().flat_map(|tile_map| tile_map.strip_prefix(&tile_maps_path)).collect::<Vec<_>>();


        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        for (index, tile_map) in tile_maps.iter().enumerate() {
            let tile_map_name = tile_maps_strip_name[index].to_string_lossy().replace('\\', "/");
            let item = QStandardItem::from_q_string(&QString::from_std_str(&tile_map_name));
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(tile_map.to_string_lossy())), 20);
            item.set_editable(false);

            let exists = send_ipc_command_async(Command::FolderExists(pack_key.clone(), format!("terrain/battles/{tile_map_name}")), response_extractor!(Response::Bool));
            if exists {
                tile_maps_to_add_model.append_row_q_standard_item(item.into_ptr());
            } else {
                tile_maps_available_model.append_row_q_standard_item(item.into_ptr());
            }
        }

        let tiles_path = ak_path.join("working_data/terrain/tiles/battle");
        let tiles = final_folders_from_subdir(&tiles_path, true)?;
        let tiles_strip_name = tiles.iter().flat_map(|tile| tile.strip_prefix(&tiles_path)).collect::<Vec<_>>();
        for (index, tile) in tiles.iter().enumerate() {
            let tile_name = tiles_strip_name[index].to_string_lossy().replace('\\', "/");

            // Ignore the database folder, as it's not a tile itself.
            if tile_name != "_tile_database/TILES" {

                let item = QStandardItem::from_q_string(&QString::from_std_str(&tile_name));
                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(tile.to_string_lossy())), 20);
                item.set_editable(false);

                let exists = send_ipc_command_async(Command::FolderExists(pack_key.clone(), format!("terrain/tiles/battle/{tile_name}")), response_extractor!(Response::Bool));
                if exists {
                    tiles_to_add_model.append_row_q_standard_item(item.into_ptr());
                } else {
                    tiles_available_model.append_row_q_standard_item(item.into_ptr());
                }
            }
        }

        // Actions
        let tile_maps_available_ptr = tile_maps_available.as_ptr();
        let tile_maps_available_filter_ptr = tile_maps_available_filter.as_ptr();
        let tile_maps_available_model_ptr = tile_maps_available_model.as_ptr();
        let tile_maps_to_add_ptr = tile_maps_to_add.as_ptr();
        let tile_maps_to_add_filter_ptr = tile_maps_to_add_filter.as_ptr();
        let tile_maps_to_add_model_ptr = tile_maps_to_add_model.as_ptr();
        let tile_maps_add_selected_slot = SlotNoArgs::new(&dialog, move || {
            let selected = tile_maps_available_ptr.selection_model().selected_indexes();
            let mut indexes = (0..selected.count()).map(|row| tile_maps_available_filter_ptr.map_to_source(selected.at(row))).collect::<Vec<_>>();
            indexes.sort_by_key(|index| index.row());
            indexes.reverse();

            for index in &indexes {
                let new_item = QStandardItem::from_q_string(&index.data_0a().to_string());
                new_item.set_data_2a(&index.data_1a(20), 20);
                new_item.set_editable(false);
                tile_maps_to_add_model_ptr.append_row_q_standard_item(new_item.into_ptr());

                tile_maps_available_model_ptr.remove_row_1a(index.row());
            }
        });

        let tile_maps_remove_selected_slot = SlotNoArgs::new(&dialog, move || {
            let selected = tile_maps_to_add_ptr.selection_model().selected_indexes();
            let mut indexes = (0..selected.count()).map(|row| tile_maps_to_add_filter_ptr.map_to_source(selected.at(row))).collect::<Vec<_>>();
            indexes.sort_by_key(|index| index.row());
            indexes.reverse();

            for index in &indexes {
                let new_item = QStandardItem::from_q_string(&index.data_0a().to_string());
                new_item.set_data_2a(&index.data_1a(20), 20);
                new_item.set_editable(false);
                tile_maps_available_model_ptr.append_row_q_standard_item(new_item.into_ptr());

                tile_maps_to_add_model_ptr.remove_row_1a(index.row());
            }
        });

        tile_maps_add_selected.released().connect(&tile_maps_add_selected_slot);
        tile_maps_available.double_clicked().connect(&tile_maps_add_selected_slot);

        tile_maps_remove_selected.released().connect(&tile_maps_remove_selected_slot);
        tile_maps_to_add.double_clicked().connect(&tile_maps_remove_selected_slot);

        let tiles_available_ptr = tiles_available.as_ptr();
        let tiles_available_filter_ptr = tiles_available_filter.as_ptr();
        let tiles_available_model_ptr = tiles_available_model.as_ptr();
        let tiles_to_add_ptr = tiles_to_add.as_ptr();
        let tiles_to_add_filter_ptr = tiles_to_add_filter.as_ptr();
        let tiles_to_add_model_ptr = tiles_to_add_model.as_ptr();
        let tiles_add_selected_slot = SlotNoArgs::new(&dialog, move || {
            let selected = tiles_available_ptr.selection_model().selected_indexes();
            let mut indexes = (0..selected.count()).map(|row| tiles_available_filter_ptr.map_to_source(selected.at(row))).collect::<Vec<_>>();
            indexes.sort_by_key(|index| index.row());
            indexes.reverse();

            for index in &indexes {
                let new_item = QStandardItem::from_q_string(&index.data_0a().to_string());
                new_item.set_data_2a(&index.data_1a(20), 20);
                new_item.set_editable(false);
                tiles_to_add_model_ptr.append_row_q_standard_item(new_item.into_ptr());

                tiles_available_model_ptr.remove_row_1a(index.row());
            }
        });

        let tiles_remove_selected_slot = SlotNoArgs::new(&dialog, move || {
            let selected = tiles_to_add_ptr.selection_model().selected_indexes();
            let mut indexes = (0..selected.count()).map(|row| tiles_to_add_filter_ptr.map_to_source(selected.at(row))).collect::<Vec<_>>();
            indexes.sort_by_key(|index| index.row());
            indexes.reverse();

            for index in &indexes {
                let new_item = QStandardItem::from_q_string(&index.data_0a().to_string());
                new_item.set_data_2a(&index.data_1a(20), 20);
                new_item.set_editable(false);
                tiles_available_model_ptr.append_row_q_standard_item(new_item.into_ptr());

                tiles_to_add_model_ptr.remove_row_1a(index.row());
            }
        });

        tiles_add_selected.released().connect(&tiles_add_selected_slot);
        tiles_available.double_clicked().connect(&tiles_add_selected_slot);

        tiles_remove_selected.released().connect(&tiles_remove_selected_slot);
        tiles_to_add.double_clicked().connect(&tiles_remove_selected_slot);

        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let tile_maps = (0..tile_maps_to_add.model().row_count_0a())
                .map(|row| PathBuf::from(tile_maps_to_add.model().index_2a(row, 0).data_1a(20).to_string().to_std_string()))
                .collect::<Vec<_>>();

            let tiles = (0..tiles_to_add.model().row_count_0a())
                .map(|row| {
                    let tile = tiles_to_add.model().index_2a(row, 0).data_1a(20).to_string().to_std_string();
                    let tile_subpath = tiles_to_add.model().index_2a(row, 0).data_0a().to_string().to_std_string().replace('\\', "/");
                    let mut tile_subpath = tile_subpath.split('/').collect::<Vec<_>>();
                    tile_subpath.pop();

                    (PathBuf::from(&tile), tile_subpath.join("/"))
                }).collect::<Vec<_>>();

            if !tile_maps.is_empty() || !tiles.is_empty() {
                Ok(Some((tile_maps, tiles)))
            } else {
                Ok(None)
            }
        } else { Ok(None) }
    }

    pub unsafe fn optimizer_dialog(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>
    ) -> Result<Option<()>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { OPTIMIZER_VIEW_DEBUG } else { OPTIMIZER_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();
        dialog.set_window_title(&qtr("optimizer_title"));

        // Create and configure the dialog.
        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let options_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "options_groupbox")?;
        let pack_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "pack_groupbox")?;
        let table_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "table_groupbox")?;
        let text_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "text_groupbox")?;
        let pts_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "pts_groupbox")?;

        let pack_remove_itm_files_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "pack_remove_itm_files_checkbox")?;
        let db_import_datacores_into_twad_key_deletes_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "db_import_datacores_into_twad_key_deletes_checkbox")?;
        let db_optimize_datacored_tables_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "db_optimize_datacored_tables_checkbox")?;
        let table_remove_duplicated_entries_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "table_remove_duplicated_entries_checkbox")?;
        let table_remove_itm_entries_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "table_remove_itm_entries_checkbox")?;
        let table_remove_itnr_entries_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "table_remove_itnr_entries_checkbox")?;
        let table_remove_empty_file_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "table_remove_empty_file_checkbox")?;
        let text_remove_unused_xml_map_folders_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "text_remove_unused_xml_map_folders_checkbox")?;
        let text_remove_unused_xml_prefab_folder_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "text_remove_unused_xml_prefab_folder_checkbox")?;
        let text_remove_agf_files_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "text_remove_agf_files_checkbox")?;
        let text_remove_model_statistics_files_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "text_remove_model_statistics_files_checkbox")?;
        let pts_remove_unused_art_sets_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "pts_remove_unused_art_sets_checkbox")?;
        let pts_remove_unused_variants_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "pts_remove_unused_variants_checkbox")?;
        let pts_remove_empty_masks_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "pts_remove_empty_masks_checkbox")?;
        let pts_remove_empty_file_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "pts_remove_empty_file_checkbox")?;

        let pack_remove_itm_files_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pack_remove_itm_files_label")?;
        let db_import_datacores_into_twad_key_deletes_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "db_import_datacores_into_twad_key_deletes_label")?;
        let db_optimize_datacored_tables_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "db_optimize_datacored_tables_label")?;
        let table_remove_duplicated_entries_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "table_remove_duplicated_entries_label")?;
        let table_remove_itm_entries_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "table_remove_itm_entries_label")?;
        let table_remove_itnr_entries_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "table_remove_itnr_entries_label")?;
        let table_remove_empty_file_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "table_remove_empty_file_label")?;
        let text_remove_unused_xml_map_folders_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "text_remove_unused_xml_map_folders_label")?;
        let text_remove_unused_xml_prefab_folder_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "text_remove_unused_xml_prefab_folder_label")?;
        let text_remove_agf_files_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "text_remove_agf_files_label")?;
        let text_remove_model_statistics_files_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "text_remove_model_statistics_files_label")?;
        let pts_remove_unused_art_sets_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pts_remove_unused_art_sets_label")?;
        let pts_remove_unused_variants_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pts_remove_unused_variants_label")?;
        let pts_remove_empty_masks_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pts_remove_empty_masks_label")?;
        let pts_remove_empty_file_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pts_remove_empty_file_label")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        instructions_label.set_text(&qtr("optimizer_instructions_label"));
        options_groupbox.set_title(&qtr("optimizer_options_title"));
        pack_groupbox.set_title(&qtr("optimizer_pack_title"));
        table_groupbox.set_title(&qtr("optimizer_table_title"));
        text_groupbox.set_title(&qtr("optimizer_text_title"));
        pts_groupbox.set_title(&qtr("optimizer_pts_title"));

        pack_remove_itm_files_label.set_text(&qtr("optimizer_pack_remove_itm_files"));
        db_import_datacores_into_twad_key_deletes_label.set_text(&qtr("optimizer_db_import_datacores_into_twad_key_deletes"));
        db_optimize_datacored_tables_label.set_text(&qtr("optimizer_db_optimize_datacored_tables"));
        table_remove_duplicated_entries_label.set_text(&qtr("optimizer_table_remove_duplicated_entries"));
        table_remove_itm_entries_label.set_text(&qtr("optimizer_table_remove_itm_entries"));
        table_remove_itnr_entries_label.set_text(&qtr("optimizer_table_remove_itnr_entries"));
        table_remove_empty_file_label.set_text(&qtr("optimizer_table_remove_empty_file"));
        text_remove_unused_xml_map_folders_label.set_text(&qtr("optimizer_text_remove_unused_xml_map_folders"));
        text_remove_unused_xml_prefab_folder_label.set_text(&qtr("optimizer_text_remove_unused_xml_prefab_folder"));
        text_remove_agf_files_label.set_text(&qtr("optimizer_text_remove_agf_files"));
        text_remove_model_statistics_files_label.set_text(&qtr("optimizer_text_remove_model_statistics_files"));
        pts_remove_unused_art_sets_label.set_text(&qtr("optimizer_pts_remove_unused_art_sets"));
        pts_remove_unused_variants_label.set_text(&qtr("optimizer_pts_remove_unused_variants"));
        pts_remove_empty_masks_label.set_text(&qtr("optimizer_pts_remove_empty_masks"));
        pts_remove_empty_file_label.set_text(&qtr("optimizer_pts_remove_empty_file"));

        {
            pack_remove_itm_files_checkbox.set_checked(settings_bool(PACK_REMOVE_ITM_FILES));
            db_import_datacores_into_twad_key_deletes_checkbox.set_checked(settings_bool(DB_IMPORT_DATACORES_INTO_TWAD_KEY_DELETES));
            db_optimize_datacored_tables_checkbox.set_checked(settings_bool(DB_OPTIMIZE_DATACORED_TABLES));
            table_remove_duplicated_entries_checkbox.set_checked(settings_bool(TABLE_REMOVE_DUPLICATED_ENTRIES));
            table_remove_itm_entries_checkbox.set_checked(settings_bool(TABLE_REMOVE_ITM_ENTRIES));
            table_remove_itnr_entries_checkbox.set_checked(settings_bool(TABLE_REMOVE_ITNR_ENTRIES));
            table_remove_empty_file_checkbox.set_checked(settings_bool(TABLE_REMOVE_EMPTY_FILE));
            text_remove_unused_xml_map_folders_checkbox.set_checked(settings_bool(TEXT_REMOVE_UNUSED_XML_MAP_FOLDERS));
            text_remove_unused_xml_prefab_folder_checkbox.set_checked(settings_bool(TEXT_REMOVE_UNUSED_XML_PREFAB_FOLDER));
            text_remove_agf_files_checkbox.set_checked(settings_bool(TEXT_REMOVE_AGF_FILES));
            text_remove_model_statistics_files_checkbox.set_checked(settings_bool(TEXT_REMOVE_MODEL_STATISTICS_FILES));
            pts_remove_unused_art_sets_checkbox.set_checked(settings_bool(PTS_REMOVE_UNUSED_ART_SETS));
            pts_remove_unused_variants_checkbox.set_checked(settings_bool(PTS_REMOVE_UNUSED_VARIANTS));
            pts_remove_empty_masks_checkbox.set_checked(settings_bool(PTS_REMOVE_EMPTY_MASKS));
            pts_remove_empty_file_checkbox.set_checked(settings_bool(PTS_REMOVE_EMPTY_FILE));
        }

        db_optimize_datacored_tables_checkbox.set_visible(false);
        db_optimize_datacored_tables_label.set_visible(false);

        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            settings_set_bool(PACK_REMOVE_ITM_FILES, pack_remove_itm_files_checkbox.is_checked());
            settings_set_bool(DB_IMPORT_DATACORES_INTO_TWAD_KEY_DELETES, db_import_datacores_into_twad_key_deletes_checkbox.is_checked());
            settings_set_bool(DB_OPTIMIZE_DATACORED_TABLES, db_optimize_datacored_tables_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_DUPLICATED_ENTRIES, table_remove_duplicated_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_ITM_ENTRIES, table_remove_itm_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_ITNR_ENTRIES, table_remove_itnr_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_EMPTY_FILE, table_remove_empty_file_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_UNUSED_XML_MAP_FOLDERS, text_remove_unused_xml_map_folders_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_UNUSED_XML_PREFAB_FOLDER, text_remove_unused_xml_prefab_folder_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_AGF_FILES, text_remove_agf_files_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_MODEL_STATISTICS_FILES, text_remove_model_statistics_files_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_UNUSED_ART_SETS, pts_remove_unused_art_sets_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_UNUSED_VARIANTS, pts_remove_unused_variants_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_EMPTY_MASKS, pts_remove_empty_masks_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_EMPTY_FILE, pts_remove_empty_file_checkbox.is_checked());

            AppUI::purge_them_all(app_ui, pack_file_contents_ui, true)?;
            GlobalSearchUI::clear(global_search_ui);

            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            let options = optimizer_options();
            let (response_1, response_2) = send_ipc_command_result_async(Command::OptimizePackFile(pack_key.clone(), options), response_extractor!(Response::HashSetStringHashSetString, v1, v2))?;
            let response_1 = response_1.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<ContainerPath>>();
            let response_2 = response_2.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<ContainerPath>>();

            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(response_1, true), DataSource::PackFile, &pack_key);
            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(response_2), DataSource::PackFile, &pack_key);
            Ok(Some(()))
        } else {
            settings_set_bool(PACK_REMOVE_ITM_FILES, pack_remove_itm_files_checkbox.is_checked());
            settings_set_bool(DB_IMPORT_DATACORES_INTO_TWAD_KEY_DELETES, db_import_datacores_into_twad_key_deletes_checkbox.is_checked());
            settings_set_bool(DB_OPTIMIZE_DATACORED_TABLES, db_optimize_datacored_tables_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_DUPLICATED_ENTRIES, table_remove_duplicated_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_ITM_ENTRIES, table_remove_itm_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_ITNR_ENTRIES, table_remove_itnr_entries_checkbox.is_checked());
            settings_set_bool(TABLE_REMOVE_EMPTY_FILE, table_remove_empty_file_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_UNUSED_XML_MAP_FOLDERS, text_remove_unused_xml_map_folders_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_UNUSED_XML_PREFAB_FOLDER, text_remove_unused_xml_prefab_folder_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_AGF_FILES, text_remove_agf_files_checkbox.is_checked());
            settings_set_bool(TEXT_REMOVE_MODEL_STATISTICS_FILES, text_remove_model_statistics_files_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_UNUSED_ART_SETS, pts_remove_unused_art_sets_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_UNUSED_VARIANTS, pts_remove_unused_variants_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_EMPTY_MASKS, pts_remove_empty_masks_checkbox.is_checked());
            settings_set_bool(PTS_REMOVE_EMPTY_FILE, pts_remove_empty_file_checkbox.is_checked());

            Ok(None)
        }
    }

    /// Update the FileView names, to ensure we have no collisions.
    pub unsafe fn update_views_names(&self) {

        // We also have to check for colliding packedfile names, so we can use their full path instead.
        let mut names = HashMap::new();
        let open_packedfiles = UI_STATE.get_open_packedfiles();
        for file_view in open_packedfiles.iter() {
            let widget = file_view.main_widget();
            if self.tab_bar_packed_file.index_of(widget) != -1 {

                // Reserved PackedFiles should have special names.
                let path = file_view.path_read();
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

        for file_view in UI_STATE.get_open_packedfiles().iter() {
            let widget = file_view.main_widget();
            let path = file_view.path_read();
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
                match file_view.data_source() {
                    DataSource::PackFile => {},
                    DataSource::ParentFiles => name.push_str("Parent"),
                    DataSource::GameFiles => name.push_str("Game"),
                    DataSource::AssKitFiles => name.push_str("AssKit"),
                    DataSource::ExternalFile => name.push_str("External"),
                }

                if !name.is_empty() {
                    if file_view.is_read_only() {
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

                if file_view.is_preview() {
                    name.push_str(" (Preview)");
                }

                let index = self.tab_bar_packed_file.index_of(widget);
                self.tab_bar_packed_file.set_tab_text(index, &QString::from_std_str(&name));
            }
        }
    }

    /// This function hides all the provided packedfile views.
    pub unsafe fn file_view_hide(
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

        for file_view in UI_STATE.get_open_packedfiles().iter() {
            let widget = file_view.main_widget();
            let index_widget = app_ui.tab_bar_packed_file.index_of(widget);
            if indexes.contains(&index_widget) {
                let path = file_view.path_read();
                if !path.is_empty() {
                    if path.starts_with(RESERVED_NAME_EXTRA_PACKFILE) {
                        purge_on_delete.push(path.to_owned());

                        let path_split = path.split('/').collect::<Vec<_>>();
                        let pack_path = path_split[1..].join("/");
                        let pack_key = std::path::Path::new(&pack_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown.pack")
                            .to_string();
                        let _ = CENTRAL_COMMAND.read().unwrap().send(Command::ClosePack(pack_key));
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
    /// This function rebuilds the parent packs in the dependencies UI.
    ///
    /// It's shared between `open_packfile` (to reload parents when a new pack is opened)
    /// and `change_game_selected` (to reload parents when the game didn't change but dependencies need rebuilding).
    pub unsafe fn rebuild_parent_packs(
        app_ui: &Rc<Self>,
        _pack_file_contents_ui: &Rc<PackFileContentsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) {
        match send_ipc_command_result_async(Command::RebuildDependencies(true), response_extractor!(Response::DependenciesInfo)) {
            Ok(dep_info) => {
                let mut parent_build_data = BuildData::new();
                parent_build_data.data = Some((ContainerInfo::default(), dep_info.parent_packed_files().to_vec()));
                dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles, "");
            }
            Err(error) => show_dialog(&app_ui.main_window, error, false),
        }
    }

    pub unsafe fn change_game_selected(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        rebuild_dependencies: bool,
        force_full_dependency_reload: bool
    ) {

        // Get the new `Game Selected` and clean his name up, so it ends up like "x_y".
        let mut new_game_selected = app_ui.game_selected_group.checked_action().text().to_std_string();
        if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
        let new_game_selected = new_game_selected.replace(' ', "_").to_lowercase();
        let mut game_changed = false;
        let mut dep_info = None;

        // Due to how the backend is optimized, we need to back our files before triggering the proper game change.
        let _ = AppUI::purge_them_all(app_ui, pack_file_contents_ui, true);

        // If the game changed or we're initializing the program, change the game selected.
        if new_game_selected != GAME_SELECTED.read().unwrap().key() || !FIRST_GAME_CHANGE_DONE.load(Ordering::SeqCst) {

            // Disable the main window if it's not yet disabled so we can avoid certain issues.
            app_ui.toggle_main_window(false);

            // Send the command to the background thread to set the new `Game Selected`. We expect two responses:
            // - New compression format.
            // - Success.
            let (_cf, dependencies_info) = send_ipc_command(Command::SetGameSelected(new_game_selected.to_owned(), rebuild_dependencies), response_extractor!(Response::CompressionFormatDependenciesInfo, v1, v2));
            *GAME_SELECTED.write().unwrap() = SUPPORTED_GAMES.game(&new_game_selected).unwrap();
            dep_info = dependencies_info;

            // Mark all open packs as modified after a game change.
            if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() > 0 {
                let tree_model = pack_file_contents_ui.packfile_contents_tree_model();
                for row in 0..tree_model.row_count_0a() {
                    let root = tree_model.item_1a(row);
                    let variant = root.data_1a(rpfm_ui_common::ITEM_PACK_KEY);
                    let key = if variant.is_valid() && !variant.is_null() { variant.to_string().to_std_string() } else { String::new() };
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::Folder(String::new())]), DataSource::PackFile, &key);
                }
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }

            // Change the GameSelected Icon.
            GameSelectedIcons::set_game_selected_icon(app_ui);

            // Set this at the end, because the backend need to check if it's our first initialization or not first.
            FIRST_GAME_CHANGE_DONE.store(true, Ordering::SeqCst);
            game_changed = true;
        }

        // Reenable the main window once everything is reloaded, regardless of if we disabled it here or not.
        if game_changed {
            app_ui.toggle_main_window(true);
        }

        // Rebuild dependencies if requested.
        if rebuild_dependencies {

            // If the game changed, the SetGameSelected command already returned dep_info with everything.
            // Use it to update parent, game, and asskit tree views.
            if let Some(dep_info) = dep_info {

                if force_full_dependency_reload {
                    app_ui.toggle_main_window(false);
                }

                let mut parent_build_data = BuildData::new();
                parent_build_data.data = Some((ContainerInfo::default(), dep_info.parent_packed_files().to_vec()));
                dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles, "");

                // Game and asskit data only change on game change, so we don't need to rebuild them if the game didn't change.
                if game_changed || force_full_dependency_reload {

                    // NOTE: We're MOVING, not copying nor referencing the RFileInfo. This info is big and moving it makes it faster.
                    let mut game_build_data = BuildData::new();
                    game_build_data.data = Some((ContainerInfo::default(), dep_info.vanilla_packed_files));

                    let mut asskit_build_data = BuildData::new();
                    asskit_build_data.data = Some((ContainerInfo::default(), dep_info.asskit_tables));
                    dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(game_build_data), DataSource::GameFiles, "");
                    dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(asskit_build_data), DataSource::AssKitFiles, "");
                }

                if force_full_dependency_reload {
                    app_ui.toggle_main_window(true);
                }
            }

            // If the game didn't change, just rebuild parent packs using the shared function.
            else {
                Self::rebuild_parent_packs(app_ui, pack_file_contents_ui, dependencies_ui);
            }
        }

        // Disable the pack-related actions and, if we have a pack open, re-enable them.
        AppUI::enable_packfile_actions(app_ui, false);
        if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() != 0 {
            AppUI::enable_packfile_actions(app_ui, true);
        }

        // If we have the setting enabled, ask the backend to generate the missing definition list.
        if settings_bool(CHECK_FOR_MISSING_TABLE_DEFINITIONS) {
            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            let _ = CENTRAL_COMMAND.read().unwrap().send(Command::GetMissingDefinitions(pack_key));
        }
    }

    /// This function creates a new PackFile and setups the UI for it.
    pub unsafe fn new_packfile(
        app_ui: &Rc<Self>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        dependencies_ui: &Rc<DependenciesUI>
    ) {

        // Tell the Background Thread to create a new PackFile and get the pack key.
        let pack_key = send_ipc_command_async(Command::NewPack, response_extractor!(Response::String));

        // Reset the autosave timer.
        let timer = settings_i32(AUTOSAVE_INTERVAL);
        if timer > 0 {
            app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
            app_ui.timer_backup_autosave.start_0a();
        }

        // Disable the main window, so the user can't interrupt the process or interfere with it.
        let window_was_disabled = app_ui.main_window.is_enabled();
        if !window_was_disabled {
            app_ui.toggle_main_window(false);
        }

        // Update the TreeView, adding the new pack without closing existing ones.
        let mut build_data = BuildData::new();
        build_data.editable = true;
        build_data.pack_key = Some(pack_key.clone());
        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::AddPack(build_data), DataSource::PackFile, &pack_key);
        global_search_ui.update_pack_sources(pack_file_contents_ui);

        // Enable the actions available for the PackFile from the `MenuBar`.
        AppUI::enable_packfile_actions(app_ui, true);

        UI_STATE.set_is_modified(false, app_ui, pack_file_contents_ui);

        // Force a dependency rebuild.
        match send_ipc_command_result_async(Command::RebuildDependencies(true), response_extractor!(Response::DependenciesInfo)) {
            Ok(response) => {
                let mut parent_build_data = BuildData::new();
                parent_build_data.data = Some((ContainerInfo::default(), response.parent_packed_files().to_vec()));
                dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles, "");
            }
            Err(error) => show_dialog(&app_ui.main_window, error, false),
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
        pack_key: &str,
    ) {
        app_ui.toggle_main_window(false);

        let mode = send_ipc_command(Command::GetPackOperationalMode(pack_key.to_string()), response_extractor!(Response::OperationalMode));

        match mode {

            // If we have a "MyMod" selected...
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                let mymods_base_path = settings_path_buf("mymods_base_path");
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

                    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
                    let settings = send_ipc_command(Command::GetPackSettings(pack_key), response_extractor!(Response::PackSettings));

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
                else { show_dialog(&app_ui.main_window, "MyMod path not configured. Go to <i>'PackFile/Settings'</i> and configure it.", false) }
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

    /// This function is used to build a snowman.
    pub unsafe fn build_starpos(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        let template_path = if cfg!(debug_assertions) { BUILD_STARPOS_VIEW_DEBUG } else { BUILD_STARPOS_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        // Create and configure the dialog.
        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let campaign_id_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "campaign_id_label")?;
        let campaign_id_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "campaign_id_combobox")?;
        let process_hlp_spd_data_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "process_hlp_spd_data_label")?;
        let process_hlp_spd_data_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "process_hlp_spd_data_checkbox")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let build_starpos_button = button_box.add_button_q_string_button_role(&qtr("build_starpos"), ButtonRole::ActionRole);
        let games_closed_button = button_box.add_button_q_string_button_role(&qtr("games_closed"), ButtonRole::YesRole);
        let campaign_id_model: QBox<QStandardItemModel> = QStandardItemModel::new_1a(&campaign_id_combobox);
        campaign_id_combobox.set_model(&campaign_id_model);
        games_closed_button.set_enabled(false);

        dialog.set_window_title(&qtr("build_starpos"));
        instructions_label.set_text(&qtr("build_starpos_instructions"));
        campaign_id_label.set_text(&qtr("campaign_id"));

        // SPD files are only available since Warhammer 1.
        let game = GAME_SELECTED.read().unwrap();
        if *game.raw_db_version() >= 2 &&
            game.key() != KEY_THRONES_OF_BRITANNIA &&
            game.key() != KEY_ATTILA &&
            game.key() != KEY_ROME_2 {
            process_hlp_spd_data_label.set_text(&qtr("process_hlp_spd_data"));
        } else {
            process_hlp_spd_data_label.set_text(&qtr("process_hlp_data"));
        }

        // HLP files seem to be available only since Rome 2.
        if *game.raw_db_version() < 2 {
            process_hlp_spd_data_checkbox.set_enabled(false);
        }

        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

        let ids = send_ipc_command_async(Command::BuildStarposGetCampaingIds(pack_key.clone()), response_extractor!(Response::HashSetString));
        let mut ids = ids.into_iter().collect::<Vec<_>>();
        ids.sort();

        if ids.is_empty() {
            return Err(anyhow!("Campaigns table either not found or found without campaign entries. Fix it, then try again."));
        }

        for id in &ids {
            campaign_id_combobox.add_item_q_string(&QString::from_std_str(id));
        }

        // Restore the last selected campaign from pack settings.
        let settings = send_ipc_command(Command::GetPackSettings(pack_key.clone()), response_extractor!(Response::PackSettings));
        if let Some(last_campaign) = settings.setting_text("starpos_last_campaign") {
            let index = campaign_id_combobox.find_text_1a(&QString::from_std_str(last_campaign));
            if index >= 0 {
                campaign_id_combobox.set_current_index(index);
            }
        }

        send_ipc_command_result_async(Command::BuildStarposCheckVictoryConditions(pack_key.clone()), response_extractor!())?;

        // Actions
        let dialog_ptr = dialog.as_ptr();
        let build_starpos_button_ptr = build_starpos_button.as_ptr();
        let games_closed_button_ptr = games_closed_button.as_ptr();
        let campaign_id_combobox_ptr = campaign_id_combobox.as_ptr();
        let process_hlp_spd_data_checkbox_ptr = process_hlp_spd_data_checkbox.as_ptr();
        let pack_key_for_closure = pack_key.clone();
        let start_build_process = SlotNoArgs::new(&dialog, move || {
            build_starpos_button_ptr.set_enabled(false);

            let campaign_id = campaign_id_combobox_ptr.current_text().to_std_string();
            let process_hlp_spd_data = process_hlp_spd_data_checkbox_ptr.is_checked();
            match send_ipc_command_result_async(Command::BuildStarpos(pack_key_for_closure.clone(), campaign_id, process_hlp_spd_data), response_extractor!()) {
                Ok(_) => games_closed_button_ptr.set_enabled(true),
                Err(error) => show_dialog(dialog_ptr, error, false),
            }
        });

        build_starpos_button.released().connect(&start_build_process);
        games_closed_button.released().connect(dialog_ptr.slot_accept());

        // Once the game has been closed, we need to cleanup the userscript file, then add the starpos to the open pack.
        if dialog.exec() == 1 {
            let campaign_id = campaign_id_combobox.current_text().to_std_string();
            let process_hlp_spd_data = process_hlp_spd_data_checkbox.is_checked();

            // Save the selected campaign to pack settings for next time.
            let mut settings = send_ipc_command(Command::GetPackSettings(pack_key.clone()), response_extractor!(Response::PackSettings));
            settings.settings_text_mut().insert("starpos_last_campaign".to_owned(), campaign_id.clone());
            let _ = CENTRAL_COMMAND.read().unwrap().send(Command::SetPackSettings(pack_key.clone(), settings));

            let paths = send_ipc_command_result_async(Command::BuildStarposPost(pack_key.clone(), campaign_id, process_hlp_spd_data), response_extractor!(Response::VecContainerPath))?;
            if !paths.is_empty() {
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths), DataSource::PackFile, &pack_key);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }

            Ok(())
        } else if games_closed_button.is_enabled() {

            // If the user did not properly followed the procedure, do a post-cleanup pass anyway to avoid the idiot's stupidity causing problems.
            let campaign_id = campaign_id_combobox.current_text().to_std_string();
            let process_hlp_spd_data = process_hlp_spd_data_checkbox.is_checked();
            send_ipc_command_result_async(Command::BuildStarposCleanup(pack_key.clone(), campaign_id, process_hlp_spd_data), response_extractor!())?;
            Ok(())
        } else {
            Ok(())
        }
    }

    /// This function builds CEO data into the open pack.
    pub unsafe fn build_ceo(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        let template_path = if cfg!(debug_assertions) { BUILD_CEO_VIEW_DEBUG } else { BUILD_CEO_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let build_ceo_button = button_box.add_button_q_string_button_role(&qtr("build_ceo"), ButtonRole::ActionRole);
        let ceo_done_button = button_box.add_button_q_string_button_role(&qtr("build_ceo_done"), ButtonRole::YesRole);
        ceo_done_button.set_enabled(false);

        dialog.set_window_title(&qtr("build_ceo"));
        instructions_label.set_text(&qtr("build_ceo_instructions"));

        let game = GAME_SELECTED.read().unwrap();
        let akit_path = settings_path_buf(&(game.key().to_owned() + "_assembly_kit"))
            .to_string_lossy().to_string();
        let bob_exe = PathBuf::from(&akit_path).join("binaries").join("bob.modder.x64.exe");
        drop(game);

        if akit_path.is_empty() || !bob_exe.exists() {
            build_ceo_button.set_enabled(false);
        }

        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

        // Actions — mirror build_starpos exactly:
        // build_ceo_button runs BOB, then enables ceo_done_button.
        // ceo_done_button closes the dialog (exec returns 1).
        // Import happens after exec() returns, outside any slot.
        let dialog_ptr = dialog.as_ptr();
        let build_ceo_button_ptr = build_ceo_button.as_ptr();
        let ceo_done_button_ptr = ceo_done_button.as_ptr();
        let pack_key_closure = pack_key.clone();
        let akit_path_closure = akit_path.clone();
        let bob_exe_str = bob_exe.to_string_lossy().to_string();

        let start_build = SlotNoArgs::new(&dialog, move || {
            build_ceo_button_ptr.set_enabled(false);
            match send_ipc_command_result_async(
                Command::BuildCeo(pack_key_closure.clone(), akit_path_closure.clone(), bob_exe_str.clone()),
                response_extractor!()
            ) {
                Ok(_) => ceo_done_button_ptr.set_enabled(true),
                Err(error) => {
                    build_ceo_button_ptr.set_enabled(true);
                    show_dialog(dialog_ptr, error, false);
                }
            }
        });

        build_ceo_button.released().connect(&start_build);
        ceo_done_button.released().connect(dialog_ptr.slot_accept());

        // After dialog closes via ceo_done_button, import ceo_data.ccd into the pack.
        if dialog.exec() == 1 {
            let paths = send_ipc_command_result_async(
                Command::BuildCeoPost(pack_key.clone(), akit_path.clone()),
                response_extractor!(Response::VecContainerPath)
            )?;
            if !paths.is_empty() {
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths), DataSource::PackFile, &pack_key);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }
            Ok(())
        } else if ceo_done_button.is_enabled() {
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Opens the CEO Builder dialog, letting the user add CEO entries that are
    /// inserted directly into the open pack's DB tables and loc file.
    pub unsafe fn build_ceo_builder(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        let template_path = if cfg!(debug_assertions) { BUILD_CEO_BUILDER_VIEW_DEBUG } else { BUILD_CEO_BUILDER_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        // ── Find widgets ──────────────────────────────────────────────────────
        let name_line_edit: QPtr<QLineEdit>         = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let type_combo_box: QPtr<QComboBox>         = find_widget(&main_widget.static_upcast(), "type_combo_box")?;
        let element_combo_box: QPtr<QComboBox>      = find_widget(&main_widget.static_upcast(), "element_combo_box")?;
        let gender_combo_box: QPtr<QComboBox>       = find_widget(&main_widget.static_upcast(), "gender_combo_box")?;
        let expanded_check_box: QPtr<QCheckBox>     = find_widget(&main_widget.static_upcast(), "expanded_check_box")?;
        let trait_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "trait_filter_line_edit")?;
        let trait_count_label: QPtr<QLabel>         = find_widget(&main_widget.static_upcast(), "trait_count_label")?;
        let trait_list_widget: QPtr<QListWidget>    = find_widget(&main_widget.static_upcast(), "trait_list_widget")?;
        let add_character_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "add_character_button")?;
        let clear_all_button: QPtr<QPushButton>     = find_widget(&main_widget.static_upcast(), "clear_all_button")?;
        let delete_selected_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "delete_selected_button")?;
        let status_label: QPtr<QLabel>              = find_widget(&main_widget.static_upcast(), "status_label")?;
        let queue_table_widget: QPtr<QTableWidget>  = find_widget(&main_widget.static_upcast(), "queue_table_widget")?;
        let button_box: QPtr<QDialogButtonBox>      = find_widget(&main_widget.static_upcast(), "button_box")?;

        dialog.set_window_title(&QString::from_std_str("CEO Builder"));

        // ── Populate dropdowns ────────────────────────────────────────────────
        for opt in &["title", "unique"] {
            type_combo_box.add_item_q_string(&QString::from_std_str(opt));
        }
        for el in &["metal", "wood", "earth", "fire", "water"] {
            element_combo_box.add_item_q_string(&QString::from_std_str(el));
        }
        for g in &["male", "female"] {
            gender_combo_box.add_item_q_string(&QString::from_std_str(g));
        }

        // ── Trait data ────────────────────────────────────────────────────────
        // (uuid, key) — sorted personality-first then alphabetical
        let raw_traits: &[(&str, &str)] = &[
            ("1322d8ef-4ae7-4253-a077-3eb22aeb234a", "3k_main_ceo_trait_personality_ambitious"),
            ("842057dd-7c18-49d6-82de-2d10196644a7", "3k_main_ceo_trait_personality_aescetic"),
            ("966be79a-588f-4f39-a593-5df50d47cdc6", "3k_main_ceo_trait_personality_arrogant"),
            ("7ea902ec-1335-4d0d-b440-d5135bbeb256", "3k_main_ceo_trait_personality_artful"),
            ("db836993-cd0a-49c9-a45e-1656379ce4aa", "3k_main_ceo_trait_personality_brave"),
            ("cf7fb952-920f-484a-a5df-f1107d8015d2", "3k_main_ceo_trait_personality_brilliant"),
            ("38597594-6c27-4316-b048-91ef4b25d313", "3k_main_ceo_trait_personality_careless"),
            ("8759c40f-851c-421d-a580-a5eb00526633", "3k_main_ceo_trait_personality_cautious"),
            ("4f0906ee-598c-468e-883a-d34ea0b310cf", "3k_main_ceo_trait_personality_charismatic"),
            ("6f143a86-89ca-4409-a9c4-ed69c893b597", "3k_main_ceo_trait_personality_charitable"),
            ("702c18a1-0229-422d-b5d6-e90d4aba8d3e", "3k_main_ceo_trait_personality_clever"),
            ("b0f2d907-e00a-4027-a629-a6e179c05fed", "3k_main_ceo_trait_personality_competative"),
            ("3e2bc021-186e-473d-96fc-0fdac5e291c9", "3k_main_ceo_trait_personality_cowardly"),
            ("dda3ffab-2d17-4f01-ae42-74ca3aa7a670", "3k_main_ceo_trait_personality_cruel"),
            ("a22964c3-8fea-4c9c-a057-3d963fa9f35c", "3k_main_ceo_trait_personality_cunning"),
            ("4921e91a-e68a-4250-bd3d-78f90ed7db2f", "3k_main_ceo_trait_personality_deceitful"),
            ("141cf773-4da3-45e2-83bc-0c0367f26b85", "3k_main_ceo_trait_personality_defiant"),
            ("2dfc8d3f-ffdc-4aa3-83aa-25a25925c8a2", "3k_main_ceo_trait_personality_determined"),
            ("3a8aa192-148b-4ecb-9811-d0843ef4c79b", "3k_main_ceo_trait_personality_direct"),
            ("6e2841f4-6be7-4a25-b312-433eacf3929d", "3k_main_ceo_trait_personality_disciplined"),
            ("8830f998-f940-4b65-bdb3-3d11cda8f21b", "3k_main_ceo_trait_personality_disloyal"),
            ("f400fe2c-71e7-4bc0-963b-a337d9e372e0", "3k_main_ceo_trait_personality_distinguished"),
            ("be172401-63b1-4160-88be-6143885f07cc", "3k_main_ceo_trait_personality_dutiful"),
            ("612147d4-0e9c-4946-93d3-dc8e5d0c0273", "3k_main_ceo_trait_personality_elusive"),
            ("ba8fa67d-df0c-443f-a97c-2cd517ae95a1", "3k_main_ceo_trait_personality_energetic"),
            ("67c7e4ae-7857-48a6-ab68-7e8c0f3acd77", "3k_main_ceo_trait_personality_enigmatic"),
            ("9d6745b6-1a21-4115-84de-cad3fd84d9c9", "3k_main_ceo_trait_personality_fiery"),
            ("24d0537d-1255-46bf-841a-280c911e6d69", "3k_main_ceo_trait_personality_fraternal"),
            ("1791d7fb-f4f1-4a08-9b51-bf8c0bf262c4", "3k_main_ceo_trait_personality_greedy"),
            ("4dc50813-10d6-4958-9224-c9f76ab6a71d", "3k_main_ceo_trait_personality_honourable"),
            ("2cb4ea6d-2563-4e27-b491-acb54d3575a1", "3k_main_ceo_trait_personality_humble"),
            ("128f4caa-b1be-4814-a994-7f7b102ab3e6", "3k_main_ceo_trait_personality_incompetent"),
            ("0fb43f29-5804-44de-853f-2b1fb380fd7b", "3k_main_ceo_trait_personality_indecisive"),
            ("60b763f4-4113-4d6d-9d20-41c14c101ed4", "3k_main_ceo_trait_personality_intimidating"),
            ("394ae8f2-354d-4e55-8fd8-ee3da7bd667f", "3k_main_ceo_trait_personality_kind"),
            ("488dec01-c0b1-442d-b391-1260b52a7cdd", "3k_main_ceo_trait_personality_loyal"),
            ("62a00784-5071-458e-9c5d-26aefb73adca", "3k_main_ceo_trait_personality_modest"),
            ("c6f3979e-8473-4d72-b553-b014a6872689", "3k_main_ceo_trait_personality_patient"),
            ("497fa6e9-03e2-425f-bef4-a2271ef7385b", "3k_main_ceo_trait_personality_pacifist"),
            ("a5afd56d-5321-413b-b78a-d0f80031bc86", "3k_main_ceo_trait_personality_perceptive"),
            ("0ffce8cb-e8b4-471f-8505-006d4b11cdaf", "3k_main_ceo_trait_personality_reckless"),
            ("421ef29b-8459-4c48-98ac-9de8b0eae820", "3k_main_ceo_trait_personality_resourceful"),
            ("bceea0ea-78d9-46a6-a039-41f26107428a", "3k_main_ceo_trait_personality_scholarly"),
            ("21464c4e-cf9b-413b-8ed2-e560bebea536", "3k_main_ceo_trait_personality_sincere"),
            ("d49696e4-c267-40d2-82ca-3e7a4528fc61", "3k_main_ceo_trait_personality_solitary"),
            ("3e2f03b4-1686-46cb-9234-a3c925c1501a", "3k_main_ceo_trait_personality_stubborn"),
            ("6f8a5744-e26f-4726-a457-e484c6be14be", "3k_main_ceo_trait_personality_superstitious"),
            ("0eb4614a-858b-4141-9e71-bd30b9d1c48b", "3k_main_ceo_trait_personality_suspicious"),
            ("12aafce5-eec1-404a-97af-f238081a0dd9", "3k_main_ceo_trait_personality_trusting"),
            ("f07741be-1b78-41e7-904a-4352abedcd91", "3k_main_ceo_trait_personality_unobservant"),
            ("fcdfb641-6683-47ef-9f15-c93c2667f961", "3k_main_ceo_trait_personality_vain"),
            ("946c69b1-fc63-4151-b229-ca065c35c839", "3k_main_ceo_trait_personality_vengeful"),
            ("4e6606d0-db05-4f4f-8e2b-b852691f507f", "3k_dlc06_ceo_trait_personality_animal_friend"),
            ("c589bc34-2fcc-416b-9dff-49125874fee6", "3k_dlc07_ceo_trait_personality_frivolous"),
            ("08cced1c-8fb3-4ee0-9cdc-e5e60930b0a7", "3k_ytr_ceo_trait_personality_benevolent"),
            ("05599324-87bd-469d-853c-e786bcf85242", "3k_ytr_ceo_trait_personality_gentle_hearted"),
            ("620b5da2-27d4-4453-add0-3538026d2180", "3k_ytr_ceo_trait_personality_heaven_creative"),
            ("9282ed80-c4c2-4cc1-aa36-3afe61633c53", "3k_ytr_ceo_trait_personality_heaven_bright"),
            ("50e4be5b-7f1d-4016-bff2-e387018a087b", "3k_ytr_ceo_trait_personality_heaven_honest"),
            ("41d52b56-3cdd-4f3d-a474-01225896777e", "3k_ytr_ceo_trait_personality_heaven_selfless"),
            ("0fcb16e4-f269-4ae5-bccd-91e34a939798", "3k_ytr_ceo_trait_personality_heaven_tolerant"),
            ("f8b9e085-6f61-45b0-af5d-88d5fda0a41f", "3k_ytr_ceo_trait_personality_heaven_tranquil"),
            ("091b51ba-4a1d-43ed-a9bc-a58bffe64ecd", "3k_ytr_ceo_trait_personality_heaven_wise"),
            ("34b5f6bc-7128-4b17-a6d5-bffee45d1445", "3k_ytr_ceo_trait_personality_land_alert"),
            ("450f9244-0b51-4677-87e9-5e9c7789fcb0", "3k_ytr_ceo_trait_personality_land_aspiring"),
            ("4f24b4d8-59c3-4882-8679-71ace42989dd", "3k_ytr_ceo_trait_personality_land_composed"),
            ("9166ffec-ff04-491d-ae02-eaf54cac4817", "3k_ytr_ceo_trait_personality_land_courageous"),
            ("bcca917c-5621-4099-bfd9-389ea7afa0af", "3k_ytr_ceo_trait_personality_land_generous"),
            ("b103f0e8-344f-4509-8e47-5f3a4ae69f9b", "3k_ytr_ceo_trait_personality_land_powerful"),
            ("e742aed5-7845-4155-afb1-1c77dde01c06", "3k_ytr_ceo_trait_personality_land_proud"),
            ("a682db5a-bc0a-4ac7-99ab-ec18063d035b", "3k_ytr_ceo_trait_personality_people_compassionate"),
            ("c86ac2ca-f798-44ec-b318-dd2a8d021dbf", "3k_ytr_ceo_trait_personality_people_amiable"),
            ("5ba8d3ac-a7db-4f9f-877e-d13c19829655", "3k_ytr_ceo_trait_personality_people_cheerful"),
            ("47e9c026-b41a-4035-8483-6d9ea8d98c14", "3k_ytr_ceo_trait_personality_people_friendly"),
            ("d86a74ce-2f66-4f69-aacf-5c8d3cc94bc2", "3k_ytr_ceo_trait_personality_people_people_pleaser"),
            ("4964463f-e2af-4803-997a-85bf58937171", "3k_ytr_ceo_trait_personality_people_stern"),
            ("1f66733d-9aa1-439a-a135-c6a4becaa42f", "3k_ytr_ceo_trait_personality_people_understanding"),
            ("ce718214-a098-4650-a162-d16cd68769d9", "3k_ytr_ceo_trait_personality_relentless"),
            ("8e5ac082-3fb5-437a-b62d-024974639289", "3k_ytr_ceo_trait_personality_stalwart"),
            ("21b52436-1a72-4894-a80f-696a151ea11c", "3k_ytr_ceo_trait_personality_strong_willed"),
            ("b7a4e25a-3f6f-4587-a38e-d7fcb1a03722", "3k_ytr_ceo_trait_personality_temperamental"),
            ("3be01b2b-842a-477d-abbe-e21739148f60", "3k_ytr_ceo_trait_personality_trustworthy"),
            ("b724c2a9-8990-4bb0-a443-16baaeb958cd", "3k_ytr_ceo_trait_personality_vindictive"),
            ("5ab1c6f3-a3a2-414a-b9b1-8c299e4c37e4", "3k_ytr_ceo_trait_personality_simple"),
            // Physical traits
            ("ef77aa8c-aadb-4086-b06d-dd89c2547e6f", "3k_main_ceo_trait_physical_agile"),
            ("2e5a1b5c-b4f4-4437-a84a-01ce949630ae", "3k_main_ceo_trait_physical_beautiful"),
            ("3fcf18c1-a5f6-4c4b-b1eb-a7f5bcfd46a7", "3k_main_ceo_trait_physical_blind"),
            ("d880a460-5f56-4083-b078-33fff466400f", "3k_main_ceo_trait_physical_clumsy"),
            ("55b77985-aa02-4202-be9b-89d67ea7e69e", "3k_main_ceo_trait_physical_coordinated"),
            ("5471c54d-c959-40e4-980f-02139bdfadba", "3k_main_ceo_trait_physical_decrepit"),
            ("9ac0f62a-1a83-4301-b93a-fdc8aecb3a3f", "3k_main_ceo_trait_physical_drunk"),
            ("b4e9aafd-b79b-450b-a811-f5a8a15c82be", "3k_main_ceo_trait_physical_eunuch"),
            ("ee70cd65-c731-4516-bd5d-f2fa2e003d09", "3k_main_ceo_trait_physical_fat"),
            ("372f4006-c06e-44cb-8459-9e159a8ec61b", "3k_main_ceo_trait_physical_fertile"),
            ("6bb06b23-55dc-4aa1-8655-b5718c2550fb", "3k_main_ceo_trait_physical_graceful"),
            ("9a5f64cb-3396-4cfa-931a-ca1ba6f30080", "3k_main_ceo_trait_physical_handsome"),
            ("d8b46f1d-4b2d-4f56-a1b9-8a933e982d8a", "3k_main_ceo_trait_physical_healthy"),
            ("f7f8c660-d5b9-433e-b29f-6a8d19305bc3", "3k_main_ceo_trait_physical_heartbroken"),
            ("ddb94412-08d7-4d87-93d4-36f874e1110c", "3k_main_ceo_trait_physical_ill"),
            ("536b6eaa-8af1-4baa-b6bc-cc267c7b9fc1", "3k_main_ceo_trait_physical_infertile"),
            ("1278743e-9bd3-46ee-9065-d3ca60710eed", "3k_main_ceo_trait_physical_lovestruck"),
            ("956fc4f2-51df-4e70-95d9-5695a3dc45a0", "3k_main_ceo_trait_physical_lumbering"),
            ("8cfb8ac5-af93-4ecf-ae96-56791c5a7ef4", "3k_main_ceo_trait_physical_mad"),
            ("5f058943-de25-4484-97c6-eb2c92506b22", "3k_main_ceo_trait_physical_maimed_arm"),
            ("d04067c9-4f2d-4260-9b36-1ddb9cdaf85b", "3k_main_ceo_trait_physical_maimed_leg"),
            ("55aebdef-5e67-4284-9b5b-95951fb93072", "3k_main_ceo_trait_physical_one-eyed"),
            ("4d45c908-e616-4c0f-a1bf-e685ce43b5af", "3k_main_ceo_trait_physical_poxxed"),
            ("802767ae-c326-4af8-a113-5dc52dafd8d8", "3k_main_ceo_trait_physical_scarred"),
            ("cfe5488a-3d60-4f11-8531-13a5d5daf9ca", "3k_main_ceo_trait_physical_sickly"),
            ("950037d4-70aa-481a-bfad-fdc9c1e5075f", "3k_main_ceo_trait_physical_shu_tiger_general"),
            ("9489eb33-af3c-4017-87c6-2134fff78201", "3k_main_ceo_trait_physical_sui_knight"),
            ("8adc8a36-d359-4940-89d3-49fc6904d282", "3k_main_ceo_trait_physical_strong"),
            ("749315ca-41c9-4019-bd2f-779fbaaab15c", "3k_main_ceo_trait_physical_tough"),
            ("5dcb5002-620f-4811-8227-a4bd0dc64565", "3k_main_ceo_trait_physical_weak"),
            ("f2feac42-1ab1-4069-af3a-42a9dc92461d", "3k_main_ceo_trait_physical_wei_elite_general"),
            ("4b625e08-f11b-4bc4-afee-b9828fa940fd", "3k_ytr_ceo_trait_physical_feared"),
            ("745807c2-f26a-4b3f-b227-b9918edb8517", "3k_ytr_ceo_trait_physical_healer_of_people"),
            ("c2adba2e-434b-41e4-bbf1-4f810aeec7f7", "3k_ytr_ceo_trait_physical_impeccable"),
            ("385024c6-d7c6-4206-a896-f6d246cbca19", "3k_ytr_ceo_trait_physical_leader_of_people"),
            ("fa3781b6-4b94-4728-8804-8cdd1166abf0", "3k_ytr_ceo_trait_physical_protector_of_people"),
            ("3af98781-93d6-43b3-8f94-b03ff1ed3138", "3k_ytr_ceo_trait_physical_sprained_ankle"),
            ("38ecd1b1-392d-4fa8-8798-9fa93749214b", "3k_ytr_ceo_trait_physical_wound"),
        ];

        // Populate list widget — store "uuid|key" in tooltip for retrieval
        for (uuid, key) in raw_traits {
            let suffix = if let Some(pos) = key.find("_trait_") { &key[pos + "_trait_".len()..] } else { key };
            let display = suffix.split('_').map(|w| {
                let mut c = w.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            }).collect::<Vec<_>>().join(" ");
            let item = QListWidgetItem::from_q_string(&QString::from_std_str(&display));
            item.set_tool_tip(&QString::from_std_str(&format!("{}|{}", uuid, key)));
            trait_list_widget.add_item_q_list_widget_item(item.into_ptr());
        }

        // ── Trait filter ──────────────────────────────────────────────────────
        let trait_list_ptr = trait_list_widget.as_ptr();
        let trait_count_ptr = trait_count_label.as_ptr();
        let filter_ptr = trait_filter_line_edit.as_ptr();

        let update_filter = SlotNoArgs::new(&dialog, move || {
            let filter = filter_ptr.text().to_std_string().to_lowercase();
            for i in 0..trait_list_ptr.count() {
                let item = trait_list_ptr.item(i);
                let text = item.text().to_std_string().to_lowercase();
                item.set_hidden(!filter.is_empty() && !text.contains(&filter));
            }
        });
        trait_filter_line_edit.text_changed().connect(&update_filter);

        // Update selected count; disable unselected items when 3 are chosen
        let update_count = SlotNoArgs::new(&dialog, move || {
            let count = trait_list_ptr.selected_items().count();
            trait_count_ptr.set_text(&QString::from_std_str(&format!("Selected: {}/3", count)));
            let enabled_flags = (ItemFlag::ItemIsEnabled | ItemFlag::ItemIsSelectable).into();
            let disabled_flags = ItemFlag::ItemIsSelectable.into(); // no ItemIsEnabled
            for i in 0..trait_list_ptr.count() {
                let item = trait_list_ptr.item(i);
                if count >= 3 && !item.is_selected() {
                    item.set_flags(disabled_flags);
                } else {
                    item.set_flags(enabled_flags);
                }
            }
        });
        trait_list_widget.item_selection_changed().connect(&update_count);

        // ── Queue table setup ─────────────────────────────────────────────────
        queue_table_widget.set_column_count(8);
        queue_table_widget.horizontal_header().set_stretch_last_section(true);
        queue_table_widget.horizontal_header()
            .resize_sections(qt_widgets::q_header_view::ResizeMode::ResizeToContents);

        // ── Shared pointers for slots ─────────────────────────────────────────
        let queue_ptr = queue_table_widget.as_ptr();
        let name_ptr = name_line_edit.as_ptr();
        let type_ptr = type_combo_box.as_ptr();
        let element_ptr = element_combo_box.as_ptr();
        let gender_ptr = gender_combo_box.as_ptr();
        let expanded_ptr = expanded_check_box.as_ptr();
        let status_ptr = status_label.as_ptr();
        let trait_list_ptr2 = trait_list_widget.as_ptr();

        // ── Add Character ─────────────────────────────────────────────────────
        let add_character = SlotNoArgs::new(&dialog, move || {
            let name = name_ptr.text().to_std_string();
            let name = name.trim();
            if name.is_empty() {
                status_ptr.set_text(&QString::from_std_str("ERR: Name cannot be empty."));
                return;
            }
            let selected = trait_list_ptr2.selected_items();
            if selected.count() != 3 {
                status_ptr.set_text(&QString::from_std_str("ERR: Select exactly 3 traits."));
                return;
            }
            let row = queue_ptr.row_count();
            queue_ptr.insert_row(row);

            let make_item = |text: &str| QTableWidgetItem::from_q_string(&QString::from_std_str(text)).into_ptr();

            queue_ptr.set_item(row, 0, make_item(name));
            queue_ptr.set_item(row, 1, make_item(&type_ptr.current_text().to_std_string()));
            queue_ptr.set_item(row, 2, make_item(&element_ptr.current_text().to_std_string()));
            queue_ptr.set_item(row, 3, make_item(&gender_ptr.current_text().to_std_string()));
            queue_ptr.set_item(row, 4, make_item(if expanded_ptr.is_checked() { "true" } else { "false" }));

            for i in 0..3i64 {
                let trait_item = &**selected.at(i);
                let display = trait_item.text().to_std_string();
                let data = trait_item.tool_tip().to_std_string();
                let cell = QTableWidgetItem::from_q_string(&QString::from_std_str(&display));
                cell.set_tool_tip(&QString::from_std_str(&data));
                queue_ptr.set_item(row, 5 + i as i32, cell.into_ptr());
            }

            name_ptr.clear();
            trait_list_ptr2.clear_selection();
            status_ptr.set_text(&QString::from_std_str(&format!("OK: {} character(s) in queue.", queue_ptr.row_count())));
        });
        add_character_button.released().connect(&add_character);

        // ── Clear All ─────────────────────────────────────────────────────────
        let clear_all = SlotNoArgs::new(&dialog, move || {
            queue_ptr.set_row_count(0);
            status_ptr.set_text(&QString::from_std_str("Queue cleared."));
        });
        clear_all_button.released().connect(&clear_all);

        // ── Delete Selected ───────────────────────────────────────────────────
        let delete_selected = SlotNoArgs::new(&dialog, move || {
            let selected = queue_ptr.selected_items();
            if selected.is_empty() { return; }
            let mut rows: Vec<i32> = (0..selected.count())
                .map(|i| queue_ptr.row(*selected.at(i)))
                .collect();
            rows.sort_unstable();
            rows.dedup();
            for r in rows.into_iter().rev() {
                queue_ptr.remove_row(r);
            }
            status_ptr.set_text(&QString::from_std_str(
                &format!("{} character(s) in queue.", queue_ptr.row_count())
            ));
        });
        delete_selected_button.released().connect(&delete_selected);

        // ── Run button ────────────────────────────────────────────────────────
        let run_button = button_box.add_button_q_string_button_role(
            &QString::from_std_str("Run"), ButtonRole::AcceptRole
        );
        let run_button_ptr = run_button.as_ptr();
        let dialog_ptr = dialog.as_ptr();
        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let pack_key_closure = pack_key.clone();

        // Store paths returned by BuildCeoEntries so the exec() block can use them.
        let added_paths = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ContainerPath>::new()));
        let added_paths_closure = added_paths.clone();

        let run_slot = SlotNoArgs::new(&dialog, move || {
            if queue_ptr.row_count() == 0 {
                status_ptr.set_text(&QString::from_std_str("ERR: Queue is empty."));
                return;
            }

            let mut entries: Vec<CeoEntryData> = Vec::new();
            for row in 0..queue_ptr.row_count() {
                let get = |col: i32| -> String { (*queue_ptr.item(row, col)).text().to_std_string() };
                let get_trait = |col: i32| -> (String, String) {
                    let data = (*queue_ptr.item(row, col)).tool_tip().to_std_string();
                    let mut parts = data.splitn(2, '|');
                    let uuid = parts.next().unwrap_or("").to_string();
                    let key  = parts.next().unwrap_or("").to_string();
                    (uuid, key)
                };
                entries.push(CeoEntryData {
                    name:     get(0),
                    option:   get(1),
                    element:  get(2),
                    gender:   get(3),
                    expanded: get(4) == "true",
                    traits:   vec![get_trait(5), get_trait(6), get_trait(7)],
                });
            }

            run_button_ptr.set_enabled(false);
            status_ptr.set_text(&QString::from_std_str("Running..."));

            match send_ipc_command_result_async(
                Command::BuildCeoEntries(pack_key_closure.clone(), entries),
                response_extractor!(Response::VecContainerPath)
            ) {
                Ok(paths) => {
                    status_ptr.set_text(&QString::from_std_str(
                        &format!("OK: {} file(s) updated.", paths.len())
                    ));
                    *added_paths_closure.borrow_mut() = paths;
                    dialog_ptr.done(1);
                }
                Err(error) => {
                    run_button_ptr.set_enabled(true);
                    status_ptr.set_text(&QString::from_std_str(&format!("ERR: {}", error)));
                }
            }
        });
        run_button.released().connect(&run_slot);

        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_reject());

        // ── Execute ───────────────────────────────────────────────────────────
        if dialog.exec() == 1 {
            let paths = added_paths.borrow().clone();
            if !paths.is_empty() {
                pack_file_contents_ui.packfile_contents_tree_view()
                    .update_treeview(true, TreeViewOperation::Add(paths), DataSource::PackFile, &pack_key);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }
        }
        Ok(())
    }

    /// This function is used to mass-update anim ids after an update.
    pub unsafe fn update_anim_ids(app_ui: &Rc<Self>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

        // We need to close all anim files before doing this, or their view may get skew. It should really be only the AnimFragment files, but I'm too lazy right now to do it.
        let _ = AppUI::purge_the_local_ones(app_ui, pack_file_contents_ui, false);

        let template_path = if cfg!(debug_assertions) { UPDATE_ANIM_IDS_VIEW_DEBUG } else { UPDATE_ANIM_IDS_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;
        let dialog = main_widget.static_downcast::<QDialog>();

        // Create and configure the dialog.
        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let starting_id_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "starting_id_label")?;
        let offset_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "offset_label")?;
        let instructions_groubox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "instructions_groubox")?;
        let starting_id_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "starting_id_spinbox")?;
        let offset_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "offset_spinbox")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        dialog.set_window_title(&qtr("update_anim_ids"));
        instructions_groubox.set_title(&qtr("instructions"));
        instructions_label.set_word_wrap(true);
        instructions_label.set_text(&qtr("update_anim_ids_instructions"));
        starting_id_label.set_text(&qtr("starting_id"));
        offset_label.set_text(&qtr("offset"));

        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let starting_id = starting_id_spinbox.value();
            let offset = offset_spinbox.value();
            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            let paths = send_ipc_command_result_async(Command::UpdateAnimIds(pack_key.clone(), starting_id, offset), response_extractor!(Response::VecContainerPath))?;
            if !paths.is_empty() {
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Modify(paths.clone()), DataSource::PackFile, &pack_key);
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths), DataSource::PackFile, &pack_key);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}
