//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Pack` diagnostics.

use getset::{Getters, MutGetters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_lib::files::{EncodeableExtraData, pack::Pack, RFile};
use rpfm_lib::utils::INVALID_CHARACTERS_WINDOWS;

use crate::diagnostics::*;



//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Pack diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PackDiagnostic {
    pack: String,
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
    FileDuplicated(String),
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
            PackDiagnosticReportType::InvalidFileName(pack_name, file_name) => format!("Invalid file name ({file_name}) in pack: {pack_name}. This file will be renamed when extracting/exporting, which may cause issues when importing back, especially with MyMods."),
            PackDiagnosticReportType::MissingLocDataFileDetected(pack_name) => format!("Missing Loc Data file in Pack: {pack_name}"),
            PackDiagnosticReportType::FileITM(path) => format!("File identical to parent/vanilla file: {path}"),
            PackDiagnosticReportType::FileOverwrite(path) => format!("File overwriting a parent/vanilla file: {path}"),
            PackDiagnosticReportType::FileDuplicated(path) => format!("File duplicated: {path}"),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PackDiagnosticReportType::InvalidPackName(_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::InvalidFileName(_,_) => DiagnosticLevel::Error,
            PackDiagnosticReportType::MissingLocDataFileDetected(_) => DiagnosticLevel::Warning,
            PackDiagnosticReportType::FileITM(_) => DiagnosticLevel::Warning,
            PackDiagnosticReportType::FileOverwrite(_) => DiagnosticLevel::Info,
            PackDiagnosticReportType::FileDuplicated(_) => DiagnosticLevel::Warning,
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
            Self::FileDuplicated(_) => "FileDuplicated",
        }, f)
    }
}

impl PackDiagnostic {

    /// This function takes care of checking for PackFile-Related for errors.
    pub fn check(packs: &mut BTreeMap<String, Pack>, dependencies: &mut Dependencies, game: &GameInfo) -> Vec<DiagnosticType> {
        let mut diagnostics = Vec::new();

        let extra_data = Some(EncodeableExtraData::new_from_game_info(game));

        for (key, pack) in packs.iter_mut() {
            let mut diagnostic = PackDiagnostic { pack: key.clone(), ..Default::default() };

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

            let results = pack.paths()
                .values()
                .filter_map(|x| if x.len() >= 2 {
                    Some(x.iter().map(|x| PackDiagnosticReport::new(PackDiagnosticReportType::FileDuplicated(x.to_string()))).collect::<Vec<_>>())
                } else {
                    None
                })
                .flatten()
                .collect::<Vec<_>>();

            if !results.is_empty() {
                diagnostic.results_mut().extend(results);
            }

            let invalid_file_names = pack.paths().par_iter()
                .map(|(path, real_paths)| (path, path.split("/"), real_paths))
                .filter(|(_, split, _)| {
                    let filename = split.clone().last().unwrap_or_default();
                    let has_invalid_chars = filename.chars().any(|c| INVALID_CHARACTERS_WINDOWS.contains(&c));
                    let has_whitespace_issues = filename.starts_with(' ') || filename.ends_with(' ');
                    let is_only_dots = !filename.is_empty() && filename.chars().all(|c| c == '.');

                    has_invalid_chars || has_whitespace_issues || is_only_dots
                })
                .filter_map(|(_, _, real_paths)| real_paths.first())
                .collect::<Vec<_>>();

            for path in invalid_file_names {
                let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidFileName(name.to_string(), path.to_string()));
                diagnostic.results_mut().push(result);
            }

            // ITM / overwrite pass. The expensive bit is `data_hash` (encode /
            // disk read), so we collect disjoint `&mut RFile` pairs and parallelise.
            let candidate_paths: HashSet<String> = pack.files().keys()
                .filter(|k| dependencies.file(k, true, true, false).is_ok())
                .cloned()
                .collect();

            if !candidate_paths.is_empty() {

                // One-shot batch: `Dependencies::file_mut` in a loop would
                // reborrow `&mut self` each call and invalidate prior refs.
                let mut dep_files = dependencies.files_mut_by_paths(&candidate_paths, true, true);

                // `HashMap::remove` moves the `&mut RFile` out so each pair
                // owns two disjoint refs, safe to send across rayon workers.
                let pairs: Vec<(&mut RFile, &mut RFile)> = pack.files_mut().iter_mut()
                    .filter_map(|(k, pack_rfile)| {
                        dep_files.remove(k).map(|dep_rfile| (pack_rfile, dep_rfile))
                    })
                    .collect();

                // Parallel hash + compare. Each worker owns a disjoint pair so
                // the two `data_hash` calls can run concurrently with the rest.
                let reports: Vec<PackDiagnosticReport> = pairs.into_par_iter()
                    .filter_map(|(rfile, dep_file)| {
                        let local_hash = rfile.data_hash(&extra_data).ok()?;
                        let dependency_hash = dep_file.data_hash(&extra_data).ok()?;
                        let path = rfile.path_in_container_raw().to_string();
                        let report_type = if local_hash == dependency_hash {
                            PackDiagnosticReportType::FileITM(path)
                        } else {
                            PackDiagnosticReportType::FileOverwrite(path)
                        };
                        Some(PackDiagnosticReport::new(report_type))
                    })
                    .collect();

                diagnostic.results_mut().extend(reports);
            }

            if !diagnostic.results().is_empty() {
                diagnostics.push(DiagnosticType::Pack(diagnostic));
            }
        }

        diagnostics
    }

}
