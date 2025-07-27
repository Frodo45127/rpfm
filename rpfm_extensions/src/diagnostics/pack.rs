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

use rpfm_lib::files::{EncodeableExtraData, pack::Pack};

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
    MissingLocDataFileDetected(String),
    FileITM(String),
    FileOverwrite(String),
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
            PackDiagnosticReportType::FileITM(path) => format!("File identical to parent/vanilla file: {path}"),
            PackDiagnosticReportType::FileOverwrite(path) => format!("File overwriting a parent/vanilla file: {path}"),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PackDiagnosticReportType::InvalidPackName(_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::InvalidFileName(_,_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::MissingLocDataFileDetected(_) => DiagnosticLevel::Warning,
            PackDiagnosticReportType::FileITM(_) => DiagnosticLevel::Warning,
            PackDiagnosticReportType::FileOverwrite(_) => DiagnosticLevel::Info,
        }
    }
}

impl Display for PackDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidPackName(_) => "InvalidPackFileName",
            Self::InvalidFileName(_,_) => "InvalidFileName",
            Self::MissingLocDataFileDetected(_) => "MissingLocDataFileDetected",
            Self::FileITM(_) => "FileITM",
            Self::FileOverwrite(_) => "FileOverwrite",
        }, f)
    }
}

impl PackDiagnostic {

    /// This function takes care of checking for PackFile-Related for errors.
    pub fn check(pack: &mut Pack, dependencies: &mut Dependencies, game: &GameInfo) -> Option<DiagnosticType> {
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
            .map(|(path, real_paths)| (path, path.split("/"), real_paths))
            .filter(|(_, split, _)| split.clone()
                .last()
                .unwrap_or_default()
                .contains(INVALID_CHARACTERS_WINDOWS))
            .filter_map(|(_, _, real_paths)| real_paths.first())
            .collect::<Vec<_>>();

        for path in invalid_file_names {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidFileName(name.to_string(), path.to_string()));
            diagnostic.results_mut().push(result);
        }

        let extra_data = Some(EncodeableExtraData::new_from_game_info(game));
        let real_paths = pack.paths()
            .values()
            .flatten()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        for path in &real_paths {
            if let Some(rfile) = pack.file_mut(path, true) {
                if let Ok(dep_file) = dependencies.file_mut(path, true, true) {

                    let mut itm = false;
                    if let Ok(local_hash) = rfile.data_hash(&extra_data) {
                        if let Ok(dependency_hash) = dep_file.data_hash(&extra_data) {
                            if local_hash == dependency_hash {
                                let result = PackDiagnosticReport::new(PackDiagnosticReportType::FileITM(rfile.path_in_container_raw().to_string()));
                                diagnostic.results_mut().push(result);
                                itm = true;
                            }
                        }
                    }

                    // To avoid duplicated reports, only mark as overwrites those not already marked as ITM, as ITM is really a subset of overwrites
                    // and would generate confusing duplicate reports otherwise.
                    if !itm {
                        let result = PackDiagnosticReport::new(PackDiagnosticReportType::FileOverwrite(rfile.path_in_container_raw().to_string()));
                        diagnostic.results_mut().push(result);
                    }
                }
            }
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Pack(diagnostic))
        } else { None }
    }

}
