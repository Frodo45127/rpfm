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

use rpfm_lib::files::rigidmodel::RigidModel;

use super::{MatchingMode, Replaceable, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an RigidModel File.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<RigidModelMatch>,
}

/// This struct represents a match within an RigidModel File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatch {

    /// First Byte index of the match.
    pos: u64,

    /// Length of the matched pattern, in bytes.
    len: i64,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for RigidModel {
    type SearchMatches = RigidModelMatches;

    fn search(&self, file_path: &str, pattern: &str, _case_sensitive: bool, matching_mode: &MatchingMode) -> RigidModelMatches {
        let mut matches = RigidModelMatches::new(file_path);

        // We do not care about case sensitivity here, as this is a byte search, not a text search.
        match matching_mode {
            MatchingMode::Regex(regex) => {
                for match_data in regex::bytes::Regex::new(regex.as_str()).unwrap().find_iter(self.data()) {
                    matches.matches.push(
                        RigidModelMatch::new(
                            match_data.start() as u64,
                            (match_data.end() - match_data.start()) as i64,
                        )
                    );
                }
            }

            MatchingMode::Pattern => {
                let length = pattern.len();

                if self.data().len() > length {
                    for index in 0..self.data().len() - length {
                        if &self.data()[index..index + length] == pattern.as_bytes() {
                            matches.matches.push(RigidModelMatch::new(index as u64, length as i64));
                        }
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
    pub fn new(pos: u64, len: i64) -> Self {
        Self {
            pos,
            len,
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, replace_pattern: &str, data: &mut Vec<u8>) -> bool {
        let mut edited = false;
        let old_data = data.to_vec();
        data.splice(self.pos as usize..self.pos as usize + self.len as usize, replace_pattern.as_bytes().to_vec());

        if old_data != *data {
            edited = true;
        }

        edited
    }
}
