//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use std::collections::HashMap;

use super::{GameInfo, InstallData, InstallType, pfh_file_type::PFHFileType, pfh_version::PFHVersion, VanillaDBTableNameLogic};

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
    games: HashMap<&'static str, GameInfo>,

    /// Order the games were released.
    order: Vec<&'static str>
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Default for SupportedGames {
    fn default() -> Self {
        let mut game_list = HashMap::new();

        // Warhammer 3
        game_list.insert(KEY_WARHAMMER_3, GameInfo {
            key: KEY_WARHAMMER_3,
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
            portrait_settings_version: Some(4),
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![
                "db/agent_subtype_ownership_content_pack_junctions_tables".to_owned(),
                "db/allied_recruitment_core_units_tables".to_owned(),
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
            icon_small: "gs_wh3.png".to_owned(),
            icon_big: "gs_big_wh3.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_142_710,
                    store_id_ak: 1_880_380,
                    executable: "Warhammer3.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1142710".to_owned(),
                    config_folder: Some("Warhammer3".to_owned()),
                });

                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_142_710,
                    store_id_ak: 1_880_380,
                    executable: "TotalWarhammer3.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data/localisation".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1142710".to_owned(),
                    config_folder: None,
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
            lua_autogen_folder: Some("output/wh3".to_owned()),
            ak_lost_fields: vec![
                "_kv_battle_ai_ability_usage_variables/description".to_owned(),
                "_kv_experience_bonuses/description".to_owned(),
                "_kv_fatigue/description".to_owned(),
                "_kv_fire_values/description".to_owned(),
                "_kv_key_buildings/description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_naval_rules/description".to_owned(),
                "_kv_rules/description".to_owned(),
                "_kv_ui_tweakers/description".to_owned(),
                "_kv_unit_ability_scaling_rules/description".to_owned(),
                "_kv_winds_of_magic_params/description".to_owned(),
                "advice_levels/locatable".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "audio_entity_types/game_expansion_key".to_owned(),
                "audio_markers/colour_blue".to_owned(),
                "audio_markers/colour_green".to_owned(),
                "audio_markers/colour_red".to_owned(),
                "audio_metadata_tags/colour_blue".to_owned(),
                "audio_metadata_tags/colour_green".to_owned(),
                "audio_metadata_tags/colour_red".to_owned(),
                "audio_metadata_tags/game_expansion_key".to_owned(),
                "audio_metadata_tags/path".to_owned(),
                "battle_animations_table/game_expansion_key".to_owned(),
                "battle_personalities/game_expansion_key".to_owned(),
                "battle_set_pieces/game_expansion_key".to_owned(),
                "battle_skeletons/game_expansion_key".to_owned(),
                "battles/game_expansion_key".to_owned(),
                "battles/objectives_team_1".to_owned(),
                "battles/objectives_team_2".to_owned(),
                "building_chains/encyclopedia_group".to_owned(),
                "building_chains/encyclopedia_include_in_index".to_owned(),
                "building_culture_variants/flavour".to_owned(),
                "building_levels/commodity_vol".to_owned(),
                "cai_algorithm_variables/description".to_owned(),
                "cai_algorithms/description".to_owned(),
                "cai_decision_interfaces/description".to_owned(),
                "cai_decision_items_non_record_bound_types/description".to_owned(),
                "cai_decision_policies/description".to_owned(),
                "cai_domain_modifier_functions/description".to_owned(),
                "cai_domain_variables/description".to_owned(),
                "cai_domains/description".to_owned(),
                "cai_queries/description".to_owned(),
                "cai_query_variables/description".to_owned(),
                "cai_task_management_system_variables/description".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_map_playable_areas/game_expansion_key".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_playable_areas/preview_border".to_owned(),
                "campaign_payload_ui_details/comment".to_owned(),
                "campaign_settlement_display_building_ids/sub_culture".to_owned(),
                "campaign_tree_types/game_expansion_key".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/game_expansion_key".to_owned(),
                "cdir_event_targets/description".to_owned(),
                "cdir_events_options/notes".to_owned(),
                "cdir_events_payloads/notes".to_owned(),
                "cdir_events_targets/description".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "mercenary_pool_to_groups_junctions/game_expansion_key".to_owned(),

                // Special table. Ignore them.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),
                "models_building/tech_file".to_owned(),

                "names_groups/Description".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "naval_units/strengths_weaknesses_text".to_owned(),
                "pdlc/game_expansion_key".to_owned(),
                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "resources/campaign_group".to_owned(),
                "scripted_bonus_value_ids/notes".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_factions/honour".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
                "wind_levels/magnitudeX".to_owned(),
                "wind_levels/magnitudeY".to_owned(),
            ],
        });

        // Troy
        game_list.insert(KEY_TROY, GameInfo {
            key: KEY_TROY,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_troy.png".to_owned(),
            icon_big: "gs_big_troy.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinEpic, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 0,
                    store_id_ak: 0,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "mods/mymods".to_owned(),
                    downloaded_mods_path: "mods".to_owned(),
                    config_folder: None,
                });

                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_099_410,
                    store_id_ak: 1_356_310,
                    executable: "Troy.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1099410".to_owned(),
                    config_folder: Some("Troy_Steam".to_owned()),
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());


                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());

                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Three Kingdoms
        game_list.insert(KEY_THREE_KINGDOMS, GameInfo {
            key: KEY_THREE_KINGDOMS,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_3k.png".to_owned(),
            icon_big: "gs_big_3k.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),

            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![
                        "audio.pack".to_owned(),
                        "audio_bl.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_dlc06.pack".to_owned(),
                        "data_dlc07.pack".to_owned(),
                        "data_ep.pack".to_owned(),
                        "data_mh.pack".to_owned(),
                        "data_yt.pack".to_owned(),
                        "data_yt_bl.pack".to_owned(),
                        "database.pack".to_owned(),
                        "fast.pack".to_owned(),
                        "fast_bl.pack".to_owned(),
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
                    store_id_ak: 1_012_260,
                    executable: "Three_Kingdoms.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/779340".to_owned(),
                    config_folder: Some("ThreeKingdoms".to_owned()),
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
                    store_id_ak: 1_012_260,
                    executable: "ThreeKingdoms.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data/localisation".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/779340".to_owned(),
                    config_folder: None,
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });
        // Warhammer 2
        game_list.insert(KEY_WARHAMMER_2, GameInfo {
            key: KEY_WARHAMMER_2,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_wh2.png".to_owned(),
            icon_big: "gs_big_wh2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![
                        "audio.basepack".to_owned(),
                        "audio_base_2.pack".to_owned(),
                        "audio_base_bl.pack".to_owned(),
                        "audio_base_br.pack".to_owned(),
                        "audio_base_cst.pack".to_owned(),
                        "audio_base_m.pack".to_owned(),
                        "audio_base_tk.pack".to_owned(),

                        // English -- Needs to go first so others can overwrite it, because only a few languages have audio files.
                        "audio_en.pack".to_owned(),
                        "audio_en_2.pack".to_owned(),
                        "audio_en_br.pack".to_owned(),
                        "audio_en_cst.pack".to_owned(),
                        "audio_en_tk.pack".to_owned(),

                        // Brazilian - No audio.
                        // Czech - No audio.

                        // German
                        "audio_ge.pack".to_owned(),
                        "audio_ge_2.pack".to_owned(),
                        "audio_ge_bm.pack".to_owned(),
                        "audio_ge_br.pack".to_owned(),
                        "audio_ge_cst.pack".to_owned(),
                        "audio_ge_tk.pack".to_owned(),
                        "audio_ge_we.pack".to_owned(),

                        // Spanish
                        "audio_sp.pack".to_owned(),
                        "audio_sp_2.pack".to_owned(),
                        "audio_sp_bm.pack".to_owned(),
                        "audio_sp_br.pack".to_owned(),
                        "audio_sp_cst.pack".to_owned(),
                        "audio_sp_tk.pack".to_owned(),
                        "audio_sp_we.pack".to_owned(),

                        // French
                        "audio_fr.pack".to_owned(),
                        "audio_fr_2.pack".to_owned(),
                        "audio_fr_bm.pack".to_owned(),
                        "audio_fr_br.pack".to_owned(),
                        "audio_fr_cst.pack".to_owned(),
                        "audio_fr_tk.pack".to_owned(),
                        "audio_fr_we.pack".to_owned(),

                        // Italian
                        "audio_it.pack".to_owned(),
                        "audio_it_2.pack".to_owned(),
                        "audio_it_bm.pack".to_owned(),
                        "audio_it_br.pack".to_owned(),
                        "audio_it_cst.pack".to_owned(),
                        "audio_it_tk.pack".to_owned(),
                        "audio_it_we.pack".to_owned(),

                        // Korean - No audio.

                        // Polish
                        "audio_pl.pack".to_owned(),
                        "audio_pl_2.pack".to_owned(),
                        "audio_pl_bm.pack".to_owned(),
                        "audio_pl_br.pack".to_owned(),
                        "audio_pl_cst.pack".to_owned(),
                        "audio_pl_tk.pack".to_owned(),
                        "audio_pl_we.pack".to_owned(),

                        // Russian
                        "audio_ru.pack".to_owned(),
                        "audio_ru_2.pack".to_owned(),
                        "audio_ru_bm.pack".to_owned(),
                        "audio_ru_br.pack".to_owned(),
                        "audio_ru_cst.pack".to_owned(),
                        "audio_ru_tk.pack".to_owned(),
                        "audio_ru_we.pack".to_owned(),

                        // Turkish - No audio
                        // Simplified Chinese - No audio
                        // Traditional Chinese - No audio

                        "boot.pack".to_owned(),
                        "campaign_variants.pack".to_owned(),
                        "campaign_variants_2.pack".to_owned(),
                        "campaign_variants_bl.pack".to_owned(),
                        "campaign_variants_pro09_.pack".to_owned(),
                        "campaign_variants_sb.pack".to_owned(),
                        "campaign_variants_sf.pack".to_owned(),
                        "campaign_variants_twa02_.pack".to_owned(),
                        "campaign_variants_wp_.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_1.pack".to_owned(),
                        "data_2.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_bm.pack".to_owned(),
                        "data_gc.pack".to_owned(),
                        "data_hb.pack".to_owned(),
                        "data_pro09_.pack".to_owned(),
                        "data_pw.pack".to_owned(),
                        "data_sb.pack".to_owned(),
                        "data_sc.pack".to_owned(),
                        "data_sf.pack".to_owned(),
                        "data_tk.pack".to_owned(),
                        "data_twa01_.pack".to_owned(),
                        "data_twa02_.pack".to_owned(),
                        "data_we.pack".to_owned(),
                        "data_wp_.pack".to_owned(),

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

                        "local_en_2.pack".to_owned(),     // English
                        "local_br_2.pack".to_owned(),     // Brazilian
                        "local_cz_2.pack".to_owned(),     // Czech
                        "local_ge_2.pack".to_owned(),     // German
                        "local_sp_2.pack".to_owned(),     // Spanish
                        "local_fr_2.pack".to_owned(),     // French
                        "local_it_2.pack".to_owned(),     // Italian
                        "local_kr_2.pack".to_owned(),     // Korean
                        "local_pl_2.pack".to_owned(),     // Polish
                        "local_ru_2.pack".to_owned(),     // Russian
                        "local_tr_2.pack".to_owned(),     // Turkish
                        "local_cn_2.pack".to_owned(),     // Simplified Chinese
                        "local_zh_2.pack".to_owned(),     // Traditional Chinese

                        "local_en_gc.pack".to_owned(),     // English
                        "local_br_gc.pack".to_owned(),     // Brazilian
                        "local_cz_gc.pack".to_owned(),     // Czech
                        "local_ge_gc.pack".to_owned(),     // German
                        "local_sp_gc.pack".to_owned(),     // Spanish
                        "local_fr_gc.pack".to_owned(),     // French
                        "local_it_gc.pack".to_owned(),     // Italian
                        "local_kr_gc.pack".to_owned(),     // Korean
                        "local_pl_gc.pack".to_owned(),     // Polish
                        "local_ru_gc.pack".to_owned(),     // Russian
                        "local_tr_gc.pack".to_owned(),     // Turkish
                        "local_cn_gc.pack".to_owned(),     // Simplified Chinese
                        "local_zh_gc.pack".to_owned(),     // Traditional Chinese

                        "models.pack".to_owned(),
                        "models_2.pack".to_owned(),
                        "models_gc.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "models2_2.pack".to_owned(),
                        "models2_gc.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "movies_2.pack".to_owned(),
                        "movies_sf.pack".to_owned(),
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
                        "variants_dds_sf.pack".to_owned(),
                        "variants_dds_wp_.pack".to_owned(),
                        "variants_dds2.pack".to_owned(),
                        "variants_dds2_2.pack".to_owned(),
                        "variants_dds2_sb.pack".to_owned(),
                        "variants_dds2_sc.pack".to_owned(),
                        "variants_dds2_sf_.pack".to_owned(),
                        "variants_dds2_wp_.pack".to_owned(),
                        "variants_gc.pack".to_owned(),
                        "variants_hb.pack".to_owned(),
                        "variants_sb.pack".to_owned(),
                        "variants_sc.pack".to_owned(),
                        "variants_sf_.pack".to_owned(),
                        "variants_wp_.pack".to_owned(),
                        "warmachines.pack".to_owned(),
                        "warmachines_2.pack".to_owned(),
                        "warmachines_hb.pack".to_owned(),
                    ],
                    use_manifest: true,
                    store_id: 594_570,
                    store_id_ak: 651_460,
                    executable: "Warhammer2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/594570".to_owned(),
                    config_folder: Some("Warhammer2".to_owned())
                });
                // TODO: check this, it may have broken with the latest update.
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
                        "campaign_variants_sf.pack".to_owned(),
                        "campaign_variants_twa02_.pack".to_owned(),
                        "campaign_variants_wp_.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_1.pack".to_owned(),
                        "data_2.pack".to_owned(),
                        "data_bl.pack".to_owned(),
                        "data_bm.pack".to_owned(),
                        "data_gc.pack".to_owned(),
                        "data_hb.pack".to_owned(),
                        "data_pro09_.pack".to_owned(),
                        "data_pw.pack".to_owned(),
                        "data_sb.pack".to_owned(),
                        "data_sc.pack".to_owned(),
                        "data_sf.pack".to_owned(),
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
                        "movies_sf.pack".to_owned(),
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
                        "variants_dds_sf.pack".to_owned(),
                        "variants_dds_wp_.pack".to_owned(),
                        "variants_dds2.pack".to_owned(),
                        "variants_dds2_2.pack".to_owned(),
                        "variants_dds2_sb.pack".to_owned(),
                        "variants_dds2_sc.pack".to_owned(),
                        "variants_dds2_sf_.pack".to_owned(),
                        "variants_dds2_wp_.pack".to_owned(),
                        "variants_gc.pack".to_owned(),
                        "variants_hb.pack".to_owned(),
                        "variants_sb.pack".to_owned(),
                        "variants_sc.pack".to_owned(),
                        "variants_sf_.pack".to_owned(),
                        "variants_wp_.pack".to_owned(),
                        "warmachines.pack".to_owned(),
                        "warmachines_2.pack".to_owned(),
                        "warmachines_hb.pack".to_owned(),
                    ],
                    use_manifest: false,
                    store_id: 594_570,
                    store_id_ak: 651_460,
                    executable: "TotalWarhammer2.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data/localisation".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/594570".to_owned(),
                    config_folder: None,
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Warhammer
        game_list.insert(KEY_WARHAMMER, GameInfo {
            key: KEY_WARHAMMER,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_wh.png".to_owned(),
            icon_big: "gs_big_wh.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 364_360,
                    store_id_ak: 463_690,
                    executable: "Warhammer.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/364360".to_owned(),
                    config_folder: Some("Warhammer".to_owned())
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
                    store_id_ak: 463_690,
                    executable: "TotalWarhammer.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/364360".to_owned(),
                    config_folder: None,
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Thrones of Britannia
        game_list.insert(KEY_THRONES_OF_BRITANNIA, GameInfo {
            key: KEY_THRONES_OF_BRITANNIA,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_tob.png".to_owned(),
            icon_big: "gs_big_tob.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 712_100,
                    store_id_ak: 817_480,
                    executable: "thrones.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/712100".to_owned(),
                    config_folder: Some("ThronesofBritannia".to_owned())
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
                    store_id_ak: 817_480,
                    executable: "ThronesOfBritannia.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/712100".to_owned(),
                    config_folder: None,
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Attila
        game_list.insert(KEY_ATTILA, GameInfo {
            key: KEY_ATTILA,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_att.png".to_owned(),
            icon_big: "gs_big_att.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 325_610,
                    store_id_ak: 343_660,
                    executable: "Attila.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/325610".to_owned(),
                    config_folder: Some("Attila".to_owned())
                });

                // Internal linux port, shares structure with the one for Windows.
                data.insert(InstallType::LnxSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 325_610,
                    store_id_ak: 343_660,
                    executable: "Attila".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/325610".to_owned(),
                    config_folder: None,
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Rome 2
        game_list.insert(KEY_ROME_2, GameInfo {
            key: KEY_ROME_2,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_rom2.png".to_owned(),
            icon_big: "gs_big_rom2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 214_950,
                    store_id_ak: 267_180,
                    executable: "Rome2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/214950".to_owned(),
                    config_folder: Some("Rome2".to_owned()),
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
                vars.insert("faction_painter_banner_primary_colour_column_name".to_owned(), "primary_hex".to_owned());
                vars.insert("faction_painter_banner_secondary_colour_column_name".to_owned(), "secondary_hex".to_owned());
                vars.insert("faction_painter_banner_tertiary_colour_column_name".to_owned(), "tertiary_hex".to_owned());
                vars.insert("faction_painter_banner_row_key".to_owned(), "banner_row".to_owned());

                vars.insert("faction_painter_uniform_table_name".to_owned(), "faction_uniform_colours_tables".to_owned());
                vars.insert("faction_painter_uniform_table_definition".to_owned(), "uniform_definition".to_owned());
                vars.insert("faction_painter_uniform_key_column_name".to_owned(), "faction_name".to_owned());
                vars.insert("faction_painter_uniform_primary_colour_column_name".to_owned(), "primary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_secondary_colour_column_name".to_owned(), "secondary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_tertiary_colour_column_name".to_owned(), "tertiary_colour_hex".to_owned());
                vars.insert("faction_painter_uniform_row_key".to_owned(), "uniform_row".to_owned());
                vars
            },
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Shogun 2
        // TODO: Ensure the PFHVersions of this one are correct.
        game_list.insert(KEY_SHOGUN_2, GameInfo {
            key: KEY_SHOGUN_2,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_sho2.png".to_owned(),
            icon_big: "gs_big_sho2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 34_330,
                    store_id_ak: 202_930,
                    executable: "shogun2.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34330".to_owned(),
                    config_folder: Some("Shogun2".to_owned())
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
                    store_id_ak: 202_930,
                    executable: "Shogun2.sh".to_owned(),
                    data_path: "share/data/data".to_owned(),
                    language_path: "share/data/data".to_owned(),
                    local_mods_path: "share/data/data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34330".to_owned(),
                    config_folder: None,
                });

                data
            },
            tool_vars: HashMap::new(),
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Napoleon
        game_list.insert(KEY_NAPOLEON, GameInfo {
            key: KEY_NAPOLEON,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: false,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_nap.png".to_owned(),
            icon_big: "gs_big_nap.png".to_owned(),
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
                    store_id_ak: 0,
                    executable: "Napoleon.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/34030".to_owned(),
                    config_folder: Some("Napoleon".to_owned()),
                });

                data
            },
            tool_vars: HashMap::new(),
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // Empire
        game_list.insert(KEY_EMPIRE, GameInfo {
            key: KEY_EMPIRE,
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
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: false,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_emp.png".to_owned(),
            icon_big: "gs_big_emp.png".to_owned(),
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
                    store_id_ak: 0,
                    executable: "Empire.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/10500".to_owned(),
                    config_folder: Some("Empire".to_owned()),
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
                    store_id_ak: 0,
                    executable: "Empire.sh".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/10500".to_owned(),
                    config_folder: None,
                });

                data
            },
            tool_vars: HashMap::new(),
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        // NOTE: There are things that depend on the order of this list, and this game must ALWAYS be the last one.
        // Otherwise, stuff that uses this list will probably break.
        // Arena
        game_list.insert(KEY_ARENA, GameInfo {
            key: KEY_ARENA,
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
            portrait_settings_version: None,
            supports_editing: false,
            db_tables_have_guid: true,
            locale_file_name: None,
            banned_packedfiles: vec![],
            icon_small: "gs_are.png".to_owned(),
            icon_big: "gs_big_are.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinWargaming, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: false,
                    store_id: 0,
                    store_id_ak: 0,
                    executable: "Arena.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "".to_owned(),
                    config_folder: None,
                });

                data
            },
            tool_vars: HashMap::new(),
            lua_autogen_folder: None,
            ak_lost_fields: vec![],
        });

        let order_list = vec![
            KEY_WARHAMMER_3,
            KEY_TROY,
            KEY_THREE_KINGDOMS,
            KEY_WARHAMMER_2,
            KEY_WARHAMMER,
            KEY_THRONES_OF_BRITANNIA,
            KEY_ATTILA,
            KEY_ROME_2,
            KEY_SHOGUN_2,
            KEY_NAPOLEON,
            KEY_EMPIRE,
            KEY_ARENA,
        ];

        Self {
            games: game_list,
            order: order_list,
        }
    }
}

/// Implementation for `SupportedGames`.
impl SupportedGames {

    /// This function returns a GameInfo from a game name.
    pub fn game(&self, key: &str) -> Option<&GameInfo> {
        self.games.get(key)
    }

    /// This function returns a vec with references to the full list of supported games.
    pub fn games(&self) -> Vec<&GameInfo> {
        self.games.values().collect::<Vec<&GameInfo>>()
    }

    /// This function returns the list of Game Keys (Game name formatted for internal use) this crate supports.
    pub fn game_keys(&self) -> Vec<&str> {
        self.games.keys().cloned().collect::<Vec<&str>>()
    }

    /// This function returns a vec with references to the full list of supported games, sorted by release date.
    pub fn games_sorted(&self) -> Vec<&GameInfo> {
        self.order.iter().map(|key| self.game(key).unwrap()).collect::<Vec<&GameInfo>>()
    }

    /// This function returns the list of Game Keys (Game name formatted for internal use) this crate supports, sorted by release date.
    pub fn game_keys_sorted(&self) -> &[&'static str] {
        &self.order
    }
}
