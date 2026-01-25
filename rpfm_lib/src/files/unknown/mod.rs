//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Wildcard handler for unsupported or unrecognized file types.
//!
//! This module provides the [`Unknown`] type, which acts as a passthrough for files that
//! don't have dedicated parsers in rpfm_lib. It stores the raw binary data without attempting
//! to decode it, allowing safe manipulation of any file type.
//!
//! # Use Cases
//!
//! - Working with file types that don't have dedicated support
//! - Reading and re-saving files without modification
//! - Custom processing of binary data outside rpfm_lib
//! - Placeholder for future file type implementations
//!
//! # Example
//!
//! ```ignore
//! use rpfm_lib::files::{Decodeable, Encodeable, unknown::Unknown, DecodeableExtraData, EncodeableExtraData};
//! use std::io::Cursor;
//!
//! // Read arbitrary binary data
//! let data = vec![0x01, 0x02, 0x03, 0x04];
//! let mut reader = Cursor::new(data.clone());
//! let unknown = Unknown::decode(&mut reader, &None).unwrap();
//!
//! // Access raw data
//! assert_eq!(unknown.data(), &data);
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Container for unsupported or unrecognized file data.
///
/// Stores raw binary data without any parsing or validation. This allows rpfm_lib
/// to handle any file type safely, even if it doesn't have a dedicated decoder.
///
/// # Fields
///
/// * `data` - Raw binary contents of the file
///
/// # Getters/Setters
///
/// All fields have public getters, mutable getters, and setters via the `getset` crate:
/// - `data()` - Get reference to data
/// - `data_mut()` - Get mutable reference to data
/// - `set_data()` - Set data
#[derive(Clone, Debug, PartialEq, Eq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Unknown {
    /// Raw binary data of the file.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Unknown {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for Unknown {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
