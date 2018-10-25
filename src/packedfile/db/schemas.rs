// In this file goes all the stuff needed for the schema decoder to work.
extern crate serde_json;

use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;

use RPFM_PATH;
use error::{ErrorKind, Result};
use super::schemas_importer;

/// This struct holds the entire schema for the currently selected game (by "game" I mean the PackFile
/// Type).
/// It has:
/// - game: the game for what the loaded definitions are intended.
/// - version: custom variable to keep track of the updates to the schema.
/// - tables_definition: the actual definitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
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
    pub field_type: FieldType,
    pub field_is_key: bool,
    pub field_is_reference: Option<(String, String)>,
    pub field_description: String,
}

/// Enum FieldType: This enum is used to define the possible types of a field in the schema.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    Float,
    Integer,
    LongInteger,
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
        let tables_definitions = vec![];

        Schema {
            tables_definitions,
        }
    }

    /// This function adds a new TableDefinitions to the schema. This checks if that table definitions
    /// already exists, and replace it in that case.
    pub fn add_table_definitions(&mut self, table_definitions: TableDefinitions) {

        let name = table_definitions.name.to_owned();
        let mut index_name = 0;
        let mut index_found = false;
        for (index, definitions) in self.tables_definitions.iter().enumerate() {
            if definitions.name == name {
                index_name = index;
                index_found = true;
                break;
            }
        }
        if index_found {
            self.tables_definitions.remove(index_name);
            self.tables_definitions.insert(index_name, table_definitions);
        }
        else {
            self.tables_definitions.push(table_definitions);
        }
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
    pub fn load(schema_file: &str) -> Result<Schema> {

        let mut schemas_path = RPFM_PATH.to_path_buf();
        schemas_path.push("schemas");

        // We load the provided schema file.
        let schema_file = BufReader::new(File::open(PathBuf::from(format!("{}/{}", schemas_path.to_string_lossy(), schema_file)))?);

        let schema = serde_json::from_reader(schema_file)?;
        Ok(schema)
    }

    /// This function takes an "Schema" object and saves it into a schema file.
    pub fn save(schema: &Schema, schema_file: &str) -> Result<()> {

        let schema_json = serde_json::to_string_pretty(schema);
        let mut schema_path = RPFM_PATH.to_path_buf();
        schema_path.push("schemas");
        schema_path.push(schema_file);

        match File::create(schema_path.to_path_buf()) {
            Ok(mut file) => {
                match file.write_all(schema_json.unwrap().as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorKind::IOGenericWrite(vec![schema_path.display().to_string();1]))?
                }
            },
            Err(_) => Err(ErrorKind::IOGenericWrite(vec![schema_path.display().to_string();1]))?
        }
    }
}

/// Implementation of "TableDefinitions"
impl TableDefinitions {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode.
    pub fn new(name: &str) -> TableDefinitions {
        let name = name.to_string();
        let versions = vec![];

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
        TableDefinition {
            version,
            fields: vec![Field::new("example_field".to_owned(), FieldType::StringU8, false, None, "delete this field if you see it".to_owned())],
        }
    }

    /// This function creates a new table definition from an imported definition from the assembly kit.
    /// Note that this import the loc fields (they need to be removed manually later) and it doesn't
    /// import the version (this... I think I can do some trick for it).
    pub fn new_from_assembly_kit(imported_table_definition: &schemas_importer::root, version: u32, table_name: &str) -> TableDefinition {
        let mut fields = vec![];
        for field in &imported_table_definition.field {

            // First, we need to disable a number of known fields that are not in the final tables. We
            // check if the current field is one of them, and ignore it if it's.
            if field.name == "game_expansion_key" || // This one exists in one of the advices tables.
                field.name == "localised_text" ||
                field.name == "localised_name" ||
                field.name == "localised_tooltip" ||
                field.name == "description" ||
                field.name == "objectives_team_1" ||
                field.name == "objectives_team_2" ||
                field.name == "short_description_text" ||
                field.name == "historical_description_text" ||
                field.name == "strengths_weaknesses_text" ||
                field.name == "onscreen" ||
                field.name == "onscreen_text" ||
                field.name == "onscreen_name" ||
                field.name == "onscreen_description" ||
                field.name == "on_screen_name" ||
                field.name == "on_screen_description" ||
                field.name == "on_screen_target" {
                continue;
            }
            let field_name = field.name.to_owned();
            let field_is_key = field.primary_key == "1";
            let field_is_reference = if field.column_source_table != None {
                Some((field.column_source_table.clone().unwrap().to_owned(), field.column_source_column.clone().unwrap()[0].to_owned()))
            }
            else {None};

            let field_type = match &*field.field_type {
                "yesno" => FieldType::Boolean,
                "single" | "decimal" | "double" => FieldType::Float,
                "autonumber" => FieldType::LongInteger, // Not always true, but better than nothing.
                "integer" => {

                    // In Warhammer 2 these tables are wrong in the definition schema.
                    if table_name.starts_with("_kv") {
                        FieldType::Float
                    }
                    else {
                        FieldType::Integer
                    }
                },
                "text" => {

                    // Key fields are ALWAYS REQUIRED. This fixes it's detection.
                    if field.name == "key" {
                        FieldType::StringU8
                    }
                    else {
                        match &*field.required {
                            "1" => {
                                // In Warhammer 2 this table has his "value" field broken.
                                if table_name == "_kv_winds_of_magic_params_tables" && field.name == "value" {
                                    FieldType::Float
                                }
                                else {
                                    FieldType::StringU8
                                }
                            },
                            "0" => FieldType::OptionalStringU8,

                            // If we reach this point, we set it to OptionalStringU16. Not because it is it
                            // (we don't have a way to distinguish String types) but to know what fields
                            // reach this point.
                            _ => FieldType::OptionalStringU16,
                        }
                    }
                }
                // If we reach this point, we set it to StringU16. Not because it is it
                // (we don't have a way to distinguish String types) but to know what fields
                // reach this point.
                _ => FieldType::StringU16,

            };

            let field_description = match field.field_description {
                Some(ref description) => description.to_owned(),
                None => String::new(),
            };

            let new_field = Field::new(
                field_name,
                field_type,
                field_is_key,
                field_is_reference,
                field_description
            );
            fields.push(new_field);
        }

        TableDefinition {
            version,
            fields,
        }
    }
}

/// Implementation of "Field"
impl Field {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode with the version we have.
    pub fn new(field_name: String, field_type: FieldType, field_is_key: bool, field_is_reference: Option<(String, String)>, field_description: String) -> Field {

        Field {
            field_name,
            field_type,
            field_is_key,
            field_is_reference,
            field_description
        }
    }
}
