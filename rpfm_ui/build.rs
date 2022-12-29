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
Build script for the RPFM UI.

Here it goes all linking/cross-language compilation/platform-specific stuff that's needed in order to compile the RPFM UI.
!*/

use std::process::{Command, exit};
use std::io::{stderr, stdout, Write};

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {
    common_config();

    #[cfg(feature = "support_modern_dds")] {
        println!("cargo:rustc-link-lib=dylib=QImage_DDS");
    }

    // Rigidmodel lib, only on windows.
    #[cfg(feature = "support_rigidmodel")] {
        println!("cargo:rustc-link-lib=dylib=QtRMV2Widget");
    }

    // This compiles the custom widgets lib.
    match Command::new("nmake").current_dir("qt_subclasses/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();

            #[cfg(feature = "strict_subclasses_compilation")] {
                if !output.stderr.is_empty() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    error.lines().filter(|line| !line.is_empty()).for_each(|line| {
                        println!("cargo:warning={:?}", line);
                    });
                    exit(98)
                }
            }
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have nmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }

    // Icon/Exe info gets added here.
    let mut res = winres::WindowsResource::new();
    res.set_icon("./../icons/rpfm.ico");
    res.set("LegalCopyright","Copyright (c) - Ismael Gutiérrez González");
    res.set("ProductName","Rusted PackFile Manager");
    if let Err(error) = res.compile() { println!("Error: {}", error); }
}

/// Linux Build Script.
#[cfg(target_os = "linux")]
fn main() {
    common_config();

    // This compiles the custom widgets lib.
    match Command::new("make").current_dir("qt_subclasses/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();

            #[cfg(feature = "strict_subclasses_compilation")] {
                if !output.stderr.is_empty() {
                    println!("cargo:warning={:?}", String::from_utf8(output.stderr.to_vec()).unwrap());
                    exit(98)
                }
            }
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have make installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}

/// MacOS Build Script.
#[cfg(target_os = "macos")]
fn main() {
    common_config();

    // This compiles the custom widgets lib.
    match Command::new("gmake").current_dir("qt_subclasses/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();

            #[cfg(feature = "strict_subclasses_compilation")] {
                if !output.stderr.is_empty() {
                    println!("cargo:warning={:?}", String::from_utf8(output.stderr.to_vec()).unwrap());
                    exit(98)
                }
            }
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have gmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}

/// This function defines common configuration stuff for all platforms.
fn common_config() {

    // This is to make RPFM able to see the extra libs we need while building.
    println!("cargo:rustc-link-search=native=./libs");
    println!("cargo:rustc-link-lib=dylib=qt_subclasses");
    println!("cargo:rustc-link-lib=dylib=KF5Completion");
    println!("cargo:rustc-link-lib=dylib=KF5IconThemes");
    println!("cargo:rustc-link-lib=dylib=KF5TextEditor");
    println!("cargo:rustc-link-lib=dylib=KF5XmlGui");
    println!("cargo:rustc-link-lib=dylib=KF5WidgetsAddons");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=./libs/*");
    println!("cargo:rerun-if-changed=./rpfm_ui/build.rs");
    println!("cargo:rerun-if-changed=./rpfm_ui/qt_subclasses/*");

    // This creates the makefile for the custom widget lib.
    match Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_subclasses.pro")
        .current_dir("qt_subclasses/").output() {
        Ok(output) => {
            stdout().write_all(&output.stdout).unwrap();
            stderr().write_all(&output.stderr).unwrap();

            #[cfg(feature = "strict_subclasses_compilation")] {
                if !output.stderr.is_empty() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    error.lines().filter(|line| !line.is_empty()).for_each(|line| {
                        println!("cargo:warning={:?}", line);
                    });
                    exit(98)
                }
            }
        }
        Err(error) => {
            stdout().write_all(error.to_string().as_bytes()).unwrap();
            stdout().write_all(b"ERROR: You either don't have qmake installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
            exit(99);
        }
    }
}
