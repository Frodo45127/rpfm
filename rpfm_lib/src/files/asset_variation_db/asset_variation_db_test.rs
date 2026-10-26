//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `AssetVariationDb` files.

use std::io::Cursor;

use crate::files::*;

use super::*;

const PATH_BIN: &str = "../test_files/asset_variation_db/test_decode_att.bin";
const PATH_XML: &str = "../test_files/asset_variation_db/test_decode_att.xml";
const PATH_ENVIRONMENT_MAP: &str = "../test_files/asset_variation_db/environment_map_att.xml";

fn load() -> AssetVariationDb {
    AssetVariationDb::decode(&mut Cursor::new(std::fs::read(PATH_BIN).unwrap()), &None).unwrap()
}

/// Every combination of one value per category, with the names of the files that match it.
fn matches_of_every_condition(db: &AssetVariationDb) -> Vec<(VariationQuery, Vec<String>)> {
    let mut queries = vec![VariationQuery::new()];
    for category in db.categories() {
        queries = queries.iter().flat_map(|query| category.values().iter().map(move |value| {
            let mut query = query.clone();
            query.insert(category.name().to_owned(), value.to_owned());
            query
        })).collect();
    }

    queries.into_iter()
        .map(|query| {
            let names = db.matches("environment", &query).variations.iter().map(|variation| variation.filename().to_owned()).collect();
            (query, names)
        })
        .collect()
}

#[test]
fn test_encode_asset_variation_db() {
    let before = std::fs::read(PATH_BIN).unwrap();
    let mut db = AssetVariationDb::decode(&mut Cursor::new(&before), &None).unwrap();

    let mut after = vec![];
    db.encode(&mut after, &None).unwrap();
    assert_eq!(before, after);
}

#[test]
fn test_xml_asset_variation_db() {
    let db = load();
    let xml = std::fs::read_to_string(PATH_XML).unwrap();

    assert_eq!(AssetVariationDb::from_xml(&xml).unwrap(), db);
    assert_eq!(db.to_xml(), xml);
}

#[test]
fn test_xml_asset_variation_db_errors() {
    assert!(AssetVariationDb::from_xml("<other serialise_version='1'/>").is_err());
    assert!(AssetVariationDb::from_xml("<warscape_asset_variation_db serialise_version='1'>").is_err());

    let xml = load().to_xml();
    assert!(AssetVariationDb::from_xml(&xml.replacen("flags='", "flags='-", 1)).is_err());
    assert!(AssetVariationDb::from_xml(&xml.replacen("serialise_version='1'", "serialise_version='2'", 1)).is_err());
}

#[test]
fn test_matches_asset_variation_db() {
    let db = load();
    assert!(db.validate().is_empty());

    let mut query = VariationQuery::new();
    query.insert("map".to_owned(), "an_unlisted_map".to_owned());
    query.insert("precipitation".to_owned(), "snow".to_owned());

    // A value in no list makes the game's first query match nothing, so it asks again with the default map.
    let matches = db.matches("environment", &query);
    assert!(db.query_mask(&query).is_none());
    assert!(matches.used_default_map);
    assert!(!matches.variations.is_empty());
    assert!(matches.variations.iter().all(|variation| variation.flags() == &0 || variation.flags() & db.bit("precipitation", "snow") != 0));

    query.insert("map".to_owned(), "default_map".to_owned());
    assert_eq!(db.matches("environment", &query).variations, matches.variations);

    // An unknown value of any other category is not rescued by the default map.
    query.insert("precipitation".to_owned(), "an_unlisted_weather".to_owned());
    assert!(db.matches("environment", &query).variations.is_empty());

    // The game finds a value by name, whatever category the query puts it under.
    assert_eq!(db.value_bit("snow"), Some(db.bit("precipitation", "snow")));
    let mut moved = VariationQuery::new();
    moved.insert("season".to_owned(), "snow".to_owned());
    assert_eq!(db.query_mask(&moved), Some(db.bit("precipitation", "snow")));
}

#[test]
fn test_edit_categories_asset_variation_db() {
    let original = load();
    let mut db = original.clone();

    // A value in two categories: the game only ever uses the first.
    assert!(db.add_value("season", "snow").is_err());
    assert!(db.rename_value("season", "sp", "snow").is_err());

    // A clone allows exactly what its source allows.
    db.clone_value("climate", "ROME_Med", "TEST_Climate").unwrap();
    for variation in db.entries()[0].variations().iter().filter(|variation| variation.flags() != &0) {
        assert_eq!(variation.flags() & db.bit("climate", "ROME_Med") != 0, variation.flags() & db.bit("climate", "TEST_Climate") != 0);
    }

    db.remove_value("climate", "TEST_Climate").unwrap();
    assert_eq!(db, original);

    db.add_category("test_category").unwrap();
    db.add_value("test_category", "test_value").unwrap();
    db.rename_category("test_category", "renamed").unwrap();
    assert!(db.add_category("renamed").is_err());
    db.remove_category("renamed").unwrap();
    assert_eq!(db, original);
}

#[test]
fn test_edit_asset_variation_db() {
    let original = load();
    let before = matches_of_every_condition(&original);

    let mut db = original.clone();
    db.add_value("season", "test_season").unwrap();
    assert_eq!(db.category("season").unwrap().values().last().unwrap(), "test_season");

    // Conditions that don't use the new value keep their matches.
    let after = matches_of_every_condition(&db).into_iter()
        .filter(|(query, _)| query.get("season").is_none_or(|season| season != "test_season"))
        .collect::<Vec<_>>();
    assert_eq!(after, before);

    db.remove_value("season", "test_season").unwrap();
    assert_eq!(db, original);

    db.rename_value("season", "sp", "spring").unwrap();
    db.rename_value("season", "spring", "sp").unwrap();
    assert_eq!(db, original);

    assert!(db.add_value("season", "sp").is_err());
    assert!(db.add_value("not_a_category", "value").is_err());
    assert!(db.remove_value("map", "default_map").is_err());
    assert!(db.rename_value("season", "sp", "su").is_err());
}

#[test]
fn test_environment_map_asset_variation_db() {
    let xml = std::fs::read_to_string(PATH_ENVIRONMENT_MAP).unwrap();
    let db = AssetVariationDb::from_environment_map(&xml).unwrap();

    assert!(db.validate().is_empty());
    assert_eq!(db.categories().len(), 8);

    let entry = db.entry("environment").unwrap();
    assert_eq!(entry.variations().len(), xml.matches("<environment_file ").count());
    assert!(entry.variations().iter().all(|variation| !variation.filename().is_empty() && *variation.flags() != 0));
}
