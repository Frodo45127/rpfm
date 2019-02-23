//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This file contains all the common stuff between DB, the DB Decoder and Locs, 
// to reduce duplicated code. It also houses the DB Decoder, because thatś 
// related with the tables.

use qt_widgets::action::Action;
use qt_widgets::table_view::TableView;

use qt_gui::gui_application::GuiApplication;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::reg_exp::RegExp;
use qt_core::qt::{CaseSensitivity, GlobalColor};

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::TABLE_STATES_UI;
use crate::QString;
use crate::ui::*;
use crate::ui::table_state::*;

pub mod packedfile_db;
pub mod db_decoder;

/// Enum MathOperationResult: used for keeping together different types of results after a math operation.
/// None is an special type, so we know that we have to ignore an specified cell.
#[derive(Clone, PartialEq)]
enum MathOperationResult {
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    None
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
/// His intended use is for just after we reload the data to the table.
fn build_columns(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    table_name: &str,
) {
    // Create a list of "Key" columns.
    let mut keys = vec![];

    // For each column, clean their name and set their width and tooltip.
    for (index, field) in definition.fields.iter().enumerate() {

        let name = clean_column_names(&field.field_name);
        let item = StandardItem::new(&QString::from_std_str(&name)).into_raw();
        unsafe { model.as_mut().unwrap().set_horizontal_header_item(index as i32, item) };

        // Depending on his type, set one width or another.
        match field.field_type {
            FieldType::Boolean => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 100); }
            FieldType::Float => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
            FieldType::Integer => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
            FieldType::LongInteger => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 140); }
            FieldType::StringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::StringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::OptionalStringU8 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
            FieldType::OptionalStringU16 => unsafe { table_view.as_mut().unwrap().set_column_width(index as i32, 350); }
        }

        // Create the tooltip for the column. To get the reference data, we iterate through every table in the schema and check their references.
        let mut tooltip_text = String::new();
        if !field.field_description.is_empty() { tooltip_text.push_str(&format!("<p>{}</p>", field.field_description)); }
        if let Some(ref reference) = field.field_is_reference {
            tooltip_text.push_str(&format!("<p>This column is a reference to:</p><p><i>\"{}/{}\"</i></p>", reference.0, reference.1));
        } else { 
            let schema = SCHEMA.lock().unwrap().clone();
            let mut referenced_columns = if let Some(schema) = schema {
                let short_table_name = table_name.split_at(table_name.len() - 7).0;
                let mut columns = vec![];
                for table in schema.tables_definitions {
                    let mut found = false;
                    for version in table.versions {
                        for field_ref in version.fields {
                            if let Some(ref_data) = field_ref.field_is_reference { 
                                if &ref_data.0 == short_table_name && ref_data.1 == field.field_name {
                                    found = true;
                                    columns.push((table.name.to_owned(), field_ref.field_name)); 
                                }
                            }
                        }
                        if found { break; }
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
        if !tooltip_text.is_empty() { unsafe { item.as_mut().unwrap().set_tool_tip(&QString::from_std_str(&tooltip_text)); }}

        // If the field is key, add that column to the "Key" list, so we can move them at the begining later.
        if field.field_is_key { keys.push(index); }
    }

    // If we have any "Key" field, move it to the beginning.
    if !keys.is_empty() {
        for (position, column) in keys.iter().enumerate() {
            unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().move_section(*column as i32, position as i32); }
        }
    }
}

// Function to check if an specific field's data is in their references.
fn check_references(
    dependency_data: &BTreeMap<i32, Vec<String>>,
    column: i32,
    item: *mut StandardItem,
) {
    // Check if it's a valid reference.
    if let Some(ref_data) = dependency_data.get(&column) {

        let text = unsafe { item.as_mut().unwrap().text().to_std_string() };
        if ref_data.contains(&text) { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::White } else { GlobalColor::Black })); } }
        else if ref_data.is_empty() { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Blue)); } }
        else { unsafe { item.as_mut().unwrap().set_foreground(&Brush::new(GlobalColor::Red)); } }
    }
}

/// This function checks if the data in the clipboard is suitable for the selected Items.
fn check_clipboard(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel,
) -> bool {

    // Get the current selection.
    let clipboard = GuiApplication::clipboard();
    let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
    let mut indexes_sorted = vec![];
    for index in 0..indexes.count(()) {
        indexes_sorted.push(indexes.at(index))
    }

    // Sort the indexes so they follow the visual index, not their logical one. This should fix situations like copying a row and getting a different order in the cells.
    let header = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap() };
    indexes_sorted.sort_unstable_by(|a, b| {
        if a.row() == b.row() {
            if header.visual_index(a.column()) < header.visual_index(b.column()) { Ordering::Less }
            else { Ordering::Greater }
        } 
        else if a.row() < b.row() { Ordering::Less }
        else { Ordering::Greater }
    });

    // If there is nothing selected, don't waste your time.
    if indexes_sorted.is_empty() { return false }

    // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
    if text.ends_with('\n') { text.pop(); }
    let text = text.replace('\n', "\t");
    let text = text.split('\t').collect::<Vec<&str>>();

    // Get the list of items selected in a format we can deal with easely.
    let mut items = vec![];
    for model_index in &indexes_sorted {
        if model_index.is_valid() {
            unsafe { items.push(model.as_mut().unwrap().item_from_index(&model_index)); }
        }
    }

    // If none of the items are valid, stop.
    if items.is_empty() { return false }

    // Zip together both vectors.
    let data = items.iter().zip(text);
    for cell in data {

        // Depending on the column, we try to encode the data in one format or another.
        let column = unsafe { cell.0.as_mut().unwrap().index().column() };
        match definition.fields[column as usize].field_type {
            FieldType::Boolean =>  if cell.1.to_lowercase() != "true" && cell.1.to_lowercase() != "false" && cell.1 != "1" && cell.1 != "0" { return false },
            FieldType::Float => if cell.1.parse::<f32>().is_err() { return false },
            FieldType::Integer => if cell.1.parse::<i32>().is_err() { return false },
            FieldType::LongInteger => if cell.1.parse::<i64>().is_err() { return false },

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

/// This function checks if the data in the clipboard is suitable for be pasted in all selected cells.
fn check_clipboard_to_fill_selection(
    definition: &TableDefinition,
    table_view: *mut TableView,
    model: *mut StandardItemModel,
    filter_model: *mut SortFilterProxyModel,
) -> bool {

    // Get the current selection.
    let clipboard = GuiApplication::clipboard();
    let text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };
    let indexes = unsafe { filter_model.as_mut().unwrap().map_selection_to_source(&table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };

    // If there is nothing selected, don't waste your time.
    if indexes.count(()) == 0 { return false }

    // For each selected index...
    for index in 0..indexes.count(()) {
        let model_index = indexes.at(index);
        if model_index.is_valid() {

            // Depending on the column, we try to encode the data in one format or another.
            let item = unsafe { model.as_mut().unwrap().item_from_index(&model_index) };
            let column = unsafe { item.as_mut().unwrap().index().column() };
            match definition.fields[column as usize].field_type {
                FieldType::Boolean => if text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != "1" && text != "0" { return false },
                FieldType::Float => if text.parse::<f32>().is_err() { return false },
                FieldType::Integer => if text.parse::<i32>().is_err() { return false },
                FieldType::LongInteger => if text.parse::<i64>().is_err() { return false },

                // All these are Strings, so we can skip their checks....
                FieldType::StringU8 |
                FieldType::StringU16 |
                FieldType::OptionalStringU8 |
                FieldType::OptionalStringU16 => {}
            }
        }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}

/// This function checks if the data in the clipboard is suitable to be appended as rows at the end of the Table.
fn check_clipboard_append_rows(
    table_view: *mut TableView,
    definition: &TableDefinition
) -> bool {

    // Get the text from the clipboard.
    let clipboard = GuiApplication::clipboard();
    let mut text = unsafe { clipboard.as_mut().unwrap().text(()).to_std_string() };

    // If the text ends in \n, remove it. Excel things. We don't use newlines, so replace them with '\t'.
    if text.ends_with('\n') { text.pop(); }
    let text = text.replace('\n', "\t");
    let text = text.split('\t').collect::<Vec<&str>>();

    // Get the index for the column.
    let mut column = 0;
    for cell in text {

        // Depending on the column, we try to encode the data in one format or another.
        let column_logical_index = unsafe { table_view.as_ref().unwrap().horizontal_header().as_ref().unwrap().logical_index(column) };
        match definition.fields[column_logical_index as usize].field_type {
            FieldType::Boolean => if cell.to_lowercase() != "true" && cell.to_lowercase() != "false" && cell != "1" && cell != "0" { return false },
            FieldType::Float => if cell.parse::<f32>().is_err() { return false },
            FieldType::Integer => if cell.parse::<i32>().is_err() { return false },
            FieldType::LongInteger => if cell.parse::<i64>().is_err() { return false },

            // All these are Strings, so we can skip their checks....
            FieldType::StringU8 |
            FieldType::StringU16 |
            FieldType::OptionalStringU8 |
            FieldType::OptionalStringU16 => {}
        }

        // Reset or increase the column count, if needed.
        if column as usize == definition.fields.len() - 1 { column = 0; } else { column += 1; }
    }

    // If we reach this place, it means none of the cells was incorrect, so we can paste.
    true
}

/// Function to filter the table. If a value is not provided by a slot, we get it from the widget itself.
fn filter_table(
    pattern: Option<QString>,
    column: Option<i32>,
    case_sensitive: Option<bool>,
    filter_model: *mut SortFilterProxyModel,
    filter_line_edit: *mut LineEdit,
    column_selector: *mut ComboBox,
    case_sensitive_button: *mut PushButton,
    update_search_stuff: *mut Action,
    packed_file_path: &Rc<RefCell<Vec<String>>>,
) {

    // Set the pattern to search.
    let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
    else { unsafe { RegExp::new(&filter_line_edit.as_mut().unwrap().text()) }};

    // Set the column selected.
    if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
    else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

    // Check if the filter should be "Case Sensitive".
    if let Some(case_sensitive) = case_sensitive { 
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
    }

    else {
        let case_sensitive = unsafe { case_sensitive_button.as_mut().unwrap().is_checked() };
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
    }

    // Filter whatever it's in that column by the text we got.
    unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }

    // Update the search stuff, if needed.
    unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

    // Add the new filter data to the state history.
    if let Some(state) = TABLE_STATES_UI.lock().unwrap().get_mut(&*packed_file_path.borrow()) {
        unsafe { state.filter_state = FilterState::new(filter_line_edit.as_mut().unwrap().text().to_std_string(), column_selector.as_mut().unwrap().current_index(), case_sensitive_button.as_mut().unwrap().is_checked()); }
    }
}
