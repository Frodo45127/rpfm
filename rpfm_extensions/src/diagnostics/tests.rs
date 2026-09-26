//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::text::Text;

use crate::lua::LuaApi;
use crate::lua::check::LuaDefinitions;

use super::*;
use super::text::{TextDiagnosticReport, TextDiagnosticReportType};

fn lua_file(source: &str) -> RFile {
    let mut text = Text::default();
    text.set_contents(source.to_owned());
    RFile::new_from_decoded(&RFileDecoded::Text(text), 0, "script/campaign/mod/test.lua")
}

/// Runs the text diagnostics over a Lua script, returning its reports.
fn check_lua(source: &str, lua_api: Option<&LuaApi>, global_ignored_diagnostics: &[String]) -> Vec<TextDiagnosticReport> {
    let file = lua_file(source);
    let mut definitions = LuaDefinitions::default();
    definitions.add_script(source);

    let diagnostic = TextDiagnostic::check("pack", &file, &BTreeMap::new(), &Dependencies::default(), global_ignored_diagnostics, &[], &HashSet::new(), &HashMap::new(), lua_api, &definitions);
    match diagnostic {
        Some(DiagnosticType::Text(diagnostic)) => diagnostic.results().clone(),
        Some(_) => panic!("text files must only produce text diagnostics"),
        None => vec![],
    }
}

#[test]
fn test_text_lua_syntax_error_without_api() {
    let reports = check_lua("local x = ", None, &[]);
    assert_eq!(reports.len(), 1);
    assert!(matches!(reports[0].report_type(), TextDiagnosticReportType::LuaSyntaxError(_, _, _)));
    assert!(matches!(reports[0].level(), DiagnosticLevel::Error));
    assert!(reports[0].message().starts_with("Syntax Error: "));

    assert!(check_lua("local x = 1", None, &[]).is_empty());
}

#[test]
fn test_text_lua_api_reports_and_ignores() {
    let mut api = LuaApi::default();
    api.add_events_script("FactionTurnStart = {}\n");

    let source = "core:add_listener(\"l\", \"NotAnEvent\", true, function() end, true)\ncore:add_listener(\"l\", \"FactionTurnStart\", true, function() end, true)";
    let reports = check_lua(source, Some(&api), &[]);
    assert_eq!(reports.len(), 1);
    assert!(matches!(reports[0].report_type(), TextDiagnosticReportType::UnknownEvent(_, _, event) if event == "NotAnEvent"));
    assert!(matches!(reports[0].level(), DiagnosticLevel::Warning));

    assert!(check_lua(source, Some(&api), &["UnknownEvent".to_owned()]).is_empty(), "ignored report types must be skipped");
}

#[test]
fn test_text_wrong_argument_count_message() {
    let report = |minimum, maximum, found| TextDiagnosticReport::new(TextDiagnosticReportType::WrongArgumentCount((0, 0), (0, 0), "f".to_owned(), minimum, maximum, found)).message();
    assert_eq!(report(1, Some(1), 0), "Wrong Argument Count: \"f\" takes 1 arguments, but 0 were passed.");
    assert_eq!(report(1, Some(2), 3), "Wrong Argument Count: \"f\" takes 1 to 2 arguments, but 3 were passed.");
    assert_eq!(report(2, None, 1), "Wrong Argument Count: \"f\" takes at least 2 arguments, but 1 were passed.");
}

/// Builds a GroupFormations file with one formation containing the provided blocks.
fn group_formations_file(blocks: Vec<rpfm_lib::files::group_formations::GroupFormationBlock>) -> RFile {
    use rpfm_lib::files::group_formations::{GroupFormation, GroupFormations, GroupFormationsFormat};

    let mut formation = GroupFormation::new("broken", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(blocks);

    let mut data = GroupFormations::default();
    data.formations_mut().push(formation);
    RFile::new_from_decoded(&RFileDecoded::GroupFormations(data), 0, "groupformations.bin")
}

#[test]
fn test_group_formations_reports_and_ignores() {
    use rpfm_lib::files::group_formations::{Block, ContainerRelative, GroupFormationBlock, Spanning};
    use super::group_formations::GroupFormationsDiagnosticReportType;

    let mut relative = ContainerRelative::default();
    relative.set_relative_block_id(7);
    let mut missing_parent = GroupFormationBlock::default();
    missing_parent.set_block(Block::ContainerRelative(relative));

    let mut empty_span = GroupFormationBlock::default();
    empty_span.set_block_id(1);
    empty_span.set_block(Block::Spanning(Spanning::default()));

    let file = group_formations_file(vec![missing_parent, empty_span]);
    let Some(DiagnosticType::GroupFormations(diagnostic)) = GroupFormationsDiagnostic::check("pack", &file, &[]) else { panic!("broken files must produce group formations diagnostics") };

    let reports = diagnostic.results();
    assert!(reports.iter().any(|report| *report.report_type() == GroupFormationsDiagnosticReportType::NoAbsoluteBlock && matches!(report.level(), DiagnosticLevel::Error)));
    assert!(reports.iter().any(|report| *report.report_type() == GroupFormationsDiagnosticReportType::MissingReference(0, 7)));
    assert!(reports.iter().any(|report| *report.report_type() == GroupFormationsDiagnosticReportType::EmptySpan(1) && matches!(report.level(), DiagnosticLevel::Warning)));
    assert_eq!(reports[0].message(), "Formation 'broken': it has no absolute block, so nothing anchors its blocks.");

    let ignored = ["GroupFormationsNoAbsoluteBlock".to_owned()];
    let Some(DiagnosticType::GroupFormations(diagnostic)) = GroupFormationsDiagnostic::check("pack", &file, &ignored) else { panic!("other issues must still be reported") };
    assert!(diagnostic.results().iter().all(|report| *report.report_type() != GroupFormationsDiagnosticReportType::NoAbsoluteBlock), "ignored report types must be skipped");
}
