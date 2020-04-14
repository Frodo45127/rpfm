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
Module with all the code for managing the view for CA_VP8 PackedFiles.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QPushButton;

use qt_core::QString;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicPtr;

use rpfm_error::Result;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::ca_vp8::SupportedFormats;
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
use self::slots::PackedFileCaVp8ViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an CA_VP8 PackedFile.
pub struct PackedFileCaVp8View {
    convert_to_camv_button: AtomicPtr<QPushButton>,
    convert_to_ivf_button: AtomicPtr<QPushButton>,
    current_format: Arc<Mutex<SupportedFormats>>,
}


/// This struct contains the raw version of each pointer in `PackedFileCaVp8View`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileCaVp8View`.
#[derive(Clone)]
pub struct PackedFileCaVp8ViewRaw {
    pub convert_to_camv_button: MutPtr<QPushButton>,
    pub convert_to_ivf_button: MutPtr<QPushButton>,
    pub current_format: Arc<Mutex<SupportedFormats>>,
    pub format_data_label: MutPtr<QLabel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileCaVp8View`.
impl PackedFileCaVp8View {

    /// This function creates a new CaVp8 View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileCaVp8(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::CaVp8PackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();

        let format_label = QLabel::from_q_string(&qtr("format"));
        let version_label = QLabel::from_q_string(&qtr("version"));
        let codec_four_cc_label = QLabel::from_q_string(&qtr("codec_four_cc"));
        let width_label = QLabel::from_q_string(&qtr("width"));
        let height_label = QLabel::from_q_string(&qtr("height"));
        let num_frames_label = QLabel::from_q_string(&qtr("num_frames"));
        let framerate_label = QLabel::from_q_string(&qtr("framerate"));

        let mut format_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_format())));
        let version_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_version())));
        let codec_four_cc_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_ref_codec_four_cc())));
        let width_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_width())));
        let height_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_height())));
        let num_frames_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_num_frames())));
        let framerate_data_label = QLabel::from_q_string(&QString::from_std_str(format!("{:?}", data.get_framerate())));

        let mut convert_to_camv_button = QPushButton::from_q_string(&qtr("convert_to_camv"));
        let mut convert_to_ivf_button = QPushButton::from_q_string(&qtr("convert_to_ivf"));

        layout.add_widget_5a(&mut convert_to_camv_button, 0, 1, 1, 1);
        layout.add_widget_5a(&mut convert_to_ivf_button, 0, 2, 1, 1);

        layout.add_widget_5a(format_label.into_ptr(), 2, 0, 1, 1);
        layout.add_widget_5a(version_label.into_ptr(), 3, 0, 1, 1);
        layout.add_widget_5a(codec_four_cc_label.into_ptr(), 5, 0, 1, 1);
        layout.add_widget_5a(width_label.into_ptr(), 6, 0, 1, 1);
        layout.add_widget_5a(height_label.into_ptr(), 7, 0, 1, 1);
        layout.add_widget_5a(num_frames_label.into_ptr(), 9, 0, 1, 1);
        layout.add_widget_5a(framerate_label.into_ptr(), 13, 0, 1, 1);

        layout.add_widget_5a(&mut format_data_label, 2, 1, 1, 1);
        layout.add_widget_5a(version_data_label.into_ptr(), 3, 1, 1, 1);
        layout.add_widget_5a(codec_four_cc_data_label.into_ptr(), 5, 1, 1, 1);
        layout.add_widget_5a(width_data_label.into_ptr(), 6, 1, 1, 1);
        layout.add_widget_5a(height_data_label.into_ptr(), 7, 1, 1, 1);
        layout.add_widget_5a(num_frames_data_label.into_ptr(), 9, 1, 1, 1);
        layout.add_widget_5a(framerate_data_label.into_ptr(), 13, 1, 1, 1);

        let packed_file_ca_vp8_view_raw = PackedFileCaVp8ViewRaw {
            convert_to_camv_button: convert_to_camv_button.into_ptr(),
            convert_to_ivf_button: convert_to_ivf_button.into_ptr(),
            current_format: Arc::new(Mutex::new(data.get_format())),
            format_data_label: format_data_label.into_ptr(),
        };

        let packed_file_ca_vp8_view_slots = PackedFileCaVp8ViewSlots::new(
            packed_file_ca_vp8_view_raw.clone(),
            *app_ui,
            *pack_file_contents_ui,
            *global_search_ui,
            &packed_file_path
        );

        let packed_file_ca_vp8_view = Self {
            convert_to_camv_button: atomic_from_mut_ptr(packed_file_ca_vp8_view_raw.convert_to_camv_button),
            convert_to_ivf_button: atomic_from_mut_ptr(packed_file_ca_vp8_view_raw.convert_to_ivf_button),
            current_format: packed_file_ca_vp8_view_raw.current_format.clone(),
        };

        connections::set_connections(&packed_file_ca_vp8_view, &packed_file_ca_vp8_view_slots);
        packed_file_view.view = ViewType::Internal(View::CaVp8(packed_file_ca_vp8_view));
        packed_file_view.packed_file_type = PackedFileType::CaVp8;

        Ok((TheOneSlot::CaVp8(packed_file_ca_vp8_view_slots), packed_file_info))
    }

    /// This function returns a copy of the format the video is currently on.
    pub fn get_current_format(&self) -> SupportedFormats {
        self.current_format.lock().unwrap().clone()
    }

    /// This function returns a pointer to the `Convert to CAMV` button.
    pub fn get_mut_ptr_convert_to_camv_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.convert_to_camv_button)
    }

    /// This function returns a pointer to the `Convert to IVF` button.
    pub fn get_mut_ptr_convert_to_ivf_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.convert_to_ivf_button)
    }
}

/// Implementation of `PackedFileCaVp8ViewRaw`.
impl PackedFileCaVp8ViewRaw {

    /// This function sets the current format to the provided one.
    pub fn set_current_format(&mut self, format: SupportedFormats) {
        *self.current_format.lock().unwrap() = format;
    }
}
