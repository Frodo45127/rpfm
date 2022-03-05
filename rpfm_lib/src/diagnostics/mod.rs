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

Notes on cells_affected:
- Both -1: affects the entire table.
- Row -1: affects all rows in single column.
- Column -1: affects all columns in single row.
!*/

use serde_derive::{Serialize, Deserialize};
use itertools::Itertools;
use fancy_regex::Regex;
use rayon::prelude::*;
use unicase::UniCase;

use std::{fmt, fmt::Display};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::DB;
use crate::dependencies::Dependencies;
use crate::games::VanillaDBTableNameLogic;
use crate::GAME_SELECTED;
use crate::packfile::{PackFile, PathType};
use crate::packedfile::{table::{DecodedData, DependencyData}, DecodedPackedFile, PackedFileType};
use crate::packfile::packedfile::{PackedFile, PackedFileInfo};
use crate::schema::FieldType;
use crate::SCHEMA;

use self::anim_fragment::{AnimFragmentDiagnostic, AnimFragmentDiagnosticReport, AnimFragmentDiagnosticReportType};
use self::config::{ConfigDiagnostic, ConfigDiagnosticReport, ConfigDiagnosticReportType};
use self::dependency_manager::{DependencyManagerDiagnostic, DependencyManagerDiagnosticReport, DependencyManagerDiagnosticReportType};
use self::packfile::{PackFileDiagnostic, PackFileDiagnosticReport, PackFileDiagnosticReportType};
use self::table::{TableDiagnostic, TableDiagnosticReport, TableDiagnosticReportType};

pub mod anim_fragment;
pub mod config;
pub mod dependency_manager;
pub mod packfile;
pub mod table;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over multiple PackedFiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics(Vec<DiagnosticType>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticType {
    AnimFragment(AnimFragmentDiagnostic),
    DB(TableDiagnostic),
    Loc(TableDiagnostic),
    PackFile(PackFileDiagnostic),
    DependencyManager(DependencyManagerDiagnostic),
    Config(ConfigDiagnostic),
}

/// This enum defines the possible results for a result of a diagnostic check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `Diagnostics`.
impl Default for Diagnostics {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DiagnosticType {
    pub fn get_path(&self) -> &[String] {
        match self {
            Self::AnimFragment(ref diag) => diag.get_path(),
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.get_path(),
            Self::PackFile(_) => &[],
            Self::DependencyManager(diag) => diag.get_path(),
            Self::Config(_) => &[],
        }
    }
}

/// Implementation of `Diagnostics`.
impl Diagnostics {

    pub fn get_ref_diagnostics(&self) -> &[DiagnosticType] {
        &self.0
    }

    pub fn get_ref_mut_diagnostics(&mut self) -> &mut Vec<DiagnosticType> {
        &mut self.0
    }

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn check(&mut self, pack_file: &PackFile, dependencies: &Dependencies) {

        // Clear the diagnostics first.
        self.0.clear();

        // First, check for config issues, as some of them may stop the checking prematurely.
        if let Some(diagnostics) = Self::check_config(dependencies) {
            let is_diagnostic_blocking = if let DiagnosticType::Config(ref diagnostic) = diagnostics {
                diagnostic.get_ref_result().iter().any(|diagnostic| matches!(diagnostic.report_type,
                    ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                    ConfigDiagnosticReportType::DependenciesCacheOutdated |
                    ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_)))
            } else { false };

            // If we have one of the blocking diagnostics, report it and return.
            self.0.push(diagnostics);
            if is_diagnostic_blocking {
                return;
            }
        }

        let files_to_ignore = pack_file.get_settings().get_diagnostics_files_to_ignore();

        // Prefetch them here, so we don't need to re-search them again.
        let vanilla_dependencies = if let Ok(dependencies) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true) { dependencies } else { return; };
        let asskit_dependencies = dependencies.get_ref_asskit_only_db_tables();

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let packed_files = pack_file.get_ref_packed_files_by_types(&[PackedFileType::AnimFragment, PackedFileType::DB, PackedFileType::Loc], false);
        let mut packed_files_split: BTreeMap<&str, Vec<&PackedFile>> = BTreeMap::new();

        for packed_file in &packed_files {
            match packed_file.get_packed_file_type(false) {
                PackedFileType::AnimFragment => {
                    if let Some(table_set) = packed_files_split.get_mut("anim_fragments") {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert("anim_fragments", vec![packed_file]);
                    }
                },
                PackedFileType::DB => {
                    if let Some(table_set) = packed_files_split.get_mut(&*packed_file.get_path()[1]) {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert(&packed_file.get_path()[1], vec![packed_file]);
                    }
                },
                PackedFileType::Loc => {
                    if let Some(table_set) = packed_files_split.get_mut("locs") {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert("locs", vec![packed_file]);
                    }
                },
                _ => {},
            }
        }

        if let Some(ref schema) = *SCHEMA.read().unwrap() {

            // Getting this here speeds up a lot path-checking later.
            let local_packed_file_path_list = pack_file.get_packed_files_all_paths_as_string();
            let local_folder_path_list = pack_file.get_folder_all_paths_as_string();

            // Process the files in batches.
            self.0 = packed_files_split.into_par_iter().filter_map(|(_, packed_files)| {

                let mut diagnostics = Vec::with_capacity(packed_files.len());
                let mut data_prev: BTreeMap<String, HashMap<String, Vec<(i32, i32)>>> = BTreeMap::new();
                let mut dependency_data_for_table = BTreeMap::new();

                for packed_file in packed_files {

                    // Ignore entire tables if their path starts with the one we have (so we can do mass ignores) and we didn't specified a field to ignore.
                    let mut ignored_fields = vec![];
                    let mut ignored_diagnostics = vec![];
                    let mut ignored_diagnostics_for_fields: HashMap<String, Vec<String>> = HashMap::new();
                    if let Some(ref files_to_ignore) = files_to_ignore {
                        for (path_to_ignore, fields, diags_to_ignore) in files_to_ignore {

                            // If the rule doesn't affect this PackedFile, ignore it.
                            if !path_to_ignore.is_empty() && packed_file.get_path().starts_with(path_to_ignore) {

                                // If we don't have either fields or diags specified, we ignore the entire file.
                                if fields.is_empty() && diags_to_ignore.is_empty() {
                                    return None;
                                }

                                // If we have both, fields and diags, disable only those diags for those fields.
                                if !fields.is_empty() && !diags_to_ignore.is_empty() {
                                    for field in fields {
                                        match ignored_diagnostics_for_fields.get_mut(field) {
                                            Some(diagnostics) => diagnostics.append(&mut diags_to_ignore.to_vec()),
                                            None => { ignored_diagnostics_for_fields.insert(field.to_owned(), diags_to_ignore.to_vec()); },
                                        }
                                    }
                                }

                                // Otherwise, check if we only have fields or diags, and put them separately.
                                else if !fields.is_empty() {
                                    ignored_fields.append(&mut fields.to_vec());
                                }

                                else if !diags_to_ignore.is_empty() {
                                    ignored_diagnostics.append(&mut diags_to_ignore.to_vec());
                                }
                            }
                        }
                    }

                    let diagnostic = match packed_file.get_packed_file_type(false) {
                        PackedFileType::AnimFragment => if let Ok(decoded) = packed_file.decode_return_ref_no_cache_no_locks(schema) {
                            Self::check_anim_fragment(&decoded, packed_file.get_path(), dependencies, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &local_packed_file_path_list, &local_folder_path_list)
                        } else { None }
                        PackedFileType::DB => {

                            // Get the dependency data for tables once per batch.
                            // That way we can speed up this a lot.
                            let decoded_packed_file = packed_file.get_ref_decoded();
                            if dependency_data_for_table.is_empty() {
                                if let DecodedPackedFile::DB(table) = decoded_packed_file {
                                    dependency_data_for_table = DB::get_dependency_data(
                                        pack_file,
                                        table.get_ref_table_name(),
                                        table.get_ref_definition(),
                                        &vanilla_dependencies,
                                        asskit_dependencies,
                                        dependencies,
                                        &[],
                                    );
                                }
                            }

                            Self::check_db(packed_file.get_ref_decoded(), packed_file.get_path(), dependencies, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &mut data_prev, &local_packed_file_path_list, &local_folder_path_list, &dependency_data_for_table)
                        },
                        PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path(), &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &mut data_prev),
                        _ => None,
                    };

                    if let Some(diagnostic) = diagnostic {
                        diagnostics.push(diagnostic);
                    }
                }

                /*
                // Check for key collisions between separate files.
                let mut duplicated_keys_already_marked: HashMap<String, HashSet<String>> = HashMap::new();
                data_prev.iter().enumerate().for_each(|(index, (path, keys))| {
                    let mut current = data_prev.len() - 1;
                    keys.iter().for_each(|(key, positions)| {
                        for (path_other_file, keys_other_file) in &data_prev {
                            if current > index {
                                current -= 1;

                                if let Some(positions_other_file) = keys_other_file.get(&*key) {

                                    // Mark current row, if not yet marked.
                                    if !duplicated_keys_already_marked.contains_key(&*path) || !duplicated_keys_already_marked.get(&*path).unwrap().contains(key) {
                                        if let Some(diagnostic) = diagnostics.iter_mut().filter_map(|x| if let DiagnosticType::DB(diag) = x { Some(diag) } else { None }).find(|x| x.get_path().join("/") == *path) {
                                            dbg!(key);
                                            dbg!(path);
                                            dbg!(path_other_file);
                                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                                cells_affected: positions.to_vec(),
                                                message: format!("Duplicated combined keys: {}.", key),
                                                report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                                                level: DiagnosticLevel::Error,
                                            });
                                        }

                                        match duplicated_keys_already_marked.get_mut(&*path) {
                                            Some(set) => { set.insert(key.to_owned()); },
                                            None => {
                                                let mut set = HashSet::new();
                                                set.insert(key.to_owned());
                                                duplicated_keys_already_marked.insert(path.to_owned(), set);
                                            }
                                        }
                                    }

                                    // Mark the other row.
                                    if !duplicated_keys_already_marked.contains_key(&*path_other_file) || !duplicated_keys_already_marked.get(&*path_other_file).unwrap().contains(key) {
                                        if let Some(diagnostic) = diagnostics.iter_mut().filter_map(|x| if let DiagnosticType::DB(diag) = x { Some(diag) } else { None }).find(|x| x.get_path().join("/") == *path_other_file) {

                                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                                cells_affected: positions_other_file.to_vec(),
                                                message: format!("Duplicated combined keys: {}.", key),
                                                report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                                                level: DiagnosticLevel::Error,
                                            });
                                        }

                                        match duplicated_keys_already_marked.get_mut(&*path_other_file) {
                                            Some(set) => { set.insert(key.to_owned()); },
                                            None => {
                                                let mut set = HashSet::new();
                                                set.insert(key.to_owned());
                                                duplicated_keys_already_marked.insert(path_other_file.to_owned(), set);
                                            }
                                        }
                                    }

                                    //if !duplicated_keys_already_marked.contains(&old_position[0].0) {
                                    //    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    //        cells_affected: old_position.to_vec(),
                                    //        message: format!("Duplicated combined keys: {}.", row_keys.values().join("| |")),
                                    //        report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                                    //        level: DiagnosticLevel::Error,
                                    //    });

                                    //    duplicated_combined_keys_already_marked.push(old_position[0].0);
                                    //}
                            }
                            }
                        }
                    })
                });
                */
                Some(diagnostics)
            }).flatten().collect();
        }

        if let Some(diagnostics) = Self::check_dependency_manager(pack_file) {
            self.0.push(diagnostics);
        }

        if let Some(diagnostics) = Self::check_packfile(pack_file) {
            self.0.push(diagnostics);
        }

        self.get_ref_mut_diagnostics().sort_by(|a, b| {
            if !a.get_path().is_empty() && !b.get_path().is_empty() {
                a.get_path().cmp(b.get_path())
            } else if a.get_path().is_empty() && !b.get_path().is_empty() {
                Ordering::Greater
            } else if !a.get_path().is_empty() && b.get_path().is_empty() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }

    /// This function takes care of checking the db tables of your mod for errors.
    fn check_db(
        packed_file: &DecodedPackedFile,
        path: &[String],
        dependencies: &Dependencies,
        ignored_fields: &[String],
        ignored_diagnostics: &[String],
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        previous_data: &mut BTreeMap<String, HashMap<String, Vec<(i32, i32)>>>,
        local_path_list: &HashSet<UniCase<String>>,
        local_folder_list: &HashSet<UniCase<String>>,
        dependency_data: &BTreeMap<i32, DependencyData>
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::DB(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);

            // Check all the columns with reference data.
            let mut columns_without_reference_table = vec![];
            let mut columns_with_reference_table_and_no_column = vec![];
            let mut keys: HashMap<String, Vec<(i32, i32)>> = HashMap::new();
            let mut duplicated_combined_keys_already_marked = vec![];

            // Before anything else, check if the table is outdated.
            if !Self::ignore_diagnostic(None, Some("OutdatedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                if table.is_outdated(dependencies) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        cells_affected: vec![],
                        message: "Possibly outdated table.".to_owned(),
                        report_type: TableDiagnosticReportType::OutdatedTable,
                        level: DiagnosticLevel::Error,
                    });
                }
            }

            // Check if it's one of the banned tables for the game selected.
            if !Self::ignore_diagnostic(None, Some("BannedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                if GAME_SELECTED.read().unwrap().is_packedfile_banned(&["db".to_owned(), table.get_table_name()]) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        cells_affected: vec![],
                        message: "Banned table.".to_owned(),
                        report_type: TableDiagnosticReportType::BannedTable,
                        level: DiagnosticLevel::Error,
                    });
                }
            }

            // Check if the table name has a number at the end, which causes very annoying bugs.
            if let Some(name) = path.last() {
                if !Self::ignore_diagnostic(None, Some("TableNameEndsInNumber"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if name.ends_with('0') ||
                        name.ends_with('1') ||
                        name.ends_with('2') ||
                        name.ends_with('3') ||
                        name.ends_with('4') ||
                        name.ends_with('5') ||
                        name.ends_with('6') ||
                        name.ends_with('7') ||
                        name.ends_with('8') ||
                        name.ends_with('9') {

                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![],
                            message: "Table name ends in number.".to_owned(),
                            report_type: TableDiagnosticReportType::TableNameEndsInNumber,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if !Self::ignore_diagnostic(None, Some("TableNameHasSpace"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if name.contains(' ') {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![],
                            message: "Table name contains spaces.".to_owned(),
                            report_type: TableDiagnosticReportType::TableNameHasSpace,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if !Self::ignore_diagnostic(None, Some("TableIsDataCoring"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    match GAME_SELECTED.read().unwrap().get_vanilla_db_table_name_logic() {
                        VanillaDBTableNameLogic::FolderName => {
                            if table.get_table_name_without_tables() == path[1] {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: vec![],
                                    message: "Table is datacoring.".to_owned(),
                                    report_type: TableDiagnosticReportType::TableIsDataCoring,
                                    level: DiagnosticLevel::Warning,
                                });
                            }
                        }

                        VanillaDBTableNameLogic::DefaultName(ref default_name) => {
                            if name == default_name {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: vec![],
                                    message: "Table is datacoring.".to_owned(),
                                    report_type: TableDiagnosticReportType::TableIsDataCoring,
                                    level: DiagnosticLevel::Warning,
                                });
                            }
                        }
                    }
                }
            }

            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let mut row_is_empty = true;
                let mut row_keys_are_empty = true;
                let mut row_keys: BTreeMap<i32, String> = BTreeMap::new();
                let fields_processed = table.get_ref_definition().get_fields_processed();
                for (column, field) in fields_processed.iter().enumerate() {

                    let cell_data = cells[column].data_to_string();

                    // Path checks.
                    if !Self::ignore_diagnostic(Some(field.get_name()), Some("FieldWithPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        if !cell_data.is_empty() {
                            if fields_processed[column].get_is_filename() {
                                let mut path_found = false;
                                let paths = {
                                    let path = if let Some(relative_path) = fields_processed[column].get_filename_relative_path() {
                                        relative_path.replace("%", &cell_data)
                                    } else {
                                        cell_data.to_owned()
                                    };

                                    // Skip paths with wildcards, as we do not support them.
                                    if path.contains('*') {
                                        path_found = true;
                                        vec![]
                                    } else {
                                        path.replace('\\', "/").replace(';', ",").split(',').map(|x| {
                                            let mut x = x.to_owned();
                                            if x.ends_with('/') {
                                                x.pop();
                                            }
                                            x
                                        }).collect::<Vec<String>>()
                                    }
                                };

                                for path in &paths {
                                    let unicased = UniCase::new(path.to_owned());
                                    if local_path_list.contains(&unicased) {
                                        path_found = true;
                                    }

                                    if !path_found && local_folder_list.contains(&unicased) {
                                        path_found = true;
                                    }

                                    if !path_found && dependencies.file_exists_on_parent_files(&unicased, true) {
                                        path_found = true;
                                    }

                                    if !path_found && dependencies.folder_exists_on_parent_files(&unicased, true) {
                                        path_found = true;
                                    }

                                    if !path_found && dependencies.file_exists_on_game_files(&unicased, true) {
                                        path_found = true;
                                    }

                                    if !path_found && dependencies.folder_exists_on_game_files(&unicased, true) {
                                        path_found = true;
                                    }

                                    if path_found {
                                        break;
                                    }
                                }

                                if !path_found {
                                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                        cells_affected: vec![(row as i32, column as i32)],
                                        message: format!("Path not found: {}.", paths.iter().join(" || ")),
                                        report_type: TableDiagnosticReportType::FieldWithPathNotFound,
                                        level: DiagnosticLevel::Warning,
                                    });
                                }
                            }
                        }
                    }

                    // Dependency checks.
                    if !Self::ignore_diagnostic(Some(field.get_name()), None, ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        if let Some((_ref_table_name, _ref_column_name)) = field.get_is_reference() {
                            match dependency_data.get(&(column as i32)) {
                                Some(ref_data) => {

                                    if ref_data.referenced_column_is_localised || ref_data.referenced_table_is_ak_only {
                                        // TODO: report missing loc data here.
                                    }
                                    /*
                                    else if ref_data.referenced_table_is_ak_only {
                                        // If it's only in the AK, ignore it.
                                    }*/

                                    // Blue cell check. Only one for each column, so we don't fill the diagnostics with this.
                                    else if ref_data.data.is_empty() {
                                        if !columns_with_reference_table_and_no_column.contains(&column) {
                                            columns_with_reference_table_and_no_column.push(column);
                                        }
                                    }

                                    // Check for non-empty cells with reference data, but the data in the cell is not in the reference data list.
                                    else if !cell_data.is_empty() && !ref_data.data.contains_key(&cell_data) {

                                        // Numeric cells with 0 are "empty" references and should not be checked.
                                        let is_number = field.get_field_type() == FieldType::I32 || field.get_field_type() == FieldType::I64;
                                        let is_valid_reference = if is_number { cell_data != "0" } else { true };
                                        if !Self::ignore_diagnostic(Some(field.get_name()), Some("InvalidReference"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && is_valid_reference {
                                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                                cells_affected: vec![(row as i32, column as i32)],
                                                message: format!("Invalid reference \"{}\" in column \"{}\".", &cell_data, field.get_name()),
                                                report_type: TableDiagnosticReportType::InvalidReference,
                                                level: DiagnosticLevel::Error,
                                            });
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
                    }

                    // Check for empty keys/rows.
                    if row_is_empty && (!cell_data.is_empty() && cell_data != "false") {
                        row_is_empty = false;
                    }

                    if row_keys_are_empty && field.get_is_key() && (!cell_data.is_empty() && cell_data != "false") {
                        row_keys_are_empty = false;
                    }

                    if !Self::ignore_diagnostic(Some(field.get_name()), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        if field.get_is_key() && field.get_field_type() != FieldType::OptionalStringU8 && field.get_field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                cells_affected: vec![(row as i32, column as i32)],
                                message: format!("Empty key for column \"{}\".", field.get_name()),
                                report_type: TableDiagnosticReportType::EmptyKeyField,
                                level: DiagnosticLevel::Warning,
                            });
                        }
                    }

                    if !Self::ignore_diagnostic(Some(field.get_name()), Some("ValueCannotBeEmpty"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        if field.get_cannot_be_empty(Some(table.get_ref_table_name())) && cell_data.is_empty() {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                cells_affected: vec![(row as i32, column as i32)],
                                message: format!("Empty value for column \"{}\".", field.get_name()),
                                report_type: TableDiagnosticReportType::ValueCannotBeEmpty,
                                level: DiagnosticLevel::Error,
                            });
                        }
                    }

                    if field.get_is_key() {
                        row_keys.insert(column as i32, cell_data);
                    }
                }

                if !Self::ignore_diagnostic(None, Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if row_is_empty {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(row as i32, -1)],
                            message: "Empty row.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyRow,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if !Self::ignore_diagnostic(None, Some("EmptyKeyFields"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if row_keys_are_empty {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>(),
                            message: "Empty key fields.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyKeyFields,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                if !Self::ignore_diagnostic(None, Some("DuplicatedCombinedKeys"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

                    // If this returns something, it means there is a duplicate.
                    let combined_keys = row_keys.values().join("| |");
                    if let Some(old_position) = keys.insert(combined_keys.to_owned(), row_keys.keys().map(|x| (row as i32, *x)).collect()) {
                        if let Some(old_pos) = old_position.get(0) {

                            // Mark previous row, if not yet marked.
                            if !duplicated_combined_keys_already_marked.contains(&old_pos.0) {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: old_position.to_vec(),
                                    message: format!("Duplicated combined keys: {}.", &combined_keys),
                                    report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                                    level: DiagnosticLevel::Error,
                                });

                                duplicated_combined_keys_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_combined_keys_already_marked.contains(&(row as i32)) {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>(),
                                    message: format!("Duplicated combined keys: {}.", &combined_keys),
                                    report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                                    level: DiagnosticLevel::Error,
                                });

                                duplicated_combined_keys_already_marked.push(row as i32);
                            }
                        }
                    }
                }
            }

            // Checks that only need to be done once per table.
            if !Self::ignore_diagnostic(None, Some("NoReferenceTableFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                for column in &columns_without_reference_table {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        cells_affected: vec![(-1, *column as i32)],
                        message: format!("No reference table found for column \"{}\".", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                        report_type: TableDiagnosticReportType::NoReferenceTableFound,
                        level: DiagnosticLevel::Info,
                    });
                }
            }
            for column in &columns_with_reference_table_and_no_column {
                if !dependencies.game_has_asskit_data_loaded() {
                    if !Self::ignore_diagnostic(None, Some("NoReferenceTableNorColumnFoundNoPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(-1, *column as i32)],
                            message: format!("No reference column found in referenced table for column \"{}\". Did you forgot to generate the Dependencies Cache, or did you generated it before installing the Assembly kit?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                            report_type: TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }
                else if !Self::ignore_diagnostic(None, Some("NoReferenceTableNorColumnFoundPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        cells_affected: vec![(-1, *column as i32)],
                        message: format!("No reference column found in referenced table for column \"{}\". Maybe a problem with the schema?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                        report_type: TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak,
                        level: DiagnosticLevel::Info,
                    });
                }
            }

            // Add this table's keys to the previous list, so they can be queried for for duplicate checks on other tables of the same type.
            previous_data.insert(path.join("/"), keys);

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::DB(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_anim_fragment(
        packed_file: &DecodedPackedFile,
        path: &[String],
        dependencies: &Dependencies,
        ignored_fields: &[String],
        ignored_diagnostics: &[String],
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        local_path_list: &HashSet<UniCase<String>>,
        local_folder_list: &HashSet<UniCase<String>>,
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::AnimFragment(table) = packed_file {
            let mut diagnostic = AnimFragmentDiagnostic::new(path);

            // Check inside the of the table in the column [3].
            for (row, _) in table.get_ref_table_data().iter().enumerate() {
                if let DecodedData::SequenceU32(ref data) = table.get_ref_table_data()[row][3] {
                    let fields_processed = data.get_ref_definition().get_fields_processed();
                    for (row, cells) in data.get_ref_table_data().iter().enumerate() {
                        for (column, field) in fields_processed.iter().enumerate() {

                            if !Self::ignore_diagnostic(Some(field.get_name()), Some("FieldWithPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                                if let DecodedData::StringU8(cell_data) = &cells[column] {
                                    if !cell_data.is_empty() {
                                        if fields_processed[column].get_is_filename() {
                                            let mut path_found = false;
                                            let mut path = cell_data.replace('\\', "/");

                                            // If it's a folder, remove the trailing /.
                                            if path.ends_with('/') {
                                                path.pop();
                                            }

                                            let unicased = UniCase::new(path.to_owned());
                                            if local_path_list.contains(&unicased) {
                                                path_found = true;
                                            }

                                            if !path_found && local_folder_list.contains(&unicased) {
                                                path_found = true;
                                            }

                                            if !path_found && dependencies.file_exists_on_parent_files(&unicased, true) {
                                                path_found = true;
                                            }

                                            if !path_found && dependencies.folder_exists_on_parent_files(&unicased, true) {
                                                path_found = true;
                                            }

                                            if !path_found && dependencies.file_exists_on_game_files(&unicased, true) {
                                                path_found = true;
                                            }

                                            if !path_found && dependencies.folder_exists_on_game_files(&unicased, true) {
                                                path_found = true;
                                            }

                                            if path_found {
                                                continue;
                                            }

                                            if !path_found {
                                                diagnostic.get_ref_mut_result().push(AnimFragmentDiagnosticReport {
                                                    cells_affected: vec![(row as i32, column as i32)],
                                                    message: format!("Path not found: {}.", path),
                                                    report_type: AnimFragmentDiagnosticReportType::FieldWithPathNotFound,
                                                    level: DiagnosticLevel::Warning,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::AnimFragment(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_loc(
        packed_file: &DecodedPackedFile,
        path: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &[String],
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        previous_data: &mut BTreeMap<String, HashMap<String, Vec<(i32, i32)>>>,
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::Loc(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);

            // Check all the columns with reference data.
            let mut keys: HashMap<String, Vec<(i32, i32)>> = HashMap::new();
            let fields = table.get_ref_definition().get_fields_processed();
            let field_key_name = fields[0].get_name().to_owned();
            let field_text_name = fields[1].get_name().to_owned();
            let mut duplicated_rows_already_marked = vec![];

            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let key = if let DecodedData::StringU16(ref data) = cells[0] { data } else { unimplemented!() };
                let data = if let DecodedData::StringU16(ref data) = cells[1] { data } else { unimplemented!() };

                if !Self::ignore_diagnostic(Some(&field_key_name), Some("InvalidLocKey"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if !key.is_empty() && (key.contains('\n') || key.contains('\t')) {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(row as i32, 0)],
                            message: "Invalid localisation key.".to_string(),
                            report_type: TableDiagnosticReportType::InvalidLocKey,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                // Only in case none of the two columns are ignored, we perform these checks.
                if !Self::ignore_diagnostic(Some(&field_key_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) &&
                    !Self::ignore_diagnostic(Some(&field_text_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

                    if key.is_empty() && data.is_empty() {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(row as i32, -1)],
                            message: "Empty row.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyRow,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                if !Self::ignore_diagnostic(Some(&field_key_name), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if key.is_empty() && !data.is_empty() {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(row as i32, 0)],
                            message: "Empty key.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyKeyField,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                // Magic Regex. It works. Don't ask why.
                if !Self::ignore_diagnostic(Some(&field_text_name), Some("InvalidEscape"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    if !data.is_empty() && Regex::new(r"(?<!\\)\\n|(?<!\\)\\t").unwrap().is_match(data).unwrap() {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            cells_affected: vec![(row as i32, 1)],
                            message: "Invalid line jump/tabulation detected in loc entry. Use \\\\n or \\\\t instead.".to_string(),
                            report_type: TableDiagnosticReportType::InvalidEscape,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                if !Self::ignore_diagnostic(Some(&field_key_name), Some("DuplicatedRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    let mut row_keys: BTreeMap<i32, String> = BTreeMap::new();
                    row_keys.insert(0, key.to_owned());
                    row_keys.insert(1, data.to_owned());

                    // If this returns something, it means there is a duplicate.
                    let combined_keys = row_keys.values().join("| |");
                    if let Some(old_position) = keys.insert(combined_keys.to_owned(), row_keys.keys().map(|x| (row as i32, *x)).collect()) {
                        if let Some(old_pos) = old_position.get(0) {

                            // Mark previous row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&old_pos.0) {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: old_position.to_vec(),
                                    message: format!("Duplicated row: {}.", &combined_keys),
                                    report_type: TableDiagnosticReportType::DuplicatedRow,
                                    level: DiagnosticLevel::Warning,
                                });

                                duplicated_rows_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&(row as i32)) {
                                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                    cells_affected: row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>(),
                                    message: format!("Duplicated row: {}.", &combined_keys),
                                    report_type: TableDiagnosticReportType::DuplicatedRow,
                                    level: DiagnosticLevel::Warning,
                                });

                                duplicated_rows_already_marked.push(row as i32);
                            }
                        }
                    }
                }
            }

            // Add this table's keys to the previous list, so they can be queried for for duplicate checks on other tables of the same type.
            previous_data.insert(path.join("/"), keys);

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::Loc(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking for PackFile-Related for errors.
    fn check_packfile(pack_file: &PackFile) -> Option<DiagnosticType> {
        let mut diagnostic = PackFileDiagnostic::new();

        let name = pack_file.get_file_name();
        if name.contains(' ') {
            diagnostic.get_ref_mut_result().push(PackFileDiagnosticReport {
                message: format!("Invalid PackFile name: {}", name),
                report_type: PackFileDiagnosticReportType::InvalidPackFileName,
                level: DiagnosticLevel::Error,
            });
        }

        if !diagnostic.get_ref_result().is_empty() {
            Some(DiagnosticType::PackFile(diagnostic))
        } else { None }
    }

    /// This function takes care of checking for errors in the Dependency Manager.
    fn check_dependency_manager(pack_file: &PackFile) ->Option<DiagnosticType> {
        let mut diagnostic = DependencyManagerDiagnostic::new();
        for (index, pack_file) in pack_file.get_packfiles_list().iter().enumerate() {

            // TODO: Make it so this also checks if the PackFile actually exists,
            if pack_file.is_empty() || !pack_file.ends_with(".pack") || pack_file.contains(' ') {
                diagnostic.get_ref_mut_result().push(DependencyManagerDiagnosticReport {
                    cells_affected: vec![(index as i32, 0)],
                    message: format!("Invalid dependency PackFile name: {}", pack_file),
                    report_type: DependencyManagerDiagnosticReportType::InvalidDependencyPackFileName,
                    level: DiagnosticLevel::Error,
                });
            }
        }

        if !diagnostic.get_ref_result().is_empty() {
            Some(DiagnosticType::DependencyManager(diagnostic))
        } else { None }
    }

    /// This function takes care of checking RPFM's configuration for errors.
    fn check_config(dependencies: &Dependencies) ->Option<DiagnosticType> {
        let mut diagnostic = ConfigDiagnostic::new();

        // First, check if the dependencies are generated. We can't do shit without them.
        if !dependencies.game_has_dependencies_generated() {
            diagnostic.get_ref_mut_result().push(
                ConfigDiagnosticReport {
                    message: "Dependency Cache not generated for the currently selected game.".to_owned(),
                    report_type: ConfigDiagnosticReportType::DependenciesCacheNotGenerated,
                    level: DiagnosticLevel::Error,
                }
            );
        } else {

            // Second, check if the dependencies are valid. we can't actually use them if not.
            match dependencies.needs_updating() {
                Ok(needs_updating) => {
                    if needs_updating {
                        diagnostic.get_ref_mut_result().push(
                            ConfigDiagnosticReport {
                                message: "Dependency Cache for the selected game is outdated and could not be loaded.".to_owned(),
                                report_type: ConfigDiagnosticReportType::DependenciesCacheOutdated,
                                level: DiagnosticLevel::Error,
                            }
                        );
                    }
                }

                Err(error) => {
                    diagnostic.get_ref_mut_result().push(
                        ConfigDiagnosticReport {
                            message: "Dependency Cache couldn't be loaded for the game selected, due to errors reading the game's folder.".to_owned(),
                            report_type: ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error.to_string()),
                            level: DiagnosticLevel::Error,
                        }
                    );

                }
            }
        }

        if let Some(path) = GAME_SELECTED.read().unwrap().get_executable_path() {
            if !path.is_file() {
                diagnostic.get_ref_mut_result().push(
                    ConfigDiagnosticReport {
                        message: "Game Path for the current Game Selected is incorrect.".to_owned(),
                        report_type: ConfigDiagnosticReportType::IncorrectGamePath,
                        level: DiagnosticLevel::Error,
                    }
                );
            }
        }

        if !diagnostic.get_ref_result().is_empty() {
            Some(DiagnosticType::Config(diagnostic))
        } else { None }
    }

    /// This function performs a limited diagnostic check on the `PackedFiles` in the provided paths, and updates the `Diagnostic` with the results.
    ///
    /// This means that, as long as you change any `PackedFile` in the `PackFile`, you should trigger this. That way, the `Diagnostics`
    /// will always be up-to-date in an efficient way.
    ///
    /// If you passed the entire `PackFile` to this and it crashed, it's not an error. I forced that crash. If you want to do that,
    /// use the normal check function, because it's a lot more efficient than this one.
    pub fn update(&mut self, pack_file: &PackFile, updated_paths: &[PathType], dependencies: &Dependencies) {

        // First, remove all current config blocking diagnostics, so they get check properly again.
        self.0.iter_mut().for_each(|x| {
            if let DiagnosticType::Config(config) = x {
                config.get_ref_mut_result().retain(|x|
                    match x.report_type {
                        ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                        ConfigDiagnosticReportType::DependenciesCacheOutdated |
                        ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) |
                        ConfigDiagnosticReportType::IncorrectGamePath => false,
                    }
                );
            }
        });

        // Next, check for config issues, as some of them may stop the checking prematurely.
        if let Some(diagnostics) = Self::check_config(dependencies) {
            let is_diagnostic_blocking = if let DiagnosticType::Config(ref diagnostic) = diagnostics {
                diagnostic.get_ref_result().iter().any(|diagnostic| matches!(diagnostic.report_type,
                    ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                    ConfigDiagnosticReportType::DependenciesCacheOutdated |
                    ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_)))
            } else { false };

            // If we have one of the blocking diagnostics, report it and return.
            self.0.push(diagnostics);
            if is_diagnostic_blocking {
                return;
            }
        }

        // Turn all our updated packs into `PackedFile` paths, and get them. Keep in mind we need to also get:
        // - Other tables related with the ones we update.
        // - Other locs, if a loc is on the list.
        let mut packed_files = vec![];
        for path_type in updated_paths {
            match path_type {
                PathType::File(path) => if let Some(packed_file) = pack_file.get_ref_packed_file_by_path(path) { packed_files.push(packed_file) },
                PathType::Folder(path) => packed_files.append(&mut pack_file.get_ref_packed_files_by_path_start(path)),

                // PackFile in this instance means the dependency manager.
                PathType::PackFile => {}
                _ => unimplemented!()
            }
        }

        let mut packed_files_complete: Vec<&PackedFile> = vec![];
        let mut locs_added = false;
        for packed_file in &packed_files {
            match packed_file.get_packed_file_type(false) {
                PackedFileType::AnimFragment => {
                    packed_files_complete.push(packed_file);
                }
                PackedFileType::DB => {
                    let tables = pack_file.get_ref_packed_files_by_path_start(&packed_file.get_path()[..=1]);
                    for table in tables {
                        if !packed_files_complete.contains(&table) {
                            packed_files_complete.push(table);
                        }
                    }
                },
                PackedFileType::Loc => {
                    if !locs_added {
                        packed_files_complete.append(&mut pack_file.get_ref_packed_files_by_type(PackedFileType::Loc, false));
                        locs_added = true;
                    }
                }

                _ => packed_files_complete.push(packed_file),
            }
        }

        // We remove the added/edited/deleted files from all the search.
        for packed_file in &packed_files_complete {
            self.get_ref_mut_diagnostics().retain(|x| x.get_path() != packed_file.get_path());
        }

        // Also remove Dependency/PackFile diagnostics, as they're going to be regenerated.
        self.get_ref_mut_diagnostics().retain(|x| match x {
            DiagnosticType::AnimFragment(_) => true,
            DiagnosticType::Config(_) => true,
            DiagnosticType::DB(_) => true,
            DiagnosticType::DependencyManager(_) => false,
            DiagnosticType::Loc(_) => true,
            DiagnosticType::PackFile(_) => false,
        });

        let files_to_ignore = pack_file.get_settings().get_diagnostics_files_to_ignore();

        // Prefetch them here, so we don't need to re-search them again.
        let vanilla_dependencies = if let Ok(dependencies) = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true) { dependencies } else { return };
        let asskit_dependencies = dependencies.get_ref_asskit_only_db_tables();

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let mut packed_files_split: BTreeMap<&str, Vec<&PackedFile>> = BTreeMap::new();
        for packed_file in &packed_files_complete {
            match packed_file.get_packed_file_type(false) {
                PackedFileType::AnimFragment => {
                    if let Some(table_set) = packed_files_split.get_mut("anim_fragments") {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert("anim_fragments", vec![packed_file]);
                    }
                },
                PackedFileType::DB => {
                    if let Some(table_set) = packed_files_split.get_mut(&*packed_file.get_path()[1]) {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert(&packed_file.get_path()[1], vec![packed_file]);
                    }
                },
                PackedFileType::Loc => {
                    if let Some(table_set) = packed_files_split.get_mut("locs") {
                        table_set.push(packed_file);
                    } else {
                        packed_files_split.insert("locs", vec![packed_file]);
                    }
                },
                _ => {},
            }
        }
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            let local_packed_file_path_list = pack_file.get_packed_files_all_paths_as_string();
            let local_folder_path_list = pack_file.get_folder_all_paths_as_string();

            for packed_files in packed_files_split.values() {
                let mut data_prev: BTreeMap<String, HashMap<String, Vec<(i32, i32)>>> = BTreeMap::new();
                let mut dependency_data_for_table = BTreeMap::new();

                for packed_file in packed_files {

                    // Prepare the ignore data for this PackedFile.
                    let mut ignored_file = false;
                    let mut ignored_fields = vec![];
                    let mut ignored_diagnostics = vec![];
                    let mut ignored_diagnostics_for_fields: HashMap<String, Vec<String>> = HashMap::new();
                    if let Some(ref files_to_ignore) = files_to_ignore {
                        for (path_to_ignore, fields, diags_to_ignore) in files_to_ignore {

                            // If the rule doesn't affect this PackedFile, ignore it.
                            if !path_to_ignore.is_empty() && packed_file.get_path().starts_with(path_to_ignore) {

                                // If we don't have either fields or diags specified, we ignore the entire file.
                                if fields.is_empty() && diags_to_ignore.is_empty() {
                                    ignored_file = true;
                                    break;
                                }

                                // If we have both, fields and diags, disable only those diags for those fields.
                                if !fields.is_empty() && !diags_to_ignore.is_empty() {
                                    for field in fields {
                                        match ignored_diagnostics_for_fields.get_mut(field) {
                                            Some(diagnostics) => diagnostics.append(&mut diags_to_ignore.to_vec()),
                                            None => { ignored_diagnostics_for_fields.insert(field.to_owned(), diags_to_ignore.to_vec()); },
                                        }
                                    }
                                }

                                // Otherwise, check if we only have fields or diags, and put them separately.
                                else if !fields.is_empty() {
                                    ignored_fields.append(&mut fields.to_vec());
                                }

                                else if !diags_to_ignore.is_empty() {
                                    ignored_diagnostics.append(&mut diags_to_ignore.to_vec());
                                }
                            }
                        }
                    }

                    // If we ignore this full file, skip to the next one.
                    if ignored_file {
                        continue;
                    }

                    let diagnostic = match packed_file.get_packed_file_type(false) {
                        PackedFileType::AnimFragment => if let Ok(decoded) = packed_file.decode_return_ref_no_cache_no_locks(schema) {
                                Self::check_anim_fragment(&decoded, packed_file.get_path(), dependencies, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &local_packed_file_path_list, &local_folder_path_list)
                            } else { None }
                        PackedFileType::DB => {

                            // Get the dependency data for tables once per batch.
                            // That way we can speed up this a lot.
                            let decoded_packed_file = packed_file.get_ref_decoded();
                            if dependency_data_for_table.is_empty() {
                                if let DecodedPackedFile::DB(table) = decoded_packed_file {
                                    dependency_data_for_table = DB::get_dependency_data(
                                        pack_file,
                                        table.get_ref_table_name(),
                                        table.get_ref_definition(),
                                        &vanilla_dependencies,
                                        asskit_dependencies,
                                        dependencies,
                                        &[],
                                    );
                                }
                            }

                            Self::check_db(packed_file.get_ref_decoded(), packed_file.get_path(), dependencies, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &mut data_prev, &local_packed_file_path_list, &local_folder_path_list, &dependency_data_for_table)
                        },
                        PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path(), &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, &mut data_prev),
                        _ => None,
                    };

                    if let Some(diagnostic) = diagnostic {
                        self.get_ref_mut_diagnostics().push(diagnostic);
                    }
                }
            }
        }

        // Check for the dependency manager.
        if let Some(diagnostics) = Self::check_dependency_manager(pack_file) {
            self.0.push(diagnostics);
        }

        // Check for the PackFile.
        if let Some(diagnostics) = Self::check_packfile(pack_file) {
            self.0.push(diagnostics);
        }

        self.get_ref_mut_diagnostics().sort_by(|a, b| {
            if !a.get_path().is_empty() && !b.get_path().is_empty() {
                a.get_path().cmp(b.get_path())
            } else if a.get_path().is_empty() && !b.get_path().is_empty() {
                Ordering::Greater
            } else if !a.get_path().is_empty() && b.get_path().is_empty() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }

    /// This function returns the PackedFileInfo for all the PackedFiles with the provided paths.
    pub fn get_update_paths_packed_file_info(&self, pack_file: &PackFile, paths: &[PathType]) -> Vec<PackedFileInfo> {
        let paths = paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect();
        let packed_files = pack_file.get_ref_packed_files_by_paths(paths);
        packed_files.iter().map(|x| From::from(*x)).collect()
    }

    /// Function to know if an specific field/diagnostic must be ignored.
    fn ignore_diagnostic(field_name: Option<&str>, diagnostic: Option<&str>, ignored_fields: &[String], ignored_diagnostics: &[String], ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>) -> bool {
        let mut ignore_diagnostic = false;

        // If we have a field, and it's in the ignored list, ignore it.
        if let Some(field_name) = field_name {
            ignore_diagnostic = ignored_fields.iter().any(|x| x == field_name);
        }

        // If we have a diagnostic, and it's in the ignored list, ignore it.
        else if let Some(diagnostic) = diagnostic {
            ignore_diagnostic = ignored_diagnostics.iter().any(|x| x == diagnostic);
        }

        // If we have not yet being ignored, check for specific diagnostics for specific fields.
        if !ignore_diagnostic {
            if let Some(field_name) = field_name {
                if let Some(diagnostic) = diagnostic {
                    if let Some(diags) = ignored_diagnostics_for_fields.get(field_name) {
                        ignore_diagnostic = diags.iter().any(|x| x == diagnostic);
                    }
                }
            }
        }

        ignore_diagnostic
    }
}

impl Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::AnimFragment(_) => "AnimFragment",
            Self::Config(_) => "Config",
            Self::DB(_) => "DB",
            Self::Loc(_) => "Loc",
            Self::PackFile(_) => "Packfile",
            Self::DependencyManager(_) => "DependencyManager",
        }, f)
    }
}
