//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is just a helper to get the schemas from the assembly kit. This is NOT INTENDED to work in
// runtime, so we just wired up when we need to create a new schema from scratch.

use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::error::Result;
use super::DB;
use super::schemas::*;
use crate::common::*;

/// This is the base "table" file. From here we just want to save the field vector.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct root {
    pub field: Vec<field>,
}

/// This is the "field" fields decoded.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct field {
    pub primary_key: String,
    pub name: String,
    pub field_type: String,
    pub required: String,
    pub max_length: Option<String>,
    // There can be multiple source_columns, but we just take the first one.
    pub column_source_column: Option<Vec<String>>,
    pub column_source_table: Option<String>,
    pub field_description: Option<String>,
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
                            if let Some(ref mut table_definitions) = schema.tables_definitions.iter_mut().find(|x| x.name == table_name) {
                                if table_definitions.versions.iter().find(|x| x.version == version).is_some() {
                                    continue;
                                }
                                else {
                                    let table_definition = TableDefinition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                                    table_definitions.add_table_definition(table_definition);
                                }
                            }

                            else {
                                let mut table_definitions = TableDefinitions::new(&table_name);
                                let table_definition = TableDefinition::new_from_assembly_kit(&imported_table_definition, version, &table_name);
                                table_definitions.add_table_definition(table_definition);
                                schema.add_table_definitions(table_definitions);
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
