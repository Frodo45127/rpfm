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

use qt_widgets::QTableView;
use qt_gui::QStandardItemModel;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::CheckState;

use cpp_core::CppBox;
use cpp_core::MutPtr;
use cpp_core::Ref;

use std::cmp::Ordering;

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

//----------------------------------------------------------------------------//
//                       Index helpers for tables
//----------------------------------------------------------------------------//

/// This function sorts the VISUAL SELECTION. That means, the selection just as you see it on screen.
/// This should be provided with the indexes OF THE VIEW/FILTER, NOT THE MODEL.
pub unsafe fn sort_indexes_visually(indexes_sorted: &mut Vec<Ref<QModelIndex>>, table_view: MutPtr<QTableView>) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    let horizontal_header = table_view.horizontal_header();
    let vertical_header = table_view.vertical_header();
    indexes_sorted.sort_unstable_by(|a, b| {
        if vertical_header.visual_index(a.row()) == vertical_header.visual_index(b.row()) {
            if horizontal_header.visual_index(a.column()) < horizontal_header.visual_index(b.column()) { Ordering::Less }
            else { Ordering::Greater }
        }
        else if vertical_header.visual_index(a.row()) < vertical_header.visual_index(b.row()) { Ordering::Less }
        else { Ordering::Greater }
    });
}

/// This function sorts the MODEL SELECTION. That means, the real selection over the model.
/// This should be provided with the indexes OF THE MODEL, NOT THE VIEW/FILTER.
pub unsafe fn sort_indexes_by_model(indexes_sorted: &mut Vec<Ref<QModelIndex>>) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    indexes_sorted.sort_unstable_by(|a, b| {
        if a.row() == b.row() {
            if a.column() < b.column() { Ordering::Less }
            else { Ordering::Greater }
        }
        else if a.row() < b.row() { Ordering::Less }
        else { Ordering::Greater }
    });
}


/// This function gives you the model's ModelIndexes from the ones from the view/filter.
pub unsafe fn get_real_indexes(indexes_sorted: &[Ref<QModelIndex>], filter_model: MutPtr<QSortFilterProxyModel>) -> Vec<CppBox<QModelIndex>> {
    indexes_sorted.iter().map(|x| filter_model.map_to_source(*x)).collect()
}

/// This function removes indexes with the same row from a list of indexes.
pub unsafe fn dedup_indexes_per_row(indexes: &mut Vec<Ref<QModelIndex>>) {
    let mut rows_done = vec![];
    let mut indexes_to_remove = vec![];
    for (pos, index) in indexes.iter().enumerate() {
        if rows_done.contains(&index.row()) { indexes_to_remove.push(pos); }
        else { rows_done.push(index.row())}
    }

    for index_to_remove in indexes_to_remove.iter().rev() {
        indexes.remove(*index_to_remove);
    }
}
