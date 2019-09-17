//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `PackFileContentsSlots`.
!*/

use qt_core::slots::SlotNoArgs;

use crate::packfile_contents_ui::PackFileContentsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub packfile_contents_tree_view_expand_all: SlotNoArgs<'static>,
    pub packfile_contents_tree_view_collapse_all: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsSlots`.
impl PackFileContentsSlots {

	/// This function creates an entire `PackFileContentsSlots` struct.
	pub fn new(pack_file_contents_ui: PackFileContentsUI) -> Self {
        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});


        // And here... we return all the slots.
		Self {
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}