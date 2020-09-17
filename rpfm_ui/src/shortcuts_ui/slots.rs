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
Module with all the code related to `ShortcutsUISlots`.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;

use crate::shortcuts_ui::ShortcutsUI;
use crate::ui_state::shortcuts::Shortcuts;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `ShortcutsUI` struct.
///
/// This means everything you can do with the stuff you have in the `ShortcutsUI` goes here.
pub struct ShortcutsUISlots {
    pub restore_default: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ShortcutsUISlots`.
impl ShortcutsUISlots {

    /// This function creates a new `ShortcutsUISlots`.
    pub unsafe fn new(ui: &Rc<ShortcutsUI>) -> Self {

        // What happens when we hit the "Restore Default" action.
        let restore_default = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                ShortcutsUI::load(&ui, &Shortcuts::new());
            }
        ));

        ShortcutsUISlots {
            restore_default
        }
    }
}
