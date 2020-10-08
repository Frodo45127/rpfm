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
Module with all the code for managing the view for AnimPack PackedFiles.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QPushButton;
use qt_widgets::QPlainTextEdit;

use qt_core::QBox;
use qt_core::QString;
use qt_core::QPtr;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;

use self::slots::PackedFileAnimPackViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimPack PackedFile.
pub struct PackedFileAnimPackView {
    file_count_data_label: QBox<QLabel>,
    file_list_data_text: QBox<QPlainTextEdit>,

    unpack_button: QBox<QPushButton>,
    path: Arc<RwLock<Vec<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimPackView`.
impl PackedFileAnimPackView {

    /// This function creates a new AnimPack View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
    ) -> Result<PackedFileInfo> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::AnimPackPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        let file_count_label = QLabel::from_q_string_q_widget(&qtr("file_count"), packed_file_view.get_mut_widget());
        let file_list_label = QLabel::from_q_string_q_widget(&qtr("file_paths"), packed_file_view.get_mut_widget());

        let file_count_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{}", data.len())), packed_file_view.get_mut_widget());
        let file_list_data_text = QPlainTextEdit::from_q_string_q_widget(&QString::from_std_str(format!("{:#?}", data)), packed_file_view.get_mut_widget());
        file_list_data_text.set_read_only(true);

        let unpack_button = QPushButton::from_q_string_q_widget(&qtr("animpack_unpack"), packed_file_view.get_mut_widget());

        layout.add_widget_5a(&unpack_button, 0, 0, 1, 2);

        layout.add_widget_5a(&file_count_label, 2, 0, 1, 1);
        layout.add_widget_5a(&file_list_label, 3, 0, 1, 1);

        layout.add_widget_5a(&file_count_data_label, 2, 1, 1, 1);
        layout.add_widget_5a(&file_list_data_text, 4, 0, 1, 2);

        layout.set_column_stretch(1, 10);

        let packed_file_animpack_view = Arc::new(PackedFileAnimPackView {
            file_count_data_label,
            file_list_data_text,
            unpack_button,
            path: packed_file_view.get_path_raw()
        });

        let packed_file_animpack_view_slots = PackedFileAnimPackViewSlots::new(
            &packed_file_animpack_view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui
        );

        connections::set_connections(&packed_file_animpack_view, &packed_file_animpack_view_slots);
        packed_file_view.view = ViewType::Internal(View::AnimPack(packed_file_animpack_view));
        packed_file_view.packed_file_type = PackedFileType::AnimPack;

        Ok(packed_file_info)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &[String]) {
        self.get_mut_ptr_file_count_data_label().set_text(&QString::from_std_str(format!("{}", data.len())));
        self.get_mut_ptr_file_list_data_text().set_plain_text(&QString::from_std_str(format!("{:#?}", data)));
    }

    /// This function returns a pointer to the file count label.
    pub fn get_mut_ptr_file_count_data_label(&self) -> &QBox<QLabel> {
        &self.file_count_data_label
    }

    /// This function returns a pointer to the file list view.
    pub fn get_mut_ptr_file_list_data_text(&self) -> &QBox<QPlainTextEdit> {
        &self.file_list_data_text
    }

    /// This function returns a pointer to the `Unpack` button.
    pub fn get_mut_ptr_unpack_button(&self) -> &QBox<QPushButton> {
        &self.unpack_button
    }
}
