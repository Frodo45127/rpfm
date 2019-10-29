//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the temporal PackFile TreeView used when adding PackedFiles from another PackFile.

This is here because we're going to treat it as another PackedFileView, though it isn't.
But this allow us to integrate it into the main PackedFileView system, so it's ok.
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
use self::slots::PackFileExtraViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the extra PackFile.
pub struct PackFileExtraView {

}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraView`.
impl PackFileExtraView {

    /// This function creates a new Image View, and sets up his slots and connections.
    pub fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
    ) -> Result<TheOneSlot> {
/*
        // Get the path of the extracted Image.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileImage(packed_file_path.borrow().to_vec()));
        let path = match CENTRAL_COMMAND.recv_message_qt() {
            Response::PathBuf(data) => data,
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        };

        // Create the widget that'll act as a container for the view.
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };

        // Create the stuff.
        let tree_view = TreeView::new().into_raw();
        let tree_model = StandardItemModel::new(()).into_raw();

        // Configure it.
        unsafe { tree_view.as_mut().unwrap().set_model(tree_model as *mut AbstractItemModel); }
        unsafe { tree_view.as_mut().unwrap().set_header_hidden(true); }
        unsafe { tree_view.as_mut().unwrap().set_expands_on_double_click(false); }
        unsafe { tree_view.as_mut().unwrap().set_animated(true); }

        // Add all the stuff to the Grid.
        unsafe { widget_layout.as_mut().unwrap().add_widget((tree_view as *mut Widget, 1, 0, 1, 1)); }
*/
        packed_file_view.view = View::PackFile(Self {});

        // Return success.
        Ok(TheOneSlot::PackFile(PackFileExtraViewSlots {}))
    }
}
