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

/// This struct contains all the pointers we need to access to EVERY widget/action created at the start of the program.
///
/// This means every widget/action that's created on start (menus, the TreeView,...) should be here.
pub struct UIState {

    //-------------------------------------------------------------------------------//
    // `Command Palette` DockWidget.
    //-------------------------------------------------------------------------------//
    pub is_modified: AtomicBool,
    pub shortcuts: Arc<RwLock<Shortcuts>>,
    pub disable_editing_from_packfile_contents: AtomicBool,
    pub open_packedfiles: Arc<RwLock<BTreeMap<Vec<String>, &'static mut Menu>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `AppUI`.
impl Default for UIState {
    
    /// This function creates an entire `AppUI` struct. Used to create the entire UI at start.
    fn default() -> Self {
        Self {
            is_modified: AtomicBool::new(false),
            shortcuts: Arc::new(RwLock::new(Shortcuts::load().unwrap_or_else(|_|Shortcuts::new()))),
            disable_editing_from_packfile_contents: AtomicBool::new(false),
            open_packedfiles: Arc::new(RwLock::new(BTreeMap::new())),
        }
        
    }
}
