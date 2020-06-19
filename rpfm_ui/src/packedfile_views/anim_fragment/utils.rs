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
