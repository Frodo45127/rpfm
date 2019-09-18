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
Module with all the code to connect `AppUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `AppUI` and `AppUISlots` structs.
!*/

use qt_core::connection::Signal;

use super::{AppUI, slots::AppUISlots};

/// This function connects all the actions from the provided `AppUI` with their slots in `AppUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub fn set_connections(app_ui: &AppUI, slots: &AppUISlots) {

	//-----------------------------------------------//
    // Command Palette connections.
    //-----------------------------------------------//
    unsafe { app_ui.command_palette_show.as_ref().unwrap().signals().triggered().connect(&slots.command_palette_show); }
    unsafe { app_ui.command_palette_hide.as_ref().unwrap().signals().triggered().connect(&slots.command_palette_hide); }

    unsafe { app_ui.command_palette_completer.as_ref().unwrap().signals().activated_qt_core_string_ref().connect(&slots.command_palette_trigger); }

    //-----------------------------------------------//
    // `PackFile` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.packfile_new_packfile.as_ref().unwrap().signals().triggered().connect(&slots.packfile_new_packfile); }    
    unsafe { app_ui.packfile_open_packfile.as_ref().unwrap().signals().triggered().connect(&slots.packfile_open_packfile); }    
    unsafe { app_ui.packfile_preferences.as_ref().unwrap().signals().triggered().connect(&slots.packfile_preferences); }    
    unsafe { app_ui.packfile_quit.as_ref().unwrap().signals().triggered().connect(&slots.packfile_quit); }    

    //-----------------------------------------------//
    // `View` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.view_toggle_packfile_contents.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_packfile_contents); }
    unsafe { app_ui.view_toggle_global_search_panel.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_global_search_panel); }
    
    //-----------------------------------------------//
    // `Game Selected` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.game_selected_three_kingdoms.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_warhammer_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_warhammer.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_thrones_of_britannia.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_attila.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_rome_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_shogun_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_napoleon.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_empire.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_arena.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }

    //-----------------------------------------------//
    // `About` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.about_about_qt.as_ref().unwrap().signals().triggered().connect(&slots.about_about_qt); }
    unsafe { app_ui.about_open_manual.as_ref().unwrap().signals().triggered().connect(&slots.about_open_manual); }
    unsafe { app_ui.about_patreon_link.as_ref().unwrap().signals().triggered().connect(&slots.about_patreon_link); }
}
