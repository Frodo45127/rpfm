// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate failure;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::widget::Widget;
use qt_widgets::table_view::TableView;
use qt_widgets::menu::Menu;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::text_edit::TextEdit;
use qt_widgets::frame::Frame;
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::{HeaderView, ResizeMode};
use qt_widgets::scroll_bar::ScrollBar;
use qt_widgets::abstract_item_view::{AbstractItemView, EditTrigger, SelectionMode};

use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::cursor::Cursor;
use qt_gui::gui_application::GuiApplication;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::key_sequence::KeySequence;
use qt_gui::font::{Font, StyleHint };
use qt_gui::font_metrics::FontMetrics;

use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::event_loop::EventLoop;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::{SlotBool, SlotCInt, SlotStringRef, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::reg_exp::RegExp;
use qt_core::qt::{Orientation, CheckState, ContextMenuPolicy, ShortcutContext, SortOrder, CaseSensitivity};

use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};

use AppUI;
use ui::*;
use packfile::packfile::PackedFile;
use packedfile::db::schemas::*;
use settings::{GameInfo, GameSelected};

/// Struct `PackedFileDBTreeView`: contains all the stuff we need to give to the program to show a
/// TableView with the data of a DB PackedFile, allowing us to manipulate it.
pub struct PackedFileDBTreeView {
    pub slot_context_menu: SlotQtCorePointRef<'static>,
    pub slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_row_filter_change_text: SlotStringRef<'static>,
    pub slot_row_filter_change_column: SlotCInt<'static>,
    pub slot_row_filter_change_case_sensitive: SlotBool<'static>,
    pub slot_context_menu_add: SlotBool<'static>,
    pub slot_context_menu_insert: SlotBool<'static>,
    pub slot_context_menu_delete: SlotBool<'static>,
    pub slot_context_menu_clone: SlotBool<'static>,
    pub slot_context_menu_copy: SlotBool<'static>,
    pub slot_context_menu_paste: SlotBool<'static>,
    pub slot_context_menu_import: SlotBool<'static>,
    pub slot_context_menu_export: SlotBool<'static>,
}

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
pub struct PackedFileDBDecoder {
    pub slot_hex_view_scroll_sync: SlotCInt<'static>,
    pub slot_use_this_bool: SlotNoArgs<'static>,
    pub slot_use_this_float: SlotNoArgs<'static>,
    pub slot_use_this_integer: SlotNoArgs<'static>,
    pub slot_use_this_long_integer: SlotNoArgs<'static>,
    pub slot_use_this_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_string_u16: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u16: SlotNoArgs<'static>,
    pub slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_context_menu_move_up: SlotBool<'static>,
    pub slot_table_view_context_menu_move_down: SlotBool<'static>,
    pub slot_table_view_context_menu_delete: SlotBool<'static>,
    pub slot_remove_all_fields: SlotNoArgs<'static>,
    pub slot_save_definition: SlotNoArgs<'static>,
    pub slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_old_versions_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_old_versions_context_menu_load: SlotBool<'static>,
    pub slot_table_view_old_versions_context_menu_delete: SlotBool<'static>,
}

/// Struct PackedFileDBDecoderStuff: contains all the ui things from the decoder view, so we can pass the easely.
#[derive(Copy, Clone)]
pub struct PackedFileDBDecoderStuff {
    pub hex_view_index: *mut TextEdit,
    pub hex_view_raw: *mut TextEdit,
    pub hex_view_decoded: *mut TextEdit,
    pub table_view: *mut TableView,
    pub table_model: *mut StandardItemModel,

    pub bool_line_edit: *mut LineEdit,
    pub float_line_edit: *mut LineEdit,
    pub integer_line_edit: *mut LineEdit,
    pub long_integer_line_edit: *mut LineEdit,
    pub string_u8_line_edit: *mut LineEdit,
    pub string_u16_line_edit: *mut LineEdit,
    pub optional_string_u8_line_edit: *mut LineEdit,
    pub optional_string_u16_line_edit: *mut LineEdit,

    pub bool_button: *mut PushButton,
    pub float_button: *mut PushButton,
    pub integer_button: *mut PushButton,
    pub long_integer_button: *mut PushButton,
    pub string_u8_button: *mut PushButton,
    pub string_u16_button: *mut PushButton,
    pub optional_string_u8_button: *mut PushButton,
    pub optional_string_u16_button: *mut PushButton,

    pub table_info_type_decoded_label: *mut Label,
    pub table_info_version_decoded_label: *mut Label,
    pub table_info_entry_count_decoded_label: *mut Label,

    pub table_view_old_versions: *mut TableView,
    pub table_model_old_versions: *mut StandardItemModel,

    pub clear_definition_button: *mut PushButton,
    pub save_button: *mut PushButton,

    pub table_view_context_menu: *mut Menu,
    pub table_view_context_menu_move_up: *mut Action,
    pub table_view_context_menu_move_down: *mut Action,
    pub table_view_context_menu_delete: *mut Action,

    pub table_view_old_versions_context_menu: *mut Menu,
    pub table_view_old_versions_context_menu_load: *mut Action,
    pub table_view_old_versions_context_menu_delete: *mut Action,
}

/// Struct PackedFileDBDecoderStuffNonUI: contains data needed for the decoder to properly work.
#[derive(Clone)]
pub struct PackedFileDBDecoderStuffNonUI {
    pub packed_file: PackedFile,
    pub initial_index: usize,
    pub header: DBHeader,
}

/// Implementation of PackedFileDBTreeView.
impl PackedFileDBTreeView {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            save_changes: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_row_filter_change_text: SlotStringRef::new(|_| {}),
            slot_row_filter_change_column: SlotCInt::new(|_| {}),
            slot_row_filter_change_case_sensitive: SlotBool::new(|_| {}),
            slot_context_menu_add: SlotBool::new(|_| {}),
            slot_context_menu_insert: SlotBool::new(|_| {}),
            slot_context_menu_delete: SlotBool::new(|_| {}),
            slot_context_menu_clone: SlotBool::new(|_| {}),
            slot_context_menu_copy: SlotBool::new(|_| {}),
            slot_context_menu_paste: SlotBool::new(|_| {}),
            slot_context_menu_import: SlotBool::new(|_| {}),
            slot_context_menu_export: SlotBool::new(|_| {}),
        }
    }

    /// This function creates a new Table with the PackedFile's View as father and returns a
    /// `PackedFileDBTreeView` with all his data.
    pub fn create_table_view(
        sender_qt: Sender<&'static str>,
        sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        packed_file_index: &usize,
    ) -> Result<Self, Error> {

        // Send the index back to the background thread, and wait until we get a response.
        sender_qt.send("decode_packed_file_db").unwrap();
        sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();

        // Prepare the event loop, so we don't hang the UI while the background thread is working.
        let mut event_loop = EventLoop::new();

        // Disable the Main Window (so we can't do other stuff).
        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

        // Until we receive a response from the worker thread...
        loop {

            // When we finally receive a response...
            if let Ok(data) = receiver_qt.borrow().try_recv() {

                // Check what the result of the patching process was.
                match data {

                    // In case of success, we get the data and build the UI for it.
                    Ok(data) => {

                        // Get the DB's data.
                        let mut packed_file_data: DBData = serde_json::from_slice(&data).unwrap();

                        // Create the TableView.
                        let mut table_view = TableView::new().into_raw();
                        let mut filter_model = SortFilterProxyModel::new().into_raw();
                        let mut model = StandardItemModel::new(()).into_raw();

                        // Create the filter's LineEdit.
                        let mut row_filter_line_edit = LineEdit::new(()).into_raw();
                        unsafe { row_filter_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

                        // Create the filter's column selector.
                        let mut row_filter_column_selector = ComboBox::new().into_raw();
                        let mut row_filter_column_list = StandardItemModel::new(()).into_raw();
                        unsafe { row_filter_column_selector.as_mut().unwrap().set_model(row_filter_column_list as *mut AbstractItemModel); }
                        for column in &packed_file_data.table_definition.fields {
                            let mut name = clean_column_names(&column.field_name);
                            unsafe { row_filter_column_selector.as_mut().unwrap().add_item(&QString::from_std_str(&name)); }
                        }

                        // Create the filter's "Case Sensitive" button.
                        let mut row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
                        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

                        // Prepare the TableView to have a Contextual Menu.
                        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

                        // Enable sorting the columns.
                        unsafe { table_view.as_mut().unwrap().set_sorting_enabled(true); }
                        unsafe { table_view.as_mut().unwrap().sort_by_column((-1, SortOrder::Ascending)); }

                        // Load the data to the Table. For some reason, if we do this after setting the titles of
                        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
                        Self::load_data_to_table_view(&packed_file_data, model);

                        // Add Table to the Grid.
                        unsafe { filter_model.as_mut().unwrap().set_source_model(model as *mut AbstractItemModel); }
                        unsafe { table_view.as_mut().unwrap().set_model(filter_model as *mut AbstractItemModel); }
                        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 3)); }
                        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_line_edit as *mut Widget, 1, 0, 1, 1)); }
                        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_case_sensitive_button as *mut Widget, 1, 1, 1, 1)); }
                        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((row_filter_column_selector as *mut Widget, 1, 2, 1, 1)); }

                        // Build the Column's "Data".
                        build_columns(&packed_file_data.table_definition, table_view, model);

                        // Set both headers visible.
                        unsafe { table_view.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
                        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }

                        // Create the Contextual Menu for the TableView.
                        let mut context_menu = Menu::new(());
                        let context_menu_add = context_menu.add_action(&QString::from_std_str("&Add Row"));
                        let context_menu_insert = context_menu.add_action(&QString::from_std_str("&Insert Row"));
                        let context_menu_delete = context_menu.add_action(&QString::from_std_str("&Delete Row"));
                        let context_menu_clone = context_menu.add_action(&QString::from_std_str("&Clone"));
                        let context_menu_copy = context_menu.add_action(&QString::from_std_str("&Copy"));
                        let context_menu_paste = context_menu.add_action(&QString::from_std_str("&Paste"));
                        let context_menu_import = context_menu.add_action(&QString::from_std_str("&Import"));
                        let context_menu_export = context_menu.add_action(&QString::from_std_str("&Export"));

                        // Set the shortcuts for these actions.
                        unsafe { context_menu_add.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+shift+a"))); }
                        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+i"))); }
                        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+del"))); }
                        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+d"))); }
                        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+c"))); }
                        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+v"))); }
                        unsafe { context_menu_import.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+w"))); }
                        unsafe { context_menu_export.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+e"))); }

                        // Set the shortcuts to only trigger in the Table.
                        unsafe { context_menu_add.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_insert.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_clone.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_copy.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_paste.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_import.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
                        unsafe { context_menu_export.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

                        // Add the actions to the TableView, so the shortcuts work.
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_add); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_insert); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_delete); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_clone); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_copy); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_paste); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_import); }
                        unsafe { table_view.as_mut().unwrap().add_action(context_menu_export); }

                        // Insert some separators to space the menu.
                        unsafe { context_menu.insert_separator(context_menu_clone); }
                        unsafe { context_menu.insert_separator(context_menu_import); }

                        // Slots for the TableView...
                        let mut slots = Self {
                            slot_context_menu: SlotQtCorePointRef::new(move |_| { context_menu.exec2(&Cursor::pos()); }),
                            slot_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(move |selection,_| {

                                   // If we have something selected, enable these actions.
                                   if selection.indexes().count(()) > 0 {
                                       unsafe {
                                           context_menu_clone.as_mut().unwrap().set_enabled(true);
                                           context_menu_copy.as_mut().unwrap().set_enabled(true);
                                           context_menu_delete.as_mut().unwrap().set_enabled(true);
                                       }
                                   }

                                   // Otherwise, disable them.
                                   else {
                                       unsafe {
                                           context_menu_clone.as_mut().unwrap().set_enabled(false);
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
                                packed_file_data,
                                sender_qt,
                                sender_qt_data,
                                receiver_qt => move |_,_,_| {

                                    // Get a local copy of the data.
                                    let mut data = packed_file_data.clone();

                                    // Update the DBData with the data in the table, or report error if it fails.
                                    if let Err(error) = Self::return_data_from_table_view(&mut data, model) {
                                        return show_dialog(&app_ui, false, format!("<p>Error while trying to save the DB Table:</p><p>{}</p><p>This is probably caused by one of the fields you just changed. Please, make sure the data in that field it's of the correct type.</p>", error.cause()));
                                    };

                                    // Tell the background thread to start saving the PackedFile.
                                    sender_qt.send("encode_packed_file_db").unwrap();

                                    // Send the new DBData.
                                    sender_qt_data.send(serde_json::to_vec(&(data, packed_file_index)).map_err(From::from)).unwrap();

                                    // Set the mod as "Modified".
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui);
                                }
                            )),

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

                            slot_context_menu_add: SlotBool::new(clone!(
                                packed_file_data => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

                                        // Create a new list of StandardItem.
                                        let mut qlist = ListStandardItemMutPtr::new(());

                                        // For each field in the definition...
                                        for field in &packed_file_data.table_definition.fields {

                                            // Create a new Item.
                                            let item = match field.field_type {

                                                // This one needs a couple of changes before turning it into an item in the table.
                                                FieldType::Boolean => {
                                                    let mut item = StandardItem::new(());
                                                    item.set_editable(false);
                                                    item.set_checkable(true);
                                                    item.set_check_state(CheckState::Checked);
                                                    item
                                                }

                                                FieldType::Float => StandardItem::new(&QString::from_std_str(format!("{}", 0.0))),
                                                FieldType::Integer => StandardItem::new(&QString::from_std_str(format!("{}", 0))),
                                                FieldType::LongInteger => StandardItem::new(&QString::from_std_str(format!("{}", 0))),

                                                // All these are Strings, so it can be together.
                                                FieldType::StringU8 |
                                                FieldType::StringU16 |
                                                FieldType::OptionalStringU8 |
                                                FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                                            };

                                            // Add the item to the list.
                                            unsafe { qlist.append_unsafe(&item.into_raw()); }
                                        }

                                        // Append the new row.
                                        unsafe { model.as_mut().unwrap().append_row(&qlist); }
                                    }
                                }
                            )),
                            slot_context_menu_insert: SlotBool::new(clone!(
                                packed_file_data => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

                                        // Create a new list of StandardItem.
                                        let mut qlist = ListStandardItemMutPtr::new(());

                                        // For each field in the definition...
                                        for field in &packed_file_data.table_definition.fields {

                                            // Create a new Item.
                                            let item = match field.field_type {

                                                // This one needs a couple of changes before turning it into an item in the table.
                                                FieldType::Boolean => {
                                                    let mut item = StandardItem::new(());
                                                    item.set_editable(false);
                                                    item.set_checkable(true);
                                                    item.set_check_state(CheckState::Checked);
                                                    item
                                                }

                                                FieldType::Float => StandardItem::new(&QString::from_std_str(format!("{}", 0.0))),
                                                FieldType::Integer => StandardItem::new(&QString::from_std_str(format!("{}", 0))),
                                                FieldType::LongInteger => StandardItem::new(&QString::from_std_str(format!("{}", 0))),

                                                // All these are Strings, so it can be together.
                                                FieldType::StringU8 |
                                                FieldType::StringU16 |
                                                FieldType::OptionalStringU8 |
                                                FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                                            };

                                            // Add the item to the list.
                                            unsafe { qlist.append_unsafe(&item.into_raw()); }
                                        }

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
                                    }
                                }
                            )),
                            slot_context_menu_delete: SlotBool::new(clone!(
                                packed_file_index,
                                app_ui,
                                is_modified,
                                packed_file_data,
                                sender_qt,
                                sender_qt_data,
                                receiver_qt => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

                                        // Get the current selection.
                                        let selection;
                                        unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                        let indexes = selection.indexes();

                                        // For each selected index...
                                        for index in (0..indexes.count(())).rev() {

                                            // Get the ModelIndex.
                                            let model_index = indexes.at(index);

                                            // Check if the ModelIndex is valid. Otherwise this can crash.
                                            if model_index.is_valid() {

                                                // Get the source ModelIndex for our filtered ModelIndex.
                                                let model_index_source;
                                                unsafe {model_index_source = filter_model.as_mut().unwrap().map_to_source(&model_index); }

                                                // Get the current row.
                                                let row = model_index_source.row();

                                                // Delete it.
                                                unsafe { model.as_mut().unwrap().remove_rows((row, 1)); }

                                                // Get a local copy of the data.
                                                let mut data = packed_file_data.clone();

                                                // Update the DBData with the data in the table, or report error if it fails.
                                                if let Err(error) = Self::return_data_from_table_view(&mut data, model) {
                                                    return show_dialog(&app_ui, false, format!("<p>Error while trying to save the DB Table:</p><p>{}</p><p>This is probably caused by one of the fields you just changed. Please, make sure the data in that field it's of the correct type.</p>", error.cause()));
                                                };

                                                // Tell the background thread to start saving the PackedFile.
                                                sender_qt.send("encode_packed_file_db").unwrap();

                                                // Send the new DBData.
                                                sender_qt_data.send(serde_json::to_vec(&(data, packed_file_index)).map_err(From::from)).unwrap();

                                                // Set the mod as "Modified".
                                                *is_modified.borrow_mut() = set_modified(true, &app_ui);
                                            }
                                        }
                                    }
                                }
                            )),
                            slot_context_menu_clone: SlotBool::new(clone!(
                                packed_file_index,
                                app_ui,
                                is_modified,
                                packed_file_data,
                                sender_qt,
                                sender_qt_data,
                                receiver_qt => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

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

                                        // For each row...
                                        for row in &rows {

                                            // Create a new list of StandardItem.
                                            let mut qlist = ListStandardItemMutPtr::new(());

                                            // For each field in the definition...
                                            for column in 0..packed_file_data.table_definition.fields.len() {

                                                // Get the original item.
                                                let original_item;
                                                unsafe { original_item = model.as_mut().unwrap().item((*row, column as i32)); }

                                                // Get a clone of the item of that column.
                                                let item;
                                                unsafe { item = original_item.as_mut().unwrap().clone(); }

                                                // Depending on the column, we try to encode the data in one format or another.
                                                match packed_file_data.table_definition.fields[column as usize].field_type {

                                                    // If it's a boolean...
                                                    FieldType::Boolean => {

                                                        // Set the item as checkable and disable his editing.
                                                        unsafe { item.as_mut().unwrap().set_checkable(true); }
                                                        unsafe { item.as_mut().unwrap().set_editable(false); }

                                                        // Depending on his original state, set it as checked or unchecked.
                                                        unsafe { item.as_mut().unwrap().set_check_state(original_item.as_mut().unwrap().check_state()); }
                                                    }
                                                    _ => unsafe { item.as_mut().unwrap().set_text(&original_item.as_mut().unwrap().text()) },
                                                }

                                                // Add the item to the list.
                                                unsafe { qlist.append_unsafe(&item); }
                                            }

                                            // Insert the new row after the original one.
                                            unsafe { model.as_mut().unwrap().insert_row((row + 1, &qlist)); }
                                        }

                                        // If we cloned anything, save the data.
                                        if rows.len() > 0 {

                                            // Get a local copy of the data.
                                            let mut data = packed_file_data.clone();

                                            // Update the DBData with the data in the table, or report error if it fails.
                                            if let Err(error) = Self::return_data_from_table_view(&mut data, model) {
                                                return show_dialog(&app_ui, false, format!("<p>Error while trying to save the DB Table:</p><p>{}</p><p>This is probably caused by one of the fields you just changed. Please, make sure the data in that field it's of the correct type.</p>", error.cause()));
                                            };

                                            // Tell the background thread to start saving the PackedFile.
                                            sender_qt.send("encode_packed_file_db").unwrap();

                                            // Send the new DBData.
                                            sender_qt_data.send(serde_json::to_vec(&(data, packed_file_index)).map_err(From::from)).unwrap();

                                            // Set the mod as "Modified".
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui);
                                        }
                                    }
                                }
                            )),


                            slot_context_menu_copy: SlotBool::new(move |_| {

                                // We only do something in case the focus is in the TableView. This should stop problems with
                                // the accels working everywhere.
                                let has_focus;
                                unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                if has_focus {

                                    // Create a string to keep all the values in a TSV format (x\tx\tx).
                                    let mut copy = String::new();

                                    // Get the current selection.
                                    let selection;
                                    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                    let indexes = selection.indexes();

                                    // For each selected index...
                                    for index in 0..indexes.count(()) {

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
                                }
                            }),

                            slot_context_menu_paste: SlotBool::new(clone!(
                                packed_file_index,
                                app_ui,
                                is_modified,
                                packed_file_data,
                                sender_qt,
                                sender_qt_data,
                                receiver_qt => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

                                        // If whatever it's in the Clipboard is pasteable in our selection...
                                        if check_clipboard(&packed_file_data.table_definition, table_view, model, filter_model) {

                                            // Get the clipboard.
                                            let clipboard = GuiApplication::clipboard();

                                            // Get the current selection.
                                            let selection;
                                            unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            // Get the text from the clipboard.
                                            let text;
                                            unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

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

                                                    // Get the column of that cell.
                                                    let column = cell.0.as_mut().unwrap().index().column();

                                                    // Depending on the column, we try to encode the data in one format or another.
                                                    match packed_file_data.table_definition.fields[column as usize].field_type {
                                                        FieldType::Boolean => {
                                                            if cell.1 == "true" { cell.0.as_mut().unwrap().set_check_state(CheckState::Checked); }
                                                            else { cell.0.as_mut().unwrap().set_check_state(CheckState::Checked); }
                                                        }
                                                        _ => cell.0.as_mut().unwrap().set_text(&QString::from_std_str(cell.1)),
                                                    }
                                                }
                                            }

                                            // If we pasted anything, save.
                                            if data.count() > 0 {

                                                // Get a local copy of the data.
                                                let mut data = packed_file_data.clone();

                                                // Update the DBData with the data in the table, or report error if it fails.
                                                if let Err(error) = Self::return_data_from_table_view(&mut data, model) {
                                                    return show_dialog(&app_ui, false, format!("<p>Error while trying to save the DB Table:</p><p>{}</p><p>This is probably caused by one of the fields you just changed. Please, make sure the data in that field it's of the correct type.</p>", error.cause()));
                                                };

                                                // Tell the background thread to start saving the PackedFile.
                                                sender_qt.send("encode_packed_file_db").unwrap();

                                                // Send the new DBData.
                                                sender_qt_data.send(serde_json::to_vec(&(data, packed_file_index)).map_err(From::from)).unwrap();

                                                // Set the mod as "Modified".
                                                *is_modified.borrow_mut() = set_modified(true, &app_ui);
                                            }
                                        }
                                    }
                                }
                            )),


                            slot_context_menu_import: SlotBool::new(clone!(
                                packed_file_index,
                                app_ui,
                                is_modified,
                                packed_file_data,
                                sender_qt,
                                sender_qt_data,
                                receiver_qt => move |_| {

                                    // We only do something in case the focus is in the TableView. This should stop problems with
                                    // the accels working everywhere.
                                    let has_focus;
                                    unsafe { has_focus = table_view.as_mut().unwrap().has_focus() };
                                    if has_focus {

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
                                            sender_qt.send("import_tsv_packed_file_db").unwrap();
                                            sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                                            // Receive the new data to load in the TableView, or an error.
                                            match receiver_qt.borrow().recv().unwrap() {

                                                // If the importing was succesful, load the data into the Table.
                                                Ok(new_data) => Self::load_data_to_table_view(&serde_json::from_slice(&new_data).unwrap(), model),

                                                // If there was an error, report it.
                                                Err(error) => return show_dialog(&app_ui, false, format!("<p>Error while importing the TSV File:</p><p>{}</p>", error.cause())),
                                            }

                                            // Build the Column's "Data".
                                            build_columns(&packed_file_data.table_definition, table_view, model);

                                            // Get a local copy of the data.
                                            let mut data = packed_file_data.clone();

                                            // Update the DBData with the data in the table, or report error if it fails.
                                            if let Err(error) = Self::return_data_from_table_view(&mut data, model) {
                                                return show_dialog(&app_ui, false, format!("<p>Error while trying to save the DB Table:</p><p>{}</p><p>This is probably caused by one of the fields you just changed. Please, make sure the data in that field it's of the correct type.</p>", error.cause()));
                                            };

                                            // Tell the background thread to start saving the PackedFile.
                                            sender_qt.send("encode_packed_file_db").unwrap();

                                            // Send the new DBData.
                                            sender_qt_data.send(serde_json::to_vec(&(data, packed_file_index)).map_err(From::from)).unwrap();

                                            // Set the mod as "Modified".
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui);
                                        }
                                    }
                                }
                            )),
                            slot_context_menu_export: SlotBool::new(clone!(
                                packed_file_index,
                                app_ui,
                                is_modified,
                                sender_qt,
                                sender_qt_data,
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
                                        sender_qt.send("export_tsv_packed_file_db").unwrap();
                                        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                                        // Receive the result of the exporting.
                                        match receiver_qt.borrow().recv().unwrap() {

                                            // If the exporting was succesful, report it.
                                            Ok(success) => {
                                                let message: String = serde_json::from_slice(&success).unwrap();
                                                return show_dialog(&app_ui, true, message);
                                            }

                                            // If there was an error, report it.
                                            Err(error) => return show_dialog(&app_ui, false, format!("<p>Error while exporting the TSV File:</p><p>{}</p>", error.cause())),
                                        }
                                    }
                                }
                            )),
                        };

                        // Actions for the TableView...
                        unsafe { (table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_context_menu); }
                        unsafe { model.as_mut().unwrap().signals().data_changed().connect(&slots.save_changes); }
                        unsafe { context_menu_add.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_add); }
                        unsafe { context_menu_insert.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_insert); }
                        unsafe { context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_delete); }
                        unsafe { context_menu_clone.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_clone); }
                        unsafe { context_menu_copy.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_copy); }
                        unsafe { context_menu_paste.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_paste); }
                        unsafe { context_menu_import.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_import); }
                        unsafe { context_menu_export.as_mut().unwrap().signals().triggered().connect(&slots.slot_context_menu_export); }

                        // Trigger the filter whenever the "filtered" text changes, the "filtered" column changes or the "Case Sensitive" button changes.
                        unsafe { row_filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slots.slot_row_filter_change_text); }
                        unsafe { row_filter_column_selector.as_mut().unwrap().signals().current_index_changed_c_int().connect(&slots.slot_row_filter_change_column); }
                        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slots.slot_row_filter_change_case_sensitive); }

                        // Initial states for the Contextual Menu Actions.
                        unsafe {
                            context_menu_add.as_mut().unwrap().set_enabled(true);
                            context_menu_insert.as_mut().unwrap().set_enabled(true);
                            context_menu_delete.as_mut().unwrap().set_enabled(false);
                            context_menu_clone.as_mut().unwrap().set_enabled(false);
                            context_menu_copy.as_mut().unwrap().set_enabled(false);
                            context_menu_paste.as_mut().unwrap().set_enabled(true);
                            context_menu_import.as_mut().unwrap().set_enabled(true);
                            context_menu_export.as_mut().unwrap().set_enabled(true);
                        }

                        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
                        unsafe { table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_context_menu_enabler); }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                        // Return the slots to keep them as hostages.
                        return Ok(slots)
                    }

                    // In case of error, report the error.
                    Err(error) => return Err(error),
                }
            }

            // Keep the UI responsive.
            event_loop.process_events(());

            // Wait a bit to not saturate a CPU core.
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// This function loads the data from a LocData into a TableView.
    pub fn load_data_to_table_view(
        packed_file_data: &DBData,
        model: *mut StandardItemModel,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        unsafe { model.as_mut().unwrap().clear(); }

        // Then we add every line to the ListStore.
        for entry in &packed_file_data.entries {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // For each field we have in the definition...
            for field in entry {

                // Create a new Item.
                let item = match *field {

                    // This one needs a couple of changes before turning it into an item in the table.
                    DecodedData::Boolean(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_editable(false);
                        item.set_checkable(true);
                        item.set_check_state(if *data { CheckState::Checked } else { CheckState::Unchecked });
                        item
                    }

                    DecodedData::Float(ref data) => StandardItem::new(&QString::from_std_str(format!("{}", data))),
                    DecodedData::Integer(ref data) => StandardItem::new(&QString::from_std_str(format!("{}", data))),
                    DecodedData::LongInteger(ref data) => StandardItem::new(&QString::from_std_str(format!("{}", data))),

                    // All these are Strings, so it can be together,
                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => StandardItem::new(&QString::from_std_str(data)),
                };

                // Add the item to the list.
                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }

            // Append the new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }

        // If the table it's empty, we add an empty row.
        if packed_file_data.entries.len() == 0 {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // For each field in the definition...
            for field in &packed_file_data.table_definition.fields {

                // Create a new Item.
                let item = match field.field_type {

                    // This one needs a couple of changes before turning it into an item in the table.
                    FieldType::Boolean => {
                        let mut item = StandardItem::new(());
                        item.set_editable(false);
                        item.set_checkable(true);
                        item.set_check_state(CheckState::Checked);
                        item
                    }

                    FieldType::Float => StandardItem::new(&QString::from_std_str(format!("{}", 0.0))),
                    FieldType::Integer => StandardItem::new(&QString::from_std_str(format!("{}", 0))),
                    FieldType::LongInteger => StandardItem::new(&QString::from_std_str(format!("{}", 0))),

                    // All these are Strings, so it can be together,
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                };

                // Add the item to the list.
                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }

            // Append the new row.
            unsafe { model.as_mut().unwrap().append_row(&qlist); }
        }
    }


    /// This function returns a DBData with all the stuff in the table. This can and will fail in case
    /// the data of a field cannot be parsed to the type of that field. Beware of that.
    pub fn return_data_from_table_view(
        packed_file_data: &mut DBData,
        model: *mut StandardItemModel,
    ) -> Result<(), Error> {

        // This list is to store the new data before passing it to the DBData, just in case it fails.
        let mut new_data: Vec<Vec<DecodedData>> = vec![];

        // Get the amount of rows we have.
        let rows;
        unsafe { rows = model.as_mut().unwrap().row_count(()); }

        // For each row we have...
        for row in 0..rows {

            let mut new_row: Vec<DecodedData> = vec![];

            // For each field in that table...
            for (column, field) in packed_file_data.table_definition.fields.iter().enumerate() {

                // Create a new Item.
                let item;
                unsafe {
                    item = match field.field_type {

                        // This one needs a couple of changes before turning it into an item in the table.
                        FieldType::Boolean => DecodedData::Boolean(if model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }),

                        // Numbers need parsing, and this can fail.
                        FieldType::Float => DecodedData::Float(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text()).parse::<f32>()?),
                        FieldType::Integer => DecodedData::Integer(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text()).parse::<i32>()?),
                        FieldType::LongInteger => DecodedData::LongInteger(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text()).parse::<i64>()?),

                        // All these are just normal Strings.
                        FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.as_mut().unwrap().item((row as i32, column as i32)).as_mut().unwrap().text())),
                    };
                }

                // Add it to the list.
                new_row.push(item);
            }

            // Add it to the list of rows.
            new_data.push(new_row);
        }

        // If we reached this place, it means there has been no errors while parsing the data, so we
        // replace the old entries with the new ones.
        packed_file_data.entries = new_data;

        // Return success.
        Ok(())
    }
}

/// Implementation of PackedFileDBDecoder.
impl PackedFileDBDecoder {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_hex_view_scroll_sync: SlotCInt::new(|_| {}),
            slot_use_this_bool: SlotNoArgs::new(|| {}),
            slot_use_this_float: SlotNoArgs::new(|| {}),
            slot_use_this_integer: SlotNoArgs::new(|| {}),
            slot_use_this_long_integer: SlotNoArgs::new(|| {}),
            slot_use_this_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_string_u16: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u16: SlotNoArgs::new(|| {}),
            slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_context_menu_move_up: SlotBool::new(|_| {}),
            slot_table_view_context_menu_move_down: SlotBool::new(|_| {}),
            slot_table_view_context_menu_delete: SlotBool::new(|_| {}),
            slot_remove_all_fields: SlotNoArgs::new(|| {}),
            slot_save_definition: SlotNoArgs::new(|| {}),
            slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_old_versions_context_menu_load: SlotBool::new(|_| {}),
            slot_table_view_old_versions_context_menu_delete: SlotBool::new(|_| {}),
        }
    }

    /// This function creates the "Decoder View" with all the stuff needed to decode a table, and it
    /// returns it if it succeed. It can fail if the provided PackedFile is not a DB Table.
    pub fn create_decoder_view(
        rpfm_path: &PathBuf,
        supported_games: Vec<GameInfo>,
        sender_qt: Sender<&'static str>,
        sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
        app_ui: &AppUI,
        packed_file_index: &usize,
    ) -> Result<(Self, Font), Error> {

        //---------------------------------------------------------------------------------------//
        // Create the UI of the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Create the hex view on the left side.
        let hex_view_group = GroupBox::new(&QString::from_std_str("PackedFile's Data")).into_raw();
        let hex_view_index = TextEdit::new(()).into_raw();
        let hex_view_raw = TextEdit::new(()).into_raw();
        let hex_view_decoded = TextEdit::new(()).into_raw();
        let hex_view_layout = GridLayout::new().into_raw();
        unsafe { hex_view_group.as_mut().unwrap().set_layout(hex_view_layout as *mut Layout); }
        unsafe { hex_view_layout.as_mut().unwrap().set_spacing(1); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_index as *mut Widget, 0, 0, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_raw as *mut Widget, 0, 1, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_decoded as *mut Widget, 0, 2, 1, 1)); }

        // Set them as "ReadOnly".
        unsafe { hex_view_index.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_raw.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_read_only(true); }

        // Set his font as "Monospace", so all the lines have the same lenght.
        let mut monospace_font = Font::new(&QString::from_std_str("Monospace"));
        monospace_font.set_style_hint(StyleHint::Monospace);
        unsafe { hex_view_index.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_raw.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_font(&monospace_font); }

        // Create the TableView at the top.
        let table_view = TableView::new().into_raw();
        let table_model = StandardItemModel::new(()).into_raw();
        unsafe { table_view.as_mut().unwrap().set_model(table_model as *mut AbstractItemModel); }
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Create the Contextual Menu for the TableView.
        let mut table_view_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_context_menu_move_up = table_view_context_menu.add_action(&QString::from_std_str("Move &Up"));
        let table_view_context_menu_move_down = table_view_context_menu.add_action(&QString::from_std_str("&Move Down"));
        let table_view_context_menu_delete = table_view_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+up"))); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+down"))); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+del"))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_up); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_down); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the frames for the info.
        let decoded_fields_frame = Frame::new().into_raw();
        let info_frame = GroupBox::new(&QString::from_std_str("Table Info")).into_raw();

        // Set their layouts.
        let decoded_fields_layout = GridLayout::new().into_raw();
        let info_layout = GridLayout::new().into_raw();
        unsafe { decoded_fields_frame.as_mut().unwrap().set_layout(decoded_fields_layout as *mut Layout); }
        unsafe { info_frame.as_mut().unwrap().set_layout(info_layout as *mut Layout); }

        // Create the stuff for the decoded fields.
        let bool_label = Label::new(&QString::from_std_str("Decoded as \"Bool\":")).into_raw();
        let float_label = Label::new(&QString::from_std_str("Decoded as \"Float\":")).into_raw();
        let integer_label = Label::new(&QString::from_std_str("Decoded as \"Integer\":")).into_raw();
        let long_integer_label = Label::new(&QString::from_std_str("Decoded as \"Long Integer\":")).into_raw();
        let string_u8_label = Label::new(&QString::from_std_str("Decoded as \"String U8\":")).into_raw();
        let string_u16_label = Label::new(&QString::from_std_str("Decoded as \"String U16\":")).into_raw();
        let optional_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U8\":")).into_raw();
        let optional_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U16\":")).into_raw();

        let bool_line_edit = LineEdit::new(()).into_raw();
        let float_line_edit = LineEdit::new(()).into_raw();
        let integer_line_edit = LineEdit::new(()).into_raw();
        let long_integer_line_edit = LineEdit::new(()).into_raw();
        let string_u8_line_edit = LineEdit::new(()).into_raw();
        let string_u16_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u8_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u16_line_edit = LineEdit::new(()).into_raw();

        let bool_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let float_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let long_integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_label as *mut Widget, 3, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_label as *mut Widget, 4, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_label as *mut Widget, 5, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_label as *mut Widget, 6, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_label as *mut Widget, 7, 0, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_line_edit as *mut Widget, 1, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_line_edit as *mut Widget, 2, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_line_edit as *mut Widget, 3, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_line_edit as *mut Widget, 4, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_line_edit as *mut Widget, 5, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_line_edit as *mut Widget, 6, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_line_edit as *mut Widget, 7, 1, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_button as *mut Widget, 0, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_button as *mut Widget, 1, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_button as *mut Widget, 2, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_button as *mut Widget, 3, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_button as *mut Widget, 4, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_button as *mut Widget, 5, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_button as *mut Widget, 6, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_button as *mut Widget, 7, 2, 1, 1)); }

        // Create stuff for the info frame.
        let table_info_type_label = Label::new(&QString::from_std_str("Table type:")).into_raw();
        let table_info_version_label = Label::new(&QString::from_std_str("Table version:")).into_raw();
        let table_info_entry_count_label = Label::new(&QString::from_std_str("Table entry count:")).into_raw();

        let table_info_type_decoded_label = Label::new(()).into_raw();
        let table_info_version_decoded_label = Label::new(()).into_raw();
        let table_info_entry_count_decoded_label = Label::new(()).into_raw();

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_label as *mut Widget, 2, 0, 1, 1)); }

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_decoded_label as *mut Widget, 0, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_decoded_label as *mut Widget, 1, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_decoded_label as *mut Widget, 2, 1, 1, 1)); }

        // Create the TableView at the top.
        let table_view_old_versions = TableView::new().into_raw();
        let table_model_old_versions = StandardItemModel::new(()).into_raw();
        unsafe { table_view_old_versions.as_mut().unwrap().set_model(table_model_old_versions as *mut AbstractItemModel); }

        // Set it as not editable.
        unsafe { table_view_old_versions.as_mut().unwrap().set_edit_triggers(Flags::from_enum(EditTrigger::NoEditTriggers)); };
        unsafe { table_view_old_versions.as_mut().unwrap().set_selection_mode(SelectionMode::Single); };

        // Sort the versions.
        unsafe { table_view_old_versions.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view_old_versions.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }

        // Hide the vertical header.
        unsafe { table_view_old_versions.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(false); }

        // Prepare it for the Context Menu.
        unsafe { table_view_old_versions.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Create the Contextual Menu for the TableView.
        let mut table_view_old_versions_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_old_versions_context_menu_load = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Load"));
        let table_view_old_versions_context_menu_delete = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+l"))); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+del"))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_load); }
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the bottom ButtonBox.
        let button_box = Frame::new().into_raw();
        let button_box_layout = GridLayout::new().into_raw();
        unsafe { button_box.as_mut().unwrap().set_layout(button_box_layout as *mut Layout); }

        // Create the bottom Buttons.
        let clear_definition_button = PushButton::new(&QString::from_std_str("Remove all fields")).into_raw();
        let save_button = PushButton::new(&QString::from_std_str("Finish it!")).into_raw();

        // Add them to the Dialog.
        unsafe { button_box_layout.as_mut().unwrap().add_widget((clear_definition_button as *mut Widget, 0, 0, 1, 1)); }
        unsafe { button_box_layout.as_mut().unwrap().add_widget((save_button as *mut Widget, 0, 1, 1, 1)); }

        // Add everything to the main grid.
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((hex_view_group as *mut Widget, 0, 0, 4, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 1, 1, 2)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((decoded_fields_frame as *mut Widget, 1, 1, 3, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((info_frame as *mut Widget, 1, 2, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((table_view_old_versions as *mut Widget, 2, 2, 1, 1)); }
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((button_box as *mut Widget, 3, 2, 1, 1)); }

        //---------------------------------------------------------------------------------------//
        // Prepare the data for the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Get the PackedFile.
        sender_qt.send("get_packed_file").unwrap();
        sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        let packed_file: PackedFile = serde_json::from_slice(&response).unwrap();

        // Get the schema of the Game Selected.
        sender_qt.send("get_schema").unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        let schema: Option<Schema> = serde_json::from_slice(&response).unwrap();

        // If the PackedFile is in the db folder...
        if packed_file.path.len() > 2 {
            if packed_file.path[0] == "db" {

                // Put all together so we can pass it easely.
                let stuff = PackedFileDBDecoderStuff {
                    hex_view_index,
                    hex_view_raw,
                    hex_view_decoded,
                    table_view,
                    table_model,
                    bool_line_edit,
                    float_line_edit,
                    integer_line_edit,
                    long_integer_line_edit,
                    string_u8_line_edit,
                    string_u16_line_edit,
                    optional_string_u8_line_edit,
                    optional_string_u16_line_edit,
                    bool_button,
                    float_button,
                    integer_button,
                    long_integer_button,
                    string_u8_button,
                    string_u16_button,
                    optional_string_u8_button,
                    optional_string_u16_button,
                    table_info_type_decoded_label,
                    table_info_version_decoded_label,
                    table_info_entry_count_decoded_label,
                    table_view_old_versions,
                    table_model_old_versions,
                    clear_definition_button,
                    save_button,
                    table_view_context_menu: table_view_context_menu.into_raw(),
                    table_view_context_menu_move_up,
                    table_view_context_menu_move_down,
                    table_view_context_menu_delete,
                    table_view_old_versions_context_menu: table_view_old_versions_context_menu.into_raw(),
                    table_view_old_versions_context_menu_load,
                    table_view_old_versions_context_menu_delete,
                };

                // Create the index to move along the data.
                let mut initial_index = 0;

                // Check if it can be read as a table.
                match DBHeader::read(&packed_file.data, &mut initial_index) {

                    // If we succeed at decoding his header...
                    Ok(header) => {

                        // Put all the "Non UI" data we need to keep together.
                        let stuff_non_ui = PackedFileDBDecoderStuffNonUI {
                            packed_file,
                            initial_index,
                            header,
                        };

                        // Get the index we are going to "manipulate".
                        let index = Rc::new(RefCell::new(stuff_non_ui.initial_index));

                        // Check if we have an schema.
                        match schema {

                            // If we have an schema...
                            Some(schema) => {

                                // Get the table definition for this table (or create a new one).
                                let table_definition = match DB::get_schema(&stuff_non_ui.packed_file.path[1], stuff_non_ui.header.version, &schema) {
                                    Some(table_definition) => Rc::new(RefCell::new(table_definition)),
                                    None => Rc::new(RefCell::new(TableDefinition::new(stuff_non_ui.header.version)))
                                };

                                //---------------------------------------------------------------------------------------//
                                // Load the data to the Decoder View...
                                //---------------------------------------------------------------------------------------//

                                // Load the static data into the Decoder View.
                                Self::load_data_to_decoder_view(&stuff, &stuff_non_ui);

                                // Update the versions list.
                                Self::update_versions_list(&stuff, &schema, &stuff_non_ui.packed_file.path[1]);

                                // Update the Decoder View's Dynamic Data (LineEdits, Table,...) and recalculate
                                // the current "index_data" (position in the vector we are decoding).
                                Self::update_decoder_view(
                                    &stuff, &stuff_non_ui,
                                    (true, &table_definition.borrow().fields),
                                    &mut index.borrow_mut()
                                );

                                // Put the schema into a Rc<RefCell<Schema>>, so we can modify it.
                                let schema = Rc::new(RefCell::new(schema));

                                //---------------------------------------------------------------------------------------//
                                // Create the slots for the decoder view...
                                //---------------------------------------------------------------------------------------//

                                // Create all the slots we need to keep alive later on.
                                let slots = Self {

                                    // Slot to sync all the TextEdit in the "Hex Data" area.
                                    slot_hex_view_scroll_sync: SlotCInt::new(clone!(
                                        stuff => move |value| {
                                            unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                        }
                                    )),

                                    // Slots for the "Use this" buttons.
                                    slot_use_this_bool: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Boolean, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_float: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Float, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Integer, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_long_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::LongInteger, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU16, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU16, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for when we change the Type of the selected field in the table.
                                    slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |initial_model_index,final_model_index,_| {

                                            // If both changed cells are from the Type column...
                                            if initial_model_index.column() == 1 && final_model_index.column() == 1 {

                                                // Update the "First row decoded" column, and get the new "index" to continue decoding.
                                                let invalid_rows = Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());

                                                // Fix the broken rows.
                                                for row in &invalid_rows {

                                                    // Get the item from the type column.
                                                    let item;
                                                    unsafe { item = stuff.table_model.as_mut().unwrap().item((*row, 1)); }

                                                    // Change it to bool.
                                                    unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str("Bool")); }
                                                }
                                            }
                                        }
                                    )),

                                    // Slot to enable/disable contextual actions depending on the selected item.
                                    slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is something selected...
                                            if !selection.indexes().is_empty() {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),

                                    // Slot to show the Contextual Menu for the TableView.
                                    slot_table_view_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),

                                    // Slots for the Contextual Menu Actions of the TableView.
                                    slot_table_view_context_menu_move_up: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move up.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list.
                                            rows.sort();
                                            rows.dedup();

                                            // For each row we have to move...
                                            for row in rows {

                                                // If we are in the row 0, skip it.
                                                if row == 0 { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row - 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_move_down: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move down.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to move...
                                            for row in rows {

                                                // Get the amount of rows in the Model.
                                                let row_count;
                                                unsafe { row_count = stuff.table_model.as_mut().unwrap().row_count(()); }

                                                // If we are in the last row, skip it.
                                                if row == (row_count - 1) { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row + 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_delete: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to remove.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to remove...
                                            for row in rows {

                                                // Remove it.
                                                unsafe { stuff.table_model.as_mut().unwrap().remove_row(row); }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Kill them all!" button.
                                    slot_remove_all_fields: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // Remove everything from the model.
                                            unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                            // Reset the index.
                                            *index.borrow_mut() = stuff_non_ui.initial_index;

                                            // Update the decoder view.
                                            Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Finish it!" button.
                                    slot_save_definition: SlotNoArgs::new(clone!(
                                        rpfm_path,
                                        supported_games,
                                        sender_qt,
                                        receiver_qt,
                                        table_definition,
                                        schema,
                                        app_ui,
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // We get the index of our table's definitions. In case we find it, we just return it. If it's not
                                            // the case, then we create a new table's definitions and return his index. To know if we didn't found
                                            // an index, we just return -1 as index.
                                            let mut table_definitions_index = match schema.borrow().get_table_definitions(&stuff_non_ui.packed_file.path[1]) {
                                                Some(table_definitions_index) => table_definitions_index as i32,
                                                None => -1i32,
                                            };

                                            // If we didn't found a table definition for our table...
                                            if table_definitions_index == -1 {

                                                // We create one.
                                                schema.borrow_mut().add_table_definitions(TableDefinitions::new(&stuff_non_ui.packed_file.path[1]));

                                                // And get his index.
                                                table_definitions_index = schema.borrow().get_table_definitions(&stuff_non_ui.packed_file.path[1]).unwrap() as i32;
                                            }

                                            // We replace his fields with the ones from the TableView.
                                            table_definition.borrow_mut().fields = Self::return_data_from_data_view(&stuff);

                                            // We add our `TableDefinition` to the main `Schema`.
                                            schema.borrow_mut().tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());

                                            // Get the Game Selected.
                                            sender_qt.send("get_game_selected").unwrap();
                                            let response = receiver_qt.borrow().recv().unwrap().unwrap();
                                            let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                                            // And try to save the main `Schema`.
                                            match Schema::save(&schema.borrow(), &rpfm_path, &supported_games.iter().filter(|x| x.folder_name == game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()) {
                                                Ok(_) => show_dialog(&app_ui, true, "Schema successfully saved."),
                                                Err(error) => show_dialog(&app_ui, false, error.cause()),
                                            }

                                            // After all that, we need to update the version list, as this may have created a new version.
                                            Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file.path[1]);
                                        }
                                    )),

                                    // Actions to manage the Context Menu in the "Versions" TableView.
                                    slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is only one version selected...
                                            if selection.indexes().count(()) == 1 {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_old_versions_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_load: SlotBool::new(clone!(
                                        schema,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<u32>().unwrap();

                                                // Get the new definition.
                                                let table_definition = DB::get_schema(&stuff_non_ui.packed_file.path[1], version, &*schema.borrow());

                                                // Remove everything from the model.
                                                unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                                // Reset the index.
                                                *index.borrow_mut() = stuff_non_ui.initial_index;

                                                // Update the decoder view.
                                                Self::update_decoder_view(&stuff, &stuff_non_ui, (true, &table_definition.unwrap().fields), &mut index.borrow_mut());
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_delete: SlotBool::new(clone!(
                                        schema,
                                        app_ui,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<u32>().unwrap();

                                                // Try to remove that version form the schema.
                                                if let Err(error) = DB::remove_table_version(&stuff_non_ui.packed_file.path[1], version, &mut schema.borrow_mut()) {
                                                    return show_dialog(&app_ui, false, format!("<p>Error while removing a version of this table's definitions:</p> <p>{}</p>", error.cause()));
                                                }

                                                // If it worked, update the list.
                                                Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file.path[1]);
                                            }
                                        }
                                    )),
                                };

                                // Sync the scroll bars of the three hex data views.
                                unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }

                                // Actions for the "Use this" buttons.
                                unsafe { stuff.bool_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_bool); }
                                unsafe { stuff.float_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_float); }
                                unsafe { stuff.integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_integer); }
                                unsafe { stuff.long_integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_long_integer); }
                                unsafe { stuff.string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u8); }
                                unsafe { stuff.string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u16); }
                                unsafe { stuff.optional_string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u8); }
                                unsafe { stuff.optional_string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u16); }

                                // Action for when we change the type of a field in the table.
                                unsafe { stuff.table_model.as_mut().unwrap().signals().data_changed().connect(&slots.slot_table_change_field_type); }

                                // Trigger the "Enable/Disable" slot every time we change the selection in the TableView.
                                unsafe { stuff.table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_context_menu_enabler); }

                                // Action to show the Contextual Menu for the Field's TableView.
                                unsafe { (stuff.table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_context_menu); }

                                // Actions of the Contextual Menu for the Field's TableView.
                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_up); }
                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_down); }
                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_delete); }

                                // Action of the "Kill them all!" button.
                                unsafe { stuff.clear_definition_button.as_mut().unwrap().signals().released().connect(&slots.slot_remove_all_fields); }

                                // Action of the "Finish it!" button.
                                unsafe { stuff.save_button.as_mut().unwrap().signals().released().connect(&slots.slot_save_definition); }

                                // Actions for the Contextual Menu in the "Versions" table.
                                unsafe { stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_old_versions_context_menu_enabler); }
                                unsafe { (stuff.table_view_old_versions as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_old_versions_context_menu); }
                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_load); }
                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_delete); }

                                // Return the slots and the font.
                                Ok((slots, monospace_font))
                            }

                            // If there is no schema, return error.
                            None => return Err(format_err!("<p>Error while trying to decode the PackedFile:</p> <p>There is no Schema for the Game Selected.</p>"))
                        }
                    },

                    // If it fails, return error.
                    Err(error) => return Err(format_err!("<p>Error while trying to decode the PackedFile:</p> <p>{}</p>", error.cause()))
                }
            }

            // Otherwise, return error.
            else { return Err(format_err!("This PackedFile is not a DB Table.")) }
        }

        // Otherwise, return error.
        else { return Err(format_err!("This PackedFile is not a DB Table.")) }
    }

    /// This function is meant to load the static data of a DB PackedFile into the decoder, or return error.
    pub fn load_data_to_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
    ) {
        // Get the FontMetrics for the size stuff.
        let font;
        unsafe { font = stuff.hex_view_index.as_mut().unwrap().document().as_mut().unwrap().default_font(); }
        let font_metrics = FontMetrics::new(&font);

        // We don't need the entire PackedFile, just his begining. Otherwise, this function takes ages to finish.
        let data_reduced = if stuff_non_ui.packed_file.data.len() > 16 * 60 { &stuff_non_ui.packed_file.data[..16 * 60] }
        else { &stuff_non_ui.packed_file.data };

        // This creates the "index" column at the left of the hex data. The logic behind this, because
        // even I have problems to understand it: lines are 4 packs of 4 bytes => 16 bytes. Amount of
        // lines is "bytes we have / 16 + 1" (+ 1 because we want to show incomplete lines too).
        // Then, for the zeroes, we default to 4.
        let mut hex_lines_text = String::new();
        let hex_lines_amount = (data_reduced.len() / 16) + 1;
        for hex_line in 0..hex_lines_amount { hex_lines_text.push_str(&format!("{:>0count$X}\n", hex_line * 16, count = 4)); }

        // Add all the "Index" lines to the TextEdit.
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_html(&QString::from_std_str(&hex_lines_text)); }

        // Resize the TextEdit.
        let text_size = font_metrics.size((0, &QString::from_std_str(hex_lines_text)));
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

        // This gets the hex data into place. In big files, this takes ages, so we cut them if they
        // are longer than 100 lines to speed up loading and fix a weird issue with big tables.
        let mut hex_raw_data = format!("{:02X?}", data_reduced);

        // Remove the first and last chars.
        hex_raw_data.remove(0);
        hex_raw_data.pop();

        // Remove all the kebab, or the commas. Whatever float your boat...
        hex_raw_data.retain(|c| c != ',');

        // `hex_view_raw` TextEdit.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_raw_string = String::new();

            // This pushes a newline after 48 characters (2 for every byte + 1 whitespace).
            for (j, i) in hex_raw_data.chars().enumerate() {
                if j % 48 == 0 && j != 0 { hex_view_raw_string.push_str("\n"); }
                hex_view_raw_string.push(i);
            }

            // Add all the "Raw" lines to the TextEdit.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_html(&QString::from_std_str(&hex_view_raw_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_raw_string)));
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

            /*
            // In theory, this should give us the equivalent byte to our index_data.
            // In practice, I'm bad at maths.
            let header_line = (stuff_non_ui.initial_index * 3 / 48) as i32;
            let header_char = (stuff_non_ui.initial_index * 3 % 48) as i32;

            raw_data_hex_buffer.apply_tag_by_name(
                "header",
                &raw_data_hex_buffer.get_start_iter(),
                &raw_data_hex_buffer.get_iter_at_line_offset(header_line, header_char)
            );*/
        }

        // `hex_view_decoded` TextView.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_decoded_string = String::new();

            // This pushes a newline after 16 characters.
            for (j, i) in data_reduced.iter().enumerate() {
                if j % 16 == 0 && j != 0 { hex_view_decoded_string.push_str("\n"); }
                let character = *i as char;

                // If is a valid UTF-8 char, show it. Otherwise, default to '.'.
                if character.is_alphanumeric() { hex_view_decoded_string.push(character); }
                else { hex_view_decoded_string.push('.'); }
            }

            // Add all the "Decoded" lines to the TextEdit.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text(&QString::from_std_str(&hex_view_decoded_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_decoded_string)));
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }
            /*
            let header_line = (self.data_initial_index / 16) as i32;
            let header_char = (self.data_initial_index % 16) as i32;

            let raw_data_decoded_buffer = self.raw_data_decoded.get_buffer().unwrap();
            raw_data_decoded_buffer.set_text(&hex_raw_data_decoded);
            raw_data_decoded_buffer.apply_tag_by_name(
                "header",
                &raw_data_decoded_buffer.get_start_iter(),
                &raw_data_decoded_buffer.get_iter_at_line_offset(header_line, header_char)
            );*/
        }

        // Load the "Info" data to the view.
        unsafe { stuff.table_info_type_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(&stuff_non_ui.packed_file.path[1])); }
        unsafe { stuff.table_info_version_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.header.version))); }
        unsafe { stuff.table_info_entry_count_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.header.entry_count))); }
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    pub fn update_versions_list(
        stuff: &PackedFileDBDecoderStuff,
        schema: &Schema,
        table_name: &str,
    ) {
        // Clear the current list.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().clear(); }

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Some(table_versions_list) = DB::get_schema_versions_list(table_name, schema) {
            for version in table_versions_list {
                let item = StandardItem::new(&QString::from_std_str(format!("{}", version.version)));
                unsafe { stuff.table_model_old_versions.as_mut().unwrap().append_row_unsafe(item.into_raw()); }
            }
        }

        // Set the title of the column.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Versions Decoded")))); }

        // Extend the column to fill the Table.
        unsafe { stuff.table_view_old_versions.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_section_resize_mode(ResizeMode::Stretch); }
    }

    /// This function updates the data shown in the decoder view when we execute it. `index_data`
    /// is the position from where to start decoding. In field_list the boolean is true for the first load.
    /// Otherwise, always pass false there.
    pub fn update_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_list: (bool, &[Field]),
        mut index_data: &mut usize
    ) {

        // Create the variables to hold the values we'll pass to the LineEdits.
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // If we are loading data to the table for the first time, we'll load to the table all the data
        // directly from the existing definition and update the initial index for decoding.
        if field_list.0 {
            for field in field_list.1 {
                Self::add_field_to_data_view(
                    &stuff,
                    &stuff_non_ui,
                    &field.field_name,
                    field.field_type.to_owned(),
                    field.field_is_key,
                    &field.field_is_reference,
                    &field.field_description,
                    &mut index_data,
                );
            }
        }

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file.data.get(*index_data).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(stuff_non_ui.packed_file.data[*index_data], &mut index_data.clone()) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&stuff_non_ui.packed_file.data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&stuff_non_ui.packed_file.data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file.data.get(*index_data + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&stuff_non_ui.packed_file.data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&stuff_non_ui.packed_file.data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if stuff_non_ui.packed_file.data.get(*index_data + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&stuff_non_ui.packed_file.data[*index_data..(*index_data + 8)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if stuff_non_ui.packed_file.data.get(*index_data + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&stuff_non_ui.packed_file.data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&stuff_non_ui.packed_file.data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        unsafe { stuff.bool_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_bool)); }
        unsafe { stuff.float_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_float)); }
        unsafe { stuff.integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_integer)); }
        unsafe { stuff.long_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_long_integer)); }
        unsafe { stuff.string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8))); }
        unsafe { stuff.string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16))); }
        unsafe { stuff.optional_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8))); }
        unsafe { stuff.optional_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16))); }

        /*
        // Then we set the TextTags to paint the hex_data.
        let raw_data_hex_text_buffer = self.raw_data.get_buffer().unwrap();

        // Clear the current index tag.
        raw_data_hex_text_buffer.remove_tag_by_name("index", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());
        raw_data_hex_text_buffer.remove_tag_by_name("entry", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (*index_data * 3 / 48) as i32;
        let index_line_end = (((*index_data * 3) + 2) / 48) as i32;
        let index_char_start = ((*index_data * 3) % 48) as i32;
        let index_char_end = (((*index_data * 3) + 2) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = ((self.data_initial_index * 3) / 48) as i32;
        let header_char = ((self.data_initial_index * 3) % 48) as i32;
        let index_line_end = ((*index_data * 3) / 48) as i32;
        let index_char_end = ((*index_data * 3) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // And then, we do the same for `raw_decoded_data`.
        let raw_data_decoded_text_buffer = self.raw_data_decoded.get_buffer().unwrap();

        // Clear the current index and entry tags.
        raw_data_decoded_text_buffer.remove_tag_by_name("index", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());
        raw_data_decoded_text_buffer.remove_tag_by_name("entry", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (*index_data / 16) as i32;
        let index_line_end = ((*index_data + 1) / 16) as i32;
        let index_char_start = (*index_data % 16) as i32;
        let index_char_end = ((*index_data + 1) % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = (self.data_initial_index / 16) as i32;
        let header_char = (self.data_initial_index % 16) as i32;
        let index_line_end = (*index_data / 16) as i32;
        let index_char_end = (*index_data % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );*/
    }

    /// This function adds fields to the decoder's table, so we can do this without depending on the
    /// updates of the decoder's view. As this has a lot of required data, lets's explain the weirdest ones:
    /// - index_data: the index to start decoding from the vector.
    /// - index_row: the position in the row. None to calculate the last position's number.
    pub fn add_field_to_data_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_name: &str,
        field_type: FieldType,
        field_is_key: bool,
        field_is_reference: &Option<(String, String)>,
        field_description: &str,
        mut index_data: &mut usize,
    ) {

        // Decode the data from the field.
        let decoded_data = Self::decode_data_by_fieldtype(
            &stuff_non_ui.packed_file.data,
            &field_type,
            &mut index_data
        );

        // Get the type of the data we are going to put into the Table.
        let field_type = match field_type {
            FieldType::Boolean => "Bool",
            FieldType::Float => "Float",
            FieldType::Integer => "Integer",
            FieldType::LongInteger => "LongInteger",
            FieldType::StringU8 => "StringU8",
            FieldType::StringU16 => "StringU16",
            FieldType::OptionalStringU8 => "OptionalStringU8",
            FieldType::OptionalStringU16 => "OptionalStringU16",
        };

        // If the field has a reference...
        if let Some(ref reference) = *field_is_reference {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(&reference.0));
            let reference_field = StandardItem::new(&QString::from_std_str(&reference.1));
            let decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Otherwise, we pass an empty reference.
        else {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(""));
            let reference_field = StandardItem::new(&QString::from_std_str(""));
            let decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Set the title of the columns and extend them, just in case is needed.
        unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_section_resize_mode(ResizeMode::Stretch); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Name")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Type")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Is key?")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Table")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((4, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Column")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((5, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("First Row Decoded")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((6, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Description")))); }
    }

    /// This function is a helper to try to decode data in different formats, returning "Error" in case
    /// of decoding error. It requires the FieldType we want to decode, the data we want to decode
    /// (vec<u8>, being the first u8 the first byte to decode) and the index of the data in the Vec<u8>.
    fn decode_data_by_fieldtype(
        field_data: &[u8],
        field_type: &FieldType,
        mut index_data: &mut usize
    ) -> String {

        // Try to decode the field, depending on what type that field is.
        match *field_type {

            // If the field is a boolean...
            FieldType::Boolean => {

                // Check if the index does even exist, to avoid crashes.
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_bool(field_data[*index_data], &mut index_data) {
                        Ok(result) => {
                            if result { "True".to_string() }
                            else { "False".to_string() }
                        }
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Float => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_float_f32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Integer => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_integer_i32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::LongInteger => {
                if field_data.get(*index_data + 7).is_some() {
                    match coding_helpers::decode_packedfile_integer_i64(&field_data[*index_data..(*index_data + 8)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU8 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU16 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU8 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU16 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
        }
    }

    /// This function is used to update the decoder view when we try to add a new field to
    /// the schema with one of the "Use this" buttons.
    pub fn use_this(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_type: FieldType,
        mut index_data: &mut usize,
    ) {

        // Try to add the field, and update the index with it.
        Self::add_field_to_data_view(
            &stuff,
            &stuff_non_ui,
            "new_field",
            field_type,
            false,
            &None,
            "",
            &mut index_data,
        );

        // Update all the dynamic data of the "Decoder" view.
        Self::update_decoder_view(
            &stuff,
            &stuff_non_ui,
            (false, &[]),
            &mut index_data,
        );
    }

    /// This function updates the "First row decoded" column in the decoder view, the current index and
    /// the decoded entries. This should be called in row changes (deletion and moving, not adding).
    fn update_first_row_decoded(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        mut index: &mut usize
    ) -> Vec<i32> {

        // Create the list of "invalid" types.
        let mut invalid_types = vec![];

        // Reset the index.
        *index = stuff_non_ui.initial_index;

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get the ModelIndex of the cell we want to update.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 5)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the row's type.
                let row_type;
                unsafe { row_type = stuff.table_model.as_mut().unwrap().index((row, 1)); }

                // Get the field's type.
                let field_type = match &*row_type.data(0).to_string().to_std_string() {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,

                    // In case of invalid type, we add it to the list and set it as bool.
                    _ => {

                        // Add the row to the list.
                        invalid_types.push(row);

                        // Return a boolean.
                        FieldType::Boolean
                    }
                };

                // Get the decoded data using it's type...
                let decoded_data = Self::decode_data_by_fieldtype(
                    &stuff_non_ui.packed_file.data,
                    &field_type,
                    &mut index
                );

                // Get the item from the "First Row Decoded" column.
                let item;
                unsafe { item = stuff.table_model.as_mut().unwrap().item_from_index(&model_index); }

                // Change it to our decoded data.
                unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(decoded_data)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Update the entire decoder to use the new index.
        Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index);

        // Return the list of broken rows.
        invalid_types
    }

    /// This function gets the data from the decoder's table, and returns it, so we can save it in a TableDefinition.
    pub fn return_data_from_data_view(
        stuff: &PackedFileDBDecoderStuff
    ) -> Vec<Field> {

        // Create the field's vector.
        let mut fields = vec![];

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get a ModelIndex from the row.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 0)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the data from each field of the row...
                let field_name;
                let field_type;
                let field_is_key;
                let ref_table;
                let ref_column;
                let field_description;

                unsafe { field_name = stuff.table_model.as_mut().unwrap().item((row, 0)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_type = stuff.table_model.as_mut().unwrap().item((row, 1)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_is_key = if stuff.table_model.as_mut().unwrap().item((row, 2)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }; }
                unsafe { ref_table = stuff.table_model.as_mut().unwrap().item((row, 3)).as_mut().unwrap().text().to_std_string(); }
                unsafe { ref_column = stuff.table_model.as_mut().unwrap().item((row, 4)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_description = stuff.table_model.as_mut().unwrap().item((row, 6)).as_mut().unwrap().text().to_std_string(); }

                // Get the proper type of the field. If invalid, default to OptionalStringU16.
                let field_type = match &*field_type {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" | _=> FieldType::OptionalStringU16,
                };

                // If there is no table referenced...
                if ref_table.is_empty() { fields.push(Field::new(field_name, field_type, field_is_key, None, field_description)); }

                // Otherwise...
                else { fields.push(Field::new(field_name, field_type, field_is_key, Some((ref_table, ref_column)), field_description)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Return the fields.
        fields
    }
}
/*
use std::cell::RefCell;
use std::rc::Rc;
use packedfile::db::schemas::*;
use packfile::packfile::PackedFile;
use settings::*;
use common::coding_helpers;
use failure::Error;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{
    TreeView, ListStore, ScrolledWindow, Button, Orientation, TextView, Label, Entry, FileChooserNative,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type, Frame, CellRendererCombo, CssProvider,
    TextTag, Popover, ModelButton, Paned, Switch, Separator, Grid, ButtonBox, ButtonBoxStyle, FileChooserAction,
    StyleContext, TreeViewGridLines, TreeViewColumnSizing, EntryIconPosition, TreeIter
};

use super::*;
use packedfile::SerializableToTSV;
use AppUI;
use packfile::update_packed_file_data_db;

/// Struct `PackedFileDBTreeView`: contains all the stuff we need to give to the program to show a
/// `TreeView` with the data of a DB PackedFile, allowing us to manipulate it.
#[derive(Clone, Debug)]
pub struct PackedFileDBTreeView {
    pub tree_view: TreeView,
    pub list_store: ListStore,
    pub list_cell_bool: Vec<CellRendererToggle>,
    pub list_cell_float: Vec<CellRendererText>,
    pub list_cell_integer: Vec<CellRendererText>,
    pub list_cell_long_integer: Vec<CellRendererText>,
    pub list_cell_string: Vec<CellRendererText>,
    pub list_cell_optional_string: Vec<CellRendererText>,
    pub list_cell_reference: Vec<CellRendererCombo>,
    pub context_menu: Popover,
    pub add_rows_entry: Entry,
}

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
#[derive(Clone, Debug)]
pub struct PackedFileDBDecoder {
    pub data_initial_index: usize,
    pub decoded_header: DBHeader,
    pub raw_data_line_index: TextView,
    pub raw_data: TextView,
    pub raw_data_decoded: TextView,
    pub table_type_label: Label,
    pub table_version_label: Label,
    pub table_entry_count_label: Label,
    pub bool_entry: Entry,
    pub float_entry: Entry,
    pub integer_entry: Entry,
    pub long_integer_entry: Entry,
    pub string_u8_entry: Entry,
    pub string_u16_entry: Entry,
    pub optional_string_u8_entry: Entry,
    pub optional_string_u16_entry: Entry,
    pub use_bool_button: Button,
    pub use_float_button: Button,
    pub use_integer_button: Button,
    pub use_long_integer_button: Button,
    pub use_string_u8_button: Button,
    pub use_string_u16_button: Button,
    pub use_optional_string_u8_button: Button,
    pub use_optional_string_u16_button: Button,
    pub fields_tree_view: TreeView,
    pub fields_list_store: ListStore,
    pub all_table_versions_tree_view: TreeView,
    pub all_table_versions_list_store: ListStore,
    pub all_table_versions_load_definition: Button,
    pub all_table_versions_remove_definition: Button,
    pub field_name_entry: Entry,
    pub is_key_field_switch: Switch,
    pub save_decoded_schema: Button,
    pub fields_tree_view_cell_bool: CellRendererToggle,
    pub fields_tree_view_cell_combo: CellRendererCombo,
    pub fields_tree_view_cell_combo_list_store: ListStore,
    pub fields_tree_view_cell_string: Vec<CellRendererText>,
    pub delete_all_fields_button: Button,
    pub decoder_grid_scroll: ScrolledWindow,
    pub context_menu: Popover,
}

/// This function serves as a way to prepare the DB TreeView and the Decoder View. It's needed because
/// unlike other decoders, DB decoder can result in two different views being created.
pub fn create_db_view(
    application: &Application,
    app_ui: &AppUI,
    rpfm_path: &PathBuf,
    pack_file: &Rc<RefCell<PackFile>>,
    packed_file_decoded_index: &usize,
    is_packedfile_opened: &Rc<RefCell<bool>>,
    schema: &Rc<RefCell<Option<Schema>>>,
    dependency_database: &Rc<RefCell<Option<Vec<PackedFile>>>>,
    game_selected: &Rc<RefCell<GameSelected>>,
    supported_games: &Rc<RefCell<Vec<GameInfo>>>,
    settings: &Settings,
) -> Result<(), Error> {

    // Get the data of the PackedFile and his name.
    let packed_file_encoded = pack_file.borrow().data.packed_files[*packed_file_decoded_index].data.to_vec();
    let table_name = pack_file.borrow().data.packed_files[*packed_file_decoded_index].path[1].to_owned();

    // Try to decode it, and return error in case of missing schema.
    let packed_file_decoded = match *schema.borrow() {
        Some(ref schema) => DB::read(&packed_file_encoded, &table_name, schema),
        None => return Err(format_err!("There is no Schema loaded for this game.")),
    };

    // We create the button to enable the "Decoding" mode.
    let decode_mode_button = Button::new_with_label("Enter decoding mode");
    decode_mode_button.set_hexpand(true);
    app_ui.packed_file_data_display.attach(&decode_mode_button, 0, 0, 1, 1);
    app_ui.packed_file_data_display.show_all();

    // Tell the program there is an open PackedFile.
    *is_packedfile_opened.borrow_mut() = true;

    // Disable the "Change game selected" function, so we cannot change the current schema with an open table.
    app_ui.menu_bar_change_game_selected.set_enabled(false);

    // When we destroy the "Enable decoding mode" button, we need to tell the program we no longer have
    // an open PackedFile. This happens when we select another PackedFile (closing a table) or when we
    // hit the button (entering the decoder, where we no longer need write access to the original file).
    decode_mode_button.connect_destroy(clone!(
        app_ui,
        is_packedfile_opened => move |_| {

            // Tell the game you no longer have an open PackedFile.
            *is_packedfile_opened.borrow_mut() = false;

            // Restore the "Change game selected" function.
            app_ui.menu_bar_change_game_selected.set_enabled(true);
        }
    ));

    // From here, we deal we the decoder stuff.
    decode_mode_button.connect_button_release_event(clone!(
        application,
        app_ui,
        schema,
        rpfm_path,
        game_selected,
        supported_games => move |_,_| {

            // We destroy the table view if exists, and the button, so we don't have to deal with resizing it.
            let childrens_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
            if !childrens_to_utterly_destroy.is_empty() {
                for i in &childrens_to_utterly_destroy {
                    i.destroy();
                }
            }

            // Then try to create the UI and if it throws an error, report it.
            if let Err(error) = PackedFileDBDecoder::create_decoder_view(
                &application,
                &app_ui,
                &rpfm_path,
                &supported_games,
                &game_selected,
                table_name.to_owned(),
                packed_file_encoded.to_vec(),
                &schema,
            ) {
                show_dialog(&app_ui.window, false, error.cause())
            };

            Inhibit(false)
        }
    ));

    // If this returns an error, we just leave the button for the decoder.
    match packed_file_decoded {
        Ok(packed_file_decoded) => {

            // Get the decoded PackedFile in a `Rc<RefCell<>>` so we can pass it to the closures.
            let packed_file_decoded = Rc::new(RefCell::new(packed_file_decoded));

            // Try to create the `TreeView`.
            if let Err(error) = PackedFileDBTreeView::create_tree_view(
                &application,
                &app_ui,
                &pack_file,
                &packed_file_decoded,
                packed_file_decoded_index,
                &dependency_database.borrow(),
                &schema.borrow().clone().unwrap(),
                &settings,
            ) { return Err(error) };

            // Return success.
            Ok(())
        }

        // If we receive an error while decoding, report it.
        Err(error) => Err(error),
    }
}


/// Implementation of `PackedFileDBTreeView`.
impl PackedFileDBTreeView{

    /// This function creates a new `TreeView` with `packed_file_data_display` as father and returns a
    /// `PackedFileDBTreeView` with all his data.
    pub fn create_tree_view(
        application: &Application,
        app_ui: &AppUI,
        pack_file: &Rc<RefCell<PackFile>>,
        packed_file_decoded: &Rc<RefCell<DB>>,
        packed_file_decoded_index: &usize,
        dependency_database: &Option<Vec<PackedFile>>,
        master_schema: &Schema,
        settings: &Settings,
    ) -> Result<(), Error> {

        // Here we define the `Accept` response for GTK, as it seems Restson causes it to fail to compile
        // if we get them to i32 directly in the `if` statement.
        // NOTE: For some bizarre reason, GTKFileChoosers return `Ok`, while native ones return `Accept`.
        let gtk_response_accept: i32 = ResponseType::Accept.into();

        // Get the table's path, so we can use it despite changing the selected file in the main TreeView.
        let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, false);

        // Get the table definition of this table.
        let table_definition = packed_file_decoded.borrow().data.table_definition.clone();

        // We create a `Vec<Type>` to hold the types of of the columns of the `TreeView`.
        let mut list_store_types: Vec<Type> = vec![];

        // The first column is an index for us to know how many entries we have.
        list_store_types.push(Type::String);

        // Depending on the type of the field, we push the `gtk::Type` equivalent to that column.
        for field in &table_definition.fields {
            match field.field_type {
                FieldType::Boolean => list_store_types.push(Type::Bool),
                FieldType::Integer => list_store_types.push(Type::I32),
                FieldType::LongInteger => list_store_types.push(Type::I64),

                // Floats are an special case. We pass them as `String` because otherwise it shows trailing zeroes.
                FieldType::Float |
                FieldType::StringU8 |
                FieldType::StringU16 |
                FieldType::OptionalStringU8 |
                FieldType::OptionalStringU16 => list_store_types.push(Type::String),
            }
        }

        // We create the `TreeView` and his `ListStore`.
        let tree_view = TreeView::new();
        let list_store = ListStore::new(&list_store_types);

        // Config for the `TreeView`.
        tree_view.set_model(Some(&list_store));
        tree_view.set_grid_lines(TreeViewGridLines::Both);
        tree_view.set_rubber_banding(true);
        tree_view.set_has_tooltip(true);
        tree_view.set_enable_search(false);
        tree_view.set_search_column(0);
        tree_view.set_margin_bottom(10);

        // We enable "Multiple" selection mode, so we can do multi-row operations.
        tree_view.get_selection().set_mode(gtk::SelectionMode::Multiple);

        // We create the "Index" cell and column.
        let cell_index = CellRendererText::new();
        let column_index = TreeViewColumn::new();

        // Config for the "Index" cell and column.
        cell_index.set_property_xalign(0.5);
        column_index.set_title("Index");
        column_index.set_clickable(true);
        column_index.set_min_width(50);
        column_index.set_alignment(0.5);
        column_index.set_sort_column_id(0);
        column_index.set_sizing(gtk::TreeViewColumnSizing::Autosize);
        column_index.pack_start(&cell_index, true);
        column_index.add_attribute(&cell_index, "text", 0);
        tree_view.append_column(&column_index);

        // We create the vectors that will hold the different cell types.
        let mut list_cell_bool = vec![];
        let mut list_cell_float = vec![];
        let mut list_cell_integer = vec![];
        let mut list_cell_long_integer = vec![];
        let mut list_cell_string = vec![];
        let mut list_cell_optional_string = vec![];
        let mut list_cell_reference = vec![];

        // We create a vector to store the key columns.
        let mut key_columns = vec![];

        // For each field in the table definition...
        for (index, field) in table_definition.fields.iter().enumerate() {

            // Clean the column name, so it has proper spaces and such.
            let field_name = clean_column_names(&field.field_name);

            // These are the specific declarations of the columns for every type implemented.
            match field.field_type {

                // If it's a Boolean...
                FieldType::Boolean => {

                    // We create the cell and the column.
                    let cell_bool = CellRendererToggle::new();
                    let column_bool = TreeViewColumn::new();

                    // Reduce the size of the checkbox to 160% the size of the font used (yes, 160% is less that his normal size).
                    cell_bool.set_property_indicator_size((settings.font.split(' ').filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<f32>>()[0] * 1.6) as i32);
                    cell_bool.set_activatable(true);

                    // Config for the column.
                    column_bool.set_title(&field_name);
                    column_bool.set_clickable(true);
                    column_bool.set_min_width(50);
                    column_bool.set_sizing(TreeViewColumnSizing::GrowOnly);
                    column_bool.set_alignment(0.5);
                    column_bool.set_sort_column_id((index + 1) as i32);
                    column_bool.pack_start(&cell_bool, true);
                    column_bool.add_attribute(&cell_bool, "active", (index + 1) as i32);
                    tree_view.append_column(&column_bool);
                    list_cell_bool.push(cell_bool);

                    // If it's marked as a "key" filed, add it to our "key" columns list.
                    if field.field_is_key { key_columns.push(column_bool); }
                }

                // If it's a float...
                FieldType::Float => {

                    // We create the cell and the column.
                    let cell_float = CellRendererText::new();
                    let column_float = TreeViewColumn::new();

                    // Config for the cell.
                    cell_float.set_property_editable(true);
                    cell_float.set_property_xalign(1.0);
                    cell_float.set_property_placeholder_text(Some("Float (2.54, 3.21, 6.8765,..)"));

                    // Config for the column.
                    column_float.set_title(&field_name);
                    column_float.set_clickable(true);
                    column_float.set_resizable(true);
                    column_float.set_min_width(50);
                    column_float.set_sizing(TreeViewColumnSizing::GrowOnly);
                    column_float.set_alignment(0.5);
                    column_float.set_sort_column_id((index + 1) as i32);
                    column_float.pack_start(&cell_float, true);
                    column_float.add_attribute(&cell_float, "text", (index + 1) as i32);
                    tree_view.append_column(&column_float);
                    list_cell_float.push(cell_float);

                    // If it's marked as a "key" filed, add it to our "key" columns list.
                    if field.field_is_key { key_columns.push(column_float); }
                }

                // If it's an integer...
                FieldType::Integer => {

                    // We create the cell and the column.
                    let cell_int = CellRendererText::new();
                    let column_int = TreeViewColumn::new();

                    // Config for the cell.
                    cell_int.set_property_editable(true);
                    cell_int.set_property_xalign(1.0);
                    cell_int.set_property_placeholder_text(Some("Integer (2, 3, 6,..)"));

                    // Config for the column.
                    column_int.set_title(&field_name);
                    column_int.set_clickable(true);
                    column_int.set_resizable(true);
                    column_int.set_min_width(50);
                    column_int.set_sizing(TreeViewColumnSizing::GrowOnly);
                    column_int.set_alignment(0.5);
                    column_int.set_sort_column_id((index + 1) as i32);
                    column_int.pack_start(&cell_int, true);
                    column_int.add_attribute(&cell_int, "text", (index + 1) as i32);
                    tree_view.append_column(&column_int);
                    list_cell_integer.push(cell_int);

                    // If it's marked as a "key" filed, add it to our "key" columns list.
                    if field.field_is_key { key_columns.push(column_int); }
                }

                // If it's a "Long Integer" (u64)...
                FieldType::LongInteger => {

                    // We create the cell and the column.
                    let cell_long_int = CellRendererText::new();
                    let column_long_int = TreeViewColumn::new();

                    // Config for the cell.
                    cell_long_int.set_property_editable(true);
                    cell_long_int.set_property_xalign(1.0);
                    cell_long_int.set_property_placeholder_text(Some("Long Integer (2, 3, 6,..)"));

                    // Config for the column.
                    column_long_int.set_title(&field_name);
                    column_long_int.set_clickable(true);
                    column_long_int.set_resizable(true);
                    column_long_int.set_min_width(50);
                    column_long_int.set_sizing(TreeViewColumnSizing::GrowOnly);
                    column_long_int.set_alignment(0.5);
                    column_long_int.set_sort_column_id((index + 1) as i32);
                    column_long_int.pack_start(&cell_long_int, true);
                    column_long_int.add_attribute(&cell_long_int, "text", (index + 1) as i32);
                    tree_view.append_column(&column_long_int);
                    list_cell_long_integer.push(cell_long_int);

                    // If it's marked as a "key" filed, add it to our "key" columns list.
                    if field.field_is_key { key_columns.push(column_long_int); }
                }

                // If it's a `String`... things gets complicated.
                FieldType::StringU8 | FieldType::StringU16 => {

                    // Check for references.
                    match field.field_is_reference {

                        // If it's a reference...
                        Some(ref origin) => {

                            // We create a vector to hold all the possible values.
                            let mut origin_combo_data = vec![];

                            // If we have a database PackFile to check for refs...
                            if let Some(ref dependency_database) = *dependency_database {

                                // For each table in the database...
                                for table in dependency_database {

                                    // If it's actually a table.
                                    if table.path.len() >= 3 {

                                        // If it's our original table...
                                        if table.path[0] == "db" && table.path[1] == format!("{}_tables", origin.0) {

                                            // If we could decode it...
                                            if let Ok(db) = DB::read(&table.data, &*table.path[1], master_schema) {

                                                // For each column in our original table...
                                                for (index, original_field) in db.data.table_definition.fields.iter().enumerate() {

                                                    // If it's our column...
                                                    if original_field.field_name == origin.1 {

                                                        // For each row...
                                                        for row in &db.data.entries {

                                                            // Check what's in our column in that row...
                                                            match row[index + 1] {

                                                                // And if it's a `String`, get his value.
                                                                DecodedData::StringU8(ref data) | DecodedData::StringU16(ref data) => origin_combo_data.push(data.to_owned()),
                                                                _ => {},
                                                            };

                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // For each table in our mod...
                                for table in &pack_file.borrow().data.packed_files {

                                    // If it's actually a table.
                                    if table.path.len() >= 3 {

                                        // If it's our original table...
                                        if table.path[0] == "db" && table.path[1] == format!("{}_tables", origin.0) {

                                            // If we could decode it...
                                            if let Ok(db) = DB::read(&table.data, &*table.path[1], master_schema) {

                                                // For each column in our original table...
                                                for (index, original_field) in db.data.table_definition.fields.iter().enumerate() {

                                                    // If it's our column...
                                                    if original_field.field_name == origin.1 {

                                                        // For each row...
                                                        for row in &db.data.entries {

                                                            // Check what's in our column in that row...
                                                            match row[index + 1] {

                                                                // And if it's a `String`...
                                                                DecodedData::StringU8(ref data) | DecodedData::StringU16(ref data) => {

                                                                    // If we don't have that field yet...
                                                                    if !origin_combo_data.contains(data) {

                                                                        // Add it to the list.
                                                                        origin_combo_data.push(data.to_owned());
                                                                    }
                                                                }
                                                                _ => {},
                                                            };
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // If we have at least one thing in the list for the combo...
                            if !origin_combo_data.is_empty() {

                                // Create the `ListStore` for the dropdown.
                                let combo_list_store = ListStore::new(&[String::static_type()]);

                                // Add all our "Reference" values to the dropdown's list.
                                for row in &origin_combo_data {
                                    combo_list_store.insert_with_values(None, &[0], &[&row]);
                                }

                                // We create the cell and the column.
                                let cell_string = CellRendererCombo::new();
                                let column_string = TreeViewColumn::new();

                                // Config for the cell.
                                cell_string.set_property_editable(true);
                                cell_string.set_property_model(Some(&combo_list_store));
                                cell_string.set_property_text_column(0);

                                // Config for the column.
                                column_string.set_title(&field_name);
                                column_string.set_clickable(true);
                                column_string.set_resizable(true);
                                column_string.set_min_width(50);
                                column_string.set_sizing(TreeViewColumnSizing::GrowOnly);
                                column_string.set_alignment(0.5);
                                column_string.set_sort_column_id((index + 1) as i32);
                                column_string.pack_start(&cell_string, true);
                                column_string.add_attribute(&cell_string, "text", (index + 1) as i32);
                                tree_view.append_column(&column_string);
                                list_cell_reference.push(cell_string);

                                // If it's marked as a "key" filed, add it to our "key" columns list.
                                if field.field_is_key { key_columns.push(column_string); }
                            }

                            // Otherwise, we fallback to the usual method.
                            else {

                                // We create the cell and the column.
                                let cell_string = CellRendererText::new();
                                let column_string = TreeViewColumn::new();

                                // Config for the cell.
                                cell_string.set_property_editable(true);
                                cell_string.set_property_placeholder_text(Some("Obligatory String"));

                                // Config for the column.
                                column_string.set_title(&field_name);
                                column_string.set_clickable(true);
                                column_string.set_resizable(true);
                                column_string.set_min_width(50);
                                column_string.set_sizing(TreeViewColumnSizing::GrowOnly);
                                column_string.set_alignment(0.5);
                                column_string.set_sort_column_id((index + 1) as i32);
                                column_string.pack_start(&cell_string, true);
                                column_string.add_attribute(&cell_string, "text", (index + 1) as i32);
                                tree_view.append_column(&column_string);
                                list_cell_string.push(cell_string);

                                // If it's marked as a "key" filed, add it to our "key" columns list.
                                if field.field_is_key { key_columns.push(column_string); }
                            }
                        },

                        // If it's not a reference, keep the normal behavior.
                        None => {

                            // We create the cell and the column.
                            let cell_string = CellRendererText::new();
                            let column_string = TreeViewColumn::new();

                            // Config for the cell.
                            cell_string.set_property_editable(true);
                            cell_string.set_property_placeholder_text(Some("Obligatory String"));

                            // Config for the column.
                            column_string.set_title(&field_name);
                            column_string.set_clickable(true);
                            column_string.set_resizable(true);
                            column_string.set_min_width(50);
                            column_string.set_sizing(TreeViewColumnSizing::GrowOnly);
                            column_string.set_alignment(0.5);
                            column_string.set_sort_column_id((index + 1) as i32);
                            column_string.pack_start(&cell_string, true);
                            column_string.add_attribute(&cell_string, "text", (index + 1) as i32);
                            tree_view.append_column(&column_string);
                            list_cell_string.push(cell_string);

                            // If it's marked as a "key" filed, add it to our "key" columns list.
                            if field.field_is_key { key_columns.push(column_string); }
                        }
                    }
                }
                FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {

                    // Check for references.
                    match field.field_is_reference {

                        // If it's a reference...
                        Some(ref origin) => {

                            // We create a vector to hold all the possible values.
                            let mut origin_combo_data = vec![];

                            // If we have a database PackFile to check for refs...
                            if let Some(ref dependency_database) = *dependency_database {

                                // For each table in the database...
                                for table in dependency_database {

                                    // If it's actually a table.
                                    if table.path.len() >= 3 {

                                        // If it's our original table...
                                        if table.path[0] == "db" && table.path[1] == format!("{}_tables", origin.0) {

                                            // If we could decode it...
                                            if let Ok(db) = DB::read(&table.data, &*table.path[1], master_schema) {

                                                // For each column in our original table...
                                                for (index, original_field) in db.data.table_definition.fields.iter().enumerate() {

                                                    // If it's our column...
                                                    if original_field.field_name == origin.1 {

                                                        // For each row...
                                                        for row in &db.data.entries {

                                                            // Check what's in our column in that row...
                                                            match row[index + 1] {

                                                                // And if it's a `String`, get his value.
                                                                DecodedData::OptionalStringU8(ref data) | DecodedData::OptionalStringU16(ref data) => origin_combo_data.push(data.to_owned()),
                                                                _ => {},
                                                            };

                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // For each table in our mod...
                                for table in &pack_file.borrow().data.packed_files {

                                    // If it's actually a table.
                                    if table.path.len() >= 3 {

                                        // If it's our original table...
                                        if table.path[0] == "db" && table.path[1] == format!("{}_tables", origin.0) {

                                            // If we could decode it...
                                            if let Ok(db) = DB::read(&table.data, &*table.path[1], master_schema) {

                                                // For each column in our original table...
                                                for (index, original_field) in db.data.table_definition.fields.iter().enumerate() {

                                                    // If it's our column...
                                                    if original_field.field_name == origin.1 {

                                                        // For each row...
                                                        for row in &db.data.entries {

                                                            // Check what's in our column in that row...
                                                            match row[index + 1] {

                                                                // And if it's a `String`...
                                                                DecodedData::OptionalStringU8(ref data) | DecodedData::OptionalStringU16(ref data) => {

                                                                    // If we don't have that field yet...
                                                                    if !origin_combo_data.contains(data) {

                                                                        // Add it to the list.
                                                                        origin_combo_data.push(data.to_owned());
                                                                    }
                                                                }
                                                                _ => {},
                                                            };
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // If we have at least one thing in the list for the combo...
                            if !origin_combo_data.is_empty() {

                                // Create the `ListStore` for the dropdown.
                                let combo_list_store = ListStore::new(&[String::static_type()]);

                                // Add all our "Reference" values to the dropdown's list.
                                for row in &origin_combo_data {
                                    combo_list_store.insert_with_values(None, &[0], &[&row]);
                                }

                                // We create the cell and the column.
                                let cell_optional_string = CellRendererCombo::new();
                                let column_optional_string = TreeViewColumn::new();

                                // Config for the cell.
                                cell_optional_string.set_property_editable(true);
                                cell_optional_string.set_property_model(Some(&combo_list_store));
                                cell_optional_string.set_property_text_column(0);

                                // Config for the column.
                                column_optional_string.set_title(&field_name);
                                column_optional_string.set_clickable(true);
                                column_optional_string.set_resizable(true);
                                column_optional_string.set_min_width(50);
                                column_optional_string.set_sizing(TreeViewColumnSizing::GrowOnly);
                                column_optional_string.set_alignment(0.5);
                                column_optional_string.set_sort_column_id((index + 1) as i32);
                                column_optional_string.pack_start(&cell_optional_string, true);
                                column_optional_string.add_attribute(&cell_optional_string, "text", (index + 1) as i32);
                                tree_view.append_column(&column_optional_string);
                                list_cell_reference.push(cell_optional_string);

                                // If it's marked as a "key" filed, add it to our "key" columns list.
                                if field.field_is_key { key_columns.push(column_optional_string); }
                            }

                            // Otherwise, we fallback to the usual method.
                            else {

                                // We create the cell and the column.
                                let cell_optional_string = CellRendererText::new();
                                let column_optional_string = TreeViewColumn::new();

                                // Config for the cell.
                                cell_optional_string.set_property_editable(true);
                                cell_optional_string.set_property_placeholder_text(Some("Optional String"));

                                // Config for the column.
                                column_optional_string.set_title(&field_name);
                                column_optional_string.set_clickable(true);
                                column_optional_string.set_resizable(true);
                                column_optional_string.set_min_width(50);
                                column_optional_string.set_sizing(TreeViewColumnSizing::GrowOnly);
                                column_optional_string.set_alignment(0.5);
                                column_optional_string.set_sort_column_id((index + 1) as i32);
                                column_optional_string.pack_start(&cell_optional_string, true);
                                column_optional_string.add_attribute(&cell_optional_string, "text", (index + 1) as i32);
                                tree_view.append_column(&column_optional_string);
                                list_cell_optional_string.push(cell_optional_string);

                                // If it's marked as a "key" filed, add it to our "key" columns list.
                                if field.field_is_key { key_columns.push(column_optional_string); }
                            }
                        },

                        // If it's not a reference, keep the normal behavior.
                        None => {

                            // We create the cell and the column.
                            let cell_optional_string = CellRendererText::new();
                            let column_optional_string = TreeViewColumn::new();

                            // Config for the cell.
                            cell_optional_string.set_property_editable(true);
                            cell_optional_string.set_property_placeholder_text(Some("Optional String"));

                            // Config for the column.
                            column_optional_string.set_title(&field_name);
                            column_optional_string.set_clickable(true);
                            column_optional_string.set_resizable(true);
                            column_optional_string.set_min_width(50);
                            column_optional_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                            column_optional_string.set_alignment(0.5);
                            column_optional_string.set_sort_column_id((index + 1) as i32);
                            column_optional_string.pack_start(&cell_optional_string, true);
                            column_optional_string.add_attribute(&cell_optional_string, "text", (index + 1) as i32);
                            tree_view.append_column(&column_optional_string);
                            list_cell_optional_string.push(cell_optional_string);

                            // If it's marked as a "key" filed, add it to our "key" columns list.
                            if field.field_is_key { key_columns.push(column_optional_string); }
                        }
                    }
                }
            }
        }

        // // We create the cell and the column that's going to serve as "filler" column at the end.
        let cell_fill = CellRendererText::new();
        let column_fill = TreeViewColumn::new();

        // Config for the column.
        column_fill.set_min_width(0);
        column_fill.pack_start(&cell_fill, true);
        tree_view.append_column(&column_fill);

        // This should put the key columns in order.
        for column in key_columns.iter().rev() {
            tree_view.move_column_after(column, Some(&column_index));
        }

        // This is the logic to set the "Search" column.
        // For each field...
        for (index, field) in table_definition.fields.iter().enumerate() {

            // If there is one named "key"...
            if field.field_name == "key" {

                // Set it as the search column.
                tree_view.set_search_column((index + 1) as i32);

                // Stop the loop.
                break;
            }
        }

        // If we haven't set it yet...
        if tree_view.get_search_column() == 0 {

            // If there are any "Key" columns...
            if !key_columns.is_empty() {

                // Set the first "Key" column as the search column.
                tree_view.set_search_column(key_columns[0].get_sort_column_id());
            }

            // Otherwise, just use the first Non-Index column.
            else { tree_view.set_search_column(1); }
        }

        // Here we create the Popover menu. It's created and destroyed with the table because otherwise
        // it'll start crashing when changing tables and trying to delete stuff. Stupid menu. Also, it can't
        // be created from a `MenuModel` like the rest, because `MenuModel`s can't hold an `Entry`.
        let context_menu = Popover::new(&tree_view);

        // Create the `Grid` that'll hold all the buttons in the Contextual Menu.
        let context_menu_grid = Grid::new();
        context_menu_grid.set_border_width(6);

        // Clean the accelerators stuff.
        remove_temporal_accelerators(&application);

        // Create the "Add row/s" button.
        let add_rows_button = ModelButton::new();
        add_rows_button.set_property_text(Some("Add rows:"));
        add_rows_button.set_action_name("app.packedfile_db_add_rows");

        // Create the entry to specify the amount of rows you want to add.
        let add_rows_entry = Entry::new();
        let add_rows_entry_buffer = add_rows_entry.get_buffer();
        add_rows_entry.set_alignment(1.0);
        add_rows_entry.set_width_chars(8);
        add_rows_entry.set_icon_from_icon_name(EntryIconPosition::Primary, "go-last");
        add_rows_entry.set_has_frame(false);
        add_rows_entry_buffer.set_max_length(Some(4));
        add_rows_entry_buffer.set_text("1");

        // Create the "Delete row/s" button.
        let delete_rows_button = ModelButton::new();
        delete_rows_button.set_property_text(Some("Delete row/s"));
        delete_rows_button.set_action_name("app.packedfile_db_delete_rows");

        // Create the separator between "Delete row/s" and the copy/paste buttons.
        let separator_1 = Separator::new(Orientation::Vertical);

        // Create the "Copy cell" button.
        let copy_cell_button = ModelButton::new();
        copy_cell_button.set_property_text(Some("Copy cell"));
        copy_cell_button.set_action_name("app.packedfile_db_copy_cell");

        // Create the "Paste cell" button.
        let paste_cell_button = ModelButton::new();
        paste_cell_button.set_property_text(Some("Paste cell"));
        paste_cell_button.set_action_name("app.packedfile_db_paste_cell");

        // Create the "Clone row/s" button.
        let clone_rows_button = ModelButton::new();
        clone_rows_button.set_property_text(Some("Clone row/s"));
        clone_rows_button.set_action_name("app.packedfile_db_clone_rows");

        // Create the "Copy row/s" button.
        let copy_rows_button = ModelButton::new();
        copy_rows_button.set_property_text(Some("Copy row/s"));
        copy_rows_button.set_action_name("app.packedfile_db_copy_rows");

        // Create the "Paste row/s" button.
        let paste_rows_button = ModelButton::new();
        paste_rows_button.set_property_text(Some("Paste row/s"));
        paste_rows_button.set_action_name("app.packedfile_db_paste_rows");

        // Create the "Copy column/s" button.
        let copy_columns_button = ModelButton::new();
        copy_columns_button.set_property_text(Some("Copy column/s"));
        copy_columns_button.set_action_name("app.packedfile_db_copy_columns");

        // Create the "Paste column/s" button.
        let paste_columns_button = ModelButton::new();
        paste_columns_button.set_property_text(Some("Paste column/s"));
        paste_columns_button.set_action_name("app.packedfile_db_paste_columns");

        // Create the separator between the "Import/Export" buttons and the rest.
        let separator_2 = Separator::new(Orientation::Vertical);

        // Create the "Import from TSV" button.
        let import_tsv_button = ModelButton::new();
        import_tsv_button.set_property_text(Some("Import from TSV"));
        import_tsv_button.set_action_name("app.packedfile_db_import_tsv");

        // Create the "Export to TSV" button.
        let export_tsv_button = ModelButton::new();
        export_tsv_button.set_property_text(Some("Export to TSV"));
        export_tsv_button.set_action_name("app.packedfile_db_export_tsv");

        // Right-click menu actions.
        let add_rows = SimpleAction::new("packedfile_db_add_rows", None);
        let delete_rows = SimpleAction::new("packedfile_db_delete_rows", None);
        let copy_cell = SimpleAction::new("packedfile_db_copy_cell", None);
        let paste_cell = SimpleAction::new("packedfile_db_paste_cell", None);
        let clone_rows = SimpleAction::new("packedfile_db_clone_rows", None);
        let copy_rows = SimpleAction::new("packedfile_db_copy_rows", None);
        let paste_rows = SimpleAction::new("packedfile_db_paste_rows", None);
        let copy_columns = SimpleAction::new("packedfile_db_copy_columns", None);
        let paste_columns = SimpleAction::new("packedfile_db_paste_columns", None);
        let import_tsv = SimpleAction::new("packedfile_db_import_tsv", None);
        let export_tsv = SimpleAction::new("packedfile_db_export_tsv", None);

        application.add_action(&add_rows);
        application.add_action(&delete_rows);
        application.add_action(&copy_cell);
        application.add_action(&paste_cell);
        application.add_action(&clone_rows);
        application.add_action(&copy_rows);
        application.add_action(&paste_rows);
        application.add_action(&copy_columns);
        application.add_action(&paste_columns);
        application.add_action(&import_tsv);
        application.add_action(&export_tsv);

        // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
        application.set_accels_for_action("app.packedfile_db_add_rows", &["<Primary><Shift>a"]);
        application.set_accels_for_action("app.packedfile_db_delete_rows", &["<Shift>Delete"]);
        application.set_accels_for_action("app.packedfile_db_copy_cell", &["<Primary>c"]);
        application.set_accels_for_action("app.packedfile_db_paste_cell", &["<Primary>v"]);
        application.set_accels_for_action("app.packedfile_db_clone_rows", &["<Primary><Shift>d"]);
        application.set_accels_for_action("app.packedfile_db_copy_rows", &["<Primary>z"]);
        application.set_accels_for_action("app.packedfile_db_paste_rows", &["<Primary>x"]);
        application.set_accels_for_action("app.packedfile_db_copy_columns", &["<Primary>k"]);
        application.set_accels_for_action("app.packedfile_db_paste_columns", &["<Primary>l"]);
        application.set_accels_for_action("app.packedfile_db_import_tsv", &["<Primary><Shift>i"]);
        application.set_accels_for_action("app.packedfile_db_export_tsv", &["<Primary><Shift>e"]);

        // Some actions need to start disabled.
        delete_rows.set_enabled(false);
        copy_cell.set_enabled(false);
        clone_rows.set_enabled(false);
        copy_rows.set_enabled(false);
        copy_columns.set_enabled(false);
        paste_cell.set_enabled(false);
        paste_rows.set_enabled(true);
        paste_columns.set_enabled(true);

        // Attach all the stuff to the Context Menu `Grid`.
        context_menu_grid.attach(&add_rows_button, 0, 0, 1, 1);
        context_menu_grid.attach(&add_rows_entry, 1, 0, 1, 1);
        context_menu_grid.attach(&delete_rows_button, 0, 1, 2, 1);
        context_menu_grid.attach(&separator_1, 0, 2, 2, 1);
        context_menu_grid.attach(&copy_cell_button, 0, 3, 2, 1);
        context_menu_grid.attach(&paste_cell_button, 0, 4, 2, 1);
        context_menu_grid.attach(&clone_rows_button, 0, 5, 2, 1);
        context_menu_grid.attach(&copy_rows_button, 0, 6, 2, 1);
        context_menu_grid.attach(&paste_rows_button, 0, 7, 2, 1);
        context_menu_grid.attach(&copy_columns_button, 0, 8, 2, 1);
        context_menu_grid.attach(&paste_columns_button, 0, 9, 2, 1);
        context_menu_grid.attach(&separator_2, 0, 10, 2, 1);
        context_menu_grid.attach(&import_tsv_button, 0, 11, 2, 1);
        context_menu_grid.attach(&export_tsv_button, 0, 12, 2, 1);

        // Add the `Grid` to the Context Menu and show it.
        context_menu.add(&context_menu_grid);
        context_menu.show_all();

        // Make a `ScrolledWindow` to put the `TreeView` into it.
        let packed_file_data_scroll = ScrolledWindow::new(None, None);
        packed_file_data_scroll.set_hexpand(true);
        packed_file_data_scroll.set_vexpand(true);

        // Add the `TreeView` to the `ScrolledWindow`, the `ScrolledWindow` to the main `Grid`, and show it.
        packed_file_data_scroll.add(&tree_view);
        app_ui.packed_file_data_display.attach(&packed_file_data_scroll, 0, 1, 1, 1);
        app_ui.packed_file_data_display.show_all();

        // Hide the Context Menu by default.
        context_menu.hide();

        // Create the PackedFileDBTreeView.
        let table = PackedFileDBTreeView {
            tree_view,
            list_store,
            list_cell_bool,
            list_cell_float,
            list_cell_integer,
            list_cell_long_integer,
            list_cell_string,
            list_cell_optional_string,
            list_cell_reference,
            context_menu,
            add_rows_entry,
        };

        // Try to load the data from the table to the `TreeView`.
        if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view (
            &packed_file_decoded.borrow().data,
            &table.list_store
        ) {
            return Err(error);
        }

        // Events for the DB Table.

        // When a tooltip gets triggered...
        table.tree_view.connect_query_tooltip(clone!(
            table_definition => move |tree_view, x, y,_, tooltip| {

                // Get the coordinates of the cell under the cursor.
                let cell_coords: (i32, i32) = tree_view.convert_widget_to_tree_coords(x, y);

                // If we got a column...
                if let Some(position) = tree_view.get_path_at_pos(cell_coords.0, cell_coords.1) {
                    if let Some(column) = position.1 {

                        // Get his ID.
                        let column = column.get_sort_column_id();

                        // We don't want to check the tooltip for the Index column, nor for the fake end column.
                        if column >= 1 && (column as usize) <= table_definition.fields.len() {

                            // If it's a reference, we put to what cell is referencing in the tooltip.
                            let tooltip_text: String =

                                if let Some(ref reference) = table_definition.fields[column as usize - 1].field_is_reference {
                                    if !table_definition.fields[column as usize - 1].field_description.is_empty() {
                                        format!("{}\n\nThis column is a reference to \"{}/{}\".",
                                            table_definition.fields[column as usize - 1].field_description,
                                            reference.0,
                                            reference.1
                                        )
                                    }
                                    else {
                                        format!("This column is a reference to \"{}/{}\".",
                                            reference.0,
                                            reference.1
                                        )
                                    }

                                }

                                // Otherwise, use the text from the description of that field.
                                else { table_definition.fields[column as usize - 1].field_description.to_owned() };

                            // If we got text to display, use it.
                            if !tooltip_text.is_empty() {
                                tooltip.set_text(&*tooltip_text);

                                // Return true to show the tooltip.
                                return true
                            }
                        }
                    }
                }

                // In any other case, return false.
                false
            }
        ));

        // Context menu actions stuff.
        {

            // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
            // Then we calculate the position where the popup must aim, and show it.
            //
            // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
            table.tree_view.connect_button_release_event(clone!(
                table,
                app_ui,
                paste_rows,
                paste_columns,
                table_definition => move |tree_view, button| {

                    // If we clicked the right mouse button...
                    if button.get_button() == 3 {

                        // If we got text in the `Clipboard`...
                        if app_ui.clipboard.wait_for_text().is_some() {

                            // If the data in the clipboard is a valid row, we enable "Paste rows".
                            if check_clipboard_row(&app_ui, &table_definition) { paste_rows.set_enabled(true); }

                            // Otherwise, we disable the "Paste rows" action.
                            else { paste_rows.set_enabled(false); }

                            // If we have a column selected...
                            if let Some(column) = table.tree_view.get_cursor().1 {

                                // If the data in the clipboard is a valid column, we enable "Paste columns".
                                if check_clipboard_column(&app_ui, &table_definition, &column) { paste_columns.set_enabled(true); }

                                // Otherwise, we disable the "Paste columns" action.
                                else { paste_columns.set_enabled(false); }
                            }

                            // Otherwise, we disable the "Paste columns" action.
                            else { paste_columns.set_enabled(false); }
                        }
                        else {
                            paste_rows.set_enabled(false);
                            paste_columns.set_enabled(false);
                        }

                        table.context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                        table.context_menu.popup();
                    }

                    Inhibit(false)
                }
            ));

            // When we close the Contextual Menu.
            table.context_menu.connect_closed(clone!(
                paste_rows,
                paste_columns => move |_| {

                    // Enable both signals, as there are checks when they are emited to stop them if
                    // it's not possible to paste anything.
                    paste_rows.set_enabled(true);
                    paste_columns.set_enabled(true);
                }
            ));

            // We we change the selection, we enable or disable the different actions of the Contextual Menu.
            table.tree_view.connect_cursor_changed(clone!(
                app_ui,
                copy_cell,
                copy_rows,
                copy_columns,
                clone_rows,
                paste_cell,
                delete_rows => move |tree_view| {

                    // If we have something selected...
                    if tree_view.get_selection().count_selected_rows() > 0 {

                        // Allow to delete, clone and copy.
                        copy_cell.set_enabled(true);
                        copy_rows.set_enabled(true);
                        copy_columns.set_enabled(true);
                        clone_rows.set_enabled(true);
                        delete_rows.set_enabled(true);
                    }

                    // Otherwise, disable them.
                    else {
                        copy_cell.set_enabled(false);
                        copy_rows.set_enabled(false);
                        copy_columns.set_enabled(false);
                        clone_rows.set_enabled(false);
                        delete_rows.set_enabled(false);
                    }

                    // If we got text in the `Clipboard`...
                    if app_ui.clipboard.wait_for_text().is_some() {

                        // Get the selected cell, if any.
                        let selected_cell = tree_view.get_cursor();

                        // If we have a cell selected and it's not in the index column, enable "Paste Cell".
                        if selected_cell.0.is_some() {
                            if let Some(column) = selected_cell.1 {
                                if column.get_sort_column_id() > 0 {
                                    paste_cell.set_enabled(true);
                                }

                                // If the cell is invalid, disable the copy for it.
                                else if column.get_sort_column_id() < 0 {
                                    copy_cell.set_enabled(false);
                                    copy_columns.set_enabled(false);
                                    paste_cell.set_enabled(false);
                                }
                                else {
                                    paste_cell.set_enabled(false);
                                }
                            }
                            else { paste_cell.set_enabled(false); }
                        }
                        else { paste_cell.set_enabled(false); }
                    }
                    else { paste_cell.set_enabled(false); }
                }
            ));

            // When we hit the "Add row" button.
            add_rows.connect_activate(clone!(
                table_definition,
                app_ui,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // First, we check if the input is a valid number, as I'm already seeing people
                        // trying to add "two" rows.
                        match table.add_rows_entry.get_buffer().get_text().parse::<u32>() {

                            // If the number is valid...
                            Ok(number_rows) => {

                                // For each row...
                                for _ in 0..number_rows {

                                    // Add an empty row at the end of the `TreeView`, filling his index.
                                    let new_row = table.list_store.append();
                                    table.list_store.set_value(&new_row, 0, &"New".to_value());

                                    // For each column we have...
                                    for column in 1..(table_definition.fields.len() + 1) {

                                        match table_definition.fields[column - 1].field_type {
                                            FieldType::Boolean => table.list_store.set_value(&new_row, column as u32, &false.to_value()),
                                            FieldType::Float => table.list_store.set_value(&new_row, column as u32, &0.0f32.to_string().to_value()),
                                            FieldType::Integer | FieldType::LongInteger => table.list_store.set_value(&new_row, column as u32, &0.to_value()),
                                            FieldType::StringU8 | FieldType::StringU16 | FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                                                table.list_store.set_value(&new_row, column as u32, &String::new().to_value());
                                            }
                                        }
                                    }
                                }
                            }

                            // If it's not a valid number, report it.
                            Err(_) => show_dialog(&app_ui.window, false, "You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows?"),
                        }
                    }
                }
            ));

            // When we hit the "Delete row" button.
            delete_rows.connect_activate(clone!(
                table_definition,
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the selected row's `TreePath`.
                        let selected_rows = table.tree_view.get_selection().get_selected_rows().0;

                        // If we have any row selected...
                        if !selected_rows.is_empty() {

                            // For each row (in reverse)...
                            for row in (0..selected_rows.len()).rev() {

                                // Remove it.
                                table.list_store.remove(&table.list_store.get_iter(&selected_rows[row]).unwrap());
                            }

                            // Try to save the new data from the `TreeView`.
                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                // If we succeed...
                                Ok(data) => {

                                    // Replace our current decoded data with the new one.
                                    packed_file_decoded.borrow_mut().data.entries = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index
                                    ) {
                                        show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                }

                                // If there is an error, report it.
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                }
            ));

            // When we hit the "Copy cell" button.
            copy_cell.connect_activate(clone!(
                app_ui,
                table_definition,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the the focused cell.
                        let focused_cell = table.tree_view.get_cursor();

                        // If there is a focused `TreePath`...
                        if let Some(tree_path) = focused_cell.0 {

                            // And a focused `TreeViewColumn`...
                            if let Some(column) = focused_cell.1 {

                                // Get his `TreeIter`.
                                let row = table.list_store.get_iter(&tree_path).unwrap();

                                // Get his column ID.
                                let column = column.get_sort_column_id();

                                // If the column is not a dummy one.
                                if column >= 0 {

                                    // If the cell is the index...
                                    if column == 0 {

                                        // Get his value and put it into the `Clipboard`.
                                        app_ui.clipboard.set_text(&table.list_store.get_value(&row, 0).get::<String>().unwrap(),);
                                    }

                                    // Otherwise...
                                    else {

                                        // Check his `field_type`...
                                        let data = match table_definition.fields[column as usize - 1].field_type {

                                            // If it's a boolean, get "true" or "false".
                                            FieldType::Boolean => {
                                                match table.list_store.get_value(&row, column).get::<bool>().unwrap() {
                                                    true => "true".to_owned(),
                                                    false => "false".to_owned(),
                                                }
                                            }

                                            // If it's an Integer or a Long Integer, turn it into a `String`. Don't know why, but otherwise integer columns crash the program.
                                            FieldType::Integer => format!("{}", table.list_store.get_value(&row, column).get::<i32>().unwrap()),
                                            FieldType::LongInteger => format!("{}", table.list_store.get_value(&row, column).get::<i64>().unwrap()),

                                            // If it's any other type, just decode it as `String`.
                                            _ => table.list_store.get_value(&row, column).get::<String>().unwrap(),
                                        };

                                        // Put the data into the `Clipboard`.
                                        app_ui.clipboard.set_text(&data);
                                    }
                                }
                            }
                        }
                    }
                }
            ));

            // When we hit the "Paste cell" button.
            paste_cell.connect_activate(clone!(
                app_ui,
                table_definition,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the the focused cell.
                        let focused_cell = table.tree_view.get_cursor();

                        // If there is a focused `TreePath`...
                        if let Some(tree_path) = focused_cell.0 {

                            // And a focused `TreeViewColumn`...
                            if let Some(column) = focused_cell.1 {

                                // If we got text from the `Clipboard`...
                                if let Some(data) = app_ui.clipboard.wait_for_text() {

                                    // Get his `TreeIter`.
                                    let row = table.list_store.get_iter(&tree_path).unwrap();

                                    // Get his column ID.
                                    let column = column.get_sort_column_id() as u32;

                                    // If the cell is in a valid column (neither index nor dummy)...
                                    if column > 0 {

                                        // Check his `field_type`...
                                        match table_definition.fields[column as usize - 1].field_type {

                                            // If it's a boolean, get "true" or "false".
                                            FieldType::Boolean => {
                                                let state = if data == "true" { true } else if data == "false" { false } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".")
                                                };
                                                table.list_store.set_value(&row, column, &state.to_value());
                                            }
                                            FieldType::Integer => {
                                                if let Ok(data) = data.parse::<i32>() {
                                                    table.list_store.set_value(&row, column, &data.to_value());
                                                } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I32.")
                                                };
                                            },
                                            FieldType::LongInteger => {
                                                if let Ok(data) = data.parse::<i64>() {
                                                    table.list_store.set_value(&row, column, &data.to_value());
                                                } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I64.")
                                                };
                                            },
                                            FieldType::Float => {
                                                if data.parse::<f32>().is_ok() {
                                                    table.list_store.set_value(&row, column, &data.to_value());
                                                } else { return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid F32.") }
                                            },

                                            // All these are Strings, so it can be together,
                                            FieldType::StringU8 |
                                            FieldType::StringU16 |
                                            FieldType::OptionalStringU8 |
                                            FieldType::OptionalStringU16 => table.list_store.set_value(&row, column, &data.to_value()),
                                        };

                                        // Try to save the new data from the `TreeView`.
                                        match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                            // If we succeed...
                                            Ok(data) => {

                                                // Replace our current decoded data with the new one.
                                                packed_file_decoded.borrow_mut().data.entries = data;

                                                // Try to save the changes to the PackFile. If there is an error, report it.
                                                if let Err(error) = update_packed_file_data_db(
                                                    &*packed_file_decoded.borrow_mut(),
                                                    &mut *pack_file.borrow_mut(),
                                                    packed_file_decoded_index
                                                ) {
                                                    show_dialog(&app_ui.window, false, error.cause());
                                                }

                                                // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                            }

                                            // If there is an error, report it.
                                            Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            ));

            // When we hit the "Clone row" button.
            clone_rows.connect_activate(clone!(
                table_definition,
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the selected row's `TreePath`.
                        let selected_rows = table.tree_view.get_selection().get_selected_rows().0;

                        // If we have any row selected...
                        if !selected_rows.is_empty() {

                            // For each selected row...
                            for tree_path in &selected_rows {

                                // We get the old `TreeIter` and create a new one.
                                let old_row = table.list_store.get_iter(tree_path).unwrap();
                                let new_row = table.list_store.append();

                                // For each column...
                                for column in 0..(table_definition.fields.len() + 1) {

                                    // First column it's always the index. Any other column, just copy the values from one `TreeIter` to the other.
                                    match column {
                                        0 => table.list_store.set_value(&new_row, column as u32, &gtk::ToValue::to_value(&format!("New"))),
                                        _ => table.list_store.set_value(&new_row, column as u32, &table.list_store.get_value(&old_row, column as i32)),
                                    }
                                }
                            }

                            // Try to save the new data from the `TreeView`.
                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                // If we succeed...
                                Ok(data) => {

                                    // Replace our current decoded data with the new one.
                                    packed_file_decoded.borrow_mut().data.entries = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index
                                    ) {
                                        show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                }

                                // If there is an error, report it.
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                }
            ));

            // When we hit the "Copy row" button.
            copy_rows.connect_activate(clone!(
                table_definition,
                app_ui,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the selected rows.
                        let selected_rows = table.tree_view.get_selection().get_selected_rows().0;

                        // Get the list of `TreeIter`s we want to copy.
                        let tree_iter_list = selected_rows.iter().map(|row| table.list_store.get_iter(row).unwrap()).collect::<Vec<TreeIter>>();

                        // Create the `String` that will copy the row that will bring that shit of TLJ down.
                        let mut copy_string = String::new();

                        // For each row...
                        for row in &tree_iter_list {

                            // For each column...
                            for column in 1..(table_definition.fields.len() + 1) {

                                // Check his `field_type`...
                                let data = match table_definition.fields[column as usize - 1].field_type {

                                    // If it's a boolean, get "true" or "false".
                                    FieldType::Boolean => {
                                        match table.list_store.get_value(row, column as i32).get::<bool>().unwrap() {
                                            true => "true".to_owned(),
                                            false => "false".to_owned(),
                                        }
                                    }

                                    // If it's an Integer or a Long Integer, turn it into a `String`. Don't know why, but otherwise integer columns crash the program.
                                    FieldType::Integer => format!("{}", table.list_store.get_value(row, column as i32).get::<i32>().unwrap()),
                                    FieldType::LongInteger => format!("{}", table.list_store.get_value(row, column as i32).get::<i64>().unwrap()),

                                    // If it's any other type, just decode it as `String`.
                                    _ => table.list_store.get_value(row, column as i32).get::<String>().unwrap(),
                                };

                                // Add the text to the copied string.
                                copy_string.push_str(&data);

                                // If it's not the last column...
                                if column < table_definition.fields.len() {

                                    // Put a tab between fields, so excel understand them.
                                    copy_string.push('\t');
                                }
                            }

                            // Add a newline at the end of every row.
                            copy_string.push('\n');
                        }

                        // Pass all the copied rows to the clipboard.
                        app_ui.clipboard.set_text(&copy_string);
                    }
                }
            ));

            // When we hit the "Paste row" button.
            paste_rows.connect_activate(clone!(
                table_definition,
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Before anything else, we check if the data in the `Clipboard` includes ONLY valid rows.
                        if check_clipboard_row(&app_ui, &table_definition) {

                            // When it gets the data from the `Clipboard`....
                            if let Some(data) = app_ui.clipboard.wait_for_text() {

                                // Get the definitions for this table.
                                let fields_type = table_definition.fields.iter().map(|x| x.field_type.clone()).collect::<Vec<FieldType>>();

                                // Store here all the decoded fields.
                                let mut fields_data = vec![];

                                // For each row in the data we received...
                                for row in data.lines() {

                                    // Get all the data from his fields.
                                    fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
                                }

                                // Get the selected row, if there is any.
                                let selected_row = table.tree_view.get_selection().get_selected_rows().0;

                                // If there is at least one line selected, use it as "base" to paste.
                                let mut tree_iter = if !selected_row.is_empty() {
                                    table.list_store.get_iter(&selected_row[0]).unwrap()
                                }

                                // Otherwise, append a new `TreeIter` to the `TreeView`, and use it.
                                else { table.list_store.append() };

                                // For each row in our fields_data list...
                                for (row_index, row) in fields_data.iter().enumerate() {

                                    // Fill the "Index" column with "New".
                                    table.list_store.set_value(&tree_iter, 0, &"New".to_value());

                                    // For each field in a row...
                                    for (index, field) in row.iter().enumerate() {

                                        // Check if that field exists in the table.
                                        let field_type = fields_type.get(index);

                                        // If it exists...
                                        if let Some(field_type) = field_type {

                                            // Check his `field_type`. We can skip all the safety checks here, because if we hit a CTD here,
                                            // something it's broken in the checking function above and needs to be fixed there.
                                            match *field_type {

                                                // If it's a boolean, get "true" or "false".
                                                FieldType::Boolean => table.list_store.set_value(&tree_iter, (index + 1) as u32, &(if field == "true" { true } else { false }).to_value()),
                                                FieldType::Integer => table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.parse::<i32>().unwrap().to_value()),
                                                FieldType::LongInteger => table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.parse::<i64>().unwrap().to_value()),

                                                // Anything else, just put it into his column with the needed type.
                                                _ => table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value()),
                                            };
                                        }
                                    }

                                    // Move to the next row. If it doesn't exist and it's not the last loop....
                                    if !table.list_store.iter_next(&tree_iter) && row_index < (fields_data.len() - 1) {

                                        // Create it.
                                        tree_iter = table.list_store.append();
                                    }
                                }

                                // Try to save the new data from the `TreeView`.
                                match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                    // If we succeed...
                                    Ok(data) => {

                                        // Replace our current decoded data with the new one.
                                        packed_file_decoded.borrow_mut().data.entries = data;

                                        // Try to save the changes to the PackFile. If there is an error, report it.
                                        if let Err(error) = update_packed_file_data_db(
                                            &*packed_file_decoded.borrow_mut(),
                                            &mut *pack_file.borrow_mut(),
                                            packed_file_decoded_index
                                        ) {
                                            show_dialog(&app_ui.window, false, error.cause());
                                        }

                                        // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                        set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                    }

                                    // If there is an error, report it.
                                    Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                }
                            };
                        }
                    }
                }
            ));

            // When we hit the "Copy column" button.
            copy_columns.connect_activate(clone!(
                table_definition,
                app_ui,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // If there is a column selected...
                        if let Some(column) = table.tree_view.get_cursor().1 {

                            // Get the selected rows.
                            let selected_rows = table.tree_view.get_selection().get_selected_rows().0;

                            // Get the number of the column.
                            let mut column_number = column.get_sort_column_id() - 1;

                            // Ignore columns with < 0, as those are index or invalid columns.
                            if column_number >= 0 {

                                // Get the type of the column.
                                let column_type = table_definition.fields[column_number as usize].field_type.clone();

                                // Get again the real column number for the table.
                                column_number += 1;

                                // Get the list of `TreeIter`s we want to copy.
                                let tree_iter_list = selected_rows.iter().map(|row| table.list_store.get_iter(row).unwrap()).collect::<Vec<TreeIter>>();

                                // Create the `String` that will copy the row that will bring that shit of TLJ down.
                                let mut copy_string = String::new();

                                // For each row...
                                for (index, row) in tree_iter_list.iter().enumerate() {

                                    // Check his `field_type`...
                                    let data = match column_type {

                                        // If it's a boolean, get "true" or "false".
                                        FieldType::Boolean => {
                                            match table.list_store.get_value(row, column_number).get::<bool>().unwrap() {
                                                true => "true".to_owned(),
                                                false => "false".to_owned(),
                                            }
                                        }

                                        // If it's an Integer or a Long Integer, turn it into a `String`. Don't know why, but otherwise integer columns crash the program.
                                        FieldType::Integer => format!("{}", table.list_store.get_value(row, column_number).get::<i32>().unwrap()),
                                        FieldType::LongInteger => format!("{}", table.list_store.get_value(row, column_number).get::<i64>().unwrap()),

                                        // If it's any other type, just decode it as `String`.
                                        _ => table.list_store.get_value(row, column_number).get::<String>().unwrap(),
                                    };

                                    // Add the text to the copied string.
                                    copy_string.push_str(&data);

                                    // If it's not the last row...
                                    if index < tree_iter_list.len() - 1 {

                                        // Put an endline between fields, so excel understand them.
                                        copy_string.push('\n');
                                    }
                                }

                                // Pass the copied column to the clipboard.
                                app_ui.clipboard.set_text(&copy_string);
                            }
                        }
                    }
                }
            ));

            // When we hit the "Paste column" button.
            paste_columns.connect_activate(clone!(
                table_definition,
                app_ui,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // Hide the context menu.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Get the selected cell.
                        let cursor = table.tree_view.get_cursor();

                        // If there is a `TreePath` selected...
                        if let Some(tree_path) = cursor.0 {

                            // And a column selected...
                            if let Some(column) = cursor.1 {

                                // Before anything else, we check if the data in the `Clipboard` includes ONLY valid a valid column.
                                if check_clipboard_column(&app_ui, &table_definition, &column) {

                                    // When it gets the data from the `Clipboard`....
                                    if let Some(data) = app_ui.clipboard.wait_for_text() {

                                        // Get the number of the column.
                                        let mut column_number = column.get_sort_column_id() - 1;

                                        // Ignore columns with < 0, as those are index or invalid columns.
                                        if column_number >= 0 {

                                            // Get the type of the column.
                                            let column_type = table_definition.fields[column_number as usize].field_type.clone();

                                            // Get again the real column number for the table.
                                            column_number += 1;

                                            // Get the data to paste, separated by lines.
                                            let fields_data = data.lines().collect::<Vec<&str>>();

                                            // Get the selected rows.
                                            let selected_rows = table.tree_view.get_selection().get_selected_rows().0;

                                            // Get the selected row.
                                            let tree_iter = if !selected_rows.is_empty() {
                                                table.list_store.get_iter(&selected_rows[0]).unwrap()
                                            }
                                            else { table.list_store.get_iter(&tree_path).unwrap() };

                                            // For each line to paste...
                                            for field in &fields_data {

                                                // Paste the cell.
                                                match column_type {

                                                    // If it's a boolean, get "true" or "false".
                                                    FieldType::Boolean => table.list_store.set_value(&tree_iter, column_number as u32, &(if *field == "true" { true } else { false }).to_value()),
                                                    FieldType::Integer => table.list_store.set_value(&tree_iter, column_number as u32, &field.parse::<i32>().unwrap().to_value()),
                                                    FieldType::LongInteger => table.list_store.set_value(&tree_iter, column_number as u32, &field.parse::<i64>().unwrap().to_value()),

                                                    // Anything else, just put it into his column with the needed type.
                                                    _ => table.list_store.set_value(&tree_iter, column_number as u32, &field.to_value()),
                                                };

                                                // If there are no more rows, stop.
                                                if !table.list_store.iter_next(&tree_iter) { break; }
                                            }

                                            // Try to save the new data from the `TreeView`.
                                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                                // If we succeed...
                                                Ok(data) => {

                                                    // Replace our current decoded data with the new one.
                                                    packed_file_decoded.borrow_mut().data.entries = data;

                                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                                    if let Err(error) = update_packed_file_data_db(
                                                        &*packed_file_decoded.borrow_mut(),
                                                        &mut *pack_file.borrow_mut(),
                                                        packed_file_decoded_index
                                                    ) {
                                                        show_dialog(&app_ui.window, false, error.cause());
                                                    }

                                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                                }

                                                // If there is an error, report it.
                                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            ));

            // When we hit the "Import from TSV" button.
            import_tsv.connect_activate(clone!(
                app_ui,
                tree_path,
                pack_file,
                packed_file_decoded,
                packed_file_decoded_index,
                table => move |_,_| {

                    // We hide the context menu first.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        // Create the `FileChooser`.
                        let file_chooser = FileChooserNative::new(
                            "Select TSV File to Import...",
                            &app_ui.window,
                            FileChooserAction::Open,
                            "Import",
                            "Cancel"
                        );

                        // Enable the TSV filter for the `FileChooser`.
                        file_chooser_filter_packfile(&file_chooser, "*.tsv");

                        // If we have selected a file to import...
                        if file_chooser.run() == gtk_response_accept {

                            // Just in case the import fails after importing (for example, due to importing a TSV from another table,
                            // or from another version of the table, and it fails while loading to table or saving to PackFile)
                            // we save a copy of the table, so we can restore it if it fails after we modify it.
                            let packed_file_data_copy = packed_file_decoded.borrow_mut().data.clone();
                            let mut restore_table = (false, format_err!(""));

                            // If there is an error importing, we report it. This only edits the data after checking
                            // that it can be decoded properly, so we don't need to restore the table in this case.
                            if let Err(error) = packed_file_decoded.borrow_mut().data.import_tsv(
                                &file_chooser.get_filename().unwrap(),
                                &tree_path[1]
                            ) {
                                return show_dialog(&app_ui.window, false, error.cause());
                            }

                            // If there is an error loading the data (wrong table imported?), report it and restore it from the old copy.
                            if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view(&packed_file_decoded.borrow().data, &table.list_store) {
                                restore_table = (true, error);
                            }

                            // If the table loaded properly, try to save the data to the encoded file.
                            if !restore_table.0 {
                                if let Err(error) = update_packed_file_data_db(
                                    &*packed_file_decoded.borrow_mut(),
                                    &mut *pack_file.borrow_mut(),
                                    packed_file_decoded_index
                                ) {
                                    restore_table = (true, error);
                                }
                            }

                            // If the import broke somewhere along the way.
                            if restore_table.0 {

                                // Restore the old copy.
                                packed_file_decoded.borrow_mut().data = packed_file_data_copy;

                                // Report the error.
                                show_dialog(&app_ui.window, false, restore_table.1.cause());
                            }

                            // If there hasn't been any error.
                            else {

                                // Here we mark the PackFile as "Modified".
                                set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                            }
                        }
                    }
                }
            ));

            // When we hit the "Export to TSV" button.
            export_tsv.connect_activate(clone!(
                app_ui,
                packed_file_decoded,
                table => move |_,_| {

                    // We hide the context menu first.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        let file_chooser = FileChooserNative::new(
                            "Export TSV File...",
                            &app_ui.window,
                            FileChooserAction::Save,
                            "Save",
                            "Cancel"
                        );

                        // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                        file_chooser.set_do_overwrite_confirmation(true);

                        // Set the default name for the TSV file (table-table_name.tsv)
                        file_chooser.set_current_name(format!("{}-{}.tsv", &tree_path[1], &tree_path[2]));

                        // If we hit "Save"...
                        if file_chooser.run() == gtk_response_accept {

                            // Try to export the TSV.
                            match packed_file_decoded.borrow().data.export_tsv(
                                &file_chooser.get_filename().unwrap(),
                                (&tree_path[1], packed_file_decoded.borrow().header.version)
                            ) {
                                Ok(result) => show_dialog(&app_ui.window, true, result),
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                }
            ));
        }

        // Things that happen when you edit a cell. All of them in loops, because oops!... or because they are in vectors.
        {

            // This loop takes care of reference cells.
            for edited_cell in &table.list_cell_reference {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text| {

                        // If we got a cell...
                        if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                            // Get his column.
                            let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                            // Change his value in the `TreeView`.
                            table.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                            // Try to save the new data from the `TreeView`.
                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                // If we succeed...
                                Ok(data) => {

                                    // Replace our current decoded data with the new one.
                                    packed_file_decoded.borrow_mut().data.entries = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index
                                    ) {
                                        show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                }

                                // If there is an error, report it.
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with string cells.
            for edited_cell in &table.list_cell_string {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text| {

                        // If we got a cell...
                        if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                            // Get his column.
                            let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                            // Change his value in the `TreeView`.
                            table.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                            // Try to save the new data from the `TreeView`.
                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                // If we succeed...
                                Ok(data) => {

                                    // Replace our current decoded data with the new one.
                                    packed_file_decoded.borrow_mut().data.entries = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index
                                    ) {
                                        show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                }

                                // If there is an error, report it.
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with optional_string cells.
            for edited_cell in &table.list_cell_optional_string {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text|{

                        // If we got a cell...
                        if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                            // Get his column.
                            let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                            // Change his value in the `TreeView`.
                            table.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                            // Try to save the new data from the `TreeView`.
                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                // If we succeed...
                                Ok(data) => {

                                    // Replace our current decoded data with the new one.
                                    packed_file_decoded.borrow_mut().data.entries = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index
                                    ) {
                                        show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                }

                                // If there is an error, report it.
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with I32 cells.
            for edited_cell in &table.list_cell_integer {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text|{

                        // Check if what we got is a valid i32 number.
                        match new_text.parse::<i32>() {

                            // If it's a valid i32 number...
                            Ok(new_number) => {

                                // If we got a cell...
                                if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                                    // Get his column.
                                    let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                    // Change his value in the `TreeView`.
                                    table.list_store.set_value(&tree_iter, edited_cell_column, &new_number.to_value());

                                    // Try to save the new data from the `TreeView`.
                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                        // If we succeed...
                                        Ok(data) => {

                                            // Replace our current decoded data with the new one.
                                            packed_file_decoded.borrow_mut().data.entries = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index
                                            ) {
                                                show_dialog(&app_ui.window, false, error.cause());
                                            }

                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                        }

                                        // If there is an error, report it.
                                        Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                    }
                                }
                            }

                            // If it isn't a valid i32 number, report it.
                            Err(error) => show_dialog(&app_ui.window, false, Error::from(error).cause()),
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with I64 cells.
            for edited_cell in &table.list_cell_long_integer {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text|{

                        // Check if what we got is a valid i64 number.
                        match new_text.parse::<i64>() {

                            // If it's a valid i64 number...
                            Ok(new_number) => {

                                // If we got a cell...
                                if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                                    // Get his column.
                                    let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                    // Change his value in the `TreeView`.
                                    table.list_store.set_value(&tree_iter, edited_cell_column, &new_number.to_value());

                                    // Try to save the new data from the `TreeView`.
                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                        // If we succeed...
                                        Ok(data) => {

                                            // Replace our current decoded data with the new one.
                                            packed_file_decoded.borrow_mut().data.entries = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index
                                            ) {
                                                show_dialog(&app_ui.window, false, error.cause());
                                            }

                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                        }

                                        // If there is an error, report it.
                                        Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                    }
                                }
                            }

                            // If it isn't a valid i32 number, report it.
                            Err(error) => show_dialog(&app_ui.window, false, Error::from(error).cause()),
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with F32 cells.
            for edited_cell in &table.list_cell_float {
                edited_cell.connect_edited(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |_ ,tree_path , new_text|{

                        // Check if what we got is a valid f32 number.
                        match new_text.parse::<f32>() {

                            // If it's a valid f32 number...
                            Ok(new_number) => {

                                // If we got a cell...
                                if let Some(tree_iter) = table.list_store.get_iter(&tree_path) {

                                    // Get his column.
                                    let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                    // Change his value in the `TreeView`.
                                    table.list_store.set_value(&tree_iter, edited_cell_column, &format!("{}", new_number).to_value());

                                    // Try to save the new data from the `TreeView`.
                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                                        // If we succeed...
                                        Ok(data) => {

                                            // Replace our current decoded data with the new one.
                                            packed_file_decoded.borrow_mut().data.entries = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index
                                            ) {
                                                show_dialog(&app_ui.window, false, error.cause());
                                            }

                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                            set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                                        }

                                        // If there is an error, report it.
                                        Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                    }
                                }
                            }

                            // If it isn't a valid i32 number, report it.
                            Err(error) => show_dialog(&app_ui.window, false, Error::from(error).cause()),
                        }
                    }
                ));
            }

            // This loop takes care of the interaction with bool cells.
            for edited_cell in &table.list_cell_bool {
                edited_cell.connect_toggled(clone!(
                    table_definition,
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index,
                    table => move |cell, tree_path| {

                        // Get his `TreeIter` and his column.
                        let tree_iter = table.list_store.get_iter(&tree_path).unwrap();
                        let edited_cell_column = table.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                        // Get his new state.
                        let state = !cell.get_active();

                        // Change it in the `ListStore`.
                        table.list_store.set_value(&tree_iter, edited_cell_column, &state.to_value());

                        // Change his state.
                        cell.set_active(state);

                        // Try to save the new data from the `TreeView`.
                        match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &table.list_store) {

                            // If we succeed...
                            Ok(data) => {

                                // Replace our current decoded data with the new one.
                                packed_file_decoded.borrow_mut().data.entries = data;

                                // Try to save the changes to the PackFile. If there is an error, report it.
                                if let Err(error) = update_packed_file_data_db(
                                    &*packed_file_decoded.borrow_mut(),
                                    &mut *pack_file.borrow_mut(),
                                    packed_file_decoded_index
                                ) {
                                    show_dialog(&app_ui.window, false, error.cause());
                                }

                                // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());

                            }

                            // If there is an error, report it.
                            Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                        }
                    }
                ));
            }

        }

        Ok(())
    }

    /// This function decodes the data of a `DBData` and loads it into a `TreeView`.
    pub fn load_data_to_tree_view(
        packed_file_data: &DBData,
        packed_file_list_store: &ListStore,
    ) -> Result<(), Error>{

        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        packed_file_list_store.clear();

        // For each row in our decoded data...
        for row in &packed_file_data.entries {

            // We create a new row to add the data.
            let current_row = packed_file_list_store.append();

            // For each field in a row...
            for (index, field) in row.iter().enumerate() {

                // Check his type and push it as is. `Float` is an exception, as it has to be formated as `String` to remove the trailing zeroes.
                match *field {
                    DecodedData::Boolean(ref data) => packed_file_list_store.set_value(&current_row, index as u32, &data.to_value()),
                    DecodedData::Float(ref data) => packed_file_list_store.set_value(&current_row, index as u32, &format!("{}", data).to_value()),
                    DecodedData::Integer(ref data) => packed_file_list_store.set_value(&current_row, index as u32, &data.to_value()),
                    DecodedData::LongInteger(ref data) => packed_file_list_store.set_value(&current_row, index as u32, &data.to_value()),

                    // All these are Strings, so it can be together,
                    DecodedData::Index(ref data) |
                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => packed_file_list_store.set_value(&current_row, index as u32, &data.to_value()),
                }
            }
        }
        Ok(())
    }

    /// This function returns a `Vec<Vec<DataDecoded>>` with all the stuff in the table. We need for it the `ListStore` of that table.
    pub fn return_data_from_tree_view(
        table_definition: &TableDefinition,
        list_store: &ListStore,
    ) -> Result<Vec<Vec<DecodedData>>, Error> {

        // Create an empty `Vec<Vec<DecodedData>>`.
        let mut decoded_data_table: Vec<Vec<DecodedData>> = vec![];

        // If we got at least one row...
        if let Some(current_line) = list_store.get_iter_first() {

            // Foreach row in the DB PackedFile.
            loop {

                // We return the index too. We deal with it in the save function, so there is no problem with that.
                let mut row: Vec<DecodedData> = vec![DecodedData::Index(list_store.get_value(&current_line, 0).get().unwrap())];

                // For each column....
                for column in 1..list_store.get_n_columns() {

                    // Check his type and act accordingly.
                    match table_definition.fields[column as usize - 1].field_type {
                        FieldType::Boolean => row.push(DecodedData::Boolean(list_store.get_value(&current_line, column).get().unwrap())),

                        // Float are special. To get rid of the trailing zeroes, we must put it into the `ListStore` as `String`, so here we have to parse it to `f32`.
                        FieldType::Float => row.push(DecodedData::Float(list_store.get_value(&current_line, column).get::<String>().unwrap().parse::<f32>().unwrap())),
                        FieldType::Integer => row.push(DecodedData::Integer(list_store.get_value(&current_line, column).get().unwrap())),
                        FieldType::LongInteger => row.push(DecodedData::LongInteger(list_store.get_value(&current_line, column).get().unwrap())),
                        FieldType::StringU8 => row.push(DecodedData::StringU8(list_store.get_value(&current_line, column).get().unwrap())),
                        FieldType::StringU16 => row.push(DecodedData::StringU16(list_store.get_value(&current_line, column).get().unwrap())),
                        FieldType::OptionalStringU8 => row.push(DecodedData::OptionalStringU8(list_store.get_value(&current_line, column).get().unwrap())),
                        FieldType::OptionalStringU16 => row.push(DecodedData::OptionalStringU16(list_store.get_value(&current_line, column).get().unwrap())),
                    }
                }

                // Add the row to the list.
                decoded_data_table.push(row);

                // If there are no more rows, stop.
                if !list_store.iter_next(&current_line) { break; }
            }
        }

        // Return the decoded data.
        Ok(decoded_data_table)
    }
}

/// Implementation of `PackedFileDBDecoder`.
impl PackedFileDBDecoder {

    /// This function creates the "Decoder View" with all the stuff needed to decode a table, and it
    /// returns it.
    pub fn create_decoder_view(
        application: &Application,
        app_ui: &AppUI,
        rpfm_path: &PathBuf,
        supported_games: &Rc<RefCell<Vec<GameInfo>>>,
        game_selected: &Rc<RefCell<GameSelected>>,
        table_name: String,
        packed_file_data: Vec<u8>,
        schema: &Rc<RefCell<Option<Schema>>>,
    ) -> Result<(), Error> {

        // Create the index for the decoding.
        let mut data_initial_index = 0;

        // Decode the header to ensure this is a valid table to decode.
        let decoded_header = DBHeader::read(&packed_file_data, &mut data_initial_index)?;

        // With this we create the "Decoder View", under the "Enable decoding mode" button.
        let decoder_grid_scroll = ScrolledWindow::new(None, None);
        let decoder_grid = Grid::new();
        decoder_grid.set_border_width(6);
        decoder_grid.set_row_spacing(6);
        decoder_grid.set_column_spacing(3);

        // In the left side, there should be a Grid with the hex data.
        let raw_data_grid = Grid::new();
        let raw_data_index = TextView::new();
        let raw_data_hex = TextView::new();
        let raw_data_decoded = TextView::new();

        // Config for the "Raw Data" stuff.
        raw_data_grid.set_border_width(6);
        raw_data_grid.set_row_spacing(6);
        raw_data_grid.set_column_spacing(3);

        raw_data_index.set_vexpand(true);

        // These three shouldn't be editables.
        raw_data_index.set_editable(false);
        raw_data_hex.set_editable(false);
        raw_data_decoded.set_editable(false);

        // Set the fonts of the labels to `monospace`, so we see them properly aligned.
        let raw_data_index_style = raw_data_index.get_style_context().unwrap();
        let raw_data_hex_style = raw_data_hex.get_style_context().unwrap();
        let raw_data_decoded_style = raw_data_decoded.get_style_context().unwrap();
        let raw_data_monospace_css = b".monospace-font { font-family: \"Courier New\", Courier, monospace }";

        let css_provider = CssProvider::new();

        css_provider.load_from_data(raw_data_monospace_css).unwrap();

        raw_data_index_style.add_provider(&css_provider, 99);
        raw_data_hex_style.add_provider(&css_provider, 99);
        raw_data_decoded_style.add_provider(&css_provider, 99);

        StyleContext::add_class(&raw_data_index_style, "monospace-font");
        StyleContext::add_class(&raw_data_hex_style, "monospace-font");
        StyleContext::add_class(&raw_data_decoded_style, "monospace-font");

        // Create the color tags for the Raw Data...
        create_text_tags(&raw_data_hex);
        create_text_tags(&raw_data_decoded);

        // In the right side, there should be a Vertical Paned, with a grid on the top, and another
        // on the bottom.
        let decoded_data_paned = Paned::new(Orientation::Vertical);
        let decoded_data_paned_bottom_grid = Grid::new();

        decoded_data_paned.set_position(500);
        decoded_data_paned_bottom_grid.set_border_width(6);
        decoded_data_paned_bottom_grid.set_row_spacing(6);
        decoded_data_paned_bottom_grid.set_column_spacing(3);

        // And here, the ScrolledWindow and the TreeView.
        let fields_tree_view_scroll = ScrolledWindow::new(None, None);
        let fields_tree_view = TreeView::new();
        let fields_list_store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), bool::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type()]);
        fields_tree_view.set_model(Some(&fields_list_store));
        fields_tree_view.set_margin_bottom(10);
        fields_tree_view.set_hexpand(true);
        fields_tree_view.set_vexpand(true);

        // This method of reordering crash the program on windows, so we only enable it for Linux.
        // NOTE: this doesn't trigger `update_first_row_decoded`.
        if cfg!(target_os = "linux") {

            // Here we set the TreeView as "drag_dest" and "drag_source", so we can drag&drop things to it.
            let targets = vec![gtk::TargetEntry::new("text/uri-list", gtk::TargetFlags::SAME_WIDGET, 0)];
            fields_tree_view.drag_source_set(gdk::ModifierType::BUTTON1_MASK, &targets, gdk::DragAction::MOVE);
            fields_tree_view.drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::MOVE);
            fields_tree_view.set_reorderable(true);
        }

        // These are the vectors to store the cells. We'll use them later on to get the events on
        // the cells, so we can edit them properly.
        let mut fields_tree_view_cell_string = vec![];

        let column_index = TreeViewColumn::new();
        let cell_index = CellRendererText::new();
        column_index.pack_start(&cell_index, true);
        column_index.add_attribute(&cell_index, "text", 0);
        column_index.set_sort_column_id(0);
        column_index.set_clickable(false);
        column_index.set_title("Index");

        let column_name = TreeViewColumn::new();
        let cell_name = CellRendererText::new();
        cell_name.set_property_editable(true);
        column_name.pack_start(&cell_name, true);
        column_name.add_attribute(&cell_name, "text", 1);
        column_name.set_sort_column_id(1);
        column_name.set_clickable(false);
        column_name.set_title("Field name");
        fields_tree_view_cell_string.push(cell_name);

        let cell_type_list_store = ListStore::new(&[String::static_type()]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"Bool"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"Float"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"Integer"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"LongInteger"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"StringU8"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"StringU16"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"OptionalStringU8"]);
        cell_type_list_store.insert_with_values(None, &[0], &[&"OptionalStringU16"]);

        let column_type = TreeViewColumn::new();
        let cell_type = CellRendererCombo::new();
        cell_type.set_property_editable(true);
        cell_type.set_property_model(Some(&cell_type_list_store));
        cell_type.set_property_text_column(0);
        column_type.pack_start(&cell_type, true);
        column_type.add_attribute(&cell_type, "text", 2);
        column_type.set_sort_column_id(2);
        column_type.set_clickable(false);
        column_type.set_title("Field Type");
        let fields_tree_view_cell_combo = cell_type;
        let fields_tree_view_cell_combo_list_store = cell_type_list_store;

        let column_key = TreeViewColumn::new();
        let cell_key = CellRendererToggle::new();
        cell_key.set_activatable(true);
        column_key.pack_start(&cell_key, true);
        column_key.add_attribute(&cell_key, "active", 3);
        column_key.set_sort_column_id(3);
        column_key.set_clickable(false);
        column_key.set_title("Is key?");
        let fields_tree_view_cell_bool = cell_key;

        let column_ref_table = TreeViewColumn::new();
        let cell_ref_table = CellRendererText::new();
        cell_ref_table.set_property_editable(true);
        column_ref_table.pack_start(&cell_ref_table, true);
        column_ref_table.add_attribute(&cell_ref_table, "text", 4);
        column_ref_table.set_sort_column_id(4);
        column_ref_table.set_clickable(false);
        column_ref_table.set_title("Ref. to table");
        fields_tree_view_cell_string.push(cell_ref_table);

        let column_ref_column = TreeViewColumn::new();
        let cell_ref_column = CellRendererText::new();
        cell_ref_column.set_property_editable(true);
        column_ref_column.pack_start(&cell_ref_column, true);
        column_ref_column.add_attribute(&cell_ref_column, "text", 5);
        column_ref_column.set_sort_column_id(5);
        column_ref_column.set_clickable(false);
        column_ref_column.set_title("Ref. to column");
        fields_tree_view_cell_string.push(cell_ref_column);

        let column_decoded = TreeViewColumn::new();
        let cell_decoded = CellRendererText::new();
        cell_decoded.set_property_editable(false);
        column_decoded.pack_start(&cell_decoded, true);
        column_decoded.add_attribute(&cell_decoded, "text", 6);
        column_decoded.set_sort_column_id(6);
        column_decoded.set_clickable(false);
        column_decoded.set_title("First row decoded");

        let column_description = TreeViewColumn::new();
        let cell_description = CellRendererText::new();
        cell_description.set_property_editable(true);
        column_description.pack_start(&cell_description, true);
        column_description.add_attribute(&cell_description, "text", 7);
        column_description.set_sort_column_id(7);
        column_description.set_clickable(false);
        column_description.set_title("Description");
        fields_tree_view_cell_string.push(cell_description);

        fields_tree_view.append_column(&column_index);
        fields_tree_view.append_column(&column_name);
        fields_tree_view.append_column(&column_type);
        fields_tree_view.append_column(&column_key);
        fields_tree_view.append_column(&column_ref_table);
        fields_tree_view.append_column(&column_ref_column);
        fields_tree_view.append_column(&column_decoded);
        fields_tree_view.append_column(&column_description);

        // Here we create the context menu for the `fields_tree_view`.
        let context_menu = Popover::new_from_model(Some(&fields_tree_view), &app_ui.db_decoder_context_menu_model);

        // Clean the accelerators stuff.
        remove_temporal_accelerators(&application);

        // Move and delete row actions.
        let move_row_up = SimpleAction::new("move-row-up", None);
        let move_row_down = SimpleAction::new("move-row-down", None);
        let delete_row = SimpleAction::new("delete-row", None);

        application.add_action(&move_row_up);
        application.add_action(&move_row_down);
        application.add_action(&delete_row);

        // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
        application.set_accels_for_action("app.move-row-up", &["<Shift>Up"]);
        application.set_accels_for_action("app.move-row-down", &["<Shift>Down"]);
        application.set_accels_for_action("app.delete-row", &["<Shift>Delete"]);

        // By default, these three should be disabled.
        move_row_up.set_enabled(false);
        move_row_down.set_enabled(false);
        delete_row.set_enabled(false);

        // From here, we config the bottom grid of the paned.
        let decoded_types_grid = Grid::new();

        decoded_types_grid.set_border_width(6);
        decoded_types_grid.set_row_spacing(6);
        decoded_types_grid.set_column_spacing(3);

        // Here we create the TextViews for the different decoding types.
        let bool_label = Label::new(Some("Decoded as \"Bool\":"));
        let float_label = Label::new(Some("Decoded as \"Float\":"));
        let integer_label = Label::new(Some("Decoded as \"Integer\":"));
        let long_integer_label = Label::new(Some("Decoded as \"Long Integer\":"));
        let string_u8_label = Label::new(Some("Decoded as \"String u8\":"));
        let string_u16_label = Label::new(Some("Decoded as \"String u16\":"));
        let optional_string_u8_label = Label::new(Some("Decoded as \"Optional String u8\":"));
        let optional_string_u16_label = Label::new(Some("Decoded as \"Optional String u16\":"));

        bool_label.set_size_request(200, 0);
        float_label.set_size_request(200, 0);
        integer_label.set_size_request(200, 0);
        long_integer_label.set_size_request(200, 0);
        string_u8_label.set_size_request(200, 0);
        string_u16_label.set_size_request(200, 0);
        optional_string_u8_label.set_size_request(200, 0);
        optional_string_u16_label.set_size_request(200, 0);

        bool_label.set_xalign(0.0);
        bool_label.set_yalign(0.5);
        float_label.set_xalign(0.0);
        float_label.set_yalign(0.5);
        integer_label.set_xalign(0.0);
        integer_label.set_yalign(0.5);
        long_integer_label.set_xalign(0.0);
        long_integer_label.set_yalign(0.5);
        string_u8_label.set_xalign(0.0);
        string_u8_label.set_yalign(0.5);
        string_u16_label.set_xalign(0.0);
        string_u16_label.set_yalign(0.5);
        optional_string_u8_label.set_xalign(0.0);
        optional_string_u8_label.set_yalign(0.5);
        optional_string_u16_label.set_xalign(0.0);
        optional_string_u16_label.set_yalign(0.5);

        let bool_entry = Entry::new();
        let float_entry = Entry::new();
        let integer_entry = Entry::new();
        let long_integer_entry = Entry::new();
        let string_u8_entry = Entry::new();
        let string_u16_entry = Entry::new();
        let optional_string_u8_entry = Entry::new();
        let optional_string_u16_entry = Entry::new();

        bool_entry.set_editable(false);
        bool_entry.set_size_request(300, 0);
        bool_entry.set_hexpand(true);

        float_entry.set_editable(false);
        float_entry.set_size_request(300, 0);
        float_entry.set_hexpand(true);

        integer_entry.set_editable(false);
        integer_entry.set_size_request(300, 0);
        integer_entry.set_hexpand(true);

        long_integer_entry.set_editable(false);
        long_integer_entry.set_size_request(300, 0);
        long_integer_entry.set_hexpand(true);

        string_u8_entry.set_editable(false);
        string_u8_entry.set_size_request(300, 0);
        string_u8_entry.set_hexpand(true);

        string_u16_entry.set_editable(false);
        string_u16_entry.set_size_request(300, 0);
        string_u16_entry.set_hexpand(true);

        optional_string_u8_entry.set_editable(false);
        optional_string_u8_entry.set_size_request(300, 0);
        optional_string_u8_entry.set_hexpand(true);

        optional_string_u16_entry.set_editable(false);
        optional_string_u16_entry.set_size_request(300, 0);
        optional_string_u16_entry.set_hexpand(true);

        let use_bool_button = Button::new_with_label("Use this");
        let use_float_button = Button::new_with_label("Use this");
        let use_integer_button = Button::new_with_label("Use this");
        let use_long_integer_button = Button::new_with_label("Use this");
        let use_string_u8_button = Button::new_with_label("Use this");
        let use_string_u16_button = Button::new_with_label("Use this");
        let use_optional_string_u8_button = Button::new_with_label("Use this");
        let use_optional_string_u16_button = Button::new_with_label("Use this");

        // From here, there is the stuff of the end column of the bottom paned.
        let general_info_grid = Grid::new();

        general_info_grid.set_border_width(6);
        general_info_grid.set_row_spacing(6);
        general_info_grid.set_column_spacing(3);

        // For the frame, we need an internal grid, as a frame it seems only can hold one child.
        let packed_file_table_info_frame = Frame::new(Some("Table info"));
        let packed_file_field_info_grid = Grid::new();

        packed_file_field_info_grid.set_border_width(6);
        packed_file_field_info_grid.set_row_spacing(6);
        packed_file_field_info_grid.set_column_spacing(3);

        // This is a dirty trick, but it werks. We create all the labels here so they end up in the struct,
        // but only attach to the Grid the ones we want.
        let table_type_decoded_label = Label::new(None);
        let table_version_decoded_label = Label::new(None);
        let table_entry_count_decoded_label = Label::new(None);

        // Load all the info of the table.
        let table_info_type_label = Label::new("Table type:");
        let table_info_version_label = Label::new("Table version:");
        let table_info_entry_count_label = Label::new("Table entry count:");

        table_info_type_label.set_xalign(0.0);
        table_info_type_label.set_yalign(0.5);
        table_info_version_label.set_xalign(0.0);
        table_info_version_label.set_yalign(0.5);
        table_info_entry_count_label.set_xalign(0.0);
        table_info_entry_count_label.set_yalign(0.5);

        table_type_decoded_label.set_xalign(0.0);
        table_type_decoded_label.set_yalign(0.5);
        table_version_decoded_label.set_xalign(0.0);
        table_version_decoded_label.set_yalign(0.5);
        table_entry_count_decoded_label.set_xalign(0.0);
        table_entry_count_decoded_label.set_yalign(0.5);

        // Form the interior of the frame here.
        packed_file_field_info_grid.attach(&table_info_type_label, 0, 0, 1, 1);
        packed_file_field_info_grid.attach(&table_info_version_label, 0, 1, 1, 1);
        packed_file_field_info_grid.attach(&table_info_entry_count_label, 0, 2, 1, 1);

        packed_file_field_info_grid.attach(&table_type_decoded_label, 1, 0, 1, 1);
        packed_file_field_info_grid.attach(&table_version_decoded_label, 1, 1, 1, 1);
        packed_file_field_info_grid.attach(&table_entry_count_decoded_label, 1, 2, 1, 1);

        packed_file_table_info_frame.add(&packed_file_field_info_grid);

        // Here are all the extra settings of the decoded table.
        let field_name_label = Label::new("Field Name:");
        let field_name_entry = Entry::new();
        field_name_label.set_xalign(0.0);
        field_name_label.set_yalign(0.5);

        let is_key_field_label = Label::new("Key field");
        let is_key_field_switch = Switch::new();
        is_key_field_label.set_xalign(0.0);
        is_key_field_label.set_yalign(0.5);

        // Same trick as before.
        let all_table_versions_tree_view = TreeView::new();
        let all_table_versions_list_store = ListStore::new(&[u32::static_type()]);
        let load_definition = Button::new_with_label("Load");
        let remove_definition = Button::new_with_label("Remove");

        // Disable these buttons by default.
        load_definition.set_sensitive(false);
        remove_definition.set_sensitive(false);

        // Here we create a little TreeView with all the versions of this table we have, in case we
        // want to decode it based on another version's definition, to save time.
        all_table_versions_tree_view.set_model(Some(&all_table_versions_list_store));

        let all_table_versions_tree_view_scroll = ScrolledWindow::new(None, None);
        all_table_versions_tree_view_scroll.set_hexpand(true);
        all_table_versions_tree_view_scroll.set_size_request(0, 110);
        all_table_versions_tree_view_scroll.add(&all_table_versions_tree_view);

        let column_versions = TreeViewColumn::new();
        let cell_version = CellRendererText::new();
        column_versions.pack_start(&cell_version, true);
        column_versions.add_attribute(&cell_version, "text", 0);
        column_versions.set_sort_column_id(0);
        column_versions.set_clickable(false);
        column_versions.set_title("Versions");

        all_table_versions_tree_view.append_column(&column_versions);

        // Buttons to load and delete the selected version from the schema.
        let button_box_definition = ButtonBox::new(Orientation::Horizontal);

        button_box_definition.set_layout(ButtonBoxStyle::End);
        button_box_definition.set_spacing(6);

        button_box_definition.pack_start(&load_definition, false, false, 0);
        button_box_definition.pack_start(&remove_definition, false, false, 0);

        general_info_grid.attach(&all_table_versions_tree_view_scroll, 0, 3, 2, 1);
        general_info_grid.attach(&button_box_definition, 0, 4, 2, 1);

        // These are the bottom buttons.
        let bottom_box = ButtonBox::new(Orientation::Horizontal);

        bottom_box.set_layout(ButtonBoxStyle::End);
        bottom_box.set_spacing(6);

        let delete_all_fields_button = Button::new_with_label("Remove all fields");
        let save_decoded_schema = Button::new_with_label("Finish It!");

        bottom_box.pack_start(&delete_all_fields_button, false, false, 0);
        bottom_box.pack_start(&save_decoded_schema, false, false, 0);

        // From here, there is just packing stuff....

        // Packing into the left ScrolledWindow...
        raw_data_grid.attach(&raw_data_index, 0, 0, 1, 1);
        raw_data_grid.attach(&raw_data_hex, 1, 0, 1, 1);
        raw_data_grid.attach(&raw_data_decoded, 2, 0, 1, 1);

        decoder_grid.attach(&raw_data_grid, 0, 0, 1, 1);

        // Packing into the rigth paned....
        decoded_data_paned.pack1(&fields_tree_view_scroll, false, false);
        decoded_data_paned.pack2(&decoded_data_paned_bottom_grid, false, false);

        decoder_grid.attach(&decoded_data_paned, 1, 0, 1, 1);

        fields_tree_view_scroll.add(&fields_tree_view);

        // Packing into the bottom side of the right paned...

        // First column of the bottom grid...
        decoded_types_grid.attach(&bool_label, 0, 0, 1, 1);
        decoded_types_grid.attach(&bool_entry, 1, 0, 1, 1);
        decoded_types_grid.attach(&use_bool_button, 2, 0, 1, 1);

        decoded_types_grid.attach(&float_label, 0, 1, 1, 1);
        decoded_types_grid.attach(&float_entry, 1, 1, 1, 1);
        decoded_types_grid.attach(&use_float_button, 2, 1, 1, 1);

        decoded_types_grid.attach(&integer_label, 0, 2, 1, 1);
        decoded_types_grid.attach(&integer_entry, 1, 2, 1, 1);
        decoded_types_grid.attach(&use_integer_button, 2, 2, 1, 1);

        decoded_types_grid.attach(&long_integer_label, 0, 3, 1, 1);
        decoded_types_grid.attach(&long_integer_entry, 1, 3, 1, 1);
        decoded_types_grid.attach(&use_long_integer_button, 2, 3, 1, 1);

        decoded_types_grid.attach(&string_u8_label, 0, 4, 1, 1);
        decoded_types_grid.attach(&string_u8_entry, 1, 4, 1, 1);
        decoded_types_grid.attach(&use_string_u8_button, 2, 4, 1, 1);

        decoded_types_grid.attach(&string_u16_label, 0, 5, 1, 1);
        decoded_types_grid.attach(&string_u16_entry, 1, 5, 1, 1);
        decoded_types_grid.attach(&use_string_u16_button, 2, 5, 1, 1);

        decoded_types_grid.attach(&optional_string_u8_label, 0, 6, 1, 1);
        decoded_types_grid.attach(&optional_string_u8_entry, 1, 6, 1, 1);
        decoded_types_grid.attach(&use_optional_string_u8_button, 2, 6, 1, 1);

        decoded_types_grid.attach(&optional_string_u16_label, 0, 7, 1, 1);
        decoded_types_grid.attach(&optional_string_u16_entry, 1, 7, 1, 1);
        decoded_types_grid.attach(&use_optional_string_u16_button, 2, 7, 1, 1);

        decoded_data_paned_bottom_grid.attach(&decoded_types_grid, 0, 0, 1, 1);

        // Second column of the bottom grid...
        general_info_grid.attach(&packed_file_table_info_frame, 0, 0, 2, 1);

        general_info_grid.attach(&field_name_label, 0, 1, 1, 1);
        general_info_grid.attach(&field_name_entry, 1, 1, 1, 1);

        general_info_grid.attach(&is_key_field_label, 0, 2, 1, 1);
        general_info_grid.attach(&is_key_field_switch, 1, 2, 1, 1);

        decoded_data_paned_bottom_grid.attach(&general_info_grid, 1, 0, 1, 10);

        // Bottom of the bottom grid...
        decoded_data_paned_bottom_grid.attach(&bottom_box, 0, 1, 2, 1);

        // Packing into the decoder grid...
        decoder_grid_scroll.add(&decoder_grid);
        app_ui.packed_file_data_display.attach(&decoder_grid_scroll, 0, 1, 1, 1);
        app_ui.packed_file_data_display.show_all();

        // Disable the "Change game selected" function, so we cannot change the current schema with the decoder open.
        app_ui.menu_bar_change_game_selected.set_enabled(false);

        // Create the view.
        let mut decoder_view = PackedFileDBDecoder {
            data_initial_index,
            decoded_header,
            raw_data_line_index: raw_data_index,
            raw_data: raw_data_hex,
            raw_data_decoded,
            table_type_label: table_type_decoded_label,
            table_version_label: table_version_decoded_label,
            table_entry_count_label: table_entry_count_decoded_label,
            bool_entry,
            float_entry,
            integer_entry,
            long_integer_entry,
            string_u8_entry,
            string_u16_entry,
            optional_string_u8_entry,
            optional_string_u16_entry,
            use_bool_button,
            use_float_button,
            use_integer_button,
            use_long_integer_button,
            use_string_u8_button,
            use_string_u16_button,
            use_optional_string_u8_button,
            use_optional_string_u16_button,
            fields_tree_view,
            fields_list_store,
            all_table_versions_tree_view,
            all_table_versions_list_store,
            all_table_versions_load_definition: load_definition,
            all_table_versions_remove_definition: remove_definition,
            field_name_entry,
            is_key_field_switch,
            save_decoded_schema,
            fields_tree_view_cell_bool,
            fields_tree_view_cell_combo,
            fields_tree_view_cell_combo_list_store,
            fields_tree_view_cell_string,
            delete_all_fields_button,
            decoder_grid_scroll,
            context_menu,
        };

        // We try to load the static data from the encoded PackedFile into the "Decoder" view.
        if let Err(error) = decoder_view.load_data_to_decoder_view(&table_name, &packed_file_data) {
            return Err(error)
        };

        // Get the index we are going to use to keep track of where the hell are we in the table.
        let index_data = Rc::new(RefCell::new(decoder_view.data_initial_index));

        // We get the Schema for his game, if exists. If we reached this point, the Schema
        // should exists. Otherwise, the button for this window will not exist.
        let table_definition = match DB::get_schema(&table_name, decoder_view.decoded_header.version, &schema.borrow().clone().unwrap()) {
            Some(table_definition) => Rc::new(RefCell::new(table_definition)),
            None => Rc::new(RefCell::new(TableDefinition::new(decoder_view.decoded_header.version)))
        };

        // Update the "Decoder" View dynamic data (entries, treeview,...) and get the
        // current "index_data" (position in the vector we are decoding).
        decoder_view.update_decoder_view(
            &packed_file_data,
            (true, &table_definition.borrow().fields),
            &mut index_data.borrow_mut()
        );

        // Update the versions list. Only if we have an schema, we can reach this point, so we just unwrap the schema.
        decoder_view.update_versions_list(&schema.borrow().clone().unwrap(), &table_name);

        // Events and stuff related to this view...

        // Version list stuff.
        {

            // This allow us to replace the definition we have loaded with one from another version of the table.
            decoder_view.all_table_versions_load_definition.connect_button_release_event(clone!(
                schema,
                app_ui,
                index_data,
                table_name,
                packed_file_data,
                decoder_view => move |_ ,_| {

                    // Only if we have a version selected, do something.
                    if let Some(version_selected) = decoder_view.all_table_versions_tree_view.get_selection().get_selected() {

                        // Get the version selected.
                        let version_to_load: u32 = decoder_view.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                        // Check if the Schema actually exists. This should never show up if the schema exists,
                        // but the compiler doesn't know it, so we have to check it.
                        match *schema.borrow_mut() {
                            Some(ref mut schema) => {

                                // Get the new definition.
                                let table_definition = DB::get_schema(&table_name, version_to_load, schema);

                                // Remove all the fields of the currently loaded definition.
                                decoder_view.fields_list_store.clear();

                                // Reset the "index_data".
                                *index_data.borrow_mut() = decoder_view.data_initial_index;

                                // Reload the decoder View with the new definition loaded.
                                decoder_view.update_decoder_view(
                                    &packed_file_data,
                                    (true, &table_definition.unwrap().fields),
                                    &mut index_data.borrow_mut()
                                );
                            }
                            None => show_dialog(&app_ui.window, false, "Cannot load a version of a table from a non-existant Schema.")
                        }
                    }

                    Inhibit(false)
                }
            ));

            // This allow us to remove an entire definition of a table for an specific version.
            // Basically, hitting this button deletes the selected definition.
            decoder_view.all_table_versions_remove_definition.connect_button_release_event(clone!(
                schema,
                app_ui,
                table_name,
                decoder_view => move |_ ,_| {

                    // Only if we have a version selected, do something.
                    if let Some(version_selected) = decoder_view.all_table_versions_tree_view.get_selection().get_selected() {

                        // Get the version selected.
                        let version_to_delete: u32 = decoder_view.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                        // Check if the Schema actually exists. This should never show up if the schema exists,
                        // but the compiler doesn't know it, so we have to check it.
                        match *schema.borrow_mut() {
                            Some(ref mut schema) => {

                                // Try to remove that version form the schema.
                                match DB::remove_table_version(&table_name, version_to_delete, schema) {

                                    // If it worked, update the list.
                                    Ok(_) => decoder_view.update_versions_list(schema, &table_name),
                                    Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                                }
                            }
                            None => show_dialog(&app_ui.window, false, "Cannot delete a version from a non-existant Schema.")
                        }
                    }

                    Inhibit(false)
                }
            ));
        }

        // Bottom box buttons.
        {


            // When we press the "Finish it!" button.
            decoder_view.save_decoded_schema.connect_button_release_event(clone!(
                app_ui,
                schema,
                table_definition,
                table_name,
                rpfm_path,
                supported_games,
                game_selected,
                decoder_view => move |_ ,_| {

                    // Check if the Schema actually exists. This should never show up if the schema exists,
                    // but the compiler doesn't know it, so we have to check it.
                    match *schema.borrow_mut() {
                        Some(ref mut schema) => {

                            // We get the index of our table's definitions. In case we find it, we just return it. If it's not
                            // the case, then we create a new table's definitions and return his index. To know if we didn't found
                            // an index, we just return -1 as index.
                            let mut table_definitions_index = match schema.get_table_definitions(&table_name) {
                                Some(table_definitions_index) => table_definitions_index as i32,
                                None => -1i32,
                            };

                            // If we didn't found a table definition for our table...
                            if table_definitions_index == -1 {

                                // We create one.
                                schema.add_table_definitions(TableDefinitions::new(&decoder_view.table_type_label.get_text().unwrap()));

                                // And get his index.
                                table_definitions_index = schema.get_table_definitions(&table_name).unwrap() as i32;
                            }

                            // We replace his fields with the ones from the `TreeView`.
                            table_definition.borrow_mut().fields = decoder_view.return_data_from_data_view();

                            // We add our `TableDefinition` to the main `Schema`.
                            schema.tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());

                            // And try to save the main `Schema`.
                            match Schema::save(schema, &rpfm_path, &supported_games.borrow().iter().filter(|x| x.folder_name == *game_selected.borrow().game).map(|x| x.schema.to_owned()).collect::<String>()) {
                                Ok(_) => show_dialog(&app_ui.window, true, "Schema successfully saved."),
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }

                            // After all that, we need to update the version list, as this may have created a new version.
                            decoder_view.update_versions_list(schema, &table_name);
                        }
                        None => show_dialog(&app_ui.window, false, "Cannot save this table's definitions:\nSchemas for this game are not supported, yet.")
                    }

                    Inhibit(false)
                }
            ));
        }



        // Return the view.
        Ok(())
    }

    /// This function loads the data from the selected table into the "Decoder View".
    pub fn load_data_to_decoder_view(
        &mut self,
        table_name: &str,
        packed_file_encoded: &[u8],
    ) -> Result<(), Error> {

        // We don't need the entire PackedFile, just his begining. Otherwise, this function takes
        // ages to finish.
        let packed_file_encoded = if packed_file_encoded.len() > 16 * 60 { &packed_file_encoded[..16 * 60] }
        else { packed_file_encoded };

        // This creates the "index" column at the left of the hex data. The logic behind this, because
        // even I have problems to understand it: lines are 4 packs of 4 bytes => 16 bytes. Amount of
        // lines is "bytes we have / 16 + 1" (+ 1 because we want to show incomplete lines too).
        // Then, the zeroes amount is the amount of chars the `hex_lines_amount` have after making it
        // a string (i.e. 2DC will be 3) + 2 (+ 1 because we divided before between it's base `16`, and
        // + 1 because we want a 0 before every entry).
        let hex_lines_amount = (packed_file_encoded.len() / 16) + 1;
        let zeroes_amount = format!("{:X}", hex_lines_amount).len() + 2;

        let mut hex_lines_text = String::new();
        for hex_line in 0..hex_lines_amount {
            hex_lines_text.push_str(&format!("{:>0count$X}\n", hex_line * 16, count = zeroes_amount));
        }
        self.raw_data_line_index.get_buffer().unwrap().set_text(&hex_lines_text);

        // This gets the hex data into place. In big files, this takes ages, so we cut them if they
        // are longer than 100 lines to speed up loading and fix a weird issue with big tables.
        let mut hex_raw_data = format!("{:02X?}", packed_file_encoded);

        // Remove the first and last chars.
        hex_raw_data.remove(0);
        hex_raw_data.pop();

        // Remove all the kebab, or the commas. Whatever float your boat...
        hex_raw_data.retain(|c| c != ',');

        // `raw_data_hex` TextView.
        {
            let mut hex_raw_data_string = String::new();

            // This pushes a newline after 48 characters (2 for every byte + 1 whitespace).
            for (j, i) in hex_raw_data.chars().enumerate() {
                if j % 48 == 0 && j != 0 {
                    hex_raw_data_string.push_str("\n");
                }
                hex_raw_data_string.push(i);
            }

            let raw_data_hex_buffer = self.raw_data.get_buffer().unwrap();
            raw_data_hex_buffer.set_text(&hex_raw_data_string);

            // In theory, this should give us the equivalent byte to our index_data.
            // In practice, I'm bad at maths.
            let header_line = (self.data_initial_index * 3 / 48) as i32;
            let header_char = (self.data_initial_index * 3 % 48) as i32;
            raw_data_hex_buffer.apply_tag_by_name(
                "header",
                &raw_data_hex_buffer.get_start_iter(),
                &raw_data_hex_buffer.get_iter_at_line_offset(header_line, header_char)
            );
        }

        // `raw_data_decoded` TextView.
        {
            let mut hex_raw_data_decoded = String::new();

            // This pushes a newline after 16 characters.
            for (j, i) in packed_file_encoded.iter().enumerate() {
                if j % 16 == 0 && j != 0 {
                    hex_raw_data_decoded.push_str("\n");
                }
                let character = *i as char;
                if character.is_alphanumeric() {
                    hex_raw_data_decoded.push(character);
                }
                else {
                    hex_raw_data_decoded.push('.');
                }
            }

            let header_line = (self.data_initial_index / 16) as i32;
            let header_char = (self.data_initial_index % 16) as i32;

            let raw_data_decoded_buffer = self.raw_data_decoded.get_buffer().unwrap();
            raw_data_decoded_buffer.set_text(&hex_raw_data_decoded);
            raw_data_decoded_buffer.apply_tag_by_name(
                "header",
                &raw_data_decoded_buffer.get_start_iter(),
                &raw_data_decoded_buffer.get_iter_at_line_offset(header_line, header_char)
            );
        }

        // Set the data of the table.
        self.table_type_label.set_text(table_name);
        self.table_version_label.set_text(&format!("{}", self.decoded_header.version));
        self.table_entry_count_label.set_text(&format!("{}", self.decoded_header.entry_count));

        Ok(())
    }

    /// This function updates the data shown in the "Decoder" box when we execute it. `index_data`
    /// is the position from where to start decoding.
    pub fn update_decoder_view(
        &self,
        packed_file_decoded: &[u8],
        field_list: (bool, &[Field]),
        mut index_data: &mut usize
    ) {

        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // If we are loading data to the table for the first time, we'll load to the table all the data
        // directly from the existing definition and update the initial index for decoding.
        if field_list.0 {
            for (index, field) in field_list.1.iter().enumerate() {
                self.add_field_to_data_view(
                    packed_file_decoded,
                    &field.field_name,
                    field.field_type.to_owned(),
                    field.field_is_key,
                    &field.field_is_reference,
                    &field.field_description,
                    &mut index_data,
                    Some(index)
                );
            }
        }

        // Check if the index does even exist, to avoid crashes.
        if packed_file_decoded.get(*index_data).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(packed_file_decoded[*index_data], &mut index_data.clone()) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&packed_file_decoded[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&packed_file_decoded[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if packed_file_decoded.get(*index_data + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&packed_file_decoded[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&packed_file_decoded[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if packed_file_decoded.get(*index_data + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&packed_file_decoded[*index_data..(*index_data + 8)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if packed_file_decoded.get(*index_data + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&packed_file_decoded[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&packed_file_decoded[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        self.bool_entry.get_buffer().set_text(decoded_bool);
        self.float_entry.get_buffer().set_text(&*decoded_float);
        self.integer_entry.get_buffer().set_text(&*decoded_integer);
        self.long_integer_entry.get_buffer().set_text(&*decoded_long_integer);
        self.string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u8));
        self.string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u16));
        self.optional_string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u8));
        self.optional_string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u16));

        // We reset these two every time we add a field.
        self.field_name_entry.get_buffer().set_text(&format!("Unknown {}", *index_data));
        self.is_key_field_switch.set_state(false);

        // Then we set the TextTags to paint the hex_data.
        let raw_data_hex_text_buffer = self.raw_data.get_buffer().unwrap();

        // Clear the current index tag.
        raw_data_hex_text_buffer.remove_tag_by_name("index", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());
        raw_data_hex_text_buffer.remove_tag_by_name("entry", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (*index_data * 3 / 48) as i32;
        let index_line_end = (((*index_data * 3) + 2) / 48) as i32;
        let index_char_start = ((*index_data * 3) % 48) as i32;
        let index_char_end = (((*index_data * 3) + 2) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = ((self.data_initial_index * 3) / 48) as i32;
        let header_char = ((self.data_initial_index * 3) % 48) as i32;
        let index_line_end = ((*index_data * 3) / 48) as i32;
        let index_char_end = ((*index_data * 3) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // And then, we do the same for `raw_decoded_data`.
        let raw_data_decoded_text_buffer = self.raw_data_decoded.get_buffer().unwrap();

        // Clear the current index and entry tags.
        raw_data_decoded_text_buffer.remove_tag_by_name("index", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());
        raw_data_decoded_text_buffer.remove_tag_by_name("entry", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (*index_data / 16) as i32;
        let index_line_end = ((*index_data + 1) / 16) as i32;
        let index_char_start = (*index_data % 16) as i32;
        let index_char_end = ((*index_data + 1) % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = (self.data_initial_index / 16) as i32;
        let header_char = (self.data_initial_index % 16) as i32;
        let index_line_end = (*index_data / 16) as i32;
        let index_char_end = (*index_data % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );
    }

    /// This function gets the data from the "Decoder" table, and returns it, so we can save it in a
    /// TableDefinition.fields.
    pub fn return_data_from_data_view(&self) -> Vec<Field> {
        let mut fields = vec![];

        // Only in case we have any line in the ListStore we try to get it. Otherwise we return an empty vector.
        if let Some(current_line) = self.fields_list_store.get_iter_first() {
            let mut done = false;
            while !done {
                let field_name = self.fields_list_store.get_value(&current_line, 1).get().unwrap();
                let field_is_key = self.fields_list_store.get_value(&current_line, 3).get().unwrap();
                let ref_table: String = self.fields_list_store.get_value(&current_line, 4).get().unwrap();
                let ref_column: String = self.fields_list_store.get_value(&current_line, 5).get().unwrap();
                let field_description: String = self.fields_list_store.get_value(&current_line, 7).get().unwrap();

                let field_type = match self.fields_list_store.get_value(&current_line, 2).get().unwrap() {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" | _=> FieldType::OptionalStringU16,
                };

                if ref_table.is_empty() {
                    fields.push(Field::new(field_name, field_type, field_is_key, None, field_description));
                }
                else {
                    fields.push(Field::new(field_name, field_type, field_is_key, Some((ref_table, ref_column)), field_description));
                }

                if !self.fields_list_store.iter_next(&current_line) {
                    done = true;
                }
            }
        }
        fields
    }
}



/// This function creates the TextTags `header` and `index` for the provided TextView.
fn create_text_tags(text_view: &TextView) {

    // Get the TagTable of the Buffer of the TextView...
    let text_buffer = text_view.get_buffer().unwrap();
    let text_buffer_tag_table = text_buffer.get_tag_table().unwrap();

    // Tag for the header (Red Background)
    let text_tag_header = TextTag::new(Some("header"));
    text_tag_header.set_property_background(Some("lightcoral"));
    text_buffer_tag_table.add(&text_tag_header);

    // Tag for the current index (Yellow Background)
    let text_tag_index = TextTag::new(Some("index"));
    text_tag_index.set_property_background(Some("goldenrod"));
    text_buffer_tag_table.add(&text_tag_index);

    // Tag for the currently decoded entry (Light Blue Background)
    let text_tag_index = TextTag::new(Some("entry"));
    text_tag_index.set_property_background(Some("lightblue"));
    text_buffer_tag_table.add(&text_tag_index);
}
*/

/// This function "process" the column names of a table, so they look like they should.
fn clean_column_names(field_name: &str) -> String {

    // Create the "New" processed `String`.
    let mut new_name = String::new();

    // Variable to know if the next character should be uppercase.
    let mut should_be_uppercase = false;

    // For each character...
    for character in field_name.chars() {

        // If it's the first character, or it should be Uppercase....
        if new_name.is_empty() || should_be_uppercase {

            // Make it Uppercase and set that flag to false.
            new_name.push_str(&character.to_uppercase().to_string());
            should_be_uppercase = false;
        }

        // If it's an underscore...
        else if character == '_' {

            // Replace it with a whitespace and set the "Uppercase" flag to true.
            new_name.push(' ');
            should_be_uppercase = true;
        }

        // Otherwise... it's a normal character.
        else { new_name.push(character); }
    }

    new_name
}


/// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
/// His intended use is for just after we reload the data to the table.
fn build_columns(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel
) {
    // Create a list of "Key" columns.
    let mut keys = vec![];

    // For each column...
    for (index, field) in definition.fields.iter().enumerate() {

        // Create the "New" processed `String`.
        let mut new_name = clean_column_names(&field.field_name);

        // Set his title.
        unsafe { model.as_mut().unwrap().set_header_data((index as i32, Orientation::Horizontal, &Variant::new0(&QString::from_std_str(&new_name)))); }

        // Depending on his type, set one width or another.
        match field.field_type {
            FieldType::Boolean => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
            FieldType::Float => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
            FieldType::Integer => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
            FieldType::LongInteger => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
            FieldType::StringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::StringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::OptionalStringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::OptionalStringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
        }

        // If the field is key, add that column to the "Key" list, so we can move them at the begining later.
        if field.field_is_key { keys.push(index); }
    }

    // If we have any "Key" field...
    if !keys.is_empty() {

        // Get the Horizontal Header of the Table.
        let header;
        unsafe { header = table_view.as_mut().unwrap().horizontal_header(); }

        // For each key column (in reverse)...
        for column in keys.iter().rev() {

            // Move the column to the begining.
            unsafe { header.as_mut().unwrap().move_section(*column as i32, 0); }
        }
    }
}

/// This function checks if the data in the clipboard is suitable for the selected Items.
fn check_clipboard(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel,
) -> bool {

    // Get the clipboard.
    let clipboard = GuiApplication::clipboard();

    // Get the current selection.
    let selection;
    unsafe { selection = table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
    let indexes = selection.indexes();

    // Get the text from the clipboard.
    let text;
    unsafe { text = QString::to_std_string(&clipboard.as_mut().unwrap().text(())); }

    // If there is something in the clipboard...
    if !text.is_empty() {

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

            // Get the column of that cell.
            let column;
            unsafe { column = cell.0.as_mut().unwrap().index().column(); }

            // Depending on the column, we try to encode the data in one format or another.
            match definition.fields[column as usize].field_type {
                    FieldType::Boolean => if cell.1 == "true" || cell.1 == "false" { continue } else { return false },
                    FieldType::Float => if cell.1.parse::<f32>().is_ok() { continue } else { return false },
                    FieldType::Integer => if cell.1.parse::<i32>().is_ok() { continue } else { return false },
                    FieldType::LongInteger => if cell.1.parse::<i64>().is_ok() { continue } else { return false },

                    // All these are Strings, so we can skip their checks....
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => continue
            }
        }

        // If we reach this place, it means none of the cells was incorrect, so we can paste.
        true
    }

    // Otherwise, we cannot paste anything.
    else { false }
}
