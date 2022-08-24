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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QApplication;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::QMessageBox;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::q_message_box::Icon;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::q_message_box::StandardButton;

use qt_gui::QFont;
use qt_gui::{QColor, q_color::NameFormat};
use qt_gui::QIcon;

use qt_core::QFlags;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::WindowState;

use cpp_core::Ptr;

use std::env::args;
use std::path::PathBuf;
use std::rc::Rc;
use std::fs::{read_dir, remove_dir_all};
use std::sync::atomic::AtomicPtr;

use rpfm_lib::games::supported_games::*;
use rpfm_lib::integrations::log::*;

#[cfg(feature = "only_for_the_brave")]
use crate::VERSION;
use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::{AppUITempSlots, AppUISlots};
use crate::ASSETS_PATH;
use crate::DARK_PALETTE;
use crate::DARK_STYLESHEET;
use crate::dependencies_ui;
use crate::dependencies_ui::DependenciesUI;
use crate::dependencies_ui::slots::DependenciesUISlots;
use crate::diagnostics_ui;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::diagnostics_ui::slots::DiagnosticsUISlots;
use crate::GAME_SELECTED;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::LIGHT_PALETTE;
use crate::references_ui;
use crate::references_ui::ReferencesUI;
use crate::references_ui::slots::ReferencesUISlots;
use crate::SUPPORTED_GAMES;

#[cfg(feature = "only_for_the_brave")]
use crate::locale::qtr;
use crate::locale::tr;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;
use crate::RPFM_PATH;
use crate::settings_ui::backend::*;
use crate::UI_STATE;
use crate::utils::atomic_from_cpp_box;
use crate::utils::show_dialog;
use crate::utils::ref_from_atomic;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to EVERY widget/action created at the start of the program.
///
/// This means every widget/action that's created on start (menus, the TreeView,...) should be here.
pub struct UI {
    pub app_ui: Rc<AppUI>,
    pub pack_file_contents_ui: Rc<PackFileContentsUI>,
    pub global_search_ui: Rc<GlobalSearchUI>,
    pub diagnostics_ui: Rc<DiagnosticsUI>,
    pub dependencies_ui: Rc<DependenciesUI>,
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
    pub warhammer_3: (AtomicPtr<QIcon>, String),
    pub troy: (AtomicPtr<QIcon>, String),
    pub three_kingdoms: (AtomicPtr<QIcon>, String),
    pub warhammer_2: (AtomicPtr<QIcon>, String),
    pub warhammer: (AtomicPtr<QIcon>, String),
    pub thrones_of_britannia: (AtomicPtr<QIcon>, String),
    pub attila: (AtomicPtr<QIcon>, String),
    pub rome_2: (AtomicPtr<QIcon>, String),
    pub shogun_2: (AtomicPtr<QIcon>, String),
    pub napoleon: (AtomicPtr<QIcon>, String),
    pub empire: (AtomicPtr<QIcon>, String),
    pub arena: (AtomicPtr<QIcon>, String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `UI`.
impl UI {

    /// This function initialize the entire `UI`.
    pub unsafe fn new(app: Ptr<QApplication>) -> Self {
        let app_ui = Rc::new(AppUI::new());
        let global_search_ui = Rc::new(GlobalSearchUI::new(&app_ui.main_window));
        let pack_file_contents_ui = Rc::new(PackFileContentsUI::new(&app_ui.main_window));
        let diagnostics_ui = Rc::new(DiagnosticsUI::new(&app_ui.main_window));
        let dependencies_ui = Rc::new(DependenciesUI::new(&app_ui.main_window));
        let references_ui = Rc::new(ReferencesUI::new(&app_ui.main_window));

        AppUITempSlots::build(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);

        let app_slots = AppUISlots::new(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let pack_file_contents_slots = PackFileContentsSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let global_search_slots = GlobalSearchSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let diagnostics_slots = DiagnosticsUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let dependencies_slots = DependenciesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let references_slots = ReferencesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&app_ui);
        app_ui::shortcuts::set_shortcuts(&app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&global_search_ui);
        global_search_ui::shortcuts::set_shortcuts(&global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&pack_file_contents_ui);
        packfile_contents_ui::shortcuts::set_shortcuts(&pack_file_contents_ui);

        dependencies_ui::connections::set_connections(&dependencies_ui, &dependencies_slots);
        dependencies_ui::tips::set_tips(&dependencies_ui);
        dependencies_ui::shortcuts::set_shortcuts(&dependencies_ui);

        references_ui::connections::set_connections(&references_ui, &references_slots);
        diagnostics_ui::connections::set_connections(&diagnostics_ui, &diagnostics_slots);

        // Apply last ui state.
        // TODO: Move all this to settings.
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));

        if !q_settings.contains(&QString::from_std_str("originalGeometry")) {
            q_settings.set_value(&QString::from_std_str("originalGeometry"), &QVariant::from_q_byte_array(&app_ui.main_window.save_geometry()));
            q_settings.set_value(&QString::from_std_str("originalWindowState"), &QVariant::from_q_byte_array(&app_ui.main_window.save_state_0a()));
        }

        app_ui.main_window.restore_geometry(&q_settings.value_1a(&QString::from_std_str("geometry")).to_byte_array());
        app_ui.main_window.restore_state_1a(&q_settings.value_1a(&QString::from_std_str("windowState")).to_byte_array());

        info!("Qt-specific settings loaded.");

        // Initialize colours.
        let mut sync_needed = false;

        let mut colour_light_table_added = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_table_added")).to_string());
        let mut colour_light_table_modified = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_table_modified")).to_string());
        let mut colour_light_diagnostic_error = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_error")).to_string());
        let mut colour_light_diagnostic_warning = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_warning")).to_string());
        let mut colour_light_diagnostic_info = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_info")).to_string());
        let mut colour_dark_table_added = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_table_added")).to_string());
        let mut colour_dark_table_modified = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_table_modified")).to_string());
        let mut colour_dark_diagnostic_error = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_error")).to_string());
        let mut colour_dark_diagnostic_warning = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_warning")).to_string());
        let mut colour_dark_diagnostic_info = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_info")).to_string());

        let mut colour_light_local_tip = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_local_tip")).to_string());
        let mut colour_light_remote_tip = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_remote_tip")).to_string());
        let mut colour_dark_local_tip = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_local_tip")).to_string());
        let mut colour_dark_remote_tip = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_remote_tip")).to_string());

        if !colour_light_table_added.is_valid() {
            colour_light_table_added = QColor::from_q_string(&QString::from_std_str("#87ca00"));
            q_settings.set_value(&QString::from_std_str("colour_light_table_added"), &QVariant::from_q_string(&colour_light_table_added.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_table_modified.is_valid() {
            colour_light_table_modified = QColor::from_q_string(&QString::from_std_str("#e67e22"));
            q_settings.set_value(&QString::from_std_str("colour_light_table_modified"), &QVariant::from_q_string(&colour_light_table_modified.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_diagnostic_error.is_valid() {
            colour_light_diagnostic_error = QColor::from_q_string(&QString::from_std_str("#ff0000"));
            q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_error"), &QVariant::from_q_string(&colour_light_diagnostic_error.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_diagnostic_warning.is_valid() {
            colour_light_diagnostic_warning = QColor::from_q_string(&QString::from_std_str("#bebe00"));
            q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_warning"), &QVariant::from_q_string(&colour_light_diagnostic_warning.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_diagnostic_info.is_valid() {
            colour_light_diagnostic_info = QColor::from_q_string(&QString::from_std_str("#55aaff"));
            q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_info"), &QVariant::from_q_string(&colour_light_diagnostic_info.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_table_added.is_valid() {
            colour_dark_table_added = QColor::from_q_string(&QString::from_std_str("#00ff00"));
            q_settings.set_value(&QString::from_std_str("colour_dark_table_added"), &QVariant::from_q_string(&colour_dark_table_added.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_table_modified.is_valid() {
            colour_dark_table_modified = QColor::from_q_string(&QString::from_std_str("#e67e22"));
            q_settings.set_value(&QString::from_std_str("colour_dark_table_modified"), &QVariant::from_q_string(&colour_dark_table_modified.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_diagnostic_error.is_valid() {
            colour_dark_diagnostic_error =  QColor::from_q_string(&QString::from_std_str("#ff0000"));
            q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_error"), &QVariant::from_q_string(&colour_dark_diagnostic_error.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_diagnostic_warning.is_valid() {
            colour_dark_diagnostic_warning = QColor::from_q_string(&QString::from_std_str("#cece67"));
            q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_warning"), &QVariant::from_q_string(&colour_dark_diagnostic_warning.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_diagnostic_info.is_valid() {
            colour_dark_diagnostic_info = QColor::from_q_string(&QString::from_std_str("#55aaff"));
            q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_info"), &QVariant::from_q_string(&colour_dark_diagnostic_info.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_local_tip.is_valid() {
            colour_light_local_tip = QColor::from_q_string(&QString::from_std_str("#363636"));
            q_settings.set_value(&QString::from_std_str("colour_light_local_tip"), &QVariant::from_q_string(&colour_light_local_tip.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_light_remote_tip.is_valid() {
            colour_light_remote_tip = QColor::from_q_string(&QString::from_std_str("#7e7e7e"));
            q_settings.set_value(&QString::from_std_str("colour_light_remote_tip"), &QVariant::from_q_string(&colour_light_remote_tip.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_local_tip.is_valid() {
            colour_dark_local_tip = QColor::from_q_string(&QString::from_std_str("#363636"));
            q_settings.set_value(&QString::from_std_str("colour_dark_local_tip"), &QVariant::from_q_string(&colour_dark_local_tip.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if !colour_dark_remote_tip.is_valid() {
            colour_dark_remote_tip = QColor::from_q_string(&QString::from_std_str("#7e7e7e"));
            q_settings.set_value(&QString::from_std_str("colour_dark_remote_tip"), &QVariant::from_q_string(&colour_dark_remote_tip.name_1a(NameFormat::HexArgb)));
            sync_needed = true;
        }

        if sync_needed {
            q_settings.sync();
        }

        // Here we also initialize the UI.
        UI_STATE.set_operational_mode(&app_ui, None);

        // Do not trigger the automatic game changed signal here, as that will trigger an expensive and useless dependency rebuild.
        info!("Setting initial Game Selected…");
        match &*setting_string("default_game") {
            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3.set_checked(true),
            KEY_TROY => app_ui.game_selected_troy.set_checked(true),
            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms.set_checked(true),
            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2.set_checked(true),
            KEY_WARHAMMER => app_ui.game_selected_warhammer.set_checked(true),
            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia.set_checked(true),
            KEY_ATTILA => app_ui.game_selected_attila.set_checked(true),
            KEY_ROME_2 => app_ui.game_selected_rome_2.set_checked(true),
            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.set_checked(true),
            KEY_NAPOLEON => app_ui.game_selected_napoleon.set_checked(true),
            KEY_EMPIRE => app_ui.game_selected_empire.set_checked(true),
            KEY_ARENA  => app_ui.game_selected_arena.set_checked(true),

            // Turns out some... lets say "not very bright individual" changed the settings file manually and broke this.
            // So just in case, by default we use WH3.
            _ => app_ui.game_selected_warhammer_3.set_checked(true),
        }
        AppUI::change_game_selected(&app_ui, &pack_file_contents_ui, &dependencies_ui, true);
        info!("Initial Game Selected set to {}.", setting_string("default_game"));

        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

        // If we want the window to start maximized...
        if setting_bool("start_maximized") {
            app_ui.main_window.set_window_state(QFlags::from(WindowState::WindowMaximized));
        }

        if !setting_string("font_name").is_empty() && !setting_string("font_size").is_empty() {
            let font = QFont::new();
            font.set_family(&QString::from_std_str(&setting_string("font_name")));
            font.set_point_size(setting_string("font_size").parse::<i32>().unwrap());
            QApplication::set_font_1a(&font);
        }

        // Add the icon themes path to the current list of paths where Qt searches for icons.
        let current_theme_search_path = QIcon::theme_search_paths();
        current_theme_search_path.push_front(&QString::from_std_str(&format!("{}/icons", RPFM_PATH.to_string_lossy())));
        QIcon::set_theme_search_paths(&current_theme_search_path);

        // On Windows, we use the dark theme switch to control the Style, StyleSheet and Palette.
        if cfg!(target_os = "windows") {
            if setting_bool("use_dark_theme") {
                QApplication::set_style_q_string(&QString::from_std_str("fusion"));
                QApplication::set_palette_1a(ref_from_atomic(&*DARK_PALETTE));
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
                QIcon::set_theme_name(&QString::from_std_str("breeze-dark"));
            } else {
                QApplication::set_style_q_string(&QString::from_std_str("windowsvista"));
                QApplication::set_palette_1a(ref_from_atomic(&*LIGHT_PALETTE));
                QIcon::set_theme_name(&QString::from_std_str("breeze"));
            }
        }

        // On MacOS, we use the dark theme switch to control the StyleSheet and Palette.
        else if cfg!(target_os = "macos") {
            if setting_bool("use_dark_theme") {
                QApplication::set_palette_1a(ref_from_atomic(&*DARK_PALETTE));
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
                QIcon::set_theme_name(&QString::from_std_str("breeze-dark"));
            } else {
                QApplication::set_palette_1a(ref_from_atomic(&*LIGHT_PALETTE));
                QIcon::set_theme_name(&QString::from_std_str("breeze"));
            }
        }

        // Show the Main Window...
        app_ui.main_window.show();

        // We get all the Arguments provided when starting RPFM, just in case we passed it a path,
        // in which case, we automatically try to open it.
        let args = args().collect::<Vec<String>>();
        if args.len() > 1 && args[1] != "--booted_from_launcher" {
            let path = PathBuf::from(&args[1]);
            if path.is_file() {
                info!("Directly opening PackFile {}.", path.to_string_lossy().to_string());
                if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path], "") {
                    show_dialog(&app_ui.main_window, error, false);
                } else if setting_bool("diagnostics_trigger_on_open") {
                    DiagnosticsUI::check(&app_ui, &diagnostics_ui);
                }
            }
        }

        if (args.len() == 1 || (args.len() > 1 && args.last().unwrap() != "--booted_from_launcher")) && !cfg!(debug_assertions) && cfg!(target_os = "windows") {
            show_dialog(&app_ui.main_window, &tr("error_not_booted_from_launcher"), false);
        }

        // If we have it enabled in the prefs, check if there are updates.
        if setting_bool("check_updates_on_start") { AppUI::check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if setting_bool("check_schema_updates_on_start") { AppUI::check_schema_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are message updates.
        if setting_bool("check_message_updates_on_start") { AppUI::check_message_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are lua autogen updates.
        if setting_bool("check_lua_autogen_updates_on_start") { AppUI::check_lua_autogen_updates(&app_ui, false) };

        // Clean up folders from previous updates, if they exist.
        if !cfg!(debug_assertions) {
            if let Ok(folders) = read_dir(&*RPFM_PATH) {
                for folder in folders {
                    if let Ok(folder) = folder {
                        let folder_path = folder.path();
                        if folder_path.is_dir() && folder_path.file_name().unwrap().to_string_lossy().starts_with("update") {
                            let _ = remove_dir_all(&folder_path);
                        }
                    }
                }
                info!("Update folders cleared.");
            }
        }

        // Show the "only for the brave" alert for specially unstable builds.
        #[cfg(feature = "only_for_the_brave")] {
            let first_boot_setting = QString::from_std_str("firstBoot".to_owned() + VERSION);
            if !q_settings.contains(&first_boot_setting) {

                let title = qtr("title_only_for_the_brave");
                let message = qtr("message_only_for_the_brave");
                QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                    Icon::Warning,
                    &title,
                    &message,
                    QFlags::from(StandardButton::Ok),
                    &app_ui.main_window,
                ).exec();

                // Set it so it doesn't popup again for this version.
                q_settings.set_value(&first_boot_setting, &QVariant::from_bool(true));
            }
        }
        info!("Initialization complete.");
        Self {
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui
        }
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub unsafe fn new() -> Self {
        Self {
            warhammer_3: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_big_file_name())),
            troy: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_big_file_name())),
            three_kingdoms: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_big_file_name())),
            warhammer_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_big_file_name())),
            warhammer: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_big_file_name())),
            thrones_of_britannia: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_big_file_name())),
            attila: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_big_file_name())),
            rome_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_big_file_name())),
            shogun_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_big_file_name())),
            napoleon: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_big_file_name())),
            empire: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_big_file_name())),
            arena: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_big_file_name())),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub unsafe fn set_game_selected_icon(app_ui: &Rc<AppUI>) {
        let (icon, big_icon) = match &*GAME_SELECTED.read().unwrap().game_key_name() {
            KEY_WARHAMMER_3 => &GAME_SELECTED_ICONS.warhammer_3,
            KEY_TROY => &GAME_SELECTED_ICONS.troy,
            KEY_THREE_KINGDOMS => &GAME_SELECTED_ICONS.three_kingdoms,
            KEY_WARHAMMER_2 => &GAME_SELECTED_ICONS.warhammer_2,
            KEY_WARHAMMER => &GAME_SELECTED_ICONS.warhammer,
            KEY_THRONES_OF_BRITANNIA => &GAME_SELECTED_ICONS.thrones_of_britannia,
            KEY_ATTILA => &GAME_SELECTED_ICONS.attila,
            KEY_ROME_2 => &GAME_SELECTED_ICONS.rome_2,
            KEY_SHOGUN_2 => &GAME_SELECTED_ICONS.shogun_2,
            KEY_NAPOLEON => &GAME_SELECTED_ICONS.napoleon,
            KEY_EMPIRE => &GAME_SELECTED_ICONS.empire,
            KEY_ARENA => &GAME_SELECTED_ICONS.arena,
            _ => unimplemented!(),
        };
        app_ui.main_window.set_window_icon(ref_from_atomic(&*icon));

        // Fix due to windows paths.
        let big_icon = if cfg!(target_os = "windows") {  big_icon.replace("\\", "/") } else { big_icon.to_owned() };

        if !settings_bool("hide_background_icon") {
            if app_ui.tab_bar_packed_file.count() == 0 {

                // WTF of the day: without the border line, this doesn't work on windows. Who knows why...?
                let border =  if cfg!(target_os = "windows") { "border: 0px solid #754EF9;" } else { "" };
                app_ui.tab_bar_packed_file.set_style_sheet(&QString::from_std_str(&format!("
                    QTabWidget::pane {{
                        background-image: url('{}');
                        background-repeat: no-repeat;
                        background-position: center;
                        {}
                    }}
                ", big_icon, border)));
            }
            else {

                // This is laggy after a while.
                app_ui.tab_bar_packed_file.set_style_sheet(&QString::from_std_str("QTabWidget::pane {background-image: url();}"));
            }
        }
    }
}
