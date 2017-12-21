// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;

use std::fs::File;
use std::io::{
    Write, Error, ErrorKind
};
use std::path::PathBuf;
use std::error;

use packedfile::loc::LocData;
use packedfile::loc::LocDataEntry;

pub mod loc;
pub mod db;
pub mod rigidmodel;

/*
--------------------------------------------------
          Functions for Loc PackedFiles
--------------------------------------------------
*/

/// Function to export a LocData to a CSV file, without headers and with the fields quoted.
/// It requires:
/// - packed_file_data_to_export: the LocData we are going to export.
/// - packed_file_path: the destination path of the CSV.
pub fn export_to_csv(
    packed_file_data: &LocData,
    packed_file_path: PathBuf
) -> Result<String, Error> {

    // We want no headers and quotes around the fields, so we need to tweak our writer first.
    let mut writer_builder = csv::WriterBuilder::new();
    writer_builder.has_headers(false);
    writer_builder.quote_style(csv::QuoteStyle::Always);

    let mut writer = writer_builder.from_writer(vec![]);

    for i in packed_file_data.packed_file_data_entries.clone() {
        writer.serialize(LocDataEntry {
            key: i.key,
            text: i.text,
            tooltip: i.tooltip,
        })?;
    }

    let csv_serialized = String::from_utf8(writer.into_inner().unwrap().to_vec()).unwrap();

    match File::create(&packed_file_path) {
        Ok(mut file) => {
            match file.write_all(&csv_serialized.as_bytes()) {
                Ok(_) => Ok(format!("Loc PackedFile successfully exported:\n{}", packed_file_path.display())),
                Err(error) => Err(Error::new(ErrorKind::Other, format!("Error while writing the following file to disk:\n{}\n\nThe problem reported is:\n{}", packed_file_path.display(), error::Error::description(&error).to_string())))
            }
        }
        Err(error) => Err(Error::new(ErrorKind::Other, format!("Error while trying to write the following file to disk:\n{}\n\nThe problem reported is:\n{}", packed_file_path.display(), error::Error::description(&error).to_string())))
    }
}


/// Function to import a LocData from a CSV file, without headers and with the fields quoted.
/// It requires:
/// - csv_file_path: the CSV we want to import.
/// It returns a Result with the new LocData or an Error, depending on what happened.
pub fn import_from_csv(
    csv_file_path: PathBuf
) -> Result<LocData, Error> {

    let mut packed_file_data_from_tree_view = LocData::new();

    // We expect no headers, so we need to tweak our reader first.
    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);
    match reader_builder.from_path(&csv_file_path) {
        Ok(mut reader) => {
            // Then we add the new entries to the decoded entry list.
            for i in reader.deserialize() {
                match i {
                    Ok(entry) => packed_file_data_from_tree_view.packed_file_data_entries.push(entry),
                    Err(error) => return Err(Error::new(ErrorKind::Other, format!("Error while trying import the csv file:\n{}\n\nThe error reported is:\n{}", &csv_file_path.display(), error::Error::description(&error).to_string())))

                }
            }
            Ok(packed_file_data_from_tree_view)
        }
        Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string()))
    }
}
