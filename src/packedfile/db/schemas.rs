// In this file goes all the stuff needed for the schema decoder to work.
extern crate serde_json;

use std::path::PathBuf;
use std::error;
use std::fs::File;
use std::io::{
    Write, Error, ErrorKind
};

/// This struct holds the entire schema for the currently selected game (by "game" I mean the PackFile
/// Type).
/// It has:
/// - game: the game for what the loaded definitions are intended.
/// - version: custom variable to keep track of the updates to the schema.
/// - tables_definition: the actual definitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub game: String,
    pub version: u32,
    pub tables_definitions: Vec<TableDefinitions>,
}

/// This struct holds the definitions for a table. It has:
/// - name: the name of the table.
/// - versions: the different versions this table has.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableDefinitions {
    pub name: String,
    pub versions: Vec<TableDefinition>,
}

/// This struct holds the definitions for a version of a table. It has:
/// - version: the version of the table these definitions are for.
/// - fields: the different fields this table has.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableDefinition {
    pub version: u32,
    pub fields: Vec<Field>,
}

/// This struct holds the type of a field of a table. It has:
/// - field_name: the name of the field.
/// - field_is_key: true if the field is a key field and his column needs to be put in the beginning of the TreeView.
/// - field_is_reference: if this field is a reference of another, this has (table name, field name).
/// - field_type: the type of the field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub field_name: String,
    pub field_is_key: bool,
    pub field_is_reference: Option<(String, String)>,
    pub field_type: FieldType,
}

/// Enum FieldType: This enum is used to define the possible types of a field in the schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    Float,
    Integer,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
}

/// Implementation of "Schema"
impl Schema {

    /// This function creates a new schema. It should only be needed to create the first table definition
    /// of a game, as the rest will continue using the same schema.
    pub fn new() -> Schema {
        let game = format!("Warhammer 2");
        let version = 1u32;
        let tables_definitions = vec![];

        Schema {
            game,
            version,
            tables_definitions,
        }
    }

    /// This function adds a new TableDefinitions to the schema.
    pub fn add_table_definitions(&mut self, table_definitions: &TableDefinitions) {
        self.tables_definitions.push(table_definitions.clone());
    }

    /// This functions returns the index of the definitions for a table.
    pub fn get_table_definitions(&self, table_name: &str) -> Option<usize> {
        for (index, table_definitions) in self.tables_definitions.iter().enumerate() {
            if table_definitions.name == table_name {
               return Some(index);
            }
        }
        None
    }

    /// This function takes an schema file and reads it into a "Schema" object.
    pub fn load() -> Result<Schema, Error> {
        let schema_file = File::open("schema_wh2.json")?;
        let schema = serde_json::from_reader(schema_file)?;
        Ok(schema)
    }

    /// This function takes an "Schema" object and saves it into a schema file.
    pub fn save(schema: &Schema) -> Result<(), Error> {
        let schema_json = serde_json::to_string_pretty(schema);
        match File::create(PathBuf::from("schema_wh2.json")) {
            Ok(mut file) => {
                match file.write_all(&schema_json.unwrap().as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string()))
                }
            },
            Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string()))
        }
    }
}

/// Implementation of "TableDefinitions"
impl TableDefinitions {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode.
    pub fn new(name: &str, version: u32) -> TableDefinitions {
        let name = name.to_string();
        let versions = vec![TableDefinition::new(version)];

        TableDefinitions {
            name,
            versions,
        }
    }

    /// This functions returns the index of the definitions for a table.
    pub fn get_table_version(&self, table_version: u32) -> Option<usize> {
        for (index, table) in self.versions.iter().enumerate() {
            if table.version == table_version {
                return Some(index);
            }
        }
        None
    }

    /// This functions adds a new TableDefinition to the list. This checks if that version of the table
    /// already exists, and replace it in that case.
    pub fn add_table_definition(&mut self, table_definition: TableDefinition) {
        let version = table_definition.version;
        let mut index_version = 0;
        let mut index_found = false;
        for (index, definition) in self.versions.iter().enumerate() {
            if definition.version == version {
                index_version = index;
                index_found = true;
                break;
            }
        }
        if index_found {
            self.versions.remove(index_version);
            self.versions.insert( index_version, table_definition);
        }
        else {
            self.versions.push(table_definition);
        }
    }
}

/// Implementation of "TableDefinition"
impl TableDefinition {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode with the version we have.
    pub fn new(version: u32) -> TableDefinition {
        let fields = vec![];

        TableDefinition {
            version,
            fields,
        }
    }

    /// This function adds a field to a table. It's just to make it easy to interact with, so we don't
    /// need to call the "Field" stuff manually.
    pub fn add_field(&mut self, field_name: String, field_type: FieldType, field_is_key: bool, field_is_reference: Option<(String, String)>) {
        self.fields.push(Field::new(field_name, field_type, field_is_key, field_is_reference));
    }
}

/// Implementation of "Field"
impl Field {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode with the version we have.
    pub fn new(field_name: String, field_type: FieldType, field_is_key: bool, field_is_reference: Option<(String, String)>) -> Field {

        Field {
            field_name,
            field_is_key,
            field_is_reference,
            field_type,
        }
    }
}