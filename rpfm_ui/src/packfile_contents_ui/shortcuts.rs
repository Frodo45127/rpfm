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
Module with all the code to setup shortcuts for `PackFileContentsUI`.

This module is, and should stay, private, as it's only here to not polute the `PackFileContentsUI` module.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;

use std::rc::Rc;

use super::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `PackFileContentsUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &Rc<PackFileContentsUI>) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the PackFile Contents TreeView's context menu actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    ui.context_menu_add_file.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["add_file"])));
    ui.context_menu_add_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["add_folder"])));
    ui.context_menu_add_from_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["add_from_packfile"])));
    ui.context_menu_new_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["create_folder"])));
    ui.context_menu_new_packed_file_db.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["create_db"])));
    ui.context_menu_new_packed_file_loc.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["create_loc"])));
    ui.context_menu_new_packed_file_text.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["create_text"])));
    ui.context_menu_new_queek_packed_file.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["create_queek"])));
    ui.context_menu_mass_import_tsv.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["mass_import_tsv"])));
    ui.context_menu_mass_export_tsv.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["mass_export_tsv"])));
    ui.context_menu_merge_tables.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["merge_tables"])));
    ui.context_menu_update_table.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["update_tables"])));
    ui.context_menu_delete.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["delete"])));
    ui.context_menu_extract.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["extract"])));
    ui.context_menu_rename.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["rename"])));
    ui.context_menu_copy_path.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["copy_path"])));
    ui.context_menu_open_decoder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_in_decoder"])));
    ui.context_menu_open_dependency_manager.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_packfiles_list"])));
    ui.context_menu_open_containing_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_containing_folder"])));
    ui.context_menu_open_packfile_settings.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_packfile_settings"])));
    ui.context_menu_open_with_external_program.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_with_external_program"])));
    ui.context_menu_open_notes.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["open_notes"])));
    ui.packfile_contents_tree_view_expand_all.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["expand_all"])));
    ui.packfile_contents_tree_view_collapse_all.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["collapse_all"])));

    // Set the shortcuts to only trigger in the TreeView.
    ui.context_menu_add_file.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_add_folder.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_add_from_packfile.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_new_folder.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_new_packed_file_db.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_new_packed_file_loc.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_new_packed_file_text.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_new_queek_packed_file.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_mass_import_tsv.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_mass_export_tsv.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_merge_tables.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_update_table.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_delete.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_extract.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_rename.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_copy_path.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_decoder.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_dependency_manager.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_containing_folder.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_packfile_settings.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_with_external_program.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_open_notes.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.packfile_contents_tree_view_expand_all.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.packfile_contents_tree_view_collapse_all.set_shortcut_context(ShortcutContext::WidgetShortcut);

    // Add the actions to the TreeView, so the shortcuts work.
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_add_file);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_add_folder);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_add_from_packfile);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_new_folder);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_new_packed_file_db);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_new_packed_file_loc);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_new_packed_file_text);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_new_queek_packed_file);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_mass_import_tsv);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_mass_export_tsv);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_merge_tables);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_update_table);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_delete);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_extract);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_rename);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_copy_path);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_decoder);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_dependency_manager);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_containing_folder);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_packfile_settings);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_with_external_program);
    ui.packfile_contents_tree_view.add_action(&ui.context_menu_open_notes);
    ui.packfile_contents_tree_view.add_action(ui.packfile_contents_tree_view_expand_all.as_ptr());
    ui.packfile_contents_tree_view.add_action(ui.packfile_contents_tree_view_collapse_all.as_ptr());

}
