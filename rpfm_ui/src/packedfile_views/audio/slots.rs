//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Module with the slots for Audio Views.

use qt_core::QBox;
use qt_core::SlotNoArgs;

use getset::Getters;
use rodio::{Decoder, Sink};

use std::io::Cursor;
use std::sync::Arc;

use rpfm_ui_common::clone;

use super::FileAudioView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a Audio view.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct AudioSlots {
    play: QBox<SlotNoArgs>,
    stop: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AudioSlots {
    pub unsafe fn new(view: &Arc<FileAudioView>)  -> Self {

        let play = SlotNoArgs::new(view.play_button(), clone!(
            view => move || {

                // We replace the sink because "stop" means "stop forever with no way to restart it".
                // This also drops any previous sink, avoiding repeated sounds.
                if let Ok(sink) = Sink::try_new(view.handle()) {
                    *view.sink().write().unwrap() = sink;

                    let cursor = Cursor::new(view.data().read().unwrap().to_vec());
                    if let Ok(decoder) = Decoder::new(cursor) {
                        view.sink().read().unwrap().append(decoder);
                        view.sink().read().unwrap().play();
                    }
                }
            }
        ));

        let stop = SlotNoArgs::new(view.stop_button(), clone!(
            view => move || {
                view.sink().read().unwrap().stop();
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            play,
            stop,
        }
    }
}
