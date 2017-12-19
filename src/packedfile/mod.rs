// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;

use packedfile::loc::LocData;
use packedfile::loc::LocDataEntry;

pub mod loc;
pub mod db;
pub mod maps;

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
) -> Result<String, String> {

    let result: Result<String, String>;

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
        });
    }
    let csv_serialized = String::from_utf8(writer.into_inner().unwrap().to_vec()).unwrap();

    match File::create(&packed_file_path) {
        Ok(mut file) => {
            match file.write_all(&csv_serialized.as_bytes()) {
                Ok(_) => {
                    result = Ok(format!("Loc PackedFile successfully exported:\n{}", packed_file_path.display()))
                }
                Err(why) => {
                    result = Err(format!("Error while writing the following file to disk:\n{}\n\nThe problem reported is:\n{}", packed_file_path.display(), why.description()))
                },
            }
        }
        Err(why) => {
            result = Err(format!("Error while trying to write the following file to disk:\n{}\n\nThe problem reported is:\n{}", packed_file_path.display(), why.description()))
        }
    }
    result
}


/// Function to import a LocData from a CSV file, without headers and with the fields quoted.
/// It requires:
/// - csv_file_path: the CSV we want to import.
/// I returns a Result with the new LocData or an Error, depending on what happened.
pub fn import_from_csv(
    csv_file_path: PathBuf
) -> Result<LocData, String> {

    let result: Result<LocData, String>;
    let mut packed_file_data_from_tree_view = LocData::new();

    // We expect no headers, so we need to tweak our reader first.
    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);
    let mut reader = reader_builder.from_path(csv_file_path).unwrap();

    // Then we add the new entries to the decoded entry list.
    let mut error_while_importing_data = false;

    for i in reader.deserialize() {
        match i {
            Ok(entry) => {
                packed_file_data_from_tree_view.packed_file_data_entries.push(entry);
            }
            Err(_) => {
                error_while_importing_data = true;
                break;
            }
        }
    }

    if error_while_importing_data {
        result = Err(format!("Error while parsing the file."));
    }
    else {
        result = Ok(packed_file_data_from_tree_view);
    }

    result
}
