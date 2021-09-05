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
Module with all the code related to the main `UIState`.

This module contains the code needed to keep track of the current state of the UI.
!*/

use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

use rpfm_lib::diagnostics::Diagnostics;
use rpfm_lib::global_search::GlobalSearch;

use crate::app_ui::AppUI;
use crate::packedfile_views::PackedFileView;
use crate::packfile_contents_ui::PackFileContentsUI;
use self::shortcuts::Shortcuts;

pub mod shortcuts;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the info we need to keep track of the current state of the UI.
pub struct UIState {

    /// This stores the current state of the PackFile.
    is_modified: AtomicBool,

    /// This stores the current shortcuts in memory, so they can be re-applied when needed.
    shortcuts: Arc<RwLock<Shortcuts>>,

    /// This stores if we have put the `PackFile Contents` view in read-only mode.
    packfile_contents_read_only: AtomicBool,

    /// This stores the list to all the widgets of the open PackedFiles.
    open_packedfiles: Arc<RwLock<Vec<PackedFileView>>>,

    /// This stores the current operational mode of the application.
    operational_mode: Arc<RwLock<OperationalMode>>,

    /// This stores the current `GlobalSearch`.
    global_search: Arc<RwLock<GlobalSearch>>,

    /// This stores the current `Diagnostics`.
    diagnostics: Arc<RwLock<Diagnostics>>,
}

/// This enum represent the current ***Operational Mode*** for RPFM.
#[derive(Debug, Clone)]
pub enum OperationalMode {

    /// MyMod mode enabled. It contains the game folder name (warhammer_2) and the name of the MyMod PackFile.
    MyMod(String, String),

    /// Normal mode enabled.
    Normal,
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
            open_packedfiles: Arc::new(RwLock::new(vec![])),
            operational_mode: Arc::new(RwLock::new(OperationalMode::Normal)),
            global_search: Arc::new(RwLock::new(GlobalSearch::default())),
            diagnostics: Arc::new(RwLock::new(Diagnostics::default())),
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
    pub unsafe fn set_is_modified(&self, is_modified: bool, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        self.is_modified.store(is_modified, Ordering::SeqCst);
        AppUI::update_window_title(app_ui, pack_file_contents_ui);
    }

    /// This function returns the current Shortcuts.
    pub fn get_shortcuts(&self) -> Shortcuts{
        self.shortcuts.read().unwrap().clone()
    }

    /// This function returns a read-only non-locking guard to the Shortcuts.
    pub fn get_shortcuts_no_lock(&self) -> RwLockReadGuard<Shortcuts> {
        self.shortcuts.read().unwrap()
    }

    /// This function replaces the current Shortcuts with the provided one.
    pub fn set_shortcuts(&self, shortcuts: &Shortcuts) {
        *self.shortcuts.write().unwrap() = shortcuts.clone();
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
    pub fn get_open_packedfiles(&self) -> RwLockReadGuard<Vec<PackedFileView>> {
        self.open_packedfiles.read().unwrap()
    }

    /// This function returns the open packedfiles list with a writing lock. This acts kinda like a setter.
    ///
    /// Use this only if you need to perform multiple write operations with this.
    pub fn set_open_packedfiles(&self) -> RwLockWriteGuard<Vec<PackedFileView>> {
        self.open_packedfiles.write().unwrap()
    }

    /// This function returns a reference to the current `Operational Mode`.
    pub fn get_operational_mode(&self) -> OperationalMode {
        self.operational_mode.read().unwrap().clone()
    }

    /// This function sets the current operational mode of the application, depending on the provided MyMod path.
    pub fn set_operational_mode(&self, app_ui: &Rc<AppUI>, mymod_path: Option<&PathBuf>) {
        match mymod_path {

            // If we received a MyMod path, we enable the MyMod mode with that path.
            Some(path) => {

                // Get the `folder_name` and the `mod_name` of our "MyMod".
                let mut path = path.clone();
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                path.pop();
                let game_folder_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

                // Set the current mode to `MyMod`.
                *self.operational_mode.write().unwrap() = OperationalMode::MyMod(game_folder_name, mod_name);

                // Enable all the "MyMod" related actions.
                unsafe { app_ui.mymod_delete_selected.set_enabled(true); }
                unsafe { app_ui.mymod_import.set_enabled(true); }
                unsafe { app_ui.mymod_export.set_enabled(true); }
            }

            // If `None` has been provided, we disable the MyMod mode.
            None => {
                *self.operational_mode.write().unwrap() = OperationalMode::Normal;

                unsafe { app_ui.mymod_delete_selected.set_enabled(false); }
                unsafe { app_ui.mymod_import.set_enabled(false); }
                unsafe { app_ui.mymod_export.set_enabled(false); }
            }
        }
    }

    /// This function returns the current global search info.
    pub fn get_global_search(&self) -> GlobalSearch{
        self.global_search.read().unwrap().clone()
    }

    /// This function replaces the current global search with the provided one.
    pub fn set_global_search(&self, global_search: &GlobalSearch) {
        *self.global_search.write().unwrap() = global_search.clone();
    }

    /// This function returns the current diagnostics info.
    pub fn get_diagnostics(&self) -> Diagnostics {
        self.diagnostics.read().unwrap().clone()
    }

    /// This function replaces the current diagnostics with the provided one.
    pub fn set_diagnostics(&self, diagnostics: &Diagnostics) {
        *self.diagnostics.write().unwrap() = diagnostics.clone();
    }
}
