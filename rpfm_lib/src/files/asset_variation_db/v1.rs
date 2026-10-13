//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use super::*;

/// Version of every record of the database. No other is known.
const RECORD_VERSION: u16 = 1;

//---------------------------------------------------------------------------//
//                    Implementation of AssetVariationDb
//---------------------------------------------------------------------------//

impl Decodeable for AssetVariationDb {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(8, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFastBinUnsupportedSignature(signature_bytes));
        }

        let mut db = Self {
            serialise_version: data.read_u16()?,
            ..Default::default()
        };

        if db.serialise_version != 1 {
            return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("AssetVariationDb"), db.serialise_version));
        }

        let category_count = data.read_u32()?;
        db.categories = Vec::with_capacity(category_count as usize);
        for _ in 0..category_count {
            read_record_version(data, "VariationCategory")?;

            let name = data.read_sized_string_u8()?;
            let value_count = data.read_u32()?;
            let values = (0..value_count).map(|_| data.read_sized_string_u8()).collect::<Result<_>>()?;
            db.categories.push(VariationCategory { name, values });
        }

        let entry_count = data.read_u32()?;
        db.entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let key = data.read_sized_string_u8()?;
            read_record_version(data, "VariationEntry")?;

            let variation_count = data.read_u32()?;
            let mut variations = Vec::with_capacity(variation_count as usize);
            for _ in 0..variation_count {
                read_record_version(data, "Variation")?;
                variations.push(Variation {
                    filename: data.read_sized_string_u8()?,
                    flags: data.read_u32()?,
                });
            }

            db.entries.push(VariationEntry { key, variations });
        }

        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(db)
    }
}

impl Encodeable for AssetVariationDb {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        if self.serialise_version != 1 {
            return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("AssetVariationDb"), self.serialise_version));
        }

        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;

        buffer.write_u32(self.categories.len() as u32)?;
        for category in &self.categories {
            buffer.write_u16(RECORD_VERSION)?;
            buffer.write_sized_string_u8(&category.name)?;
            buffer.write_u32(category.values.len() as u32)?;
            for value in &category.values {
                buffer.write_sized_string_u8(value)?;
            }
        }

        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_sized_string_u8(&entry.key)?;
            buffer.write_u16(RECORD_VERSION)?;
            buffer.write_u32(entry.variations.len() as u32)?;
            for variation in &entry.variations {
                buffer.write_u16(RECORD_VERSION)?;
                buffer.write_sized_string_u8(&variation.filename)?;
                buffer.write_u32(variation.flags)?;
            }
        }

        Ok(())
    }
}

/// Reads the version of a record, failing if it's not the known one.
fn read_record_version<R: ReadBytes>(data: &mut R, record: &str) -> Result<()> {
    let version = data.read_u16()?;
    if version != RECORD_VERSION {
        return Err(RLibError::DecodingFastBinUnsupportedVersion(record.to_owned(), version));
    }

    Ok(())
}
