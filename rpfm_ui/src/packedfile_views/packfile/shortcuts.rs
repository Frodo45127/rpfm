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
Module with all the code to setup shortcuts for `PackFileExtraView`.
!*/

use qt_gui::key_sequence::KeySequence;

use qt_core::qt::ShortcutContext;

use super::PackFileExtraView;
use crate::QString;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `PackFileExtraView` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub fn set_shortcuts(ui: &PackFileExtraView) {
    ui.get_expand_all().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["expand_all"])));
    ui.get_collapse_all().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&UI_STATE.shortcuts.read().unwrap().tree_view["collapse_all"])));

    ui.get_expand_all().set_shortcut_context(ShortcutContext::Widget);
    ui.get_collapse_all().set_shortcut_context(ShortcutContext::Widget);
}
