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
Module with the slots for CA_VP8 Views.
!*/

use qt_core::QString;
use qt_core::Slot;

use std::cell::RefCell;
use std::rc::Rc;

use rpfm_lib::packedfile::ca_vp8::SupportedFormats;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::ca_vp8::PackedFileCaVp8ViewRaw;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a CA_VP8 PackedFile.
pub struct PackedFileCaVp8ViewSlots {
    pub convert_to_camv: Slot<'static>,
    pub convert_to_ivf: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileCaVp8ViewSlots`.
impl PackedFileCaVp8ViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: PackedFileCaVp8ViewRaw,
        mut app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI,
        packed_file_path: &Rc<RefCell<Vec<String>>>
    )  -> Self {

        // Slot to change the format of the video to CAMV.
        let convert_to_camv = Slot::new(clone!(
            mut packed_file_path,
            mut view => move || {
                view.set_current_format(SupportedFormats::Camv);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Camv)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&packed_file_path.borrow(), &mut app_ui, global_search_ui, &mut pack_file_contents_ui);
                }
            }
        ));

        // Slot to change the format of the video to IVF.
        let convert_to_ivf = Slot::new(clone!(
            mut packed_file_path,
            mut view => move || {
                view.set_current_format(SupportedFormats::Ivf);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Ivf)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&packed_file_path.borrow(), &mut app_ui, global_search_ui, &mut pack_file_contents_ui);
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            convert_to_camv,
            convert_to_ivf,
        }
    }
}

