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
Module with all the code to connect `PackedFileAnimFragmentView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileAnimFragmentView` and `PackedFileAnimFragmentViewSlots` structs.
!*/

use qt_gui::QStandardItemModel;

use qt_core::QSortFilterProxyModel;

use cpp_core::MutPtr;

use super::{PackedFileAnimFragmentView, slots::PackedFileAnimFragmentViewSlots};

/// This function connects all the actions from the provided `PackedFileAnimFragmentView` with their slots in `PackedFileAnimFragmentViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackedFileAnimFragmentView, slots: &PackedFileAnimFragmentViewSlots) {
    let filter_1: MutPtr<QSortFilterProxyModel> = ui.get_mut_ptr_table_1().model().static_downcast_mut();
    let model_1: MutPtr<QStandardItemModel> = filter_1.source_model().static_downcast_mut();

    let filter_2: MutPtr<QSortFilterProxyModel> = ui.get_mut_ptr_table_2().model().static_downcast_mut();
    let model_2: MutPtr<QStandardItemModel> = filter_2.source_model().static_downcast_mut();

    ui.get_mut_ptr_table_1().horizontal_header().sort_indicator_changed().connect(&slots.sort_order_column_changed_1);
    ui.get_mut_ptr_table_2().horizontal_header().sort_indicator_changed().connect(&slots.sort_order_column_changed_2);

    ui.get_mut_ptr_table_1().custom_context_menu_requested().connect(&slots.show_context_menu_1);
    ui.get_mut_ptr_table_2().custom_context_menu_requested().connect(&slots.show_context_menu_2);

    ui.get_mut_ptr_table_1().selection_model().selection_changed().connect(&slots.context_menu_enabler_1);
    ui.get_mut_ptr_table_2().selection_model().selection_changed().connect(&slots.context_menu_enabler_2);

    model_1.item_changed().connect(&slots.item_changed_1);
    model_2.item_changed().connect(&slots.item_changed_2);
/*
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
    ui.get_mut_ptr_context_menu_rewrite_selection().triggered().connect(&slots.rewrite_selection);
    ui.get_mut_ptr_context_menu_undo().triggered().connect(&slots.undo);
    ui.get_mut_ptr_context_menu_redo().triggered().connect(&slots.redo);
    ui.get_mut_ptr_context_menu_resize_columns().triggered().connect(&slots.resize_columns);
    ui.get_mut_ptr_smart_delete().triggered().connect(&slots.smart_delete);*/
}
