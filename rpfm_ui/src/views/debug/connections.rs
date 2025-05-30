//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `DebugView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `DebugView` and `DebugViewSlots` structs.
!*/

use std::sync::Arc;

use super::DebugView;
use super::slots::DebugViewSlots;

/// This function connects all the actions from the provided `DebugView` with their slots in `DebugViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<DebugView>, slots: &DebugViewSlots) {
    ui.save_button.released().connect(&slots.save);

}
