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

use qt_core::Slot;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::text::PackedFileTextViewRaw;
use crate::UI_STATE;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Text PackedFile.
pub struct PackedFileTextViewSlots {
    pub save: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextViewSlots`.
impl PackedFileTextViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(packed_file_view: &PackedFileTextViewRaw, mut app_ui: AppUI, mut pack_file_contents_ui: PackFileContentsUI, global_search_ui: GlobalSearchUI) -> Self {

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = Slot::new(clone!(packed_file_view => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_view.path.read().unwrap()) {
                    if let Err(error) = packed_file.save(&mut app_ui, global_search_ui, &mut pack_file_contents_ui) {
                        show_dialog(packed_file_view.get_mut_editor(), error, false);
                    }
                }
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            save,
        }
    }
}
