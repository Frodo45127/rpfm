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
This crate is a simple tool to make easy to find what's "modding" your game.
!*/

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use rpfm_error::Result;

use rpfm_lib::packfile::Manifest;
use rpfm_lib::common::*;

/// Guess you know what this function does....
fn main() {

    // It's simple:
    // - Get the current path.
    // - Check the data folder for a manifest.
    // - Parse the manifest.
    // - Get a list of all files in that folder not in the manifest.
    // - Put it in a txt file.
    // - Open the file in a text editor.
    let data_path = match get_data_folder(None) {
        Ok(path) => path,
        Err(error) => return println!("Error while trying to get the data folder: {}", error.to_terminal()),
    };

    let manifest = match Manifest::read_from_folder(&data_path) {
        Ok(path) => path,
        Err(error) => return println!("Error while trying to parse the manifest file: {}", error.to_terminal()),
    };

    let files = match get_files_from_subdir(&data_path) {
        Ok(files) => files,
        Err(error) => return println!("Error while trying to identify the mods inside /data: {}", error.to_terminal()),
    };

    let mut mod_files = files.iter()
        .filter(|x| !manifest.is_path_in_manifest(x))
        .map(|y| y.to_str().unwrap().to_owned())
        .collect::<Vec<String>>();

    mod_files.sort();

    let text = mod_files.join("\n");
    if let Err(error) = write_to_disk(&text) {
        println!("Error while writing the results to disk: {}", error.to_terminal());
    }
}

fn get_data_folder(base_path: Option<&Path>) -> Result<PathBuf> {
    let mut data_path = if let Some(path) = base_path {
        path.to_owned()
    } else {
        std::env::current_exe()?
    };
    data_path.pop();
    data_path.push("data");
    Ok(data_path)
}

fn write_to_disk(text: &str) -> Result<()> {
    let mut results_file_path = std::env::current_exe().unwrap();
    results_file_path.pop();
    results_file_path.push("mod_check_results.txt");

    let mut file = BufWriter::new(File::create(&results_file_path)?);
    file.write_all(text.as_bytes())?;
    open::that(&results_file_path)?;
    Ok(())
}
