//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write Font/CUF files.
//!
//! Currently, only Empire fonts have been tested.
//!
//! Most of the code here was implemented thanks to the research done by the Europa Barbarorum Team for their CUF tool.
//!
//! You may find some of their comments here, ported for reference.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".cuf";

#[cfg(test)] mod font_test;

const SIGNATURE: &[u8; 4] = b"CUF0";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Font {
    properties: CUFProperties,
    glyphs: BTreeMap<u16, Glyph>,

    supports_kerning: bool,
    kerning_skip: u16,
    kerning_blocks: Vec<Vec<u8>>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CUFProperties {

    /// Unknown purpose. First CUF property.
    first_prop: u16,

    /// Unknown purpose. Second CUF property.
    second_prop: u16,

    /// Index of the value which appears to have something to do with line height. Underscore line? Base line?
    line_height: u16,

    /// Unknown purpose. Fourth CUF property.
    fourth_prop: u16,

    /// Unknown purpose. Fifth CUF property.
    fifth_prop: u16,

    /// Index of the value which appears to correspond to a ‘baseline’ of sorts in the CUF file format.
    baseline: u16,

    /// Index of the value which determines y-offset w.r.t. the bounding box of a string of text in this font.
    layout_y_offset: u16,

    /// Used to specify how wide a space is for justification and text wrapping calculations.
    space_justify: u16,

    /// Index of the value which determines x-offset w.r.t. the bounding box of a string of text in this font.
    layout_x_offset: u16,

    /// Index of the value which determines a maximum width for glyphs.
    /// Glyphs which are wider than the maximum specified for this property will appear cut-off.
    ///
    /// There appears to be no effect on the position of a glyph
    /// after a glyph of which the advance is larger than the value specified for this setting.
    ///
    /// Note that individual glyphs contain sufficient information to calculate a much more optimal bounding box than by simply using
    /// multiples of the value corresponding to this index.
    h_size: u16,

    /// Index of the value which determines a maximum height for glyphs.
    /// The corresponding value probably should include leading.
    /// Glyphs which are taller than the maximum specified for this property will appear cut-off.
    ///
    /// Too small values for this property may result in crashes or unspecified errors on exit in M2TW.
    ///
    /// Note that individual glyphs contain sufficient information to calculate a much more optimal bounding box than by simply using
    /// multiples of the value corresponding to this index.
    v_size: u16,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Glyph {
    code: u16,
    character: u16,
    alloc_height: i8,
    alloc_width: u8,
    width: u8,
    height: u8,
    kerning: u32,
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                          Implementation of Font
//---------------------------------------------------------------------------//

impl Decodeable for Font {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(4, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFontUnsupportedSignature(signature_bytes));
        }

        let mut font = Self::default();

        // Get the properties of the font.
        font.properties.first_prop = data.read_u16()?;
        font.properties.second_prop = data.read_u16()?;
        font.properties.line_height = data.read_u16()?;
        font.properties.fourth_prop = data.read_u16()?;
        font.properties.fifth_prop = data.read_u16()?;
        font.properties.baseline = data.read_u16()?;
        font.properties.layout_y_offset = data.read_u16()?;
        font.properties.space_justify = data.read_u16()?;
        font.properties.layout_x_offset = data.read_u16()?;
        font.properties.h_size = data.read_u16()?;
        font.properties.v_size = data.read_u16()?;

        // These are used glyph count, and the size of the data section. Unused by the decoder.
        let _glyph_count = data.read_u16()?;
        let _glyph_data_size = data.read_u32()?;

        // Get the glyphs/characters table. This is a list of u16 from 0 to u16 max value.
        //
        // 0xFFFF values are unused.
        for index in 0..=u16::MAX {
            let code = data.read_u16()?;
            if code == 0xFFFF {
                continue;
            }

            let mut glyph = Glyph::default();

            glyph.code = code;
            glyph.character = index;

            font.glyphs.insert(index, glyph);
        }

        // Get the glyph dimensions data.
        for index in 0..=u16::MAX {
            if let Some(glyph) = font.glyphs_mut().get_mut(&index) {
                glyph.alloc_height = data.read_i8()?;
                glyph.alloc_width = data.read_u8()?;
                glyph.width = data.read_u8()?;
                glyph.height = data.read_u8()?;
            }
        }

        // Get the glyph data offset for the next section. As they're consecutive and have a fixed size, we really don't use
        // these offsets, but this code is left here for format documentation.
        //
        // This list only contains the glyphs that are used, because fuck consistency.
        for _ in 0..font.glyphs().len() {
            let _offset = data.read_u32()?;
        }

        for glyph in font.glyphs_mut().values_mut() {
            let size = glyph.height as usize * glyph.width as usize;
            if size != 0 {
                glyph.data = data.read_slice(size, false)?;
            }
        }

        // Get the glyph kerning info. This seems to be only from certain files onward, so a fail here has to be considered as
        // not an error.
        if let Ok(kerning_size) = data.read_u16() {
            font.supports_kerning = true;

            // Codes lower than the skip one do not have kerning data.
            font.kerning_skip = data.read_u16()?;

            for _ in 0..kerning_size {
                let block = data.read_slice(kerning_size as usize, false)?;
                font.kerning_blocks.push(block);
            }
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(font)
    }
}

impl Encodeable for Font {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;

        buffer.write_u16(*self.properties().first_prop())?;
        buffer.write_u16(*self.properties().second_prop())?;
        buffer.write_u16(*self.properties().line_height())?;
        buffer.write_u16(*self.properties().fourth_prop())?;
        buffer.write_u16(*self.properties().fifth_prop())?;
        buffer.write_u16(*self.properties().baseline())?;
        buffer.write_u16(*self.properties().layout_y_offset())?;
        buffer.write_u16(*self.properties().space_justify())?;
        buffer.write_u16(*self.properties().layout_x_offset())?;
        buffer.write_u16(*self.properties().h_size())?;
        buffer.write_u16(*self.properties().v_size())?;

        buffer.write_u16(self.glyphs().len() as u16)?;

        let mut glyphs = vec![];
        let mut dimensions = vec![];
        let mut offsets = vec![];
        let mut data = vec![];

        for index in 0..=u16::MAX {
            match self.glyphs().get(&index) {
                Some(glyph) => {
                    glyphs.write_u16(glyph.code)?;

                    dimensions.write_i8(glyph.alloc_height)?;
                    dimensions.write_u8(glyph.alloc_width)?;
                    dimensions.write_u8(glyph.width)?;
                    dimensions.write_u8(glyph.height)?;

                    if glyph.data().is_empty() &&
                        glyph.alloc_height == 0 &&
                        glyph.alloc_width == 0 &&
                        glyph.width == 0 &&
                        glyph.height == 0 {
                        offsets.write_u32(0)?;
                    } else {
                        offsets.write_u32(data.len() as u32)?;

                        data.write_all(&glyph.data)?;
                    }

                },
                None => {
                    glyphs.write_u16(0xFFFF)?;
                },
            }
        }
        buffer.write_u32(data.len() as u32)?;

        buffer.write_all(&glyphs)?;
        buffer.write_all(&dimensions)?;
        buffer.write_all(&offsets)?;
        buffer.write_all(&data)?;

        if self.supports_kerning {
            buffer.write_u16(self.kerning_blocks.len() as u16)?;
            buffer.write_u16(self.kerning_skip)?;

            for block in self.kerning_blocks() {
                buffer.write_all(block)?;
            }
        }

        Ok(())
    }
}
