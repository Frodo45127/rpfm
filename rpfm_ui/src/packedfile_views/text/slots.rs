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
Module with the slots for Text Views.
!*/

use qt_core::slots::SlotNoArgs;

use std::cell::RefCell;
use std::rc::Rc;

use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::text::PackedFileTextViewRaw;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Text PackedFile.
pub struct PackedFileTextViewSlots {
    pub save: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextViewSlots`.
impl PackedFileTextViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(packed_file_view: PackedFileTextViewRaw, global_search_ui: GlobalSearchUI, packed_file_path: &Rc<RefCell<Vec<String>>>) -> Self {

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = SlotNoArgs::new(clone!(packed_file_path => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {
                    packed_file.save(&packed_file_path.borrow(), global_search_ui);
                }
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            save,
        }
    }
}
