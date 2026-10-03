//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods to check the open packs for problems.
//!
//! The server keeps the results of the last check, so clients get a summary when running it,
//! and read the results they need with [`ListDiagnostics`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use rpfm_extensions::diagnostics::Diagnostics;

use super::{Done, Request};

/// Default amount of results returned by [`ListDiagnostics`].
pub const DEFAULT_DIAGNOSTICS_LIMIT: usize = 100;

/// `diagnostics.run`: checks the open packs for problems, keeping the results for `diagnostics.list`. Runs as a job.
///
/// A running check stops when other requests arrive, and is queued again behind them. A new check replaces the
/// one that hasn't ended yet, which ends cancelled, and also checks the paths the replaced one would have checked.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunDiagnostics {

    /// If set, only files and folders of the open packs at these paths are checked again, keeping the
    /// results of the last check for the rest. If not set, or if there are no previous results, everything is checked.
    #[serde(default)]
    pub paths: Vec<String>,

    /// Report types to skip, like `OutdatedTable` or `InvalidReference`.
    #[serde(default)]
    pub ignored_types: Vec<String>,

    /// If references to tables only in the Assembly Kit are checked too.
    #[serde(default)]
    pub check_assembly_kit_only_references: bool,
}

/// Summary of the results of a check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticsSummary {

    /// Amount of results.
    pub total: usize,

    /// Amount of results of each level.
    pub by_level: BTreeMap<DiagnosticLevel, usize>,

    /// Amount of results of each report type.
    pub by_type: BTreeMap<String, usize>,
}

/// Severity of a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {

    /// A suggestion, or information about the mod.
    Info,

    /// Something that may be a mistake, or cause problems in some situations.
    Warning,

    /// Something that will likely cause problems, like invalid references or malformed data.
    Error,
}

/// `diagnostics.list`: returns results of the last check, optionally filtered.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListDiagnostics {

    /// If set, only results of these levels are returned.
    #[serde(default)]
    pub levels: Option<Vec<DiagnosticLevel>>,

    /// If set, only results of these report types are returned.
    #[serde(default)]
    pub report_types: Option<Vec<String>>,

    /// If set, only results of this pack are returned.
    #[serde(default)]
    pub pack: Option<String>,

    /// Only results of files whose path starts with this are returned.
    #[serde(default)]
    pub path_prefix: String,

    /// Amount of matching results to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of results to return. Defaults to 100.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A page of the results of a check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticList {

    /// The results in the page.
    pub results: Vec<DiagnosticResult>,

    /// Amount of results matching the request, in all pages.
    pub total: usize,
}

/// A problem found in a check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticResult {

    /// Key of the pack with the problem. Empty for problems of the configuration.
    pub pack: String,

    /// Path of the file with the problem. Empty for problems of a whole pack or of the configuration.
    pub path: String,

    /// What was checked: `db`, `loc`, `text`, `pack`, `config`, `dependency`, `portrait_settings`,
    /// `group_formations` or `anim_fragment_battle`.
    pub checked: String,

    /// Type of the problem, like `InvalidReference`. Used to ignore it.
    pub report_type: String,

    /// Severity of the problem.
    pub level: DiagnosticLevel,

    /// Description of the problem.
    pub message: String,

    /// Table cells with the problem, as `[row, column]`. -1 means the whole row or column.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<[i32; 2]>,

    /// Names of the columns of the cells with the problem.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
}

/// `diagnostics.ignore`: makes the next checks of a pack skip some results. Saved in the pack's settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IgnoreDiagnostics {

    /// Key of the pack.
    pub pack: String,

    /// Files whose path starts with this are affected.
    pub path: String,

    /// If set, only these columns of the files are affected.
    #[serde(default)]
    pub columns: Vec<String>,

    /// If set, only these report types are skipped. Otherwise, everything is skipped.
    #[serde(default)]
    pub report_types: Vec<String>,
}

impl Request for RunDiagnostics {
    const METHOD: &'static str = "diagnostics.run";
    type Response = DiagnosticsSummary;
    const IS_JOB: bool = true;
}

impl Request for ListDiagnostics {
    const METHOD: &'static str = "diagnostics.list";
    type Response = DiagnosticList;
}

/// `diagnostics.report`: returns the results of the last diagnostics check, as the check returns them.
///
/// Meant for clients showing the results with all their details. Others should use `diagnostics.list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetDiagnosticsReport {}

impl Request for GetDiagnosticsReport {
    const METHOD: &'static str = "diagnostics.report";
    type Response = Diagnostics;
}

impl Request for IgnoreDiagnostics {
    const METHOD: &'static str = "diagnostics.ignore";
    type Response = Done;
}
