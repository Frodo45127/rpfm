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
Module with all the code to setup shortcuts for `PackedFileDecoderView`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use std::sync::Arc;

use super::PackedFileDecoderView;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `PackedFileDecoderView` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &Arc<PackedFileDecoderView>) {
    /*
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    ui.get_mut_ptr_table_view_context_menu_move_up().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["move_up"])));
    ui.get_mut_ptr_table_view_context_menu_move_down().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["move_down"])));
    ui.get_mut_ptr_table_view_context_menu_move_left().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["move_left"])));
    ui.get_mut_ptr_table_view_context_menu_move_rigth().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["move_right"])));
    ui.get_mut_ptr_table_view_context_menu_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["delete"])));
    ui.get_mut_ptr_table_view_old_versions_context_menu_load().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["load"])));
    ui.get_mut_ptr_table_view_old_versions_context_menu_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_decoder["delete"])));

    ui.get_mut_ptr_table_view_context_menu_move_up().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_context_menu_move_down().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_context_menu_move_left().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_context_menu_move_rigth().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_context_menu_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_old_versions_context_menu_load().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_mut_ptr_table_view_old_versions_context_menu_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);

    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_context_menu_move_up().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_context_menu_move_down().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_context_menu_move_left().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_context_menu_move_rigth().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_context_menu_delete().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_old_versions_context_menu_load().as_ptr());
    ui.get_mut_ptr_table_view().add_action(ui.get_mut_ptr_table_view_old_versions_context_menu_delete().as_ptr());

    */
}
