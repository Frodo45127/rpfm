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

use csv::{ReaderBuilder, WriterBuilder, QuoteStyle};
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::io::{BufReader, Read, Write};
use std::fs::File;
use std::path::PathBuf;

use crate::DEPENDENCY_DATABASE;
use crate::FAKE_DEPENDENCY_DATABASE;
use crate::common::*;
use crate::common::coding_helpers::*;
use crate::error::{Error, ErrorKind, Result};
use crate::packfile::{PackFile, PathType};
use crate::packfile::packedfile::PackedFile;
use crate::packedfile::loc::*;
use crate::packedfile::db::*;
use crate::packedfile::db::schemas::{FieldType, Schema, TableDefinition};

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
    DB(String, String, i32),

    // Name of the File.
    Text(String),
}

/// `DecodedData`: This enum is used to store the data from the different fields of a row of a DB/Loc PackedFile.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Boolean(bool),
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
}

/// Const to use in the header of TSV PackedFiles.
pub const TSV_HEADER_PACKFILE_LIST: &str = "PackFile List";
pub const TSV_HEADER_LOC_PACKEDFILE: &str = "Loc PackedFile";

//----------------------------------------------------------------//
// Generic Functions for PackedFiles.
//----------------------------------------------------------------//

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
    let packed_files = vec![PackedFile::read_from_vec(path, get_current_time(), false, data); 1];
    let added_paths = pack_file.add_packed_files(&packed_files);
    if added_paths.len() < packed_files.len() { Err(ErrorKind::ReservedFiles)? }

    // Return the path to update the UI.
    Ok(())
}

/// This function merges (if it's possible) the provided DB and LOC tables into one with the name and, if asked,
/// it deletes the source files. Table_type means true: DB, false: LOC.
pub fn merge_tables( 
    pack_file: &mut PackFile,
    source_paths: &[Vec<String>],
    name: &str,
    delete_source_paths: bool,
    table_type: bool,
) -> Result<(Vec<String>, Vec<PathType>)> {
    
    let mut db_files = vec![];
    let mut loc_files = vec![];

    // Decode them depending on their type.
    for path in source_paths {
        let packed_file = pack_file.packed_files.iter().find(|x| &x.path == path).ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))?;
        let packed_file_data = packed_file.get_data()?;
        
        if table_type { 
            if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                db_files.push(DB::read(&packed_file_data, &path[1], &schema)?); 
            }
            else { return Err(ErrorKind::SchemaNotFound)? }
        }
        else { loc_files.push(Loc::read(&packed_file_data)?); }
    }

    // Merge them all into one, and return error if any problem arise.
    let packed_file_data = if table_type {
        let mut final_entries_list = vec![];
        let mut version = -2;
        let mut table_definition = TableDefinition::new(0);

        for table in &mut db_files {
            if version == -2 { 
                version = table.version; 
                table_definition = table.table_definition.clone();
            }
            else if table.version != version { return Err(ErrorKind::InvalidFilesForMerging)? }

            final_entries_list.append(&mut table.entries);
        }

        let mut new_table = DB::new(&db_files[0].db_type, version, table_definition);
        new_table.entries = final_entries_list;
        new_table.save()
    }

    else {
        let mut final_entries_list = vec![];
        for table in &mut loc_files {
            final_entries_list.append(&mut table.entries);
        }

        let mut new_table = Loc::new();
        new_table.entries = final_entries_list;
        new_table.save()
    };

    // And then, we reach the part where we have to do the "saving to PackFile" stuff.
    let mut path = source_paths[0].to_vec();
    path.pop();
    path.push(name.to_owned());
    let packed_file = PackedFile::read_from_vec(path, get_current_time(), false, packed_file_data);

    // If we want to remove the source files, this is the moment.
    let mut deleted_paths = vec![];
    if delete_source_paths {
        for path in source_paths {
            let index = pack_file.packed_files.iter().position(|x| &x.path == path).unwrap();
            deleted_paths.push(pack_file.packed_files[index].path.to_vec());
            pack_file.remove_packedfile(index);
        }
    }

    // Prepare the paths to return.
    let added_path = pack_file.add_packed_files(&[packed_file]).get(0).ok_or_else(|| Error::from(ErrorKind::ReservedFiles))?.to_vec();
    deleted_paths.retain(|x| x != &added_path);

    let mut tree_paths = vec![];
    for path in &deleted_paths {
        tree_paths.push(PathType::File(path.to_vec()));
    }
    Ok((added_path, tree_paths))
}

/// This function retrieves the entire Dependency Data for a given table definition.
///
/// NOTE: It's here and not in DB because we may get an use for this in LOC PackedFiles.
/// NOTE2: We don't lock the LazyStatics here. We get them as arguments instead. The reason
/// is that way we can put this in a loop without relocking on every freaking PackedFile,
/// which can be extremely slow, depending on the situation.
pub fn get_dependency_data(
    table_definition: &TableDefinition,
    schema: &Schema,
    dep_db: &mut Vec<PackedFile>,
    fake_dep_db: &[DB],
    pack_file: &PackFile
) -> BTreeMap<i32, Vec<String>> {

    // If we reach this point, we build the dependency data of the table.
    let mut dep_data = BTreeMap::new();
    for (column, field) in table_definition.fields.iter().enumerate() {
        if let Some(ref dependency_data) = field.field_is_reference {
            if !dependency_data.0.is_empty() && !dependency_data.1.is_empty() {

                // If the column is a reference column, fill his referenced data.
                let mut data = vec![];
                let mut iter = dep_db.iter_mut();
                while let Some(packed_file) = iter.find(|x| x.path.starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                    if let Ok(table) = DB::read(&packed_file.get_data_and_keep_it().unwrap(), &format!("{}_tables", dependency_data.0), &schema) {
                        if let Some(column_index) = table.table_definition.fields.iter().position(|x| x.field_name == dependency_data.1) {
                            for row in table.entries.iter() {

                                // For now we assume any dependency is a string.
                                match row[column_index] { 
                                    DecodedData::StringU8(ref entry) |
                                    DecodedData::StringU16(ref entry) |
                                    DecodedData::OptionalStringU8(ref entry) |
                                    DecodedData::OptionalStringU16(ref entry) => data.push(entry.to_owned()),
                                    _ => {}
                                }
                            }
                        }
                    } 
                }

                // Same thing for the fake dependency list, if exists.
                let mut iter = fake_dep_db.iter();
                if let Some(table) = iter.find(|x| x.db_type == format!("{}_tables", dependency_data.0)) {
                    if let Some(column_index) = table.table_definition.fields.iter().position(|x| x.field_name == dependency_data.1) {
                        for row in table.entries.iter() {

                            // For now we assume any dependency is a string.
                            match row[column_index] { 
                                DecodedData::StringU8(ref entry) |
                                DecodedData::StringU16(ref entry) |
                                DecodedData::OptionalStringU8(ref entry) |
                                DecodedData::OptionalStringU16(ref entry) => data.push(entry.to_owned()),
                                _ => {}
                            }
                        }
                    }
                }

                // The same for our own PackFile.
                let mut iter = pack_file.packed_files.iter();
                while let Some(packed_file) = iter.find(|x| x.path.starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                    if let Ok(packed_file_data) = packed_file.get_data() {
                        if let Ok(table) = DB::read(&packed_file_data, &format!("{}_tables", dependency_data.0), &schema) {
                            if let Some(column_index) = table.table_definition.fields.iter().position(|x| x.field_name == dependency_data.1) {
                                for row in table.entries.iter() {

                                    // For now we assume any dependency is a string.
                                    match row[column_index] { 
                                        DecodedData::StringU8(ref entry) |
                                        DecodedData::StringU16(ref entry) |
                                        DecodedData::OptionalStringU8(ref entry) |
                                        DecodedData::OptionalStringU16(ref entry) => data.push(entry.to_owned()),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }

                // Sort and dedup the data found.
                data.sort_unstable_by(|a, b| a.cmp(&b));
                data.dedup();

                dep_data.insert(column as i32, data);
            }
        }
    }

    // Return the data, ignoring all possible failures.
    dep_data
}

/// This function checks all the DB Tables of the provided PackFile for dependency errors.
pub fn check_tables( 
    pack_file: &mut PackFile,
) -> Result<()> {

    // Get the schema, or return an error.
    match SCHEMA.lock().unwrap().clone() {
        Some(schema) => {

            let mut broken_tables = vec![];
            let mut dep_db = DEPENDENCY_DATABASE.lock().unwrap();
            let fake_dep_db = FAKE_DEPENDENCY_DATABASE.lock().unwrap();

            // Due to how mutability works, we have first to get the data of every table,
            // then iterate them and decode them.
            for packed_file in pack_file.packed_files.iter_mut() {
                if packed_file.path.starts_with(&["db".to_owned()]) {
                    packed_file.load_data()?;
                }
            }

            for packed_file in pack_file.packed_files.iter() {
                if packed_file.path.starts_with(&["db".to_owned()]) {
                    if let Ok(db_data) = db::DB::read(&(packed_file.get_data().unwrap()), &packed_file.path[1], &schema) {
                        let dep_data = get_dependency_data(&db_data.table_definition, &schema, &mut dep_db, &fake_dep_db, &pack_file);

                        // If we got some dependency data (the referenced tables actually exists), check every
                        // referenced field of every referenced column for errors.
                        if !dep_data.is_empty() {
                            let mut columns = vec![];
                            for row in db_data.entries {
                                for (column, dep_data) in dep_data.iter() {
                                    let field_data = match row[*column as usize] { 
                                        DecodedData::StringU8(ref entry) |
                                        DecodedData::StringU16(ref entry) |
                                        DecodedData::OptionalStringU8(ref entry) |
                                        DecodedData::OptionalStringU16(ref entry) => &entry,
                                        _ => "NoData"
                                    };

                                    if field_data != "NoData" && !field_data.is_empty() && !dep_data.contains(&field_data.to_owned()) {
                                        columns.push(*column);
                                    }
                                }
                            }

                            // If we got missing refs, sort the columns, dedup them and turn them into a nice string for the error message.
                            // Columns + 1 is so we don't start counting on zero. Easier for the user to see.
                            if !columns.is_empty() {
                                columns.sort();
                                columns.dedup();
                                let mut columns = columns.iter().map(|x| format!("{},", *x + 1)).collect::<String>();
                                columns.pop();
                                broken_tables.push(format!("Table: {}/{}, Column/s: {}", &packed_file.path[1], &packed_file.path[2], columns)); 
                            }
                        }
                    }
                }
            }

            // If all tables are Ok, return Ok. Otherwise, return an error with the list of broken tables.
            if broken_tables.is_empty() { Ok(()) }
            else { Err(ErrorKind::DBMissingReferences(broken_tables))? }
        }
        None => Err(ErrorKind::SchemaNotFound)?
    }
}

//----------------------------------------------------------------//
// TSV Functions for PackedFiles.
//----------------------------------------------------------------//

/// This function imports a TSV file and loads his contents into a DB Table.
pub fn import_tsv(
    definition: &TableDefinition,
    path: &PathBuf,
    name: &str,
    version: i32,
) -> Result<Vec<Vec<DecodedData>>> {

    // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
    // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(false)
        .has_headers(false)
        .flexible(true)
        .from_path(&path)?;

    // If we succesfully load the TSV file into a reader, check the first two lines to ensure 
    // it's a valid TSV for our specific DB/Loc.
    let mut entries = vec![];
    for (row, record) in reader.records().enumerate() {
        if let Ok(record) = record {

            // The first line should contain the "table_folder_name"/"Loc PackedFile/PackFile List", and the version (1 for Locs).
            if row == 0 { 
                if record.get(0).unwrap_or("error") != name { return Err(ErrorKind::ImportTSVWrongTypeTable)?; }
                if record.get(1).unwrap_or("-1").parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? != version { 
                    return Err(ErrorKind::ImportTSVWrongVersion)?;
                }
            }

            // The second line contains the column headers. Is just to help people in other programs,
            // not needed to be check.
            else if row == 1 { continue }

            // Then read the rest of the rows as a normal TSV.
            else if record.len() == definition.fields.len() {
                let mut entry = vec![];
                for (column, field) in record.iter().enumerate() {
                    match definition.fields[column].field_type {
                        FieldType::Boolean => {
                            let value = field.to_lowercase();
                            if value == "true" || value == "1" { entry.push(DecodedData::Boolean(true)); }
                            else if value == "false" || value == "0" { entry.push(DecodedData::Boolean(false)); }
                            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?; }
                        }
                        FieldType::Float => entry.push(DecodedData::Float(field.parse::<f32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                        FieldType::Integer => entry.push(DecodedData::Integer(field.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                        FieldType::LongInteger => entry.push(DecodedData::LongInteger(field.parse::<i64>().map_err(|_| Error::from(ErrorKind::ImportTSVIncorrectRow(row, column)))?)),
                        FieldType::StringU8 => entry.push(DecodedData::StringU8(field.to_owned())),
                        FieldType::StringU16 => entry.push(DecodedData::StringU16(field.to_owned())),
                        FieldType::OptionalStringU8 => entry.push(DecodedData::OptionalStringU8(field.to_owned())),
                        FieldType::OptionalStringU16 => entry.push(DecodedData::OptionalStringU16(field.to_owned())),
                    }
                }
                entries.push(entry);
            }

            // If it fails here, return an error with the len of the record instead a field.
            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()))?; }
        }

        else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
    }

    // If we reached this point without errors, we replace the old data with the new one and return success.
    Ok(entries)
}

/// This function creates a TSV file with the contents of the DB/Loc PackedFile.
pub fn export_tsv(
    data: &[Vec<DecodedData>], 
    path: &PathBuf,
    headers: &[String], 
    first_row_data: (&str, i32)
) -> Result<()> {

    // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
    // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(QuoteStyle::Never)
        .has_headers(false)
        .flexible(true)
        .from_writer(vec![]);

    // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
    writer.serialize(first_row_data)?;
    writer.serialize(headers)?;

    // Then we serialize each entry in the DB Table.
    for entry in data { writer.serialize(&entry)?; }

    // Then, we try to write it on disk. If there is an error, report it.
    let mut file = File::create(&path)?;
    file.write_all(String::from_utf8(writer.into_inner().unwrap())?.as_bytes())?;

    Ok(())
}

//----------------------------------------------------------------//
// Mass-TSV Functions for PackedFiles.
//----------------------------------------------------------------//

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

                let definition = TableDefinition::new_loc_definition();
                match import_tsv(&definition, &path, tsv_info[0], 1) {
                    Ok(data) => {

                        let mut loc = Loc::new();
                        loc.entries = data;
                        let raw_data = loc.save();
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
                        packed_files.push(PackedFile::read_from_vec(path, get_current_time(), false, raw_data));
                    }
                    Err(_) => error_files.push(path.to_string_lossy().to_string()),
                }
            }

            // If there are two fields in the header, either it's a table or an invalid TSV.
            else if tsv_info.len() == 2 {

                // Get the type and the version of the table and check if it's in the schema.
                let table_type = tsv_info[0];
                let table_version = tsv_info[1].parse::<i32>().unwrap();
                
                let table_definition = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    if let Some(table_definition) = DB::get_schema(&table_type, table_version, &schema) { table_definition }
                    else { error_files.push(path.to_string_lossy().to_string()); continue }
                } else { error_files.push(path.to_string_lossy().to_string()); continue };

                // If it's a DB Table, use their logic for the importing.
                match import_tsv(&table_definition, &path, &table_type, table_version) {
                    Ok(data) => {

                        let mut db = DB::new(table_type, table_version, table_definition);
                        db.entries = data;
                        let raw_data = db.save();
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
                        packed_files.push(PackedFile::read_from_vec(path, get_current_time(), false, raw_data));
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
    let added_paths = pack_file.add_packed_files(&packed_files);
    if added_paths.len() < packed_files.len() { Err(ErrorKind::ReservedFiles)? }
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
                                let headers = db.table_definition.fields.iter().map(|x| x.field_name.to_owned()).collect::<Vec<String>>();
                                match export_tsv(&db.entries, &export_path, &headers, (&packed_file.path[1], db.version)) {
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
                        let headers = TableDefinition::new_loc_definition().fields.iter().map(|x| x.field_name.to_owned()).collect::<Vec<String>>();
                        match export_tsv(&loc.entries, &export_path, &headers, ("Loc PackedFile", 1)) {
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
