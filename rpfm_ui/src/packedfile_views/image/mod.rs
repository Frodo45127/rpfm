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

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::image::Image;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

#[cfg(feature = "support_modern_dds")]
use crate::ffi::get_dds_qimage;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_resizable_label_safe, set_pixmap_on_resizable_label_safe};
use crate::packedfile_views::{PackedFileView, View, ViewType};

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
        packed_file_view: &mut PackedFileView,
    ) -> Result<PackedFileInfo> {

        // Get the path of the extracted Image.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (image, packed_file_info) = match response {
            Response::ImagePackedFileInfo((image, packed_file_info)) => (image, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Create the image in the UI.
        let byte_array = QByteArray::from_slice(image.get_data()).into_ptr();

        #[cfg(feature = "support_modern_dds")]
        let mut image = QPixmap::new();

        #[cfg(not(feature = "support_modern_dds"))]
        let image = QPixmap::new();

        // If it fails to load and it's a dds, try the modern loader if its enabled.
        if !image.load_from_data_q_byte_array(byte_array.as_ref().unwrap()) {

            #[cfg(feature = "support_modern_dds")] {
                if packed_file_info.path.last().unwrap().to_lowercase().ends_with(".dds") {
                    let image_new = get_dds_qimage(&byte_array);
                    if !image_new.is_null() {
                        image = QPixmap::from_image_1a(image_new.as_ref().unwrap());
                    } else {
                        return Err(ErrorKind::ImageDecode("The image is not supported by the previsualizer.".to_owned()).into());
                    }
                } else {
                    return Err(ErrorKind::ImageDecode("The image is not supported by the previsualizer.".to_owned()).into());
                }
            }

            #[cfg(not(feature = "support_modern_dds"))] {
                return Err(ErrorKind::ImageDecode("The image is not supported by the previsualizer.".to_owned()).into());
            }
        }

        // Get the size of the holding widget.
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        let label = new_resizable_label_safe(&packed_file_view.get_mut_widget().as_ptr(), &image.as_ptr());
        label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        layout.add_widget_5a(&label, 0, 0, 1, 1);

        packed_file_view.packed_file_type = PackedFileType::Image;
        packed_file_view.view = ViewType::Internal(View::Image(Self {
            label,
            image
        }));

        // Return success.
        Ok(packed_file_info)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Image) {
        let byte_array = QByteArray::from_slice(data.get_data());
        self.image.load_from_data_q_byte_array(byte_array.into_ptr().as_ref().unwrap());
        set_pixmap_on_resizable_label_safe(&self.label.as_ptr(), &self.image.as_ptr());
    }
}
