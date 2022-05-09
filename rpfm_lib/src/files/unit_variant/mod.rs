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

use getset::*;

use crate::error::{RLibError, Result};
use crate::{decoder::Decoder, encoder::Encoder, schema::Schema};
use crate::files::{Decodeable, Encodeable, FileType};

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

    /// This function checks if the provided data is an UnitVariant.
    pub fn is_unit_variant(packed_file_data: &[u8]) -> bool {
        if let Ok(signature) = packed_file_data.decode_string_u8(0, SIGNATURE.len()) {
            signature == SIGNATURE
        } else { false }
    }

    /// This function tries to read the header of an UIC PackedFile from raw data.
    pub fn read_header(packed_file_data: &[u8], index: &mut usize) -> Result<(u32, u32, u32)> {
        if let Ok(signature) = packed_file_data.decode_string_u8(0, SIGNATURE.len()) {
            if signature != SIGNATURE {
                return Err(RLibError::DecodingUnitVariantNotAUnitVariant)
            }
        }

        let version = packed_file_data.decode_packedfile_integer_u32(SIGNATURE.len(), index)?;
        let categories_count = packed_file_data.decode_packedfile_integer_u32(*index, index)?;

        // We don't use them, but it's good to know what they are.
        let _categories_index = packed_file_data.decode_packedfile_integer_u32(*index, index)?;
        let _equipments_index = packed_file_data.decode_packedfile_integer_u32(*index, index)?;

        // V2 has an extra number here. No idea what it is.
        let unknown_1 = if version == 2 { packed_file_data.decode_packedfile_integer_u32(*index, index)? } else { 0 };

        Ok((version, categories_count, unknown_1))
    }

    pub fn get_header_size(&self) -> u32 {
        if self.version == 2 { 24 } else { 20 }
    }
}

impl Decodeable for UnitVariant {

    fn file_type(&self) -> FileType {
        FileType::UnitVariant
    }

    fn decode(packed_file_data: &[u8], _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {

        let mut index = SIGNATURE.len();
        let (version, categories_count, unknown_1) = Self::read_header(packed_file_data, &mut index)?;

        // Get the categories.
        let mut categories = vec![];
        for _ in 0..categories_count {

            let (name, _) = packed_file_data.decode_string_u16_0padded(index, 512)?;
            index += 512;

            let id = packed_file_data.decode_packedfile_integer_u64(index, &mut index)?;
            let equipments_on_this_category = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;
            let _equipments_before_this_category = packed_file_data.decode_packedfile_integer_u32(index, &mut index)?;

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
                let equipment_1 = packed_file_data.decode_string_u16_0padded(index, 512)?;
                index += 512;
                let equipment_2 = packed_file_data.decode_string_u16_0padded(index, 512)?;
                index += 512;
                index += 2;

                category.equipments.push((equipment_1.0, equipment_2.0));
            }
        }

        // Trigger an error if there's left data on the source.
        if index != packed_file_data.len() {
            return Err(RLibError::DecodingMismatchSizeError(packed_file_data.len(), index))
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
    fn encode(&self) -> Vec<u8> {

        let mut encoded_equipments = vec![];
        let mut encoded_categories = vec![];

        let mut equipments_count = 0;
        for category in &self.categories {
            encoded_categories.encode_string_u16_0padded_cropped(&category.name, 512);
            encoded_categories.encode_integer_u64(category.id);
            encoded_categories.encode_integer_u32(category.equipments.len() as u32);
            encoded_categories.encode_integer_u32(equipments_count);
            for equipment in &category.equipments {
                encoded_equipments.encode_string_u16_0padded_cropped(&equipment.0, 512);
                encoded_equipments.encode_string_u16_0padded_cropped(&equipment.1, 512);

                // Two bytes, not one!!!
                encoded_equipments.push(0);
                encoded_equipments.push(0);
            }

            equipments_count += category.equipments.len() as u32;
        }

        let mut data = vec![];
        data.encode_string_u8(SIGNATURE);
        data.encode_integer_u32(self.version);
        data.encode_integer_u32(self.categories.len() as u32);

        data.encode_integer_u32(self.get_header_size());
        data.encode_integer_u32(self.get_header_size() + encoded_categories.len() as u32);

        if self.version == 2 {
            data.encode_integer_u32(self.unknown_1);
        }

        data.append(&mut encoded_categories);
        data.append(&mut encoded_equipments);

        data
    }
}
