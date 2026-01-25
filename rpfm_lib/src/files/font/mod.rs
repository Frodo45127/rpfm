//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! CUF (Creative Assembly Unicode Font) file format support.
//!
//! This module handles `.cuf` font files used by Total War games to render text.
//! CUF files contain glyph data, metrics, and optional kerning information for
//! bitmap-based font rendering.
//!
//! # File Format
//!
//! CUF files use a custom binary format with the signature `CUF0`. The structure includes:
//! - Font properties (line height, baseline, spacing, etc.)
//! - Glyph mapping table (character code to glyph index)
//! - Glyph dimensions (allocated size, actual size)
//! - Glyph bitmap data (8-bit grayscale)
//! - Optional kerning data (pair-wise spacing adjustments)
//!
//! # Testing Status
//!
//! Currently, only Empire: Total War fonts have been thoroughly tested. Other games
//! may use slight variations of the format.
//!
//! # Credits
//!
//! Most of the reverse-engineering work was done by the Europa Barbarorum Team for
//! their CUF tool. Their comments and insights have been ported here for reference.
//!
//! # Glyph Storage
//!
//! Glyphs are stored as 8-bit grayscale bitmaps. The format uses a sparse representation
//! where only used character codes (non-0xFFFF values) have associated glyph data.
//!
//! # Kerning
//!
//! Kerning support is optional and only present in some font files. When present,
//! kerning data provides spacing adjustments for specific character pairs to improve
//! visual appearance.
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::font::Font;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode a font file
//! let font = Font::decode(&mut data, &None)?;
//!
//! // Access font properties
//! println!("Line height: {}", font.properties().line_height());
//! println!("Supports kerning: {}", font.supports_kerning());
//!
//! // Access glyphs
//! for (char_code, glyph) in font.glyphs() {
//!     println!("Character {}: {}x{}", char_code, glyph.width(), glyph.height());
//! }
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// File extension for CUF font files.
pub const EXTENSION: &str = ".cuf";

#[cfg(test)] mod font_test;

/// CUF file format signature (`CUF0`).
const SIGNATURE: &[u8; 4] = b"CUF0";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a CUF font file.
///
/// Contains font properties, glyph data, and optional kerning information.
/// Glyphs are stored in a sparse map indexed by character code (0-65535).
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Font {
    /// Font rendering properties and metrics.
    properties: CUFProperties,

    /// Map of character codes to glyph data.
    ///
    /// Only contains entries for characters actually defined in the font.
    /// Character codes map to Unicode-like values.
    glyphs: BTreeMap<u16, Glyph>,

    /// Whether this font file includes kerning data.
    supports_kerning: bool,

    /// Character codes below this value do not have kerning data.
    kerning_skip: u16,

    /// Kerning adjustment blocks (one per character code >= kerning_skip).
    kerning_blocks: Vec<Vec<u8>>,
}

/// Font properties controlling text layout and rendering.
///
/// Many of these properties are indices or references whose exact purpose is still
/// being researched. Comments from the Europa Barbarorum Team's research have been
/// preserved for reference.
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CUFProperties {
    /// Unknown purpose (first CUF property).
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

/// Represents a single glyph (character) in a font.
///
/// Contains the character's bitmap data and rendering metrics. Glyphs store both
/// allocated dimensions (for layout) and actual bitmap dimensions (for rendering).
///
/// # Bitmap Data
///
/// The `data` field contains 8-bit grayscale pixel data in row-major order:
/// - Each byte represents one pixel's intensity/alpha (0-255)
/// - Total size is `width × height` bytes
/// - Empty glyphs (spaces, etc.) may have zero-sized data
///
/// # Dimensions
///
/// Two sets of dimensions are stored:
/// - **Allocated** (`alloc_width`, `alloc_height`): Space reserved for layout
/// - **Actual** (`width`, `height`): Size of the bitmap data
///
/// Allocated height can be negative for characters with descenders (e.g., 'g', 'y').
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Glyph {
    /// Glyph code/index in the font.
    ///
    /// This is the internal glyph identifier used by the font file format.
    code: u16,

    /// Unicode character code this glyph represents.
    ///
    /// Maps to the Unicode character value (0-65535 range for BMP).
    /// This is the character that will be displayed when this glyph is rendered.
    character: u16,

    /// Allocated height in the font layout (can be negative).
    ///
    /// This is the vertical space reserved for the glyph in text layout.
    /// Negative values indicate descenders (parts of characters below the baseline).
    /// For example, lowercase 'g' or 'y' typically have negative allocated heights.
    alloc_height: i8,

    /// Allocated width in the font layout.
    ///
    /// This is the horizontal advance width - how far to move the cursor after
    /// rendering this glyph. May differ from the actual bitmap width.
    alloc_width: u8,

    /// Actual bitmap width in pixels.
    ///
    /// The width of the glyph's pixel data. The `data` field contains
    /// `width × height` bytes of bitmap information.
    width: u8,

    /// Actual bitmap height in pixels.
    ///
    /// The height of the glyph's pixel data. The `data` field contains
    /// `width × height` bytes of bitmap information.
    height: u8,

    /// Kerning adjustment value.
    ///
    /// Used for pair-wise spacing adjustments between specific character combinations.
    /// The exact interpretation depends on the kerning data in the font.
    kerning: u32,

    /// 8-bit grayscale bitmap data.
    ///
    /// Contains the glyph's pixel data in row-major order:
    /// - Size: `width × height` bytes
    /// - Format: One byte per pixel (0 = transparent, 255 = opaque)
    /// - Empty for characters with no visual representation (e.g., spaces)
    ///
    /// # Example Layout
    ///
    /// For a 3×2 glyph, data is stored as:
    /// ```text
    /// [row0_col0, row0_col1, row0_col2, row1_col0, row1_col1, row1_col2]
    /// ```
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
