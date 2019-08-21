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
Module with all the code to setup shortcuts for `AppUI`.

This module is, and should stay, private, as it's only here to not polute the `AppUI` module.
!*/

use qt_gui::key_sequence::KeySequence;

use qt_core::qt::ShortcutContext;

use super::AppUI;
use crate::QString;

/// This function setup all the shortcuts used by the actions in the provided `AppUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub fn shortcuts(app_ui: AppUI) {
    unsafe { app_ui.command_palette_show.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("Ctrl+Shift+P"))); }
    unsafe { app_ui.command_palette_hide.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("Esc"))); }

    unsafe { app_ui.command_palette_show.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.command_palette_hide.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    
    unsafe { app_ui.main_window.as_mut().unwrap().add_action(app_ui.command_palette_show); }
    unsafe { app_ui.main_window.as_mut().unwrap().add_action(app_ui.command_palette_hide); }
}
