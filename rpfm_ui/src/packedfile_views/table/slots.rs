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

use qt_core::slots::{SlotNoArgs, SlotStringRef};

use std::cell::RefCell;
use std::rc::Rc;

use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::table::PackedFileTableViewRaw;
use crate::QString;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Table PackedFile.
pub struct PackedFileTableViewSlots {
    pub filter_line_edit: SlotStringRef<'static>,
    pub save: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableViewSlots`.
impl PackedFileTableViewSlots {

    /// This function creates the entire slot pack for images.
    pub fn new(packed_file_view: PackedFileTableViewRaw, global_search_ui: GlobalSearchUI, pack_file_contents_ui: PackFileContentsUI, packed_file_path: &Rc<RefCell<Vec<String>>>) -> Self {

        // When we want to filter when changing the pattern to filter with...
        let filter_line_edit = SlotStringRef::new(move |string| {
            packed_file_view.filter_table(Some(QString::from_std_str(string.to_std_string())));
        });

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = SlotNoArgs::new(clone!(packed_file_path => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {
                    packed_file.save(&packed_file_path.borrow(), global_search_ui, &pack_file_contents_ui);
                }
            }
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            filter_line_edit,
            save,
        }
    }
}
