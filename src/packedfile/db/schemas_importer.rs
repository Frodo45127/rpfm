// This is just a helper to get the schemas from the assembly kit. This is NOT INTENDED to work in
// runtime, so we just wired up when we need to create a new schema from scratch.

extern crate serde_xml_rs;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use self::serde_xml_rs::deserialize;
use super::DBHeader;
use super::schemas::*;
use ::common;

/// This is the base "table" file. From here we just want to save the field vector.
#[derive(Debug, Deserialize)]
pub struct root {
    pub field: Vec<field>,
}

/// This is the "field" fields decoded.
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

/// This function creates an schema, and decodes all the tables from the folder we say it into it.
pub fn import_schema() {

    // We get the new schema.
    let mut schema = Schema::new(format!("TW:Warhammer 2"));

    // Then we get all the schema files. We unwrap it, as we want it to crash oon error.
    let assembly_kit_schemas = common::get_assembly_kit_schemas(&PathBuf::from("/home/frodo45127/schema_stuff/db_schemas/")).unwrap();

    // For each file...
    for path in assembly_kit_schemas.iter() {

        // We just do this in Debug builds, so we use a print to check when a table throws an error.
        println!("{:?}", path);

        // We read the file and deserialize it...
        let mut file = File::open(&path).expect("Couldn't open file");
        let imported_table_definition: root = deserialize(file).unwrap();

        // Then we create a new table_definitions, a new imported table definition, and add it to the schema.
        let mut file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let table_name = format!("{}_tables", file_name.split_off(5));

        // We need it's version too, so... We only add it if his table actually exists.
        match common::get_files_from_subdir(&PathBuf::from(format!("/home/frodo45127/schema_stuff/db_tables/{}/", table_name))) {
            Ok(db_file_path) => {

                // If we found something...
                if db_file_path.len() > 0 {
                    match File::open(&db_file_path[0]) {
                        Ok(ref mut file) => {

                            // Read the table...
                            let mut pack_file_buffered = vec![];
                            file.read_to_end(&mut pack_file_buffered).expect("Error reading file.");

                            // Get it's version...
                            let header = DBHeader::read(pack_file_buffered).unwrap();
                            let version = header.0.packed_file_header_packed_file_version;

                            // And add it to the schema.
                            schema.add_table_definitions(TableDefinitions::new(&table_name));
                            let index = schema.get_table_definitions(&table_name).unwrap();
                            schema.tables_definitions[index].add_table_definition(TableDefinition::new_from_assembly_kit(imported_table_definition, version, table_name));

                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Schema::save(&schema);
}