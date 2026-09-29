//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Settings backup operations, used by the settings dialog to undo a "Restore Defaults".

use crate::settings::{Settings, SETTINGS, SETTINGS_CHANGED};

use super::SessionState;

impl SessionState {

    /// Keeps a copy of the settings, to restore them later.
    pub fn backup_settings(&mut self, settings: Settings) {
        self.backup_settings = settings;
    }

    /// Replaces the settings with the last backup, notifying every connected client.
    pub fn restore_backup_settings(&self) {
        let snapshot = self.backup_settings.snapshot();
        *SETTINGS.write().unwrap() = self.backup_settings.clone();
        let _ = SETTINGS_CHANGED.send(snapshot);
    }
}
