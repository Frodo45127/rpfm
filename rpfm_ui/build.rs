//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::process::{Command, exit};
use std::io::{stdout, Write};

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
    res.set("LegalCopyright","Copyright (c) 2017-2019 Ismael Gutiérrez González");
    res.set("ProductName","Rusted PackFile Manager");
    if let Err(error) = res.compile() { println!("Error: {}", std::error::Error::description(&error).to_string()); }

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libs/*");
}

#[cfg(not(target_os = "windows"))]
fn main() {

	// This is to make RPFM able to see the qt_custom_rpfm lib file while building.
	println!("cargo:rustc-link-search=native=../libs/*");
    println!("cargo:rustc-link-lib=dylib=qt_subclasses");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../libs/*");
    println!("cargo:rerun-if-changed=qt_subclasses/*");

    // These check whether you have qmake and make installed, because they're needed to get the custom widget's lib compiled.
    if Command::new("qmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have qmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(99);
    } 

    if Command::new("make").output().is_err() {
        stdout().write(b"ERROR: You either don't have make installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(992);
    } 

    // This creates the makefile for the custom widget lib.
    Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_subclasses.pro")
        .current_dir("qt_subclasses/")
        .output().unwrap();

    // This compiles the custom widgets lib.
    Command::new("make")
        .current_dir("qt_subclasses/")
        .output().unwrap();
}
