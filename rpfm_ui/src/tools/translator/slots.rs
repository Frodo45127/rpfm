//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::QFlags;
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
                    let base_index = before.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.save_from_detailed_view(index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let base_index = after.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.load_to_detailed_view(index.as_ref());
                }
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
                    if row > 0 {
                        let new_row = row - 1;
                        selection_model.clear();

                        for column in 0..ui.table().table_model().column_count_0a() {
                            let new_index = ui.table().table_filter().index_2a(new_row, column);
                            selection_model.select_q_model_index_q_flags_selection_flag(
                                &new_index,
                                QFlags::from(SelectionFlag::Select)
                            );
                        }
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
                    if ui.table().table_filter().row_count_0a() > 0 && row < ui.table().table_filter().row_count_0a() - 1 {
                        let new_row = row + 1;
                        selection_model.clear();

                        for column in 0..ui.table().table_model().column_count_0a() {
                            let new_index = ui.table().table_filter().index_2a(new_row, column);
                            selection_model.select_q_model_index_q_flags_selection_flag(
                                &new_index,
                                QFlags::from(SelectionFlag::Select)
                            );
                        }
                    }
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
            import_from_translated_pack,
        }
    }
}
