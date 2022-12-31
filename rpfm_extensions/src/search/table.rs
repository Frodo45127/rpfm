//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `TableMatches`.

This module contains the code needed to get table matches from a `GlobalSearch`.
!*/

use getset::{Getters, MutGetters};
use regex::Regex;

use rpfm_lib::files::{db::DB, loc::Loc, table::DecodedData};
use rpfm_lib::schema::Field;

use super::{MatchingMode, Replaceable, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a table.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableMatches {

    /// The path of the table.
    path: String,

    /// The list of matches within a table.
    matches: Vec<TableMatch>,
}

/// This struct represents a match on a row of a Table PackedFile (DB & Loc).
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableMatch {

    // The name of the column where the match is.
    column_name: String,

    // The logical index of the column where the match is. This should be -1 when the column is hidden.
    column_number: u32,

    // The row number of this match. This should be -1 when the row is hidden by a filter.
    row_number: i64,

    // The contents of the matched cell.
    contents: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for DB {
    type SearchMatches = TableMatches;

    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(file_path);

        if let Ok(table_data) = self.data(&None) {
            let fields_processed = self.definition().fields_processed();

            for (row_number, row) in table_data.iter().enumerate() {
                for (column_number, cell) in row.iter().enumerate() {
                    matches.match_decoded_data(&cell.data_to_string(), pattern_to_search, case_sensitive, matching_mode, &fields_processed, column_number as u32, row_number as i64);
                }
            }
        }

        matches
    }
}

impl Searchable for Loc {
    type SearchMatches = TableMatches;

    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(file_path);

        if let Ok(table_data) = self.data(&None) {
            let fields_processed = self.definition().fields_processed();

            for (row_number, row) in table_data.iter().enumerate() {
                for (column_number, cell) in row.iter().enumerate() {
                    matches.match_decoded_data(&cell.data_to_string(), pattern_to_search, case_sensitive, matching_mode, &fields_processed, column_number as u32, row_number as i64);
                }
            }
        }
        matches
    }
}

impl Replaceable for DB {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &TableMatches) -> bool {
        let mut edited = false;

        if let Ok(data) = self.data(&None) {
            let mut data = data.to_vec();
            for search_match in search_matches.matches() {
                edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, &mut data);
            }

            let _ = self.set_data(None, &data);
        }

        edited
    }
}

impl Replaceable for Loc {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &TableMatches) -> bool {
        let mut edited = false;

        if let Ok(data) = self.data(&None) {
            let mut data = data.to_vec();
            for search_match in search_matches.matches() {
                edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, &mut data);
            }

            let _ = self.set_data(&data);
        }

        edited
    }
}

/// Implementation of `TableMatches`.
impl TableMatches {

    /// This function creates a new `TableMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }

    /// This function check if the provided `&str` matches our search.
    fn match_decoded_data(
        &mut self,
        text: &str,
        pattern: &str,
        case_sensitive: bool,
        matching_mode: &MatchingMode,
        fields_processed: &[Field],
        column_number: u32,
        row_number: i64,
    ) {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if regex.is_match(text) {
                    let column_name = fields_processed[column_number as usize].name();
                    self.matches.push(TableMatch::new(column_name, column_number, row_number, text));
                }
            }

            MatchingMode::Pattern => {
                if case_sensitive {
                    if text.contains(pattern) {
                        let column_name = fields_processed[column_number as usize].name();
                        self.matches.push(TableMatch::new(column_name, column_number, row_number, text));
                    }
                }
                else {
                    let text_lower = text.to_lowercase();
                    if text_lower.contains(pattern) {
                        let column_name = fields_processed[column_number as usize].name();
                        self.matches.push(TableMatch::new(column_name, column_number, row_number, text));
                    }
                }
            }
        }
    }
}

/// Implementation of `TableMatch`.
impl TableMatch {

    /// This function creates a new `TableMatch` with the provided data.
    pub fn new(column_name: &str, column_number: u32, row_number: i64, contents: &str) -> Self {
        Self {
            column_name: column_name.to_owned(),
            column_number,
            row_number,
            contents: contents.to_owned(),
        }
    }

    /// This function replaces all the matches in the provided text.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut [Vec<DecodedData>]) -> bool {
        let mut edited = false;

        if let Some(row) = data.get_mut(self.row_number as usize) {
            if let Some(cell) = row.get_mut(self.column_number as usize) {
                let previous_data = cell.data_to_string().to_string();

                match matching_mode {
                    MatchingMode::Regex(regex) => {
                        if regex.is_match(&cell.data_to_string()) {
                            let _ = cell.set_data(&regex.replace_all(&previous_data, replace_pattern));
                        }
                    }
                    MatchingMode::Pattern => {
                        let mut text = cell.data_to_string().to_string();
                        if case_sensitive {
                            let mut index = 0;
                            while let Some(start) = text.find(pattern) {

                                // Advance the index so we don't get trapped in an infinite loop... again.
                                if start >= index {
                                    let end = start + pattern.len();
                                    text.replace_range(start..end, replace_pattern);
                                    index = end;
                                } else {
                                    break;
                                }
                            }
                        }
                        else {

                            let regex = Regex::new(&format!("(?i){}", regex::escape(pattern))).unwrap();
                            let mut index = 0;
                            while let Some(match_data) = regex.find(&text.to_owned()) {

                                 // Advance the index so we don't get trapped in an infinite loop... again.
                                if match_data.start() >= index {
                                    text.replace_range(match_data.start()..match_data.end(), replace_pattern);
                                    index = match_data.end();
                                } else {
                                    break;
                                }
                            }
                        }

                        let _ = cell.set_data(&text);
                    }
                }

                if previous_data != cell.data_to_string() {
                    edited = true;
                }
            }
        }

        edited
    }
}
