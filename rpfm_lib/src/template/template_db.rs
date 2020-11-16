//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with TemplateDB.
!*/

use serde_derive::{Serialize, Deserialize};

use rpfm_error::{ErrorKind, Result};

use crate::dependencies::Dependencies;
use crate::packfile::{PackFile, packedfile::PackedFile};
use crate::packedfile::DecodedPackedFile;
use crate::packedfile::table::db::DB;
use crate::packedfile::table::{DecodedData, Table};
use crate::schema::{FieldType, Schema};

use super::TemplateField;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a DB Table that's part of a template.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TemplateDB {

    /// Options required for the table to be used in the template.
    pub required_options: Vec<String>,

    /// Name of the DB Table whose definition will be used for the template.
    pub table: String,

    /// Name of the table file within the PackFile, once the template is applied.
    pub name: String,

    /// This means: Rows(Values in Row(Field Name, Value)).
    default_data: Vec<Vec<TemplateField>>,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `TemplateDB`.
impl TemplateDB {

    /// This function builds a full TemplateDB from a PackedFile, if said PackedFile is a decodeable DB Table.
    pub fn new_from_packedfile(packed_file: &PackedFile) -> Result<Self> {
        let mut template = Self::default();
        template.name = packed_file.get_path().last().unwrap().to_owned();
        template.table = match packed_file.get_path().get(1) {
            Some(table) => table.to_owned(),
            None => return Err(ErrorKind::Generic.into()),
        };

        match packed_file.get_decoded_from_memory()? {
            DecodedPackedFile::DB(table) => {
                let definition = table.get_ref_definition();
                for row in table.get_ref_table_data() {
                    let mut row_data = vec![];
                    for (column, field) in row.iter().enumerate() {
                        row_data.push(TemplateField::new(&[], definition.get_fields_processed().get(column).unwrap().get_name(), &field.data_to_string()));
                    }
                    template.default_data.push(row_data);
                }
            },

            _ => return Err(ErrorKind::Generic.into()),
        }

        Ok(template)
    }

    /// This function applies the provided TemplateDB to the open PackFile.
    ///
    /// In case the table already exists, the new data is added at the end of it.
    pub fn apply_to_packfile(&self, options: &[String], pack_file: &mut PackFile, schema: &Schema, dependencies: &Dependencies) -> Result<PackedFile> {

        let path = vec!["db".to_owned(), self.table.to_owned(), self.name.to_owned()];

        let mut table = if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
            if let Ok(table) = packed_file.decode_return_ref_no_locks(&schema) {
                if let DecodedPackedFile::DB(table) = table {
                    table.clone()
                } else { DB::new(&self.name, None, schema.get_ref_last_definition_db(&self.table, &dependencies)?) }
            } else { DB::new(&self.name, None, schema.get_ref_last_definition_db(&self.table, &dependencies)?) }
        } else { DB::new(&self.name, None, schema.get_ref_last_definition_db(&self.table, &dependencies)?) };

        let mut data = table.get_table_data();
        for row in &self.default_data {
            let mut new_row = Table::get_new_row(table.get_ref_definition());
            for (index, field) in table.get_ref_definition().get_fields_processed().iter().enumerate() {
                if let Some(template_field) = row.iter().find(|x| x.get_field_name() == field.get_name()) {

                    // Only change the field if the proper options are enabled.
                    if template_field.has_required_options(options) {
                        new_row[index] = match field.get_ref_field_type() {
                            FieldType::Boolean => {
                                let value = template_field.get_field_value().to_lowercase();
                                if value == "true" || value == "1" { DecodedData::Boolean(true) }
                                else { DecodedData::Boolean(false) }
                            }
                            FieldType::F32 => DecodedData::F32(template_field.get_field_value().parse::<f32>()?),
                            FieldType::I16 => DecodedData::I16(template_field.get_field_value().parse::<i16>()?),
                            FieldType::I32 => DecodedData::I32(template_field.get_field_value().parse::<i32>()?),
                            FieldType::I64 => DecodedData::I64(template_field.get_field_value().parse::<i64>()?),
                            FieldType::StringU8 => DecodedData::StringU8(template_field.get_field_value().to_owned()),
                            FieldType::StringU16 => DecodedData::StringU16(template_field.get_field_value().to_owned()),
                            FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(template_field.get_field_value().to_owned()),
                            FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(template_field.get_field_value().to_owned()),

                            // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                            FieldType::SequenceU16(_) => unimplemented!(),
                            FieldType::SequenceU32(_) => unimplemented!()
                        }
                    }
                }
            }

            data.push(new_row);
        }
        table.set_table_data(&data)?;

        Ok(PackedFile::new_from_decoded(&DecodedPackedFile::DB(table), &path))
    }

    /// This function replaces the parametrized fields of the TemplateDB with the user-provided values.
    pub fn replace_params(&mut self, key: &str, value: &str) {
        self.name = self.name.replace(&format!("{{@{}}}", key), value);
        self.default_data.iter_mut()
            .for_each(|x| x.iter_mut()
                .for_each(|y| *y.get_ref_mut_field_value() = y.get_field_value().replace(&format!("{{@{}}}", key), value))
            );
    }

    /// This function is used to check if we have all the options required to use this field in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }
}

