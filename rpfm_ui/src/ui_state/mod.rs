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

use qt_widgets::widget::Widget;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};

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

    /// This stores the current state of the PackFile.
    is_modified: AtomicBool,

    /// This stores the current shortcuts in memory, so they can be re-applied when needed.
    pub shortcuts: Arc<RwLock<Shortcuts>>,

    //s This stores if we have put the `PackFile Contents` view in read-only mode.
    packfile_contents_read_only: AtomicBool,

    /// This stores the list to all the widgets of the open PackedFiles.
    open_packedfiles: Arc<RwLock<BTreeMap<Vec<String>, &'static mut Widget>>>,

    /// This stores the current operational mode of the application.
    operational_mode: Arc<RwLock<OperationalMode>>,

    /// This stores the variable that tell us if we need to trigger a MyMod menu rebuild or not.
    mymod_menu_needs_rebuild: AtomicBool,

    /// This stores the current `GlobalSearch`.
    global_search: Arc<RwLock<GlobalSearch>>,
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
            packfile_contents_read_only: AtomicBool::new(false),
            open_packedfiles: Arc::new(RwLock::new(BTreeMap::new())),
            operational_mode: Arc::new(RwLock::new(OperationalMode::Normal)),
            mymod_menu_needs_rebuild: AtomicBool::new(false),
            global_search: Arc::new(RwLock::new(GlobalSearch::default())),
        }
    }
}

/// Implementation of `UIState`.
impl UIState {

    /// This function gets the flag that stores if the open PackFile has been modified or not.
    pub fn get_is_modified(&self) -> bool {
        self.is_modified.load(Ordering::SeqCst)
    }

    /// This function sets the flag that stores if the open PackFile has been modified or not.
    pub fn set_is_modified(&self, is_modified: bool) {
        self.is_modified.store(is_modified, Ordering::SeqCst);
    }

    /// This function gets if the `PackFile Contents` TreeView is in read-only mode or not.
    pub fn get_packfile_contents_read_only(&self) -> bool {
        self.packfile_contents_read_only.load(Ordering::SeqCst)
    }

    /// This function gets if the `PackFile Contents` TreeView is in read-only mode or not.
    pub fn set_packfile_contents_read_only(&self, is_read_only: bool) {
        self.packfile_contents_read_only.store(is_read_only, Ordering::SeqCst);
    }

    /// This function returns the open packedfiles list with a reading lock.
    pub fn get_open_packedfiles(&self) -> RwLockReadGuard<BTreeMap<Vec<String>, &'static mut Widget>> {
        self.open_packedfiles.read().unwrap()
    }

    /// This function returns the open packedfiles list with a writing lock. This acts kinda like a setter.
    ///
    /// Use this only if you need to perform multiple write operations with this.
    pub fn set_open_packedfiles(&self) -> RwLockWriteGuard<BTreeMap<Vec<String>, &'static mut Widget>> {
        self.open_packedfiles.write().unwrap()
    }

    /// This function returns a reference to the current `Operational Mode`.
    pub fn get_operational_mode(&self) -> OperationalMode { 
        self.operational_mode.read().unwrap().get_ref_operational_mode().clone()
    }

    /// This function sets the current operational mode of the application, depending on the provided MyMod path.
    pub fn set_operational_mode(&self, app_ui: &AppUI, mymod_path: Option<&PathBuf>) {
        self.operational_mode.write().unwrap().set_operational_mode(app_ui, mymod_path);
    }

    /// This function gets the flag for rebuilding the MyMod menu next time we try to open it.
    pub fn get_mymod_menu_needs_rebuild(&self) -> bool {
        self.mymod_menu_needs_rebuild.load(Ordering::SeqCst)
    }

    /// This function sets the flag for rebuilding the MyMod menu next time we try to open it.
    pub fn set_mymod_menu_needs_rebuild(&self, rebuild: bool) {
        self.mymod_menu_needs_rebuild.store(rebuild, Ordering::SeqCst);
    }

    /// This function returns the current global search info.
    pub fn get_global_search(&self) -> GlobalSearch{
        self.global_search.read().unwrap().clone()
    }

    /// This function replaces the current global search with the provided one.
    pub fn set_global_search(&self, global_search: &GlobalSearch) {
        *self.global_search.write().unwrap() = global_search.clone();
    }
}