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

#[derive(Debug, Clone)]
pub struct PackFileDiagnostic {
    path: Vec<String>,
    result: Vec<PackFileDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone)]
pub struct PackFileDiagnosticReport {
    pub message: String,
    pub level: DiagnosticLevel,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackedFileDiagnostic`.
impl PackFileDiagnostic {
    pub fn new() -> Self {
        Self {
            path: vec![],
            result: vec![],
        }
    }

    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    pub fn get_ref_result(&self) -> &[PackFileDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<PackFileDiagnosticReport> {
        &mut self.result
    }
}
