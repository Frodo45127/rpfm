//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `AnimFragment` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of an anim fragment diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentDiagnostic {
    path: String,
    result: Vec<AnimFragmentDiagnosticReport>
}

/// This struct defines an individual anim fragment diagnostic result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimFragmentDiagnosticReport {

    /// List of cells, in "row, column" format.
    ///
    /// If the full row or full column are affected, use -1.
    pub cells_affected: Vec<(i32, i32)>,
    pub message: String,
    pub report_type: AnimFragmentDiagnosticReportType,
    pub level: DiagnosticLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimFragmentDiagnosticReportType {
    FieldWithPathNotFound,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl AnimFragmentDiagnostic {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            result: vec![],
        }
    }
}

impl Display for AnimFragmentDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::FieldWithPathNotFound => "FieldWithPathNotFound"
        }, f)
    }
}
