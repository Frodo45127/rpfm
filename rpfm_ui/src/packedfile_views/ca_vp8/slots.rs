//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_core::QBox;
use qt_core::QString;
use qt_core::SlotNoArgs;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::packedfile::ca_vp8::SupportedFormats;

use crate::app_ui::AppUI;
use crate::packedfile_views::DataSource;
use crate::packedfile_views::ca_vp8::PackedFileCaVp8View;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a CA_VP8 PackedFile.
pub struct PackedFileCaVp8ViewSlots {
    pub convert_to_camv: QBox<SlotNoArgs>,
    pub convert_to_ivf: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileCaVp8ViewSlots`.
impl PackedFileCaVp8ViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileCaVp8View>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    )  -> Self {

        // Slot to change the format of the video to CAMV.
        let convert_to_camv = SlotNoArgs::new(&view.format_data_label, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.set_current_format(SupportedFormats::Camv);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Camv)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == *view.path.read().unwrap()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // Slot to change the format of the video to IVF.
        let convert_to_ivf = SlotNoArgs::new(&view.format_data_label, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.set_current_format(SupportedFormats::Ivf);
                view.format_data_label.set_text(&QString::from_std_str(format!("{:?}", SupportedFormats::Ivf)));
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == *view.path.read().unwrap()) {

                    // This can never fail, so ignore the result.
                    let _ = packed_file.save(&app_ui, &pack_file_contents_ui);
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

