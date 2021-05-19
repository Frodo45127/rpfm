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
Module with the slots for PackFile Settings Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;
use std::sync::Arc;

use crate::AppUI;
use crate::packedfile_views::{DataSource, PackedFileType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use super::PackFileSettingsView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a PackFile Settings.
pub struct PackFileSettingsSlots {
    pub apply: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileSettingsSlots`.
impl PackFileSettingsSlots {

    /// This function creates the entire slot pack for PackFile Settings Views.
    pub unsafe fn new(
        view: &Arc<PackFileSettingsView>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    )  -> Self {

        // Slot to apply settings changes.
        let apply = SlotNoArgs::new(view.get_ref_apply_button(), clone!(
            app_ui,
            pack_file_contents_ui=> move || {
                if let Some(pack_file_view) = UI_STATE.get_open_packedfiles().iter()
                    .filter(|x| x.get_data_source() == DataSource::PackFile)
                    .find(|x| matches!(x.get_packed_file_type(), PackedFileType::PackFileSettings)) {
                    let _ = pack_file_view.save(&app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            apply,
        }
    }
}
