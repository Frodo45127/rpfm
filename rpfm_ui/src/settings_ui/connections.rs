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
Module with all the code to connect `SettingsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `SettingsUI` and `SettingsUISlots` structs.
!*/

use qt_core::connection::Signal;

use super::{SettingsUI, slots::SettingsUISlots};

/// This function connects all the actions from the provided `SettingsUI` with their slots in `SettingsUIlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub fn set_connections(settings_ui: &SettingsUI, slots: &SettingsUISlots) {
    unsafe { settings_ui.paths_mymod_button.as_mut().unwrap().signals().released().connect(&slots.select_mymod_path); }

    for (key, button) in settings_ui.paths_games_buttons.iter() {
        unsafe { button.as_mut().unwrap().signals().released().connect(&slots.select_game_paths[key]); }
    }

    unsafe { settings_ui.button_box_shortcuts_button.as_mut().unwrap().signals().released().connect(&slots.shortcuts); }
    unsafe { settings_ui.button_box_restore_default_button.as_mut().unwrap().signals().released().connect(&slots.restore_default); }
    unsafe { settings_ui.button_box_text_editor_settings_button.as_mut().unwrap().signals().released().connect(&slots.text_editor); }
    unsafe { settings_ui.button_box_accept_button.as_mut().unwrap().signals().released().connect(&settings_ui.dialog.as_mut().unwrap().slots().accept()); }
    unsafe { settings_ui.button_box_cancel_button.as_mut().unwrap().signals().released().connect(&settings_ui.dialog.as_mut().unwrap().slots().close()); }
}
