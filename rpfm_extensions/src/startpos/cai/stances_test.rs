//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeSet;

use super::*;
use crate::startpos::cai::cai_test::{game_cai_record, test_inputs};
use crate::startpos::cai::CAI_PATH;
use crate::startpos::layout::child_path;
use crate::startpos::startpos_test::{test_data_tables, TEST_CAMPAIGN};

/// Type code of the strategic stance manager desire.
const STANCE_MANAGER_CODE: u32 = 144;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_marks_match_the_game_startpos() {
    let inputs = test_inputs();
    let keys = inputs.world.factions().iter().map(|faction| faction.key().as_str()).collect::<Vec<_>>();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let stances = StartingStances::new(test_data_tables(), &index, &keys).unwrap();

    // Each manager's faction component, and the faction components its stance manager marked.
    let position = inputs.plan.factions().iter().enumerate().map(|(position, ids)| (ids.faction(), position)).collect::<HashMap<_, _>>();
    let managers_path = child_path(CAI_PATH, "CAI_INTERFACE_MANAGERS");
    let desires_path = format!("{managers_path}/CAI_BDI_POOL/CAI_BDI_POOL_DESIRES");
    let marks_path = format!("{desires_path}/CAI_STRATEGIC_STANCE_MANAGER/CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION");
    let layout = &inputs.layout;
    let (mut matches, mut total) = (0, 0);
    for manager in layout.read(&managers_path, game_cai_record("CAI_INTERFACE_MANAGERS")).unwrap() {
        let faction = layout.read(&child_path(&managers_path, "CAI_FACTION_MANAGER"), manager.record("CAI_FACTION_MANAGER").unwrap()).unwrap()[0].u32("^+1").unwrap();
        let pool = layout.read(&child_path(&managers_path, "CAI_BDI_POOL"), manager.record("CAI_BDI_POOL").unwrap()).unwrap();
        let desires = layout.read(&desires_path, pool[0].record("CAI_BDI_POOL_DESIRES").unwrap()).unwrap();
        let stance_manager = desires.iter().find(|desire| desire.u32("^+1") == Some(STANCE_MANAGER_CODE)).unwrap().record("CAI_STRATEGIC_STANCE_MANAGER").unwrap();
        let manager_groups = layout.read(&child_path(&desires_path, "CAI_STRATEGIC_STANCE_MANAGER"), stance_manager).unwrap();
        let game = layout.read(&marks_path, manager_groups[0].record("CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION").unwrap()).unwrap().iter()
            .map(|mark| position[&mark.u32("^+1").unwrap()])
            .collect::<BTreeSet<_>>();

        let ours = stances.marks(position[&faction]).iter().copied().collect::<BTreeSet<_>>();
        if ours != game {
            println!("{}: ours {ours:?}, game {game:?}", keys.get(position[&faction]).unwrap_or(&"rebels"));
        }
        matches += ours.intersection(&game).count();
        total += ours.union(&game).count();
    }

    println!("{matches}/{total} marks match");
    assert_eq!(matches, total);
}
