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

/// Macro to generate all the connections at once.
macro_rules! connection_generator {
    (
        $ui:ident,
        $slots:ident,

        $get_mut_ptr_table:ident,
        $get_mut_ptr_context_menu_add_rows:ident,
        $get_mut_ptr_context_menu_insert_rows:ident,
        $get_mut_ptr_context_menu_delete_rows:ident,
        $get_mut_ptr_context_menu_clone_and_append:ident,
        $get_mut_ptr_context_menu_clone_and_insert:ident,
        $get_mut_ptr_context_menu_copy:ident,
        $get_mut_ptr_context_menu_paste:ident,
        $get_mut_ptr_context_menu_invert_selection:ident,
        $get_mut_ptr_context_menu_reset_selection:ident,
        $get_mut_ptr_context_menu_rewrite_selection:ident,
        $get_mut_ptr_context_menu_undo:ident,
        $get_mut_ptr_context_menu_redo:ident,
        $get_mut_ptr_context_menu_resize_columns:ident,
        $get_mut_ptr_smart_delete:ident,

        $item_changed:ident,
        $sort_order_column_changed:ident,
        $show_context_menu:ident,
        $context_menu_enabler:ident,
        $add_rows:ident,
        $insert_rows:ident,
        $delete_rows:ident,
        $clone_and_append:ident,
        $clone_and_insert:ident,
        $copy:ident,
        $paste:ident,
        $invert_selection:ident,
        $reset_selection:ident,
        $rewrite_selection:ident,
        $undo:ident,
        $redo:ident,
        $resize_columns:ident,
        $smart_delete:ident,
    ) => {
        let filter: MutPtr<QSortFilterProxyModel> = $ui.$get_mut_ptr_table().model().static_downcast_mut();
        let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();

        $ui.$get_mut_ptr_table().horizontal_header().sort_indicator_changed().connect(&$slots.$sort_order_column_changed);
        model.item_changed().connect(&$slots.$item_changed);

        $ui.$get_mut_ptr_table().custom_context_menu_requested().connect(&$slots.$show_context_menu);
        $ui.$get_mut_ptr_table().selection_model().selection_changed().connect(&$slots.$context_menu_enabler);
        $ui.$get_mut_ptr_context_menu_add_rows().triggered().connect(&$slots.$add_rows);
        $ui.$get_mut_ptr_context_menu_insert_rows().triggered().connect(&$slots.$insert_rows);
        $ui.$get_mut_ptr_context_menu_delete_rows().triggered().connect(&$slots.$delete_rows);
        $ui.$get_mut_ptr_context_menu_clone_and_append().triggered().connect(&$slots.$clone_and_append);
        $ui.$get_mut_ptr_context_menu_clone_and_insert().triggered().connect(&$slots.$clone_and_insert);
        $ui.$get_mut_ptr_context_menu_copy().triggered().connect(&$slots.$copy);
        $ui.$get_mut_ptr_context_menu_paste().triggered().connect(&$slots.$paste);
        $ui.$get_mut_ptr_context_menu_invert_selection().triggered().connect(&$slots.$invert_selection);
        $ui.$get_mut_ptr_context_menu_reset_selection().triggered().connect(&$slots.$reset_selection);
        $ui.$get_mut_ptr_context_menu_rewrite_selection().triggered().connect(&$slots.$rewrite_selection);
        $ui.$get_mut_ptr_context_menu_undo().triggered().connect(&$slots.$undo);
        $ui.$get_mut_ptr_context_menu_redo().triggered().connect(&$slots.$redo);
        $ui.$get_mut_ptr_context_menu_resize_columns().triggered().connect(&$slots.$resize_columns);
        $ui.$get_mut_ptr_smart_delete().triggered().connect(&$slots.$smart_delete);
    }
}

/// This function connects all the actions from the provided `PackedFileAnimFragmentView` with their slots in `PackedFileAnimFragmentViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackedFileAnimFragmentView, slots: &PackedFileAnimFragmentViewSlots) {
    connection_generator!(
        ui,
        slots,

        get_mut_ptr_table_1,
        get_mut_ptr_context_menu_add_rows_1,
        get_mut_ptr_context_menu_insert_rows_1,
        get_mut_ptr_context_menu_delete_rows_1,
        get_mut_ptr_context_menu_clone_and_append_1,
        get_mut_ptr_context_menu_clone_and_insert_1,
        get_mut_ptr_context_menu_copy_1,
        get_mut_ptr_context_menu_paste_1,
        get_mut_ptr_context_menu_invert_selection_1,
        get_mut_ptr_context_menu_reset_selection_1,
        get_mut_ptr_context_menu_rewrite_selection_1,
        get_mut_ptr_context_menu_undo_1,
        get_mut_ptr_context_menu_redo_1,
        get_mut_ptr_context_menu_resize_columns_1,
        get_mut_ptr_smart_delete_1,

        item_changed_1,
        sort_order_column_changed_1,
        show_context_menu_1,
        context_menu_enabler_1,
        add_rows_1,
        insert_rows_1,
        delete_rows_1,
        clone_and_append_1,
        clone_and_insert_1,
        copy_1,
        paste_1,
        invert_selection_1,
        reset_selection_1,
        rewrite_selection_1,
        undo_1,
        redo_1,
        resize_columns_1,
        smart_delete_1,
    );

    connection_generator!(
        ui,
        slots,

        get_mut_ptr_table_2,
        get_mut_ptr_context_menu_add_rows_2,
        get_mut_ptr_context_menu_insert_rows_2,
        get_mut_ptr_context_menu_delete_rows_2,
        get_mut_ptr_context_menu_clone_and_append_2,
        get_mut_ptr_context_menu_clone_and_insert_2,
        get_mut_ptr_context_menu_copy_2,
        get_mut_ptr_context_menu_paste_2,
        get_mut_ptr_context_menu_invert_selection_2,
        get_mut_ptr_context_menu_reset_selection_2,
        get_mut_ptr_context_menu_rewrite_selection_2,
        get_mut_ptr_context_menu_undo_2,
        get_mut_ptr_context_menu_redo_2,
        get_mut_ptr_context_menu_resize_columns_2,
        get_mut_ptr_smart_delete_2,

        item_changed_2,
        sort_order_column_changed_2,
        show_context_menu_2,
        context_menu_enabler_2,
        add_rows_2,
        insert_rows_2,
        delete_rows_2,
        clone_and_append_2,
        clone_and_insert_2,
        copy_2,
        paste_2,
        invert_selection_2,
        reset_selection_2,
        rewrite_selection_2,
        undo_2,
        redo_2,
        resize_columns_2,
        smart_delete_2,
    );
}
