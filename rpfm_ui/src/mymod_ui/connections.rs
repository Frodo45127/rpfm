//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `MyModUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `MyModUI` and `MyModSlots` structs.
!*/

use super::{MyModUI, slots::MyModUISlots};

/// This function connects all the actions from the provided `MyModUI` with their slots in `MyModSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &MyModUI, slots: &MyModUISlots) {
    ui.mymod_name_line_edit().text_changed().connect(&slots.mymod_name_change);
    ui.mymod_game_combobox().current_text_changed().connect(&slots.mymod_game_change);
    ui.mymod_cancel_button().released().connect(ui.mymod_dialog.slot_close());
    ui.mymod_accept_button().released().connect(ui.mymod_dialog.slot_accept());
}
