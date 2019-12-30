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
Module with all the code to connect `ShortcutsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ShortcutsUI` and `ShortcutsUISlots` structs.
!*/

use qt_core::connection::Signal;

use super::{ShortcutsUI, slots::ShortcutsUISlots};

/// This function connects all the actions from the provided `ShortcutsUI` with their slots in `ShortcutsUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub fn set_connections(ui: &ShortcutsUI, slots: &ShortcutsUISlots) {
    unsafe { ui.restore_default_button.as_mut().unwrap().signals().released().connect(&slots.restore_default); }
    unsafe { ui.cancel_button.as_mut().unwrap().signals().released().connect(&ui.dialog.as_mut().unwrap().slots().close()); }
    unsafe { ui.accept_button.as_mut().unwrap().signals().released().connect(&ui.dialog.as_mut().unwrap().slots().accept()); }
}
