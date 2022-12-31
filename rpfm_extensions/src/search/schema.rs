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
Module with all the code related to the `SchemaMatches`.

This module contains the code needed to get schema matches from a `GlobalSearch`.
!*/

use getset::{Getters, MutGetters};

use rpfm_lib::schema::Schema;

use super::{MatchingMode, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a Schema.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SchemaMatches {

    /// The list of matches within the versioned file.
    matches: Vec<SchemaMatch>,
}

/// This struct represents a match on a column name within a Schema.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SchemaMatch {

    // The type of versioned file we have.
    table_name: String,

    // Version of the definition with a match.
    version: i32,

    // Column of the match.
    column: u32,

    // Full name of the matched column.
    column_name: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for Schema {
    type SearchMatches = SchemaMatches;

    /// This function performs a search over the provided Text PackedFile.
    fn search(&self, _file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> SchemaMatches {
        let mut matches = SchemaMatches::new();

        for (table_name, definitions) in self.definitions() {
            match matching_mode {
                MatchingMode::Regex(regex) => {
                    for definition in definitions {
                        for (index, field) in definition.fields_processed().iter().enumerate() {
                            if regex.is_match(field.name()) {
                                matches.matches.push(SchemaMatch::new(
                                    table_name,
                                    *definition.version(),
                                    index as u32,
                                    field.name()
                                ));
                            }
                        }
                    }
                }

                // If we're searching a pattern, we just check every text PackedFile, line by line.
                MatchingMode::Pattern => {
                    let pattern = if case_sensitive { pattern_to_search.to_owned() } else { pattern_to_search.to_lowercase() };
                    for definition in definitions {
                        for (index, field) in definition.fields_processed().iter().enumerate() {
                            if case_sensitive {
                                if field.name().contains(&pattern) {
                                    matches.matches.push(SchemaMatch::new(
                                        table_name,
                                        *definition.version(),
                                        index as u32,
                                        field.name()
                                    ));
                                }
                            }
                            else {
                                let name = field.name().to_lowercase();
                                if name.contains(&pattern) {
                                    matches.matches.push(SchemaMatch::new(
                                        table_name,
                                        *definition.version(),
                                        index as u32,
                                        field.name()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        matches
    }
}

/// Implementation of `SchemaMatches`.
impl SchemaMatches {

    /// This function creates a new `SchemaMatches` for the provided path.
    pub fn new() -> Self {
        Self {
            matches: vec![],
        }
    }
}

/// Implementation of `SchemaMatch`.
impl SchemaMatch {

    /// This function creates a new `SchemaMatch` with the provided data.
    pub fn new(table_name: &str, version: i32, column: u32, column_name: &str) -> Self {
        Self {
            table_name: table_name.to_owned(),
            version,
            column,
            column_name: column_name.to_owned(),
        }
    }
}
