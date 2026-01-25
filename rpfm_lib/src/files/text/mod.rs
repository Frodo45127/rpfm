//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Plain text file handling with encoding detection and format recognition.
//!
//! This module provides the [`Text`] type for working with plain text files in Total War
//! PackFiles. It supports multiple encodings and automatically detects file formats based
//! on extensions to enable syntax highlighting and validation in editors.
//!
//! # Supported Encodings
//!
//! - **ISO-8859-15**: Western European character set (legacy support)
//! - **UTF-8**: Modern Unicode encoding (default)
//! - **UTF-8 with BOM**: UTF-8 with Byte Order Mark
//! - **UTF-16 LE**: UTF-16 Little Endian with BOM
//!
//! Encoding is automatically detected by examining Byte Order Marks (BOMs) and attempting
//! to decode the data. When encoding, the original encoding is preserved.
//!
//! # Format Detection
//!
//! The module automatically detects file formats based on file extensions, enabling appropriate
//! syntax highlighting and validation. Supported formats include Lua scripts, XML configuration,
//! JSON data, shader code, and more.
//!
//! # Supported File Extensions
//!
//! The following table lists all file extensions recognized as text files:
//!
//! | ----------------------------- | ---------- | ------------------------------------------- |
//! | Extension                     | Format     | Description                                 |
//! | ----------------------------- | ---------- | ------------------------------------------- |
//! | `.agf`                        | `Plain`    |                                             |
//! | `.bat`                        | `Bat`      | Windows batch script.                       |
//! | `.battle_script`              | `Lua`      | Battle script in Lua.                       |
//! | `.battle_speech_camera`       | `Plain`    | Camera settings for battle speeches.        |
//! | `.benchmark`                  | `Xml`      | Benchmark settings.                         |
//! | `.bob`                        | `Plain`    | BoB settings file.                          |
//! | `.cco`                        | `Plain`    |                                             |
//! | `.cindyscene`                 | `Xml`      | Cutscene editor data.                       |
//! | `.cindyscenemanager`          | `Xml`      | Cutscene manager data.                      |
//! | `.code-snippets`              | `Json`     | VSCode snippet file.                        |
//! | `.code-workspace`             | `Json`     | VSCode workspace file.                      |
//! | `.css`                        | `Css`      | CSS stylesheet.                             |
//! | `.csv`                        | `Plain`    | Comma-separated values file.                |
//! | `.environment`                | `Xml`      | Environment settings.                       |
//! | `.environment_group`          | `Xml`      | Environment group settings.                 |
//! | `.environment_group.override` | `Xml`      | Environment group overrides.                |
//! | `.fbx`                        | `Plain`    | Autodesk FBX (text format).                 |
//! | `.fx`                         | `Cpp`      | DirectX effect file.                        |
//! | `.fx_fragment`                | `Cpp`      | DirectX effect fragment.                    |
//! | `.glsl`                       | `Cpp`      | OpenGL shader source.                       |
//! | `.h`                          | `Cpp`      | C/C++ header file.                          |
//! | `.hlsl`                       | `Hlsl`     | High Level Shading Language.                |
//! | `.htm`                        | `Html`     | HTML document.                              |
//! | `.html`                       | `Html`     | HTML document.                              |
//! | `.inl`                        | `Cpp`      | C++ inline file.                            |
//! | `.json`                       | `Json`     | JSON data file.                             |
//! | `.js`                         | `Js`       | JavaScript file.                            |
//! | `.kfa`                        | `Xml`      | Battle Audio Event file.                    |
//! | `.kfc`                        | `Xml`      | Battle Camera file.                         |
//! | `.kfe`                        | `Xml`      | Battle Effect file.                         |
//! | `.kfe_temp`                   | `Xml`      | Battle Effect (temporary).                  |
//! | `.kfl`                        | `Xml`      | Battle Point Light file.                    |
//! | `.kfl_temp`                   | `Xml`      | Battle Point Light (temporary).             |
//! | `.kfsl`                       | `Xml`      | Battle Spot Light file.                     |
//! | `.kfp`                        | `Xml`      | Battle Prop file.                           |
//! | `.kfcs`                       | `Xml`      | Battle Composite Scene file.                |
//! | `.kfcs_temp`                  | `Xml`      | Battle Composite Scene (temporary).         |
//! | `.ktr`                        | `Xml`      | Battle Tracker file.                        |
//! | `.ktr_temp`                   | `Xml`      | Battle Tracker (temporary).                 |
//! | `.lighting`                   | `Xml`      | Lighting configuration.                     |
//! | `.log`                        | `Plain`    | Log file.                                   |
//! | `.lua`                        | `Lua`      | Lua script file.                            |
//! | `.material`                   | `Xml`      | Material definition.                        |
//! | `.md`                         | `Markdown` | Markdown documentation.                     |
//! | `.model_statistics`           | `Xml`      | Model statistics data.                      |
//! | `.mvscene`                    | `Xml`      | Movie scene file.                           |
//! | `.py`                         | `Python`   | Python script.                              |
//! | `.sbs`                        | `Xml`      | Substance Designer file.                    |
//! | `.shader`                     | `Xml`      | Shader definition.                          |
//! | `.sql`                        | `Sql`      | SQL query file.                             |
//! | `.tai`                        | `Plain`    |                                             |
//! | `.technique`                  | `Xml`      | Rendering technique definition.             |
//! | `.texture_array`              | `Plain`    | List of campaign map textures.              |
//! | `.tsv`                        | `Plain`    | Tab-separated values file.                  |
//! | `.twui`                       | `Lua`      | Total War UI file (Lua format).             |
//! | `.txt`                        | `Plain`    | Plain text file.                            |
//! | `.xml`                        | `Xml`      | XML file.                                   |
//! | `.xml_temp`                   | `Xml`      | XML (temporary).                            |
//! | `.xml.shader`                 | `Xml`      | Shader metadata (XML).                      |
//! | `.xml.material`               | `Xml`      | Material metadata (XML).                    |
//! | `.xt`                         | `Plain`    | Text file (typo variant).                   |
//! | `.yml`                        | `Yaml`     | YAML configuration file.                    |
//! | `.yaml`                       | `Yaml`     | YAML configuration file.                    |
//!
//! Note: `.variantmeshdefinition` and `.wsmodel` are also supported but listed separately in the code.

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
pub const EXTENSIONS: [(&str, TextFormat); 63] = [
    (".agf", TextFormat::Plain),
    (".bat", TextFormat::Bat),
    (".battle_script", TextFormat::Lua),
    (".battle_speech_camera", TextFormat::Plain),
    (".benchmark", TextFormat::Xml),
    (".bob", TextFormat::Plain),
    (".cco", TextFormat::Plain),
    (".cindyscene", TextFormat::Xml),
    (".cindyscenemanager", TextFormat::Xml),
    (".code-snippets", TextFormat::Json),
    (".code-workspace", TextFormat::Json),
    (".css", TextFormat::Css),
    (".csv", TextFormat::Plain),
    (".environment", TextFormat::Xml),
    (".environment_group", TextFormat::Xml),
    (".environment_group.override", TextFormat::Xml),
    (".fbx", TextFormat::Plain),
    (".fx", TextFormat::Cpp),
    (".fx_fragment", TextFormat::Cpp),
    (".glsl", TextFormat::Cpp),
    (".h", TextFormat::Cpp),
    (".hlsl", TextFormat::Hlsl),
    (".htm", TextFormat::Html),
    (".html", TextFormat::Html),
    (".inl", TextFormat::Cpp),
    (".json", TextFormat::Json),
    (".js", TextFormat::Js),
    (".kfa", TextFormat::Xml),
    (".kfc", TextFormat::Xml),
    (".kfe", TextFormat::Xml),
    (".kfe_temp", TextFormat::Xml),
    (".kfl", TextFormat::Xml),
    (".kfl_temp", TextFormat::Xml),
    (".kfsl", TextFormat::Xml),
    (".kfp", TextFormat::Xml),
    (".kfcs", TextFormat::Xml),
    (".kfcs_temp", TextFormat::Xml),
    (".ktr", TextFormat::Xml),
    (".ktr_temp", TextFormat::Xml),
    (".lighting", TextFormat::Xml),
    (".log", TextFormat::Plain),
    (".lua", TextFormat::Lua),
    (".md", TextFormat::Markdown),
    (".model_statistics", TextFormat::Xml),
    (".mvscene", TextFormat::Xml),
    (".py", TextFormat::Python),
    (".sbs", TextFormat::Xml),
    (".shader", TextFormat::Xml),
    (".sql", TextFormat::Sql),
    (".tai", TextFormat::Plain),
    (".technique", TextFormat::Xml),
    (".texture_array", TextFormat::Plain),
    (".tsv", TextFormat::Plain),
    (".twui", TextFormat::Lua),
    (".txt", TextFormat::Plain),
    (".xml", TextFormat::Xml),
    (".xml_temp", TextFormat::Xml),
    (".xml.shader", TextFormat::Xml),
    (".xml.material", TextFormat::Xml),
    (".xt", TextFormat::Plain),
    (".yml", TextFormat::Yaml),
    (".yaml", TextFormat::Yaml),
    (".material", TextFormat::Xml),     // This has to be under xml.material
];

/// Extension for VMD, or Variant Mesh Definitions.
pub const EXTENSION_VMD: (&str, TextFormat) = (".variantmeshdefinition", TextFormat::Xml);

/// Extension for WS Models.
pub const EXTENSION_WSMODEL: (&str, TextFormat) = (".wsmodel", TextFormat::Xml);

#[cfg(test)] mod text_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// In-memory representation of a decoded text file.
///
/// Stores the text contents along with encoding and format metadata. The encoding
/// is preserved when re-encoding to maintain file compatibility.
///
/// # Fields
///
/// * `encoding` - Character encoding detected or specified for the file
/// * `format` - File format detected from extension (for syntax highlighting)
/// * `contents` - Decoded text contents as a UTF-8 Rust string
///
/// # Getters/Setters
///
/// All fields have public getters, mutable getters, and setters via the `getset` crate:
/// - `encoding()`, `encoding_mut()`, `set_encoding()`
/// - `format()`, `format_mut()`, `set_format()`
/// - `contents()`, `contents_mut()`, `set_contents()`
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::{Decodeable, text::Text, DecodeableExtraData};
/// use std::io::Cursor;
///
/// let data = b"Hello, World!";
/// let mut reader = Cursor::new(data);
/// let text = Text::decode(&mut reader, &None).unwrap();
///
/// assert_eq!(text.contents(), "Hello, World!");
/// ```
#[derive(Default, PartialEq, Eq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Text {

    /// Character encoding of the file.
    encoding: Encoding,

    /// Detected file format based on extension.
    format: TextFormat,

    /// Decoded text contents.
    contents: String
}

/// Character encoding types supported for text files.
///
/// Different Total War games and file types use different encodings. This enum
/// represents all encodings that rpfm_lib can read and write.
///
/// # Encoding Detection
///
/// Encodings are detected in the following order:
/// 1. Check for UTF-8 BOM (`0xEF 0xBB 0xBF`)
/// 2. Check for UTF-16 LE BOM (`0xFF 0xFE`)
/// 3. Attempt UTF-8 decode without BOM
/// 4. Attempt ISO-8859-1 decode
///
/// # Re-encoding
///
/// When a text file is saved, the original encoding is preserved to maintain
/// compatibility with the game engine.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Encoding {
    /// ISO-8859-1 encoding (Western European, legacy support).
    Iso8859_1,

    /// UTF-8 encoding without BOM (default for new files).
    Utf8,

    /// UTF-8 encoding with BOM marker.
    Utf8Bom,

    /// UTF-16 Little Endian encoding with BOM marker.
    Utf16Le,
}

/// File format types for syntax highlighting and validation.
///
/// Based on file extension, text files are classified into different formats.
/// This allows text editors to apply appropriate syntax highlighting, code
/// completion, and validation rules.
///
/// # Format Detection
///
/// Format is determined by matching the file extension against the [`EXTENSIONS`]
/// table. If no match is found, defaults to [`TextFormat::Plain`].
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TextFormat {
    /// Windows batch script (`.bat`).
    Bat,

    /// C++ code or GLSL shaders (`.cpp`, `.h`, `.glsl`, `.inl`, `.fx`).
    Cpp,

    /// HTML documents (`.html`, `.htm`).
    Html,

    /// HLSL shader code (`.hlsl`).
    Hlsl,

    /// JSON data files (`.json`, `.code-snippets`, `.code-workspace`).
    Json,

    /// JavaScript code (`.js`).
    Js,

    /// CSS stylesheets (`.css`).
    Css,

    /// Lua scripts (`.lua`, `.twui`, `.battle_script`).
    Lua,

    /// Markdown documentation (`.md`).
    Markdown,

    /// Plain text with no specific format (`.txt`, `.csv`, `.tsv`, `.log`, etc.).
    Plain,

    /// Python scripts (`.py`).
    Python,

    /// SQL queries (`.sql`).
    Sql,

    /// XML configuration and data files (`.xml`, `.kf*`, `.cindyscene`, etc.).
    Xml,

    /// YAML configuration files (`.yaml`, `.yml`).
    Yaml,
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

    /// Detects the character encoding of text data.
    ///
    /// Examines the data stream to determine its encoding by checking for Byte Order Marks
    /// (BOMs) and attempting to decode as different encodings.
    ///
    /// # Detection Algorithm
    ///
    /// 1. **UTF-8 BOM**: Checks for `0xEF 0xBB 0xBF` at the start
    /// 2. **UTF-16 LE BOM**: Checks for `0xFF 0xFE` at the start
    /// 3. **UTF-8 without BOM**: Attempts to decode entire file as UTF-8
    /// 4. **ISO-8859-1**: Attempts to decode as ISO-8859-1
    ///
    /// # Arguments
    ///
    /// * `data` - Reader positioned at the start of the text data
    ///
    /// # Returns
    ///
    /// The detected [`Encoding`], or an error if no supported encoding matches.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingTextUnsupportedEncodingOrNotATextFile`] if:
    /// - The data cannot be decoded as any supported encoding
    /// - The file is not actually a text file
    ///
    /// # Side Effects
    ///
    /// After detection, the reader is repositioned:
    /// - After the BOM if one was found
    /// - At the start if no BOM was found
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
                data.rewind()?;
                return Ok(Encoding::Utf8)
            }

            data.rewind()?;
            let iso_8859_1_string = data.read_string_u8_iso_8859_15(len as usize);
            if iso_8859_1_string.is_ok() {
                data.rewind()?;
                return Ok(Encoding::Iso8859_1)
            }
        }

        // If we reach this, we do not support the format.
        data.rewind()?;
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
