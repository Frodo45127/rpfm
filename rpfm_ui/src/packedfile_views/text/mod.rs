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
use crate::communications::*;
use crate::ffi::{add_text_hover_safe, clear_text_hovers_safe, cursor_row_safe, get_text_safe, new_text_editor_safe, scroll_to_row_safe, set_text_safe};
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
    pack_key: Arc<RwLock<String>>,
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
            pack_key: file_view.pack_key().clone(),
        });

        let slots = PackedFileTextViewSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);
        view.refresh_lua_hovers();

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
        self.refresh_lua_hovers();
    }

    /// This function updates the docs shown when hovering what a Lua script uses, based on the current text of the editor.
    ///
    /// Views of other kinds of files are left untouched.
    pub unsafe fn refresh_lua_hovers(&self) {
        let is_lua = self.packed_file_path.as_ref().is_some_and(|path| path.read().unwrap().ends_with(".lua"));
        if !is_lua {
            return;
        }

        let source = get_text_safe(&self.editor).to_std_string();
        let hovers = send_ipc_command_async(Command::LuaHovers(source), response_extractor!(Response::VecU64U64U64U64String));

        let editor = self.editor.as_ptr();
        clear_text_hovers_safe(&editor);
        for (start_line, start_column, end_line, end_column, html) in hovers {
            add_text_hover_safe(&editor, ((start_line, start_column), (end_line, end_column)), &QString::from_std_str(html).as_ptr());
        }
    }
}
