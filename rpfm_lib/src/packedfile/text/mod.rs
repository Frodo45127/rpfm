//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with Text PackedFiles.

Text PackedFiles are any kind of plain text packedfile, like lua, xml, txt,...
The only thing to take into account is that this only work for UTF-8 encoded files.
!*/

use serde_derive::{Serialize, Deserialize};

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};

/// UTF-8 BOM (Byte Order Mark).
const BOM_UTF_8: [u8;3] = [0xEF,0xBB,0xBF];

// UTF-16 BOM (Byte Order Mark), Big Endian.
//const BOM_UTF_16_BE: [u8;2] = [0xFE,0xFF];

/// UTF-16 BOM (Byte Order Mark), Little Endian.
const BOM_UTF_16_LE: [u8;2] = [0xFF,0xFE];

/// List of extensions for files this lib can decode as Text PackedFiles, with their respective type.
pub const EXTENSIONS: [(&str, TextType); 22] = [
    (".inl", TextType::Cpp),
    (".lua", TextType::Lua),
    (".xml", TextType::Xml),
    (".technique", TextType::Xml),
    (".xml.shader", TextType::Xml),
    (".xml.material", TextType::Xml),
    (".variantmeshdefinition", TextType::Xml),
    (".environment", TextType::Xml),
    (".lighting", TextType::Xml),
    (".wsmodel", TextType::Xml),
    (".benchmark", TextType::Xml),
    (".cindyscene", TextType::Xml),
    (".cindyscenemanager", TextType::Xml),
    (".csv", TextType::Plain),
    (".tsv", TextType::Plain),
    (".tai", TextType::Plain),
    (".battle_speech_camera", TextType::Plain),
    (".bob", TextType::Plain),
    (".txt", TextType::Plain),
    (".htm", TextType::Html),
    (".html", TextType::Html),
    (".json", TextType::Json),
];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire Text PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Text {

    /// The encoding used by the text of the PackedFile.
    encoding: SupportedEncodings,

    /// Type of text this PackedFile has.
    text_type: TextType,

    /// The text inside the PackedFile.
    contents: String
}

/// This enum contains the list of encoding RPFM supports.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SupportedEncodings {
    Utf8,
    //Utf16Be,
    Utf16Le,
    Iso8859_1,
    //Iso8859_15,
}

/// This enum contains the list of text types RPFM supports.
///
/// This is so you can do things depending on the language the text file is written.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TextType {
    Html,
    Xml,
    Lua,
    Cpp,
    Markdown,
    Json,
    Plain,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `Default` for `Text`.
impl Default for Text {
    fn default() -> Self {
        Self {
            encoding: SupportedEncodings::Utf8,
            text_type: TextType::Plain,
            contents: String::new(),
        }
    }
}

/// Implementation of `Text`.
impl Text {

    /// This function creates a new empty `Text`. Akin to `default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// This function creates a `Text` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {

        // First, check for BOMs. 2 bytes for UTF-16 BOMs, 3 for UTF-8. If no BOM is found, we assume UTF-8 or ISO5589-1.
        let (packed_file_data, guessed_encoding) = if packed_file_data.is_empty() { (packed_file_data, SupportedEncodings::Utf8) }
        else if packed_file_data.len() > 2 && packed_file_data[0..3] == BOM_UTF_8 { (&packed_file_data[3..], SupportedEncodings::Utf8) }
        //else if packed_file_data.len() > 1 && packed_file_data[0..2] == BOM_UTF_16_BE { (&packed_file_data[2..], SupportedEncodings::UTF16_BE) }
        else if packed_file_data.len() > 1 && packed_file_data[0..2] == BOM_UTF_16_LE { (&packed_file_data[2..], SupportedEncodings::Utf16Le) }
        else { (packed_file_data, SupportedEncodings::Utf8) };

        // This is simple: we try to decode it depending on what the guesser gave us. If all fails, return error.
        let (encoding, contents) = match guessed_encoding {
            SupportedEncodings::Utf8 | SupportedEncodings::Iso8859_1 => {
                match packed_file_data.decode_string_u8(0, packed_file_data.len()) {
                    Ok(string) => (SupportedEncodings::Utf8, string),
                    Err(_) => match packed_file_data.decode_string_u8_iso_8859_1(0, packed_file_data.len()) {
                        Ok(string) => (SupportedEncodings::Iso8859_1, string),
                        Err(_) => return Err(ErrorKind::TextDecodeWrongEncodingOrNotATextFile.into()),
                    }
                }
            }

            SupportedEncodings::Utf16Le => {
                match packed_file_data.decode_string_u16(0, packed_file_data.len()) {
                    Ok(string) => (SupportedEncodings::Utf16Le, string),
                    Err(_) => return Err(ErrorKind::TextDecodeWrongEncodingOrNotATextFile.into()),
                }
            }
        };

        // Without the path we can't know the text type, so we left it as plain, and overwrite it later.
        let text_type = TextType::Plain;

        Ok(Self {
            encoding,
            text_type,
            contents,
        })
    }

    /// This function takes a `Text` and encodes it to `Vec<u8>`.
    ///
    /// TODO: Make this save other than UTF-8.
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut data = vec![];
        match self.encoding {
            SupportedEncodings::Utf8 => data.encode_string_u8(&self.contents),
            SupportedEncodings::Iso8859_1 => data.encode_string_u8_iso_8859_1(&self.contents),

            // For UTF-16 we always have to add the BOM. Otherwise we have no way to easely tell what this file is.
            SupportedEncodings::Utf16Le => {
                data.append(&mut BOM_UTF_16_LE.to_vec());
                data.encode_string_u16(&self.contents)
            },
        }

        Ok(data)
    }

    /// This function returns the encoding used by the text file.
    pub fn get_encoding(&self) -> SupportedEncodings {
        self.encoding
    }

    /// This function returns a reference to the contents of the text file.
    pub fn get_ref_contents(&self) -> &str {
        &self.contents
    }

    /// This function sets the contents of the text file.
    pub fn set_contents(&mut self, contents: &str) {
        self.contents = contents.to_owned();
    }

    /// This function sets the encoding used to save the text file.
    pub fn set_encoding(&mut self, encoding: SupportedEncodings) {
        self.encoding = encoding;
    }

    /// This function returns a reference to the contents of the text file.
    pub fn get_text_type(&self) -> TextType {
        self.text_type
    }

    /// This function sets the encoding used to save the text file.
    pub fn set_text_type(&mut self, text_type: TextType) {
        self.text_type = text_type;
    }
}
