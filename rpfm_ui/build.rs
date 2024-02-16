//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

#[cfg(target_os = "windows")] use std::fs::{copy, DirBuilder};
use std::io::{stderr, stdout, Write};
use std::process::{Command, exit};

/// Windows Build Script.
#[cfg(target_os = "windows")]
fn main() {
    common_config();
    let target_path = format!("./../target/{}/", if cfg!(debug_assertions) { "debug" } else { "release"});

    #[cfg(feature = "support_modern_dds")] {
        println!("cargo:rustc-link-lib=dylib=QImage_DDS");
    }

    // Rigidmodel lib, only on windows.
    #[cfg(feature = "support_rigidmodel")] {
        println!("cargo:rustc-link-lib=dylib=QtRMV2Widget");
    }

    // Model renderer, only on windows.
    #[cfg(feature = "support_model_renderer")] {
        let assets_path = "./../assets/";
        DirBuilder::new().recursive(true).create(assets_path).unwrap();

        println!("cargo:rustc-link-lib=dylib=ImportExport");
        println!("cargo:rustc-link-lib=dylib=Rldx");
        println!("cargo:rustc-link-lib=dylib=QtRenderingWidget");

        // This compiles the model renderer and related libs. Only in debug mode, as on releases we may not have access to the source code,
        // so we use precompiled binaries instead.
        if cfg!(debug_assertions) {

            // TODO: unhardcode this path once the folder is moved to a 3rdparty subrepo.
            let renderer_path = "./../../QtRenderingWidget/";
            println!("cargo:rerun-if-changed={}", renderer_path);

            match Command::new("msbuild")
                .arg("./QtRenderingWidget_RPFM.sln")
                .arg("-m")                      // Enable multithread build.
                .arg("-t:Build")
                //.arg("-t:Rebuild")            // If the linker misbehaves, use this instead of Build.
                .arg("-p:Configuration=Release")
                .arg("-p:Platform=x64")
                .current_dir(renderer_path).output() {
                Ok(output) => {
                    stdout().write_all(&output.stdout).unwrap();
                    stderr().write_all(&output.stderr).unwrap();

                    // On ANY error, fail compilation.
                    if !output.stderr.is_empty() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        error.lines().filter(|line| !line.is_empty()).for_each(|line| {
                            println!("cargo:warning={:?}", line);
                        });
                        exit(98)
                    }

                    // If nothing broke, copy the files to the correct folders.
                    copy(renderer_path.to_owned() + "x64/Release/ImportExport.lib", "./../3rdparty/builds/ImportExport.lib").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/QtRenderingWidget.lib", "./../3rdparty/builds/QtRenderingWidget.lib").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/Rldx.lib", "./../3rdparty/builds/Rldx.lib").unwrap();

                    copy(renderer_path.to_owned() + "x64/Release/PS_Attila_Weigted.cso", assets_path.to_owned() + "PS_Attila_Weigted.cso").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/PS_NoTextures.cso", assets_path.to_owned() + "PS_NoTextures.cso").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/PS_Simple.cso", assets_path.to_owned() + "PS_Simple.cso").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/PS_Three_Kingdoms.cso", assets_path.to_owned() + "PS_Three_Kingdoms.cso").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/PS_Troy.cso", assets_path.to_owned() + "PS_Troy.cso").unwrap();
                    copy(renderer_path.to_owned() + "x64/Release/VS_Simple.cso", assets_path.to_owned() + "VS_Simple.cso").unwrap();

                    copy(renderer_path.to_owned() + "Rldx/Rldx/RenderResources/Textures/CubeMaps/LandscapeCubeMapIBLDiffuse.dds", assets_path.to_owned() + "LandscapeCubeMapIBLDiffuse.dds").unwrap();
                    copy(renderer_path.to_owned() + "Rldx/Rldx/RenderResources/Textures/CubeMaps/LandscapeCubeMapIBLSpecular.dds", assets_path.to_owned() + "LandscapeCubeMapIBLSpecular.dds").unwrap();
                    copy(renderer_path.to_owned() + "Rldx/Rldx/RenderResources/Textures/CubeMaps/SkyCubemapIBLDiffuse.dds", assets_path.to_owned() + "SkyCubemapIBLDiffuse.dds").unwrap();
                    copy(renderer_path.to_owned() + "Rldx/Rldx/RenderResources/Textures/CubeMaps/SkyCubemapIBLSpecular.dds", assets_path.to_owned() + "SkyCubemapIBLSpecular.dds").unwrap();

                    copy(renderer_path.to_owned() + "QtRenderingWidget/myfile.spritefont", assets_path.to_owned() + "myfile.spritefont").unwrap();
                }
                Err(error) => {
                    stdout().write_all(error.to_string().as_bytes()).unwrap();
                    stdout().write_all(b"ERROR: You either don't have msbuild installed, it's not in the path, or there was an error while executing it. Fix that before continuing.").unwrap();
                    exit(99);
                }
            }
        }
    }

    // This compiles the custom widgets lib.
    match Command::new("nmake").current_dir("./../3rdparty/src/qt_rpfm_extensions/").output() {
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

    // Copy the icon theme so it can be accessed by debug builds.
    DirBuilder::new().recursive(true).create(target_path.clone() + "data/icons/breeze/").unwrap();
    DirBuilder::new().recursive(true).create(target_path.clone() + "data/icons/breeze-dark/").unwrap();
    copy("./../icons/breeze-icons.rcc", target_path.clone() + "data/icons/breeze/breeze-icons.rcc").unwrap();
    copy("./../icons/breeze-icons-dark.rcc", target_path.clone() + "data/icons/breeze-dark/breeze-icons-dark.rcc").unwrap();
}

/// Linux Build Script.
#[cfg(target_os = "linux")]
fn main() {
    common_config();

    // This compiles the custom widgets lib.
    match Command::new("make").current_dir("./../3rdparty/src/qt_rpfm_extensions/").output() {
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
    match Command::new("gmake").current_dir("./../3rdparty/src/qt_rpfm_extensions/").output() {
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
    println!("cargo:rustc-link-search=native=./3rdparty/builds");
    println!("cargo:rustc-link-lib=dylib=qt_rpfm_extensions");
    println!("cargo:rustc-link-lib=dylib=KF5Completion");
    println!("cargo:rustc-link-lib=dylib=KF5IconThemes");
    println!("cargo:rustc-link-lib=dylib=KF5TextEditor");
    println!("cargo:rustc-link-lib=dylib=KF5XmlGui");
    println!("cargo:rustc-link-lib=dylib=KF5WidgetsAddons");

    // Force cargo to rerun this script if any of these files is changed.
    println!("cargo:rerun-if-changed=./3rdparty/builds/*");
    println!("cargo:rerun-if-changed=./3rdparty/src/qt_rpfm_extensions/*");
    println!("cargo:rerun-if-changed=./rpfm_ui/build.rs");

    // This creates the makefile for the custom widget lib.
    match Command::new("qmake")
        .arg("-o")
        .arg("Makefile")
        .arg("qt_rpfm_extensions.pro")
        .current_dir("./../3rdparty/src/qt_rpfm_extensions/").output() {
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
