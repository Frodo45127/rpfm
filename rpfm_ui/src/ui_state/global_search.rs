//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `UIState`.

This module contains the code needed to keep track of the current state of the UI.
!*/

use qt_widgets::menu::Menu;

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;

use crate::shortcuts::Shortcuts;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum represents a match when using the "Global Search" feature.
///  - `DB`: (path, Vec(column_name, column_number, row_number, text).
///  - `Loc`: (path, Vec(column_name, row_number, text)
#[derive(Debug, Clone)]
pub enum GlobalMatch {
    DB((Vec<String>, Vec<(String, i32, i64, String)>)),
    Loc((Vec<String>, Vec<(String, i32, i64, String)>)),
    Text((Vec<String>, Vec<(u64, u64, i64)>)),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `AppUI`.
impl GlobalMatch {
    
    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    fn new() {
        
    }
}
