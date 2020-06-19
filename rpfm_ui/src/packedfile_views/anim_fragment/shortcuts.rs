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
Module with all the code to setup shortcuts for `PackedFileAnimFragmentView`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use super::PackedFileAnimFragmentView;
use crate::UI_STATE;

/// Macro to generate all the shortcuts at once.
macro_rules! shortcut_generator {
    (
        $ui:ident,

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
    ) => {

        let shortcuts = UI_STATE.get_shortcuts_no_lock();

        // Set the shortcuts for these actions.
        $ui.$get_mut_ptr_context_menu_add_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["add_row"])));
        $ui.$get_mut_ptr_context_menu_insert_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["insert_row"])));
        $ui.$get_mut_ptr_context_menu_delete_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["delete_row"])));
        $ui.$get_mut_ptr_context_menu_clone_and_insert().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_insert_row"])));
        $ui.$get_mut_ptr_context_menu_clone_and_append().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_append_row"])));
        $ui.$get_mut_ptr_context_menu_copy().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["copy"])));
        $ui.$get_mut_ptr_context_menu_paste().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["paste"])));
        $ui.$get_mut_ptr_context_menu_rewrite_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["rewrite_selection"])));
        $ui.$get_mut_ptr_context_menu_invert_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["selection_invert"])));
        $ui.$get_mut_ptr_context_menu_reset_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["revert_selection"])));
        $ui.$get_mut_ptr_context_menu_resize_columns().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["resize_columns"])));
        $ui.$get_mut_ptr_smart_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["smart_delete"])));
        $ui.$get_mut_ptr_context_menu_undo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["undo"])));
        $ui.$get_mut_ptr_context_menu_redo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["redo"])));

        // Set the shortcuts to only trigger in the Table.
        $ui.$get_mut_ptr_context_menu_add_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_insert_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_delete_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_clone_and_insert().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_clone_and_append().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_copy().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_paste().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_rewrite_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_invert_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_reset_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_resize_columns().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_smart_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_undo().set_shortcut_context(ShortcutContext::WidgetShortcut);
        $ui.$get_mut_ptr_context_menu_redo().set_shortcut_context(ShortcutContext::WidgetShortcut);

        // Add the actions to the TableView, so the shortcuts work.
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_add_rows());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_insert_rows());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_delete_rows());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_clone_and_insert());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_clone_and_append());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_copy());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_paste());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_rewrite_selection());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_invert_selection());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_reset_selection());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_resize_columns());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_smart_delete());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_undo());
        $ui.$get_mut_ptr_table().add_action($ui.$get_mut_ptr_context_menu_redo());
    }
}

/// This function setup all the shortcuts used by the actions in the provided `PackedFileAnimFragmentView` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &mut PackedFileAnimFragmentView) {
     shortcut_generator!(
        ui,

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
    );

    shortcut_generator!(
        ui,

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
    );
}
