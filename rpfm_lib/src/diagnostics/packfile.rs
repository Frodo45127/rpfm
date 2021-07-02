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

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the PackFile-wide diagnostic results done over a PackFile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackFileDiagnostic {
    result: Vec<PackFileDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileDiagnosticReport {
    pub message: String,
    pub report_type: PackFileDiagnosticReportType,
    pub level: DiagnosticLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackFileDiagnosticReportType {
    InvalidPackFileName
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackedFileDiagnostic`.
impl PackFileDiagnostic {
    pub fn new() -> Self {
        Self {
            result: vec![],
        }
    }

    pub fn get_ref_result(&self) -> &[PackFileDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<PackFileDiagnosticReport> {
        &mut self.result
    }
}

impl Display for PackFileDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidPackFileName => "InvalidPackFileName",
        }, f)
    }
}
