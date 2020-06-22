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

use qt_core::QString;

use cpp_core::MutPtr;

use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicPtr;

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;

use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::mut_ptr_from_atomic;
use self::slots::PackedFileAnimPackViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimPack PackedFile.
pub struct PackedFileAnimPackView {
    unpack_button: AtomicPtr<QPushButton>,
}

/// This struct contains the raw version of each pointer in `PackedFileAnimPackView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileAnimPackView`.
#[derive(Clone)]
pub struct PackedFileAnimPackViewRaw {
    pub unpack_button: MutPtr<QPushButton>,
    pub path: Arc<RwLock<Vec<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimPackView`.
impl PackedFileAnimPackView {

    /// This function creates a new AnimPack View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::AnimPackPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();

        let file_count_label = QLabel::from_q_string(&qtr("file_count"));
        let file_list_label = QLabel::from_q_string(&qtr("file_paths"));

        let file_count_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{}", data.len())));
        let file_list_data_text = QPlainTextEdit::from_q_string(&QString::from_std_str(format!("{:#?}", data)));

        let mut unpack_button = QPushButton::from_q_string(&qtr("animpack_unpack"));

        layout.add_widget_5a(&mut unpack_button, 0, 1, 1, 1);

        layout.add_widget_5a(file_count_label.into_ptr(), 2, 0, 1, 1);
        layout.add_widget_5a(file_list_label.into_ptr(), 3, 0, 1, 1);

        layout.add_widget_5a(file_count_data_label.into_ptr(), 2, 1, 1, 1);
        layout.add_widget_5a(file_list_data_text.into_ptr(), 3, 1, 1, 1);

        let packed_file_animpack_view_raw = PackedFileAnimPackViewRaw {
            unpack_button: unpack_button.into_ptr(),
            path: packed_file_view.get_path_raw()
        };

        let packed_file_animpack_view_slots = PackedFileAnimPackViewSlots::new(
            packed_file_animpack_view_raw.clone(),
            *app_ui,
            *pack_file_contents_ui,
            *global_search_ui,
        );

        let packed_file_animpack_view = Self {
            unpack_button: atomic_from_mut_ptr(packed_file_animpack_view_raw.unpack_button),
        };

        connections::set_connections(&packed_file_animpack_view, &packed_file_animpack_view_slots);
        packed_file_view.view = ViewType::Internal(View::AnimPack(packed_file_animpack_view));
        packed_file_view.packed_file_type = PackedFileType::AnimPack;

        Ok((TheOneSlot::AnimPack(packed_file_animpack_view_slots), packed_file_info))
    }

    /// This function returns a pointer to the `Unpack` button.
    pub fn get_mut_ptr_unpack_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.unpack_button)
    }
}
