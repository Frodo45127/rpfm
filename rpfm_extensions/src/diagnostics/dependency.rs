//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Dependency` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_lib::files::pack::RESERVED_NAME_DEPENDENCIES_MANAGER;

use crate::diagnostics::DiagnosticReport;
use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Dependency diagnostic.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct DependencyDiagnostic {
    path: String,
    results: Vec<DependencyDiagnosticReport>
}

/// This struct defines an individual dependency diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct DependencyDiagnosticReport {

    /// List of cells, in "row, column" format.
    ///
    /// If the full row or full column are affected, use -1.
    cells_affected: Vec<(i32, i32)>,
    report_type: DependencyDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyDiagnosticReportType {
    InvalidDependencyPackName(String)
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Default for DependencyDiagnostic {
    fn default() -> Self {
        Self {
            path: RESERVED_NAME_DEPENDENCIES_MANAGER.to_owned(),
            results: vec![],
        }
    }
}

impl DependencyDiagnosticReport {
    pub fn new(report_type: DependencyDiagnosticReportType, cells_affected: &[(i32, i32)]) -> Self {
        Self {
            cells_affected: cells_affected.to_vec(),
            report_type
        }
    }
}

impl DiagnosticReport for DependencyDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            DependencyDiagnosticReportType::InvalidDependencyPackName(pack_name) => format!("Invalid dependency Pack name: {}", pack_name),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            DependencyDiagnosticReportType::InvalidDependencyPackName(_) => DiagnosticLevel::Error,
        }
    }
}

impl Display for DependencyDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidDependencyPackName(_) => "InvalidPackName",
        }, f)
    }
}
