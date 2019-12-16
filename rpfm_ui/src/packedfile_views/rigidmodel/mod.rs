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
Module with all the code for managing the view for RigidModel PackedFiles.
!*/


use qt_widgets::grid_layout::GridLayout;
use qt_widgets::widget::Widget;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor, set_text};
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;
use self::slots::PackedFileRigidModelViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a RigidModel PackedFile.
pub struct PackedFileRigidModelView {
    editor: AtomicPtr<Widget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelView`.
impl PackedFileRigidModelView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
    ) -> Result<TheOneSlot> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileRigidModel(packed_file_path.borrow().to_vec()));
        let rigid_model = match CENTRAL_COMMAND.recv_message_qt() {
            Response::RigidModel(rigid_model) => rigid_model,
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        };

        let editor = unsafe { new_text_editor(packed_file_view.get_mut_widget()) };

        packed_file_view.view = View::RigidModel(Self{
            editor: AtomicPtr::new(editor)
        });

        // Return success.
        Ok(TheOneSlot::RigidModel(PackedFileRigidModelViewSlots {}))
    }

    /// This function returns a mutable reference to the editor widget.
    pub fn get_ref_mut_view(&self) -> &mut Widget {
        unsafe { self.editor.load(Ordering::SeqCst).as_mut().unwrap() }
    }
}
