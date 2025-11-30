//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code for managing audio views.

use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QToolButton;

use qt_core::QPtr;

use anyhow::Result;
use getset::Getters;
use rodio::{OutputStreamBuilder, Sink};

use std::sync::{Arc, RwLock};

use rpfm_lib::files::{audio::Audio, FileType};
use rpfm_ui_common::locale::qtr;

use rpfm_ui_common::utils::{find_widget, load_template};

use crate::packedfile_views::{FileView, View, ViewType};

mod connections;
mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/audio_view.ui";
const VIEW_RELEASE: &str = "ui/audio_view.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct FileAudioView {
    play_button: QPtr<QToolButton>,
    stop_button: QPtr<QToolButton>,

    data: Arc<RwLock<Vec<u8>>>,
    sink: Arc<RwLock<Sink>>,
    stream: rodio::OutputStream,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FileAudioView {

    /// This function creates a new Audio View.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &Audio
    ) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        let warning_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "warning_label")?;
        warning_label.set_text(&qtr("audio_wem_warning_label"));

        let play_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "play_button")?;
        let stop_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "stop_button")?;

        let data = Arc::new(RwLock::new(data.data().to_vec()));

        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream.mixer());

        let view = Arc::new(Self {
            play_button,
            stop_button,

            data,
            sink: Arc::new(RwLock::new(sink)),
            stream,
        });

        let slots = slots::AudioSlots::new(&view);
        connections::set_connections(&view, &slots);

        file_view.file_type = FileType::Audio;
        file_view.view_type = ViewType::Internal(View::Audio(view));

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Audio) {
        *self.data.write().unwrap() = data.data().to_vec();
    }
}
