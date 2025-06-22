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

const INVALID_CHARACTERS_WINDOWS: [char; 9] = [
    '<',
    '>',
    ':',
    '"',
    '/',
    '\\',
    '|',
    '?',
    '*',
];

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
    InvalidPackName(String),
    InvalidFileName(String, String),
    MissingLocDataFileDetected(String)
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
            PackDiagnosticReportType::InvalidFileName(pack_name, file_name) => format!("Invalid file name ({file_name}) in pack: {pack_name}"),
            PackDiagnosticReportType::MissingLocDataFileDetected(pack_name) => format!("Missing Loc Data file in Pack: {pack_name}"),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PackDiagnosticReportType::InvalidPackName(_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::InvalidFileName(_,_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::MissingLocDataFileDetected(_) => DiagnosticLevel::Warning,
        }
    }
}

impl Display for PackDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidPackName(_) => "InvalidPackFileName",
            Self::InvalidFileName(_,_) => "InvalidFileName",
            Self::MissingLocDataFileDetected(_) => "MissingLocDataFileDetected",
        }, f)
    }
}

impl PackDiagnostic {

    /// This function takes care of checking for PackFile-Related for errors.
    pub fn check(pack: &Pack) -> Option<DiagnosticType> {
        let mut diagnostic = PackDiagnostic::default();

        let name = pack.disk_file_name();
        if name.contains(' ') {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidPackName(name.to_string()));
            diagnostic.results_mut().push(result);
        }

        let (existing, new) = pack.missing_locs_paths();
        if pack.paths().contains_key(&existing) {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::MissingLocDataFileDetected(existing));
            diagnostic.results_mut().push(result);
        }

        if pack.paths().contains_key(&new) {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::MissingLocDataFileDetected(new));
            diagnostic.results_mut().push(result);
        }

        let invalid_file_names = pack.paths().par_iter()
            .filter(|(path, _)| path.contains(INVALID_CHARACTERS_WINDOWS))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        for path in invalid_file_names {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidFileName(name.to_string(), path.to_string()));
            diagnostic.results_mut().push(result);
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Pack(diagnostic))
        } else { None }
    }

}
