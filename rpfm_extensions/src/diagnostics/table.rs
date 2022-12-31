//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Table` diagnostics.

use getset::{Getters, MutGetters};
use itertools::Itertools;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use crate::diagnostics::DiagnosticReport;
use super::DiagnosticLevel;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Table diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableDiagnostic {
    path: String,
    results: Vec<TableDiagnosticReport>
}

/// This struct defines an individual table diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableDiagnosticReport {

    /// List of cells, in "row, column" format.
    ///
    /// If the full row or full column are affected, use -1.
    cells_affected: Vec<(i32, i32)>,
    report_type: TableDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableDiagnosticReportType {
    OutdatedTable,
    InvalidReference(String, String),
    EmptyRow,
    EmptyKeyField(String),
    EmptyKeyFields,
    DuplicatedCombinedKeys(String),
    NoReferenceTableFound(String),
    NoReferenceTableNorColumnFoundPak(String),
    NoReferenceTableNorColumnFoundNoPak(String),
    InvalidEscape,
    DuplicatedRow(String),
    InvalidLocKey,
    TableNameEndsInNumber,
    TableNameHasSpace,
    TableIsDataCoring,
    FieldWithPathNotFound(Vec<String>),
    BannedTable,
    ValueCannotBeEmpty(String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl TableDiagnostic {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            results: vec![],
        }
    }
}

impl TableDiagnosticReport {
    pub fn new(report_type: TableDiagnosticReportType, cells_affected: &[(i32, i32)]) -> Self {
        Self {
            cells_affected: cells_affected.to_vec(),
            report_type
        }
    }
}

impl DiagnosticReport for TableDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            TableDiagnosticReportType::OutdatedTable => "Possibly outdated table".to_owned(),
            TableDiagnosticReportType::InvalidReference(cell_data, field_name) => format!("Invalid reference \"{}\" in column \"{}\".", cell_data, field_name),
            TableDiagnosticReportType::EmptyRow => "Empty row.".to_owned(),
            TableDiagnosticReportType::EmptyKeyField(field_name) => format!("Empty key for column \"{}\".", field_name),
            TableDiagnosticReportType::EmptyKeyFields => "Empty key fields.".to_owned(),
            TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys) => format!("Duplicated combined keys: {}.", &combined_keys),
            TableDiagnosticReportType::NoReferenceTableFound(field_name) => format!("No reference table found for column \"{}\".", field_name),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(field_name) => format!("No reference column found in referenced table for column \"{}\". Maybe a problem with the schema?", field_name),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(field_name) => format!("No reference column found in referenced table for column \"{}\". Did you forgot to generate the Dependencies Cache, or did you generated it before installing the Assembly kit?", field_name),
            TableDiagnosticReportType::InvalidEscape => "Invalid line jump/tabulation detected in loc entry. Use \\\\n or \\\\t instead.".to_owned(),
            TableDiagnosticReportType::DuplicatedRow(combined_keys) => format!("Duplicated row: {}.", combined_keys),
            TableDiagnosticReportType::InvalidLocKey => "Invalid localisation key.".to_owned(),
            TableDiagnosticReportType::TableNameEndsInNumber => "Table name ends in number.".to_owned(),
            TableDiagnosticReportType::TableNameHasSpace => "Table name contains spaces.".to_owned(),
            TableDiagnosticReportType::TableIsDataCoring => "Table is datacoring.".to_owned(),
            TableDiagnosticReportType::FieldWithPathNotFound(paths) => format!("Path not found: {}.", paths.iter().join(" || ")),
            TableDiagnosticReportType::BannedTable => "Banned table.".to_owned(),
            TableDiagnosticReportType::ValueCannotBeEmpty(field_name) => format!("Empty value for column \"{}\".", field_name),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            TableDiagnosticReportType::OutdatedTable => DiagnosticLevel::Error,
            TableDiagnosticReportType::InvalidReference(_,_) => DiagnosticLevel::Error,
            TableDiagnosticReportType::EmptyRow => DiagnosticLevel::Error,
            TableDiagnosticReportType::EmptyKeyField(_) => DiagnosticLevel::Warning,
            TableDiagnosticReportType::EmptyKeyFields => DiagnosticLevel::Warning,
            TableDiagnosticReportType::DuplicatedCombinedKeys(_) => DiagnosticLevel::Error,
            TableDiagnosticReportType::NoReferenceTableFound(_) => DiagnosticLevel::Info,
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(_) => DiagnosticLevel::Info,
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(_) => DiagnosticLevel::Warning,
            TableDiagnosticReportType::InvalidEscape => DiagnosticLevel::Warning,
            TableDiagnosticReportType::DuplicatedRow(_) => DiagnosticLevel::Warning,
            TableDiagnosticReportType::InvalidLocKey => DiagnosticLevel::Error,
            TableDiagnosticReportType::TableNameEndsInNumber => DiagnosticLevel::Error,
            TableDiagnosticReportType::TableNameHasSpace => DiagnosticLevel::Error,
            TableDiagnosticReportType::TableIsDataCoring => DiagnosticLevel::Warning,
            TableDiagnosticReportType::FieldWithPathNotFound(_) => DiagnosticLevel::Warning,
            TableDiagnosticReportType::BannedTable => DiagnosticLevel::Error,
            TableDiagnosticReportType::ValueCannotBeEmpty(_) => DiagnosticLevel::Error,
        }
    }
}

impl Display for TableDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::OutdatedTable => "OutdatedTable",
            Self::InvalidReference(_,_) => "InvalidReference",
            Self::EmptyRow => "EmptyRow",
            Self::EmptyKeyField(_) => "EmptyKeyField",
            Self::EmptyKeyFields => "EmptyKeyFields",
            Self::DuplicatedCombinedKeys(_) => "DuplicatedCombinedKeys",
            Self::NoReferenceTableFound(_) => "NoReferenceTableFound",
            Self::NoReferenceTableNorColumnFoundPak(_) => "NoReferenceTableNorColumnFoundPak",
            Self::NoReferenceTableNorColumnFoundNoPak(_) => "NoReferenceTableNorColumnFoundNoPak",
            Self::InvalidEscape => "InvalidEscape",
            Self::DuplicatedRow(_) => "DuplicatedRow",
            Self::InvalidLocKey => "InvalidLocKey",
            Self::TableNameEndsInNumber => "TableNameEndsInNumber",
            Self::TableNameHasSpace => "TableNameHasSpace",
            Self::TableIsDataCoring => "TableIsDataCoring",
            Self::FieldWithPathNotFound(_) => "FieldWithPathNotFound",
            Self::BannedTable => "BannedTable",
            Self::ValueCannotBeEmpty(_) => "ValueCannotBeEmpty",
        }, f)
    }
}
