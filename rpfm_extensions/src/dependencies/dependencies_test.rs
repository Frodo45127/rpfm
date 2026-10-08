//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Helpers to build dependencies for the tests of other modules.

use rpfm_lib::files::RFile;

use super::Dependencies;

impl Dependencies {

    /// This function creates dependencies holding the provided files as vanilla files, for tests.
    pub(crate) fn new_with_vanilla_files(files: Vec<RFile>) -> Self {
        Self {
            vanilla_files: files.into_iter().map(|file| (file.path_in_container_raw().to_owned(), file)).collect(),
            ..Default::default()
        }
    }
}
