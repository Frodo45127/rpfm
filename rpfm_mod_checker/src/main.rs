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
use std::process::exit;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packfile::{Manifest, PackFile, PFHFileType};
use rpfm_lib::common::*;

/// Guess you know what this function does....
fn main() {
    let mut data_txt = check_data();
    let content_txt = check_content();
    data_txt.push_str(&content_txt);

    if let Err(error) = write_to_disk(&data_txt) {
        println!("Error while writing the results to disk: {}", error.to_terminal());
    }
}

/// This function checks for non-vanilla files in the data folder, and reports them.
fn check_data() -> String {

    let mut text = String::new();
    let data_path = match get_data_folder(None) {
        Ok(path) => path,
        Err(error) => {
            println!("Error while trying to get the data folder: {}", error.to_terminal());
            exit(1);
        }
    };

    let manifest = match Manifest::read_from_folder(&data_path) {
        Ok(path) => path,
        Err(error) => {
            println!("Error while trying to parse the manifest file: {}", error.to_terminal());
            exit(2);
        },
    };

    let files = match get_files_from_subdir(&data_path, true) {
        Ok(files) => files,
        Err(error) => {
            println!("Error while trying to identify the mods inside /data: {}", error.to_terminal());
            exit(3);
        }
    };

    let mut mod_files = files.iter()
        .filter(|x| !manifest.is_path_in_manifest(x) && !x.ends_with("manifest.txt"))
        .map(|y| y.to_str().unwrap().to_owned())
        .collect::<Vec<String>>();

    mod_files.sort();

    if !mod_files.is_empty() {
        text.push_str("Non-Vanilla files found in /data folder:");
        let entries = mod_files.join("\n - ");
        text.push_str("\n - ");
        text.push_str(&entries);
    }

    text
}

/// This function checks for non-vanilla files in the content folder of the game (content on Steam, mods on Epic), and reports them.
fn check_content() -> String {

    let mut text = String::new();
    let content_path = match get_content_folder(None) {
        Ok(path) => path,
        Err(error) => return error.to_string(),
    };

    let files = match get_files_from_subdir(&content_path, true) {
        Ok(files) => files,
        Err(error) => {
            println!("Error while trying to identify the mods inside /mods: {}", error.to_terminal());
            exit(4);
        }
    };

    let mut movie_files = files.iter()
        .filter_map(|x| PackFile::read(x, true).ok())
        .filter(|y| y.get_pfh_file_type() == PFHFileType::Movie)
        .map(|z| z.get_file_path().to_str().unwrap().to_owned())
        .collect::<Vec<String>>();

    movie_files.sort();

    if !movie_files.is_empty() {
        text.push_str("\n\nMovie Packs found in /mods folder (these mods wont show up in the launcher and will not work):");
        let entries = movie_files.join("\n - ");
        text.push_str("\n - ");
        text.push_str(&entries);
    }

    text
}

fn get_content_folder(base_path: Option<&Path>) -> Result<PathBuf> {
    let mut content_path = if let Some(path) = base_path {
        path.to_owned()
    } else {
        let mut path = std::env::current_exe()?;
        path.pop();
        path
    };

    let mut exe_path = content_path.clone();
    exe_path.push("Troy.exe");
    if exe_path.is_file() {
        content_path.push("mods");
        Ok(content_path)
    }
    else {
        Err(ErrorKind::NoHTMLError("\n\nNot in Troy's folder. Skipping content check.".to_owned()).into())
    }
}


fn get_data_folder(base_path: Option<&Path>) -> Result<PathBuf> {
    let mut data_path = if let Some(path) = base_path {
        path.to_owned()
    } else {
        let mut path = std::env::current_exe()?;
        path.pop();
        path
    };
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
