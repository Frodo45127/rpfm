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
Module with the slots for Table Views.
!*/

use qt_core::slots::SlotStringRef;

use crate::packedfile_views::table::PackedFileTableViewRaw;
use crate::QString;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Table PackedFile.
pub struct PackedFileTableViewSlots {
    pub filter_line_edit: SlotStringRef<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableViewSlots`.
impl PackedFileTableViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(packed_file_view: PackedFileTableViewRaw) -> Self {

        // When we want to filter when changing the pattern to filter with...
        let filter_line_edit = SlotStringRef::new(move |string| {
            packed_file_view.filter_table(Some(QString::from_std_str(string.to_std_string())));

        });

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            filter_line_edit,
        }
    }
}
