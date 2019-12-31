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
Module with all the code related to `SettingsUISlots`.
!*/

use qt_widgets::widget::Widget;

use qt_core::slots::SlotNoArgs;

use std::collections::BTreeMap;

use rpfm_lib::settings::Settings;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi;
use crate::settings_ui::SettingsUI;
use crate::shortcuts_ui::ShortcutsUI;
use crate::UI_STATE;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SettingsUI` struct.
///
/// This means everything you can do with the stuff you have in the `SettingsUI` goes here.
pub struct SettingsUISlots {
    pub restore_default: SlotNoArgs<'static>,
    pub select_mymod_path: SlotNoArgs<'static>,
    pub select_game_paths: BTreeMap<String, SlotNoArgs<'static>>,
    pub shortcuts: SlotNoArgs<'static>,
    pub text_editor: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUISlots`.
impl SettingsUISlots {

    /// This function creates a new `SettingsUISlots`.
    pub fn new(ui: &SettingsUI) -> Self {

        // What happens when we hit thr "Restore Default" button.
        let mut ui_default = ui.clone();
        let restore_default = SlotNoArgs::new(move || {
            ui_default.load(&Settings::new())
        });

        // What happens when we hit the "..." button for MyMods.
        let select_mymod_path = SlotNoArgs::new(clone!(
            ui => move || {
            ui.update_entry_path(None);
        }));

        // What happens when we hit any of the "..." buttons for the games.
        let mut select_game_paths = BTreeMap::new();
        for key in ui.paths_games_line_edits.keys() {
            select_game_paths.insert(
                key.to_owned(),
                SlotNoArgs::new(clone!(
                    key,
                    ui => move || {
                    ui.update_entry_path(Some(&key));
                }))
            );
        }

        // What happens when we hit the "Shortcuts" button.
        let shortcuts = SlotNoArgs::new(clone!(ui => move || {

            // Create the Shortcuts Dialog. If we got new shortcuts, try to save them and report any error.
            if let Some(shortcuts) = ShortcutsUI::new(ui.dialog as *mut Widget) {
                CENTRAL_COMMAND.send_message_qt(Command::SetShortcuts(shortcuts.clone()));
                match CENTRAL_COMMAND.recv_message_qt() {
                    Response::Success => UI_STATE.set_shortcuts(&shortcuts),
                    Response::Error(error) => show_dialog(ui.dialog as *mut Widget, error, false),
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                }
            }
        }));

        // What happens when we hit the "Text Editor Preferences" button.
        let text_editor = SlotNoArgs::new(clone!(ui => move || {
            unsafe { ffi::open_text_editor_config(ui.dialog as *mut Widget); }
        }));

        // And here... we return all the slots.
		Self {
            restore_default,
            select_mymod_path,
            select_game_paths,
            shortcuts,
            text_editor
		}
	}
}
