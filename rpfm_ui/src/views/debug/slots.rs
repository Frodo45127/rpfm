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
Module with the slots for Debug Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::sync::Arc;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::tr;
use crate::utils::{show_dialog, log_to_status_bar};
use crate::views::debug::DebugView;

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
            match view.save_view() {
                Ok(decoded_packed_file) => {
                    let receiver = CENTRAL_COMMAND.send_background(Command::SavePackedFileFromView(view.get_path(), decoded_packed_file));
                    let response = CENTRAL_COMMAND.recv_try(&receiver);
                    match response {
                        Response::Success => log_to_status_bar(&tr("debug_view_save_success")),
                        Response::Error(error) => show_dialog(&view.editor, error, false),

                        // In ANY other situation, it's a message problem.
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
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
