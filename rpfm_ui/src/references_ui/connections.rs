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
Module with all the code to connect `ReferencesUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ReferencesUI` and `ReferencesUISlots` structs.
!*/

use super::{ReferencesUI, slots::ReferencesUISlots};

/// This function connects all the actions from the provided `ReferencesUI` with their slots in `ReferencesUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ReferencesUI, slots: &ReferencesUISlots) {
    ui.references_table_view.double_clicked().connect(&slots.references_open_result);
}
