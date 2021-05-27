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

This module contains the code needed to get a `Diagnostics` over an entire `PackFile`.
!*/

use rayon::prelude::*;
use fancy_regex::Regex;

use std::{fmt, fmt::Display};
use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::DB;
use crate::dependencies::Dependencies;
use crate::games::VanillaDBTableNameLogic;
use crate::GAME_SELECTED;
use crate::packfile::{PackFile, PathType};
use crate::packedfile::{table::DecodedData, DecodedPackedFile, PackedFileType};
use crate::packfile::packedfile::{PackedFile, PackedFileInfo};
use crate::schema::FieldType;
use crate::SUPPORTED_GAMES;

use self::config::{ConfigDiagnostic, ConfigDiagnosticReport, ConfigDiagnosticReportType};
use self::dependency_manager::{DependencyManagerDiagnostic, DependencyManagerDiagnosticReport, DependencyManagerDiagnosticReportType};
use self::packfile::{PackFileDiagnostic, PackFileDiagnosticReport, PackFileDiagnosticReportType};
use self::table::{TableDiagnostic, TableDiagnosticReport, TableDiagnosticReportType};

pub mod config;
pub mod dependency_manager;
pub mod packfile;
pub mod table;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over multiple PackedFiles.
#[derive(Debug, Clone)]
pub struct Diagnostics(Vec<DiagnosticType>);

#[derive(Debug, Clone)]
pub enum DiagnosticType {
    DB(TableDiagnostic),
    Loc(TableDiagnostic),
    PackFile(PackFileDiagnostic),
    DependencyManager(DependencyManagerDiagnostic),
    Config(ConfigDiagnostic),
}

/// This enum defines the possible results for a result of a diagnostic check.
#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

//---------------------------------------------------------------p----------------//
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
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.get_path(),
            Self::PackFile(_) => &[],
            Self::DependencyManager(_) => &[],
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

        // First, check if the dependencies are generated. We can't do shit without them.
        let mut config_diagnostic = ConfigDiagnostic::new();
        if !dependencies.game_has_dependencies_generated() {
            config_diagnostic.get_ref_mut_result().push(
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
                        config_diagnostic.get_ref_mut_result().push(
                            ConfigDiagnosticReport {
                                message: "Dependency Cache for the selected game is outdated and could not be loaded.".to_owned(),
                                report_type: ConfigDiagnosticReportType::DependenciesCacheOutdated,
                                level: DiagnosticLevel::Error,
                            }
                        );
                    }
                }

                Err(error) => {
                    config_diagnostic.get_ref_mut_result().push(
                        ConfigDiagnosticReport {
                            message: "Dependency Cache couldn't be loaded for the game selected, due to errors reading the game's folder.".to_owned(),
                            report_type: ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error.to_string()),
                            level: DiagnosticLevel::Error,
                        }
                    );

                }
            }
        }

        if !config_diagnostic.get_ref_result().is_empty() {
            self.0.push(DiagnosticType::Config(config_diagnostic));
            return;
        }


        let files_to_ignore = pack_file.get_settings().get_diagnostics_files_to_ignore();

        // Prefetch them here, so we don't need to re-search them again.
        let vanilla_dependencies = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true);
        let asskit_dependencies = dependencies.get_ref_asskit_only_db_tables();

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let packed_files = pack_file.get_ref_packed_files_by_types(&[PackedFileType::DB, PackedFileType::Loc], false);
        let mut packed_files_split: BTreeMap<&str, Vec<&PackedFile>> = BTreeMap::new();

        for packed_file in &packed_files {
            match packed_file.get_packed_file_type(false) {
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

        // Process the files in batches.
        self.0 = packed_files_split.into_par_iter().filter_map(|(_, packed_files)| {

            let mut diagnostics = Vec::with_capacity(packed_files.len());
            let mut data_prev = Vec::with_capacity(packed_files.len());
            for packed_file in packed_files {

                // Ignore entire tables if their path starts with the one we have (so we can do mass ignores) and we didn't specified a field to ignore.
                let mut ignored_fields = vec![];
                let mut ignored_diagnostics = vec![];
                if let Some(ref files_to_ignore) = files_to_ignore {
                    for (file_to_ignore, fields, diags_to_ignore) in files_to_ignore {
                        if !file_to_ignore.is_empty() && packed_file.get_path().starts_with(&file_to_ignore) && fields.is_empty() && diags_to_ignore.is_empty() {
                            return None;
                        } else if !file_to_ignore.is_empty() && packed_file.get_path().starts_with(&file_to_ignore) {
                            if !fields.is_empty() {
                                ignored_fields = fields.to_vec();
                            }

                            if !diags_to_ignore.is_empty() {
                                ignored_diagnostics = diags_to_ignore.to_vec();
                            }

                            break;
                        }
                    }
                }

                let diagnostic = match packed_file.get_packed_file_type(false) {
                    PackedFileType::DB => Self::check_db(pack_file, packed_file.get_ref_decoded(), packed_file.get_path(), &dependencies, &vanilla_dependencies, &asskit_dependencies, &ignored_fields, &ignored_diagnostics, &mut data_prev),
                    PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path(), &ignored_fields, &ignored_diagnostics, &mut data_prev),
                    _ => None,
                };

                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(diagnostic);
                }
            }

            Some(diagnostics)
        }).flatten().collect();

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
        pack_file: &PackFile,
        packed_file: &DecodedPackedFile,
        path: &[String],
        dependencies: &Dependencies,
        vanilla_dependencies: &[PackedFile],
        asskit_dependencies: &[DB],
        ignored_fields: &[String],
        ignored_diagnostics: &[String],
        previous_data: &mut Vec<Vec<String>>,
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::DB(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);
            let dependency_data = DB::get_dependency_data(
                &pack_file,
                table.get_ref_table_name(),
                table.get_ref_definition(),
                vanilla_dependencies,
                asskit_dependencies,
                &dependencies,
                &[],
            );

            // Check all the columns with reference data.
            let mut columns_without_reference_table = vec![];
            let mut columns_with_reference_table_and_no_column = vec![];
            let mut keys = vec![];

            // Before anything else, check if the table is outdated.
            if ignored_diagnostics.iter().all(|x| x != "OutdatedTable") {
                if table.is_outdated(&dependencies) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: -1,
                        message: "Possibly outdated table.".to_owned(),
                        report_type: TableDiagnosticReportType::OutdatedTable,
                        level: DiagnosticLevel::Error,
                    });
                }
            }

            // Check if the table name has a number at the end, which causes very annoying bugs.
            if let Some(ref name) = path.last() {
                if name.ends_with("0") ||
                    name.ends_with("1") ||
                    name.ends_with("2") ||
                    name.ends_with("3") ||
                    name.ends_with("4") ||
                    name.ends_with("5") ||
                    name.ends_with("6") ||
                    name.ends_with("7") ||
                    name.ends_with("8") ||
                    name.ends_with("9") {

                    if ignored_diagnostics.iter().all(|x| x != "TableNameEndsInNumber") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: -1,
                            message: "Table name ends in number.".to_owned(),
                            report_type: TableDiagnosticReportType::TableNameEndsInNumber,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if name.contains(' ') {
                    if ignored_diagnostics.iter().all(|x| x != "TableNameHasSpace") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: -1,
                            message: "Table name contains spaces.".to_owned(),
                            report_type: TableDiagnosticReportType::TableNameHasSpace,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if ignored_diagnostics.iter().all(|x| x != "TableIsDataCoring") {
                    if let Some(supported_game) = SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()) {
                        match supported_game.vanilla_db_table_name_logic {
                            VanillaDBTableNameLogic::FolderName => {
                                if table.get_table_name_without_tables() == path[1] {
                                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                        column_number: 0,
                                        row_number: -1,
                                        message: "Table is datacoring.".to_owned(),
                                        report_type: TableDiagnosticReportType::TableIsDataCoring,
                                        level: DiagnosticLevel::Warning,
                                    });
                                }
                            }

                            VanillaDBTableNameLogic::DefaultName(ref default_name) => {
                                if *name == default_name {
                                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                        column_number: 0,
                                        row_number: -1,
                                        message: "Table is datacoring.".to_owned(),
                                        report_type: TableDiagnosticReportType::TableIsDataCoring,
                                        level: DiagnosticLevel::Warning,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let mut row_is_empty = true;
                let mut row_keys_are_empty = true;
                let mut local_keys = vec![];
                for (column, field) in table.get_ref_definition().get_fields_processed().iter().enumerate() {
                    if ignored_fields.contains(&field.get_name().to_owned()) {
                        continue;
                    }

                    let cell_data = cells[column].data_to_string();

                    // Dependency checks.
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
                                    if ignored_diagnostics.iter().all(|x| x != "InvalidReference") {
                                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                            column_number: column as u32,
                                            row_number: row as i64,
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

                    // Check for empty keys/rows.
                    if row_is_empty && (!cell_data.is_empty() && cell_data != "false") {
                        row_is_empty = false;
                    }

                    if row_keys_are_empty && field.get_is_key() && (!cell_data.is_empty() && cell_data != "false") {
                        row_keys_are_empty = false;
                    }

                    if field.get_is_key() && field.get_field_type() != FieldType::OptionalStringU8 && field.get_field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                        if ignored_diagnostics.iter().all(|x| x != "EmptyKeyField") {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                column_number: column as u32,
                                row_number: row as i64,
                                message: format!("Empty key for column \"{}\".", field.get_name()),
                                report_type: TableDiagnosticReportType::EmptyKeyField,
                                level: DiagnosticLevel::Warning,
                            });
                        }
                    }

                    if field.get_is_key() {
                        local_keys.push(cell_data);
                    }
                }

                if row_is_empty {
                    if ignored_diagnostics.iter().all(|x| x != "EmptyRow") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: row as i64,
                            message: "Empty row.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyRow,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if row_keys_are_empty {
                    if ignored_diagnostics.iter().all(|x| x != "EmptyKeyFields") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: row as i64,
                            message: "Empty key fields.".to_string(),
                            report_type: TableDiagnosticReportType::EmptyKeyFields,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                if !local_keys.is_empty() && (keys.contains(&local_keys) || previous_data.contains(&local_keys)) {
                    if ignored_diagnostics.iter().all(|x| x != "DuplicatedCombinedKeys") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: row as i64,
                            message: format!("Duplicated combined keys: {}.", local_keys.join("| |")),
                            report_type: TableDiagnosticReportType::DuplicatedCombinedKeys,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }
                else {
                    keys.push(local_keys);
                }
            }

            // Checks that only need to be done once per table.
            for column in &columns_without_reference_table {
                if ignored_diagnostics.iter().all(|x| x != "NoReferenceTableFound") {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: *column as u32,
                        row_number: -1,
                        message: format!("No reference table found for column \"{}\".", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                        report_type: TableDiagnosticReportType::NoReferenceTableFound,
                        level: DiagnosticLevel::Info,
                    });
                }
            }

            for column in &columns_with_reference_table_and_no_column {
                if !dependencies.game_has_asskit_data_loaded() {
                    if ignored_diagnostics.iter().all(|x| x != "NoReferenceTableNorColumnFoundNoPak") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: *column as u32,
                            row_number: -1,
                            message: format!("No reference column found in referenced table for column \"{}\". Did you forgot to generate the PAK file for this game?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                            report_type: TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }
                else {
                    if ignored_diagnostics.iter().all(|x| x != "NoReferenceTableNorColumnFoundPak") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: *column as u32,
                            row_number: -1,
                            message: format!("No reference column found in referenced table for column \"{}\". Maybe a problem with the schema?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                            report_type: TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak,
                            level: DiagnosticLevel::Info,
                        });
                    }
                }
            }

            // Add this table's keys to the previous list, so they can be queried for for duplicate checks on other tables of the same type.
            previous_data.append(&mut keys);

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::DB(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_loc(
        packed_file: &DecodedPackedFile,
        path: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &[String],
        previous_data: &mut Vec<Vec<String>>,
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::Loc(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);

            // Check all the columns with reference data.
            let mut keys = vec![];
            let fields = table.get_ref_definition().get_fields_processed();
            let field_key_name = fields[0].get_name().to_owned();
            let field_text_name = fields[1].get_name().to_owned();

            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let key = if let DecodedData::StringU16(ref data) = cells[0] { data } else { unimplemented!() };
                let data = if let DecodedData::StringU16(ref data) = cells[1] { data } else { unimplemented!() };

                if !key.is_empty() && (key.contains('\n') || key.contains('\t')) {
                    if ignored_diagnostics.iter().all(|x| x != "InvalidLocKey") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 0,
                            row_number: row as i64,
                            message: "Invalid localisation key.".to_string(),
                            report_type: TableDiagnosticReportType::InvalidLocKey,
                            level: DiagnosticLevel::Error,
                        });
                    }
                }

                if !ignored_fields.contains(&field_key_name) && !ignored_fields.contains(&field_text_name) {
                    if key.is_empty() && data.is_empty() {
                        if ignored_diagnostics.iter().all(|x| x != "EmptyRow") {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                column_number: 0,
                                row_number: row as i64,
                                message: "Empty row.".to_string(),
                                report_type: TableDiagnosticReportType::EmptyRow,
                                level: DiagnosticLevel::Warning,
                            });
                        }
                    }

                    if key.is_empty() && !data.is_empty() {
                        if ignored_diagnostics.iter().all(|x| x != "EmptyKeyField") {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                column_number: 0,
                                row_number: row as i64,
                                message: "Empty key.".to_string(),
                                report_type: TableDiagnosticReportType::EmptyKeyField,
                                level: DiagnosticLevel::Warning,
                            });
                        }
                    }
                }

                // Magic Regex. It works. Don't ask why.
                if !ignored_fields.contains(&field_text_name) && !data.is_empty() && Regex::new(r"(?<!\\)\\n|(?<!\\)\\t").unwrap().is_match(data).unwrap() {
                    if ignored_diagnostics.iter().all(|x| x != "InvalidEscape") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: 1,
                            row_number: row as i64,
                            message: "Invalid line jump/tabulation detected in loc entry. Use \\\\n or \\\\t instead.".to_string(),
                            report_type: TableDiagnosticReportType::InvalidEscape,
                            level: DiagnosticLevel::Warning,
                        });
                    }
                }

                if !ignored_fields.contains(&field_key_name) {
                    let local_keys = vec![key.to_owned(), data.to_owned()];
                    if keys.contains(&local_keys) || previous_data.contains(&local_keys) {
                        if ignored_diagnostics.iter().all(|x| x != "DuplicatedRow") {
                            diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                column_number: 0,
                                row_number: row as i64,
                                message: "Duplicated row.".to_string(),
                                report_type: TableDiagnosticReportType::DuplicatedRow,
                                level: DiagnosticLevel::Warning,
                            });
                        }
                    }
                    else {
                        keys.push(local_keys);
                    }
                }
            }

            // Add this table's keys to the previous list, so they can be queried for for duplicate checks on other tables of the same type.
            previous_data.append(&mut keys);

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
                    column_number: 0,
                    row_number: index as i64,
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


    /// This function performs a limited diagnostic check on the `PackedFiles` in the provided paths, and updates the `Diagnostic` with the results.
    ///
    /// This means that, as long as you change any `PackedFile` in the `PackFile`, you should trigger this. That way, the `Diagnostics`
    /// will always be up-to-date in an efficient way.
    ///
    /// If you passed the entire `PackFile` to this and it crashed, it's not an error. I forced that crash. If you want to do that,
    /// use the normal check function, because it's a lot more efficient than this one.
    pub fn update(&mut self, pack_file: &PackFile, updated_paths: &[PathType], dependencies: &Dependencies) {

        self.0.iter_mut().for_each(|x| {
            if let DiagnosticType::Config(config) = x {
                config.get_ref_mut_result().retain(|x|
                    match x.report_type {
                        ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                        ConfigDiagnosticReportType::DependenciesCacheOutdated |
                        ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) => false
                    }
                );
            }
        });

        // First, check if the dependencies are generated. We can't do shit without them.
        let mut config_diagnostic = ConfigDiagnostic::new();
        if !dependencies.game_has_dependencies_generated() {
            config_diagnostic.get_ref_mut_result().push(
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
                        config_diagnostic.get_ref_mut_result().push(
                            ConfigDiagnosticReport {
                                message: "Dependency Cache for the selected game is outdated and could not be loaded.".to_owned(),
                                report_type: ConfigDiagnosticReportType::DependenciesCacheOutdated,
                                level: DiagnosticLevel::Error,
                            }
                        );
                    }
                }

                Err(error) => {
                    config_diagnostic.get_ref_mut_result().push(
                        ConfigDiagnosticReport {
                            message: "Dependency Cache couldn't be loaded for the game selected, due to errors reading the game's folder.".to_owned(),
                            report_type: ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error.to_string()),
                            level: DiagnosticLevel::Error,
                        }
                    );

                }
            }
        }

        if !config_diagnostic.get_ref_result().is_empty() {
            self.0.push(DiagnosticType::Config(config_diagnostic));
            return;
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
            DiagnosticType::Config(_) => true,
            DiagnosticType::DB(_) => true,
            DiagnosticType::DependencyManager(_) => false,
            DiagnosticType::Loc(_) => true,
            DiagnosticType::PackFile(_) => false,
        });

        let files_to_ignore = pack_file.get_settings().get_diagnostics_files_to_ignore();

        // Prefetch them here, so we don't need to re-search them again.
        let vanilla_dependencies = dependencies.get_db_and_loc_tables_from_cache(true, false, true, true);
        let asskit_dependencies = dependencies.get_ref_asskit_only_db_tables();

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let mut packed_files_split: BTreeMap<&str, Vec<&PackedFile>> = BTreeMap::new();
        for packed_file in &packed_files_complete {
            match packed_file.get_packed_file_type(false) {
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

        for (_, packed_files) in &packed_files_split {
            let mut data_prev = Vec::with_capacity(packed_files.len());

            for packed_file in packed_files {
                let mut ignored_fields = vec![];
                let mut ignored_diagnostics = vec![];
                if let Some(ref files_to_ignore) = files_to_ignore {
                    for (file_to_ignore, fields, diags_to_ignore) in files_to_ignore {
                        if !file_to_ignore.is_empty() && packed_file.get_path().starts_with(&file_to_ignore) && fields.is_empty() && diags_to_ignore.is_empty() {
                            continue;
                        } else if !file_to_ignore.is_empty() && packed_file.get_path().starts_with(&file_to_ignore) {
                            if !fields.is_empty() {
                                ignored_fields = fields.to_vec();
                            }

                            if !diags_to_ignore.is_empty() {
                                ignored_diagnostics = diags_to_ignore.to_vec();
                            }

                            break;
                        }
                    }
                }

                let diagnostic = match packed_file.get_packed_file_type(false) {
                    PackedFileType::DB => Self::check_db(pack_file, packed_file.get_ref_decoded(), packed_file.get_path(), &dependencies, &vanilla_dependencies, &asskit_dependencies, &ignored_fields, &ignored_diagnostics, &mut data_prev),
                    PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path(), &ignored_fields, &ignored_diagnostics, &mut data_prev),
                    _ => None,
                };

                if let Some(diagnostic) = diagnostic {
                    self.get_ref_mut_diagnostics().push(diagnostic);
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
}

impl Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::Config(_) => "Config",
            Self::DB(_) => "DB",
            Self::Loc(_) => "Loc",
            Self::PackFile(_) => "Packfile",
            Self::DependencyManager(_) => "DependencyManager",
        }, f)
    }
}
