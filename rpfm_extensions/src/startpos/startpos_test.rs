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

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::esf::ESF;

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
