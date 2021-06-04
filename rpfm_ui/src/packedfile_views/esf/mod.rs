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
Module with all the code for managing the ESF Views.
!*/

use std::sync::Arc;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::esf::ESF;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::views::debug::DebugView;

use crate::packedfile_views::PackedFileView;

use super::{ViewType, View};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the ESF PackedFile.
pub struct PackedFileESFView {
    debug_view: Arc<DebugView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileESFView`.
impl PackedFileESFView {

    /// This function creates a new PackedFileESFView, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
    ) -> Result<Option<PackedFileInfo>> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::DecodedPackedFilePackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // For now just build a debug view.
        let debug_view = DebugView::new_view(
            &packed_file_view.get_mut_widget(),
            data,
            packed_file_view.get_path_raw(),
        )?;

        let packed_file_debug_view = Self {
            debug_view,
        };

        packed_file_view.view = ViewType::Internal(View::ESF(Arc::new(packed_file_debug_view)));
        packed_file_view.packed_file_type = PackedFileType::ESF;

        Ok(Some(packed_file_info))
    }

    /// This function tries to reload the current view with the provided data.
    pub unsafe fn reload_view(&self, data: &ESF) {
        let text = serde_json::to_string_pretty(&data).unwrap();
        self.debug_view.reload_view(&text);
    }
}
