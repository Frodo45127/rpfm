//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::ESF;

use super::*;
use crate::startpos::extract::extract_tables;
use crate::startpos::layout::StartposLayout;
use crate::startpos::missions::parse_victory_objectives;
use crate::startpos::startpos_test::{test_data_path, test_data_tables, test_startpos};
use crate::startpos::writer::writer_test::write_test_startpos;

/// Returns the victory objectives of the test data, without the settlements of loot and raze objectives (the game
/// doesn't keep them).
fn kept_objectives() -> Vec<VictoryObjectives> {
    let text = std::fs::read_to_string(test_data_path("victory_objectives.txt")).unwrap();
    parse_victory_objectives(&text).unwrap().into_iter().map(|objectives| {
        let missions = objectives.missions().iter().map(|mission| {
            let kept = mission.objectives().iter().map(|objective| match objective {
                MissionObjective::LootOrSackNDifferentSettlementsIncluding { total, .. } => MissionObjective::LootOrSackNDifferentSettlementsIncluding { total: *total, regions: vec![] },
                MissionObjective::RazeOrSackNDifferentSettlementsIncluding { total, .. } => MissionObjective::RazeOrSackNDifferentSettlementsIncluding { total: *total, regions: vec![] },
                other => other.clone(),
            }).collect();
            VictoryMission::new(mission.key().to_owned(), mission.victory_type().to_owned(), mission.issuer().to_owned(), kept)
        }).collect();
        VictoryObjectives::new(objectives.faction().to_owned(), objectives.campaign_type(), missions)
    }).collect()
}

/// Extracts the victory objectives of a startpos.
fn extracted_objectives(startpos: &ESF, layout: &StartposLayout) -> Vec<VictoryObjectives> {
    extract_tables(startpos, layout, test_data_tables()).unwrap().victory_objectives().to_vec()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_victory_objectives_extract_from_a_generated_startpos() {
    let (startpos, layout) = write_test_startpos();

    assert_eq!(extracted_objectives(&startpos, &layout), kept_objectives());
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_victory_objectives_extract_from_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();

    assert_eq!(extracted_objectives(test_startpos(), &layout), kept_objectives());
}
