//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code to decrypt encrypted data in Total War PackFiles.
//!
//! The [`Decryptable`] trait provides functions to decrypt various parts of encrypted PackFiles,
//! including file data, file sizes, and file paths. An implementation for anything that implements
//! [`ReadBytes`] + [`Read`] + [`Seek`] is provided.
//!
//! # Encryption Scheme
//!
//! Total War games use a custom encryption scheme with different keys for different parts of the PackFile:
//! - **Index String Key**: 64-byte key for decrypting file paths
//! - **Index U32 Key**: [`u32`] key for decrypting file sizes
//! - **Data Key**: [`u64`] key for decrypting file data
//!
//! # Historical Context
//!
//! The encryption scheme has evolved over time:
//! - **Shogun 2 to Arena**: Used older keys (now commented out in the code)
//! - **Modern Games**: Use the current key set introduced after Arena
//!
//! [`Read`]: std::io::Read
//! [`Seek`]: std::io::Seek

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::io::{Read, Seek};

use crate::error::Result;
use crate::binary::ReadBytes;

// Old 64-byte key used in Arena and all the way back to Shogun 2 for decrypting file paths.
// This key is no longer used but is kept for reference and backwards compatibility with older PackFiles.
// static INDEX_STRING_KEY: &str = "L2{B3dPL7L*v&+Q3ZsusUhy[BGQn(Uq$f>JQdnvdlf{-K:>OssVDr#TlYU|13B}r";

// Old u32 key used in Arena's encrypted PackFiles for decrypting file sizes.
// This key is no longer used but is kept for reference and backwards compatibility with older PackFiles.
// static INDEX_U32_KEY: u32 = 0x1509_1984;

/// Current 64-byte key used for decrypting PackedFile paths in the encrypted index.
///
/// This key rotates through its 64 bytes during the decryption process. Each character of the
/// encrypted path is XORed with the corresponding byte from this key (wrapping around after 64 bytes).
static INDEX_STRING_KEY: [u8; 64] = *b"#:AhppdV-!PEfz&}[]Nv?6w4guU%dF5.fq:n*-qGuhBJJBm&?2tPy!geW/+k#pG?";

/// Current [`u32`] key used for decrypting PackedFile sizes in the encrypted index.
///
/// This key is combined with a position-based secondary key during the XOR decryption process.
static INDEX_U32_KEY: u32 = 0xE10B_73F4;

/// Current [`u64`] key used for decrypting PackedFile data.
///
/// This key is used in 8-byte chunks to decrypt the actual file data. The decryption
/// formula is: `decrypted = encrypted XOR (DATA_KEY * !position)`.
static DATA_KEY: u64 = 0x8FEB_2A67_40A6_920E;

/// Trait for decrypting encrypted PackFile data.
///
/// This trait provides methods to decrypt different parts of encrypted PackFiles using
/// Total War's custom encryption scheme. The decryption uses three different keys for
/// different types of data (file data, file sizes, and file paths).
///
/// # Implementation
///
/// This trait is automatically implemented for any type that implements
/// [`ReadBytes`] + [`Read`] + [`Seek`].
///
/// [`Read`]: std::io::Read
/// [`Seek`]: std::io::Seek
pub trait Decryptable: ReadBytes + Read + Seek {

    /// Decrypts the data of an encrypted PackedFile.
    ///
    /// This function decrypts data in 8-byte chunks using the DATA_KEY. The file is first
    /// padded to a multiple of 8 bytes if needed, then decrypted chunk by chunk. Note that
    /// the last chunk is NOT encrypted and is copied as-is.
    ///
    /// # Returns
    ///
    /// A [`Vec<u8>`] containing the decrypted data, or an error if decryption fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::Cursor;
    /// use rpfm_lib::encryption::Decryptable;
    ///
    /// let encrypted_data = vec![/* encrypted bytes */];
    /// let mut cursor = Cursor::new(encrypted_data);
    /// let decrypted = cursor.decrypt()?;
    /// ```
    fn decrypt(&mut self) -> Result<Vec<u8>> {

        // First, make sure the file ends in a multiple of 8. If not, extend it with zeros.
        // We need it because the decoding is done in packs of 8 bytes.
        let ciphertext_len = self.len()? as usize;
        let mut ciphertext = self.read_slice(ciphertext_len, false)?;
        let size = ciphertext.len();
        let padding = 8 - (size % 8);
        if padding < 8 {
            ciphertext.resize(size + padding, 0);
        }

        // Then decrypt the file in packs of 8. It's faster than in packs of 4.
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        let mut edi: u64 = 0;
        let chunks = ciphertext.len() / 8;
        for i in 0..chunks {

            // The last chunk is NOT ENCRYPTED.
            let esi = edi as usize;
            if i == chunks - 1 {
                plaintext.extend_from_slice(&ciphertext[esi..esi + 8]);
            } else {
                let mut prod = DATA_KEY.wrapping_mul(!edi);
                prod ^= (&ciphertext[esi..esi + 8]).read_u64::<LittleEndian>().unwrap();
                plaintext.write_u64::<LittleEndian>(prod).unwrap();
            }
            edi += 8
        }

        // Remove the extra bytes we added in the first step.
        plaintext.truncate(size);
        Ok(plaintext)
    }

    /// Decrypts the size of a PackedFile from the encrypted index.
    ///
    /// This function reads and decrypts a [`u32`] value representing the size of a PackedFile.
    /// The decryption uses both the INDEX_U32_KEY and a second key derived from the position
    /// in the index.
    ///
    /// # Arguments
    ///
    /// * `second_key` - The secondary key, typically the number of PackedFiles after this one in the index.
    ///
    /// # Returns
    ///
    /// The decrypted file size as a [`u32`], or an error if reading fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::Cursor;
    /// use rpfm_lib::encryption::Decryptable;
    ///
    /// let encrypted_index = vec![/* encrypted index bytes */];
    /// let mut cursor = Cursor::new(encrypted_index);
    /// let packed_files_remaining = 5;
    /// let size = cursor.decrypt_u32(packed_files_remaining)?;
    /// ```
    fn decrypt_u32(&mut self, second_key: u32) -> Result<u32> {
        let bytes = self.read_u32()?;
        Ok(bytes ^ INDEX_U32_KEY ^ !second_key)
    }

    /// Decrypts the path of a PackedFile from the encrypted index.
    ///
    /// This function reads and decrypts a null-terminated string representing the path of a PackedFile.
    /// The decryption uses the INDEX_STRING_KEY (a 64-byte rotating key) and a second key derived
    /// from the file's properties.
    ///
    /// # Arguments
    ///
    /// * `second_key` - The secondary key, typically derived from the file's decrypted size.
    ///
    /// # Returns
    ///
    /// The decrypted file path as a [`String`], or an error if reading fails.
    ///
    /// # Implementation Note
    ///
    /// This function reads the encrypted string byte-by-byte until a null terminator (0x00) is found.
    /// Each byte is XORed with the corresponding byte from INDEX_STRING_KEY (rotating through the 64-byte key)
    /// and the inverted second_key.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::io::Cursor;
    /// use rpfm_lib::encryption::Decryptable;
    ///
    /// let encrypted_index = vec![/* encrypted index bytes */];
    /// let mut cursor = Cursor::new(encrypted_index);
    /// let file_size = 1024u8;
    /// let path = cursor.decrypt_string(file_size)?;
    /// ```
    fn decrypt_string(&mut self, second_key: u8) -> Result<String> {
        let mut path: String = String::new();
        let mut index = 0;
        loop {
            let character = self.read_u8()? ^ INDEX_STRING_KEY[index % INDEX_STRING_KEY.len()] ^ !second_key;
            index += 1;
            if character == 0 { break; }
            path.push(character as char);
        }
        Ok(path)
    }
}

impl<R: ReadBytes + Read + Seek> Decryptable for R {}
