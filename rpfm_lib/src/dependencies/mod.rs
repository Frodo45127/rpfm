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
Module with all the code related to the `Dependencies`.

This module contains the code needed to manage the dependencies of the currently open PackFile.
!*/

use crate::DB;
use crate::PackedFile;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the dependency data for the different features within RPFM.
#[derive(Default, Debug, Clone)]
pub struct Dependencies {

    /// PackedFiles from the dependencies of the currently open PackFile.
    dependency_database: Vec<PackedFile>,

    /// DB Files from the Pak File of the current game. Only for dependency checking, do not use it as base for new tables.
    fake_dependency_database: Vec<DB>,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Dependencies`.
impl Dependencies {

    pub fn get_ref_dependency_database(&self) -> &[PackedFile] {
        &self.dependency_database
    }

    pub fn get_ref_mut_dependency_database(&mut self) -> &mut Vec<PackedFile> {
        &mut self.dependency_database
    }

    pub fn get_ref_fake_dependency_database(&self) -> &[DB] {
        &self.fake_dependency_database
    }

    pub fn get_ref_mut_fake_dependency_database(&mut self) -> &mut Vec<DB> {
        &mut self.fake_dependency_database
    }
}
