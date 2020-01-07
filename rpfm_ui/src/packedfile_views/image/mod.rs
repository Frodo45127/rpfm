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

use qt_widgets::grid_layout::GridLayout;
use qt_widgets::label::Label;
use qt_widgets::widget::Widget;

use qt_gui::pixmap::Pixmap;

use qt_core::qt::AspectRatioMode;
use qt_core::flags::Flags;

use std::cell::RefCell;
use std::rc::Rc;

use rpfm_error::Result;

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
    pub fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
    ) -> Result<TheOneSlot> {

        // Get the path of the extracted Image.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileImage(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let path = match response {
            Response::PathBuf(data) => data,
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Get the image's path.
        let path_string = path.to_string_lossy().as_ref().to_string();
        let image = Pixmap::new(&QString::from_std_str(&path_string));

        // Get the size of the holding widget.
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        let widget_height = unsafe { layout.as_mut().unwrap().parent_widget().as_mut().unwrap().height() };
        let widget_width = unsafe { layout.as_mut().unwrap().parent_widget().as_mut().unwrap().width() };

        let scaled_image = if image.height() >= widget_height || image.width() >= widget_width {
            image.scaled((widget_height - 25, widget_width - 25, AspectRatioMode::KeepAspectRatio))
        } else { image };

        // Create a Label.
        let label = Label::new(()).into_raw();
        unsafe { label.as_mut().unwrap().set_alignment(Flags::from_int(132))}
        unsafe { label.as_mut().unwrap().set_pixmap(&scaled_image); }
        unsafe { layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }

        packed_file_view.view = View::Image(Self {});

        // Return success.
        Ok(TheOneSlot::Image(PackedFileImageViewSlots {}))
    }
}
