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
Module with the utils for AnimFragments.
!*/

use qt_gui::QListOfQStandardItem;

use cpp_core::CppBox;

use rpfm_lib::schema::Definition;

use crate::packedfile_views::table::utils::get_default_item_from_field;
use super::*;

//----------------------------------------------------------------------------//
//                       Index helpers for tables
//----------------------------------------------------------------------------//

/// This function returns a new default row.
pub unsafe fn get_new_row(table_definition: &Definition) -> CppBox<QListOfQStandardItem> {
    let mut qlist = QListOfQStandardItem::new();
    for field in &table_definition.fields {

        // If the column in question is a bitwise field, split it in as many columns as needed.
        if let Some((_, amount)) = BITWISE_FIELDS.iter().find(|x| x.0 == field.name) {
            for _ in 0..*amount {
                let item = get_item_from_decoded_data(&DecodedData::Boolean(false));
                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
        }
        else {
            let item = get_default_item_from_field(field);
            add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
        }
    }
    qlist
}

/// This function takes care of loading the data into the AnimFragment View.
pub unsafe fn load_data(
    mut integer_1: MutPtr<QLineEdit>,
    mut integer_2: MutPtr<QLineEdit>,
    table_1: MutPtr<QTableView>,
    table_2: MutPtr<QTableView>,
    original_data: &AnimFragment
) -> Result<()> {
    match original_data.get_table_data().get(0) {
        Some(data) => {
            integer_1.set_text(&QString::from_std_str(&data[1].data_to_string()));
            integer_2.set_text(&QString::from_std_str(&data[2].data_to_string()));

            let filter: MutPtr<QSortFilterProxyModel> = table_1.model().static_downcast_mut();
            let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
            if let Some(data) = data.get(0) {
                if let DecodedData::SequenceU32(data) = data {
                    let definition = data.get_definition();
                    for entry in data.get_table_data(){
                        PackedFileAnimFragmentView::load_entry(table_model, &entry, &definition);
                    }
                    PackedFileAnimFragmentView::build_columns(table_1, &data.get_definition());
                }
            }

            let filter: MutPtr<QSortFilterProxyModel> = table_2.model().static_downcast_mut();
            let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
            if let Some(data) = data.get(3) {
                if let DecodedData::SequenceU32(data) = data {
                    let definition = data.get_definition();
                    for entry in data.get_table_data(){
                        PackedFileAnimFragmentView::load_entry(table_model, &entry, &definition);
                    }
                    PackedFileAnimFragmentView::build_columns(table_2, &data.get_definition());
                }
            }

            Ok(())
        }
        None => Err(ErrorKind::Generic.into()),
    }
}
