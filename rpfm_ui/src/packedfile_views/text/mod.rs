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
Module with all the code for managing the view for Text PackedFiles.
!*/

use qt_widgets::grid_layout::GridLayout;
use qt_widgets::plain_text_edit::PlainTextEdit;
use qt_widgets::widget::Widget;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;
use self::slots::PackedFileTextViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a Text PackedFile.
pub struct PackedFileTextView {
    editor: AtomicPtr<PlainTextEdit>,
}


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextView`.
impl PackedFileTextView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
    ) -> Result<TheOneSlot> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileText(packed_file_path.borrow().to_vec()));
        let text = match CENTRAL_COMMAND.recv_message_qt() {
            Response::Text(text) => text,
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        };

        let editor = PlainTextEdit::new(&QString::from_std_str(&text.get_ref_contents()));
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        unsafe { layout.as_mut().unwrap().add_widget((editor.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }

        packed_file_view.view = View::Text(Self{
            editor: AtomicPtr::new(editor.into_raw())
        });

        // Return success.
        Ok(TheOneSlot::Text(PackedFileTextViewSlots {}))
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_editor(&self) -> &mut PlainTextEdit {
        unsafe { self.editor.load(Ordering::SeqCst).as_mut().unwrap() }
    }
}
