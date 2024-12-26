//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code related to the `Diagnostics`.
//!
//! This module contains the code needed to get a `Diagnostics` over an entire `PackFile`.
//!
//! Notes on cells_affected:
//! - Both -1: affects the entire table.
//! - Row -1: affects all rows in single column.
//! - Column -1: affects all columns in single row.

use getset::{Getters, MutGetters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::cmp::Ordering;
use std::{fmt, fmt::Display};
use std::path::Path;

use rpfm_lib::error::Result;
use rpfm_lib::files::{ContainerPath, Container, DecodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded};
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::{FieldType, Schema};

use crate::dependencies::{Dependencies, TableReferences};
use crate::REGEX_INVALID_ESCAPES;

use self::anim_fragment_battle::*;
use self::config::*;
use self::dependency::*;
use self::pack::*;
use self::portrait_settings::*;
use self::table::*;
use self::text::TextDiagnostic;

pub mod anim_fragment_battle;
pub mod config;
pub mod dependency;
pub mod pack;
pub mod portrait_settings;
pub mod table;
pub mod text;

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
    AnimFragmentBattle(AnimFragmentBattleDiagnostic),
    Config(ConfigDiagnostic),
    Dependency(DependencyDiagnostic),
    DB(TableDiagnostic),
    Loc(TableDiagnostic),
    Pack(PackDiagnostic),
    PortraitSettings(PortraitSettingsDiagnostic),
    Text(TextDiagnostic),
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
            Self::AnimFragmentBattle(ref diag) => diag.path(),
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.path(),
            Self::Pack(_) => "",
            Self::PortraitSettings(diag) => diag.path(),
            Self::Text(diag) => diag.path(),
            Self::Dependency(diag) => diag.path(),
            Self::Config(_) => "",
        }
    }
}

impl Diagnostics {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn check(&mut self, pack: &mut Pack, dependencies: &mut Dependencies, schema: &Schema, game_info: &GameInfo, game_path: &Path, paths_to_check: &[ContainerPath], check_ak_only_refs: bool) {

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
        if let Some(diagnostics) = ConfigDiagnostic::check(dependencies, game_info, game_path) {
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

        // To make sure we can read any non-db and non-loc file, we need to pre-decode them here.
        {
            // Extra data to decode animfragmentbattle files.
            let mut extra_data = DecodeableExtraData::default();
            extra_data.set_game_key(Some(game_info.key()));
            let extra_data = Some(extra_data);

            pack.files_by_type_mut(&[FileType::AnimFragmentBattle, FileType::Text, FileType::PortraitSettings])
                .par_iter_mut()
                .for_each(|file| { let _ = file.decode(&extra_data, true, false); });
        }

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let files = if paths_to_check.is_empty() {
            pack.files_by_type(&[FileType::AnimFragmentBattle, FileType::DB, FileType::Loc, FileType::Text, FileType::PortraitSettings])
        } else {
            pack.files_by_type_and_paths(&[FileType::AnimFragmentBattle, FileType::DB, FileType::Loc, FileType::Text, FileType::PortraitSettings], paths_to_check, false)
        };

        let mut files_split: HashMap<&str, Vec<&RFile>> = HashMap::new();
        let mut we_need_loc_data = false;
        for file in &files {
            match file.file_type() {
                FileType::AnimFragmentBattle => {
                    if let Some(table_set) = files_split.get_mut("anim_fragment_battle") {
                        table_set.push(file);
                    } else {
                        files_split.insert("anim_fragment_battle", vec![file]);
                    }
                },
                FileType::DB => {
                    we_need_loc_data = true;

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
                FileType::Text => {
                    if let Some(name) = file.file_name() {
                        if name.ends_with(".lua") {
                            if let Some(table_set) = files_split.get_mut("lua") {
                                table_set.push(file);
                            } else {
                                files_split.insert("lua", vec![file]);
                            }
                        }
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
        let local_file_path_list = pack.paths_cache();

        let loc_files = pack.files_by_type(&[FileType::Loc]);
        let loc_decoded = loc_files.iter()
            .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
            .map(|file| file.data())
            .collect::<Vec<_>>();

        // Loc data takes a few ms to get, and it's only needed if we're going to check on tables, for the lookup data. So only get it if we really need it.
        let loc_data = if we_need_loc_data {
            Some(loc_decoded.par_iter()
            .flat_map(|data| data.par_iter()
                .map(|entry| (entry[0].data_to_string(), entry[1].data_to_string()))
                .collect::<Vec<(_,_)>>()
            ).collect::<HashMap<_,_>>())
        } else {
            None
        };

        // That way we can get it fast on the first try, and skip.
        let table_names = files_split.iter().filter(|(key, _)| **key != "anim_fragment_battle" && **key != "locs" && **key != "lua" && **key != "portrait_settings").map(|(key, _)| key.to_string()).collect::<Vec<_>>();

        // If table names is empty this triggers a full regeneration, which is slow as fuck. So make sure to avoid that if we're only doing a partial check.
        if !table_names.is_empty() || (table_names.is_empty() && paths_to_check.is_empty()) {
            dependencies.generate_local_db_references(schema, pack, &table_names);
        }

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
                    FileType::AnimFragmentBattle => AnimFragmentBattleDiagnostic::check(
                        file,
                        dependencies,
                        &self.diagnostics_ignored,
                        &ignored_fields,
                        &ignored_diagnostics,
                        &ignored_diagnostics_for_fields,
                        local_file_path_list,
                    ),
                    FileType::DB => {

                        // Get the dependency data for tables once per batch.
                        // That way we can speed up this a lot.
                        let file_decoded = file.decoded().ok()?;
                        if table_references.is_empty() {
                            if let RFileDecoded::DB(table) = file_decoded {
                                table_references = dependencies.db_reference_data(schema, pack, table.table_name(), table.definition(), &loc_data);
                            }
                        }

                        TableDiagnostic::check_db(
                            file,
                            dependencies,
                            &self.diagnostics_ignored,
                            &ignored_fields,
                            &ignored_diagnostics,
                            &ignored_diagnostics_for_fields,
                            game_info,
                            local_file_path_list,
                            &table_references,
                            check_ak_only_refs,
                        )
                    },
                    FileType::Loc => TableDiagnostic::check_loc(file, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields),
                    FileType::Text => TextDiagnostic::check(file, pack, dependencies, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields),
                    FileType::PortraitSettings => PortraitSettingsDiagnostic::check(file, &art_set_ids, &variant_filenames, dependencies, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, local_file_path_list),
                    _ => None,
                };

                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(diagnostic);
                }
            }

            Some(diagnostics)
        }).flatten().collect());

        if let Some(diagnostics) = DependencyDiagnostic::check(pack) {
            self.results_mut().push(diagnostics);
        }

        if let Some(diagnostics) = PackDiagnostic::check(pack) {
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

    /// This function converts an entire diagnostics struct into a JSon string.
    pub fn json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(From::from)
    }
}

impl Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::AnimFragmentBattle(_) => "AnimFragmentBattle",
            Self::Config(_) => "Config",
            Self::DB(_) => "DB",
            Self::Loc(_) => "Loc",
            Self::Pack(_) => "Packfile",
            Self::PortraitSettings(_) => "PortraitSettings",
            Self::Text(_) => "Text",
            Self::Dependency(_) => "DependencyManager",
        }, f)
    }
}
