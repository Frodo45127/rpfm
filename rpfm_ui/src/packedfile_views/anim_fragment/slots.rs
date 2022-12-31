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
Module with the slots for AnimFragment Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;
use std::sync::{Arc, atomic::Ordering};

use rpfm_lib::files::ContainerPath;

use crate::app_ui::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::setting_bool;
use crate::utils::show_dialog;
use crate::UI_STATE;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimFragmentPackedFile.
pub struct PackedFileAnimFragmentViewSlots {
    pub delayed_updates: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimFragmentViewSlots`.
impl PackedFileAnimFragmentViewSlots {

    /// This function creates the entire slot pack for AnimPack PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileAnimFragmentView>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>
    )  -> Self {

        let delayed_updates = SlotNoArgs::new(&view.table_view().table_view_ptr(), clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            view => move || {

                // Only save to the backend if both, the save and undo locks are disabled. Otherwise this will cause locks.
                if view.data_source() == DataSource::PackFile && !view.table_view().save_lock().load(Ordering::SeqCst) && !view.table_view().undo_lock().load(Ordering::SeqCst) {
                    if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *view.packed_file_path.read().unwrap() && x.get_data_source() == DataSource::PackFile) {
                        if let Err(error) = packed_file.save(&app_ui, &pack_file_contents_ui) {
                            show_dialog(&view.table_view().table_view_ptr(), error, false);
                        } else if setting_bool("diagnostics_trigger_on_table_edit") && diagnostics_ui.diagnostics_dock_widget().is_visible() {
                            let path_types = vec![ContainerPath::File(view.packed_file_path.read().unwrap().to_string())];
                            DiagnosticsUI::check_on_path(&app_ui, &diagnostics_ui, path_types);
                        }
                    }
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            delayed_updates,
        }
    }
}

