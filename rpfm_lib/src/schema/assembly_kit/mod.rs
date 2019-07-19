//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here it goes all the code related with the schema integration with the Assembly Kit.
// And by `integration` I mean the code that parses Assembly Kit tables and schemas 
// to a format we can actually read, and everything else related to the Assembly Kit.

// Here will go all the code needed for the parsing of raw table files and their fake schemas, and putting them into pak files.
// There are multiple types of tables due to CA changing their format for them:
// V0: Empire and Napoleon.
// V1: Shogun 2.
// V2: Anything since Rome 2.

use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;
use bincode;

use std::fs::{File, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::common::*;
use rpfm_error::{Result, Error, ErrorKind};
use crate::packedfile::db::DB;
use crate::packedfile::DecodedData;
use crate::schema::*;
use crate::DEPENDENCY_DATABASE;
use crate::GAME_SELECTED;
use crate::SUPPORTED_GAMES;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit Schema Files into.
//---------------------------------------------------------------------------//

/// This is the base `table definition` file. In files, this is the equivalent to a `TWaD_` file. 
/// It contains a vector with all the fields that forms it.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct root {
    pub field: Vec<field>,
}

/// This struct holds a decoded `field` structure.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct field {
    pub primary_key: String,
    pub name: String,
    pub field_type: String,
    pub required: String,
    pub max_length: Option<String>,

    // There can be multiple source_columns, but we just take the first one. The second one is usually the lookup one,
    // and we don't support lookup columns yet.
    pub column_source_column: Option<Vec<String>>,
    pub column_source_table: Option<String>,
    pub field_description: Option<String>,
}

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit DB Files into.
//---------------------------------------------------------------------------//

/// This is the base `table data` file. In files, this is the equivalent to the xml file with all the data in the table.
/// It contains a vector with all the rows of data in the xml table file.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct dataroot {
	pub rows: Vec<datarow>,
}

/// This is the "datarow", for decoding rows of data.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct datarow {
    pub datafield: Vec<datafield>,
}

/// This is the "datafield", for decoding data of fields.
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

/// This function process all the tables from the game's raw table folder and it turns them into a single processed file,
/// as fake tables with version -1. That will allow us to use them for dependency checking and for populating combos.
pub fn process_raw_tables(
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
                            result = File::open(&file).map_err(|error| From::from(error));
                            break;
                        }
                    }

                    // In case it fails at finding the data file, ignore that schema.
                    if result.is_err() { continue; } else { result }
                }?;

                // If the table already exist in the data.pack, skip it.
                let mut exist = false;
                for table in &*dep_db {
                    if table.path[1] == table_name {
                        exist = true;
                        break;
                    }
                }

                if exist { continue; }

                // Then deserialize the definition of the table into something we can use.
                let imported_definition: root = from_reader(definition_file)?;
                let imported_table_definition = TableDefinition::new_fake_from_assembly_kit(&imported_definition, -1, &table_name);

                // Before deserializing the data, due to limitations of serde_xml_rs, we have to rename all rows, beacuse unique names for
                // rows in each file is not supported for deserializing. Same for the fields, we have to change them to something more generic.
                let mut buffer = String::new();
                data_file.read_to_string(&mut buffer)?;
                buffer = buffer.replace(&format!("<{} record_uuid", file_name_no_xml), "<rows record_uuid"); 
                buffer = buffer.replace(&format!("<{}>", file_name_no_xml), "<rows>"); 
                buffer = buffer.replace(&format!("</{}>", file_name_no_xml), "</rows>");
                for field in &imported_table_definition.fields {
                    let field_name_regex = Regex::new(&format!("\n<{}>", field.field_name)).unwrap();
                    let field_name_regex2 = Regex::new(&format!("\n<{} .+?\">", field.field_name)).unwrap();
                    buffer = field_name_regex.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.field_name)).to_string();
                    buffer = field_name_regex2.replace_all(&buffer, &*format!("\n<datafield field_name=\"{}\">", field.field_name)).to_string();
                    buffer = buffer.replace(&format!("</{}>", field.field_name), "</datafield>");
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
                                if field_def.field_name == field.field_name {
                                    exists = true;
                                    entry.push(match field_def.field_type {
                                        FieldType::Boolean => DecodedData::Boolean(if field.field_data == "true" || field.field_data == "1" { true } else { false }),
                                        FieldType::Float => DecodedData::Float(if let Ok(data) = field.field_data.parse::<f32>() { data } else { 0.0 }),
                                        FieldType::Integer => DecodedData::Integer(if let Ok(data) = field.field_data.parse::<i32>() { data } else { 0 }),
                                        FieldType::LongInteger => DecodedData::LongInteger(if let Ok(data) = field.field_data.parse::<i64>() { data } else { 0 }),
                                        FieldType::StringU8 => DecodedData::StringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::StringU16 => DecodedData::StringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
                                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(if field.field_data == "Frodo Best Waifu" { String::new() } else { field.field_data.to_string() }),
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
                    let mut processed_db_file = DB::new(&table_name, -1, &imported_table_definition);
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

/// This function is the response to our prayers. It takes the Assembly Kit's DB Files to create basic definitions of each 
/// undecoded table from the folder you provide it.
/// 
/// It requires:
/// - schema: The schema where all the definitions will be put. None to put all the definitions into a new schema.
/// - assembly_kit_schemas_path: this is the path with the TWaD_*****.xml syntax. They are usually in GameFolder/assembly_kit/raw_data/db/.
/// - db_binary_path: this is a path containing all the tables extracted from the game we want the schemas. It should have xxx_table/table inside.
pub fn import_schema(
    schema: Option<Schema>,
    assembly_kit_schemas_path: &PathBuf,
    db_binary_path: &PathBuf,
) -> Result<()> {

    // Get the schema, then get all the raw schema files.
    let mut schema = if let Some(schema) = schema { schema } else { Schema::new() };
    let assembly_kit_schemas = get_raw_definitions(assembly_kit_schemas_path, 2).unwrap();
    for path in &assembly_kit_schemas {

        // Always print his path. If it breaks, we want to know where.
        println!("{:?}", path);

        // We read the file and deserialize it as a `root`.
        let file = File::open(&path).expect("Couldn't open file");
        let imported_table_definition: root = from_reader(file).unwrap();

        // Get his name and version. We only add it if his table actually exists.
        let mut file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let table_name = format!("{}_tables", file_name.split_off(5));
        let mut db_binary_path = db_binary_path.clone();
        db_binary_path.push(table_name.to_owned());
        match get_files_from_subdir(&db_binary_path) {
            Ok(db_file_path) => {

                // If we found something...
                if !db_file_path.is_empty() {
                    match File::open(&db_file_path[0]) {
                        Ok(ref mut file) => {

                            // Read the table...
                            let mut pack_file_buffered = vec![];
                            file.read_to_end(&mut pack_file_buffered).expect("Error reading file.");

                            // Get his version and, if there is not a table with that version in the current schema, add it. Otherwise, ignore it.
                            let version = DB::get_header_data(&pack_file_buffered).unwrap().0;

                            if let Ok(ref mut versioned_file) = schema.get_mut_versioned_file_db(&table_name) {
                                if versioned_file.get_version(version).is_err() {
                                    let table_definition = TableDefinition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                                    versioned_file.add_version(&table_definition);
                                } else {
                                    continue;
                                }
                            }

                            else {
                                let table_definition = TableDefinition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                                let versioned_file = VersionedFile::DB(table_name, vec![table_definition]);
                                schema.add_versioned_file(&versioned_file);
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Schema::save(&schema, "schema_wh.json")?;

    Ok(())
}
