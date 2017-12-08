// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate ordermap;
extern crate gtk;
extern crate glib;

use gtk::prelude::*;
use gtk::{
    TreeView, TreeSelection, ListStore, ScrolledWindow,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type
};

use self::glib::prelude::*;
use self::glib::{AnyValue, Value};

use self::ordermap::OrderMap;

/// Struct PackedFileDBTreeView: contains all the stuff we need to give to the program to show a
/// TreeView with the data of a DB PackedFile, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileDBTreeView {
    pub packed_file_tree_view: TreeView,
    pub packed_file_list_store: ListStore,
}

#[derive(Clone, Debug)]
pub enum DecodedData {
    Index(String),
    Boolean(bool),
    String(String),
    OptionalString(String),
    Integer(u32),
    Float(f32)
}

/// Implementation of "PackedFileDBTreeView".
impl PackedFileDBTreeView{

    /// This function creates a new TreeView with "packed_file_data_display" as father and returns a
    /// PackedFileDBTreeView with all his data.
    pub fn create_tree_view(packed_file_data_display: &ScrolledWindow, packed_file_decoded: &::packedfile::db::DB) -> PackedFileDBTreeView {

        // First, we create the slice we are going to use to create the TreeView, based on the structure
        // of the DB PackedFile.
        let mut list_store_structure: Vec<Type> = vec![];
        let packed_file_structure = packed_file_decoded.packed_file_data.packed_file_data_structure.clone().unwrap();

        // The first column is an index for us to know how many entries we have.
        list_store_structure.push(Type::String);

        for (_, field_type) in packed_file_structure.iter() {
            match &**field_type {
                "boolean" => {
                    list_store_structure.push(Type::Bool);
                }
                "string_ascii" => {
                    list_store_structure.push(Type::String);
                }
                "optstring_ascii" => {
                    list_store_structure.push(Type::String);
                }
                "int" => {
                    list_store_structure.push(Type::U32);
                }
                "float" => {
                    list_store_structure.push(Type::F32);
                }
                _ => {
                    println!("Unkown field_type {}", field_type);
                }
            }
        }

        // Then, we create the new ListStore, the new TreeView, and prepare the TreeView to display the data
        let packed_file_tree_view = TreeView::new();
        let packed_file_list_store = ListStore::new(&list_store_structure);

        packed_file_tree_view.set_model(Some(&packed_file_list_store));
        packed_file_tree_view.set_grid_lines(gtk::TreeViewGridLines::Both);

        // Now we create the columns we need for this specific table. Always with an index column first.
        let cell_index = CellRendererText::new();
        let column_index = TreeViewColumn::new();
        column_index.set_title("Index");
        column_index.set_clickable(true);
        column_index.set_max_width(60);
        column_index.set_sort_column_id(0);
        column_index.pack_start(&cell_index, true);
        column_index.add_attribute(&cell_index, "text", 0);
        packed_file_tree_view.append_column(&column_index);

        let mut index = 1;
        for (name, field_type) in packed_file_structure.iter() {
            match &**field_type {
                "boolean" => {
                    let cell_bool = CellRendererToggle::new();
                    cell_bool.set_activatable(true);
                    let column_bool = TreeViewColumn::new();
                    column_bool.set_title(&**name);
                    column_bool.set_clickable(true);
                    column_bool.set_min_width(50);
                    column_bool.set_fixed_width(75);
                    column_bool.set_sort_column_id(index);
                    column_bool.pack_start(&cell_bool, true);
                    column_bool.add_attribute(&cell_bool, "active", index);
                    packed_file_tree_view.append_column(&column_bool);
                }
                "string_ascii" => {
                    let cell_string = CellRendererText::new();
                    cell_string.set_property_editable(true);
                    let column_string = TreeViewColumn::new();
                    column_string.set_title(&**name);
                    column_string.set_clickable(true);
                    column_string.set_resizable(true);
                    column_string.set_min_width(50);
                    column_string.set_fixed_width(200);
                    column_string.set_sort_column_id(index);
                    column_string.pack_start(&cell_string, true);
                    column_string.add_attribute(&cell_string, "text", index);
                    packed_file_tree_view.append_column(&column_string);
                }
                "optstring_ascii" => {
                    let cell_optional_string = CellRendererText::new();
                    cell_optional_string.set_property_editable(true);
                    let column_optional_string = TreeViewColumn::new();
                    column_optional_string.set_title(&**name);
                    column_optional_string.set_clickable(true);
                    column_optional_string.set_resizable(true);
                    column_optional_string.set_min_width(50);
                    column_optional_string.set_fixed_width(200);
                    column_optional_string.set_sort_column_id(index);
                    column_optional_string.pack_start(&cell_optional_string, true);
                    column_optional_string.add_attribute(&cell_optional_string, "text", index);
                    packed_file_tree_view.append_column(&column_optional_string);
                }
                "int" => {
                    let cell_int = CellRendererText::new();
                    cell_int.set_property_editable(true);
                    let column_int = TreeViewColumn::new();
                    column_int.set_title(&**name);
                    column_int.set_clickable(true);
                    column_int.set_resizable(true);
                    column_int.set_min_width(50);
                    column_int.set_fixed_width(100);
                    column_int.set_sort_column_id(index);
                    column_int.pack_start(&cell_int, true);
                    column_int.add_attribute(&cell_int, "text", index);
                    packed_file_tree_view.append_column(&column_int);
                }
                "float" => {
                    let cell_float = CellRendererText::new();
                    cell_float.set_property_editable(true);
                    let column_float = TreeViewColumn::new();
                    column_float.set_title(&**name);
                    column_float.set_clickable(true);
                    column_float.set_resizable(true);
                    column_float.set_min_width(50);
                    column_float.set_fixed_width(100);
                    column_float.set_sort_column_id(index);
                    column_float.pack_start(&cell_float, true);
                    column_float.add_attribute(&cell_float, "text", index);
                    packed_file_tree_view.append_column(&column_float);
                }
                _ => {
                    println!("Unkown field_type {}", field_type);
                }
            }
            index += 1;
        }

        packed_file_tree_view.set_enable_search(false);

        packed_file_data_display.add(&packed_file_tree_view);
        packed_file_data_display.show_all();

        PackedFileDBTreeView {
            packed_file_tree_view,
            packed_file_list_store,
        }
    }

    /// This function decodes the data of a DB PackedFile and loads it into a TreeView.
    pub fn load_data_to_tree_view(
        packed_file_data: Vec<u8>,
        packed_file_data_structure: &Option<OrderMap<String, String>>,
        packed_file_tree_view: &TreeView,
        packed_file_list_store: &ListStore,
        packed_file_data_entry_count: u32
    ) {

        let packed_file_structure = packed_file_data_structure.clone().unwrap();

        // First, we delete all the data from the ListStore.
        packed_file_list_store.clear();

        // Second, we create the array for the positions in the ListStore.
        let mut column_numbers: Vec<u32> = vec![];
        for i in 0..packed_file_tree_view.get_n_columns() {
            column_numbers.push(i.clone());
        }

        // Third, we decode the values from the raw data to the ListStore.
        let mut entries: Vec<Vec<DecodedData>> = vec![];
        let mut entry: Vec<DecodedData> = vec![];

        let mut index = 0;
        for i in 0..packed_file_data_entry_count {
            for j in column_numbers.iter() {
                if *j == 0 {
                    let entry_index = DecodedData::Index(format!("{:0count$}", (i + 1), count = (packed_file_data_entry_count as usize / 10) + 1));
                    entry.push(entry_index);
                }
                else {
                    let field = packed_file_structure.get_index((*j as usize) - 1).unwrap();
                    let field_name = field.0;
                    let field_type = field.1;

                    match &**field_type {
                        "boolean" => {
                            let data = ::packedfile::db::helpers::decode_bool(packed_file_data.to_vec(), index);
                            index = data.1;
                            entry.push(DecodedData::Boolean(data.0));
                        }
                        "string_ascii" => {
                            let data = ::packedfile::db::helpers::decode_string_u8(packed_file_data.to_vec(), index);
                            index = data.1;
                            entry.push(DecodedData::String(data.0));
                        }
                        "optstring_ascii" => {
                            let data = ::packedfile::db::helpers::decode_optional_string_u8(packed_file_data.to_vec(), index);
                            index = data.1;
                            entry.push(DecodedData::OptionalString(data.0));
                        }
                        "int" => {
                            let data = ::packedfile::db::helpers::decode_integer_u32(packed_file_data.to_vec(), index);
                            index = data.1;
                            entry.push(DecodedData::Integer(data.0));
                        }
                        "float" => {
                            let data = ::packedfile::db::helpers::decode_float_u32(packed_file_data.to_vec(), index);
                            index = data.1;
                            entry.push(DecodedData::Float(data.0));
                        }
                        _ => {
                            println!("Unkown field_type {}", field_type);
                        }

                    }
                }

            }
            entries.push(entry.clone());
            entry.clear();
        }

        // Then we add every line to the ListStore.
        for entry in entries {
            let mut index = 0;
            let current_row = packed_file_list_store.append();

            for field in entry {
                let gtk_value_field;
                match field {
                    DecodedData::Index(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Boolean(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::String(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::OptionalString(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Integer(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    DecodedData::Float(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    _ => gtk_value_field = gtk::ToValue::to_value("Error"),
                }

                packed_file_list_store.set_value(&current_row, index, &gtk_value_field);
                index += 1;
            }
            index = 0;
        }
    }
}