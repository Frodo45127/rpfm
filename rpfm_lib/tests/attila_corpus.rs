//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Round trips of every file of a type in Attila's vanilla packs.
//!
//! These need the game, so they're ignored by default. Run them with the game's folder in `RPFM_TEST_ATTILA_PATH`:
//!
//! ```text
//! RPFM_TEST_ATTILA_PATH="/path/to/Total War Attila" cargo test -p rpfm_lib --test attila_corpus -- --ignored
//! ```

use std::io::Cursor;
use std::path::PathBuf;

use rpfm_lib::files::{Decodeable, Encodeable, pack::Pack};
use rpfm_lib::files::fxc::Fxc;
use rpfm_lib::games::supported_games::{KEY_ATTILA, SupportedGames};

/// Amount of failures listed in the panic message.
const MAX_LISTED_FAILURES: usize = 20;

/// Decodes and encodes back every file whose path matches, and fails if any of them doesn't come back byte for byte.
fn round_trip<T: Decodeable + Encodeable>(matches: impl Fn(&str) -> bool) {
    let game_path = PathBuf::from(std::env::var("RPFM_TEST_ATTILA_PATH").expect("RPFM_TEST_ATTILA_PATH must be the game's folder"));
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).expect("Attila is a supported game");
    let mut pack = Pack::read_and_merge_ca_packs(game, &game_path).expect("the vanilla packs can be read");

    let mut total = 0;
    let mut failures = vec![];
    for file in pack.files_mut().values_mut() {
        let path = file.path_in_container_raw().to_lowercase();
        if !matches(&path) {
            continue;
        }

        total += 1;
        let before = file.encode(&None, false, false, true).expect("the file can be read").expect("the data is returned");
        let result = T::decode(&mut Cursor::new(&before), &None).and_then(|mut decoded| {
            let mut after = vec![];
            decoded.encode(&mut after, &None).map(|_| after)
        });

        match result {
            Ok(after) if after == before => {},
            Ok(after) => failures.push(format!("{path}: {} bytes encoded as {}", before.len(), after.len())),
            Err(error) => failures.push(format!("{path}: {error}")),
        }
    }

    assert!(total > 0, "no file matched");
    assert!(failures.is_empty(), "{} of {total} files failed:\n{}", failures.len(), failures.iter().take(MAX_LISTED_FAILURES).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
#[ignore = "needs Attila, in RPFM_TEST_ATTILA_PATH"]
fn fxc() {
    round_trip::<Fxc>(|path| path.ends_with(".fxc"));
}
