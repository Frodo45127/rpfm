//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with a SQLite Instance.
!*/

use rusqlite::Connection;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//



//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

pub fn init_database() -> Connection {
    Connection::open_in_memory().unwrap()
}
