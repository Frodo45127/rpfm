//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decrypting Pack data.

use std::io::Cursor;

use crate::games::pfh_version::PFHVersion;
use super::Decryptable;

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
