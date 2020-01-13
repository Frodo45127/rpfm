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
Module with all the code for managing the view for Table PackedFiles.
!*/

use qt_widgets::combo_box::ComboBox;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::qt::CheckState;
use qt_core::reg_exp::RegExp;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::variant::Variant;


use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::Result;
use rpfm_lib::packedfile::table::{DecodedData, db::DB, loc::Loc};
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::{Definition, Field, FieldType, Schema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::new_tableview_frozen;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;

use self::slots::PackedFileTableViewSlots;

mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum is used to distinguish between the different types of tables we can decode.
#[derive(Clone)]
pub enum TableType {
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    Loc(Loc),
}

/// This struct contains pointers to all the widgets in a Table View.
pub struct PackedFileTableView {
    table_view_primary: AtomicPtr<TableView>,
    table_view_frozen: AtomicPtr<TableView>,

    filter_line_edit: AtomicPtr<LineEdit>,
}

/// This struct contains the raw version of each pointer in `PackedFileTableView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileTableView`.
#[derive(Clone, Copy)]
pub struct PackedFileTableViewRaw {
    pub table_view_primary: *mut TableView,
    pub table_view_frozen: *mut TableView,

    pub filter_line_edit: *mut LineEdit,
}


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableView`.
impl PackedFileTableView {

    /// This function creates a new Table View, and sets up his slots and connections.
    pub fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
        global_search_ui: &GlobalSearchUI,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        // Get the decoded Table.
        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileTable(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (table_data, packed_file_info) = match response {
            Response::DBPackedFileInfo((table, packed_file_info)) => (TableType::DB(table), packed_file_info),
            Response::LocPackedFileInfo((table, packed_file_info)) => (TableType::Loc(table), packed_file_info),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let table_definition = match table_data {
            TableType::DependencyManager(_) => {
                let schema = SCHEMA.read().unwrap();
                schema.as_ref().unwrap().get_versioned_file_dep_manager().unwrap().get_version_list()[0].clone()
            },
            TableType::DB(ref table) => table.get_definition(),
            TableType::Loc(ref table) => table.get_definition(),
        };

        // Create the "Undo" stuff needed for the Undo/Redo functions to work.
        //let undo_lock = Rc::new(RefCell::new(false));
        //let undo_redo_enabler = Action::new(()).into_raw();
        //if table_state_data.borrow().get(&*packed_file_path.borrow()).is_none() {
            //let _y = table_state_data.borrow_mut().insert(packed_file_path.borrow().to_vec(), TableStateData::new_empty());
        //}

        // Create the "Paste Lock", so we don't save in every freaking edit.
        //let save_lock = Rc::new(RefCell::new(false));

        // Prepare the model and the filter..
        let mut filter_model = SortFilterProxyModel::new();
        let model = StandardItemModel::new(());
        unsafe { filter_model.set_source_model(model.into_raw() as *mut AbstractItemModel); }

        // Prepare the TableViews.
        let mut table_view_frozen = TableView::new();
        let table_view = unsafe { new_tableview_frozen(filter_model.into_raw() as *mut AbstractItemModel, table_view_frozen.as_mut_ptr()) };
        table_view_frozen.hide();
        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.lock().unwrap().settings_bool["extend_last_column_on_tables"] {
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        }

        // Create the filter's LineEdit.
        let mut row_filter_line_edit = LineEdit::new(());
        row_filter_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!"));

        // Create the filter's column selector.
        // TODO: Make this follow the visual order of the columns, NOT THE LOGICAL ONE.
        let mut row_filter_column_selector = ComboBox::new();
        let row_filter_column_list = StandardItemModel::new(());
        unsafe { row_filter_column_selector.set_model(row_filter_column_list.as_mut_ptr() as *mut AbstractItemModel) };
        for column in &table_definition.fields {
            let name = Self::clean_column_names(&column.name);
            row_filter_column_selector.add_item(&QString::from_std_str(&name));
        }

        // Create the filter's "Case Sensitive" button.
        let row_filter_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { row_filter_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        Self::load_data(table_view, &table_data, &table_definition);

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        let packed_file_name = if packed_file_path.borrow().len() == 3 && packed_file_path.borrow()[1] == "db" { packed_file_path.borrow()[2].to_owned() } else { "".to_owned() };
        Self::build_columns(table_view, table_view_frozen.as_mut_ptr(), &table_definition, &packed_file_name);

        // Add Table to the Grid.
        let layout = unsafe { packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout };
        unsafe { layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 0, 1, 3)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_line_edit.as_mut_ptr() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_case_sensitive_button as *mut Widget, 2, 1, 1, 1)); }
        unsafe { layout.as_mut().unwrap().add_widget((row_filter_column_selector.into_raw() as *mut Widget, 2, 2, 1, 1)); }

        let packed_file_table_view_raw = PackedFileTableViewRaw {
            table_view_primary: table_view,
            table_view_frozen: table_view_frozen.into_raw(),
            filter_line_edit: row_filter_line_edit.into_raw(),
        };


        let packed_file_table_view_slots = PackedFileTableViewSlots::new(packed_file_table_view_raw, *global_search_ui, &packed_file_path);


        let packed_file_table_view = Self {
            table_view_primary: AtomicPtr::new(packed_file_table_view_raw.table_view_primary),
            table_view_frozen: AtomicPtr::new(packed_file_table_view_raw.table_view_frozen),
            filter_line_edit: AtomicPtr::new(packed_file_table_view_raw.filter_line_edit),
        };
        connections::set_connections(&packed_file_table_view, &packed_file_table_view_slots);

        packed_file_view.view = View::Table(packed_file_table_view);

        // Return success.
        Ok((TheOneSlot::Table(packed_file_table_view_slots), packed_file_info))
    }


    /// This function loads the data from a compatible `PackedFile` into a TableView.
    pub fn load_data(
        view: *mut TableView,
        data: &TableType,
        table_definition: &Definition,
        //dependency_data: &BTreeMap<i32, Vec<String>>,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        // This wipes out header information, so remember to run "build_columns" after this.
        let table_view = unsafe { view.as_ref().unwrap() };
        let filter = unsafe { (table_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter.source_model() as *mut StandardItemModel).as_mut().unwrap() };
        model.clear();

        // Set the right data, depending on the table type you get.
        let data = match data {
            TableType::DependencyManager(data) => &*data,
            TableType::DB(data) => &*data.get_ref_table_data(),
            TableType::Loc(data) => data.get_ref_table_data(),
        };

        for entry in data {
            let mut qlist = ListStandardItemMutPtr::new(());
            for (index, field) in entry.iter().enumerate() {

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

                    // Floats need to be tweaked to fix trailing zeroes and precission issues, like turning 0.5000004 into 0.5.
                    // Also, they should be limited to 3 decimals.
                    DecodedData::Float(ref data) => {
                        let data = {
                            let data_str = format!("{}", data);
                            if let Some(position) = data_str.find('.') {
                                let decimals = &data_str[position..].len();
                                if *decimals > 3 { format!("{:.3}", data).parse::<f32>().unwrap() }
                                else { *data }
                            }
                            else { *data }
                        };

                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(data), 2));
                        item
                    },
                    DecodedData::Integer(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new0(*data), 2));
                        item
                    },
                    DecodedData::LongInteger(ref data) => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(*data), 2));
                        item
                    },
                    // All these are Strings, so it can be together,
                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => StandardItem::new(&QString::from_std_str(data)),
                    DecodedData::Sequence(_) => StandardItem::new(&QString::from_std_str("Non Editable Sequence")),
                };

                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] && table_definition.fields[index].is_reference.is_some() {
                    //Self::check_references(dependency_data, index as i32, item.as_mut_ptr());
                }

                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }
            model.append_row(&qlist);
        }

        // If the table it's empty, we add an empty row and delete it, so the "columns" get created.
        if data.is_empty() {
            let mut qlist = ListStandardItemMutPtr::new(());
            for field in &table_definition.fields {
                let item = match field.field_type {
                    FieldType::Boolean => {
                        let mut item = StandardItem::new(());
                        item.set_editable(false);
                        item.set_checkable(true);
                        item.set_check_state(CheckState::Checked);
                        item
                    }
                    FieldType::Float => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(0.0f32), 2));
                        item
                    },
                    FieldType::Integer => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new0(0i32), 2));
                        item
                    },
                    FieldType::LongInteger => {
                        let mut item = StandardItem::new(());
                        item.set_data((&Variant::new2(0i64), 2));
                        item
                    },
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => StandardItem::new(&QString::from_std_str("")),
                    FieldType::Sequence(_) => StandardItem::new(&QString::from_std_str("Non Editable Sequence")),
                };
                unsafe { qlist.append_unsafe(&item.into_raw()); }
            }
            model.append_row(&qlist);
            model.remove_rows((0, 1));
        }

        // Here we assing the ItemDelegates, so each type has his own widget with validation included.
        // LongInteger uses normal string controls due to QSpinBox being limited to i32.
        // The rest don't need any kind of validation. For now.
        /*
        for (column, field) in table_definition.fields.iter().enumerate() {
            match field.field_type {
                FieldType::Boolean => {},
                FieldType::Float => unsafe { qt_custom_stuff::new_doublespinbox_item_delegate(table_view as *mut Object, column as i32) },
                FieldType::Integer => unsafe { qt_custom_stuff::new_spinbox_item_delegate(table_view as *mut Object, column as i32, 32) },
                FieldType::LongInteger => unsafe { qt_custom_stuff::new_spinbox_item_delegate(table_view as *mut Object, column as i32, 64) },
                FieldType::StringU8 => {},
                FieldType::StringU16 => {},
                FieldType::OptionalStringU8 => {},
                FieldType::OptionalStringU16 => {},
            }
        }

        // We build the combos lists here, so it get's rebuilt if we import a TSV and clear the table.
        if !SETTINGS.lock().unwrap().settings_bool["disable_combos_on_tables"] {
            for (column, data) in dependency_data {
                let mut list = StringList::new(());
                data.iter().for_each(|x| list.append(&QString::from_std_str(x)));
                let list: *mut StringList = &mut list;
                unsafe { qt_custom_stuff::new_combobox_item_delegate(table_view as *mut Object, *column, list as *const StringList, true)};
            }
        }*/
    }

    /// This function returns a pointer to the TableView widget.
    pub fn get_table(&self) -> &mut TableView {
        unsafe { self.table_view_primary.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    pub fn get_filter_line_edit(&self) -> &mut LineEdit {
        unsafe { self.filter_line_edit.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    /// This function "process" the column names of a table, so they look like they should.
    fn clean_column_names(field_name: &str) -> String {
        let mut new_name = String::new();
        let mut should_be_uppercase = false;

        for character in field_name.chars() {

            if new_name.is_empty() || should_be_uppercase {
                new_name.push_str(&character.to_uppercase().to_string());
                should_be_uppercase = false;
            }

            else if character == '_' {
                new_name.push(' ');
                should_be_uppercase = true;
            }

            else { new_name.push(character); }
        }

        new_name
    }


    /// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
    /// His intended use is for just after we load/reload the data to the table.
    fn build_columns(
        table_view_primary: *mut TableView,
        table_view_frozen: *mut TableView,
        definition: &Definition,
        table_name: &str,
    ) {
        let table_view_primary = unsafe { table_view_primary.as_mut().unwrap() };
        let table_view_frozen = unsafe { table_view_frozen.as_ref().unwrap() };
        let filter = unsafe { (table_view_primary.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter.source_model() as *mut StandardItemModel).as_mut().unwrap() };
        let schema = SCHEMA.read().unwrap();

        // Create a list of "Key" columns.
        let mut keys = vec![];

        // For each column, clean their name and set their width and tooltip.
        for (index, field) in definition.fields.iter().enumerate() {

            let name = Self::clean_column_names(&field.name);
            let item = StandardItem::new(&QString::from_std_str(&name)).into_raw();
            unsafe { model.set_horizontal_header_item(index as i32, item); }

            // Depending on his type, set one width or another.
            match field.field_type {
                FieldType::Boolean => table_view_primary.set_column_width(index as i32, 100),
                FieldType::Float => table_view_primary.set_column_width(index as i32, 140),
                FieldType::Integer => table_view_primary.set_column_width(index as i32, 140),
                FieldType::LongInteger => table_view_primary.set_column_width(index as i32, 140),
                FieldType::StringU8 => table_view_primary.set_column_width(index as i32, 350),
                FieldType::StringU16 => table_view_primary.set_column_width(index as i32, 350),
                FieldType::OptionalStringU8 => table_view_primary.set_column_width(index as i32, 350),
                FieldType::OptionalStringU16 => table_view_primary.set_column_width(index as i32, 350),
                FieldType::Sequence(_) => table_view_primary.set_column_width(index as i32, 350),
            }

            Self::set_tooltip(&schema, &field, table_name, item);

            // If the field is key, add that column to the "Key" list, so we can move them at the beginning later.
            if field.is_key { keys.push(index); }
        }

        // If we have any "Key" field, move it to the beginning.
        if !keys.is_empty() {
            for (position, column) in keys.iter().enumerate() {
                unsafe { table_view_primary.horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
                unsafe { table_view_frozen.horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
            }
        }
    }

    /// This function sets the tooltip for the provided column header, if the column should have one.
    fn set_tooltip(schema: &Option<Schema>, field: &Field, table_name: &str, item: *mut StandardItem) {

        // If we passed it a table name, build the tooltip based on it. The logic is simple:
        // - If we have a description, we add it to the tooltip.
        // - If the column references another column, we add it to the tooltip.
        // - If the column is referenced by another column, we add it to the tooltip.
        if !table_name.is_empty() {
            let mut tooltip_text = String::new();


            if !field.description.is_empty() {
                tooltip_text.push_str(&format!("<p>{}</p>", field.description));
            }

            if let Some(ref reference) = field.is_reference {
                tooltip_text.push_str(&format!("<p>This column is a reference to:</p><p><i>\"{}/{}\"</i></p>", reference.0, reference.1));
            }

            else {
                let mut referenced_columns = if let Some(ref schema) = schema {
                    let short_table_name = table_name.split_at(table_name.len() - 7).0;
                    let mut columns = vec![];

                    // We get all the db definitions from the schema, then iterate all of them to find what tables reference our own.
                    for versioned_file in schema.get_ref_versioned_file_db_all() {
                        if let VersionedFile::DB(ref_table_name, ref_definition) = versioned_file {
                            let mut found = false;
                            for ref_version in ref_definition {
                                for ref_field in &ref_version.fields {
                                    if let Some((ref_ref_table, ref_ref_field)) = &ref_field.is_reference {
                                        if ref_ref_table == short_table_name && ref_ref_field == &field.name {
                                            found = true;
                                            columns.push((ref_table_name.to_owned(), ref_field.name.to_owned()));
                                        }
                                    }
                                }
                                if found { break; }
                            }
                        }
                    }
                    columns
                } else { vec![] };

                referenced_columns.sort_unstable();
                if !referenced_columns.is_empty() {
                    tooltip_text.push_str("<p>Fields that reference this column:</p>");
                    for (index, reference) in referenced_columns.iter().enumerate() {
                        tooltip_text.push_str(&format!("<i>\"{}/{}\"</i><br>", reference.0, reference.1));

                        // There is a bug that causes tooltips to be displayed out of screen if they're too big. This fixes it.
                        if index == 50 {
                            tooltip_text.push_str(&format!("<p>And many more. Exactly, {} more. Too many to show them here.</p>nnnn", referenced_columns.len() as isize - 50));
                            break ;
                        }
                    }

                    // Dirty trick to remove the last <br> from the tooltip, or the nnnn in case that text get used.
                    tooltip_text.pop();
                    tooltip_text.pop();
                    tooltip_text.pop();
                    tooltip_text.pop();
                }
            }

            // We only add the tooltip if we got something to put into it.
            if !tooltip_text.is_empty() {
                unsafe { item.as_mut().unwrap().set_tool_tip(&QString::from_std_str(&tooltip_text)); }
            }
        }
    }

    /// Function to filter the table. If a value is not provided by a slot, we get it from the widget itself.
    fn filter_table(
        &self,
        pattern: Option<QString>,
    ) {
println!("1");
        // Set the pattern to search.
        let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
        else { unsafe { RegExp::new(&self.get_filter_line_edit().text()) }};
println!("2");

        // Set the column selected.
        //if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
        //else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

        // Check if the filter should be "Case Sensitive".
        /*
        if let Some(case_sensitive) = case_sensitive {
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }

        else {
            let case_sensitive = unsafe { case_sensitive_button.as_mut().unwrap().is_checked() };
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }*/

        // Filter whatever it's in that column by the text we got.
        let filter_model = self.get_table().model() as *mut SortFilterProxyModel;
        unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }
/*
        // Update the search stuff, if needed.
        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

        // Add the new filter data to the state history.
        if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
            unsafe { state.filter_state = FilterState::new(filter_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_mut().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
        }*/
    }
}

impl PackedFileTableViewRaw {

    /// Function to filter the table. If a value is not provided by a slot, we get it from the widget itself.
    fn filter_table(
        &self,
        pattern: Option<QString>,
    ) {

        // Set the pattern to search.
        let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
        else { unsafe { RegExp::new(&self.filter_line_edit.as_mut().unwrap().text()) }};

        // Set the column selected.
        //if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
        //else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

        // Check if the filter should be "Case Sensitive".
        /*
        if let Some(case_sensitive) = case_sensitive {
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }

        else {
            let case_sensitive = unsafe { case_sensitive_button.as_mut().unwrap().is_checked() };
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }*/

        // Filter whatever it's in that column by the text we got.
        let filter_model = unsafe { self.table_view_primary.as_mut().unwrap().model() as *mut SortFilterProxyModel };
        unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }
/*
        // Update the search stuff, if needed.
        unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

        // Add the new filter data to the state history.
        if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
            unsafe { state.filter_state = FilterState::new(filter_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_mut().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
        }*/
    }
}
