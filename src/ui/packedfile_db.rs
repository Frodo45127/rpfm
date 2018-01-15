// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate ordermap;
extern crate gtk;
extern crate glib;
extern crate hex_slice;
extern crate encoding;

use packedfile::db::*;
use packedfile::db::schemas::*;
use common::coding_helpers;

use std::io::{
    Error, ErrorKind
};
use gtk::prelude::*;
use gtk::{
    Box, TreeView, ListStore, ScrolledWindow, Button, Orientation, TextView, Label, Entry, ToggleButton,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type, WrapMode, Justification, TreeStore
};

use self::ordermap::OrderMap;
use self::hex_slice::AsHex;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::ISO_8859_1;

/// Struct PackedFileDBTreeView: contains all the stuff we need to give to the program to show a
/// TreeView with the data of a DB PackedFile, allowing us to manipulate it.
#[derive(Clone, Debug)]
pub struct PackedFileDBTreeView {
    pub packed_file_tree_view: TreeView,
    pub packed_file_list_store: ListStore,
    pub packed_file_tree_view_cell_bool: Vec<CellRendererToggle>,
    pub packed_file_tree_view_cell_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_optional_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_integer: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_float: Vec<CellRendererText>,
}

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
#[derive(Clone, Debug)]
pub struct PackedFileDBDecoder {
    pub raw_data_line_index: Label,
    pub raw_data: TextView,
    //pub raw_data_decoded: TextView,
    pub table_type_label: Label,
    pub table_version_label: Label,
    pub table_entry_count_label: Label,
    pub bool_entry: Entry,
    pub float_entry: Entry,
    pub integer_entry: Entry,
    pub string_u8_entry: Entry,
    pub string_u16_entry: Entry,
    pub optional_string_u8_entry: Entry,
    pub optional_string_u16_entry: Entry,
    pub use_bool_button: Button,
    pub use_float_button: Button,
    pub use_integer_button: Button,
    pub use_string_u8_button: Button,
    pub use_string_u16_button: Button,
    pub use_optional_string_u8_button: Button,
    pub use_optional_string_u16_button: Button,
    pub fields_list_store: ListStore,
    pub field_name_entry: Entry,
    pub is_key_field_button: ToggleButton,
    pub save_decoded_schema: Button,
}

/// Implementation of "PackedFileDBTreeView".
impl PackedFileDBTreeView{

    /// This function creates a new TreeView with "packed_file_data_display" as father and returns a
    /// PackedFileDBTreeView with all his data.
    pub fn create_tree_view(
        packed_file_data_display: &Box,
        packed_file_decoded: &::packedfile::db::DB
    ) -> Result<PackedFileDBTreeView, Error> {

        // First, we create the Vec<Type> we are going to use to create the TreeView, based on the structure
        // of the DB PackedFile.
        let mut list_store_table_definition: Vec<Type> = vec![];
        let packed_file_table_definition = packed_file_decoded.packed_file_data.table_definition.clone();

        // The first column is an index for us to know how many entries we have.
        list_store_table_definition.push(Type::String);

        // Depending on the type of the field, we push the gtk::Type equivalent to that column.
        for field in packed_file_table_definition.fields.iter() {
            match field.field_type {
                FieldType::Boolean => {
                    list_store_table_definition.push(Type::Bool);
                }
                FieldType::Float => {
                    list_store_table_definition.push(Type::F32);
                }
                FieldType::Integer => {
                    list_store_table_definition.push(Type::U32);
                }
                FieldType::StringU8 | FieldType::StringU16 | FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                    list_store_table_definition.push(Type::String);
                }
            }
        }

        // Then, we create the new ListStore, the new TreeView, and prepare the TreeView to display the data
        let packed_file_tree_view = TreeView::new();
        let packed_file_list_store = ListStore::new(&list_store_table_definition);

        packed_file_tree_view.set_model(Some(&packed_file_list_store));
        packed_file_tree_view.set_grid_lines(gtk::TreeViewGridLines::Both);

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
        let mut packed_file_tree_view_cell_string = vec![];
        let mut packed_file_tree_view_cell_optional_string = vec![];
        let mut packed_file_tree_view_cell_integer = vec![];
        let mut packed_file_tree_view_cell_float = vec![];

        let mut index = 1;
        let mut key_columns = vec![];
        for field in packed_file_table_definition.fields.iter() {

            // These are the specific declarations of the columns for every type implemented.
            match field.field_type {
                FieldType::Boolean => {
                    let cell_bool = CellRendererToggle::new();
                    cell_bool.set_activatable(true);
                    let column_bool = TreeViewColumn::new();
                    column_bool.set_title(&field.field_name);
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
                    column_float.set_title(&field.field_name);
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
                    column_int.set_title(&field.field_name);
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
                FieldType::StringU8 | FieldType::StringU16 => {
                    let cell_string = CellRendererText::new();
                    cell_string.set_property_editable(true);
                    cell_string.set_property_placeholder_text(Some("Obligatory String"));
                    let column_string = TreeViewColumn::new();
                    column_string.set_title(&field.field_name);
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
                FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                    let cell_optional_string = CellRendererText::new();
                    cell_optional_string.set_property_editable(true);
                    cell_optional_string.set_property_placeholder_text(Some("Optional String"));
                    let column_optional_string = TreeViewColumn::new();
                    column_optional_string.set_title(&field.field_name);
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
            index += 1;
        }

        // This should put the key columns in order.
        for column in key_columns.iter().rev() {
            packed_file_tree_view.move_column_after(&column, Some(&column_index));
        }

        // Disabled search. Not sure why I disabled it, but until all the decoding/encoding stuff is
        // done, better keep it disable so it doesn't interfere with the events.
        packed_file_tree_view.set_enable_search(false);

        let packed_file_data_scroll = ScrolledWindow::new(None, None);

        packed_file_data_scroll.add(&packed_file_tree_view);
        packed_file_data_display.pack_start(&packed_file_data_scroll, true, true, 0);
        packed_file_data_display.show_all();

        Ok(PackedFileDBTreeView {
            packed_file_tree_view,
            packed_file_list_store,
            packed_file_tree_view_cell_bool,
            packed_file_tree_view_cell_string,
            packed_file_tree_view_cell_optional_string,
            packed_file_tree_view_cell_integer,
            packed_file_tree_view_cell_float,
        })
    }

    /// This function decodes the data of a DB PackedFile and loads it into a TreeView.
    pub fn load_data_to_tree_view(
        packed_file_data: Vec<Vec<::packedfile::db::DecodedData>>,
        packed_file_list_store: &ListStore,
    ) -> Result<(), Error>{

        // First, we delete all the data from the ListStore.
        packed_file_list_store.clear();

        // Then we add every line to the ListStore.
        for row in packed_file_data {
            let mut index = 0;

            // Due to issues with types and gtk-rs, we need to create an empty line and then add the
            // values to it, one by one.
            let current_row = packed_file_list_store.append();

            for field in row {
                let gtk_value_field;
                match field {
                    DecodedData::Index(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Boolean(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Float(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Integer(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::StringU8(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::StringU16(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::OptionalStringU8(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::OptionalStringU16(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::RawData(_) => return Err(Error::new(ErrorKind::Other, format!("Error: trying to load RawData into a DB Table PackedFile."))),
                }
                packed_file_list_store.set_value(&current_row, index, &gtk_value_field);
                index += 1;
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

                let mut packed_file_data_from_tree_view_entry: Vec<DecodedData> = vec![];

                for column in 1..columns {
                    let field_type = &table_definition.fields[column as usize - 1].field_type;
                    match *field_type {
                        FieldType::Boolean => {
                            let data: bool = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Boolean(data));
                        }
                        FieldType::Float => {
                            let data: f32 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Float(data));
                        }
                        FieldType::Integer => {
                            let data: u32 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(DecodedData::Integer(data));
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

    /// This function creates the "Decoder" box with all the stuff needed to decode a table, and it
    /// returns that box.
    pub fn create_decoder_view(packed_file_data_display: &Box) -> PackedFileDBDecoder {
        // With this we create the "Decoder" box, under the DB Table.
        let decoder_box = Box::new(Orientation::Horizontal, 0);
        let decoder_box_scroll = ScrolledWindow::new(None, None);
        decoder_box_scroll.add(&decoder_box);
        packed_file_data_display.pack_end(&decoder_box_scroll, true, true, 0);

        // Then we create the TextView for the raw data of the DB PackedFile.
        let raw_data_box = Box::new(Orientation::Horizontal, 0);
        let raw_data_line_index = Label::new(None);
        let raw_data = TextView::new();
        //let raw_data_decoded = TextView::new();

        raw_data_line_index.set_alignment(1.0, 0.0);
        raw_data.set_justification(Justification::Fill);
        raw_data.set_size_request(280,0);
        raw_data.set_wrap_mode(WrapMode::Word);
        //raw_data_decoded.set_wrap_mode(WrapMode::Word);

        raw_data_box.pack_start(&raw_data_line_index, false, false, 4);
        raw_data_box.pack_start(&raw_data, false, false, 4);
        //raw_data_box.pack_start(&raw_data_decoded, false, false, 4);

        let packed_file_raw_data_scroll = ScrolledWindow::new(None, None);
        packed_file_raw_data_scroll.set_size_request(320, 0);
        //packed_file_raw_data_scroll.set_max_content_width(400);

        // Then, the big box to put all the stuff we need to decode.
        let packed_file_decoded_data_box = Box::new(Orientation::Vertical, 0);

        // Then, the box for all the labels, fields and buttons.
        let bool_box = Box::new(Orientation::Horizontal, 0);
        let float_box = Box::new(Orientation::Horizontal, 0);
        let integer_box = Box::new(Orientation::Horizontal, 0);
        let string_u8_box = Box::new(Orientation::Horizontal, 0);
        let string_u16_box = Box::new(Orientation::Horizontal, 0);
        let optional_string_u8_box = Box::new(Orientation::Horizontal, 0);
        let optional_string_u16_box = Box::new(Orientation::Horizontal, 0);

        let bool_label = Label::new(Some("Decoded as \"Bool\":"));
        let float_label = Label::new(Some("Decoded as \"Float\":"));
        let integer_label = Label::new(Some("Decoded as \"Integer\":"));
        let string_u8_label = Label::new(Some("Decoded as \"String u8\":"));
        let string_u16_label = Label::new(Some("Decoded as \"String u16\":"));
        let optional_string_u8_label = Label::new(Some("Decoded as \"Optional String u8\":"));
        let optional_string_u16_label = Label::new(Some("Decoded as \"Optional String u16\":"));

        bool_label.set_size_request(200, 0);
        float_label.set_size_request(200, 0);
        integer_label.set_size_request(200, 0);
        string_u8_label.set_size_request(200, 0);
        string_u16_label.set_size_request(200, 0);
        optional_string_u8_label.set_size_request(200, 0);
        optional_string_u16_label.set_size_request(200, 0);

        bool_label.set_alignment(0.0, 0.5);
        float_label.set_alignment(0.0, 0.5);
        integer_label.set_alignment(0.0, 0.5);
        string_u8_label.set_alignment(0.0, 0.5);
        string_u16_label.set_alignment(0.0, 0.5);
        optional_string_u8_label.set_alignment(0.0, 0.5);
        optional_string_u16_label.set_alignment(0.0, 0.5);

        let bool_entry = Entry::new();
        let float_entry = Entry::new();
        let integer_entry = Entry::new();
        let string_u8_entry = Entry::new();
        let string_u16_entry = Entry::new();
        let optional_string_u8_entry = Entry::new();
        let optional_string_u16_entry = Entry::new();

        let use_bool_button = Button::new_with_label("Use this");
        let use_float_button = Button::new_with_label("Use this");
        let use_integer_button = Button::new_with_label("Use this");
        let use_string_u8_button = Button::new_with_label("Use this");
        let use_string_u16_button = Button::new_with_label("Use this");
        let use_optional_string_u8_button = Button::new_with_label("Use this");
        let use_optional_string_u16_button = Button::new_with_label("Use this");

        bool_box.pack_start(&bool_label, false, false, 10);
        bool_box.pack_start(&bool_entry, false, false, 0);
        bool_box.pack_start(&use_bool_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&bool_box, false, false, 2);

        float_box.pack_start(&float_label, false, false, 10);
        float_box.pack_start(&float_entry, false, false, 0);
        float_box.pack_start(&use_float_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&float_box, false, false, 2);

        integer_box.pack_start(&integer_label, false, false, 10);
        integer_box.pack_start(&integer_entry, false, false, 0);
        integer_box.pack_start(&use_integer_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&integer_box, false, false, 2);

        string_u8_box.pack_start(&string_u8_label, false, false, 10);
        string_u8_box.pack_start(&string_u8_entry, false, false, 0);
        string_u8_box.pack_start(&use_string_u8_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&string_u8_box, false, false, 2);

        string_u16_box.pack_start(&string_u16_label, false, false, 10);
        string_u16_box.pack_start(&string_u16_entry, false, false, 0);
        string_u16_box.pack_start(&use_string_u16_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&string_u16_box, false, false, 2);

        optional_string_u8_box.pack_start(&optional_string_u8_label, false, false, 10);
        optional_string_u8_box.pack_start(&optional_string_u8_entry, false, false, 0);
        optional_string_u8_box.pack_start(&use_optional_string_u8_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&optional_string_u8_box, false, false, 2);

        optional_string_u16_box.pack_start(&optional_string_u16_label, false, false, 10);
        optional_string_u16_box.pack_start(&optional_string_u16_entry, false, false, 0);
        optional_string_u16_box.pack_start(&use_optional_string_u16_button, false, false, 0);
        packed_file_decoded_data_box.pack_start(&optional_string_u16_box, false, false, 2);

        // Then, we put another box (boxception) and put in it the data of the table, the buttons
        // to set the field as "key" and for finishing the decoding.
        let packed_file_field_settings_box = Box::new(Orientation::Vertical, 0);
        let fields_tree_view = TreeView::new();
        let fields_list_store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), bool::static_type(), String::static_type(), String::static_type()]);
        fields_tree_view.set_model(Some(&fields_list_store));

        let column_index = TreeViewColumn::new();
        let cell_index = CellRendererText::new();
        column_index.pack_start(&cell_index, true);
        column_index.add_attribute(&cell_index, "text", 0);
        column_index.set_title("Index");


        let column_name = TreeViewColumn::new();
        let cell_name = CellRendererText::new();
        column_name.pack_start(&cell_name, true);
        column_name.add_attribute(&cell_name, "text", 1);
        column_name.set_title("Field name");

        let column_type = TreeViewColumn::new();
        let cell_type = CellRendererText::new();
        column_type.pack_start(&cell_type, true);
        column_type.add_attribute(&cell_type, "text", 2);
        column_type.set_title("Field Type");

        let column_key = TreeViewColumn::new();
        let cell_key = CellRendererToggle::new();
        column_key.pack_start(&cell_key, true);
        column_key.add_attribute(&cell_key, "active", 3);
        column_key.set_title("Is key?");

        let column_ref_table = TreeViewColumn::new();
        let cell_ref_table = CellRendererText::new();
        column_ref_table.pack_start(&cell_ref_table, true);
        column_ref_table.add_attribute(&cell_ref_table, "text", 4);
        column_ref_table.set_title("Ref. to table");

        let column_ref_column = TreeViewColumn::new();
        let cell_ref_column = CellRendererText::new();
        column_ref_column.pack_start(&cell_ref_column, true);
        column_ref_column.add_attribute(&cell_ref_column, "text", 5);
        column_ref_column.set_title("Ref. to column");

        fields_tree_view.append_column(&column_index);
        fields_tree_view.append_column(&column_name);
        fields_tree_view.append_column(&column_type);
        fields_tree_view.append_column(&column_key);
        fields_tree_view.append_column(&column_ref_table);
        fields_tree_view.append_column(&column_ref_column);

        let fields_tree_view_scroll = ScrolledWindow::new(None, None);
        fields_tree_view_scroll.set_size_request(400, 350);

        let packed_file_field_settings_box_table_type = Box::new(Orientation::Horizontal, 0);
        let packed_file_decoded_data_table_type_label = Label::new("Table Type:");
        let table_type_label = Label::new("0");
        let packed_file_field_settings_box_table_version = Box::new(Orientation::Horizontal, 0);
        let packed_file_decoded_data_table_version_label = Label::new("Table Version:");
        let table_version_label = Label::new("1");
        let packed_file_field_settings_box_table_entry_count = Box::new(Orientation::Horizontal, 0);
        let packed_file_decoded_data_table_entry_count_label = Label::new("Table Entry Count:");
        let table_entry_count_label = Label::new("2");
        let field_name_box = Box::new(Orientation::Horizontal, 0);
        let field_name_label = Label::new("Field Name:");
        let field_name_entry = Entry::new();

        let is_key_field_button = ToggleButton::new_with_label("Key field");
        let save_decoded_schema = Button::new_with_label("Finish It!");

        packed_file_field_settings_box_table_type.pack_start(&packed_file_decoded_data_table_type_label, false, false, 2);
        packed_file_field_settings_box_table_type.pack_start(&table_type_label, false, false, 2);

        packed_file_field_settings_box_table_version.pack_start(&packed_file_decoded_data_table_version_label, false, false, 2);
        packed_file_field_settings_box_table_version.pack_start(&table_version_label, false, false, 2);

        packed_file_field_settings_box_table_entry_count.pack_start(&packed_file_decoded_data_table_entry_count_label, false, false, 2);
        packed_file_field_settings_box_table_entry_count.pack_start(&table_entry_count_label, false, false, 2);

        field_name_box.pack_start(&field_name_label, false, false, 2);
        field_name_box.pack_start(&field_name_entry, false, false, 2);

        fields_tree_view_scroll.add(&fields_tree_view);
        packed_file_field_settings_box.pack_start(&fields_tree_view_scroll, false, false, 2);
        packed_file_field_settings_box.pack_start(&field_name_box, false, false, 2);
        packed_file_field_settings_box.pack_start(&packed_file_field_settings_box_table_type, false, false, 2);
        packed_file_field_settings_box.pack_start(&packed_file_field_settings_box_table_version, false, false, 2);
        packed_file_field_settings_box.pack_start(&packed_file_field_settings_box_table_entry_count, false, false, 2);
        packed_file_field_settings_box.pack_start(&is_key_field_button, false, false, 2);
        packed_file_field_settings_box.pack_end(&save_decoded_schema, false, false, 2);

        packed_file_raw_data_scroll.add(&raw_data_box);
        decoder_box.add(&packed_file_raw_data_scroll);
        decoder_box.pack_end(&packed_file_field_settings_box, true, true, 0);
        decoder_box.pack_end(&packed_file_decoded_data_box, true, true, 0);

        packed_file_data_display.show_all();

        PackedFileDBDecoder {
            raw_data_line_index,
            raw_data,
            //raw_data_decoded,
            table_type_label,
            table_version_label,
            table_entry_count_label,
            bool_entry,
            float_entry,
            integer_entry,
            string_u8_entry,
            string_u16_entry,
            optional_string_u8_entry,
            optional_string_u16_entry,
            use_bool_button,
            use_float_button,
            use_integer_button,
            use_string_u8_button,
            use_string_u16_button,
            use_optional_string_u8_button,
            use_optional_string_u16_button,
            fields_list_store,
            field_name_entry,
            is_key_field_button,
            save_decoded_schema,
        }
    }

    /// This function creates the "Decoder" box with all the stuff needed to decode a table.
    pub fn load_data_to_decoder_view(
        packed_file_decoder_view: &PackedFileDBDecoder,
        packed_file_table_type: &str,
        packed_file_encoded: &Vec<u8>
    ) -> Result<(DBHeader, usize), Error> {
        let db_header = DBHeader::read(packed_file_encoded.to_vec())?;

        let hex_lines = (packed_file_encoded.len() / 16) + 1;
        let mut hex_lines_text = "00\n".to_string();
        for hex_line in 1..hex_lines {
            hex_lines_text.push_str(&format!("{:X}\n", hex_line * 16));
        }
        packed_file_decoder_view.raw_data_line_index.set_text(&hex_lines_text);

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
        let hex_raw_data = hex_raw_data.replace(" 0]", " 00");
        let hex_raw_data = hex_raw_data.replace(" 1]", " 01");
        let hex_raw_data = hex_raw_data.replace(" 2]", " 02");
        let hex_raw_data = hex_raw_data.replace(" 3]", " 03");
        let hex_raw_data = hex_raw_data.replace(" 4]", " 04");
        let hex_raw_data = hex_raw_data.replace(" 5]", " 05");
        let hex_raw_data = hex_raw_data.replace(" 6]", " 06");
        let hex_raw_data = hex_raw_data.replace(" 7]", " 07");
        let hex_raw_data = hex_raw_data.replace(" 8]", " 08");
        let hex_raw_data = hex_raw_data.replace(" 9]", " 09");
        let hex_raw_data = hex_raw_data.replace(" A]", " 0A");
        let hex_raw_data = hex_raw_data.replace(" B]", " 0B");
        let hex_raw_data = hex_raw_data.replace(" C]", " 0C");
        let hex_raw_data = hex_raw_data.replace(" D]", " 0D");
        let hex_raw_data = hex_raw_data.replace(" E]", " 0E");
        let hex_raw_data = hex_raw_data.replace(" F]", " 0F");

        packed_file_decoder_view.raw_data.get_buffer().unwrap().set_text(&hex_raw_data);


        packed_file_decoder_view.table_type_label.set_text(packed_file_table_type);
        packed_file_decoder_view.table_version_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_version));
        packed_file_decoder_view.table_entry_count_label.set_text(&format!("{}", db_header.0.packed_file_header_packed_file_entry_count));
        Ok(db_header)
    }

    /// This function updates the data shown in the "Decoder" box when we execute it.
    pub fn update_decoder_view(
        packed_file_decoder: &PackedFileDBDecoder,
        packed_file_decoded: Vec<u8>,
        table_definition: &TableDefinition,
        index_data: usize,
    ) {

        // We need to get the length of the vector first, to avoid crashes due to non-existant indexes
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

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
        if (index_data + 4) < packed_file_decoded.len() {
            decoded_float = match coding_helpers::decode_packedfile_float_u32(
                packed_file_decoded[index_data..(index_data + 4)].to_vec(),
                index_data
            ) {
                Ok(data) => data.0.to_string(),
                Err(_) => "Error".to_string()
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_u32(
                packed_file_decoded[index_data..(index_data + 4)].to_vec(),
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

        decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(
            packed_file_decoded[index_data..].to_vec(),
            index_data
        ) {
            Ok(data) => data.0,
            Err(_) => "Error".to_string()
        };

        decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(
            packed_file_decoded[index_data..].to_vec(),
            index_data
        ) {
            Ok(data) => data.0,
            Err(_) => "Error".to_string()
        };

        decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(
            packed_file_decoded[index_data..].to_vec(),
            index_data
        ) {
            Ok(data) => data.0,
            Err(_) => "Error".to_string()
        };

        decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(
            packed_file_decoded[index_data..].to_vec(),
            index_data
        ) {
            Ok(data) => data.0,
            Err(_) => "Error".to_string()
        };

        // We update all the decoded entries here.
        packed_file_decoder.bool_entry.get_buffer().set_text(decoded_bool);
        packed_file_decoder.float_entry.get_buffer().set_text(&*decoded_float);
        packed_file_decoder.integer_entry.get_buffer().set_text(&*decoded_integer);
        packed_file_decoder.string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u8));
        packed_file_decoder.string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_string_u16));
        packed_file_decoder.optional_string_u8_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u8));
        packed_file_decoder.optional_string_u16_entry.get_buffer().set_text(&format!("{:?}", decoded_optional_string_u16));

        // We clear the store, then we rebuild it.
        packed_file_decoder.fields_list_store.clear();
        for (index, field) in table_definition.fields.iter().enumerate() {
            let field_type = match field.field_type {
                FieldType::Boolean => "bool",
                FieldType::Float => "float",
                FieldType::Integer => "integer",
                FieldType::StringU8 => "string_u8",
                FieldType::StringU16 => "string_u16",
                FieldType::OptionalStringU8 => "optional_string_u8",
                FieldType::OptionalStringU16 => "optional_string_u16",
            };
            if let Some(ref reference) = field.field_is_reference {
                packed_file_decoder.fields_list_store.insert_with_values(
                    None,
                    &[0, 1, 2, 3, 4, 5],
                    &[&format!(
                        "{:0count$}", index, count = (table_definition.fields.len().to_string().len() + 1)),
                        &field.field_name,
                        &field_type,
                        &field.field_is_key,
                        &reference.0,
                        &reference.1
                    ]
                );
            }
            else {
                packed_file_decoder.fields_list_store.insert_with_values(
                    None,
                    &[0, 1, 2, 3, 4, 5],
                    &[
                        &format!("{:0count$}", index, count = (table_definition.fields.len().to_string().len() + 1)),
                        &field.field_name,
                        &field_type,
                        &field.field_is_key,
                        &String::new(),
                        &String::new()
                    ]
                );
            }

        }
        // We reset these two every time we add a field.
        packed_file_decoder.field_name_entry.get_buffer().set_text("");
        packed_file_decoder.is_key_field_button.set_active(false);
    }

}