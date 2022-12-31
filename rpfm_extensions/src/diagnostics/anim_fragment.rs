//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `AnimFragment` diagnostics.

use getset::{Getters, MutGetters};
use itertools::Itertools;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use super::{DiagnosticLevel, DiagnosticReport};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of an anim fragment diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentDiagnostic {
    path: String,
    results: Vec<AnimFragmentDiagnosticReport>
}

/// This struct defines an individual anim fragment diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentDiagnosticReport {

    /// List of cells, in "row, column" format.
    ///
    /// If the full row or full column are affected, use -1.
    cells_affected: Vec<(i32, i32)>,
    report_type: AnimFragmentDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimFragmentDiagnosticReportType {
    FieldWithPathNotFound(Vec<String>),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AnimFragmentDiagnostic {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            results: vec![],
        }
    }
}

impl AnimFragmentDiagnosticReport {
    pub fn new(report_type: AnimFragmentDiagnosticReportType, cells_affected: &[(i32, i32)]) -> Self {
        Self {
            cells_affected: cells_affected.to_vec(),
            report_type
        }
    }
}

impl DiagnosticReport for AnimFragmentDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            AnimFragmentDiagnosticReportType::FieldWithPathNotFound(paths) => format!("Path not found: {}.", paths.iter().join(" || ")),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            AnimFragmentDiagnosticReportType::FieldWithPathNotFound(_) => DiagnosticLevel::Warning,
        }
    }
}

impl Display for AnimFragmentDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::FieldWithPathNotFound(_) => "FieldWithPathNotFound"

        }, f)
    }
}
