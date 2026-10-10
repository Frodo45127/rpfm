//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::NodeType;

use super::*;
use crate::startpos::esf_nav::{as_u32, child_record};
use crate::startpos::startpos_test::{field, field_str, test_data_tables, test_startpos, world, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the region religions to the fixture: the rebel region has a tie between two religions.
fn rebel_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "start_pos_region_religions", &["region", "religion", "percentage"], &[
        &["12", "rel_minor", "20.0"],
        &["12", "rel_first", "40.0"],
        &["12", "rel_second", "40.0"],
        &["10", "rel_other", "100.0"],
    ]);
    tables
}

#[test]
fn test_rebel_region_setup() {
    let tables = rebel_tables();
    let index = fixture_index(&tables);

    let setup = rebel_setup(&tables, index.region("reg_r").unwrap()).unwrap().unwrap();

    assert_eq!(setup.faction(), "rebels_x");
    assert_eq!(setup.religion(), "rel_first");
    assert_eq!(setup.flag().path(), "flags/rebels");
    assert_eq!(setup.flag().colours(), [[91, 91, 91], [0, 0, 0], [0, 0, 0]]);
}

#[test]
fn test_owned_regions_use_the_default_setup() {
    let tables = rebel_tables();
    let index = fixture_index(&tables);

    assert_eq!(rebel_setup(&tables, index.region("reg_a1").unwrap()).unwrap(), None);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_rebels_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();

    let mut mismatches = vec![];
    for region in world_regions(world(test_startpos())) {
        let key = field_str(region, 1);
        let manager = child_record(region, "REGION_REBEL_MANAGER").unwrap();
        let active = matches!(field(manager, 1), NodeType::Bool(value) if *value.value());
        let setup = child_record(manager, "REBEL_SETUP").unwrap();
        let flag = child_record(setup, "FACTION_FLAG_AND_COLOURS").unwrap();
        let colours = (1..=9).map(|position| as_u32(field(flag, position)).unwrap()).collect::<Vec<_>>();

        let generated = rebel_setup(tables, index.region(key).unwrap()).unwrap();
        let matches = match &generated {
            None => !active,
            Some(generated) => active
                && field_str(setup, 1) == generated.faction()
                && field_str(setup, 2) == generated.religion()
                && field_str(flag, 0) == generated.flag().path()
                && colours == generated.flag().colours().iter().flatten().map(|&byte| u32::from(byte)).collect::<Vec<_>>(),
        };
        if !matches {
            mismatches.push(key.to_owned());
        }
    }

    assert!(mismatches.is_empty(), "{mismatches:?}");
}
