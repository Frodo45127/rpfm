// In this file we define the PackedFile type Loc for decoding and encoding it.
// This is the type used by localisation files.
extern crate failure;
extern crate csv;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use self::failure::Error;
use self::csv::{
    ReaderBuilder, WriterBuilder, QuoteStyle
};
use common::coding_helpers;
use super::SerializableToTSV;

/// Struct Loc: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile divided in 2 parts:
/// - packed_file_header: header of the PackedFile, decoded.
/// - packed_file_data: data of the PackedFile, decoded.
#[derive(Clone)]
pub struct Loc {
    pub packed_file_header: LocHeader,
    pub packed_file_data: LocData,
}

/// Struct LocHeader: This stores the header of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's header in different parts:
/// - packed_file_header_byte_order_mark: an u16 (2 bytes) that marks the beginning of the PackedFile (FF FE).
/// - packed_file_header_packed_file_type: LOC (3 bytes) in our case. After this it should be a 0 byte.
/// - packed_file_header_packed_file_version: if this is not 1, the file is invalid, don't know why.
/// - packed_file_header_packed_file_entry_count: amount of entries in the file.
#[derive(Clone)]
pub struct LocHeader {
    pub packed_file_header_byte_order_mark: u16,
    pub packed_file_header_packed_file_type: String,
    pub packed_file_header_packed_file_version: u32,
    pub packed_file_header_packed_file_entry_count: u32,
}

/// Struct LocData: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's data in a Vec<LocDataEntry>.
#[derive(Clone, Debug)]
pub struct LocData {
    pub packed_file_data_entries: Vec<LocDataEntry>,
}

/// Struct LocDataEntry: This stores an entry of a decoded Localisation PackedFile in memory.
/// It stores the entry's data in multiple parts:
/// - key: the "key" column of the entry.
/// - text: the text you'll see ingame.
/// - tooltip (bool): this one I believe it was to enable or disable certain lines ingame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocDataEntry {
    pub key: String,
    pub text: String,
    pub tooltip: bool,
}

/// Implementation of "Loc"
impl Loc {

    /// This function creates a new decoded Loc from the data of a PackedFile. Note that this assume
    /// the file is a loc. It'll crash otherwise.
    pub fn read(packed_file_data: &[u8]) -> Result<Loc, Error> {
        match LocHeader::read(&packed_file_data[..14]) {
            Ok(packed_file_header) => {
                match LocData::read(&packed_file_data[14..], &packed_file_header.packed_file_header_packed_file_entry_count) {
                    Ok(packed_file_data) =>
                        Ok(Loc {
                            packed_file_header,
                            packed_file_data,
                        }),
                    Err(error) => Err(error)
                }
            }
            Err(error) => Err(error)
        }
    }


    /// This function takes a LocHeader and a LocData and put them together in a Vec<u8>, encoding an
    /// entire LocFile ready to write on disk.
    pub fn save(packed_file_decoded: &Loc) -> Vec<u8> {
        let mut packed_file_data_encoded = LocData::save(&packed_file_decoded.packed_file_data);
        let mut packed_file_header_encoded = LocHeader::save(&packed_file_decoded.packed_file_header, packed_file_data_encoded.1);

        let mut packed_file_encoded: Vec<u8> = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded.0);
        packed_file_encoded
    }
}

/// Implementation of "LocHeader"
impl LocHeader {

    /// This function creates a new empty LocHeader.
    pub fn new() -> LocHeader {
        let packed_file_header_byte_order_mark = 0;
        let packed_file_header_packed_file_type = String::new();
        let packed_file_header_packed_file_version = 0;
        let packed_file_header_packed_file_entry_count = 0;

        LocHeader {
            packed_file_header_byte_order_mark,
            packed_file_header_packed_file_type,
            packed_file_header_packed_file_version,
            packed_file_header_packed_file_entry_count,
        }
    }

    /// This function creates a new decoded LocHeader from the data of a PackedFile. To see what are
    /// these values, check the LocHeader struct.
    pub fn read(packed_file_header: &[u8]) -> Result<LocHeader, Error> {
        let mut loc_header = LocHeader::new();

        loc_header.packed_file_header_byte_order_mark = coding_helpers::decode_integer_u16(&packed_file_header[0..2])?;
        loc_header.packed_file_header_packed_file_type = coding_helpers::decode_string_u8(&packed_file_header[2..5])?;
        loc_header.packed_file_header_packed_file_version = coding_helpers::decode_integer_u32(&packed_file_header[6..10])?;
        loc_header.packed_file_header_packed_file_entry_count = coding_helpers::decode_integer_u32(&packed_file_header[10..14])?;

        Ok(loc_header)
    }

    /// This function takes a LocHeader and an entry count and creates a Vec<u8> encoded version of
    /// the LocHeader, ready to write it on disk.
    pub fn save(packed_file_header_decoded: &LocHeader, packed_file_entry_count: u32) -> Vec<u8> {
        let mut packed_file_header_encoded: Vec<u8> = vec![];

        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u16(packed_file_header_decoded.packed_file_header_byte_order_mark));
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_string_u8(&packed_file_header_decoded.packed_file_header_packed_file_type));
        packed_file_header_encoded.push(0);
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u32(packed_file_header_decoded.packed_file_header_packed_file_version));
        packed_file_header_encoded.extend_from_slice(&coding_helpers::encode_integer_u32(packed_file_entry_count));

        packed_file_header_encoded
    }
}

/// Implementation of "LocData"
impl LocData {

    /// This function returns an empty LocData.
    pub fn new() -> LocData {
        let packed_file_data_entries: Vec<LocDataEntry> = vec![];
        LocData {
            packed_file_data_entries,
        }
    }

    /// This function creates a new decoded LocData from the data of a PackedFile. A LocData is a
    /// Vec<LocDataEntry>. This pass through all the data of the Loc PackedFile and decodes every
    /// entry.
    pub fn read(packed_file_data: &[u8], packed_file_entry_count: &u32) -> Result<LocData, Error> {
        let mut packed_file_data_entries: Vec<LocDataEntry> = vec![];

        let mut entry_offset: u32 = 0;
        let mut entry_field_offset: u32;
        let mut entry_size_byte_offset: u32 = 0;
        let mut entry_field: u32 = 0;
        let mut entry_field_size: u16 = 0;

        // For each entry
        for _ in 0..*packed_file_entry_count {

            let mut key: String = String::new();
            let mut text: String = String::new();
            let tooltip: bool;

            loop {

                // The first 2 bytes of a String is the length of the String in reversed utf-16.
                if entry_size_byte_offset == 0 && entry_field < 2 {

                    entry_field_size = coding_helpers::decode_integer_u16(packed_file_data[(entry_offset as usize)..(entry_offset as usize) + 2].into())?;
                    entry_size_byte_offset = 2;
                }
                else {
                    entry_field_offset = 0;
                    match entry_field {

                        // If is the key or the text, we decode it. Remember, the chars are reversed
                        // utf-16 so they use 2 bytes and need to be reversed before using them.
                        0 | 1 => {
                            let string_encoded_begin = (entry_offset + entry_field_offset + entry_size_byte_offset) as usize;
                            let string_encoded_end = (entry_offset + entry_field_offset + entry_size_byte_offset + (u32::from(entry_field_size * 2))) as usize;
                            let string_encoded: Vec<u8> = packed_file_data[string_encoded_begin..string_encoded_end].to_vec();
                            let string_decoded = coding_helpers::decode_string_u16(&string_encoded)?;

                            if entry_field == 0 {
                                key = string_decoded;
                            }
                            else {
                                text = string_decoded;
                            }
                            entry_field_offset += u32::from(entry_field_size * 2);

                            entry_field += 1;
                            entry_offset = entry_offset + entry_size_byte_offset + entry_field_offset;
                            entry_size_byte_offset = 0;
                        }

                        // If it's the boolean, it's a byte, so it doesn't have a size byte offset.
                        _ => {
                            tooltip = coding_helpers::decode_bool(packed_file_data[(entry_offset as usize)])?;
                            packed_file_data_entries.push(LocDataEntry::new(key, text, tooltip));

                            entry_field = 0;
                            entry_offset += 1;
                            break;
                        }
                    }
                }
            }
        }
        Ok(LocData {
            packed_file_data_entries,
        })
    }

    /// This function takes an entire LocData and encode it to Vec<u8> to write it on disk. Also, it
    /// returns his entry count for the header.
    pub fn save(packed_file_data_decoded: &LocData) -> (Vec<u8>, u32) {
        let mut packed_file_data_encoded: Vec<u8> = vec![];
        let mut packed_file_entry_count = 0;

        for i in &packed_file_data_decoded.packed_file_data_entries {
            packed_file_data_encoded.append(&mut coding_helpers::encode_packedfile_string_u16(&i.key));
            packed_file_data_encoded.append(&mut coding_helpers::encode_packedfile_string_u16(&i.text));
            packed_file_data_encoded.push(coding_helpers::encode_bool(i.tooltip));
            packed_file_entry_count += 1;
        }
        (packed_file_data_encoded, packed_file_entry_count)
    }
}

/// Implementation of "LocDataEntry"
impl LocDataEntry {

    /// This function takes the key, text and tooltip values and makes a LocDataEntry with them.
    pub fn new(key: String, text: String, tooltip: bool) -> LocDataEntry {
        LocDataEntry {
            key,
            text,
            tooltip,
        }
    }
}

/// Implementation of `SerializableToTSV` for `LocData`.
impl SerializableToTSV for LocData {

    /// This function imports a TSV file and loads his contents into a Loc PackedFile.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<(), Error> {

        // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
        // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
        match ReaderBuilder::new()
            .delimiter(b'\t')
            .quoting(false)
            .has_headers(false)
            .flexible(false)
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
                            return Err(format_err!("This TSV file it's not from a Loc PackedFile."));
                        }
                    }

                    // If it fails, return error.
                    Err(_) => return Err(format_err!("This TSV file's doesn't have a header.")),
                }

                // Then we add the new entries to the decoded entry list, or return error if any of the entries is invalid.
                for (index, reader_entry) in reader.deserialize().enumerate() {

                    // We skip the first line (header).
                    if index > 0 {
                        match reader_entry {
                            Ok(entry) => packed_file_data.push(entry),
                            Err(_) => return Err(format_err!("Error while trying import the TSV file:\n{}", &tsv_file_path.display()))
                        }
                    }
                }

                // If we reached this point without errors, we replace the old data with the new one.
                self.packed_file_data_entries = packed_file_data;

                // Return success.
                Ok(())
            }

            // If we couldn't read the TSV file, return error.
            Err(_) => Err(format_err!("Error while trying to read the TSV file:\n{}.", &tsv_file_path.display()))
        }
    }

    /// This function creates a TSV file with the contents of a Loc PackedFile.
    fn export_tsv(&self, packed_file_path: &PathBuf, extra_info: (&str, u32)) -> Result<String, Error> {

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
        for entry in &self.packed_file_data_entries {

            // We don't want the index, as that's not really needed outside the program.
            writer.serialize(entry)?;
        }

        // Then, we try to write it on disk. If there is an error, report it.
        match File::create(&packed_file_path) {
            Ok(mut file) => {
                match file.write_all(String::from_utf8(writer.into_inner()?)?.as_bytes()) {
                    Ok(_) => Ok(format!("DB PackedFile successfully exported:\n{}", packed_file_path.display())),
                    Err(_) => Err(format_err!("Error while writing the following file to disk:\n{}", packed_file_path.display()))
                }
            }
            Err(_) => Err(format_err!("Error while trying to write the following file to disk:\n{}", packed_file_path.display()))
        }
    }
}
