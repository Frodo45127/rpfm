//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.

use serde_derive::{Serialize, Deserialize};

use std::io::{ BufReader, Read };
use std::fs::File;
use std::path::PathBuf;

use crate::common::*;
use crate::common::coding_helpers::*;
use crate::error::{Error, ErrorKind, Result};
use crate::packfile::PackFile;
use crate::packfile::packedfile::PackedFile;
use crate::packedfile::loc::*;
use crate::packedfile::db::*;

use crate::SCHEMA;
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

    /// `import_tsv`: Requires `&mut self`, a `&PathBuf` with the path of the TSV file and (in case of a table)
    /// the name of our DB Table's table (xxx_tables). Returns success or an error.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, db_name: &str) -> Result<()>;

    /// `export_tsv`: Requires `&self`, the destination path for the TSV file and (in case of a table) 
    /// a name and a number (version) to put in the header of the TSV file. Returns sucess or an error.
    fn export_tsv(&self, tsv_file_path: &PathBuf, db_info: (&str, u32)) -> Result<()>;
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
) -> Result<()> {

    // Depending on their type, we do different things to prepare the PackedFile and get his data.
    let data = match packed_file_type {

        // If it's a Loc PackedFile, create it and generate his data.
        PackedFileType::Loc(_) => Loc::new().save(),

        // If it's a DB table...
        PackedFileType::DB(_, table, version) => {

            // Try to get his table definition.
            let table_definition = match *SCHEMA.lock().unwrap() {
                Some(ref schema) => DB::get_schema(&table, version, &schema),
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
    pack_file.add_packedfiles(vec![PackedFile::read_from_vec(path, get_current_time(), false, data); 1]);

    // Return the path to update the UI.
    Ok(())
}

/// This function is used to Mass-Import TSV files into a PackFile. Note that this will OVERWRITE any
/// existing PackedFile that has a name conflict with the TSV files provided.
pub fn tsv_mass_import(
    tsv_paths: &[PathBuf],
    name: &str,
    pack_file: &mut PackFile
) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {

    // Create a list of PackedFiles succesfully imported, and another for the ones that didn't work.
    // The a third one to return the PackedFiles that were overwritten, so the UI can have an easy time updating his TreeView.
    let mut packed_files: Vec<PackedFile> = vec![];
    let mut packed_files_to_remove = vec![];
    let mut error_files = vec![];

    for path in tsv_paths {

        // We open it and read it to a string. We use the first row to check what kind of TSV is, and the second one we ignore it.
        let mut tsv = String::new();
        BufReader::new(File::open(&path)?).read_to_string(&mut tsv)?;

        // We get his first line, if it have it. Otherwise, we return an error in this file.
        if let Some(line) = tsv.lines().next() {

            // Split the first line by \t so we can get the info of the table.
            let tsv_info = line.split('\t').collect::<Vec<&str>>();

            // If it's a Loc PackedFile, use loc importing logic to try to import it.
            if tsv_info.len() == 1 && tsv_info[0] == "Loc PackedFile" {

                let mut loc = Loc::new();
                match loc.import_tsv(&path, "") {
                    Ok(_) => {

                        let data = loc.save();
                        let mut path = vec!["text".to_owned(), "db".to_owned(), format!("{}.loc", name)];

                        // If that path already exists in the list of new PackedFiles to add, change it using the index.
                        let mut index = 1;
                        while packed_files.iter().any(|x| x.path == path) {
                            path[2] = format!("{}_{}.loc", name, index);
                            index += 1;
                        }

                        // If that path already exist in the PackFile, add it to the "remove" list.
                        if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                        // Create and add the new PackedFile to the list of PackedFiles to add.
                        packed_files.push(PackedFile::read_from_vec(path, get_current_time(), false, data));
                    }
                    Err(_) => error_files.push(path.to_string_lossy().to_string()),
                }
            }

            // If there are two fields in the header, either it's a table or an invalid TSV.
            else if tsv_info.len() == 2 {

                // Get the type and the version of the table and check if it's in the schema.
                let table_type = tsv_info[0];
                let table_version = tsv_info[1].parse::<u32>().unwrap();
                
                let table_definition = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    if let Some(table_definition) = DB::get_schema(&table_type, table_version, &schema) { table_definition }
                    else { error_files.push(path.to_string_lossy().to_string()); continue }
                } else { error_files.push(path.to_string_lossy().to_string()); continue };

                // If it's a DB Table, use their logic for the importing.
                let mut db = DB::new(table_type, table_version, table_definition);
                match db.import_tsv(&path, &table_type) {
                    Ok(_) => {

                        let data = db.save();
                        let mut path = vec!["db".to_owned(), table_type.to_owned(), name.to_owned()];
                        
                        // If that path already exists in the list of new PackedFiles to add, change it using the index.
                        let mut index = 1;
                        while packed_files.iter().any(|x| x.path == path) {
                            path[2] = format!("{}_{}.loc", name, index);
                            index += 1;
                        }

                        // If that path already exists in the PackFile, add it to the "remove" list.
                        if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                        // Create and add the new PackedFile to the list of PackedFiles to add.
                        packed_files.push(PackedFile::read_from_vec(path, get_current_time(), false, data));
                    }
                    Err(_) => error_files.push(path.to_string_lossy().to_string()),
                }
            }
            else { error_files.push(path.to_string_lossy().to_string()) }
        }
        else { error_files.push(path.to_string_lossy().to_string()) }
    }

    // If any of the files returned error, return error.
    if !error_files.is_empty() {
        let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<String>();
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
    Ok((packed_files_to_remove, tree_path))
}

/// This function is used to Mass-Export TSV files from a PackFile. Note that this will OVERWRITE any
/// existing file that has a name conflict with the TSV files provided.
pub fn tsv_mass_export(
    export_path: &PathBuf,
    pack_file: &mut PackFile
) -> Result<String> {

    // Lists of PackedFiles that couldn't be exported for one thing or another and exported PackedFile names,
    // so we make sure we don't overwrite those with the following ones.
    let mut error_list = vec![];
    let mut exported_files = vec![];

    for packed_file in &mut pack_file.packed_files {

        // We check if his path is empty first to avoid false positives related with "starts_with" function.
        if !packed_file.path.is_empty() {

            // If the PackedFile is a DB Table and we have an schema, try to decode it and export it.
            if packed_file.path.starts_with(&["db".to_owned()]) && packed_file.path.len() == 3 {
                match *SCHEMA.lock().unwrap() {
                    Some(ref schema) => {
                        match DB::read(&(packed_file.get_data_and_keep_it()?), &packed_file.path[1], &schema) {
                            Ok(db) => {

                                // His name will be "db_name_file_name.tsv". If that's taken, we'll add an index until we find one available.
                                let mut name = format!("{}_{}.tsv", packed_file.path[1], packed_file.path.last().unwrap().to_owned());
                                let mut export_path = export_path.to_path_buf();

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                let mut index = 1;
                                while exported_files.contains(&name) {
                                    name = format!("{}_{}_{}.tsv", packed_file.path[1], packed_file.path.last().unwrap().to_owned(), index);
                                    index += 1;
                                }

                                export_path.push(name.to_owned());
                                match db.export_tsv(&export_path, (&packed_file.path[1], db.version)) {
                                    Ok(_) => exported_files.push(name.to_owned()),
                                    Err(error) => error_list.push((packed_file.path.to_vec().join("\\"), error)),
                                }
                            }
                            Err(error) => error_list.push((packed_file.path.to_vec().join("\\"), error)),
                        }
                    }
                    None => error_list.push((packed_file.path.to_vec().join("\\"), Error::from(ErrorKind::SchemaNotFound))),
                }
            }

            // Otherwise, we check if it's a Loc PackedFile, and try to decode it and export it.
            else if packed_file.path.last().unwrap().ends_with(".loc") {
                match Loc::read(&(packed_file.get_data_and_keep_it()?)) {
                    Ok(loc) => {

                        // His name will be "file_name.tsv". If that's taken, we'll add an index until we find one available.
                        let mut name = format!("{}.tsv", packed_file.path.last().unwrap().to_owned());
                        let mut export_path = export_path.to_path_buf();

                        // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                        let mut index = 1;
                        while exported_files.contains(&name) {
                            name = format!("{}_{}.tsv", packed_file.path.last().unwrap().to_owned(), index);
                            index += 1;
                        }

                        export_path.push(name.to_owned());
                        match loc.export_tsv(&export_path, ("", 0)) {
                            Ok(_) => exported_files.push(name.to_owned()),
                            Err(error) => error_list.push((packed_file.path.to_vec().join("\\"), error)),
                        }
                    }
                    Err(error) => error_list.push((packed_file.path.to_vec().join("\\"), error)),
                }
            }
        }
    }

    // If there has been errors, return ok with the list of errors.
    if !error_list.is_empty() {
        let error_files_string = error_list.iter().map(|x| format!("<li>{}</li>", x.0)).collect::<String>();
        Ok(format!("<p>All exportable files have been exported, except the following ones:</p><ul>{}</ul>", error_files_string))
    }

    // Otherwise, just return success and an empty error list.
    else { Ok("<p>All exportable files have been exported.</p>".to_owned()) }
}
