//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Diagnostics and Lua script operations.

use anyhow::{anyhow, Result};

use rpfm_extensions::diagnostics::{DiagnosticLevel as CheckLevel, DiagnosticReport, Diagnostics, DiagnosticType};
use rpfm_extensions::lua::check::{check_script, LuaDefinitions};
use rpfm_extensions::lua::harness::{run_tests, LuaScripts, LuaTestOptions, LuaTestReport};

use rpfm_ipc::api::{ApiError, Done};
use rpfm_ipc::api::diagnostics::{DEFAULT_DIAGNOSTICS_LIMIT, DiagnosticLevel, DiagnosticList, DiagnosticResult, DiagnosticsSummary, IgnoreDiagnostics, ListDiagnostics, RunDiagnostics};

use rpfm_lib::files::{Container, ContainerPath};

use rpfm_telemetry::info;

use rpfm_ipc::settings::Settings;

use super::{SessionState, cached_lua_api};

/// Results of a diagnostics check.
pub(super) struct DiagnosticsResults {

    /// The results as the check returns them, to check only some paths again later.
    diagnostics: Diagnostics,

    /// The results flattened, one per problem, for listing.
    results: Vec<DiagnosticResult>,
}

/// A hover of a Lua script: its start line and column, end line and column (all 0-based), and its docs as rich text.
pub type LuaHover = (u64, u64, u64, u64, String);

impl SessionState {

    /// Checks the open packs for problems.
    ///
    /// Does nothing if there is no schema.
    ///
    /// # Arguments
    ///
    /// * `diagnostics` - Results of a previous check, or a new one with the diagnostics to ignore.
    /// * `paths_to_check` - Paths to recheck, keeping the previous results for the rest. If empty, everything is checked.
    /// * `check_ak_only_refs` - If references to tables only in the Assembly Kit are checked.
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    ///
    /// # Returns
    ///
    /// The updated diagnostics.
    pub fn check_diagnostics(&mut self, mut diagnostics: Diagnostics, paths_to_check: &[ContainerPath], check_ak_only_refs: bool, settings: &Settings) -> Diagnostics {
        if let Some(ref schema) = self.schema {
            let game_path = settings.path_buf(self.game.key());
            let lua_api = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies);
            diagnostics.check(&mut self.packs, &mut self.dependencies, schema, &self.game, &game_path, paths_to_check, check_ak_only_refs, lua_api);
        }

        info!("Checking diagnostics: done.");
        diagnostics
    }

    /// Checks the open packs for problems, keeping the results for [`Self::list_diagnostics`].
    ///
    /// # Arguments
    ///
    /// * `request` - What to check.
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    ///
    /// # Returns
    ///
    /// A summary of the results.
    pub fn run_diagnostics(&mut self, request: &RunDiagnostics, settings: &Settings) -> DiagnosticsSummary {
        let paths = request.paths.iter()
            .map(|path| if self.packs.values().any(|pack| pack.has_file(path)) {
                ContainerPath::File(path.to_owned())
            } else {
                ContainerPath::Folder(path.trim_end_matches('/').to_owned())
            })
            .collect::<Vec<_>>();

        let mut diagnostics = match self.diagnostics.take() {
            Some(previous) if !paths.is_empty() => previous.diagnostics,
            _ => Diagnostics::default(),
        };

        diagnostics.diagnostics_ignored_mut().clone_from(&request.ignored_types);
        let diagnostics = self.check_diagnostics(diagnostics, &paths, request.check_assembly_kit_only_references, settings);
        let results = flatten_diagnostics(&diagnostics);
        let summary = summarize(&results);

        self.diagnostics = Some(DiagnosticsResults { diagnostics, results });
        summary
    }

    /// Returns a page of the results of the last diagnostics check matching the request.
    ///
    /// # Errors
    ///
    /// Fails if there are no results yet.
    pub fn list_diagnostics(&self, request: &ListDiagnostics) -> Result<DiagnosticList> {
        let results = &self.diagnostics.as_ref().ok_or(ApiError::DiagnosticsNotRun)?.results;
        let matching = results.iter()
            .filter(|result| request.levels.as_ref().is_none_or(|levels| levels.contains(&result.level)))
            .filter(|result| request.report_types.as_ref().is_none_or(|types| types.contains(&result.report_type)))
            .filter(|result| request.pack.as_ref().is_none_or(|pack| *pack == result.pack))
            .filter(|result| result.path.starts_with(&request.path_prefix));

        let mut page = vec![];
        let mut total = 0;
        let limit = request.limit.unwrap_or(DEFAULT_DIAGNOSTICS_LIMIT);
        for result in matching {
            if total >= request.offset && page.len() < limit {
                page.push(result.clone());
            }

            total += 1;
        }

        Ok(DiagnosticList { results: page, total })
    }

    /// Makes the next diagnostics checks of a pack skip some results, saving the rule in the pack's settings.
    ///
    /// # Errors
    ///
    /// Fails if the pack isn't open, or if any value has characters the rule format uses as separators.
    pub fn ignore_diagnostics(&mut self, request: &IgnoreDiagnostics) -> Result<Done> {
        let values = std::iter::once(&request.path).chain(&request.columns).chain(&request.report_types);
        if let Some(value) = values.clone().find(|value| value.contains([';', '\n'])) {
            return Err(ApiError::InvalidParams(format!("Values can't contain ';' or line breaks: {value}")).into());
        }

        if let Some(value) = request.columns.iter().chain(&request.report_types).find(|value| value.contains(',')) {
            return Err(ApiError::InvalidParams(format!("Columns and report types can't contain ',': {value}")).into());
        }

        let line = format!("\n{};{};{}", request.path, request.columns.join(","), request.report_types.join(","));
        self.add_pack_ignored_diagnostics_line(&request.pack, line)?;
        Ok(Done {})
    }

    /// Returns the hovers of a Lua script, with the docs of the game's Lua API. Empty if the API is not available.
    pub fn lua_hovers(&mut self, source: &str, settings: &Settings) -> Vec<LuaHover> {
        let Some(lua_api) = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies) else { return vec![] };

        check_script(source, Some(lua_api), &LuaDefinitions::default()).hovers().iter()
            .filter_map(|hover| {
                let ((start_line, start_column), (end_line, end_column)) = *hover.range();
                lua_api.hover_html(hover.target()).map(|html| (start_line, start_column, end_line, end_column, html))
            })
            .collect()
    }

    /// Runs Lua tests against the scripts of the game and the open packs, outside of the game.
    ///
    /// # Arguments
    ///
    /// * `test_source` - Code of the Lua test file.
    /// * `campaign` - Campaign whose vanilla scripts to load. If `None`, only the script libraries and the mods are loaded.
    /// * `settings` - Settings, to find the game's Assembly Kit.
    ///
    /// # Returns
    ///
    /// The results of the tests.
    ///
    /// # Errors
    ///
    /// Fails if the game's Lua API is not available, or if the tests can't be loaded.
    pub fn lua_run_tests(&mut self, test_source: &str, campaign: Option<String>, settings: &Settings) -> Result<LuaTestReport> {
        let lua_api = cached_lua_api(&mut self.lua_api_cache, &self.game, settings, &self.dependencies)
            .ok_or_else(|| anyhow!("The Lua API of the game is not available. Lua tests need the game's Assembly Kit to be installed."))?;

        let scripts = LuaScripts::from_game_and_packs(&self.dependencies, &self.packs);
        let key_values = |table_name| self.dependencies.db_key_values(Some(&self.packs), table_name)
            .map(|(_, keys)| keys.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut options = LuaTestOptions::default();
        options.set_campaign(campaign);
        options.set_faction_keys(key_values("factions_tables"));
        options.set_region_keys(key_values("regions_tables"));

        Ok(run_tests(lua_api, &scripts, test_source, &options)?)
    }
}

/// Flattens the results of a check into one result per problem.
fn flatten_diagnostics(diagnostics: &Diagnostics) -> Vec<DiagnosticResult> {
    diagnostics.results().iter()
        .flat_map(|diagnostic| match diagnostic {
            DiagnosticType::DB(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "db", diagnostic.results(), |report| report.report_type().to_string(), |report| (cells(report.cells_affected()), report.column_names().clone())),
            DiagnosticType::Loc(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "loc", diagnostic.results(), |report| report.report_type().to_string(), |report| (cells(report.cells_affected()), report.column_names().clone())),
            DiagnosticType::AnimFragmentBattle(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "anim_fragment_battle", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
            DiagnosticType::Config(diagnostic) => results_of("", "", "config", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
            DiagnosticType::Dependency(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "dependency", diagnostic.results(), |report| report.report_type().to_string(), |report| (cells(report.cells_affected()), vec![])),
            DiagnosticType::GroupFormations(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "group_formations", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
            DiagnosticType::Pack(diagnostic) => results_of(diagnostic.pack(), "", "pack", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
            DiagnosticType::PortraitSettings(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "portrait_settings", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
            DiagnosticType::Text(diagnostic) => results_of(diagnostic.pack(), diagnostic.path(), "text", diagnostic.results(), |report| report.report_type().to_string(), no_cells),
        })
        .collect()
}

/// Builds the results of the reports of a checked file.
///
/// # Arguments
///
/// * `pack` - Key of the pack of the file.
/// * `path` - Path of the file.
/// * `checked` - What was checked, like `db` or `pack`.
/// * `reports` - Problems found.
/// * `report_type` - Returns the type of a problem.
/// * `cells` - Returns the table cells and columns of a problem.
fn results_of<R: DiagnosticReport>(pack: &str, path: &str, checked: &str, reports: &[R], report_type: impl Fn(&R) -> String, cells: impl Fn(&R) -> (Vec<[i32; 2]>, Vec<String>)) -> Vec<DiagnosticResult> {
    reports.iter()
        .map(|report| {
            let (cells, columns) = cells(report);
            DiagnosticResult {
                pack: pack.to_owned(),
                path: path.to_owned(),
                checked: checked.to_owned(),
                report_type: report_type(report),
                level: match report.level() {
                    CheckLevel::Info => DiagnosticLevel::Info,
                    CheckLevel::Warning => DiagnosticLevel::Warning,
                    CheckLevel::Error => DiagnosticLevel::Error,
                },
                message: report.message(),
                cells,
                columns,
            }
        })
        .collect()
}

/// Returns no cells nor columns, for reports that aren't about table cells.
fn no_cells<R>(_: &R) -> (Vec<[i32; 2]>, Vec<String>) {
    (vec![], vec![])
}

/// Converts `(row, column)` cells to `[row, column]` arrays.
fn cells(cells: &[(i32, i32)]) -> Vec<[i32; 2]> {
    cells.iter().map(|(row, column)| [*row, *column]).collect()
}

/// Counts results by level and by type.
fn summarize(results: &[DiagnosticResult]) -> DiagnosticsSummary {
    let mut summary = DiagnosticsSummary { total: results.len(), ..DiagnosticsSummary::default() };
    for result in results {
        *summary.by_level.entry(result.level).or_default() += 1;
        *summary.by_type.entry(result.report_type.clone()).or_default() += 1;
    }

    summary
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn result(level: DiagnosticLevel, report_type: &str) -> DiagnosticResult {
        DiagnosticResult {
            pack: "my_mod.pack".to_owned(),
            path: "db/units_tables/my_mod".to_owned(),
            checked: "db".to_owned(),
            report_type: report_type.to_owned(),
            level,
            message: String::new(),
            cells: vec![],
            columns: vec![],
        }
    }

    #[test]
    fn summary_counts_by_level_and_type() {
        let results = vec![
            result(DiagnosticLevel::Error, "InvalidReference"),
            result(DiagnosticLevel::Error, "InvalidReference"),
            result(DiagnosticLevel::Warning, "OutdatedTable"),
        ];

        let summary = summarize(&results);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_level, BTreeMap::from([(DiagnosticLevel::Error, 2), (DiagnosticLevel::Warning, 1)]));
        assert_eq!(summary.by_type, BTreeMap::from([("InvalidReference".to_owned(), 2), ("OutdatedTable".to_owned(), 1)]));
    }
}
