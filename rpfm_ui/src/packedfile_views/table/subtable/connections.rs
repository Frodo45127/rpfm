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
Module with all the code to connect `SubTableView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `SubTableView` and `SubTableViewSlots` structs.
!*/

use super::{SubTableView, slots::SubTableViewSlots};

/// This function connects all the actions from the provided `SubTableView` with their slots in `SubTableViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &SubTableView, slots: &SubTableViewSlots) {
    ui.table_view.horizontal_header().sort_indicator_changed().connect(&slots.sort_order_column_changed);

    ui.table_view.custom_context_menu_requested().connect(&slots.show_context_menu);

    ui.table_view.selection_model().selection_changed().connect(&slots.context_menu_enabler);
    ui.context_menu_add_rows.triggered().connect(&slots.add_rows);
    ui.context_menu_delete_rows.triggered().connect(&slots.delete_rows);
    ui.context_menu_copy.triggered().connect(&slots.copy);

    ui.table_view.double_clicked().connect(&slots.open_subtable);
}
