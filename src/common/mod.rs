// In this file are all the helper functions used by the code (no GTK here)
// As we may or may not use them, all functions here should have the "#[allow(dead_code)]"
// var set, so the compiler doesn't spam us every time we try to compile.

use std::string::String;
use std::path::PathBuf;

use std::fs;
use std::path::Path;

pub mod coding_helpers;

// This function may be a little stupid, but it turn easely some bytes into a readable String.
#[allow(dead_code)]
pub fn latin1_to_string(s: &[u8]) -> String {
    s.iter()
        .map(|&c| {
            c as char
        })
        .collect()
}

// Turn a u32 into an array of 4 u8. For byte conversion.
#[allow(dead_code)]
pub fn u32_to_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

// Turn a u32 into an array of 4 u8, and reverse the array before returning it. For byte conversion.
#[allow(dead_code)]
pub fn u32_to_u8_reverse(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    let mut array = [b1, b2, b3, b4];
    array.reverse();
    return array
}

// Turn a u16 into an array of 2 u8, and reverse the array before returning it. For byte conversion.
#[allow(dead_code)]
pub fn u16_to_u8_reverse(x:u16) -> [u8;2] {
    let b1 : u8 = ((x >> 8) & 0xff) as u8;
    let b2 : u8 = (x & 0xff) as u8;
    let mut array = [b1, b2];
    array.reverse();
    return array
}

// This function takes a PathBuf and turn it into a Vec<String>.
// Useful for managing paths more easily.
#[allow(dead_code)]
pub fn path_to_vec_strings(path: PathBuf) -> Vec<String> {
    let mut path_processed: Vec<String> = vec![];

    for i in path.iter() {
        path_processed.push(i.to_str().unwrap().to_string());
    }

    path_processed
}

// This function takes a Vec<String> and concatenate them into one String.
// Useful for managing paths.
#[allow(dead_code)]
pub fn vec_strings_to_path_string(vec_strings: Vec<String>) -> String {
    let mut path_processed: String = String::new();

    for (i, j) in vec_strings.iter().enumerate() {
        path_processed.push_str(j);
        if (i + 1) < (vec_strings.len()) {
            path_processed.push_str("\\");
        }
    }
    path_processed
}
// This function takes a &Path and returns a Vec<PathBuf> with the paths of every file under the
// original &Path.
#[allow(dead_code)]
pub fn get_files_from_subdir(current_path: &Path) -> Vec<PathBuf> {

    let mut file_list: Vec<PathBuf> = vec![];

    // For every file in this folder
    for file in fs::read_dir(current_path).unwrap() {

        // Get his path
        let file_path = file.unwrap().path().clone();

        // If it's a file, to the file_list it goes
        if file_path.is_file() {
            file_list.push(file_path);
        }

        // If it's a folder, get all the files from it and his subfolders recursively
        else if file_path.is_dir() {
            let mut subfolder_files_path = get_files_from_subdir(&file_path);
            file_list.append(&mut subfolder_files_path);
        }
    }

    // Return the list of paths
    file_list
}