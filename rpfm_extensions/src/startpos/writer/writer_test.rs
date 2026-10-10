//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::fs::File;
use std::io::BufReader;

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::esf::RecordNode;
use rpfm_lib::files::trade_routes::TradeRoutes;

use super::*;
use crate::startpos::esf_nav::as_record;
use crate::startpos::layout::StartposLayout;
use crate::startpos::startpos_test::{compare, test_data_path, test_data_tables, test_startpos, Report};
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

/// Share of the game's fields outside `WORLD` and `CAI_INTERFACE` the writer must match. Raised as the writer covers
/// more records.
const MIN_MATCH_RATIO: f64 = 0.86;

/// Records compared elsewhere (`WORLD`) or not written yet (`CAI_INTERFACE`).
const SKIPPED: [&str; 2] = ["WORLD", "CAI_INTERFACE"];

/// Compares the records under a record one by one, going into the records holding skipped ones.
fn compare_sections(layout: &StartposLayout, ours: &RecordNode, theirs: &RecordNode, path: &str, report: &mut Report) {
    for their_child in theirs.children().iter().flatten().filter_map(as_record) {
        let name = their_child.name().as_str();
        if SKIPPED.contains(&name) {
            continue;
        }
        let child_path = format!("{path}/{name}");
        let Some(our_child) = ours.children().iter().flatten().filter_map(as_record).find(|record| record.name() == name) else {
            report.count(format!("{child_path}:<missing>"), false);
            continue;
        };
        if matches!(name, "CAMPAIGN_ENV" | "CAMPAIGN_MODEL") {
            compare_sections(layout, our_child, their_child, &child_path, report);
        } else {
            compare(layout, our_child, their_child, &child_path, report);
        }
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_startpos_matches_the_game_startpos() {
    let tables = test_data_tables();
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let world = WorldInputs { tables, index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let trade_routes = TradeRoutes::decode(&mut BufReader::new(File::open(test_data_path("trade_routes.ptd")).unwrap()), &None).unwrap();
    let trade_network = TradeNetwork::from_trade_routes(&trade_routes);
    let inputs = StartposInputs { world: &world, trade_network: &trade_network };
    let plan = game_plan(&index);

    let written = write_startpos(&inputs, &plan).unwrap();
    let (NodeType::Record(ours), NodeType::Record(theirs)) = (written.root_node(), test_startpos().root_node()) else { panic!("the roots aren't records") };
    let mut report = Report::default();
    compare_sections(&layout, ours, theirs, ROOT_PATH, &mut report);

    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    assert_eq!(written.signature(), test_startpos().signature());
    assert!(ratio >= MIN_MATCH_RATIO, "{matches}/{total} ({ratio:.4}) fields match. Worst:\n{}", report.worst(80));
}
