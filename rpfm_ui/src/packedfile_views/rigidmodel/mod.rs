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
Module with all the code for managing the view for RigidModel PackedFiles.

This module simply calls QtRME lib with some data and the lib is the one taking care of all the processing.
!*/


use qt_widgets::QGridLayout;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::QPtr;

use std::sync::Arc;

use rpfm_error::{Result, ErrorKind};
use rpfm_macros::*;

use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::rigidmodel::RigidModel;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::packedfile_views::{PackedFileView, View, ViewType};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a RigidModel PackedFile.
#[derive(GetRef)]
pub struct PackedFileRigidModelView {
    editor: QBox<QWidget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelView`.
impl PackedFileRigidModelView {

    /// This function creates a new RigidModel View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
    ) -> Result<Option<PackedFileInfo>> {

        // Get the decoded data from the backend.
        let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CentralCommand::recv(&receiver);
        let (rigid_model, packed_file_info) = match response {
            Response::RigidModelPackedFileInfo((rigid_model, packed_file_info)) => (rigid_model, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Create the new view and populate it.
        let data = QByteArray::from_slice(&rigid_model.data);
        let editor = new_rigid_model_view_safe(&mut packed_file_view.get_mut_widget().as_ptr());
        set_rigid_model_view_safe(&mut editor.as_ptr(), &data.as_ptr())?;

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&editor, 0, 0, 1, 1);

        let view = Arc::new(PackedFileRigidModelView{
            editor,
        });

        packed_file_view.packed_file_type = PackedFileType::RigidModel;
        packed_file_view.view = ViewType::Internal(View::RigidModel(view));

        Ok(Some(packed_file_info))
    }

    /// Function to save the view and encode it into a RigidModel struct.
    pub unsafe fn save_view(&self) -> Result<RigidModel> {
        let qdata = get_rigid_model_from_view_safe(&self.editor)?;
        let data = std::slice::from_raw_parts(qdata.data_mut() as *mut u8, qdata.length() as usize).to_vec();
        Ok(RigidModel {
            data
        })
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &RigidModel) -> Result<()> {
        let byte_array = QByteArray::from_slice(&data.data);
        set_rigid_model_view_safe(&mut self.editor.as_ptr(), &byte_array.as_ptr())
    }
}
