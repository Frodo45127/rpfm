//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/// Build script for the entire project.
#[cfg(target_os = "windows")]
use winres;

// Windows specific stuff.
#[cfg(target_os = "windows")]
fn main() {

	// This is to make RPFM able to see the qt_custom_rpfm lib file while building.
	println!("cargo:rustc-link-search=native=./libs");
    println!("cargo:rustc-link-lib=dylib=qt_custom_rpfm");
	
    // Icon/Exe info gets added here.
    let mut res = winres::WindowsResource::new();
    res.set_icon("img/rpfm.ico");
    res.set("LegalCopyright","Copyright (c) 2017-2018 Ismael Gutiérrez González");
    res.set("ProductName","Rusted PackFile Manager");
    if let Err(error) = res.compile() { println!("Error: {}", std::error::Error::description(&error).to_string()); }

    // Force cargo to rerun this script if it's changed.
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(target_os = "windows"))]
fn main() {

	// This is to make RPFM able to see the qt_custom_rpfm lib file while building.
	println!("cargo:rustc-link-search=native=./libs");
    println!("cargo:rustc-link-lib=dylib=qt_custom_rpfm");

    // Force cargo to rerun this script if it's changed.
    println!("cargo:rerun-if-changed=build.rs");
}
