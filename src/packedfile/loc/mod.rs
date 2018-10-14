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

/// This const represents the value that every LOC PackedFile has in their first 2 bytes.
const BYTEORDER_MARK: u16 = 65279; // FF FE

/// This const represents the value that every LOC PackedFile has in their 2-5 bytes.
const PACKED_FILE_TYPE: &str = "LOC";

/// This const represents the value that every LOC PackedFile has in their 6-10 bytes.
const PACKED_FILE_VERSION: u32 = 1;

/// `Loc`: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile's data in a Vec<LocEntry>.
#[derive(Clone, Debug)]
pub struct Loc {
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
        Self { entries: vec![] }
    }

    /// This function creates a new decoded Loc from the data of a PackedFile.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {

        // A valid Loc PackedFile has at least 14 bytes. This ensures they exists before anything else.
        if packed_file_data.len() < 14 { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }

        // More checks to ensure this is a valid Loc PAckedFile.
        if BYTEORDER_MARK != decode_integer_u16(&packed_file_data[0..2])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_TYPE != decode_string_u8(&packed_file_data[2..5])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        if PACKED_FILE_VERSION != decode_integer_u32(&packed_file_data[6..10])? { return Err(ErrorKind::LocPackedFileIsNotALocPackedFile)? }
        let entry_count = decode_integer_u32(&packed_file_data[10..14])?;

        // Get all the entries and return the Loc.
        let mut entries = vec![];
        let mut index = 14 as usize;
        for _ in 0..entry_count {

            // Decode the three fields.
            let key = if index < packed_file_data.len() { decode_packedfile_string_u16(&packed_file_data[index..], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            let text = if index < packed_file_data.len() { decode_packedfile_string_u16(&packed_file_data[index..], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            let tooltip = if index < packed_file_data.len() { decode_packedfile_bool(packed_file_data[index], &mut index)? } else { return Err(ErrorKind::LocPackedFileCorrupted)? };
            let mut entry = LocEntry::new(key, text, tooltip);
            entries.push(entry);
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        if index != packed_file_data.len() { return Err(ErrorKind::PackedFileSizeIsNotWhatWeExpect(packed_file_data.len(), index))? }

        Ok(Self { entries })

    }

    /// This function takes a LocHeader and a LocData and put them together in a Vec<u8>, encoding an
    /// entire LocFile ready to write on disk.
    pub fn save(&self) -> Vec<u8> {

        // Create the vector to hold them all.
        let mut packed_file: Vec<u8> = vec![];

        // Encode the header.
        packed_file.extend_from_slice(&encode_integer_u16(BYTEORDER_MARK));
        packed_file.extend_from_slice(&encode_string_u8(PACKED_FILE_TYPE));
        packed_file.push(0);
        packed_file.extend_from_slice(&encode_integer_u32(PACKED_FILE_VERSION));
        packed_file.extend_from_slice(&encode_integer_u32(self.entries.len() as u32));

        // Encode the data.
        for entry in &self.entries {
            packed_file.append(&mut encode_packedfile_string_u16(&entry.key));
            packed_file.append(&mut encode_packedfile_string_u16(&entry.text));
            packed_file.push(encode_bool(entry.tooltip));
        }

        // And return the encoded PackedFile.
        packed_file
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
impl SerializableToTSV for Loc {

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
