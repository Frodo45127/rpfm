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
Module with the slots for External Views.
!*/

use qt_core::Slot;

use open::that_in_background;

use std::cell::RefCell;
use std::env::temp_dir;
use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a External PackedFile.
pub struct PackedFileExternalViewSlots {
    pub stop_watching: Slot<'static>,
    pub open_folder: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileExternalViewSlots`.
impl PackedFileExternalViewSlots {

    /// This function creates the entire slot pack for External PackedFile Views.
    pub unsafe fn new(
        mut app_ui: AppUI,
        pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI,
        diagnostics_ui: DiagnosticsUI,
        packed_file_path: &Rc<RefCell<Vec<String>>>
    )  -> Self {

        // Slot to close the open view.
        let stop_watching = Slot::new(clone!(
            packed_file_path => move || {
                if let Err(error) = app_ui.purge_that_one_specifically(global_search_ui, pack_file_contents_ui, diagnostics_ui, &packed_file_path.borrow(), true) {
                    show_dialog(app_ui.main_window, error, false);
                }
            }
        ));

        // Slot to open the folder of the current PackedFile in the file manager.
        let open_folder = Slot::new(move || {
            let _ = that_in_background(temp_dir());
        });

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            stop_watching,
            open_folder,
        }
    }
}

