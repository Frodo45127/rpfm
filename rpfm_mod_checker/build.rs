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
Build script for the RPFM Mod Checker.

Here it goes all linking/cross-language compilation/platform-specific stuff that's needed in order to compile the RPFM Mod Checker.
!*/

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {

    // Icon/Exe info gets added here.
    let mut res = winres::WindowsResource::new();
    res.set_icon("./../img/rpfm.ico");
    res.set("LegalCopyright","Copyright (c) 2017-2020 Ismael Gutiérrez González");
    res.set("ProductName","Rusted PackFile Manager - Mod Checker Tool");
    if let Err(error) = res.compile() { println!("Error: {}", error); }
}

/// Generic Build Script.
#[cfg(not(target_os = "windows"))]
fn main() {}
