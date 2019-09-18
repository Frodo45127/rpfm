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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::main_window::MainWindow;

use qt_gui::icon::Icon;

use rpfm_lib::GAME_SELECTED;
use rpfm_lib::SUPPORTED_GAMES;

use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::AppUISlots;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;
use crate::QString;

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
    pub pack_file_contents_slots: PackFileContentsSlots,
    pub global_search_slots: GlobalSearchSlots,
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
    pub three_kingdoms: Icon,
    pub warhammer_2: Icon,
    pub warhammer: Icon,
    pub thrones_of_britannia: Icon,
    pub attila: Icon,
    pub rome_2: Icon,
    pub shogun_2: Icon,
    pub napoleon: Icon,
    pub empire: Icon,
    pub arena: Icon,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `UI`.
impl UI {

    /// This function initialize the entire `UI`.
    pub fn new() -> (Self, Slots) {
        let app_ui = AppUI::default();
        let global_search_ui = GlobalSearchUI::new(app_ui.main_window);
        let pack_file_contents_ui = PackFileContentsUI::new(app_ui.main_window);
        
        let app_slots = AppUISlots::new(app_ui, global_search_ui, pack_file_contents_ui);
        let global_search_slots = GlobalSearchSlots::new(global_search_ui);
        let pack_file_contents_slots = PackFileContentsSlots::new(pack_file_contents_ui);

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&app_ui);
        app_ui::shortcuts::set_shortcuts(&app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&global_search_ui);
        global_search_ui::shortcuts::set_shortcuts(&global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&pack_file_contents_ui);
        packfile_contents_ui::shortcuts::set_shortcuts(&pack_file_contents_ui);
        
        (Self {
            app_ui,
            global_search_ui,
            pack_file_contents_ui
        },
        Slots {
            app_slots,
            global_search_slots,
            pack_file_contents_slots
        })
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub fn new() -> Self {
        Self {
            three_kingdoms: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("three_kingdoms").unwrap().game_selected_icon))),
            warhammer_2: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("warhammer_2").unwrap().game_selected_icon))),
            warhammer: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("warhammer").unwrap().game_selected_icon))),
            thrones_of_britannia: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("thrones_of_britannia").unwrap().game_selected_icon))),
            attila: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("attila").unwrap().game_selected_icon))),
            rome_2: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("rome_2").unwrap().game_selected_icon))),
            shogun_2: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("shogun_2").unwrap().game_selected_icon))),
            napoleon: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("napoleon").unwrap().game_selected_icon))),
            empire: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("empire").unwrap().game_selected_icon))),
            arena: Icon::new(&QString::from_std_str(format!("img/{}", SUPPORTED_GAMES.get("arena").unwrap().game_selected_icon))),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub fn set_game_selected_icon(main_window: *mut MainWindow) {
        let main_window = unsafe { main_window.as_mut().unwrap() };
        let icon = match &**GAME_SELECTED.lock().unwrap() {
            "three_kingdoms" => &GAME_SELECTED_ICONS.three_kingdoms,
            "warhammer_2" => &GAME_SELECTED_ICONS.warhammer_2,
            "warhammer" => &GAME_SELECTED_ICONS.warhammer,
            "thrones_of_britannia" => &GAME_SELECTED_ICONS.thrones_of_britannia,
            "attila" => &GAME_SELECTED_ICONS.attila,
            "rome_2" => &GAME_SELECTED_ICONS.rome_2,
            "shogun_2" => &GAME_SELECTED_ICONS.shogun_2,
            "napoleon" => &GAME_SELECTED_ICONS.napoleon,
            "empire" => &GAME_SELECTED_ICONS.empire,
            "arena" => &GAME_SELECTED_ICONS.arena,
            _ => unimplemented!(),
        };
        main_window.set_window_icon(icon);
    }
}