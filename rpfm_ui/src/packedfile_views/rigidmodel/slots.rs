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
Module with the slots for RigidModel Views.
!*/

use qt_core::slots::SlotNoArgs;

use std::cell::RefCell;
use std::rc::Rc;

use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::rigidmodel::PackedFileRigidModelViewRaw;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an RigidModel PackedFile.
pub struct PackedFileRigidModelViewSlots {
    pub save: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelViewSlots`.
impl PackedFileRigidModelViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(packed_file_view: PackedFileRigidModelViewRaw, global_search_ui: GlobalSearchUI, packed_file_path: &Rc<RefCell<Vec<String>>>) -> Self {

        // When we want to save the contents of the UI to the backend...
        let save = SlotNoArgs::new(clone!(packed_file_path => move || {
            if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {
                packed_file.save(&packed_file_path.borrow(), global_search_ui);
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            save,
        }
    }
}

