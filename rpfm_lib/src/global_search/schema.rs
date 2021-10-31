//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a Schema.
#[derive(Debug, Clone)]
pub struct SchemaMatches {

    // The type of versioned file we have.
    pub versioned_file_type: String,

    // The name of the versioned file, for versioned files that have it.
    pub versioned_file_name: Option<String>,

    /// The list of matches within the versioned file.
    pub matches: Vec<SchemaMatch>,
}

/// This struct represents a match on a column name within a Schema.
#[derive(Debug, Clone)]
pub struct SchemaMatch {

    // Version of the definition with a match.
    pub version: i32,

    // Column of the match.
    pub column: u32,

    // Full name of the matched column.
    pub name: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SchemaMatches`.
impl SchemaMatches {

    /// This function creates a new `SchemaMatches` for the provided path.
    pub fn new(versioned_file_type: String, versioned_file_name: Option<String>) -> Self {
        Self {
            versioned_file_type,
            versioned_file_name,
            matches: vec![],
        }
    }
}

/// Implementation of `SchemaMatch`.
impl SchemaMatch {

    /// This function creates a new `SchemaMatch` with the provided data.
    pub fn new(version: i32, column: u32, name: String) -> Self {
        Self {
            version,
            column,
            name,
        }
    }
}
