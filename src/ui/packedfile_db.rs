// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate gtk;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate hex_slice;
extern crate failure;

use std::cell::RefCell;
use std::rc::Rc;
use packedfile::db::*;
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
use packedfile::SerializableToCSV;
use AppUI;
use packfile::update_packed_file_data_db;

use self::hex_slice::AsHex;

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


/// Implementation of `PackedFileDBTreeView`.
impl PackedFileDBTreeView{

    /// This function creates a new `TreeView` with `packed_file_data_display` as father and returns a
    /// `PackedFileDBTreeView` with all his data.
    pub fn create_tree_view(
        application: &Application,
        app_ui: &AppUI,
        pack_file: Rc<RefCell<PackFile>>,
        packed_file_decoded: Rc<RefCell<DB>>,
        packed_file_decoded_index: &usize,
        dependency_database: &Option<Vec<PackedFile>>,
        master_schema: &Schema,
        settings: &Settings,
    ) -> Result<(), Error> {

        // Here we define the `Accept` response for GTK, as it seems Restson causes it to fail to compile
        // if we get them to i32 directly in the `if` statement.
        // NOTE: For some bizarre reason, GTKFileChoosers return `Ok`, while native ones return `Accept`.
        let gtk_response_accept: i32 = ResponseType::Accept.into();

        // Get the table definition of this table.
        let table_definition = packed_file_decoded.borrow().packed_file_data.table_definition.clone();

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

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {

                                        // If we could decode it...
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1 {

                                                    // For each row...
                                                    for row in &db.packed_file_data.packed_file_data {

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

                                // For each table in our mod...
                                for table in &pack_file.borrow().pack_file_data.packed_files {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {

                                        // If we could decode it...
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1 {

                                                    // For each row...
                                                    for row in &db.packed_file_data.packed_file_data {

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

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {

                                        // If we could decode it...
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1 {

                                                    // For each row...
                                                    for row in &db.packed_file_data.packed_file_data {

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

                                // For each table in our mod...
                                for table in &pack_file.borrow().pack_file_data.packed_files {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {

                                        // If we could decode it...
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1 {

                                                    // For each row...
                                                    for row in &db.packed_file_data.packed_file_data {

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

        // Create the separator between the "Import/Export" buttons and the rest.
        let separator_2 = Separator::new(Orientation::Vertical);

        // Create the "Import from CSV" button.
        let import_csv_button = ModelButton::new();
        import_csv_button.set_property_text(Some("Import from CSV"));
        import_csv_button.set_action_name("app.packedfile_db_import_csv");

        // Create the "Export to CSV" button.
        let export_csv_button = ModelButton::new();
        export_csv_button.set_property_text(Some("Export to CSV"));
        export_csv_button.set_action_name("app.packedfile_db_export_csv");

        // Right-click menu actions.
        let add_rows = SimpleAction::new("packedfile_db_add_rows", None);
        let delete_rows = SimpleAction::new("packedfile_db_delete_rows", None);
        let copy_cell = SimpleAction::new("packedfile_db_copy_cell", None);
        let paste_cell = SimpleAction::new("packedfile_db_paste_cell", None);
        let clone_rows = SimpleAction::new("packedfile_db_clone_rows", None);
        let copy_rows = SimpleAction::new("packedfile_db_copy_rows", None);
        let paste_rows = SimpleAction::new("packedfile_db_paste_rows", None);
        let import_csv = SimpleAction::new("packedfile_db_import_csv", None);
        let export_csv = SimpleAction::new("packedfile_db_export_csv", None);

        application.add_action(&add_rows);
        application.add_action(&delete_rows);
        application.add_action(&copy_cell);
        application.add_action(&paste_cell);
        application.add_action(&clone_rows);
        application.add_action(&copy_rows);
        application.add_action(&paste_rows);
        application.add_action(&import_csv);
        application.add_action(&export_csv);

        // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
        application.set_accels_for_action("app.packedfile_db_add_rows", &["<Primary><Shift>a"]);
        application.set_accels_for_action("app.packedfile_db_delete_rows", &["<Shift>Delete"]);
        application.set_accels_for_action("app.packedfile_db_copy_cell", &["<Primary>c"]);
        application.set_accels_for_action("app.packedfile_db_paste_cell", &["<Primary>v"]);
        application.set_accels_for_action("app.packedfile_db_clone_rows", &["<Primary><Shift>d"]);
        application.set_accels_for_action("app.packedfile_db_copy_rows", &["<Primary>z"]);
        application.set_accels_for_action("app.packedfile_db_paste_rows", &["<Primary>x"]);
        application.set_accels_for_action("app.packedfile_db_import_csv", &["<Primary><Shift>i"]);
        application.set_accels_for_action("app.packedfile_db_export_csv", &["<Primary><Shift>e"]);

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
        context_menu_grid.attach(&separator_2, 0, 8, 2, 1);
        context_menu_grid.attach(&import_csv_button, 0, 9, 2, 1);
        context_menu_grid.attach(&export_csv_button, 0, 10, 2, 1);

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
            &packed_file_decoded.borrow().packed_file_data,
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
                table => move |tree_view, button| {

                    // If we clicked the right mouse button...
                    if button.get_button() == 3 {

                        table.context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                        table.context_menu.popup();
                    }

                    Inhibit(false)
                }
            ));

            // We check if we can delete something on selection changes.
            table.tree_view.connect_cursor_changed(clone!(
                copy_cell,
                copy_rows,
                clone_rows,
                delete_rows => move |tree_view| {

                    // If we have something selected...
                    if tree_view.get_selection().count_selected_rows() > 0 {

                        // Allow to delete, clone and copy.
                        copy_cell.set_enabled(true);
                        copy_rows.set_enabled(true);
                        clone_rows.set_enabled(true);
                        delete_rows.set_enabled(true);
                    }

                    // Otherwise, disable them.
                    else {
                        copy_cell.set_enabled(false);
                        copy_rows.set_enabled(false);
                        clone_rows.set_enabled(false);
                        delete_rows.set_enabled(false);
                    }
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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

                                    // If the cell is the index...
                                    if column == 0 {

                                        // Don't do anything.
                                        return
                                    }

                                    // Otherwise...
                                    else {

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
                                                packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                // Try to save the changes to the PackFile. If there is an error, report it.
                                                if let Err(error) = update_packed_file_data_db(
                                                    &*packed_file_decoded.borrow_mut(),
                                                    &mut *pack_file.borrow_mut(),
                                                    packed_file_decoded_index as usize
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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

                            // Create the `String` to hold the data from the string.
                            let mut row_text = String::new();

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

                                // Add the text to the copied row.
                                row_text.push_str(&format!("\"{}\"", data));

                                // If it's not the last column...
                                if column < table_definition.fields.len() {

                                    // Put a comma between fields, so excel understand them.
                                    row_text.push_str(",");
                                }
                            }

                            // Add the copied row to the list.
                            copy_string.push_str(&format!("{}\n", row_text));
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

                        // When it gets the data from the `Clipboard`....
                        if let Some(data) = app_ui.clipboard.wait_for_text() {

                            // Get the definitions for this table.
                            let fields_type = table_definition.fields.iter().map(|x| x.field_type.clone()).collect::<Vec<FieldType>>();

                            // Store here all the decoded fields.
                            let mut fields_data = vec![];

                            // Get the type of the data copied. If it's in CSV format...
                            if data.find("\",\"").is_some() {

                                // For each row in the data we received...
                                for row in data.lines() {

                                    // Remove the "" at the beginning and at the end.
                                    let mut row = row.to_owned();
                                    row.pop();
                                    row.remove(0);

                                    // Get all the data from his fields.
                                    fields_data.push(row.split("\",\"").map(|x| x.to_owned()).collect::<Vec<String>>());
                                }
                            }

                            // Otherwise, we asume it's a TSV copy from excel.
                            // TODO: Check this with other possible sources.
                            else {

                                // For each row in the data we received...
                                for row in data.lines() {

                                    // Get all the data from his fields.
                                    fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
                                }
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

                                        // Check his `field_type`...
                                        match *field_type {

                                            // If it's a boolean, get "true" or "false".
                                            FieldType::Boolean => {
                                                let state = if field == "true" { true } else if field == "false" { false } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".")
                                                };
                                                table.list_store.set_value(&tree_iter, (index + 1) as u32, &state.to_value());
                                            }
                                            FieldType::Integer => {
                                                if let Ok(field) = field.parse::<i32>() {
                                                    table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I32.")
                                                };
                                            },
                                            FieldType::LongInteger => {
                                                if let Ok(field) = field.parse::<i64>() {
                                                    table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                } else {
                                                    return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I64.")
                                                };
                                            },
                                            FieldType::Float => {
                                                if field.parse::<f32>().is_ok() {
                                                    table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                } else { return show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid F32.") }
                                            },

                                            // All these are Strings, so it can be together,
                                            FieldType::StringU8 |
                                            FieldType::StringU16 |
                                            FieldType::OptionalStringU8 |
                                            FieldType::OptionalStringU16 => table.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value()),
                                        };
                                    }

                                    // If the field doesn't exists, return.
                                    else { return }
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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
            ));

            // When we hit the "Import from CSV" button.
            import_csv.connect_activate(clone!(
                app_ui,
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
                            "Select CSV File to Import...",
                            &app_ui.window,
                            FileChooserAction::Open,
                            "Import",
                            "Cancel"
                        );

                        // Enable the CSV filter for the `FileChooser`.
                        file_chooser_filter_packfile(&file_chooser, "*.csv");

                        // If we have selected a file to import...
                        if file_chooser.run() == gtk_response_accept {

                            // Just in case the import fails after importing (for example, due to importing a CSV from another table,
                            // or from another version of the table, and it fails while loading to table or saving to PackFile)
                            // we save a copy of the table, so we can restore it if it fails after we modify it.
                            let packed_file_data_copy = packed_file_decoded.borrow_mut().packed_file_data.clone();
                            let mut restore_table = (false, format_err!(""));

                            // If there is an error importing, we report it. This only edits the data after checking
                            // that it can be decoded properly, so we don't need to restore the table in this case.
                            if let Err(error) = DBData::import_csv(
                                &mut packed_file_decoded.borrow_mut().packed_file_data,
                                &file_chooser.get_filename().unwrap()
                            ) {
                                return show_dialog(&app_ui.window, false, error.cause());
                            }

                            // If there is an error loading the data (wrong table imported?), report it and restore it from the old copy.
                            if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view(&packed_file_decoded.borrow().packed_file_data, &table.list_store) {
                                restore_table = (true, error);
                            }

                            // If the table loaded properly, try to save the data to the encoded file.
                            if !restore_table.0 {
                                if let Err(error) = update_packed_file_data_db(&*packed_file_decoded.borrow_mut(), &mut *pack_file.borrow_mut(), packed_file_decoded_index as usize) {
                                    restore_table = (true, error);
                                }
                            }

                            // If the import broke somewhere along the way.
                            if restore_table.0 {

                                // Restore the old copy.
                                packed_file_decoded.borrow_mut().packed_file_data = packed_file_data_copy;

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

            // When we hit the "Export to CSV" button.
            export_csv.connect_activate(clone!(
                app_ui,
                packed_file_decoded,
                table => move |_,_| {

                    // We hide the context menu first.
                    table.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView. This should stop problems with
                    // the accels working everywhere.
                    if table.tree_view.has_focus() {

                        let file_chooser = FileChooserNative::new(
                            "Export CSV File...",
                            &app_ui.window,
                            FileChooserAction::Save,
                            "Save",
                            "Cancel"
                        );

                        // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                        file_chooser.set_do_overwrite_confirmation(true);

                        // Get it's tree_path and it's default name (table-table_name.csv)
                        let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
                        file_chooser.set_current_name(format!("{}-{}.csv", &tree_path[1], &tree_path.last().unwrap()));

                        // If we hit "Save"...
                        if file_chooser.run() == gtk_response_accept {

                            // Try to export the CSV.
                            match DBData::export_csv(&packed_file_decoded.borrow_mut().packed_file_data, &file_chooser.get_filename().unwrap()) {
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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
                                    packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                    if let Err(error) = update_packed_file_data_db(
                                        &*packed_file_decoded.borrow_mut(),
                                        &mut *pack_file.borrow_mut(),
                                        packed_file_decoded_index as usize
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
                                            packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index as usize
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
                                            packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index as usize
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
                                            packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                            if let Err(error) = update_packed_file_data_db(
                                                &*packed_file_decoded.borrow_mut(),
                                                &mut *pack_file.borrow_mut(),
                                                packed_file_decoded_index as usize
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
                                packed_file_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                // Try to save the changes to the PackFile. If there is an error, report it.
                                if let Err(error) = update_packed_file_data_db(
                                    &*packed_file_decoded.borrow_mut(),
                                    &mut *pack_file.borrow_mut(),
                                    packed_file_decoded_index as usize
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
        for row in &packed_file_data.packed_file_data {

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
        db_header: (DBHeader, usize),
        schema: &Rc<RefCell<Option<Schema>>>,
    ) -> Result<(), Error> {

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
            data_initial_index: db_header.1,
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
        let table_definition = match DB::get_schema(&table_name, db_header.0.packed_file_header_packed_file_version, &schema.borrow().clone().unwrap()) {
            Some(table_definition) => Rc::new(RefCell::new(table_definition)),
            None => Rc::new(RefCell::new(TableDefinition::new(db_header.0.packed_file_header_packed_file_version)))
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

            // We check if we can allow actions on selection changes.
            decoder_view.all_table_versions_tree_view.connect_cursor_changed(clone!(
                decoder_view => move |tree_view| {

                    // If nothing is selected, enable all the actions.
                    if tree_view.get_selection().count_selected_rows() > 0 {
                        decoder_view.all_table_versions_remove_definition.set_sensitive(true);
                        decoder_view.all_table_versions_load_definition.set_sensitive(true);
                    }

                    // Otherwise, disable them.
                    else {
                        decoder_view.all_table_versions_remove_definition.set_sensitive(false);
                        decoder_view.all_table_versions_load_definition.set_sensitive(false);
                    }
                }
            ));

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

            // When we press the "Delete all fields" button.
            decoder_view.delete_all_fields_button.connect_button_release_event(clone!(
                packed_file_data,
                index_data,
                decoder_view => move |delete_all_fields_button,_| {

                    // Clear the `TreeView`.
                    decoder_view.fields_list_store.clear();

                    // Reset the "index_data".
                    *index_data.borrow_mut() = decoder_view.data_initial_index;

                    // Disable this button.
                    delete_all_fields_button.set_sensitive(false);

                    // Re-update the "Decoder" View.
                    decoder_view.update_decoder_view(
                        &packed_file_data,
                        (false, &[]),
                        &mut index_data.borrow_mut()
                    );

                    Inhibit(false)
                }
            ));

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

        // Fields TreeView actions.
        {

            // We check if we can allow actions on selection changes.
            decoder_view.fields_tree_view.connect_cursor_changed(clone!(
                move_row_up,
                move_row_down,
                delete_row => move |tree_view| {

                    // If something is selected, enable all the actions.
                    if tree_view.get_selection().count_selected_rows() > 0 {
                        move_row_up.set_enabled(true);
                        move_row_down.set_enabled(true);
                        delete_row.set_enabled(true);
                    }

                    // Otherwise, disable them.
                    else {
                        move_row_up.set_enabled(false);
                        move_row_down.set_enabled(false);
                        delete_row.set_enabled(false);
                    }
                }
            ));

            // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
            // Then we calculate the position where the popup must aim, and show it.
            //
            // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
            decoder_view.fields_tree_view.connect_button_release_event(clone!(
                decoder_view => move |tree_view, button| {

                    // If we clicked the right mouse button...
                    if button.get_button() == 3 {

                        decoder_view.context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                        decoder_view.context_menu.popup();
                    }

                    Inhibit(false)
                }
            ));

            // When we press the "Move up" button.
            move_row_up.connect_activate(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_,_| {

                    // Hide the contextual menu.
                    decoder_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                    // the accels working everywhere.
                    if decoder_view.fields_tree_view.has_focus() {

                        // Get the current iter.
                        let current_iter = decoder_view.fields_tree_view.get_selection().get_selected().unwrap().1;
                        let new_iter = current_iter.clone();

                        // If there is a previous iter, swap them.
                        if decoder_view.fields_list_store.iter_previous(&new_iter) {
                            decoder_view.fields_list_store.move_before(&current_iter, &new_iter);

                            // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                            decoder_view.update_first_row_decoded(&packed_file_data, false, &mut index_data.borrow_mut());
                        }
                    }
                }
            ));

            // When we press the "Move down" button.
            move_row_down.connect_activate(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_,_| {

                    // Hide the contextual menu.
                    decoder_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                    // the accels working everywhere.
                    if decoder_view.fields_tree_view.has_focus() {

                        // Get the current iter.
                        let current_iter = decoder_view.fields_tree_view.get_selection().get_selected().unwrap().1;
                        let new_iter = current_iter.clone();

                        // If there is a next iter, swap them.
                        if decoder_view.fields_list_store.iter_next(&new_iter) {
                            decoder_view.fields_list_store.move_after(&current_iter, &new_iter);

                            // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                            decoder_view.update_first_row_decoded(&packed_file_data, false, &mut index_data.borrow_mut());
                        }
                    }
                }
            ));

            // This allow us to remove a field from the list, using the decoder_delete_row action.
            delete_row.connect_activate(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_,_| {

                    // Hide the contextual menu.
                    decoder_view.context_menu.popdown();

                    // We only do something in case the focus is in the TreeView or in any of the moving buttons. This should stop problems with
                    // the accels working everywhere.
                    if decoder_view.fields_tree_view.has_focus() {

                        // If there is something selected, delete it.
                        if let Some(selection) = decoder_view.fields_tree_view.get_selection().get_selected() {
                            decoder_view.fields_list_store.remove(&selection.1);
                        }

                        // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                        decoder_view.update_first_row_decoded(&packed_file_data, false, &mut index_data.borrow_mut());
                    }
                }
            ));
        }

        // Logic for all the "Use this" buttons. Basically, they just check if it's possible to use their decoder
        // for the bytes we have, and advance the index and add their type to the fields view.
        {

            // When we hit the "Use this" button for boolean fields.
            decoder_view.use_bool_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::Boolean,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for float fields.
            decoder_view.use_float_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::Float,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for integer fields.
            decoder_view.use_integer_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::Integer,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for long integer fields.
            decoder_view.use_long_integer_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::LongInteger,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for string U8 fields.
            decoder_view.use_string_u8_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::StringU8,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for string u16 fields.
            decoder_view.use_string_u16_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::StringU16,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for optional string u8 fields.
            decoder_view.use_optional_string_u8_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::OptionalStringU8,
                    );

                    Inhibit(false)
                }
            ));

            // When we hit the "Use this" button for optional string u16 fields.
            decoder_view.use_optional_string_u16_button.connect_button_release_event(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_ ,_|{

                    // Add the field to the table, update it, and get the new "index_data".
                    decoder_view.use_this(
                        &mut index_data.borrow_mut(),
                        &packed_file_data,
                        FieldType::OptionalStringU16,
                    );

                    Inhibit(false)
                }
            ));
        }

        // Interaction with the fields TreeView.
        {

            // This allow us to change a field's data type in the TreeView.
            decoder_view.fields_tree_view_cell_combo.connect_edited(clone!(
                index_data,
                packed_file_data,
                decoder_view => move |_, tree_path, new_value| {

                    // Get his iter and change it. Not to hard.
                    let tree_iter = &decoder_view.fields_list_store.get_iter(&tree_path).unwrap();
                    decoder_view.fields_list_store.set_value(tree_iter, 2, &new_value.to_value());

                    // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                    decoder_view.update_first_row_decoded(&packed_file_data, false, &mut index_data.borrow_mut());
                }
            ));

            // This allow us to set as "key" a field in the TreeView.
            decoder_view.fields_tree_view_cell_bool.connect_toggled(clone!(
                decoder_view => move |cell, tree_path| {

                    // Get his `TreeIter`.
                    let tree_iter = decoder_view.fields_list_store.get_iter(&tree_path).unwrap();

                    // Get his new state.
                    let state = !cell.get_active();

                    // Change it in the `ListStore`.
                    decoder_view.fields_list_store.set_value(&tree_iter, 3, &state.to_value());

                    // Change his state.
                    cell.set_active(state);
                }
            ));

            // This loop takes care of the interaction with string cells.
            for edited_cell in &decoder_view.fields_tree_view_cell_string {
                edited_cell.connect_edited(clone!(
                    decoder_view => move |_ ,tree_path , new_text| {

                        // Get his iter.
                        let tree_iter = decoder_view.fields_list_store.get_iter(&tree_path).unwrap();

                        // Get his column.
                        let edited_cell_column = decoder_view.fields_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                        // Set his new value.
                        decoder_view.fields_list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());
                    }
                ));
            }
        }

        // Destruction event.
        {
            // When the decoder is destroyed...
            decoder_view.decoder_grid_scroll.connect_destroy(clone!(
                app_ui => move |_| {

                    // Restore the "Change game selected" function.
                    app_ui.menu_bar_change_game_selected.set_enabled(true);
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

        // Get the header of the Table, if it's a table.
        let db_header = DBHeader::read(packed_file_encoded)?;

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
        let mut hex_raw_data = format!("{:X}", packed_file_encoded.as_hex());

        hex_raw_data.remove(0);

        // We need to do this 2 times, because the first one skips chars if they are consecutive.
        let hex_raw_data = hex_raw_data.replace(" 0 ", " 00 ");
        let hex_raw_data = hex_raw_data.replace(" 1 ", " 01 ");
        let hex_raw_data = hex_raw_data.replace(" 2 ", " 02 ");
        let hex_raw_data = hex_raw_data.replace(" 3 ", " 03 ");
        let hex_raw_data = hex_raw_data.replace(" 4 ", " 04 ");
        let hex_raw_data = hex_raw_data.replace(" 5 ", " 05 ");
        let hex_raw_data = hex_raw_data.replace(" 6 ", " 06 ");
        let hex_raw_data = hex_raw_data.replace(" 7 ", " 07 ");
        let hex_raw_data = hex_raw_data.replace(" 8 ", " 08 ");
        let hex_raw_data = hex_raw_data.replace(" 9 ", " 09 ");
        let hex_raw_data = hex_raw_data.replace(" A ", " 0A ");
        let hex_raw_data = hex_raw_data.replace(" B ", " 0B ");
        let hex_raw_data = hex_raw_data.replace(" C ", " 0C ");
        let hex_raw_data = hex_raw_data.replace(" D ", " 0D ");
        let hex_raw_data = hex_raw_data.replace(" E ", " 0E ");
        let hex_raw_data = hex_raw_data.replace(" F ", " 0F ");

        let hex_raw_data = hex_raw_data.replace(" 0 ", " 00 ");
        let hex_raw_data = hex_raw_data.replace(" 1 ", " 01 ");
        let hex_raw_data = hex_raw_data.replace(" 2 ", " 02 ");
        let hex_raw_data = hex_raw_data.replace(" 3 ", " 03 ");
        let hex_raw_data = hex_raw_data.replace(" 4 ", " 04 ");
        let hex_raw_data = hex_raw_data.replace(" 5 ", " 05 ");
        let hex_raw_data = hex_raw_data.replace(" 6 ", " 06 ");
        let hex_raw_data = hex_raw_data.replace(" 7 ", " 07 ");
        let hex_raw_data = hex_raw_data.replace(" 8 ", " 08 ");
        let hex_raw_data = hex_raw_data.replace(" 9 ", " 09 ");
        let hex_raw_data = hex_raw_data.replace(" A ", " 0A ");
        let hex_raw_data = hex_raw_data.replace(" B ", " 0B ");
        let hex_raw_data = hex_raw_data.replace(" C ", " 0C ");
        let hex_raw_data = hex_raw_data.replace(" D ", " 0D ");
        let hex_raw_data = hex_raw_data.replace(" E ", " 0E ");
        let hex_raw_data = hex_raw_data.replace(" F ", " 0F ");

        // This filtering doesn't work with the last char, so we need to pass that one separated.
        let hex_raw_data = hex_raw_data.replace(" 0]", " 00]");
        let hex_raw_data = hex_raw_data.replace(" 1]", " 01]");
        let hex_raw_data = hex_raw_data.replace(" 2]", " 02]");
        let hex_raw_data = hex_raw_data.replace(" 3]", " 03]");
        let hex_raw_data = hex_raw_data.replace(" 4]", " 04]");
        let hex_raw_data = hex_raw_data.replace(" 5]", " 05]");
        let hex_raw_data = hex_raw_data.replace(" 6]", " 06]");
        let hex_raw_data = hex_raw_data.replace(" 7]", " 07]");
        let hex_raw_data = hex_raw_data.replace(" 8]", " 08]");
        let hex_raw_data = hex_raw_data.replace(" 9]", " 09]");
        let hex_raw_data = hex_raw_data.replace(" A]", " 0A]");
        let hex_raw_data = hex_raw_data.replace(" B]", " 0B]");
        let hex_raw_data = hex_raw_data.replace(" C]", " 0C]");
        let hex_raw_data = hex_raw_data.replace(" D]", " 0D]");
        let hex_raw_data = hex_raw_data.replace(" E]", " 0E]");
        let mut hex_raw_data = hex_raw_data.replace(" F]", " 0F]");
        hex_raw_data.pop();

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
        self.table_version_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_version));
        self.table_entry_count_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_entry_count));

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

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    pub fn update_versions_list(
        &self,
        schema: &Schema,
        table_name: &str,
    ) {
        // Clear the current list.
        self.all_table_versions_list_store.clear();

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Some(table_versions_list) = DB::get_schema_versions_list(table_name, schema) {
            for version in table_versions_list {
                self.all_table_versions_list_store.insert_with_values(None, &[0], &[&version.version]);
            }
        }
    }

    /// This function adds fields to the "Decoder" table, so we can do this without depending on the
    /// updates of the Decoder view. As this has a lot of required data, lets's explain the weirdest ones:
    /// - index_data: the index to start decoding from the vector.
    /// - index_row: the position in the row. None to calculate the last position's number.
    pub fn add_field_to_data_view(
        &self,
        packed_file_decoded: &[u8],
        field_name: &str,
        field_type: FieldType,
        field_is_key: bool,
        field_is_reference: &Option<(String, String)>,
        field_description: &str,
        mut index_data: &mut usize,
        index_row: Option<usize>
    ) {

        let field_index = match index_row {
            Some(index) => format!("{:0count$}", index + 1, count = 3),
            None => {

                // Get the first iter of the table, then move until you get the last one.
                match self.fields_list_store.get_iter_first() {
                    Some(tree_iter) => {

                        loop {
                            let tree_iter_copy = tree_iter.clone();
                            if !self.fields_list_store.iter_next(&tree_iter_copy) { break; }
                            else { self.fields_list_store.iter_next(&tree_iter); }
                        }

                        // Then, get his number, and add 1 to it.
                        let index = self.fields_list_store.get_value(&tree_iter, 0).get::<String>().unwrap().parse::<i32>().unwrap();
                        format!("{:0count$}", index + 1, count = 3)
                    }
                    None => format!("001"),
                }
            },
        };

        let decoded_data = decode_data_by_fieldtype(
            packed_file_decoded,
            &field_type,
            &mut index_data
        );

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

        if let Some(ref reference) = *field_is_reference {
            self.fields_list_store.insert_with_values(
                None,
                &[0, 1, 2, 3, 4, 5, 6, 7],
                &[
                    &field_index,
                    &field_name,
                    &field_type,
                    &field_is_key,
                    &reference.0,
                    &reference.1,
                    &decoded_data,
                    &field_description,
                ]
            );
        }
        else {
            self.fields_list_store.insert_with_values(
                None,
                &[0, 1, 2, 3, 4, 5, 6, 7],
                &[
                    &field_index,
                    &field_name,
                    &field_type,
                    &field_is_key,
                    &String::new(),
                    &String::new(),
                    &decoded_data,
                    &field_description,
                ]
            );
        }
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

    /// This function is used to update the `PackedFileDBDecoder` when we try to add a new field to
    /// the schema with one of the "Use this" buttons.
    pub fn use_this(
        &self,
        mut index_data: &mut usize,
        packed_file_data_encoded: &[u8],
        field_type: FieldType,
    ) {

        // Try to add the field, and update the index with it.
        self.add_field_to_data_view(
            packed_file_data_encoded,
            &self.field_name_entry.get_buffer().get_text(),
            field_type,
            self.is_key_field_switch.get_active(),
            &None,
            &String::new(),
            &mut index_data,
            None
        );

        // Update all the dynamic data of the "Decoder" view.
        self.update_decoder_view(
            packed_file_data_encoded,
            (false, &[]),
            &mut index_data,
        );

        // Enable the "Delete all fields" button.
        self.delete_all_fields_button.set_sensitive(true);
    }

    /// This function updates the "First row decoded" column in the Decoder View, the current index and
    /// the decoded entries. This should be called in row changes (deletion and moving, not adding).
    fn update_first_row_decoded(&self, packedfile: &[u8], get_start_of_list: bool, mut index: &mut usize) {
        let iter = self.fields_list_store.get_iter_first();

        // Reset the index.
        *index = self.data_initial_index;
        if let Some(current_iter) = iter {
            loop {

                // Get the type from the column...
                let field_type = match self.fields_list_store.get_value(&current_iter, 2).get().unwrap() {
                    "Bool"=> FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" | _ => FieldType::OptionalStringU16,
                };

                // Get the decoded data using it's type...
                let decoded_data = decode_data_by_fieldtype(
                    packedfile,
                    &field_type,
                    &mut index
                );

                // Set the new values.
                self.fields_list_store.set_value(&current_iter, 6, &gtk::ToValue::to_value(&decoded_data));

                // If we are trying to get the start of a list, and we are at the iter of the list, stop.
                if get_start_of_list && self.fields_tree_view.get_selection().iter_is_selected(&current_iter) { break; }

                // Break the loop once you run out of rows.
                if !self.fields_list_store.iter_next(&current_iter) { break; }
            }
        }
        self.update_decoder_view(packedfile, (false, &[]), &mut index);
    }
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

/// This function creates the TextTags `header` and `index` for the provided TextView.
pub fn create_text_tags(text_view: &TextView) {

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

/// This function "process" the column names of a table, so they look like they should.
pub fn clean_column_names(field_name: &str) -> String {

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
