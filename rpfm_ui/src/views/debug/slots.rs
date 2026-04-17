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
Module with the slots for Debug Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::communications::{Command, send_ipc_command_result_async};
use crate::views::debug::DebugView;
use crate::utils::{log_to_status_bar, show_dialog, tr};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the Debug View of a PackedFile.
pub struct DebugViewSlots {
    pub save: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `DebugViewSlots`.
impl DebugViewSlots {

    /// This function creates the entire slot pack for the Debug View.
    pub unsafe fn new(
        view: &Arc<DebugView>,
    ) -> Self {

        // When we want to try to save the data to the backend.
        let save = SlotNoArgs::new(&view.editor, clone!(
            view => move || {
            rpfm_telemetry::track_action("Debug View: Save");
            match view.save_view() {
                Ok(decoded_packed_file) => {
                    let pack_key = String::new();
                    match send_ipc_command_result_async(Command::SavePackedFileFromView(pack_key, view.get_path(), decoded_packed_file), response_extractor!()) {
                        Ok(()) => log_to_status_bar(&tr("debug_view_save_success")),
                        Err(error) => show_dialog(&view.editor, error, false),
                    }
                }
                Err(error) => show_dialog(&view.editor, error, false),
            }
        }));

        Self {
            save,
        }
    }
}
