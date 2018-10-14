// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;

use std::io::{ BufReader, Read };
use std::fs::File;
use std::path::PathBuf;

use common::*;
use common::coding_helpers::*;
use error::{ErrorKind, Result};
use packfile::packfile::PackFile;
use packfile::packfile::PackedFile;
use packedfile::loc::*;
use packedfile::db::*;
use packedfile::db::schemas::*;

pub mod loc;
pub mod db;
pub mod rigidmodel;

/// This enum specifies the PackedFile types we can create.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PackedFileType {

    // Name of the File.
    Loc(String),

    // Name of the File, Name of the table, version of the table.
    DB(String, String, u32),

    // Name of the File.
    Text(String),
}

/*
--------------------------------------------------
             Traits for PackedFiles
--------------------------------------------------
*/

/// Trait `SerializableToTSV`: This trait needs to be implemented by all the structs that can be
/// export to and import from a TSV file, like `LocData` and `DBData`.
pub trait SerializableToTSV {

    /// `import_tsv`: Requires `&mut self`, a `&PathBuf` with the path of the TSV file and the name of our PackedFile.
    /// Returns success or an error.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<()>;

    /// `export_tsv`: Requires `&self`, the destination path for the TSV file and a name and a number (version)
    /// to put in the header of the TSV file. Returns a success message, or an error.
    fn export_tsv(&self, tsv_file_path: &PathBuf, extra_data: (&str, u32)) -> Result<String>;
}

/*
--------------------------------------------------
           Functions for PackedFiles
--------------------------------------------------
*/

/// This function is used to create a PackedFile outtanowhere. It returns his new path.
pub fn create_packed_file(
    pack_file: &mut PackFile,
    packed_file_type: PackedFileType,
    path: Vec<String>,
    schema: &Option<Schema>,
) -> Result<()> {

    // Depending on their type, we do different things to prepare the PackedFile and get his data.
    let data = match packed_file_type {

        // If it's a Loc PackedFile, create it and generate his data.
        PackedFileType::Loc(_) => Loc::new().save(),

        // If it's a DB table...
        PackedFileType::DB(_, table, version) => {

            // Try to get his table definition.
            let table_definition = match schema {
                Some(schema) => DB::get_schema(&table, version, &schema),
                None => return Err(ErrorKind::SchemaNotFound)?
            };

            // If there is a table definition, create the new table. Otherwise, return error.
            match table_definition {
                Some(table_definition) => DB::new(&table, version, table_definition).save(),
                None => return Err(ErrorKind::SchemaTableDefinitionNotFound)?
            }
        }

        // If it's a Text PackedFile, return an empty encoded string.
        PackedFileType::Text(_) => encode_string_u8(""),
    };

    // Create and add the new PackedFile to the PackFile.
    pack_file.add_packedfiles(vec![PackedFile::read(get_current_time(), path, data); 1]);

    // Return the path to update the UI.
    Ok(())
}

/// This function is used to Mass-Import TSV files into a PackFile. Note that this will OVERWRITE any
/// existing PackedFile that has a name conflict with the TSV files provided.
pub fn tsv_mass_import(
    tsv_paths: &[PathBuf],
    name: &str,
    schema: &Option<Schema>,
    pack_file: &mut PackFile
) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {

    // Create a list of PackedFiles succesfully imported, and another for the ones that didn't work.
    let mut packed_files: Vec<PackedFile> = vec![];
    let mut packed_files_to_remove = vec![];
    let mut error_files = vec![];

    // For each TSV File we have...
    for (index, path) in tsv_paths.iter().enumerate() {

        // We open it and read it to a string.
        let mut tsv = String::new();
        BufReader::new(File::open(&path)?).read_to_string(&mut tsv)?;

        // We get his first line, if it have it.
        if let Some(line) = tsv.lines().next() {

            // Split the first line by \t so we can get the info of the table.
            let tsv_info = line.split('\t').collect::<Vec<&str>>();

            // If there are two fields in the header...
            if tsv_info.len() == 2 {

                // If the header is from a Loc PackedFile...
                if tsv_info[0] == "Loc PackedFile" && tsv_info[1] == "9001" {

                    // Create a new default Loc PackedFile.
                    let mut loc = Loc::new();

                    // Try to import the TSV's data into it.
                    match loc.import_tsv(&path, tsv_info[0]) {
                        Ok(_) => {

                            // Save it.
                            let data = loc.save();

                            // Create his new path.
                            let mut path = vec!["text".to_owned(), "db".to_owned(), format!("{}.loc", name)];

                            // If that path already exists in the list of PackedFiles to add, change it using the index.
                            for packed_file in &packed_files {
                                if packed_file.path == path {
                                    path[2] = format!("{}_{}.loc", name, index);
                                }
                            }

                            // If that path already exist in the PackFile, add it to the "remove" list.
                            if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                            // Create and add the new PackedFile to the PackFile.
                            packed_files.push(PackedFile::read(get_current_time(), path, data));
                        }

                        // In case of error, add it to the error list.
                        Err(_) => error_files.push(path),
                    }
                }

                // Otherwise, it's a table or an invalid TSV.
                else {

                    // Get the type and the version of the table.
                    let table_type = tsv_info[0];
                    let table_version = tsv_info[1].parse::<u32>().unwrap();

                    // If we managed to find it in the schema...
                    let table_definition = if let Some(ref schema) = schema {
                        if let Some(table_definition) = DB::get_schema(&table_type, table_version, &schema) {
                            table_definition

                        // If we didn't found the schema or the table_definition, add it to the error list.
                        } else { error_files.push(path); continue }
                    } else { error_files.push(path); continue };

                    // Create a new default DB PackedFile.
                    let mut db = DB::new(table_type, table_version, table_definition);

                    // Try to import the TSV's data into it.
                    match db.import_tsv(&path, &table_type) {
                        Ok(_) => {

                            // Save it.
                            let data  = db.save();

                            // Change his path.
                            let mut path = vec!["db".to_owned(), table_type.to_owned(), name.to_owned()];

                            // If that path already exists in the list of PackedFiles to add, change it using the index.
                            for packed_file in &packed_files {
                                if packed_file.path == path {
                                    path[2] = format!("{}_{}", name, index);
                                }
                            }

                            // If that path already exists in the PackFile, add it to the "remove" list.
                            if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                            // Create and add the new PackedFile to the PackFile.
                            packed_files.push(PackedFile::read(get_current_time(), path, data));
                        }

                        // In case of error, add it to the error list.
                        Err(_) => error_files.push(path),
                    }
                }
            }

            // If the file has an incorrect header, add it to the error list.
            else { error_files.push(path) }
        }

        // If the header is empty, add it to the error list.
        else { error_files.push(path) }
    }

    // If any of the files returned error, return error.
    if !error_files.is_empty() {
        let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x.display().to_string())).collect::<Vec<String>>();
        return Err(ErrorKind::MassImport(error_files_string))?
    }

    // Get the "TreePath" of the new PackFiles to return them.
    let tree_path = packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>();

    // Remove all the "conflicting" PackedFiles from the PackFile, before adding the new ones.
    let mut indexes = vec![];
    for packed_file_to_remove in &packed_files_to_remove {
        for (index, packed_file) in pack_file.packed_files.iter().enumerate() {
            if packed_file.path == *packed_file_to_remove {
                indexes.push(index);
                break;
            }
        }
    }
    indexes.iter().rev().for_each(|x| pack_file.remove_packedfile(*x) );

    // We add all the files to the PackFile, and return success.
    pack_file.add_packedfiles(packed_files);

    // And return success.
    Ok((packed_files_to_remove, tree_path))
}

/// This function is used to Mass-Export TSV files from a PackFile. Note that this will OVERWRITE any
/// existing file that has a name conflict with the TSV files provided.
pub fn tsv_mass_export(
    export_path: &PathBuf,
    schema: &Option<Schema>,
    pack_file: &PackFile
) -> Result<String> {

    // List of PackedFiles that couldn't be exported for one thing or another.
    let mut error_list = vec![];

    // List of exported file's names, so we don't overwrite them one with another.
    let mut exported_files = vec![];

    // For each PackedFile we have...
    for packed_file in &pack_file.packed_files {

        // Check if it's "valid for exportation". We check if his path is empty first to avoid false
        // positives related with "starts_with" function.
        if !packed_file.path.is_empty() {

            // If the PackedFile is a DB Table...
            if packed_file.path.starts_with(&["db".to_owned()]) && packed_file.path.len() == 3 {

                // Check if we have an schema for the game.
                match schema {

                    // If we have it...
                    Some(schema) => {

                        // Try to decode the data of the PackedFile.
                        match DB::read(&(packed_file.get_data()?), &packed_file.path[1], &schema) {

                            // In case of success...
                            Ok(db) => {

                                // Get his name to add it to the path.
                                let mut name = format!("{}_{}.tsv", packed_file.path[1], packed_file.path.last().unwrap().to_owned());

                                // Get the final exported path.
                                let mut export_path = export_path.to_path_buf();

                                // Create the index for the duplicate checks.
                                let mut index = 1;

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                loop {

                                    // If the name is not in the exported list, is valid.
                                    if !exported_files.contains(&name) { break; }

                                    // If the name was invalid, add to it the index.
                                    else { name = format!("{}_{}_{}.tsv", packed_file.path[1], packed_file.path.last().unwrap().to_owned(), index); }

                                    // Increase the index.
                                    index += 1;
                                }

                                // Add whatever name we got to the path.
                                export_path.push(name.to_owned());

                                // Try to export it to the provided path.
                                match db.export_tsv(&export_path, (&packed_file.path[1], db.version)) {

                                    // If success, add it to the exported files list.
                                    Ok(_) => exported_files.push(name.to_owned()),

                                    // In case of error, add it to the error list.
                                    Err(_) => error_list.push(packed_file.path.to_vec()),
                                }
                            }

                            // In case of error, add it to the error list.
                            Err(_) => error_list.push(packed_file.path.to_vec()),
                        }
                    }

                    // If we don't have it, add the PackedFile to the error list.
                    None => error_list.push(packed_file.path.to_vec()),
                }
            }

            // Otherwise, we check if it's a Loc PackedFile.
            else if packed_file.path.last().unwrap().ends_with(".loc") {

                // Try to decode the data of the PackedFile.
                match Loc::read(&(packed_file.get_data()?)) {

                    // In case of success...
                    Ok(loc) => {

                        // Get his name to add it to the path.
                        let mut name = format!("{}.tsv", packed_file.path.last().unwrap().to_owned());

                        // Get the final exported path.
                        let mut export_path = export_path.to_path_buf();

                        // Create the index for the duplicate checks.
                        let mut index = 1;

                        // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                        loop {

                            // If the name is not in the exported list, is valid.
                            if !exported_files.contains(&name) { break; }

                            // If the name was invalid, add to it the index.
                            else { name = format!("{}_{}.tsv", packed_file.path.last().unwrap().to_owned(), index); }

                            // Increase the index.
                            index += 1;
                        }

                        // Add whatever name we got to the path.
                        export_path.push(name.to_owned());

                        // Try to export it to the provided path.
                        match loc.export_tsv(&export_path, ("Loc PackedFile", 9001)) {

                            // If success, add it to the exported files list.
                            Ok(_) => exported_files.push(name.to_owned()),

                            // In case of error, add it to the error list.
                            Err(_) => error_list.push(packed_file.path.to_vec()),
                        }
                    }

                    // In case of error, add it to the error list.
                    Err(_) => error_list.push(packed_file.path.to_vec()),
                }
            }
        }
    }

    // If there has been errors, return ok with the list of errors.
    if !error_list.is_empty() {
        let error_files_string = error_list.iter().map(|x| format!("<li>{:#?}</li>", x)).collect::<String>();
        Ok(format!("<p>All exportable files have been exported, except the following ones:</p><ul>{}</ul>", error_files_string))
    }

    // Otherwise, just return success and an empty error list.
    else { Ok("<p>All exportable files have been exported.</p>".to_owned()) }
}
