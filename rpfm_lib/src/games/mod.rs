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
use crate::packfile::PFHVersion;

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

/// This struct represents the list of games supported by this lib.
pub type SupportedGames = IndexMap<&'static str, GameInfo>;

/// This struct holds all the info needed for a game to be "supported" by RPFM features.
#[derive(Clone, Debug)]
pub struct GameInfo {

    /// This is the name it'll show up for the user. The *pretty name*. For example, in a dropdown (Warhammer 2).
    pub display_name: &'static str,

    /// This is the PFHVersion used at the start of every PackFile for that game.
    pub pfh_version: Vec<PFHVersion>,

    /// This is the full name of the schema file used for the game. For example: `schema_wh2.ron`.
    pub schema: String,

    /// These are the PackFiles from where we load the data for db references. Since 1.0, we use data.pack or equivalent for this.
    pub db_packs: Vec<String>,

    /// These are the PackFiles from where we load the data for loc special stuff.
    pub loc_packs: Vec<String>,

    /// This is the type of installs supported for the game in question, with install-specific data.
    pub install_type: Vec<InstallType>,

    /// This is the **type** of raw files the game uses. -1 is "Don't have Assembly Kit". 0 is Empire/Nappy. 1 is Shogun 2. 2 is anything newer than Shogun 2.
    pub raw_db_version: i16,

    /// This is the file containing the processed data from the raw db files from the Assembly Kit. If no Asskit is released for the game, set this to none.
    pub pak_file: Option<String>,

    /// This is the file used for checking scripts with Kailua. If there is no file, set it as None.
    pub ca_types_file: Option<String>,

    /// If we can save `PackFile` files for the game.
    pub supports_editing: bool,

    /// Name of the icon used to display the game as `Game Selected`, in an UI.
    pub game_selected_icon: String,

    /// Name of the big icon used to display the game as `Game Selected`, in an UI.
    pub game_selected_big_icon: String,

    /// Logic used to name vanilla tables.
    pub vanilla_db_table_name_logic: VanillaDBTableNameLogic,
}

/// This enum holds the info needed to deal with particularities of the games if they're installed from X or Y store.
#[derive(Clone, Debug)]
pub enum InstallType {

    /// Steam variant. Contains the steamID of the game.
    Steam(u64),

    /// Epic Store Variant.
    Epic,

    /// Wargaming Variant.
    Wargaming,
}

/// This enum holds the info about each game approach at naming db tables.
#[derive(Clone, Debug)]
pub enum VanillaDBTableNameLogic {

    /// This variant is for games where the table name is their folder's name.
    FolderName,

    /// This variant is for games where all tables are called the same.
    DefaultName(String),
}

/// This function returns a `SupportedGames` struct with the list of all games supported by this lib inside.
pub fn get_supported_games_list() -> SupportedGames {
    let mut list = SupportedGames::new();

    // Troy
    list.insert(KEY_TROY, GameInfo {
        display_name: DISPLAY_NAME_TROY,
        pfh_version: vec![PFHVersion::PFH6, PFHVersion::PFH5],
        schema: "schema_troy.ron".to_owned(),
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
        install_type: vec![InstallType::Epic],
        raw_db_version: 2,
        pak_file: Some("troy.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_troy.png".to_owned(),
        game_selected_big_icon: "gs_big_troy.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Three Kingdoms
    list.insert(KEY_THREE_KINGDOMS, GameInfo {
        display_name: DISPLAY_NAME_THREE_KINGDOMS,
        pfh_version: vec![PFHVersion::PFH5],
        schema: "schema_3k.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(779_340)],
        raw_db_version: 2,
        pak_file: Some("3k.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_3k.png".to_owned(),
        game_selected_big_icon: "gs_big_3k.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Warhammer 2
    list.insert(KEY_WARHAMMER_2, GameInfo {
        display_name: DISPLAY_NAME_WARHAMMER_2,
        pfh_version: vec![PFHVersion::PFH5],
        schema: "schema_wh2.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(594_570)],
        raw_db_version: 2,
        pak_file: Some("wh2.pak2".to_owned()),
        ca_types_file: Some("ca_types_wh2".to_owned()),
        supports_editing: true,
        game_selected_icon: "gs_wh2.png".to_owned(),
        game_selected_big_icon: "gs_big_wh2.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
    });

    // Warhammer
    list.insert(KEY_WARHAMMER, GameInfo {
        display_name: DISPLAY_NAME_WARHAMMER,
        pfh_version: vec![PFHVersion::PFH4],
        schema: "schema_wh.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(364_360)],
        raw_db_version: 2,
        pak_file: Some("wh.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_wh.png".to_owned(),
        game_selected_big_icon: "gs_big_wh.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Thrones of Britannia
    list.insert(KEY_THRONES_OF_BRITANNIA, GameInfo {
        display_name: DISPLAY_NAME_THRONES_OF_BRITANNIA,
        pfh_version: vec![PFHVersion::PFH4],
        schema: "schema_tob.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(712_100)],
        raw_db_version: 2,
        pak_file: Some("tob.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_tob.png".to_owned(),
        game_selected_big_icon: "gs_big_tob.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Attila
    list.insert(KEY_ATTILA, GameInfo {
        display_name: DISPLAY_NAME_ATTILA,
        pfh_version: vec![PFHVersion::PFH4],
        schema: "schema_att.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(325_610)],
        raw_db_version: 2,
        pak_file: Some("att.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_att.png".to_owned(),
        game_selected_big_icon: "gs_big_att.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Rome 2
    list.insert(KEY_ROME_2, GameInfo {
        display_name: DISPLAY_NAME_ROME_2,
        pfh_version: vec![PFHVersion::PFH4],
        schema: "schema_rom2.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(214_950)],
        raw_db_version: 2,
        pak_file: Some("rom2.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_rom2.png".to_owned(),
        game_selected_big_icon: "gs_big_rom2.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Shogun 2
    list.insert(KEY_SHOGUN_2, GameInfo {
        display_name: DISPLAY_NAME_SHOGUN_2,
        pfh_version: vec![PFHVersion::PFH3, PFHVersion::PFH2],
        schema: "schema_sho2.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(34_330)],
        raw_db_version: 1,
        pak_file: Some("sho2.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_sho2.png".to_owned(),
        game_selected_big_icon: "gs_big_sho2.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Napoleon
    list.insert(KEY_NAPOLEON, GameInfo {
        display_name: DISPLAY_NAME_NAPOLEON,
        pfh_version: vec![PFHVersion::PFH0],
        schema: "schema_nap.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(34_030)],
        raw_db_version: 0,
        pak_file: Some("nap.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_nap.png".to_owned(),
        game_selected_big_icon: "gs_big_nap.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // Empire
    list.insert(KEY_EMPIRE, GameInfo {
        display_name: DISPLAY_NAME_EMPIRE,
        pfh_version: vec![PFHVersion::PFH0],
        schema: "schema_emp.ron".to_owned(),
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
        install_type: vec![InstallType::Steam(10_500)],
        raw_db_version: 0,
        pak_file: Some("emp.pak2".to_owned()),
        ca_types_file: None,
        supports_editing: true,
        game_selected_icon: "gs_emp.png".to_owned(),
        game_selected_big_icon: "gs_big_emp.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    // NOTE: There are things that depend on the order of this list, and this game must ALWAYS be the last one.
    // Otherwise, stuff that uses this list will probably break.
    // Arena
    list.insert(KEY_ARENA, GameInfo {
        display_name: DISPLAY_NAME_ARENA,
        pfh_version: vec![PFHVersion::PFH5, PFHVersion::PFH4],
        schema: "schema_are.ron".to_owned(),
        db_packs: vec!["wad.pack".to_owned()],
        loc_packs: vec!["local_ex.pack".to_owned()],
        install_type: vec![InstallType::Wargaming],
        raw_db_version: -1,
        pak_file: None,
        ca_types_file: None,
        supports_editing: false,
        game_selected_icon: "gs_are.png".to_owned(),
        game_selected_big_icon: "gs_big_are.png".to_owned(),
        vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
    });

    list
}
