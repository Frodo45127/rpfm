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
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rpfm_lib::files::esf::NodeType;
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::schema::{Definition, Field, FieldType};

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{compare, test_data_path, test_startpos, Report, TEST_CAMPAIGN};
use crate::startpos::world::writer::writer_test::plan_from_startpos;
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

/// Returns a colour-mapped TGA header with an id of 1 byte and a palette of 24-bit entries.
fn tga_header(palette_length: u16) -> Vec<u8> {
    let mut bytes = vec![1, 1, TGA_COLOUR_MAPPED, 0, 0];
    bytes.extend_from_slice(&palette_length.to_le_bytes());
    bytes.push(24);
    bytes.resize(TGA_HEADER_SIZE, 0);
    bytes.push(0xFF);
    bytes
}

#[test]
fn test_tga_palette_is_read_as_rgb() {
    let mut bytes = tga_header(2);
    bytes.extend_from_slice(&[3, 2, 1, 30, 20, 10, 99]);

    assert_eq!(tga_palette(&bytes).unwrap(), vec![[1, 2, 3], [10, 20, 30]]);
}

#[test]
fn test_tga_palette_rejects_other_images() {
    let mut true_colour = tga_header(0);
    true_colour[2] = 2;

    assert!(tga_palette(&true_colour).is_err());
    assert!(tga_palette(&tga_header(2)).is_err());
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

/// Campaigns with a game-built startpos, and the share of their fields a startpos generated without it must match.
const CAMPAIGNS: [(&str, f64); 4] = [("main_attila", 0.899), ("pro_attila", 0.906), ("bel_attila", 0.906), ("cha_attila", 0.899)];

/// Returns the path of the game's startpos of a campaign, in the test data's game folder.
fn game_startpos_path(campaign: &str) -> std::path::PathBuf {
    test_data_path("game").join("data").join("campaigns").join(campaign).join("startpos.esf")
}

/// Returns a pack with a campaign's victory objectives, from the test data game's Assembly Kit.
fn victory_objectives_pack(campaign: &str) -> Pack {
    let path = test_data_path("game").join("assembly_kit/raw_data/EmpireDesignData/campaigns").join(campaign).join("victory_objectives.txt");
    let mut file = RFile::new_from_vec(&std::fs::read(path).unwrap(), FileType::Text, 0, VICTORY_OBJECTIVES_PATH);
    file.guess_file_type().unwrap();
    let mut pack = Pack::default();
    pack.insert(file).unwrap();
    pack
}

/// Compares the records of a startpos with a game-built one: the campaign model's sections, the preopen map info and
/// the save game header.
fn compare_startposes(layout: &StartposLayout, ours: &ESF, theirs: &ESF) -> Report {
    let (NodeType::Record(ours), NodeType::Record(theirs)) = (ours.root_node(), theirs.root_node()) else { panic!("a startpos root isn't a record") };
    let model = |root| child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    let mut report = Report::default();
    for section in model(theirs).children().iter().flatten().filter_map(as_record) {
        let path = format!("CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/{}", section.name());
        match child_record(model(ours), section.name()) {
            Some(our_section) => compare(layout, our_section, section, &path, &mut report),
            None => report.count(format!("{path}:<missing>"), false),
        }
    }
    for name in ["CAMPAIGN_PREOPEN_MAP_INFO", "SAVE_GAME_HEADER"] {
        if let (Some(our_record), Some(their_record)) = (child_record(ours, name), child_record(theirs, name)) {
            compare(layout, our_record, their_record, &format!("CAMPAIGN_STARTPOS/{name}"), &mut report);
        }
    }
    report
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_new_campaigns_match_the_game_startposes() {
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();
    let schema = Schema::load(&test_data_path("schema.ron"), None).unwrap();
    let mut dependencies = Dependencies::load(&test_data_path("att.pak"), &Some(schema.clone())).unwrap();
    dependencies.rebuild(&Some(schema.clone()), &[], None, game, &test_data_path("game"), Path::new("")).unwrap();
    let tables = load_tables(&dependencies, &Pack::default(), &schema).unwrap();

    // Each campaign takes its values from the layouts of the other campaigns only, as a new campaign would. Records
    // and slots only it has come from all of them, but slots start out of records, so its values don't leak.
    let decode = |campaign: &str| ESF::decode(&mut BufReader::new(File::open(game_startpos_path(campaign))?), &None);
    let shapes = StartposLayout::from_esfs(CAMPAIGNS.iter().map(|(campaign, _)| decode(campaign))).unwrap();
    let mut failures = vec![];
    for (campaign, min_ratio) in CAMPAIGNS {
        let others = CAMPAIGNS.iter().filter(|(other, _)| *other != campaign).map(|(other, _)| decode(other));
        let layout = StartposLayout::from_esfs(others).unwrap().with_fallback(&shapes);
        let theirs = ESF::decode(&mut BufReader::new(File::open(game_startpos_path(campaign)).unwrap()), &None).unwrap();

        let pack = victory_objectives_pack(campaign);
        let sources = StartposSources::from_tables(tables.clone(), &dependencies, &pack, campaign).unwrap();
        let ours = generate_startpos_with_plan(&sources, &layout, |index| plan_from_startpos(index, &theirs, &tables)).unwrap();

        let report = compare_startposes(&StartposLayout::from_esf(&theirs).unwrap(), &ours, &theirs);
        let (matches, total) = report.totals();
        let ratio = matches as f64 / total as f64;
        println!("{campaign}: {matches}/{total} ({ratio:.4}) fields match. Worst:\n{}", report.worst(40));
        if ratio < min_ratio {
            failures.push(format!("{campaign}: {ratio:.4} < {min_ratio}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
