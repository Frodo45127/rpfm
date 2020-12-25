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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QApplication;

use qt_gui::QFont;
use qt_gui::QIcon;

use qt_core::QFlags;
use qt_core::QString;
use qt_core::WindowState;

use cpp_core::Ptr;

use std::env::args;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::AtomicPtr;

use rpfm_lib::GAME_SELECTED;
use rpfm_lib::games::*;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::{AppUITempSlots, AppUISlots};
use crate::ASSETS_PATH;
use crate::DARK_PALETTE;
use crate::DARK_STYLESHEET;
use crate::diagnostics_ui;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::diagnostics_ui::slots::DiagnosticsUISlots;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::LIGHT_PALETTE;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;
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
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
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

        AppUITempSlots::build(&app_ui, &pack_file_contents_ui, &global_search_ui);

        let app_slots = AppUISlots::new(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui);
        let pack_file_contents_slots = PackFileContentsSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);
        let global_search_slots = GlobalSearchSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);
        let diagnostics_slots = DiagnosticsUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);
            dbg!(app_ui.main_window.palette().color_1a(qt_gui::q_palette::ColorRole::Text).name_1a(qt_gui::q_color::NameFormat::HexRgb).to_std_string());

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&app_ui);
        app_ui::shortcuts::set_shortcuts(&app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&global_search_ui);
        global_search_ui::shortcuts::set_shortcuts(&global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&pack_file_contents_ui);
        packfile_contents_ui::shortcuts::set_shortcuts(&pack_file_contents_ui);

        diagnostics_ui::connections::set_connections(&diagnostics_ui, &diagnostics_slots);

        // Here we also initialize the UI.
        UI_STATE.set_operational_mode(&app_ui, None);

        match &*SETTINGS.read().unwrap().settings_string["default_game"] {
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
            KEY_ARENA  => app_ui.game_selected_arena.trigger(),
            _ => unimplemented!()
        }

        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

        // If we want the window to start maximized...
        if SETTINGS.read().unwrap().settings_bool["start_maximized"] {
            app_ui.main_window.set_window_state(QFlags::from(WindowState::WindowMaximized));
        }

        if !SETTINGS.read().unwrap().settings_bool["enable_diagnostics_tool"] {
            app_ui.view_toggle_diagnostics_panel.toggle();
        }

        if !SETTINGS.read().unwrap().settings_string["font_name"].is_empty() && !SETTINGS.read().unwrap().settings_string["font_size"].is_empty() {
            let font = QFont::new();
            font.set_family(&QString::from_std_str(&SETTINGS.read().unwrap().settings_string["font_name"]));
            font.set_point_size(SETTINGS.read().unwrap().settings_string["font_size"].parse::<i32>().unwrap());
            QApplication::set_font_1a(&font);
        }

        // On Windows, we use the dark theme switch to control the Style, StyleSheet and Palette.
        if cfg!(target_os = "windows") {
            if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
                QApplication::set_style_q_string(&QString::from_std_str("fusion"));
                QApplication::set_palette_1a(ref_from_atomic(&*DARK_PALETTE));
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
            } else {
                QApplication::set_style_q_string(&QString::from_std_str("windowsvista"));
                QApplication::set_palette_1a(ref_from_atomic(&*LIGHT_PALETTE));
            }
        }

        // On MacOS, we use the dark theme switch to control the StyleSheet and Palette.
        else if cfg!(target_os = "macos") {
            if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
                QApplication::set_palette_1a(ref_from_atomic(&*DARK_PALETTE));
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
            } else {
                QApplication::set_palette_1a(ref_from_atomic(&*LIGHT_PALETTE));
            }
        }

        // Show the Main Window...
        app_ui.main_window.show();

        // We get all the Arguments provided when starting RPFM, just in case we passed it a path,
        // in which case, we automatically try to open it.
        let args = args().collect::<Vec<String>>();
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.is_file() {
                if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &[path], "") {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        }

        // If we have it enabled in the prefs, check if there are updates.
        if SETTINGS.read().unwrap().settings_bool["check_updates_on_start"] { AppUI::check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if SETTINGS.read().unwrap().settings_bool["check_schema_updates_on_start"] { AppUI::check_schema_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are templates updates.
        if SETTINGS.read().unwrap().settings_bool["check_template_updates_on_start"] { AppUI::check_template_updates(&app_ui, false) };

        Self {
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
        }
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub unsafe fn new() -> Self {
        Self {
            troy: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_TROY).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_TROY).unwrap().game_selected_big_icon)),
            three_kingdoms: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_THREE_KINGDOMS).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_THREE_KINGDOMS).unwrap().game_selected_big_icon)),
            warhammer_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_WARHAMMER_2).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_WARHAMMER_2).unwrap().game_selected_big_icon)),
            warhammer: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_WARHAMMER).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_WARHAMMER).unwrap().game_selected_big_icon)),
            thrones_of_britannia: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_THRONES_OF_BRITANNIA).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_THRONES_OF_BRITANNIA).unwrap().game_selected_big_icon)),
            attila: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ATTILA).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ATTILA).unwrap().game_selected_big_icon)),
            rome_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ROME_2).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ROME_2).unwrap().game_selected_big_icon)),
            shogun_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_SHOGUN_2).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_SHOGUN_2).unwrap().game_selected_big_icon)),
            napoleon: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_NAPOLEON).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_NAPOLEON).unwrap().game_selected_big_icon)),
            empire: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_EMPIRE).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_EMPIRE).unwrap().game_selected_big_icon)),
            arena: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/img/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ARENA).unwrap().game_selected_icon)))), format!("{}/img/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.get(KEY_ARENA).unwrap().game_selected_big_icon)),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub unsafe fn set_game_selected_icon(app_ui: &Rc<AppUI>) {
        let (icon, big_icon) = match &**GAME_SELECTED.read().unwrap() {
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

        if !SETTINGS.read().unwrap().settings_bool["hide_background_icon"] {
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
