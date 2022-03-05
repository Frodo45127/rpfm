//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `Diagnostics`.

This module contains the code needed to get a `Diagnostics` over an entire `PackFile`.
!*/

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over a single PackedFile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableDiagnostic {
    path: Vec<String>,
    result: Vec<TableDiagnosticReport>
}

/// This struct defines an individual diagnostic result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiagnosticReport {

    /// List of cells, in "row, column" format. If the full row or full column are affected, use -1.
    pub cells_affected: Vec<(i32, i32)>,
    pub message: String,
    pub report_type: TableDiagnosticReportType,
    pub level: DiagnosticLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableDiagnosticReportType {
    OutdatedTable,
    InvalidReference,
    EmptyRow,
    EmptyKeyField,
    EmptyKeyFields,
    DuplicatedCombinedKeys,
    NoReferenceTableFound,
    NoReferenceTableNorColumnFoundPak,
    NoReferenceTableNorColumnFoundNoPak,
    InvalidEscape,
    DuplicatedRow,
    InvalidLocKey,
    TableNameEndsInNumber,
    TableNameHasSpace,
    TableIsDataCoring,
    FieldWithPathNotFound,
    BannedTable,
    ValueCannotBeEmpty,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TableDiagnostic`.
impl TableDiagnostic {
    pub fn new(path: &[String]) -> Self {
        Self {
            path: path.to_vec(),
            result: vec![],
        }
    }

    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    pub fn get_ref_result(&self) -> &[TableDiagnosticReport] {
        &self.result
    }

    pub fn get_ref_mut_result(&mut self) -> &mut Vec<TableDiagnosticReport> {
        &mut self.result
    }
}

impl Display for TableDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::OutdatedTable => "OutdatedTable",
            Self::InvalidReference => "InvalidReference",
            Self::EmptyRow => "EmptyRow",
            Self::EmptyKeyField => "EmptyKeyField",
            Self::EmptyKeyFields => "EmptyKeyFields",
            Self::DuplicatedCombinedKeys => "DuplicatedCombinedKeys",
            Self::NoReferenceTableFound => "NoReferenceTableFound",
            Self::NoReferenceTableNorColumnFoundPak => "NoReferenceTableNorColumnFoundPak",
            Self::NoReferenceTableNorColumnFoundNoPak => "NoReferenceTableNorColumnFoundNoPak",
            Self::InvalidEscape => "InvalidEscape",
            Self::DuplicatedRow => "DuplicatedRow",
            Self::InvalidLocKey => "InvalidLocKey",
            Self::TableNameEndsInNumber => "TableNameEndsInNumber",
            Self::TableNameHasSpace => "TableNameHasSpace",
            Self::TableIsDataCoring => "TableIsDataCoring",
            Self::FieldWithPathNotFound => "FieldWithPathNotFound",
            Self::BannedTable => "BannedTable",
            Self::ValueCannotBeEmpty => "ValueCannotBeEmpty",
        }, f)
    }
}
