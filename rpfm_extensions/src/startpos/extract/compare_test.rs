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

/// Returns a row of values by column.
fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(column, value)| (column.to_string(), value.to_string())).collect()
}

#[test]
fn test_numbers_compare_with_tolerance() {
    assert!(same_value("50.0000", "50"));
    assert!(same_value("10", "10.00001"));
    assert!(!same_value("10", "11"));
    assert!(!same_value("a", "b"));
}

#[test]
fn test_pairs_without_direction_are_sorted() {
    let mut neutral = values(&[("faction1", "b"), ("faction2", "a"), ("stance", "neutral")]);
    let mut vassal = values(&[("faction1", "b"), ("faction2", "a"), ("stance", "vassal")]);
    let mut spouses = values(&[("character", "b"), ("related_to", "a"), ("relationship", "spouse")]);

    normalise("start_pos_diplomacy", &mut neutral);
    normalise("start_pos_diplomacy", &mut vassal);
    normalise("start_pos_family_relationships", &mut spouses);

    assert_eq!((neutral["faction1"].as_str(), neutral["faction2"].as_str()), ("a", "b"));
    assert_eq!((vassal["faction1"].as_str(), vassal["faction2"].as_str()), ("b", "a"));
    assert_eq!((spouses["character"].as_str(), spouses["related_to"].as_str()), ("a", "b"));
}

#[test]
fn test_horde_buildings_are_packed() {
    let mut row = values(&[("primary_building", "a"), ("secondary_building_1", ""), ("secondary_building_2", "b")]);

    normalise("start_pos_horde_details", &mut row);

    assert_eq!(row["secondary_building_1"], "b");
    assert_eq!(row["secondary_building_2"], "");
}

#[test]
fn test_province_values_compare_as_sums() {
    let provinces = [("r1", "p"), ("r2", "p")].iter().map(|(region, province)| (region.to_string(), province.to_string())).collect();
    let mut rows = BTreeMap::new();
    rows.insert("r1".to_owned(), values(&[("region", "r1"), ("owning_faction", "f"), ("base_fertility", "2"), ("development_points", "1"), ("initial_public_order", "90")]));
    rows.insert("r2".to_owned(), values(&[("region", "r2"), ("owning_faction", "g"), ("base_fertility", "3"), ("development_points", "4"), ("initial_public_order", "80")]));

    sum_by_province(&mut rows, &provinces);

    assert_eq!(rows["r1"]["base_fertility"], "5");
    assert_eq!(rows["r2"]["base_fertility"], "5");
    assert_eq!(rows["r1"]["development_points"], "1");
    assert_eq!(rows["r2"]["initial_public_order"], "80");
}
