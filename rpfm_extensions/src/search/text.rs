//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use getset::{Getters, MutGetters, Setters};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use rpfm_lib::files::text::Text;

use super::{find_in_string, MatchingMode, Replaceable, SearchSource, Searchable, replace_match_string};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a text PackedFile.
#[derive(Debug, Clone, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TextMatches {

    /// The path of the file.
    path: String,

    /// The search source that produced these matches.
    #[serde(default)]
    source: SearchSource,

    /// The container name (pack file name) this file belongs to.
    #[serde(default)]
    container_name: String,

    /// The list of matches within the file.
    matches: Vec<TextMatch>,

    /// List of matched strings, so they can be shared between matches to reduce ram usage.
    matches_strings: Vec<String>,
}

/// This struct represents a match on a piece of text within a Text PackedFile.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TextMatch {

    /// Row of the first character of the match.
    row: u64,

    /// Byte where the match starts.
    start: usize,

    /// Byte where the match ends.
    end: usize,

    /// Index of the line of text containing the match.
    text_index: usize,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for Text {
    type SearchMatches = TextMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> TextMatches {
        let mut matches = TextMatches::new(file_path);

        for (row, data) in self.contents().lines().enumerate() {
            let mut added = false;

            match matching_mode {
                MatchingMode::Regex(regex) => {
                    for match_data in regex.find_iter(data) {
                        matches.matches.push(
                            TextMatch::new(
                                row as u64,
                                match_data.start(),
                                match_data.end(),
                                if !added {
                                    matches.matches_strings.len()
                                } else {
                                    matches.matches_strings.len() - 1
                                },
                            )
                        );

                        if !added {
                            matches.matches_strings.push(data.to_owned());
                            added = true;
                        }
                    }
                }

                // If we're searching a pattern, we just check every text PackedFile, line by line.
                MatchingMode::Pattern(regex) => {
                    for (start, end, _) in &find_in_string(data, pattern, case_sensitive, regex) {
                        matches.matches.push(
                            TextMatch::new(
                                row as u64,
                                *start,
                                *end,
                                if !added {
                                    matches.matches_strings.len()
                                } else {
                                    matches.matches_strings.len() - 1
                                },
                            )
                        );

                        if !added {
                            matches.matches_strings.push(data.to_owned());
                            added = true;
                        }
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for Text {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &TextMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self.contents_mut());
        }

        edited
    }
}

impl TextMatches {

    /// This function creates a new `TextMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
            matches_strings: vec![],
            source: SearchSource::default(),
            container_name: String::new(),
        }
    }
}

impl TextMatch {

    /// This function creates a new `TextMatch` with the provided data.
    pub fn new(row: u64, start: usize, end: usize, text_index: usize) -> Self {
        Self {
            row,
            start,
            end,
            text_index,
        }
    }

    /// This function replaces all the matches in the provided text.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut String) -> bool {
        let mut edited = false;

        let new_data = data.lines()
            .enumerate()
            .map(|(row, line)| {
                if self.row == row as u64 {
                    let (previous_data, mut current_data) = (line, line.to_owned());
                    edited |= replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, previous_data, &mut current_data);
                    current_data
                } else {
                    line.to_owned()
                }
            }).join("\n");

        if edited {
            *data = new_data;
        }

        edited
    }
}
