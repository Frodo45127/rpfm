//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Build script for the RPFM Lib.

Here it goes all linking/cross-language compilation/platform-specific stuff that's needed in order to compile the RPFM Lib.
!*/

/// Windows specific stuff.
#[cfg(target_os = "windows")]
fn main() {
    common_config();
}

/// Linux specific stuff.
#[cfg(target_os = "linux")]
fn main() {
    common_config();
}

/// MacOS specific stuff.
#[cfg(target_os = "macos")]
fn main() {
    common_config();
}

/// This function defines common configuration stuff for all platforms.
fn common_config() {

    // This is to make RPFM able to see the LZMA lib file while building.
    println!("cargo:rustc-link-search=native=./libs");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=./libs/*");
    println!("cargo:rerun-if-changed=./rpfm_lib/build.rs");
}
