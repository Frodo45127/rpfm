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
Example for listing all the DB Tables from a PackFile.
!*/

use std::path::PathBuf;

use rpfm_lib::packfile::PackFile;
use rpfm_lib::packedfile::PackedFileType;

fn main() {
    match PackFile::read(&PathBuf::from("test_files/example_list_tables.pack"), true) {
        Ok(packfile) => {
            let packed_files = packfile.get_ref_packed_files_by_type(&PackedFileType::DB, false);
            if packed_files.is_empty() {
                println!("No DB Tables in this PackedFile.");
            }
            else {
                println!("This PackFile contains the following tables:");
                packed_files.iter().for_each(|x| println!("- {}", x.get_path().join("/")));
            }
        }
        Err(error) => println!("{:?}", error),
    }
}
