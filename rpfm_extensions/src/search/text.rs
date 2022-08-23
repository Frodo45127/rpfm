//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `TextMatches`.

This module contains the code needed to get text matches from a `GlobalSearch`.
!*/

use getset::Getters;

use rpfm_lib::files::text::Text;

use super::{MatchingMode, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a text PackedFile.
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct TextMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<TextMatch>,
}

/// This struct represents a match on a piece of text within a Text PackedFile.
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct TextMatch {

    // Column of the first character of the match.
    column: u64,

    // Row of the first character of the match.
    row: u64,

    // Length of the matched pattern.
    len: i64,

    // Line of text containing the match.
    text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for Text {
    type SearchMatches = TextMatches;

    /// This function performs a search over the provided Text PackedFile.
    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TextMatches {

        let mut matches = TextMatches::new(file_path);
        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (row, data) in self.contents().lines().enumerate() {
                    for match_data in regex.find_iter(data) {
                        matches.matches.push(
                            TextMatch::new(
                                match_data.start() as u64,
                                row as u64,
                                (match_data.end() - match_data.start()) as i64,
                                data.to_owned()
                            )
                        );
                    }
                }
            }

            // If we're searching a pattern, we just check every text PackedFile, line by line.
            MatchingMode::Pattern => {
                let pattern = if case_sensitive { pattern_to_search.to_owned() } else { pattern_to_search.to_lowercase() };
                let length = pattern_to_search.chars().count();
                let mut column = 0;

                for (row, data) in self.contents().lines().enumerate() {
                    while let Some(text) = data.get(column..) {
                        if case_sensitive {
                            match text.find(&pattern) {
                                Some(position) => {
                                    matches.matches.push(TextMatch::new(position as u64, row as u64, length as i64, data.to_owned()));
                                    column += position + length;
                                }
                                None => break,
                            }
                        }
                        else {
                            let text = text.to_lowercase();
                            match text.find(&pattern) {
                                Some(position) => {
                                    matches.matches.push(TextMatch::new(position as u64, row as u64, length as i64, data.to_owned()));
                                    column += position + length;
                                }
                                None => break,
                            }
                        }
                    }

                    column = 0;
                }
            }
        }

        matches
    }
}

/// Implementation of `TextMatches`.
impl TextMatches {

    /// This function creates a new `TextMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

/// Implementation of `TextMatch`.
impl TextMatch {

    /// This function creates a new `TextMatch` with the provided data.
    pub fn new(column: u64, row: u64, len: i64, text: String) -> Self {
        Self {
            column,
            row,
            len,
            text,
        }
    }
}
