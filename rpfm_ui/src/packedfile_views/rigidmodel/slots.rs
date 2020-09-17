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
Module with the slots for RigidModel Views.
!*/

use qt_core::SlotNoArgs;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::rigidmodel::PackedFileRigidModelViewRaw;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an RigidModel PackedFile.
pub struct PackedFileRigidModelViewSlots {
    pub save: SlotNoArgs,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelViewSlots`.
impl PackedFileRigidModelViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(packed_file_view: &PackedFileRigidModelViewRaw, mut app_ui: &Rc<AppUI>, global_search_ui: &Rc<GlobalSearchUI>, mut pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Self {

        // When we want to save the contents of the UI to the backend...
        let save = SlotNoArgs::new(clone!(packed_file_view => move || {
            if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_view.path.read().unwrap()) {
                if let Err(_error) = packed_file.save(&app_ui, global_search_ui, &pack_file_contents_ui) {
                    //show_dialog(packed_file_view.get_table_view_primary(), error, false);
                }
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            save,
        }
    }
}

