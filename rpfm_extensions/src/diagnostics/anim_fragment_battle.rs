//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `AnimFragmentBattle` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::collections::{HashMap, HashSet};
use std::{fmt, fmt::Display};

use rpfm_lib::files::{RFile, RFileDecoded};

use crate::dependencies::Dependencies;
use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of an anim fragment battle diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentBattleDiagnostic {
    path: String,
    pack: String,
    results: Vec<AnimFragmentBattleDiagnosticReport>
}

/// This struct defines an individual anim fragment battle diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentBattleDiagnosticReport {
    locomotion_graph: bool,
    entry: Option<(usize, Option<(usize, bool, bool, bool)>)>,
    report_type: AnimFragmentBattleDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimFragmentBattleDiagnosticReportType {
    LocomotionGraphPathNotFound(String),
    FilePathNotFound(String),
    MetaFilePathNotFound(String),
    SndFilePathNotFound(String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AnimFragmentBattleDiagnosticReport {
    pub fn new(report_type: AnimFragmentBattleDiagnosticReportType, locomotion_graph: bool, entry: Option<(usize, Option<(usize, bool, bool, bool)>)>) -> Self {
        Self {
            locomotion_graph,
            entry,
            report_type
        }
    }
}

impl DiagnosticReport for AnimFragmentBattleDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(path) => format!("Locomotion Graph file not found: {path}."),
            AnimFragmentBattleDiagnosticReportType::FilePathNotFound(path) => format!("'File Path' file not found: {path}."),
            AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(path) => format!("'Meta File Path' file not found: {path}."),
            AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(path) => format!("'Snd File Path' file not found: {path}."),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(_) => DiagnosticLevel::Warning,
            AnimFragmentBattleDiagnosticReportType::FilePathNotFound(_) => DiagnosticLevel::Warning,
            AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(_) => DiagnosticLevel::Warning,
            AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(_) => DiagnosticLevel::Warning,
        }
    }
}

impl Display for AnimFragmentBattleDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::LocomotionGraphPathNotFound(_) => "LocomotionGraphPathNotFound",
            Self::FilePathNotFound(_) => "FilePathNotFound",
            Self::MetaFilePathNotFound(_) => "MetaFilePathNotFound",
            Self::SndFilePathNotFound(_) => "SndFilePathNotFound",
        }, f)
    }
}

impl AnimFragmentBattleDiagnostic {
    pub fn new(path: &str, pack: &str) -> Self {
        Self {
            path: path.to_owned(),
            pack: pack.to_owned(),
            results: vec![],
        }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    pub fn check(
        file: &RFile,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        local_path_list: &HashMap<String, Vec<String>>,
    ) ->Option<DiagnosticType> {
        if let Ok(RFileDecoded::AnimFragmentBattle(fragment)) = file.decoded() {
            let mut diagnostic = AnimFragmentBattleDiagnostic::new(file.path_in_container_raw(), file.container_name().as_deref().unwrap_or_else(|| ""));

            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("LocomotionGraphPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !fragment.locomotion_graph().is_empty() {
                let path = fragment.locomotion_graph().replace('\\', "/");
                let mut path_found = false;

                if !path_found && local_path_list.get(&path.to_lowercase()).is_some() {
                    path_found = true;
                }

                if !path_found && dependencies.file_exists(&path, true, true, true) {
                    path_found = true;
                }

                if !path_found {
                    let result = AnimFragmentBattleDiagnosticReport::new(AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(fragment.locomotion_graph().to_owned()), true, None);
                    diagnostic.results_mut().push(result);
                }
            }

            for (row, entry) in fragment.entries().iter().enumerate() {
                for (subrow, anim_ref) in entry.anim_refs().iter().enumerate() {
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("FilePathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !anim_ref.file_path().is_empty() {
                        let path = anim_ref.file_path().replace('\\', "/");
                        let mut path_found = false;

                        if !path_found && local_path_list.get(&path.to_lowercase()).is_some() {
                            path_found = true;
                        }

                        if !path_found && dependencies.file_exists(&path, true, true, true) {
                            path_found = true;
                        }

                        if !path_found {
                            let result = AnimFragmentBattleDiagnosticReport::new(AnimFragmentBattleDiagnosticReportType::FilePathNotFound(anim_ref.file_path().to_owned()), false, Some((row, Some((subrow, true, false, false)))));
                            diagnostic.results_mut().push(result);
                        }
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("MetaFilePathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !anim_ref.meta_file_path().is_empty() {
                        let path = anim_ref.meta_file_path().replace('\\', "/");
                        let mut path_found = false;

                        if !path_found && local_path_list.get(&path.to_lowercase()).is_some() {
                            path_found = true;
                        }

                        if !path_found && dependencies.file_exists(&path, true, true, true) {
                            path_found = true;
                        }

                        if !path_found {
                            let result = AnimFragmentBattleDiagnosticReport::new(AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(anim_ref.meta_file_path().to_owned()), false, Some((row, Some((subrow, false, true, false)))));
                            diagnostic.results_mut().push(result);
                        }
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("SndFilePathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !anim_ref.snd_file_path().is_empty() {
                        let path = anim_ref.snd_file_path().replace('\\', "/");
                        let mut path_found = false;

                        if !path_found && local_path_list.get(&path.to_lowercase()).is_some() {
                            path_found = true;
                        }

                        if !path_found && dependencies.file_exists(&path, true, true, true) {
                            path_found = true;
                        }

                        if !path_found {
                            let result = AnimFragmentBattleDiagnosticReport::new(AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(anim_ref.snd_file_path().to_owned()), false, Some((row, Some((subrow, false, false, true)))));
                            diagnostic.results_mut().push(result);
                        }
                    }
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::AnimFragmentBattle(diagnostic))
            } else { None }
        } else { None }
    }
}
