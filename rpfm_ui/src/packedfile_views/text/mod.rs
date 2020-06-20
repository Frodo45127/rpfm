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

use qt_widgets::QGridLayout;
use qt_widgets::QWidget;

use cpp_core::MutPtr;

use std::sync::atomic::AtomicPtr;
use std::sync::{Arc, RwLock};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::text::{Text, TextType};
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor_safe, set_text_safe};
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::QString;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::mut_ptr_from_atomic;
use self::slots::PackedFileTextViewSlots;

pub mod slots;

const CPP: &str = "C++";
const HTML: &str = "HTML";
const LUA: &str = "Lua";
const XML: &str = "XML";
const PLAIN: &str = "Normal";
const MARKDOWN: &str = "Markdown";
const JSON: &str = "JSON";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a Text PackedFile.
pub struct PackedFileTextView {
    editor: AtomicPtr<QWidget>,
}

/// This struct contains the raw version of each pointer in `PackedFileTextViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileTextViewRaw`.
#[derive(Clone)]
pub struct PackedFileTextViewRaw {
    pub editor: MutPtr<QWidget>,
    pub path: Arc<RwLock<Vec<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextView`.
impl PackedFileTextView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, Option<PackedFileInfo>)> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (text, packed_file_info) = match response {
            Response::TextPackedFileInfo((text, packed_file_info)) => (text, Some(packed_file_info)),

            // If only the text comes in, it's not a PackedFile.
            Response::Text(text) => (text, None),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut highlighting_mode = match text.get_text_type() {
            TextType::Cpp => QString::from_std_str(CPP),
            TextType::Html => QString::from_std_str(HTML),
            TextType::Lua => QString::from_std_str(LUA),
            TextType::Xml => QString::from_std_str(XML),
            TextType::Plain => QString::from_std_str(PLAIN),
            TextType::Markdown => QString::from_std_str(MARKDOWN),
            TextType::Json => QString::from_std_str(JSON),
        };

        let mut editor = new_text_editor_safe(&mut packed_file_view.get_mut_widget());
        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();
        layout.add_widget_5a(editor, 0, 0, 1, 1);

        set_text_safe(&mut editor, &mut QString::from_std_str(text.get_ref_contents()), &mut highlighting_mode);

        let packed_file_text_view_raw = PackedFileTextViewRaw {editor, path: packed_file_view.get_path_raw() };
        let packed_file_text_view_slots = PackedFileTextViewSlots::new(&packed_file_text_view_raw, *app_ui, *pack_file_contents_ui, *global_search_ui);
        let packed_file_text_view = Self { editor: atomic_from_mut_ptr(packed_file_text_view_raw.editor)};

        packed_file_view.packed_file_type = PackedFileType::Text(text.get_text_type());
        packed_file_view.view = ViewType::Internal(View::Text(packed_file_text_view));

        // Return success.
        Ok((TheOneSlot::Text(packed_file_text_view_slots), packed_file_info))
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> MutPtr<QWidget> {
        mut_ptr_from_atomic(&self.editor)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Text) {
        let mut editor = mut_ptr_from_atomic(&self.editor);

        let mut highlighting_mode = match data.get_text_type() {
            TextType::Cpp => QString::from_std_str(CPP),
            TextType::Html => QString::from_std_str(HTML),
            TextType::Lua => QString::from_std_str(LUA),
            TextType::Xml => QString::from_std_str(XML),
            TextType::Plain => QString::from_std_str(PLAIN),
            TextType::Markdown => QString::from_std_str(MARKDOWN),
            TextType::Json => QString::from_std_str(JSON),
        };

        set_text_safe(&mut editor, &mut QString::from_std_str(data.get_ref_contents()), &mut highlighting_mode);
    }
}

/// Implementation of `PackedFileTextViewRaw`.
impl PackedFileTextViewRaw {

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> MutPtr<QWidget> {
        self.editor
    }
}
