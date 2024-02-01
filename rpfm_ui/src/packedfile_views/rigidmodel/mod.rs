//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
#[cfg(feature = "support_rigidmodel")] use qt_core::QByteArray;
use qt_core::QPtr;

use std::sync::Arc;
#[cfg(feature = "support_model_renderer")] use std::sync::RwLock;

use anyhow::Result;
use getset::*;

use rpfm_lib::files::FileType;
use rpfm_lib::files::rigidmodel::RigidModel;

use crate::ffi::*;
use crate::packedfile_views::{FileView, View, ViewType};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a RigidModel PackedFile.
#[derive(Getters)]
pub struct PackedFileRigidModelView {
    #[cfg(feature = "support_rigidmodel")] editor: QBox<QWidget>,
    #[cfg(feature = "support_model_renderer")] renderer: QBox<QWidget>,

    #[cfg(feature = "support_model_renderer")] path: Option<Arc<RwLock<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelView`.
impl PackedFileRigidModelView {

    /// This function creates a new RigidModel View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &RigidModel,
    ) -> Result<()> {

        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        let view = Arc::new(PackedFileRigidModelView{
            #[cfg(feature = "support_rigidmodel")] editor: {
                let data = QByteArray::from_slice(data.data());
                let editor = new_rigid_model_view_safe(&mut file_view.main_widget().as_ptr());
                set_rigid_model_view_safe(&mut editor.as_ptr(), &data.as_ptr())?;
                layout.add_widget_5a(&editor, 0, 0, 1, 1);
                editor
            },

            #[cfg(feature = "support_model_renderer")] renderer: {
                let renderer = create_q_rendering_widget(&mut file_view.main_widget().as_ptr());
                add_new_primary_asset(&renderer.as_ptr(), &file_view.path().read().unwrap(), data.data());
                layout.add_widget_5a(&renderer, 0, 1, 1, 1);
                renderer
            },

            #[cfg(feature = "support_model_renderer")] path: Some(file_view.path_raw()),
        });

        file_view.file_type = FileType::RigidModel;
        file_view.view_type = ViewType::Internal(View::RigidModel(view));

        Ok(())
    }

    /// Function to save the view and encode it into a RigidModel struct.
    #[cfg(feature = "support_rigidmodel")] pub unsafe fn save_view(&self) -> Result<RigidModel> {
        let mut rigidmodel = RigidModel::default();
        let qdata = get_rigid_model_from_view_safe(&self.editor)?;
        let data = std::slice::from_raw_parts(qdata.data_mut() as *mut u8, qdata.length() as usize).to_vec();
        rigidmodel.set_data(data);
        Ok(rigidmodel)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    #[cfg(any(feature = "support_rigidmodel", feature = "support_model_renderer"))] pub unsafe fn reload_view(&self, data: &RigidModel) -> Result<()> {

        #[cfg(feature = "support_rigidmodel")] {
            let byte_array = QByteArray::from_slice(data.data());
            set_rigid_model_view_safe(&mut self.editor.as_ptr(), &byte_array.as_ptr())?;
        }

        #[cfg(feature = "support_model_renderer")] {
            if let Some(ref path) = self.path {
                add_new_primary_asset(&self.renderer.as_ptr(), &path.read().unwrap(), data.data());
            }
        }

        Ok(())
    }
}
