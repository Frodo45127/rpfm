//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `GroupFormations` diagnostics.
//!
//! The checks themselves live in `rpfm_lib`, as the GroupFormations editor uses them too. This module
//! turns their results into diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_lib::files::{RFile, RFileDecoded};
use rpfm_lib::files::group_formations::validation::ValidationIssue;

use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a GroupFormations diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct GroupFormationsDiagnostic {
    path: String,
    pack: String,
    results: Vec<GroupFormationsDiagnosticReport>
}

/// This struct defines an individual GroupFormations diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct GroupFormationsDiagnosticReport {

    /// Index of the formation with the issue, in the file.
    formation_index: usize,

    /// Name of the formation with the issue.
    formation_name: String,

    report_type: GroupFormationsDiagnosticReportType,
}

/// Kinds of GroupFormations issues. They match the issues found by the file's validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupFormationsDiagnosticReportType {
    DuplicateFormationName,
    NoAbsoluteBlock,
    DuplicateBlockId(u32),
    MissingReference(u32, u32),
    ForwardReference(u32, u32),
    ReferenceCycle(Vec<u32>),
    EmptySpan(u32),
    InvalidThresholds(u32),
    NoEntityPreferences(u32),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GroupFormationsDiagnosticReport {
    pub fn new(formation_index: usize, formation_name: &str, report_type: GroupFormationsDiagnosticReportType) -> Self {
        Self {
            formation_index,
            formation_name: formation_name.to_owned(),
            report_type,
        }
    }
}

impl GroupFormationsDiagnosticReportType {

    /// Returns the block the issue points to, if it points to a specific one.
    pub fn block_id(&self) -> Option<u32> {
        match self {
            Self::DuplicateFormationName |
            Self::NoAbsoluteBlock => None,
            Self::DuplicateBlockId(block_id) |
            Self::MissingReference(block_id, _) |
            Self::ForwardReference(block_id, _) |
            Self::EmptySpan(block_id) |
            Self::InvalidThresholds(block_id) |
            Self::NoEntityPreferences(block_id) => Some(*block_id),
            Self::ReferenceCycle(block_ids) => block_ids.first().copied(),
        }
    }
}

impl From<ValidationIssue> for GroupFormationsDiagnosticReportType {
    fn from(issue: ValidationIssue) -> Self {
        match issue {
            ValidationIssue::DuplicateFormationName => Self::DuplicateFormationName,
            ValidationIssue::NoAbsoluteBlock => Self::NoAbsoluteBlock,
            ValidationIssue::DuplicateBlockId(block_id) => Self::DuplicateBlockId(block_id),
            ValidationIssue::MissingReference { block_id, referenced_id } => Self::MissingReference(block_id, referenced_id),
            ValidationIssue::ForwardReference { block_id, referenced_id } => Self::ForwardReference(block_id, referenced_id),
            ValidationIssue::ReferenceCycle(block_ids) => Self::ReferenceCycle(block_ids),
            ValidationIssue::EmptySpan(block_id) => Self::EmptySpan(block_id),
            ValidationIssue::InvalidThresholds(block_id) => Self::InvalidThresholds(block_id),
            ValidationIssue::NoEntityPreferences(block_id) => Self::NoEntityPreferences(block_id),
        }
    }
}

impl DiagnosticReport for GroupFormationsDiagnosticReport {
    fn message(&self) -> String {
        let name = &self.formation_name;
        match &self.report_type {
            GroupFormationsDiagnosticReportType::DuplicateFormationName => format!("Formation '{name}': another formation has the same name."),
            GroupFormationsDiagnosticReportType::NoAbsoluteBlock => format!("Formation '{name}': it has no absolute block, so nothing anchors its blocks."),
            GroupFormationsDiagnosticReportType::DuplicateBlockId(block_id) => format!("Formation '{name}': more than one block uses the id {block_id}."),
            GroupFormationsDiagnosticReportType::MissingReference(block_id, referenced_id) => format!("Formation '{name}': block {block_id} references block {referenced_id}, which doesn't exist."),
            GroupFormationsDiagnosticReportType::ForwardReference(block_id, referenced_id) => format!("Formation '{name}': block {block_id} references block {referenced_id}, which is defined after it."),
            GroupFormationsDiagnosticReportType::ReferenceCycle(block_ids) => {
                let block_ids = block_ids.iter().map(|block_id| block_id.to_string()).collect::<Vec<_>>().join(", ");
                format!("Formation '{name}': blocks {block_ids} depend on each other in a loop.")
            },
            GroupFormationsDiagnosticReportType::EmptySpan(block_id) => format!("Formation '{name}': span {block_id} has no members."),
            GroupFormationsDiagnosticReportType::InvalidThresholds(block_id) => format!("Formation '{name}': block {block_id} has a minimum amount of units above its maximum."),
            GroupFormationsDiagnosticReportType::NoEntityPreferences(block_id) => format!("Formation '{name}': block {block_id} has no entity preferences, so no unit can be placed in it."),
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            GroupFormationsDiagnosticReportType::NoAbsoluteBlock |
            GroupFormationsDiagnosticReportType::DuplicateBlockId(_) |
            GroupFormationsDiagnosticReportType::MissingReference(_, _) |
            GroupFormationsDiagnosticReportType::ReferenceCycle(_) => DiagnosticLevel::Error,
            GroupFormationsDiagnosticReportType::DuplicateFormationName |
            GroupFormationsDiagnosticReportType::ForwardReference(_, _) |
            GroupFormationsDiagnosticReportType::EmptySpan(_) |
            GroupFormationsDiagnosticReportType::InvalidThresholds(_) |
            GroupFormationsDiagnosticReportType::NoEntityPreferences(_) => DiagnosticLevel::Warning,
        }
    }
}

impl Display for GroupFormationsDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::DuplicateFormationName => "GroupFormationsDuplicateFormationName",
            Self::NoAbsoluteBlock => "GroupFormationsNoAbsoluteBlock",
            Self::DuplicateBlockId(_) => "GroupFormationsDuplicateBlockId",
            Self::MissingReference(_, _) => "GroupFormationsMissingReference",
            Self::ForwardReference(_, _) => "GroupFormationsForwardReference",
            Self::ReferenceCycle(_) => "GroupFormationsReferenceCycle",
            Self::EmptySpan(_) => "GroupFormationsEmptySpan",
            Self::InvalidThresholds(_) => "GroupFormationsInvalidThresholds",
            Self::NoEntityPreferences(_) => "GroupFormationsNoEntityPreferences",
        }, f)
    }
}

impl GroupFormationsDiagnostic {
    pub fn new(path: &str, pack: &str) -> Self {
        Self {
            path: path.to_owned(),
            pack: pack.to_owned(),
            results: vec![],
        }
    }

    /// This function takes care of checking a GroupFormations file for errors.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack the file belongs to.
    /// * `file` - The file to check. It must be already decoded.
    /// * `global_ignored_diagnostics` - Diagnostics disabled in the diagnostics panel.
    ///
    /// # Returns
    ///
    /// The diagnostic with the issues of the file, or `None` if it has none or isn't decoded.
    pub fn check(
        pack_key: &str,
        file: &RFile,
        global_ignored_diagnostics: &[String],
    ) -> Option<DiagnosticType> {
        let Ok(RFileDecoded::GroupFormations(data)) = file.decoded() else { return None };

        let mut diagnostic = GroupFormationsDiagnostic::new(file.path_in_container_raw(), pack_key);
        for (formation_index, issue) in data.validate() {
            let report_type = GroupFormationsDiagnosticReportType::from(issue);
            if global_ignored_diagnostics.contains(&report_type.to_string()) {
                continue;
            }

            let formation_name = data.formations().get(formation_index).map(|formation| formation.name().as_str()).unwrap_or_default();
            diagnostic.results_mut().push(GroupFormationsDiagnosticReport::new(formation_index, formation_name, report_type));
        }

        if diagnostic.results().is_empty() {
            None
        } else {
            Some(DiagnosticType::GroupFormations(diagnostic))
        }
    }
}
