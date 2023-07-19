//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters};

use rpfm_lib::files::atlas::Atlas;

use super::{find_in_string, MatchingMode, replace_match_string, Replaceable, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an Atlas File.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AtlasMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<AtlasMatch>,
}

/// This struct represents a match within an Atlas File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AtlasMatch {

    /// The name of the column where the match is.
    column_name: String,

    /// The logical index of the column where the match is.
    column_number: u32,

    /// The row number of this match.
    row_number: i64,

    /// Byte where the match starts.
    start: usize,

    /// Byte where the match ends.
    end: usize,

    /// The contents of the matched cell.
    text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for Atlas {
    type SearchMatches = AtlasMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> AtlasMatches {
        let mut matches = AtlasMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (row, entry) in self.entries().iter().enumerate() {
                    for entry_match in regex.find_iter(entry.string1()) {
                        matches.matches.push(
                            AtlasMatch::new(
                                "String1",
                                0,
                                row as i64,
                                entry_match.start(),
                                entry_match.end(),
                                entry.string1(),
                            )
                        );
                    }

                    for entry_match in regex.find_iter(entry.string2()) {
                        matches.matches.push(
                            AtlasMatch::new(
                                "String2",
                                0,
                                row as i64,
                                entry_match.start(),
                                entry_match.end(),
                                entry.string2(),
                            )
                        );
                    }
                }
            }

            MatchingMode::Pattern(regex) => {
                let pattern = if case_sensitive || regex.is_some() {
                    pattern.to_owned()
                } else {
                    pattern.to_lowercase()
                };

                for (row, entry) in self.entries().iter().enumerate() {
                    for (start, end, _) in &find_in_string(entry.string1(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AtlasMatch::new(
                                "String1",
                                0,
                                row as i64,
                                *start,
                                *end,
                                entry.string1(),
                            )
                        );
                    }

                    for (start, end, _) in &find_in_string(entry.string2(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AtlasMatch::new(
                                "String2",
                                1,
                                row as i64,
                                *start,
                                *end,
                                entry.string2(),
                            )
                        );
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for Atlas {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &AtlasMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self);
        }

        edited
    }
}

impl AtlasMatches {

    /// This function creates a new `AtlasMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl AtlasMatch {

    /// This function creates a new `AtlasMatch` with the provided data.
    pub fn new(column_name: &str, column_number: u32, row_number: i64, start: usize, end: usize, contents: &str) -> Self {
        Self {
            column_name: column_name.to_owned(),
            column_number,
            row_number,
            start,
            end,
            text: contents.to_owned(),
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut Atlas) -> bool {
        let mut edited = false;

        if let Some(entry) = data.entries_mut().get_mut(self.row_number as usize) {

            // Get all the previous data and references of data to manipulate here, so we don't duplicate a lot of code per-field in the match mode part.
            let (previous_data, current_data) = {
                if self.column_number == 0 {
                    (entry.string1().to_owned(), entry.string1_mut())
                } else if self.column_number == 1 {
                    (entry.string2().to_owned(), entry.string2_mut())
                }

                // This is an error.
                else {
                    return false
                }
            };

            edited = replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, &previous_data, current_data);
        }

        edited
    }
}
