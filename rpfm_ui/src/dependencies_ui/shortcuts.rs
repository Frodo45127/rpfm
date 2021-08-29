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
Module with all the code to setup shortcuts for `DependenciesUI`.

This module is, and should stay, private, as it's only here to not polute the `DependenciesUI` module.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;

use std::rc::Rc;

use super::DependenciesUI;
use crate::QString;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `DependenciesUI`.
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &Rc<DependenciesUI>) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Dependencies TreeView's context menu actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    ui.context_menu_import.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["import_from_dependencies"])));
    ui.context_menu_copy_path.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["copy_path"])));

    ui.dependencies_tree_view_expand_all.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["expand_all"])));
    ui.dependencies_tree_view_collapse_all.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["collapse_all"])));

    // Set the shortcuts to only trigger in the TreeView.
    ui.context_menu_import.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.context_menu_copy_path.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.dependencies_tree_view_expand_all.set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.dependencies_tree_view_collapse_all.set_shortcut_context(ShortcutContext::WidgetShortcut);

    // Add the actions to the TreeView, so the shortcuts work.
    ui.dependencies_tree_view.add_action(&ui.context_menu_import);
    ui.dependencies_tree_view.add_action(&ui.context_menu_copy_path);
    ui.dependencies_tree_view.add_action(ui.dependencies_tree_view_expand_all.as_ptr());
    ui.dependencies_tree_view.add_action(ui.dependencies_tree_view_collapse_all.as_ptr());

}
