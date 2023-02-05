//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the generic debug view.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;

use std::sync::{Arc, RwLock};

use rpfm_lib::files::{FileType, RFileDecoded};

use crate::ffi::{new_text_editor_safe, set_text_safe, get_text_safe};
use crate::locale::qtr;
use crate::views::debug::slots::DebugViewSlots;

use crate::QString;

const JSON: &str = "JSON";

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the Debug View of a PackedFile.
pub struct DebugView {
    editor: QBox<QWidget>,
    save_button: QBox<QPushButton>,
    packed_file_type: FileType,
    path: Arc<RwLock<String>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `DebugView`.
impl DebugView {

    /// This function creates a new Debug View, and sets up his slots and connections.
    pub unsafe fn new_view(
        parent: &QBox<QWidget>,
        packed_file: RFileDecoded,
        packed_file_path: Arc<RwLock<String>>
    ) -> Result<Arc<Self>> {
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        let editor = new_text_editor_safe(&parent.static_upcast());
        let save_button = QPushButton::from_q_string_q_widget(&qtr("save_changes"), parent);

        layout.add_widget_5a(&editor, 0, 0, 1, 1);
        layout.add_widget_5a(&save_button, 2, 0, 1, 1);

        let (packed_file_type, text) = match packed_file {
            RFileDecoded::AnimFragment(data) => (FileType::AnimFragment, serde_json::to_string_pretty(&data)?),
            RFileDecoded::AnimsTable(data) => (FileType::AnimsTable, serde_json::to_string_pretty(&data)?),
            RFileDecoded::MatchedCombat(data) => (FileType::MatchedCombat, serde_json::to_string_pretty(&data)?),
            RFileDecoded::UnitVariant(data) => (FileType::UnitVariant, serde_json::to_string_pretty(&data)?),
            RFileDecoded::ESF(data) => (FileType::ESF, serde_json::to_string_pretty(&data)?),
            _ => unimplemented!(),
        };

        set_text_safe(&editor.static_upcast(), &QString::from_std_str(text).as_ptr(), &QString::from_std_str(JSON).as_ptr());

        let packed_file_debug_view = Arc::new(Self {
            editor,
            save_button,
            packed_file_type,
            path: packed_file_path,
        });

        let packed_file_debug_view_slots = DebugViewSlots::new(&packed_file_debug_view);
        connections::set_connections(&packed_file_debug_view, &packed_file_debug_view_slots);

        Ok(packed_file_debug_view)
    }

    /// This function tries to parse the passed file as a PackedFile and returns it.
    pub fn save_view(&self) -> Result<RFileDecoded> {
        let string = get_text_safe(&self.editor).to_std_string();

        let decoded_packed_file = match self.packed_file_type {
            FileType::AnimFragment => RFileDecoded::AnimFragment(serde_json::from_str(&string)?),
            FileType::AnimsTable => RFileDecoded::AnimsTable(serde_json::from_str(&string)?),
            FileType::MatchedCombat => RFileDecoded::MatchedCombat(serde_json::from_str(&string)?),
            FileType::UnitVariant => RFileDecoded::UnitVariant(serde_json::from_str(&string)?),
            FileType::ESF => RFileDecoded::ESF(serde_json::from_str(&string)?),
            _ => unimplemented!(),
        };

        Ok(decoded_packed_file)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &str) {
        let highlighting_mode = QString::from_std_str(JSON);
        set_text_safe(&self.editor.static_upcast(), &QString::from_std_str(data).as_ptr(), &highlighting_mode.as_ptr());
    }

    /// This function returns a copy of the path of this `DebugView`.
    pub fn get_path(&self) -> String {
      self.path.read().unwrap().to_owned()
    }
}
