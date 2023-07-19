//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary UnitVariant files.
//!
//! UnitVariants define data about what textures and parts unit models use within a unit.
//! They're usually xml files covered by the [`super::text`] module, but in older games
//! (Shogun 2) they're binary files instead, and thus they need their own logic.
//!
//! # UnitVariant Structure
//!
//! ## Header
//! ### V1
//!
//! | Bytes | Type     | Data                                                         |
//! | ----- | -------- | ------------------------------------------------------------ |
//! | 4     | StringU8 | Signature of the file.                                       |
//! | 4     | [u32]    | Version of the file.                                         |
//! | 4     | [u32]    | Category count.                                              |
//! | 4     | [u32]    | Index from the start to the begining of the categories data. |
//! | 4     | [u32]    | Index from the start to the begining of the variants data.   |
//!
//! ### V2
//!
//! | Bytes | Type     | Data                                                         |
//! | ----- | -------- | ------------------------------------------------------------ |
//! | 4     | StringU8 | Signature of the file.                                       |
//! | 4     | [u32]    | Version of the file.                                         |
//! | 4     | [u32]    | Category count.                                              |
//! | 4     | [u32]    | Index from the start to the begining of the categories data. |
//! | 4     | [u32]    | Index from the start to the begining of the variants data.   |
//! | 4     | [u32]    | Unknown.                                                     |
//!
//! ## Data
//!
//! This is valid for all versions.
//!
//! | Bytes                | Type                                 | Data                                                   |
//! | -------------------- | ------------------------------------ | ------------------------------------------------------ |
//! | 528 * Category Count | [Category](#category-structure) List | List of categories in the UnitVariant.                 |
//! | 1026 * Variant Count | [Variant](#variant-structure) List   | List of variants in the categories of the UnitVariant. |
//!
//! ### Category Structure
//!
//! This is valid for all versions.
//!
//! | Bytes | Type      | Data                                          |
//! | ----- | --------- | --------------------------------------------- |
//! | 512   | StringU16 | Category name. 00-Padded, Max 256 characters. |
//! | 8     | [u64]     | Category Id.                                  |
//! | 4     | [u32]     | Variant count for this category.              |
//! | 4     | [u32]     | Variants before the ones of this category.    |
//!
//!  ### Variant Structure
//!
//! This is valid for all versions.
//!
//! | Bytes | Type      | Data                                                |
//! | ----- | --------- | --------------------------------------------------- |
//! | 512   | StringU16 | Mesh file path. 00-Padded, Max 256 characters.      |
//! | 512   | StringU16 | Texture folder path. 00-Padded, Max 256 characters. |
//! | 2     | [u16]     | Unknown, possibly a termination.                    |

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{RLibError, Result};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// Signature/Magic Numbers/Whatever of an UnitVariant.
const SIGNATURE: &str = "VRNT";

const HEADER_LENGTH_V1: u32 = 20;
const HEADER_LENGTH_V2: u32 = 24;

/// Extension used by UnitVariants.
pub const EXTENSION: &str = ".unit_variant";

#[cfg(test)] mod unit_variant_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire UnitVariant decoded in memory.
#[derive(Eq, PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct UnitVariant {

    /// Version of the UnitVariant.
    version: u32,

    /// Not sure what this is.
    unknown_1: u32,

    /// Variant categories.
    categories: Vec<Category>,
}

/// This holds a variant category.
#[derive(Eq, PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Category {

    /// Name of the category.
    name: String,

    /// Id of the category.
    id: u64,

    /// Variants of this category.
    variants: Vec<Variant>,
}

/// This holds a `Variant` of a Category.
#[derive(Eq, PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Variant {

    /// The file path (case insensitive) of the mesh file of this variant.
    mesh_file: String,

    /// The folder path (case insensitive) of the textures of this variant.
    texture_folder: String,

    /// Unknown value.
    unknown_value: u16
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `UnitVariant`.
impl UnitVariant {

    /// This function tries to read the header of an UnitVariant from a raw data input.
    ///
    /// It returns, in this order: version, amount of categories, and unknown_1.
    fn read_header<R: ReadBytes>(data: &mut R) -> Result<(u32, u32, u32)> {
        let signature = data.read_string_u8(SIGNATURE.len())?;
        if signature != SIGNATURE {
            return Err(RLibError::DecodingUnitVariantNotAUnitVariant)
        }

        let version = data.read_u32()?;
        let categories_count = data.read_u32()?;

        // We don't use them, but it's good to know what they are.
        let _categories_index = data.read_u32()?;
        let _variants_index = data.read_u32()?;

        // V2 has an extra number here. No idea what it is.
        let unknown_1 = if version == 2 { data.read_u32()? } else { 0 };

        Ok((version, categories_count, unknown_1))
    }

    /// This function returns the header binary lenght of the UnitVariant.
    pub fn get_header_size(&self) -> u32 {
        if self.version == 2 { HEADER_LENGTH_V2 } else { HEADER_LENGTH_V1 }
    }
}

impl Decodeable for UnitVariant {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let (version, categories_count, unknown_1) = Self::read_header(data)?;

        // Get the categories.
        let mut categories = Vec::with_capacity(categories_count as usize);
        for _ in 0..categories_count {
            let name = data.read_string_u16_0padded(512)?;
            let id = data.read_u64()?;
            let variants_on_this_category = data.read_u32()?;
            let _variants_before_this_category = data.read_u32()?;

            let category = Category {
                name,
                id,
                variants: Vec::with_capacity(variants_on_this_category as usize),
            };

            categories.push(category)
        }

        // Read the variants.
        for category in &mut categories {
            for _ in 0..category.variants.capacity() {
                let mesh_file = data.read_string_u16_0padded(512)?;
                let texture_folder = data.read_string_u16_0padded(512)?;
                let unknown_value = data.read_u16()?;

                category.variants.push(Variant {
                    mesh_file,
                    texture_folder,
                    unknown_value
                });
            }
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        // If we've reached this, we've successfully decoded the entire UnitVariant.
        Ok(Self {
            version,
            unknown_1,
            categories
        })
    }
}

impl Encodeable for UnitVariant {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {

        let mut encoded_variants = vec![];
        let mut encoded_categories = vec![];

        let mut variants_count = 0;
        for category in &self.categories {
            encoded_categories.write_string_u16_0padded(&category.name, 512, true)?;
            encoded_categories.write_u64(category.id)?;
            encoded_categories.write_u32(category.variants.len() as u32)?;
            encoded_categories.write_u32(variants_count)?;
            for variant in &category.variants {
                encoded_variants.write_string_u16_0padded(&variant.mesh_file, 512, true)?;
                encoded_variants.write_string_u16_0padded(&variant.texture_folder, 512, true)?;
                encoded_variants.write_u16(variant.unknown_value)?;
            }

            variants_count += category.variants.len() as u32;
        }

        buffer.write_string_u8(SIGNATURE)?;
        buffer.write_u32(self.version)?;
        buffer.write_u32(self.categories.len() as u32)?;

        buffer.write_u32(self.get_header_size())?;
        buffer.write_u32(self.get_header_size() + encoded_categories.len() as u32)?;

        if self.version == 2 {
            buffer.write_u32(self.unknown_1)?;
        }

        buffer.write_all(&encoded_categories)?;
        buffer.write_all(&encoded_variants)?;

        Ok(())
    }
}
