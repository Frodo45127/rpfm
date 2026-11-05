//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Victory missions of The Dawnless Days, an Attila mod with objective types vanilla Attila doesn't use, against its
//! game-built startpos. To run them, point `RPFM_STARTPOS_TEST_DATA_TDD` to a folder with the mod's
//! `victory_objectives.txt`, its game-built `startpos.esf` and vanilla Attila's main campaign `attila_startpos.esf`.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::esf::ESF;

use super::*;
use crate::startpos::esf_nav::as_record;
use crate::startpos::missions::parse_victory_objectives;

/// Returns a file of the TDD test data folder.
pub(crate) fn tdd_data_path(file_name: &str) -> PathBuf {
    let folder = std::env::var("RPFM_STARTPOS_TEST_DATA_TDD").unwrap_or_else(|_| panic!("set RPFM_STARTPOS_TEST_DATA_TDD to run this test"));
    PathBuf::from(folder).join(file_name)
}

/// Decodes an ESF file of the TDD test data folder.
pub(crate) fn tdd_esf(file_name: &str) -> ESF {
    ESF::decode(&mut BufReader::new(File::open(tdd_data_path(file_name)).unwrap()), &None).unwrap()
}

/// Returns TDD's victory objectives, from its script.
pub(crate) fn tdd_objectives() -> Vec<VictoryObjectives> {
    parse_victory_objectives(&std::fs::read_to_string(tdd_data_path("victory_objectives.txt")).unwrap()).unwrap()
}

/// Returns every record with a name under a record, depth first.
fn records_named<'a>(record: &'a RecordNode, name: &str, found: &mut Vec<&'a RecordNode>) {
    if record.name() == name {
        found.push(record);
    }
    for child in record.children().iter().flatten().filter_map(as_record) {
        records_named(child, name, found);
    }
}

/// Returns the victory mission objectives of a startpos, in order.
fn startpos_objectives(startpos: &ESF) -> Vec<RecordNode> {
    let NodeType::Record(root) = startpos.root_node() else { panic!("no root record") };
    let mut found = vec![];
    records_named(root, "VICTORY_MISSION_OPTIONS", &mut found);
    let mut objectives = vec![];
    records_named(found[0], "CAMPAIGN_MISSION_OBJECTIVE", &mut objectives);
    objectives.into_iter().cloned().collect()
}

/// Writes TDD's victory missions with a layout and checks their objectives are the ones of its startpos.
fn assert_objectives_match(layout: &StartposLayout) {
    let theirs = startpos_objectives(&tdd_esf("startpos.esf"));

    let options = write_victory_mission_options(layout, &tdd_objectives()).unwrap();
    let mut ours = vec![];
    records_named(&options, "CAMPAIGN_MISSION_OBJECTIVE", &mut ours);

    assert_eq!(ours.len(), theirs.len());
    for (index, (ours, theirs)) in ours.into_iter().zip(&theirs).enumerate() {
        assert_eq!(ours, theirs, "objective {index}");
    }
}

#[test]
#[ignore = "needs TDD's data, see the module docs"]
fn test_tdd_objectives_match_its_startpos_with_vanilla_layouts() {
    assert_objectives_match(&StartposLayout::from_esf(&tdd_esf("attila_startpos.esf")).unwrap());
}

#[test]
#[ignore = "needs TDD's data, see the module docs"]
fn test_tdd_objectives_match_its_startpos_with_its_layouts() {
    assert_objectives_match(&StartposLayout::from_esf(&tdd_esf("startpos.esf")).unwrap());
}
