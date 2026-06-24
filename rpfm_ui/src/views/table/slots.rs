//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QMenu;
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::AcceptMode;
use qt_widgets::SlotOfIntSortOrder;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QCursor;
use qt_gui::SlotOfQStandardItem;

use qt_core::QBox;
use qt_core::QItemSelection;
use qt_core::QString;
use qt_core::QSignalBlocker;
use qt_core::{SlotOfBool, SlotOfInt, SlotNoArgs, SlotOfQItemSelectionQItemSelection, SlotOfQModelIndex, SlotOfQString};

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, atomic::Ordering, RwLock};

use rpfm_ipc::helpers::DataSource;
use rpfm_ipc::settings_keys::*;

use rpfm_lib::files::{ContainerPath, RFileDecoded};

use rpfm_ui_common::clone;
use rpfm_ui_common::utils::{atomic_from_ptr, ref_from_atomic};

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::utils::set_modified;
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::settings_bool;
use crate::UI_STATE;
use crate::utils::{show_dialog, log_to_status_bar, qtr};

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
    pub search: QBox<SlotOfBool>,
    pub cascade_edition: QBox<SlotNoArgs>,
    pub patch_column: QBox<SlotNoArgs>,
    pub find_references: QBox<SlotNoArgs>,
    pub go_to_definition: QBox<SlotNoArgs>,
    pub go_to_file: QBox<SlotNoArgs>,
    pub go_to_loc: Vec<QBox<SlotNoArgs>>,
    pub hide_show_columns: Vec<QBox<SlotOfInt>>,
    pub hide_show_columns_all: QBox<SlotOfInt>,
    pub freeze_columns: Vec<QBox<SlotOfInt>>,
    pub freeze_columns_all: QBox<SlotOfInt>,
    pub header_context_menu: QBox<SlotOfQPoint>,
    pub open_subtable: QBox<SlotOfQModelIndex>,
    pub profile_apply: QBox<SlotOfQString>,
    pub profile_delete: QBox<SlotOfQString>,
    pub profile_new: QBox<SlotNoArgs>,
    pub profile_set_as_default: QBox<SlotOfQString>,
    pub toggle_flagged_rows_filter: QBox<SlotOfBool>,
    pub header_funnel_clicked: QBox<SlotOfInt>,
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
            rpfm_telemetry::track_action("Delayed Table Updates");

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

                    if settings_bool(DIAGNOSTICS_TRIGGER_ON_TABLE_EDIT) && diagnostics_ui.diagnostics_dock_widget().is_visible() {
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
                rpfm_telemetry::track_action("Sort Order");
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
            app_ui,
            pack_file_contents_ui,
            view => move |_,_| {
            rpfm_telemetry::track_action("Update Context Menu for Table");
            view.context_menu_update();

            TableView::update_key_deletes_list(&view, &app_ui, &pack_file_contents_ui);
        }));

        // When we want to respond to a change in one item in the model.
        let item_changed = SlotOfQStandardItem::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |item| {
                rpfm_telemetry::track_action("Table Item Change");

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
                            if settings_bool(ENABLE_LOOKUPS) {
                                let dependency_data = view.dependency_data.read().unwrap();
                                if let Some(column_data) = dependency_data.get(&item.column()) {
                                    match column_data.data().get(&item.text().to_std_string()) {
                                        Some(lookup) => item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup)), ITEM_SUB_DATA),
                                        None => item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str("")), ITEM_SUB_DATA),
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
                            if settings_bool(ENABLE_ICONS) && field.is_filename(patches) {
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

                                        let mut found = false;
                                        for path in paths_split {
                                            if let Some(icon) = column_data.1.get(path) {
                                                let icon = ref_from_atomic(icon);
                                                item.set_icon(icon);
                                                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(path)), ITEM_ICON_PATH);
                                                found = true;
                                                break;
                                            }
                                        }

                                        if !found {
                                            item.set_icon(&QIcon::new());
                                            item.set_data_2a(&QVariant::new(), ITEM_ICON_PATH);
                                        }

                                        // For tooltips, we just nuke all the catched pngs. It's simpler than trying to go one by one and finding the ones that need updating.
                                        item.set_data_2a(&QVariant::new(), ITEM_ICON_CACHE);
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
                                    set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                                }
                            }
                        }
                    }
                }

                if settings_bool(TABLE_RESIZE_ON_EDIT) {
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
                rpfm_telemetry::track_action("Add Rows");
                view.append_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let insert_rows = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                rpfm_telemetry::track_action("Insert Rows");
                view.insert_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                rpfm_telemetry::track_action("Delete Rows");
                view.smart_delete(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // When you want to delete all rows not in the current filter...
        let delete_rows_not_in_filter = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                rpfm_telemetry::track_action("Delete Rows Not in Filter");
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
            rpfm_telemetry::track_action("Clone and Append");
            if let Some(ref packed_file_path) = view.packed_file_path {
                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                    set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        }));

        // When you want to clone and append one or more rows.
        let clone_and_insert = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            view.insert_rows(true);
            rpfm_telemetry::track_action("Clone and Insert");
            if let Some(ref packed_file_path) = view.packed_file_path {
                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                    set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        }));

        // When you want to copy one or more cells.
        let copy = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            rpfm_telemetry::track_action("Copy");
            view.copy_selection();
        }));

        // When you want to copy a table as a lua table.
        let copy_as_lua_table = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            rpfm_telemetry::track_action("Copy as Lua Table");
            view.copy_selection_as_lua_table();
        }));

        // When you want to copy a table to a filter string.
        let copy_to_filter_value = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
            rpfm_telemetry::track_action("Copy selection to filter");
            view.copy_selection_to_filter();
        }));

        // When you want to copy one or more cells.
        let paste = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui => move || {
            rpfm_telemetry::track_action("Paste");
            view.paste(&app_ui, &pack_file_contents_ui);
        }));

        // When you want to paste a row at the end of the table...
        let paste_as_new_row = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui => move || {
            rpfm_telemetry::track_action("Paste as New Row");
                view.paste_as_new_row(&app_ui, &pack_file_contents_ui);
            }
        ));

        // When we want to invert the selection of the table.
        let invert_selection = SlotNoArgs::new(&view.table_view, clone!(
            mut view => move || {
            rpfm_telemetry::track_action("Invert Selection");
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
            rpfm_telemetry::track_action("Reset Selection");
            view.reset_selection();
        }));

        // When we want to rewrite the selected items using a formula.
        let rewrite_selection = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            rpfm_telemetry::track_action("Rewrite Selection");
            view.rewrite_selection(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to revert the selected items to their vanilla/parent values.
        let revert_value = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            rpfm_telemetry::track_action("Revert Values");
            view.revert_values(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to rewrite the selected items using a formula.
        let generate_ids = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            rpfm_telemetry::track_action("Generate Ids");
            view.generate_ids(&app_ui, &pack_file_contents_ui);
        }));

        // When we want to undo the last action.
        let undo = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                rpfm_telemetry::track_action("Undo");
                view.undo_redo(true, 0);
                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());
                view.context_menu_update();
                if view.history_undo.read().unwrap().is_empty() {
                    if let Some(ref packed_file_path) = view.packed_file_path {
                        if let DataSource::PackFile = *view.data_source.read().unwrap() {
                            set_modified(false, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
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
                rpfm_telemetry::track_action("Redo");
                view.undo_redo(false, 0);
                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());
                view.context_menu_update();
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let DataSource::PackFile = *view.data_source.read().unwrap() {
                        set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
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
                    rpfm_telemetry::track_action("Import TSV");

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view,
                        &qtr("tsv_select_title"),
                    );

                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Run it and, if we receive 1 (Accept), try to import the TSV file.
                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        let pack_key = view.pack_key.read().unwrap().clone();
                        match send_ipc_command_result_async(Command::ImportTSV(pack_key, packed_file_path.read().unwrap().to_owned(), path), response_extractor!(Response::RFileDecoded)) {
                            Ok(data) => {
                                let data = match data {
                                    RFileDecoded::DB(data) => TableType::DB(data),
                                    RFileDecoded::Loc(data) => TableType::Loc(data),
                                    _ => unimplemented!(),
                                };
                                //let old_data = view.get_copy_of_table();
                                //let old_definition = view.table_definition.read().unwrap().clone();

                                view.undo_lock.store(true, Ordering::SeqCst);

                                let table_name = match data {
                                    TableType::DB(ref db) => Some(db.table_name().to_owned()),
                                    _ => None,
                                };

                                // Due to this being able to import different versions of the table we have, we need to update all the definition-specific data
                                // before loading the new data to the UI. Otherwise we may trigger index out of bounds errors.
                                if let TableType::DB(ref data) = data {
                                    *view.table_definition.write().unwrap() = data.definition().clone();

                                    let definition = view.table_definition.read().unwrap();
                                    let table_name = if let Some(name) = view.table_name() { name.to_owned() } else { "".to_owned() };

                                    // Get the reference data for this table, to speedup reference searching.
                                    *view.reference_map.write().unwrap() = referencing_columns_for_table(&table_name, &definition).unwrap_or_default();

                                    // Regenerate the references for this table, as we may have different columns with new references.
                                    let pack_key = view.pack_key.read().unwrap().clone();
                                    if let Ok(data) = get_reference_data(*view.packed_file_type, &table_name, &definition, true, &pack_key) {
                                        view.set_dependency_data(&data);
                                    }
                                }

                                load_data(
                                    &view.table_view_ptr(),
                                    &view.table_definition(),
                                    table_name.as_deref(),
                                    &view.dependency_data,
                                    &data,
                                    &view.timer_delayed_updates,
                                    view.get_data_source(),
                                    &view.vanilla_hashed_tables.read().unwrap(),
                                    &view.reference_map
                                );

                                // Prepare the diagnostic pass.
                                view.start_delayed_updates_timer();

                                view.undo_lock.store(false, Ordering::SeqCst);

                                // Due to versioning bugs and discrepancies between backend and frontend, we need to clear the undo history, so this operation cannot be reverted.
                                //view.history_undo.write().unwrap().push(TableOperations::ImportTSV(old_data));
                                view.history_undo.write().unwrap().clear();
                                view.history_redo.write().unwrap().clear();
                                update_undo_model(&view.table_model_ptr(), &view.undo_model_ptr());

                                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                                    set_modified(true, &packed_file_path.read().unwrap(), &view.pack_key.read().unwrap(), &app_ui, &pack_file_contents_ui);
                                }
                            },
                            Err(error) => return show_dialog(&view.table_view, error, false),
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
                    rpfm_telemetry::track_action("Export TSV");

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view,
                        &qtr("tsv_export_title")
                    );

                    file_dialog.set_accept_mode(AcceptMode::AcceptSave);
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

                        let pack_key = view.pack_key.read().unwrap().clone();
                        if let Err(error) = send_ipc_command_result_async(Command::ExportTSV(pack_key, packed_file_path.read().unwrap().to_string(), path, view.get_data_source()), response_extractor!()) {
                            show_dialog(&view.table_view, error, false);
                        }
                    }
                }
            }
        ));

        // When we want to resize the columns depending on their contents...
        let resize_columns = SlotNoArgs::new(&view.table_view, clone!(view => move || {
            view.table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
            if settings_bool(EXTEND_LAST_COLUMN_ON_TABLES) {
                view.table_view.horizontal_header().set_stretch_last_section(false);
                view.table_view.horizontal_header().set_stretch_last_section(true);
            }
        }));

        // When you want to use the "Smart Delete" feature...
        let smart_delete = SlotNoArgs::new(&view.table_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                rpfm_telemetry::track_action("Smart Delete");
                view.smart_delete(false, &app_ui, &pack_file_contents_ui);
            }
        ));

        let search = SlotOfBool::new(&view.table_view, clone!(
            mut view => move |_| {
            rpfm_telemetry::track_action("Search");
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
                rpfm_telemetry::track_action("Cascade Edition");
                view.cascade_edition(&app_ui, &pack_file_contents_ui);
            }
        ));

        let patch_column = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
                rpfm_telemetry::track_action("Patch Column");
                if let Err(error) = view.patch_column() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let find_references = SlotNoArgs::new(&view.table_view, clone!(
            references_ui,
            view => move || {
            rpfm_telemetry::track_action("Find References");

            let selection = view.table_view.selection_model().selection();
            if selection.count() == 1 {
                let filter_index = selection.take_at(0).indexes().take_at(0);
                let index = view.table_filter.map_to_source(filter_index.as_ref());
                if index.is_valid() && !view.table_model.item_from_index(&index).is_checkable() {
                    if let Some(field) = view.table_definition.read().unwrap().fields_processed().get(index.column() as usize) {
                        if let Some(reference_data) = view.reference_map.read().unwrap().get(field.name()) {

                            // Stop if we have another find already running.
                            if references_ui.references_table_view().is_enabled() {
                                references_ui.references_dock_widget().show();
                                references_ui.references_table_view().set_enabled(false);

                                let pack_key = view.pack_key.read().unwrap().clone();
                                let selected_value = index.data_0a().to_string().to_std_string();
                                match send_ipc_command_result_async(Command::SearchReferences(pack_key, reference_data.clone(), selected_value), response_extractor!(Response::VecDataSourceStringStringStringUsizeUsize)) {
                                    Ok(data) => {
                                        references_ui.load_references_to_ui(data);

                                        // Reenable the table.
                                        references_ui.references_table_view().set_enabled(true);
                                    }
                                    Err(error) => {
                                        references_ui.references_table_view().set_enabled(true);
                                        show_dialog(&view.table_view, error, false);
                                    }
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
                rpfm_telemetry::track_action("Go To Definition");
                if let Some(error) = view.go_to_definition(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui) {
                    log_to_status_bar(&error);
                }
            }
        ));

        let go_to_file = SlotNoArgs::new(&view.table_view, clone!(
            view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move || {
                rpfm_telemetry::track_action("Go To File");
                if let Some(error) = view.go_to_file(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui) {
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
                    rpfm_telemetry::track_action("Go To Loc");
                    if let Some(error) = view.go_to_loc(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, &field_name) {
                        log_to_status_bar(&error);
                    }
                }
            ));

            go_to_loc.push(slot);
        }

        let mut hide_show_columns = vec![];
        let mut freeze_columns = vec![];

        let fields = view.table_definition().fields_processed_sorted(settings_bool(TABLES_USE_OLD_COLUMN_ORDER));
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

        let header_context_menu = SlotOfQPoint::new(&view.table_view, clone!(
            mut view => move |pos| {

                let header = view.table_view.horizontal_header();
                let logical_index = header.logical_index_at_int(pos.x());
                if logical_index < 0 { return; }

                // Build the reverse mapping: logical column index -> sidebar checkbox index.
                // The sidebar checkboxes are in sorted field order, not logical column order.
                let fields = view.table_definition().fields_processed_sorted(settings_bool(TABLES_USE_OLD_COLUMN_ORDER));
                let fields_processed = view.table_definition().fields_processed();
                let sidebar_index = fields.iter().enumerate().find_map(|(sidebar_pos, field)| {
                    fields_processed.iter().position(|x| x == field).and_then(|col_idx| {
                        if col_idx == logical_index as usize { Some(sidebar_pos) } else { None }
                    })
                });

                let menu = QMenu::from_q_widget(header.as_ptr().static_upcast::<qt_widgets::QWidget>());

                // "Hide Column" action.
                let hide_action = menu.add_action_q_string(&qtr("column_header_hide"));
                let is_hidden = view.table_view.is_column_hidden(logical_index);
                hide_action.set_checkable(true);
                hide_action.set_checked(is_hidden);

                // "Freeze Column" action.
                let is_frozen = sidebar_index.and_then(|idx| view.sidebar_freeze_checkboxes().get(idx).map(|cb| cb.is_checked())).unwrap_or(false);
                let freeze_action = menu.add_action_q_string(&qtr("column_header_freeze"));
                freeze_action.set_checkable(true);
                freeze_action.set_checked(is_frozen);

                let chosen = menu.exec_1a_mut(&qt_gui::QCursor::pos_0a());
                if !chosen.is_null() {
                    if chosen.as_mut_raw_ptr() == hide_action.as_mut_raw_ptr() {
                        if let Some(idx) = sidebar_index {
                            if let Some(cb) = view.sidebar_hide_checkboxes().get(idx) {
                                cb.set_checked(!is_hidden);
                            }
                        }
                    } else if chosen.as_mut_raw_ptr() == freeze_action.as_mut_raw_ptr() {
                        if let Some(idx) = sidebar_index {
                            if let Some(cb) = view.sidebar_freeze_checkboxes().get(idx) {
                                cb.set_checked(!is_frozen);
                            }
                        }
                    }
                }
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
                rpfm_telemetry::track_action("Open Subtable");
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
                rpfm_telemetry::track_action("Apply Profile");

                view.clone().apply_table_view_profile(&key.to_std_string());
            }
        ));

        let profile_delete = SlotOfQString::new(&view.table_view, clone!(
            view => move |key| {
                rpfm_telemetry::track_action("Delete Profile");

                view.delete_table_view_profile(&key.to_std_string());
                if let Err(error) = view.save_table_view_profiles() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let profile_new = SlotNoArgs::new(&view.table_view, clone!(
            view => move || {
                rpfm_telemetry::track_action("New Profile");

                if let Err(error) = view.new_profile_dialog() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let profile_set_as_default = SlotOfQString::new(&view.table_view, clone!(
            view => move |key| {
                rpfm_telemetry::track_action("Set Default Profile");

                *view.profile_default.write().unwrap() = key.to_std_string().to_owned();

                if let Err(error) = view.save_table_view_profiles() {
                    show_dialog(&view.table_view, error, false);
                }
            }
        ));

        let toggle_flagged_rows_filter = SlotOfBool::new(&view.table_view, clone!(
            view => move |_| {
                rpfm_telemetry::track_action("Toggle Flagged Rows Filter");
                view.filter_table();
            }
        ));

        // Clicking a column header's filter funnel adds a chip pre-targeting that logical
        // column and focuses its value field so the user can type the pattern straight away.
        let header_funnel_clicked = SlotOfInt::new(&view.table_view, clone!(
            view => move |logical_index| {
                rpfm_telemetry::track_action("Table Filter: Header Funnel");
                if let Some(bar) = view.filter_bar_arc() {
                    let state = FilterChipState {
                        column_index: logical_index,
                        ..FilterChipState::default()
                    };
                    if bar.add_chip(&view, state, true).is_ok() {
                        view.filter_table();
                    }
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
            search,
            cascade_edition,
            patch_column,
            find_references,
            go_to_definition,
            go_to_file,
            go_to_loc,
            hide_show_columns,
            hide_show_columns_all,
            freeze_columns,
            freeze_columns_all,
            header_context_menu,
            open_subtable,
            profile_apply,
            profile_delete,
            profile_new,
            profile_set_as_default,
            toggle_flagged_rows_filter,
            header_funnel_clicked,
        }
    }
}
