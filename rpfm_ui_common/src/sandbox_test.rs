//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for the sandbox functions.

use std::path::{Path, PathBuf};

use crate::sandbox::*;

#[test]
fn test_app_id_from_info() {
    let info = "[Application]\nname=com.github.frodo45127.rpfm\nruntime=runtime/org.kde.Platform/x86_64/6.10\n\n[Context]\nfilesystems=host;\n";
    assert_eq!(app_id_from_info(info), Some("com.github.frodo45127.rpfm".to_owned()));
}

#[test]
fn test_app_id_from_info_ignores_other_groups() {
    let info = "[Instance]\nname=not-an-app-id\n\n[Application]\nname=com.github.frodo45127.rpfm\n";
    assert_eq!(app_id_from_info(info), Some("com.github.frodo45127.rpfm".to_owned()));
}

#[test]
fn test_app_id_from_info_without_app_group() {
    assert_eq!(app_id_from_info("[Context]\nfilesystems=host;\n"), None);
}

#[test]
fn test_owner_app_data_path() {
    let path = Path::new("/home/frodo/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Total War WARHAMMER III");
    assert_eq!(owner_app_data_path(path), Some(PathBuf::from("/home/frodo/.var/app/com.valvesoftware.Steam")));
}

#[test]
fn test_owner_app_data_path_with_relocated_home() {
    let path = Path::new("/var/home/frodo/.var/app/net.lutris.Lutris/data/games");
    assert_eq!(owner_app_data_path(path), Some(PathBuf::from("/var/home/frodo/.var/app/net.lutris.Lutris")));
}

#[test]
fn test_owner_app_data_path_outside_apps_folder() {
    assert_eq!(owner_app_data_path(Path::new("/home/frodo/Games/Total War WARHAMMER III")), None);
    assert_eq!(owner_app_data_path(Path::new("/home/frodo/.var/app")), None);
}
