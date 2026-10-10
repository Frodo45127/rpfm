//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Shared helpers of the startpos tests.
//!
//! Tests that compare against game data are `#[ignore]`d, because the game files can't be distributed. To run
//! them, point `RPFM_STARTPOS_TEST_DATA` to a folder with Attila's main campaign files (`map_data.esf`,
//! `trade_routes.ptd`) and a game-built `startpos.esf`, then run `cargo test -- --ignored`.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::OnceLock;

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::esf::{ESF, NodeType, RecordNode};

use super::esf_nav::{as_record, as_u32, child_record};

/// Environment variable with the folder of the game data used by the ignored tests.
const TEST_DATA_VAR: &str = "RPFM_STARTPOS_TEST_DATA";

/// Returns the path of a file of the startpos test data folder.
pub(crate) fn test_data_path(file_name: &str) -> PathBuf {
    let folder = std::env::var(TEST_DATA_VAR).unwrap_or_else(|_| panic!("set {TEST_DATA_VAR} to run this test"));
    PathBuf::from(folder).join(file_name)
}

/// Decodes an ESF file of the startpos test data folder.
pub(crate) fn test_data_esf(file_name: &str) -> ESF {
    let mut reader = BufReader::new(File::open(test_data_path(file_name)).unwrap());
    ESF::decode(&mut reader, &None).unwrap()
}

/// Returns the game-built startpos of the test data, decoded once for every test.
pub(crate) fn test_startpos() -> &'static ESF {
    static STARTPOS: OnceLock<ESF> = OnceLock::new();
    STARTPOS.get_or_init(|| test_data_esf("startpos.esf"))
}

/// Returns the `WORLD` record of a startpos.
pub(crate) fn world(startpos: &ESF) -> &RecordNode {
    ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "WORLD"].iter()
        .try_fold(as_record(startpos.root_node()).unwrap(), |record, name| child_record(record, name))
        .unwrap()
}

/// Returns the records of an array record (one per group, the group's first record), in order.
pub(crate) fn array_items<'a>(array: &'a RecordNode, item: &'a str) -> Vec<&'a RecordNode> {
    array.children().iter()
        .filter_map(|group| group.iter().filter_map(as_record).find(|record| record.name() == item))
        .collect()
}

/// Returns the factions of a startpos `WORLD`, in order.
pub(crate) fn world_factions(world: &RecordNode) -> Vec<&RecordNode> {
    child_record(world, "FACTION_ARRAY").map(|array| array_items(array, "FACTION")).unwrap_or_default()
}

/// Returns the child at a position of a record's first group (`#n` in the research notes).
pub(crate) fn field(record: &RecordNode, position: usize) -> &NodeType {
    &record.children()[0][position]
}

/// Returns the string at a position of a record's first group.
pub(crate) fn field_str(record: &RecordNode, position: usize) -> &str {
    match field(record, position) {
        NodeType::Ascii(value) | NodeType::Utf16(value) => value,
        other => panic!("not a string: {other:?}"),
    }
}

/// Returns the regions of a startpos `WORLD`, in order.
pub(crate) fn world_regions(world: &RecordNode) -> Vec<&RecordNode> {
    child_record(world, "REGION_MANAGER")
        .and_then(|manager| child_record(manager, "REGIONS_ARRAY"))
        .map(|array| array_items(array, "REGION"))
        .unwrap_or_default()
}

/// Returns the unsigned number at a position of a record's first group.
pub(crate) fn field_u32(record: &RecordNode, position: usize) -> u32 {
    as_u32(field(record, position)).unwrap_or_else(|| panic!("not a number: {:?}", field(record, position)))
}
