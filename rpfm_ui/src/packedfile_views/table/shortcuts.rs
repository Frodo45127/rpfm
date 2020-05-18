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
Module with all the code to setup shortcuts for `PackedFileTableView`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use super::PackedFileTableView;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `PackedFileTableView` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &mut PackedFileTableView) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    // Set the shortcuts for these actions.
    ui.get_mut_ptr_context_menu_add_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["add_row"])));
    ui.get_mut_ptr_context_menu_insert_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["insert_row"])));
    ui.get_mut_ptr_context_menu_delete_rows().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["delete_row"])));
    ui.get_mut_ptr_context_menu_clone_and_insert().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_insert_row"])));
    ui.get_mut_ptr_context_menu_clone_and_append().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["clone_and_append_row"])));
    ui.get_mut_ptr_context_menu_copy().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["copy"])));
    ui.get_mut_ptr_context_menu_copy_as_lua_table().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["copy_as_lua_table"])));
    ui.get_mut_ptr_context_menu_paste().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["paste"])));
    ui.get_mut_ptr_context_menu_rewrite_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["rewrite_selection"])));
    ui.get_mut_ptr_context_menu_invert_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["selection_invert"])));
    ui.get_mut_ptr_context_menu_reset_selection().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["revert_selection"])));
    ui.get_mut_ptr_context_menu_search().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["search"])));
    ui.get_mut_ptr_context_menu_sidebar().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["sidebar"])));
    ui.get_mut_ptr_context_menu_import_tsv().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["import_tsv"])));
    ui.get_mut_ptr_context_menu_export_tsv().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["export_tsv"])));
    ui.get_mut_ptr_smart_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["smart_delete"])));
    ui.get_mut_ptr_context_menu_undo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["undo"])));
    ui.get_mut_ptr_context_menu_redo().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["redo"])));

    // Set the shortcuts to only trigger in the Table.
    ui.get_mut_ptr_context_menu_add_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_insert_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_delete_rows().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_clone_and_insert().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_clone_and_append().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_copy().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_copy_as_lua_table().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_paste().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_rewrite_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_invert_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_reset_selection().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_search().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_sidebar().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_import_tsv().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_export_tsv().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_smart_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_undo().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_context_menu_redo().set_shortcut_context(ShortcutContext::WidgetShortcut);

    // Add the actions to the TableView, so the shortcuts work.
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_add_rows());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_insert_rows());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_delete_rows());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_clone_and_insert());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_clone_and_append());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_copy());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_copy_as_lua_table());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_paste());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_rewrite_selection());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_invert_selection());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_reset_selection());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_search());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_sidebar());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_import_tsv());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_export_tsv());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_smart_delete());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_undo());
    ui.get_mut_ptr_table_view_primary().add_action(ui.get_mut_ptr_context_menu_redo());
}
