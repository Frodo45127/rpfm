//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here should be all the functions related with encryption/decryption.

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::num::Wrapping;

use crate::decoder::Decoder;

// Old key used in Arena, and all the way back to Shogun 2.
// static INDEX_STRING_KEY: &str = "L2{B3dPL7L*v&+Q3ZsusUhy[BGQn(Uq$f>JQdnvdlf{-K:>OssVDr#TlYU|13B}r";

// Old key used in Arena's encrypted PackFiles.
// static INDEX_U32_KEY: u32 = 0x1509_1984;

// Decryption keys. Each one for a piece of the PackFile. The commented ones are old keys no longer used, but valid for old PackFiles.
static INDEX_STRING_KEY: [u8; 64] = *b"#:AhppdV-!PEfz&}[]Nv?6w4guU%dF5.fq:n*-qGuhBJJBm&?2tPy!geW/+k#pG?";
static INDEX_U32_KEY: u32 = 0xE10B_73F4;
static DATA_KEY: Wrapping<u64> = Wrapping(0x8FEB_2A67_40A6_920E);

pub trait Decryptable {

    fn decrypt(&self, use_padding: bool) -> Vec<u8>;
    fn decrypt_u32(&self, second_key: u32) -> Result<u32>;
    fn decrypt_string(&self, second_key: u8, offset: &mut usize) -> String;
}

impl Decryptable for [u8] {

    fn decrypt(&self, use_padding: bool) -> Vec<u8> {

        // First, make sure the file ends in a multiple of 8. If not, extend it with zeros.
        // We need it because the decoding is done in packs of 8 bytes.
        let mut ciphertext = self.to_vec();
        let size = ciphertext.len();
        let padding = 8 - (size % 8);
        if padding < 8 { ciphertext.resize(size + padding, 0) };

        // Then decrypt the file in packs of 8. It's faster than in packs of 4.
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        let mut edi: u32 = 0;
        for _ in 0..ciphertext.len()/8 {

            let mut prod = (DATA_KEY * Wrapping(u64::from(!edi))).0;
            let esi = edi as usize;
            prod ^= (&ciphertext[esi..esi + 8]).read_u64::<LittleEndian>().unwrap();
            plaintext.write_u64::<LittleEndian>(prod).unwrap();
            edi += 8
        }

        // Remove the extra bytes we added in the first step.
        plaintext.truncate(size);
        plaintext
    }

    /// This function decrypts the size of a PackedFile. Requires:
    /// - 'ciphertext': the encrypted size of the PackedFile, read directly as LittleEndian::u32.
    /// - 'packed_files_after_this_one': the amount of items after this one in the Index.
    fn decrypt_u32(&self, second_key: u32) -> Result<u32> {
        let bytes = self.decode_integer_u32(0)?;
        Ok(bytes ^ INDEX_U32_KEY ^ !second_key)
    }

    /// This function decrypts the path of a PackedFile. Requires:
    /// - 'ciphertext': the encrypted data of the PackedFile, read from the begining of the encrypted path.
    /// - 'decrypted_size': the decrypted size of the PackedFile.
    /// - 'offset': offset to know in what position of the index we should continue decoding the next entry.
    fn decrypt_string(&self, second_key: u8, offset: &mut usize) -> String {
        let mut path: String = String::new();
        let mut index = 0;
        loop {
            let character = self[index] ^ INDEX_STRING_KEY[index % INDEX_STRING_KEY.len()] ^ !second_key;
            index += 1;
            if character == 0 { break; }
            path.push(character as char);
        }
        *offset += index;
        path
    }
}


