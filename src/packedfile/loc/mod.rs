// In this file we define the PackedFile type Loc for decoding and encoding it.
// This is the type used by localisation files.
extern crate csv;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use self::csv::{ ReaderBuilder, WriterBuilder, QuoteStyle };

use common::coding_helpers::*;
use error::{Error, ErrorKind, Result};
use super::SerializableToTSV;

/// `Loc`: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile divided in 2 parts:
/// - header: header of the PackedFile, decoded.
/// - data: data of the PackedFile, decoded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loc {
    pub header: LocHeader,
    pub data: LocData,
}

/// `LocHeader`: This stores the header of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - byte_order_mark: an u16 (2 bytes) that marks the beginning of the PackedFile (FF FE).
/// - packed_file_type: LOC (3 bytes) in our case. After this it should be a 0 byte.
/// - packed_file_version: if this is not 1, the file is invalid, don't know why.
/// - packed_file_entry_count: amount of entries in the file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocHeader {
    pub byte_order_mark: u16,
    pub packed_file_type: String,
    pub packed_file_version: u32,
    pub packed_file_entry_count: u32,
}

/// `LocData`: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's data in a Vec<LocEntry>.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocData {
    pub entries: Vec<LocEntry>,
}

/// `LocEntry`: This stores an entry of a decoded Localisation PackedFile in memory.
/// It stores the entry's data in multiple parts:
/// - key: the "key" column of the entry.
/// - text: the text you'll see ingame.
/// - tooltip (bool): this one I believe it was to enable or disable certain lines ingame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocEntry {
    pub key: String,
    pub text: String,
    pub tooltip: bool,
}

/// Implementation of "Loc".
impl Loc {

    /// This function creates a new empty Loc PackedFile.
    pub fn new() -> Self {
        Self{
            header: LocHeader::new(),
            data: LocData::new(),
        }
    }

    /// This function creates a new decoded Loc from the data of a PackedFile. Note that this assume
    /// the file is a loc. It'll crash otherwise.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {
        match LocHeader::read(&packed_file_data[..14]) {
            Ok(header) => {
                match LocData::read(&packed_file_data[14..], &header) {
                    Ok(data) =>
                        Ok(Self {
                            header,
                            data,
                        }),
                    Err(error) => Err(error)
                }
            }
            Err(error) => Err(error)
        }
    }

    /// This function takes a LocHeader and a LocData and put them together in a Vec<u8>, encoding an
    /// entire LocFile ready to write on disk.
    pub fn save(&self) -> Vec<u8> {

        // Encode the header.
        let mut packed_file = LocHeader::save(&self.header, self.data.entries.len() as u32);

        // Add the data to the encoded header.
        LocData::save(&self.data, &mut packed_file);

        // Return the encoded PackedFile.
        packed_file
    }
}

/// Implementation of "LocHeader".
impl LocHeader {

    /// This function creates a new empty LocHeader.
    pub fn new() -> Self {
        Self {
            byte_order_mark: 65279,
            packed_file_type: "LOC".to_owned(),
            packed_file_version: 1,
            packed_file_entry_count: 0,
        }
    }

    /// This function creates a new decoded LocHeader from the data of a PackedFile. To see what are
    /// these values, check the LocHeader struct.
    pub fn read(packed_file_header: &[u8]) -> Result<Self> {

        // Create a new header.
        let mut header = Self::new();

        // Get the values of the file.
        header.byte_order_mark = decode_integer_u16(&packed_file_header[0..2])?;
        header.packed_file_type = decode_string_u8(&packed_file_header[2..5])?;
        header.packed_file_version = decode_integer_u32(&packed_file_header[6..10])?;
        header.packed_file_entry_count = decode_integer_u32(&packed_file_header[10..14])?;

        // Return the new header.
        Ok(header)
    }

    /// This function takes a LocHeader and an entry count and creates a Vec<u8> encoded version of
    /// the LocHeader, ready to write it on disk.
    pub fn save(&self, entry_count: u32) -> Vec<u8> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.extend_from_slice(&encode_integer_u16(self.byte_order_mark));
        packed_file.extend_from_slice(&encode_string_u8(&self.packed_file_type));
        packed_file.push(0);
        packed_file.extend_from_slice(&encode_integer_u32(self.packed_file_version));
        packed_file.extend_from_slice(&encode_integer_u32(entry_count));

        // Return the encoded PackedFile.
        packed_file
    }
}

/// Implementation of "LocData".
impl LocData {

    /// This function returns an empty LocData.
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// This function creates a new decoded LocData from the data of a PackedFile. This pass through
    /// all the data of the Loc PackedFile and decodes every entry.
    pub fn read(packed_file_data: &[u8], header: &LocHeader) -> Result<Self> {

        // Create a new list of entries.
        let mut entries = vec![];

        // Create the index to move through the file.
        let mut index = 0 as usize;

        // For each entry we have...
        for _ in 0..header.packed_file_entry_count {

            // Decode the three fields.
            let key = decode_packedfile_string_u16(&packed_file_data[index..], &mut index)?;
            let text = decode_packedfile_string_u16(&packed_file_data[index..], &mut index)?;
            let tooltip = decode_packedfile_bool(packed_file_data[index], &mut index)?;

            // Create our new entry.
            let mut entry = LocEntry::new(key, text, tooltip);

            // Add the entry to the list.
            entries.push(entry);
        }

        // Return the entries.
        Ok(Self { entries })
    }

    /// This function takes an entire LocData and encode it to Vec<u8> to write it on disk. Also, it
    /// returns his entry count for the header.
    pub fn save(&self, packed_file: &mut Vec<u8>) {
        for entry in &self.entries {
            packed_file.append(&mut encode_packedfile_string_u16(&entry.key));
            packed_file.append(&mut encode_packedfile_string_u16(&entry.text));
            packed_file.push(encode_bool(entry.tooltip));
        }
    }
}

/// Implementation of "LocDataEntry"
impl LocEntry {

    /// This function takes the key, text and tooltip values and makes a LocDataEntry with them.
    pub fn new(key: String, text: String, tooltip: bool) -> Self {
        Self {
            key,
            text,
            tooltip,
        }
    }
}

/// Implementation of `SerializableToTSV` for `LocData`.
impl SerializableToTSV for LocData {

    /// This function imports a TSV file and loads his contents into a Loc PackedFile.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<()> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        match ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(true)
            .from_path(&tsv_file_path) {

            // If we succesfully read the TSV file into a reader...
            Ok(mut reader) => {

                // We create here the vector to store the date while it's being decoded.
                let mut packed_file_data = vec![];

                // We use the headers to make sure this TSV file belongs to a Loc PackedFile.
                match reader.headers() {
                    Ok(header) => {

                        // Get the type and number of his original PackedFile.
                        let tsv_type = header.get(0).unwrap_or("error");
                        let its_over_9000 = header.get(1).unwrap_or("8999").parse::<u32>().unwrap_or(8999);

                        // If it's not of type "Loc PackedFile" or not over 9000, it's not Goku.
                        if tsv_type != packed_file_type || its_over_9000 != 9001 {
                            return Err(ErrorKind::ImportTSVWrongTypeLoc)?;
                        }
                    }

                    // If it fails, return error.
                    Err(_) => return Err(ErrorKind::ImportTSVIncorrectFirstRow)?,
                }

                // Then we add the new entries to the decoded entry list, or return error if any of the entries is invalid.
                for (index, reader_entry) in reader.deserialize().enumerate() {

                    // We skip the first line (header).
                    if index > 0 {
                        match reader_entry {
                            Ok(entry) => packed_file_data.push(entry),
                            Err(error) => return Err(Error::from(error))
                        }
                    }
                }

                // If we reached this point without errors, we replace the old data with the new one.
                self.entries = packed_file_data;

                // Return success.
                Ok(())
            }

            // If we couldn't read the TSV file, return error.
            Err(error) => Err(Error::from(error))
        }
    }

    /// This function creates a TSV file with the contents of a Loc PackedFile.
    fn export_tsv(&self, packed_file_path: &PathBuf, extra_info: (&str, u32)) -> Result<String> {

        // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        let mut writer = WriterBuilder::new()
            .delimiter(b'\t')
            .quote_style(QuoteStyle::Never)
            .has_headers(false)
            .flexible(true)
            .from_writer(vec![]);

        // We serialize the extra info provided, so we can check it when importing('cause why not?).
        writer.serialize(extra_info)?;

        // For every entry, we serialize every one of his fields (except the index).
        for entry in &self.entries {

            // We don't want the index, as that's not really needed outside the program.
            writer.serialize(entry)?;
        }

        // Then, we try to write it on disk. If there is an error, report it.
        match File::create(&packed_file_path) {
            Ok(mut file) => {
                match file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes()) {
                    Ok(_) => Ok(format!("<p>Loc PackedFile successfully exported:</p><ul><li>{}</li></ul>", packed_file_path.display())),
                    Err(_) => Err(ErrorKind::IOGenericWrite(vec![packed_file_path.display().to_string();1]))?
                }
            }
            Err(_) => Err(ErrorKind::IOGenericWrite(vec![packed_file_path.display().to_string();1]))?
        }
    }
}
