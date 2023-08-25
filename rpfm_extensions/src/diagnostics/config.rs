//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Config` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};
use std::path::Path;

use rpfm_lib::games::GameInfo;

use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Config diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ConfigDiagnostic {
    results: Vec<ConfigDiagnosticReport>
}

/// This struct defines an individual config diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ConfigDiagnosticReport {
    report_type: ConfigDiagnosticReportType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigDiagnosticReportType {
    DependenciesCacheNotGenerated,
    DependenciesCacheOutdated,
    DependenciesCacheCouldNotBeLoaded(String),
    IncorrectGamePath,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ConfigDiagnosticReport {
    pub fn new(report_type: ConfigDiagnosticReportType) -> Self {
        Self {
            report_type
        }
    }
}

impl DiagnosticReport for ConfigDiagnosticReport {
    fn message(&self) -> String {
        match self.report_type {
            ConfigDiagnosticReportType::DependenciesCacheNotGenerated => "Dependency Cache not generated for the currently selected game.".to_owned(),
            ConfigDiagnosticReportType::DependenciesCacheOutdated => "Dependency Cache for the selected game is outdated and could not be loaded.".to_owned(),
            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) => "Dependency Cache couldn't be loaded for the game selected, due to errors reading the game's folder.".to_owned(),
            ConfigDiagnosticReportType::IncorrectGamePath => "Game Path for the current Game Selected is incorrect.".to_owned(),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            ConfigDiagnosticReportType::DependenciesCacheNotGenerated => DiagnosticLevel::Error,
            ConfigDiagnosticReportType::DependenciesCacheOutdated => DiagnosticLevel::Error,
            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) => DiagnosticLevel::Error,
            ConfigDiagnosticReportType::IncorrectGamePath => DiagnosticLevel::Error,
        }
    }
}

impl Display for ConfigDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::DependenciesCacheNotGenerated => "DependenciesCacheNotGenerated",
            Self::DependenciesCacheOutdated => "DependenciesCacheOutdated",
            Self::DependenciesCacheCouldNotBeLoaded(_) => "DependenciesCacheCouldNotBeLoaded",
            Self::IncorrectGamePath => "IncorrectGamePath",
        }, f)
    }
}

impl ConfigDiagnostic {

    /// This function takes care of checking RPFM's configuration for errors.
    pub fn check(dependencies: &Dependencies, game_info: &GameInfo, game_path: &Path) -> Option<DiagnosticType> {
        let mut diagnostic = ConfigDiagnostic::default();

        // First, check if we have the game folder correctly configured. We can't do anything without it.
        let exe_path = game_info.executable_path(game_path).filter(|path| path.is_file());
        if exe_path.is_none() {
            diagnostic.results_mut().push(ConfigDiagnosticReport::new(ConfigDiagnosticReportType::IncorrectGamePath));
        }

        // If we have the correct folder, check if the vanilla data of the dependencies is loaded.
        else if !dependencies.is_vanilla_data_loaded(false) {
            diagnostic.results_mut().push(ConfigDiagnosticReport::new(ConfigDiagnosticReportType::DependenciesCacheNotGenerated));
        }

        // If we have vanilla data, check if the dependencies need updating due to changes in the game files.
        else {
            match dependencies.needs_updating(game_info, game_path) {
                Ok(needs_updating) => {
                    if needs_updating {
                        diagnostic.results_mut().push(ConfigDiagnosticReport::new(ConfigDiagnosticReportType::DependenciesCacheOutdated));
                    }
                }

                Err(error) => {
                    diagnostic.results_mut().push(ConfigDiagnosticReport::new(ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error.to_string())));
                }
            }
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Config(diagnostic))
        } else { None }
    }
}
