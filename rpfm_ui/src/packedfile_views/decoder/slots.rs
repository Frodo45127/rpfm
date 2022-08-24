//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the slots for Decoder Views.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::{SlotOfBool, SlotOfInt, SlotOfQItemSelectionQItemSelection, SlotNoArgs, SlotOfQModelIndexQModelIndexQVectorOfInt};

use cpp_core::Ref;

use bincode::deserialize;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_error::ErrorKind;

use rpfm_lib::packedfile::table::animtable::AnimTable;
use rpfm_lib::packedfile::table::anim_fragment::AnimFragment;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::loc::Loc;
use rpfm_lib::packedfile::table::matched_combat::MatchedCombat;
use rpfm_lib::packedfile::table::Table;
use rpfm_lib::packedfile::PackedFileType;
use crate::SCHEMA;
use rpfm_lib::schema::{Definition, FieldType};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::show_dialog;
use crate::utils::show_debug_dialog;
use crate::UI_STATE;

use super::get_definition;
use super::get_header_size;
use super::PackedFileDecoderView;
use super::PackedFileDecoderMutableData;
use super::DECODER_EXTENSION;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Text PackedFile.
pub struct PackedFileDecoderViewSlots {
    pub hex_view_scroll_sync: QBox<SlotOfInt>,
    pub hex_view_selection_raw_sync: QBox<SlotNoArgs>,
    pub hex_view_selection_decoded_sync: QBox<SlotNoArgs>,

    pub use_this_bool: QBox<SlotNoArgs>,
    pub use_this_f32: QBox<SlotNoArgs>,
    pub use_this_f64: QBox<SlotNoArgs>,
    pub use_this_i16: QBox<SlotNoArgs>,
    pub use_this_i32: QBox<SlotNoArgs>,
    pub use_this_i64: QBox<SlotNoArgs>,
    pub use_this_colour_rgb: QBox<SlotNoArgs>,
    pub use_this_string_u8: QBox<SlotNoArgs>,
    pub use_this_string_u16: QBox<SlotNoArgs>,
    pub use_this_optional_string_u8: QBox<SlotNoArgs>,
    pub use_this_optional_string_u16: QBox<SlotNoArgs>,
    pub use_this_sequence_u32: QBox<SlotNoArgs>,

    pub table_change_field_type: QBox<SlotOfQModelIndexQModelIndexQVectorOfInt>,

    pub table_view_context_menu_move_up: QBox<SlotOfBool>,
    pub table_view_context_menu_move_down: QBox<SlotOfBool>,
    pub table_view_context_menu_move_left: QBox<SlotOfBool>,
    pub table_view_context_menu_move_right: QBox<SlotOfBool>,
    pub table_view_context_menu_delete: QBox<SlotOfBool>,

    pub table_view_context_menu: QBox<SlotOfQPoint>,
    pub table_view_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,

    pub table_view_versions_context_menu: QBox<SlotOfQPoint>,
    pub table_view_versions_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,

    pub table_view_old_versions_context_menu_load: QBox<SlotOfBool>,
    pub table_view_old_versions_context_menu_delete: QBox<SlotOfBool>,

    pub import_from_assembly_kit: QBox<SlotNoArgs>,
    pub test_definition: QBox<SlotNoArgs>,
    pub remove_all_fields: QBox<SlotNoArgs>,
    pub save_definition: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileDecoderViewSlots`.
impl PackedFileDecoderViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(
        view: &Arc<PackedFileDecoderView>,
        mutable_data: PackedFileDecoderMutableData,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Self {

        // Slot to keep scroll in views in sync.
        let hex_view_scroll_sync = SlotOfInt::new(&view.table_view, clone!(
            mut view => move |value| {
            view.hex_view_index.vertical_scroll_bar().set_value(value);
            view.hex_view_raw.vertical_scroll_bar().set_value(value);
            view.hex_view_decoded.vertical_scroll_bar().set_value(value);
        }));

        // Slot to keep selection in views in sync.
        let hex_view_selection_raw_sync = SlotNoArgs::new(&view.table_view, clone!(
            mut view => move || {
            view.hex_selection_sync(true);
        }));

        // Slot to keep selection in views in sync.
        let hex_view_selection_decoded_sync = SlotNoArgs::new(&view.table_view, clone!(
            mut view => move || {
            view.hex_selection_sync(false);
        }));

        // Slot to use a boolean value.
        let use_this_bool = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::Boolean, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a float value.
        let use_this_f32 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::F32, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a long float value.
        let use_this_f64 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::F64, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use an integer value.
        let use_this_i16 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::I16, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use an integer value.
        let use_this_i32 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::I32, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a long integer value.
        let use_this_i64 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::I64, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a 4byte colour value.
        let use_this_colour_rgb = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::ColourRGB, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a string u8 value.
        let use_this_string_u8 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::StringU8, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use a string u16 value.
        let use_this_string_u16 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::StringU16, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use an optional string u8 value.
        let use_this_optional_string_u8 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::OptionalStringU8, &mut mutable_data.index.lock().unwrap());
        }));

        // Slot to use an optional string u16 value.
        let use_this_optional_string_u16 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::OptionalStringU16, &mut mutable_data.index.lock().unwrap());
        }));


        // Slot to use a sequence u32 value.
        let use_this_sequence_u32 = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
            let _ = view.use_this(FieldType::SequenceU32(Box::new(Definition::new(-100))), &mut mutable_data.index.lock().unwrap());
        }));

        // Slot for when we change the Type of the selected field in the table.
        let table_change_field_type = SlotOfQModelIndexQModelIndexQVectorOfInt::new(&view.table_view, clone!(
            mutable_data,
            view => move |initial_model_index,final_model_index,_| {
                if initial_model_index.column() == 2 && final_model_index.column() == 2 {
                    let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                }
            }
        ));

        // Slots for the "Move up" contextual action of the TableView.
        let table_view_context_menu_move_up = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view.selection_model().selection();
                let indexes = selection.indexes();
                let mut rows = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();

                rows.sort_by_key(|x| x.row());
                rows.dedup_by_key(|x| x.row());

                for child in rows {
                    let parent = child.parent();
                    if parent.is_valid() {
                        if child.row() == 0 { continue; }
                        else {
                            let row_data = view.table_model.item_from_index(&parent).take_row(child.row() - 1);
                            view.table_model.item_from_index(&parent).insert_row_int_q_list_of_q_standard_item(child.row(), &row_data);
                        }

                    }
                    else if child.row() == 0 { continue; }
                    else {
                        let row_data = view.table_model.take_row(child.row() - 1);
                        view.table_model.insert_row_int_q_list_of_q_standard_item(child.row(), &row_data);
                    }
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                view.table_view.expand_all();
            }
        ));

        // Slots for the "Move down" contextual action of the TableView.
        let table_view_context_menu_move_down = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view.selection_model().selection();
                let indexes = selection.indexes();
                let mut rows = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();

                rows.sort_by_key(|x| x.row());
                rows.dedup_by_key(|x| x.row());

                rows.reverse();

                for child in rows {
                    let parent = child.parent();
                    if parent.is_valid() {

                        let row_count = view.table_model.item_from_index(&parent).row_count();
                        if child.row() == (row_count - 1) { continue; }
                        else {
                            let row_data = view.table_model.item_from_index(&parent).take_row(child.row() + 1);
                            view.table_model.item_from_index(&parent).insert_row_int_q_list_of_q_standard_item(child.row(), &row_data);
                        }

                    }
                    else {
                        let row_count = view.table_model.row_count_0a();
                        if child.row() == (row_count - 1) { continue; }
                        else {
                            let row_data = view.table_model.take_row(child.row() + 1);
                            view.table_model.insert_row_int_q_list_of_q_standard_item(child.row(), &row_data);
                        }
                    }
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                view.table_view.expand_all();
            }
        ));

        // Slots for the "Move left" contextual action of the TableView.
        let table_view_context_menu_move_left = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view.selection_model().selection();
                let indexes = selection.indexes();
                let mut rows = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();

                rows.sort_by_key(|x| x.row());
                rows.dedup_by_key(|x| x.row());

                for child in rows {

                    // Only move left if we're not yet in the top level.
                    let parent = child.parent();
                    if parent.is_valid() {
                        let row_data = view.table_model.item_from_index(&parent).take_row(child.row());
                        let big_parent = parent.parent();
                        if big_parent.is_valid() {
                            view.table_model.item_from_index(&parent.parent()).insert_row_int_q_list_of_q_standard_item(parent.row() + 1, &row_data);

                        }
                        else {
                            view.table_model.insert_row_int_q_list_of_q_standard_item(parent.row() + 1, &row_data);
                        }
                    }
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                view.table_view.expand_all();
            }
        ));

        // Slots for the "Move right" contextual action of the TableView.
        let table_view_context_menu_move_right = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view.selection_model().selection();
                let indexes = selection.indexes();
                let mut rows = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();

                rows.sort_by_key(|x| x.row());
                rows.dedup_by_key(|x| x.row());

                for child in rows {

                    // Only move right if the one above is in a lower level.
                    let parent = child.parent();
                    if child.row() > 0 {
                        let item = if parent.is_valid() {
                            view.table_model.item_from_index(&parent).child_1a(child.row() - 1)
                        }
                        else {
                            view.table_model.item_1a(child.row() - 1)
                        };

                        if item.has_children() || view.table_model.item_from_index(&item.index().sibling_at_column(2)).text().to_std_string() == "SequenceU32" {
                            let row_data = if parent.is_valid() {
                                view.table_model.item_from_index(&parent).take_row(child.row())
                            }
                            else {
                                view.table_model.take_row(child.row())
                            };
                            item.append_row_q_list_of_q_standard_item(&row_data);
                        }
                    }
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                view.table_view.expand_all();
            }
        ));

        // Slots for the "Delete" contextual action of the TableView.
        let table_view_context_menu_delete = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view.selection_model().selection();
                let indexes = selection.indexes();
                let mut rows = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();

                rows.sort_by_key(|x| x.row());
                rows.dedup_by_key(|x| x.row());
                rows.reverse();

                for child in rows {

                    // Only move right if the one above is in a lower level.
                    let parent = child.parent();
                    if child.parent().is_valid() {
                        view.table_model.item_from_index(&parent).child_1a(child.row());
                    }
                    else {
                        view.table_model.remove_row_1a(child.row());
                    }
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
            }
        ));

        // Slot to show the Contextual Menu for the fields table view.
        let table_view_context_menu = SlotOfQPoint::new(&view.table_view, clone!(
            mut view => move |_| {
            view.table_view_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // Slot to enable/disable contextual actions depending on the selected item.
        let table_view_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(&view.table_view, clone!(
            mut view => move |selection, _| {

                // If there is something selected...
                if !selection.indexes().is_empty() {
                    view.table_view_context_menu_move_up.set_enabled(true);
                    view.table_view_context_menu_move_down.set_enabled(true);
                    view.table_view_context_menu_move_left.set_enabled(true);
                    view.table_view_context_menu_move_right.set_enabled(true);
                    view.table_view_context_menu_delete.set_enabled(true);
                }

                // Otherwise, disable everything.
                else {
                    view.table_view_context_menu_move_up.set_enabled(false);
                    view.table_view_context_menu_move_down.set_enabled(false);
                    view.table_view_context_menu_move_left.set_enabled(false);
                    view.table_view_context_menu_move_right.set_enabled(false);
                    view.table_view_context_menu_delete.set_enabled(false);
                }
            }
        ));

        // Slot to show the Contextual Menu for the Other Versions table view.
        let table_view_versions_context_menu = SlotOfQPoint::new(&view.table_view, clone!(
            mut view => move |_| {
            view.table_view_old_versions_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // Slot to enable/disable contextual actions depending on the selected item.
        let table_view_versions_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(&view.table_view, clone!(
            mut view => move |selection, _| {

                // If there is something selected...
                if !selection.indexes().is_empty() {
                    view.table_view_old_versions_context_menu_load.set_enabled(true);
                    view.table_view_old_versions_context_menu_delete.set_enabled(true);
                }

                // Otherwise, disable everything.
                else {
                    view.table_view_old_versions_context_menu_load.set_enabled(false);
                    view.table_view_old_versions_context_menu_delete.set_enabled(false);
                }
            }
        ));

        // Slots for the "Load" contextual action of the Version's TableView.
        let table_view_old_versions_context_menu_load = SlotOfBool::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move |_| {

                let selection = view.table_view_old_versions.selection_model().selection();
                let indexes = selection.indexes();
                if indexes.count_0a() == 1 {
                    let model_index = indexes.at(0);
                    let version = view.table_model_old_versions.item_from_index(model_index).text().to_std_string().parse::<i32>().unwrap();
                    if view.get_mut_ptr_packed_file_info_version_decoded_spinbox().is_enabled() {
                        view.get_mut_ptr_packed_file_info_version_decoded_spinbox().set_value(version);
                    }

                    // Get the new definition.
                    let definition = get_definition(
                        view.packed_file_type,
                        &view.packed_file_path,
                        &view.packed_file_data,
                        Some(version)
                    ).unwrap();

                    // Reset the definition we have.
                    view.table_model.clear();
                    *mutable_data.index.lock().unwrap() = get_header_size(view.packed_file_type, &view.packed_file_data).unwrap();

                    // Update the decoder view.
                    let _ = view.update_view(definition.get_ref_fields(), true, &mut mutable_data.index.lock().unwrap());
                }

                let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
            }
        ));

        // Slots for the "Delete" contextual action of the Version's TableView.
        let table_view_old_versions_context_menu_delete = SlotOfBool::new(&view.table_view, clone!(
            mut view => move |_| {

                let selection = view.table_view_old_versions.selection_model().selection();
                let indexes = selection.indexes();
                if indexes.count_0a() == 1 {
                    let model_index = indexes.at(0);
                    let version = view.table_model_old_versions.item_from_index(model_index).text().to_std_string().parse::<i32>().unwrap();

                    if let Some(ref mut schema) = *SCHEMA.write().unwrap() {
                        let versioned_file = match view.packed_file_type {
                            PackedFileType::AnimTable => schema.get_ref_mut_versioned_file_animtable(),
                            PackedFileType::AnimFragment => schema.get_ref_mut_versioned_file_anim_fragment(),
                            PackedFileType::DB => schema.get_ref_mut_versioned_file_db(&view.packed_file_path[1]),
                            PackedFileType::Loc => schema.get_ref_mut_versioned_file_loc(),
                            PackedFileType::MatchedCombat => schema.get_ref_mut_versioned_file_matched_combat(),
                            _ => unimplemented!(),
                        }.unwrap();

                        versioned_file.remove_version(version);
                    }
                    view.load_versions_list();
                }
            }
        ));

        // Slot for the "Import from Assembly Kit" button.
        let import_from_assembly_kit = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
                match view.import_from_assembly_kit() {
                    Ok(field_list) => {
                        println!("Amount of possible definitions: {}.", field_list.len());
                        if let Some(field_list) = field_list.get(0) {

                            // If it worked, update the decoder view.
                            view.table_model.clear();
                            *mutable_data.index.lock().unwrap() = get_header_size(view.packed_file_type, &view.packed_file_data).unwrap();
                            let _ = view.update_view(field_list, true, &mut mutable_data.index.lock().unwrap());
                            let _ = view.update_rows_decoded(&mut mutable_data.index.lock().unwrap(), None, None);
                        }

                        else {
                            show_dialog(&view.table_view, "No valid definitions found.", false)
                        }
                    }

                    // If it failed, tell us why.
                    Err(error) => show_dialog(&view.table_view, error, false),
                }
            }
        ));

        // Slot for the "Test Definition" button.
        let test_definition = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            view => move || {
                let schema = view.add_definition_to_schema();

                match view.packed_file_type {

                    PackedFileType::AnimTable => match AnimTable::read(&view.packed_file_data, &schema, true) {
                        Ok(_) => show_dialog(&view.table_view, "Seems ok.", true),
                        Err(error) => {
                            if let ErrorKind::TableIncompleteError(error, data) = error.kind() {
                                let data: Table = deserialize(data).unwrap();
                                show_debug_dialog(&app_ui.main_window, &format!("{}\n{:#?}", error, data.get_table_data()));
                            } else {
                                show_dialog(&app_ui.main_window, error, true);
                            }
                        }
                    }

                    PackedFileType::AnimFragment => match AnimFragment::read(&view.packed_file_data, &schema, true) {
                        Ok(_) => show_dialog(&view.table_view, "Seems ok.", true),
                        Err(error) => {
                            if let ErrorKind::TableIncompleteError(error, data) = error.kind() {
                                let data: Table = deserialize(data).unwrap();
                                show_debug_dialog(&app_ui.main_window, &format!("{}\n{:#?}", error, data.get_table_data()));
                            } else {
                                show_dialog(&app_ui.main_window, error, true);
                            }
                        }
                    }

                    PackedFileType::DB => match DB::read(&view.packed_file_data, &view.packed_file_path[1], &schema, true) {
                        Ok(_) => show_dialog(&view.table_view, "Seems ok.", true),
                        Err(error) => {
                            if let ErrorKind::TableIncompleteError(error, data) = error.kind() {
                                let data: Table = deserialize(data).unwrap();
                                show_debug_dialog(&app_ui.main_window, &format!("{}\n{:#?}", error, data.get_table_data()));
                            } else {
                                show_dialog(&app_ui.main_window, error, true);
                            }
                        }
                    }

                    PackedFileType::Loc => match Loc::read(&view.packed_file_data, &schema, true) {
                        Ok(_) => show_dialog(&view.table_view, "Seems ok.", true),
                        Err(error) => {
                            if let ErrorKind::TableIncompleteError(error, data) = error.kind() {
                                let data: Table = deserialize(data).unwrap();
                                show_debug_dialog(&app_ui.main_window, &format!("{}\n{:#?}", error, data.get_table_data()));
                            } else {
                                show_dialog(&app_ui.main_window, error, true);
                            }
                        }
                    }

                   PackedFileType::MatchedCombat => match MatchedCombat::read(&view.packed_file_data, &schema, true) {
                        Ok(_) => show_dialog(&view.table_view, "Seems ok.", true),
                        Err(error) => {
                            if let ErrorKind::TableIncompleteError(error, data) = error.kind() {
                                let data: Table = deserialize(data).unwrap();
                                show_debug_dialog(&app_ui.main_window, &format!("{}\n{:#?}", error, data.get_table_data()));
                            } else {
                                show_dialog(&app_ui.main_window, error, true);
                            }
                        }
                    }

                    _ => unimplemented!()
                }
            }
        ));

        // Slot for the "Kill them all!" button.
        let remove_all_fields = SlotNoArgs::new(&view.table_view, clone!(
            mut mutable_data,
            mut view => move || {
                view.table_model.clear();
                *mutable_data.index.lock().unwrap() = get_header_size(view.packed_file_type, &view.packed_file_data).unwrap();
                let _ = view.update_view(&[], true, &mut mutable_data.index.lock().unwrap());
            }
        ));

        // Slot for the "Finish it!" button.
        let save_definition = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let schema = view.add_definition_to_schema();

                // Save and close all PackedFiles that use our definition.
                let mut packed_files_to_save = vec![];
                for open_path in UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).map(|x| x.get_ref_path()) {
                    if open_path.len() > 2 &&
                        open_path[0] == view.packed_file_path[0] &&
                        open_path[1] == view.packed_file_path[1] &&
                        !open_path[2].ends_with(DECODER_EXTENSION) {
                        packed_files_to_save.push(open_path.to_vec());
                    }
                }

                for path in &packed_files_to_save {
                    if let Err(error) = AppUI::purge_that_one_specifically(
                        &app_ui,
                        &pack_file_contents_ui,
                        path,
                        DataSource::PackFile,
                        true,
                    ) {
                        show_dialog(&view.table_view, error, false);
                    }
                }

                let _ = CENTRAL_COMMAND.send_background(Command::CleanCache(packed_files_to_save));
                let receiver = CENTRAL_COMMAND.send_background(Command::SaveSchema(schema));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => show_dialog(&view.table_view, "Schema successfully saved.", true),
                    Response::Error(error) => show_dialog(&view.table_view, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                view.load_versions_list();
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            hex_view_scroll_sync,
            hex_view_selection_raw_sync,
            hex_view_selection_decoded_sync,

            use_this_bool,
            use_this_f32,
            use_this_f64,
            use_this_i16,
            use_this_i32,
            use_this_i64,
            use_this_colour_rgb,
            use_this_string_u8,
            use_this_string_u16,
            use_this_optional_string_u8,
            use_this_optional_string_u16,
            use_this_sequence_u32,

            table_change_field_type,

            table_view_context_menu_move_up,
            table_view_context_menu_move_down,
            table_view_context_menu_move_left,
            table_view_context_menu_move_right,
            table_view_context_menu_delete,

            table_view_context_menu,
            table_view_context_menu_enabler,

            table_view_versions_context_menu,
            table_view_versions_context_menu_enabler,

            table_view_old_versions_context_menu_load,
            table_view_old_versions_context_menu_delete,

            import_from_assembly_kit,
            test_definition,
            remove_all_fields,
            save_definition,
        }
    }
}
