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
Module with all the code to connect `TableView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `TableView` and `TableViewSlots` structs.
!*/

use std::sync::Arc;

use super::{FilterView, TableView, slots::{FilterViewSlots, TableViewSlots}};

/// This function connects all the actions from the provided `TableView` with their slots in `TableViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<TableView>, slots: &TableViewSlots) {
    ui.get_mut_ptr_table_view_primary().horizontal_header().sort_indicator_changed().connect(&slots.sort_order_column_changed);

    ui.get_mut_ptr_table_view_primary().custom_context_menu_requested().connect(&slots.show_context_menu);
    ui.get_mut_ptr_table_view_frozen().custom_context_menu_requested().connect(&slots.show_context_menu);

    ui.get_mut_ptr_table_model().item_changed().connect(&slots.item_changed);
    ui.get_mut_ptr_table_view_primary().selection_model().selection_changed().connect(&slots.context_menu_enabler);
    ui.get_mut_ptr_context_menu_add_rows().triggered().connect(&slots.add_rows);
    ui.get_mut_ptr_context_menu_insert_rows().triggered().connect(&slots.insert_rows);
    ui.get_mut_ptr_context_menu_delete_rows().triggered().connect(&slots.delete_rows);
    ui.get_mut_ptr_context_menu_delete_rows_not_in_filter().triggered().connect(&slots.delete_rows_not_in_filter);
    ui.get_mut_ptr_context_menu_clone_and_append().triggered().connect(&slots.clone_and_append);
    ui.get_mut_ptr_context_menu_clone_and_insert().triggered().connect(&slots.clone_and_insert);
    ui.get_mut_ptr_context_menu_copy().triggered().connect(&slots.copy);
    ui.get_mut_ptr_context_menu_copy_as_lua_table().triggered().connect(&slots.copy_as_lua_table);
    ui.get_mut_ptr_context_menu_paste().triggered().connect(&slots.paste);
    ui.get_mut_ptr_context_menu_paste_as_new_row().triggered().connect(&slots.paste_as_new_row);
    ui.get_mut_ptr_context_menu_invert_selection().triggered().connect(&slots.invert_selection);
    ui.get_mut_ptr_context_menu_reset_selection().triggered().connect(&slots.reset_selection);
    ui.get_mut_ptr_context_menu_rewrite_selection().triggered().connect(&slots.rewrite_selection);
    ui.get_mut_ptr_context_menu_generate_ids().triggered().connect(&slots.generate_ids);
    ui.get_mut_ptr_context_menu_undo().triggered().connect(&slots.undo);
    ui.get_mut_ptr_context_menu_redo().triggered().connect(&slots.redo);
    ui.get_mut_ptr_context_menu_import_tsv().triggered().connect(&slots.import_tsv);
    ui.get_mut_ptr_context_menu_export_tsv().triggered().connect(&slots.export_tsv);
    ui.get_mut_ptr_context_menu_resize_columns().triggered().connect(&slots.resize_columns);
    ui.get_mut_ptr_context_menu_sidebar().triggered().connect(&slots.sidebar);
    ui.get_mut_ptr_context_menu_search().triggered().connect(&slots.search);
    ui.get_mut_ptr_context_menu_cascade_edition().triggered().connect(&slots.cascade_edition);
    ui.get_mut_ptr_context_menu_patch_column().triggered().connect(&slots.patch_column);
    ui.get_mut_ptr_context_menu_go_to_definition().triggered().connect(&slots.go_to_definition);
    ui.get_mut_ptr_smart_delete().triggered().connect(&slots.smart_delete);

    ui.get_go_to_loc_actions().iter()
        .zip(slots.go_to_loc.iter())
        .for_each(|(x, y)| { x.triggered().connect(y); });

    ui.get_hide_show_checkboxes_all().state_changed().connect(&slots.hide_show_columns_all);
    ui.get_freeze_checkboxes_all().state_changed().connect(&slots.freeze_columns_all);

    ui.get_hide_show_checkboxes().iter()
        .zip(slots.hide_show_columns.iter())
        .for_each(|(x, y)| { x.state_changed().connect(y); });

    ui.get_freeze_checkboxes().iter()
        .zip(slots.freeze_columns.iter())
        .for_each(|(x, y)| { x.state_changed().connect(y); });

    ui.get_mut_ptr_search_search_button().released().connect(&slots.search_search);
    ui.get_mut_ptr_search_prev_match_button().released().connect(&slots.search_prev_match);
    ui.get_mut_ptr_search_next_match_button().released().connect(&slots.search_next_match);
    ui.get_mut_ptr_search_replace_current_button().released().connect(&slots.search_replace_current);
    ui.get_mut_ptr_search_replace_all_button().released().connect(&slots.search_replace_all);
    ui.get_mut_ptr_search_close_button().released().connect(&slots.search_close);
    ui.get_mut_ptr_search_search_line_edit().text_changed().connect(&slots.search_check_regex);

    ui.get_mut_ptr_table_view_primary().double_clicked().connect(&slots.open_subtable);

    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);
}

pub unsafe fn set_connections_filter(ui: &FilterView, slots: &FilterViewSlots) {
    ui.filter_line_edit.text_changed().connect(&slots.filter_line_edit);
    ui.filter_match_group_selector.current_index_changed().connect(&slots.filter_match_group_selector);
    ui.filter_column_selector.current_index_changed().connect(&slots.filter_column_selector);
    ui.filter_case_sensitive_button.toggled().connect(&slots.filter_case_sensitive_button);
    ui.filter_show_blank_cells_button.toggled().connect(&slots.filter_show_blank_cells_button);
    ui.filter_timer_delayed_updates.timeout().connect(&slots.filter_trigger);
    ui.filter_line_edit.text_changed().connect(&slots.filter_check_regex);
    ui.filter_add.released().connect(&slots.filter_add);
    ui.filter_remove.released().connect(&slots.filter_remove);
}
