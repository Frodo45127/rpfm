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
Module with all the code to setup shortcuts for a table view.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use std::sync::Arc;

use super::TableView;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `TableViewSlots`.
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &Arc<TableView>) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    // Set the shortcuts for these actions.
    ui.context_menu_add_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["add_row"])));
    ui.context_menu_insert_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["insert_row"])));
    ui.context_menu_delete_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["delete_row"])));
    ui.context_menu_delete_rows_not_in_filter().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["delete_filtered_out_rows"])));
    ui.context_menu_clone_and_insert().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_insert_row"])));
    ui.context_menu_clone_and_append().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_append_row"])));
    ui.context_menu_copy().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["copy"])));
    ui.context_menu_copy_as_lua_table().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["copy_as_lua_table"])));
    ui.context_menu_paste().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["paste"])));
    ui.context_menu_paste_as_new_row().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["paste_as_new_row"])));
    ui.context_menu_generate_ids().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["generate_ids"])));
    ui.context_menu_rewrite_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["rewrite_selection"])));
    ui.context_menu_invert_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["selection_invert"])));
    ui.context_menu_reset_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["revert_selection"])));
    ui.context_menu_resize_columns().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["resize_columns"])));
    ui.context_menu_search().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["search"])));
    ui.context_menu_sidebar().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["sidebar"])));
    ui.context_menu_import_tsv().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["import_tsv"])));
    ui.context_menu_export_tsv().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["export_tsv"])));
    ui.context_menu_smart_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["smart_delete"])));
    ui.context_menu_undo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["undo"])));
    ui.context_menu_redo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["redo"])));
    ui.context_menu_cascade_edition().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["rename_references"])));
    ui.context_menu_go_to_definition().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["go_to_definition"])));

    // Set the shortcuts to only trigger in the Table.
    ui.context_menu_add_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_insert_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_delete_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_delete_rows_not_in_filter().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_clone_and_insert().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_clone_and_append().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_copy().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_copy_as_lua_table().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_paste().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_paste_as_new_row().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_generate_ids().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_rewrite_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_invert_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_reset_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_search().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_sidebar().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_import_tsv().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_export_tsv().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_resize_columns().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_smart_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_undo().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_redo().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_cascade_edition().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_go_to_definition().set_shortcut_context(ShortcutContext::WidgetShortcut);

    // Add the actions to the TableView, so the shortcuts work.
    ui.table_view_primary_ptr().add_action(ui.context_menu_add_rows());
    ui.table_view_primary_ptr().add_action(ui.context_menu_insert_rows());
    ui.table_view_primary_ptr().add_action(ui.context_menu_delete_rows());
    ui.table_view_primary_ptr().add_action(ui.context_menu_delete_rows_not_in_filter());
    ui.table_view_primary_ptr().add_action(ui.context_menu_clone_and_insert());
    ui.table_view_primary_ptr().add_action(ui.context_menu_clone_and_append());
    ui.table_view_primary_ptr().add_action(ui.context_menu_copy());
    ui.table_view_primary_ptr().add_action(ui.context_menu_copy_as_lua_table());
    ui.table_view_primary_ptr().add_action(ui.context_menu_paste());
    ui.table_view_primary_ptr().add_action(ui.context_menu_paste_as_new_row());
    ui.table_view_primary_ptr().add_action(ui.context_menu_generate_ids());
    ui.table_view_primary_ptr().add_action(ui.context_menu_rewrite_selection());
    ui.table_view_primary_ptr().add_action(ui.context_menu_invert_selection());
    ui.table_view_primary_ptr().add_action(ui.context_menu_reset_selection());
    ui.table_view_primary_ptr().add_action(ui.context_menu_resize_columns());
    ui.table_view_primary_ptr().add_action(ui.context_menu_search());
    ui.table_view_primary_ptr().add_action(ui.context_menu_sidebar());
    ui.table_view_primary_ptr().add_action(ui.context_menu_import_tsv());
    ui.table_view_primary_ptr().add_action(ui.context_menu_export_tsv());
    ui.table_view_primary_ptr().add_action(ui.context_menu_smart_delete());
    ui.table_view_primary_ptr().add_action(ui.context_menu_undo());
    ui.table_view_primary_ptr().add_action(ui.context_menu_redo());
    ui.table_view_primary_ptr().add_action(ui.context_menu_cascade_edition());
    ui.table_view_primary_ptr().add_action(ui.context_menu_go_to_definition());
}
