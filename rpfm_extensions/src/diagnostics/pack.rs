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

use rpfm_lib::{files::pack::Pack, games::supported_games::KEY_WARHAMMER_3};

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
    InvalidPackName(String),
    UpperCaseScriptOrTableFileName(String),
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
            PackDiagnosticReportType::UpperCaseScriptOrTableFileName(file_name) => format!("Script or table with uppercase in Pack: {file_name}"),
            PackDiagnosticReportType::MissingLocDataFileDetected(pack_name) => format!("Missing Loc Data file in Pack: {pack_name}"),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PackDiagnosticReportType::InvalidPackName(_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::UpperCaseScriptOrTableFileName(_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::MissingLocDataFileDetected(_) => DiagnosticLevel::Warning,
        }
    }
}

impl Display for PackDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidPackName(_) => "InvalidPackFileName",
            Self::UpperCaseScriptOrTableFileName(_) => "UpperCaseScriptOrTableFileName",
            Self::MissingLocDataFileDetected(_) => "MissingLocDataFileDetected",
        }, f)
    }
}

impl PackDiagnostic {

    /// This function takes care of checking for PackFile-Related for errors.
    pub fn check(pack: &Pack, game_info: &GameInfo) -> Option<DiagnosticType> {
        let mut diagnostic = PackDiagnostic::default();

        let name = pack.disk_file_name();
        if name.contains(' ') {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidPackName(name));
            diagnostic.results_mut().push(result);
        }

        if game_info.key() == KEY_WARHAMMER_3 {
            diagnostic.results_mut().extend_from_slice(&mut pack.paths()
                .par_iter()
                .filter(|(x, _)| x.starts_with("db/") || x.starts_with("script/"))
                .map(|(_, x)| x.to_owned())
                .flatten()
                .filter_map(|x| {
                    let vec = x.split('/').map(|x| x.to_owned()).collect::<Vec<_>>();
                    let last = vec.last().map(|y| (y.to_owned(), x));
                    last
                })
                .filter(|(y, _)| y.chars().any(|z| z.is_uppercase()))
                .map(|(_, x)| PackDiagnosticReport::new(PackDiagnosticReportType::UpperCaseScriptOrTableFileName(x.to_string())))
                .collect::<Vec<_>>()
            );
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

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Pack(diagnostic))
        } else { None }
    }

}
