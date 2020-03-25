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
Module with all the code for managing the view for External PackedFiles.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QPushButton;

use qt_core::QString;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::atomic::AtomicPtr;
use std::path::PathBuf;

use rpfm_error::Result;
use rpfm_lib::packedfile::PackedFileType;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, TheOneSlot, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::mut_ptr_from_atomic;
use self::slots::PackedFileExternalViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an external PackedFile.
pub struct PackedFileExternalView {
    external_path: Arc<PathBuf>,
    stop_watching_button: AtomicPtr<QPushButton>,
    open_folder_button: AtomicPtr<QPushButton>,
}

/// This struct contains the raw version of each pointer in `PackedFileExternalView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileExternalView`.
#[derive(Clone)]
pub struct PackedFileExternalViewRaw {
    pub stop_watching_button: MutPtr<QPushButton>,
    pub open_folder_button: MutPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileExternalView`.
impl PackedFileExternalView {

    /// This function creates a new CaVp8 View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        app_ui: &AppUI,
        packed_file_view: &mut PackedFileView,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<TheOneSlot> {

        CENTRAL_COMMAND.send_message_qt(Command::OpenPackedFileInExternalProgram(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let external_path = match response {
            Response::PathBuf(external_path) => external_path,
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();

        let current_name_label = QLabel::from_q_string(&qtr("external_current_path"));
        let current_name_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", external_path.display())));
        let mut stop_watching_button = QPushButton::from_q_string(&qtr("stop_watching"));
        let mut open_folder_button = QPushButton::from_q_string(&qtr("open_folder"));

        layout.add_widget_5a(current_name_label.into_ptr(), 0, 0, 1, 1);
        layout.add_widget_5a(current_name_data_label.into_ptr(), 0, 1, 1, 1);
        layout.add_widget_5a(&mut stop_watching_button, 1, 0, 1, 1);
        layout.add_widget_5a(&mut open_folder_button, 1, 1, 1, 1);

        let packed_file_external_view_raw = PackedFileExternalViewRaw {
            stop_watching_button: stop_watching_button.into_ptr(),
            open_folder_button: open_folder_button.into_ptr(),
        };

        let packed_file_external_view_slots = PackedFileExternalViewSlots::new(
            packed_file_external_view_raw.clone(),
            *app_ui,
            *pack_file_contents_ui,
            *global_search_ui,
            &packed_file_path
        );

        let packed_file_external_view = Self {
            external_path: Arc::new(external_path),
            stop_watching_button: atomic_from_mut_ptr(packed_file_external_view_raw.stop_watching_button),
            open_folder_button: atomic_from_mut_ptr(packed_file_external_view_raw.open_folder_button),
        };

        connections::set_connections(&packed_file_external_view, &packed_file_external_view_slots);
        packed_file_view.view = ViewType::External(packed_file_external_view);
        packed_file_view.packed_file_type = PackedFileType::Unknown;

        Ok(TheOneSlot::External(packed_file_external_view_slots))
    }

    /// This function returns a copy of the external path of the PackedFile.
    pub fn get_external_path(&self) -> PathBuf {
        self.external_path.to_path_buf()
    }

    /// This function returns a pointer to the `Stop Waching` button.
    pub fn get_mut_ptr_stop_watching_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.stop_watching_button)
    }

    /// This function returns a pointer to the `Open Folder` button.
    pub fn get_mut_ptr_open_folder_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.open_folder_button)
    }
}
