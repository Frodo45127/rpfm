//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the stuff needed for the undo system to work for tables.

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::model_index::ModelIndex;

use std::cmp::Ordering;

//----------------------------------------------------------------------------//
//         Custom Struct for storing automatically ordered ModelIndex
//----------------------------------------------------------------------------//

/// Rust doesn't allow implementing traits for types you don't own, so we have to wrap ModelIndex for ordering it.
/// Don't like it a bit.
pub struct ModelIndexWrapped {
    pub model_index: ModelIndex
}

//----------------------------------------------------------------------------//
//                   Implementation of `ModelIndexWrapped`
//----------------------------------------------------------------------------//

impl ModelIndexWrapped {
    pub fn new(model_index: ModelIndex) -> Self {
        ModelIndexWrapped {
            model_index
        }
    }

    pub fn get(&self) -> &ModelIndex {
        &self.model_index
    }
}

impl Ord for ModelIndexWrapped {
    fn cmp(&self, other: &ModelIndexWrapped) -> Ordering {
        let order = self.model_index.row().cmp(&other.model_index.row());
        if order == Ordering::Equal { self.model_index.column().cmp(&other.model_index.column()) }
        else { order }
    }
}

impl PartialOrd for ModelIndexWrapped {
    fn partial_cmp(&self, other: &ModelIndexWrapped) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ModelIndexWrapped {}
impl PartialEq for ModelIndexWrapped {
    fn eq(&self, other: &ModelIndexWrapped) -> bool {
        self.model_index.row() == other.model_index.row() && self.model_index.column() == other.model_index.column()
    }
}

//----------------------------------------------------------------------------//
//                       Undo/Redo helpers for tables
//----------------------------------------------------------------------------//

/// This function is used to update the background or undo table when a change is made in the main table.
pub fn update_undo_model(model: *mut StandardItemModel, undo_model: *mut StandardItemModel) {
    unsafe {
        undo_model.as_mut().unwrap().clear();
        for row in 0..model.as_mut().unwrap().row_count(()) {
            for column in 0..model.as_mut().unwrap().column_count(()) {
                let item = &*model.as_mut().unwrap().item((row, column));
                undo_model.as_mut().unwrap().set_item((row, column, item.clone()));
            }    
        }
    }
}

/// This function causes the model to use the same colors the undo_model uses. It's for loading the "modified" state
/// of the table when you modify it, close it and open it again.
/// NOTE: This assumes both models are a copy one from another. Any discrepance in their sizes will send the program crashing to hell.
pub fn load_colors_from_undo_model(model: *mut StandardItemModel, undo_model: *mut StandardItemModel) {
    unsafe {
        for row in 0..undo_model.as_mut().unwrap().row_count(()) {
            for column in 0..undo_model.as_mut().unwrap().column_count(()) {
                let color = &undo_model.as_mut().unwrap().item((row, column)).as_mut().unwrap().background();
                model.as_mut().unwrap().item((row, column)).as_mut().unwrap().set_background(color);
            }    
        }
    }
}
