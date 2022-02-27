//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
pub const DISPLAY_NAME_WARHAMMER_3: &str = "Warhammer 3";
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
pub const KEY_WARHAMMER_3: &str = "warhammer_3";
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

/// Default Implementation for `SupportedGames`.
impl Default for SupportedGames {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation for `SupportedGames`.
impl SupportedGames {

    /// This function builds and generates the entire SupportedGames list. For initialization.
    pub fn new() -> Self {
        let mut game_list = IndexMap::new();

        // Warhammer 3
        game_list.insert(KEY_WARHAMMER_3, GameInfo {
            display_name: DISPLAY_NAME_WARHAMMER_3,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_wh3.ron".to_owned(),
            dependencies_cache_file_name: "wh3.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: Some("language.txt".to_owned()),
            banned_packedfiles: vec![
                "db/agent_subtype_ownership_content_pack_junctions_tables".to_owned(),
                "db/allied_recruitment_unit_permissions_tables".to_owned(),
                "db/battle_ownership_content_pack_junctions_tables".to_owned(),
                "db/building_chain_ownership_content_pack_junctions_tables".to_owned(),
                "db/building_level_ownership_content_pack_junctions_tables".to_owned(),
                "db/campaign_map_playable_area_ownership_content_pack_junctions_tables".to_owned(),
                "db/faction_ownership_content_pack_junctions_tables".to_owned(),
                "db/loading_screen_quote_ownership_content_pack_junctions_tables".to_owned(),
                "db/main_unit_ownership_content_pack_junctions_tables".to_owned(),
                "db/ownership_products_tables".to_owned(),
                "db/ownership_content_packs_tables".to_owned(),
                "db/ownership_content_pack_required_products_tables".to_owned(),
                "db/ownership_content_pack_requirements_tables".to_owned(),
                "db/ritual_ownership_content_pack_junctions_tables".to_owned(),
                "db/technology_ownership_content_pack_junctions_tables".to_owned(),
            ],
            game_selected_icon: "gs_wh3.png".to_owned(),
            game_selected_big_icon: "gs_big_wh3.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_142_710,
                    executable: "Warhammer3.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1142710".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "banner_colour_primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "banner_colour_secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "banner_colour_tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "uniform_colour_primary".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "uniform_colour_secondary".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "uniform_colour_tertiary".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "faction_row".to_owned());
                vars
            },
        });

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
            dependencies_cache_file_name: "troy.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            game_selected_icon: "gs_troy.png".to_owned(),
            game_selected_big_icon: "gs_big_troy.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinEpic, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 0,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "mods/mymods".to_owned(),
                    downloaded_mods_path: "mods".to_owned(),
                });

                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_099_410,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1099410".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());


                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());

                vars
            },
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
            dependencies_cache_file_name: "3k.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_3k.png".to_owned(),
            game_selected_big_icon: "gs_big_3k.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),

            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 779_340,
                    executable: "Three_Kingdoms.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/779340".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "audio.pack".to_owned(),
                        "audio_bl.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_dlc06.pack".to_owned(),
                        "data_dlc07.pack".to_owned(),
                        "../../../data/data_dlc07.pack".to_owned(),
                        "data_ep.pack".to_owned(),
                        "data_mh.pack".to_owned(),
                        "data_yt.pack".to_owned(),
                        "data_yt_bl.pack".to_owned(),
                        "database.pack".to_owned(),
                        "fast.pack".to_owned(),
                        "fast_bl.pack".to_owned(),
                        "localisation/en/local_en.pack".to_owned(),     // English
                        "localisation/br/local_br.pack".to_owned(),     // Brazilian
                        "localisation/cz/local_cz.pack".to_owned(),     // Czech
                        "localisation/ge/local_ge.pack".to_owned(),     // German
                        "localisation/sp/local_sp.pack".to_owned(),     // Spanish
                        "localisation/fr/local_fr.pack".to_owned(),     // French
                        "localisation/it/local_it.pack".to_owned(),     // Italian
                        "localisation/kr/local_kr.pack".to_owned(),     // Korean
                        "localisation/pl/local_pl.pack".to_owned(),     // Polish
                        "localisation/ru/local_ru.pack".to_owned(),     // Russian
                        "localisation/tr/local_tr.pack".to_owned(),     // Turkish
                        "localisation/cn/local_cn.pack".to_owned(),     // Simplified Chinese
                        "localisation/zh/local_zh.pack".to_owned(),     // Traditional Chinese
                        "models.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "movies_bl.pack".to_owned(),
                        "movies_dlc06.pack".to_owned(),
                        "movies_ep.pack".to_owned(),
                        "movies_mh.pack".to_owned(),
                        "movies_wb.pack".to_owned(),
                        "movies_yt.pack".to_owned(),
                        "movies_yt_bl.pack".to_owned(),
                        "movies2.pack".to_owned(),
                        "shaders.pack".to_owned(),
                        "shaders_bl.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "terrain3.pack".to_owned(),
                        "terrain4.pack".to_owned(),
                        "terrain5.pack".to_owned(),
                        "variants.pack".to_owned(),
                        "variants_bl.pack".to_owned(),
                        "variants_dds.pack".to_owned(),
                        "variants_dds_bl.pack".to_owned(),
                        "vegetation.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 779_340,
                    executable: "ThreeKingdoms.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/779340".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
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
            dependencies_cache_file_name: "wh2.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_wh2.png".to_owned(),
            game_selected_big_icon: "gs_big_wh2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 594_570,
                    executable: "Warhammer2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/594570".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "audio.pack".to_owned(),
                        "audio_2.pack".to_owned(),
                        "audio_bl.pack".to_owned(),
                        "audio_bm.pack".to_owned(),
                        "audio_br.pack".to_owned(),
                        "audio_cst.pack".to_owned(),
                        "audio_gc.pack".to_owned(),
                        "audio_m.pack".to_owned(),
                        "audio_tk.pack".to_owned(),
                        "audio_we.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "campaign_variants.pack".to_owned(),
                        "campaign_variants_2.pack".to_owned(),
                        "campaign_variants_bl.pack".to_owned(),
                        "campaign_variants_pro09_.pack".to_owned(),
                        "campaign_variants_sb.pack".to_owned(),
                        "campaign_variants_twa02_.pack".to_owned(),
                        "campaign_variants_wp_.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_1.pack".to_owned(),
                        "data_2.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_bm.pack".to_owned(),
                        "data_gv.pack".to_owned(),
                        "data_hb.pack".to_owned(),
                        "data_pro09_.pack".to_owned(),
                        "data_pw.pack".to_owned(),
                        "data_sb.pack".to_owned(),
                        "data_sc.pack".to_owned(),
                        "data_tk.pack".to_owned(),
                        "data_twa01_.pack".to_owned(),
                        "data_twa02_.pack".to_owned(),
                        "data_we.pack".to_owned(),
                        "data_wp_.pack".to_owned(),

                        "localisation/en/local_en.pack".to_owned(),     // English
                        "localisation/br/local_br.pack".to_owned(),     // Brazilian
                        "localisation/cz/local_cz.pack".to_owned(),     // Czech
                        "localisation/ge/local_ge.pack".to_owned(),     // German
                        "localisation/sp/local_sp.pack".to_owned(),     // Spanish
                        "localisation/fr/local_fr.pack".to_owned(),     // French
                        "localisation/it/local_it.pack".to_owned(),     // Italian
                        "localisation/kr/local_kr.pack".to_owned(),     // Korean
                        "localisation/pl/local_pl.pack".to_owned(),     // Polish
                        "localisation/ru/local_ru.pack".to_owned(),     // Russian
                        "localisation/tr/local_tr.pack".to_owned(),     // Turkish
                        "localisation/cn/local_cn.pack".to_owned(),     // Simplified Chinese
                        "localisation/zh/local_zh.pack".to_owned(),     // Traditional Chinese

                        "localisation/en/local_en_2.pack".to_owned(),     // English
                        "localisation/br/local_br_2.pack".to_owned(),     // Brazilian
                        "localisation/cz/local_cz_2.pack".to_owned(),     // Czech
                        "localisation/ge/local_ge_2.pack".to_owned(),     // German
                        "localisation/sp/local_sp_2.pack".to_owned(),     // Spanish
                        "localisation/fr/local_fr_2.pack".to_owned(),     // French
                        "localisation/it/local_it_2.pack".to_owned(),     // Italian
                        "localisation/kr/local_kr_2.pack".to_owned(),     // Korean
                        "localisation/pl/local_pl_2.pack".to_owned(),     // Polish
                        "localisation/ru/local_ru_2.pack".to_owned(),     // Russian
                        "localisation/tr/local_tr_2.pack".to_owned(),     // Turkish
                        "localisation/cn/local_cn_2.pack".to_owned(),     // Simplified Chinese
                        "localisation/zh/local_zh_2.pack".to_owned(),     // Traditional Chinese

                        "localisation/en/local_en_gc.pack".to_owned(),     // English
                        "localisation/br/local_br_gc.pack".to_owned(),     // Brazilian
                        "localisation/cz/local_cz_gc.pack".to_owned(),     // Czech
                        "localisation/ge/local_ge_gc.pack".to_owned(),     // German
                        "localisation/sp/local_sp_gc.pack".to_owned(),     // Spanish
                        "localisation/fr/local_fr_gc.pack".to_owned(),     // French
                        "localisation/it/local_it_gc.pack".to_owned(),     // Italian
                        "localisation/kr/local_kr_gc.pack".to_owned(),     // Korean
                        "localisation/pl/local_pl_gc.pack".to_owned(),     // Polish
                        "localisation/ru/local_ru_gc.pack".to_owned(),     // Russian
                        "localisation/tr/local_tr_gc.pack".to_owned(),     // Turkish
                        "localisation/cn/local_cn_gc.pack".to_owned(),     // Simplified Chinese
                        "localisation/zh/local_zh_gc.pack".to_owned(),     // Traditional Chinese

                        "models.pack".to_owned(),
                        "models_2.pack".to_owned(),
                        "models_gc.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "models2_2.pack".to_owned(),
                        "models2_gc.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "movies_2.pack".to_owned(),
                        "movies2.pack".to_owned(),
                        "movies3.pack".to_owned(),
                        "shaders.pack".to_owned(),
                        "shaders_bl.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain_2.pack".to_owned(),
                        "terrain_gc.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "terrain2_2.pack".to_owned(),
                        "terrain2_gc.pack".to_owned(),
                        "terrain3.pack".to_owned(),
                        "terrain3_2.pack".to_owned(),
                        "terrain3_gc.pack".to_owned(),
                        "terrain4.pack".to_owned(),
                        "terrain4_2.pack".to_owned(),
                        "terrain5.pack".to_owned(),
                        "terrain7.pack".to_owned(),
                        "terrain7_2.pack".to_owned(),
                        "terrain7_gc.pack".to_owned(),
                        "terrain8.pack".to_owned(),
                        "terrain8_2.pack".to_owned(),
                        "terrain9.pack".to_owned(),
                        "variants.pack".to_owned(),
                        "variants_2.pack".to_owned(),
                        "variants_bl.pack".to_owned(),
                        "variants_dds.pack".to_owned(),
                        "variants_dds_2.pack".to_owned(),
                        "variants_dds_bl.pack".to_owned(),
                        "variants_dds_gc.pack".to_owned(),
                        "variants_dds_sb.pack".to_owned(),
                        "variants_dds_wp_.pack".to_owned(),
                        "variants_dds2.pack".to_owned(),
                        "variants_dds2_2.pack".to_owned(),
                        "variants_dds2_sb.pack".to_owned(),
                        "variants_dds2_sc.pack".to_owned(),
                        "variants_dds2_wp_.pack".to_owned(),
                        "variants_gc.pack".to_owned(),
                        "variants_hb.pack".to_owned(),
                        "variants_sb.pack".to_owned(),
                        "variants_sc.pack".to_owned(),
                        "variants_wp_.pack".to_owned(),
                        "warmachines.pack".to_owned(),
                        "warmachines_2.pack".to_owned(),
                        "warmachines_hb.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 594_570,
                    executable: "TotalWarhammer2.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/594570".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
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
            dependencies_cache_file_name: "wh.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_wh.png".to_owned(),
            game_selected_big_icon: "gs_big_wh.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 364_360,
                    executable: "Warhammer.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/364360".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "boot.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_bm.pack".to_owned(),
                        "data_ch.pack".to_owned(),
                        "data_m.pack".to_owned(),
                        "data_no.pack".to_owned(),
                        "data_we.pack".to_owned(),
                        "data_we_m.pack".to_owned(),

                        "localisation/local_en.pack".to_owned(),     // English
                        "localisation/local_br.pack".to_owned(),     // Brazilian
                        "localisation/local_cz.pack".to_owned(),     // Czech
                        "localisation/local_ge.pack".to_owned(),     // German
                        "localisation/local_sp.pack".to_owned(),     // Spanish
                        "localisation/local_fr.pack".to_owned(),     // French
                        "localisation/local_it.pack".to_owned(),     // Italian
                        "localisation/local_kr.pack".to_owned(),     // Korean
                        "localisation/local_pl.pack".to_owned(),     // Polish
                        "localisation/local_ru.pack".to_owned(),     // Russian
                        "localisation/local_tr.pack".to_owned(),     // Turkish
                        "localisation/local_cn.pack".to_owned(),     // Simplified Chinese
                        "localisation/local_zh.pack".to_owned(),     // Traditional Chinese

                        "localisation/local_en_bl.pack".to_owned(),     // English
                        "localisation/local_br_bl.pack".to_owned(),     // Brazilian
                        "localisation/local_cz_bl.pack".to_owned(),     // Czech
                        "localisation/local_ge_bl.pack".to_owned(),     // German
                        "localisation/local_sp_bl.pack".to_owned(),     // Spanish
                        "localisation/local_fr_bl.pack".to_owned(),     // French
                        "localisation/local_it_bl.pack".to_owned(),     // Italian
                        "localisation/local_kr_bl.pack".to_owned(),     // Korean
                        "localisation/local_pl_bl.pack".to_owned(),     // Polish
                        "localisation/local_ru_bl.pack".to_owned(),     // Russian
                        "localisation/local_tr_bl.pack".to_owned(),     // Turkish
                        "localisation/local_cn_bl.pack".to_owned(),     // Simplified Chinese
                        "localisation/local_zh_bl.pack".to_owned(),     // Traditional Chinese

                        "localisation/local_en_bm.pack".to_owned(),     // English
                        "localisation/local_br_bm.pack".to_owned(),     // Brazilian
                        "localisation/local_cz_bm.pack".to_owned(),     // Czech
                        "localisation/local_ge_bm.pack".to_owned(),     // German
                        "localisation/local_sp_bm.pack".to_owned(),     // Spanish
                        "localisation/local_fr_bm.pack".to_owned(),     // French
                        "localisation/local_it_bm.pack".to_owned(),     // Italian
                        "localisation/local_kr_bm.pack".to_owned(),     // Korean
                        "localisation/local_pl_bm.pack".to_owned(),     // Polish
                        "localisation/local_ru_bm.pack".to_owned(),     // Russian
                        "localisation/local_tr_bm.pack".to_owned(),     // Turkish
                        "localisation/local_cn_bm.pack".to_owned(),     // Simplified Chinese
                        "localisation/local_zh_bm.pack".to_owned(),     // Traditional Chinese

                        "localisation/local_en_we.pack".to_owned(),     // English
                        "localisation/local_br_we.pack".to_owned(),     // Brazilian
                        "localisation/local_cz_we.pack".to_owned(),     // Czech
                        "localisation/local_ge_we.pack".to_owned(),     // German
                        "localisation/local_sp_we.pack".to_owned(),     // Spanish
                        "localisation/local_fr_we.pack".to_owned(),     // French
                        "localisation/local_it_we.pack".to_owned(),     // Italian
                        "localisation/local_kr_we.pack".to_owned(),     // Korean
                        "localisation/local_pl_we.pack".to_owned(),     // Polish
                        "localisation/local_ru_we.pack".to_owned(),     // Russian
                        "localisation/local_tr_we.pack".to_owned(),     // Turkish
                        "localisation/local_cn_we.pack".to_owned(),     // Simplified Chinese
                        "localisation/local_zh_we.pack".to_owned(),     // Traditional Chinese

                        "models.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "shaders.pack".to_owned(),
                        "shaders_bl.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "terrain3.pack".to_owned(),
                        "terrain4.pack".to_owned(),
                        "terrain5.pack".to_owned(),
                        "terrain6.pack".to_owned(),
                        "terrain7.pack".to_owned(),
                        "variants.pack".to_owned(),
                        "variants_bl.pack".to_owned(),
                        "variants_dds.pack".to_owned(),
                        "variants_dds_bl.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 364_360,
                    executable: "TotalWarhammer.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/364360".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
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
            dependencies_cache_file_name: "tob.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_tob.png".to_owned(),
            game_selected_big_icon: "gs_big_tob.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 712_100,
                    executable: "Thrones.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/712100".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "blood.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "data.pack".to_owned(),
                        "localisation/en/local_en.pack".to_owned(),     // English
                        "localisation/br/local_br.pack".to_owned(),     // Brazilian
                        "localisation/cz/local_cz.pack".to_owned(),     // Czech
                        "localisation/ge/local_ge.pack".to_owned(),     // German
                        "localisation/sp/local_sp.pack".to_owned(),     // Spanish
                        "localisation/fr/local_fr.pack".to_owned(),     // French
                        "localisation/it/local_it.pack".to_owned(),     // Italian
                        "localisation/kr/local_kr.pack".to_owned(),     // Korean
                        "localisation/pl/local_pl.pack".to_owned(),     // Polish
                        "localisation/ru/local_ru.pack".to_owned(),     // Russian
                        "localisation/tr/local_tr.pack".to_owned(),     // Turkish
                        "localisation/cn/local_cn.pack".to_owned(),     // Simplified Chinese
                        "localisation/zh/local_zh.pack".to_owned(),     // Traditional Chinese
                        "models.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "models3.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "music.pack".to_owned(),
                        "sound.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "tiles.pack".to_owned(),
                        "tiles2.pack".to_owned(),
                        "tiles3.pack".to_owned(),
                        "tiles4.pack".to_owned(),
                        "viking.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 712_100,
                    executable: "ThronesOfBritannia.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/712100".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
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
            dependencies_cache_file_name: "att.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_att.png".to_owned(),
            game_selected_big_icon: "gs_big_att.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 325_610,
                    executable: "Attila.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/325610".to_owned(),
                });

                // Internal linux port, shares structure with the one for Windows.
                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 325_610,
                    executable: "Attila".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/325610".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
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
            dependencies_cache_file_name: "rom2.pak2".to_owned(),
            raw_db_version: 2,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_rom2.png".to_owned(),
            game_selected_big_icon: "gs_big_rom2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 214_950,
                    executable: "Rome2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/214950".to_owned(),
                });

                data
            },
            tool_vars: {
                let mut vars = HashMap::new();
                vars.insert("faction_painter_factions_table_name".to_owned(), "factions_tables".to_owned());
                vars.insert("faction_painter_factions_table_definition".to_owned(), "factions_definition".to_owned());
                vars.insert("faction_painter_factions_row_key".to_owned(), "faction_row".to_owned());

                vars.insert("faction_painter_banner_table_name".to_owned(), "faction_banners_tables".to_owned());
                vars.insert("faction_painter_banner_table_definition".to_owned(), "banner_definition".to_owned());
                vars.insert("faction_painter_banner_key_column_name".to_owned(), "key".to_owned());
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
        });

        // Shogun 2
        // TODO: Ensure the PFHVersions of this one are correct.
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
            dependencies_cache_file_name: "sho2.pak2".to_owned(),
            raw_db_version: 1,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_sho2.png".to_owned(),
            game_selected_big_icon: "gs_big_sho2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 34_330,
                    executable: "Shogun2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34330".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "boot.pack".to_owned(),
                        "bp_orig.pack".to_owned(),
                        "data.pack".to_owned(),
                        "localization/local_en.pack".to_owned(),     // English
                        "localization/local_br.pack".to_owned(),     // Brazilian
                        "localization/local_cz.pack".to_owned(),     // Czech
                        "localization/local_ge.pack".to_owned(),     // German
                        "localization/local_sp.pack".to_owned(),     // Spanish
                        "localization/local_fr.pack".to_owned(),     // French
                        "localization/local_it.pack".to_owned(),     // Italian
                        "localization/local_kr.pack".to_owned(),     // Korean
                        "localization/local_pl.pack".to_owned(),     // Polish
                        "localization/local_ru.pack".to_owned(),     // Russian
                        "localization/local_tr.pack".to_owned(),     // Turkish
                        "localization/local_cn.pack".to_owned(),     // Simplified Chinese
                        "localization/local_zh.pack".to_owned(),     // Traditional Chinese
                        "models.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "shaders.pack".to_owned(),
                        "sound.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "../fots/data_fots.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 34_330,
                    executable: "Shogun2.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34330".to_owned(),
                });

                data
            },
            tool_vars: HashMap::new(),
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
            dependencies_cache_file_name: "nap.pak2".to_owned(),
            raw_db_version: 0,
            supports_editing: true,
            db_tables_have_guid: false,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_nap.png".to_owned(),
            game_selected_big_icon: "gs_big_nap.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
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
                    use_manifest: false,
                    store_id: 34_030,
                    executable: "Napoleon.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34030".to_owned(),
                });

                data
            },
            tool_vars: HashMap::new(),
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
            dependencies_cache_file_name: "emp.pak2".to_owned(),
            raw_db_version: 0,
            supports_editing: true,
            db_tables_have_guid: false,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_emp.png".to_owned(),
            game_selected_big_icon: "gs_big_emp.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
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
                    use_manifest: false,
                    store_id: 10_500,
                    executable: "Empire.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/10500".to_owned(),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![
                        "anim.pack".to_owned(),
                        "battlepresets.pack".to_owned(),
                        "battleterrain.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "groupformations.pack".to_owned(),
                        "../languages/local_en.pack".to_owned(),     // English
                        "../languages/local_br.pack".to_owned(),     // Brazilian
                        "../languages/local_cz.pack".to_owned(),     // Czech
                        "../languages/local_ge.pack".to_owned(),     // German
                        "../languages/local_sp.pack".to_owned(),     // Spanish
                        "../languages/local_fr.pack".to_owned(),     // French
                        "../languages/local_it.pack".to_owned(),     // Italian
                        "../languages/local_kr.pack".to_owned(),     // Korean
                        "../languages/local_pl.pack".to_owned(),     // Polish
                        "../languages/local_ru.pack".to_owned(),     // Russian
                        "../languages/local_tr.pack".to_owned(),     // Turkish
                        "../languages/local_cn.pack".to_owned(),     // Simplified Chinese
                        "../languages/local_zh.pack".to_owned(),     // Traditional Chinese
                        "main.pack".to_owned(),
                        "models.pack".to_owned(),
                        "patch.pack".to_owned(),
                        "patch_media.pack".to_owned(),
                        "../languages/patch_en.pack".to_owned(),     // English Patch
                        "../languages/patch_br.pack".to_owned(),     // Brazilian Patch
                        "../languages/patch_cz.pack".to_owned(),     // Czech Patch
                        "../languages/patch_ge.pack".to_owned(),     // German Patch
                        "../languages/patch_sp.pack".to_owned(),     // Spanish Patch
                        "../languages/patch_fr.pack".to_owned(),     // French Patch
                        "../languages/patch_it.pack".to_owned(),     // Italian Patch
                        "../languages/patch_kr.pack".to_owned(),     // Korean Patch
                        "../languages/patch_pl.pack".to_owned(),     // Polish Patch
                        "../languages/patch_ru.pack".to_owned(),     // Russian Patch
                        "../languages/patch_tr.pack".to_owned(),     // Turkish Patch
                        "../languages/patch_cn.pack".to_owned(),     // Simplified Chinese Patch
                        "../languages/patch_zh.pack".to_owned(),     // Traditional Chinese Patch
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
                    use_manifest: false,
                    store_id: 10_500,
                    executable: "Empire.sh".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/10500".to_owned(),
                });

                data
            },
            tool_vars: HashMap::new(),
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
            dependencies_cache_file_name: "are.pack2".to_owned(),
            raw_db_version: -1,
            supports_editing: false,
            db_tables_have_guid: true,
            locale_file: None,
            banned_packedfiles: vec![],
            game_selected_icon: "gs_are.png".to_owned(),
            game_selected_big_icon: "gs_big_are.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinWargaming, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: false,
                    store_id: 0,
                    executable: "Arena.exe".to_owned(),
                    data_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "".to_owned(),
                });

                data
            },
            tool_vars: HashMap::new(),
        });

        Self {
            games: game_list,
        }
    }

    /// This function returns a GameInfo from a game name.
    pub fn get_supported_game_from_key(&self, key: &str) -> Result<&GameInfo> {
        self.games.get(key).ok_or_else(|| ErrorKind::GameNotSupported.into())
    }

    /// This function returns a vec with references to the full list of supported games.
    pub fn get_games(&self) -> Vec<&GameInfo> {
        self.games.values().collect::<Vec<&GameInfo>>()
    }
}
