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
Module with all the code for managing the view for External PackedFiles.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QPushButton;

use qt_core::QBox;
use qt_core::QString;
use qt_core::QPtr;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::files::FileType;

use crate::app_ui::AppUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use self::slots::PackedFileExternalViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an external PackedFile.
pub struct PackedFileExternalView {
    external_path: Arc<PathBuf>,
    stop_watching_button: QBox<QPushButton>,
    open_folder_button: QBox<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileExternalView`.
impl PackedFileExternalView {

    /// This function creates a new CaVp8 View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<String>>,
        app_ui: &Rc<AppUI>,
        packed_file_view: &mut PackedFileView,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        external_path: &Path,
    ) {
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        let current_name_label = QLabel::from_q_string_q_widget(&qtr("external_current_path"), packed_file_view.get_mut_widget());
        let current_name_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{:?}", external_path.display())), packed_file_view.get_mut_widget());
        let stop_watching_button = QPushButton::from_q_string_q_widget(&qtr("stop_watching"), packed_file_view.get_mut_widget());
        let open_folder_button = QPushButton::from_q_string_q_widget(&qtr("open_folder"), packed_file_view.get_mut_widget());

        layout.add_widget_5a(&current_name_label, 0, 0, 1, 1);
        layout.add_widget_5a(&current_name_data_label, 0, 1, 1, 1);
        layout.add_widget_5a(&stop_watching_button, 1, 0, 1, 1);
        layout.add_widget_5a(&open_folder_button, 1, 1, 1, 1);

        let packed_file_external_view = Arc::new(PackedFileExternalView {
            external_path: Arc::new(external_path.to_owned()),
            stop_watching_button,
            open_folder_button,
        });

        let packed_file_external_view_slots = PackedFileExternalViewSlots::new(
            &packed_file_external_view,
            app_ui,
            pack_file_contents_ui,
            packed_file_path
        );

        connections::set_connections(&packed_file_external_view, &packed_file_external_view_slots);
        packed_file_view.view = ViewType::External(packed_file_external_view);
        packed_file_view.packed_file_type = FileType::Unknown;
    }

    /// This function returns a copy of the external path of the PackedFile.
    pub fn get_external_path(&self) -> PathBuf {
        self.external_path.to_path_buf()
    }

    /// This function returns a pointer to the `Stop Waching` button.
    pub fn get_mut_ptr_stop_watching_button(&self) -> &QBox<QPushButton> {
        &self.stop_watching_button
    }

    /// This function returns a pointer to the `Open Folder` button.
    pub fn get_mut_ptr_open_folder_button(&self) -> &QBox<QPushButton> {
        &self.open_folder_button
    }
}
