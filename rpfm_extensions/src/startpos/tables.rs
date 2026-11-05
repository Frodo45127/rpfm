//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Database tables used to build a startpos, accessed by column name.
//!
//! The startpos build reads the `start_pos_*` tables of the campaign pack plus many game tables (units, factions,
//! buildings...). This module collects them from decoded DB files, merging the fragments of each table, and
//! keeps the decoded values so numbers keep their exact binary value (the game copies some `f32` values as is).

use std::borrow::Cow;
use std::collections::HashMap;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::db::DB;
use rpfm_lib::files::table::DecodedData;

#[cfg(test)] #[path = "tables_test.rs"] pub(crate) mod tables_test;

/// Buckets of a new chained hash set of the game (see [`hash_set_order`]).
const HASH_SET_FIRST_BUCKETS: usize = 4;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The tables available to the startpos build, by table name (without the `_tables` suffix).
#[derive(Debug, Clone, Default)]
pub struct StartposTables {
    tables: HashMap<String, Table>,
}

/// A table: its columns and the rows of every fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {

    /// Table name, without the `_tables` suffix.
    name: String,

    /// Column positions, by name.
    columns: HashMap<String, usize>,

    /// Rows, in fragment order and then in file order.
    rows: Vec<Vec<DecodedData>>,
}

/// A row of a table.
#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    table: &'a Table,
    data: &'a [DecodedData],
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl StartposTables {

    /// Creates an empty table set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the rows of a DB fragment. Fragments of the same table are merged in the order they're added.
    ///
    /// # Errors
    ///
    /// Returns an error if a later fragment lacks a column of the first fragment of its table.
    pub fn add_db(&mut self, db: &DB) -> Result<()> {
        let name = db.table_name_without_tables();
        let columns = db.definition().fields_processed().iter().map(|field| field.name().to_owned()).collect::<Vec<_>>();
        self.add_rows(&name, &columns, &db.data())
    }

    /// Adds rows to a table, creating it if needed.
    ///
    /// # Arguments
    ///
    /// * `name` - Table name, without the `_tables` suffix.
    /// * `columns` - Column names of the rows.
    /// * `rows` - Rows, with their values in `columns` order.
    ///
    /// # Errors
    ///
    /// Returns an error if the table already exists and `columns` lacks one of its columns.
    pub fn add_rows<S: AsRef<str>>(&mut self, name: &str, columns: &[S], rows: &[Vec<DecodedData>]) -> Result<()> {
        let table = self.tables.entry(name.to_owned()).or_insert_with(|| Table {
            name: name.to_owned(),
            columns: columns.iter().enumerate().map(|(index, column)| (column.as_ref().to_owned(), index)).collect(),
            rows: vec![],
        });

        // Fragments may order their columns differently, so rows are rearranged into the table's order.
        let mut order = vec![0; table.columns.len()];
        for (column, &target) in &table.columns {
            order[target] = columns.iter().position(|other| other.as_ref() == column)
                .ok_or_else(|| RLibError::StartposMissingColumn(name.to_owned(), column.to_owned()))?;
        }

        table.rows.extend(rows.iter().map(|row| order.iter().map(|&source| row[source].clone()).collect()));
        Ok(())
    }

    /// Adds a column to a table that lacks it, with the same value in every row. Does nothing if the table has the
    /// column or doesn't exist.
    pub fn add_missing_column(&mut self, name: &str, column: &str, value: DecodedData) {
        let Some(table) = self.tables.get_mut(name) else { return };
        if table.columns.contains_key(column) {
            return;
        }
        table.columns.insert(column.to_owned(), table.columns.len());
        for row in &mut table.rows {
            row.push(value.clone());
        }
    }

    /// Returns a table by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the table hasn't been added.
    pub fn table(&self, name: &str) -> Result<&Table> {
        self.tables.get(name).ok_or_else(|| RLibError::StartposMissingTable(name.to_owned()))
    }
}

impl Table {

    /// Returns the table name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the rows of the table, in order.
    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.rows.iter().map(|data| Row { table: self, data })
    }

    /// Returns the rows of the table in the order of the game's key lookup table, given the key column.
    ///
    /// The game indexes each table by key in an open addressing table with one slot per row (13 if empty): a
    /// row goes to the slot [`key_hash`] modulo the row count gives, or the next free one. Code iterating that
    /// index visits the rows in slot order.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column.
    pub fn rows_in_key_order(&self, column: &str) -> Result<Vec<Row<'_>>> {
        self.rows_in_key_order_by(|row| Ok(row.str(column)?.into_owned()))
    }

    /// Returns the rows of the table in the order of the game's key lookup table, given each row's key (tables with
    /// several key columns join their values, as in `province` + `region`). See [`Self::rows_in_key_order`].
    ///
    /// # Errors
    ///
    /// Returns an error if the key of a row can't be built.
    pub fn rows_in_key_order_by(&self, key: impl Fn(&Row<'_>) -> Result<String>) -> Result<Vec<Row<'_>>> {
        let keys = self.rows().map(|row| key(&row)).collect::<Result<Vec<_>>>()?;
        let mut slots: Vec<Option<usize>> = vec![None; keys.len().max(1)];
        for (row, key) in keys.iter().enumerate() {
            let mut slot = key_hash(key.as_bytes()) as usize % slots.len();
            while slots[slot].is_some() {
                slot = (slot + 1) % slots.len();
            }
            slots[slot] = Some(row);
        }
        Ok(slots.into_iter().flatten().map(|row| Row { table: self, data: &self.rows[row] }).collect())
    }

    /// Returns the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns if the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns the names of the columns, in no particular order.
    pub fn column_names(&self) -> impl Iterator<Item = &str> {
        self.columns.keys().map(String::as_str)
    }

    /// Returns the position of a column.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column.
    pub fn column(&self, column: &str) -> Result<usize> {
        self.columns.get(column).copied().ok_or_else(|| RLibError::StartposMissingColumn(self.name.to_owned(), column.to_owned()))
    }
}

/// Returns the hash the game gives a table key (`FUN_100e9730` in Thrones): the bytes from last to first, each
/// sign-extended and xored in at a shift that cycles through 0, 8, 16 and 24 bits.
pub fn key_hash(key: &[u8]) -> u32 {
    key.iter().rev().zip([0, 8, 16, 24].into_iter().cycle())
        .fold(0xf73a_51c9, |hash, (&byte, shift)| hash ^ ((byte as i8 as i32 as u32) << shift))
}

/// Returns keys in the order the game's chained hash sets iterate them, given their insertion order.
///
/// The set starts with [`HASH_SET_FIRST_BUCKETS`] buckets and grows to twice plus one when it holds more keys than
/// buckets (its maximum load is 1); keys go to the end of the chain of bucket [`key_hash`] modulo the bucket count,
/// and growing moves them bucket by bucket. Repeated keys are kept once. Verified in Thrones' DLL (constructor
/// `FUN_10b46290`, inserts like `FUN_10b65220`).
pub fn hash_set_order<'k>(keys: impl IntoIterator<Item = &'k str>) -> Vec<&'k str> {
    hash_set_order_by(keys, |key| key_hash(key.as_bytes()))
}

/// Returns items in the order the game's chained hash sets iterate them, given their insertion order and their hash.
/// See [`hash_set_order`].
pub fn hash_set_order_by<T: PartialEq + Copy>(items: impl IntoIterator<Item = T>, hash: impl Fn(&T) -> u32) -> Vec<T> {
    let mut buckets: Vec<Vec<T>> = vec![vec![]; HASH_SET_FIRST_BUCKETS];
    let mut count = 0;
    for item in items {
        let bucket = hash(&item) as usize % buckets.len();
        if buckets[bucket].contains(&item) {
            continue;
        }
        count += 1;
        if count > buckets.len() {
            let mut grown = vec![vec![]; buckets.len() * 2 + 1];
            for moved in buckets.into_iter().flatten() {
                let bucket = hash(&moved) as usize % grown.len();
                grown[bucket].push(moved);
            }
            buckets = grown;
        }
        let bucket = hash(&item) as usize % buckets.len();
        buckets[bucket].push(item);
    }
    buckets.into_iter().flatten().collect()
}

impl<'a> Row<'a> {

    /// Returns a value of the row.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column.
    pub fn value(&self, column: &str) -> Result<&'a DecodedData> {
        Ok(&self.data[self.table.column(column)?])
    }

    /// Returns a colour of the row, stored as an `RRGGBB` hex text, as its red, green and blue channels.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column, or the value isn't an `RRGGBB` hex text.
    pub fn colour(&self, column: &str) -> Result<[u8; 3]> {
        let hex = self.str(column)?;
        let invalid = || RLibError::StartposInvalidValue(self.table.name.to_owned(), column.to_owned(), hex.to_string());
        if hex.len() != 6 {
            return Err(invalid());
        }

        let mut colour = [0; 3];
        for (channel, component) in colour.iter_mut().enumerate() {
            *component = u8::from_str_radix(hex.get(channel * 2..channel * 2 + 2).ok_or_else(invalid)?, 16).map_err(|_| invalid())?;
        }
        Ok(colour)
    }

    /// Returns a value of the row as text (numbers and booleans are formatted).
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column.
    pub fn str(&self, column: &str) -> Result<Cow<'a, str>> {
        Ok(self.value(column)?.data_to_string())
    }

    /// Returns a boolean value of the row.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column, or the value isn't a boolean.
    pub fn bool(&self, column: &str) -> Result<bool> {
        match self.value(column)? {
            DecodedData::Boolean(value) => Ok(*value),
            other => Err(self.invalid(column, other)),
        }
    }

    /// Returns an integer value of the row.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column, or the value isn't an integer.
    pub fn int(&self, column: &str) -> Result<i64> {
        match self.value(column)? {
            DecodedData::I16(value) | DecodedData::OptionalI16(value) => Ok(i64::from(*value)),
            DecodedData::I32(value) | DecodedData::OptionalI32(value) => Ok(i64::from(*value)),
            DecodedData::I64(value) | DecodedData::OptionalI64(value) => Ok(*value),
            other => Err(self.invalid(column, other)),
        }
    }

    /// Returns a floating point value of the row, as stored.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column, or the value isn't a float.
    pub fn f32(&self, column: &str) -> Result<f32> {
        match self.value(column)? {
            DecodedData::F32(value) => Ok(*value),
            DecodedData::F64(value) => Ok(*value as f32),
            other => Err(self.invalid(column, other)),
        }
    }

    /// Returns a numeric value of the row (integer or float) as a float.
    ///
    /// # Errors
    ///
    /// Returns an error if the table has no such column, or the value isn't a number.
    pub fn number(&self, column: &str) -> Result<f64> {
        match self.value(column)? {
            DecodedData::F32(value) => Ok(f64::from(*value)),
            DecodedData::F64(value) => Ok(*value),
            _ => self.int(column).map(|value| value as f64),
        }
    }

    /// Builds the error for a value of an unexpected type.
    fn invalid(&self, column: &str, value: &DecodedData) -> RLibError {
        RLibError::StartposInvalidValue(self.table.name.to_owned(), column.to_owned(), format!("{value:?}"))
    }
}
