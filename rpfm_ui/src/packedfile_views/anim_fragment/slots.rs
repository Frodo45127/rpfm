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
Module with the slots for AnimFragment Views.
!*/

use qt_core::QString;
use qt_core::Slot;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use super::PackedFileAnimFragmentViewRaw;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimFragment PackedFile.
pub struct PackedFileAnimFragmentViewSlots {
    //pub convert_to_camv: Slot<'static>,
    //pub convert_to_ivf: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimFragmentViewSlots`.
impl PackedFileAnimFragmentViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: PackedFileAnimFragmentViewRaw,
        mut app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI
    )  -> Self {
/*
        // Slot to change the format of the video to CAMV.
        let convert_to_camv = Slot::new(clone!(
            mut view,
            mut view => move || {
                view.set_current_format(SupportedFormats::Camv);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Camv)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *view.path.read().unwrap()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&mut app_ui, global_search_ui, &mut pack_file_contents_ui);
                }
            }
        ));

        // Slot to change the format of the video to IVF.
        let convert_to_ivf = Slot::new(clone!(
            mut view,
            mut view => move || {
                view.set_current_format(SupportedFormats::Ivf);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Ivf)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *view.path.read().unwrap()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&mut app_ui, global_search_ui, &mut pack_file_contents_ui);
                }
            }
        ));
*/
        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            //convert_to_camv,
            //convert_to_ivf,
        }
    }
}

