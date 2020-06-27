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

use qt_widgets::SlotOfQPoint;
use qt_widgets::SlotOfIntSortOrder;

use qt_gui::QCursor;

use qt_core::QVariant;
use qt_core::QString;
use qt_core::QModelIndex;
use qt_core::{Slot, SlotOfQItemSelectionQItemSelection, SlotOfQModelIndex};

use cpp_core::Ref;

use rpfm_lib::packedfile::table::Table;

use crate::packedfile_views::table::subtable::SubTableView;
use crate::packedfile_views::table::ITEM_IS_SEQUENCE;
use crate::packedfile_views::table::ITEM_SEQUENCE_DATA;
use crate::packedfile_views::table::utils::delete_rows;
use crate::packedfile_views::table::utils::sort_column;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Table PackedFile.
pub struct SubTableViewSlots {
    pub sort_order_column_changed: SlotOfIntSortOrder<'static>,
    pub show_context_menu: SlotOfQPoint<'static>,
    pub context_menu_enabler: SlotOfQItemSelectionQItemSelection<'static>,
    pub add_rows: Slot<'static>,
    pub delete_rows: Slot<'static>,
    pub copy: Slot<'static>,

    pub open_subtable: SlotOfQModelIndex<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `SubTableViewSlots`.
impl SubTableViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(packed_file_view: &SubTableView) -> Self {

        let sort_order_column_changed = SlotOfIntSortOrder::new(clone!(
            packed_file_view => move |column, _| {
                sort_column(packed_file_view.table_view, column, packed_file_view.column_sort_state.clone());
            }
        ));

        // When we want to show the context menu.
        let show_context_menu = SlotOfQPoint::new(clone!(
            mut packed_file_view => move |_| {
            packed_file_view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(clone!(
            mut packed_file_view => move |_,_| {
            packed_file_view.context_menu_update();
        }));

        // When you want to append a row to the table...
        let add_rows = Slot::new(clone!(
            mut packed_file_view => move || {
                packed_file_view.append_rows(false);
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = Slot::new(clone!(
            mut packed_file_view => move || {

                // Get all the selected rows.
                let selection = packed_file_view.table_view.selection_model().selection();
                let indexes = packed_file_view.table_filter.map_selection_to_source(&selection).indexes();
                let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

                // Dedup the list and reverse it.
                rows_to_delete.sort();
                rows_to_delete.dedup();
                rows_to_delete.reverse();
                delete_rows(packed_file_view.table_model, &rows_to_delete);
            }
        ));

        // When you want to copy one or more cells.
        let copy = Slot::new(clone!(
            packed_file_view => move || {
            packed_file_view.copy_selection();
        }));

        let open_subtable = SlotOfQModelIndex::new(clone!(
            mut packed_file_view => move |model_index| {
                if model_index.data_1a(ITEM_IS_SEQUENCE).to_bool() {
                    let data = model_index.data_1a(ITEM_SEQUENCE_DATA).to_string().to_std_string();
                    let table: Table = serde_json::from_str(&data).unwrap();
                    if let Some(new_data) = SubTableView::show(packed_file_view.table_view, &table) {
                        packed_file_view.table_filter.set_data_3a(
                            model_index,
                            &QVariant::from_q_string(&QString::from_std_str(new_data)),
                            ITEM_SEQUENCE_DATA
                        );
                    }
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            sort_order_column_changed,
            show_context_menu,
            context_menu_enabler,
            add_rows,
            delete_rows,
            copy,

            open_subtable,
        }
    }
}
