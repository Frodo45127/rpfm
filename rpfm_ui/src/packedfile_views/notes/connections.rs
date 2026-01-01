//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `NotesView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `NotesView` and `NotesSlots` structs.
!*/

use std::sync::Arc;

use super::{NotesView, slots::NotesSlots};

/// This function connects all the actions from the provided `NotesView` with their slots in `NotesSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<NotesView>, slots: &NotesSlots) {
    ui.new_button.released().connect(slots.new_tip());

    ui.list.double_clicked().connect(slots.open_link());
    ui.list.custom_context_menu_requested().connect(slots.context_menu());
    ui.list.selection_model().selection_changed().connect(slots.context_menu_enabler());

    ui.context_menu_edit.triggered().connect(slots.edit_tip());
    ui.context_menu_delete.triggered().connect(slots.delete_tip());
}
