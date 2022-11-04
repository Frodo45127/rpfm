//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This crate is a simple program to restart RPFM when it gets updated.

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

use std::env::current_exe;
use std::process::*;
use std::thread;
use std::time;

fn main() {

    let mut path = current_exe().unwrap();
    path.pop();

    // Debug builds need to use the root repo folder as current dir.
    let rpfm_path = if cfg!(debug_assertions) {
        path.pop();
        path.pop();
        path
    } else {
        path
    };

    let mut rpfm_exe_path = current_exe().unwrap();
    rpfm_exe_path.pop();

    if cfg!(target_os = "windows") {
        rpfm_exe_path.push("rpfm_ui.exe");
    } else {
        rpfm_exe_path.push("rpfm_ui");
    };

    // Sleep for a sec to give the previous program time to close.
    let sec = time::Duration::from_millis(1000);
    thread::sleep(sec);

    Command::new(&rpfm_exe_path)
        .current_dir(&rpfm_path)
        .spawn().unwrap();
}
