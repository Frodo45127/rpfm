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
Module with all the code related to the `TableMatches`.

This module contains the code needed to get table matches from a `GlobalSeach`.
!*/

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a table.
#[derive(Debug, Clone)]
pub struct TableMatches {

    /// The path of the table.
    pub path: Vec<String>,

    /// The list of matches whithin a table.
    pub matches: Vec<TableMatch>,
}

/// This struct represents a match on a row of a Table PackedFile (DB & Loc).
#[derive(Debug, Clone)]
pub struct TableMatch {

    // The name of the column where the match is.
    pub column_name: String,

    // The logical index of the column where the match is. This should be -1 when the column is hidden.
    pub column_number: u32,

    // The row number of this match. This should be -1 when the row is hidden by a filter.
    pub row_number: i64,

    // The contents of the matched cell.
    pub contents: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TableMatches`.
impl TableMatches {

    /// This function creates a new `TableMatches` for the provided path.
    pub fn new(path: &[String]) -> Self {
        Self {
            path: path.to_vec(),
            matches: vec![],
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
}
