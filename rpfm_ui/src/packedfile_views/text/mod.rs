//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
use crate::ffi::{cursor_row_safe, new_text_editor_safe, scroll_to_row_safe, set_text_safe};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, FileView, View, ViewType};
use crate::packedfile_views::text::slots::PackedFileTextViewSlots;

mod connections;
mod slots;

const BAT: &str = "MS-DOS Batch";
const CPP: &str = "C++";
const HTML: &str = "HTML";
const LUA: &str = "Lua";
const XML: &str = "XML";
const PLAIN: &str = "Normal";
const MARKDOWN: &str = "Markdown";
const JSON: &str = "JSON";
const CSS: &str = "CSS";
const JS: &str = "Javascript";
const PYTHON: &str = "Python";
const SQL: &str = "SQL";
const YAML: &str = "Yaml";

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
        file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        data: &Text,
    ) {

        let highlighting_mode = match data.format() {
            TextFormat::Bat => QString::from_std_str(BAT),
            TextFormat::Cpp => QString::from_std_str(CPP),
            TextFormat::Html => QString::from_std_str(HTML),
            TextFormat::Hlsl => QString::from_std_str(CPP),
            TextFormat::Lua => QString::from_std_str(LUA),
            TextFormat::Xml => QString::from_std_str(XML),
            TextFormat::Plain => QString::from_std_str(PLAIN),
            TextFormat::Markdown => QString::from_std_str(MARKDOWN),
            TextFormat::Json => QString::from_std_str(JSON),
            TextFormat::Css => QString::from_std_str(CSS),
            TextFormat::Js => QString::from_std_str(JS),
            TextFormat::Python => QString::from_std_str(PYTHON),
            TextFormat::Sql => QString::from_std_str(SQL),
            TextFormat::Yaml => QString::from_std_str(YAML),
        };

        let editor = new_text_editor_safe(&file_view.main_widget().static_upcast());
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&editor, 0, 0, 1, 1);

        set_text_safe(&editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());

        let view = Arc::new(PackedFileTextView {
            editor,
            packed_file_path: Some(file_view.path_raw()),
            data_source: file_view.data_source.clone(),
        });

        let slots = PackedFileTextViewSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        file_view.file_type = FileType::Text;
        file_view.view_type = ViewType::Internal(View::Text(view));
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> &QBox<QWidget> {
        &self.editor
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Text) {

        let highlighting_mode = match data.format() {
            TextFormat::Bat => QString::from_std_str(BAT),
            TextFormat::Cpp => QString::from_std_str(CPP),
            TextFormat::Html => QString::from_std_str(HTML),
            TextFormat::Hlsl => QString::from_std_str(CPP),
            TextFormat::Lua => QString::from_std_str(LUA),
            TextFormat::Xml => QString::from_std_str(XML),
            TextFormat::Plain => QString::from_std_str(PLAIN),
            TextFormat::Markdown => QString::from_std_str(MARKDOWN),
            TextFormat::Json => QString::from_std_str(JSON),
            TextFormat::Css => QString::from_std_str(CSS),
            TextFormat::Js => QString::from_std_str(JS),
            TextFormat::Python => QString::from_std_str(PYTHON),
            TextFormat::Sql => QString::from_std_str(SQL),
            TextFormat::Yaml => QString::from_std_str(YAML),
        };

        let row_number = cursor_row_safe(&self.editor.as_ptr());
        set_text_safe(&self.editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());

        // Try to scroll to the line we were before.
        scroll_to_row_safe(&self.editor.as_ptr(), row_number);
    }
}
