//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters};
use regex::bytes::RegexBuilder;
use serde_derive::{Deserialize, Serialize};

use rpfm_lib::files::rigidmodel::RigidModel;

use super::{find_in_bytes, MatchingMode, Replaceable, Searchable, replace_match_bytes};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an RigidModel File.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<RigidModelMatch>,
}

/// This struct represents a match within an RigidModel File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatch {

    /// First Byte index of the match.
    pos: usize,

    /// Length of the matched pattern, in bytes.
    len: usize,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for RigidModel {
    type SearchMatches = RigidModelMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> RigidModelMatches {
        let mut matches = RigidModelMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {

                // We can assume that, if the original regex was valid, this one is too.
                let regex = RegexBuilder::new(regex.as_str()).case_insensitive(!case_sensitive).build().unwrap();
                for match_data in regex.find_iter(self.data()) {
                    matches.matches.push(
                        RigidModelMatch::new(
                            match_data.start(),
                            match_data.end() - match_data.start(),
                        )
                    );
                }
            }

            MatchingMode::Pattern(regex) => {
                let regex = regex.as_ref().map(|regex| RegexBuilder::new(regex.as_str()).case_insensitive(!case_sensitive).build().unwrap());

                if self.data().len() > pattern.len() {
                    for (start, length) in &find_in_bytes(self.data(), pattern, case_sensitive, &regex) {
                        matches.matches.push(RigidModelMatch::new(*start, *length));
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for RigidModel {

    fn replace(&mut self, _pattern: &str, replace_pattern: &str, _case_sensitive: bool, _matching_mode: &MatchingMode, search_matches: &RigidModelMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(replace_pattern, self.data_mut());
        }

        edited
    }
}

impl RigidModelMatches {

    /// This function creates a new `RigidModelMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl RigidModelMatch {

    /// This function creates a new `RigidModelMatch` with the provided data.
    pub fn new(pos: usize, len: usize) -> Self {
        Self {
            pos,
            len,
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, replace_pattern: &str, data: &mut Vec<u8>) -> bool {
        replace_match_bytes(replace_pattern, self.pos, self.len, data)
    }
}
