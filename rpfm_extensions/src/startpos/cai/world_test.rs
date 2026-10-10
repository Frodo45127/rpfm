//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;
use crate::startpos::cai::cai_test::{game_cai_record, test_inputs, translate, id_translation};
use crate::startpos::startpos_test::{compare, test_data_esf, test_data_tables, test_startpos, world, Report, TEST_CAMPAIGN};

/// Share of the world mirror's fields the writer must match. The rest are the beliefs the components refer to (left
/// empty, the game rebuilds them), the international trade routes (left empty), ports' cells, and the open fields of
/// the factions (see their tests).
const MIN_WORLD_MATCH_RATIO: f64 = 0.99;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_world_mirror_matches_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let (map_data, hlp_data) = (test_data_esf("map_data.esf"), test_data_esf("hlp_data.esf"));
    let inputs = CaiInputs { tables: test_data_tables(), index: &index, layout: &layout, world: world(test_startpos()), map_data: &map_data, hlp_data: &hlp_data, trade_splines: 2486, campaign: TEST_CAMPAIGN };

    let context = CaiContext::new(&inputs).unwrap();
    let written = context.write_world(&inputs).unwrap();
    let written = translate(&written, &id_translation(&test_inputs()));
    let mut report = Report::default();
    compare(&layout, &written, game_cai_record("CAI_WORLD"), CAI_WORLD_PATH, &mut report);

    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    println!("{matches}/{total} fields match. Worst:\n{}", report.worst(30));
    assert!(ratio >= MIN_WORLD_MATCH_RATIO, "{matches}/{total} ({ratio:.4}) fields match");
}
