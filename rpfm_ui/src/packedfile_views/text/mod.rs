//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_lib::files::{FileType, text::*};

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
    packed_file_path: Option<Arc<RwLock<String>>>,
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
        data: &Text,
    ) {

        let highlighting_mode = match data.format() {
            TextFormat::Cpp => QString::from_std_str(CPP),
            TextFormat::Html => QString::from_std_str(HTML),
            TextFormat::Lua => QString::from_std_str(LUA),
            TextFormat::Xml => QString::from_std_str(XML),
            TextFormat::Plain => QString::from_std_str(PLAIN),
            TextFormat::Markdown => QString::from_std_str(MARKDOWN),
            TextFormat::Json => QString::from_std_str(JSON),
        };

        let editor = new_text_editor_safe(&packed_file_view.get_mut_widget().static_upcast());
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&editor, 0, 0, 1, 1);

        set_text_safe(&editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());

        let view = Arc::new(PackedFileTextView {
            editor,
            packed_file_path: Some(packed_file_view.get_path_raw()),
            data_source: Arc::new(RwLock::new(packed_file_view.get_data_source())),
        });

        let slots = PackedFileTextViewSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        packed_file_view.packed_file_type = FileType::Text;
        packed_file_view.view = ViewType::Internal(View::Text(view));
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> &QBox<QWidget> {
        &self.editor
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Text) {

        let highlighting_mode = match data.format() {
            TextFormat::Cpp => QString::from_std_str(CPP),
            TextFormat::Html => QString::from_std_str(HTML),
            TextFormat::Lua => QString::from_std_str(LUA),
            TextFormat::Xml => QString::from_std_str(XML),
            TextFormat::Plain => QString::from_std_str(PLAIN),
            TextFormat::Markdown => QString::from_std_str(MARKDOWN),
            TextFormat::Json => QString::from_std_str(JSON),
        };

        set_text_safe(&self.editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());
    }
}
