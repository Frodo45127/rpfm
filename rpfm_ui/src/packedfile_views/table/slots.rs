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
Module with the slots for Table Views.
!*/

use qt_widgets::QWidget;

use qt_core::{SlotOfBool, Slot, SlotOfQString};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rpfm_lib::schema::Definition;

use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::table::PackedFileTableViewRaw;
use crate::utils::show_dialog;

use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Table PackedFile.
pub struct PackedFileTableViewSlots {
    pub filter_line_edit: SlotOfQString<'static>,
    pub toggle_lookups: SlotOfBool<'static>,
    pub save: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableViewSlots`.
impl PackedFileTableViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(
        mut packed_file_view: PackedFileTableViewRaw,
        global_search_ui: GlobalSearchUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        table_definition: &Definition,
        dependency_data: &BTreeMap<i32, Vec<(String, String)>>
    ) -> Self {

        // When we want to filter when changing the pattern to filter with...
        let filter_line_edit = SlotOfQString::new(move |_| {
            packed_file_view.filter_table();
        });

        // When we want to toggle the lookups on and off.
        let toggle_lookups = SlotOfBool::new(clone!(
            table_definition,
            dependency_data => move |_| {
            packed_file_view.toggle_lookups(&table_definition, &dependency_data);
        }));

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = Slot::new(clone!(packed_file_path => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {
                    if let Err(error) = packed_file.save(&packed_file_path.borrow(), global_search_ui, &mut pack_file_contents_ui) {
                        show_dialog(packed_file_view.get_table_view_primary(), error, false);
                    }
                }
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            filter_line_edit,
            toggle_lookups,
            save,
        }
    }
}
