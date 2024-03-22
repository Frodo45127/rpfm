//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Pack` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_lib::files::pack::Pack;

use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Pack diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PackDiagnostic {
    results: Vec<PackDiagnosticReport>
}

/// This struct defines an individual pack diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PackDiagnosticReport {
    report_type: PackDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackDiagnosticReportType {
    InvalidPackName(String)
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackDiagnosticReport {
    pub fn new(report_type: PackDiagnosticReportType) -> Self {
        Self {
            report_type
        }
    }
}

impl DiagnosticReport for PackDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            PackDiagnosticReportType::InvalidPackName(pack_name) => format!("Invalid Pack name: {pack_name}"),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PackDiagnosticReportType::InvalidPackName(_) => DiagnosticLevel::Error,
        }
    }
}

impl Display for PackDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidPackName(_) => "InvalidPackFileName",
        }, f)
    }
}

impl PackDiagnostic {

    /// This function takes care of checking for PackFile-Related for errors.
    pub fn check(pack: &Pack) -> Option<DiagnosticType> {
        let mut diagnostic = PackDiagnostic::default();

        let name = pack.disk_file_name();
        if name.contains(' ') {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidPackName(name));
            diagnostic.results_mut().push(result);
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Pack(diagnostic))
        } else { None }
    }

}
