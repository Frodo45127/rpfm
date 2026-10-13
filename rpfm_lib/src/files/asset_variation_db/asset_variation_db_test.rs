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

    // The game adds no bit for a map it doesn't list, so it matches every map. Tools ask for the default map instead.
    let matches = db.matches("environment", &query);
    let listed = db.matches_listed_map("environment", &query);
    assert!(!matches.used_default_map);
    assert!(listed.used_default_map);
    assert!(listed.variations.len() <= matches.variations.len());
    assert!(listed.variations.iter().all(|variation| variation.flags() == &0 || variation.flags() & db.bit("precipitation", "snow") != 0));
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
