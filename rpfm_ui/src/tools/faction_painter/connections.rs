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
Module with all the code to connect `ToolFactionPainter` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ToolFactionPainter` and `ToolFactionPainterSlots` structs.
!*/

use super::{ToolFactionPainter, slots::ToolFactionPainterSlots};

/// This function connects all the actions from the provided `ToolFactionPainter` with their slots in `ToolFactionPainterSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ToolFactionPainter, slots: &ToolFactionPainterSlots) {
    //ui.restore_default_button.released().connect(&slots.restore_default);
    //ui.cancel_button.released().connect(ui.dialog.slot_close());
    //ui.accept_button.released().connect(ui.dialog.slot_accept());
}
