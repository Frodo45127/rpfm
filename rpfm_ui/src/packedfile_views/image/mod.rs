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
Module with all the code for managing the view for Images.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;

use qt_gui::QPixmap;

use cpp_core::CppBox;

use qt_core::QFlags;
use qt_core::AlignmentFlag;
use qt_core::QByteArray;
use qt_core::QPtr;

use anyhow::{anyhow, Result};
use rpfm_lib::files::{FileType, image::Image};

use crate::ffi::{new_resizable_label_safe, set_pixmap_on_resizable_label_safe};
use crate::packedfile_views::{FileView, View, ViewType};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an Image PackedFile.
pub struct PackedFileImageView {
    label: QPtr<QLabel>,
    image: CppBox<QPixmap>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileImageView`.
impl PackedFileImageView {

    /// This function creates a new Image View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &Image
    ) -> Result<()> {

        // Create the image in the UI.
        let byte_array = if let Some(data) = data.converted_data() {
            QByteArray::from_slice(data).into_ptr()
        } else {
            QByteArray::from_slice(data.data()).into_ptr()
        };

        let image = QPixmap::new();

        // If it fails to load and it's a dds, try the modern loader if its enabled.
        if !image.load_from_data_q_byte_array(byte_array.as_ref().unwrap()) {
            return Err(anyhow!("The image is not supported by the previsualizer."));
        }

        // Get the size of the holding widget.
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        let label = new_resizable_label_safe(&file_view.main_widget().as_ptr(), &image.as_ptr());
        label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        layout.add_widget_5a(&label, 0, 0, 1, 1);

        file_view.file_type = FileType::Image;
        file_view.view_type = ViewType::Internal(View::Image(Self {
            label,
            image
        }));

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Image) {
        let byte_array = QByteArray::from_slice(data.data());
        self.image.load_from_data_q_byte_array(byte_array.into_ptr().as_ref().unwrap());
        set_pixmap_on_resizable_label_safe(&self.label.as_ptr(), &self.image.as_ptr());
    }
}
