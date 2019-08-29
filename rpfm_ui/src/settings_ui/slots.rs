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

use qt_core::slots::SlotNoArgs;

use std::collections::BTreeMap;

use crate::settings_ui::SettingsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SettingsUI` struct.
///
/// This means everything you can do with the stuff you have in the `SettingsUI` goes here.
pub struct SettingsUISlots {
    pub select_mymod_path: SlotNoArgs<'static>,
    pub select_game_paths: BTreeMap<String, SlotNoArgs<'static>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUISlots`.
impl SettingsUISlots {

    /// This function creates a new `SettingsUISlots`.
    pub fn new(settings_ui: &SettingsUI) -> Self {

        // What happens when we hit the "..." button for MyMods.
        let select_mymod_path = SlotNoArgs::new(clone!(
            settings_ui => move || {
            settings_ui.update_entry_path(None);
        }));

        // What happens when we hit any of the "..." buttons for the games.
        let mut select_game_paths = BTreeMap::new();
        for key in settings_ui.paths_games_line_edits.keys() {
            select_game_paths.insert(
                key.to_owned(), 
                SlotNoArgs::new(clone!(
                    key,
                    settings_ui => move || {
                    settings_ui.update_entry_path(Some(&key));
                }))
            );
        }
/*
        // What happens when we hit the "Shortcuts" button.
        let slot_shortcuts = SlotNoArgs::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move || {

                // Create the Shortcuts Dialog. If we got new shortcuts...
                if let Some(shortcuts) = ShortcutsDialog::create_shortcuts_dialog(dialog) {

                    // Send the signal to save them.
                    sender_qt.send(Commands::SetShortcuts).unwrap();
                    sender_qt_data.send(Data::Shortcuts(shortcuts)).unwrap();

                    // If there was an error.
                    if let Data::Error(error) = check_message_validity_recv2(&receiver_qt) { 

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If there was and IO error while saving the shortcuts, report it.
                            ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound | ErrorKind::IOGeneric => show_dialog(app_ui.window, false, error.kind()),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }

                    }
                }
            }
        ));

        */
        // And here... we return all the slots.
		Self {
            select_mymod_path,
            select_game_paths,
		}
	}
}
