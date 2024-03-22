//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_lib::files::{db::DB, loc::Loc, table::DecodedData};
use rpfm_lib::schema::Field;

use super::{find_in_string, MatchingMode, Replaceable, Searchable, replace_match_string};

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

    /// Byte where the match starts.
    start: usize,

    /// Byte where the match ends.
    end: usize,

    // The contents of the matched cell.
    text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for DB {
    type SearchMatches = TableMatches;

    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(file_path);

        let fields_processed = self.definition().fields_processed();

        for (row_number, row) in self.data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                matches.match_decoded_data(&cell.data_to_string(), pattern_to_search, case_sensitive, matching_mode, &fields_processed, column_number as u32, row_number as i64);
            }
        }

        matches
    }
}

impl Searchable for Loc {
    type SearchMatches = TableMatches;

    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(file_path);

        let fields_processed = self.definition().fields_processed();

        for (row_number, row) in self.data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                matches.match_decoded_data(&cell.data_to_string(), pattern_to_search, case_sensitive, matching_mode, &fields_processed, column_number as u32, row_number as i64);
            }
        }

        matches
    }
}

impl Replaceable for DB {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &TableMatches) -> bool {
        let mut edited = false;

        for search_match in search_matches.matches() {
            if let Some(row) = self.data_mut().get_mut(search_match.row_number as usize) {
                if let Some(data) = row.get_mut(search_match.column_number as usize) {
                    edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, data);
                }
            }
        }

        edited
    }
}

impl Replaceable for Loc {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &TableMatches) -> bool {
        let mut edited = false;

        for search_match in search_matches.matches() {
            if let Some(row) = self.data_mut().get_mut(search_match.row_number as usize) {
                if let Some(data) = row.get_mut(search_match.column_number as usize) {
                    edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, data);
                }
            }
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
                for entry_match in regex.find_iter(text) {
                    let column_name = fields_processed[column_number as usize].name();
                    self.matches.push(TableMatch::new(column_name, column_number, row_number, entry_match.start(), entry_match.end(), text));
                }
            }

            MatchingMode::Pattern(regex) => {
                for (start, end, _) in &find_in_string(text, pattern, case_sensitive, regex) {
                    let column_name = fields_processed[column_number as usize].name();
                    self.matches.push(TableMatch::new(column_name, column_number, row_number, *start, *end, text));
                }
            }
        }
    }
}

/// Implementation of `TableMatch`.
impl TableMatch {

    /// This function creates a new `TableMatch` with the provided data.
    pub fn new(column_name: &str, column_number: u32, row_number: i64, start: usize, end: usize, text: &str) -> Self {
        Self {
            column_name: column_name.to_owned(),
            column_number,
            row_number,
            start,
            end,
            text: text.to_owned(),
        }
    }

    /// This function replaces all the matches in the provided text.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut DecodedData) -> bool {
        let (previous_data, mut current_data) = (data.data_to_string().to_string(), data.data_to_string().to_string());
        let edited = replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, &previous_data, &mut current_data);
        data.set_data(&current_data).is_ok() && edited
    }
}
