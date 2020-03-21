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
use qt_widgets::QMainWindow;

use qt_gui::QIcon;

use qt_core::QFlags;
use qt_core::QString;
use qt_core::WindowState;

use cpp_core::MutPtr;

use std::cell::RefCell;
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
use crate::DARK_PALETTE;
use crate::DARK_STYLESHEET;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::LIGHT_PALETTE;
use crate::packedfile_views::TheOneSlot;
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
#[derive(Copy, Clone)]
pub struct UI {
    pub app_ui: AppUI,
    pub pack_file_contents_ui: PackFileContentsUI,
    pub global_search_ui: GlobalSearchUI,
}

/// This struct contains all the slots of the main UI, so we got all of them in one place.
pub struct Slots {
    pub app_slots: AppUISlots,
    pub app_temp_slots: Rc<RefCell<AppUITempSlots>>,
    pub pack_file_contents_slots: PackFileContentsSlots,
    pub global_search_slots: GlobalSearchSlots,
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
    pub three_kingdoms: AtomicPtr<QIcon>,
    pub warhammer_2: AtomicPtr<QIcon>,
    pub warhammer: AtomicPtr<QIcon>,
    pub thrones_of_britannia: AtomicPtr<QIcon>,
    pub attila: AtomicPtr<QIcon>,
    pub rome_2: AtomicPtr<QIcon>,
    pub shogun_2: AtomicPtr<QIcon>,
    pub napoleon: AtomicPtr<QIcon>,
    pub empire: AtomicPtr<QIcon>,
    pub arena: AtomicPtr<QIcon>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `UI`.
impl UI {

    /// This function initialize the entire `UI`.
    pub unsafe fn new(mut app: MutPtr<QApplication>, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> (Self, Slots) {
        let mut app_ui = AppUI::new();
        let mut global_search_ui = GlobalSearchUI::new(app_ui.main_window);
        let mut pack_file_contents_ui = PackFileContentsUI::new(app_ui.main_window);

        let app_temp_slots = Rc::new(RefCell::new(AppUITempSlots::new(app_ui, pack_file_contents_ui, global_search_ui, &slot_holder)));
        let app_slots = AppUISlots::new(app_ui, global_search_ui, pack_file_contents_ui, &app_temp_slots, &slot_holder);
        let pack_file_contents_slots = PackFileContentsSlots::new(app_ui, pack_file_contents_ui, global_search_ui, slot_holder);
        let global_search_slots = GlobalSearchSlots::new(app_ui, global_search_ui, pack_file_contents_ui, &slot_holder);

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&mut app_ui);
        app_ui::shortcuts::set_shortcuts(&mut app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&mut global_search_ui);
        global_search_ui::shortcuts::set_shortcuts(&mut global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&mut pack_file_contents_ui);
        packfile_contents_ui::shortcuts::set_shortcuts(&mut pack_file_contents_ui);

        // Here we also initialize the UI.
        app_ui.update_window_title(&pack_file_contents_ui);
        UI_STATE.set_operational_mode(&mut app_ui, None);

        let game_selected = GAME_SELECTED.read().unwrap().to_owned();
        match &*game_selected {
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

        // Show the Main Window...
        app_ui.main_window.show();

        // We get all the Arguments provided when starting RPFM, just in case we passed it a path,
        // in which case, we automatically try to open it.
        let args = args().collect::<Vec<String>>();
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.is_file() {
                if let Err(error) = app_ui.open_packfile(&mut pack_file_contents_ui, &mut global_search_ui, &[path], "", &slot_holder) {
                    show_dialog(app_ui.main_window, error, false);
                }
            }
        }

        // If we want the window to start maximized...
        if SETTINGS.lock().unwrap().settings_bool["start_maximized"] {
            app_ui.main_window.set_window_state(QFlags::from(WindowState::WindowMaximized));
        }

        // On Windows, we use the dark theme switch to control the Style, StyleSheet and Palette.
        if cfg!(target_os = "windows") {
            if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] {
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
            if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] {
                QApplication::set_palette_1a(ref_from_atomic(&*DARK_PALETTE));
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
            } else {
                QApplication::set_palette_1a(ref_from_atomic(&*LIGHT_PALETTE));
            }
        }

        // If we have it enabled in the prefs, check if there are updates.
        if SETTINGS.lock().unwrap().settings_bool["check_updates_on_start"] { app_ui.check_updates(false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if SETTINGS.lock().unwrap().settings_bool["check_schema_updates_on_start"] { app_ui.check_schema_updates(false) };

        // This is to get the new schemas. It's controlled by a global const. Don't enable this unless you know what you're doing.
        //if GENERATE_NEW_SCHEMA {

            // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
            // The second one should contain all the tables of the game, extracted directly from `data.pack`.
            //let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/test stuff/db_raw");
            //let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/test stuff/db_bin/");
            //match import_schema(Schema::load("schema_wh.json").ok(), &assembly_kit_schemas_path, &testing_tables_path) {
                //Ok(_) => show_dialog(app_ui.window, true, "Schema successfully created."),
                //Err(error) => show_dialog(app_ui.window, false, error),
            //}

            // Close the program with code 69
            //return 69
        //}


        (Self {
            app_ui,
            global_search_ui,
            pack_file_contents_ui
        },
        Slots {
            app_slots,
            app_temp_slots,
            global_search_slots,
            pack_file_contents_slots,
        })
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub unsafe fn new() -> Self {
        Self {
            three_kingdoms: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_THREE_KINGDOMS).unwrap().game_selected_icon)))),
            warhammer_2: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_WARHAMMER_2).unwrap().game_selected_icon)))),
            warhammer: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_WARHAMMER).unwrap().game_selected_icon)))),
            thrones_of_britannia: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_THRONES_OF_BRITANNIA).unwrap().game_selected_icon)))),
            attila: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_ATTILA).unwrap().game_selected_icon)))),
            rome_2: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_ROME_2).unwrap().game_selected_icon)))),
            shogun_2: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_SHOGUN_2).unwrap().game_selected_icon)))),
            napoleon: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_NAPOLEON).unwrap().game_selected_icon)))),
            empire: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_EMPIRE).unwrap().game_selected_icon)))),
            arena: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get(KEY_ARENA).unwrap().game_selected_icon)))),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub unsafe fn set_game_selected_icon(mut main_window: MutPtr<QMainWindow>) {
        let icon = match &**GAME_SELECTED.read().unwrap() {
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
        main_window.set_window_icon(ref_from_atomic(&*icon));
    }
}
