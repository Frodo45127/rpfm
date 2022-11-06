//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module to hold all table functions specific of the local backend.

use super::Table;

//----------------------------------------------------------------//
// Implementations for `Table`.
//----------------------------------------------------------------//

impl Table {

    #[cfg(not(feature = "integration_sqlite"))]
    pub(crate) fn decode<R: ReadBytes>(
        data: &mut R,
        definition: &Definition,
        entry_count: Option<u32>,
        return_incomplete: bool,
        table_name: &str,
    ) -> Result<Self> {

        let table_data = Self::decode_table(data, definition, entry_count, return_incomplete)?;
        let table = Self {
            definition: definition.clone(),
            table_name: table_name.to_owned(),
            table_data: TableData::Local(table_data),
        };

        Ok(table)
    }
}
