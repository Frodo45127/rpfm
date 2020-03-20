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
Module with all the code to connect `PackedFileTableView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileTableView` and `PackedFileTableViewSlots` structs.
!*/

use super::{PackedFileTableView, slots::PackedFileTableViewSlots};

/// This function connects all the actions from the provided `PackedFileTableView` with their slots in `PackedFileTableViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackedFileTableView, slots: &PackedFileTableViewSlots) {
    ui.get_mut_ptr_filter_line_edit().text_changed().connect(&slots.filter_line_edit);

    ui.get_mut_ptr_table_view_primary().custom_context_menu_requested().connect(&slots.show_context_menu);
    ui.get_mut_ptr_table_view_frozen().custom_context_menu_requested().connect(&slots.show_context_menu);

    ui.get_mut_ptr_table_model().item_changed().connect(&slots.item_changed);
    ui.get_mut_ptr_table_view_primary().selection_model().selection_changed().connect(&slots.context_menu_enabler);
    ui.get_mut_ptr_context_menu_add_rows().triggered().connect(&slots.add_rows);
    ui.get_mut_ptr_context_menu_insert_rows().triggered().connect(&slots.insert_rows);
    ui.get_mut_ptr_context_menu_delete_rows().triggered().connect(&slots.delete_rows);
    ui.get_mut_ptr_context_menu_clone_and_append().triggered().connect(&slots.clone_and_append);
    ui.get_mut_ptr_context_menu_clone_and_insert().triggered().connect(&slots.clone_and_insert);
    ui.get_mut_ptr_context_menu_copy().triggered().connect(&slots.copy);
    ui.get_mut_ptr_context_menu_copy_as_lua_table().triggered().connect(&slots.copy_as_lua_table);
    ui.get_mut_ptr_context_menu_paste().triggered().connect(&slots.paste);
    ui.get_mut_ptr_context_menu_invert_selection().triggered().connect(&slots.invert_selection);
    ui.get_mut_ptr_context_menu_reset_selection().triggered().connect(&slots.reset_selection);
    ui.get_mut_ptr_context_menu_undo().triggered().connect(&slots.undo);
    ui.get_mut_ptr_context_menu_redo().triggered().connect(&slots.redo);
}
