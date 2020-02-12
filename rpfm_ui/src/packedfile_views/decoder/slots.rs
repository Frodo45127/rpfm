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
Module with the slots for Text Views.
!*/

use qt_core::slots::{SlotCInt, SlotNoArgs};

use std::cell::RefCell;
use std::rc::Rc;

use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;

use super::PackedFileDecoderViewRaw;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Text PackedFile.
pub struct PackedFileDecoderViewSlots {
    pub hex_view_scroll_sync: SlotCInt<'static>,
    pub hex_view_selection_raw_sync: SlotNoArgs<'static>,
    pub hex_view_selection_decoded_sync: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileDecoderViewSlots`.
impl PackedFileDecoderViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(view: PackedFileDecoderViewRaw, pack_file_contents_ui: PackFileContentsUI, global_search_ui: GlobalSearchUI, packed_file_path: &Rc<RefCell<Vec<String>>>) -> Self {

        // Slot to keep scroll in views in sync.
        let hex_view_scroll_sync = SlotCInt::new(move |value| {
            unsafe { view.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
            unsafe { view.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
            unsafe { view.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
        });

        // Slot to keep selection in views in sync.
        let hex_view_selection_raw_sync = SlotNoArgs::new(move || {
            view.hex_selection_sync(true);
        });

        // Slot to keep selection in views in sync.
        let hex_view_selection_decoded_sync = SlotNoArgs::new(move || {
            view.hex_selection_sync(false);
        });

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            hex_view_scroll_sync,
            hex_view_selection_raw_sync,
            hex_view_selection_decoded_sync,
        }
    }
}
