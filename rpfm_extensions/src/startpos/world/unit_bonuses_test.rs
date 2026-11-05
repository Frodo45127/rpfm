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
use crate::startpos::world::world_test::add_table;

/// Tables with a forge (bronze armour for infantry, bronze weapons for everyone, province-wide) and a barracks
/// (better weapons for spearmen, in its region only).
fn bonus_tables() -> StartposTables {
    let mut tables = StartposTables::new();
    add_table(&mut tables, "effect_bonus_value_basic_junction", &["bonus_value_id", "effect"], &[
        &["unit_stat_bonus_weapon", "weapon_bronze"],
        &["gdp_agriculture", "farm_income"],
    ]);
    add_table(&mut tables, "effect_bonus_value_unit_category_junction", &["bonus_value_id", "effect", "unit_category"], &[
        &["unit_stat_bonus_armour", "armour_bronze", "inf_melee"],
    ]);
    add_table(&mut tables, "effect_bonus_value_unit_class_junction", &["bonus_value_id", "effect", "unit_class"], &[
        &["unit_stat_bonus_weapon", "weapon_spears", "inf_spr"],
    ]);
    add_table(&mut tables, "building_effects_junction", &["building", "effect", "effect_scope", "value"], &[
        &["forge", "armour_bronze", "province_to_province_own", "5"],
        &["forge", "weapon_bronze", "province_to_province_own", "5"],
        &["barracks", "weapon_spears", "region_to_region_own", "8"],
        &["farm", "farm_income", "region_to_region_own", "10"],
    ]);
    add_table(&mut tables, "land_units", &["key", "category", "class"], &[
        &["spearmen_land", "inf_melee", "inf_spr"],
        &["archers_land", "inf_ranged", "inf_mis"],
    ]);
    add_table(&mut tables, "naval_units", &["key", "category", "class"], &[]);
    add_table(&mut tables, "main_units", &["unit", "is_naval", "land_unit", "naval_unit"], &[
        &["spearmen", "false", "spearmen_land", ""],
        &["archers", "false", "archers_land", ""],
    ]);
    tables
}

#[test]
fn test_bonuses_by_category_and_source() {
    let rules = StatBonusRules::new(&bonus_tables()).unwrap();

    // Armour (stat 0) only for the melee category; weapons (stat 2) for everyone.
    assert_eq!(rules.bonuses("spearmen", [("forge", false)]), vec![(0, 5), (2, 5)]);
    assert_eq!(rules.bonuses("archers", [("forge", false)]), vec![(2, 5)]);
    assert!(rules.bonuses("archers", [("farm", true)]).is_empty());
}

#[test]
fn test_bonuses_take_the_largest_source_and_their_scope() {
    let rules = StatBonusRules::new(&bonus_tables()).unwrap();

    // The class bonus (8) beats the basic one (5); the barracks only reaches its own region.
    assert_eq!(rules.bonuses("spearmen", [("forge", false), ("barracks", true)]), vec![(0, 5), (2, 8)]);
    assert_eq!(rules.bonuses("spearmen", [("forge", false), ("barracks", false)]), vec![(0, 5), (2, 5)]);
}
