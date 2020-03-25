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

use qt_widgets::QWidget;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicPtr;

use rpfm_error::Result;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor_safe};
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::utils::atomic_from_mut_ptr;

use self::slots::PackedFileRigidModelViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a RigidModel PackedFile.
pub struct PackedFileRigidModelView {
    editor: AtomicPtr<QWidget>,
}

/// This struct contains the raw version of each pointer in `PackedFileRigidViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileRigidModelView`.
#[derive(Clone, Copy)]
pub struct PackedFileRigidModelViewRaw {
    pub editor: MutPtr<QWidget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileRigidModelView`.
impl PackedFileRigidModelView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileRigidModel(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (_rigid_model, packed_file_info) = match response {
            Response::RigidModelPackedFileInfo((rigid_model, packed_file_info)) => (rigid_model, packed_file_info),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let editor = new_text_editor_safe(&mut packed_file_view.get_mut_widget());

        let packed_file_rigid_model_view_raw = PackedFileRigidModelViewRaw {editor};
        let packed_file_rigid_model_view_slots = PackedFileRigidModelViewSlots::new(packed_file_rigid_model_view_raw, *global_search_ui, *pack_file_contents_ui, &packed_file_path);
        let packed_file_rigid_model_view = Self { editor: atomic_from_mut_ptr(packed_file_rigid_model_view_raw.editor) };

        packed_file_view.view = ViewType::Internal(View::RigidModel(packed_file_rigid_model_view));

        // Return success.
        Ok((TheOneSlot::RigidModel(packed_file_rigid_model_view_slots), packed_file_info))
    }
}
