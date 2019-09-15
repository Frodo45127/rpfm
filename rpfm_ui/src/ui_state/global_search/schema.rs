//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `SchemaMatches`.

This module contains the code needed to get schema matches from a `GlobalSeach`.
!*/

use rpfm_lib::schema::VersionedFile;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a Schema.
#[derive(Debug, Clone)]
pub struct SchemaMatches {

    /// The version file the matches are in.
    pub versioned_file: VersionedFile,

    /// The list of matches whithin the versioned file.
    pub matches: Vec<SchemaMatch>,
}

/// This struct represents a match on a column name within a Schema.
#[derive(Debug, Clone)]
pub struct SchemaMatch {

    // Column of the match.
    pub column: u32,

    // Version of the definition with a match.
    pub version: i32,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SchemaMatches`.
impl SchemaMatches {

    /// This function creates a new `SchemaMatches` for the provided path.
    pub fn new(versioned_file: &VersionedFile) -> Self {
        Self {
            versioned_file: versioned_file.clone(),
            matches: vec![],
        }
    }
}

/// Implementation of `SchemaMatch`.
impl SchemaMatch {

    /// This function creates a new `SchemaMatch` with the provided data.
    pub fn new(column: u32, version: i32) -> Self {
        Self {
            column,
            version,
        }
    }
}