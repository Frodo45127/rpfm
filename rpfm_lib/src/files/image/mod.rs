//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Image file handling with DDS conversion support.
//!
//! This module provides the [`Image`] type for working with image files in Total War PackFiles.
//! It stores raw image data and provides automatic conversion of DDS textures to PNG format
//! for easier viewing and editing.
//!
//! # Supported Formats
//!
//! The following image formats are recognized:
//! - **JPEG** (`.jpg`, `.jpeg`) - Standard photo compression
//! - **PNG** (`.png`) - Lossless compressed images with alpha
//! - **TGA** (`.tga`) - Targa images (common in game assets)
//! - **DDS** (`.dds`) - DirectDraw Surface textures (Total War's primary format)
//! - **GIF** (`.gif`) - Animated or simple images
//!
//! # DDS Conversion
//!
//! DDS files are automatically converted to PNG format when decoded to enable easier
//! viewing in standard image viewers and editors. The original DDS data is preserved
//! for re-encoding.
//!
//! The conversion process handles various DDS formats:
//! - Standard DDS formats supported by the `image` crate
//! - BC3_UNORM compressed textures via re-compression
//! - RGBA_U8 color formats
//!
//! # Example
//!
//! ```ignore
//! use rpfm_lib::files::{Decodeable, image::Image, DecodeableExtraData};
//! use std::io::Cursor;
//!
//! // Read a DDS texture
//! let dds_data = std::fs::read("texture.dds").unwrap();
//! let mut reader = Cursor::new(dds_data);
//! let mut extra = DecodeableExtraData::default();
//! extra.set_is_dds(true);
//! let image = Image::decode(&mut reader, &Some(extra)).unwrap();
//!
//! // Access converted PNG data for viewing
//! if let Some(png_data) = image.converted_data() {
//!     // Display in image viewer
//! }
//! ```

use dds::{ColorFormat, CompressionQuality, Decoder, Encoder, Format, header::Header, ImageView, ImageViewMut};
use getset::*;
use image::{ImageFormat, ImageReader};
use serde_derive::{Serialize, Deserialize};

use std::io::{BufWriter, Cursor};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extensions used by Images.
pub const EXTENSIONS: [&str; 6] = [
    ".jpg",
    ".jpeg",
    ".tga",
    ".dds",
    ".png",
    ".gif"
];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of an image file.
///
/// Stores the raw image data in its original format, plus optionally converted PNG data
/// for DDS textures. The original data is preserved to allow lossless re-encoding.
///
/// # Fields
///
/// * `data` - Raw binary data in the original image format
/// * `converted_data` - For DDS files, PNG-converted data for viewing (optional)
///
/// # Getters
///
/// Fields have public getters via the `getset` crate:
/// - `data()` - Get reference to original image data
/// - `converted_data()` - Get reference to converted PNG data (DDS only)
///
/// # DDS Handling
///
/// When a DDS file is decoded:
/// 1. `data` contains the original DDS bytes
/// 2. `converted_data` contains PNG-converted bytes for display
/// 3. Encoding writes back the original DDS data from `data`
///
/// For non-DDS formats:
/// 1. `data` contains the image bytes
/// 2. `converted_data` is `None`
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::{Decodeable, Encodeable, image::Image};
/// use std::io::Cursor;
///
/// # let png_bytes = vec![137, 80, 78, 71]; // PNG header
/// let mut reader = Cursor::new(png_bytes);
/// let image = Image::decode(&mut reader, &None).unwrap();
///
/// // Access original data
/// let original = image.data();
/// ```
#[derive(Default, PartialEq, Eq, Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Image {
    /// Original raw image data in native format.
    data: Vec<u8>,

    /// PNG-converted data for DDS textures (for viewing/editing).
    converted_data: Option<Vec<u8>>,
}

//---------------------------------------------------------------------------//
//                           Implementation of Image
//---------------------------------------------------------------------------//

impl Decodeable for Image {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        let mut converted_data = None;

        if let Some(extra_data) = extra_data {

            // For dds files, we try to convert them to png instead.
            if extra_data.is_dds {

                match ImageReader::new(Cursor::new(&data))
                    .with_guessed_format()?
                    .decode() {
                    Ok(image) => {
                        let mut cdata = vec![];
                        image.write_to(&mut Cursor::new(&mut cdata), ImageFormat::Png)?;
                        converted_data = Some(cdata);
                    }

                    // If it fails, use the dds crate to re-convert it to a format we can read.
                    Err(_) => {

                        // Decode the data from the file into a single dds texture.
                        let mut decoder = Decoder::new(Cursor::new(&data))?;
                        let size = decoder.main_size();
                        let mut dds_data = vec![0_u8; size.pixels() as usize * 4];

                        // This can through None if the format is wrong.
                        if let Some(view) = ImageViewMut::new(&mut dds_data, size, ColorFormat::RGBA_U8) {
                            decoder.read_surface(view)?;

                            // Then re-encode it into a dds file compatible with Image.
                            let format = Format::BC3_UNORM;
                            let header = Header::new_image(size.width, size.height, format).with_mipmaps();

                            let mut ddata = vec![];
                            let writer = BufWriter::new(&mut ddata);
                            let mut encoder = Encoder::new(writer, format, &header)?;
                            encoder.encoding.quality = CompressionQuality::Fast;
                            encoder.mipmaps.generate = true;

                            // Same with this: can fail if format is wrong.
                            if let Some(view) = ImageView::new(&dds_data, size, ColorFormat::RGBA_U8) {
                                encoder.write_surface(view)?;
                                encoder.finish()?;

                                // Then try again to turn it into a png.
                                let image = ImageReader::new(Cursor::new(&ddata))
                                    .with_guessed_format()?
                                    .decode()?;

                                let mut cdata = vec![];
                                image.write_to(&mut Cursor::new(&mut cdata), ImageFormat::Png)?;
                                converted_data = Some(cdata);
                            } else {
                                return Err(RLibError::DecodingDDSColourFormatUnsupported)
                            }
                        } else {
                            return Err(RLibError::DecodingDDSColourFormatUnsupported)
                        }
                    },
                }
            }
        }

        Ok(Self {
            data,
            converted_data,
        })
    }
}

impl Encodeable for Image {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
