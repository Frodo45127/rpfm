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

use qt_core::AspectRatioMode;
use qt_core::QFlags;
use qt_core::AlignmentFlag;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;

use rpfm_error::Result;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;
use self::slots::PackedFileImageViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an Image PackedFile.
pub struct PackedFileImageView {}


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileImageView`.
impl PackedFileImageView {

    /// This function creates a new Image View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        // Get the path of the extracted Image.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileImage(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (path, packed_file_info) = match response {
            Response::PathBufPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Get the image's path.
        let path_string = path.to_string_lossy().as_ref().to_string();
        let image = QPixmap::from_q_string(&QString::from_std_str(&path_string));

        // Get the size of the holding widget.
        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();
        let widget_height = layout.parent_widget().height();
        let widget_width = layout.parent_widget().width();

        let scaled_image = if image.height() >= widget_height || image.width() >= widget_width {
            image.scaled_2_int_aspect_ratio_mode(widget_height - 25, widget_width - 25, AspectRatioMode::KeepAspectRatio)
        } else { image };

        // Create a Label.
        let mut label = QLabel::new();
        label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        label.set_pixmap(&scaled_image);
        layout.add_widget_5a(&mut label, 0, 0, 1, 1);

        packed_file_view.view = View::Image(Self {});

        // Return success.
        Ok((TheOneSlot::Image(PackedFileImageViewSlots {}), packed_file_info))
    }
}
