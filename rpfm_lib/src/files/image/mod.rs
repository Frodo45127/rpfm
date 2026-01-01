//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a dummy module to read/write images.
//!
//! Read support just stores the raw data of the image, so you can pass it to another
//! lib/program to read it. Write support just writes that data back to the source.
//!
//! Supported extensions are:
//! - `.jpg`
//! - `.jpeg`
//! - `.tga`
//! - `.png`
//! - `.dds`
//! - `.gif`
//!
//! NOTE: DDS files are converted to png in order for a viewer to use them more easily.

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

/// This represents an entire Image File decoded in memory.
#[derive(Default, PartialEq, Eq, Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Image {
    data: Vec<u8>,
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
