//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `MyModUISlots`.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;

use rpfm_ui_common::clone;

use crate::mymod_ui::MyModUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the New MyMod Dialog.
pub struct MyModUISlots {
    pub mymod_update_dialog: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `MyModUISlots`.
impl MyModUISlots {

    /// This function creates an entire `MyModUISlots` struct.
    pub unsafe fn new(mymod_ui: &Rc<MyModUI>) -> Self {
        let mymod_update_dialog = SlotNoArgs::new(&mymod_ui.dialog, clone!(
            mymod_ui => move || {
            mymod_ui.update_dialog();
        }));

        Self {
            mymod_update_dialog,
        }
    }
}
