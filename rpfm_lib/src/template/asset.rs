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
Module with all the code to deal with template assets.
!*/

use serde_derive::{Serialize, Deserialize};

use crate::PackedFile;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a binary asset that will be added to the PackFile as part of the template.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Asset {

    /// Options required for the assets to be used in the template.
    pub required_options: Vec<String>,

    /// File path of the asset in the filesystem, relative to the base assets folder of the template.
    pub file_path: String,

    /// Path of the asset within the PackFile, once it gets added to the PackFile.
    pub packed_file_path: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of Asset.
impl Asset {

    /// This function builds a full TemplateDB from a PackedFile, if said PackedFile is a decodeable DB Table.
    pub fn new_from_packedfile(packed_file: &PackedFile) -> Self {
        let mut template = Self::default();
        template.file_path = packed_file.get_path().join("/");
        template.packed_file_path = packed_file.get_path().join("/");

        template
    }

    /// This function is used to check if we have all the options required to use this field in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }

    /// This function replaces the parametrized fields of the Asset with the user-provided values.
    pub fn replace_params(&mut self, key: &str, value: &str) {
        self.packed_file_path = self.packed_file_path.replace(&format!("{{@{}}}", key), value);
    }
}
