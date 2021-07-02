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

This module contains the code needed to get a `Diagnostics` over animfragments.
!*/

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over an animfragment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimFragmentDiagnostic {
    path: Vec<String>,
    result: Vec<AnimFragmentDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimFragmentDiagnosticReport {

    /// List of cells, in "row, column" format. If the full row or full column are affected, use -1.
    pub cells_affected: Vec<(i32, i32)>,
    pub message: String,
    pub report_type: AnimFragmentDiagnosticReportType,
    pub level: DiagnosticLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimFragmentDiagnosticReportType {
    FieldWithPathNotFound,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AnimFragmentDiagnostic`.
impl AnimFragmentDiagnostic {
    pub fn new(path: &[String]) -> Self {
        Self {
            path: path.to_vec(),
            result: vec![],
        }
    }

    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    pub fn get_ref_result(&self) -> &[AnimFragmentDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<AnimFragmentDiagnosticReport> {
        &mut self.result
    }
}

impl Display for AnimFragmentDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::FieldWithPathNotFound => "FieldWithPathNotFound"
        }, f)
    }
}
