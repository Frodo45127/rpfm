//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use std::sync::{Arc, RwLock};

use crate::compression::CompressionFormat;
use super::{GameInfo, InstallData, InstallType, pfh_file_type::PFHFileType, pfh_version::PFHVersion, VanillaDBTableNameLogic};

// Display Name for all the Supported Games.
pub const DISPLAY_NAME_PHARAOH_DYNASTIES: &str = "Pharaoh Dynasties";
pub const DISPLAY_NAME_PHARAOH: &str = "Pharaoh";
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
pub const KEY_PHARAOH_DYNASTIES: &str = "pharaoh_dynasties";        // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_PHARAOH: &str = "pharaoh";                            // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_WARHAMMER_3: &str = "warhammer_3";                    // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_TROY: &str = "troy";                                  // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_THREE_KINGDOMS: &str = "three_kingdoms";              // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_WARHAMMER_2: &str = "warhammer_2";                    // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_WARHAMMER: &str = "warhammer";                        // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_THRONES_OF_BRITANNIA: &str = "thrones_of_britannia";  // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_ATTILA: &str = "attila";                              // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_ROME_2: &str = "rome_2";                              // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_SHOGUN_2: &str = "shogun_2";                          // Filtered and revised incorrect AK fields. Startpos tables done.
pub const KEY_NAPOLEON: &str = "napoleon";                          // Pending of schema review for incorrect AK fields. Pending decoding starpos tables.
pub const KEY_EMPIRE: &str = "empire";                              // Pending of schema review for incorrect AK fields. Pending decoding starpos tables.
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

        // Pharaoh/Dynasties. This is really the same as pharaoh, but CA re-released as a separate game, so we treat it as a separate game too.
        game_list.insert(KEY_PHARAOH_DYNASTIES, GameInfo {
            key: KEY_PHARAOH_DYNASTIES,
            display_name: DISPLAY_NAME_PHARAOH_DYNASTIES,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_ph_dyn.ron".to_owned(),
            dependencies_cache_file_name: "ph_dyn.pak2".to_owned(),
            raw_db_version: 2,
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_ph_dyn.png".to_owned(),
            icon_big: "gs_big_ph_dyn.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 2_951_630,
                    store_id_ak: 2_951_670,
                    executable: "Pharaoh.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/2951630".to_owned(),
                    config_folder: Some("PharaohDynasties".to_owned()),
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
            lua_autogen_folder: None,
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
                "achievements/game_expansion_key".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "audio_entity_types/actor_type".to_owned(),
                "audio_entity_types/game_expansion_key".to_owned(),
                "audio_entity_types/switch".to_owned(),
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
                "building_chains/encyclopedia_name".to_owned(),
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
                "campaign_tree_types/game_expansion_key".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/game_expansion_key".to_owned(),
                "cdir_events_mission_option_junctions/game_expansion_key".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "government_types/elected_ministers".to_owned(),
                "government_types/hereditary_ministers".to_owned(),
                "government_types/rank".to_owned(),

                // This is a loc that's unused, so the bruteforce pass fails to mark it as a loc.
                "land_units/concealed_name".to_owned(),

                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "main_units/audio_voiceover_culture_override".to_owned(),

                // Special table. Ignore them.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),
                "models_building/tech_file".to_owned(),

                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),

                // Pretty sure this is a loc that's not exported in Dave, so the bruteforce pass fails to mark it as a loc.
                "pooled_resources/negative_display_name".to_owned(),

                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "scripted_bonus_value_ids/notes".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_faction_effect_bundles/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_family_relationships/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_technologies/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Lzma1
            ]
        });

        // Pharaoh
        game_list.insert(KEY_PHARAOH, GameInfo {
            key: KEY_PHARAOH,
            display_name: DISPLAY_NAME_PHARAOH,
            pfh_versions: {
                let mut data = HashMap::new();
                data.insert(PFHFileType::Boot, PFHVersion::PFH5);
                data.insert(PFHFileType::Release, PFHVersion::PFH5);
                data.insert(PFHFileType::Patch, PFHVersion::PFH5);
                data.insert(PFHFileType::Mod, PFHVersion::PFH5);
                data.insert(PFHFileType::Movie, PFHVersion::PFH5);
                data
            },
            schema_file_name: "schema_ph.ron".to_owned(),
            dependencies_cache_file_name: "ph.pak2".to_owned(),
            raw_db_version: 2,
            portrait_settings_version: None,
            supports_editing: true,
            db_tables_have_guid: true,
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_ph.png".to_owned(),
            icon_big: "gs_big_ph.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![],
                    use_manifest: true,
                    store_id: 1_937_780, // Normal Game.
                    //store_id: 2_254_160, // Early Access Weekend
                    store_id_ak: 1_937_790,
                    executable: "Pharaoh.exe".to_owned(),
                    data_path: "data".to_owned(),
                    language_path: "data".to_owned(),
                    local_mods_path: "data".to_owned(),
                    downloaded_mods_path: "./../../workshop/content/1937780".to_owned(), // Normal Game.
                    //downloaded_mods_path: "./../../workshop/content/2254160".to_owned(), // Early Access Weekend
                    config_folder: Some("Pharaoh".to_owned()),
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
            lua_autogen_folder: None,
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
                "achievements/game_expansion_key".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "audio_entity_types/actor_type".to_owned(),
                "audio_entity_types/game_expansion_key".to_owned(),
                "audio_entity_types/switch".to_owned(),
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
                "building_chains/encyclopedia_name".to_owned(),
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
                "campaign_tree_types/game_expansion_key".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/game_expansion_key".to_owned(),
                "cdir_events_mission_option_junctions/game_expansion_key".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "government_types/elected_ministers".to_owned(),
                "government_types/hereditary_ministers".to_owned(),
                "government_types/rank".to_owned(),

                // This is a loc that's unused, so the bruteforce pass fails to mark it as a loc.
                "land_units/concealed_name".to_owned(),

                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "main_units/audio_voiceover_culture_override".to_owned(),

                // Special table. Ignore them.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),
                "models_building/tech_file".to_owned(),

                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),

                // Pretty sure this is a loc that's not exported in Dave, so the bruteforce pass fails to mark it as a loc.
                "pooled_resources/negative_display_name".to_owned(),

                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "scripted_bonus_value_ids/notes".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_faction_effect_bundles/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Lzma1
            ]
        });

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
                "agent_recruitment_categories/onscreen_description".to_owned(),
                "alternative_pooled_resource_junctions/description".to_owned(),
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
                "pdlc/game_expansion_key".to_owned(),
                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "resources/campaign_group".to_owned(),
                "scripted_bonus_value_ids/notes".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_factions/honour".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_naval_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_settlement_garrisons/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_victory_conditions/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Zstd,
                CompressionFormat::Lz4,
                CompressionFormat::Lzma1
            ]
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
                "achievements/game_expansion_key".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "audio_entity_types/actor_type".to_owned(),
                "audio_entity_types/game_expansion_key".to_owned(),
                "audio_entity_types/switch".to_owned(),
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
                "cai_task_management_system_variables/description".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_map_playable_areas/game_expansion_key".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_playable_areas/preview_border".to_owned(),
                "campaign_payload_ui_details/comment".to_owned(),
                "campaign_tree_types/game_expansion_key".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/game_expansion_key".to_owned(),
                "cdir_events_mission_option_junctions/game_expansion_key".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "government_types/elected_ministers".to_owned(),
                "government_types/hereditary_ministers".to_owned(),
                "government_types/rank".to_owned(),
                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "main_units/audio_voiceover_culture".to_owned(),
                "main_units/audio_voiceover_culture_override".to_owned(),

                // Special table. Ignore them.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),
                "models_building/tech_file".to_owned(),

                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),
                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_faction_effect_bundles/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_family_relationships/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_naval_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_regions_to_unit_resources/unique".to_owned(),
                "start_pos_settlement_garrisons/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_technologies/unique".to_owned(),
                "start_pos_victory_conditions/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Lzma1
            ]
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
                "achievements/game_expansion_key".to_owned(),
                "advice_levels/locatable".to_owned(),
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
                "cai_task_management_system_variables/description".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_diplomacy_automatic_deal_situations/comments".to_owned(),
                "campaign_effect_scopes/source_for_design_ref_only".to_owned(),
                "campaign_map_playable_areas/game_expansion_key".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_playable_areas/preview_border".to_owned(),
                "campaign_payload_ui_details/comment".to_owned(),
                "campaign_settlement_display_building_ids/sub_culture".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/game_expansion_key".to_owned(),
                "ccp_balance_values/description".to_owned(),
                "cdir_events_options/notes".to_owned(),
                "cdir_events_post_generation_conditions/notes".to_owned(),
                "cdir_events_targets/description".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "ceo_initial_datas/template_manager".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "cultures/audio_state".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),
                "experience_triggers/target".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "hero_battle_conversation_strings/game_expansion_key".to_owned(),
                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "loading_screen_speech_strings/game_expansion_key".to_owned(),

                // Special table. Ignore it.
                "models_building/cs2_file".to_owned(),

                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),
                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_family_relationships/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_technologies/unique".to_owned(),
                "start_pos_victory_conditions/unique".to_owned(),
                "taxes_effects_jct/ai_only".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Lzma1
            ]
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
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_wh2.png".to_owned(),
            icon_big: "gs_big_wh2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::DefaultName("data__".to_owned()),
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![
                        "audio_base.pack".to_owned(),
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
                "achievements/game_expansion_key".to_owned(),
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
                "cdir_events_mission_option_junctions/game_expansion_key".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "government_types/elected_ministers".to_owned(),
                "government_types/hereditary_ministers".to_owned(),
                "government_types/rank".to_owned(),
                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "mercenary_pool_to_groups_junctions/game_expansion_key".to_owned(),

                // Special table. Ignore them.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),
                "models_building/tech_file".to_owned(),

                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),
                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "ritual_chains/description".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_faction_effect_bundles/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_family_relationships/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_naval_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_regions_to_unit_resources/unique".to_owned(),
                "start_pos_settlement_garrisons/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_technologies/unique".to_owned(),
                "start_pos_victory_conditions/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_sets/game_expansion_key".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "videos/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![
                CompressionFormat::Lzma1
            ]
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
            locale_file_name: Some("language.txt".to_owned()),
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
                "achievements/game_expansion_key".to_owned(),
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
                "cdir_events_mission_option_junctions/game_expansion_key".to_owned(),
                "cdir_military_generator_configs/game_expansion_key".to_owned(),
                "cdir_military_generator_templates/game_expansion_key".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),
                "character_skills/pre_battle_speech_parameter".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/icon_name".to_owned(),
                "diplomatic_relations_religion/relations_modifier".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),
                "experience_triggers/target".to_owned(),
                "faction_groups/ui_icon".to_owned(),
                "factions/game_expansion_key".to_owned(),
                "frontend_faction_leaders/game_expansion_key".to_owned(),
                "government_types/elected_ministers".to_owned(),
                "government_types/hereditary_ministers".to_owned(),
                "government_types/rank".to_owned(),
                "land_units/game_expansion_key".to_owned(),
                "loading_screen_quotes/game_expansion_key".to_owned(),
                "mercenary_pool_to_groups_junctions/game_expansion_key".to_owned(),
                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "names_groups/game_expansion_key".to_owned(),
                "pdlc/game_expansion_key".to_owned(),

                // Possible case of incorrect definition, but as it's not used by the game, we left it as is.
                "plagues/military_force_effects_bundle".to_owned(),
                "plagues/region_effect_bundle".to_owned(),

                "projectiles/game_expansion_key".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/is_sea".to_owned(),
                "scripted_objectives/game_expansion_key".to_owned(),
                "start_pos_calendars/unique".to_owned(),
                "start_pos_character_ancillaries/unique".to_owned(),
                "start_pos_character_to_settlements/unique".to_owned(),
                "start_pos_character_traits/unique".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_diplomacy/unique".to_owned(),
                "start_pos_factions/unique".to_owned(),
                "start_pos_land_units/unique".to_owned(),
                "start_pos_past_events/unique".to_owned(),
                "start_pos_region_religions/unique".to_owned(),
                "start_pos_region_slot_templates/unique".to_owned(),
                "start_pos_regions/unique".to_owned(),
                "start_pos_settlements/unique".to_owned(),
                "start_pos_technologies/unique".to_owned(),
                "start_pos_victory_conditions/unique".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "trait_info/applicable_to".to_owned(),
                "trigger_effects/game_expansion_key".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/game_expansion_key".to_owned(),
                "warscape_animated/game_expansion_key".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
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
            ak_lost_fields: vec![
                "_kv_experience_bonuses/description".to_owned(),
                "_kv_fatigue/description".to_owned(),
                "_kv_fire_values/description".to_owned(),
                "_kv_key_buildings/description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_naval_rules/description".to_owned(),
                "_kv_rules/description".to_owned(),

                // Possible loc incorrectly marked.
                "ancillaries/effect_text".to_owned(),

                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "battlefield_building_transformations/description".to_owned(),

                // This is due to multiple outdated bob tables in the vanilla packs. It's technically a bug.
                "battlefield_buildings/blood_pack_model_override_folder".to_owned(),

                "battles/objectives_team_1".to_owned(),
                "battles/objectives_team_2".to_owned(),
                "building_chains/encyclopedia_group".to_owned(),
                "building_chains/encyclopedia_include_in_index".to_owned(),
                "building_levels/building_category".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_payload_ui_details/comment".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/data_directory".to_owned(),

                // Possible loc incorrectly marked.
                "campaigns/encyclopedia_name_override".to_owned(),

                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),

                // Possible loc incorrectly marked.
                "character_trait_levels/effect_text".to_owned(),

                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/in_encyclopaedia".to_owned(),
                "encyclopedia_blocks/video".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),
                "experience_triggers/target".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "incident_heading_texts/localised_heading_text".to_owned(),

                // Special table, ignore it.
                "models_building/cs2_file".to_owned(),
                "models_building/logic_file".to_owned(),
                "models_building/model_file".to_owned(),

                // Special table, ignore it.
                "models_sieges/logic_file".to_owned(),

                // The table is entry, so no idea if it exists or not..
                "mount_variants/key".to_owned(),

                "multiplayer_mininum_length_funds/description".to_owned(),
                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/palette_entry".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "sound_events/name".to_owned(),

                "start_pos_diplomacy/relations_modifier".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_links/encyclopedia_child_link_position".to_owned(),
                "technology_node_links/encyclopedia_parent_link_position".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "unit_class/icon".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_att.png".to_owned(),
            icon_big: "gs_big_att.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![
                        "belisarius.pack".to_owned(),
                        "blood.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "charlemagne.pack".to_owned(),
                        "data.pack".to_owned(),

                        "local_en_shared_rome2.pack".to_owned(),    // English, but present for a few more languages. Loads before language-specific packs.

                        "local_cz.pack".to_owned(),                 // Czech
                        "local_en.pack".to_owned(),                 // English
                        "local_fr.pack".to_owned(),                 // French
                        "local_ge.pack".to_owned(),                 // German
                        "local_it.pack".to_owned(),                 // Italian
                        "local_pl.pack".to_owned(),                 // Polish
                        "local_ru.pack".to_owned(),                 // Russian
                        "local_tr.pack".to_owned(),                 // Turkish
                        "local_sp.pack".to_owned(),                 // Spanish

                        "models.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "models3.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "music.pack".to_owned(),

                        "music_en_shared_rome2.pack".to_owned(),    // English, but present for a few more languages. Loads before language-specific packs.

                        "music_fr.pack".to_owned(),                 // French
                        "music_ge.pack".to_owned(),                 // German
                        "music_it.pack".to_owned(),                 // Italian
                        "music_ru.pack".to_owned(),                 // Russian
                        "music_sp.pack".to_owned(),                 // Spanish

                        "slavs.pack".to_owned(),
                        "sound.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "tiles.pack".to_owned(),
                        "tiles2.pack".to_owned(),
                        "tiles3.pack".to_owned(),
                        "tiles4.pack".to_owned(),
                    ],
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
            ak_lost_fields: vec![
                "_kv_experience_bonuses/description".to_owned(),
                "_kv_fatigue/description".to_owned(),
                "_kv_fire_values/description".to_owned(),
                "_kv_key_buildings/description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_naval_rules/description".to_owned(),
                "_kv_rules/description".to_owned(),

                // Possible loc incorrectly marked.
                "ancillaries/effect_text".to_owned(),

                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "battlefield_building_transformations/description".to_owned(),

                // This is due to multiple outdated bob tables in the vanilla packs. It's technically a bug.
                "battlefield_buildings/blood_pack_model_override_folder".to_owned(),

                "battles/objectives_team_1".to_owned(),
                "battles/objectives_team_2".to_owned(),
                "building_chains/encyclopedia_group".to_owned(),
                "building_chains/encyclopedia_include_in_index".to_owned(),
                "building_levels/building_category".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_payload_ui_details/comment".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/data_directory".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),

                // Possible loc incorrectly marked.
                "character_trait_levels/effect_text".to_owned(),

                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/in_encyclopaedia".to_owned(),
                "encyclopedia_blocks/video".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),
                "experience_triggers/target".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "incident_heading_texts/localised_heading_text".to_owned(),

                // Special table, ignore it.
                "models_building/cs2_file".to_owned(),
                "models_building/model_file".to_owned(),

                // Special table, ignore it.
                "models_sieges/display_path".to_owned(),
                "models_sieges/logic_file".to_owned(),
                "models_sieges/model_file".to_owned(),

                "mount_variants/key".to_owned(),
                "multiplayer_mininum_length_funds/description".to_owned(),
                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/palette_entry".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "sound_events/name".to_owned(),

                "start_pos_diplomacy/relations_modifier".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_links/encyclopedia_child_link_position".to_owned(),
                "technology_node_links/encyclopedia_parent_link_position".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "unit_class/icon".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
            banned_packedfiles: vec![],
            icon_small: "gs_rom2.png".to_owned(),
            icon_big: "gs_big_rom2.png".to_owned(),
            vanilla_db_table_name_logic: VanillaDBTableNameLogic::FolderName,
            install_data: {
                let mut data = HashMap::new();
                data.insert(InstallType::WinSteam, InstallData {
                    vanilla_packs: vec![
                        "blood_rome2.pack".to_owned(),
                        "boot.pack".to_owned(),
                        "data.pack".to_owned(),
                        "data_rome2.pack".to_owned(),
                        "divided.pack".to_owned(),
                        "gaul.pack".to_owned(),
                        "greeks.pack".to_owned(),
                        "invasion.pack".to_owned(),

                        "local_en_shared_rome2.pack".to_owned(),    // English, but present for a few more languages. Loads before language-specific packs.

                        "local_cz.pack".to_owned(),                 // Czech
                        "local_en.pack".to_owned(),                 // English
                        "local_fr.pack".to_owned(),                 // French
                        "local_ge.pack".to_owned(),                 // German
                        "local_it.pack".to_owned(),                 // Italian
                        "local_pl.pack".to_owned(),                 // Polish
                        "local_ru.pack".to_owned(),                 // Russian
                        "local_tr.pack".to_owned(),                 // Turkish
                        "local_sp.pack".to_owned(),                 // Spanish

                        "models.pack".to_owned(),
                        "models_rome2.pack".to_owned(),
                        "models2.pack".to_owned(),
                        "models2_rome2.pack".to_owned(),
                        "models3_rome2.pack".to_owned(),
                        "movies.pack".to_owned(),
                        "movies_rome2.pack".to_owned(),
                        "music.pack".to_owned(),
                        "music_rome2.pack".to_owned(),

                        "music_en_shared_rome2.pack".to_owned(),    // English, but present for a few more languages. Loads before language-specific packs.

                        "music_fr.pack".to_owned(),                 // French
                        "music_ge.pack".to_owned(),                 // German
                        "music_it.pack".to_owned(),                 // Italian
                        "music_ru.pack".to_owned(),                 // Russian
                        "music_sp.pack".to_owned(),                 // Spanish

                        "punic.pack".to_owned(),
                        "sound.pack".to_owned(),
                        "sound_rome2.pack".to_owned(),
                        "terrain.pack".to_owned(),
                        "terrain_rome2.pack".to_owned(),
                        "terrain2.pack".to_owned(),
                        "terrain2_rome2.pack".to_owned(),
                        "terrain3_rome2.pack".to_owned(),
                        "tiles.pack".to_owned(),
                        "tiles_rome2.pack".to_owned(),
                        "tiles2.pack".to_owned(),
                        "tiles2_rome2.pack".to_owned(),
                        "tiles3.pack".to_owned(),
                        "tiles3_rome2.pack".to_owned(),
                        "tiles4.pack".to_owned(),
                        "tiles4_rome2.pack".to_owned()
                    ],
                    use_manifest: false,
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
            ak_lost_fields: vec![
                "_kv_experience_bonuses/description".to_owned(),
                "_kv_fatigue/description".to_owned(),
                "_kv_key_buildings/description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_rules/description".to_owned(),

                // Possible loc incorrectly marked.
                "ancillaries/effect_text".to_owned(),

                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),

                // Weird one. The game uses an older version than the one in the AK,
                // which is missing this column. The newly exported one contains it.
                "banditry_events/duration".to_owned(),

                "battlefield_building_transformations/description".to_owned(),
                "battles/objectives_team_1".to_owned(),
                "battles/objectives_team_2".to_owned(),
                "building_chains/encyclopedia_group".to_owned(),
                "building_chains/encyclopedia_include_in_index".to_owned(),
                "building_levels/building_category".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_ai_personalities/description".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/data_directory".to_owned(),
                "character_skill_level_to_effects_junctions/is_factionwide".to_owned(),

                // Possible loc incorrectly marked.
                "character_trait_levels/effect_text".to_owned(),

                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "deployables/in_encyclopaedia".to_owned(),
                "encyclopedia_blocks/video".to_owned(),
                "event_log_descriptions/notes".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),
                "experience_triggers/target".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "incident_heading_texts/localised_heading_text".to_owned(),

                "mount_variants/key".to_owned(),
                "multiplayer_mininum_length_funds/description".to_owned(),
                "names/nobility".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/palette_entry".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "sound_events/name".to_owned(),

                "start_pos_diplomacy/relations_modifier".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "technology_node_links/encyclopedia_child_link_position".to_owned(),
                "technology_node_links/encyclopedia_parent_link_position".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
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
            ak_lost_fields: vec![
                "_kv_experience_bonuses/description".to_owned(),
                "_kv_fatigue/description".to_owned(),
                "_kv_key_buildings/description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_rules/description".to_owned(),
                "_kv_special_ability_effects/description".to_owned(),
                "agents/animation_set".to_owned(),
                "agents/associated_unit".to_owned(),
                "agents/in_encyclopedia".to_owned(),

                // Possible loc incorrectly marked.
                "ancillaries/effect_text".to_owned(),

                "ancillary_info/author".to_owned(),
                "ancillary_info/comment".to_owned(),
                "ancillary_info/historical_example".to_owned(),

                "avatar_dojos/codependency_key".to_owned(),
                "avatar_gempei_dojos/codependency_key".to_owned(),
                "avatar_ranks/avatar_naval_unit_cost".to_owned(),
                "avatar_unit_group_ids/description".to_owned(),
                "avatar_units/anti_spam_cost_increase".to_owned(),
                "avatar_units/anti_spam_limit".to_owned(),
                "battle_script_strings/game_area".to_owned(),
                "battlefield_building_transformations/description".to_owned(),

                // Special case. These exist, but due to the game having fragments of this table
                // with old version that don't have these columns, they get reported as missing.
                "battlefield_buildings/fortwall_penetration_chance".to_owned(),
                "battlefield_buildings/radar_icon".to_owned(),
                "battlefield_buildings/spawned_unit".to_owned(),
                "battlefield_buildings/visible_in_public_ted".to_owned(),

                "battles/objectives_team_1".to_owned(),
                "battles/objectives_team_2".to_owned(),
                "building_levels/building_category".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_ai_personalities/description".to_owned(),
                "campaign_map_playable_areas/mapname".to_owned(),
                "campaign_map_playable_areas/maxx".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/minx".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_towns_and_ports/region".to_owned(),
                "campaign_variables/description".to_owned(),
                "campaigns/data_directory".to_owned(),
                "character_trait_levels/effect_text".to_owned(),
                "character_traits/author".to_owned(),
                "character_traits/comment".to_owned(),
                "event_log_descriptions/notes".to_owned(),
                "experience_triggers/condition".to_owned(),
                "experience_triggers/event".to_owned(),

                // Possible loc, doesn't exist in the ak.
                "incident_heading_texts/localised_heading_text".to_owned(),

                "mount_variants/key".to_owned(),
                "multiplayer_mininum_length_funds/description".to_owned(),
                "projectiles/bounce_angle".to_owned(),
                "projectiles/preflight_rules".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "regions/in_encyclopedia".to_owned(),
                "regions/palette_entry".to_owned(),
                "slots_art/underlay_rotation".to_owned(),
                "slots_art/underlay_scale".to_owned(),
                "start_pos_diplomacy/relations_modifier".to_owned(),
                "start_pos_forts/fort_name".to_owned(),
                "start_pos_regions/encyclopedia_order".to_owned(),
                "technologies/in_encyclopedia".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "uniforms/IconScreenshotCameraPreset".to_owned(),
                "uniforms/InfoScreenshotCameraPreset".to_owned(),
                "uniforms/ManAnimation".to_owned(),
                "uniforms/MountAnimation".to_owned(),
                "unit_stats_land/desert_effect".to_owned(),
                "unit_stats_land/snow_effect".to_owned(),
                "unit_stats_land/tropics_effect".to_owned(),
                "unit_stats_land/unit_class".to_owned(),
                "unit_stats_naval/repair_cost_port".to_owned(),
                "unit_stats_naval/repair_cost_sea".to_owned(),
                "unit_stats_naval/ship_rating_icon".to_owned(),
                "units/era".to_owned(),
                "units/fitness".to_owned(),
                "units/pdlc".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
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
            ak_lost_fields: vec![
                "_kv_fatigue/Gen_description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_rules/Gen_description".to_owned(),
                "abilities/Gen_effect_text".to_owned(),
                "abilities/is_active".to_owned(),
                "advice_levels/Gen_TempField_x002A_1".to_owned(),
                "advice_levels/Gen_TempField_x002A_2".to_owned(),
                "advice_levels/Gen_onscreen_text".to_owned(),
                "agent_attribute_situations/Gen_effect_text".to_owned(),
                "agent_attributes/Gen_effect_text".to_owned(),
                "agents/associated_unit".to_owned(),
                "ancillaries/Gen_colour_text".to_owned(),
                "ancillaries/Gen_effect_text".to_owned(),
                "ancillaries/Gen_exclusion_text".to_owned(),
                "ancillaries/Gen_explanation_text".to_owned(),
                "ancillary_info/Gen_comment".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "anim_reference_poses/Gen_path".to_owned(),
                "battle_personalities/equipment_theme".to_owned(),
                "battle_script_strings/game_area".to_owned(),
                "battle_weather_types/naval_appropriate".to_owned(),
                "battlefield_building_transformations/description".to_owned(),
                "battlefield_buildings/onscreen_name".to_owned(),
                "battlefield_deployable_siege_items/Gen_string".to_owned(),
                "battles/Gen_description".to_owned(),
                "battles/Gen_objectives_team_1".to_owned(),
                "battles/Gen_objectives_team_2".to_owned(),
                "bribe_actions/action".to_owned(),
                "bribe_actions/onscreen".to_owned(),
                "building_chains/Gen_chain_tooltip".to_owned(),
                "building_description_texts/Gen_TempField_x002A_0".to_owned(),
                "building_description_texts/Gen_long_description".to_owned(),
                "building_levels/Gen_condition".to_owned(),
                "building_levels/building_category".to_owned(),
                "building_units_allowed/Gen_conditions".to_owned(),
                "building_units_allowed/XP".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_ai_personalities/description".to_owned(),
                "campaign_anim_transitions/Gen_path".to_owned(),
                "campaign_anim_transitions/ID".to_owned(),
                "campaign_anims/Gen_path".to_owned(),
                "campaign_map_playable_areas/Gen_TempField_x002A_0".to_owned(),
                "campaign_map_playable_areas/mapname".to_owned(),
                "campaign_map_playable_areas/maxx".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/minx".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_settlements/template_name".to_owned(),
                "campaign_map_slots/template".to_owned(),
                "campaign_map_tooltips/Gen_TempField_x002A_0".to_owned(),
                "campaign_map_tooltips/Gen_TempField_x002A_1".to_owned(),
                "campaign_map_towns_and_ports/region".to_owned(),
                "campaign_map_towns_and_ports/template".to_owned(),
                "campaign_variables/Gen_description".to_owned(),
                "character_trait_levels/Gen_colour_text".to_owned(),
                "character_trait_levels/Gen_effect_text".to_owned(),
                "character_trait_levels/Gen_epithet_text".to_owned(),
                "character_trait_levels/Gen_explanation_text".to_owned(),
                "character_trait_levels/Gen_gain_text".to_owned(),
                "character_trait_levels/Gen_removal_text".to_owned(),
                "character_traits/Gen_comment".to_owned(),
                "character_traits/author".to_owned(),
                "commodities/price_elasticity_of_demand".to_owned(),
                "cursors/hotspotX".to_owned(),
                "cursors/hotspotY".to_owned(),
                "diplomacy_strings/Gen_TempField_x002A_1".to_owned(),
                "diplomatic_relations_religion/religion_A".to_owned(),
                "diplomatic_relations_religion/religion_B".to_owned(),
                "effect_bonus_value_projectile_junctions/bonus_value_id".to_owned(),
                "effects/Gen_description".to_owned(),
                "events/Gen_conditions".to_owned(),
                "events/Gen_event_text".to_owned(),
                "faction_groups/Afghanistan".to_owned(),
                "faction_groups/AfricanNatives".to_owned(),
                "faction_groups/AmerindIroquoisTribes".to_owned(),
                "faction_groups/AmerindTribesIII".to_owned(),
                "faction_groups/AmerindWoodlandTribes".to_owned(),
                "faction_groups/Austria".to_owned(),
                "faction_groups/BarbaryPirates".to_owned(),
                "faction_groups/Baroda".to_owned(),
                "faction_groups/Bavaria".to_owned(),
                "faction_groups/CrimeanKhanate".to_owned(),
                "faction_groups/Denmark".to_owned(),
                "faction_groups/EuropeanRebels".to_owned(),
                "faction_groups/France".to_owned(),
                "faction_groups/Genoa".to_owned(),
                "faction_groups/GreatBritain".to_owned(),
                "faction_groups/Greece".to_owned(),
                "faction_groups/Gwalior".to_owned(),
                "faction_groups/Haiti".to_owned(),
                "faction_groups/HanoverHesse".to_owned(),
                "faction_groups/Holland".to_owned(),
                "faction_groups/IndianRebels".to_owned(),
                "faction_groups/Indore".to_owned(),
                "faction_groups/IslamicRebels".to_owned(),
                "faction_groups/Malta".to_owned(),
                "faction_groups/Malwa".to_owned(),
                "faction_groups/Mamelukes".to_owned(),
                "faction_groups/MarathaConfederacy".to_owned(),
                "faction_groups/Modena".to_owned(),
                "faction_groups/Morocco".to_owned(),
                "faction_groups/MughalEmpire".to_owned(),
                "faction_groups/Mysore".to_owned(),
                "faction_groups/OttomanEmpire".to_owned(),
                "faction_groups/PapalStates".to_owned(),
                "faction_groups/Parma".to_owned(),
                "faction_groups/Pirates".to_owned(),
                "faction_groups/Poland".to_owned(),
                "faction_groups/Pomerania".to_owned(),
                "faction_groups/Portugal".to_owned(),
                "faction_groups/Prussia".to_owned(),
                "faction_groups/Punjab".to_owned(),
                "faction_groups/Russia".to_owned(),
                "faction_groups/SafavidEmpire".to_owned(),
                "faction_groups/Savoy".to_owned(),
                "faction_groups/Saxony".to_owned(),
                "faction_groups/Silesia".to_owned(),
                "faction_groups/SlaveRebels".to_owned(),
                "faction_groups/Spain".to_owned(),
                "faction_groups/Sweden".to_owned(),
                "faction_groups/Switzerland".to_owned(),
                "faction_groups/Tatars".to_owned(),
                "faction_groups/Tuscany".to_owned(),
                "faction_groups/USA".to_owned(),
                "faction_groups/Ujjain".to_owned(),
                "faction_groups/Venice".to_owned(),
                "faction_groups/Westphalia".to_owned(),
                "faction_groups/Wurttemberg".to_owned(),
                "factions/icons_path_units".to_owned(),
                "historical_characters/Gen_spawn_conditions".to_owned(),
                "ministerial_positions_by_gov_types/onscreen_name".to_owned(),
                "mission_activities/check_event".to_owned(),
                "mission_activities/evaluate_event".to_owned(),
                "mission_effects/Gen_text".to_owned(),
                "missions/Gen_TempField_x002A_0".to_owned(),
                "missions/Gen_cancel_condition".to_owned(),
                "missions/Gen_failure_condition".to_owned(),
                "missions/Gen_success_condition".to_owned(),
                "missions/cancellation_effect".to_owned(),
                "missions/failure_effect".to_owned(),
                "missions/success_effect".to_owned(),
                "mount_variants/key".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "pdlc/ID".to_owned(),
                "pdlc/SteamID".to_owned(),
                "policies/Gen_prerequisites".to_owned(),
                "projectile_impacts/buildings".to_owned(),
                "projectile_shot_type_enum/is_artillery".to_owned(),
                "projectile_shot_type_enum/is_smallarms".to_owned(),
                "projectile_trails/min_apparent_width_distance".to_owned(),
                "projectiles/below_waterline_damage_modifer".to_owned(),
                "projectiles/bounce_angle".to_owned(),
                "projectiles/can_bounce".to_owned(),
                "projectiles/preflight_rules".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "public_order_factors/Gen_TempField_x002A_0".to_owned(),
                "public_order_factors/Gen_TempField_x002A_1".to_owned(),
                "quotes/Gen_TempField_x002A_0".to_owned(),
                "quotes_people/Gen_TempField_x002A_0".to_owned(),
                "random_localisation_strings/Gen_string".to_owned(),
                "region_economics_factors/Gen_TempField_x002A_0".to_owned(),
                "regions/palette_entry".to_owned(),
                "sea_climate_details/sea_deep_colour".to_owned(),
                "sea_climate_details/sea_shallow_colour".to_owned(),
                "sea_climate_details/sky_colour".to_owned(),
                "sea_climate_details/sun_colour".to_owned(),
                "slots_art/minibuildings_differ_at_quality".to_owned(),
                "slots_art/underlay_differs_with_building".to_owned(),
                "slots_art/underlay_rotation".to_owned(),
                "slots_art/underlay_scale".to_owned(),
                "technologies/Gen_TempField_x002A_0".to_owned(),
                "technologies/Gen_TempField_x002A_1".to_owned(),
                "town_wealth_growth_factors/Gen_TempField_x002A_0".to_owned(),
                "town_wealth_growth_factors/Gen_TempField_x002A_1".to_owned(),
                "trade_details/Gen_TempField_x002A_0".to_owned(),
                "trade_nodes/ID".to_owned(),
                "trait_triggers/Gen_TempField_x002A_0".to_owned(),
                "trees/is_conifer".to_owned(),
                "trees/is_high_altitude".to_owned(),
                "trees/is_shrub".to_owned(),
                "trees/tree".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "uniforms/Faction".to_owned(),
                "uniforms/Filename".to_owned(),
                "uniforms/IconScreenshotCameraPreset".to_owned(),
                "uniforms/InfoScreenshotCameraPreset".to_owned(),
                "uniforms/ManAnimation".to_owned(),
                "uniforms/MountAnimation".to_owned(),
                "uniforms/Uniform_Name".to_owned(),
                "uniforms/Unit".to_owned(),
                "unit_class/Gen_TempField_x002A_0".to_owned(),
                "unit_regiment_names/unit_class".to_owned(),
                "unit_stats_land/desert_effect".to_owned(),
                "unit_stats_land/dismounted_formation_spacing_horizontal".to_owned(),
                "unit_stats_land/dismounted_formation_spacing_vertical".to_owned(),
                "unit_stats_land/fatigue_resistant".to_owned(),
                "unit_stats_land/is_immune_to_attrition".to_owned(),
                "unit_stats_land/melee_defence".to_owned(),
                "unit_stats_land/snow_effect".to_owned(),
                "unit_stats_land/tropics_effect".to_owned(),
                "unit_stats_land/unit_class".to_owned(),
                "unit_stats_land_experience_bonuses/melee_defence".to_owned(),
                "unit_stats_naval/collision_momentum_modifer".to_owned(),
                "unit_stats_naval/reactivate_cost".to_owned(),
                "unit_stats_naval/repair_cost_port".to_owned(),
                "unit_stats_naval/repair_cost_sea".to_owned(),
                "unit_stats_naval/ship_rating_icon".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_armour".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_critical".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_hits".to_owned(),
                "unit_stats_naval/stat_bar_manoeuvrability_rating".to_owned(),
                "unit_stats_naval_crew_to_factions/gunner_type".to_owned(),
                "unit_stats_naval_crew_to_factions/key".to_owned(),
                "unit_stats_naval_crew_to_factions/marine_type".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_1".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_2".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_3".to_owned(),
                "unit_stats_naval_crew_to_factions/seaman_type".to_owned(),
                "units/era".to_owned(),
                "units/fitness".to_owned(),
                "unrest_cause_to_demands/demand".to_owned(),
                "warscape_rigid/category".to_owned(),
                "warscape_rigid_lod_range/LOD_id".to_owned(),
                "warscape_trees/model".to_owned(),
                "warscape_underlay_textures/filepath".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
                "wind_levels/magnitudeX".to_owned(),
                "wind_levels/magnitudeY".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
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
                        "movies.pack".to_owned(),
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
                        "movies.pack".to_owned(),
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
            ak_lost_fields: vec![
                "_kv_fatigue/Gen_description".to_owned(),
                "_kv_morale/description".to_owned(),
                "_kv_naval_morale/description".to_owned(),
                "_kv_rules/Gen_description".to_owned(),
                "abilities/Gen_effect_text".to_owned(),
                "abilities/is_active".to_owned(),
                "abilities/project_specific".to_owned(),
                "advice_levels/Gen_TempField_x002A_1".to_owned(),
                "advice_levels/Gen_TempField_x002A_2".to_owned(),
                "advice_levels/Gen_onscreen_text".to_owned(),
                "agent_attribute_situations/Gen_effect_text".to_owned(),
                "agent_attribute_situations/project_specific".to_owned(),
                "agent_attributes/Gen_effect_text".to_owned(),
                "agent_attributes/project_specific".to_owned(),
                "agent_culture_details/project_specific".to_owned(),
                "agent_spawning_to_building_chains/project_specific".to_owned(),
                "agent_spawning_to_government_types/project_specific".to_owned(),
                "agent_spawnings/project_specific".to_owned(),
                "agent_to_agent_abilities/project_specific".to_owned(),
                "agent_to_agent_attributes/project_specific".to_owned(),
                "agent_to_bribe_actions/project_specific".to_owned(),
                "agents/associated_unit".to_owned(),
                "agents/project_specific".to_owned(),
                "ancillaries/Gen_colour_text".to_owned(),
                "ancillaries/Gen_effect_text".to_owned(),
                "ancillaries/Gen_exclusion_text".to_owned(),
                "ancillaries/Gen_explanation_text".to_owned(),
                "ancillary_info/Gen_comment".to_owned(),
                "ancillary_info/author".to_owned(),
                "ancillary_info/historical_example".to_owned(),
                "anim_reference_poses/Gen_path".to_owned(),
                "battle_bridge_subculture_jcts/project_specific".to_owned(),
                "battle_city_subculture_jct/project_specific".to_owned(),
                "battle_personalities/equipment_theme".to_owned(),
                "battle_script_strings/game_area".to_owned(),
                "battle_terrain_farms/project_specific".to_owned(),
                "battle_type_faction_presets/project_specific".to_owned(),
                "battle_weather_types/naval_appropriate".to_owned(),
                "battlefield_building_transformations/description".to_owned(),
                "battlefield_buildings/onscreen_name".to_owned(),
                "battlefield_deployable_siege_items/Gen_string".to_owned(),
                "battles/Gen_description".to_owned(),
                "battles/Gen_objectives_team_1".to_owned(),
                "battles/Gen_objectives_team_2".to_owned(),
                "bribe_actions/action".to_owned(),
                "bribe_actions/onscreen".to_owned(),
                "building_chains/Gen_chain_tooltip".to_owned(),
                "building_description_texts/Gen_TempField_x002A_0".to_owned(),
                "building_description_texts/Gen_long_description".to_owned(),
                "building_faction_variants/project_specific".to_owned(),
                "building_level_required_technology_junctions/project_specific".to_owned(),
                "building_levels/Gen_condition".to_owned(),
                "building_levels/building_category".to_owned(),
                "building_research_thread_junction/research_points_per_turn".to_owned(),
                "building_units_allowed/Gen_conditions".to_owned(),
                "building_units_allowed/XP".to_owned(),
                "campaign_ai_manager_behaviour_junctions/project_specific".to_owned(),
                "campaign_ai_managers/description".to_owned(),
                "campaign_ai_managers/project_specific".to_owned(),
                "campaign_ai_personalities/description".to_owned(),
                "campaign_ai_personalities/project_specific".to_owned(),
                "campaign_ai_personality_junctions/project_specific".to_owned(),
                "campaign_anim_action_to_sets/project_specific".to_owned(),
                "campaign_anim_sets/project_specific".to_owned(),
                "campaign_anim_transitions/Gen_path".to_owned(),
                "campaign_anim_transitions/ID".to_owned(),
                "campaign_anim_transitions/project_specific".to_owned(),
                "campaign_anims/Gen_path".to_owned(),
                "campaign_anims/project_specific".to_owned(),
                "campaign_character_anim_set_agent_junctions/project_specific".to_owned(),
                "campaign_character_anim_sets/project_specific".to_owned(),
                "campaign_character_anim_walk_anim_junctions/project_specific".to_owned(),
                "campaign_character_anims_junctions/project_specific".to_owned(),
                "campaign_map_playable_areas/Gen_TempField_x002A_0".to_owned(),
                "campaign_map_playable_areas/mapname".to_owned(),
                "campaign_map_playable_areas/maxx".to_owned(),
                "campaign_map_playable_areas/maxy".to_owned(),
                "campaign_map_playable_areas/minx".to_owned(),
                "campaign_map_playable_areas/miny".to_owned(),
                "campaign_map_playable_areas/project_specific".to_owned(),
                "campaign_map_settlements/project_specific".to_owned(),
                "campaign_map_settlements/template_name".to_owned(),
                "campaign_map_slots/project_specific".to_owned(),
                "campaign_map_slots/template".to_owned(),
                "campaign_map_tooltips/Gen_TempField_x002A_0".to_owned(),
                "campaign_map_tooltips/Gen_TempField_x002A_1".to_owned(),
                "campaign_map_tooltips/project_specific".to_owned(),
                "campaign_map_towns_and_ports/project_specific".to_owned(),
                "campaign_map_towns_and_ports/region".to_owned(),
                "campaign_variables/Gen_description".to_owned(),
                "character_trait_levels/Gen_colour_text".to_owned(),
                "character_trait_levels/Gen_effect_text".to_owned(),
                "character_trait_levels/Gen_epithet_text".to_owned(),
                "character_trait_levels/Gen_explanation_text".to_owned(),
                "character_trait_levels/Gen_gain_text".to_owned(),
                "character_trait_levels/Gen_removal_text".to_owned(),
                "character_traits/Gen_comment".to_owned(),
                "character_traits/author".to_owned(),
                "climates/project_specific".to_owned(),
                "commodities/price_elasticity_of_demand".to_owned(),
                "cultures/project_specific".to_owned(),
                "cultures_subcultures/project_specific".to_owned(),
                "cursors/hotspotX".to_owned(),
                "cursors/hotspotY".to_owned(),
                "diplomacy_factor_strings/project_specific".to_owned(),
                "diplomacy_negotiation_faction_override_strings/project_specific".to_owned(),
                "diplomacy_negotiation_strings/project_specific".to_owned(),
                "diplomacy_strings/Gen_TempField_x002A_1".to_owned(),
                "diplomatic_relations_religion/religion_A".to_owned(),
                "diplomatic_relations_religion/religion_B".to_owned(),
                "effect_bonus_value_basic_junction/project_specific".to_owned(),
                "effect_bonus_value_commodity_junction/project_specific".to_owned(),
                "effect_bonus_value_population_class_and_religion_junction/project_specific".to_owned(),
                "effect_bonus_value_population_class_junction/project_specific".to_owned(),
                "effect_bonus_value_projectile_junctions/bonus_value_id".to_owned(),
                "effect_bonus_value_projectile_junctions/project_specific".to_owned(),
                "effect_bonus_value_religion_junction/project_specific".to_owned(),
                "effect_bonus_value_resource_junction/project_specific".to_owned(),
                "effect_bonus_value_shot_type_junctions/project_specific".to_owned(),
                "effect_bonus_value_unit_ability_junctions/project_specific".to_owned(),
                "effect_bonus_value_unit_category_junction/project_specific".to_owned(),
                "effect_bonus_value_unit_class_junction/project_specific".to_owned(),
                "effects/Gen_description".to_owned(),
                "events/Gen_conditions".to_owned(),
                "events/Gen_event_text".to_owned(),
                "faction_groups/Afghanistan".to_owned(),
                "faction_groups/AfricanNatives".to_owned(),
                "faction_groups/AmerindIroquoisTribes".to_owned(),
                "faction_groups/AmerindTribesIII".to_owned(),
                "faction_groups/AmerindWoodlandTribes".to_owned(),
                "faction_groups/Austria".to_owned(),
                "faction_groups/BarbaryPirates".to_owned(),
                "faction_groups/Baroda".to_owned(),
                "faction_groups/Bavaria".to_owned(),
                "faction_groups/CrimeanKhanate".to_owned(),
                "faction_groups/Denmark".to_owned(),
                "faction_groups/EuropeanRebels".to_owned(),
                "faction_groups/France".to_owned(),
                "faction_groups/Genoa".to_owned(),
                "faction_groups/GreatBritain".to_owned(),
                "faction_groups/Greece".to_owned(),
                "faction_groups/Gwalior".to_owned(),
                "faction_groups/Haiti".to_owned(),
                "faction_groups/HanoverHesse".to_owned(),
                "faction_groups/Holland".to_owned(),
                "faction_groups/IndianRebels".to_owned(),
                "faction_groups/Indore".to_owned(),
                "faction_groups/IslamicRebels".to_owned(),
                "faction_groups/Malta".to_owned(),
                "faction_groups/Malwa".to_owned(),
                "faction_groups/Mamelukes".to_owned(),
                "faction_groups/MarathaConfederacy".to_owned(),
                "faction_groups/Modena".to_owned(),
                "faction_groups/Morocco".to_owned(),
                "faction_groups/MughalEmpire".to_owned(),
                "faction_groups/Mysore".to_owned(),
                "faction_groups/OttomanEmpire".to_owned(),
                "faction_groups/PapalStates".to_owned(),
                "faction_groups/Parma".to_owned(),
                "faction_groups/Pirates".to_owned(),
                "faction_groups/Poland".to_owned(),
                "faction_groups/Pomerania".to_owned(),
                "faction_groups/Portugal".to_owned(),
                "faction_groups/Prussia".to_owned(),
                "faction_groups/Punjab".to_owned(),
                "faction_groups/Russia".to_owned(),
                "faction_groups/SafavidEmpire".to_owned(),
                "faction_groups/Savoy".to_owned(),
                "faction_groups/Saxony".to_owned(),
                "faction_groups/Silesia".to_owned(),
                "faction_groups/SlaveRebels".to_owned(),
                "faction_groups/Spain".to_owned(),
                "faction_groups/Sweden".to_owned(),
                "faction_groups/Switzerland".to_owned(),
                "faction_groups/Tatars".to_owned(),
                "faction_groups/Tuscany".to_owned(),
                "faction_groups/USA".to_owned(),
                "faction_groups/Ujjain".to_owned(),
                "faction_groups/Venice".to_owned(),
                "faction_groups/Westphalia".to_owned(),
                "faction_groups/Wurttemberg".to_owned(),
                "factions/icons_path_units".to_owned(),
                "groupings_military/project_specific".to_owned(),
                "historical_characters/Gen_spawn_conditions".to_owned(),
                "loading_screens/project_specific".to_owned(),
                "ministerial_positions_by_gov_types/onscreen_name".to_owned(),
                "mission_activities/check_event".to_owned(),
                "mission_activities/evaluate_event".to_owned(),
                "mission_effects/Gen_text".to_owned(),
                "missions/Gen_TempField_x002A_0".to_owned(),
                "missions/Gen_cancel_condition".to_owned(),
                "missions/Gen_failure_condition".to_owned(),
                "missions/Gen_success_condition".to_owned(),
                "missions/cancellation_effect".to_owned(),
                "missions/failure_effect".to_owned(),
                "mount_variants/key".to_owned(),
                "mount_variants/project_specific".to_owned(),
                "mounts/project_specific".to_owned(),
                "names/project_specific".to_owned(),
                "names_groups/Description".to_owned(),
                "names_groups/ID".to_owned(),
                "pdlc/ID".to_owned(),
                "pdlc/SteamID".to_owned(),
                "pdlc/project_specific".to_owned(),
                "policies/Gen_prerequisites".to_owned(),
                "projectile_impacts/buildings".to_owned(),
                "projectile_shot_type_enum/is_artillery".to_owned(),
                "projectile_shot_type_enum/is_smallarms".to_owned(),
                "projectile_trails/min_apparent_width_distance".to_owned(),
                "projectiles/below_waterline_damage_modifer".to_owned(),
                "projectiles/bounce_angle".to_owned(),
                "projectiles/preflight_rules".to_owned(),
                "projectiles_explosions/non_lethal_detonation".to_owned(),
                "projectiles_explosions/project_specific".to_owned(),
                "public_order_factors/Gen_TempField_x002A_0".to_owned(),
                "public_order_factors/Gen_TempField_x002A_1".to_owned(),
                "quotes/Gen_TempField_x002A_0".to_owned(),
                "quotes/culture".to_owned(),
                "quotes/quote_person".to_owned(),
                "quotes_people/Gen_TempField_x002A_0".to_owned(),
                "random_localisation_strings/Gen_string".to_owned(),
                "random_localisation_strings/project_specific".to_owned(),
                "regions/palette_entry".to_owned(),
                "regions/project_specific".to_owned(),
                "sea_climate_details/sea_deep_colour".to_owned(),
                "sea_climate_details/sea_shallow_colour".to_owned(),
                "sea_climate_details/sky_colour".to_owned(),
                "sea_climate_details/sun_colour".to_owned(),
                "slots_art/minibuildings_differ_at_quality".to_owned(),
                "slots_art/project_specific".to_owned(),
                "slots_art/underlay_differs_with_building".to_owned(),
                "slots_art/underlay_rotation".to_owned(),
                "slots_art/underlay_scale".to_owned(),
                "technologies/Gen_TempField_x002A_0".to_owned(),
                "technologies/Gen_TempField_x002A_1".to_owned(),
                "technology_required_building_levels_junctions/project_specific".to_owned(),
                "technology_required_technology_junctions/project_specific".to_owned(),
                "town_wealth_growth_factors/Gen_TempField_x002A_0".to_owned(),
                "town_wealth_growth_factors/Gen_TempField_x002A_1".to_owned(),
                "trade_details/Gen_TempField_x002A_0".to_owned(),
                "trait_triggers/Gen_TempField_x002A_0".to_owned(),
                "trees/is_conifer".to_owned(),
                "trees/is_high_altitude".to_owned(),
                "trees/is_shrub".to_owned(),
                "trees/tree".to_owned(),
                "trigger_events/from_ui".to_owned(),
                "trigger_events/project_specific".to_owned(),
                "unit_regiment_names/unit_class".to_owned(),
                "unit_stats_land/desert_effect".to_owned(),
                "unit_stats_land/dismounted_formation_spacing_horizontal".to_owned(),
                "unit_stats_land/dismounted_formation_spacing_vertical".to_owned(),
                "unit_stats_land/fatigue_resistant".to_owned(),
                "unit_stats_land/melee_defence".to_owned(),
                "unit_stats_land/snow_effect".to_owned(),
                "unit_stats_land/tropics_effect".to_owned(),
                "unit_stats_land/unit_class".to_owned(),
                "unit_stats_land_experience_bonuses/melee_defence".to_owned(),
                "unit_stats_naval/collision_momentum_modifer".to_owned(),
                "unit_stats_naval/reactivate_cost".to_owned(),
                "unit_stats_naval/repair_cost_port".to_owned(),
                "unit_stats_naval/repair_cost_sea".to_owned(),
                "unit_stats_naval/ship_rating_icon".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_armour".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_critical".to_owned(),
                "unit_stats_naval/side_panels_above_water_2_hits".to_owned(),
                "unit_stats_naval/stat_bar_manoeuvrability_rating".to_owned(),
                "unit_stats_naval_crew_to_factions/gunner_type".to_owned(),
                "unit_stats_naval_crew_to_factions/key".to_owned(),
                "unit_stats_naval_crew_to_factions/marine_type".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_1".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_2".to_owned(),
                "unit_stats_naval_crew_to_factions/officer_3".to_owned(),
                "unit_stats_naval_crew_to_factions/seaman_type".to_owned(),
                "units/era".to_owned(),
                "units/fitness".to_owned(),
                "units_to_groupings_military_permissions/project_specific".to_owned(),
                "unrest_cause_to_demands/demand".to_owned(),
                "warscape_rigid/category".to_owned(),
                "warscape_rigid_lod_range/LOD_id".to_owned(),
                "warscape_trees/model".to_owned(),
                "warscape_underlay_textures/filepath".to_owned(),
                "warscape_underlay_textures/height".to_owned(),
                "warscape_underlay_textures/orientation-angle".to_owned(),
                "warscape_underlay_textures/width".to_owned(),
                "wind_levels/magnitudeX".to_owned(),
                "wind_levels/magnitudeY".to_owned(),
            ],
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
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
            locale_file_name: Some("language.txt".to_owned()),
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
            install_type_cache: Arc::new(RwLock::new(HashMap::new())),
            compression_formats_supported: vec![]
        });

        let order_list = vec![
            KEY_PHARAOH_DYNASTIES,
            KEY_PHARAOH,
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
