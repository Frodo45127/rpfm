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
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::AcceptMode;
use qt_widgets::SlotOfIntSortOrder;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QBrush;
use qt_gui::QCursor;
use qt_gui::SlotOfQStandardItem;

use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QItemSelection;
use qt_core::QSignalBlocker;
use qt_core::{SlotOfBool, SlotOfInt, SlotNoArgs, SlotOfQString, SlotOfQItemSelectionQItemSelection, SlotOfQModelIndex};

use cpp_core::Ref;

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, atomic::Ordering, RwLock};

use rpfm_lib::packedfile::table::Table;

use crate::app_ui::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::utils::set_modified;
use crate::pack_tree::*;
use crate::utils::{check_regex, show_dialog};
use crate::UI_STATE;

use super::utils::*;
use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a Table PackedFile.
pub struct TableViewSlots {
    pub toggle_lookups: QBox<SlotOfBool>,
    pub sort_order_column_changed: QBox<SlotOfIntSortOrder>,
    pub show_context_menu: QBox<SlotOfQPoint>,
    pub context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    pub item_changed: QBox<SlotOfQStandardItem>,
    pub add_rows: QBox<SlotNoArgs>,
    pub insert_rows: QBox<SlotNoArgs>,
    pub delete_rows: QBox<SlotNoArgs>,
    pub clone_and_append: QBox<SlotNoArgs>,
    pub clone_and_insert: QBox<SlotNoArgs>,
    pub copy: QBox<SlotNoArgs>,
    pub copy_as_lua_table: QBox<SlotNoArgs>,
    pub paste: QBox<SlotNoArgs>,
    pub paste_as_new_row: QBox<SlotNoArgs>,
    pub invert_selection: QBox<SlotNoArgs>,
    pub reset_selection: QBox<SlotNoArgs>,
    pub rewrite_selection: QBox<SlotNoArgs>,
    pub save: QBox<SlotNoArgs>,
    pub undo: QBox<SlotNoArgs>,
    pub redo: QBox<SlotNoArgs>,
    pub import_tsv: QBox<SlotOfBool>,
    pub export_tsv: QBox<SlotOfBool>,
    pub smart_delete: QBox<SlotNoArgs>,
    pub resize_columns: QBox<SlotNoArgs>,
    pub sidebar: QBox<SlotOfBool>,
    pub search: QBox<SlotOfBool>,
    pub hide_show_columns: Vec<QBox<SlotOfInt>>,
    pub hide_show_columns_all: QBox<SlotOfInt>,
    pub freeze_columns: Vec<QBox<SlotOfInt>>,
    pub freeze_columns_all: QBox<SlotOfInt>,
    pub search_search: QBox<SlotNoArgs>,
    pub search_prev_match: QBox<SlotNoArgs>,
    pub search_next_match: QBox<SlotNoArgs>,
    pub search_replace_current: QBox<SlotNoArgs>,
    pub search_replace_all: QBox<SlotNoArgs>,
    pub search_close: QBox<SlotNoArgs>,
    pub search_check_regex: QBox<SlotOfQString>,
    pub open_subtable: QBox<SlotOfQModelIndex>,
}

/// This struct contains the slots of the view of a table filter.
pub struct FilterViewSlots {
    pub filter_line_edit: QBox<SlotOfQString>,
    pub filter_column_selector: QBox<SlotOfInt>,
    pub filter_case_sensitive_button: QBox<SlotNoArgs>,
    pub filter_check_regex: QBox<SlotOfQString>,
    pub filter_add: QBox<SlotNoArgs>,
    pub filter_remove: QBox<SlotNoArgs>,
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
        packed_file_path: Option<Arc<RwLock<Vec<String>>>>,
    ) -> Self {

        // When we want to toggle the lookups on and off.
        let toggle_lookups = SlotOfBool::new(&view.table_view_primary, clone!(
            view => move |_| {
            view.toggle_lookups();
        }));

        let sort_order_column_changed = SlotOfIntSortOrder::new(&view.table_view_primary, clone!(
            view => move |column, _| {
                sort_column(&view.get_mut_ptr_table_view_primary(), column, view.column_sort_state.clone());
            }
        ));

        // When we want to show the context menu.
        let show_context_menu = SlotOfQPoint::new(&view.table_view_primary, clone!(
            mut view => move |_| {
            view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(&view.table_view_primary, clone!(
            mut view => move |_,_| {
            view.context_menu_update();
        }));

        // When we want to respond to a change in one item in the model.
        let item_changed = SlotOfQStandardItem::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |item| {

                // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                if !view.undo_lock.load(Ordering::SeqCst) {
                    let item_old = view.undo_model.item_2a(item.row(), item.column());

                    // Only trigger this if the values are actually different. Checkable cells are tricky. Nested cells an go to hell.
                    if (item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state()) ||
                        item_old.data_1a(ITEM_IS_SEQUENCE).to_bool() && 0 != item_old.data_1a(ITEM_SEQUENCE_DATA).to_string().compare_q_string(&item.data_1a(ITEM_SEQUENCE_DATA).to_string()) {
                        let mut edition = Vec::with_capacity(1);
                        edition.push(((item.row(), item.column()), atomic_from_ptr((&*item_old).clone())));
                        let operation = TableOperations::Editing(edition);
                        view.history_undo.write().unwrap().push(operation);
                        view.history_redo.write().unwrap().clear();

                        {
                            // We block the saving for painting, so this doesn't get rettriggered again.
                            let blocker = QSignalBlocker::from_q_object(&view.table_model);
                            let color = get_color_modified();
                            let item = item;
                            item.set_background(&QBrush::from_q_color(color.as_ref().unwrap()));
                            blocker.unblock();
                        }

                        // For pasting, or really any heavy operation, only do these tasks the last iteration of the operation.
                        if !view.save_lock.load(Ordering::SeqCst) {
                            update_undo_model(&view.get_mut_ptr_table_model(), &view.get_mut_ptr_undo_model());
                            view.context_menu_update();
                            if let Some(ref packed_file_path) = packed_file_path {
                                TableSearch::update_search(&view);

                                view.start_diagnostic_check();

                                set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                            }
                        }
                    }
                }

                if SETTINGS.read().unwrap().settings_bool["table_resize_on_edit"] {
                    view.table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
                }
            }
        ));

        // When you want to append a row to the table...
        let add_rows = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.append_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let insert_rows = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.insert_rows(false);
                if let Some(ref packed_file_path) = view.packed_file_path {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {

                // Get all the selected rows.
                let selection = view.table_view_primary.selection_model().selection();
                let indexes = view.table_filter.map_selection_to_source(&selection).indexes();
                let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

                // Dedup the list and reverse it.
                rows_to_delete.sort();
                rows_to_delete.dedup();
                rows_to_delete.reverse();
                let rows_splitted = delete_rows(&view.get_mut_ptr_table_model(), &rows_to_delete);

                // If we deleted something, try to save the PackedFile to the main PackFile.
                if !rows_to_delete.is_empty() {
                    view.history_undo.write().unwrap().push(TableOperations::RemoveRows(rows_splitted));
                    view.history_redo.write().unwrap().clear();

                    // Prepare the diagnostic pass.
                    view.start_diagnostic_check();

                    update_undo_model(&view.get_mut_ptr_table_model(), &view.get_mut_ptr_undo_model());
                    if let Some(ref packed_file_path) = view.packed_file_path {
                        set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When you want to clone and insert one or more rows.
        let clone_and_append = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            view.append_rows(true);
            if let Some(ref packed_file_path) = view.packed_file_path {
                set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
            }
        }));

        // When you want to clone and append one or more rows.
        let clone_and_insert = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
            view.insert_rows(true);
            if let Some(ref packed_file_path) = view.packed_file_path {
                set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
            }
        }));

        // When you want to copy one or more cells.
        let copy = SlotNoArgs::new(&view.table_view_primary, clone!(
            view => move || {
            view.copy_selection();
        }));

        // When you want to copy a table as a lua table.
        let copy_as_lua_table = SlotNoArgs::new(&view.table_view_primary, clone!(
            view => move || {
            view.copy_selection_as_lua_table();
        }));

        // When you want to copy one or more cells.
        let paste = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
            view.paste();
        }));

        // When you want to paste a row at the end of the table...
        let paste_as_new_row = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                view.paste_as_new_row();
            }
        ));

        // When we want to invert the selection of the table.
        let invert_selection = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
            let rows = view.table_filter.row_count_0a();
            let columns = view.table_filter.column_count_0a();
            if rows > 0 && columns > 0 {
                let selection_model = view.table_view_primary.selection_model();
                let first_item = view.table_filter.index_2a(0, 0);
                let last_item = view.table_filter.index_2a(rows - 1, columns - 1);
                let selection = QItemSelection::new_2a(&first_item, &last_item);
                selection_model.select_q_item_selection_q_flags_selection_flag(&selection, QFlags::from(SelectionFlag::Toggle));
            }
        }));

        // When we want to reset the selected items of the table to their original value.
        let reset_selection = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
            view.reset_selection();
        }));

        // When we want to rewrite the selected items using a formula.
        let rewrite_selection = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
            view.rewrite_selection();
        }));

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            view => move || {

            // Only save to the backend if both, the save and undo locks are disabled. Otherwise this will cause locks.
            if !view.save_lock.load(Ordering::SeqCst) && !view.undo_lock.load(Ordering::SeqCst) {
                if let Some(ref packed_file_path) = view.packed_file_path {
                    if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_path.read().unwrap()) {
                        if let Err(error) = packed_file.save(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui) {
                            show_dialog(&view.table_view_primary, error, false);
                        }
                    }
                }
            }
        }));

        // When we want to undo the last action.
        let undo = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.undo_redo(true, 0);
                update_undo_model(&view.get_mut_ptr_table_model(), &view.get_mut_ptr_undo_model());
                view.context_menu_update();
                if view.history_undo.read().unwrap().is_empty() {
                    if let Some(ref packed_file_path) = view.packed_file_path {
                        set_modified(false, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                    }
                }
            }
        ));

        // When we want to redo the last undone action.
        let redo = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.undo_redo(false, 0);
                update_undo_model(&view.get_mut_ptr_table_model(), &view.get_mut_ptr_undo_model());
                view.context_menu_update();
                if let Some(ref packed_file_path) = view.packed_file_path {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        // When we want to import a TSV file.
        let import_tsv = SlotOfBool::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |_| {

                // For now only import if this is the parent table.
                if let Some(ref packed_file_path) = view.packed_file_path {

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view_primary,
                        &qtr("tsv_select_title"),
                    );

                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Run it and, if we receive 1 (Accept), try to import the TSV file.
                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        CENTRAL_COMMAND.send_message_qt(Command::ImportTSV((packed_file_path.read().unwrap().to_vec(), path)));
                        let response = CENTRAL_COMMAND.recv_message_qt_try();
                        match response {
                            Response::TableType(data) => {
                                let old_data = view.get_copy_of_table();

                                view.undo_lock.store(true, Ordering::SeqCst);
                                load_data(
                                    &view.get_mut_ptr_table_view_primary(),
                                    &view.get_mut_ptr_table_view_frozen(),
                                    &view.get_ref_table_definition(),
                                    &view.dependency_data,
                                    &data
                                );

                                // Prepare the diagnostic pass.
                                view.start_diagnostic_check();

                                let table_name = match data {
                                    TableType::DB(_) => packed_file_path.read().unwrap().get(1).cloned(),
                                    _ => None,
                                };

                                build_columns(
                                    &view.get_mut_ptr_table_view_primary(),
                                    Some(&view.get_mut_ptr_table_view_frozen()),
                                    &view.get_ref_table_definition(),
                                    table_name.as_ref()
                                );

                                view.undo_lock.store(false, Ordering::SeqCst);

                                view.history_undo.write().unwrap().push(TableOperations::ImportTSV(old_data));
                                view.history_redo.write().unwrap().clear();
                                update_undo_model(&view.get_mut_ptr_table_model(), &view.get_mut_ptr_undo_model());
                                set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                            },
                            Response::Error(error) => return show_dialog(&view.table_view_primary, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }

                        //unsafe { update_search_stuff.as_mut().unwrap().trigger(); }
                        view.context_menu_update();
                    }
                }
            }
        ));

        // When we want to export the table as a TSV File.
        let export_tsv = SlotOfBool::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            view => move |_| {

                if let Some(ref packed_file_path) = view.packed_file_path {

                    // Create a File Chooser to get the destination path and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &view.table_view_primary,
                        &qtr("tsv_export_title")
                    );

                    file_dialog.set_accept_mode(AcceptMode::AcceptSave);
                    file_dialog.set_confirm_overwrite(true);
                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
                    file_dialog.set_default_suffix(&QString::from_std_str("tsv"));

                    // Run it and, if we receive 1 (Accept), export the DB Table, saving it's contents first.
                    if file_dialog.exec() == 1 {

                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_path.read().unwrap()) {
                            if let Err(error) = packed_file.save(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui) {
                                return show_dialog(&view.table_view_primary, error, false);
                            }
                        }

                        CENTRAL_COMMAND.send_message_qt(Command::ExportTSV((packed_file_path.read().unwrap().to_vec(), path)));
                        let response = CENTRAL_COMMAND.recv_message_qt_try();
                        match response {
                            Response::Success => return,
                            Response::Error(error) => return show_dialog(&view.table_view_primary, error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }
                    }
                }
            }
        ));

        // When we want to resize the columns depending on their contents...
        let resize_columns = SlotNoArgs::new(&view.table_view_primary, clone!(view => move || {
            view.table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
            if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
                view.table_view_primary.horizontal_header().set_stretch_last_section(false);
                view.table_view_primary.horizontal_header().set_stretch_last_section(true);
            }
        }));

        // When you want to use the "Smart Delete" feature...
        let smart_delete = SlotNoArgs::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.smart_delete();
                if let Some(ref packed_file_path) = view.packed_file_path {
                    set_modified(true, &packed_file_path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        let sidebar = SlotOfBool::new(&view.table_view_primary, clone!(
            mut view => move |_| {
            match view.sidebar_scroll_area.is_visible() {
                true => view.sidebar_scroll_area.hide(),
                false => view.sidebar_scroll_area.show()
            }
        }));

        let search = SlotOfBool::new(&view.table_view_primary, clone!(
            mut view => move |_| {
            match view.search_widget.is_visible() {
                true => view.search_widget.hide(),
                false => {
                    view.search_widget.show();
                    view.search_search_line_edit.set_focus_0a();
                }
            }
        }));

        let mut hide_show_columns = vec![];
        let mut freeze_columns = vec![];

        let fields = get_fields_sorted(&view.get_ref_table_definition());
        for (index, _) in fields.iter().enumerate() {
            let hide_show_slot = SlotOfInt::new(&view.table_view_primary, clone!(
                mut view => move |state| {
                    let state = state == 2;
                    view.table_view_primary.set_column_hidden(index as i32, state);
                }
            ));

            let freeze_slot = SlotOfInt::new(&view.table_view_primary, clone!(
                mut view => move |_| {
                    toggle_freezer_safe(&view.table_view_primary, index as i32);
                }
            ));

            hide_show_columns.push(hide_show_slot);
            freeze_columns.push(freeze_slot);
        }

        let hide_show_columns_all = SlotOfInt::new(&view.table_view_primary, clone!(
            mut view => move |state| {
                let state = state == 2;
                view.get_hide_show_checkboxes().iter().for_each(|x| x.set_checked(state))
            }
        ));

        let freeze_columns_all = SlotOfInt::new(&view.table_view_primary, clone!(
            mut view => move |state| {
                let state = state == 2;
                view.get_freeze_checkboxes().iter().for_each(|x| x.set_checked(state))
            }
        ));

        //------------------------------------------------------//
        // Slots related with the search panel.
        //------------------------------------------------------//

        let search_search = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                TableSearch::search(&view);
            }
        ));

        let search_prev_match = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                TableSearch::prev_match(&view);
            }
        ));

        let search_next_match = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                TableSearch::next_match(&view);
            }
        ));

        let search_replace_current = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                TableSearch::replace_current(&view);
            }
        ));

        let search_replace_all = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                TableSearch::replace_all(&view);
            }
        ));

        let search_close = SlotNoArgs::new(&view.table_view_primary, clone!(
            mut view => move || {
                view.search_widget.hide();
                view.table_view_primary.set_focus_0a();
            }
        ));

        // What happens when we trigger the "Check Regex" action.
        let search_check_regex = SlotOfQString::new(&view.table_view_primary, clone!(
            mut view => move |string| {
            check_regex(&string.to_std_string(), view.search_search_line_edit.static_upcast());
        }));

        let open_subtable = SlotOfQModelIndex::new(&view.table_view_primary, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            view => move |model_index| {
                if model_index.data_1a(ITEM_IS_SEQUENCE).to_bool() {
                    let data = model_index.data_1a(ITEM_SEQUENCE_DATA).to_string().to_std_string();
                    let table: Table = serde_json::from_str(&data).unwrap();
                    let table_data = match *view.packed_file_type {
                        PackedFileType::DB => TableType::DB(From::from(table)),
                        PackedFileType::Loc => TableType::Loc(From::from(table)),
                        PackedFileType::MatchedCombat => TableType::MatchedCombat(From::from(table)),
                        PackedFileType::AnimTable => TableType::AnimTable(From::from(table)),
                        PackedFileType::DependencyPackFilesList => unimplemented!("This should never happen, unless you messed up the schemas"),
                        _ => unimplemented!("You forgot to implement subtables for this kind of packedfile"),
                    };
                    if let Some(new_data) = open_subtable(
                        view.table_view_primary.static_upcast(),
                        &app_ui,
                        &global_search_ui,
                        &pack_file_contents_ui,
                        &diagnostics_ui,
                        table_data
                    ) {
                        view.table_filter.set_data_3a(
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
            toggle_lookups,
            sort_order_column_changed,
            show_context_menu,
            context_menu_enabler,
            item_changed,
            add_rows,
            insert_rows,
            delete_rows,
            clone_and_append,
            clone_and_insert,
            copy,
            copy_as_lua_table,
            paste,
            paste_as_new_row,
            invert_selection,
            reset_selection,
            rewrite_selection,
            save,
            undo,
            redo,
            import_tsv,
            export_tsv,
            smart_delete,
            resize_columns,
            sidebar,
            search,
            hide_show_columns,
            hide_show_columns_all,
            freeze_columns,
            freeze_columns_all,
            search_search,
            search_prev_match,
            search_next_match,
            search_replace_current,
            search_replace_all,
            search_close,
            search_check_regex,
            open_subtable,
        }
    }
}


/// Implementation for `FilterViewSlots`.
impl FilterViewSlots {
    pub unsafe fn new(
        view: &Arc<FilterView>,
        parent_view: &Arc<TableView>,
    ) -> Self {

        // When we want to filter the table...
        let filter_line_edit = SlotOfQString::new(&view.filter_widget, clone!(
            parent_view => move |_| {
            parent_view.filter_table();
        }));

        let filter_column_selector = SlotOfInt::new(&view.filter_widget, clone!(
            parent_view => move |_| {
            parent_view.filter_table();
        }));

        let filter_case_sensitive_button = SlotNoArgs::new(&view.filter_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        // What happens when we trigger the "Check Regex" action.
        let filter_check_regex = SlotOfQString::new(&view.filter_widget, clone!(
            view => move |string| {
            check_regex(&string.to_std_string(), view.filter_line_edit.static_upcast());
        }));

        let filter_add = SlotNoArgs::new(&view.filter_widget, clone!(
            parent_view => move || {
            FilterView::new(&parent_view);
        }));

        let filter_remove = SlotNoArgs::new(&view.filter_widget, clone!(
            parent_view => move || {
            if parent_view.get_ref_filters().len() > 1 {
                parent_view.filter_base_widget.layout().remove_widget(parent_view.get_ref_filters().last().unwrap().filter_widget.as_ptr());
                parent_view.get_ref_mut_filters().pop();
                parent_view.filter_table();
            }
        }));

        Self {
            filter_line_edit,
            filter_column_selector,
            filter_case_sensitive_button,
            filter_check_regex,
            filter_add,
            filter_remove,
        }
    }
}
