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
Module with the slots for External Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use open::that;

use std::cell::RefCell;
use std::env::temp_dir;
use std::rc::Rc;
use std::sync::Arc;

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, PackedFileExternalView};
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a External PackedFile.
pub struct PackedFileExternalViewSlots {
    pub stop_watching: QBox<SlotNoArgs>,
    pub open_folder: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileExternalViewSlots`.
impl PackedFileExternalViewSlots {

    /// This function creates the entire slot pack for External PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileExternalView>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        packed_file_path: &Rc<RefCell<String>>
    )  -> Self {

        // Slot to close the open view.
        let stop_watching = SlotNoArgs::new(&view.stop_watching_button, clone!(
            app_ui,
            pack_file_contents_ui,
            packed_file_path => move || {
                if let Err(error) = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, &packed_file_path.borrow(), DataSource::PackFile, true) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));

        // Slot to open the folder of the current PackedFile in the file manager.
        let open_folder = SlotNoArgs::new(&view.stop_watching_button, move || {
            let _ = that(temp_dir());
        });

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            stop_watching,
            open_folder,
        }
    }
}

