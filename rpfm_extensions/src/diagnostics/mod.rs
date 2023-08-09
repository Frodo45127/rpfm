//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use getset::{Getters, MutGetters};
use itertools::Itertools;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::path::Path;
use std::{fmt, fmt::Display};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use rpfm_lib::error::Result;
use rpfm_lib::files::{ContainerPath, Container, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::{FieldType, Schema};

use crate::dependencies::{Dependencies, TableReferences};
use crate::REGEX_INVALID_ESCAPES;

use self::anim_fragment::*;
use self::config::*;
use self::dependency::*;
use self::pack::*;
use self::portrait_settings::*;
use self::table::*;

pub mod anim_fragment;
pub mod config;
pub mod dependency;
pub mod pack;
pub mod portrait_settings;
pub mod table;

//-------------------------------------------------------------------------------//
//                              Trait definitions
//-------------------------------------------------------------------------------//

/// This trait represents a diagnostic with a level and a message.
pub trait DiagnosticReport {

    /// This function returns the message associated with the diagnostic implementing this.
    fn message(&self) -> String;

    /// This function returns the level associated with the diagnostic implementing this.
    fn level(&self) -> DiagnosticLevel;
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the results of a diagnostics check over a Pack.
///
/// It also contains some configuration used on the diagnostic themselfs.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Diagnostics {

    /// List of ignored folders for diagnostics.
    folders_ignored: Vec<String>,

    /// List of ignored files for diagnostics.
    files_ignored: Vec<String>,

    /// List of ignored table fields for diagnostics.
    fields_ignored: Vec<String>,

    /// List of ignored diagnostics.
    diagnostics_ignored: Vec<String>,

    /// Results of a diagnostics check.
    results: Vec<DiagnosticType>
}

/// This enum contains the different types of diagnostics we can have.
///
/// One enum to hold them all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticType {
    AnimFragment(AnimFragmentDiagnostic),
    Config(ConfigDiagnostic),
    Dependency(DependencyDiagnostic),
    DB(TableDiagnostic),
    Loc(TableDiagnostic),
    Pack(PackDiagnostic),
    PortraitSettings(PortraitSettingsDiagnostic),
}

/// This enum defines the possible level of a diagnostic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    #[default]
    Info,
    Warning,
    Error,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Default for DiagnosticType {
    fn default() -> Self {
        Self::Pack(PackDiagnostic::default())
    }
}

impl DiagnosticType {
    pub fn path(&self) -> &str {
        match self {
            Self::AnimFragment(ref diag) => diag.path(),
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.path(),
            Self::Pack(_) => "",
            Self::PortraitSettings(diag) => diag.path(),
            Self::Dependency(diag) => diag.path(),
            Self::Config(_) => "",
        }
    }
}

impl Diagnostics {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn check(&mut self, pack: &Pack, dependencies: &mut Dependencies, game_info: &GameInfo, game_path: &Path, paths_to_check: &[ContainerPath], schema: &Schema, check_ak_only_refs: bool) {

        // Clear the diagnostics first if we're doing a full check, or only the config ones and the ones for the path to update if we're doing a partial check.
        if paths_to_check.is_empty() {
            self.results.clear();
        } else {
            self.results.retain(|diagnostic| !paths_to_check.contains(&ContainerPath::File(diagnostic.path().to_string())));
            self.results.iter_mut().for_each(|x| {
                if let DiagnosticType::Config(config) = x {
                    config.results_mut().retain(|x|
                        match x.report_type() {
                            ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                            ConfigDiagnosticReportType::DependenciesCacheOutdated |
                            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) |
                            ConfigDiagnosticReportType::IncorrectGamePath => false,
                        }
                    );
                }
            });
        }

        // First, check for config issues, as some of them may stop the checking prematurely.
        if let Some(diagnostics) = Self::check_config(dependencies, game_info, game_path) {
            let is_diagnostic_blocking = if let DiagnosticType::Config(ref diagnostic) = diagnostics {
                diagnostic.results().iter().any(|diagnostic| matches!(diagnostic.report_type(),
                    ConfigDiagnosticReportType::IncorrectGamePath |
                    ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                    ConfigDiagnosticReportType::DependenciesCacheOutdated |
                    ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_)))
            } else { false };

            // If we have one of the blocking diagnostics, report it and return.
            self.results.push(diagnostics);
            if is_diagnostic_blocking {
                return;
            }
        }

        let files_to_ignore = pack.settings().diagnostics_files_to_ignore();

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let files = if paths_to_check.is_empty() {
            pack.files_by_type(&[FileType::AnimFragment, FileType::DB, FileType::Loc, FileType::PortraitSettings])
        } else {
            pack.files_by_type_and_paths(&[FileType::AnimFragment, FileType::DB, FileType::Loc, FileType::PortraitSettings], paths_to_check, false)
        };

        let mut files_split: HashMap<&str, Vec<&RFile>> = HashMap::new();

        for file in &files {
            match file.file_type() {
                FileType::AnimFragment => {
                    if let Some(table_set) = files_split.get_mut("anim_fragments") {
                        table_set.push(file);
                    } else {
                        files_split.insert("anim_fragments", vec![file]);
                    }
                },
                FileType::DB => {
                    let path_split = file.path_in_container_split();
                    if path_split.len() > 2 {
                        if let Some(table_set) = files_split.get_mut(path_split[1]) {
                            table_set.push(file);
                        } else {
                            files_split.insert(path_split[1], vec![file]);
                        }
                    }
                },
                FileType::Loc => {
                    if let Some(table_set) = files_split.get_mut("locs") {
                        table_set.push(file);
                    } else {
                        files_split.insert("locs", vec![file]);
                    }
                },
                FileType::PortraitSettings => {
                    if let Some(table_set) = files_split.get_mut("portrait_settings") {
                        table_set.push(file);
                    } else {
                        files_split.insert("portrait_settings", vec![file]);
                    }
                },
                _ => {},
            }
        }

        // Getting this here speeds up a lot path-checking later.
        let local_file_path_list = pack.paths_raw().into_iter().collect::<HashSet<_>>();
        let local_folder_path_list = pack.paths_folders_raw();

        // TODO: Currently, AnimFragments are only check if they've been preloaded. Fix that

        // TODO: Get the table reference data here, outside the parallel loop.
        // That way we can get it fast on the first try, and skip.
        let table_names = files_split.iter().filter(|(key, _)| **key != "anim_fragments" && **key != "locs" && **key != "portrait_settings").map(|(key, _)| key.to_string()).collect::<Vec<_>>();
        dependencies.generate_local_db_references(pack, &table_names);

        // Caches for Portrait Settings diagnostics.
        let art_set_ids = dependencies.db_values_from_table_name_and_column_name(Some(pack), "campaign_character_arts_tables", "art_set_id", true, true);
        let variant_filenames = dependencies.db_values_from_table_name_and_column_name(Some(pack), "variants_tables", "variant_filename", true, true);

        // Process the files in batches.
        self.results.append(&mut files_split.par_iter().filter_map(|(_, files)| {

            let mut diagnostics = Vec::with_capacity(files.len());
            let mut table_references = HashMap::new();

            for file in files {
                let (ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) = Self::ignore_data_for_file(file, &files_to_ignore)?;

                let diagnostic = match file.file_type() {
                    //FileType::AnimFragment => Self::check_anim_fragment(
                    //    file,
                    //    dependencies,
                    //    &self.diagnostics_ignored,
                    //    &ignored_fields,
                    //    &ignored_diagnostics,
                    //    &ignored_diagnostics_for_fields,
                    //    &local_file_path_list,
                    //    &local_folder_path_list
                    //),
                    FileType::DB => {

                        // Get the dependency data for tables once per batch.
                        // That way we can speed up this a lot.
                        let file_decoded = file.decoded().ok()?;
                        if table_references.is_empty() {
                            if let RFileDecoded::DB(table) = file_decoded {
                                table_references = dependencies.db_reference_data(pack, table.table_name(), table.definition());
                            }
                        }

                        Self::check_db(
                            file,
                            dependencies,
                            &self.diagnostics_ignored,
                            &ignored_fields,
                            &ignored_diagnostics,
                            &ignored_diagnostics_for_fields,
                            game_info,
                            schema,
                            &local_file_path_list,
                            &local_folder_path_list,
                            &table_references,
                            check_ak_only_refs,
                        )
                    },
                    FileType::Loc => Self::check_loc(file, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields),
                    FileType::PortraitSettings => PortraitSettingsDiagnostic::check(file, &art_set_ids, &variant_filenames, dependencies, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields),
                    _ => None,
                };

                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(diagnostic);
                }
            }

            Some(diagnostics)
        }).flatten().collect());

        if let Some(diagnostics) = Self::check_dependency_manager(pack) {
            self.results_mut().push(diagnostics);
        }

        if let Some(diagnostics) = Self::check_pack(pack) {
            self.results_mut().push(diagnostics);
        }

        self.results_mut().sort_by(|a, b| {
            if !a.path().is_empty() && !b.path().is_empty() {
                a.path().cmp(b.path())
            } else if a.path().is_empty() && !b.path().is_empty() {
                Ordering::Greater
            } else if !a.path().is_empty() && b.path().is_empty() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }

    /// This function takes care of checking the db tables of your mod for errors.
    fn check_db(
        file: &RFile,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        game_info: &GameInfo,
        schema: &Schema,
        local_path_list: &HashSet<&str>,
        local_folder_list: &HashSet<String>,
        dependency_data: &HashMap<i32, TableReferences>,
        check_ak_only_refs: bool,
    ) ->Option<DiagnosticType> {
        if let Ok(RFileDecoded::DB(table)) = file.decoded() {
            let mut diagnostic = TableDiagnostic::new(file.path_in_container_raw());

            // Before anything else, check if the table is outdated.
            if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("OutdatedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && Self::is_table_outdated(table.table_name(), *table.definition().version(), dependencies) {
                let result = TableDiagnosticReport::new(TableDiagnosticReportType::OutdatedTable, &[], &[]);
                diagnostic.results_mut().push(result);
            }

            // Check if it's one of the banned tables for the game selected.
            if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("BannedTable"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && game_info.is_file_banned(file.path_in_container_raw()) {
                let result = TableDiagnosticReport::new(TableDiagnosticReportType::BannedTable, &[], &[]);
                diagnostic.results_mut().push(result);
            }

            // Check if the table name has a number at the end, which causes very annoying bugs.
            if let Some(name) = file.file_name() {
                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameEndsInNumber"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && (name.ends_with('0') ||
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

                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableNameHasSpace"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && name.contains(' ') {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::TableNameHasSpace, &[], &[]);
                    diagnostic.results_mut().push(result);
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("TableIsDataCoring"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
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
            let schema_patches = schema.patches_for_table(table.table_name());

            for (row, cells) in table_data.iter().enumerate() {
                let mut row_is_empty = true;
                let mut row_keys_are_empty = true;
                let mut row_keys: BTreeMap<i32, String> = BTreeMap::new();
                for (column, field) in fields_processed.iter().enumerate() {
                    let cell_data = cells[column].data_to_string();

                    // Path checks.
                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("FieldWithPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !cell_data.is_empty() && fields_processed[column].is_filename() {
                        let mut path_found = false;
                        let paths = {
                            let path = if let Some(relative_path) = fields_processed[column].filename_relative_path() {
                                relative_path.replace('%', &cell_data)
                            } else {
                                cell_data.to_string()
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
                            if local_path_list.par_iter().any(|path_2| caseless::canonical_caseless_match_str(path_2, path)) {
                                path_found = true;
                            }

                            if !path_found && local_folder_list.par_iter().any(|path_2| caseless::canonical_caseless_match_str(path_2, path)) {
                                path_found = true;
                            }

                            if !path_found && dependencies.file_exists(path, true, true, true) {
                                path_found = true;
                            }

                            if !path_found && dependencies.folder_exists(path, true, true, true) {
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
                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), None, ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && field.is_reference().is_some() {
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
                                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("InvalidReference"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && is_valid_reference {
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

                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && field.is_key(patches) && key_amount == 1 && *field.field_type() != FieldType::OptionalStringU8 && *field.field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField(field.name().to_string()), &[(row as i32, column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("ValueCannotBeEmpty"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && cell_data.is_empty() && field.cannot_be_empty(schema_patches) {
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::ValueCannotBeEmpty(field.name().to_string()), &[(row as i32, column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }

                    if field.is_key(patches) {
                        row_keys.insert(column as i32, cell_data.to_string());
                    }
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && row_is_empty {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &fields_processed);
                    diagnostic.results_mut().push(result);
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("EmptyKeyFields"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && row_keys_are_empty {
                    let cells_affected = row_keys.keys().map(|column| (row as i32, *column)).collect::<Vec<(i32, i32)>>();
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyFields, &cells_affected, &fields_processed);
                    diagnostic.results_mut().push(result);
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("DuplicatedCombinedKeys"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

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
            if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                for column in &columns_without_reference_table {
                    let field_name = fields_processed[*column].name().to_string();
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableFound(field_name), &[(-1, *column as i32)], &fields_processed);
                    diagnostic.results_mut().push(result);
                }
            }
            for column in &columns_with_reference_table_and_no_column {
                if !dependencies.is_asskit_data_loaded() {
                    if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundNoPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                        let field_name = fields_processed[*column].name().to_string();
                        let result = TableDiagnosticReport::new(TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(field_name), &[(-1, *column as i32)], &fields_processed);
                        diagnostic.results_mut().push(result);
                    }
                }
                else if !Self::ignore_diagnostic(global_ignored_diagnostics, None, Some("NoReferenceTableNorColumnFoundPak"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
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
    /*
    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_anim_fragment(
        file: &RFile,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
        local_path_list: &HashSet<&str>,
        local_folder_list: &HashSet<String>,
    ) ->Option<DiagnosticType> {
        if let Ok(RFileDecoded::AnimFragment(table)) = file.decoded() {
            let mut diagnostic = AnimFragmentDiagnostic::new(file.path_in_container_raw());

            // Check inside the of the table in the column [3].
            let fields_processed = table.definition().fields_processed();
            for (row, cells) in table.table().data(&None).unwrap().iter().enumerate() {
                for (column, field) in fields_processed.iter().enumerate() {
                    let cell_data = cells[column].data_to_string();

                    if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field.name()), Some("FieldWithPathNotFound"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !cell_data.is_empty() && fields_processed[column].is_filename() {
                        let mut path_found = false;
                        let mut path = cell_data.replace('\\', "/");

                        // If it's a folder, remove the trailing /.
                        if path.ends_with('/') {
                            path.pop();
                        }

                        if local_path_list.par_iter().any(|path_2| caseless::canonical_caseless_match_str(path_2, &path)) {
                            path_found = true;
                        }

                        if !path_found && local_folder_list.par_iter().any(|path_2| caseless::canonical_caseless_match_str(path_2, &path)) {
                            path_found = true;
                        }

                        if !path_found && dependencies.file_exists(&path, true, true, true) {
                            path_found = true;
                        }

                        if !path_found && dependencies.folder_exists(&path, true, true, true) {
                            path_found = true;
                        }

                        if path_found {
                            continue;
                        }

                        if !path_found {
                            let result = AnimFragmentDiagnosticReport::new(AnimFragmentDiagnosticReportType::FieldWithPathNotFound(vec![path]), &[(row as i32, column as i32)]);
                            diagnostic.results_mut().push(result);
                        }
                    }
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::AnimFragment(diagnostic))
            } else { None }
        } else { None }
    }*/

    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_loc(
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
                let key = if let DecodedData::StringU16(ref data) = cells[0] { data } else { unimplemented!() };
                let data = if let DecodedData::StringU16(ref data) = cells[1] { data } else { unimplemented!() };

                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("InvalidLocKey"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !key.is_empty() && (key.contains('\n') || key.contains('\t')) {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidLocKey, &[(row as i32, 0)], &fields);
                    diagnostic.results_mut().push(result);
                }

                // Only in case none of the two columns are ignored, we perform these checks.
                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("EmptyRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && key.is_empty() && data.is_empty() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyRow, &[(row as i32, -1)], &fields);
                    diagnostic.results_mut().push(result);
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("EmptyKeyField"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && key.is_empty() && !data.is_empty() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::EmptyKeyField("Key".to_string()), &[(row as i32, 0)], &fields);
                    diagnostic.results_mut().push(result);
                }

                // Magic Regex. It works. Don't ask why.
                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_text_name), Some("InvalidEscape"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) && !data.is_empty() && REGEX_INVALID_ESCAPES.is_match(data).unwrap() {
                    let result = TableDiagnosticReport::new(TableDiagnosticReportType::InvalidEscape, &[(row as i32, 1)], &fields);
                    diagnostic.results_mut().push(result);
                }

                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("DuplicatedRow"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {
                    let mut row_keys: BTreeMap<i32, String> = BTreeMap::new();
                    row_keys.insert(0, key.to_owned());
                    row_keys.insert(1, data.to_owned());

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

                if !Self::ignore_diagnostic(global_ignored_diagnostics, Some(field_key_name), Some("DuplicatedCombinedKeys"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) {

                    // If this returns something, it means there is a duplicate.
                    let combined_keys = key.to_owned();
                    if let Some(old_position) = keys.insert(combined_keys.to_owned(), vec![(row as i32, 0)]) {
                        if let Some(old_pos) = old_position.first() {

                            // Mark previous row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&old_pos.0) {
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys.to_string()), &old_position, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(old_pos.0);
                            }

                            // Mark current row, if not yet marked.
                            if !duplicated_rows_already_marked.contains(&(row as i32)) {
                                let cells_affected = vec![(row as i32, 0)];
                                let result = TableDiagnosticReport::new(TableDiagnosticReportType::DuplicatedCombinedKeys(combined_keys), &cells_affected, &fields);
                                diagnostic.results_mut().push(result);
                                duplicated_combined_keys_already_marked.push(row as i32);
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

    /// This function takes care of checking for PackFile-Related for errors.
    fn check_pack(pack: &Pack) -> Option<DiagnosticType> {
        let mut diagnostic = PackDiagnostic::default();

        let name = pack.disk_file_name();
        if name.contains(' ') {
            let result = PackDiagnosticReport::new(PackDiagnosticReportType::InvalidPackName(name));
            diagnostic.results_mut().push(result);
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Pack(diagnostic))
        } else { None }
    }

    /// This function takes care of checking for errors in the Dependency Manager.
    fn check_dependency_manager(pack: &Pack) ->Option<DiagnosticType> {
        let mut diagnostic = DependencyDiagnostic::default();
        for (index, pack) in pack.dependencies().iter().enumerate() {

            // TODO: Make it so this also checks if the PackFile actually exists,
            if pack.is_empty() || !pack.ends_with(".pack") || pack.contains(' ') {
                let result = DependencyDiagnosticReport::new(DependencyDiagnosticReportType::InvalidDependencyPackName(pack.to_string()), &[(index as i32, 0)]);
                diagnostic.results_mut().push(result);
            }
        }

        if !diagnostic.results().is_empty() {
            Some(DiagnosticType::Dependency(diagnostic))
        } else { None }
    }

    /// This function takes care of checking RPFM's configuration for errors.
    fn check_config(dependencies: &Dependencies, game_info: &GameInfo, game_path: &Path) -> Option<DiagnosticType> {
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

    /// Function to know if an specific field/diagnostic must be ignored.
    fn ignore_diagnostic(global_ignored_diagnostics: &[String], field_name: Option<&str>, diagnostic: Option<&str>, ignored_fields: &[String], ignored_diagnostics: &HashSet<String>, ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>) -> bool {
        let mut ignore_diagnostic = false;

        if let Some(diagnostic) = diagnostic {
            return global_ignored_diagnostics.iter().any(|x| x == diagnostic);
        }

        // If we have a field, and it's in the ignored list, ignore it.
        if let Some(field_name) = field_name {
            ignore_diagnostic = ignored_fields.iter().any(|x| x == field_name);
        }

        // If we have a diagnostic, and it's in the ignored list, ignore it.
        else if let Some(diagnostic) = diagnostic {
            ignore_diagnostic = ignored_diagnostics.get(diagnostic).is_some();
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

    /// Ignore entire tables if their path starts with the one we have (so we can do mass ignores) and we didn't specified a field to ignore.
    fn ignore_data_for_file(file: &RFile, files_to_ignore: &Option<Vec<(String, Vec<String>, Vec<String>)>>) -> Option<(Vec<String>, HashSet<String>, HashMap<String, Vec<String>>)> {
        let mut ignored_fields = vec![];
        let mut ignored_diagnostics = HashSet::new();
        let mut ignored_diagnostics_for_fields: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(ref files_to_ignore) = files_to_ignore {
            for (path_to_ignore, fields, diags_to_ignore) in files_to_ignore {

                // If the rule doesn't affect this PackedFile, ignore it.
                if !path_to_ignore.is_empty() && file.path_in_container_raw().starts_with(path_to_ignore) {

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
                        ignored_diagnostics.extend(diags_to_ignore.to_vec());
                    }
                }
            }
        }
        Some((ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields))
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

    /// This function converts an entire diagnostics struct into a JSon string.
    pub fn json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(From::from)
    }
}

impl Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::AnimFragment(_) => "AnimFragment",
            Self::Config(_) => "Config",
            Self::DB(_) => "DB",
            Self::Loc(_) => "Loc",
            Self::Pack(_) => "Packfile",
            Self::PortraitSettings(_) => "PortraitSettings",
            Self::Dependency(_) => "DependencyManager",
        }, f)
    }
}
