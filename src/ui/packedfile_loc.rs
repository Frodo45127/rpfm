// In this file are all the helper functions used by the UI when decoding Loc PackedFiles.
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::action::Action;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::menu::Menu;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::cursor::Cursor;
use qt_gui::gui_application::GuiApplication;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::slots::SlotStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::{SlotBool, SlotCInt, SlotStringRef, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::reg_exp::RegExp;
use qt_core::qt::{Orientation, CheckState, ContextMenuPolicy, ShortcutContext, SortOrder, CaseSensitivity, GlobalColor};

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use AppUI;
use Commands;
use QString;
use common::*;
use error::Result;
use ui::*;
use settings::Settings;
use settings::shortcuts::Shortcuts;

/// Struct `PackedFileLocTreeView`: contains all the stuff we need to give to the program to show a
/// `TreeView` with the data of a Loc PackedFile, allowing us to manipulate it.
pub struct PackedFileLocTreeView {
    pub slot_context_menu: SlotQtCorePointRef<'static>,
    pub slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_item_changed: SlotStandardItemMutPtr<'static>,
    pub slot_row_filter_change_text: SlotStringRef<'static>,
    pub slot_row_filter_change_column: SlotCInt<'static>,
    pub slot_row_filter_change_case_sensitive: SlotBool<'static>,
    pub slot_context_menu_add: SlotBool<'static>,
    pub slot_context_menu_insert: SlotBool<'static>,
    pub slot_context_menu_delete: SlotBool<'static>,
    pub slot_context_menu_copy: SlotBool<'static>,
    pub slot_context_menu_paste: SlotBool<'static>,
    pub slot_context_menu_paste_as_new_lines: SlotBool<'static>,
    pub slot_context_menu_import: SlotBool<'static>,
    pub slot_context_menu_export: SlotBool<'static>,
    pub slot_smart_delete: SlotBool<'static>,
}

/// Implementation of PackedFileLocTreeView.
impl PackedFileLocTreeView {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_item_changed: SlotStandardItemMutPtr::new(|_| {}),
            slot_row_filter_change_text: SlotStringRef::new(|_| {}),
            slot_row_filter_change_column: SlotCInt::new(|_| {}),
            slot_row_filter_change_case_sensitive: SlotBool::new(|_| {}),
            slot_context_menu_add: SlotBool::new(|_| {}),
            slot_context_menu_insert: SlotBool::new(|_| {}),
            slot_context_menu_delete: SlotBool::new(|_| {}),
            slot_context_menu_copy: SlotBool::new(|_| {}),
            slot_context_menu_paste: SlotBool::new(|_| {}),
            slot_context_menu_paste_as_new_lines: SlotBool::new(|_| {}),
            slot_context_menu_import: SlotBool::new(|_| {}),
            slot_context_menu_export: SlotBool::new(|_| {}),
            slot_smart_delete: SlotBool::new(|_| {}),
        }
    }

    /// This function creates a new TreeView with the PackedFile's View as father and returns a
    /// `PackedFileLocTreeView` with all his data.
    pub fn create_tree_view(
        ui_message_sender: Sender<Commands>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        packed_file_index: &usize,
    ) -> Result<Self> {

        // Send the index back to the background thread, and wait until we get a response.
        ui_message_sender.send(Commands::DecodePackedFileLoc).unwrap();
        //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();

        // Get the Loc's data from the other thread.
        let packed_file_data: LocData = match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(error) => return Err(error)
        };

        // Create the TableView.
        let table_view = TableView::new().into_raw();
        let filter_model = SortFilterProxyModel::new().into_raw();
        let model = StandardItemModel::new(()).into_raw();

        // Make the last column fill all the available space.
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Create the filter's LineEdit.
        let row_filter_line_edit = LineEdit::new(()).into_raw();
        unsafe { row_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        // Create the filter's column selector.
        let row_filter_column_selector = ComboBox::new().into_raw();
        let row_filter_column_list = StandardItemModel::new(()).into_raw();
        unsafe { row_filter_column_selector.as_mut().unwrap().set_model(row_filter_column_list as *mut AbstractItemModel); }
        unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Key")); }
        unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Text")); }

        // Create the filter's "Case Sensitive" button.
        let row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Prepare the TableView to have a Contextual Menu.
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Enable sorting the columns.
        unsafe { table_view.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view.as_mut().unwrap().sort_by_column((-1, SortOrder::Ascending)); }

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data_to_tree_view(&packed_file_data, model);

        // Configure the table to fit Loc PackedFiles.
        unsafe { table_view.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }

        // Add Table to the Grid.
        unsafe { filter_model.as_mut().unwrap().set_source_model(model as *mut AbstractItemModel); }
        unsafe { table_view.as_mut().unwrap().set_model(filter_model as *mut AbstractItemModel); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 3)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_line_edit as *mut Widget, 1, 0, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_case_sensitive_button as *mut Widget, 1, 1, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_column_selector as *mut Widget, 1, 2, 1, 1)); }

        // Build the columns.
        build_columns(table_view, model);

        // Get the settings.
        ui_message_sender.send(Commands::GetSettings).unwrap();
        let settings: Settings = match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(_) => panic!(THREADS_MESSAGE_ERROR)
        };

        // If we want to let the columns resize themselfs...
        if *settings.settings_bool.get("adjust_columns_to_content").unwrap() {
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }

        // Action to make the delete button delete contents.
        let smart_delete = Action::new(()).into_raw();

        // Create the Contextual Menu for the TableView.
        let mut context_menu = Menu::new(());
        let context_menu_add = context_menu.add_action(&QString::from_std_str("&Add Row"));
        let context_menu_insert = context_menu.add_action(&QString::from_std_str("&Insert Row"));
        let context_menu_delete = context_menu.add_action(&QString::from_std_str("&Delete Row"));
        let context_menu_copy = context_menu.add_action(&QString::from_std_str("&Copy"));

        let mut context_menu_paste_submenu = Menu::new(&QString::from_std_str("&Paste..."));
        let context_menu_paste = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste in Selection"));
        let context_menu_paste_as_new_lines = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste as New Rows"));

        let context_menu_import = context_menu.add_action(&QString::from_std_str("&Import"));
        let context_menu_export = context_menu.add_action(&QString::from_std_str("&Export"));

        // Get the current shortcuts.
        ui_message_sender.send(Commands::GetShortcuts).unwrap();
        let shortcuts: Shortcuts = match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(_) => panic!(THREADS_MESSAGE_ERROR)
        };

        // Set the shortcuts for these actions.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("add_row").unwrap()))); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("insert_row").unwrap()))); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("delete_row").unwrap()))); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("copy").unwrap()))); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("paste").unwrap()))); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("paste_as_new_row").unwrap()))); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("import_tsv").unwrap()))); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("export_tsv").unwrap()))); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.packed_files_loc.get("smart_delete").unwrap()))); }

        // Set the shortcuts to only trigger in the Table.
        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_import.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { context_menu_export.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { smart_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TableView, so the shortcuts work.
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_add); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_insert); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_delete); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste_as_new_lines); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_import); }
        unsafe { table_view.as_mut().unwrap().add_action(context_menu_export); }
        unsafe { table_view.as_mut().unwrap().add_action(smart_delete); }

        // Status Tips for the actions.
        unsafe { context_menu_add.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add an empty row at the end of the table.")); }
        unsafe { context_menu_insert.as_mut().unwrap().set_status_tip(&QString::from_std_str("Insert an empty row just above the one selected.")); }
        unsafe { context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete all the selected rows.")); }
        unsafe { context_menu_copy.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy whatever is selected to the Clipboard.")); }
        unsafe { context_menu_paste.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to paste whatever is in the Clipboard as new lines at the end of the table. Does nothing if the data is not compatible with the cell.")); }
        unsafe { context_menu_import.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a TSV file into this table, replacing all the data.")); }
        unsafe { context_menu_export.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export this table's data into a TSV file.")); }

        // Insert some separators to space the menu, and the paste submenu.
        unsafe { context_menu.insert_separator(context_menu_copy); }
        unsafe { context_menu.insert_menu(context_menu_import, context_menu_paste_submenu.into_raw()); }
        unsafe { context_menu.insert_separator(context_menu_import); }

        // Slots for the TableView...
        let slots = Self {
            slot_context_menu: SlotQtCorePointRef::new(move |_| { context_menu.exec2(&Cursor::pos()); }),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(move  |_,_| {

                    // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
                    let selection_model;
                    let selection;
                    unsafe { selection_model = table_view.as_mut().unwrap().selection_model(); }
                    unsafe { selection = selection_model.as_mut().unwrap().selected_indexes(); }

                    // If we have something selected, enable these actions.
                    if selection.count(()) > 0 {
                        unsafe {
                            context_menu_copy.as_mut().unwrap().set_enabled(true);
                            context_menu_delete.as_mut().unwrap().set_enabled(true);
                        }
                    }

                    // Otherwise, disable them.
                    else {
                        unsafe {
                            context_menu_copy.as_mut().unwrap().set_enabled(false);
                            context_menu_delete.as_mut().unwrap().set_enabled(false);
                        }
                    }
                }
            ),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_,_,_| {

                    // Tell the background thread to start saving the PackedFile.
                    ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                    // Get the new LocData to send.
                    let new_loc_data = Self::return_data_from_tree_view(model);

                    // Send the new LocData.
                    //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                    // Get the incomplete path of the edited PackedFile.
                    ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                    //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                    let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                        Ok(data) => data,
                        Err(_) => panic!(THREADS_MESSAGE_ERROR)
                    };

                    // Set the mod as "Modified".
                    *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                }
            )),
            slot_item_changed: SlotStandardItemMutPtr::new(|item| {
                unsafe { item.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Yellow)); }
            }),
            slot_row_filter_change_text: SlotStringRef::new(move |filter_text| {

                // Get the column selected.
                unsafe { filter_model.as_mut().unwrap().set_filter_key_column(row_filter_column_selector.as_mut().unwrap().current_index()); }

                // Check if the filter should be "Case Sensitive".
                let case_sensitive;
                unsafe { case_sensitive = row_filter_case_sensitive_button.as_mut().unwrap().is_checked(); }

                // Get the Regex and set his "Case Sensitivity".
                let mut reg_exp = RegExp::new(filter_text);
                if case_sensitive { reg_exp.set_case_sensitivity(CaseSensitivity::Sensitive); }
                else { reg_exp.set_case_sensitivity(CaseSensitivity::Insensitive); }

                // Filter whatever it's in that column by the text we got.
                unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&reg_exp); }
            }),
            slot_row_filter_change_column: SlotCInt::new(move |index| {

                // Get the column selected.
                unsafe { filter_model.as_mut().unwrap().set_filter_key_column(index); }

                // Check if the filter should be "Case Sensitive".
                let case_sensitive;
                unsafe { case_sensitive = row_filter_case_sensitive_button.as_mut().unwrap().is_checked(); }

                // Get the Regex and set his "Case Sensitivity".
                let mut reg_exp;
                unsafe { reg_exp = RegExp::new(&row_filter_line_edit.as_mut().unwrap().text()); }
                if case_sensitive { reg_exp.set_case_sensitivity(CaseSensitivity::Sensitive); }
                else { reg_exp.set_case_sensitivity(CaseSensitivity::Insensitive); }

                // Filter whatever it's in that column by the text we got.
                unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&reg_exp); }
            }),
            slot_row_filter_change_case_sensitive: SlotBool::new(move |case_sensitive| {

                // Get the column selected.
                unsafe { filter_model.as_mut().unwrap().set_filter_key_column(row_filter_column_selector.as_mut().unwrap().current_index()); }

                // Get the Regex and set his "Case Sensitivity".
                let mut reg_exp;
                unsafe { reg_exp = RegExp::new(&row_filter_line_edit.as_mut().unwrap().text()); }
                if case_sensitive { reg_exp.set_case_sensitivity(CaseSensitivity::Sensitive); }
                else { reg_exp.set_case_sensitivity(CaseSensitivity::Insensitive); }

                // Filter whatever it's in that column by the text we got.
                unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&reg_exp); }
            }),
            slot_context_menu_add: SlotBool::new(move |_| {

                // Create a new list of StandardItem.
                let mut qlist = ListStandardItemMutPtr::new(());

                // Create an empty row.
                let mut key = StandardItem::new(&QString::from_std_str(""));
                let mut text = StandardItem::new(&QString::from_std_str(""));
                let mut tooltip = StandardItem::new(());
                tooltip.set_editable(false);
                tooltip.set_checkable(true);
                tooltip.set_check_state(CheckState::Checked);

                // Paint the cells.
                key.set_background(&Brush::new(GlobalColor::Green));
                text.set_background(&Brush::new(GlobalColor::Green));
                tooltip.set_background(&Brush::new(GlobalColor::Green));

                // Add an empty row to the list.
                unsafe { qlist.append_unsafe(&key.into_raw()); }
                unsafe { qlist.append_unsafe(&text.into_raw()); }
                unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

                // Append the new row.
                unsafe { model.as_mut().unwrap().append_row(&qlist); }
            }),
            slot_context_menu_insert: SlotBool::new(move |_| {

                // Create a new list of StandardItem.
                let mut qlist = ListStandardItemMutPtr::new(());

                // Create an empty row.
                let mut key = StandardItem::new(&QString::from_std_str(""));
                let mut text = StandardItem::new(&QString::from_std_str(""));
                let mut tooltip = StandardItem::new(());
                tooltip.set_editable(false);
                tooltip.set_checkable(true);
                tooltip.set_check_state(CheckState::Checked);

                // Paint the cells.
                key.set_background(&Brush::new(GlobalColor::Green));
                text.set_background(&Brush::new(GlobalColor::Green));
                tooltip.set_background(&Brush::new(GlobalColor::Green));

                // Add an empty row to the list.
                unsafe { qlist.append_unsafe(&key.into_raw()); }
                unsafe { qlist.append_unsafe(&text.into_raw()); }
                unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

                // Get the current row.
                let selection;
                unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                // If there is any row selected...
                if selection.indexes().count(()) > 0 {

                    // Get the current filtered ModelIndex.
                    let model_index_list = selection.indexes();
                    let model_index = model_index_list.at(0);

                    // Check if the ModelIndex is valid. Otherwise this can crash.
                    if model_index.is_valid() {

                        // Get the source ModelIndex for our filtered ModelIndex.
                        let model_index_source;
                        unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                        // Get the current row.
                        let row = model_index_source.row();

                        // Insert the new row where the current one is.
                        unsafe { model.as_mut().unwrap().insert_row((row, &qlist)); }
                    }
                }

                // Otherwise, just do the same the "Add Row" do.
                else { unsafe { model.as_mut().unwrap().append_row(&qlist); } }
            }),
            slot_context_menu_delete: SlotBool::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_| {

                    // Get the current selection.
                    let selection;
                    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                    let indexes = selection.indexes();

                    // Get all the selected rows.
                    let mut rows: Vec<i32> = vec![];
                    for index in 0..indexes.size() {

                        // Get the ModelIndex.
                        let model_index = indexes.at(index);

                        // Check if the ModelIndex is valid. Otherwise this can crash.
                        if model_index.is_valid() {

                            // Get the source ModelIndex for our filtered ModelIndex.
                            let model_index_source;
                            unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                            // Get the current row.
                            let row = model_index_source.row();

                            // Add it to the list.
                            rows.push(row);
                        }
                    }

                    // Dedup the list and reverse it.
                    rows.sort();
                    rows.dedup();
                    rows.reverse();

                    // Delete evey selected row. '_y' is ignorable.
                    let mut _y = false;
                    unsafe { rows.iter().for_each(|x| _y = model.as_mut().unwrap().remove_rows((*x, 1))); }

                    // If we deleted anything, save the data.
                    if rows.len() > 0 {

                        // Tell the background thread to start saving the PackedFile.
                        ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                        // Get the new LocData to send.
                        let new_loc_data = Self::return_data_from_tree_view(model);

                        // Send the new LocData.
                        //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                        // Get the incomplete path of the edited PackedFile.
                        ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                        //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                        let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                            Ok(data) => data,
                            Err(_) => panic!(THREADS_MESSAGE_ERROR)
                        };

                        // Set the mod as "Modified".
                        *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                    }
                }
            )),
            slot_context_menu_copy: SlotBool::new(move |_| {

                // Create a string to keep all the values in a TSV format (x\tx\tx).
                let mut copy = String::new();

                // Get the current selection.
                let selection;
                unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                let indexes = selection.indexes();

                // Create a variable to check the row of the model_index.
                let mut row = 0;

                // For each selected index...
                for (cycle, index) in (0..indexes.count(())).enumerate() {

                    // Get his filtered ModelIndex.
                    let model_index = indexes.at(index);

                    // Check if the ModelIndex is valid. Otherwise this can crash.
                    if model_index.is_valid() {

                        // Get the source ModelIndex for our filtered ModelIndex.
                        let model_index_source;
                        unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                        // Get his StandardItem.
                        let standard_item;
                        unsafe { standard_item = model.as_mut().unwrap().item_from_index(&model_index_source); }

                        // If this is the first time we loop, get the row.
                        if cycle == 0 { row = model_index_source.row(); }

                        // Otherwise, if our current row is different than our last row...
                        else if model_index_source.row() != row {

                            // Replace the last \t with a \n
                            copy.pop();
                            copy.push('\n');

                            // Update the row.
                            row = model_index_source.row();
                        }

                        unsafe {

                            // If it's checkable, we need to get a bool.
                            if standard_item.as_mut().unwrap().is_checkable() {

                                // Turn his CheckState into a bool and add it to the copy string.
                                if standard_item.as_mut().unwrap().check_state() == CheckState::Checked { copy.push_str("true"); }
                                else {copy.push_str("false"); }
                            }

                            // Otherwise, it's a string.
                            else {

                                // Get his text and push them to the copy string.
                                copy.push_str(&QString::to_std_string(&standard_item.as_mut().unwrap().text()));
                            }
                        }

                        // Add a \t to separate fields except if it's the last field.
                        if index < (indexes.count(()) - 1) { copy.push('\t'); }
                    }
                }

                // Put the baby into the oven.
                unsafe { GuiApplication::clipboard().as_mut().unwrap().set_text(&QString::from_std_str(copy)); }
            }),
            slot_context_menu_paste: SlotBool::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_| {

                    // If whatever it's in the Clipboard is pasteable in our selection...
                    if check_clipboard(table_view, model, filter_model) {

                        // Get the clipboard.
                        let clipboard = GuiApplication::clipboard();

                        // Get the current selection.
                        let selection;
                        unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                        let indexes = selection.indexes();

                        // Get the text from the clipboard.
                        let mut text;
                        unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

                        // If the text ends in \n, remove it. Excel things.
                        if text.ends_with('\n') { text.pop(); }

                        // We don't use newlines, so replace them with '\t'.
                        let text = text.replace('\n', "\t");

                        // Split the text into individual strings.
                        let text = text.split('\t').collect::<Vec<&str>>();

                        // Vector to store the selected items.
                        let mut items = vec![];

                        // For each selected index...
                        for index in 0..indexes.count(()) {

                            // Get the filtered ModelIndex.
                            let model_index = indexes.at(index);

                            // Check if the ModelIndex is valid. Otherwise this can crash.
                            if model_index.is_valid() {

                                // Get the source ModelIndex for our filtered ModelIndex.
                                let model_index_source;
                                unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                                // Get his StandardItem and add it to the Vector.
                                unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index_source)); }
                            }
                        }

                        // Zip together both vectors.
                        let data = items.iter().zip(text);

                        // For each cell we have...
                        for cell in data.clone() {

                            unsafe {

                                // If it's checkable, we need to check or uncheck the cell.
                                if cell.0.as_mut().unwrap().is_checkable() {
                                    if cell.1 == "true" { cell.0.as_mut().unwrap().set_check_state(CheckState::Checked); }
                                    else { cell.0.as_mut().unwrap().set_check_state(CheckState::Unchecked); }
                                }

                                // Otherwise, it's just a string.
                                else { cell.0.as_mut().unwrap().set_text(&QString::from_std_str(cell.1)); }

                                // Paint the cells.
                                cell.0.as_mut().unwrap().set_background(&Brush::new(GlobalColor::Yellow));
                            }
                        }

                        // If we pasted anything, save.
                        if data.count() > 0 {

                            // Tell the background thread to start saving the PackedFile.
                            ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                            // Get the new LocData to send.
                            let new_loc_data = Self::return_data_from_tree_view(model);

                            // Send the new LocData.
                            //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                            // Get the incomplete path of the edited PackedFile.
                            ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                            //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                            let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                                Ok(data) => data,
                                Err(_) => panic!(THREADS_MESSAGE_ERROR)
                            };

                            // Set the mod as "Modified".
                            *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                        }
                    }
                }
            )),

            slot_context_menu_paste_as_new_lines: SlotBool::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_| {

                    // If whatever it's in the Clipboard is pasteable i...
                    if check_clipboard_append_rows() {

                        // Get the clipboard.
                        let clipboard = GuiApplication::clipboard();

                        // Get the text from the clipboard.
                        let mut text;
                        unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

                        // If the text ends in \n, remove it. Excel things.
                        if text.ends_with('\n') { text.pop(); }

                        // We don't use newlines, so replace them with '\t'.
                        let text = text.replace('\n', "\t");

                        // Split the text into individual strings.
                        let text = text.split('\t').collect::<Vec<&str>>();

                        // Get the index for the column and row.
                        let mut column = 0;

                        // Create a new list of StandardItem.
                        let mut qlist = ListStandardItemMutPtr::new(());

                        // For each text we have to paste...
                        for cell in &text {

                            // Create the item to add to the row.
                            let mut item = StandardItem::new(());

                            // If we are adding the last column, use a bool.
                            if column == 2 {
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(if *cell == "true" { CheckState::Checked } else { CheckState::Unchecked });
                                item.set_background(&Brush::new(GlobalColor::Green));
                            }

                            // Otherwise, create a normal cell.
                            else {
                                item.set_text(&QString::from_std_str(cell));
                                item.set_background(&Brush::new(GlobalColor::Green));
                            }

                            // Add the item to the list.
                            unsafe { qlist.append_unsafe(&item.into_raw()); }

                            // If we are in the last column...
                            if column == 2 {

                                // Append the list to the Table.
                                unsafe { model.as_mut().unwrap().append_row(&qlist); }

                                // Reset the list.
                                qlist = ListStandardItemMutPtr::new(());

                                // Reset the column count.
                                column = 0;
                            }

                            // Otherwise, increase the column count.
                            else { column += 1; }
                        }

                        // If the last list was incomplete...
                        if column != 0 {

                            // If we lack Text and Tooltip.
                            if column == 1 {

                                // Add the text column.
                                let mut item = StandardItem::new(&QString::from_std_str(""));
                                item.set_background(&Brush::new(GlobalColor::Green));
                                unsafe { qlist.append_unsafe(&item.into_raw()); }

                                // Add the tooltip column.
                                let mut item = StandardItem::new(());
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(CheckState::Checked);
                                item.set_background(&Brush::new(GlobalColor::Green));
                                unsafe { qlist.append_unsafe(&item.into_raw()); }
                            }

                            // Otherwise, we just lack tooltip.
                            else {

                                // Add the tooltip column.
                                let mut item = StandardItem::new(());
                                item.set_editable(false);
                                item.set_checkable(true);
                                item.set_check_state(CheckState::Checked);
                                item.set_background(&Brush::new(GlobalColor::Green));
                                unsafe { qlist.append_unsafe(&item.into_raw()); }
                            }

                            // Append the list to the Table.
                            unsafe { model.as_mut().unwrap().append_row(&qlist); }
                        }

                        // If we pasted anything, save.
                        if !text.is_empty() {

                            // Tell the background thread to start saving the PackedFile.
                            ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                            // Get the new LocData to send.
                            let new_loc_data = Self::return_data_from_tree_view(model);

                            // Send the new LocData.
                            //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                            // Get the incomplete path of the edited PackedFile.
                            ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                            //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                            let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                                Ok(data) => data,
                                Err(_) => panic!(THREADS_MESSAGE_ERROR)
                            };

                            // Set the mod as "Modified".
                            *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                        }
                    }
                }
            )),

            slot_context_menu_import: SlotBool::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_| {

                    // Create the FileDialog to get the PackFile to open.
                    let mut file_dialog;
                    unsafe { file_dialog = FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Select TSV File to Import..."),
                    )); }

                    // Filter it so it only shows TSV Files.
                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Get the path of the selected file and turn it in a Rust's PathBuf.
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        // Tell the background thread to start importing the TSV.
                        ui_message_sender.send(Commands::ImportTSVPackedFileLoc).unwrap();
                        //TODO sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                        // Receive the new data to load in the TableView, or an error.
                        match check_message_validity_recv(&receiver_qt) {

                            // If the importing was succesful, load the data into the Table.
                            Ok(new_loc_data) => Self::load_data_to_tree_view(&new_loc_data, model),

                            // If there was an error, report it.
                            Err(error) => return show_dialog(app_ui.window, false, error.kind()),
                        }

                        // Build the columns.
                        build_columns(table_view, model);

                        // Get the settings.
                        ui_message_sender.send(Commands::GetSettings).unwrap();
                        let settings: Settings = match check_message_validity_recv(&receiver_qt) {
                            Ok(data) => data,
                            Err(_) => panic!(THREADS_MESSAGE_ERROR)
                        };

                        // If we want to let the columns resize themselfs...
                        if *settings.settings_bool.get("adjust_columns_to_content").unwrap() {
                            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                        }

                        // Tell the background thread to start saving the PackedFile.
                        ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                        // Get the new LocData to send.
                        let new_loc_data = Self::return_data_from_tree_view(model);

                        // Send the new LocData.
                        //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                        // Get the incomplete path of the edited PackedFile.
                        ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                        //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                        let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                            Ok(data) => data,
                            Err(_) => panic!(THREADS_MESSAGE_ERROR)
                        };

                        // Set the mod as "Modified".
                        *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                    }
                }
            )),
            slot_context_menu_export: SlotBool::new(clone!(
                app_ui,
                ui_message_sender,
                receiver_qt => move |_| {

                    // Create a File Chooser to get the destination path.
                    let mut file_dialog;
                    unsafe { file_dialog = FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Export TSV File..."),
                    )); }

                    // Set it to save mode.
                    file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);

                    // Ask for confirmation in case of overwrite.
                    file_dialog.set_confirm_overwrite(true);

                    // Filter it so it only shows TSV Files.
                    file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));

                    // Set the default suffix to ".tsv", in case we forgot to write it.
                    file_dialog.set_default_suffix(&QString::from_std_str("tsv"));

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Get the path of the selected file and turn it in a Rust's PathBuf.
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        // Tell the background thread to start exporting the TSV.
                        ui_message_sender.send(Commands::ExportTSVPackedFileLoc).unwrap();
                        //TODO sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                        // Receive the result of the exporting.
                        match check_message_validity_recv(&receiver_qt) {

                            // If the exporting was succesful, report it.
                            Ok(success) => {
                                let success: String = success;
                                return show_dialog(app_ui.window, true, success);
                            }

                            // If there was an error, report it.
                            Err(error) => return show_dialog(app_ui.window, false, error.kind()),
                        }
                    }
                }
            )),
            slot_smart_delete: SlotBool::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move |_| {

                    // Get the current selection.
                    let selection;
                    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                    let indexes = selection.indexes();

                    // Get all the cells selected, separated by rows.
                    let mut cells: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
                    for index in 0..indexes.size() {

                        // Get the ModelIndex.
                        let model_index = indexes.at(index);

                        // Check if the ModelIndex is valid. Otherwise this can crash.
                        if model_index.is_valid() {

                            // Get the source ModelIndex for our filtered ModelIndex.
                            let model_index_source;
                            unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                            // Get the current row and column.
                            let row = model_index_source.row();
                            let column = model_index_source.column();

                            // Check if we have any cell in that row and add/insert the new one.
                            let mut x = false;
                            match cells.get_mut(&row) {
                                Some(cells) => cells.push(column),
                                None => { x = true },
                            }
                            if x { cells.insert(row, vec![column]); }
                        }
                    }

                    for (key, values) in cells.iter().rev() {
                        if values.len() == 3 { unsafe { model.as_mut().unwrap().remove_rows((*key, 1)); } }
                        else { 
                            for column in values {

                                let item;
                                unsafe { item = model.as_mut().unwrap().item((*key, *column)); }

                                unsafe { if item.as_mut().unwrap().is_checkable() { item.as_mut().unwrap().set_check_state(CheckState::Unchecked); }
                                else { item.as_mut().unwrap().set_text(&QString::from_std_str("")); } }
                            }
                        }
                    }

                    // If we deleted anything, save the data.
                    if !cells.is_empty() {

                        // Tell the background thread to start saving the PackedFile.
                        ui_message_sender.send(Commands::EncodePackedFileLoc).unwrap();

                        // Get the new LocData to send.
                        let new_loc_data = Self::return_data_from_tree_view(model);

                        // Send the new LocData.
                        //TODO sender_qt_data.send(serde_json::to_vec(&(new_loc_data, packed_file_index)).map_err(From::from)).unwrap();

                        // Get the incomplete path of the edited PackedFile.
                        ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                        //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                        let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                            Ok(data) => data,
                            Err(_) => panic!(THREADS_MESSAGE_ERROR)
                        };

                        // Set the mod as "Modified".
                        *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                    }
                }
            )),
        };

        // Actions for the TableView...
        unsafe { (table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_context_menu); }
        unsafe { model.as_mut().unwrap().signals().data_changed().connect(&slots.save_changes); }
        unsafe { model.as_mut().unwrap().signals().item_changed().connect(&slots.slot_item_changed); }
        unsafe { context_menu_add.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_add); }
        unsafe { context_menu_insert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_insert); }
        unsafe { context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_delete); }
        unsafe { context_menu_copy.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy); }
        unsafe { context_menu_paste.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste); }
        unsafe { context_menu_paste_as_new_lines.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste_as_new_lines); }
        unsafe { context_menu_import.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_import); }
        unsafe { context_menu_export.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_export); }

        unsafe { smart_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_smart_delete); }

        // Trigger the filter whenever the "filtered" text changes, the "filtered" column changes or the "Case Sensitive" button changes.
        unsafe { row_filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slots.slot_row_filter_change_text); }
        unsafe { row_filter_column_selector.as_mut().unwrap().signals().current_index_changed_c_int().connect(&slots.slot_row_filter_change_column); }
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slots.slot_row_filter_change_case_sensitive); }

        // Initial states for the Contextual Menu Actions.
        unsafe {
            context_menu_add.as_mut().unwrap().set_enabled(true);
            context_menu_insert.as_mut().unwrap().set_enabled(true);
            context_menu_delete.as_mut().unwrap().set_enabled(false);
            context_menu_copy.as_mut().unwrap().set_enabled(false);
            context_menu_paste.as_mut().unwrap().set_enabled(true);
            context_menu_paste_as_new_lines.as_mut().unwrap().set_enabled(true);
            context_menu_import.as_mut().unwrap().set_enabled(true);
            context_menu_export.as_mut().unwrap().set_enabled(true);
        }

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

        // Return the slots to keep them as hostages.
        return Ok(slots)
    }

    /// This function loads the data from a LocData into a TableView.
    pub fn load_data_to_tree_view(
        packed_file_data: &LocData,
        model: *mut StandardItemModel,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        unsafe { model.as_mut().unwrap().clear(); }

        // Then we add every line to the ListStore.
        for entry in &packed_file_data.entries {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(&entry.key));
            let text = StandardItem::new(&QString::from_std_str(&entry.text));
            let mut tooltip = StandardItem::new(());
            tooltip.set_editable(false);
            tooltip.set_checkable(true);
            tooltip.set_check_state(if entry.tooltip { CheckState::Checked } else { CheckState::Unchecked });

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }
            unsafe { qlist.append_unsafe(&text.into_raw()); }
            unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If there are no entries, add an empty row with default values, so Qt shows the table anyway.
        if packed_file_data.entries.len() == 0 {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let key = StandardItem::new(&QString::from_std_str(""));
            let text = StandardItem::new(&QString::from_std_str(""));
            let mut tooltip = StandardItem::new(());
            tooltip.set_editable(false);
            tooltip.set_checkable(true);
            tooltip.set_check_state(CheckState::Checked);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&key.into_raw()); }
            unsafe { qlist.append_unsafe(&text.into_raw()); }
            unsafe { qlist.append_unsafe(&tooltip.into_raw()); }

            // Just append a new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }

            // Remove the row, so the columns stay.
            unsafe { model.as_mut().unwrap().remove_rows((0, 1)); }
        }
    }

    /// This function returns a LocData with all the stuff in the table.
    pub fn return_data_from_tree_view(
        model: *mut StandardItemModel,
    ) -> LocData {

        // Create an empty `LocData`.
        let mut loc_data = LocData::new();

        // Get the amount of rows we have.
        let rows;
        unsafe { rows = model.as_mut().unwrap().row_count(()); }

        // For each row we have...
        for row in 0..rows {

            // Make a new entry with the data from the `ListStore`, and push it to our new `LocData`.
            unsafe {
                loc_data.entries.push(
                    LocEntry::new(
                        QString::to_std_string(&model.as_mut().unwrap().item((row as i32, 0)).as_mut().unwrap().text()),
                        QString::to_std_string(&model.as_mut().unwrap().item((row as i32, 1)).as_mut().unwrap().text()),
                        if model.as_mut().unwrap().item((row as i32, 2)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false },
                    )
                );
            }
        }

        // Return the new LocData.
        loc_data
    }
}

/// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
/// His intended use is for just after we reload the data to the table.
fn build_columns(
    table_view: *mut TableView,
    model: *mut StandardItemModel
) {

    // Fix the columns titles.
    unsafe { model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Key")))); }
    unsafe { model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Text")))); }
    unsafe { model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Tooltip")))); }

    unsafe { table_view.as_mut().unwrap().set_column_width(0, 450); }
    unsafe { table_view.as_mut().unwrap().set_column_width(1, 450); }
    unsafe { table_view.as_mut().unwrap().set_column_width(2, 60); }
}

/// This function checks if the data in the clipboard is suitable for the selected Items.
fn check_clipboard(
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel
) -> bool {

    // Get the clipboard.
    let clipboard = GuiApplication::clipboard();

    // Get the current selection.
    let selection;
    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
    let indexes = selection.indexes();

    // Get the text from the clipboard.
    let mut text;
    unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

    // If the text ends in \n, remove it. Excel things.
    if text.ends_with('\n') { text.pop(); }

    // We don't use newlines, so replace them with '\t'.
    let text = text.replace('\n', "\t");

    // Split the text into individual strings.
    let text = text.split('\t').collect::<Vec<&str>>();

    // Vector to store the selected items.
    let mut items = vec![];

    // For each selected index...
    for index in 0..indexes.count(()) {

        // Get the filtered ModelIndex.
        let model_index = indexes.at(index);

        // Check if the ModelIndex is valid. Otherwise this can crash.
        if model_index.is_valid() {

            // Get the source ModelIndex for our filtered ModelIndex.
            let model_index_source;
            unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

            // Get his StandardItem and add it to the Vector.
            unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index_source)); }
        }
    }

    // Zip together both vectors.
    let data = items.iter().zip(text);

    // For each cell we have...
    for cell in data {

        unsafe {

            // If it's checkable, we need to see if his text it's a bool.
            if cell.0.as_mut().unwrap().is_checkable() {
                if cell.1 == "true" || cell.1 == "false" { continue } else { return false }
            }

            // Otherwise, it's just a string.
            else { continue }
        }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}

/// This function checks if the data in the clipboard is suitable to be appended as rows at the end of the Table.
fn check_clipboard_append_rows() -> bool {

    // Get the clipboard.
    let clipboard = GuiApplication::clipboard();

    // Get the text from the clipboard.
    let mut text;
    unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

    // If the text ends in \n, remove it. Excel things.
    if text.ends_with('\n') { text.pop(); }

    // We don't use newlines, so replace them with '\t'.
    let text = text.replace('\n', "\t");

    // Split the text into individual strings.
    let text = text.split('\t').collect::<Vec<&str>>();

    // Get the index for the column.
    let mut column = 0;

    // For each text we have to paste...
    for cell in text {

        // If the column is 2, ensure it's a boolean.
        if column == 2 {
            if cell != "true" && cell != "false" { return false }
        }

        // Reset or increase the column count, if needed.
        if column == 2 { column = 0; } else { column += 1; }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}
