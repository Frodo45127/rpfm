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

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::text::{Text, TextType};
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_text_editor_safe, set_text_safe};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, PackedFileView, View, ViewType};
use crate::packedfile_views::text::slots::PackedFileTextViewSlots;

mod connections;
mod slots;

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
    editor: QBox<QWidget>,
    packed_file_path: Option<Arc<RwLock<Vec<String>>>>,
    data_source: Arc<RwLock<DataSource>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTextView`.
impl PackedFileTextView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<Option<PackedFileInfo>> {

        // Get the decoded Text.
        let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CentralCommand::recv(&receiver);
        let (text, packed_file_info) = match response {
            Response::TextPackedFileInfo((text, packed_file_info)) => (text, Some(packed_file_info)),

            // If only the text comes in, it's not a PackedFile.
            Response::Text(text) => (text, None),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let highlighting_mode = match text.get_text_type() {
            TextType::Cpp => QString::from_std_str(CPP),
            TextType::Html => QString::from_std_str(HTML),
            TextType::Lua => QString::from_std_str(LUA),
            TextType::Xml => QString::from_std_str(XML),
            TextType::Plain => QString::from_std_str(PLAIN),
            TextType::Markdown => QString::from_std_str(MARKDOWN),
            TextType::Json => QString::from_std_str(JSON),
        };

        let editor = new_text_editor_safe(&packed_file_view.get_mut_widget().static_upcast());
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&editor, 0, 0, 1, 1);

        set_text_safe(&editor.static_upcast(), &QString::from_std_str(text.get_ref_contents()).as_ptr(), &highlighting_mode.as_ptr());

        let view = Arc::new(PackedFileTextView {
            editor,
            packed_file_path: Some(packed_file_view.get_path_raw()),
            data_source: Arc::new(RwLock::new(packed_file_view.get_data_source())),
        });

        let slots = PackedFileTextViewSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        packed_file_view.packed_file_type = PackedFileType::Text(text.get_text_type());
        packed_file_view.view = ViewType::Internal(View::Text(view));

        // Return success.
        Ok(packed_file_info)
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> &QBox<QWidget> {
        &self.editor
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Text) {

        let highlighting_mode = match data.get_text_type() {
            TextType::Cpp => QString::from_std_str(CPP),
            TextType::Html => QString::from_std_str(HTML),
            TextType::Lua => QString::from_std_str(LUA),
            TextType::Xml => QString::from_std_str(XML),
            TextType::Plain => QString::from_std_str(PLAIN),
            TextType::Markdown => QString::from_std_str(MARKDOWN),
            TextType::Json => QString::from_std_str(JSON),
        };

        set_text_safe(&self.editor.static_upcast(), &QString::from_std_str(data.get_ref_contents()).as_ptr(), &highlighting_mode.as_ptr());
    }
}
