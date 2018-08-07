// This is just a helper to get the schemas from the assembly kit. This is NOT INTENDED to work in
// runtime, so we just wired up when we need to create a new schema from scratch.
extern crate serde_xml_rs;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use error::Result;
use self::serde_xml_rs::deserialize;
use super::DBHeader;
use super::schemas::*;
use common::*;

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

/// This function creates an schema, and decodes all the tables from the folder we say it into it.
/// This is intended to use just after compilation, providing the needed folders as const from the
/// main.rs file. The paths are:
/// - assembly_kit_schemas_path: this is the path with the TWaD_*****.xml syntax. They are usually in GameFolder/assembly_kit/raw_data/db/.
/// - testing_tables_path: this is a path containing all the tables extracted from the game we want the schemas. It should have xxx_table/table.
pub fn import_schema(
    assembly_kit_schemas_path: &PathBuf,
    testing_tables_path: &PathBuf,
    rpfm_path: &PathBuf,
) -> Result<()> {

    // We get the new schema.
    let mut schema = Schema::new();

    // Then we get all the schema files. We unwrap it, as we want it to crash oon error.
    let assembly_kit_schemas = get_assembly_kit_schemas(assembly_kit_schemas_path).unwrap();

    // For each file...
    for path in &assembly_kit_schemas {

        // We just do this in Debug builds, so we use a print to check when a table throws an error.
        println!("{:?}", path);

        // We read the file and deserialize it...
        let file = File::open(&path).expect("Couldn't open file");
        let imported_table_definition: root = deserialize(file).unwrap();

        // Then we create a new table_definitions, a new imported table definition, and add it to the schema.
        let mut file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let table_name = format!("{}_tables", file_name.split_off(5));

        // We need it's version too, so... We only add it if his table actually exists.
        let mut testing_tables_path = testing_tables_path.clone();
        testing_tables_path.push(table_name.to_owned());
        match get_files_from_subdir(&testing_tables_path) {
            Ok(db_file_path) => {

                // If we found something...
                if !db_file_path.is_empty() {
                    match File::open(&db_file_path[0]) {
                        Ok(ref mut file) => {

                            // Read the table...
                            let mut pack_file_buffered = vec![];
                            file.read_to_end(&mut pack_file_buffered).expect("Error reading file.");

                            // Get it's version...
                            let header = DBHeader::read(&pack_file_buffered, &mut 0).unwrap();
                            let version = header.version;

                            // And add it to the schema.
                            schema.add_table_definitions(TableDefinitions::new(&table_name));
                            let index = schema.get_table_definitions(&table_name).unwrap();
                            schema.tables_definitions[index].add_table_definition(TableDefinition::new_from_assembly_kit(&imported_table_definition, version, &table_name));

                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Schema::save(&schema, rpfm_path, "PFH5")?;

    Ok(())
}
