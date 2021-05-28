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

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over a single PackedFile.
#[derive(Debug, Clone, Default)]
pub struct DependencyManagerDiagnostic {
    result: Vec<DependencyManagerDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone)]
pub struct DependencyManagerDiagnosticReport {
    pub cells_affected: Vec<(i32, i32)>,
    pub message: String,
    pub report_type: DependencyManagerDiagnosticReportType,
    pub level: DiagnosticLevel,
}

#[derive(Debug, Clone)]
pub enum DependencyManagerDiagnosticReportType {
    InvalidDependencyPackFileName
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DependencyManagerDiagnostic`.
impl DependencyManagerDiagnostic {
    pub fn new() -> Self {
        Self {
            result: vec![],
        }
    }

    pub fn get_ref_result(&self) -> &[DependencyManagerDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<DependencyManagerDiagnosticReport> {
        &mut self.result
    }
}

impl Display for DependencyManagerDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidDependencyPackFileName => "InvalidPackFileName",
        }, f)
    }
}
