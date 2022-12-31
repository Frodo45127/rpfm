//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write Text files.
//!
//! Text files are any kind of plain-text files, really. Encodings supported by this lib are:
//! - `ISO-8859-15`
//! - `UTF-8`
//! - `UTF-16` (LittleEndian)
//!
//! Also, the module automatically tries to guess the language of a Text file, so programs
//! can query the guess language format and apply extended functionality.
//!
//! The full list of file extension this lib supports as `Text` files is:
//!
//! | ------------------------ | -------- | ------------------------------------------- |
//! | Extension                | Language | Description                                 |
//! | ------------------------ | -------- | ------------------------------------------- |
//! | `.battle_speech_camera`  | `Plain`  | Camera settings file for battle speeches.   |
//! | `.benchmark`             | `Xml`    | Benchmark settings.                         |
//! | `.bob`                   | `Plain`  | BoB settings file.                          |
//! | `.cindyscene`            | `Xml`    | Cutscene editor data.                       |
//! | `.cindyscenemanager`     | `Xml`    | Cutscene editor data.                       |
//! | `.code-snippets`         | `Json`   | VSCode snippet file.                        |
//! | `.code-workspace`        | `Json`   | VSCode workspace file.                      |
//! | `.csv`                   | `Plain`  | Normal CSV file.                            |
//! | `.css`                   | `Css`    | Normal CSS file.                            |
//! | `.environment`           | `Xml`    |                                             |
//! | `.htm`                   | `Html`   | Normal HTML file.                           |
//! | `.html`                  | `Html`   | Normal HTML file.                           |
//! | `.inl`                   | `Cpp`    |                                             |
//! | `.json`                  | `Json`   | Normal JSON file.                           |
//! | `.js`                    | `Js`     | Normal Javascript file.                     |
//! | `.kfa`                   | `Xml`    | Battle Audio Event file.                    |
//! | `.kfe`                   | `Xml`    | Battle Effect file.                         |
//! | `.kfl`                   | `Xml`    | Battle Point Light file.                    |
//! | `.kfsl`                  | `Xml`    | Battle Spot Light file.                     |
//! | `.kfp`                   | `Xml`    | Battle Prop file.                           |
//! | `.kfcs`                  | `Xml`    | Battle Composite Scene file.                |
//! | `.lighting`              | `Xml`    |                                             |
//! | `.lua`                   | `Lua`    | LUA Script file.                            |
//! | `.material`              | `Xml`    |                                             |
//! | `.tai`                   | `Plain`  |                                             |
//! | `.technique`             | `Xml`    |                                             |
//! | `.texture_array`         | `Plain`  | List of Campaign Map textures.              |
//! | `.tsv`                   | `Plain`  | Normal TSV file.                            |
//! | `.twui`                  | `Lua`    | TWui file, in lua format.                   |
//! | `.txt`                   | `Plain`  | Plain TXT file.                             |
//! | `.variantmeshdefinition` | `Xml`    |                                             |
//! | `.wsmodel`               | `Xml`    |                                             |
//! | `.xml`                   | `Xml`    | Normal XML file.                            |
//! | `.xml.shader`            | `Xml`    | Shader setup metadata.                      |
//! | `.xml.material`          | `Xml`    |                                             |

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::io::SeekFrom;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::DecodeableExtraData;

/// UTF-8 BOM (Byte Order Mark).
const BOM_UTF_8: [u8;3] = [0xEF,0xBB,0xBF];

/// UTF-16 BOM (Byte Order Mark), Little Endian.
const BOM_UTF_16_LE: [u8;2] = [0xFF,0xFE];

/// List of extensions we recognize as `Text` files, with their respective known format.
pub const EXTENSIONS: [(&str, TextFormat); 35] = [
    (".battle_speech_camera", TextFormat::Plain),
    (".benchmark", TextFormat::Xml),
    (".bob", TextFormat::Plain),
    (".cindyscene", TextFormat::Xml),
    (".cindyscenemanager", TextFormat::Xml),
    (".code-snippets", TextFormat::Json),
    (".code-workspace", TextFormat::Json),
    (".css", TextFormat::Css),
    (".csv", TextFormat::Plain),
    (".environment", TextFormat::Xml),
    (".htm", TextFormat::Html),
    (".html", TextFormat::Html),
    (".inl", TextFormat::Cpp),
    (".json", TextFormat::Json),
    (".js", TextFormat::Js),
    (".kfa", TextFormat::Xml),
    (".kfe", TextFormat::Xml),
    (".kfl", TextFormat::Xml),
    (".kfsl", TextFormat::Xml),
    (".kfp", TextFormat::Xml),
    (".kfcs", TextFormat::Xml),
    (".lighting", TextFormat::Xml),
    (".lua", TextFormat::Lua),
    (".tai", TextFormat::Plain),
    (".technique", TextFormat::Xml),
    (".texture_array", TextFormat::Plain),
    (".tsv", TextFormat::Plain),
    (".twui", TextFormat::Lua),
    (".txt", TextFormat::Plain),
    (".variantmeshdefinition", TextFormat::Xml),
    (".wsmodel", TextFormat::Xml),
    (".xml", TextFormat::Xml),
    (".xml.shader", TextFormat::Xml),
    (".xml.material", TextFormat::Xml),
    (".material", TextFormat::Xml),     // This has to be under xml.material
];

#[cfg(test)] mod text_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `Text` file decoded in memory.
#[derive(Default, PartialEq, Eq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Text {

    /// The encoding used by the file.
    encoding: Encoding,

    /// The format of the file.
    format: TextFormat,

    /// The text inside the file.
    contents: String
}

/// This enum represents the multiple encodings we can read/write to.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Encoding {
    Iso8859_1,
    Utf8,
    Utf8Bom,
    Utf16Le,
}

/// This enum represents the formats we know.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TextFormat {
    Cpp,
    Html,
    Json,
    Js,
    Css,
    Lua,
    Markdown,
    Plain,
    Xml,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `Default` for `Encoding`.
impl Default for Encoding {

    /// This returns `Encoding::Utf8`, as it's our default encoding.
    fn default() -> Self {
        Encoding::Utf8
    }
}

/// Implementation of `Default` for `TextFormat`.
impl Default for TextFormat {

    /// This returns `TextFormat::Plain`, as it's our default format.
    fn default() -> Self {
        TextFormat::Plain
    }
}

impl Text {

    pub fn detect_encoding<R: ReadBytes>(data: &mut R) -> Result<Encoding> {
        let len = data.len()?;

        // First, check for BOMs. 2 bytes for UTF-16 BOMs, 3 for UTF-8.
        if len > 2 && data.read_slice(3, true)? == BOM_UTF_8 {
            data.seek(SeekFrom::Start(3))?;
            return Ok(Encoding::Utf8Bom)
        }
        else if len > 1 && data.read_slice(2, true)? == BOM_UTF_16_LE {
            data.seek(SeekFrom::Start(2))?;
            return Ok(Encoding::Utf16Le)
        }

        // If no BOM is found, we assume UTF-8 if it decodes properly.
        else {
            let utf8_string = data.read_string_u8(len as usize);
            if utf8_string.is_ok() {
                data.seek(SeekFrom::Start(0))?;
                return Ok(Encoding::Utf8)
            }

            data.seek(SeekFrom::Start(0))?;
            let iso_8859_1_string = data.read_string_u8_iso_8859_15(len as usize);
            if iso_8859_1_string.is_ok() {
                data.seek(SeekFrom::Start(0))?;
                return Ok(Encoding::Iso8859_1)
            }
        }

        // If we reach this, we do not support the format.
        data.seek(SeekFrom::Start(0))?;
        Err(RLibError::DecodingTextUnsupportedEncodingOrNotATextFile)
    }
}

impl Decodeable for Text {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let len = data.len()?;
        let encoding = Self::detect_encoding(data)?;
        let contents = match encoding {
            Encoding::Iso8859_1 => data.read_string_u8_iso_8859_15(len as usize)
                .map_err(|_| RLibError::DecodingTextUnsupportedEncodingOrNotATextFile)?,

            Encoding::Utf8 |
            Encoding::Utf8Bom => {
                let curr_pos = data.stream_position()?;
                data.read_string_u8((len - curr_pos) as usize)
                    .map_err(|_| RLibError::DecodingTextUnsupportedEncodingOrNotATextFile)?
            },
            Encoding::Utf16Le => {
                let curr_pos = data.stream_position()?;
                data.read_string_u16((len - curr_pos) as usize)
                    .map_err(|_| RLibError::DecodingTextUnsupportedEncodingOrNotATextFile)?
            }
        };

        // Try to get the format of the file.
        let format = match extra_data {
            Some(extra_data) => match extra_data.file_name {
                Some(file_name) => {
                    match EXTENSIONS.iter().find_map(|(extension, format)| if file_name.ends_with(extension) { Some(format) } else { None }) {
                        Some(format) => *format,
                        None => TextFormat::Plain,
                    }
                }
                None => TextFormat::Plain,
            }

            None => TextFormat::Plain,
        };

        Ok(Self {
            encoding,
            format,
            contents,
        })
    }
}

impl Encodeable for Text {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        match self.encoding {
            Encoding::Iso8859_1 => buffer.write_string_u8_iso_8859_1(&self.contents),
            Encoding::Utf8 => buffer.write_string_u8(&self.contents),
            Encoding::Utf8Bom => {
                buffer.write_all(&BOM_UTF_8)?;
                buffer.write_string_u8(&self.contents)
            },

            // For UTF-16 we always have to add the BOM. Otherwise we have no way to easily tell what this file is.
            Encoding::Utf16Le => {
                buffer.write_all(&BOM_UTF_16_LE)?;
                buffer.write_string_u16(&self.contents)
            },
        }
    }
}
