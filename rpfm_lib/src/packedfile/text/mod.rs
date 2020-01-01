//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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
const BOM: [u8;3] = [0xEF,0xBB,0xBF];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire Text PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Text {

    /// The encoding used by the text of the PackedFile.
    encoding: SupportedEncodings,

    /// The text inside the PackedFile.
    contents: String
}

/// This enum contains the list of encoding RPFM supports.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SupportedEncodings {
    UTF8,
    //UTF16,
    Iso8859_1,
    //Iso8859_15,
}

/// This enum contains the list of text types RPFM supports.
///
/// This is so you can do things depending on the language the text file is written.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TextType {
    Xml,
    Lua,
    Plain,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

/// Implementation of `Default` for `Text`.
impl Default for Text {
    fn default() -> Self {
        Self {
            encoding: SupportedEncodings::UTF8,
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

        // Check for BOM and remove it if it finds it. It's useless in UTF-8, after all.
        let packed_file_data = if packed_file_data.len() > 2 && packed_file_data[0..3] == BOM { &packed_file_data[3..] } else { packed_file_data };

        // This is simple: we try to decode it as an UTF-8 text file. If that doesn't work, we try as 8859-1.
        // If that fails too... fuck you and your abnormal encodings.
        let (encoding, contents) = match packed_file_data.decode_string_u8(0, packed_file_data.len()) {
            Ok(string) => (SupportedEncodings::UTF8, string),
            Err(_) => match packed_file_data.decode_string_u8_iso_8859_1(0, packed_file_data.len()) {
                Ok(string) => (SupportedEncodings::Iso8859_1, string),
                Err(_) => return Err(ErrorKind::TextDecodeWrongEncodingOrNotATextFile.into()),
            }
        };

        Ok(Self {
            encoding,
            contents,
        })
    }

    /// This function takes a `Text` and encodes it to `Vec<u8>`.
    ///
    /// TODO: Make this save other than UTF-8.
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut data = vec![];
        match self.encoding {
            SupportedEncodings::UTF8 => data.encode_string_u8(&self.contents),
            _ => unimplemented!(),
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
}
