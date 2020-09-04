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
Module with all the code related to the main `DiagnosticsUISlots`.
!*/

use qt_core::{Slot, SlotOfQModelIndex};

use crate::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::packfile_contents_ui::PackFileContentsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the diagnostics panel.
pub struct DiagnosticsUISlots {
    pub diagnostics_open_result: SlotOfQModelIndex<'static>,
    pub toggle_filters_by_level: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUISlots`.
impl DiagnosticsUISlots {

    /// This function creates an entire `DiagnosticsUISlots` struct.
    pub unsafe fn new(
        app_ui: AppUI,
        mut diagnostics_ui: DiagnosticsUI,
        pack_file_contents_ui: PackFileContentsUI
    ) -> Self {

        // What happens when we try to open the file corresponding to one of the matches.
        let diagnostics_open_result = SlotOfQModelIndex::new(move |model_index_filter| {
            DiagnosticsUI::open_match(app_ui, pack_file_contents_ui, model_index_filter.as_ptr());
        });

        let toggle_filters_by_level = Slot::new(move || {
            diagnostics_ui.filter_by_level();
        });

        // And here... we return all the slots.
        Self {
            diagnostics_open_result,
            toggle_filters_by_level,
        }
    }
}
