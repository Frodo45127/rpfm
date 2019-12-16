//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Build script for the RPFM UI.

Here it goes all linking/cross-language compilation/platform-specific stuff that's needed in order to compile the RPFM UI.
!*/

use std::process::{Command, exit};
use std::io::{stdout, Write};

/// This crate is only needed for the Windows Build.
#[cfg(target_os = "windows")]
use winres;

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {
    common_config();

    // These check whether you have qmake and nmake installed, because they're needed to get the custom widget's lib compiled.
    if Command::new("qmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have qmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(98);
    }

    if Command::new("nmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have nmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(99);
    }

    // This creates the makefile for the custom widget lib.
    Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_subclasses.pro")
        .current_dir("qt_subclasses/")
        .output().unwrap();

    // This compiles the custom widgets lib.
    Command::new("nmake")
        .current_dir("qt_subclasses/")
        .output().unwrap();

    // Icon/Exe info gets added here.
    let mut res = winres::WindowsResource::new();
    res.set_icon("img/rpfm.ico");
    res.set("LegalCopyright","Copyright (c) 2017-2019 Ismael Gutiérrez González");
    res.set("ProductName","Rusted PackFile Manager");
    if let Err(error) = res.compile() { println!("Error: {}", std::error::Error::description(&error).to_string()); }
}

/// Linux Build Script.
#[cfg(target_os = "linux")]
fn main() {
    common_config();

    // These check whether you have qmake and make installed, because they're needed to get the custom widget's lib compiled.
    if Command::new("qmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have qmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(98);
    }

    if Command::new("make").output().is_err() {
        stdout().write(b"ERROR: You either don't have make installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(99);
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

/// MacOS Build Script.
#[cfg(target_os = "macos")]
fn main() {
    common_config();

    // These check whether you have qmake and gmake installed, because they're needed to get the custom widget's lib compiled.
    if Command::new("qmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have qmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(98);
    }

    if Command::new("gmake").output().is_err() {
        stdout().write(b"ERROR: You either don't have gmake installed, or it's not in the path. Fix that before continuing.").unwrap();
        exit(99);
    }

    // This creates the makefile for the custom widget lib.
    Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_subclasses.pro")
        .current_dir("qt_subclasses/")
        .output().unwrap();

    // This compiles the custom widgets lib.
    Command::new("gmake")
        .current_dir("qt_subclasses/")
        .output().unwrap();
}

/// This function defines common configuration stuff for all platforms.
fn common_config() {

    // This is to make RPFM able to see the extra libs we need while building.
    println!("cargo:rustc-link-search=native=../libs/*");
    println!("cargo:rustc-link-lib=dylib=qt_subclasses");
    println!("cargo:rustc-link-lib=dylib=ktexteditor");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../libs/*");
    println!("cargo:rerun-if-changed=qt_subclasses/*");
}
