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
Module that defines the games this lib supports.

This module defines the list of games this lib support for any `Game-Specific` feature.
You should have no business here, except for supporting a new game.
!*/

use indexmap::IndexMap;

use std::collections::HashMap;

use rpfm_error::{Result, ErrorKind};

use crate::packfile::{PFHFileType, PFHVersion};
use super::{GameInfo, VanillaDBTableNameLogic, InstallData, InstallType};

// Display Name for all the Supported Games.
pub const DISPLAY_NAME_TROY: &str = "Troy";
pub const DISPLAY_NAME_THREE_KINGDOMS: &str = "Three Kingdoms";
pub const DISPLAY_NAME_WARHAMMER_2: &str = "Warhammer 2";
pub const DISPLAY_NAME_WARHAMMER: &str = "Warhammer";
pub const DISPLAY_NAME_THRONES_OF_BRITANNIA: &str = "Thrones of Britannia";
pub const DISPLAY_NAME_ATTILA: &str = "Attila";
pub const DISPLAY_NAME_ROME_2: &str = "Rome 2";
pub const DISPLAY_NAME_SHOGUN_2: &str = "Shogun 2";
pub const DISPLAY_NAME_NAPOLEON: &str = "Napoleon";
pub const DISPLAY_NAME_EMPIRE: &str = "Empire";
pub const DISPLAY_NAME_ARENA: &str = "Arena";

// Key for all the supported games.
pub const KEY_TROY: &str = "troy";
pub const KEY_THREE_KINGDOMS: &str = "three_kingdoms";
pub const KEY_WARHAMMER_2: &str = "warhammer_2";
pub const KEY_WARHAMMER: &str = "warhammer";
pub const KEY_THRONES_OF_BRITANNIA: &str = "thrones_of_britannia";
pub const KEY_ATTILA: &str = "attila";
pub const KEY_ROME_2: &str = "rome_2";
pub const KEY_SHOGUN_2: &str = "shogun_2";
pub const KEY_NAPOLEON: &str = "napoleon";
pub const KEY_EMPIRE: &str = "empire";
pub const KEY_ARENA: &str = "arena";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the list of games supported by this lib.
pub struct SupportedGames {

    /// List of games supported.
    games: IndexMap<&'static str, GameInfo>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `SupportedGames`.
impl SupportedGames {

    /// This function builds and generates the entire SupportedGames list. For initialization.
    pub fn new() -> Self {
        let mut game_list = IndexMap::new();

        // Troy
        game_list.insert(KEY_TROY, GameInfo {
            display_name: DISPLAY_NAME_TROY,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH6);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_troy.ron".to_owned(),
            depenencies_cache_file_name: "troy.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_troy.png".to_owned(),
            game_selected_big_icon: "gs_big_troy.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinEpic, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 0,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "mods/mymods".to_owned(),
                    downloaded_mods_path: "mods".to_owned(),
                });

                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 1_099_410,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1099410".to_owned(),
                });

                data
            }
        });

        // Three Kingdoms
        game_list.insert(KEY_THREE_KINGDOMS, GameInfo {
            display_name: DISPLAY_NAME_THREE_KINGDOMS,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_3k.ron".to_owned(),
            depenencies_cache_file_name: "3k.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_3k.png".to_owned(),
            game_selected_big_icon: "gs_big_3k.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),

            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["database.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 779_340,
                    executable: "Three_Kingdoms.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/779340".to_owned(),
                });

                data
            }
        });
        // Warhammer 2
        game_list.insert(KEY_WARHAMMER_2, GameInfo {
            display_name: DISPLAY_NAME_WARHAMMER_2,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_wh2.ron".to_owned(),
            depenencies_cache_file_name: "wh2.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_wh2.png".to_owned(),
            game_selected_big_icon: "gs_big_wh2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 594_570,
                    executable: "Warhammer2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/594570".to_owned(),
                });

                data
            }
        });

        // Warhammer
        game_list.insert(KEY_WARHAMMER, GameInfo {
            display_name: DISPLAY_NAME_WARHAMMER,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH4);
                data.insert(PFHFileType::Release, PFHVersion::PFH4);
                data.insert(PFHFileType::Patch, PFHVersion::PFH4);
                data.insert(PFHFileType::Mod, PFHVersion::PFH4);
                data.insert(PFHFileType::Movie, PFHVersion::PFH4);
                data
            },
            schema_file_name: "schema_wh.ron".to_owned(),
            depenencies_cache_file_name: "wh.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_wh.png".to_owned(),
            game_selected_big_icon: "gs_big_wh.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec![
                        "data.pack".to_owned(),         // Central data PackFile
                        "data_bl.pack".to_owned(),      // Blood DLC Data
                        "data_bm.pack".to_owned()       // Beastmen DLC Data
                    ],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 364_360,
                    executable: "Warhammer.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/364360".to_owned(),
                });

                data
            }
        });

        // Thrones of Britannia
        game_list.insert(KEY_THRONES_OF_BRITANNIA, GameInfo {
            display_name: DISPLAY_NAME_THRONES_OF_BRITANNIA,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH4);
                data.insert(PFHFileType::Release, PFHVersion::PFH4);
                data.insert(PFHFileType::Patch, PFHVersion::PFH4);
                data.insert(PFHFileType::Mod, PFHVersion::PFH4);
                data.insert(PFHFileType::Movie, PFHVersion::PFH4);
                data
            },
            schema_file_name: "schema_tob.ron".to_owned(),
            depenencies_cache_file_name: "tob.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_tob.png".to_owned(),
            game_selected_big_icon: "gs_big_tob.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 712_100,
                    executable: "Thrones.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/712100".to_owned(),
                });

                data
            }
        });

        // Attila
        game_list.insert(KEY_ATTILA, GameInfo {
            display_name: DISPLAY_NAME_ATTILA,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH4);
                data.insert(PFHFileType::Release, PFHVersion::PFH4);
                data.insert(PFHFileType::Patch, PFHVersion::PFH4);
                data.insert(PFHFileType::Mod, PFHVersion::PFH4);
                data.insert(PFHFileType::Movie, PFHVersion::PFH4);
                data
            },
            schema_file_name: "schema_att.ron".to_owned(),
            depenencies_cache_file_name: "att.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_att.png".to_owned(),
            game_selected_big_icon: "gs_big_att.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 325_610,
                    executable: "Attila.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/325610".to_owned(),
                });

                data
            }
        });

        // Rome 2
        game_list.insert(KEY_ROME_2, GameInfo {
            display_name: DISPLAY_NAME_ROME_2,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH4);
                data.insert(PFHFileType::Release, PFHVersion::PFH4);
                data.insert(PFHFileType::Patch, PFHVersion::PFH4);
                data.insert(PFHFileType::Mod, PFHVersion::PFH4);
                data.insert(PFHFileType::Movie, PFHVersion::PFH4);
                data
            },
            schema_file_name: "schema_rom2.ron".to_owned(),
            depenencies_cache_file_name: "rom2.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_rom2.png".to_owned(),
            game_selected_big_icon: "gs_big_rom2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data_rome2.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 214_950,
                    executable: "Rome2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/214950".to_owned(),
                });

                data
            }
        });

        // Shogun 2
        // TODO: Revisar los pfhversion de este.
        game_list.insert(KEY_SHOGUN_2, GameInfo {
            display_name: DISPLAY_NAME_SHOGUN_2,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH2);
                data.insert(PFHFileType::Release, PFHVersion::PFH2);
                data.insert(PFHFileType::Patch, PFHVersion::PFH2);
                data.insert(PFHFileType::Mod, PFHVersion::PFH3);
                data.insert(PFHFileType::Movie, PFHVersion::PFH2);
                data
            },
            schema_file_name: "schema_sho2.ron".to_owned(),
            depenencies_cache_file_name: "sho2.pak2".to_owned(),
            raw_db_version: 1,
            supports_editing: true,
            db_tables_have_guid: true,
            game_selected_icon: "gs_sho2.png".to_owned(),
            game_selected_big_icon: "gs_big_sho2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec!["data.pack".to_owned()],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                    ],
                    vanilla_packs: vec![],
                    store_id: 34_330,
                    executable: "Shogun2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34330".to_owned(),
                });

                data
            }
        });

        // Napoleon
        game_list.insert(KEY_NAPOLEON, GameInfo {
            display_name: DISPLAY_NAME_NAPOLEON,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH0);
                data.insert(PFHFileType::Release, PFHVersion::PFH0);
                data.insert(PFHFileType::Patch, PFHVersion::PFH0);
                data.insert(PFHFileType::Mod, PFHVersion::PFH0);
                data.insert(PFHFileType::Movie, PFHVersion::PFH0);
                data
            },
            schema_file_name: "schema_nap.ron".to_owned(),
            depenencies_cache_file_name: "nap.pak2".to_owned(),
            raw_db_version: 0,
            supports_editing: true,
            db_tables_have_guid: false,
            game_selected_icon: "gs_nap.png".to_owned(),
            game_selected_big_icon: "gs_big_nap.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec![                     // NOTE: Patches 5 and 7 has no table changes, so they should not be here.
                        "data.pack".to_owned(),         // Main DB PackFile
                        "patch.pack".to_owned(),        // First Patch
                        "patch2.pack".to_owned(),       // Second Patch
                        "patch3.pack".to_owned(),       // Third Patch
                        "patch4.pack".to_owned(),       // Fourth Patch
                        "patch6.pack".to_owned(),       // Six Patch
                    ],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),         // English
                        "local_en_patch.pack".to_owned(),   // English Patch
                        "local_br.pack".to_owned(),         // Brazilian
                        "local_br_patch.pack".to_owned(),   // Brazilian Patch
                        "local_cz.pack".to_owned(),         // Czech
                        "local_cz_patch.pack".to_owned(),   // Czech Patch
                        "local_ge.pack".to_owned(),         // German
                        "local_ge_patch.pack".to_owned(),   // German Patch
                        "local_sp.pack".to_owned(),         // Spanish
                        "local_sp_patch.pack".to_owned(),   // Spanish Patch
                        "local_fr.pack".to_owned(),         // French
                        "local_fr_patch.pack".to_owned(),   // French Patch
                        "local_it.pack".to_owned(),         // Italian
                        "local_it_patch.pack".to_owned(),   // Italian Patch
                        "local_kr.pack".to_owned(),         // Korean
                        "local_kr_patch.pack".to_owned(),   // Korean Patch
                        "local_pl.pack".to_owned(),         // Polish
                        "local_pl_patch.pack".to_owned(),   // Polish Patch
                        "local_ru.pack".to_owned(),         // Russian
                        "local_ru_patch.pack".to_owned(),   // Russian Patch
                        "local_tr.pack".to_owned(),         // Turkish
                        "local_tr_patch.pack".to_owned(),   // Turkish Patch
                        "local_cn.pack".to_owned(),         // Simplified Chinese
                        "local_cn_patch.pack".to_owned(),   // Simplified Chinese Patch
                        "local_zh.pack".to_owned(),         // Traditional Chinese
                        "local_zh_patch.pack".to_owned(),   // Traditional Chinese Patch
                    ],
                    vanilla_packs: vec![
                        "battleterrain.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "buildings.pack".to_owned(),
                        "data.pack".to_owned(),
                        "local_en.pack".to_owned(),         // English
                        "local_en_patch.pack".to_owned(),   // English Patch
                        "local_br.pack".to_owned(),         // Brazilian
                        "local_br_patch.pack".to_owned(),   // Brazilian Patch
                        "local_cz.pack".to_owned(),         // Czech
                        "local_cz_patch.pack".to_owned(),   // Czech Patch
                        "local_ge.pack".to_owned(),         // German
                        "local_ge_patch.pack".to_owned(),   // German Patch
                        "local_sp.pack".to_owned(),         // Spanish
                        "local_sp_patch.pack".to_owned(),   // Spanish Patch
                        "local_fr.pack".to_owned(),         // French
                        "local_fr_patch.pack".to_owned(),   // French Patch
                        "local_it.pack".to_owned(),         // Italian
                        "local_it_patch.pack".to_owned(),   // Italian Patch
                        "local_kr.pack".to_owned(),         // Korean
                        "local_kr_patch.pack".to_owned(),   // Korean Patch
                        "local_pl.pack".to_owned(),         // Polish
                        "local_pl_patch.pack".to_owned(),   // Polish Patch
                        "local_ru.pack".to_owned(),         // Russian
                        "local_ru_patch.pack".to_owned(),   // Russian Patch
                        "local_tr.pack".to_owned(),         // Turkish
                        "local_tr_patch.pack".to_owned(),   // Turkish Patch
                        "local_cn.pack".to_owned(),         // Simplified Chinese
                        "local_cn_patch.pack".to_owned(),   // Simplified Chinese Patch
                        "local_zh.pack".to_owned(),         // Traditional Chinese
                        "local_zh_patch.pack".to_owned(),   // Traditional Chinese Patch
                        "media.pack".to_owned(),
                        "patch.pack".to_owned(),
                        "patch_media.pack".to_owned(),
                        "patch_media2.pack".to_owned(),
                        "patch_media2.pack".to_owned(),
                        "patch2.pack".to_owned(),
                        "patch3.pack".to_owned(),
                        "patch4.pack".to_owned(),
                        "patch5.pack".to_owned(),
                        "patch6.pack".to_owned(),
                        "patch7.pack".to_owned(),
                        "rigidmodels.pack".to_owned(),
                        "sound.pack".to_owned(),
                        "variantmodels.pack".to_owned(),
                        "variantmodels2.pack".to_owned(),
                    ],
                    store_id: 34_030,
                    executable: "Napoleon.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34030".to_owned(),
                });

                data
            }
        });

        // Empire
        game_list.insert(KEY_EMPIRE, GameInfo {
            display_name: DISPLAY_NAME_EMPIRE,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH0);
                data.insert(PFHFileType::Release, PFHVersion::PFH0);
                data.insert(PFHFileType::Patch, PFHVersion::PFH0);
                data.insert(PFHFileType::Mod, PFHVersion::PFH0);
                data.insert(PFHFileType::Movie, PFHVersion::PFH0);
                data
            },
            schema_file_name: "schema_emp.ron".to_owned(),
            depenencies_cache_file_name: "emp.pak2".to_owned(),
            raw_db_version: 0,
            supports_editing: true,
            db_tables_have_guid: false,
            game_selected_icon: "gs_emp.png".to_owned(),
            game_selected_big_icon: "gs_big_emp.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    db_packs: vec![
                        "main.pack".to_owned(),         // Main DB PackFile
                        "models.pack".to_owned(),       // Models PackFile (contains model-related DB Tables)
                        "patch.pack".to_owned(),        // First Patch
                        "patch2.pack".to_owned(),       // Second Patch
                        "patch3.pack".to_owned(),       // Third Patch
                        "patch4.pack".to_owned(),       // Fourth Patch
                        "patch5.pack".to_owned(),       // Fifth Patch
                    ],
                    loc_packs: vec![
                        "local_en.pack".to_owned(),     // English
                        "patch_en.pack".to_owned(),     // English Patch
                        "local_br.pack".to_owned(),     // Brazilian
                        "patch_br.pack".to_owned(),     // Brazilian Patch
                        "local_cz.pack".to_owned(),     // Czech
                        "patch_cz.pack".to_owned(),     // Czech Patch
                        "local_ge.pack".to_owned(),     // German
                        "patch_ge.pack".to_owned(),     // German Patch
                        "local_sp.pack".to_owned(),     // Spanish
                        "patch_sp.pack".to_owned(),     // Spanish Patch
                        "local_fr.pack".to_owned(),     // French
                        "patch_fr.pack".to_owned(),     // French Patch
                        "local_it.pack".to_owned(),     // Italian
                        "patch_it.pack".to_owned(),     // Italian Patch
                        "local_kr.pack".to_owned(),     // Korean
                        "patch_kr.pack".to_owned(),     // Korean Patch
                        "local_pl.pack".to_owned(),     // Polish
                        "patch_pl.pack".to_owned(),     // Polish Patch
                        "local_ru.pack".to_owned(),     // Russian
                        "patch_ru.pack".to_owned(),     // Russian Patch
                        "local_tr.pack".to_owned(),     // Turkish
                        "patch_tr.pack".to_owned(),     // Turkish Patch
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "patch_cn.pack".to_owned(),     // Simplified Chinese Patch
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                        "patch_zh.pack".to_owned(),     // Traditional Chinese Patch
                    ],
                    vanilla_packs: vec![
                        "anim.pack".to_owned(),
                        "battlepresets.pack".to_owned(),
                        "battleterrain.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "groupformations.pack".to_owned(),
                        "local_en.pack".to_owned(),     // English
                        "local_br.pack".to_owned(),     // Brazilian
                        "local_cz.pack".to_owned(),     // Czech
                        "local_ge.pack".to_owned(),     // German
                        "local_sp.pack".to_owned(),     // Spanish
                        "local_fr.pack".to_owned(),     // French
                        "local_it.pack".to_owned(),     // Italian
                        "local_kr.pack".to_owned(),     // Korean
                        "local_pl.pack".to_owned(),     // Polish
                        "local_ru.pack".to_owned(),     // Russian
                        "local_tr.pack".to_owned(),     // Turkish
                        "local_cn.pack".to_owned(),     // Simplified Chinese
                        "local_zh.pack".to_owned(),     // Traditional Chinese
                        "main.pack".to_owned(),
                        "models.pack".to_owned(),
                        "patch.pack".to_owned(),
                        "patch_media.pack".to_owned(),
                        "patch_en.pack".to_owned(),     // English Patch
                        "patch_br.pack".to_owned(),     // Brazilian Patch
                        "patch_cz.pack".to_owned(),     // Czech Patch
                        "patch_ge.pack".to_owned(),     // German Patch
                        "patch_sp.pack".to_owned(),     // Spanish Patch
                        "patch_fr.pack".to_owned(),     // French Patch
                        "patch_it.pack".to_owned(),     // Italian Patch
                        "patch_kr.pack".to_owned(),     // Korean Patch
                        "patch_pl.pack".to_owned(),     // Polish Patch
                        "patch_ru.pack".to_owned(),     // Russian Patch
                        "patch_tr.pack".to_owned(),     // Turkish Patch
                        "patch_cn.pack".to_owned(),     // Simplified Chinese Patch
                        "patch_zh.pack".to_owned(),     // Traditional Chinese Patch
                        "patch2.pack".to_owned(),
                        "patch3.pack".to_owned(),
                        "patch4.pack".to_owned(),
                        "patch5.pack".to_owned(),
                        "seasurfaces.pack".to_owned(),
                        "sound_non_wavefile_data.pack".to_owned(),
                        "sounds.pack".to_owned(),
                        "sounds_animation_triggers.pack".to_owned(),
                        "sounds_campaign.pack".to_owned(),
                        "sounds_music.pack".to_owned(),
                        "sounds_other.pack".to_owned(),
                        "sounds_placeholder.pack".to_owned(),
                        "sounds_sfx.pack".to_owned(),
                        "subtitles.pack".to_owned(),
                        "supertexture.pack".to_owned(),
                        "terrain_templates.pack".to_owned(),
                        "testdata.pack".to_owned(),
                        "ui.pack".to_owned(),
                        "ui_movies.pack".to_owned(),
                        "voices.pack".to_owned(),
                    ],
                    store_id: 10_500,
                    executable: "Empire.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/10500".to_owned(),
                });

                data
            }
        });

        // NOTE: There are things that depend on the order of this list, and this game must ALWAYS be the last one.
        // Otherwise, stuff that uses this list will probably break.
        // Arena
        game_list.insert(KEY_ARENA, GameInfo {
            display_name: DISPLAY_NAME_ARENA,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_are.ron".to_owned(),
            depenencies_cache_file_name: "are.pack2".to_owned(),
            raw_db_version: -1,
            supports_editing: false,
            db_tables_have_guid: true,
            game_selected_icon: "gs_are.png".to_owned(),
            game_selected_big_icon: "gs_big_are.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinWargaming, InstallData {
                    db_packs: vec!["wad.pack".to_owned()],
                    loc_packs: vec!["local_ex.pack".to_owned()],
                    vanilla_packs: vec![],
                    store_id: 0,
                    executable: "Arena.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "".to_owned(),
                });

                data
            }
        });

        Self {
            games: game_list,
        }
    }

    /// This function returns a GameInfo from a game name.
    pub fn get_supported_game_from_key(&self, key: &str) -> Result<&GameInfo> {
        self.games.get(key).ok_or(ErrorKind::GameNotSupported.into())
    }

    /// This function returns a vec with references to the full list of supported games.
    pub fn get_games(&self) -> Vec<&GameInfo> {
        self.games.values().collect::<Vec<&GameInfo>>()
    }
}
