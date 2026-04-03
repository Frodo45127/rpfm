//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use qt_core::QEventLoop;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::search::GlobalSearch;

pub use rpfm_ipc::messages::OperationalMode;

use crate::app_ui::AppUI;
use crate::packedfile_views::FileView;
use crate::packfile_contents_ui::PackFileContentsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the info we need to keep track of the current state of the UI.
pub struct UIState {

    /// This stores the current state of the PackFile.
    is_modified: AtomicBool,

    /// This stores if we have put the `PackFile Contents` view in read-only mode.
    packfile_contents_read_only: AtomicBool,

    /// This stores the list to all the widgets of the open PackedFiles.
    open_packedfiles: Arc<RwLock<Vec<FileView>>>,

    /// This stores the current `GlobalSearch`.
    global_search: Arc<RwLock<GlobalSearch>>,

    /// This stores the current `Diagnostics`.
    diagnostics: Arc<RwLock<Diagnostics>>,
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
            packfile_contents_read_only: AtomicBool::new(false),
            open_packedfiles: Arc::new(RwLock::new(vec![])),
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

    /// This function gets if the `PackFile Contents` TreeView is in read-only mode or not.
    pub fn get_packfile_contents_read_only(&self) -> bool {
        self.packfile_contents_read_only.load(Ordering::SeqCst)
    }

    /// This function gets if the `PackFile Contents` TreeView is in read-only mode or not.
    pub fn set_packfile_contents_read_only(&self, is_read_only: bool) {
        self.packfile_contents_read_only.store(is_read_only, Ordering::SeqCst);
    }

    /// This function returns the open packedfiles list with a reading lock.
    pub fn get_open_packedfiles(&'_ self) -> RwLockReadGuard<'_, Vec<FileView>> {
        self.open_packedfiles.read().unwrap()
    }

    /// This function returns the open packedfiles list with a writing lock. This acts kinda like a setter.
    ///
    /// Use this only if you need to perform multiple write operations with this.
    pub fn set_open_packedfiles(&'_ self) -> RwLockWriteGuard<'_, Vec<FileView>> {
        loop {
            match self.open_packedfiles.try_write() {
                Ok(writer) => return writer,
                Err(error) => match error {
                    TryLockError::Poisoned(_) => panic!("Poisoned? This should never happen."),

                    // On blocking error, retrigger all UI events, as there should be one holding the lock.
                    // Keep doing it until the lock is freed an we can re-lock it here.
                    TryLockError::WouldBlock => {
                        let event_loop = unsafe { QEventLoop::new_0a() };
                        unsafe { event_loop.process_events_0a(); };
                    }
                }
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
