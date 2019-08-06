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

use rpfm_error::{Error, ErrorKind, Result};

use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{fmt, fmt::Display};
use std::fs::File;
use std::path::PathBuf;

use crate::SETTINGS;
use crate::DEPENDENCY_DATABASE;
use crate::FAKE_DEPENDENCY_DATABASE;
use crate::common::*;
use crate::packfile::{PackFile, PathType};
use crate::packfile::packedfile::PackedFile;
use crate::packedfile::loc::*;
use crate::packedfile::db::*;
use crate::schema::{FieldType, Schema, Definition};

use crate::SCHEMA;
pub mod loc;
pub mod db;
pub mod rigidmodel;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This enum specifies the different types of `PackedFile` we can find in a `PackFile`.
///
/// Some of his variants contain useful information about the PackedFile itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PackedFileType {

    // Contains his *table name* (name of his parent folder in the PackFile), and his version.
    DB(String, i32),

    // Name of the File.
    Loc,

    // Name of the File.
    Text,
}

/// This enum specifies the PackedFile types we can decode.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodeablePackedFileType {
    DB,
    Loc,
    Text,
    Image,
    RigidModel,
    
    // Wildcard for undecodeable PackFiles.
    None
}

/// `DecodedData`: This enum is used to store the data from the different fields of a row of a DB/Loc PackedFile.
///
/// NOTE: `Sequence` it's a recursive type. A Sequence/List means you got a repeated sequence of fields
/// inside a single field. Used, for example, in certain model tables.
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
    Sequence(Vec<Vec<DecodedData>>)
}

/// Const to use in the header of TSV PackedFiles.
pub const TSV_HEADER_PACKFILE_LIST: &str = "PackFile List";
pub const TSV_HEADER_LOC_PACKEDFILE: &str = "Loc PackedFile";

//----------------------------------------------------------------//
// Implementations for `DecodeablePackedFileType`.
//----------------------------------------------------------------//

/// Display implementation of `DecodeablePackedFileType`.
impl Display for DecodeablePackedFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeablePackedFileType::DB => write!(f, "DB"),
            DecodeablePackedFileType::Loc => write!(f, "LOC"),
            DecodeablePackedFileType::Text => write!(f, "Text"),
            DecodeablePackedFileType::Image => write!(f, "Image"),
            DecodeablePackedFileType::RigidModel => write!(f, "RigidModel"),
            DecodeablePackedFileType::None => write!(f, "Unknown"),
        }
    }
}

//----------------------------------------------------------------//
// Generic Functions for PackedFiles.
//----------------------------------------------------------------//

/// This function returns the type of the provided PackedFile path.
pub fn get_packed_file_type(path: &[String]) -> DecodeablePackedFileType {
    if let Some(packedfile_name) = path.last() {

        // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
        if path[0] == "db" { DecodeablePackedFileType::DB }

        // If it ends in ".loc", it's a localisation PackedFile.
        else if packedfile_name.ends_with(".loc") { DecodeablePackedFileType::Loc }

        // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
        else if packedfile_name.ends_with(".rigid_model_v2") { DecodeablePackedFileType::RigidModel }

        // If it ends in any of these, it's a plain text PackedFile.
        else if packedfile_name.ends_with(".lua") ||
                packedfile_name.ends_with(".xml") ||
                packedfile_name.ends_with(".xml.shader") ||
                packedfile_name.ends_with(".xml.material") ||
                packedfile_name.ends_with(".variantmeshdefinition") ||
                packedfile_name.ends_with(".environment") ||
                packedfile_name.ends_with(".lighting") ||
                packedfile_name.ends_with(".wsmodel") ||
                packedfile_name.ends_with(".csv") ||
                packedfile_name.ends_with(".tsv") ||
                packedfile_name.ends_with(".inl") ||
                packedfile_name.ends_with(".battle_speech_camera") ||
                packedfile_name.ends_with(".bob") ||
                packedfile_name.ends_with(".cindyscene") ||
                packedfile_name.ends_with(".cindyscenemanager") ||
                //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                packedfile_name.ends_with(".txt") { DecodeablePackedFileType::Text }

        // If it ends in any of these, it's an image.
        else if packedfile_name.ends_with(".jpg") ||
                packedfile_name.ends_with(".jpeg") ||
                packedfile_name.ends_with(".tga") ||
                packedfile_name.ends_with(".dds") ||
                packedfile_name.ends_with(".png") { DecodeablePackedFileType::Image }

        // Otherwise, we don't have a decoder for that PackedFile... yet.
        else { DecodeablePackedFileType::None }
    }

    // If we didn't got a name, it means something broke. Return none.
    else { DecodeablePackedFileType::None }
}

/// This function is used to create a PackedFile outtanowhere. It returns his new path.
pub fn create_packed_file(
    pack_file: &mut PackFile,
    packed_file_type: PackedFileType,
    path: Vec<String>,
) -> Result<()> {

    // Depending on their type, we do different things to prepare the PackedFile and get his data.
    let data = match packed_file_type {

        // If it's a Loc PackedFile, create it and generate his data.
        PackedFileType::Loc => Loc::new().save(),

        // If it's a DB table...
        PackedFileType::DB(table, version) => {

            // Try to get his table definition.
            let schema = SCHEMA.lock().unwrap();
            let table_definition = match *schema {
                Some(ref schema) => schema.get_versioned_file_db(&table)?.get_version(version)?,
                None => return Err(ErrorKind::SchemaNotFound)?
            };

            // If there is a table definition, create the new table. Otherwise, return error.
            DB::new(&table, &table_definition).save()
        }

        // If it's a Text PackedFile, return an empty encoded string.
        PackedFileType::Text => vec![],
    };

    // Create and add the new PackedFile to the PackFile.
    let packed_files = vec![PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, data); 1];
    pack_file.add_packed_files(&packed_files, true)?;

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
        if let Some(packed_file) = pack_file.get_ref_packed_file_by_path(path) {
            let packed_file_data = packed_file.get_data()?;
            
            if table_type { 
                if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    db_files.push(DB::read(&packed_file_data, &path[1], &schema)?); 
                }
                else { return Err(ErrorKind::SchemaNotFound)? }
            }
            else { loc_files.push(Loc::read(&packed_file_data)?); }
        }
    }

    // Merge them all into one, and return error if any problem arise.
    let packed_file_data = if table_type {
        let mut final_entries_list = vec![];
        let mut version = -2;
        let mut table_definition = Definition::new(0);

        for table in &mut db_files {
            if version == -2 { 
                version = table.definition.version; 
                table_definition = table.definition.clone();
            }
            else if table.definition.version != version { return Err(ErrorKind::InvalidFilesForMerging)? }

            final_entries_list.append(&mut table.entries);
        }

        let mut new_table = DB::new(&db_files[0].name, &table_definition);
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
    let packed_file = PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, packed_file_data);

    // If we want to remove the source files, this is the moment.
    let mut deleted_paths = vec![];
    if delete_source_paths {
        for path in source_paths {
            pack_file.remove_packed_file_by_path(path);
            deleted_paths.push(path);
        }
    }

    // Prepare the paths to return.
    let added_path = pack_file.add_packed_files(&[packed_file], true)?.get(0).ok_or_else(|| Error::from(ErrorKind::ReservedFiles))?.to_vec();
    deleted_paths.retain(|x| x != &&added_path);

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
    table_definition: &Definition,
    schema: &Schema,
    dep_db: &mut Vec<PackedFile>,
    fake_dep_db: &[DB],
    pack_file: &PackFile
) -> BTreeMap<i32, Vec<String>> {

    // If we reach this point, we build the dependency data of the table.
    let mut dep_data = BTreeMap::new();
    for (column, field) in table_definition.fields.iter().enumerate() {
        if let Some(ref dependency_data) = field.is_reference {
            if !dependency_data.0.is_empty() && !dependency_data.1.is_empty() {

                // If the column is a reference column, fill his referenced data.
                let mut data = vec![];
                let mut iter = dep_db.iter_mut();
                while let Some(packed_file) = iter.find(|x| x.get_path().starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                    if let Ok(table) = DB::read(&packed_file.get_data_and_keep_it().unwrap(), &format!("{}_tables", dependency_data.0), &schema) {
                        if let Some(column_index) = table.definition.fields.iter().position(|x| x.name == dependency_data.1) {
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
                if let Some(table) = iter.find(|x| x.name == format!("{}_tables", dependency_data.0)) {
                    if let Some(column_index) = table.definition.fields.iter().position(|x| x.name == dependency_data.1) {
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
                let iter = pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]);
                while let Some(packed_file) = iter.iter().find(|x| x.get_path().starts_with(&["db".to_owned(), format!("{}_tables", dependency_data.0)])) {
                    if let Ok(packed_file_data) = packed_file.get_data() {
                        if let Ok(table) = DB::read(&packed_file_data, &format!("{}_tables", dependency_data.0), &schema) {
                            if let Some(column_index) = table.definition.fields.iter().position(|x| x.name == dependency_data.1) {
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
            for packed_file in pack_file.get_ref_mut_packed_files_by_path_start(&["db".to_owned()]) {
                packed_file.load_data()?;
            }

            for packed_file in pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]) {
                if packed_file.get_path().starts_with(&["db".to_owned()]) {
                    if let Ok(db_data) = db::DB::read(&(packed_file.get_data().unwrap()), &packed_file.get_path()[1], &schema) {
                        let dep_data = get_dependency_data(&db_data.definition, &schema, &mut dep_db, &fake_dep_db, &pack_file);

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
                                broken_tables.push(format!("Table: {}/{}, Column/s: {}", &packed_file.get_path()[1], &packed_file.get_path()[2], columns)); 
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
    definition: &Definition,
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
                        FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?
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

/// This function imports a TSV file into a new DB File, using the provided schema to find out the correct format.
pub fn import_tsv_to_binary_file(
    schema: &Schema,
    source_path: &PathBuf,
    destination_path: &PathBuf,
) -> Result<()> {

    // We want the reader to have no quotes, tab as delimiter and custom headers, because otherwise
    // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(false)
        .has_headers(true)
        .flexible(true)
        .from_path(&source_path)?;

    // If we succesfully load the TSV file into a reader, check the first line to ensure it's a valid TSV file.
    let table_type;
    let table_version;
    {
        let headers = reader.headers()?;
        table_type = if let Some(table_type) = headers.get(0) { table_type.to_owned() } else { return Err(ErrorKind::ImportTSVWrongTypeTable)? };
        table_version = if let Some(table_version) = headers.get(1) { table_version.parse::<i32>().map_err(|_| Error::from(ErrorKind::ImportTSVInvalidVersion))? } else { return Err(ErrorKind::ImportTSVInvalidVersion)? };
    }

    // Get his definition depending on his first line's contents.
    let definition = if table_type == "Loc PackedFile" { schema.get_versioned_file_loc()?.get_version(table_version)?.clone() }
    else { schema.get_versioned_file_db(&table_type)?.get_version(table_version)?.clone() };

    // Try to import the entries of the file.
    let mut entries = vec![];
    for (row, record) in reader.records().enumerate() {
        if let Ok(record) = record {

            // The second line contains the column headers. Is just to help people in other programs, not needed to be check.
            if row == 0 { continue }

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
                        FieldType::Sequence(_) => return Err(ErrorKind::ImportTSVIncorrectRow(row, column))?
                    }
                }
                entries.push(entry);
            }

            // If it fails here, return an error with the len of the record instead a field.
            else { return Err(ErrorKind::ImportTSVIncorrectRow(row, record.len()))?; }
        }

        else { return Err(ErrorKind::ImportTSVIncorrectRow(row, 0))?; }
    }

    // If we reached this point without errors, we create the File in memory and add the entries to it.
    let data = if table_type == "Loc PackedFile" {
        let mut file = Loc::new();
        file.entries = entries;
        file.save()
    }
    else {
        let mut file = DB::new(&table_type, &definition);
        file.entries = entries;
        file.save()   
    };

    // Then, we try to write it on disk. If there is an error, report it.
    let mut file = BufWriter::new(File::create(&destination_path)?);
    file.write_all(&data)?;

    // If all worked, return success.
    Ok(())
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

/// This function creates a TSV file with the contents of the DB/Loc File.
pub fn export_tsv_from_binary_file(
    schema: &Schema,
    source_path: &PathBuf,
    destination_path: &PathBuf
) -> Result<()> {

    // We want the writer to have no quotes, tab as delimiter and custom headers, because otherwise
    // Excel, Libreoffice and all the programs that edit this kind of files break them on save.
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(QuoteStyle::Never)
        .has_headers(false)
        .flexible(true)
        .from_path(destination_path)?;

    // We don't know what type this file is, so we try to decode it as a Loc. If that fails, we try
    // to decode it as a DB using the name of his parent folder. If that fails too, run before it explodes! 
    let mut file = BufReader::new(File::open(source_path)?);
    let mut data = vec![];
    file.read_to_end(&mut data)?;

    let (table_type, version, entries) = if let Ok(data) = Loc::read(&data) { ("Loc PackedFile", 1, data.entries) }
    else {
        let table_type = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
        if let Ok(data) = DB::read(&data, table_type, schema) { (table_type, data.definition.version, data.entries) }
        else { return Err(ErrorKind::ImportTSVWrongTypeTable)? }
    };

    let definition = if table_type == "Loc PackedFile" { schema.get_versioned_file_loc()?.get_version(version)?.clone() }
    else { schema.get_versioned_file_db(&table_type)?.get_version(version)?.clone() };

    // We serialize the info of the table (name and version) in the first line, and the column names in the second one.
    writer.serialize((table_type, version))?;
    writer.serialize(definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>())?;

    // Then we serialize each entry in the DB Table.
    for entry in entries { writer.serialize(&entry)?; }
    writer.flush().map_err(From::from)
}

//----------------------------------------------------------------//
// Mass-TSV Functions for PackedFiles.
//----------------------------------------------------------------//

/// This function is used to Mass-Import TSV files into a PackFile. Note that this will OVERWRITE any
/// existing PackedFile that has a name conflict with the TSV files provided.
pub fn tsv_mass_import(
    tsv_paths: &[PathBuf],
    name: Option<String>,
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

            // Split the first line by \t so we can get the info of the table. Only if we have 2 items, continue.
            let tsv_info = line.split('\t').collect::<Vec<&str>>();
            if tsv_info.len() == 2 {

                // Get the type and the version of the table, and with that, get his definition.
                let table_type = tsv_info[0];
                let table_version = match tsv_info[1].parse::<i32>() {
                    Ok(version) => version,
                    Err(_) => {
                        error_files.push(path.to_string_lossy().to_string()); 
                        continue
                    }
                };
                
                let table_definition = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    schema.get_versioned_file_db(&table_type)?.get_version(table_version)?.clone()
                } else { error_files.push(path.to_string_lossy().to_string()); continue };

                // Then, import whatever we have and, depending on what we have, save it.
                match import_tsv(&table_definition, &path, &table_type, table_version) {
                    Ok(data) => {
                        match table_type {

                            // Loc Tables.
                            "Loc PackedFile" => {
                                let mut loc = Loc::new();
                                loc.entries = data;
                                let raw_data = loc.save();

                                // Depending on the name received, call it one thing or another.
                                let name = match name {
                                    Some(ref name) => name.to_string(),
                                    None => path.file_stem().unwrap().to_str().unwrap().to_string(),
                                };

                                let mut path = vec!["text".to_owned(), "db".to_owned(), format!("{}.loc", name)];

                                // If that path already exists in the list of new PackedFiles to add, change it using the index.
                                let mut index = 1;
                                while packed_files.iter().any(|x| x.get_path() == &*path) {
                                    path[2] = format!("{}_{}.loc", name, index);
                                    index += 1;
                                }

                                // If that path already exist in the PackFile, add it to the "remove" list.
                                if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                                // Create and add the new PackedFile to the list of PackedFiles to add.
                                packed_files.push(PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, raw_data));
                            }
        
                            // DB Tables.
                            _ => {
                                let mut db = DB::new(table_type, &table_definition);
                                db.entries = data;
                                let raw_data = db.save();

                                // Depending on the name received, call it one thing or another.
                                let name = match name {
                                    Some(ref name) => name.to_string(),
                                    None => path.file_stem().unwrap().to_str().unwrap().to_string(),
                                };

                                let mut path = vec!["db".to_owned(), table_type.to_owned(), name.to_owned()];
                        
                                // If that path already exists in the list of new PackedFiles to add, change it using the index.
                                let mut index = 1;
                                while packed_files.iter().any(|x| x.get_path() == &*path) {
                                    path[2] = format!("{}_{}", name, index);
                                    index += 1;
                                }
                                
                                // If that path already exists in the PackFile, add it to the "remove" list.
                                if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                                // Create and add the new PackedFile to the list of PackedFiles to add.
                                packed_files.push(PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, raw_data));
                            }
                        }
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
    let tree_path = packed_files.iter().map(|x| x.get_path().to_vec()).collect::<Vec<Vec<String>>>();

    // Remove all the "conflicting" PackedFiles from the PackFile, before adding the new ones.
    for packed_file_to_remove in &packed_files_to_remove {
        pack_file.remove_packed_file_by_path(packed_file_to_remove);
    }

    // We add all the files to the PackFile, and return success.
    pack_file.add_packed_files(&packed_files, true)?;
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

    // If the PackedFile is a DB Table and we have an schema, try to decode it and export it.
    match *SCHEMA.lock().unwrap() {
        Some(ref schema) => {

            let mut packed_files = pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]);
            packed_files.append(&mut pack_file.get_ref_packed_files_by_path_end(&[".loc".to_owned()]));
            for packed_file in &mut packed_files {

                // We check if his path is empty first to avoid false positives related with "starts_with" function.
                if !packed_file.get_path().is_empty() {

                    if packed_file.get_path().starts_with(&["db".to_owned()]) && packed_file.get_path().len() == 3 {
                        match DB::read(&(packed_file.get_data()?), &packed_file.get_path()[1], &schema) {
                            Ok(db) => {

                                // His name will be "db_name_file_name.tsv". If that's taken, we'll add an index until we find one available.
                                let mut name = format!("{}_{}.tsv", packed_file.get_path()[1], packed_file.get_path().last().unwrap().to_owned());
                                let mut export_path = export_path.to_path_buf();

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                let mut index = 1;
                                while exported_files.contains(&name) {
                                    name = format!("{}_{}_{}.tsv", packed_file.get_path()[1], packed_file.get_path().last().unwrap().to_owned(), index);
                                    index += 1;
                                }

                                export_path.push(name.to_owned());
                                let headers = db.definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>();
                                match export_tsv(&db.entries, &export_path, &headers, (&packed_file.get_path()[1], db.definition.version)) {
                                    Ok(_) => exported_files.push(name.to_owned()),
                                    Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                                }
                            }
                            Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                        }
                    }
                    
                    // Otherwise, we check if it's a Loc PackedFile, and try to decode it and export it.
                    else if packed_file.get_path().last().unwrap().ends_with(".loc") {
                        match Loc::read(&(packed_file.get_data()?)) {
                            Ok(loc) => {

                                // His name will be "file_name.tsv". If that's taken, we'll add an index until we find one available.
                                let mut name = format!("{}.tsv", packed_file.get_path().last().unwrap().to_owned());
                                let mut export_path = export_path.to_path_buf();

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                let mut index = 1;
                                while exported_files.contains(&name) {
                                    name = format!("{}_{}.tsv", packed_file.get_path().last().unwrap().to_owned(), index);
                                    index += 1;
                                }

                                export_path.push(name.to_owned());
                                let headers = schema.get_versioned_file_loc()?.get_version(1)?.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>();
                                match export_tsv(&loc.entries, &export_path, &headers, ("Loc PackedFile", 1)) {
                                    Ok(_) => exported_files.push(name.to_owned()),
                                    Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                                }
                            }
                            Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                        }
                    }
                }
            }
        }
        None => error_list.push(("".to_string(), Error::from(ErrorKind::SchemaNotFound))),
    }

    // If there has been errors, return ok with the list of errors.
    if !error_list.is_empty() {
        let error_files_string = error_list.iter().map(|x| format!("<li>{}</li>", x.0)).collect::<String>();
        Ok(format!("<p>All exportable files have been exported, except the following ones:</p><ul>{}</ul>", error_files_string))
    }

    // Otherwise, just return success and an empty error list.
    else { Ok("<p>All exportable files have been exported.</p>".to_owned()) }
}

/// This function is used to optimize the size of a PackFile. It does two things: removes unchanged rows
/// from tables (and if the table is empty, it removes it too) and it cleans the PackFile of extra .xml files 
/// often created by map editors. It requires just the PackFile to optimize and the dependency PackFile.
pub fn optimize_packfile(pack_file: &mut PackFile) -> Result<Vec<Vec<String>>> {
    
    // List of PackedFiles to delete. This includes empty DB Tables and empty Loc PackedFiles.
    let mut files_to_delete: Vec<Vec<String>> = vec![];

    // Get a list of every Loc and DB PackedFiles in our dependency's files. For performance reasons, we decode every one of them here.
    // Otherwise, they may have to be decoded multiple times, making this function take ages to finish. 
    let game_locs = DEPENDENCY_DATABASE.lock().unwrap().iter()
        .filter(|x| x.get_path().last().unwrap().ends_with(".loc"))
        .map(|x| x.get_data())
        .filter(|x| x.is_ok())
        .map(|x| Loc::read(&x.unwrap()))
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect::<Vec<Loc>>();

    let mut game_dbs = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
        DEPENDENCY_DATABASE.lock().unwrap().iter()
            .filter(|x| x.get_path().len() == 3 && x.get_path()[0] == "db")
            .map(|x| (x.get_data(), x.get_path()[1].to_owned()))
            .filter(|x| x.0.is_ok())
            .map(|x| (DB::read(&x.0.unwrap(), &x.1, &schema)))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<DB>>()
    } else { vec![] };

    // Due to precision issues with float fields, we have to round every float field from the tables to 3 decimals max.
    game_dbs.iter_mut().for_each(|x| x.entries.iter_mut()
        .for_each(|x| x.iter_mut()
        .for_each(|x| if let DecodedData::Float(data) = x { *data = (*data * 1000f32).round() / 1000f32 })
    ));

    let database_path_list = DEPENDENCY_DATABASE.lock().unwrap().iter().map(|x| x.get_path().to_vec()).collect::<Vec<Vec<String>>>();
    for packed_file in &mut pack_file.get_ref_mut_all_packed_files() {

        // Unless we specifically wanted to, ignore the same-name-as-vanilla files,
        // as those are probably intended to overwrite vanilla files, not to be optimized.
        if database_path_list.contains(&packed_file.get_path().to_vec()) && !SETTINGS.lock().unwrap().settings_bool["optimize_not_renamed_packedfiles"] { continue; }

        // If it's a DB table and we have an schema...
        if packed_file.get_path().len() == 3 && packed_file.get_path()[0] == "db" && !game_dbs.is_empty() {
            if let Some(ref schema) = *SCHEMA.lock().unwrap() {

                // Try to decode our table.
                let mut optimized_table = match DB::read(&(packed_file.get_data_and_keep_it()?), &packed_file.get_path()[1], &schema) {
                    Ok(table) => table,
                    Err(_) => continue,
                };

                // We have to round our floats too.
                optimized_table.entries.iter_mut()
                    .for_each(|x| x.iter_mut()
                    .for_each(|x| if let DecodedData::Float(data) = x { *data = (*data * 1000f32).round() / 1000f32 })
                );

                // For each vanilla DB Table that coincide with our own, compare it row by row, cell by cell, with our own DB Table. Then delete in reverse every coincidence.
                for game_db in &game_dbs {
                    if game_db.name == optimized_table.name && game_db.definition.version == optimized_table.definition.version {
                        let rows_to_delete = optimized_table.entries.iter().enumerate().filter(|(_, entry)| game_db.entries.contains(entry)).map(|(row, _)| row).collect::<Vec<usize>>();
                        for row in rows_to_delete.iter().rev() {
                            optimized_table.entries.remove(*row);
                        } 
                    }
                }

                // Save the data to the PackFile and, if it's empty, add it to the deletion list.
                packed_file.set_data(optimized_table.save());
                if optimized_table.entries.is_empty() { files_to_delete.push(packed_file.get_path().to_vec()); }
            }

            // Otherwise, we just check if it's empty. In that case, we delete it.
            else if let Ok((_, _, entry_count, _)) = DB::get_header(&(packed_file.get_data()?)) {
                if entry_count == 0 { files_to_delete.push(packed_file.get_path().to_vec()); }
            }
        }

        // If it's a Loc PackedFile and there are some Locs in our dependencies...
        else if packed_file.get_path().last().unwrap().ends_with(".loc") && !game_locs.is_empty() {

            // Try to decode our Loc. If it's empty, skip it and continue with the next one.
            let mut optimized_loc = match Loc::read(&(packed_file.get_data_and_keep_it()?)) {
                Ok(loc) => if !loc.entries.is_empty() { loc } else { continue },
                Err(_) => continue,
            };

            // For each vanilla Loc, compare it row by row, cell by cell, with our own Loc. Then delete in reverse every coincidence.
            for game_loc in &game_locs {
                let rows_to_delete = optimized_loc.entries.iter().enumerate().filter(|(_, entry)| game_loc.entries.contains(entry)).map(|(row, _)| row).collect::<Vec<usize>>();
                for row in rows_to_delete.iter().rev() {
                    optimized_loc.entries.remove(*row);
                } 
            }

            // Save the data to the PackFile and, if it's empty, add it to the deletion list.
            packed_file.set_data(optimized_loc.save());
            if optimized_loc.entries.is_empty() { files_to_delete.push(packed_file.get_path().to_vec()); }
        }
    }

    // If there are files to delete, get his type and delete them
    if !files_to_delete.is_empty() {
        for tree_path in &files_to_delete {
            pack_file.remove_packed_file_by_path(tree_path);
        }
    }

    // Return the deleted file's types.
    Ok(files_to_delete)
}