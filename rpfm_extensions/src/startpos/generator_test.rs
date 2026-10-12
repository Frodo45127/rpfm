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

use rpfm_lib::files::table::DecodedData;
use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::schema::{Definition, Field, FieldType};

use super::*;
use crate::startpos::startpos_test::{test_data_path, test_startpos, TEST_CAMPAIGN};
use crate::startpos::tables::tables_test::text;
use crate::startpos::world::legacy::legacy_test::overrides_from_pack;

/// Returns a fragment of a table with a key column and a value column, with the given rows.
fn fragment(rows: &[(&str, i32)]) -> DB {
    let mut definition = Definition::new(1, None);
    definition.set_fields(vec![
        Field { name: "key".to_owned(), field_type: FieldType::StringU8, is_key: true, ..Default::default() },
        Field { name: "value".to_owned(), field_type: FieldType::I32, ..Default::default() },
    ]);
    let mut db = DB::new(&definition, None, "units_tables");
    db.set_data(&rows.iter().map(|(key, value)| vec![text(key), DecodedData::I32(*value)]).collect::<Vec<_>>()).unwrap();
    db
}

#[test]
fn test_later_fragments_replace_rows_with_the_same_key() {
    let mut tables = StartposTables::new();
    add_merged(&mut tables, "units", &[fragment(&[("a", 1), ("b", 2)]), fragment(&[("b", 20), ("c", 3)])]).unwrap();

    let rows = tables.table("units").unwrap().rows().map(|row| (row.str("key").unwrap().into_owned(), row.int("value").unwrap())).collect::<Vec<_>>();
    assert_eq!(rows, vec![("a".to_owned(), 1), ("b".to_owned(), 20), ("c".to_owned(), 3)]);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_layouts_are_extracted_once_and_picked_by_campaign() {
    let game_data = tempfile::tempdir().unwrap();
    for campaign in [TEST_CAMPAIGN, "other_campaign"] {
        let folder = game_data.path().join("campaigns").join(campaign);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::copy(test_data_path("startpos.esf"), folder.join("startpos.esf")).unwrap();
    }
    let layouts_folder = tempfile::tempdir().unwrap();
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();

    let layouts = GameLayouts::load_or_extract(layouts_folder.path(), game, game_data.path()).unwrap();
    for name in [layout_file_name(game.key(), None), layout_file_name(game.key(), Some(TEST_CAMPAIGN)), layout_file_name(game.key(), Some("other_campaign"))] {
        assert!(layouts_folder.path().join(name).is_file());
    }

    // Two copies of a startpos give the same most common values as one, so every layout matches the startpos's.
    let expected = StartposLayout::from_esf(test_startpos()).unwrap();
    let mut tables = StartposTables::new();
    tables.add_rows("campaigns", &["campaign_name", "map_name"], &[vec![text(TEST_CAMPAIGN), text("main_map")], vec![text("my_campaign"), text("main_map")]]).unwrap();
    assert_eq!(layouts.for_campaign(&tables, TEST_CAMPAIGN), expected);
    assert_eq!(layouts.for_campaign(&tables, "my_campaign"), expected);
    assert_eq!(layouts.for_campaign(&tables, "campaign_on_a_new_map"), expected);

    let cached = GameLayouts::load_or_extract(layouts_folder.path(), game, game_data.path()).unwrap();
    assert_eq!(cached.for_campaign(&tables, TEST_CAMPAIGN), expected);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_startpos_generates_from_a_pack() {
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();
    let schema = Schema::load(&test_data_path("schema.ron"), None).unwrap();
    let packs = ["data.pack", "local.pack", "main_attila.pack"].map(test_data_path);
    let pack = Pack::read_and_merge(&packs, game, true, false, false).unwrap();

    let dependencies = Dependencies::default();
    let tables = load_tables(&dependencies, &pack, &schema).unwrap();
    let sources = StartposSources::from_tables(tables, &dependencies, &pack, TEST_CAMPAIGN).unwrap();
    let startpos = generate_startpos(&sources, &StartposLayout::from_esf(test_startpos()).unwrap()).unwrap();

    assert_eq!(startpos.signature(), test_startpos().signature());
    // The localisation covers every campaign's characters; the campaign's ones match its Assembly Kit data.
    let index = CampaignIndex::new(&sources.tables, TEST_CAMPAIGN).unwrap();
    let campaign_overrides = index.characters().iter().map(|character| character.id().to_owned()).filter(|id| sources.legacy_overrides.contains(id)).collect::<BTreeSet<_>>();
    assert_eq!(campaign_overrides, overrides_from_pack());
}
