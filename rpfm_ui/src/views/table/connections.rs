//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `TableView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `TableView` and `TableViewSlots` structs.
!*/

use std::sync::Arc;

use super::{TableView, slots::TableViewSlots};

/// This function connects all the actions from the provided `TableView` with their slots in `TableViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<TableView>, slots: &TableViewSlots) {
    ui.table_view_ptr().horizontal_header().sort_indicator_changed().connect(&slots.sort_order_column_changed);

    ui.table_view_ptr().custom_context_menu_requested().connect(&slots.show_context_menu);

    ui.table_model_ptr().item_changed().connect(&slots.item_changed);
    ui.table_view_ptr().selection_model().selection_changed().connect(&slots.context_menu_enabler);

    ui.context_menu_add_rows().triggered().connect(&slots.add_rows);
    ui.context_menu_insert_rows().triggered().connect(&slots.insert_rows);
    ui.context_menu_delete_rows().triggered().connect(&slots.delete_rows);
    ui.context_menu_delete_rows_not_in_filter().triggered().connect(&slots.delete_rows_not_in_filter);
    ui.context_menu_clone_and_append().triggered().connect(&slots.clone_and_append);
    ui.context_menu_clone_and_insert().triggered().connect(&slots.clone_and_insert);
    ui.context_menu_copy().triggered().connect(&slots.copy);
    ui.context_menu_copy_as_lua_table().triggered().connect(&slots.copy_as_lua_table);
    ui.context_menu_copy_to_filter_value().triggered().connect(&slots.copy_to_filter_value);
    ui.context_menu_paste().triggered().connect(&slots.paste);
    ui.context_menu_paste_as_new_row().triggered().connect(&slots.paste_as_new_row);
    ui.context_menu_invert_selection().triggered().connect(&slots.invert_selection);
    ui.context_menu_reset_selection().triggered().connect(&slots.reset_selection);
    ui.context_menu_rewrite_selection().triggered().connect(&slots.rewrite_selection);
    ui.context_menu_generate_ids().triggered().connect(&slots.generate_ids);
    ui.context_menu_undo().triggered().connect(&slots.undo);
    ui.context_menu_redo().triggered().connect(&slots.redo);
    ui.context_menu_import_tsv().triggered().connect(&slots.import_tsv);
    ui.context_menu_export_tsv().triggered().connect(&slots.export_tsv);
    ui.context_menu_resize_columns().triggered().connect(&slots.resize_columns);
    ui.context_menu_sidebar().triggered().connect(&slots.sidebar);
    ui.context_menu_search().triggered().connect(&slots.search);
    ui.context_menu_cascade_edition().triggered().connect(&slots.cascade_edition);
    ui.context_menu_find_references().triggered().connect(&slots.find_references);
    ui.context_menu_patch_column().triggered().connect(&slots.patch_column);
    ui.context_menu_go_to_definition().triggered().connect(&slots.go_to_definition);
    ui.context_menu_smart_delete().triggered().connect(&slots.smart_delete);

    ui.context_menu_go_to_loc().iter()
        .zip(slots.go_to_loc.iter())
        .for_each(|(x, y)| { x.triggered().connect(y); });

    ui.sidebar_hide_checkboxes_all().state_changed().connect(&slots.hide_show_columns_all);
    ui.sidebar_freeze_checkboxes_all().state_changed().connect(&slots.freeze_columns_all);

    ui.sidebar_hide_checkboxes().iter()
        .zip(slots.hide_show_columns.iter())
        .for_each(|(x, y)| { x.state_changed().connect(y); });

    ui.sidebar_freeze_checkboxes().iter()
        .zip(slots.freeze_columns.iter())
        .for_each(|(x, y)| { x.state_changed().connect(y); });

    ui.table_view_ptr().double_clicked().connect(&slots.open_subtable);

    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);
}
