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
Module with all the code for managing the view for RigidModel PackedFiles.
!*/

use qt_widgets::widget::Widget;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor};
use crate::global_search_ui::GlobalSearchUI;
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

/// This struct contains the raw version of each pointer in `PackedFileRigidViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileRigidModelView`.
#[derive(Clone, Copy)]
pub struct PackedFileRigidModelViewRaw {
    pub editor: *mut Widget,
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
        global_search_ui: &GlobalSearchUI,
    ) -> Result<TheOneSlot> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileRigidModel(packed_file_path.borrow().to_vec()));
        let rigid_model = match CENTRAL_COMMAND.recv_message_qt() {
            Response::RigidModel(rigid_model) => rigid_model,
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        };

        let editor = unsafe { new_text_editor(packed_file_view.get_mut_widget()) };

        let packed_file_rigid_model_view_raw = PackedFileRigidModelViewRaw {editor};
        let packed_file_rigid_model_view_slots = PackedFileRigidModelViewSlots::new(packed_file_rigid_model_view_raw, *global_search_ui, &packed_file_path);
        let packed_file_rigid_model_view = Self { editor: AtomicPtr::new(packed_file_rigid_model_view_raw.editor)};

        packed_file_view.view = View::RigidModel(packed_file_rigid_model_view);

        // Return success.
        Ok(TheOneSlot::RigidModel(packed_file_rigid_model_view_slots))
    }

    /// This function returns a mutable reference to the editor widget.
    pub fn get_ref_mut_view(&self) -> &mut Widget {
        unsafe { self.editor.load(Ordering::SeqCst).as_mut().unwrap() }
    }
}
