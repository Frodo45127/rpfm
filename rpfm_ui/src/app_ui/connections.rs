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
    // `View` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.view_toggle_packfile_contents.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_packfile_contents); }
    unsafe { app_ui.view_toggle_global_search_panel.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_global_search_panel); }
    
    //-----------------------------------------------//
    // `About` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.about_about_qt.as_ref().unwrap().signals().triggered().connect(&slots.about_about_qt); }
    unsafe { app_ui.about_open_manual.as_ref().unwrap().signals().triggered().connect(&slots.about_open_manual); }
    unsafe { app_ui.about_patreon_link.as_ref().unwrap().signals().triggered().connect(&slots.about_patreon_link); }

    //--------------------------------------------------------//
    // PackFile Contents TreeView's context menu connections.
    //--------------------------------------------------------//
    unsafe { app_ui.packfile_contents_tree_view_expand_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_expand_all); }
    unsafe { app_ui.packfile_contents_tree_view_collapse_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_collapse_all); }
}
