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
Module with all the code related to `ToolFactionPainterSlots`.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `ShortcutsUI` struct.
///
/// This means everything you can do with the stuff you have in the `ShortcutsUI` goes here.
pub struct ToolFactionPainterSlots {
    pub restore_default: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ToolFactionPainterSlots`.
impl ToolFactionPainterSlots {

    /// This function creates a new `ToolFactionPainterSlots`.
    pub unsafe fn new(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, ui: &Rc<ToolFactionPainter>) -> Self {

        // What happens when we hit the "Restore Default" action.
        let restore_default = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                //ShortcutsUI::load(&ui, &Shortcuts::new());
            }
        ));

        ToolFactionPainterSlots {
            restore_default
        }
    }
}
