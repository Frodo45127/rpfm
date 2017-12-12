// In this file are all the helper functions used by the UI when decoding DB PackedFiles.
extern crate ordermap;
extern crate gtk;
extern crate glib;

use gtk::prelude::*;
use gtk::{
    Box, TreeView, ListStore, ScrolledWindow,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type
};

use self::ordermap::OrderMap;

/// Struct PackedFileDBTreeView: contains all the stuff we need to give to the program to show a
/// TreeView with the data of a DB PackedFile, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileDBTreeView {
    pub packed_file_tree_view: TreeView,
    pub packed_file_list_store: ListStore,
    pub packed_file_tree_view_cell_bool: Vec<CellRendererToggle>,
    pub packed_file_tree_view_cell_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_optional_string: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_integer: Vec<CellRendererText>,
    pub packed_file_tree_view_cell_float: Vec<CellRendererText>,
}

/// Implementation of "PackedFileDBTreeView".
impl PackedFileDBTreeView{

    /// This function creates a new TreeView with "packed_file_data_display" as father and returns a
    /// PackedFileDBTreeView with all his data.
    pub fn create_tree_view(
        packed_file_data_display: &Box,
        packed_file_decoded: &::packedfile::db::DB
    ) -> PackedFileDBTreeView {

        // First, we create the Vec<Type> we are going to use to create the TreeView, based on the structure
        // of the DB PackedFile.
        let mut list_store_structure: Vec<Type> = vec![];
        let packed_file_structure = packed_file_decoded.packed_file_data.packed_file_data_structure.clone().unwrap();

        // The first column is an index for us to know how many entries we have.
        list_store_structure.push(Type::String);

        // Depending on the type of the field, we push the gtk::Type equivalent to that column.
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
                    // This should only fire when we try to open a table with a non-implemented type.
                    // TODO: implement the types "string" and "oopstring". I guess those are u16 strings.
                    println!("Unkown field_type 3 {}", field_type);
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

        let mut packed_file_tree_view_cell_bool = vec![];
        let mut packed_file_tree_view_cell_string = vec![];
        let mut packed_file_tree_view_cell_optional_string = vec![];
        let mut packed_file_tree_view_cell_integer = vec![];
        let mut packed_file_tree_view_cell_float = vec![];

        let mut index = 1;
        for (name, field_type) in packed_file_structure.iter() {

            // These are the specific declarations of the columns for every type implemented.
            // FIXME: the name of the columns has no spaces nor underscores.
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
                    packed_file_tree_view_cell_bool.push(cell_bool);
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
                    packed_file_tree_view_cell_string.push(cell_string);
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
                    packed_file_tree_view_cell_optional_string.push(cell_optional_string);
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
                    packed_file_tree_view_cell_integer.push(cell_int);
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
                    packed_file_tree_view_cell_float.push(cell_float);
                }
                _ => {
                    // This should only fire when we try to open a table with a non-implemented type.
                    // TODO: implement the types "string" and "oopstring". I guess those are u16 strings.
                    println!("Unkown field_type 2 {}", field_type);
                }
            }
            index += 1;
        }

        // Disabled search. Not sure why I disabled it, but until all the decoding/enconding stuff is
        // done, better keep it disable so it doesn't interfere with the events.
        packed_file_tree_view.set_enable_search(false);

        let packed_file_data_scroll = ScrolledWindow::new(None, None);
        packed_file_data_scroll.add(&packed_file_tree_view);
        packed_file_data_display.pack_end(&packed_file_data_scroll, true, true, 0);
        packed_file_data_display.show_all();

        PackedFileDBTreeView {
            packed_file_tree_view,
            packed_file_list_store,
            packed_file_tree_view_cell_bool,
            packed_file_tree_view_cell_string,
            packed_file_tree_view_cell_optional_string,
            packed_file_tree_view_cell_integer,
            packed_file_tree_view_cell_float,
        }
    }

    /// This function decodes the data of a DB PackedFile and loads it into a TreeView.
    pub fn load_data_to_tree_view(
        packed_file_data: Vec<Vec<::packedfile::db::DecodedData>>,
        packed_file_list_store: &ListStore,
    ) {

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
                    ::packedfile::db::DecodedData::Index(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::Boolean(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::String(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::OptionalString(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::Integer(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::Float(data) => gtk_value_field = gtk::ToValue::to_value(&data),
                    ::packedfile::db::DecodedData::RawData(_) => gtk_value_field = gtk::ToValue::to_value("Error"),
                }
                packed_file_list_store.set_value(&current_row, index, &gtk_value_field);
                index += 1;
            }
        }
    }

    /// This function returns a Vec<DataDecoded> with all the stuff in the table. We need for it the
    /// ListStore, and it'll return a Vec<DataDecoded> with all the stuff from the table.
    pub fn return_data_from_tree_view(
        packed_file_data_structure: &Option<OrderMap<String, String>>,
        packed_file_list_store: &ListStore,
    ) -> Vec<Vec<::packedfile::db::DecodedData>> {

        let mut packed_file_data_from_tree_view: Vec<Vec<::packedfile::db::DecodedData>> = vec![];

        // Only in case we have any line in the ListStore we try to get it. Otherwise we return an
        // empty vector.
        if let Some(current_line) = packed_file_list_store.get_iter_first() {
            let columns = packed_file_list_store.get_n_columns();

            // Foreach row in the DB PackedFile.
            let mut done = false;
            while !done {

                let mut packed_file_data_from_tree_view_entry: Vec<::packedfile::db::DecodedData> = vec![];

                for column in 1..columns {
                    let column_structure = packed_file_data_structure.clone().unwrap();
                    let field_type = column_structure.get_index(column as usize - 1).unwrap().1;
                    match &**field_type {
                        "boolean" => {
                            let data: bool = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(::packedfile::db::DecodedData::Boolean(data));
                        }
                        "string_ascii" => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(::packedfile::db::DecodedData::String(data));
                        }
                        "optstring_ascii" => {
                            let data: String = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(::packedfile::db::DecodedData::OptionalString(data));
                        }
                        "int" => {
                            let data: u32 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(::packedfile::db::DecodedData::Integer(data));
                        }
                        "float" => {
                            let data: f32 = packed_file_list_store.get_value(&current_line, column).get().unwrap();
                            packed_file_data_from_tree_view_entry.push(::packedfile::db::DecodedData::Float(data));
                        }
                        _ => {
                            // If this fires up, the table has a non-implemented field. Current non-
                            // implemented fields are "string" and "oopstring".
                            println!("Unkown field_type {}", field_type);
                        }
                    }
                }
                packed_file_data_from_tree_view.push(packed_file_data_from_tree_view_entry);

                if !packed_file_list_store.iter_next(&current_line) {
                    done = true;
                }
            }
        }
        packed_file_data_from_tree_view
    }
}