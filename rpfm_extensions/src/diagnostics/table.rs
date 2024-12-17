//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_lib::schema::Field;

use crate::diagnostics::*;

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

    /// Name of the columns that corresponds to the affected cells.
    column_names: Vec<String>,
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

impl TableDiagnosticReport {
    pub fn new(report_type: TableDiagnosticReportType, cells_affected: &[(i32, i32)], fields: &[Field]) -> Self {
        let mut fields_affected = cells_affected.iter().map(|(_, column)| *column).collect::<Vec<_>>();
        fields_affected.sort();
        fields_affected.dedup();

        if fields_affected.contains(&-1) {
            fields_affected = vec![-1];
        }

        Self {
            cells_affected: cells_affected.to_vec(),
            column_names: fields_affected.iter().flat_map(|index| {
                if index == &-1 {
                    fields.iter().map(|field| field.name().to_owned()).collect()
                } else {
                    vec![fields[*index as usize].name().to_owned()]
                }
            }).collect(),
            report_type
        }
    }
}

impl DiagnosticReport for TableDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            TableDiagnosticReportType::OutdatedTable => "Possibly outdated table".to_owned(),
            TableDiagnosticReportType::InvalidReference(cell_data, field_name) => format!("Invalid reference \"{cell_data}\" in column \"{field_name}\"."),
            TableDiagnosticReportType::EmptyRow => "Empty row.".to_owned(),
            TableDiagnosticReportType::EmptyKeyField(field_name) => format!("Empty key for column \"{field_name}\"."),
            TableDiagnosticReportType::EmptyKeyFields => "Empty key fields.".to_owned(),
            TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys) => format!("Duplicated combined keys: {}.", &combined_keys),
            TableDiagnosticReportType::NoReferenceTableFound(field_name) => format!("No reference table found for column \"{field_name}\"."),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(field_name) => format!("No reference column found in referenced table for column \"{field_name}\". Maybe a problem with the schema?"),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(field_name) => format!("No reference column found in referenced table for column \"{field_name}\". Did you forget to generate the Dependencies Cache, or did you generate it before installing the Assembly kit?"),
            TableDiagnosticReportType::InvalidEscape => "Invalid line jump/tabulation detected in loc entry. Use \\\\n or \\\\t instead.".to_owned(),
            TableDiagnosticReportType::DuplicatedRow(combined_keys) => format!("Duplicated row: {combined_keys}."),
            TableDiagnosticReportType::InvalidLocKey => "Invalid localisation key.".to_owned(),
            TableDiagnosticReportType::TableNameEndsInNumber => "Table name ends in number.".to_owned(),
            TableDiagnosticReportType::TableNameHasSpace => "Table name contains spaces.".to_owned(),
            TableDiagnosticReportType::TableIsDataCoring => "Table is datacoring.".to_owned(),
            TableDiagnosticReportType::FieldWithPathNotFound(paths) => format!("Path not found: {}.", paths.iter().join(" || ")),
            TableDiagnosticReportType::BannedTable => "Banned table.".to_owned(),
            TableDiagnosticReportType::ValueCannotBeEmpty(field_name) => format!("Empty value for column \"{field_name}\"."),
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


impl TableDiagnostic {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            results: vec![],
        }
    }

    /// This function is used to check if a table is outdated or not.
    fn is_table_outdated(table_name: &str, table_version: i32, dependencies: &Dependencies) -> bool {
        if let Ok(vanilla_dbs) = dependencies.db_data(table_name, true, false) {
            if let Some(max_version) = vanilla_dbs.iter()
                .filter_map(|x| {
                    if let Ok(RFileDecoded::DB(table)) = x.decoded() {
                        Some(table.definition().version())
                    } else {
                        None
                    }
                }).max_by(|x, y| x.cmp(y)) {
                if *max_version != table_version {
                    return true
                }
            }
        }

        false
    }

    /// This function takes care of checking the db tables of your mod for errors.
    pub fn check_db(
        file: &RFile,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        game_info: &GameInfo,
        local_path_list: &HashMap<String, Vec<String>>,
        dependency_data: &HashMap<i32, TableReferences>,
        check_ak_only_refs: bool,
    ) ->Option<DiagnosticType> {
        if let Ok(RFileDecoded::DB(table)) = file.decoded() {
            let mut diagnostic = TableDiagnostic::new(file.path_in_container_raw());

            // Before anything else, check if the table is outdated.
            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("OutdatedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && Self::is_table_outdated(table.table_name(), *table.definition().version(), dependencies) {
                let result = TableDiagnosticReport::new(TableDiagnosticReportType::OutdatedTable, &[], &[]);
                diagnostic.results_mut().push(result);
            }

            // Check if it's one of the banned tables for the game selected.
            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("BannedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && game_info.is_file_banned(file.path_in_container_raw()) {
                let result = TableDiagnosticReport::new(TableDiagnosticReportType::BannedTable, &[], &[]);
                diagnostic.results_mut().push(result);
            }

            // Check if the table name has a number at the end, which causes very annoying bugs.
            if let Some(name) = file.file_name() {
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameEndsInNumber"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && (name.ends_with('0') ||
                    name.ends_with('1') ||
                    name.ends_with('2') ||
                    name.ends_with('3') ||
                    name.ends_with('4') ||
                    name.ends_with('5') ||
                    name.ends_with('6') ||
                    name.ends_with('7') ||
                    name.ends_with('8') || name.ends_with('9')) {

                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableNameEndsInNumber, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameHasSpace"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && name.contains(' ') {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableNameHasSpace, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableIsDataCoring"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    match game_info.vanilla_db_table_name_logic() {
                        VanillaDBTableNameLogic::FolderName => {
                            if table.table_name_without_tables() == file.path_in_container_split()[2] {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableIsDataCoring, &[], &[]);
                                diagnostic.results_mut().push(result);
                            }
                        }

                        VanillaDBTableNameLogic::DefaultName(ref default_name) => {
                            if name == default_name {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableIsDataCoring, &[], &[]);
                                diagnostic.results_mut().push(result);
                            }
                        }
                    }
                }
            }

            // Check all the columns with reference data.
            let fields_processed = table.definition().fields_processed();
            let patches = Some(table.definition().patches());
            let key_amount = fields_processed.iter().filter(|field| field.is_key(patches)).count();
            let table_data = table.data();
            let mut columns_without_reference_table = vec![];
            let mut columns_with_reference_table_and_no_column = vec![];
            let mut keys: HashMap<String, Vec<(i32, i32)>> = HashMap::with_capacity(table_data.len());
            let mut duplicated_combined_keys_already_marked = vec![];

            // Columns we can try to check for paths.
            let mut ignore_path_columns = vec![];
            for (column, field) in fields_processed.iter().enumerate() {
                if let Some(rel_paths) = field.filename_relative_path(patches) {
                    if rel_paths.iter().any(|path| path.contains('*')) {
                        ignore_path_columns.push(column);
                    }
                }
            }

            for (row, cells) in table_data.iter().enumerate() {
                let mut row_is_empty = true;
                let mut row_keys_are_empty = true;
                let mut row_keys: BTreeMap<i32, Cow<str>> = BTreeMap::new();
                for (column, field) in fields_processed.iter().enumerate() {

                    // Skip unused field on diagnostics.
                    if field.unused(patches) {
                        continue;
                    }

                    let cell_data = cells[column].data_to_string();

                    // Path checks.
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("FieldWithPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                        !cell_data.is_empty() &&
                        cell_data != "." &&
                        cell_data != "x" &&
                        cell_data != "false" &&
                        cell_data != "building_placeholder" &&
                        cell_data != "placeholder" &&
                        cell_data != "PLACEHOLDER" &&
                        cell_data != "placeholder.png" &&
                        cell_data != "placehoder.png" &&
                        fields_processed[column].is_filename(patches) &&
                        !ignore_path_columns.contains(&column) {

                        let mut path_found = false;
                        let relative_paths = fields_processed[column].filename_relative_path(patches);
                        let paths = if let Some(relative_paths) = relative_paths {
                            relative_paths.iter()
                                .map(|x| {
                                    let mut paths = vec![];
                                    let cell_data = cell_data.replace('\\', "/");
                                    for cell_data in cell_data.split(',') {

                                        // When analysing paths, fix the ones in older games starting with / or data/.
                                        let mut start_offset = 0;
                                        if cell_data.starts_with("/") {
                                            start_offset += 1;
                                        }
                                        if cell_data.starts_with("data/") {
                                            start_offset += 5;
                                        }

                                        paths.push(x.replace('%', &cell_data[start_offset..]));
                                    }

                                    paths
                                })
                                .flatten()
                                .collect::<Vec<_>>()
                        } else {
                            let mut paths = vec![];
                            let cell_data = cell_data.replace('\\', "/");
                            for cell_data in cell_data.split(',') {

                                // When analysing paths, fix the ones in older games starting with / or data/.
                                let mut start_offset = 0;
                                if cell_data.starts_with("/") {
                                    start_offset += 1;
                                }
                                if cell_data.starts_with("data/") {
                                    start_offset += 5;
                                }

                                paths.push(cell_data[start_offset..].to_string());
                            }

                            paths
                        };

                        for path in &paths {
                            if !path_found && local_path_list.get(&path.to_lowercase()).is_some() {
                                path_found = true;
                            }

                            if !path_found && dependencies.file_exists(path, true, true, true) {
                                path_found = true;
                            }

                            if path_found {
                                break;
                            }
                        }

                        if !path_found {
                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::FieldWithPathNotFound(paths), &[(row as i32, column as i32)], &fields_processed);
                            diagnostic.results_mut().push(result);
                        }
                    }

                    // Dependency checks.
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), None, ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && field.is_reference(patches).is_some() {
                        match dependency_data.get(&(column as i32)) {
                            Some(ref_data) => {

                                if *ref_data.referenced_column_is_localised() {
                                    // TODO: report missing loc data here.
                                }
                                /*
                                else if ref_data.referenced_table_is_ak_only {
                                    // If it's only in the AK, ignore it.
                                }*/

                                // Blue cell check. Only one for each column, so we don't fill the diagnostics with this.
                                else if ref_data.data().is_empty() {
                                    if !columns_with_reference_table_and_no_column.contains(&column) {
                                        columns_with_reference_table_and_no_column.push(column);
                                    }
                                }

                                // Check for non-empty cells with reference data, but the data in the cell is not in the reference data list.
                                else if !cell_data.is_empty() && !ref_data.data().contains_key(&*cell_data) && (!*ref_data.referenced_table_is_ak_only() || check_ak_only_refs) {

                                    // Numeric cells with 0 are "empty" references and should not be checked.
                                    let is_number = *field.field_type() == FieldType::I32 || *field.field_type() == FieldType::I64 || *field.field_type() == FieldType::OptionalI32 || *field.field_type() == FieldType::OptionalI64;
                                    let is_valid_reference = if is_number { cell_data != "0" } else { true };
                                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("InvalidReference"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && is_valid_reference {
                                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidReference(cell_data.to_string(), field.name().to_string()), &[(row as i32, column as i32)], &fields_processed);
                                        diagnostic.results_mut().push(result);
                                    }
                                }
                            }
                            None => {
                                if !columns_without_reference_table.contains(&column) {
                                    columns_without_reference_table.push(column);
                                }
                            }
                        }
                    }

                    // Check for empty keys/rows.
                    if row_is_empty && (!cell_data.is_empty() && cell_data != "false") {
                        row_is_empty = false;
                    }

                    if row_keys_are_empty && field.is_key(patches) && (!cell_data.is_empty() && cell_data != "false") {
                        row_keys_are_empty = false;
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && field.is_key(patches) && key_amount == 1 && *field.field_type() != FieldType::OptionalStringU8 && *field.field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField(field.name().to_string()), &[(row as i32, column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("ValueCannotBeEmpty"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && cell_data.is_empty() && field.cannot_be_empty(patches) {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::ValueCannotBeEmpty(field.name().to_string()), &[(row as i32, column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    if field.is_key(patches) {
                        row_keys.insert(column as i32, cell_data);
                    }
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && row_is_empty {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &fields_processed);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyKeyFields"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && row_keys_are_empty && key_amount > 1 {
                    let cells_affected = row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>();
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyFields, &cells_affected, &fields_processed);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("DuplicatedCombinedKeys"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

                    // If this returns something, it means there is a duplicate.
                    let combined_keys = row_keys.values().join("| |");
                    if let Some(old_position) = keys.insert(combined_keys.to_owned(), row_keys.keys().map(|x| (row as i32, *x)).collect()) {
                        if let Some(old_pos) = old_position.first() {

                            // Mark previous row, if not yet marked.
                            if !duplicated_combined_keys_already_marked.contains(&old_pos.0) {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys.to_string()), &old_position, &fields_processed);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_combined_keys_already_marked.contains(&(row as i32)) {
                                let cells_affected = row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>();
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys), &cells_affected, &fields_processed);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(row as i32);
                            }
                        }
                    }
                }
            }

            // Checks that only need to be done once per table.
            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                for column in &columns_without_reference_table {
                    let field_name = fields_processed[*column].name().to_string();
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableFound(field_name), &[(-1, *column as i32)], &fields_processed);
                    diagnostic.results_mut().push(result);
                }
            }
            for column in &columns_with_reference_table_and_no_column {
                if !dependencies.is_asskit_data_loaded() {
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundNoPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        let field_name = fields_processed[*column].name().to_string();
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(field_name), &[(-1, *column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }
                }
                else if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    let field_name = fields_processed[*column].name().to_string();
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(field_name), &[(-1, *column as i32)], &fields_processed);
                    diagnostic.results_mut().push(result);
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::DB(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    pub fn check_loc(
        file: &RFile,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
    ) ->Option<DiagnosticType> {
        if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
            let mut diagnostic = TableDiagnostic::new(file.path_in_container_raw());

            // Check all the columns with reference data.
            let mut keys: HashMap<String, Vec<(i32, i32)>> = HashMap::new();
            let fields = table.definition().fields_processed();
            let field_key_name = fields[0].name();
            let field_text_name = fields[1].name();
            let mut duplicated_rows_already_marked = vec![];
            let mut duplicated_combined_keys_already_marked = vec![];

            for (row, cells) in table.data().iter().enumerate() {
                let key = cells[0].data_to_string();
                let data = cells[1].data_to_string();

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("InvalidLocKey"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !key.is_empty() && (key.contains('\n') || key.contains('\t')) {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidLocKey, &[(row as i32, 0)], &fields);
                    diagnostic.results_mut().push(result);
                }

                // Only in case none of the two columns are ignored, we perform these checks.
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && key.is_empty() && data.is_empty() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &fields);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && key.is_empty() && !data.is_empty() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField("Key".to_string()), &[(row as i32, 0)], &fields);
                    diagnostic.results_mut().push(result);
                }

                // Magic Regex. It works. Don't ask why.
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("InvalidEscape"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !data.is_empty() && REGEX_INVALID_ESCAPES.is_match(&data).unwrap() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidEscape, &[(row as i32, 1)], &fields);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("DuplicatedCombinedKeys"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

                    // If this returns something, it means there is a duplicate.
                    if let Some(old_position) = keys.insert(key.to_string(), vec![(row as i32, 0)]) {
                        if let Some(old_pos) = old_position.first() {

                            // Mark previous row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&old_pos.0) {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(key.to_string()), &old_position, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&(row as i32)) {
                                let cells_affected = vec![(row as i32, 0)];
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(key.to_string()), &cells_affected, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(row as i32);
                            }
                        }
                    }
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("DuplicatedRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    let mut row_keys: BTreeMap<i32, Cow<str>> = BTreeMap::new();
                    row_keys.insert(0, key);
                    row_keys.insert(1, data);

                    // If this returns something, it means there is a duplicate.
                    let combined_keys = row_keys.values().join("| |");
                    if let Some(old_position) = keys.insert(combined_keys.to_owned(), row_keys.keys().map(|x| (row as i32, *x)).collect()) {
                        if let Some(old_pos) = old_position.first() {

                            // Mark previous row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&old_pos.0) {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedRow(combined_keys.to_string()), &old_position, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_rows_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&(row as i32)) {
                                let cells_affected = row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>();
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedRow(combined_keys), &cells_affected, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_rows_already_marked.push(row as i32);
                            }
                        }
                    }
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::Loc(diagnostic))
            } else { None }
        } else { None }
    }
}
