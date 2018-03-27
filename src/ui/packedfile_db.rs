// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate hex_slice;
extern crate failure;

use packedfile::db::*;
use packedfile::db::schemas::*;
use packfile::packfile::PackedFile;
use common::coding_helpers;
use failure::Error;
use gtk::prelude::*;
use gtk::{
    Box, TreeView, ListStore, ScrolledWindow, Button, Orientation, TextView, Label, Entry,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type, Frame, CellRendererCombo, CssProvider,
    TextTag, Popover, ModelButton, Paned, Switch, Separator, Grid, ButtonBox, ButtonBoxStyle,
    StyleContext
};

use self::hex_slice::AsHex;

/// Struct PackedFileDBTreeView: contains all the stuff we need to give to the program to show a
/// TreeView with the data of a DB PackedFile, allowing us to manipulate it.
#[derive(Clone, Debug)]
pub struct PackedFileDBTreeView {
    pub packed_file_tree_view: TreeView,
    pub packed_file_list_store: ListStore,
    pub packed_file_tree_view_cell_bool: Vec<CellRendererToggle>,
    pub packed_file_tree_view_cell_float: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_integer: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_long_integer: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_optional_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_reference: Vec<CellRendererCombo>,
    pub packed_file_popover_menu: Popover,
    pub packed_file_popover_menu_add_rows_entry: Entry,
}

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
#[derive(Clone, Debug)]
pub struct PackedFileDBDecoder {
    pub data_initial_index: i32,
    pub raw_data_line_index: Label,
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
    pub move_up_button: ModelButton,
    pub move_down_button: ModelButton,
}

/// Implementation of "PackedFileDBTreeView".
impl PackedFileDBTreeView{

    /// This function creates a new TreeView with "packed_file_data_display" as father and returns a
    /// PackedFileDBTreeView with all his data.
    pub fn create_tree_view(
        packed_file_data_display: &Grid,
        packed_file_decoded: &DB,
        dependency_database: Option<Vec<PackedFile>>,
        local_dependency_database: &[PackedFile],
        master_schema: &Schema
    ) -> Result<PackedFileDBTreeView, Error> {

        // First, we create the Vec<Type> we are going to use to create the TreeView, based on the structure
        // of the DB PackedFile.
        let mut list_store_table_definition: Vec<Type> = vec![];
        let packed_file_table_definition = packed_file_decoded.packed_file_data.table_definition.clone();

        // The first column is an index for us to know how many entries we have.
        list_store_table_definition.push(Type::String);

        // Depending on the type of the field, we push the gtk::Type equivalent to that column.
        for field in &packed_file_table_definition.fields {
            match field.field_type {
                FieldType::Boolean => {
                    list_store_table_definition.push(Type::Bool);
                }
                FieldType::Integer => {
                    list_store_table_definition.push(Type::I32);
                }
                FieldType::LongInteger => {
                    list_store_table_definition.push(Type::I64);
                }
                FieldType::Float | FieldType::StringU8 | FieldType::StringU16 | FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                    list_store_table_definition.push(Type::String);
                }
            }
        }

        // Then, we create the new ListStore, the new TreeView, and prepare the TreeView to display the data
        let packed_file_tree_view = TreeView::new();
        let packed_file_list_store = ListStore::new(&list_store_table_definition);

        packed_file_tree_view.set_model(Some(&packed_file_list_store));
        packed_file_tree_view.set_grid_lines(gtk::TreeViewGridLines::Both);
        packed_file_tree_view.set_rubber_banding(true);
        packed_file_tree_view.set_margin_bottom(10);

        // Now we create the columns we need for this specific table. Always with an index column first.
        let cell_index = CellRendererText::new();
        cell_index.set_property_xalign(0.5);
        let column_index = TreeViewColumn::new();
        column_index.set_title("Index");
        column_index.set_clickable(true);
        column_index.set_min_width(50);
        column_index.set_sizing(gtk::TreeViewColumnSizing::Autosize);
        column_index.set_alignment(0.5);
        column_index.set_sort_column_id(0);
        column_index.pack_start(&cell_index, true);
        column_index.add_attribute(&cell_index, "text", 0);
        packed_file_tree_view.append_column(&column_index);

        let mut packed_file_tree_view_cell_bool = vec![];
        let mut packed_file_tree_view_cell_float = vec![];
        let mut packed_file_tree_view_cell_integer = vec![];
        let mut packed_file_tree_view_cell_long_integer = vec![];
        let mut packed_file_tree_view_cell_string = vec![];
        let mut packed_file_tree_view_cell_optional_string = vec![];
        let mut packed_file_tree_view_cell_reference: Vec<CellRendererCombo> = vec![];

        let mut index = 1;
        let mut key_columns = vec![];
        for field in &packed_file_table_definition.fields {

            // We need to fix the names here, so the column names are not broken.
            let mut new_name: String = String::new();
            let mut should_be_uppercase = false;
            for character in field.field_name.to_owned().chars() {
                let new_character: char;
                if new_name.is_empty() || should_be_uppercase {
                    new_character = character.to_uppercase().to_string().chars().nth(0).unwrap();
                    should_be_uppercase = false;
                }
                else if character == "_".chars().nth(0).unwrap() {
                    new_character = " ".chars().nth(0).unwrap();
                    should_be_uppercase = true;
                }
                else {
                    new_character = character;
                    should_be_uppercase = false;
                }
                new_name.push(new_character);
            }
            let field_name = new_name;

            // These are the specific declarations of the columns for every type implemented.
            match field.field_type {
                FieldType::Boolean => {
                    let cell_bool = CellRendererToggle::new();
                    // TODO: Make this respond dinamically to the font size.
                    // Reduce the size of the checkbox.
                    cell_bool.set_property_indicator_size(16i32);
                    cell_bool.set_activatable(true);
                    let column_bool = TreeViewColumn::new();
                    column_bool.set_title(&field_name);
                    column_bool.set_clickable(true);
                    column_bool.set_min_width(50);
                    column_bool.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                    column_bool.set_alignment(0.5);
                    column_bool.set_sort_column_id(index);
                    column_bool.pack_start(&cell_bool, true);
                    column_bool.add_attribute(&cell_bool, "active", index);
                    packed_file_tree_view.append_column(&column_bool);
                    packed_file_tree_view_cell_bool.push(cell_bool);
                    if field.field_is_key {
                        key_columns.push(column_bool);
                    }
                }
                FieldType::Float => {
                    let cell_float = CellRendererText::new();
                    cell_float.set_property_editable(true);
                    cell_float.set_property_xalign(1.0);
                    cell_float.set_property_placeholder_text(Some("Float (2.54, 3.21, 6.8765,..)"));
                    let column_float = TreeViewColumn::new();
                    column_float.set_title(&field_name);
                    column_float.set_clickable(true);
                    column_float.set_resizable(true);
                    column_float.set_min_width(50);
                    column_float.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                    column_float.set_alignment(0.5);
                    column_float.set_sort_column_id(index);
                    column_float.pack_start(&cell_float, true);
                    column_float.add_attribute(&cell_float, "text", index);
                    packed_file_tree_view.append_column(&column_float);
                    packed_file_tree_view_cell_float.push(cell_float);
                    if field.field_is_key {
                        key_columns.push(column_float);
                    }
                }
                FieldType::Integer => {
                    let cell_int = CellRendererText::new();
                    cell_int.set_property_editable(true);
                    cell_int.set_property_xalign(1.0);
                    cell_int.set_property_placeholder_text(Some("Integer (2, 3, 6,..)"));
                    let column_int = TreeViewColumn::new();
                    column_int.set_title(&field_name);
                    column_int.set_clickable(true);
                    column_int.set_resizable(true);
                    column_int.set_min_width(50);
                    column_int.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                    column_int.set_alignment(0.5);
                    column_int.set_sort_column_id(index);
                    column_int.pack_start(&cell_int, true);
                    column_int.add_attribute(&cell_int, "text", index);
                    packed_file_tree_view.append_column(&column_int);
                    packed_file_tree_view_cell_integer.push(cell_int);
                    if field.field_is_key {
                        key_columns.push(column_int);
                    }
                }
                FieldType::LongInteger => {
                    let cell_long_int = CellRendererText::new();
                    cell_long_int.set_property_editable(true);
                    cell_long_int.set_property_xalign(1.0);
                    cell_long_int.set_property_placeholder_text(Some("Long Integer (2, 3, 6,..)"));
                    let column_long_int = TreeViewColumn::new();
                    column_long_int.set_title(&field_name);
                    column_long_int.set_clickable(true);
                    column_long_int.set_resizable(true);
                    column_long_int.set_min_width(50);
                    column_long_int.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                    column_long_int.set_alignment(0.5);
                    column_long_int.set_sort_column_id(index);
                    column_long_int.pack_start(&cell_long_int, true);
                    column_long_int.add_attribute(&cell_long_int, "text", index);
                    packed_file_tree_view.append_column(&column_long_int);
                    packed_file_tree_view_cell_long_integer.push(cell_long_int);
                    if field.field_is_key {
                        key_columns.push(column_long_int);
                    }
                }
                FieldType::StringU8 | FieldType::StringU16 => {

                    // Check for references.
                    match field.field_is_reference {

                        // If it's a reference, use a combo with all unique values of it's original column.
                        Some(ref origin) => {
                            let mut origin_combo_data = vec![];

                            // If we have a database to check for refs...
                            if let Some(ref dependency_database) = dependency_database {

                                // For each table in the database...
                                for table in dependency_database {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1.to_owned() {

                                                    // Get it's position + 1 to compensate for the index.
                                                    for row in &db.packed_file_data.packed_file_data {
                                                        match row[index + 1] {
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
                                for table in local_dependency_database {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1.to_owned() {

                                                    // Get it's position + 1 to compensate for the index.
                                                    for row in &db.packed_file_data.packed_file_data {
                                                        match row[index + 1] {
                                                            DecodedData::StringU8(ref data) | DecodedData::StringU16(ref data) => {

                                                                // If the data is not already in the combo, we add it.
                                                                let mut exists = false;
                                                                for i in &origin_combo_data {
                                                                    if i == data {
                                                                        exists = true;
                                                                        break;
                                                                    }
                                                                }
                                                                if !exists {
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

                                let cell_string_list_store = ListStore::new(&[String::static_type()]);
                                for row in &origin_combo_data {
                                    cell_string_list_store.insert_with_values(None, &[0], &[&row]);
                                }

                                let column_string = TreeViewColumn::new();
                                let cell_string = CellRendererCombo::new();
                                cell_string.set_property_editable(true);
                                cell_string.set_property_model(Some(&cell_string_list_store));
                                cell_string.set_property_text_column(0);
                                column_string.set_title(&field_name);
                                column_string.set_clickable(true);
                                column_string.set_resizable(true);
                                column_string.set_min_width(50);
                                column_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                                column_string.set_alignment(0.5);
                                column_string.set_sort_column_id(index);
                                column_string.pack_start(&cell_string, true);
                                column_string.add_attribute(&cell_string, "text", index);
                                packed_file_tree_view.append_column(&column_string);
                                packed_file_tree_view_cell_reference.push(cell_string);
                                if field.field_is_key {
                                    key_columns.push(column_string);
                                }
                            }

                            // Otherwise, we fallback to the usual method.
                            else {
                                let cell_string = CellRendererText::new();
                                cell_string.set_property_editable(true);
                                cell_string.set_property_placeholder_text(Some("Obligatory String"));
                                let column_string = TreeViewColumn::new();
                                column_string.set_title(&field_name);
                                column_string.set_clickable(true);
                                column_string.set_resizable(true);
                                column_string.set_min_width(50);
                                column_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                                column_string.set_alignment(0.5);
                                column_string.set_sort_column_id(index);
                                column_string.pack_start(&cell_string, true);
                                column_string.add_attribute(&cell_string, "text", index);
                                packed_file_tree_view.append_column(&column_string);
                                packed_file_tree_view_cell_string.push(cell_string);
                                if field.field_is_key {
                                    key_columns.push(column_string);
                                }
                            }
                        },

                        // If it's not a reference, keep the normal behavior.
                        None => {
                            let cell_string = CellRendererText::new();
                            cell_string.set_property_editable(true);
                            cell_string.set_property_placeholder_text(Some("Obligatory String"));
                            let column_string = TreeViewColumn::new();
                            column_string.set_title(&field_name);
                            column_string.set_clickable(true);
                            column_string.set_resizable(true);
                            column_string.set_min_width(50);
                            column_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                            column_string.set_alignment(0.5);
                            column_string.set_sort_column_id(index);
                            column_string.pack_start(&cell_string, true);
                            column_string.add_attribute(&cell_string, "text", index);
                            packed_file_tree_view.append_column(&column_string);
                            packed_file_tree_view_cell_string.push(cell_string);
                            if field.field_is_key {
                                key_columns.push(column_string);
                            }
                        }
                    }
                }
                FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {

                    // Check for references.
                    match field.field_is_reference {

                        // If it's a reference, use a combo with all unique values of it's original column.
                        Some(ref origin) => {
                            let mut origin_combo_data = vec![];

                            // If we have a database to check for refs...
                            if let Some(ref dependency_database) = dependency_database {

                                // For each table in the database...
                                for table in dependency_database {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {

                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1.to_owned() {

                                                    // Get it's position + 1 to compensate for the index.
                                                    for row in &db.packed_file_data.packed_file_data {
                                                        match row[index + 1] {
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
                                for table in local_dependency_database {

                                    // If it's our original table...
                                    if table.packed_file_path[1] == format!("{}_tables", origin.0) {
                                        if let Ok(db) = DB::read(&table.packed_file_data, &*table.packed_file_path[1], master_schema) {

                                            // For each column in our original table...
                                            for (index, original_field) in db.packed_file_data.table_definition.fields.iter().enumerate() {

                                                // If it's our column...
                                                if original_field.field_name == origin.1.to_owned() {

                                                    // Get it's position + 1 to compensate for the index.
                                                    for row in &db.packed_file_data.packed_file_data {
                                                        match row[index + 1] {
                                                            DecodedData::StringU8(ref data) | DecodedData::StringU16(ref data) => {

                                                                // If the data is not already in the combo, we add it.
                                                                let mut exists = false;
                                                                for i in &origin_combo_data {
                                                                    if i == data {
                                                                        exists = true;
                                                                        break;
                                                                    }
                                                                }
                                                                if !exists {
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

                                let cell_string_list_store = ListStore::new(&[String::static_type()]);
                                for row in &origin_combo_data {
                                    cell_string_list_store.insert_with_values(None, &[0], &[&row]);
                                }

                                let column_optional_string = TreeViewColumn::new();
                                let cell_optional_string = CellRendererCombo::new();
                                cell_optional_string.set_property_editable(true);
                                cell_optional_string.set_property_model(Some(&cell_string_list_store));
                                cell_optional_string.set_property_text_column(0);
                                column_optional_string.set_title(&field_name);
                                column_optional_string.set_clickable(true);
                                column_optional_string.set_resizable(true);
                                column_optional_string.set_min_width(50);
                                column_optional_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                                column_optional_string.set_alignment(0.5);
                                column_optional_string.set_sort_column_id(index);
                                column_optional_string.pack_start(&cell_optional_string, true);
                                column_optional_string.add_attribute(&cell_optional_string, "text", index);
                                packed_file_tree_view.append_column(&column_optional_string);
                                packed_file_tree_view_cell_reference.push(cell_optional_string);
                                if field.field_is_key {
                                    key_columns.push(column_optional_string);
                                }
                            }

                            // Otherwise, we fallback to the usual method.
                            else {
                                let cell_optional_string = CellRendererText::new();
                                cell_optional_string.set_property_editable(true);
                                cell_optional_string.set_property_placeholder_text(Some("Optional String"));
                                let column_optional_string = TreeViewColumn::new();
                                column_optional_string.set_title(&field_name);
                                column_optional_string.set_clickable(true);
                                column_optional_string.set_resizable(true);
                                column_optional_string.set_min_width(50);
                                column_optional_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                                column_optional_string.set_alignment(0.5);
                                column_optional_string.set_sort_column_id(index);
                                column_optional_string.pack_start(&cell_optional_string, true);
                                column_optional_string.add_attribute(&cell_optional_string, "text", index);
                                packed_file_tree_view.append_column(&column_optional_string);
                                packed_file_tree_view_cell_optional_string.push(cell_optional_string);
                                if field.field_is_key {
                                    key_columns.push(column_optional_string);
                                }
                            }
                        },

                        // If it's not a reference, keep the normal behavior.
                        None => {
                            let cell_optional_string = CellRendererText::new();
                            cell_optional_string.set_property_editable(true);
                            cell_optional_string.set_property_placeholder_text(Some("Optional String"));
                            let column_optional_string = TreeViewColumn::new();
                            column_optional_string.set_title(&field_name);
                            column_optional_string.set_clickable(true);
                            column_optional_string.set_resizable(true);
                            column_optional_string.set_min_width(50);
                            column_optional_string.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
                            column_optional_string.set_alignment(0.5);
                            column_optional_string.set_sort_column_id(index);
                            column_optional_string.pack_start(&cell_optional_string, true);
                            column_optional_string.add_attribute(&cell_optional_string, "text", index);
                            packed_file_tree_view.append_column(&column_optional_string);
                            packed_file_tree_view_cell_optional_string.push(cell_optional_string);
                            if field.field_is_key {
                                key_columns.push(column_optional_string);
                            }
                        }
                    }
                }
            }
            index += 1;
        }

        // This column is to make the last column not go to the end of the table.
        let cell_fill = CellRendererText::new();
        let column_fill = TreeViewColumn::new();
        column_fill.set_min_width(0);
        column_fill.set_sizing(gtk::TreeViewColumnSizing::GrowOnly);
        column_fill.set_alignment(0.5);
        column_fill.set_sort_column_id(index);
        column_fill.pack_start(&cell_fill, true);
        packed_file_tree_view.append_column(&column_fill);

        // This should put the key columns in order.
        for column in key_columns.iter().rev() {
            packed_file_tree_view.move_column_after(column, Some(&column_index));
        }

        // Disabled search. Not sure why I disabled it, but until all the decoding/encoding stuff is
        // done, better keep it disable so it doesn't interfere with the events.
        packed_file_tree_view.set_enable_search(false);

        // Here we create the Popover menu. It's created and destroyed with the table because otherwise
        // it'll start crashing when changing tables and trying to delete stuff. Stupid menu.
        let packed_file_popover_menu = Popover::new(&packed_file_tree_view);

        let packed_file_popover_menu_box = Box::new(Orientation::Vertical, 0);
        packed_file_popover_menu_box.set_border_width(6);

        let packed_file_popover_menu_box_add_rows_box = Box::new(Orientation::Horizontal, 0);

        let packed_file_popover_menu_add_rows_button = ModelButton::new();
        packed_file_popover_menu_add_rows_button.set_property_text(Some("Add rows:"));
        packed_file_popover_menu_add_rows_button.set_action_name("app.packedfile_db_add_rows");

        let packed_file_popover_menu_add_rows_entry = Entry::new();
        let packed_file_popover_menu_add_rows_entry_buffer = packed_file_popover_menu_add_rows_entry.get_buffer();
        packed_file_popover_menu_add_rows_entry.set_alignment(1.0);
        packed_file_popover_menu_add_rows_entry.set_width_chars(8);
        packed_file_popover_menu_add_rows_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, "go-last");
        packed_file_popover_menu_add_rows_entry.set_has_frame(false);
        packed_file_popover_menu_add_rows_entry_buffer.set_max_length(Some(4));
        packed_file_popover_menu_add_rows_entry_buffer.set_text("1");

        let packed_file_popover_menu_delete_rows_button = ModelButton::new();
        packed_file_popover_menu_delete_rows_button.set_property_text(Some("Delete row/s"));
        packed_file_popover_menu_delete_rows_button.set_action_name("app.packedfile_db_delete_rows");

        let packed_file_popover_menu_clone_rows_button = ModelButton::new();
        packed_file_popover_menu_clone_rows_button.set_property_text(Some("Clone row/s"));
        packed_file_popover_menu_clone_rows_button.set_action_name("app.packedfile_db_clone_rows");

        let separator = Separator::new(Orientation::Vertical);
        let packed_file_popover_menu_import_from_csv_button = ModelButton::new();
        packed_file_popover_menu_import_from_csv_button.set_property_text(Some("Import from CSV"));
        packed_file_popover_menu_import_from_csv_button.set_action_name("app.packedfile_db_import_csv");

        let packed_file_popover_menu_export_to_csv_button = ModelButton::new();
        packed_file_popover_menu_export_to_csv_button.set_property_text(Some("Export to CSV"));
        packed_file_popover_menu_export_to_csv_button.set_action_name("app.packedfile_db_export_csv");

        packed_file_popover_menu_box_add_rows_box.pack_start(&packed_file_popover_menu_add_rows_button, true, true, 0);
        packed_file_popover_menu_box_add_rows_box.pack_end(&packed_file_popover_menu_add_rows_entry, true, true, 0);

        packed_file_popover_menu_box.pack_start(&packed_file_popover_menu_box_add_rows_box, true, true, 0);
        packed_file_popover_menu_box.pack_start(&packed_file_popover_menu_delete_rows_button, true, true, 0);
        packed_file_popover_menu_box.pack_start(&packed_file_popover_menu_clone_rows_button, true, true, 0);

        packed_file_popover_menu_box.pack_start(&separator, true, true, 0);
        packed_file_popover_menu_box.pack_start(&packed_file_popover_menu_import_from_csv_button, true, true, 0);
        packed_file_popover_menu_box.pack_start(&packed_file_popover_menu_export_to_csv_button, true, true, 0);

        packed_file_popover_menu.add(&packed_file_popover_menu_box);
        packed_file_popover_menu.show_all();

        let packed_file_data_scroll = ScrolledWindow::new(None, None);
        packed_file_tree_view.set_hexpand(true);
        packed_file_tree_view.set_vexpand(true);

        packed_file_data_scroll.add(&packed_file_tree_view);
        packed_file_data_display.attach(&packed_file_data_scroll, 0, 1, 1, 1);

        packed_file_data_display.show_all();

        // We hide the popover by default.
        packed_file_popover_menu.hide();

        Ok(PackedFileDBTreeView {
            packed_file_popover_menu,
            packed_file_popover_menu_add_rows_entry,
            packed_file_tree_view,
            packed_file_list_store,
            packed_file_tree_view_cell_bool,
            packed_file_tree_view_cell_float,
            packed_file_tree_view_cell_integer,
            packed_file_tree_view_cell_long_integer,
            packed_file_tree_view_cell_string,
            packed_file_tree_view_cell_optional_string,
            packed_file_tree_view_cell_reference,
        })
    }

    /// This function decodes the data of a DB PackedFile and loads it into a TreeView.
    pub fn load_data_to_tree_view(
        packed_file_data: &DBData,
        packed_file_list_store: &ListStore,
    ) -> Result<(), Error>{

        // First, we delete all the data from the ListStore.
        packed_file_list_store.clear();

        // Then we add every line to the ListStore.
        for row in &packed_file_data.packed_file_data {

            // Due to issues with types and gtk-rs, we need to create an empty line and then add the
            // values to it, one by one.
            let current_row = packed_file_list_store.append();

            for (index, field) in row.iter().enumerate() {
                let gtk_value_field;
                match *field {
                    DecodedData::Boolean(ref data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Float(ref data) => gtk_value_field = gtk::ToValue::to_value(&format!("{}", data)),
                    DecodedData::Integer(ref data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::LongInteger(ref data) => gtk_value_field = gtk::ToValue::to_value(&data),

                    // All these are Strings, so it can be together,
                    DecodedData::Index(ref data) |
                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => gtk_value_field = gtk::ToValue::to_value(&data),
                }
                packed_file_list_store.set_value(&current_row, index as u32, &gtk_value_field);
            }
        }
        Ok(())
    }

    /// This function returns a Vec<DataDecoded> with all the stuff in the table. We need for it the
    /// ListStore, and it'll return a Vec<DataDecoded> with all the stuff from the table.
    pub fn return_data_from_tree_view(
        table_definition: &TableDefinition,
        packed_file_list_store: &ListStore,
    ) -> Result<Vec<Vec<::packedfile::db::DecodedData>>, Error> {

        let mut packed_file_data_from_tree_view: Vec<Vec<DecodedData>> = vec![];

        // Only in case we have any line in the ListStore we try to get it. Otherwise we return an
        // empty vector.
        if let Some(current_line) = packed_file_list_store.get_iter_first() {
            let columns = packed_file_list_store.get_n_columns();

            // Foreach row in the DB PackedFile.
            let mut done = false;
            while !done {

                // We return the index too. We deal with it in the save function, so there is no problem
                let mut packed_file_data_from_tree_view_entry: Vec<DecodedData> = vec![DecodedData::Index(packed_file_list_store.get_value(&current_line, 0).get().unwrap())];

                for column in 1..columns {
                    let field_type = &table_definition.fields[column as usize - 1].field_type;
                    match *field_type {
                        FieldType::Boolean => {
                            let data: bool = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Boolean(data));
                        }
                        FieldType::Float => {
                            let data: f32 = packed_file_list_store.get_value(&current_line, column).get::<String>().unwrap().parse::<f32>().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Float(data));
                        }
                        FieldType::Integer => {
                            let data: i32 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Integer(data));
                        }
                        FieldType::LongInteger => {
                            let data: i64 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::LongInteger(data));
                        }
                        FieldType::StringU8 => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::StringU8(data));
                        }
                        FieldType::StringU16 => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::StringU16(data));
                        }
                        FieldType::OptionalStringU8 => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::OptionalStringU8(data));
                        }
                        FieldType::OptionalStringU16 => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::OptionalStringU16(data));
                        }
                    }
                }
                packed_file_data_from_tree_view.push(packed_file_data_from_tree_view_entry);

                if !packed_file_list_store.iter_next(&current_line) {
                    done = true;
                }
            }
        }
        Ok(packed_file_data_from_tree_view)
    }
}

impl PackedFileDBDecoder {

    /// This function creates the "Decoder View" with all the stuff needed to decode a table, and it
    /// returns it.
    pub fn create_decoder_view(packed_file_data_display: &Grid) -> PackedFileDBDecoder {

        // With this we create the "Decoder View", under the "Enable decoding mode" button.
        let decoder_grid_scroll = ScrolledWindow::new(None, None);
        let decoder_grid = Grid::new();
        decoder_grid.set_border_width(6);
        decoder_grid.set_row_spacing(6);
        decoder_grid.set_column_spacing(3);

        // In the left side, there should be a Grid with the hex data.
        let raw_data_grid = Grid::new();
        let raw_data_index = Label::new(None);
        let raw_data_hex = TextView::new();
        let raw_data_decoded = TextView::new();

        // Config for the "Raw Data" stuff.
        raw_data_grid.set_border_width(6);
        raw_data_grid.set_row_spacing(6);
        raw_data_grid.set_column_spacing(3);

        raw_data_index.set_vexpand(true);
        raw_data_index.set_xalign(1.0);
        raw_data_index.set_yalign(0.0);

        // These two shouldn't be editables.
        raw_data_hex.set_editable(false);
        raw_data_decoded.set_editable(false);

        // Set the fonts of the labels to `monospace`, so we see them properly aligned.
        let raw_data_index_style = raw_data_index.get_style_context().unwrap();
        let raw_data_hex_style = raw_data_hex.get_style_context().unwrap();
        let raw_data_decoded_style = raw_data_decoded.get_style_context().unwrap();
        let raw_data_monospace_css = ".monospace-font { font-family: \"Courier New\", Courier, monospace } .monospace-font-bold { font-family: \"Courier New\", Courier, monospace; font-weight: bold; }".as_bytes();

        let css_provider = CssProvider::new();

        css_provider.load_from_data(raw_data_monospace_css).unwrap();

        raw_data_index_style.add_provider(&css_provider, 99);
        raw_data_hex_style.add_provider(&css_provider, 99);
        raw_data_decoded_style.add_provider(&css_provider, 99);

        StyleContext::add_class(&raw_data_index_style, "monospace-font-bold");
        StyleContext::add_class(&raw_data_hex_style, "monospace-font");
        StyleContext::add_class(&raw_data_decoded_style, "monospace-font");

        // Create the color tags for the Raw Data...
        create_text_tags(&raw_data_hex);
        create_text_tags(&raw_data_decoded);

        // In the right side, there should be a Vertical Paned, with a grid on the top, and another
        // on the bottom.
        let decoded_data_paned = Paned::new(Orientation::Vertical);
        let decoded_data_paned_top_grid = Grid::new();
        let decoded_data_paned_bottom_grid = Grid::new();

        decoded_data_paned.set_position(500);
        decoded_data_paned_top_grid.set_border_width(6);
        decoded_data_paned_top_grid.set_row_spacing(6);
        decoded_data_paned_top_grid.set_column_spacing(3);
        decoded_data_paned_bottom_grid.set_border_width(6);
        decoded_data_paned_bottom_grid.set_row_spacing(6);
        decoded_data_paned_bottom_grid.set_column_spacing(3);

        // In the top grid, there should be a column with two buttons, and another with a ScrolledWindow,
        // with a TreeView inside.

        // Here we create the buttons to move the decoded rows up&down.
        let row_up = ModelButton::new();
        let row_down = ModelButton::new();

        row_up.set_property_text(Some("Up"));
        row_up.set_action_name("app.move_row_up");
        row_down.set_property_text(Some("Down"));
        row_down.set_action_name("app.move_row_down");

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

        let table_info_type_label = Label::new("Table type:");
        let table_info_version_label = Label::new("Table version:");
        let table_info_entry_count_label = Label::new("Table entry count:");

        table_info_type_label.set_xalign(0.0);
        table_info_type_label.set_yalign(0.5);
        table_info_version_label.set_xalign(0.0);
        table_info_version_label.set_yalign(0.5);
        table_info_entry_count_label.set_xalign(0.0);
        table_info_entry_count_label.set_yalign(0.5);

        let table_type_decoded_label = Label::new(None);
        let table_version_decoded_label = Label::new(None);
        let table_entry_count_decoded_label = Label::new(None);

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

        // Here we create a little TreeView with all the versions of this table we have, in case we
        // want to decode it based on another version's definition, to save time.
        let all_table_versions_tree_view = TreeView::new();
        let all_table_versions_list_store = ListStore::new(&[u32::static_type()]);
        all_table_versions_tree_view.set_model(Some(&all_table_versions_list_store));

        let all_table_versions_tree_view_scroll = ScrolledWindow::new(None, None);
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

        let load_definition = Button::new_with_label("Load");
        let remove_definition = Button::new_with_label("Remove");

        button_box_definition.pack_start(&load_definition, false, false, 0);
        button_box_definition.pack_start(&remove_definition, false, false, 0);

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
        decoded_data_paned.pack1(&decoded_data_paned_top_grid, false, false);
        decoded_data_paned.pack2(&decoded_data_paned_bottom_grid, false, false);

        decoder_grid.attach(&decoded_data_paned, 1, 0, 1, 1);

        // Packing into the top side of the right paned...
        decoded_data_paned_top_grid.attach(&row_up, 0, 0 ,1 ,1);
        decoded_data_paned_top_grid.attach(&row_down, 0, 1 ,1 ,1);

        fields_tree_view_scroll.add(&fields_tree_view);
        decoded_data_paned_top_grid.attach(&fields_tree_view_scroll, 1, 0 ,1 ,2);

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

        general_info_grid.attach(&all_table_versions_tree_view_scroll, 0, 3, 2, 1);
        general_info_grid.attach(&button_box_definition, 0, 4, 2, 1);

        decoded_data_paned_bottom_grid.attach(&general_info_grid, 1, 0, 1, 10);

        // Bottom of the bottom grid...
        decoded_data_paned_bottom_grid.attach(&bottom_box, 0, 1, 2, 1);

        // Packing into the decoder grid...
        decoder_grid_scroll.add(&decoder_grid);
        packed_file_data_display.attach(&decoder_grid_scroll, 0, 1, 1, 1);
        packed_file_data_display.show_all();

        PackedFileDBDecoder {
            data_initial_index: 0i32,
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
            move_up_button: row_up,
            move_down_button: row_down,
        }
    }

    /// This function loads the data from the selected table into the "Decoder View".
    pub fn load_data_to_decoder_view(
        packed_file_decoder_view: &mut PackedFileDBDecoder,
        packed_file_table_type: &str,
        packed_file_encoded: &[u8],
        data_initial_index: usize
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
        packed_file_decoder_view.raw_data_line_index.set_text(&hex_lines_text);

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

            let raw_data_hex_buffer = packed_file_decoder_view.raw_data.get_buffer().unwrap();
            raw_data_hex_buffer.set_text(&hex_raw_data_string);

            // In theory, this should give us the equivalent byte to our index_data.
            // In practice, I'm bad at maths.
            let header_line = (data_initial_index * 3 / 48) as i32;
            let header_char = (data_initial_index * 3 % 48) as i32;
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

            let header_line = (data_initial_index / 16) as i32;
            let header_char = (data_initial_index % 16) as i32;

            let raw_data_decoded_buffer = packed_file_decoder_view.raw_data_decoded.get_buffer().unwrap();
            raw_data_decoded_buffer.set_text(&hex_raw_data_decoded);
            raw_data_decoded_buffer.apply_tag_by_name(
                "header",
                &raw_data_decoded_buffer.get_start_iter(),
                &raw_data_decoded_buffer.get_iter_at_line_offset(header_line, header_char)
            );
        }

        packed_file_decoder_view.table_type_label.set_text(packed_file_table_type);
        packed_file_decoder_view.table_version_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_version));
        packed_file_decoder_view.table_entry_count_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_entry_count));

        // Save the initial index, for future uses.
        packed_file_decoder_view.data_initial_index = data_initial_index as i32;
        Ok(())
    }

    /// This function updates the data shown in the "Decoder" box when we execute it. It requires:
    /// - packed_file_decoder: &PackedFileDBDecoder, the decoder object.
    /// - packed_file_decoded: Vec<u8>, PackedFile's Data to decode.
    /// - table_definition: Option<&TableDefinition>, a ref to the table definitions. None will
    ///   skip the load of data to the table.
    /// - index_data: usize, the index where to start decoding.
    /// - load_from_existing_definition: bool, if true, then we load the data from a definition.
    ///   If false, we update the entire table. If false, we just update the text entries.
    pub fn update_decoder_view(
        packed_file_decoder: &PackedFileDBDecoder,
        packed_file_decoded: &[u8],
        table_definition: Option<&TableDefinition>,
        index_data: usize,
    ) -> usize {

        // We need to get the length of the vector first, to avoid crashes due to non-existant indexes
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        let mut index_data = index_data;

        // If we are loading data to the table for the first time, we'll load to the table all the data
        // directly from the existing definition and update the initial index for decoding.
        if let Some(table_definition) = table_definition {
            for (index, field) in table_definition.fields.iter().enumerate() {
                index_data = PackedFileDBDecoder::add_field_to_data_view(
                    packed_file_decoder,
                    packed_file_decoded,
                    table_definition,
                    &field.field_name,
                    field.field_type.to_owned(),
                    field.field_is_key,
                    &field.field_is_reference,
                    &field.field_description,
                    index_data,
                    Some(index)
                );
            }
        }

        // Check if the index does even exist, to avoid crashes.
        if index_data < packed_file_decoded.len() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(
                packed_file_decoded[index_data],
                index_data
            ) {
                Ok(data) => {
                    if data.0 {
                        "True"
                    }
                    else {
                        "False"
                    }
                },
                Err(_) => "Error"
            };
        }
        else {
            decoded_bool = "Error";
        }

        // Check if the index does even exist, to avoid crashes.
        if (index_data + 4) <= packed_file_decoded.len() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(
                &packed_file_decoded[index_data..(index_data + 4)],
                index_data
            ) {
                Ok(data) => data.0.to_string(),
                Err(_) => "Error".to_string()
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(
                &packed_file_decoded[index_data..(index_data + 4)],
                index_data
            ) {
                Ok(data) => data.0.to_string(),
                Err(_) => "Error".to_string()
            };
        }
        else {
            decoded_float = "Error".to_string();
            decoded_integer = "Error".to_string();
        }

        // Check if the index does even exist, to avoid crashes.
        if (index_data + 8) <= packed_file_decoded.len() {
            decoded_long_integer = match coding_helpers::decode_packedfile_integer_i64(
                &packed_file_decoded[index_data..(index_data + 8)],
                index_data
            ) {
                Ok(data) => data.0.to_string(),
                Err(_) => "Error".to_string()
            };
        }
        else {
            decoded_long_integer = "Error".to_string();
        }

        // Check that the index exist, to avoid crashes.
        if index_data < packed_file_decoded.len() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(
                &packed_file_decoded[index_data..],
                index_data
            ) {
                Ok(data) => data.0,
                Err(_) => "Error".to_string()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(
                &packed_file_decoded[index_data..],
                index_data
            ) {
                Ok(data) => data.0,
                Err(_) => "Error".to_string()
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(
                &packed_file_decoded[index_data..],
                index_data
            ) {
                Ok(data) => data.0,
                Err(_) => "Error".to_string()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(
                &packed_file_decoded[index_data..],
                index_data
            ) {
                Ok(data) => data.0,
                Err(_) => "Error".to_string()
            };
        }
        else {
            decoded_string_u8 = "Error".to_string();
            decoded_string_u16 = "Error".to_string();
            decoded_optional_string_u8 = "Error".to_string();
            decoded_optional_string_u16 = "Error".to_string();
        }

        // We update all the decoded entries here.
        packed_file_decoder.bool_entry.get_buffer().set_text(decoded_bool);
        packed_file_decoder.float_entry.get_buffer().set_text(&*decoded_float);
        packed_file_decoder.integer_entry.get_buffer().set_text(&*decoded_integer);
        packed_file_decoder.long_integer_entry.get_buffer().set_text(&*decoded_long_integer);
        packed_file_decoder.string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u8));
        packed_file_decoder.string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u16));
        packed_file_decoder.optional_string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u8));
        packed_file_decoder.optional_string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u16));

        // We reset these two every time we add a field.
        packed_file_decoder.field_name_entry.get_buffer().set_text(&format!("Unknown {}", index_data));
        packed_file_decoder.is_key_field_switch.set_state(false);

        // Then we set the TextTags to paint the hex_data.
        let raw_data_hex_text_buffer = packed_file_decoder.raw_data.get_buffer().unwrap();

        // Clear the current index tag.
        raw_data_hex_text_buffer.remove_tag_by_name("index", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());
        raw_data_hex_text_buffer.remove_tag_by_name("entry", &raw_data_hex_text_buffer.get_start_iter(), &raw_data_hex_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (index_data * 3 / 48) as i32;
        let index_line_end = (((index_data * 3) + 2) / 48) as i32;
        let index_char_start = ((index_data * 3) % 48) as i32;
        let index_char_end = (((index_data * 3) + 2) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = ((packed_file_decoder.data_initial_index * 3) / 48) as i32;
        let header_char = ((packed_file_decoder.data_initial_index * 3) % 48) as i32;
        let index_line_end = ((index_data * 3) / 48) as i32;
        let index_char_end = ((index_data * 3) % 48) as i32;
        raw_data_hex_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_hex_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_hex_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // And then, we do the same for `raw_decoded_data`.
        let raw_data_decoded_text_buffer = packed_file_decoder.raw_data_decoded.get_buffer().unwrap();

        // Clear the current index and entry tags.
        raw_data_decoded_text_buffer.remove_tag_by_name("index", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());
        raw_data_decoded_text_buffer.remove_tag_by_name("entry", &raw_data_decoded_text_buffer.get_start_iter(), &raw_data_decoded_text_buffer.get_end_iter());

        // Set a new index tag.
        let index_line_start = (index_data / 16) as i32;
        let index_line_end = ((index_data + 1) / 16) as i32;
        let index_char_start = (index_data % 16) as i32;
        let index_char_end = ((index_data + 1) % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "index",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_start, index_char_start),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Then, we paint the currently decoded entry. Just to look cool.
        let header_line = (packed_file_decoder.data_initial_index / 16) as i32;
        let header_char = (packed_file_decoder.data_initial_index % 16) as i32;
        let index_line_end = (index_data / 16) as i32;
        let index_char_end = (index_data % 16) as i32;
        raw_data_decoded_text_buffer.apply_tag_by_name(
            "entry",
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(header_line, header_char),
            &raw_data_decoded_text_buffer.get_iter_at_line_offset(index_line_end, index_char_end)
        );

        // Returns the new "index_data" to keep decoding.
        index_data
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    pub fn update_versions_list(
        packed_file_decoder: &PackedFileDBDecoder,
        schema: &Schema,
        table_name: &str,
    ) {
        // Clear the current list.
        packed_file_decoder.all_table_versions_list_store.clear();

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Some(table_versions_list) = DB::get_schema_versions_list(table_name, &schema) {
            for version in table_versions_list {
                packed_file_decoder.all_table_versions_list_store.insert_with_values(None, &[0], &[&version.version]);
            }
        }
    }

    /// This function adds fields to the "Decoder" table, so we can do this without depending on the
    /// updates of the Decoder view. As this has a lot of required data, lets's explain:
    /// - packed_file_decoder: a PackedFileDBDecoder reference, so we can update the field list.
    /// - packed_file_decoded: the data to decode.
    /// - table_definition: needed to get the "index" number properly.
    /// - field_name: the name of the field.
    /// - field_type: the type of the field.
    /// - field_is_key: if the field is key or not.
    /// - field_is_reference: the reference data of the field, if it's a reference to another field.
    /// - field_description: the description of the field. If the field is new, this is just String::new().
    /// - index_data: the index to start decoding from the vector.
    ///
    /// We return the index of the next field in the data.
    /// NOTE: In case of error, we return the same index, NOT AN ERROR. That way, we deal with the
    /// possible error here instead on the UI.
    pub fn add_field_to_data_view(
        packed_file_decoder: &PackedFileDBDecoder,
        packed_file_decoded: &[u8],
        table_definition: &TableDefinition,
        field_name: &str,
        field_type: FieldType,
        field_is_key: bool,
        field_is_reference: &Option<(String, String)>,
        field_description: &str,
        index_data: usize,
        index_row: Option<usize>
    ) -> usize {

        let field_index = match index_row {
            Some(index) => format!("{:0count$}", index + 1, count = (table_definition.fields.len().to_string().len() + 1)),
            None => "New".to_owned(),
        };

        let decoded_data = decode_data_by_fieldtype(
            packed_file_decoded,
            &field_type,
            index_data
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
            packed_file_decoder.fields_list_store.insert_with_values(
                None,
                &[0, 1, 2, 3, 4, 5, 6, 7],
                &[
                    &field_index,
                    &field_name,
                    &field_type,
                    &field_is_key,
                    &reference.0,
                    &reference.1,
                    &decoded_data.0,
                    &field_description,
                ]
            );
        }
        else {
            packed_file_decoder.fields_list_store.insert_with_values(
                None,
                &[0, 1, 2, 3, 4, 5, 6, 7],
                &[
                    &field_index,
                    &field_name,
                    &field_type,
                    &field_is_key,
                    &String::new(),
                    &String::new(),
                    &decoded_data.0,
                    &field_description,
                ]
            );
        }

        // We return the updated index.
        decoded_data.1
    }

    /// This function gets the data from the "Decoder" table, and returns it, so we can save it in a
    /// TableDefinition.fields.
    pub fn return_data_from_data_view(&self) -> Vec<Field> {
        let mut fields = vec![];

        // Only in case we have any line in the ListStore we try to get it. Otherwise we return an
        // empty LocData.
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


/// This function is a helper to try to decode data in different formats, returning "Error" in case
/// of decoding error. It requires the FieldType we want to decode, the data we want to decode
/// (vec<u8>, being the first u8 the first byte to decode) and the index of the data in the Vec<u8>.
pub fn decode_data_by_fieldtype(field_data: &[u8], field_type: &FieldType, index_data: usize) -> (String, usize) {
    match *field_type {
        FieldType::Boolean => {
            // Check if the index does even exist, to avoid crashes.
            if field_data.get(index_data).is_some() {
                match coding_helpers::decode_packedfile_bool(field_data[index_data], index_data) {
                    Ok(result) => {
                        if result.0 {
                            ("True".to_string(), result.1)
                        }
                        else {
                            ("False".to_string(), result.1)
                        }
                    }
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::Float => {
            if field_data.get(index_data..(index_data + 4)).is_some() {
                match coding_helpers::decode_packedfile_float_f32(&field_data[index_data..(index_data + 4)], index_data) {
                    Ok(result) => (result.0.to_string(), result.1),
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::Integer => {
            if field_data.get(index_data..(index_data + 4)).is_some() {
                match coding_helpers::decode_packedfile_integer_i32(&field_data[index_data..(index_data +4)], index_data) {
                    Ok(result) => (result.0.to_string(), result.1),
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::LongInteger => {
            if field_data.get(index_data..(index_data + 8)).is_some() {
                match coding_helpers::decode_packedfile_integer_i64(&field_data[index_data..(index_data +8)], index_data) {
                    Ok(result) => (result.0.to_string(), result.1),
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::StringU8 => {
            if field_data.get(index_data).is_some() {
                match coding_helpers::decode_packedfile_string_u8(&field_data[index_data..], index_data) {
                    Ok(result) => result,
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::StringU16 => {
            if field_data.get(index_data).is_some() {
                match coding_helpers::decode_packedfile_string_u16(&field_data[index_data..], index_data) {
                    Ok(result) => result,
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::OptionalStringU8 => {
            if field_data.get(index_data).is_some() {
                match coding_helpers::decode_packedfile_optional_string_u8(&field_data[index_data..], index_data) {
                    Ok(result) => result,
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
        },
        FieldType::OptionalStringU16 => {
            if field_data.get(index_data).is_some() {
                match coding_helpers::decode_packedfile_optional_string_u16(&field_data[index_data..], index_data) {
                    Ok(result) => result,
                    Err(_) => ("Error".to_owned(), index_data),
                }
            }
            else {
                ("Error".to_owned(), index_data)
            }
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
