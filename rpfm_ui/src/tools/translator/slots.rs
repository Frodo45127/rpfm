//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::QItemSelection;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::rc::Rc;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct ToolTranslatorSlots {
    load_data_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    move_selection_up: QBox<SlotNoArgs>,
    move_selection_down: QBox<SlotNoArgs>,
    copy_from_source: QBox<SlotNoArgs>,
    import_from_translated_pack: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ToolTranslatorSlots {

    /// This function creates a new `ToolTranslatorSlots`.
    pub unsafe fn new(ui: &Rc<ToolTranslator>) -> Self {

        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, before| {
                info!("Triggering 'load_data_to_detailed_view' for Translator.");

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    ui.save_from_detailed_view();
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let base_index = after.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.load_to_detailed_view(index.as_ref());
                }

                ui.table().filters()[0].start_delayed_updates_timer();
            }
        ));

        let move_selection_up = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'move_selection_up' for Translator.");

                let selection_model = ui.table().table_view().selection_model();
                let selection = selection_model.selection();
                let indexes = selection.indexes();
                if indexes.count_0a() > 0 {
                    let index = indexes.at(0);
                    let row = index.row();

                    // Only do something if we're not in the top row.
                    if row > 0 {

                        // This triggers a load of the editing file.
                        let new_row = ui.current_row.read().unwrap().unwrap_or(row).checked_sub(1).unwrap_or_default();
                        let column_count = ui.table().table_model().column_count_0a();
                        let start_index = ui.table().table_filter().index_2a(new_row, 0);
                        let end_index = ui.table().table_filter().index_2a(new_row, column_count - 1);
                        let new_selection = QItemSelection::new_2a(&start_index, &end_index);

                        // This triggers a save of the editing item.
                        selection_model.select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                        selection_model.clear();
                        selection_model.select_q_item_selection_q_flags_selection_flag(&new_selection, SelectionFlag::Toggle.into());
                    }
                }
            }
        ));

        let move_selection_down = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'move_selection_down' for Translator.");

                let selection_model = ui.table().table_view().selection_model();
                let selection = selection_model.selection();
                let indexes = selection.indexes();
                if indexes.count_0a() > 0 {
                    let index = indexes.at(0);
                    let row = index.row();
                    let row_count = ui.table().table_filter().row_count_0a();
                    if row_count > 0 && row < row_count - 1 {

                        // This triggers a load of the editing file.
                        let new_row = ui.current_row.read().unwrap().unwrap_or(row) + 1;
                        let column_count = ui.table().table_model().column_count_0a();
                        let start_index = ui.table().table_filter().index_2a(new_row, 0);
                        let end_index = ui.table().table_filter().index_2a(new_row, column_count - 1);
                        let new_selection = QItemSelection::new_2a(&start_index, &end_index);

                        // This triggers a save of the editing item.
                        selection_model.select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                        selection_model.clear();
                        selection_model.select_q_item_selection_q_flags_selection_flag(&new_selection, SelectionFlag::Toggle.into());
                    }
                }
            }
        ));

        let copy_from_source = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'copy_from_source' for Translator.");

                let selection_model = ui.table().table_view().selection_model();
                let selection = selection_model.selection();
                let indexes = selection.indexes();
                if indexes.count_0a() > 0 {
                    let index = indexes.at(0);
                    let mapped_index = ui.table().table_filter().map_to_source(index);

                    // This one is just copy the value, mark it as translated, then trigger the move selection down.
                    let original_value_item = ui.table.table_model().item_2a(mapped_index.row(), 3);
                    let original_value = original_value_item.text().to_std_string();
                    let trans_value_item = ui.table.table_model().item_2a(mapped_index.row(), 4);
                    trans_value_item.set_text(&QString::from_std_str(original_value));

                    ui.table.table_model().item_2a(mapped_index.row(), 1).set_check_state(CheckState::Unchecked);

                    ui.action_move_down().triggered();
                }
            }
        ));

        let import_from_translated_pack = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'import_from_translated_pack' for Translator.");

                if let Err(error) = ui.import_from_another_pack() {
                    show_dialog(ui.tool.main_widget(), error, false);
                }
            }
        ));

        ToolTranslatorSlots {
            load_data_to_detailed_view,
            move_selection_up,
            move_selection_down,
            copy_from_source,
            import_from_translated_pack,
        }
    }
}
