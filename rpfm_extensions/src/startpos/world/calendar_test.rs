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
use crate::startpos::world::world_test::{add_table, FIXTURE_CAMPAIGN};

/// Returns the calendars of the fixture campaign and another one.
fn calendar_tables() -> StartposTables {
    let mut tables = StartposTables::new();
    add_table(&mut tables, "start_pos_calendars", &["campaign", "start_year", "turns_per_year", "start_season"], &[
        &["other", "1147", "4", "season_spring"],
        &[FIXTURE_CAMPAIGN, "1285", "12", "season_summer"],
    ]);
    tables
}

#[test]
fn test_calendar_of_the_campaign() {
    let tables = calendar_tables();

    let calendar = Calendar::new(&tables, FIXTURE_CAMPAIGN).unwrap();

    assert_eq!((calendar.start_year(), calendar.turns_per_year(), calendar.start_season().as_str()), (1285, 12, "season_summer"));
}

#[test]
fn test_calendar_of_a_campaign_without_one() {
    assert!(Calendar::new(&calendar_tables(), "missing").is_err());
}
