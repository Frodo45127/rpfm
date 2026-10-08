//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the summaries of diagnostics results.

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
