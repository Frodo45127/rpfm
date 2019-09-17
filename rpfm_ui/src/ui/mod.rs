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

use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::AppUISlots;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;

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