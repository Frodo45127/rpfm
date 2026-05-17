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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QApplication;
#[cfg(feature = "only_for_the_brave")] use qt_widgets::QMessageBox;
#[cfg(feature = "only_for_the_brave")] use qt_widgets::q_message_box::Icon;
#[cfg(feature = "only_for_the_brave")] use qt_widgets::q_message_box::StandardButton;

use qt_gui::QFont;
use qt_gui::QIcon;

use qt_core::QByteArray;
use qt_core::QCoreApplication;
use qt_core::q_event_loop::ProcessEventsFlag;
use qt_core::QFlags;
use qt_core::QString;
use qt_core::WindowState;

use anyhow::Result;

use std::env::args;
use std::path::PathBuf;
use std::rc::Rc;
use std::fs::{read_dir, remove_dir_all};
use std::sync::atomic::AtomicPtr;

use rpfm_ipc::helpers::DataSource;
use rpfm_ipc::settings_keys::*;

use rpfm_lib::games::supported_games::*;
use rpfm_telemetry::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::PROGRAM_PATH;
use rpfm_ui_common::utils::{atomic_from_cpp_box, ref_from_atomic};

#[cfg(feature = "only_for_the_brave")] use crate::VERSION;
use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::{AppUITempSlots, AppUISlots};
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
use crate::references_ui;
use crate::references_ui::ReferencesUI;
use crate::references_ui::slots::ReferencesUISlots;
use crate::SUPPORTED_GAMES;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;
use crate::settings_ui::backend::*;
use crate::UI_STATE;
use crate::updater_ui::UpdaterUI;
use crate::utils::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to EVERY widget/action created at the start of the program.
///
/// This means every widget/action that's created on start (menus, the TreeView,...) should be here.
pub struct UI {
    pub app_ui: Rc<AppUI>,
    pub _pack_file_contents_ui: Rc<PackFileContentsUI>,
    pub _global_search_ui: Rc<GlobalSearchUI>,
    pub _diagnostics_ui: Rc<DiagnosticsUI>,
    pub _dependencies_ui: Rc<DependenciesUI>,
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
    pub pharaoh_dynasties: (AtomicPtr<QIcon>, String),
    pub pharaoh: (AtomicPtr<QIcon>, String),
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

    /// Build every Qt widget, slot and connection without touching the WebSocket.
    pub unsafe fn build_offline() -> Result<Self> {
        let app_ui = Rc::new(AppUI::new());
        let global_search_ui = Rc::new(GlobalSearchUI::new(app_ui.main_window())?);
        let pack_file_contents_ui = Rc::new(PackFileContentsUI::new(&app_ui)?);
        let diagnostics_ui = Rc::new(DiagnosticsUI::new(&app_ui)?);
        let dependencies_ui = Rc::new(DependenciesUI::new(&app_ui)?);
        let references_ui = Rc::new(ReferencesUI::new(app_ui.main_window())?);

        // Show the main-window skeleton as soon as it exists.
        log_to_status_bar("Loading interface...");
        app_ui.toggle_main_window(false);
        app_ui.main_window().show();
        QCoreApplication::process_events_q_flags_process_events_flag_int(
            QFlags::from(ProcessEventsFlag::AllEvents),
            100,
        );

        let app_slots = AppUISlots::new(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let pack_file_contents_slots = PackFileContentsSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let global_search_slots = GlobalSearchSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let diagnostics_slots = DiagnosticsUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let dependencies_slots = DependenciesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let references_slots = ReferencesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&pack_file_contents_ui);

        dependencies_ui::connections::set_connections(&dependencies_ui, &dependencies_slots);
        dependencies_ui::tips::set_tips(&dependencies_ui);

        diagnostics_ui::connections::set_connections(&diagnostics_ui, &diagnostics_slots);
        references_ui::connections::set_connections(&references_ui, &references_slots);

        // Apply last ui state. The settings cache is loaded from disk in
        // `main()`, so these reads are non-blocking.
        app_ui.main_window().restore_geometry(&QByteArray::from_slice(&settings_raw_data(GEOMETRY)));
        app_ui.main_window().restore_state_1a(&QByteArray::from_slice(&settings_raw_data(WINDOW_STATE)));

        // Apply the font.
        let font_name = settings_string(FONT_NAME);
        let font_size = settings_i32(FONT_SIZE);
        let font = QFont::from_q_string_int(&QString::from_std_str(font_name), font_size);
        QApplication::set_font_1a(&font);

        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

        if settings_bool(START_MAXIMIZED) {
            app_ui.main_window().set_window_state(QFlags::from(WindowState::WindowMaximized));
        }

        reload_theme(&app_ui);

        // Clean up folders from previous updates while we wait for the
        // WebSocket. Filesystem-only, no IPC dependency.
        if !cfg!(debug_assertions) {
            if let Ok(folders) = read_dir(&*PROGRAM_PATH) {
                for folder in folders.flatten() {
                    let folder_path = folder.path();
                    if folder_path.is_dir() && folder_path.file_name().unwrap().to_string_lossy().starts_with("update") {
                        let _ = remove_dir_all(&folder_path);
                    }
                }
                info!("Update folders cleared.");
            }
        }

        // Window's already shown and disabled (see top of function).
        log_to_status_bar("Connecting to backend...");

        Ok(Self {
            app_ui,
            _global_search_ui: global_search_ui,
            _pack_file_contents_ui: pack_file_contents_ui,
            _diagnostics_ui: diagnostics_ui,
            _dependencies_ui: dependencies_ui
        })
    }

    /// Run every initialisation step that needs the WebSocket to be alive.
    pub unsafe fn finish_online_init(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) {
        // Load the settings cache from the server.
        load_settings_cache_from_server();

        // Now that the server has replied with telemetry settings, honour them.
        rpfm_telemetry::set_usage_telemetry_enabled(settings_bool(ENABLE_USAGE_TELEMETRY));
        rpfm_telemetry::set_crash_reports_enabled(settings_bool(ENABLE_CRASH_REPORTS));

        // Initialize settings. Ignore errors here, as we have no way to show them yet.
        init_app_exclusive_settings(app_ui);

        // This is here because this needs the connection to be operative to work.
        AppUITempSlots::build(app_ui, pack_file_contents_ui, global_search_ui);

        // Do not trigger the automatic game changed signal here, as that will trigger an expensive and useless dependency rebuild.
        info!("Setting initial Game Selected…");
        match &*settings_string(DEFAULT_GAME) {
            KEY_PHARAOH_DYNASTIES => app_ui.game_selected_pharaoh_dynasties().set_checked(true),
            KEY_PHARAOH => app_ui.game_selected_pharaoh().set_checked(true),
            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3().set_checked(true),
            KEY_TROY => app_ui.game_selected_troy().set_checked(true),
            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms().set_checked(true),
            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2().set_checked(true),
            KEY_WARHAMMER => app_ui.game_selected_warhammer().set_checked(true),
            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia().set_checked(true),
            KEY_ATTILA => app_ui.game_selected_attila().set_checked(true),
            KEY_ROME_2 => app_ui.game_selected_rome_2().set_checked(true),
            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2().set_checked(true),
            KEY_NAPOLEON => app_ui.game_selected_napoleon().set_checked(true),
            KEY_EMPIRE => app_ui.game_selected_empire().set_checked(true),
            KEY_ARENA  => app_ui.game_selected_arena().set_checked(true),

            // Turns out some... lets say "not very bright individual" changed the settings file manually and broke this.
            // So just in case, by default we use WH3.
            _ => app_ui.game_selected_warhammer_3().set_checked(true),
        }

        AppUI::change_game_selected(app_ui, pack_file_contents_ui, dependencies_ui, true, false);
        info!("Initial Game Selected set to {}.", settings_string(DEFAULT_GAME));

        // We get all the Arguments provided when starting RPFM, just in case we passed it a path,
        // in which case, we automatically try to open it.
        let args = args().collect::<Vec<String>>();
        if args.len() > 1 {
            let mut paths = args[1..].iter().map(PathBuf::from).collect::<Vec<_>>();
            let mut rfiles = vec![];

            // If the last path is not a pack, we consider it a file in the pack.
            if paths.len() > 1 {
                let mut paths_to_remove = vec![];
                for (index, path) in paths.iter().enumerate() {
                    if path.file_name().is_some_and(|x| !x.to_string_lossy().ends_with(".pack")) {
                        paths_to_remove.push(index);
                    }
                }

                paths_to_remove.reverse();
                for index in &paths_to_remove {
                    rfiles.push(paths.remove(*index));
                }
            }

            // Remove non-file paths.
            paths = paths.into_iter().filter(|path| path.is_file()).collect::<Vec<_>>();

            if !paths.is_empty() {
                info!("Directly opening Pack/s {paths:?}.");
                if let Err(error) = AppUI::open_packfile(app_ui, pack_file_contents_ui, global_search_ui, dependencies_ui, &paths, "", false) {
                    show_dialog(app_ui.main_window(), error, false);

                } else {

                    // Ignore errors here.
                    if !rfiles.is_empty() {
                        for file in &rfiles {
                            let path = file.to_string_lossy().to_string();
                            AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path), false, false, DataSource::PackFile);
                        }
                    }

                    if settings_bool(DIAGNOSTICS_TRIGGER_ON_OPEN) {
                        DiagnosticsUI::check(app_ui, diagnostics_ui);
                    }
                }
            }
        }

        // Check for updates, ignoring errors.
        let _ = UpdaterUI::new_with_precheck(app_ui);

        // Show the "only for the brave" alert for specially unstable builds.
        #[cfg(feature = "only_for_the_brave")] {
            let first_boot_setting = "firstBoot".to_owned() + VERSION;
            if !settings_bool(&first_boot_setting) {

                let title = qtr("title_only_for_the_brave");
                let message = qtr("message_only_for_the_brave");
                QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                    Icon::Warning,
                    &title,
                    &message,
                    QFlags::from(StandardButton::Ok),
                    app_ui.main_window(),
                ).exec();

                // Set it so it doesn't popup again for this version.
                set_settings_bool(&first_boot_setting, true);
            }
        }

        // Re-enable input — the user can finally drive the app.
        app_ui.toggle_main_window(true);
        log_to_status_bar("Initialization complete.");
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub unsafe fn new() -> Self {
        Self {
            pharaoh_dynasties: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH_DYNASTIES).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH_DYNASTIES).unwrap().icon_big())),
            pharaoh: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_PHARAOH).unwrap().icon_big())),
            warhammer_3: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_big())),
            troy: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_big())),
            three_kingdoms: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_big())),
            warhammer_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_big())),
            warhammer: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_big())),
            thrones_of_britannia: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_big())),
            attila: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_big())),
            rome_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_big())),
            shogun_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_big())),
            napoleon: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_big())),
            empire: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_big())),
            arena: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_small())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_big())),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub unsafe fn set_game_selected_icon(app_ui: &Rc<AppUI>) {
        let (icon, _big_icon) = match GAME_SELECTED.read().unwrap().key() {
            KEY_PHARAOH_DYNASTIES => &GAME_SELECTED_ICONS.pharaoh_dynasties,
            KEY_PHARAOH => &GAME_SELECTED_ICONS.pharaoh,
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
        app_ui.main_window().set_window_icon(ref_from_atomic(icon));

        app_ui.toggle_welcome_visibility();
    }
}
