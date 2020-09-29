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
Module with all the code related to the `Diagnostics`.

This module contains the code needed to get a `Diagnostics` over an entire `PackFile`.
!*/

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over a single PackedFile.
#[derive(Debug, Clone)]
pub struct TableDiagnostic {
    path: Vec<String>,
    result: Vec<TableDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone)]
pub struct TableDiagnosticReport {
    pub column_number: u32,
    pub row_number: i64,
    pub message: String,
    pub level: DiagnosticLevel,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TableDiagnostic`.
impl TableDiagnostic {
    pub fn new(path: &[String]) -> Self {
        Self {
            path: path.to_vec(),
            result: vec![],
        }
    }

    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    pub fn get_ref_result(&self) -> &[TableDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<TableDiagnosticReport> {
        &mut self.result
    }
}
