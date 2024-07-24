//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::SlotNoArgs;
#[cfg(feature = "support_model_renderer")] use rpfm_ui_common::utils::show_dialog;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::packedfile_views::{DataSource, utils::set_modified};
use crate::packfile_contents_ui::PackFileContentsUI;
use super::FileVMDView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an VMD PackedFile.
pub struct FileVMDViewSlots {
    pub modified: QBox<SlotNoArgs>,
    #[cfg(feature = "support_model_renderer")] pub reload_render: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `FileVMDViewSlots`.
impl FileVMDViewSlots {

    /// This function creates the entire slot pack for Texts.
    pub unsafe fn new(view: &Arc<FileVMDView>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Self {

        let modified = SlotNoArgs::new(&view.editor, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Modified VMD File` By Slot");
                if let Some(ref packed_file_path) = view.path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {

                        // TODO: calculate a checksum of the file to also detect when it has gone back to its "unmodified" state.
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        #[cfg(feature = "support_model_renderer")]
        let reload_render = SlotNoArgs::new(&view.editor, clone!(
            view => move || {
                info!("Triggering `Reload VMD Renderer` By Slot");
                if let Err(error) = view.reload_render() {
                    show_dialog(&view.editor, error, false);
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            modified,
            #[cfg(feature = "support_model_renderer")] reload_render
        }
    }
}
