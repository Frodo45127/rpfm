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


use qt_core::QFlags;
use qt_core::AlignmentFlag;
use qt_core::QByteArray;

use cpp_core::MutPtr;

use std::sync::atomic::AtomicPtr;

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::image::Image;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_resizable_label_safe, set_pixmap_on_resizable_label_safe};
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::utils::{atomic_from_mut_ptr, mut_ptr_from_atomic};
use self::slots::PackedFileImageViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an Image PackedFile.
pub struct PackedFileImageView {
    label: AtomicPtr<QLabel>,
    image: AtomicPtr<QPixmap>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileImageView`.
impl PackedFileImageView {

    /// This function creates a new Image View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        // Get the path of the extracted Image.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (image, packed_file_info) = match response {
            Response::ImagePackedFileInfo((image, packed_file_info)) => (image, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Create the image in the UI.
        let byte_array = QByteArray::from_slice(image.get_data());
        let mut image = QPixmap::new().into_ptr();
        image.load_from_data_q_byte_array(byte_array.into_ptr().as_ref().unwrap());

        // Get the size of the holding widget.
        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();
        let mut label = new_resizable_label_safe(&mut packed_file_view.get_mut_widget(), &mut image);
        label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        layout.add_widget_5a(label.as_mut_raw_ptr(), 0, 0, 1, 1);

        packed_file_view.packed_file_type = PackedFileType::Image;
        packed_file_view.view = ViewType::Internal(View::Image(Self {
            image: atomic_from_mut_ptr(image),
            label: atomic_from_mut_ptr(label)
        }));

        // Return success.
        Ok((TheOneSlot::Image(PackedFileImageViewSlots {}), packed_file_info))
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Image) {
        let mut image = mut_ptr_from_atomic(&self.image);
        let mut label = mut_ptr_from_atomic(&self.label);

        let byte_array = QByteArray::from_slice(data.get_data());
        image.load_from_data_q_byte_array(byte_array.into_ptr().as_ref().unwrap());
        set_pixmap_on_resizable_label_safe(&mut label, &mut image);
    }
}
