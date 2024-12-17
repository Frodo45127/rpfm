//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::AcceptMode;
use qt_widgets::SlotOfIntSortOrder;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QCursor;
use qt_gui::SlotOfQStandardItem;

use qt_core::QBox;
use qt_core::QItemSelection;
use qt_core::QSignalBlocker;
use qt_core::{SlotOfBool, SlotOfInt, SlotNoArgs, SlotOfQItemSelectionQItemSelection, SlotOfQModelIndex, SlotOfQString};

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, atomic::Ordering, RwLock};

use rpfm_lib::files::{ContainerPath, RFileDecoded};
use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::utils::set_modified;
use crate::references_ui::ReferencesUI;
use crate::utils::{log_to_status_bar, show_dialog};
use crate::UI_STATE;
use super::utils::*;
use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a Table PackedFile.
pub struct TableViewSlots {
    pub delayed_updates: QBox<SlotNoArgs>,
    pub sort_order_column_changed: QBox<SlotOfIntSortOrder>,
    pub show_context_menu: QBox<SlotOfQPoint>,
    pub context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    pub item_changed: QBox<SlotOfQStandardItem>,
    pub add_rows: QBox<SlotNoArgs>,
    pub insert_rows: QBox<SlotNoArgs>,
    pub delete_rows: QBox<SlotNoArgs>,
    pub delete_rows_not_in_filter: QBox<SlotNoArgs>,
    pub clone_and_append: QBox<SlotNoArgs>,
    pub clone_and_insert: QBox<SlotNoArgs>,
    pub copy: QBox<SlotNoArgs>,
    pub copy_as_lua_table: QBox<SlotNoArgs>,
    pub copy_to_filter_value: QBox<SlotNoArgs>,
    pub paste: QBox<SlotNoArgs>,
    pub paste_as_new_row: QBox<SlotNoArgs>,
    pub invert_selection: QBox<SlotNoArgs>,
    pub reset_selection: QBox<SlotNoArgs>,
    pub rewrite_selection: QBox<SlotNoArgs>,
    pub revert_value: QBox<SlotNoArgs>,
    pub generate_ids: QBox<SlotNoArgs>,
    pub undo: QBox<SlotNoArgs>,
    pub redo: QBox<SlotNoArgs>,
    pub import_tsv: QBox<SlotOfBool>,
    pub export_tsv: QBox<SlotOfBool>,
    pub smart_delete: QBox<SlotNoArgs>,
    pub resize_columns: QBox<SlotNoArgs>,
    pub sidebar: QBox<SlotOfBool>,
    pub search: QBox<SlotOfBool>,
    pub cascade_edition: QBox<SlotNoArgs>,
    pub patch_column: QBox<SlotNoArgs>,
    pub find_references: QBox<SlotNoArgs>,
    pub go_to_definition: QBox<SlotNoArgs>,
    pub go_to_loc: Vec<QBox<SlotNoArgs>>,
    pub hide_show_columns: Vec<QBox<SlotOfInt>>,
    pub hide_show_columns_all: QBox<SlotOfInt>,
    pub freeze_columns: Vec<QBox<SlotOfInt>>,
    pub freeze_columns_all: QBox<SlotOfInt>,
    pub open_subtable: QBox<SlotOfQModelIndex>,
    pub profile_apply: QBox<SlotOfQString>,
    pub profile_delete: QBox<SlotOfQString>,
    pub profile_new: QBox<SlotNoArgs>,
    pub profile_set_as_default: QBox<SlotOfQString>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `TableViewSlots`.
impl TableViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(
        view: &Arc<TableView>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        packed_file_path: Option<Arc<RwLock<String>>>,
    ) -> Self {

        // When we want to update the diagnostic/global search data of this table.
        let delayed_updates = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            view => move || {
            info!("Triggering `Delayed Table Updates` By Slot");

            // Only save to the backend if both, the save and undo locks are disabled. Otherwise this will cause locks.
            if view.get_data_source() == DataSource::PackFile && !view.save_lock.load(Ordering::SeqCst) && !view.undo_lock.load(Ordering::SeqCst) {
                if let Some(ref packed_file_path) = view.packed_file_path {
                    let mut paths_to_check = vec![];
                    if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *packed_file_path.read().unwrap() && x.data_source() == DataSource::PackFile) {
                        if let Err(error) = packed_file.save(&app_ui, &pack_file_contents_ui) {
                            show_dialog(&view.table_view, error, false);
                        } else if let Some(path) = view.get_packed_file_path() {
                            paths_to_check.push(path);
                        }
                    }

                    if setting_bool("diagnostics_trigger_on_table_edit") && diagnostics_ui.diagnostics_dock_widget().is_visible() {
                        for path in &paths_to_check {
                            let path_types = vec![ContainerPath::File(path.to_owned())];
                            DiagnosticsUI::check_on_path(&app_ui, &diagnostics_ui, path_types);
                        }
                    }
                }
            }

            // Update the line counter, just in case data changed that caused the counter to be incorrect.
            view.update_line_counter();
        }));

        let sort_order_column_changed = SlotOfIntSortOrder::new(&view.table_view, clone!(
            view => move |column, _| {
                info!("Triggering `Sort Order` By Slot");
                sort_column(&view.table_view_ptr(), column, view.column_sort_state.clone());
            }
        ));

        // When we want to show the context menu.
        let show_context_menu = SlotOfQPoint::new(&view.table_view, clone!(
            view => move |_| {
            view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(&view.table_view, clone!(
            view => move |_,_| {
            info!("Triggering `Update Context Menu for Table` By Slot");
            view.context_menu_update();
        }));

        // When we want to respond to a change in one item in the model.
        let item_changed = SlotOfQStandardItem::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |item| {
                info!("Triggering `Table Item Change` By Slot");

                // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                if !view.undo_lock.load(Ordering::SeqCst) {
                    let item_old = view.undo_model.item_2a(item.row(), item.column());

                    // Only trigger this if the values are actually different. Checkable cells are tricky. Nested cells an go to hell.
                    if (item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state()) ||
                        item_old.data_1a(ITEM_IS_SEQUENCE).to_bool() && 0 != item_old.data_1a(ITEM_SEQUENCE_DATA).to_string().compare_q_string(&item.data_1a(ITEM_SEQUENCE_DATA).to_string()) {
                        let edition = vec![((item.row(), item.column()), atomic_from_ptr((*item_old).clone()))];
                        let operation = TableOperations::Editing(edition);
                        view.history_undo.write().unwrap().push(operation);
                        view.history_redo.write().unwrap().clear();

                        {
                            // We block the saving for painting, so this doesn't get retriggered again.
                            let blocker = QSignalBlocker::from_q_object(&view.table_model);
                            item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_MODIFIED);

                            let definition = view.table_definition();
                            let patches = Some(definition.patches());
                            let fields_processed = definition.fields_processed();
                            let field = &fields_processed[item.column() as usize];

                            // Update the lookup data while the model is blocked.
                            if setting_bool("enable_lookups") {
                                let dependency_data = view.dependency_data.read().unwrap();
                                if let Some(column_data) = dependency_data.get(&item.column()) {
                                    if let Some(lookup) = column_data.data().get(&item.text().to_std_string()) {
                                        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup)), ITEM_SUB_DATA);
                                    }
                                }

                                // If the edited field is used as lookup of another field on the same table, update it too.
                                //
                                // Only non-reference key columns in single-key tables can have lookups, so we only check those.
                                let key_amount = fields_processed.iter().filter(|x| x.is_key(patches)).count();
                                if key_amount == 1 {

                                    for (column, field_ref) in fields_processed.iter().enumerate() {
                                        let mut lookup_string = String::new();
                                        let item_looking_up = view.table_model.item_2a(item.row(), column as i32);

                                        if field_ref.is_key(patches) && field_ref.is_reference(patches).is_none() {
                                            if let Some(lookups_ref) = field_ref.lookup(patches) {
                                                for lookup_ref in lookups_ref {
                                                    if field.name() == lookup_ref {
                                                        let data = item.data_1a(2).to_string().to_std_string();

                                                        if !lookup_string.is_empty() {
                                                            lookup_string.push(':');
                                                        }

                                                        lookup_string.push_str(&data);
                                                    }
                                                }
                                            }
                                        }

                                        if !lookup_string.is_empty() {
                                            item_looking_up.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup_string)), ITEM_SUB_DATA);
                                        }
                                    }
                                }
                            }

                            // If the edited column has icons we need to fetch the new icon from the backend and apply it.
                            if setting_bool("enable_icons") && field.is_filename(patches) {
                                let mut icons = BTreeMap::new();
                                let data = vec![vec![get_field_from_view(&view.table_model.static_upcast(), field, item.row(), item.column())]];

                                if request_backend_files(&data, 0, field, patches, &mut icons).is_ok() {
                                    if let Some(column_data) = icons.get(&0) {
                                        let cell_data = data[0][0].data_to_string().replace('\\', "/");

                                        // For paths, we need to fix the ones in older games starting with / or data/.
                                        let mut start_offset = 0;
                                        if cell_data.starts_with("/") {
                                            start_offset += 1;
                                        }
                                        if cell_data.starts_with("data/") {
                                            start_offset += 5;
                                        }

                                        let paths_join = column_data.0.replace('%', &cell_data[start_offset..]).to_lowercase();
                                        let paths_split = paths_join.split(';');
                                        for path in paths_split {
                                            if let Some(icon) = column_data.1.get(path) {
                                                let icon = ref_from_atomic(icon);
                                                item.set_icon(icon);
                                                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(path)), 52);

                                                // Nuke any cached png from the tooltips.
                                                item.set_data_2a(&QVariant::new(), 50);
                                            }
                                        }
                                    }
                                }
                            }

                            view.update_row_diff_marker(&definition, item.row());

                            blocker.unblock();
                        }

                        // For pasting, or really any heavy operation, only do these tasks the last iteration of the operation.
                        if !view.save_lock.load(Ordering::SeqCst) {
                            update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());
                            view.context_menu_update();
                            if let Some(ref packed_file_path) = packed_file_path {
                                if let Some(search_view) = &*view.search_view() {
                                    search_view.update_search(&view);
                                }

                                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                                }
                            }
                        }
                    }
                }

                if setting_bool("table_resize_on_edit") {
                    view.table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
                }

                view.start_delayed_updates_timer();
            }
        ));

        // When you want to append a row to the table...
        let add_rows = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Add Rows` By Slot");
                view.append_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let insert_rows = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Insert Rows` By Slot");
                view.insert_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Delete Rows` By Slot");
                view.smart_delete(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // When you want to delete all rows not in the current filter...
        let delete_rows_not_in_filter = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Delete Rows Not in Filter` By Slot");
                if AppUI::are_you_sure_edition(&app_ui, "are_you_sure_delete_filtered_out_rows") {
                    view.delete_filtered_out_rows(&app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // When you want to clone and insert one or more rows.
        let clone_and_append = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            view.append_rows(true);
            info!("Triggering `Clone and Append` By Slot");
            if let Some(ref packed_file_path) = view.packed_file_path {
                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        }));

        // When you want to clone and append one or more rows.
        let clone_and_insert = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            view.insert_rows(true);
            info!("Triggering `Clone and Insert` By Slot");
            if let Some(ref packed_file_path) = view.packed_file_path {
                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        }));

        // When you want to copy one or more cells.
        let copy = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            info!("Triggering `Copy` By Slot");
            view.copy_selection();
        }));

        // When you want to copy a table as a lua table.
        let copy_as_lua_table = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            info!("Triggering `Copy as Lua Table` By Slot");
            view.copy_selection_as_lua_table();
        }));

        // When you want to copy a table to a filter string.
        let copy_to_filter_value = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            info!("Triggering `Copy selection to filter` By Slot");
            view.copy_selection_to_filter();
        }));

        // When you want to copy one or more cells.
        let paste = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui => move || {
            info!("Triggering `Paste` By Slot");
            view.paste(&app_ui, &pack_file_contents_ui);
        }));

        // When you want to paste a row at the end of the table...
        let paste_as_new_row = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui => move || {
            info!("Triggering `Paste as New Row` By Slot");
                view.paste_as_new_row(&app_ui, &pack_file_contents_ui);
            }
        ));

        // When we want to invert the selection of the table.
        let invert_selection = SlotNoArgs::new(&view.table_view, clone!(
            mut view => move || {
            info!("Triggering `Invert Selection` By Slot");
            let rows = view.table_filter.row_count_0a();
            let columns = view.table_filter.column_count_0a();
            if rows > 0 && columns > 0 {
                let selection_model = view.table_view.selection_model();
                let first_item = view.table_filter.index_2a(0, 0);
                let last_item = view.table_filter.index_2a(rows - 1, columns - 1);
                let selection = QItemSelection::new_2a(&first_item, &last_item);
                selection_model.select_q_item_selection_q_flags_selection_flag(&selection, QFlags::from(SelectionFlag::Toggle));
            }
        }));

        // When we want to reset the selected items of the table to their original value.
        let reset_selection = SlotNoArgs::new(&view.table_view, clone!(
            mut view => move || {
            view.reset_selection();
        }));

        // When we want to rewrite the selected items using a formula.
        let rewrite_selection = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            info!("Triggering `Rewrite Selection` By Slot");
            view.rewrite_selection(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to revert the selected items to their vanilla/parent values.
        let revert_value = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            info!("Triggering `Revert Values` By Slot");
            view.revert_values(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to rewrite the selected items using a formula.
        let generate_ids = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            info!("Triggering `Generate Ids` By Slot");
            view.generate_ids(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to undo the last action.
        let undo = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Undo` By Slot");
                view.undo_redo(true, 0);
                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());
                view.context_menu_update();
                if view.history_undo.read().unwrap().is_empty() {
                    if let Some(ref packed_file_path) = view.packed_file_path {
                        if let DataSource::PackFile = *view.data_source.read().unwrap() {
                            set_modified(false, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                        }
                    }
                }
            }
        ));

        // When we want to redo the last undone action.
        let redo = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Redo` By Slot");
                view.undo_redo(false, 0);
                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());
                view.context_menu_update();
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When we want to import a TSV file.
        let import_tsv = SlotOfBool::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |_| {

                // For now only import if this is the parent table.
                if let Some(ref packed_file_path) = view.packed_file_path {
                    info!("Triggering `Import TSV` By Slot");

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view,
                        &qtr("tsv_select_title"),
                    );

                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Run it and, if we receive 1 (Accept), try to import the TSV file.
                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        let receiver = CENTRAL_COMMAND.send_background(Command::ImportTSV(packed_file_path.read().unwrap().to_owned(), path));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::RFileDecoded(data) => {
                                let data = match data {
                                    RFileDecoded::DB(data) => TableType::DB(data),
                                    RFileDecoded::Loc(data) => TableType::Loc(data),
                                    _ => unimplemented!(),
                                };
                                let old_data = view.get_copy_of_table();

                                view.undo_lock.store(true, Ordering::SeqCst);

                                let table_name = match data {
                                    TableType::DB(ref db) => Some(db.table_name().to_owned()),
                                    _ => None,
                                };

                                load_data(
                                    &view.table_view_ptr(),
                                    &view.table_definition(),
                                    table_name.as_deref(),
                                    &view.dependency_data,
                                    &data,
                                    &view.timer_delayed_updates,
                                    view.get_data_source(),
                                    &view.vanilla_hashed_tables.read().unwrap()
                                );

                                // Prepare the diagnostic pass.
                                view.start_delayed_updates_timer();

                                view.undo_lock.store(false, Ordering::SeqCst);

                                view.history_undo.write().unwrap().push(TableOperations::ImportTSV(old_data));
                                view.history_redo.write().unwrap().clear();
                                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());

                                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                                }
                            },
                            Response::Error(error) => return show_dialog(&view.table_view, error, false),
                            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                        }

                        //unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                        view.context_menu_update();
                    }
                }
            }
        ));

        // When we want to export the table as a TSV File.
        let export_tsv = SlotOfBool::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |_| {
                if let Some(ref packed_file_path) = view.packed_file_path {
                    info!("Triggering `Export TSV` By Slot");

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view,
                        &qtr("tsv_export_title")
                    );

                    file_dialog.set_accept_mode(AcceptMode::AcceptSave);
                    file_dialog.set_confirm_overwrite(true);
                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
                    file_dialog.set_default_suffix(&QString::from_std_str("tsv"));

                    // Run it and, if we receive 1 (Accept), export the DB Table, saving it's contents first.
                    if file_dialog.exec() == 1 {

                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *packed_file_path.read().unwrap() && x.data_source() == DataSource::PackFile) {
                            if let Err(error) = packed_file.save(&app_ui, &pack_file_contents_ui) {
                                return show_dialog(&view.table_view, error, false);
                            }
                        }

                        let receiver = CENTRAL_COMMAND.send_background(Command::ExportTSV(packed_file_path.read().unwrap().to_string(), path, view.get_data_source()));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::Success => (),
                            Response::Error(error) => show_dialog(&view.table_view, error, false),
                            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                        }
                    }
                }
            }
        ));

        // When we want to resize the columns depending on their contents...
        let resize_columns = SlotNoArgs::new(&view.table_view, clone!(view => move || {
            view.table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
            if setting_bool("extend_last_column_on_tables") {
                view.table_view.horizontal_header().set_stretch_last_section(false);
                view.table_view.horizontal_header().set_stretch_last_section(true);
            }
        }));

        // When you want to use the "Smart Delete" feature...
        let smart_delete = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Smart Delete` By Slot");
                view.smart_delete(false, &app_ui, &pack_file_contents_ui);
            }
        ));

        let sidebar = SlotOfBool::new(&view.table_view, clone!(
            mut view => move |_| {
            match view.sidebar_scroll_area.is_visible() {
                true => view.sidebar_scroll_area.hide(),
                false => view.sidebar_scroll_area.show()
            }
        }));

        let search = SlotOfBool::new(&view.table_view, clone!(
            mut view => move |_| {
            info!("Triggering `Search` By Slot");
            if let Some(search_view) = &*view.search_view() {
                match search_view.main_widget().is_visible() {
                    true => search_view.main_widget().hide(),
                    false => {
                        search_view.main_widget().show();
                        search_view.search_line_edit().set_focus_0a();
                    }
                }
            }
        }));

        let cascade_edition = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui => move || {
                info!("Triggering `Cascade Edition` By Slot");
                view.cascade_edition(&app_ui, &pack_file_contents_ui);
            }
        ));

        let patch_column = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
                info!("Triggering `Patch Column` By Slot");
                if let Err(error) = view.patch_column() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let find_references = SlotNoArgs::new(&view.table_view, clone!(
            references_ui,
            view => move || {

            let selection = view.table_view.selection_model().selection();
            if selection.count_0a() == 1 {
                let filter_index = selection.take_at(0).indexes().take_at(0);
                let index = view.table_filter.map_to_source(filter_index.as_ref());
                if index.is_valid() && !view.table_model.item_from_index(&index).is_checkable() {
                    if let Some(field) = view.table_definition.read().unwrap().fields_processed().get(index.column() as usize) {
                        if let Some(reference_data) = view.reference_map.get(field.name()) {

                            // Stop if we have another find already running.
                            if references_ui.references_table_view().is_enabled() {
                                references_ui.references_dock_widget().show();
                                references_ui.references_table_view().set_enabled(false);

                                let selected_value = index.data_0a().to_string().to_std_string();
                                let receiver = CENTRAL_COMMAND.send_background(Command::SearchReferences(reference_data.clone(), selected_value));
                                let response = CENTRAL_COMMAND.recv_try(&receiver);
                                match response {
                                    Response::VecDataSourceStringStringUsizeUsize(data) => {
                                        references_ui.load_references_to_ui(data);

                                        // Reenable the table.
                                        references_ui.references_table_view().set_enabled(true);
                                    }
                                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                                }
                            }
                        }
                    }
                }
            }
        }));

        let go_to_definition = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move || {
                info!("Triggering `Go To Definition` By Slot");
                if let Some(error) = view.go_to_definition(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui) {
                    log_to_status_bar(&error);
                }
            }
        ));

        let mut go_to_loc = vec![];

        for field in view.table_definition().localised_fields() {
            let field_name = field.name().to_owned();
            let slot = SlotNoArgs::new(&view.table_view, clone!(
                view,
                app_ui,
                pack_file_contents_ui,
                global_search_ui,
                diagnostics_ui,
                dependencies_ui,
                references_ui => move || {
                    info!("Triggering `Go To Loc` By Slot");
                    if let Some(error) = view.go_to_loc(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, &field_name) {
                        log_to_status_bar(&error);
                    }
                }
            ));

            go_to_loc.push(slot);
        }

        let mut hide_show_columns = vec![];
        let mut freeze_columns = vec![];

        let fields = view.table_definition().fields_processed_sorted(setting_bool("tables_use_old_column_order"));
        let fields_processed = view.table_definition().fields_processed();
        for field in &fields {
            if let Some(index) = fields_processed.iter().position(|x| x == field) {
                let hide_show_slot = SlotOfInt::new(&view.table_view, clone!(
                    mut view => move |state| {
                        let state = state == 2;
                        view.table_view.set_column_hidden(index as i32, state);
                    }
                ));

                let freeze_slot = SlotOfInt::new(&view.table_view, clone!(
                    mut view => move |_| {
                        toggle_freezer_safe(&view.table_view, index as i32);
                    }
                ));

                hide_show_columns.push(hide_show_slot);
                freeze_columns.push(freeze_slot);
            }
        }

        let hide_show_columns_all = SlotOfInt::new(&view.table_view, clone!(
            mut view => move |state| {
                let state = state == 2;
                view.sidebar_hide_checkboxes().iter().for_each(|x| x.set_checked(state))
            }
        ));

        let freeze_columns_all = SlotOfInt::new(&view.table_view, clone!(
            mut view => move |state| {
                let state = state == 2;
                view.sidebar_freeze_checkboxes().iter().for_each(|x| x.set_checked(state))
            }
        ));

        let open_subtable = SlotOfQModelIndex::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui,
            view => move |model_index| {
                info!("Triggering `Open Subtable` By Slot");
                open_subtable(
                    model_index,
                    &view,
                    &app_ui,
                    &global_search_ui,
                    &pack_file_contents_ui,
                    &diagnostics_ui,
                    &dependencies_ui,
                    &references_ui,
                    None
                )
            }
        ));

        let profile_apply = SlotOfQString::new(&view.table_view, clone!(
            view => move |key| {
                info!("Triggering `Apply Profile` By Slot");

                view.apply_table_view_profile(&key.to_std_string());
            }
        ));

        let profile_delete = SlotOfQString::new(&view.table_view, clone!(
            view => move |key| {
                info!("Triggering `Delete Profile` By Slot");

                view.delete_table_view_profile(&key.to_std_string());
                if let Err(error) = view.save_table_view_profiles() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let profile_new = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
                info!("Triggering `New Profile` By Slot");

                if let Err(error) = view.new_profile_dialog() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let profile_set_as_default = SlotOfQString::new(&view.table_view, clone!(
            view => move |key| {
                info!("Triggering `Set Default Profile` By Slot");

                *view.profile_default.write().unwrap() = key.to_std_string().to_owned();

                if let Err(error) = view.save_table_view_profiles() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            delayed_updates,
            sort_order_column_changed,
            show_context_menu,
            context_menu_enabler,
            item_changed,
            add_rows,
            insert_rows,
            delete_rows,
            delete_rows_not_in_filter,
            clone_and_append,
            clone_and_insert,
            copy,
            copy_as_lua_table,
            copy_to_filter_value,
            paste,
            paste_as_new_row,
            invert_selection,
            reset_selection,
            rewrite_selection,
            revert_value,
            generate_ids,
            undo,
            redo,
            import_tsv,
            export_tsv,
            smart_delete,
            resize_columns,
            sidebar,
            search,
            cascade_edition,
            patch_column,
            find_references,
            go_to_definition,
            go_to_loc,
            hide_show_columns,
            hide_show_columns_all,
            freeze_columns,
            freeze_columns_all,
            open_subtable,
            profile_apply,
            profile_delete,
            profile_new,
            profile_set_as_default,
        }
    }
}

