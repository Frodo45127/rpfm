//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Output stream for encoding ESF nodes the way the games write them, so encoded files match the games' byte for byte.
//!
//! The games write ESF files through a 1 MiB buffer. Sizes (of records, groups, entries and arrays) are written as
//! 5-byte placeholders and filled in once known: if the placeholder is still in the buffer, it shrinks to the size's
//! minimal CAULEB128 encoding; if the buffer was flushed since, the size keeps all 5 bytes, padded with `0x80`. That's
//! why only a few sizes of an ESF file are padded, along the records open at each flush.

use std::io::{self, Write};

use crate::binary::WriteBytes;
use crate::error::Result;

#[cfg(test)] #[path = "stream_test.rs"] mod stream_test;

/// Size of the games' write buffer.
const BUFFER_SIZE: usize = 0x10_0000;

/// Free bytes the buffer keeps: a write that would leave fewer flushes the buffer first.
const BUFFER_MARGIN: usize = 1;

/// Width of a size placeholder, the longest CAULEB128 encoding of a `u32`.
const PLACEHOLDER_SIZE: usize = 5;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Encoded ESF nodes, with the state of the game's write buffer.
#[derive(Debug, Default)]
pub(crate) struct EsfStream {

    /// Encoded data.
    data: Vec<u8>,

    /// Bytes of the file written before the data (its header), which also went through the buffer.
    prefix: usize,

    /// File position where the buffer's content starts: everything before it was flushed.
    buffer_start: usize,
}

/// A size written as a placeholder, waiting for its value.
#[derive(Debug)]
pub(crate) struct SizeField {

    /// Position of the placeholder in the data.
    offset: usize,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl EsfStream {

    /// Creates a stream for the data of a file whose header (`prefix` bytes) was already written.
    pub(crate) fn new(prefix: usize) -> Self {
        Self { data: vec![], prefix, buffer_start: 0 }
    }

    /// Returns the encoded data.
    pub(crate) fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Returns the length of the encoded data.
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    /// Writes a size placeholder, the way the games do: four zero bytes, then another one.
    pub(crate) fn open_size(&mut self) -> Result<SizeField> {
        let offset = self.data.len();
        self.write_all(&[0; PLACEHOLDER_SIZE - 1])?;
        self.write_all(&[0])?;
        Ok(SizeField { offset })
    }

    /// Returns the position right after a size placeholder that hasn't been filled yet.
    pub(crate) fn end_of(&self, field: &SizeField) -> usize {
        field.offset + PLACEHOLDER_SIZE
    }

    /// Fills a size placeholder: shrunk to its minimal encoding while it's in the buffer, padded to 5 bytes otherwise.
    ///
    /// # Returns
    ///
    /// The position right after the size.
    pub(crate) fn close_size(&mut self, field: SizeField, value: u32) -> Result<usize> {
        let range = field.offset..field.offset + PLACEHOLDER_SIZE;
        let mut encoded = vec![];
        if self.prefix + field.offset >= self.buffer_start {
            encoded.write_cauleb128(value, 0)?;
            let end = field.offset + encoded.len();
            self.data.splice(range, encoded);
            Ok(end)
        } else {
            encoded.write_cauleb128(value, PLACEHOLDER_SIZE)?;
            self.data[range].copy_from_slice(&encoded);
            Ok(field.offset + PLACEHOLDER_SIZE)
        }
    }

    /// Writes data with its size before it, as arrays are written.
    pub(crate) fn write_sized(&mut self, data: &[u8]) -> Result<()> {
        let field = self.open_size()?;
        self.write_all(data)?;
        self.close_size(field, data.len() as u32)?;
        Ok(())
    }

    /// Returns how many bytes the buffer holds.
    fn buffered(&self) -> usize {
        self.prefix + self.data.len() - self.buffer_start
    }

    /// Marks everything written so far as flushed.
    fn flush_buffer(&mut self) {
        self.buffer_start = self.prefix + self.data.len();
    }
}

impl Write for EsfStream {

    /// Writes data through the buffer: data that doesn't fit flushes it first, and data as big as the buffer skips it.
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let limit = BUFFER_SIZE - BUFFER_MARGIN;
        if self.buffered() + data.len() > limit {
            self.flush_buffer();
        }
        self.data.extend_from_slice(data);
        if data.len() >= limit {
            self.flush_buffer();
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
