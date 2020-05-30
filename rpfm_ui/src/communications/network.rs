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
This module defines the code used for network communication.
!*/

use restson::RestPath;
use serde_derive::{Serialize, Deserialize};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the response from the server when asking if there is a new release or not.
#[derive(Debug, Serialize, Deserialize)]
pub struct LastestRelease {
    pub name: String,
    pub html_url: String,
    pub body: String
}

/// This enum controls the possible responses from the server when checking for RPFM updates.
#[derive(Debug, Serialize, Deserialize)]
pub enum APIResponse {

    /// This means a major update was found.
    SuccessNewUpdate(LastestRelease),

    /// This means a minor update was found.
    SuccessNewUpdateHotfix(LastestRelease),

    /// This means no update was found.
    SuccessNoUpdate,

    /// This means don't know if there was an update or not, because the version we got was invalid.
    SuccessUnknownVersion,

    /// This means there was an error when checking for updates.
    Error,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `RestPath` for `LastestRelease`.
impl RestPath<()> for LastestRelease {
    fn get_path(_: ()) -> Result<String, restson::Error> {
        Ok(String::from("repos/frodo45127/rpfm/releases/latest"))
    }
}
