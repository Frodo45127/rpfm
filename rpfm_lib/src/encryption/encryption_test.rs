//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for encrypting and decrypting Pack data.

use std::fs;
use std::io::Cursor;

use crate::compression::CompressionFormat;
use crate::files::{Container, EncodeableExtraData, FileType, RFile};
use crate::files::pack::{Pack, PFHFlags};
use crate::games::{pfh_version::PFHVersion, supported_games::*};
use super::{Decryptable, Encryptable};

/// Trailing bytes that don't fill a full 8-byte chunk, which are stored unencrypted.
const PLAIN_TAIL: [u8; 3] = [0xAA, 0xBB, 0xCC];

/// Builds a ciphertext from the given encrypted chunks plus the unencrypted tail.
fn ciphertext(encrypted_chunks: &[u8]) -> Vec<u8> {
    let mut data = encrypted_chunks.to_vec();
    data.extend_from_slice(&PLAIN_TAIL);
    data
}

/// Builds the expected plaintext from the given decrypted chunks plus the unencrypted tail.
fn plaintext(decrypted_chunks: &[u8]) -> Vec<u8> {
    let mut data = decrypted_chunks.to_vec();
    data.extend_from_slice(&PLAIN_TAIL);
    data
}

#[test]
fn test_decrypt_pfh4() {

    // Start of a .wem file from Attila's music.pack.
    let encrypted = [0xa0, 0x24, 0x1f, 0xf9, 0xce, 0xca, 0x98, 0xb0, 0xd5, 0x9c, 0x72, 0xff, 0x0a, 0x79, 0x16, 0x11];
    let decrypted = b"RIFF\x68\xad\x23\x00WAVEfmt ";

    let result = Cursor::new(ciphertext(&encrypted)).decrypt(PFHVersion::PFH4).unwrap();
    assert_eq!(result, plaintext(decrypted));
}

#[test]
fn test_decrypt_pfh5() {

    // Start of a .wem file from Warhammer 3's audio_base_m.pack.
    let encrypted = [0xa0, 0x24, 0x1f, 0xf9, 0xa6, 0xf3, 0x16, 0x70, 0xd5, 0x9c, 0x72, 0xff, 0x38, 0xef, 0xcf, 0xd0];
    let decrypted = b"RIFF\x3e\x26\x02\x00WAVEfmt ";

    let result = Cursor::new(ciphertext(&encrypted)).decrypt(PFHVersion::PFH5).unwrap();
    assert_eq!(result, plaintext(decrypted));
}

#[test]
fn test_encrypt_pfh4() {

    // Start of a .wem file from Attila's music.pack.
    let encrypted = [0xa0, 0x24, 0x1f, 0xf9, 0xce, 0xca, 0x98, 0xb0, 0xd5, 0x9c, 0x72, 0xff, 0x0a, 0x79, 0x16, 0x11];
    let decrypted = b"RIFF\x68\xad\x23\x00WAVEfmt ";

    let mut result = vec![];
    result.encrypt(&plaintext(decrypted), PFHVersion::PFH4).unwrap();
    assert_eq!(result, ciphertext(&encrypted));
}

#[test]
fn test_encrypt_pfh5() {

    // Start of a .wem file from Warhammer 3's audio_base_m.pack.
    let encrypted = [0xa0, 0x24, 0x1f, 0xf9, 0xa6, 0xf3, 0x16, 0x70, 0xd5, 0x9c, 0x72, 0xff, 0x38, 0xef, 0xcf, 0xd0];
    let decrypted = b"RIFF\x3e\x26\x02\x00WAVEfmt ";

    let mut result = vec![];
    result.encrypt(&plaintext(decrypted), PFHVersion::PFH5).unwrap();
    assert_eq!(result, ciphertext(&encrypted));
}

#[test]
fn test_encrypt_u32_roundtrip() {
    let value = 0x0012_3456;
    let second_key = 7;

    let mut encrypted = vec![];
    encrypted.encrypt_u32(value, second_key).unwrap();

    assert_ne!(encrypted, value.to_le_bytes());
    assert_eq!(Cursor::new(encrypted).decrypt_u32(second_key).unwrap(), value);
}

#[test]
fn test_encrypt_string_roundtrip() {

    // Longer than the 64-byte key, so the key wraps around.
    let path = "db\\land_units_tables\\a_very_long_table_file_name_to_force_the_index_key_to_wrap";
    let second_key = 0x5A;

    let mut encrypted = vec![];
    encrypted.encrypt_string(path, second_key).unwrap();

    assert_eq!(encrypted.len(), path.len() + 1);
    assert!(!contains(&encrypted, path.as_bytes()));
    assert_eq!(Cursor::new(encrypted).decrypt_string(second_key).unwrap(), path);
}

/// Checks if `haystack` contains `needle` anywhere.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

/// Saves a Pack with encrypted index and data to disk, then reads it back and checks its contents survived.
fn encrypted_pack_roundtrip(game_key: &str, pfh_version: PFHVersion, compression_format: CompressionFormat) {
    let games = SupportedGames::default();
    let game = games.game(game_key).unwrap();

    // Sizes cover empty data, less than one chunk, exact chunks, and chunks with a remainder.
    let files = [
        ("db/units_tables/encrypted", vec![], 1_600_000_001),
        ("text/short.txt", b"short".to_vec(), 1_600_000_002),
        ("text/exact.txt", b"sixteen bytes!!!".to_vec(), 1_600_000_003),
        ("text/folder/long.txt", (0..=255u8).cycle().take(1037).collect(), 1_600_000_004),
    ];

    let mut pack = Pack::new_with_version(pfh_version);
    pack.set_bitmask(PFHFlags::HAS_ENCRYPTED_INDEX | PFHFlags::HAS_ENCRYPTED_DATA | PFHFlags::HAS_INDEX_WITH_TIMESTAMPS);
    pack.set_compression_format(compression_format, game);
    for (path, data, timestamp) in &files {
        pack.insert(RFile::new_from_vec(data, FileType::Text, *timestamp, path)).unwrap();
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let pack_path = temp_dir.path().join("encrypted.pack");
    let extra_data = EncodeableExtraData::new_from_game_info_and_settings(game, compression_format, true);
    pack.save(Some(&pack_path), game, &Some(extra_data)).unwrap();

    let raw = fs::read(&pack_path).unwrap();
    assert!(!contains(&raw, b"text\\folder\\long.txt"));
    assert!(!contains(&raw, b"sixteen bytes!!!"));

    let mut decoded = Pack::read_and_merge(&[pack_path], game, false, false, false).unwrap();
    assert_eq!(decoded.pfh_version(), pfh_version);
    assert!(decoded.bitmask().contains(PFHFlags::HAS_ENCRYPTED_INDEX | PFHFlags::HAS_ENCRYPTED_DATA));
    assert_eq!(decoded.files().len(), files.len());

    for (path, data, timestamp) in &files {
        let file = decoded.file_mut(path, false).unwrap();
        file.load().unwrap();
        assert_eq!(file.cached().unwrap(), data.as_slice(), "data mismatch in {path}");
        assert_eq!(file.timestamp(), Some(*timestamp), "timestamp mismatch in {path}");
    }
}

#[test]
fn test_encrypted_pack_roundtrip_pfh4() {
    encrypted_pack_roundtrip(KEY_ATTILA, PFHVersion::PFH4, CompressionFormat::None);
}

#[test]
fn test_encrypted_pack_roundtrip_pfh5() {
    encrypted_pack_roundtrip(KEY_WARHAMMER_3, PFHVersion::PFH5, CompressionFormat::None);
}

#[test]
fn test_encrypted_pack_roundtrip_pfh5_compressed() {
    encrypted_pack_roundtrip(KEY_WARHAMMER_3, PFHVersion::PFH5, CompressionFormat::Lz4);
}

#[test]
fn test_encrypted_pack_roundtrip_pfh6() {
    encrypted_pack_roundtrip(KEY_TROY, PFHVersion::PFH6, CompressionFormat::None);
}
