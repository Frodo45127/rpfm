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
Module with the slots for Image Views.
!*/

use qt_widgets::widget::Widget;

use qt_core::slots::SlotCInt;

use std::sync::atomic::AtomicPtr;

use crate::packedfile_views::TheOneSlot;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a PackedFile.
pub struct PackedFileImageViewSlots {
    hide: AtomicPtr<SlotCInt<'static>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileImageViewSlots`.
impl PackedFileImageViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(packed_file_view_widget: *mut Widget) -> Self{

        let hide: *mut SlotCInt<'static> = &mut SlotCInt::new(move |index| {
            unsafe { packed_file_view_widget.as_mut().unwrap().hide() };
        });

        let slots = Self {
            hide: AtomicPtr::new(hide),
        };

        return slots;
    }
}
