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
Module with all the code for managing the view for CA_VP8 PackedFiles.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QString;
use qt_core::QPtr;

use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use rpfm_lib::files::{video::*, FileType};

use crate::app_ui::AppUI;
use crate::backend::VideoInfo;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;

use self::slots::PackedFileVideoViewSlots;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an CA_VP8 PackedFile.
pub struct PackedFileVideoView {
    format_data_label: QBox<QLabel>,
    version_data_label: QBox<QLabel>,
    codec_four_cc_data_label: QBox<QLabel>,
    width_data_label: QBox<QLabel>,
    height_data_label: QBox<QLabel>,
    num_frames_data_label: QBox<QLabel>,
    framerate_data_label: QBox<QLabel>,

    convert_to_camv_button: QBox<QPushButton>,
    convert_to_ivf_button: QBox<QPushButton>,
    current_format: Arc<Mutex<SupportedFormats>>,
    path: Arc<RwLock<String>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileVideoView`.
impl PackedFileVideoView {

    /// This function creates a new CaVp8 View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        data: &VideoInfo,
    ) {

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        layout.set_contents_margins_4a(4, 4, 4, 4);
        layout.set_spacing(2);

        let format_label = QLabel::from_q_string_q_widget(&qtr("format"), packed_file_view.get_mut_widget());
        let version_label = QLabel::from_q_string_q_widget(&qtr("version"), packed_file_view.get_mut_widget());
        let codec_four_cc_label = QLabel::from_q_string_q_widget(&qtr("codec_four_cc"), packed_file_view.get_mut_widget());
        let width_label = QLabel::from_q_string_q_widget(&qtr("width"), packed_file_view.get_mut_widget());
        let height_label = QLabel::from_q_string_q_widget(&qtr("height"), packed_file_view.get_mut_widget());
        let num_frames_label = QLabel::from_q_string_q_widget(&qtr("num_frames"), packed_file_view.get_mut_widget());
        let framerate_label = QLabel::from_q_string_q_widget(&qtr("framerate"), packed_file_view.get_mut_widget());

        let format_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{:?}.", data.format())), packed_file_view.get_mut_widget());
        let version_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{}.", data.version())), packed_file_view.get_mut_widget());
        let codec_four_cc_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{}.", data.codec_four_cc())), packed_file_view.get_mut_widget());
        let width_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{} px.", data.width())), packed_file_view.get_mut_widget());
        let height_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{} px.", data.height())), packed_file_view.get_mut_widget());
        let num_frames_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{}", data.num_frames())), packed_file_view.get_mut_widget());
        let framerate_data_label = QLabel::from_q_string_q_widget(&QString::from_std_str(format!("{} FPS.", data.framerate())), packed_file_view.get_mut_widget());

        let convert_to_camv_button = QPushButton::from_q_string_q_widget(&qtr("convert_to_camv"), packed_file_view.get_mut_widget());
        let convert_to_ivf_button = QPushButton::from_q_string_q_widget(&qtr("convert_to_ivf"), packed_file_view.get_mut_widget());

        let instructions_label = QLabel::from_q_string_q_widget(&qtr("instructions_ca_vp8"), packed_file_view.get_mut_widget());

        let fill_widget = QWidget::new_1a(packed_file_view.get_mut_widget());
        let fill_widget2 = QWidget::new_1a(packed_file_view.get_mut_widget());

        layout.add_widget_5a(&convert_to_camv_button, 0, 0, 1, 1);
        layout.add_widget_5a(&convert_to_ivf_button, 0, 1, 1, 1);

        layout.add_widget_5a(&format_label, 2, 0, 1, 1);
        layout.add_widget_5a(&version_label, 3, 0, 1, 1);
        layout.add_widget_5a(&codec_four_cc_label, 5, 0, 1, 1);
        layout.add_widget_5a(&width_label, 6, 0, 1, 1);
        layout.add_widget_5a(&height_label, 7, 0, 1, 1);
        layout.add_widget_5a(&num_frames_label, 9, 0, 1, 1);
        layout.add_widget_5a(&framerate_label, 13, 0, 1, 1);

        layout.add_widget_5a(&format_data_label, 2, 1, 1, 1);
        layout.add_widget_5a(&version_data_label, 3, 1, 1, 1);
        layout.add_widget_5a(&codec_four_cc_data_label, 5, 1, 1, 1);
        layout.add_widget_5a(&width_data_label, 6, 1, 1, 1);
        layout.add_widget_5a(&height_data_label, 7, 1, 1, 1);
        layout.add_widget_5a(&num_frames_data_label, 9, 1, 1, 1);
        layout.add_widget_5a(&framerate_data_label, 13, 1, 1, 1);

        layout.add_widget_5a(&instructions_label, 20, 0, 1, 2);

        layout.add_widget_5a(&fill_widget, 99, 0, 1, 2);
        layout.add_widget_5a(&fill_widget2, 0, 2, 1, 1);

        layout.set_row_stretch(99, 99);
        layout.set_column_stretch(2, 99);

        let packed_file_ca_vp8_view = Arc::new(PackedFileVideoView {
            format_data_label,
            version_data_label,
            codec_four_cc_data_label,
            width_data_label,
            height_data_label,
            num_frames_data_label,
            framerate_data_label,
            convert_to_camv_button,
            convert_to_ivf_button,
            current_format: Arc::new(Mutex::new(*data.format())),
            path: packed_file_view.get_path_raw()
        });

        let packed_file_ca_vp8_view_slots = PackedFileVideoViewSlots::new(
            &packed_file_ca_vp8_view,
            app_ui,
            pack_file_contents_ui
        );

        connections::set_connections(&packed_file_ca_vp8_view, &packed_file_ca_vp8_view_slots);
        packed_file_view.view = ViewType::Internal(View::Video(packed_file_ca_vp8_view));
        packed_file_view.packed_file_type = FileType::Video;
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &VideoInfo) {
        self.get_mut_ptr_format_data_label().set_text(&QString::from_std_str(format!("{:?}", data.format())));
        self.get_mut_ptr_version_data_label().set_text(&QString::from_std_str(format!("{:?}", data.version())));
        self.get_mut_ptr_codec_four_cc_data_label().set_text(&QString::from_std_str(format!("{:?}", data.codec_four_cc())));
        self.get_mut_ptr_width_data_label().set_text(&QString::from_std_str(format!("{:?}", data.width())));
        self.get_mut_ptr_height_data_label().set_text(&QString::from_std_str(format!("{:?}", data.height())));
        self.get_mut_ptr_num_frames_data_label().set_text(&QString::from_std_str(format!("{:?}", data.num_frames())));
        self.get_mut_ptr_framerate_data_label().set_text(&QString::from_std_str(format!("{:?}", data.framerate())));
    }

    /// This function returns a copy of the format the video is currently on.
    pub fn get_current_format(&self) -> SupportedFormats {
        *self.current_format.lock().unwrap()
    }

    /// This function sets the current format to the provided one.
    pub fn set_current_format(&self, format: SupportedFormats) {
        *self.current_format.lock().unwrap() = format;
    }

    /// This function returns a pointer to the format_data Label.
    pub fn get_mut_ptr_format_data_label(&self) -> &QBox<QLabel> {
        &self.format_data_label
    }

    /// This function returns a pointer to the version_data Label.
    pub fn get_mut_ptr_version_data_label(&self) -> &QBox<QLabel> {
        &self.version_data_label
    }

    /// This function returns a pointer to the codec_four_cc_data Label.
    pub fn get_mut_ptr_codec_four_cc_data_label(&self) -> &QBox<QLabel> {
        &self.codec_four_cc_data_label
    }

    /// This function returns a pointer to the width_data Label.
    pub fn get_mut_ptr_width_data_label(&self) -> &QBox<QLabel> {
        &self.width_data_label
    }

    /// This function returns a pointer to the height_data Label.
    pub fn get_mut_ptr_height_data_label(&self) -> &QBox<QLabel> {
        &self.height_data_label
    }

    /// This function returns a pointer to the num_frames_data Label.
    pub fn get_mut_ptr_num_frames_data_label(&self) -> &QBox<QLabel> {
        &self.num_frames_data_label
    }

    /// This function returns a pointer to the framerate_data Label.
    pub fn get_mut_ptr_framerate_data_label(&self) -> &QBox<QLabel> {
        &self.framerate_data_label
    }

    /// This function returns a pointer to the `Convert to CAMV` button.
    pub fn get_mut_ptr_convert_to_camv_button(&self) -> &QBox<QPushButton> {
        &self.convert_to_camv_button
    }

    /// This function returns a pointer to the `Convert to IVF` button.
    pub fn get_mut_ptr_convert_to_ivf_button(&self) -> &QBox<QPushButton> {
        &self.convert_to_ivf_button
    }
}
