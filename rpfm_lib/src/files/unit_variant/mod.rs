//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with binary Unit Variants.

Binary unit variants are the unit variants used from Empire to Shogun 2.
!*/

use crate::files::DecodeableExtraData;
use getset::*;

use std::io::SeekFrom;

use crate::error::{RLibError, Result};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{Decodeable, Encodeable};

const SIGNATURE: &str = "VRNT";

/// Size of the header of an UnitVariant PackedFile.
pub const HEADER_SIZE: usize = 4;

pub const EXTENSION: &str = ".unit_variant";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire UnitVariant decoded in memory.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters)]
pub struct UnitVariant {
    version: u32,
    unknown_1: u32,
    categories: Vec<Category>,
}

/// This holds a category of equipments.
#[derive(PartialEq, Clone, Debug, Default,  Getters, Setters)]
pub struct Category {
    name: String,
    id: u64,
    equipments: Vec<(String, String)>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `UnitVariant`.
impl UnitVariant {
    /*
    /// This function checks if the provided data is an UnitVariant.
    pub fn is_unit_variant(packed_file_data: &[u8]) -> bool {
        if let Ok(signature) = packed_file_data.read_string_u8(0, SIGNATURE.len()) {
            signature == SIGNATURE
        } else { false }
    }*/

    /// This function tries to read the header of an UIC PackedFile from raw data.
    fn read_header<R: ReadBytes>(data: &mut R) -> Result<(u32, u32, u32)> {
        if let Ok(signature) = data.read_string_u8(SIGNATURE.len()) {
            if signature != SIGNATURE {
                return Err(RLibError::DecodingUnitVariantNotAUnitVariant)
            }
        }

        let version = data.read_u32()?;
        let categories_count = data.read_u32()?;

        // We don't use them, but it's good to know what they are.
        let _categories_index = data.read_u32()?;
        let _equipments_index = data.read_u32()?;

        // V2 has an extra number here. No idea what it is.
        let unknown_1 = if version == 2 { data.read_u32()? } else { 0 };

        Ok((version, categories_count, unknown_1))
    }

    pub fn get_header_size(&self) -> u32 {
        if self.version == 2 { 24 } else { 20 }
    }
}

impl Decodeable for UnitVariant {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        data.seek(SeekFrom::Start(SIGNATURE.len() as u64))?;
        let (version, categories_count, unknown_1) = Self::read_header(data)?;

        // Get the categories.
        let mut categories = vec![];
        for _ in 0..categories_count {

            let name = data.read_string_u16_0padded(512)?;

            let id = data.read_u64()?;
            let equipments_on_this_category = data.read_u32()?;
            let _equipments_before_this_category = data.read_u32()?;

            let category = Category {
                name,
                id,
                equipments: Vec::with_capacity(equipments_on_this_category as usize),
            };

            categories.push(category)
        }

        // Read the equipments.
        for category in &mut categories {
            for _ in 0..category.equipments.capacity() {
                let equipment_1 = data.read_string_u16_0padded(512)?;
                let equipment_2 = data.read_string_u16_0padded(512)?;
                let _no_idea = data.read_u16()?;

                category.equipments.push((equipment_1, equipment_2));
            }
        }

        // Trigger an error if there's left data on the source.
        let curr_pos = data.stream_position()?;
        let len = data.len()?;
        if curr_pos != len {
            return Err(RLibError::DecodingMismatchSizeError(len as usize, curr_pos as usize))
        }

        // If we've reached this, we've successfully decoded the entire UnitVariant.
        Ok(Self {
            version,
            unknown_1,
            categories
        })
    }
}

impl Encodeable for UnitVariant {
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: Option<DecodeableExtraData>) -> Result<()> {

        let mut encoded_equipments = vec![];
        let mut encoded_categories = vec![];

        let mut equipments_count = 0;
        for category in &self.categories {
            encoded_categories.write_string_u16_0padded(&category.name, 512, true)?;
            encoded_categories.write_u64(category.id)?;
            encoded_categories.write_u32(category.equipments.len() as u32)?;
            encoded_categories.write_u32(equipments_count)?;
            for equipment in &category.equipments {
                encoded_equipments.write_string_u16_0padded(&equipment.0, 512, true)?;
                encoded_equipments.write_string_u16_0padded(&equipment.1, 512, true)?;

                // Two bytes, not one!!!
                encoded_equipments.push(0);
                encoded_equipments.push(0);
            }

            equipments_count += category.equipments.len() as u32;
        }

        buffer.write_string_u8(SIGNATURE)?;
        buffer.write_u32(self.version)?;
        buffer.write_u32(self.categories.len() as u32)?;

        buffer.write_u32(self.get_header_size())?;
        buffer.write_u32(self.get_header_size() + encoded_categories.len() as u32)?;

        if self.version == 2 {
            buffer.write_u32(self.unknown_1)?;
        }

        buffer.write_all(&mut encoded_categories)?;
        buffer.write_all(&mut encoded_equipments)?;

        Ok(())
    }
}
