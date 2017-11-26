// In this file are all the functions that the UI needs to interact with the PackFile logic.
// As a rule, there should be no GTK-related stuff in this module or his childrens.

use std::fs::{
    File, DirBuilder
};
use std::path::{
    Path, PathBuf
};
use std::io::{
    Read, Write
};

use std::error::Error;

use ::packedfile::loc::Loc;

pub mod packfile;

/*
--------------------------------------------------------
                PackFile-Related Functions
--------------------------------------------------------
*/

// This function creates a new PackFile with the name received.
pub fn new_packfile(file_name: String) -> packfile::PackFile {
    let pack_file = packfile::PackFile::new_with_name(file_name);
    pack_file
}


// This function is used to open the PackFiles. It requires the path of the PackFile to open, and
// it returns the PackFile decoded (if success) or an error message (if error).
pub fn open_packfile(pack_file_path: PathBuf) -> Result<packfile::PackFile, String> {

    // First, we get his name and path.
    let mut pack_file_path_string = pack_file_path.clone();
    pack_file_path_string.pop();
    let pack_file_path_string = pack_file_path_string.to_str().unwrap().to_string();
    let pack_file_name = pack_file_path.file_name().unwrap().to_str().unwrap().to_string();

    // Then we open it, read it, and store his content in raw format.
    let mut file = File::open(&pack_file_path).expect("Couldn't open file");
    let mut pack_file_buffered = vec![];
    file.read_to_end(&mut pack_file_buffered).expect("Error reading file.");


    let pack_file: Result<packfile::PackFile, String>;

    // If the file has less than 4 bytes, the file is not valid.
    if pack_file_buffered.len() <= 4 {
        pack_file = Err(format!("The file doesn't even have 4 bytes."));
    }
    // If the header's first 4 bytes are "PFH5", it's a valid file, so we read it.
    else if ::common::latin1_to_string(&pack_file_buffered[0..4]) == "PFH5"  {
        pack_file = Ok(packfile::PackFile::read(pack_file_buffered, pack_file_name, pack_file_path_string));
    }
    // If we reach this point, the file is not valid.
    else {
        pack_file = Err(format!("The file is not a Warhammer 2 PackFile."));
    }

    pack_file
}


// This function is used to take an open PackFile, encode it and save it into the disk. We return
// a result with a message of success or error.
// It requires:
// - pack_file: a &mut pack_file::PackFile. It's the PackFile we are going to save.
// - new_path: an Option<PathBuf> with the path were we are going to save the PackFile. None if we
//             are saving it in the same path it's when we opened it.
pub fn save_packfile(
    pack_file: &mut packfile::PackFile,
    new_path: Option<PathBuf>
) -> Result<String, String> {

    // If we haven't received a new_path, we assume the path is the original path of the file.
    // If that one is empty too (should never happen), we panic and cry.
    if new_path != None {
        let mut pack_file_path = new_path.clone().unwrap();
        pack_file_path.pop();
        let pack_file_path_string = pack_file_path.to_str().unwrap().to_string();
        let pack_file_name_string = new_path.unwrap().file_name().unwrap().to_str().unwrap().to_string();

        pack_file.pack_file_extra_data.file_path = pack_file_path_string;
        pack_file.pack_file_extra_data.file_name = pack_file_name_string;
    }

    let pack_file_path_string;
    if !pack_file.pack_file_extra_data.file_path.is_empty() {
        pack_file_path_string = format!("{}/{}", pack_file.pack_file_extra_data.file_path, pack_file.pack_file_extra_data.file_name);
    }
    else {
        panic!("Saving an empty path is almost as bad as dividing by 0. Almost");
    }

    let pack_file_path = Path::new(&pack_file_path_string);

    // Once we have the destination path saved, we proceed to save the PackedFile to that path and
    // return Ok or one of the 2 possible errors.
    let save_result: Result<String, String>;

    match File::create(pack_file_path) {
        Ok(mut file) => {
            let pack_file_encoded: Vec<u8> = packfile::PackFile::save(pack_file);
            match file.write_all(&pack_file_encoded) {
                Ok(_) => {
                    save_result = Ok(format!("File saved succesfuly:\n{}", pack_file_path.display()))
                }
                Err(why) => {
                    save_result = Err(format!("Error while writing the following file to disk:\n{}\n\nThe problem reported is:\n{}", pack_file_path.display(), why.description()))
                },
            }
        }
        Err(why) => {
            save_result = Err(format!("Error while trying to write the following file to disk:\n{}\n\nThe problem reported is:\n{}", pack_file_path.display(), why.description()))
        }
    }
    save_result
}


// This function is used to add a file to a PackFile, processing it and turning it into a PackedFile.
// It returns the a success or error message, depending on whether the file has been added, or not.
// It requires:
// - pack_file: a &mut pack_file::PackFile. It's the PackFile where we are going add the file.
// - file_path: a PathBuf with the current path of the file.
// - tree_path: a Vec<String> with the path in the TreeView where we are going to add the file.
pub fn add_file_to_packfile(
    pack_file: &mut packfile::PackFile,
    file_path: PathBuf,
    tree_path: Vec<String>
) -> Result<String, String> {

    let result: Result<String, String>;

    // First we make a quick check to see if the file is already in the PackFile.
    let mut duplicated_file = false;
    for i in &pack_file.pack_file_data.packed_files {
        if &i.packed_file_path == &tree_path {
            duplicated_file = true;
            break;
        }
    }

    // If the file is not already in the PackFile, we add it. To add a file we:
    // - Increase the amount of files in the header by 1;
    // - We calculate the size in bytes of the file, and pass it to the decode function.
    // - We add the new PackedFile to the packed_files Vec<PackedFile> of the PackFile.
    if !duplicated_file {
        let mut file_data = vec![];
        let mut file = File::open(&file_path).expect("Couldn't open file");
        file.read_to_end(&mut file_data).expect("Error reading file.");
        pack_file.pack_file_header.packed_file_count += 1;
        let file_size = file_data.len() as u32;
        let new_packed_file = packfile::PackedFile::add(file_size, tree_path, file_data);
        pack_file.pack_file_data.packed_files.push(new_packed_file);

        result = Ok(format!("File added."));
    }
    else {
        result = Err(format!("There is already a file with that name in that folder. Delete that file first."));
    }

    result
}


// This function is used to delete a PackedFile or a group of PackedFiles under the same tree_path
// from the PackFile. We just need the open PackFile and the tree_path of the file/folder to delete.
pub fn delete_from_packfile(
    pack_file: &mut packfile::PackFile,
    tree_path: Vec<String>,) {

    let mut index: i32 = 0;
    let mut is_a_file = false;
    let mut is_a_folder = false;
    let tree_path = tree_path;

    // First, we check if the file is a folder or a file.
    for i in &pack_file.pack_file_data.packed_files  {
        if &i.packed_file_path == &tree_path {
            is_a_file = true;
            break;
        }
        index += 1;
    }

    if !is_a_file {
        for i in &pack_file.pack_file_data.packed_files  {
            if i.packed_file_path.starts_with(&tree_path) {
                is_a_folder = true;
                break;
            }
        }
    }

    // If it's a file, in order to delete it we need to:
    // - Reduce the amount of files in the header.
    // - We get his index (I think this needs a proper rework) of the PackedFile to delete.
    // - We remove the PackedFile from the PackFile.
    if is_a_file {
        pack_file.pack_file_header.packed_file_count -= 1;
        pack_file.pack_file_data.packed_files.remove(index as usize);
    }

    // If it's a folder, we remove all files using that exact tree_path in his own tree_path, one
    // by one.
    else if is_a_folder {
        index = 0;
        for _i in 0..pack_file.pack_file_data.packed_files.len() {
            if index as usize <= pack_file.pack_file_data.packed_files.len(){
                if pack_file.pack_file_data.packed_files[index as usize].packed_file_path.starts_with(&tree_path) {
                    pack_file.pack_file_header.packed_file_count -= 1;
                    pack_file.pack_file_data.packed_files.remove(index as usize);
                    index -= 1;
                }
            }
            else {
                break;
            }
            index += 1;
        }
    }
}


// This function is used to extract a PackedFile or a folder from the PackFile.
// It requires:
// - pack_file: the PackFile from where we want to extract the PackedFile.
// - tree_path: the tree_path of the PackedFile we want to extract.
// - extracted_path: the destination path of the file we want to extract.
pub fn extract_from_packfile(
    pack_file: &packfile::PackFile,
    tree_path: Vec<String>,
    extracted_path: PathBuf
) -> Result<String, String> {

    let result: Result<String, String>;
    let mut file_result: Result<String, String> = Err(format!("It's a folder, not a file."));

    let mut index: i32 = 0;
    let mut is_a_file = false;
    let mut is_a_folder = false;
    let tree_path = tree_path;

    let mut files_extracted = 0;
    let mut files_errors = 0;

    // First, we check if the tree_path is a folder or a file.
    for i in &pack_file.pack_file_data.packed_files {
        if &i.packed_file_path == &tree_path {
            is_a_file = true;
            break;
        }
        index += 1;
    }

    if !is_a_file {
        for i in &pack_file.pack_file_data.packed_files  {
            if i.packed_file_path.starts_with(&tree_path) {
                is_a_folder = true;
                break;
            }
        }
    }

    // If it's a file, we encode the PackedFile and write it in disk in a file.
    if is_a_file {
        match File::create(&extracted_path) {
            Ok(mut extracted_file) => {
                let packed_file_encoded: (Vec<u8>, Vec<u8>) = packfile::PackedFile::save(&pack_file.pack_file_data.packed_files[index as usize]);
                match extracted_file.write_all(&packed_file_encoded.1) {
                    Ok(_) => file_result = Ok(format!("File extracted succesfuly:\n{}", extracted_path.display())),
                    Err(why) => file_result = Err(format!("Error while writing the following file to disk:\n{}\n\nThe problem reported is:\n{}", extracted_path.display(), why.description())),
                }
            }
            Err(why) => file_result = Err(format!("Error while trying to write the following dile to disk:\n{}\n\nThe problem reported is:\n{}", extracted_path.display(), why.description())),
        }
    }

    // If it's a folder, first we get all the PackedFiles inside that folder.
    // Then we remove the start of the tree_path of the extracted files, as we only need the folder we
    // want to extract and his childrens.
    else if is_a_folder {
        let mut files_to_extract: Vec<packfile::PackedFile> = vec![];
        for i in &pack_file.pack_file_data.packed_files {
            if i.packed_file_path.starts_with(&tree_path) {
                files_to_extract.push(i.clone());
            }
        }

        let base_path = extracted_path.clone();
        let mut current_path = base_path.clone();

        for i in &mut files_to_extract {
            i.packed_file_path.drain(..(tree_path.len() - 1));

            for (j, k) in i.packed_file_path.iter().enumerate() {
                current_path.push(&k);

                // If the current String is the last one of the tree_path, it's a file, so we
                // write it into the disk.
                if (j + 1) == i.packed_file_path.len() {
                    match File::create(&current_path) {
                        Ok(mut extracted_file) => {
                            let packed_file_encoded: (Vec<u8>, Vec<u8>) = packfile::PackedFile::save(&i);
                            match extracted_file.write_all(&packed_file_encoded.1) {
                                Ok(_) => files_extracted += 1,
                                Err(_) => files_errors += 1,
                            }
                        }
                        Err(_) => files_errors += 1,
                    }
                }

                // If it's a folder, we create it and set is as the new parent. If it already exists,
                // we'll know by an error while creating it.
                else {
                    match DirBuilder::new().create(&current_path) {
                        Ok(_) => continue,
                        Err(_) => continue,
                    }
                }
            }
            current_path = base_path.clone();
        }
    }

    // Here we set the result we are going to return.
    if is_a_file {
        result = file_result;
    }
    else if files_errors > 0 {
        result = Err(format!("{} errors extracting files.", files_errors));
    }
    else {
        result = Ok(format!("{} files extracted. No errors detected.", files_extracted));
    }

    result
}


// This function is used to rename anything in the TreeView (yes, PackFile included).
// It requires:
// - pack_file: a &mut pack_file::PackFile. It's the PackFile opened.
// - tree_path: a Vec<String> with the tree_path of the file to rename.
// - new_name: the new name of the file to rename.
pub fn rename_packed_file(
    pack_file: &mut packfile::PackFile,
    tree_path: Vec<String>,
    new_name: &String
) -> Result<String, String> {

    let result: Result<String, String>;

    let mut index: i32 = 0;
    let mut is_a_file = false;
    let mut is_a_folder = false;
    let mut tree_path = tree_path;

    // First we check if the name is valid, and return an error if the new name is invalid.
    if new_name == tree_path.last().unwrap() {
        result = Err(format!("New name is the same as old name."));
    }
    else if new_name.is_empty() {
        result = Err(format!("Only my hearth can be empty."));
    }
    else if new_name.contains(" ") {
        result = Err(format!("Spaces are not valid characters."));
    }

    // If the name is valid, we check the length of the tree_path. If it's 1, we are renaming the
    // PackFile, not a PackedFile.
    else if tree_path.len() == 1 {
        pack_file.pack_file_extra_data.file_name = new_name.clone();
        result = Ok(format!("PackFile renamed."));
    }

    // If we reach this point, we can rename the file/folder.
    else {
        // First we delete the PackFile name from the tree_path, as we don't have that one stored
        // in our paths.
        tree_path.reverse();
        tree_path.pop();
        tree_path.reverse();

        // Second, we check if the file is a folder or a file.
        for i in &pack_file.pack_file_data.packed_files {
            if &i.packed_file_path == &tree_path {
                is_a_file = true;
                break;
            }
            index += 1;
        }

        if !is_a_file {
            for i in &pack_file.pack_file_data.packed_files {
                if i.packed_file_path.starts_with(&tree_path) {
                    is_a_folder = true;
                    break;
                }
            }
        }

        // Now we create the new tree_path, while conserving the old one for checks
        let mut new_tree_path = tree_path.clone();
        new_tree_path.pop();
        new_tree_path.push(new_name.clone());

        // If it's a file and it doesn't exist yet, we change the name of the file in the PackedFile
        // list. Otherwise, return an error.
        if is_a_file {
            let mut new_tree_path_already_exist = false;
            for i in &pack_file.pack_file_data.packed_files {
                if &i.packed_file_path == &new_tree_path {
                    new_tree_path_already_exist = true;
                    break;
                }
            }

            if !new_tree_path_already_exist {
                pack_file.pack_file_data.packed_files[index as usize].packed_file_path.pop();
                pack_file.pack_file_data.packed_files[index as usize].packed_file_path.push(new_name.clone());
                result = Ok(format!("File renamed."));
            }
            else {
                result = Err(format!("This name is already being used by another file in this path."));
            }
        }

        // If it's a folder, we check first if there is already a folder in the same folder with
        // that name, in which case we return an error.
        else if is_a_folder {

            let mut new_folder_already_exist = false;
            for i in &pack_file.pack_file_data.packed_files {
                if i.packed_file_path.starts_with(&new_tree_path) && i.packed_file_path.len() > new_tree_path.len() {
                    new_folder_already_exist = true;
                    break;
                }
            }

            // If the folder doesn't exist yet, we change the name of the folder we want to rename
            // in the path of every file that starts with his path.
            if !new_folder_already_exist {
                index = 0;
                let index_position = tree_path.len() - 1;
                for _i in 0..pack_file.pack_file_data.packed_files.len() {
                    if index as usize <= pack_file.pack_file_data.packed_files.len() {
                        if pack_file.pack_file_data.packed_files[index as usize].packed_file_path.starts_with(&tree_path) {
                            pack_file.pack_file_data.packed_files[index as usize].packed_file_path.remove(index_position);
                            pack_file.pack_file_data.packed_files[index as usize].packed_file_path.insert(index_position, new_name.clone());
                        }
                    } else {
                        break;
                    }
                    index += 1;
                }
                result = Ok(format!("Folder renamed."));
            }
            else {
                result = Err(format!("This name is already being used by another folder in this path."));
            }
        }
        else {
            result = Err(format!("This should never happend."));
        }
    }
    result
}

/*
--------------------------------------------------------
             PackedFile-Related Functions
--------------------------------------------------------
*/

// This function saves the data of the edited PackedFile in the main PackFile after a change has
// been done by the user. Checking for valid characters is done before this, so be careful to not break it.
pub fn update_packed_file_data(
    packed_file_data_decoded: &Loc,
    pack_file: &mut packfile::PackFile,
    index: usize,
) {
    let mut packed_file_data_encoded = Loc::save(&packed_file_data_decoded).to_vec();
    let packed_file_data_encoded_size = packed_file_data_encoded.len() as u32;

    // Replace the old raw data of the PackedFile with the new one, and update his size.
    &pack_file.pack_file_data.packed_files[index].packed_file_data.clear();
    &pack_file.pack_file_data.packed_files[index].packed_file_data.append(&mut packed_file_data_encoded);
    pack_file.pack_file_data.packed_files[index].packed_file_size = packed_file_data_encoded_size;
}


/*
--------------------------------------------------------
         Special PackedFile-Related Functions
--------------------------------------------------------
*/

// This function is used to patch and clean a PackFile exported with Terry, so the SiegeAI (if there
// is SiegeAI inplemented in the map) is patched and the extra useless .xml files are deleted.
// It requires a mut ref to a decoded PackFile, and returns an String (Result<Success, Error>).
pub fn patch_siege_ai (
    pack_file: &mut packfile::PackFile
) -> Result<String, String> {

    let save_result: Result<String, String>;

    let mut files_patched = 0;
    let mut files_deleted = 0;
    let mut files_to_delete: Vec<Vec<String>> = vec![];
    let mut packfile_is_empty = true;
    let mut multiple_defensive_hill_hints = false;

    // For every PackedFile in the PackFile we check first if it's in the usual map folder, as we
    // don't want to touch files outside that folder.
    for i in &mut pack_file.pack_file_data.packed_files {
        if i.packed_file_path.starts_with(&["terrain".to_string(), "tiles".to_string(), "battle".to_string(), "_assembly_kit".to_string()]) {
            if i.packed_file_path.last() != None {
                let x = i.packed_file_path.last().unwrap();
                packfile_is_empty = false;

                // If it's one of the possible candidates for Patching, we first check if it has
                // an Area Node in it, as that's the base for SiegeAI. If it has an Area Node,
                // we search the Defensive Hill and Patch it. After that, we check if there are
                // more Defensive Hills in the file. If there are more, we return success but
                // notify the modder that the file should have only one.
                if x == "bmd_data.bin"
                    || x == "catchment_01_layer_bmd_data.bin"
                    || x == "catchment_02_layer_bmd_data.bin"
                    || x == "catchment_03_layer_bmd_data.bin"
                    || x == "catchment_04_layer_bmd_data.bin"
                    || x == "catchment_05_layer_bmd_data.bin"
                    || x == "catchment_06_layer_bmd_data.bin"
                    || x == "catchment_07_layer_bmd_data.bin"
                    || x == "catchment_08_layer_bmd_data.bin"
                    || x == "catchment_09_layer_bmd_data.bin" {

                    if i.packed_file_data.windows(19).find(|window: &&[u8]
                            |String::from_utf8_lossy(window) == "AIH_SIEGE_AREA_NODE") != None {

                        let patch = "AIH_FORT_PERIMETER".to_string();
                        let index = i.packed_file_data.windows(18)
                            .position(
                                |window: &[u8]
                                |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL");

                        if index != None {
                            for j in 0..18 {
                                i.packed_file_data[index.unwrap() + (j as usize)] = patch.chars().nth(j).unwrap() as u8;
                            }
                            files_patched += 1;
                        }
                        if i.packed_file_data.windows(18).find(|window: &&[u8]
                                |String::from_utf8_lossy(window) == "AIH_DEFENSIVE_HILL") != None {
                            multiple_defensive_hill_hints = true;
                        }
                    }
                }

                // If it's an xml, we add it to the list of files_to_delete, as all the .xml files
                // in this folder are useless and only increase the size of the PackFile.
                else if x.ends_with(".xml") {
                    files_to_delete.push(i.packed_file_path.to_vec());
                }
            }
        }
    }

    // If there are files to delete, we delete them.
    if !files_to_delete.is_empty() {
        for i in files_to_delete.iter() {
            delete_from_packfile(pack_file, i.to_vec());
            files_deleted += 1;
        }
    }

    // And now we return success or error depending on what happened during the patching process.
    if packfile_is_empty {
        save_result = Err(format!("This packfile is empty, so we can't patch it."));
    }
    else if files_patched == 0 && files_deleted == 0 {
        save_result = Err(format!("There are not files in this Packfile that could be patched/deleted."));
    }
    else if files_patched >= 0 || files_deleted >= 0 {
        if files_patched == 0 {
            save_result = Ok(format!("No file suitable for patching has been found.\n{} files deleted.", files_deleted));
        }
        else if multiple_defensive_hill_hints {
            if files_deleted == 0 {
                save_result = Ok(format!("{} files patched.\nNo file suitable for deleting has been found.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit.",
                 files_patched));
            }
            else {
                save_result = Ok(format!("{} files patched.\n{} files deleted.\
                \n\n\
                WARNING: Multiple Defensive Hints have been found and we only patched the first one.\
                 If you are using SiegeAI, you should only have one Defensive Hill in the map (the \
                 one acting as the perimeter of your fort/city/castle). Due to SiegeAI being present, \
                 in the map, normal Defensive Hills will not work anyways, and the only thing they do \
                 is interfere with the patching process. So, if your map doesn't work properly after \
                 patching, delete all the extra Defensive Hill Hints. They are the culprit.",
                files_patched, files_deleted));
            }
        }
        else if files_deleted == 0 {
            save_result = Ok(format!("{} files patched.\nNo file suitable for deleting has been found.", files_patched));
        }
        else {
            save_result = Ok(format!("{} files patched.\n{} files deleted.", files_patched, files_deleted));
        }
    }
    else {
        save_result = Err(format!("This should never happend."));
    }

    save_result
}
