//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/// Build script for the RPFM Lib.

// Windows specific stuff.
#[cfg(target_os = "windows")]
fn main() {

	// This is to make RPFM able to see the qt_custom_rpfm lib file while building.
	println!("cargo:rustc-link-search=native=./libs");
	
    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libs/*");
}

#[cfg(not(target_os = "windows"))]
fn main() {

	// This is to make RPFM able to see the qt_custom_rpfm lib file while building.
	println!("cargo:rustc-link-search=native=./libs");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libs/*");
}
