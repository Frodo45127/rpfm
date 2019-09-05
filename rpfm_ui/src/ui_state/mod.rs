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
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crate::app_ui::AppUI;
use crate::shortcuts::Shortcuts;
use crate::ui_state::op_mode::OperationalMode;
use crate::ui_state::global_search::GlobalSearch;

mod op_mode;
pub mod global_search;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the info we need to keep track of the current state of the UI.
pub struct UIState {
    is_modified: AtomicBool,
    pub shortcuts: Arc<RwLock<Shortcuts>>,
    pub disable_editing_from_packfile_contents: AtomicBool,
    pub open_packedfiles: Arc<RwLock<BTreeMap<Vec<String>, &'static mut Menu>>>,
    pub operational_mode: Arc<RwLock<OperationalMode>>,
    mymod_menu_needs_rebuild: AtomicBool,
    pub global_search: Arc<RwLock<GlobalSearch>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `UIState`.
impl Default for UIState {
    
    /// This function creates an entire `UIState` struct. Used to create the initial `UIState`.
    fn default() -> Self {
        Self {
            is_modified: AtomicBool::new(false),
            shortcuts: Arc::new(RwLock::new(Shortcuts::load().unwrap_or_else(|_|Shortcuts::new()))),
            disable_editing_from_packfile_contents: AtomicBool::new(false),
            open_packedfiles: Arc::new(RwLock::new(BTreeMap::new())),
            operational_mode: Arc::new(RwLock::new(OperationalMode::Normal)),
            mymod_menu_needs_rebuild: AtomicBool::new(false),
            global_search: Arc::new(RwLock::new(GlobalSearch::default())),
        }
        
    }
}

/// Implementation of `UIState`.
impl UIState {

    /// This function sets the flag that stores if the open PackFile has been modified or not.
    pub fn set_is_modified(&self, is_modified: bool) {
        self.is_modified.store(is_modified, Ordering::SeqCst);
    }

    /// This function gets the flag that stores if the open PackFile has been modified or not.
    pub fn get_is_modified(&self) -> bool {
        self.is_modified.load(Ordering::SeqCst)
    }

    /// This function sets the current operational mode of the application, depending on the provided MyMod path.
    pub fn set_operational_mode(&self, app_ui: &AppUI, mymod_path: Option<&PathBuf>) {
        self.operational_mode.write().unwrap().set_operational_mode(app_ui, mymod_path);
    }

    /// This function returns a reference to the current `Operational Mode`.
    pub fn get_ref_operational_mode(&self) -> OperationalMode { 
        self.operational_mode.read().unwrap().get_ref_operational_mode().clone()
    }

    /// This function sets the flag for rebuilding the MyMod menu next time we try to open it.
    pub fn set_mymod_menu_needs_rebuild(&self, rebuild: bool) {
        self.mymod_menu_needs_rebuild.store(rebuild, Ordering::SeqCst);
    }

    /// This function gets the flag for rebuilding the MyMod menu next time we try to open it.
    pub fn get_mymod_menu_needs_rebuild(&self) -> bool {
        self.mymod_menu_needs_rebuild.load(Ordering::SeqCst)
    }
}