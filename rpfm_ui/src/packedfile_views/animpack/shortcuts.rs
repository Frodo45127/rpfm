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
Module with all the code to setup shortcuts for `PackedFileAnimPackView`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;

use std::sync::Arc;

use crate::QString;
use crate::UI_STATE;
use super::PackedFileAnimPackView;

/// This function setup all the shortcuts used by the actions in the provided `PackedFileAnimPackView` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(ui: &Arc<PackedFileAnimPackView>) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    ui.get_ref_pack_expand_all().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["expand_all"])));
    ui.get_ref_pack_collapse_all().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["collapse_all"])));
    ui.get_ref_anim_pack_expand_all().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["expand_all"])));
    ui.get_ref_anim_pack_collapse_all().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["collapse_all"])));
    ui.get_ref_anim_pack_delete().set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packfile_contents_tree_view["delete"])));

    ui.get_ref_pack_expand_all().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_ref_pack_collapse_all().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_ref_anim_pack_expand_all().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_ref_anim_pack_collapse_all().set_shortcut_context(ShortcutContext::WidgetShortcut);
    ui.get_ref_anim_pack_delete().set_shortcut_context(ShortcutContext::WidgetShortcut);

}
