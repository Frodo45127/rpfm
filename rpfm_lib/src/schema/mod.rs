//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file goes all the stuff needed for the schema decoder to work.

use serde_derive::{Serialize, Deserialize};

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::{fmt, fmt::Display};

use crate::SUPPORTED_GAMES;
use crate::config::get_config_path;
use crate::updater::Versions;
use rpfm_error::{ErrorKind, Result};

pub mod assembly_kit;

/// Name of the schemas versions file.
const SCHEMA_VERSIONS_FILE: &'static str = "versions.json";

/// URL used to download new schemas.
pub const SCHEMA_UPDATE_URL_MASTER: &'static str = "https://raw.githubusercontent.com/Frodo45127/rpfm/master/schemas/";

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
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TableDefinitions {
    pub name: String,
    pub versions: Vec<TableDefinition>,
}

/// This struct holds the definitions for a version of a table. It has:
/// - version: the version of the table these definitions are for.
/// - fields: the different fields this table has.
///
/// NOTE: the versions are:
/// - 0: for unversioned tables.
/// - 1+: for versioned tables.
/// - 1: for LOC Definitions.
/// - -1: for Fake Definitions.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TableDefinition {
    pub version: i32,
    pub fields: Vec<Field>,
}

/// This struct holds the type of a field of a table. It has:
/// - field_name: the name of the field.
/// - field_is_key: true if the field is a key field and his column needs to be put in the beginning of the TreeView.
/// - field_is_reference: if this field is a reference of another, this has (table name, field name).
/// - field_type: the type of the field.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Field {
    pub field_name: String,
    pub field_type: FieldType,
    pub field_is_key: bool,
    pub field_is_reference: Option<(String, String)>,
    pub field_description: String,
}

/// Enum FieldType: This enum is used to define the possible types of a field in the schema.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    Float,
    Integer,
    LongInteger,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16
}

/// Implementation of "Schema"
impl Schema {

    /// This function creates a new schema. It should only be needed to create the first table definition
    /// of a game, as the rest will continue using the same schema.
    pub fn new() -> Self {
        Self {
            tables_definitions: vec![],
        }
    }

    /// This function adds a new TableDefinitions to the schema. This checks if that table definitions
    /// already exists, and replace it in that case.
    pub fn add_table_definitions(&mut self, table_definitions: TableDefinitions) {
        match self.tables_definitions.iter().position(|x| x.name == table_definitions.name) {
            Some(position) => { self.tables_definitions.splice(position..position + 1, [table_definitions].iter().cloned()); },
            None => self.tables_definitions.push(table_definitions),
        }
    }

    /// This functions returns the index of the definitions for a table.
    #[deprecated(since="1.7.0", note="Please use `get_definitions` instead")]
    pub fn get_table_definitions(&self, table_name: &str) -> Option<usize> {
        self.tables_definitions.iter().position(|x| x.name == table_name)
    }

    // This function returns a definition under the provided name if it exists. Otherwise it return `Error`.
    pub fn get_definitions(&self, table_name: &str) -> Result<&TableDefinitions> {
        if let Some(index) = self.tables_definitions.iter().position(|x| x.name == table_name) {
            Ok(&self.tables_definitions[index])
        } else { Err(ErrorKind::SchemaTableDefinitionNotFound)? }
    }

    /// This function takes an schema file and reads it into a "Schema" object.
    pub fn load(schema_file: &str) -> Result<Self> {

        let mut file_path = get_config_path()?.join("schemas");
        file_path.push(schema_file);

        let file = BufReader::new(File::open(&file_path)?);
        serde_json::from_reader(file).map_err(|x| From::from(x))
    }

    /// This function takes an "Schema" object and saves it into a schema file.
    pub fn save(&self, schema_file: &str) -> Result<()> {

        let mut file_path = get_config_path()?.join("schemas");
        file_path.push(schema_file);

        let mut file = File::create(&file_path)?;
        let json = serde_json::to_string_pretty(&self)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// This function generates the diff between the local schemas and the remote ones and, if it detects
    /// that you're using the git repo (debug), it adds the diff to the proper place in the docs. 
    pub fn generate_schema_diff() -> Result<()> {

        // To avoid doing a lot of useless checking, we only check for schemas with different version.
        let local_schema_versions: Versions = serde_json::from_reader(BufReader::new(File::open(get_config_path()?.join("schemas/versions.json"))?))?;
        let current_schema_versions: Versions = reqwest::get(&format!("{}/{}", SCHEMA_UPDATE_URL_MASTER, SCHEMA_VERSIONS_FILE))?.json()?;
        let mut schemas_to_update = vec![];

        // If the game's schema is not in the repo (when adding a new game's support) skip it.
        for (game, version_local) in &local_schema_versions {
            let version_current = if let Some(version_current) = current_schema_versions.get(game) { version_current } else { continue };
            if version_local != version_current { schemas_to_update.push((game.to_owned(), version_local)); }
        }

        for (game_name, game) in SUPPORTED_GAMES.iter() {

            // Skip all the games with an unchanged version.
            let schema_name = &game.schema;
            let mut schema_version = 0;
            let mut skip_it = true;
            for (schema_to_update, schema_version_to_update) in &mut schemas_to_update {
                if schema_to_update == schema_name { 
                    skip_it = false;
                    schema_version = **schema_version_to_update;
                    break;
                }
            }
            if skip_it { continue; }

            // For this, first we get both schemas. Then, compare them table by table looking for differences.
            // Uncomment and tweak the commented schema_current to test against a local schema.
            let schema_local = Schema::load(schema_name).unwrap();
            //let schema_current = Schema::load("schema_att.json").unwrap();
            let schema_current: Schema = reqwest::get(&format!("{}/{}", SCHEMA_UPDATE_URL_MASTER, schema_name))?.json()?;

            // Lists to store the different types of differences.
            let mut diff = String::new();
            let mut new_tables = vec![];
            let mut new_versions: Vec<String> = vec![];
            let mut new_corrections: Vec<String> = vec![];

            // For each table, we need to check EVERY possible difference.
            for table_local in &schema_local.tables_definitions {
                match schema_current.tables_definitions.iter().find(|x| x.name == table_local.name) {

                    // If we find it, we have to check if it has changes. If it has them, then we analize them.
                    Some(table_current) => {
                        if table_local != table_current {
                            for version_local in &table_local.versions {
                                match table_current.versions.iter().find(|x| x.version == version_local.version) {
                                    
                                    // If the version has been found, it's a correction for a current version. So we check every
                                    // field for references.
                                    Some(version_current) => version_local.get_pretty_diff(&version_current, &table_local.name, &mut new_corrections),

                                    // If the version hasn't been found, is a new version. We have to compare it with
                                    // the old one and get his changes.
                                    None => {

                                        // If we have more versions, get the highest one before the one we have. Tables are automatically
                                        // sorted on save, so we can just get the first one of the current list.
                                        if table_local.versions.len() > 1 {
                                            let old_version = &table_current.versions[0];
                                            version_local.get_pretty_diff(&old_version, &table_local.name, &mut new_versions);
                                        }
                                    },
                                }
                            }
                        }
                    }

                    // If the table hasn't been found, it's a new table we decoded.
                    None => new_tables.push(table_local.name.to_owned()),
                }
            }

            // Here we put together all the differences.
            for (index, table) in new_tables.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **New tables decoded**:\n");
                }
                diff.push_str(&format!("  - *{}*.", table));
                diff.push_str("\n");

                if index == new_tables.len() - 1 {
                    diff.push_str("\n");
                }
            }

            for (index, version) in new_versions.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **Updated Tables**:\n");
                }
                diff.push_str(&format!("{}", version));
                diff.push_str("\n");

                if index == new_versions.len() - 1 {
                    diff.push_str("\n");
                }
            }

            for (index, correction) in new_corrections.iter().enumerate() {
                if index == 0 {
                    diff.push_str("- **Fixed Tables**:\n");
                }
                diff.push_str(&format!("{}", correction));
                diff.push_str("\n");

                if index == new_corrections.len() - 1 {
                    diff.push_str("\n");
                }
            }

            // If it's not empty, save it. Otherwise, we just ignore it. 
            if !diff.is_empty() {

                // If we are in debug mode, save it to his proper file in the docs.
                if cfg!(debug_assertions) { 
                    let mut docs_path = std::env::current_dir().unwrap().to_path_buf();
                    docs_path.push("docs_src");
                    docs_path.push("changelogs_tables");
                    docs_path.push(game_name);
                    docs_path.push(&format!("{:03}.md", schema_version));

                    let mut docs_changelog_path = docs_path.to_path_buf();
                    docs_changelog_path.pop();
                    docs_changelog_path.push("changelog.md");

                    // Fix the text so it has the MarkDown title before writing it.
                    diff.insert_str(0, &format!("# {:03}\n\nIt contains the following changes:\n\n", schema_version));
                    let mut file = File::create(docs_path)?;
                    file.write_all(diff.as_bytes())?;

                    // Now, we have to add the file with includes to his respective changelog.
                    let mut base_file = String::new();
                    BufReader::new(File::open(&docs_changelog_path)?).read_to_string(&mut base_file)?;
                    let include_index_line = base_file.find("-----------------------------------").unwrap();
                    let include_data_line = base_file.rfind("-----------------------------------").unwrap();
                    base_file.insert_str(include_data_line + 35, &format!("\n{{{{ #include {:03}.md }}}}", schema_version));
                    base_file.insert_str(include_index_line + 35, &format!("\n- [{:03}](#{:03})", schema_version, schema_version));
                    let mut file = File::create(docs_changelog_path)?;
                    file.write_all(base_file.as_bytes())?;
                }

                // Otherwise, save it to a file in RPFM's folder.
                else { 
                    let mut changes_path = get_config_path()?.to_path_buf();
                    changes_path.push(&format!("changelog_{}.txt", schema_name));
                    let mut file = File::create(changes_path)?;
                    file.write_all(diff.as_bytes())?;
                }

            }
        }

        // If everything worked, return success.
        Ok(())
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
    #[deprecated(since="1.7.0", note="Please use `get_version` instead")]
    pub fn get_table_version(&self, table_version: i32) -> Option<usize> {
        for (index, table) in self.versions.iter().enumerate() {
            if table.version == table_version {
                return Some(index);
            }
        }
        None
    }

    // This function returns a definition under the provided version if it exists. Otherwise it return `Error`.
    pub fn get_version(&self, version: i32) -> Result<&TableDefinition> {
        if let Some(index) = self.versions.iter().position(|x| x.version == version) {
            Ok(&self.versions[index])
        } else { Err(ErrorKind::SchemaTableNotFound)? }
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
            self.versions.insert(index_version, table_definition);
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
    pub fn new(version: i32) -> TableDefinition {
        TableDefinition {
            version,
            fields: vec![],
        }
    }

    /// This function creates a new table definition from an imported definition from the assembly kit.
    /// Note that this import the loc fields (they need to be removed manually later) and it doesn't
    /// import the version (this... I think I can do some trick for it).
    pub fn new_from_assembly_kit(imported_table_definition: &assembly_kit::root, version: i32, table_name: &str) -> TableDefinition {
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
        
    /// This function creates a new fake table definition from an imported definition from the assembly kit.
    /// For use with the raw tables processing.
    pub fn new_fake_from_assembly_kit(imported_table_definition: &assembly_kit::root, version: i32, table_name: &str) -> TableDefinition {
        let mut fields = vec![];
        for field in &imported_table_definition.field {

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

    /// This generates a new fake definition for LOC PackedFiles.
    pub fn new_loc_definition() -> Self {
        let version = 1;
        let mut fields = vec![];
        fields.push(Field::new("key".to_owned(), FieldType::StringU16, false, None, "".to_owned()));
        fields.push(Field::new("text".to_owned(), FieldType::StringU16, false, None, "".to_owned()));
        fields.push(Field::new("tooltip".to_owned(), FieldType::Boolean, false, None, "".to_owned()));
        Self {
            version,
            fields,
        }
    }

    /// This generates a new fake definition for the Dependency PackFile's List.
    pub fn new_dependency_manager_definition() -> Self {
        Self {
            version: 1,
            fields: vec![Field::new("PackFile's List".to_owned(), FieldType::StringU8, false, None, "".to_owned())],
        }
    }

    /// This function generates a MarkDown diff of two versions of an specific table and adds it to the provided changes list.
    pub fn get_pretty_diff(
        &self,
        version_current: &Self,
        table_name: &str,
        changes: &mut Vec<String>,
    ) {

        // Here it's were things get complex. We have to get, field by field, and check:
        // - If they exists.
        // - If they are in the same position. (TODO)
        // - If they are different, in which case we have to check on what.
        // Changed fields have: Vec<(field_name, vec<(changed_variant, (before, after))>)>.
        let mut new_fields: Vec<Field> = vec![];
        let mut changed_fields: Vec<(String, Vec<(String, (String, String))>)> = vec![];
        let mut removed_fields: Vec<String> = vec![];
        for field_local in &self.fields {
            match version_current.fields.iter().find(|x| x.field_name == field_local.field_name) {
                Some(field_current) => {

                    // If they are different, we have to find what do they have different, so we
                    // only show that in the changelog.
                    let mut changes = vec![];
                    if field_local != field_current {
                        if field_local.field_type != field_current.field_type {
                            changes.push(("Type".to_owned(), (format!("{}", field_current.field_type), format!("{}", field_local.field_type))));
                        }

                        if field_local.field_is_key != field_current.field_is_key {
                            changes.push(("Is Key".to_owned(), (format!("{}", field_current.field_is_key), format!("{}", field_local.field_is_key))));
                        }

                        if field_local.field_is_reference != field_current.field_is_reference {
                            changes.push(("Is Reference".to_owned(), 
                                (
                                    if let Some((ref_table, ref_column)) = &field_current.field_is_reference { format!("{}, {}", ref_table, ref_column) }
                                    else { String::new() },
                                    if let Some((ref_table, ref_column)) = &field_local.field_is_reference { format!("{}, {}", ref_table, ref_column) }
                                    else { String::new() }        
                                )
                            ));
                        }

                        if field_local.field_description != field_current.field_description {
                            changes.push(("Description".to_owned(), (field_current.field_description.to_owned(), field_local.field_description.to_owned())));
                        }
                    }

                    if !changes.is_empty() {
                        changed_fields.push((field_local.field_name.to_owned(), changes));
                    }
                },

                // If the field doesn't exists, it's new.
                None => new_fields.push(field_local.clone()),
            }
        }

        // We have to check for removed fields too.
        for field_current in &version_current.fields {
            if self.fields.iter().find(|x| x.field_name == field_current.field_name).is_none() {
                removed_fields.push(field_current.field_name.to_owned());
            }
        }

        if !new_fields.is_empty() || !changed_fields.is_empty() || !removed_fields.is_empty() {
            changes.push(format!("  - ***{}***:", table_name));
        } 

        for (index, new_field) in new_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **New fields**:".to_owned()); }
            changes.push(format!("      - ***{}***:", new_field.field_name));
            changes.push(format!("        - **Type**: *{}*.", new_field.field_type));
            changes.push(format!("        - **Is Key**: *{}*.", new_field.field_is_key));
            if let Some((ref_table, ref_column)) = &new_field.field_is_reference {
                changes.push(format!("        - **Is Reference**: *{}*/*{}*.", ref_table, ref_column));
            }
            if !new_field.field_description.is_empty() {
                changes.push(format!("        - **Description**: *{}*.", new_field.field_description));
            }
        }

        for (index, changed_field) in changed_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **Changed fields**:".to_owned()); }
            changes.push(format!("      - **{}**:", changed_field.0));

            for changed_variant in &changed_field.1 {
                changes.push(format!("        - ***{}***: *{}* => *{}*.", changed_variant.0, (changed_variant.1).0, (changed_variant.1).1));
            }
        }

        for (index, removed_field) in removed_fields.iter().enumerate() {
            if index == 0 { changes.push("    - **Removed fields**:".to_owned()); }
            changes.push(format!("      - *{}*.", removed_field));
        }
    }
}

/// Implementation of "Field".
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

/// Display implementation of FieldType.
impl Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::Float => write!(f, "Float"),
            FieldType::Integer => write!(f, "Integer"),
            FieldType::LongInteger => write!(f, "Long Integer"),
            FieldType::StringU8 => write!(f, "StringU8"),
            FieldType::StringU16 => write!(f, "StringU16"),
            FieldType::OptionalStringU8 => write!(f, "OptionalStringU8"),
            FieldType::OptionalStringU16 => write!(f, "OptionalStringU16"),
        }
    }
}
