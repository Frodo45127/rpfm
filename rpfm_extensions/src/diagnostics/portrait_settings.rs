//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `PortraitSettings` diagnostics.

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

/// This struct contains the results of a PortraitSettings diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PortraitSettingsDiagnostic {
    path: String,
    results: Vec<PortraitSettingsDiagnosticReport>
}

/// This struct defines an individual PortraitSettings diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PortraitSettingsDiagnosticReport {
    report_type: PortraitSettingsDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortraitSettingsDiagnosticReportType {
    DatacoredPortraitSettings,
    InvalidArtSetId(String),
    InvalidVariantFilename(String, String),
    FileDiffuseNotFoundForVariant(String, String, String),
    FileMask1NotFoundForVariant(String, String, String),
    FileMask2NotFoundForVariant(String, String, String),
    FileMask3NotFoundForVariant(String, String, String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PortraitSettingsDiagnosticReport {
    pub fn new(report_type: PortraitSettingsDiagnosticReportType) -> Self {
        Self {
            report_type
        }
    }
}

impl DiagnosticReport for PortraitSettingsDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings => "Datacored Portrait Settings file.".to_string(),
            PortraitSettingsDiagnosticReportType::InvalidArtSetId(art_set_id) => format!("Invalid Art Set Id '{art_set_id}' in Portrait Settings file."),
            PortraitSettingsDiagnosticReportType::InvalidVariantFilename(art_set_id, variant_filename) => format!("Invalid Variant Filename '{variant_filename}' for Art Set Id '{art_set_id}'. "),
            PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(art_set_id, variant_filename, path) => format!("File not found for Art Set Id '{art_set_id}', Variant Filename '{variant_filename}', File Diffuse '{path}'."),
            PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(art_set_id, variant_filename, path) => format!("File not found for Art Set Id '{art_set_id}', Variant Filename '{variant_filename}', File Mask 1 '{path}'."),
            PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(art_set_id, variant_filename, path) => format!("File not found for Art Set Id '{art_set_id}', Variant Filename '{variant_filename}', File Mask 2 '{path}'."),
            PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(art_set_id, variant_filename, path) => format!("File not found for Art Set Id '{art_set_id}', Variant Filename '{variant_filename}', File Mask 3 '{path}'."),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings => DiagnosticLevel::Warning,
            PortraitSettingsDiagnosticReportType::InvalidArtSetId(_) => DiagnosticLevel::Error,
            PortraitSettingsDiagnosticReportType::InvalidVariantFilename(_, _) => DiagnosticLevel::Error,
            PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(_, _, _) => DiagnosticLevel::Error,
            PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(_, _, _) => DiagnosticLevel::Error,
            PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(_, _, _) => DiagnosticLevel::Error,
            PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(_, _, _) => DiagnosticLevel::Error,
        }
    }
}

impl Display for PortraitSettingsDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::DatacoredPortraitSettings => "DatacoredPortraitSettings",
            Self::InvalidArtSetId(_) => "InvalidArtSetId",
            Self::InvalidVariantFilename(_, _) => "InvalidVariantFilename",
            Self::FileDiffuseNotFoundForVariant(_, _, _) => "FileDiffuseNotFoundForVariant",
            Self::FileMask1NotFoundForVariant(_, _, _) => "FileMask1NotFoundForVariant",
            Self::FileMask2NotFoundForVariant(_, _, _) => "FileMask2NotFoundForVariant",
            Self::FileMask3NotFoundForVariant(_, _, _) => "FileMask3NotFoundForVariant",
        }, f)
    }
}

impl PortraitSettingsDiagnostic {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            results: vec![],
        }
    }

    /// This function takes care of checking for PortraitSettings-Related for errors.
    pub fn check(
        file: &RFile,
        art_set_ids: &HashSet<String>,
        variant_filenames: &HashSet<String>,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        local_path_list: &HashMap<String, Vec<String>>,
    ) -> Option<DiagnosticType> {
        if let Ok(RFileDecoded::PortraitSettings(portrait_settings)) = file.decoded() {
            let mut diagnostic = Self::new(file.path_in_container_raw());
            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("DatacoredPortraitSettings"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && dependencies.file_exists(file.path_in_container_raw(), true, false, false)  {
                let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings);
                diagnostic.results_mut().push(result);
            }

            for entry in portrait_settings.entries() {
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("InvalidArtSetId"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && art_set_ids.get(entry.id()).is_none()  {
                    let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::InvalidArtSetId(entry.id().to_owned()));
                    diagnostic.results_mut().push(result);
                }

                for variant in entry.variants() {
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("InvalidVariantFilename"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && variant_filenames.get(variant.filename()).is_none()  {
                        let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::InvalidVariantFilename(entry.id().to_owned(), variant.filename().to_owned()));
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("FileDiffuseNotFoundForVariant"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                        (
                            local_path_list.get(&variant.file_diffuse().to_lowercase()).is_none() &&
                            !dependencies.file_exists(variant.file_diffuse(), true, true, true)
                        ) && !variant.file_diffuse().is_empty() {
                        let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(entry.id().to_owned(), variant.filename().to_owned(), variant.file_diffuse().to_owned()));
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("FileMask1NotFoundForVariant"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                        (
                            local_path_list.get(&variant.file_mask_1().to_lowercase()).is_none() &&
                            !dependencies.file_exists(variant.file_mask_1(), true, true, true)
                        ) && !variant.file_mask_1().is_empty() {
                        let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(entry.id().to_owned(), variant.filename().to_owned(), variant.file_mask_1().to_owned()));
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("FileMask2NotFoundForVariant"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                        (
                            local_path_list.get(&variant.file_mask_2().to_lowercase()).is_none() &&
                            !dependencies.file_exists(variant.file_mask_2(), true, true, true)
                        ) && !variant.file_mask_2().is_empty() {
                        let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(entry.id().to_owned(), variant.filename().to_owned(), variant.file_mask_2().to_owned()));
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("FileMask3NotFoundForVariant"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                        (
                            local_path_list.get(&variant.file_mask_3().to_lowercase()).is_none() &&
                            !dependencies.file_exists(variant.file_mask_3(), true, true, true)
                        ) && !variant.file_mask_3().is_empty() {
                        let result = PortraitSettingsDiagnosticReport::new(PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(entry.id().to_owned(), variant.filename().to_owned(), variant.file_mask_3().to_owned()));
                        diagnostic.results_mut().push(result);
                    }
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::PortraitSettings(diagnostic))
            } else { None }
        } else { None }
    }
}
