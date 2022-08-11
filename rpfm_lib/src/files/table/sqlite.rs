//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module to hold all table functions specific of the SQLite backend.

use crate::files::table::SQLData;
use crate::schema::Field;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params_from_iter;

use crate::{files::ReadBytes, schema::FieldType};
use crate::error::Result;
use crate::schema::Definition;

use super::{DecodedData, Table, TableData};

//----------------------------------------------------------------//
// Implementations for `Table`.
//----------------------------------------------------------------//

impl Table {

    pub(crate) fn decode<R: ReadBytes>(
        pool: &Option<&Pool<SqliteConnectionManager>>,
        data: &mut R,
        definition: &Definition,
        entry_count: Option<u32>,
        return_incomplete: bool,
        table_name: &str,
    ) -> Result<Self> {

        let table_data = Self::decode_table(data, definition, entry_count, return_incomplete)?;
        let table_data = match pool {
            Some(pool) => {

                // Try to create the table, in case it doesn't exist yet. Ignore a failure here, as it'll mean the table already exists.
                let params: Vec<String> = vec![];
                let create_table = definition.map_to_sql_create_table_string(true, table_name, None);
                let _ = pool.get()?.execute(&create_table, params_from_iter(params.into_iter())).map(|_| ());

                // Load the data to the database.
                let table_unique_id = rand::random::<u64>();
                Self::insert(&pool, &table_data, definition, table_name, table_unique_id, true)?;
                TableData::Sql(SQLData {
                    table_unique_id,
                })
            },
            None => TableData::Local(table_data),
        };

        let table = Self {
            definition: definition.clone(),
            table_name: table_name.to_owned(),
            table_data,
        };

        Ok(table)
    }

    /// This function inserts the provided rows of data into a database.
    pub fn insert(
        pool: &Pool<SqliteConnectionManager>,
        data: &[Vec<DecodedData>],
        definition: &Definition,
        table_name: &str,
        table_unique_id: u64,
        key_first: bool
    ) -> Result<()> {
        let mut params = vec![];
        let values = data.iter().map(|row| {
            format!("({}, {})", table_unique_id, row.iter().map(|field| {
                match field {
                    DecodedData::Boolean(data) => if *data { "1".to_owned() } else { "0".to_owned() },
                    DecodedData::F32(data) => format!("{:.4}", data),
                    DecodedData::F64(data) => format!("{:.4}", data),
                    DecodedData::I16(data) => format!("\"{}\"", data),
                    DecodedData::I32(data) => format!("\"{}\"", data),
                    DecodedData::I64(data) => format!("\"{}\"", data),
                    DecodedData::ColourRGB(data) => format!("\"{}\"", data.replace("\"", "\\\"")),
                    DecodedData::StringU8(data) => format!("\"{}\"", data.replace("\"", "\\\"")),
                    DecodedData::StringU16(data) => format!("\"{}\"", data.replace("\"", "\\\"")),
                    DecodedData::OptionalI16(data) => format!("\"{}\"", data),
                    DecodedData::OptionalI32(data) => format!("\"{}\"", data),
                    DecodedData::OptionalI64(data) => format!("\"{}\"", data),
                    DecodedData::OptionalStringU8(data) => format!("\"{}\"", data.replace("\"", "\\\"")),
                    DecodedData::OptionalStringU16(data) => format!("\"{}\"", data.replace("\"", "\\\"")),
                    DecodedData::SequenceU16(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                    DecodedData::SequenceU32(data) => {
                        params.push(data.to_vec());
                        "?".to_owned()
                    },
                }
            }).collect::<Vec<_>>().join(","))
        }).collect::<Vec<_>>().join(",");

        let query = format!("INSERT OR REPLACE INTO \"{}_v{}\" {} VALUES {}",
            table_name.replace("\"", "'"),
            definition.version(),
            definition.map_to_sql_insert_into_string(key_first),
            values
        );

        pool.get()?.execute(&query, params_from_iter(params.iter()))
            .map(|_| ())
            .map_err(From::from)
    }

    /// This function inserts the provided rows of data into a database.
    pub fn select_all_from_table(
        pool: &Pool<SqliteConnectionManager>,
        table_name: &str,
        table_version: i32,
        table_unique_id: u64,
        fields: &[Field],
    ) -> Result<Vec<Vec<DecodedData>>> {
        let field_names = fields.iter().map(|field| field.name()).collect::<Vec<&str>>().join(",");
        let query = format!("SELECT {} FROM \"{}_v{}\" WHERE table_unique_id = {} order by ROWID",
            field_names,
            table_name.replace("\"", "'"),
            table_version,
            table_unique_id
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let mut data = Vec::with_capacity(fields.len());
            for i in 0..fields.len() {
                data.push(match fields[i].field_type() {
                    FieldType::Boolean => DecodedData::Boolean(row.get(i)?),
                    FieldType::F32 => DecodedData::F32(row.get(i)?),
                    FieldType::F64 => DecodedData::F64(row.get(i)?),
                    FieldType::I16 => DecodedData::I16(row.get(i)?),
                    FieldType::I32 => DecodedData::I32(row.get(i)?),
                    FieldType::I64 => DecodedData::I64(row.get(i)?),
                    FieldType::ColourRGB => DecodedData::ColourRGB(row.get(i)?),
                    FieldType::StringU8 => DecodedData::StringU8(row.get(i)?),
                    FieldType::StringU16 => DecodedData::StringU16(row.get(i)?),
                    FieldType::OptionalI16 => DecodedData::OptionalI16(row.get(i)?),
                    FieldType::OptionalI32 => DecodedData::OptionalI32(row.get(i)?),
                    FieldType::OptionalI64 => DecodedData::OptionalI64(row.get(i)?),
                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(row.get(i)?),
                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(row.get(i)?),
                    FieldType::SequenceU16(_) => DecodedData::SequenceU16(row.get(i)?),
                    FieldType::SequenceU32(_) => DecodedData::SequenceU32(row.get(i)?),
                });
            }

            Ok(data)
        })?;

        let mut data = vec![];
        for row in rows {
            data.push(row?);
        }

        Ok(data)
    }

    /// This function inserts the provided rows of data into a database.
    pub fn count_table(
        pool: &Pool<SqliteConnectionManager>,
        table_name: &str,
        table_version: i32,
        table_unique_id: u64,
    ) -> Result<u64> {
        let query = format!("SELECT COUNT(*) FROM \"{}_v{}\" WHERE table_unique_id = {}",
            table_name.replace("\"", "'"),
            table_version,
            table_unique_id
        );

        let conn = pool.get()?;
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        let mut count = 0;
        if let Some(row) = rows.next()? {
            count = row.get(0)?;
        }

        Ok(count)
    }
}
