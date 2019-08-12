//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with the Assembly Kit's DB Files and Schemas.

This module contains all the code related with the *schema integration* with the Assembly Kit.
And by *integration* I mean the code that parses Assembly Kit tables and schemas to a format we can actually read.

Also, here is the code responsible for the creation of fake schemas from Assembly Kit files, and for putting them into PAK (Processed Assembly Kit) files.
For more information about PAK files, check the `generate_pak_file()` function. There are multiple types of Assembly Kit table files due to CA changing their format:
- `0`: Empire and Napoleon.
- `1`: Shogun 2.
- `2`: Anything since Rome 2.

Currently, due to the complexity of parsing the table type `0`, we don't have support for PAK files in Empire and Napoleon.
!*/

use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::{File, DirBuilder, read_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rpfm_error::{Result, Error, ErrorKind};

use crate::{DEPENDENCY_DATABASE, GAME_SELECTED, SCHEMA, SUPPORTED_GAMES};
use crate::common::*;
use crate::packfile::PackFile;
use crate::packedfile::table::db::DB;
use crate::packedfile::DecodedData;

use crate::schema::*;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit Schema Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to a `Definition` struct. In files, this is the equivalent to a `TWaD_` file.
/// 
/// It contains a vector with all the fields that forms it.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct root {
    pub field: Vec<field>,
}

/// This is the raw equivalent to a `Field` struct.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct field {

    /// Ìf the field is primary key. `1` for `true`, `0` for false.
    pub primary_key: String,

    /// The name of the field.
    pub name: String,

    /// The type of the field in the Assembly Kit.
    pub field_type: String,

    /// If the field is required or can be blank.
    pub required: String,

    /// The default value of the field.
    pub default_value: Option<String>,

    /// The max allowed lenght for the data in the field.
    pub max_length: Option<String>,

    /// If the field's data corresponds to a filename.
    pub is_filename: Option<String>,

    /// Path where the file in the data of the field can be, if it's restricted to one path.
    pub filename_relative_path: Option<String>,

    /// No idea, but for what I saw, it's not useful for modders.
    pub fragment_path: Option<String>,

    /// Reference source column. First one is the referenced column, the rest, if exists, are the lookup columns concatenated.
    pub column_source_column: Option<Vec<String>>,

    /// Reference source table.
    pub column_source_table: Option<String>,

    /// Description of what the field does.
    pub field_description: Option<String>,

    /// If it has to be exported for the encyclopaedia? No idea really. `1` for `true`, `0` for false.
    pub encyclopaedia_export: Option<String>
}

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit DB Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to the `entries` field in a `DB` struct. In files, this is the equivalent to the `.xml` file with all the data in the table.
///
/// It contains a vector with all the rows of data in the `.xml` table file.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct dataroot {
	pub rows: Vec<datarow>,
}

/// This is the raw equivalent to a row of data from a DB file.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct datarow {
    pub datafield: Vec<datafield>,
}

/// This is the raw equivalent to a `DecodedData`.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct datafield {
    pub field_name: String,

    #[serde(rename = "$value")]
    pub field_data: String,
}

//---------------------------------------------------------------------------//
// Functions to process the Raw DB Tables from the Assembly Kit.
//---------------------------------------------------------------------------//

/// This function generates a PAK (Processed Assembly Kit) file from the raw tables found in the provided path.
///
/// This works by processing all the tables from the game's raw table folder and turning them into a single processed file,
/// as fake tables with version -1. That will allow us to use them for dependency checking and for populating combos.
///
/// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK file.
pub fn generate_pak_file(
    raw_db_path: &PathBuf,
    version: i16,
) -> Result<()> {

    // We get all the files to load.
    let definitions = get_raw_definitions(raw_db_path, version)?;
    let data = get_raw_data(raw_db_path, version)?;
    let mut processed_db_files = vec![];
    let dep_db = DEPENDENCY_DATABASE.lock().unwrap();

    // For each file, create a DB file from it.
    for definition in &definitions {

        // If we have a debug version, print each table we process so, if it fails, we know where.
        if cfg!(debug_assertions) { println!("{:?}", definition); }

        // Depending on the version, we have to use one logic or another.
        match version {

            // Version 2 is Rome 2+. Version 1 is Shogun 2. Almost the same format, but we have to
            // provide a different path for Shogun 2, so it has his own version.
            2 | 1 => {

                // We read both files (TWad and Table) and get them to memory.
                let file_name = definition.file_name().unwrap().to_str().unwrap().split_at(5).1;
                let file_name_no_xml = file_name.split_at(file_name.len() - 4).0;
                let table_name = format!("{}_tables", file_name_no_xml);
                
                // This file is present in Rome 2, Attila and Thrones. It's almost 400mb. And we don't need it.
                if file_name == "translated_texts.xml" { continue; }
                
                let definition_file = File::open(&definition).unwrap();
                let mut data_file = {
                    let mut result = Err(Error::from(ErrorKind::IOFileNotFound));
                    for file in &data {
                        if file.file_name().unwrap().to_str().unwrap() == file_name {
                            result = File::open(&file).map_err(From::from);
                            break;
                        }
                    }

                    // In case it fails at finding the data file, ignore that schema.
                    if result.is_err() { continue; } else { result }
                }?;

                // If the table already exist in the data.pack, skip it.
                let mut exist = false;
                for table in &*dep_db {
                    if table.get_ref_raw().get_path()[1] == table_name {
                        exist = true;
                        break;
                    }
                }

                if exist { continue; }

                // Then deserialize the definition of the table into something we can use.
                let imported_definition: root = from_reader(definition_file)?;
                let imported_table_definition = Definition::new_fake_from_assembly_kit(&imported_definition, &table_name);

                // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, beacuse unique names for
                // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
                let mut buffer = String::new();
                data_file.read_to_string(&mut buffer)?;
                buffer = buffer.replace(&format!("<{} record_uuid", file_name_no_xml), "<rows record_uuid"); 
                buffer = buffer.replace(&format!("<{}>", file_name_no_xml), "<rows>"); 
                buffer = buffer.replace(&format!("</{}>", file_name_no_xml), "</rows>");
                for field in &imported_table_definition.fields {
                    let field_name_regex = Regex::new(&format!("\n<{}>", field.name)).unwrap();
                    let field_name_regex2 = Regex::new(&format!("\n<{} .+?\">", field.name)).unwrap();
                    buffer = field_name_regex.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.name)).to_string();
                    buffer = field_name_regex2.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.name)).to_string();
                    buffer = buffer.replace(&format!("</{}>", field.name), "</datafield>");
                }

                // Serde shits itself if it sees an empty field, so we have to work around that.
                let field_data_regex1 = Regex::new("\"></datafield>").unwrap();
                let field_data_regex2 = Regex::new("\"> </datafield>").unwrap();
                let field_data_regex3 = Regex::new("\">  </datafield>").unwrap();
                buffer = field_data_regex1.replace_all(&buffer, "\">Frodo Best Waifu</datafield>").to_string();
                buffer = field_data_regex2.replace_all(&buffer, "\"> Frodo Best Waifu</datafield>").to_string();
                buffer = field_data_regex3.replace_all(&buffer, "\">  Frodo Best Waifu</datafield>").to_string();
                
                // Only if the table has data we deserialize it.
                if buffer.contains("</rows>\r\n</dataroot>") {
                    //if cfg!(debug_assertions) { println!("{}", buffer); }
                    let imported_data: dataroot = from_reader(buffer.as_bytes())?;

                    // Now we get that mess we've created and make readable data from it.
                    let mut entries = vec![];
                    for row in &imported_data.rows {
                        let mut entry = vec![];

                        // Some games (Thrones, Attila, Rome 2 and Shogun 2) may have missing fields when said field is empty.
                        // To compensate it, if we don't find a field from the definition in the table, we add it empty.
                        for field_def in &imported_table_definition.fields {
                            let mut exists = false;
                            for field in &row.datafield {
                                if field_def.name == field.field_name {
                                    exists = true;
                                    entry.push(match field_def.field_type {
                                        FieldType::Boolean => DecodedData::Boolean(field.field_data == "true" || field.field_data == "1"),
                                        FieldType::Float => DecodedData::Float(if let Ok(data) = field.field_data.parse::<f32>() { data } else { 0.0 }),
                                        FieldType::Integer => DecodedData::Integer(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                                        FieldType::LongInteger => DecodedData::LongInteger(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                                        FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        
                                        // This type is not used in the raw tables so, if we find it, we skip it.
                                        FieldType::Sequence(_) => continue,
                                    });
                                    break;
                                }
                            }

                            // If the field doesn't exist, we create it empty.
                            if !exists {
                                entry.push(DecodedData::OptionalStringU8(String::new()));
                            }
                        }
                        entries.push(entry);
                    }

                    // Then create the DB object, and add it to the list.
                    let mut processed_db_file = DB::new(&table_name, &imported_table_definition);
                    processed_db_file.entries = entries;
                    processed_db_files.push(processed_db_file);
                }

                // Otherwise skip it.
                else { continue; }
            },

            // Version 0 is Napoleon and Empire. These two don't have an assembly kit, but CA released years ago their table files.
            // So... these are kinda unique. The schemas are xsd files, and the data format is kinda different.
            0 => continue,

            // Any other version is unsupported or a game without Assembly Kit.
            _ => {}
        }
    }

    // Save our new PAK File where it should be.
    let mut pak_path = get_config_path()?;
    let game_selected = GAME_SELECTED.lock().unwrap();
    let pak_name = SUPPORTED_GAMES.get(&**game_selected).unwrap().pak_file.clone().unwrap();
    pak_path.push("pak_files");
    
    DirBuilder::new().recursive(true).create(&pak_path)?;
    pak_path.push(pak_name);

    let mut file = File::create(pak_path)?;
    file.write_all(&bincode::serialize(&processed_db_files)?)?;

    // If we reach this point, return success.
    Ok(())   
}

/// This function uses the provided Assembly Kit path to *complete* our schema's missing data.
///
/// It takes the Assembly Kit's DB Files and matches them against our own schema files, filling missing info, 
/// or even generating new definitions if there are none for the tables.
/// 
/// It requires:
/// - schema: The schema where all the definitions will be put. None to put all the definitions into a new schema.
/// - assembly_kit_schemas_path: this is the path with the TWaD_*****.xml syntax. They are usually in GameFolder/assembly_kit/raw_data/db/.
/// - db_binary_path: this is a path containing all the tables extracted from the game we want the schemas. It should have xxx_table/table inside.
pub fn import_schema_from_raw_files(ass_kit_path: Option<PathBuf>) -> Result<()> {
    if let Some(mut schema) = SCHEMA.lock().unwrap().clone() {
        
        // This has to do a different process depending on the `raw_db_version`.
        let raw_db_version = SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].raw_db_version;
        match raw_db_version {
            2 | 1 => {
                let packfile_db_path = get_game_selected_db_pack_path(&**GAME_SELECTED.lock().unwrap()).ok_or_else(|| Error::from(ErrorKind::SchemaNotFound))?;
                let packfile_db = PackFile::open_packfiles(&packfile_db_path, true, false, false)?;

                let mut ass_kit_schemas_path = 
                    if raw_db_version == 1 { 
                        if let Some(path) = ass_kit_path { path }
                        else { return Err(ErrorKind::SchemaNotFound)? }
                    }
                    else if let Some(path) = get_game_selected_assembly_kit_path(&**GAME_SELECTED.lock().unwrap()) { path }
                    else { return Err(ErrorKind::SchemaNotFound)? };
                    
                ass_kit_schemas_path.push("raw_data");
                ass_kit_schemas_path.push("db");

                for path in &get_raw_definitions(&ass_kit_schemas_path, raw_db_version)? {

                    // Always print his path. If it breaks, we want to know where.
                    println!("{:?}", path);

                    // We read the file and deserialize it as a `root`.
                    let file = BufReader::new(File::open(&path)?);
                    let imported_table_definition: root = from_reader(file).unwrap();

                    // Get his name and version. We only add it if his table actually exists.
                    let mut file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    let table_name = format!("{}_tables", file_name.split_off(5));

                    // Get his version and, if there is not a table with that version in the current schema, add it. Otherwise, ignore it.
                    let packed_files = packfile_db.get_ref_packed_files_by_path_start(&["db".to_owned(), table_name.to_owned()]);
                    if !packed_files.is_empty() {
                        let packed_file = packed_files[0];
                        let version = DB::get_header(&packed_file.get_ref_raw().get_data()?).unwrap().0;

                        if let Ok(ref mut versioned_file) = schema.get_mut_versioned_file_db(&table_name) {
                            if versioned_file.get_version(version).is_err() {
                                let table_definition = Definition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                                versioned_file.add_version(&table_definition);
                            } else {
                                continue;
                            }
                        }

                        else {
                            let table_definition = Definition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                            let versioned_file = VersionedFile::DB(table_name, vec![table_definition]);
                            schema.add_versioned_file(&versioned_file);
                        }
                    }
                }

                Schema::save(&schema, &SUPPORTED_GAMES[&**GAME_SELECTED.lock().unwrap()].schema)?;

                Ok(())
            }
            0 => { Err(ErrorKind::SchemaNotFound)? }
            _ => { Err(ErrorKind::SchemaNotFound)? }
        }
    }

    else { Err(ErrorKind::SchemaNotFound)? }
}

//---------------------------------------------------------------------------//
// Utility functions to process raw files from the Assembly Kit.
//---------------------------------------------------------------------------//

/// This function returns you all the raw Assembly Kit Table Definition files from the provided folder.
pub fn get_raw_definitions(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        
                        // If it's a file and starts with "TWaD_", to the file_list it goes (except if it's one of those special files).
                        if version == 1 || version == 2 {
                            if file_path.is_file() &&
                                file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_") &&
                                !file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_TExc") &&
                                file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_schema_validation" &&
                                file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_relationships" &&
                                file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_validation" &&
                                file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_tables" &&
                                file_path.file_stem().unwrap().to_str().unwrap() != "TWaD_queries" {
                                file_list.push(file_path);
                            }
                        }

                        // In this case, we just catch all the xsd files on the folder.
                        else if version == 0 && 
                            file_path.is_file() &&
                            file_path.file_stem().unwrap().to_str().unwrap().to_string().ends_with(".xsd") {
                            file_list.push(file_path);   
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()))?,
                }
            }
        }
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()))?,
    }

    // Sort the files alphabetically.
    file_list.sort();
    Ok(file_list)
}

/// This function returns you all the raw Assembly Kit Table Data files from the provided folder.
pub fn get_raw_data(current_path: &Path, version: i16) -> Result<Vec<PathBuf>> {

    let mut file_list: Vec<PathBuf> = vec![];
    match read_dir(current_path) {

        // If we don't have any problems reading it...
        Ok(files_in_current_path) => {
            for file in files_in_current_path {
                match file {
                    Ok(file) => {
                        let file_path = file.path();
                        
                        // If it's a file and it doesn't start with "TWaD_", to the file_list it goes.
                        if version == 1 || version == 2 {
                            if file_path.is_file() && !file_path.file_stem().unwrap().to_str().unwrap().to_string().starts_with("TWaD_") {
                                file_list.push(file_path);
                            }
                        }

                        // In this case, if it's an xml, to the file_list it goes.
                        else if version == 0 &&
                            file_path.is_file() && 
                            !file_path.file_stem().unwrap().to_str().unwrap().to_string().ends_with(".xml") {
                            file_list.push(file_path);
                        }
                    }
                    Err(_) => return Err(ErrorKind::IOReadFile(current_path.to_path_buf()))?,
                }
            }
        }
        Err(_) => return Err(ErrorKind::IOReadFolder(current_path.to_path_buf()))?,
    }

    // Sort the files alphabetically.
    file_list.sort();
    Ok(file_list)
}
