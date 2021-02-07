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
This crate is a launcher for the RPFM UI, so it has a restart capability for updates.
!*/

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

use std::env::{args, current_dir, current_exe};
use std::path::PathBuf;
use std::process::*;

/// Guess you know what this function does....
fn main() {

    let rpfm_path: PathBuf = if cfg!(debug_assertions) {
        current_dir().unwrap()
    } else {
        let mut path = current_exe().unwrap();
        path.pop();
        path
    };

    let mut rpfm_ui_exe_path: PathBuf = current_exe().unwrap();
    rpfm_ui_exe_path.pop();

    if cfg!(target_os = "windows") {
        rpfm_ui_exe_path.push("rpfm_ui.exe");
    } else {
        rpfm_ui_exe_path.push("rpfm_ui");
    };

    let mut args = args().collect::<Vec<String>>();
    args.push("--booted_from_launcher".to_string());

    // Code 10 is what we use to restart the program on exit.
    while let Some(code) = Command::new(&rpfm_ui_exe_path)
        .current_dir(&rpfm_path)
        .args(&args[1..])
        .output().unwrap().status.code() {
        if code != 10 {
            exit(code);
        }
    }
}
