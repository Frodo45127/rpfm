//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Usually this kind of stuff goes into the background thread, but this is only used in the UI. And I'm tired, so this'll stay here for the moment.

use qt_gui::standard_item_model::StandardItemModel;

use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;

use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};

use rpfm_lib::config::get_config_path;
use crate::TABLE_STATES_UI;
use rpfm_error::Result;
use crate::ui::packedfile_table::TableOperations;

/// Name of the file to load/save from.
const TABLES_STATE_FILE: &str = "table_state.json";

/// This struct keeps the current state of the "configurable" stuff from a TableView.
/// - Filter: Keeps the `String` used for the filter, the column filtered and if it's case sensitive or not.
/// - Search: Keeps the `String` used search, the `String` used to replace, the column filtered, if it's case sensitive or not and the currently selected match.
/// - Columns: Keeps the order the user sets for the columns.
#[derive(Clone, Serialize, Deserialize)]
pub struct TableStateUI {
    pub filter_state: FilterState,
    pub search_state: SearchState,
    pub columns_state: ColumnsState,
}

/// This Struct stores the last state of the filter of a TableView.
#[derive(Clone, Serialize, Deserialize)]
pub struct FilterState {
    pub text: String,
    pub column: i32,
    pub is_case_sensitive: bool
}

/// This Struct stores the last state of the search widget of a TableView.
#[derive(Clone, Serialize, Deserialize)]
pub struct SearchState {
    pub search_text: String,
    pub replace_text: String,
    pub column: i32,
    pub is_case_sensitive: bool,
}

/// This Struct stores the last state of the columns of a TableView. For sorting_column, no order is 0, ascending is 1, descending is 2.
/// - visual_history: a BTreeMap of all columns, with their logical position as key and a list of all his known positions listed in chronological order.
#[derive(Clone, Serialize, Deserialize)]
pub struct ColumnsState {
    pub sorting_column: (i32, i8),
    pub visual_history: Vec<VisualHistory>,
}

/// This struct stores the "data" changes of a table, like the undo/redo history, and the painted cells.
pub struct TableStateData {
    pub undo_history: Vec<TableOperations>,
    pub redo_history: Vec<TableOperations>,
    pub undo_model: *mut StandardItemModel,
}

/// This enum stores the visual changes of the columns of a table, such us:
/// - ColumnMoved: Keeps track of every moved column, including his visual indexes before and after movement.
/// - ColumnFrozen: Keeps track of every frozen column, including if it's being frozen or not, the logical index, and the visual index before/after the movement.
/// - ColumnHidden: Keeps track of every hidden column, including if it's being hidden or not, and his logical index.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VisualHistory {
    ColumnMoved(i32, i32),
    ColumnFrozen(bool, i32, i32),
    ColumnHidden(bool, i32)
}

/// Implementation of TableState.
impl TableStateUI {

    /// This function creates a BTreeMap with the different TableStates needed for RPFM.
    pub fn new() -> BTreeMap<Vec<String>, Self> {
        BTreeMap::new()
    }

    /// This function creates a single empty TableState.
    pub fn new_empty() -> Self {
        Self {
            filter_state: FilterState::new(String::new(), 0, false),
            search_state: SearchState::new(String::new(), String::new(), 0, false),
            columns_state: ColumnsState::new((-1, 0), vec![]),
        }
    }

    /// This function takes a table_state.json file and reads it into a "TableState" object.
    pub fn load() -> Result<BTreeMap<Vec<String>, Self>> {

        let path = get_config_path()?.join(TABLES_STATE_FILE);
        let file = BufReader::new(File::open(path)?);
        let states: BTreeMap<String, Self> = serde_json::from_reader(file)?;

        // We need to process the states because serde only admits Strings as key.
        let mut states_processed = BTreeMap::new();
        for entry in states { states_processed.insert(entry.0.split('\\').map(|x| x.to_owned()).collect(), entry.1); }
        Ok(states_processed)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save() -> Result<()> {

        // Try to open the settings file.
        let path = get_config_path()?.join(TABLES_STATE_FILE);
        let mut file = BufWriter::new(File::create(path)?);

        // Same than when loading. We have to process the states to make them compatible with serde.
        let history = &*TABLE_STATES_UI.lock().unwrap();
        let mut states_processed = BTreeMap::new();
        for entry in history { states_processed.insert(entry.0.join("\\"), entry.1); }
        let states = serde_json::to_string_pretty(&states_processed);
        file.write_all(states.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }

    /*/// This function encodes the provided TableStateUI into his item.
    pub fn to_item(&self, item: *mut StandardItem) {
        let data = QString::from_std_str(&serde_json::to_string(self).unwrap());
        unsafe { item.as_mut().unwrap().set_data((&Variant::new0(&data), 30)) };
    }

    /// This function decodes the TableStateUI from an item and returns it.
    pub fn from_item(item: *mut StandardItem) -> Self {
        let data = unsafe { item.as_mut().unwrap().data(30).to_string().to_std_string() };
        if data.is_empty() { Self::new_empty() }
        else { serde_json::from_str(&data).unwrap() }
    }*/
}

/// Implementation of FilterState.
impl FilterState {

    /// This function creates the FilterState of a TableView.
    pub fn new(text: String, column: i32, is_case_sensitive: bool) -> Self {
        Self {
            text,
            column,
            is_case_sensitive
        }
    }
}

/// Implementation of SearchState.
impl SearchState {

    /// This function creates the SearchState of a TableView.
    pub fn new(search_text: String, replace_text: String, column: i32, is_case_sensitive: bool) -> Self {
        Self {
            search_text,
            replace_text,
            column,
            is_case_sensitive
        }
    }
}

/// Implementation of ColumnsState.
impl ColumnsState {

    /// This function creates the ColumnState of a TableView.
    pub fn new(sorting_column: (i32, i8), visual_history: Vec<VisualHistory>) -> Self {
        Self {
            sorting_column,
            visual_history,
        }
    }
}

/// Implementation of TableStateData.
impl TableStateData {

    /// This function creates a BTreeMap with the different TableStates needed for RPFM.
    pub fn new() -> BTreeMap<Vec<String>, Self> {
        BTreeMap::new()
    }

    /// This function creates a single empty TableStateData.
    pub fn new_empty() -> Self {
        Self {
            undo_history: vec![],
            redo_history: vec![],
            undo_model: StandardItemModel::new(()).into_raw(),
        }
    }
}
