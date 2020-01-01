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
Module with all the code for managing the view for Text PackedFiles.
!*/

use qt_widgets::grid_layout::GridLayout;
use qt_widgets::widget::Widget;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;
use rpfm_lib::packedfile::text::TextType;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor, set_text};
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;
use self::slots::PackedFileTextViewSlots;

pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a Text PackedFile.
pub struct PackedFileTextView {
    editor: AtomicPtr<Widget>,
}

/// This struct contains the raw version of each pointer in `PackedFileTextViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileTextViewRaw`.
#[derive(Clone, Copy)]
pub struct PackedFileTextViewRaw {
    pub editor: *mut Widget,
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
        global_search_ui: &GlobalSearchUI,
        text_type: TextType,
    ) -> Result<TheOneSlot> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileText(packed_file_path.borrow().to_vec()));
        let text = match CENTRAL_COMMAND.recv_message_qt() {
            Response::Text(text) => text,
            Response::Error(error) => return Err(error),
            _ => panic!(THREADS_COMMUNICATION_ERROR),
        };

        let mut highlighting_mode = match text_type {
            TextType::Lua => QString::from_std_str("Lua"),
            TextType::Xml => QString::from_std_str("XML"),
            TextType::Plain => QString::from_std_str("Normal"),
        };

        let editor = unsafe { new_text_editor(packed_file_view.get_mut_widget()) };
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        unsafe { layout.as_mut().unwrap().add_widget((editor, 0, 0, 1, 1)); }

        unsafe { set_text(editor, &mut QString::from_std_str(text.get_ref_contents()), &mut highlighting_mode) };

        let packed_file_text_view_raw = PackedFileTextViewRaw {editor};
        let packed_file_text_view_slots = PackedFileTextViewSlots::new(packed_file_text_view_raw, *global_search_ui, &packed_file_path);
        let packed_file_text_view = Self { editor: AtomicPtr::new(packed_file_text_view_raw.editor)};

        packed_file_view.view = View::Text(packed_file_text_view);

        // Return success.
        Ok(TheOneSlot::Text(packed_file_text_view_slots))
    }

    /// This function returns a mutable reference to the editor widget.
    pub fn get_ref_mut_editor(&self) -> &mut Widget {
        unsafe { self.editor.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> *mut Widget {
        self.editor.load(Ordering::SeqCst)
    }
}
