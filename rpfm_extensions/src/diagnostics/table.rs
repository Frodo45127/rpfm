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

use rpfm_lib::files::table::DecodedData;
use rpfm_lib::schema::{DefinitionPatch, Field};

use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Table diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableDiagnostic {
    path: String,
    pack: String,
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
    AlteredTable,
}

/// Internal struct with cached data of tables.
///
/// Used to cache multiple tables and having them being aware of each other on check.
struct TableInfo<'a> {
    path: &'a str,
    container_name: &'a str,
    fields_processed: Vec<Field>,
    patches: Option<&'a DefinitionPatch>,
    key_amount: usize,
    table_data: Cow<'a, [Vec<DecodedData>]>,
    default_row: Vec<DecodedData>,
    ignored_fields: Vec<String>,
    ignored_diagnostics: HashSet<String>,
    ignored_diagnostics_for_fields: HashMap<String, Vec<String>>,
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
            TableDiagnosticReportType::AlteredTable => "Altered Table".to_owned(),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            TableDiagnosticReportType::OutdatedTable => DiagnosticLevel::Error,
            TableDiagnosticReportType::InvalidReference(_,_) => DiagnosticLevel::Error,
            TableDiagnosticReportType::EmptyRow => DiagnosticLevel::Warning,
            TableDiagnosticReportType::EmptyKeyField(_) => DiagnosticLevel::Error,
            TableDiagnosticReportType::EmptyKeyFields => DiagnosticLevel::Warning,
            TableDiagnosticReportType::DuplicatedCombinedKeys(_) => DiagnosticLevel::Warning,
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
            TableDiagnosticReportType::AlteredTable => DiagnosticLevel::Error,
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
            Self::AlteredTable => "AlteredTable",
        }, f)
    }
}


impl TableDiagnostic {
    pub fn new(path: &str, pack: &str) -> Self {
        Self {
            path: path.to_owned(),
            pack: pack.to_owned(),
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
        files: &[&RFile],
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        game_info: &GameInfo,
        local_path_list: &HashMap<String, Vec<String>>,
        check_ak_only_refs: bool,
        files_to_ignore: &Option<Vec<(String, Vec<String>, Vec<String>)>>,
        pack: &Pack,
        schema: &Schema,
        loc_data: &Option<HashMap<Cow<str>, Cow<str>>>
    ) -> Vec<DiagnosticType> {
        let mut diagnostics = vec![];

        if files.is_empty() {
            return diagnostics;
        }

        // Get the dependency data for tables once per batch. That way we can speed up this a lot.
        let file = files.first().and_then(|x| x.decoded().ok());
        let dependency_data = if let Some(RFileDecoded::DB(table)) = file {
            dependencies.db_reference_data(schema, pack, table.table_name(), table.definition(), loc_data)
        } else {
            return diagnostics
        };

        // So, the way we do this semi-optimized, is we do a first loop getting all the cached data we're going to need,
        // then do the real loop, having the data of all files available for checking diagnostics.
        let mut table_infos = vec![];
        for file in files {
            let (ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) = Diagnostics::ignore_data_for_file(file, files_to_ignore).unwrap_or_default();
            if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                let fields_processed = table.definition().fields_processed();
                let patches = Some(table.definition().patches());
                let table_data = table.data();

                table_infos.push(TableInfo {
                    path: file.path_in_container_raw(),
                    container_name: file.container_name().as_deref().unwrap_or(""),
                    key_amount: fields_processed.iter().filter(|field| field.is_key(patches)).count(),
                    fields_processed,
                    patches,
                    table_data,
                    default_row: table.new_row(),
                    ignored_fields,
                    ignored_diagnostics,
                    ignored_diagnostics_for_fields
                });
            }
        }

        let mut global_keys: HashMap<Vec<&DecodedData>, Vec<(Vec<(i32, i32)>, usize)>> = HashMap::with_capacity(table_infos.iter().map(|x| x.table_data.len()).sum());
        let dec_files = files.iter()
            .filter_map(|x| match x.decoded().ok() {
                Some(RFileDecoded::DB(ref table)) => Some((table, x)),
                _ => None,
            })
            .collect::<Vec<_>>();

        for (index, (table, file)) in dec_files.iter().enumerate() {
            let is_twad_key_deletes = table.table_name().starts_with("twad_key_deletes");
            let check_ak_only = check_ak_only_refs || table.table_name().starts_with("start_pos_");
            if let Some(table_info) = table_infos.get(index) {
                let mut diagnostic = TableDiagnostic::new(file.path_in_container_raw(), file.container_name().as_deref().unwrap_or(""));

                // Before anything else, check if the table is outdated.
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("OutdatedTable"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && Self::is_table_outdated(table.table_name(), *table.definition().version(), dependencies) {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::OutdatedTable, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                // Check if it's one of the banned tables for the game selected.
                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("BannedTable"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && game_info.is_file_banned(file.path_in_container_raw()) {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::BannedTable, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("AlteredTable"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && table.altered() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::AlteredTable, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                // Check if the table name has a number at the end, which causes very annoying bugs.
                if let Some(name) = file.file_name() {
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameEndsInNumber"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && (name.ends_with('0') ||
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

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameHasSpace"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && name.contains(' ') {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableNameHasSpace, &[], &[]);
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableIsDataCoring"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) {
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

                // Columns we can try to check for paths.
                let mut ignore_path_columns = vec![];
                for (column, field) in table_info.fields_processed.iter().enumerate() {
                    if let Some(rel_paths) = field.filename_relative_path(table_info.patches) {
                        if rel_paths.iter().any(|path| path.contains('*')) {
                            ignore_path_columns.push(column);
                        }
                    }
                }

                let mut no_ref_table_nor_column_found_marked = HashSet::new();
                let mut no_ref_table_found_marked = HashSet::new();

                for (row, cells) in table_info.table_data.iter().enumerate() {
                    let mut row_keys_are_empty = true;
                    let mut row_keys: BTreeMap<i32, &DecodedData> = BTreeMap::new();
                    for (column, field) in table_info.fields_processed.iter().enumerate() {

                        // Skip unused field on diagnostics.
                        //if field.unused(patches) {
                        //    continue;
                        //}

                        let cell_data = cells[column].data_to_string();

                        // Path checks.
                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("FieldWithPathNotFound"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) &&
                            !cell_data.is_empty() &&
                            cell_data != "." &&
                            cell_data != "x" &&
                            cell_data != "false" &&
                            cell_data != "building_placeholder" &&
                            cell_data != "placeholder" &&
                            cell_data != "PLACEHOLDER" &&
                            cell_data != "placeholder.png" &&
                            cell_data != "placehoder.png" &&
                            table_info.fields_processed[column].is_filename(table_info.patches) &&
                            !ignore_path_columns.contains(&column) {

                            let mut path_found = false;
                            let relative_paths = table_info.fields_processed[column].filename_relative_path(table_info.patches);
                            let paths = if let Some(relative_paths) = relative_paths {
                                relative_paths.iter()
                                    .flat_map(|x| {
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
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::FieldWithPathNotFound(paths), &[(row as i32, column as i32)], &table_info.fields_processed);
                                diagnostic.results_mut().push(result);
                            }
                        }

                        // Dependency checks.
                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), None, &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) &&
                            (field.is_reference(table_info.patches).is_some() ||
                                (
                                    is_twad_key_deletes &&
                                    field.name() == "table_name"
                                )
                            ) {

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
                                    //
                                    // NOTE: This diagnostics should be one, per column. Once it's done, do not create a new one
                                    // for the same column in subsequent rows.
                                    else if ref_data.data().is_empty() && (no_ref_table_nor_column_found_marked.is_empty() || !no_ref_table_nor_column_found_marked.contains(&column)) {
                                        if !dependencies.is_asskit_data_loaded() {
                                            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundNoPak"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) {
                                                let field_name = table_info.fields_processed[column].name().to_string();
                                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(field_name), &[(-1, column as i32)], &table_info.fields_processed);
                                                diagnostic.results_mut().push(result);
                                            }
                                        }
                                        else if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundPak"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) {
                                            let field_name = table_info.fields_processed[column].name().to_string();
                                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(field_name), &[(-1, column as i32)], &table_info.fields_processed);
                                            diagnostic.results_mut().push(result);
                                        }

                                        no_ref_table_nor_column_found_marked.insert(column);
                                    }

                                    // Check for non-empty cells with reference data, but the data in the cell is not in the reference data list.
                                    else if !ref_data.data().is_empty() && !cell_data.is_empty() && !ref_data.data().contains_key(&*cell_data) && (!*ref_data.referenced_table_is_ak_only() || check_ak_only) {

                                        // Numeric cells with 0 are "empty" references and should not be checked.
                                        let is_number = *field.field_type() == FieldType::I32 || *field.field_type() == FieldType::I64 || *field.field_type() == FieldType::OptionalI32 || *field.field_type() == FieldType::OptionalI64;
                                        let is_valid_reference = if is_number { cell_data != "0" } else { true };
                                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("InvalidReference"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && is_valid_reference {
                                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidReference(cell_data.to_string(), field.name().to_string()), &[(row as i32, column as i32)], &table_info.fields_processed);
                                            diagnostic.results_mut().push(result);
                                        }
                                    }
                                }
                                None => {

                                    // This diagnostic also needs to be done once per column.
                                    if no_ref_table_found_marked.is_empty() || !no_ref_table_found_marked.contains(&column) {
                                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableFound"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) {
                                            let field_name = table_info.fields_processed[column].name().to_string();
                                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableFound(field_name), &[(-1, column as i32)], &table_info.fields_processed);
                                            diagnostic.results_mut().push(result);
                                        }
                                        no_ref_table_found_marked.insert(column);
                                    }
                                }
                            }
                        }

                        if row_keys_are_empty && field.is_key(table_info.patches) && (!cell_data.is_empty() && cell_data != "false") {
                            row_keys_are_empty = false;
                        }

                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("EmptyKeyField"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && field.is_key(table_info.patches) && table_info.key_amount == 1 && *field.field_type() != FieldType::OptionalStringU8 && *field.field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField(field.name().to_string()), &[(row as i32, column as i32)], &table_info.fields_processed);
                            diagnostic.results_mut().push(result);
                        }

                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("ValueCannotBeEmpty"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && cell_data.is_empty() && field.cannot_be_empty(table_info.patches) {
                            let result = TableDiagnosticReport::new(TableDiagnosticReportType::ValueCannotBeEmpty(field.name().to_string()), &[(row as i32, column as i32)], &table_info.fields_processed);
                            diagnostic.results_mut().push(result);
                        }

                        if field.is_key(table_info.patches) {
                            row_keys.insert(column as i32, &cells[column]);
                        }
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyRow"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && cells == &table_info.default_row {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &table_info.fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyKeyFields"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && row_keys_are_empty && table_info.key_amount > 1 {
                        let cells_affected = row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>();
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyFields, &cells_affected, &table_info.fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    let keys = row_keys.values().copied().collect::<Vec<_>>();
                    let values = (row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>(), index);
                    match global_keys.get_mut(&keys) {
                        Some(val) => val.push(values),
                        None => { global_keys.insert(keys, vec![values]); },
                    }
                }

                if !diagnostic.results().is_empty() {
                    diagnostics.push(DiagnosticType::DB(diagnostic));
                }
            }
        }

        // This diagnostics needs the row keys data for all tables to be generated. So we have to perform it outside the usual check loop.
        //
        // Also, unlike other diagnostics, we don't know what entries of this should be ignored (if any) until we find the duplicates.
        global_keys.iter()
            .filter(|(_, val)| val.len() > 1)
            .for_each(|(key, val)| {
                for (pos, index) in val {
                    if let Some(table_info) = table_infos.get(*index) {
                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics,
                            None,
                            Some("DuplicatedCombinedKeys"),
                            &table_info.ignored_fields,
                            &table_info.ignored_diagnostics,
                            &table_info.ignored_diagnostics_for_fields
                        ) {

                            match diagnostics.iter_mut().find(|x| x.path() == table_info.path) {
                                Some(diag) => if let DiagnosticType::DB(ref mut diag) = diag {
                                    diag.results_mut().push(
                                        TableDiagnosticReport::new(
                                            TableDiagnosticReportType::DuplicatedCombinedKeys(
                                                key.iter().map(|x| x.data_to_string()).join("| |")
                                            ),
                                            pos,
                                            &table_info.fields_processed
                                        )
                                    )
                                }
                                None => {
                                    let mut diag = TableDiagnostic::new(table_info.path, table_info.container_name);
                                        diag.results_mut().push(
                                        TableDiagnosticReport::new(
                                            TableDiagnosticReportType::DuplicatedCombinedKeys(
                                                key.iter().map(|x| x.data_to_string()).join("| |")
                                            ),
                                            pos,
                                            &table_info.fields_processed
                                        )
                                    );

                                    // Add the new diagnostic and update the cached references so this one is also searched when processing other entries.
                                    diagnostics.push(DiagnosticType::DB(diag));
                                }
                            }
                        }
                    }
                }
            });

        diagnostics
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    pub fn check_loc(
        files: &[&RFile],
        global_ignored_diagnostics: &[String],
        files_to_ignore: &Option<Vec<(String, Vec<String>, Vec<String>)>>
    ) -> Vec<DiagnosticType> {
        let mut diagnostics = vec![];

        // So, the way we do this semi-optimized, is we do a first loop getting all the cached data we're going to need,
        // then do the real loop, having the data of all files available for checking diagnostics.
        let mut table_infos = vec![];
        for file in files {
            let (ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) = Diagnostics::ignore_data_for_file(file, files_to_ignore).unwrap_or_default();
            if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                let fields_processed = table.definition().fields_processed();
                let patches = Some(table.definition().patches());
                let table_data = table.data();

                table_infos.push(TableInfo {
                    path: file.path_in_container_raw(),
                    container_name: file.container_name().as_deref().unwrap_or(""),
                    key_amount: fields_processed.iter().filter(|field| field.is_key(patches)).count(),
                    fields_processed,
                    patches,
                    table_data,
                    default_row: table.new_row(),
                    ignored_fields,
                    ignored_diagnostics,
                    ignored_diagnostics_for_fields
                });
            }
        }

        let dec_files = files.iter()
            .filter_map(|x| match x.decoded().ok() {
                Some(RFileDecoded::Loc(ref table)) => Some((table, x)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut global_keys: HashMap<&DecodedData, Vec<((i32, i32), usize)>> = HashMap::with_capacity(table_infos.iter().map(|x| x.table_data.len()).sum());

        for (index, (table, file)) in dec_files.iter().enumerate() {
            if let Some(table_info) = table_infos.get(index) {
                let mut diagnostic = TableDiagnostic::new(file.path_in_container_raw(), file.container_name().as_deref().unwrap_or(""));
                let fields = table.definition().fields_processed();
                let field_key_name = fields[0].name();
                let field_text_name = fields[1].name();

                for (row, cells) in table_info.table_data.iter().enumerate() {
                    let key = cells[0].data_to_string();
                    let data = cells[1].data_to_string();
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("InvalidLocKey"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && !key.is_empty() && (key.contains('\n') || key.contains('\r') || key.contains('\t')) {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidLocKey, &[(row as i32, 0)], &fields);
                        diagnostic.results_mut().push(result);
                    }

                    // Only in case none of the two columns are ignored, we perform these checks.
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyRow"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("EmptyRow"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && key.is_empty() && data.is_empty() {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &fields);
                        diagnostic.results_mut().push(result);
                    }

                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyKeyField"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && key.is_empty() && !data.is_empty() {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField("Key".to_string()), &[(row as i32, 0)], &fields);
                        diagnostic.results_mut().push(result);
                    }

                    // Magic Regex. It works. Don't ask why.
                    if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("InvalidEscape"), &table_info.ignored_fields, &table_info.ignored_diagnostics, &table_info.ignored_diagnostics_for_fields) && !data.is_empty() && REGEX_INVALID_ESCAPES.is_match(&data).unwrap() {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidEscape, &[(row as i32, 1)], &fields);
                        diagnostic.results_mut().push(result);
                    }

                    match global_keys.get_mut(&cells[0]) {
                        Some(val) => val.push(((row as i32, 0i32), index)),
                        None => { global_keys.insert(&cells[0], vec![((row as i32, 0i32), index)]); },
                    }
                }


                if !diagnostic.results().is_empty() {
                    diagnostics.push(DiagnosticType::Loc(diagnostic));
                }
            }
        }

        // This diagnostics needs the row keys data for all tables to be generated. So we have to perform it outside the usual check loop.
        //
        // Also, unlike other diagnostics, we don't know what entries of this should be ignored (if any) until we find the duplicates.
        global_keys.iter()
            .filter(|(_, val)| val.len() > 1)
            .for_each(|(key, val)| {
                for (pos, index) in val {
                    if let Some(table_info) = table_infos.get(*index) {
                        if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics,
                            None,
                            Some("DuplicatedCombinedKeys"),
                            &table_info.ignored_fields,
                            &table_info.ignored_diagnostics,
                            &table_info.ignored_diagnostics_for_fields
                        ) {

                            match diagnostics.iter_mut().find(|x| x.path() == table_info.path) {
                                Some(diag) => if let DiagnosticType::Loc(ref mut diag) = diag {
                                    diag.results_mut().push(
                                        TableDiagnosticReport::new(
                                            TableDiagnosticReportType::DuplicatedCombinedKeys(
                                                key.data_to_string().to_string()
                                            ),
                                            &[*pos],
                                            &table_info.fields_processed
                                        )
                                    )
                                }
                                None => {
                                    let mut diag = TableDiagnostic::new(table_info.path, table_info.container_name);
                                        diag.results_mut().push(
                                        TableDiagnosticReport::new(
                                            TableDiagnosticReportType::DuplicatedCombinedKeys(
                                                key.data_to_string().to_string()
                                            ),
                                            &[*pos],
                                            &table_info.fields_processed
                                        )
                                    );

                                    // Add the new diagnostic and update the cached references so this one is also searched when processing other entries.
                                    diagnostics.push(DiagnosticType::Loc(diag));
                                }
                            }
                        }

                    }
                }

                let mut values = Vec::with_capacity(val.len());
                for (pos, index) in val {
                    if let Some(table_info) = table_infos.get(*index) {
                        values.push((&table_info.table_data[pos.0 as usize][1], pos.0, index));
                    }
                }

                values.sort_unstable_by_key(|x| x.0.data_to_string());
                let dups = values.iter().duplicates_by(|x| x.0);
                let poss = values.iter().positions(|x| dups.clone().any(|y| y.0 == x.0));

                for pos in poss {
                    if let Some((data, row, index)) = values.get(pos) {
                        if let Some(table_info) = table_infos.get(**index) {
                            if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics,
                                None,
                                Some("DuplicatedRow"),
                                &table_info.ignored_fields,
                                &table_info.ignored_diagnostics,
                                &table_info.ignored_diagnostics_for_fields
                            ) {

                                match diagnostics.iter_mut().find(|x| x.path() == table_info.path) {
                                    Some(diag) => if let DiagnosticType::Loc(ref mut diag) = diag {
                                        diag.results_mut().push(
                                            TableDiagnosticReport::new(
                                                TableDiagnosticReportType::DuplicatedRow(
                                                    String::from(table_info.table_data[*row as usize][0].data_to_string()) + "| |" + &data.data_to_string()
                                                ),
                                                &[(*row, 0), (*row, 1)],
                                                &table_info.fields_processed
                                            )
                                        )
                                    }
                                    None => {
                                        let mut diag = TableDiagnostic::new(table_info.path, table_info.container_name);
                                            diag.results_mut().push(
                                            TableDiagnosticReport::new(
                                                TableDiagnosticReportType::DuplicatedRow(
                                                    String::from(table_info.table_data[*row as usize][0].data_to_string()) + "| |" + &data.data_to_string()
                                                ),
                                                &[(*row, 0), (*row, 1)],
                                                &table_info.fields_processed
                                            )
                                        );

                                        // Add the new diagnostic and update the cached references so this one is also searched when processing other entries.
                                        diagnostics.push(DiagnosticType::Loc(diag));
                                    }
                                }
                            }
                        }
                    }
                }
            });

        diagnostics
    }
}
