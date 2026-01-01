//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::packedfile_views::{DataSource, utils::set_modified};
use crate::packfile_contents_ui::PackFileContentsUI;
use super::PackedFileTextView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Text PackedFile.
pub struct PackedFileTextViewSlots {
    pub modified: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextViewSlots`.
impl PackedFileTextViewSlots {

    /// This function creates the entire slot pack for Texts.
    pub unsafe fn new(view: &Arc<PackedFileTextView>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Self {

        let modified = SlotNoArgs::new(&view.editor, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Modified Text File` By Slot");
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {

                        // TODO: calculate a checksum of the file to also detect when it has gone back to its "unmodified" state.
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            modified,
        }
    }
}
