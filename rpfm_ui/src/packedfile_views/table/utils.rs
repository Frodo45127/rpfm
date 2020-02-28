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
In this file are all the utility functions we need for the tables to work.
!*/

use qt_gui::QStandardItemModel;

use cpp_core::MutPtr;

//----------------------------------------------------------------------------//
//                       Undo/Redo helpers for tables
//----------------------------------------------------------------------------//

/// This function is used to update the background or undo table when a change is made in the main table.
pub unsafe fn update_undo_model(model: MutPtr<QStandardItemModel>, mut undo_model: MutPtr<QStandardItemModel>) {
    undo_model.clear();
    for row in 0..model.row_count_0a() {
        for column in 0..model.column_count_0a() {
            let item = &*model.item_2a(row, column);
            undo_model.set_item_3a(row, column, item.clone());
        }
    }
}

/// This function causes the model to use the same colors the undo_model uses. It's for loading the "modified" state
/// of the table when you modify it, close it and open it again.
/// NOTE: This assumes both models are a copy one from another. Any discrepance in their sizes will send the program crashing to hell.
pub unsafe fn load_colors_from_undo_model(model: MutPtr<QStandardItemModel>, undo_model: MutPtr<QStandardItemModel>) {
    for row in 0..undo_model.row_count_0a() {
        for column in 0..undo_model.column_count_0a() {
            let color = &undo_model.item_2a(row, column).background();
            model.item_2a(row, column).set_background(color);
        }
    }
}
