//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use qt_gui::key_sequence::KeySequence;

use qt_core::qt::ShortcutContext;

use super::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `PackFileContentsUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub fn set_shortcuts(ui: &PackFileContentsUI) {

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the PackFile Contents TreeView's context menu actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    unsafe { ui.context_menu_add_file.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["add_file"]))); }
    unsafe { ui.context_menu_add_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["add_folder"]))); }
    unsafe { ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["add_from_packfile"]))); }
    unsafe { ui.context_menu_check_tables.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["check_tables"]))); }
    unsafe { ui.context_menu_create_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["create_folder"]))); }
    unsafe { ui.context_menu_create_db.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["create_db"]))); }
    unsafe { ui.context_menu_create_loc.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["create_loc"]))); }
    unsafe { ui.context_menu_create_text.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["create_text"]))); }
    unsafe { ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["mass_import_tsv"]))); }
    unsafe { ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["mass_export_tsv"]))); }
    unsafe { ui.context_menu_merge_tables.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["merge_tables"]))); }
    unsafe { ui.context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["delete"]))); }
    unsafe { ui.context_menu_extract.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["extract"]))); }
    unsafe { ui.context_menu_rename.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["rename"]))); }
    unsafe { ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_in_decoder"]))); }
    unsafe { ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_packfiles_list"]))); }
    unsafe { ui.context_menu_open_containing_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_containing_folder"]))); }
    unsafe { ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_with_external_program"]))); }
    unsafe { ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_in_multi_view"]))); }
    unsafe { ui.context_menu_open_notes.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["open_notes"]))); }
    unsafe { ui.context_menu_global_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["global_search"]))); }
    unsafe { ui.packfile_contents_tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["expand_all"]))); }
    unsafe { ui.packfile_contents_tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["collapse_all"]))); }

    // Set the shortcuts to only trigger in the TreeView.
    unsafe { ui.context_menu_add_file.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_add_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_check_tables.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_create_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_create_db.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_create_loc.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_create_text.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_merge_tables.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_extract.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_rename.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_containing_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_open_notes.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.context_menu_global_search.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.packfile_contents_tree_view_expand_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
    unsafe { ui.packfile_contents_tree_view_collapse_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

    // Add the actions to the TreeView, so the shortcuts work.
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_add_file); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_add_folder); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_add_from_packfile); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_check_tables); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_create_folder); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_create_db); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_create_loc); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_create_text); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_mass_import_tsv); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_mass_export_tsv); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_merge_tables); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_delete); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_extract); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_rename); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_decoder); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_dependency_manager); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_containing_folder); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_with_external_program); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_in_multi_view); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_open_notes); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.context_menu_global_search); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.packfile_contents_tree_view_expand_all); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().add_action(ui.packfile_contents_tree_view_collapse_all); }

}
