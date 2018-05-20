// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;
extern crate failure;

use failure::Error;
use std::io::{ BufReader, Read };
use std::fs::File;
use std::path::PathBuf;
use packfile::packfile::PackFile;
use packfile::packfile::PackedFile;
use packedfile::loc::*;
use packedfile::db::*;
use packedfile::db::schemas::*;

pub mod loc;
pub mod db;
pub mod rigidmodel;

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
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<(), Error>;

    /// `export_tsv`: Requires `&self`, the destination path for the TSV file and a name and a number (version)
    /// to put in the header of the TSV file. Returns a success message, or an error.
    fn export_tsv(&self, tsv_file_path: &PathBuf, extra_data: (&str, u32)) -> Result<String, Error>;
}

/// This function is used to Mass-Import TSV files into a PackFile. Note that this will OVERWRITE any
/// existing PackedFile that has a name conflict with the TSV files provided.
pub fn tsv_mass_import(
    tsv_paths: &[PathBuf],
    name: &str,
    schema: &Option<Schema>,
    pack_file: &mut PackFile
) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>), Error> {

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
                    match loc.data.import_tsv(&path, tsv_info[0]) {
                        Ok(_) => {

                            // Save it.
                            let data = loc.save();

                            // Create his new path.
                            let mut path = vec!["text".to_owned(), "db".to_owned(), format!("{}.loc", name)];

                            // If that path already exists in th list of PackedFiles to add, change it using the index.
                            for packed_file in &packed_files {
                                if packed_file.path == path {
                                    path[2] = format!("{}_{}.loc", name, index);
                                }
                            }

                            // If that path already exist in the PackFile, add it to the "remove" list.
                            if pack_file.data.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                            // Create and add the new PackedFile to the PackFile.
                            packed_files.push(PackedFile::read(data.len() as u32, path, data));
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
                    match db.data.import_tsv(&path, &table_type) {
                        Ok(_) => {

                            // Save it.
                            let data  = db.save();

                            // Change his path.
                            let mut path = vec!["db".to_owned(), table_type.to_owned(), name.to_owned()];

                            // If that path already exists in th list of PackedFiles to add, change it using the index.
                            for packed_file in &packed_files {
                                if packed_file.path == path {
                                    path[2] = format!("{}_{}", name, index);
                                }
                            }

                            // If that path already exists in the PackFile, add it to the "remove" list.
                            if pack_file.data.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                            // Create and add the new PackedFile to the PackFile.
                            packed_files.push(PackedFile::read(data.len() as u32, path, data));
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
        return Err(format_err!("The following files returned error when trying to import them:\n\n{:#?}", error_files))
    }

    // Get the "TreePath" of the new PackFiles to return them.
    let tree_path = packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>();

    // Remove all the "conflicting" PackedFiles from the PackFile, before adding the new ones.
    let mut indexes = vec![];
    for packed_file_to_remove in &packed_files_to_remove {
        for (index, packed_file) in pack_file.data.packed_files.iter_mut().enumerate() {
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
