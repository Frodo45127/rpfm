//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `Diagnostics`.

This module contains the code needed to get a `Diagnostics` over the current RPFM's config.
!*/

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over RPFM's config.
#[derive(Debug, Clone, Default)]
pub struct ConfigDiagnostic {
    result: Vec<ConfigDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone)]
pub struct ConfigDiagnosticReport {
    pub message: String,
    pub level: DiagnosticLevel,
    pub report_type: ConfigDiagnosticReportType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigDiagnosticReportType {
    DependenciesCacheNotGenerated,
    DependenciesCacheOutdated,
    DependenciesCacheCouldNotBeLoaded(String)
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ConfigDiagnostic`.
impl ConfigDiagnostic {
    pub fn new() -> Self {
        Self {
            result: vec![],
        }
    }

    pub fn get_ref_result(&self) -> &[ConfigDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<ConfigDiagnosticReport> {
        &mut self.result
    }
}

impl Display for ConfigDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::DependenciesCacheNotGenerated => "DependenciesCacheNotGenerated",
            Self::DependenciesCacheOutdated => "DependenciesCacheOutdated",
            Self::DependenciesCacheCouldNotBeLoaded(_) => "DependenciesCacheCouldNotBeLoaded",
        }, f)
    }
}
