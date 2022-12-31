//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code for the SQLite backend for DB Tables.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::error::Result;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

pub fn init_database() -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::memory();
    Pool::new(manager).map_err(From::from)
}
