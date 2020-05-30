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
Module with all the code related to `SettingsUISlots`.
!*/

use qt_widgets::QFontDialog;

use qt_gui::QGuiApplication;
use qt_gui::QFontDatabase;
use qt_gui::q_font_database::SystemFont;

use qt_core::Slot;

use cpp_core::MutPtr;

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
    pub restore_default: Slot<'static>,
    pub select_mymod_path: Slot<'static>,
    pub select_game_paths: BTreeMap<String, Slot<'static>>,
    pub shortcuts: Slot<'static>,
    pub text_editor: Slot<'static>,
    pub font_settings: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUISlots`.
impl SettingsUISlots {

    /// This function creates a new `SettingsUISlots`.
    pub unsafe fn new(ui: &mut SettingsUI) -> Self {

        // What happens when we hit thr "Restore Default" button.
        let restore_default = Slot::new(clone!(mut ui => move || {
            ui.load(&Settings::new());
            QGuiApplication::set_font(&QFontDatabase::system_font(SystemFont::GeneralFont));
        }));

        // What happens when we hit the "..." button for MyMods.
        let select_mymod_path = Slot::new(clone!(
            ui => move || {
            ui.update_entry_path(None);
        }));

        // What happens when we hit any of the "..." buttons for the games.
        let mut select_game_paths = BTreeMap::new();
        for key in ui.paths_games_line_edits.keys() {
            select_game_paths.insert(
                key.to_owned(),
                Slot::new(clone!(
                    key,
                    ui => move || {
                    ui.update_entry_path(Some(&key));
                }))
            );
        }

        // What happens when we hit the "Shortcuts" button.
        let shortcuts = Slot::new(clone!(ui => move || {

            // Create the Shortcuts Dialog. If we got new shortcuts, try to save them and report any error.
            if let Some(shortcuts) = ShortcutsUI::new(ui.dialog) {
                CENTRAL_COMMAND.send_message_qt(Command::SetShortcuts(shortcuts.clone()));
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::Success => UI_STATE.set_shortcuts(&shortcuts),
                    Response::Error(error) => show_dialog(ui.dialog, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }));

        // What happens when we hit the "Text Editor Preferences" button.
        let text_editor = Slot::new(clone!(mut ui => move || {
            ffi::open_text_editor_config_safe(&mut ui.dialog);
        }));

        let font_settings = Slot::new(clone!(mut ui => move || {
            let font_changed: *mut bool = &mut false;
            let font_changed = MutPtr::from_raw(font_changed);
            let current_font = QGuiApplication::font();
            let new_font = QFontDialog::get_font_bool_q_font_q_widget(font_changed, current_font.as_ref(), ui.dialog);
            if *font_changed {
                QGuiApplication::set_font(new_font.as_ref());
            }
        }));

        // And here... we return all the slots.
		Self {
            restore_default,
            select_mymod_path,
            select_game_paths,
            shortcuts,
            text_editor,
            font_settings
		}
	}
}
